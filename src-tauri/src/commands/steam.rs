use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::{
    bottle::BottleManager,
    graphics::{DllOverrideConfig, GraphicsManager},
    process_supervisor::{GameProcess, ProcessStatus, ProcessSupervisor},
    steam_bridge::SteamApp,
    steam_compat,
    wine_manager::WineManager,
    AppState,
};
use tauri::Emitter;

const HOMEBREW_BIN_PATHS: [&str; 2] = ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"];

fn find_host_executable(name: &str, standard_paths: &[&str]) -> Option<PathBuf> {
    standard_paths
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
        .or_else(|| {
            std::env::var_os("PATH").and_then(|path| {
                std::env::split_paths(&path)
                    .map(|directory| directory.join(name))
                    .find(|candidate| candidate.is_file())
            })
        })
}

fn homebrew_not_found_error() -> String {
    format!(
        "Homebrew was not found. Budu checked {} and your app PATH. \\\n+         Install Homebrew from https://brew.sh, then choose Setup SteamCMD again.",
        HOMEBREW_BIN_PATHS.join(" and ")
    )
}

fn steamcmd_not_found_error() -> String {
    "SteamCMD was not found after setup. Budu checked /opt/homebrew/bin/steamcmd, \\
     /usr/local/bin/steamcmd, and your app PATH. Reopen Budu after installing SteamCMD, \\
     then try again."
        .into()
}

fn command_failure_details(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => format!("Process exited with status {:?}.", output.status.code()),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("{stderr}\n{stdout}"),
    }
}

fn run_with_progress(cmd: &mut Command, handle: &tauri::AppHandle) -> Result<(), String> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| format!("spawn: {e}"))?;
    let out = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let err = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    if let Some(o) = child.stdout.take() {
        let h = handle.clone();
        let l = out.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(o).lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    let _ = h.emit("steam:install-progress", line.clone());
                    l.lock().unwrap().push(line);
                }
            }
        });
    }
    if let Some(e) = child.stderr.take() {
        let h = handle.clone();
        let l = err.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(e).lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    let _ = h.emit("steam:install-progress", line.clone());
                    l.lock().unwrap().push(line);
                }
            }
        });
    }
    let s = child.wait().map_err(|e| format!("wait: {e}"))?;
    if !s.success() {
        let out_lines = out.lock().unwrap();
        let err_lines = err.lock().unwrap();
        let mut d = String::new();
        for l in out_lines.iter().chain(err_lines.iter()).rev().take(8) {
            d.push_str(l);
            d.push('\n');
        }
        return Err(format!("exited {:?}\n{d}", s.code()));
    }
    Ok(())
}

#[derive(Debug)]
struct LaunchSpec {
    wine_bin: PathBuf,
    prefix: PathBuf,
    working_dir: PathBuf,
    args: Vec<String>,
    environment: std::collections::HashMap<String, String>,
}

fn effective_graphics_backend(
    graphics_manager: &GraphicsManager,
    requested: crate::config::GraphicsBackend,
) -> crate::config::GraphicsBackend {
    graphics_manager
        .detect_backends()
        .into_iter()
        .find(|backend| backend.backend == requested && backend.installed)
        .map(|backend| backend.backend)
        .unwrap_or_else(|| graphics_manager.best_backend_for_d3d(11))
}

fn apply_graphics_environment(
    backend: crate::config::GraphicsBackend,
    environment: &mut std::collections::HashMap<String, String>,
    dll_overrides: String,
) {
    if backend == crate::config::GraphicsBackend::DXMT {
        environment.remove("WINEDLLPATH_PREPEND");
        environment
            .entry("DXMT_LOG_LEVEL".into())
            .or_insert_with(|| "error".into());
        environment.remove("WINEDLLOVERRIDES");
    } else if !dll_overrides.is_empty() {
        environment.insert("WINEDLLOVERRIDES".into(), dll_overrides);
    }
}

