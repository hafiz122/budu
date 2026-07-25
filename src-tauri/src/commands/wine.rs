use crate::{wine_manager::WineVersion, AppState};

#[tauri::command]
pub async fn wine_list_versions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<WineVersion>, String> {
    state.wine_manager.list_versions()
}

#[tauri::command]
pub async fn wine_install_version(
    version: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.wine_manager.install_version(&version).await
}

#[tauri::command]
pub async fn wine_get_default_version(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.config.default_wine_version.clone())
}
