import sqlite3
import json
from typing import List, Dict, Optional
from datetime import datetime


class RepackDatabase:
    """SQLite database handler for FitGirl repacks."""

    def __init__(self, db_path: str = "repacks.db"):
        self.db_path = db_path
        self.conn = None
        self.cursor = None

    def connect(self):
        """Connect to the database."""
        self.conn = sqlite3.connect(self.db_path)
        self.conn.row_factory = sqlite3.Row
        self.cursor = self.conn.cursor()

    def close(self):
        """Close the database connection."""
        if self.conn:
            self.conn.close()

    def __enter__(self):
        """Context manager entry."""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()

    def create_tables(self):
        """Create database tables if they don't exist."""
        self.cursor.execute(
            """
            CREATE TABLE IF NOT EXISTS repacks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                genres_tags TEXT,
                company TEXT,
                languages TEXT,
                original_size TEXT,
                repack_size TEXT,
                url TEXT UNIQUE,
                date TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        """
        )

        self.cursor.execute(
            """
            CREATE TABLE IF NOT EXISTS magnet_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repack_id INTEGER NOT NULL,
                source TEXT NOT NULL,
                magnet TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (repack_id) REFERENCES repacks (id) ON DELETE CASCADE,
                UNIQUE(repack_id, source)
            )
        """
        )

        # Create indexes for faster queries
        self.cursor.execute(
            """
            CREATE INDEX IF NOT EXISTS idx_repacks_title 
            ON repacks(title)
        """
        )

        self.cursor.execute(
            """
            CREATE INDEX IF NOT EXISTS idx_repacks_date 
            ON repacks(date DESC)
        """
        )

        self.cursor.execute(
            """
            CREATE INDEX IF NOT EXISTS idx_repacks_url 
            ON repacks(url)
        """
        )

        self.cursor.execute(
            """
            CREATE INDEX IF NOT EXISTS idx_magnet_links_repack_id 
            ON magnet_links(repack_id)
        """
        )

        self.conn.commit()
        print("Database tables created successfully")

    def insert_repack(self, repack: Dict) -> Optional[int]:
        """Insert or update a repack in the database."""
        try:
            url = repack.get("url")

            # Skip entries without URL (like "Upcoming Repacks")
            if not url:
                print(f"Skipping '{repack.get('title')}' - no URL")
                return None

            # Insert or replace repack
            self.cursor.execute(
                """
                INSERT INTO repacks (
                    title, genres_tags, company, languages, 
                    original_size, repack_size, url, date, updated_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                ON CONFLICT(url) 
                DO UPDATE SET
                    title = excluded.title,
                    genres_tags = excluded.genres_tags,
                    company = excluded.company,
                    languages = excluded.languages,
                    original_size = excluded.original_size,
                    repack_size = excluded.repack_size,
                    date = excluded.date,
                    updated_at = CURRENT_TIMESTAMP
            """,
                (
                    repack.get("title"),
                    repack.get("genres_tags"),
                    repack.get("company"),
                    repack.get("languages"),
                    repack.get("original_size"),
                    repack.get("repack_size"),
                    url,
                    repack.get("date"),
                ),
            )

            # Get the repack ID
            repack_id = self.cursor.lastrowid
            if repack_id == 0:
                # If update occurred, get the existing ID
                self.cursor.execute(
                    """
                    SELECT id FROM repacks 
                    WHERE url = ?
                """,
                    (url,),
                )
                result = self.cursor.fetchone()
                if result:
                    repack_id = result["id"]

            # Insert magnet links
            magnet_links = repack.get("magnet_links", [])
            for magnet in magnet_links:
                self.cursor.execute(
                    """
                    INSERT INTO magnet_links (repack_id, source, magnet)
                    VALUES (?, ?, ?)
                    ON CONFLICT(repack_id, source)
                    DO UPDATE SET magnet = excluded.magnet
                """,
                    (repack_id, magnet.get("source"), magnet.get("magnet")),
                )

            self.conn.commit()
            return repack_id

        except sqlite3.Error as e:
            print(f"Error inserting repack '{repack.get('title')}': {e}")
            self.conn.rollback()
            return None

    def insert_repacks_batch(self, repacks: List[Dict]) -> int:
        """Insert multiple repacks in a batch."""
        count = 0
        skipped = 0
        for repack in repacks:
            result = self.insert_repack(repack)
            if result:
                count += 1
            else:
                skipped += 1

        if skipped > 0:
            print(f"Skipped {skipped} entries without URLs")
        print(f"Inserted/updated {count} repacks")
        return count

    def get_repack_by_id(self, repack_id: int) -> Optional[Dict]:
        """Get a repack by ID."""
        self.cursor.execute(
            """
            SELECT * FROM repacks WHERE id = ?
        """,
            (repack_id,),
        )
        row = self.cursor.fetchone()
        if not row:
            return None

        repack = dict(row)

        # Get magnet links
        self.cursor.execute(
            """
            SELECT source, magnet FROM magnet_links 
            WHERE repack_id = ?
        """,
            (repack_id,),
        )
        repack["magnet_links"] = [dict(row) for row in self.cursor.fetchall()]

        return repack

    def get_repack_by_url(self, url: str) -> Optional[Dict]:
        """Get a repack by URL."""
        self.cursor.execute(
            """
            SELECT * FROM repacks WHERE url = ?
        """,
            (url,),
        )
        row = self.cursor.fetchone()
        if not row:
            return None

        repack = dict(row)
        repack_id = repack["id"]

        # Get magnet links
        self.cursor.execute(
            """
            SELECT source, magnet FROM magnet_links 
            WHERE repack_id = ?
        """,
            (repack_id,),
        )
        repack["magnet_links"] = [dict(row) for row in self.cursor.fetchall()]

        return repack

    def get_repack_by_title(self, title: str) -> Optional[Dict]:
        """Get a repack by title."""
        self.cursor.execute(
            """
            SELECT * FROM repacks WHERE title = ? 
            ORDER BY date DESC LIMIT 1
        """,
            (title,),
        )
        row = self.cursor.fetchone()
        if not row:
            return None

        repack = dict(row)
        repack_id = repack["id"]

        # Get magnet links
        self.cursor.execute(
            """
            SELECT source, magnet FROM magnet_links 
            WHERE repack_id = ?
        """,
            (repack_id,),
        )
        repack["magnet_links"] = [dict(row) for row in self.cursor.fetchall()]

        return repack

    def get_all_repacks(self, limit: int = 100, offset: int = 0) -> List[Dict]:
        """Get all repacks with pagination."""
        self.cursor.execute(
            """
            SELECT * FROM repacks 
            ORDER BY date DESC 
            LIMIT ? OFFSET ?
        """,
            (limit, offset),
        )

        repacks = []
        for row in self.cursor.fetchall():
            repack = dict(row)
            repack_id = repack["id"]

            # Get magnet links
            self.cursor.execute(
                """
                SELECT source, magnet FROM magnet_links 
                WHERE repack_id = ?
            """,
                (repack_id,),
            )
            repack["magnet_links"] = [dict(row) for row in self.cursor.fetchall()]
            repacks.append(repack)

        return repacks

    def search_repacks(self, query: str) -> List[Dict]:
        """Search repacks by title, genres, or company."""
        search_pattern = f"%{query}%"
        self.cursor.execute(
            """
            SELECT * FROM repacks 
            WHERE title LIKE ? 
               OR genres_tags LIKE ? 
               OR company LIKE ?
            ORDER BY date DESC
        """,
            (search_pattern, search_pattern, search_pattern),
        )

        repacks = []
        for row in self.cursor.fetchall():
            repack = dict(row)
            repack_id = repack["id"]

            # Get magnet links
            self.cursor.execute(
                """
                SELECT source, magnet FROM magnet_links 
                WHERE repack_id = ?
            """,
                (repack_id,),
            )
            repack["magnet_links"] = [dict(row) for row in self.cursor.fetchall()]
            repacks.append(repack)

        return repacks

    def get_stats(self) -> Dict:
        """Get database statistics."""
        self.cursor.execute("SELECT COUNT(*) as count FROM repacks")
        total_repacks = self.cursor.fetchone()["count"]

        self.cursor.execute("SELECT COUNT(*) as count FROM magnet_links")
        total_magnets = self.cursor.fetchone()["count"]

        self.cursor.execute(
            """
            SELECT COUNT(DISTINCT company) as count 
            FROM repacks 
            WHERE company IS NOT NULL
        """
        )
        total_companies = self.cursor.fetchone()["count"]

        return {
            "total_repacks": total_repacks,
            "total_magnet_links": total_magnets,
            "total_companies": total_companies,
        }

    def export_to_json(self, filename: str = "repacks_export.json"):
        """Export all repacks to JSON file."""
        repacks = self.get_all_repacks(limit=999999)
        with open(filename, "w", encoding="utf-8") as f:
            json.dump(repacks, f, indent=2, ensure_ascii=False)
        print(f"Exported {len(repacks)} repacks to {filename}")