fn build_launch_spec(
    wine_manager: &WineManager,
    bottle_manager: &BottleManager,
    graphics_manager: &GraphicsManager,
    bottle_id: &str,
    executable: &Path,
) -> Result<LaunchSpec, String> {
    if !executable.is_file() {
        return Err(format!("File not found: {}", executable.display()));
    }

    let prefix = bottle_manager.resolve_bottle_path(bottle_id)?;
    let config = bottle_manager.get_config(bottle_id)?;
    let wine_bin = wine_manager.resolve_wine_bin(&config.bottle.wine_version)?;
    let override_config = DllOverrideConfig {
        dxgi: config.graphics.dxgi_override_native,
        d3d9: false,
        d3d10: config.graphics.d3d10_override_native,
        d3d11: config.graphics.d3d11_override_native,
        d3d12: config.graphics.d3d12_override_native,
    };
    let backend = effective_graphics_backend(graphics_manager, config.graphics.backend);
    let dll_overrides = graphics_manager.build_dll_overrides(backend, &override_config);
    if backend == crate::config::GraphicsBackend::DXMT {
        let dxmt_path = graphics_manager
            .dxmt_path()
            .ok_or_else(|| "DXMT runtime is not installed".to_string())?;
        wine_manager.install_dxmt_for_prefix(&wine_bin, &dxmt_path, &prefix)?;
    }

    let mut environment = config.environment;
    environment
        .entry("WINEDEBUG".into())
        .or_insert_with(|| "-all".into());
    environment
        .entry("MVK_CONFIG_LOG_LEVEL".into())
        .or_insert_with(|| "0".into());
    apply_graphics_environment(backend, &mut environment, dll_overrides);

    let executable_arg = executable.to_string_lossy().to_string();
    let mut args = if let Some(desktop) = config
        .windows
        .virtual_desktop
        .filter(|value| !value.trim().is_empty())
    {
        vec![
            "explorer".into(),
            format!("/desktop=Budu,{}", desktop.trim()),
            executable_arg,
        ]
    } else {
        vec![executable_arg]
    };
    if let Some(options) = config
        .steam
        .launch_options
        .filter(|value| !value.trim().is_empty())
    {
        args.extend(
            shell_words::split(&options)
                .map_err(|e| format!("Invalid launch options in bottle config: {e}"))?,
        );
    }

    Ok(LaunchSpec {
        wine_bin,
        prefix,
        working_dir: executable
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_path_buf(),
        args,
        environment,
    })
}

fn build_steam_launch_spec(
    wine_manager: &WineManager,
    bottle_manager: &BottleManager,
    graphics_manager: &GraphicsManager,
    bottle_id: &str,
    steam_executable: &Path,
    app_id: &str,
) -> Result<LaunchSpec, String> {
    if !steam_executable.is_file() {
        return Err(format!(
            "Steam is not installed in {}",
            steam_executable.display()
        ));
    }
    if !app_id.chars().all(|character| character.is_ascii_digit()) {
        return Err("Steam App ID must contain only digits".into());
    }

    let config = bottle_manager.get_config(bottle_id)?;
    let wine_bin = wine_manager.resolve_steam_wine_bin(&config.bottle.wine_version)?;
    let prefix = bottle_manager.resolve_bottle_path(bottle_id)?;
    let override_config = DllOverrideConfig {
        dxgi: config.graphics.dxgi_override_native,
        d3d9: false,
        d3d10: config.graphics.d3d10_override_native,
        d3d11: config.graphics.d3d11_override_native,
        d3d12: config.graphics.d3d12_override_native,
    };
    let backend = effective_graphics_backend(graphics_manager, config.graphics.backend);
    let dll_overrides = graphics_manager.build_dll_overrides(backend, &override_config);
    if backend == crate::config::GraphicsBackend::DXMT {
        let dxmt_path = graphics_manager
            .dxmt_path()
            .ok_or_else(|| "DXMT runtime is not installed".to_string())?;
        wine_manager.install_dxmt_for_prefix(&wine_bin, &dxmt_path, &prefix)?;
    }
    let mut environment = config.environment;
    environment
        .entry("WINEDEBUG".into())
        .or_insert_with(|| "-all".into());
    environment
        .entry("MVK_CONFIG_LOG_LEVEL".into())
        .or_insert_with(|| "0".into());
    apply_graphics_environment(backend, &mut environment, dll_overrides);

    let steam_path = format!(
        r"C:\Program Files (x86)\Steam\{}",
        steam_executable
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Steam executable has an invalid filename".to_string())?
    );
    let launch_options = config
        .steam
        .launch_options
        .filter(|value| !value.trim().is_empty());
    let steam_args = if let Some(options) = launch_options {
        let mut args = vec![
            steam_path,
            "-noverifyfiles".into(),
            "-applaunch".into(),
            app_id.to_string(),
        ];
        args.extend(
            shell_words::split(&options)
                .map_err(|error| format!("Invalid launch options in bottle config: {error}"))?,
        );
        args
    } else {
        // `-applaunch` selects the first configured Steam launch option. For
        // games such as Phasmophobia that option is VR. The desktop-shortcut
        // protocol selects the publisher's default launch option instead.
        vec![steam_path, format!("steam://rungameid/{app_id}")]
    };
    let args = if let Some(desktop) = config
        .windows
        .virtual_desktop
        .filter(|value| !value.trim().is_empty())
    {
        let mut desktop_args = vec![
            "explorer".into(),
            format!("/desktop=Budu,{}", desktop.trim()),
        ];
        desktop_args.extend(steam_args);
        desktop_args
    } else {
        steam_args
    };

    Ok(LaunchSpec {
        wine_bin,
        prefix,
        working_dir: steam_executable
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_path_buf(),
        args,
        environment,
    })
}

