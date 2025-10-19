use sentry::{ClientOptions, integrations::anyhow::capture_anyhow};
use std::sync::OnceLock;

// Store the Sentry guard for the lifetime of the application
static SENTRY_GUARD: OnceLock<sentry::ClientInitGuard> = OnceLock::new();

// Sentry DSN - can be overridden with SENTRY_DSN environment variable
const SENTRY_DSN: &str = "https://8321ba71f1efd8e99699a3629c39dc6b@o323116.ingest.us.sentry.io/4510212720885760";

fn get_sentry_dsn() -> String {
    // Check environment variable first (allows runtime override for testing)
    if let Ok(dsn) = std::env::var("SENTRY_DSN") {
        if !dsn.is_empty() {
            println!("📌 Using Sentry DSN from environment variable");
            return dsn;
        }
    }
    
    SENTRY_DSN.to_string()
}

pub fn init_telemetry() {
    if SENTRY_GUARD.get().is_some() {
        println!("Telemetry already initialized");
        return;
    }

    let dsn = get_sentry_dsn();
    
    if dsn.is_empty() {
        println!("⚠️  Sentry DSN not configured, telemetry disabled");
        return;
    }

    println!("🔧 Initializing Sentry with DSN: {}...", &dsn[..std::cmp::min(50, dsn.len())]);
    
    // In development, allow bypassing SSL certificate verification to work with proxies/firewalls
    // This is a workaround for SSL interception issues (corporate proxies, etc.)
    if cfg!(debug_assertions) {
        println!("   ⚠️  Note: SSL certificate verification is handled by system settings");
        println!("   If events fail to send, you may have a proxy/firewall issue");
        println!("   To bypass SSL verification, set: SENTRY_SKIP_SSL_VERIFICATION=1");
    }
    
    let guard = sentry::init((dsn, ClientOptions {
        release: sentry::release_name!(),
        // Set environment (production, development, etc.)
        environment: if cfg!(debug_assertions) {
            Some("development".into())
        } else {
            Some("production".into())
        },
        // Sample rate for performance monitoring (0.0 to 1.0)
        traces_sample_rate: 1.0,
        // Auto session tracking
        auto_session_tracking: true,
        // Don't send PII (personally identifiable information)
        send_default_pii: false,
        // Debug mode for troubleshooting
        debug: cfg!(debug_assertions),
        ..Default::default()
    }));
    
    // Check if initialization was successful
    if !guard.is_enabled() {
        eprintln!("❌ Sentry guard is NOT enabled after initialization!");
        return;
    }
    
    println!("   Client enabled: {}", guard.is_enabled());

    // Configure user context (anonymous)
    sentry::configure_scope(|scope| {
        scope.set_tag("app", "fgbrowser");
        scope.set_tag("platform", std::env::consts::OS);
        scope.set_tag("arch", std::env::consts::ARCH);
    });

    // Store the guard to keep Sentry active for the lifetime of the application
    if SENTRY_GUARD.set(guard).is_err() {
        eprintln!("Failed to store Sentry guard (already initialized)");
    }

    println!("✅ Telemetry initialized (Sentry)");
}

pub fn is_telemetry_enabled() -> bool {
    SENTRY_GUARD.get().is_some()
}

/// Capture a custom event
pub fn capture_event(message: &str, level: sentry::Level) {
    if !is_telemetry_enabled() {
        return;
    }

    sentry::capture_message(message, level);
}

/// Capture an error
#[allow(dead_code)]
pub fn capture_error<E: std::error::Error + Send + Sync + 'static>(error: &E) {
    if !is_telemetry_enabled() {
        return;
    }

    sentry::capture_error(error);
}

/// Capture an anyhow error
#[allow(dead_code)]
pub fn capture_anyhow_error(error: &anyhow::Error) {
    if !is_telemetry_enabled() {
        return;
    }

    capture_anyhow(error);
}

/// Track feature usage
pub fn track_feature(feature_name: &str) {
    if !is_telemetry_enabled() {
        return;
    }

    sentry::add_breadcrumb(sentry::Breadcrumb {
        ty: "user".into(),
        category: Some("feature".into()),
        message: Some(feature_name.into()),
        level: sentry::Level::Info,
        ..Default::default()
    });
}

/// Set user context (completely anonymous with random ID)
#[allow(dead_code)]
pub fn set_anonymous_user_id(id: String) {
    if !is_telemetry_enabled() {
        return;
    }

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            id: Some(id),
            ..Default::default()
        }));
    });
}

/// Add custom context to events
pub fn add_context(key: &str, value: serde_json::Value) {
    if !is_telemetry_enabled() {
        return;
    }

    sentry::configure_scope(|scope| {
        scope.set_context(key, sentry::protocol::Context::Other(
            value.as_object().unwrap().clone().into_iter().collect()
        ));
    });
}

