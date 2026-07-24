use crate::{compat::CompatEntry, AppState};

#[tauri::command]
pub async fn compat_lookup(
    app_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Option<CompatEntry>, String> {
    let engine = state.compat_engine.lock().unwrap();
    Ok(engine.lookup(&app_id).cloned())
}

#[tauri::command]
pub async fn compat_search(
    query: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<CompatEntry>, String> {
    let engine = state.compat_engine.lock().unwrap();
    Ok(engine.search(&query).into_iter().cloned().collect())
}

#[tauri::command]
pub async fn compat_submit_report(
    report: crate::compat::CompatReport,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // In production: POST to community API or write to a submissions queue.
    tracing::info!(
        "Received compatibility report for {}: {:?}",
        report.app_name,
        report.rating
    );
    let _ = state;
    Ok(())
}