fn spawn_tracked(
    spec: LaunchSpec,
    bottle_id: String,
    steam_app_id: Option<String>,
    command_label: String,
    logs_dir: PathBuf,
    supervisor: Arc<Mutex<ProcessSupervisor>>,
) -> Result<u32, String> {
    let bottle_log_dir = logs_dir.join(&bottle_id);
    std::fs::create_dir_all(&bottle_log_dir)
        .map_err(|error| format!("Failed to create launch log directory: {error}"))?;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(bottle_log_dir.join("stdout.log"))
        .map_err(|error| format!("Failed to open launch log: {error}"))?;
    let error_log = log
        .try_clone()
        .map_err(|error| format!("Failed to open launch error log: {error}"))?;

    let mut command = Command::new(&spec.wine_bin);
    command.env_clear().envs(&spec.environment);
    configure_wine_runner(&mut command, &spec.wine_bin, &spec.prefix)?;
    command
        .current_dir(&spec.working_dir)
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(error_log));
    command.args(&spec.args);
    inherit_safe_host_environment(&mut command);
    let mut child = command
        .spawn()
        .map_err(|e| format!("Failed to launch: {e}"))?;
    let pid = child.id();

    supervisor.lock().unwrap().register(GameProcess {
        pid,
        bottle_id,
        steam_app_id,
        command: command_label,
        started_at: chrono::Utc::now(),
        status: ProcessStatus::Running,
        exit_code: None,
    });

    std::thread::spawn(move || {
        let (status, exit_code) = match child.wait() {
            Ok(exit) if exit.success() => (ProcessStatus::Exited, exit.code()),
            Ok(exit) => (ProcessStatus::Crashed, exit.code()),
            Err(error) => {
                tracing::warn!("Failed waiting for child process {pid}: {error}");
                (ProcessStatus::Crashed, None)
            }
        };
        supervisor.lock().unwrap().complete(pid, status, exit_code);
    });

    Ok(pid)
}

fn inherit_safe_host_environment(command: &mut Command) {
    for key in [
        "HOME",
        "USER",
        "LOGNAME",
        "PATH",
        "SHELL",
        "TMPDIR",
        "LANG",
        "LC_ALL",
        "DISPLAY",
        "XDG_RUNTIME_DIR",
        "__CF_USER_TEXT_ENCODING",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
}

fn configure_wine_runner(
    command: &mut Command,
    _wine_bin: &Path,
    prefix: &Path,
) -> Result<(), String> {
    command.env("WINEPREFIX", prefix);
    Ok(())
}

fn is_separate_steam_prefix(shared_prefix: &Path, game_prefix: &Path) -> bool {
    let shared = shared_prefix
        .canonicalize()
        .unwrap_or_else(|_| shared_prefix.to_path_buf());
    let game = game_prefix
        .canonicalize()
        .unwrap_or_else(|_| game_prefix.to_path_buf());
    shared != game
}

fn stop_wine_prefix(
    wine_manager: &WineManager,
    wine_version: &str,
    prefix: &Path,
) -> Result<(), String> {
    let wineserver = wine_manager.resolve_wineserver_bin(wine_version)?;
    let mut command = Command::new(&wineserver);
    command.env_clear();
    inherit_safe_host_environment(&mut command);
    configure_wine_runner(&mut command, &wineserver, prefix)?;
    let output = command
        .arg("-k")
        .output()
        .map_err(|error| format!("Failed to stop the existing Steam session: {error}"))?;
    // `wineserver -k` exits with code 1 and no diagnostic when the prefix has
    // no running server. That is the normal state after a restart (or after a
    // previous game has already exited), so there is no Steam session to stop.
    if wineserver_was_stopped(&output) {
        Ok(())
    } else {
        Err(format!(
            "Failed to stop the existing Steam session: {}",
            command_failure_details(&output)
        ))
    }
}

fn wineserver_was_stopped(output: &std::process::Output) -> bool {
    output.status.success()
        || (output.status.code() == Some(1) && output.stdout.is_empty() && output.stderr.is_empty())
}

/// Steam's files are shared to save disk space, but its Wine prefix must be
/// exclusive to the game being launched. Otherwise a shared-prefix Steam GUI
/// can remain logged in and receive multiplayer invites intended for the game.
fn stop_shared_steam_before_game_launch(state: &AppState, bottle_id: &str) -> Result<(), String> {
    let game_prefix = state.bottle_manager.resolve_bottle_path(bottle_id)?;
    let shared_prefix = state.steam_bridge.steam_bottle_path();
    if !is_separate_steam_prefix(&shared_prefix, &game_prefix) {
        return Ok(());
    }
    stop_wine_prefix(
        &state.wine_manager,
        &state.config.default_wine_version,
        &shared_prefix,
    )
}

fn prepare_steam_ui(state: &AppState, steam_executable: &Path) -> Result<(), String> {
    let shim = steam_compat::resolve_shim(state.resource_dir.as_deref())?;
    steam_compat::install_for_steam(steam_executable, &shim)?;
    Ok(())
}

fn link_shared_steam_into_bottle(
    bottle_prefix: &Path,
    shared_steam_executable: &Path,
) -> Result<PathBuf, String> {
    if !shared_steam_executable.is_file() {
        return Err(format!(
            "Steam executable not found: {}",
            shared_steam_executable.display()
        ));
    }
    let shared_steam_dir = shared_steam_executable
        .parent()
        .ok_or_else(|| "Steam executable has no parent directory".to_string())?
        .canonicalize()
        .map_err(|error| format!("Failed to resolve shared Steam directory: {error}"))?;
    let link = bottle_prefix.join("drive_c/Program Files (x86)/Steam");
    std::fs::create_dir_all(
        link.parent()
            .ok_or_else(|| "Bottle Steam path has no parent directory".to_string())?,
    )
    .map_err(|error| format!("Failed to prepare bottle Steam directory: {error}"))?;

    if let Ok(existing_target) = std::fs::read_link(&link) {
        let resolved_target = if existing_target.is_absolute() {
            existing_target
        } else {
            link.parent().unwrap().join(existing_target)
        }
        .canonicalize()
        .map_err(|error| format!("Failed to resolve bottle Steam link: {error}"))?;
        if resolved_target != shared_steam_dir {
            return Err(format!(
                "Bottle Steam link points to an unexpected directory: {}",
                resolved_target.display()
            ));
        }
    } else if link.exists() {
        return Err(format!(
            "Bottle already contains an unmanaged Steam directory: {}",
            link.display()
        ));
    } else {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&shared_steam_dir, &link)
            .map_err(|error| format!("Failed to link shared Steam into bottle: {error}"))?;
        #[cfg(not(unix))]
        return Err("Shared Steam bottle linking is only supported on macOS".into());
    }

    let executable_name = shared_steam_executable
        .file_name()
        .ok_or_else(|| "Steam executable has no filename".to_string())?;
    Ok(link.join(executable_name))
}

