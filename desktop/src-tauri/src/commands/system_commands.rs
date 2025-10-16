use crate::constants::AppConstants;
use arboard::Clipboard;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn get_app_constants() -> AppConstants {
    AppConstants::new()
}

#[tauri::command]
pub async fn open_magnet_link(magnet: String) -> Result<(), String> {
    // This will be handled by the system's default torrent client
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &magnet])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&magnet)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&magnet)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())
}

#[derive(Debug, serde::Serialize)]
pub struct DiskInfo {
    pub total: u64,
    pub free: u64,
    pub used: u64,
}

#[tauri::command]
pub async fn get_disk_info(db_path: State<'_, PathBuf>) -> Result<DiskInfo, String> {
    use sysinfo::Disks;
    use std::path::Path;
    
    let disks = Disks::new_with_refreshed_list();
    let db_path = db_path.inner().clone();
    
    // Get the absolute path of the database directory
    let db_dir = db_path.parent()
        .and_then(|p| p.canonicalize().ok())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    
    // Find the disk that contains the database directory
    let mut target_disk = None;
    for disk in disks.list() {
        let mount_point = disk.mount_point();
        // Check if the database path is on this disk
        if db_dir.starts_with(mount_point) {
            // Keep the most specific mount point (longest path)
            if let Some((_, current_mount)) = target_disk {
                if mount_point.as_os_str().len() > Path::new(current_mount).as_os_str().len() {
                    target_disk = Some((disk, mount_point));
                }
            } else {
                target_disk = Some((disk, mount_point));
            }
        }
    }
    
    // If we found a matching disk, use it; otherwise fall back to the first disk
    let disk = target_disk
        .map(|(d, _)| d)
        .or_else(|| disks.list().first())
        .ok_or("No disks found")?;
    
    let total = disk.total_space();
    let available = disk.available_space();
    let used = total.saturating_sub(available);
    
    Ok(DiskInfo {
        total,
        free: available,
        used,
    })
}

#[tauri::command]
pub async fn select_download_folder() -> Result<Option<String>, String> {
    use rfd::FileDialog;
    
    let folder = FileDialog::new()
        .set_title("Select Download Location")
        .pick_folder();
    
    Ok(folder.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn open_download_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

