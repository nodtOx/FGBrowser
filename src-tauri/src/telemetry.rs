use sentry::{ClientOptions, integrations::anyhow::capture_anyhow};
use std::sync::atomic::{AtomicBool, Ordering};

static TELEMETRY_INITIALIZED: AtomicBool = AtomicBool::new(false);

// Sentry DSN
const SENTRY_DSN: &str = "https://8321ba71f1efd8e99699a3629c39dc6b@o323116.ingest.us.sentry.io/4510212720885760";

pub fn init_telemetry(enabled: bool) {
    if TELEMETRY_INITIALIZED.load(Ordering::Relaxed) {
        println!("Telemetry already initialized");
        return;
    }

    if !enabled {
        println!("Telemetry disabled by user");
        return;
    }

    if SENTRY_DSN.is_empty() {
        println!("Sentry DSN not configured, telemetry disabled");
        return;
    }

    let _guard = sentry::init((SENTRY_DSN, ClientOptions {
        release: sentry::release_name!(),
        // Set environment (production, development, etc.)
        environment: if cfg!(debug_assertions) {
            Some("development".into())
        } else {
            Some("production".into())
        },
        // Sample rate for performance monitoring (0.0 to 1.0)
        traces_sample_rate: 0.1,
        // Auto session tracking
        auto_session_tracking: true,
        // Don't send PII (personally identifiable information)
        send_default_pii: false,
        ..Default::default()
    }));

    // Configure user context (anonymous)
    sentry::configure_scope(|scope| {
        scope.set_tag("app", "fgbrowser");
        scope.set_tag("platform", std::env::consts::OS);
        scope.set_tag("arch", std::env::consts::ARCH);
    });

    TELEMETRY_INITIALIZED.store(true, Ordering::Relaxed);
    println!("✅ Telemetry initialized (Sentry)");

    // Keep guard alive for the lifetime of the application
    std::mem::forget(_guard);
}

pub fn is_telemetry_enabled() -> bool {
    TELEMETRY_INITIALIZED.load(Ordering::Relaxed)
}

/// Capture a custom event
pub fn capture_event(message: &str, level: sentry::Level) {
    if !is_telemetry_enabled() {
        return;
    }

    sentry::capture_message(message, level);
}

/// Capture an error
pub fn capture_error<E: std::error::Error + Send + Sync + 'static>(error: &E) {
    if !is_telemetry_enabled() {
        return;
    }

    sentry::capture_error(error);
}

/// Capture an anyhow error
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