// ── SteamCMD ──────────────────────────────────────────────

#[tauri::command]
pub async fn steamcmd_install(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // SteamCMD downloads are shared; managed Wine prefixes remain per game.
    let steam_bottle = state.steam_bridge.steam_bottle_path();
    std::fs::create_dir_all(&steam_bottle).map_err(|e| format!("mkdir: {e}"))?;

    if state.steam_bridge.steamcmd_bin_path().is_some() {
        return Ok(());
    }
    let brew =
        find_host_executable("brew", &HOMEBREW_BIN_PATHS).ok_or_else(homebrew_not_found_error)?;
    let _ = app_handle.emit("steam:install-progress", "Installing SteamCMD...");
    let output = Command::new(&brew)
        .args(["install", "steamcmd"])
        .output()
        .map_err(|error| format!("Could not run {}: {error}", brew.display()))?;
    if !output.status.success() {
        return Err(format!(
            "SteamCMD setup failed while running `{}` install steamcmd:\n{}",
            brew.display(),
            command_failure_details(&output)
        ));
    }
    state
        .steam_bridge
        .steamcmd_bin_path()
        .ok_or_else(steamcmd_not_found_error)?;
    let _ = app_handle.emit("steam:installed", ());
    Ok(())
}

fn steamcmd_terminal_instructions(app_id: &str, install: &Path) -> String {
    format!(
        "In SteamCMD, enter these commands one at a time:\n\
         force_install_dir \"{}\"\n\
         login YOUR_STEAM_USERNAME\n\
         app_update {app_id} validate\n\
         quit\n\n\
         Do not paste your password into Budu. SteamCMD will ask for it directly. \
         When the download finishes, return to Budu and click Refresh.",
        install.display()
    )
}

#[tauri::command]
pub async fn steamcmd_open_terminal(
    app_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let install = state.steam_bridge.prepare_steamcmd_download(&app_id)?;
    let steamcmd = state
        .steam_bridge
        .steamcmd_bin_path()
        .ok_or_else(steamcmd_not_found_error)?;
    Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "tell app \"Terminal\" to do script \"{}; echo '--- Type quit to exit ---'; read\"",
            steamcmd.display()
        ))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(steamcmd_terminal_instructions(&app_id, &install))
}

#[tauri::command]
pub async fn steamcmd_download(
    app_id: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let install = state.steam_bridge.prepare_steamcmd_download(&app_id)?;
    let steamcmd = state
        .steam_bridge
        .steamcmd_bin_path()
        .ok_or_else(steamcmd_not_found_error)?;
    let _ = app_handle.emit("steam:install-progress", format!("download {app_id}..."));
    let mut cmd = Command::new(steamcmd);
    cmd.arg("+@sSteamCmdForcePlatformType")
        .arg("windows")
        .arg("+force_install_dir")
        .arg(install.to_string_lossy().to_string());
    cmd.arg("+login").arg("anonymous");
    cmd.arg("+app_update")
        .arg(&app_id)
        .arg("validate")
        .arg("+quit");
    run_with_progress(&mut cmd, &app_handle)?;
    state
        .steam_bridge
        .finalize_steamcmd_download(&app_id, &install)?;
    let _ = app_handle.emit("steam:download-complete", &app_id);
    Ok(())
}

