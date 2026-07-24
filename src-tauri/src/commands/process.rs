use crate::{
    process_supervisor::{GameProcess, Signal},
    AppState,
};

#[tauri::command]
pub async fn process_list(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<GameProcess>, String> {
    let sup = state.process_supervisor.lock().unwrap();
    Ok(sup.list())
}

#[tauri::command]
pub async fn process_signal(
    pid: u32,
    signal: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let sig = match signal.as_str() {
        "term" => {
            #[cfg(target_os = "macos")]
            unsafe { libc::kill(pid as i32, libc::SIGTERM); }
            crate::process_supervisor::Signal::Term
        }
        "kill" => {
            #[cfg(target_os = "macos")]
            unsafe { libc::kill(pid as i32, libc::SIGKILL); }
            crate::process_supervisor::Signal::Kill
        }
        _ => return Err(format!("Unknown signal: {signal}")),
    };

    let mut sup = state.process_supervisor.lock().unwrap();
    sup.update_status(pid, crate::process_supervisor::ProcessStatus::Killed, Some(9));
    Ok(())
}

#[tauri::command]
pub async fn process_stdout_for_bottle(
    bottle_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    // In production: tail the log file for this bottle.
    let log_path = state
        .config
        .logs_dir
        .join(&bottle_id)
        .join("stdout.log");

    if log_path.exists() {
        let contents = std::fs::read_to_string(&log_path)
            .map_err(|e| format!("Failed to read log: {e}"))?;
        Ok(contents.lines().map(String::from).collect())
    } else {
        Ok(Vec::new())
    }
}
