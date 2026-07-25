use crate::{
    bottle::resolve_existing_child,
    process_supervisor::{GameProcess, ProcessStatus, Signal},
    AppState,
};

#[tauri::command]
pub async fn process_list(state: tauri::State<'_, AppState>) -> Result<Vec<GameProcess>, String> {
    let sup = state.process_supervisor.lock().unwrap();
    Ok(sup.list())
}

#[tauri::command]
pub async fn process_signal(
    pid: u32,
    signal: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let signal = parse_signal(&signal)?;
    {
        let supervisor = state.process_supervisor.lock().unwrap();
        let process = supervisor
            .get(pid)
            .ok_or_else(|| format!("Process {pid} is not tracked"))?;
        if process.status != ProcessStatus::Running {
            return Err(format!("Process {pid} is not running"));
        }
    }

    send_os_signal(pid, signal)?;

    let mut supervisor = state.process_supervisor.lock().unwrap();
    if supervisor
        .get(pid)
        .is_some_and(|process| process.status == ProcessStatus::Running)
    {
        supervisor.update_status(pid, ProcessStatus::Killed, None);
    }
    Ok(())
}

fn parse_signal(value: &str) -> Result<Signal, String> {
    match value {
        "term" => Ok(Signal::Term),
        "kill" => Ok(Signal::Kill),
        _ => Err(format!("Unknown signal: {value}")),
    }
}

#[cfg(target_os = "macos")]
fn send_os_signal(pid: u32, signal: Signal) -> Result<(), String> {
    let signal = match signal {
        Signal::Term => nix::sys::signal::Signal::SIGTERM,
        Signal::Kill => nix::sys::signal::Signal::SIGKILL,
    };
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid as i32), signal)
        .map_err(|e| format!("Failed to signal process {pid}: {e}"))
}

#[cfg(not(target_os = "macos"))]
fn send_os_signal(_pid: u32, _signal: Signal) -> Result<(), String> {
    Err("Process signaling is only supported on macOS".into())
}

#[tauri::command]
pub async fn process_stdout_for_bottle(
    bottle_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    state.bottle_manager.resolve_bottle_path(&bottle_id)?;
    let bottle_log_dir = state.config.logs_dir.join(&bottle_id);
    if !bottle_log_dir.exists() {
        return Ok(Vec::new());
    }
    let log_path = resolve_existing_child(&state.config.logs_dir, &bottle_id, "Bottle log")?
        .join("stdout.log");

    if log_path.exists() {
        let contents =
            std::fs::read_to_string(&log_path).map_err(|e| format!("Failed to read log: {e}"))?;
        Ok(contents.lines().map(String::from).collect())
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_signal() {
        assert!(parse_signal("stop").is_err());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn sends_signal_to_a_live_process() {
        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .unwrap();
        send_os_signal(child.id(), Signal::Kill).unwrap();
        let status = child.wait().unwrap();
        assert!(!status.success());
    }
}