def import_from_json(json_file: str, db_path: str = "repacks.db"):
    """Import repacks from JSON file to database."""
    with open(json_file, "r", encoding="utf-8") as f:
        repacks = json.load(f)

    with RepackDatabase(db_path) as db:
        db.create_tables()
        count = db.insert_repacks_batch(repacks)
        stats = db.get_stats()
        print(f"\nDatabase Statistics:")
        print(f"  Total repacks: {stats['total_repacks']}")
        print(f"  Total magnet links: {stats['total_magnet_links']}")
        print(f"  Total companies: {stats['total_companies']}")


def main():
    """Example usage."""
    # Query examples
    with RepackDatabase() as db:
        stats = db.get_stats()
        print("\n" + "=" * 60)
        print("DATABASE STATISTICS")
        print("=" * 60)
        print(f"Total repacks: {stats['total_repacks']}")
        print(f"Total magnet links: {stats['total_magnet_links']}")
        print(f"Total companies: {stats['total_companies']}")

        if stats["total_repacks"] > 0:
            print("\n" + "=" * 60)
            print("Searching for 'Little Nightmares'...")
            results = db.search_repacks("Little Nightmares")
            for repack in results:
                print(f"\nTitle: {repack['title']}")
                print(f"Size: {repack['repack_size']}")
                print(f"Magnet links: {len(repack['magnet_links'])}")
        else:
            print("\nDatabase is empty. Run crawler.py to populate it.")


if __name__ == "__main__":
    main()