// ── Process killer ─────────────────────────────────────────

#[tauri::command]
pub async fn kill_wine() -> Result<(), String> {
    let _ = Command::new("killall")
        .args(["-9", "wineserver", "winedevice", "wine64"])
        .output();
    let _ = Command::new("killall")
        .args(["-9", "steam.exe", "Phasmophobia", "Raft"])
        .output();
    Ok(())
}

fn restore_original_steam_dlls(game_directory: &Path) -> Result<usize, String> {
    let originals: Vec<_> = walkdir::WalkDir::new(game_directory)
        .max_depth(6)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("steam_api64.dll.orig")
        })
        .collect();
    for original in &originals {
        let target = original.path().with_extension("");
        std::fs::copy(original.path(), &target).map_err(|error| {
            format!(
                "Failed to restore {} from backup: {error}",
                target.display()
            )
        })?;
    }
    Ok(originals.len())
}

// ── Game launcher ──────────────────────────────────────────

#[tauri::command]
pub async fn run_exe(
    path: String,
    bottle_id: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let _steam_session_guard = state
        .steam_session_lock
        .lock()
        .map_err(|_| "Steam launch coordination is unavailable".to_string())?;
    let exe = std::path::PathBuf::from(&path);

    let _ = app_handle.emit(
        "steam:install-progress",
        format!(
            "Launching: {}",
            exe.file_name().unwrap_or_default().to_string_lossy()
        ),
    );
    let steam_app_id = state.steam_bridge.app_id_for_executable(&exe);
    let spec = if let Some(app_id) = steam_app_id.as_deref() {
        let _ = app_handle.emit(
            "steam:install-progress",
            "Closing Budu's separate Steam session...",
        );
        stop_shared_steam_before_game_launch(&state, &bottle_id)?;
        let shared_prefix = state
            .steam_bridge
            .prefix_for_executable(&exe)
            .unwrap_or_else(|| state.steam_bridge.steam_bottle_path());
        let shared_steam_executable = {
            let lowercase = shared_prefix.join("drive_c/Program Files (x86)/Steam/steam.exe");
            let titlecase = shared_prefix.join("drive_c/Program Files (x86)/Steam/Steam.exe");
            if lowercase.is_file() {
                lowercase
            } else {
                titlecase
            }
        };
        let bottle_prefix = state.bottle_manager.resolve_bottle_path(&bottle_id)?;
        let steam_executable =
            link_shared_steam_into_bottle(&bottle_prefix, &shared_steam_executable)?;
        restore_original_steam_dlls(exe.parent().unwrap_or_else(|| Path::new("/")))?;
        prepare_steam_ui(&state, &steam_executable)?;
        let _ = app_handle.emit(
            "steam:install-progress",
            format!("Steam game detected. Launching App {app_id} through Steam..."),
        );
        build_steam_launch_spec(
            &state.wine_manager,
            &state.bottle_manager,
            &state.graphics_manager,
            &bottle_id,
            &steam_executable,
            app_id,
        )?
    } else {
        build_launch_spec(
            &state.wine_manager,
            &state.bottle_manager,
            &state.graphics_manager,
            &bottle_id,
            &exe,
        )?
    };
    spawn_tracked(
        spec,
        bottle_id,
        steam_app_id,
        exe.to_string_lossy().to_string(),
        state.config.logs_dir.clone(),
        state.process_supervisor.clone(),
    )?;

    let _ = app_handle.emit(
        "steam:install-progress",
        "Game launched -- check your screen.",
    );
    Ok(())
}

// ── Steam GUI ──────────────────────────────────────────────

#[tauri::command]
pub async fn steam_status(
    state: tauri::State<'_, AppState>,
) -> Result<crate::steam_bridge::SteamStatus, String> {
    Ok(state.steam_bridge.get_status())
}
#[tauri::command]
pub async fn steam_list_games(state: tauri::State<'_, AppState>) -> Result<Vec<SteamApp>, String> {
    state.steam_bridge.list_installed_games()
}

#[tauri::command]
pub async fn steam_install(
    setup_path: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let sb = state.steam_bridge.steam_bottle_path();
    std::fs::create_dir_all(&sb)
        .map_err(|e| format!("Failed to create shared Steam directory: {e}"))?;
    let wine = state
        .wine_manager
        .resolve_steam_wine_bin(&state.config.default_wine_version)?;
    let mut command = Command::new(&wine);
    command.env_clear();
    inherit_safe_host_environment(&mut command);
    configure_wine_runner(&mut command, &wine, &sb)?;
    let o = command
        .arg(&setup_path)
        .arg("/S")
        .output()
        .map_err(|e| e.to_string())?;
    if !o.status.success() {
        return Err(String::from_utf8_lossy(&o.stderr).to_string());
    }
    let _ = app.emit("steam:installed", ());
    Ok(())
}

