// CLI tool for crawling repack sites without running the GUI
// This reuses the same crawler logic as the Tauri app
// Usage: cargo run --bin cli -- [COMMAND] [OPTIONS]

use anyhow::Result;
use clap::{Parser, Subcommand};
use desktop_lib::commands::save_repacks_to_db;
use desktop_lib::crawler::FitGirlCrawler;
use desktop_lib::database::Database;
use std::path::PathBuf;

fn find_database_path() -> PathBuf {
    // Try multiple locations to find the database, same as Tauri app
    let locations = vec![
        // In desktop folder (when running from src-tauri/)
        PathBuf::from("../repacks.db"),
        // In project root (when running from project root)
        PathBuf::from("repacks.db"),
        // Relative to this binary
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("repacks.db")))
            .unwrap_or_else(|| PathBuf::from("repacks.db")),
        // In workspace root
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("repacks.db"),
    ];

    for path in &locations {
        if path.exists() {
            return path.clone();
        }
    }

    // Default: use the desktop folder location
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("repacks.db")
}

#[derive(Parser)]
#[command(name = "FitBoy CLI")]
#[command(about = "Crawl repack sites and update the database from command line", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the database file (auto-detected if not specified)
    #[arg(short, long)]
    database: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Crawl pages and save to database
    Crawl {
        /// Number of pages to crawl
        #[arg(short, long, default_value = "5")]
        pages: u32,

        /// Maximum number of pages (0 = no limit)
        #[arg(short, long, default_value = "0")]
        max_pages: u32,
    },

    /// Update popular repacks
    Popular {
        /// Period: "month" or "year" or "both"
        #[arg(short, long, default_value = "both")]
        period: String,
    },

    /// Show database statistics
    Stats,

    /// Check database health
    Check,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Auto-detect or use provided database path
    let is_auto_detected = cli.database.is_none();
    let db_path = cli.database.unwrap_or_else(find_database_path);
    
    // Initialize database
    let db = Database::new(db_path.clone())?;
    
    if cli.verbose || is_auto_detected {
        println!("📦 Database: {:?}", db_path);
        if is_auto_detected {
            println!("   (auto-detected)\n");
        } else {
            println!();
        }
    }

    match cli.command {
        Commands::Crawl { pages, max_pages } => {
            crawl_pages(&db_path, pages, max_pages, cli.verbose).await?;
        }
        Commands::Popular { period } => {
            update_popular(&db, &db_path, &period, cli.verbose).await?;
        }
        Commands::Stats => {
            show_stats(&db)?;
        }
        Commands::Check => {
            check_database(&db)?;
        }
    }

    Ok(())
}

async fn crawl_pages(db_path: &PathBuf, pages: u32, max_pages: u32, verbose: bool) -> Result<()> {
    let crawler = FitGirlCrawler::new()?;
    
    println!("🔍 Starting crawler...");
    println!("📄 Pages to crawl: {}", if max_pages > 0 { max_pages.to_string() } else { pages.to_string() });
    println!("🌐 Site: https://fitgirl-repacks.site\n");

    let pages_to_crawl = if max_pages > 0 { max_pages } else { pages };

    for page in 1..=pages_to_crawl {
        println!("📃 Crawling page {}...", page);

        match crawler.crawl_page(page).await {
            Ok(repacks) => {
                println!("  Found {} repacks", repacks.len());

                if verbose {
                    for repack in &repacks {
                        println!("  • {}", repack.title);
                    }
                }

                // Save to database using the same function as Tauri app
                match save_repacks_to_db(&repacks, db_path) {
                    Ok(_) => println!("  ✅ Saved to database\n"),
                    Err(e) => println!("  ❌ Error saving: {}\n", e),
                }
            }
            Err(e) => {
                println!("  ❌ Error: {}\n", e);
            }
        }

        // Delay between pages
        if page < pages_to_crawl {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    println!("🎉 Crawling complete!");

    Ok(())
}

async fn update_popular(db: &Database, db_path: &PathBuf, period: &str, verbose: bool) -> Result<()> {
    let crawler = FitGirlCrawler::new()?;
    
    let periods: Vec<&str> = if period == "both" {
        vec!["month", "year"]
    } else {
        vec![period]
    };

    for p in periods {
        println!("🌟 Updating popular repacks ({})...", p);

        match crawler.fetch_popular_repacks(p).await {
            Ok(entries) => {
                println!("  Found {} popular games", entries.len());

                // Save popular repacks
                for (rank, entry) in entries.iter().enumerate() {
                    let rank = (rank + 1) as i32;
                    
                    if verbose {
                        println!("  {}. {}", rank, entry.title);
                    }

                    if let Err(e) = db.save_popular_repack(
                        &entry.url,
                        &entry.title,
                        entry.image_url.as_deref(),
                        rank,
                        p
                    ) {
                        if verbose && !e.to_string().contains("UNIQUE constraint") {
                            println!("    [ERROR] {}", e);
                        }
                    }
                }

                // Update links to existing games
                if let Err(e) = db.update_popular_repack_links(Some(p)) {
                    println!("  ⚠️  Error linking: {}", e);
                } else {
                    println!("  ✅ Linked to existing games");
                }

                // Fetch full details for popular games
                println!("  📥 Fetching game details...");
                let mut fetched = 0;
                let mut errors = 0;

                for entry in entries.iter() {
                    if verbose {
                        println!("    Fetching: {}", entry.url);
                    }

                    match crawler.crawl_single_game(&entry.url).await {
                        Ok(Some(repack)) => {
                            if save_repacks_to_db(&[repack], db_path).is_ok() {
                                fetched += 1;
                            }
                        }
                        Ok(None) => {
                            errors += 1;
                            if verbose {
                                println!("      [WARN] Could not extract data");
                            }
                        }
                        Err(e) => {
                            errors += 1;
                            if verbose {
                                println!("      [ERROR] {}", e);
                            }
                        }
                    }

                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }

                println!("  ✅ Fetched: {}, Errors: {}\n", fetched, errors);
            }
            Err(e) => {
                println!("  ❌ Error: {}\n", e);
            }
        }
    }

    println!("🎉 Popular repacks updated!");
    Ok(())
}

fn show_stats(db: &Database) -> Result<()> {
    println!("📊 Database Statistics\n");

    let stats = db.get_stats()?;
    
    println!("Total Games: {}", stats.total_games);
    println!("Total Magnet Links: {}", stats.total_magnets);

    // Get categories
    let categories = db.get_categories_with_counts()?;
    println!("Total Categories: {}", categories.len());

    println!("\n📈 Top Categories:");
    for (i, category) in categories.iter().enumerate().take(10) {
        println!("  {}. {} ({})", i + 1, category.name, category.game_count);
    }

    Ok(())
}

fn check_database(db: &Database) -> Result<()> {
    println!("🔍 Checking database health...\n");

    let stats = db.get_stats()?;
    let categories = db.get_categories_with_counts()?;
    
    println!("✅ Database is accessible");
    println!("   Total Games: {}", stats.total_games);
    println!("   Total Categories: {}", categories.len());
    println!("   Total Magnet Links: {}", stats.total_magnets);

    // Check for games without clean names
    println!("\n🔍 Checking data quality...");
    let games = db.get_all_games(999999, 0)?;
    let games_without_clean: Vec<_> = games.iter()
        .filter(|g| g.clean_name.is_none())
        .collect();
    
    if games_without_clean.is_empty() {
        println!("  ✅ All games have clean names");
    } else {
        println!("  ⚠️  {} games without clean names", games_without_clean.len());
    }

    println!("\n✅ Database check complete");

    Ok(())
}

