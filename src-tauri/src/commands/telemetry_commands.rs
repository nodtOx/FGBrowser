use crate::telemetry;

#[tauri::command]
pub fn track_app_launch() {
    telemetry::capture_event("App launched", sentry::Level::Info);
    println!("📊 Tracked: App launch");
}

#[tauri::command]
pub fn track_feature_usage(feature: String) {
    telemetry::track_feature(&feature);
    println!("📊 Tracked feature: {}", feature);
}

#[tauri::command]
pub fn track_crawler_run(pages_crawled: i32, games_found: i32) {
    telemetry::add_context("crawler", serde_json::json!({
        "pages_crawled": pages_crawled,
        "games_found": games_found,
    }));
    
    telemetry::capture_event("Crawler completed", sentry::Level::Info);
    println!("📊 Tracked: Crawler run ({} pages, {} games)", pages_crawled, games_found);
}

#[tauri::command]
pub fn track_error(error_message: String, context: Option<String>) {
    if let Some(ctx) = context {
        sentry::configure_scope(|scope| {
            scope.set_extra("context", ctx.into());
        });
    }

    telemetry::capture_event(&error_message, sentry::Level::Error);
    println!("📊 Tracked error: {}", error_message);
}

#[tauri::command]
pub fn is_telemetry_enabled() -> bool {
    telemetry::is_telemetry_enabled()
}

#[tauri::command]
pub fn test_sentry_integration() -> String {
    let is_init = telemetry::is_telemetry_enabled();
    println!("🧪 Testing Sentry integration...");
    println!("   Telemetry initialized: {}", is_init);
    
    if !is_init {
        let msg = "FAILED: Telemetry not initialized. Check console for errors.";
        println!("   ❌ {}", msg);
        return msg.to_string();
    }

    println!("   Sending test events...");
    println!("   NOTE: Sentry events are sent asynchronously in the background");
    println!("   Watch the console for any error messages from Sentry");
    
    // Send a simple test message first
    let event_id = sentry::capture_message("Test Error Message from FGBrowser", sentry::Level::Error);
    println!("   ✓ Event ID: {}", event_id);
    
    // Send multiple test events
    telemetry::capture_event("Test Info Message", sentry::Level::Info);
    println!("   ✓ Sent Info event");
    
    telemetry::capture_event("Test Warning Message", sentry::Level::Warning);
    println!("   ✓ Sent Warning event");
    
    telemetry::capture_event("Test Error Message", sentry::Level::Error);
    println!("   ✓ Sent Error event");
    
    // Add context and send an error
    sentry::configure_scope(|scope| {
        scope.set_extra("test_timestamp", sentry::protocol::Value::String(chrono::Utc::now().to_rfc3339()));
        scope.set_extra("test_user", "manual_test_user".into());
        scope.set_tag("test", "manual");
        scope.set_tag("source", "test_button");
    });
    
    sentry::capture_message("Manual Sentry Test from Rust", sentry::Level::Error);
    println!("   ✓ Sent test message with context");
    
    // Capture an exception (but don't actually panic - use Result::Err instead)
    let test_result: Result<(), String> = Err("Test exception for Sentry".to_string());
    if let Err(e) = test_result {
        sentry::capture_message(&format!("Test Exception: {}", e), sentry::Level::Error);
        println!("   ✓ Sent test exception");
    }
    
    println!("   ");
    println!("   If you don't see errors above, events were queued successfully.");
    println!("   Events may take 5-10 seconds to appear in Sentry dashboard.");
    println!("   ");
    println!("   Check: https://o323116.ingest.us.sentry.io/issues/");
    
    let msg = format!("SUCCESS: 5 test events sent to Sentry.\nFirst event ID: {}\n\nCheck console for details and wait 5-10 seconds for events to appear in dashboard.", event_id);
    println!("   ✅ Done");
    msg
}

