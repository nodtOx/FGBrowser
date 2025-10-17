use std::path::PathBuf;
use std::fs;
use sha2::{Sha256, Digest};
use reqwest;
use tauri::{AppHandle, Manager};

pub struct ImageCache;

impl ImageCache {
    /// Get the cache directory path
    pub fn get_cache_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
        let cache_dir = app_handle
            .path()
            .app_cache_dir()
            .map_err(|e| format!("Failed to get cache dir: {}", e))?
            .join("images");
        
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .map_err(|e| format!("Failed to create cache dir: {}", e))?;
        }
        
        Ok(cache_dir)
    }
    
    /// Generate a cache key (filename) from a URL
    fn get_cache_key(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        
        // Try to preserve the file extension
        let extension = url
            .split('?').next() // Remove query params
            .and_then(|s| s.rsplit('.').next())
            .and_then(|ext| {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "jpg" || ext_lower == "jpeg" || ext_lower == "png" || ext_lower == "webp" || ext_lower == "gif" {
                    Some(ext_lower)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "jpg".to_string());
        
        format!("{}.{}", hash, extension)
    }
    
    /// Download an image from URL and save it to cache
    async fn download_image(url: &str, cache_path: &PathBuf) -> Result<(), String> {
        let response = reqwest::get(url)
            .await
            .map_err(|e| format!("Failed to download image: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }
        
        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read image bytes: {}", e))?;
        
        fs::write(cache_path, bytes)
            .map_err(|e| format!("Failed to write image to cache: {}", e))?;
        
        Ok(())
    }
    
    /// Check if an image is cached and return the path if it exists
    pub fn check_cached(app_handle: &AppHandle, url: &str) -> Result<Option<String>, String> {
        let cache_dir = Self::get_cache_dir(app_handle)?;
        let cache_key = Self::get_cache_key(url);
        let cache_path = cache_dir.join(&cache_key);
        
        if cache_path.exists() {
            let path_str = cache_path
                .to_str()
                .ok_or("Failed to convert path to string")?
                .to_string();
            Ok(Some(path_str))
        } else {
            Ok(None)
        }
    }
    
    /// Get a cached image URL or download it if not cached
    pub async fn get_cached_image_url(app_handle: &AppHandle, url: &str) -> Result<String, String> {
        let cache_dir = Self::get_cache_dir(app_handle)?;
        let cache_key = Self::get_cache_key(url);
        let cache_path = cache_dir.join(&cache_key);
        
        // Check if image is already cached
        if !cache_path.exists() {
            // Download and cache the image
            Self::download_image(url, &cache_path).await?;
        }
        
        // Convert the path to a string that can be used with convertFileSrc on the frontend
        let path_str = cache_path
            .to_str()
            .ok_or("Failed to convert path to string")?
            .to_string();
        
        Ok(path_str)
    }
    
    /// Clear the image cache
    pub fn clear_cache(app_handle: &AppHandle) -> Result<(), String> {
        let cache_dir = Self::get_cache_dir(app_handle)?;
        
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)
                .map_err(|e| format!("Failed to clear cache: {}", e))?;
            
            // Recreate the directory
            fs::create_dir_all(&cache_dir)
                .map_err(|e| format!("Failed to recreate cache dir: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Get cache size in bytes
    pub fn get_cache_size(app_handle: &AppHandle) -> Result<u64, String> {
        let cache_dir = Self::get_cache_dir(app_handle)?;
        
        if !cache_dir.exists() {
            return Ok(0);
        }
        
        let mut total_size = 0u64;
        
        if let Ok(entries) = fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    }
                }
            }
        }
        
        Ok(total_size)
    }
}

