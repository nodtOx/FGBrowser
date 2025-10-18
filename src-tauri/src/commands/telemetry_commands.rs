use crate::telemetry;

#[tauri::command]
pub fn track_app_launch() {
    telemetry::capture_event("App launched", sentry::Level::Info);
}

#[tauri::command]
pub fn track_feature_usage(feature: String) {
    telemetry::track_feature(&feature);
}

#[tauri::command]
pub fn track_crawler_run(pages_crawled: i32, games_found: i32) {
    if !telemetry::is_telemetry_enabled() {
        return;
    }

    telemetry::add_context("crawler", serde_json::json!({
        "pages_crawled": pages_crawled,
        "games_found": games_found,
    }));
    
    telemetry::capture_event("Crawler completed", sentry::Level::Info);
}

#[tauri::command]
pub fn track_error(error_message: String, context: Option<String>) {
    if !telemetry::is_telemetry_enabled() {
        return;
    }

    if let Some(ctx) = context {
        sentry::configure_scope(|scope| {
            scope.set_extra("context", ctx.into());
        });
    }

    telemetry::capture_event(&error_message, sentry::Level::Error);
}

#[tauri::command]
pub fn is_telemetry_enabled() -> bool {
    telemetry::is_telemetry_enabled()
}

