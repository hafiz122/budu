use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a running game process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProcess {
    /// OS process ID.
    pub pid: u32,
    /// Bottle ID this process belongs to.
    pub bottle_id: String,
    /// Steam App ID, if launched via Steam.
    pub steam_app_id: Option<String>,
    /// The command that was executed.
    pub command: String,
    /// When the process was started.
    pub started_at: DateTime<Utc>,
    /// Process status.
    pub status: ProcessStatus,
    /// Exit code, if terminated.
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessStatus {
    Running,
    Exited,
    Crashed,
    Killed,
}

/// Signal to send to a running process.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Signal {
    /// Graceful shutdown (SIGTERM → wineserver -k).
    Term,
    /// Force kill (SIGKILL).
    Kill,
}

/// Restart policy for crashed games.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RestartPolicy {
    Never,
    OnCleanExit,
    Always,
}

pub struct ProcessSupervisor {
    /// Active processes tracked by this supervisor.
    active: std::collections::HashMap<u32, GameProcess>,
}

impl ProcessSupervisor {
    pub fn new() -> Self {
        Self {
            active: std::collections::HashMap::new(),
        }
    }

    /// Register a new process.
    pub fn register(&mut self, process: GameProcess) {
        self.active.insert(process.pid, process);
    }

    /// Update the status of a tracked process.
    pub fn update_status(
        &mut self,
        pid: u32,
        status: ProcessStatus,
        exit_code: Option<i32>,
    ) {
        if let Some(proc) = self.active.get_mut(&pid) {
            proc.status = status;
            proc.exit_code = exit_code;
        }
    }

    /// Remove a process from tracking.
    pub fn unregister(&mut self, pid: u32) {
        self.active.remove(&pid);
    }

    /// List all tracked processes.
    pub fn list(&self) -> Vec<GameProcess> {
        let mut procs: Vec<GameProcess> = self.active.values().cloned().collect();
        procs.sort_by_key(|p| p.started_at);
        procs
    }

    /// List processes for a specific bottle.
    pub fn list_for_bottle(&self, bottle_id: &str) -> Vec<GameProcess> {
        self.active
            .values()
            .filter(|p| p.bottle_id == bottle_id)
            .cloned()
            .collect()
    }

    /// Determine whether a Wine crash message indicates a real crash.
    pub fn is_crash_message(stderr_line: &str) -> bool {
        let crash_patterns = [
            "wine: Unhandled page fault",
            "wine: Unhandled exception",
            "Stack overflow",
            "Assertion failed",
            "abnormal program termination",
            "EXCEPTION_ACCESS_VIOLATION",
        ];
        crash_patterns
            .iter()
            .any(|p| stderr_line.contains(p))
    }

    /// Build command-line arguments to launch a game in a given bottle.
    pub fn build_wine_command(
        wine_bin: &str,
        prefix_path: &str,
        dll_overrides: &str,
        executable: &str,
        args: &[String],
    ) -> std::process::Command {
        let mut cmd = std::process::Command::new(wine_bin);
        cmd.env("WINEPREFIX", prefix_path);
        if !dll_overrides.is_empty() {
            cmd.env("WINEDLLOVERRIDES", dll_overrides);
        }
        cmd.arg(executable);
        for arg in args {
            cmd.arg(arg);
        }
        cmd
    }
}

impl Default for ProcessSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_list() {
        let mut supervisor = ProcessSupervisor::new();
        let proc = GameProcess {
            pid: 12345,
            bottle_id: "test-bottle".into(),
            steam_app_id: Some("123".into()),
            command: "wine64 game.exe".into(),
            started_at: Utc::now(),
            status: ProcessStatus::Running,
            exit_code: None,
        };
        supervisor.register(proc);
        assert_eq!(supervisor.list().len(), 1);
        assert_eq!(supervisor.list_for_bottle("test-bottle").len(), 1);
        assert_eq!(supervisor.list_for_bottle("other").len(), 0);
    }

    #[test]
    fn test_crash_detection() {
        assert!(
            ProcessSupervisor::is_crash_message("wine: Unhandled page fault at address 0xDEAD")
        );
        assert!(!ProcessSupervisor::is_crash_message("Loading module kernel32.dll"));
    }
}