#[tauri::command]
pub async fn steam_run_client(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let _steam_session_guard = state
        .steam_session_lock
        .lock()
        .map_err(|_| "Steam launch coordination is unavailable".to_string())?;
    let exe = state.steam_bridge.steam_exe_path();
    if !exe.exists() {
        return Err("Steam GUI not installed".into());
    }
    let wine = state
        .wine_manager
        .resolve_steam_wine_bin(&state.config.default_wine_version)?;
    let prefix = state.steam_bridge.steam_bottle_path();
    // Opening the Steam UI twice in its shared prefix creates two clients
    // under the same account. Replace any earlier shared-prefix client first.
    stop_wine_prefix(
        &state.wine_manager,
        &state.config.default_wine_version,
        &prefix,
    )?;
    prepare_steam_ui(&state, &exe)?;
    let mut command = Command::new(&wine);
    command.env_clear();
    inherit_safe_host_environment(&mut command);
    configure_wine_runner(&mut command, &wine, &prefix)?;
    command
        .arg(exe.to_string_lossy().to_string())
        .arg("-noverifyfiles")
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn steam_launch_game(
    app_id: String,
    bottle_id: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let _steam_session_guard = state
        .steam_session_lock
        .lock()
        .map_err(|_| "Steam launch coordination is unavailable".to_string())?;
    let games = state.steam_bridge.list_installed_games()?;
    let g = games
        .iter()
        .find(|g| g.app_id == app_id)
        .ok_or_else(|| format!("App {app_id} not found"))?;
    let exe = if g.install_dir.extension().is_some_and(|e| e == "exe") {
        g.install_dir.clone()
    } else {
        find_exe(&g.install_dir)?
    };
    let config = state.bottle_manager.get_config(&bottle_id)?;
    if config.steam.app_id.as_deref() != Some(app_id.as_str()) {
        return Err(format!(
            "Bottle '{bottle_id}' is not assigned to Steam App {app_id}"
        ));
    }

    let _ = app_handle.emit(
        "steam:install-progress",
        "Closing Budu's separate Steam session...",
    );
    stop_shared_steam_before_game_launch(&state, &bottle_id)?;

    let shared_steam_executable = state.steam_bridge.steam_exe_path();
    let bottle_prefix = state.bottle_manager.resolve_bottle_path(&bottle_id)?;
    let steam_executable = link_shared_steam_into_bottle(&bottle_prefix, &shared_steam_executable)?;
    restore_original_steam_dlls(exe.parent().unwrap_or_else(|| Path::new("/")))?;
    prepare_steam_ui(&state, &steam_executable)?;
    let _ = app_handle.emit(
        "steam:install-progress",
        format!("Launching {} through Steam...", g.name),
    );
    let spec = build_steam_launch_spec(
        &state.wine_manager,
        &state.bottle_manager,
        &state.graphics_manager,
        &bottle_id,
        &steam_executable,
        &app_id,
    )?;
    let pid = spawn_tracked(
        spec,
        bottle_id,
        Some(app_id.clone()),
        exe.to_string_lossy().to_string(),
        state.config.logs_dir.clone(),
        state.process_supervisor.clone(),
    )?;
    Ok(format!("launched:{app_id}:{pid}"))
}

fn find_exe(dir: &std::path::PathBuf) -> Result<std::path::PathBuf, String> {
    for e in walkdir::WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
    {
        if e.path().extension().is_some_and(|x| x == "exe") {
            let n = e
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            if !n.contains("uninstall") && !n.contains("unins") && n != "steam.exe" {
                return Ok(e.path().to_path_buf());
            }
        }
    }
    Err(format!("No exe in {}", dir.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bottle::BottleConfig,
        config::{AppConfig, GraphicsBackend},
    };
    use tempfile::TempDir;

    #[test]
    fn terminal_instructions_never_include_a_password() {
        let instructions = steamcmd_terminal_instructions("480", Path::new("/tmp/download"));

        assert!(instructions.contains("app_update 480 validate"));
        assert!(instructions.contains("login YOUR_STEAM_USERNAME"));
        assert!(!instructions.contains("YOUR_STEAM_PASSWORD"));
    }

    #[test]
    fn missing_homebrew_error_names_the_locations_that_were_checked() {
        let error = homebrew_not_found_error();

        assert!(error.contains("/opt/homebrew/bin/brew"));
        assert!(error.contains("/usr/local/bin/brew"));
        assert!(error.contains("https://brew.sh"));
    }

    #[test]
    fn managed_game_prefix_requires_shared_steam_shutdown() {
        let tmp = TempDir::new().unwrap();
        let shared = tmp.path().join("steam");
        let game = tmp.path().join("bottles/raft");
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::create_dir_all(&game).unwrap();

        assert!(is_separate_steam_prefix(&shared, &game));
        assert!(!is_separate_steam_prefix(&shared, &shared));
    }

    #[test]
    fn stopped_wineserver_without_a_running_prefix_is_not_an_error() {
        let output = Command::new("sh").args(["-c", "exit 1"]).output().unwrap();

        assert!(wineserver_was_stopped(&output));
    }

    #[test]
    fn stopped_wineserver_with_a_diagnostic_is_an_error() {
        let output = Command::new("sh")
            .args(["-c", "echo unable to connect >&2; exit 1"])
            .output()
            .unwrap();

        assert!(!wineserver_was_stopped(&output));
    }

    #[test]
    #[cfg(unix)]
    fn links_shared_steam_into_a_managed_bottle() {
        let tmp = TempDir::new().unwrap();
        let shared = tmp.path().join("shared/Steam");
        let prefix = tmp.path().join("bottle");
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::create_dir_all(&prefix).unwrap();
        let shared_executable = shared.join("steam.exe");
        std::fs::write(&shared_executable, b"steam").unwrap();

        let linked = link_shared_steam_into_bottle(&prefix, &shared_executable).unwrap();

        assert_eq!(
            linked,
            prefix.join("drive_c/Program Files (x86)/Steam/steam.exe")
        );
        assert_eq!(
            linked.parent().unwrap().canonicalize().unwrap(),
            shared.canonicalize().unwrap()
        );
        assert_eq!(
            link_shared_steam_into_bottle(&prefix, &shared_executable).unwrap(),
            linked
        );
    }

    #[test]
    fn dxmt_removes_incompatible_loader_overrides() {
        let mut environment = std::collections::HashMap::from([
            ("WINEDLLPATH".into(), "/existing/wine/dlls".into()),
            ("WINEDLLPATH_PREPEND".into(), "/obsolete".into()),
            ("WINEDLLOVERRIDES".into(), "dxgi=n".into()),
        ]);

        apply_graphics_environment(GraphicsBackend::DXMT, &mut environment, String::new());

        assert_eq!(
            environment.get("WINEDLLPATH").map(String::as_str),
            Some("/existing/wine/dlls")
        );
        assert!(!environment.contains_key("WINEDLLPATH_PREPEND"));
        assert!(!environment.contains_key("WINEDLLOVERRIDES"));
    }

    #[test]
    fn launch_spec_uses_saved_bottle_configuration() {
        let tmp = TempDir::new().unwrap();
        let config = AppConfig {
            data_dir: tmp.path().to_path_buf(),
            wine_dir: tmp.path().join("wine"),
            bottles_dir: tmp.path().join("bottles"),
            runtimes_dir: tmp.path().join("runtimes"),
            logs_dir: tmp.path().join("logs"),
            compat_db_path: tmp.path().join("compat/db.json"),
            default_wine_version: "test-wine".into(),
            default_graphics_backend: GraphicsBackend::D3DMetal,
        };
        config.ensure_dirs().unwrap();
        let wine_bin = config.wine_dir.join("test-wine/bin/wine64");
        std::fs::create_dir_all(wine_bin.parent().unwrap()).unwrap();
        std::fs::write(&wine_bin, b"test").unwrap();
        std::fs::write(config.wine_dir.join("test-wine/bin/wineserver"), b"test").unwrap();
        let executable = tmp.path().join("game.exe");
        std::fs::write(&executable, b"test").unwrap();

        let bottle_manager = BottleManager::new(config.clone());
        let bottle =
            tokio_test::block_on(bottle_manager.create_bottle("Test", "test-wine", Some("730")))
                .unwrap();
        let mut bottle_config: BottleConfig = bottle_manager.get_config(&bottle.id).unwrap();
        bottle_config.graphics.backend = GraphicsBackend::WineD3D;
        bottle_config
            .environment
            .insert("CUSTOM".into(), "1".into());
        bottle_config.windows.virtual_desktop = Some("1280x720".into());
        bottle_config.steam.launch_options = Some(r#"-novid "two words""#.into());
        bottle_manager
            .save_config(&bottle.id, &bottle_config)
            .unwrap();

        let spec = build_launch_spec(
            &WineManager::new(config.clone()),
            &bottle_manager,
            &GraphicsManager::new(config),
            &bottle.id,
            &executable,
        )
        .unwrap();

        assert_eq!(spec.prefix, bottle.prefix_path.canonicalize().unwrap());
        assert_eq!(
            spec.environment.get("CUSTOM").map(String::as_str),
            Some("1")
        );
        assert!(spec
            .environment
            .get("WINEDLLOVERRIDES")
            .is_some_and(|value| value.contains("dxgi=b")));
        assert_eq!(spec.args[0], "explorer");
        assert_eq!(spec.args[1], "/desktop=Budu,1280x720");
        assert_eq!(&spec.args[3..], &["-novid", "two words"]);
    }

    #[test]
    fn steam_launch_spec_uses_managed_game_prefix_and_app_id() {
        let tmp = TempDir::new().unwrap();
        let config = AppConfig {
            data_dir: tmp.path().to_path_buf(),
            wine_dir: tmp.path().join("wine"),
            bottles_dir: tmp.path().join("bottles"),
            runtimes_dir: tmp.path().join("runtimes"),
            logs_dir: tmp.path().join("logs"),
            compat_db_path: tmp.path().join("compat/db.json"),
            default_wine_version: "test-wine".into(),
            default_graphics_backend: GraphicsBackend::D3DMetal,
        };
        config.ensure_dirs().unwrap();
        let wine_bin = config.wine_dir.join("test-wine/bin/wine64");
        std::fs::create_dir_all(wine_bin.parent().unwrap()).unwrap();
        std::fs::write(&wine_bin, b"test").unwrap();
        std::fs::write(config.wine_dir.join("test-wine/bin/wineserver"), b"test").unwrap();
        let steam_prefix = tmp.path().join("shared-steam");
        let steam_executable = steam_prefix.join("drive_c/Program Files (x86)/Steam/steam.exe");
        std::fs::create_dir_all(steam_executable.parent().unwrap()).unwrap();
        std::fs::write(&steam_executable, b"steam").unwrap();

        let bottle_manager = BottleManager::new(config.clone());
        let bottle =
            tokio_test::block_on(bottle_manager.create_bottle("Raft", "test-wine", Some("648800")))
                .unwrap();
        let spec = build_steam_launch_spec(
            &WineManager::new(config.clone()),
            &bottle_manager,
            &GraphicsManager::new(config),
            &bottle.id,
            &steam_executable,
            "648800",
        )
        .unwrap();

        assert_eq!(spec.prefix, bottle.prefix_path.canonicalize().unwrap());
        assert_eq!(
            spec.environment.get("WINEDLLOVERRIDES").map(String::as_str),
            Some("dxgi=b;d3d11=b;d3d10=b;d3d9=b")
        );
        assert_eq!(
            spec.args,
            [
                String::from(r"C:\Program Files (x86)\Steam\steam.exe"),
                String::from("steam://rungameid/648800")
            ]
        );
    }

    #[test]
    fn steam_launch_spec_honors_saved_launch_options() {
        let tmp = TempDir::new().unwrap();
        let config = AppConfig {
            data_dir: tmp.path().to_path_buf(),
            wine_dir: tmp.path().join("wine"),
            bottles_dir: tmp.path().join("bottles"),
            runtimes_dir: tmp.path().join("runtimes"),
            logs_dir: tmp.path().join("logs"),
            compat_db_path: tmp.path().join("compat/db.json"),
            default_wine_version: "test-wine".into(),
            default_graphics_backend: GraphicsBackend::D3DMetal,
        };
        config.ensure_dirs().unwrap();
        let wine_bin = config.wine_dir.join("test-wine/bin/wine64");
        std::fs::create_dir_all(wine_bin.parent().unwrap()).unwrap();
        std::fs::write(&wine_bin, b"test").unwrap();
        std::fs::write(config.wine_dir.join("test-wine/bin/wineserver"), b"test").unwrap();
        let steam_executable = tmp
            .path()
            .join("shared-steam/drive_c/Program Files (x86)/Steam/steam.exe");
        std::fs::create_dir_all(steam_executable.parent().unwrap()).unwrap();
        std::fs::write(&steam_executable, b"steam").unwrap();

        let bottle_manager = BottleManager::new(config.clone());
        let bottle =
            tokio_test::block_on(bottle_manager.create_bottle("Test", "test-wine", Some("730")))
                .unwrap();
        let mut bottle_config = bottle_manager.get_config(&bottle.id).unwrap();
        bottle_config.steam.launch_options = Some(r#"-novid "two words""#.into());
        bottle_manager
            .save_config(&bottle.id, &bottle_config)
            .unwrap();

        let spec = build_steam_launch_spec(
            &WineManager::new(config.clone()),
            &bottle_manager,
            &GraphicsManager::new(config),
            &bottle.id,
            &steam_executable,
            "730",
        )
        .unwrap();

        assert_eq!(
            spec.args,
            [
                String::from(r"C:\Program Files (x86)\Steam\steam.exe"),
                String::from("-noverifyfiles"),
                String::from("-applaunch"),
                String::from("730"),
                String::from("-novid"),
                String::from("two words")
            ]
        );
    }

    #[test]
    fn restores_original_steam_api_dll() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path().join("Raft_Data/Plugins/x86_64");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        let dll = plugin_dir.join("steam_api64.dll");
        let backup = plugin_dir.join("steam_api64.dll.orig");
        std::fs::write(&dll, b"replacement").unwrap();
        std::fs::write(&backup, b"original").unwrap();

        assert_eq!(restore_original_steam_dlls(tmp.path()).unwrap(), 1);
        assert_eq!(std::fs::read(dll).unwrap(), b"original");
    }
}
