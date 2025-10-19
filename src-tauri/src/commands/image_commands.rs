#[tauri::command]
pub async fn get_cached_image(app_handle: tauri::AppHandle, url: String) -> Result<String, String> {
    if url.is_empty() {
        return Err("Image URL is empty".to_string());
    }
    
    crate::image_cache::ImageCache::get_cached_image_url(&app_handle, &url).await
}

#[tauri::command]
pub async fn check_image_cached(app_handle: tauri::AppHandle, url: String) -> Result<Option<String>, String> {
    if url.is_empty() {
        return Ok(None);
    }
    
    crate::image_cache::ImageCache::check_cached(&app_handle, &url)
}

#[tauri::command]
pub async fn cache_image_background(app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    if url.is_empty() {
        return Ok(());
    }
    
    // Cache the image in the background (fire and forget)
    tauri::async_runtime::spawn(async move {
        let _ = crate::image_cache::ImageCache::get_cached_image_url(&app_handle, &url).await;
    });
    
    Ok(())
}

#[tauri::command]
pub fn clear_image_cache(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::image_cache::ImageCache::clear_cache(&app_handle)
}

#[tauri::command]
pub fn get_image_cache_size(app_handle: tauri::AppHandle) -> Result<u64, String> {
    crate::image_cache::ImageCache::get_cache_size(&app_handle)
}

