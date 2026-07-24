use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use tauri::Emitter;
use crate::{steam_bridge::SteamApp, AppState};

const STEAMCMD_BIN: &str = "/opt/homebrew/bin/steamcmd";

fn run_with_progress(cmd: &mut Command, handle: &tauri::AppHandle) -> Result<(), String> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| format!("spawn: {e}"))?;
    let out = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let err = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    if let Some(o) = child.stdout.take() {
        let h = handle.clone();
        let l = out.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(o).lines() {
                if let Ok(line) = line {
                    if !line.trim().is_empty() {
                        let _ = h.emit("steam:install-progress", line.clone());
                        l.lock().unwrap().push(line);
                    }
                }
            }
        });
    }
    if let Some(e) = child.stderr.take() {
        let h = handle.clone();
        let l = err.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(e).lines() {
                if let Ok(line) = line {
                    if !line.trim().is_empty() {
                        let _ = h.emit("steam:install-progress", line.clone());
                        l.lock().unwrap().push(line);
                    }
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

// ── SteamCMD ──────────────────────────────────────────────

#[tauri::command]
pub async fn steamcmd_install(app_handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Create the Steam bottle directory on disk
    let steam_bottle = state.steam_bridge.steam_bottle_path();
    std::fs::create_dir_all(&steam_bottle).map_err(|e| format!("mkdir: {e}"))?;

    if std::path::PathBuf::from(STEAMCMD_BIN).exists() { return Ok(()); }
    let _ = app_handle.emit("steam:install-progress", "Installing SteamCMD...");
    let s = Command::new("brew").args(["install", "steamcmd"]).status().map_err(|e| e.to_string())?;
    if !s.success() { return Err("brew install steamcmd failed".into()); }
    let _ = app_handle.emit("steam:installed", ());
    Ok(())
}

#[tauri::command]
pub async fn steamcmd_open_terminal(_state: tauri::State<'_, AppState>) -> Result<(), String> {
    Command::new("osascript").arg("-e").arg(format!("tell app \"Terminal\" to do script \"{}; echo '--- Type quit to exit ---'; read\"", STEAMCMD_BIN)).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn steamcmd_download(app_id: String, username: Option<String>, password: Option<String>, app_handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let install = state.steam_bridge.steamapps_path().join("common");
    std::fs::create_dir_all(&install).ok();
    let _ = app_handle.emit("steam:install-progress", format!("download {app_id}..."));
    let mut cmd = Command::new(STEAMCMD_BIN);
    cmd.arg("+@sSteamCmdForcePlatformType").arg("windows").arg("+force_install_dir").arg(install.to_string_lossy().to_string());
    if let (Some(u), Some(p)) = (&username, &password) { cmd.arg("+login").arg(u).arg(p); } else { cmd.arg("+login").arg("anonymous"); }
    cmd.arg("+app_update").arg(&app_id).arg("validate").arg("+quit");
    run_with_progress(&mut cmd, &app_handle)?;
    let _ = app_handle.emit("steam:download-complete", &app_id);
    Ok(())
}

// ── Process killer ─────────────────────────────────────────

#[tauri::command]
pub async fn kill_wine() -> Result<(), String> {
    let _ = Command::new("killall").args(["-9", "wineserver", "winedevice", "wine64"]).output();
    let _ = Command::new("killall").args(["-9", "steam.exe", "Phasmophobia", "Raft"]).output();
    Ok(())
}

// ── Goldberg Steam Emu setup ───────────────────────────────

const GOLDBERG_URL: &str = "https://gitlab.com/Mr_Goldberg/goldberg_emulator/-/jobs/artifacts/master/download?job=build";

fn ensure_goldberg(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = dirs_next();

    if !dir.join("steam_api64.dll").exists() {
        let _ = app_handle.emit("steam:install-progress", "Downloading Steam emulator...");
        let zip = dir.join("goldberg.zip");
        let s = Command::new("curl").args(["-L", "-o", &zip.to_string_lossy().to_string(), GOLDBERG_URL])
            .status().map_err(|e| format!("curl: {e}"))?;
        if !s.success() { return Err("Download failed".into()); }
        Command::new("unzip").args(["-o", &zip.to_string_lossy().to_string(), "-d", &dir.to_string_lossy().to_string()])
            .status().map_err(|e| format!("unzip: {e}"))?;
    }

    // Find the DLL in extracted files
    let dll = dir.join("steam_api64.dll");
    if !dll.exists() {
        // Try common locations in the goldberg zip
        for candidate in &["steam_api64.dll", "experimental/steam_api64.dll"] {
            let c = dir.join(candidate);
            if c.exists() { return Ok(c); }
        }
        return Err("Goldberg DLL not found in zip".into());
    }
    Ok(dll)
}

fn dirs_next() -> std::path::PathBuf {
    std::path::PathBuf::from("/tmp/goldberg_emu")
}

fn patch_game_for_offline(exe: &std::path::Path, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let fallback = std::path::PathBuf::from("/");
    let game_dir = exe.parent().unwrap_or(&fallback);

    // Find steam_api64.dll in the game tree
    let steam_dlls: Vec<_> = walkdir::WalkDir::new(game_dir).max_depth(6).into_iter()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy() == "steam_api64.dll")
        .collect();

    if steam_dlls.is_empty() {
        let _ = app_handle.emit("steam:install-progress", "No Steam DRM found -- launching directly.");
        return Ok(());
    }

    let gdb = ensure_goldberg(app_handle)?;
    let _ = app_handle.emit("steam:install-progress", "Patching Steam DRM (offline mode)...");

    for dll in &steam_dlls {
        // Backup original
        let orig = dll.path().with_extension("dll.orig");
        if !orig.exists() {
            std::fs::copy(dll.path(), &orig).ok();
        }
        // Replace with goldberg
        std::fs::copy(&gdb, dll.path()).map_err(|e| format!("copy: {e}"))?;
    }

    // Write steam_appid.txt -- look for existing or use directory name to guess
    let appid_path = game_dir.join("steam_appid.txt");
    let existing = std::fs::read_to_string(&appid_path).unwrap_or_default();
    if existing.trim().is_empty() || existing.trim() == "480" {
        // Try to find appid from a nearby steam_appid.txt or from the game folder name
        let mut wrote = false;
        // Check common dir (steamapps/common)
        if let Some(common) = game_dir.parent() {
            let steamapps = common.parent();
            if let Some(sa) = steamapps {
                // Parse appmanifest files to find the appid for this game
                if let Ok(entries) = std::fs::read_dir(sa) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("appmanifest_") && name.ends_with(".acf") {
                            if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                                if contents.contains(game_dir.file_name().unwrap_or_default().to_string_lossy().as_ref()) {
                                    for line in contents.lines() {
                                        let parts: Vec<&str> = line.trim().split('"').collect();
                                        if parts.len() >= 4 && parts[1] == "appid" {
                                            std::fs::write(&appid_path, parts[3]).ok();
                                            wrote = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if !wrote {
            // Last resort: keep existing or write placeholder
            if existing.trim().is_empty() {
                std::fs::write(&appid_path, "480").ok();
            }
        }
    }

    let _ = app_handle.emit("steam:install-progress", "DRM patched. Launching offline...");
    Ok(())
}

// ── Game launcher ──────────────────────────────────────────

#[tauri::command]
pub async fn run_exe(path: String, app_handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let exe = std::path::PathBuf::from(&path);
    if !exe.exists() { return Err(format!("File not found: {path}")); }

    let wine = state.wine_manager.resolve_wine_bin("9.14-staging")?;
    let prefix = state.steam_bridge.steam_bottle_path().to_string_lossy().to_string();
    let root = std::path::PathBuf::from("/");
    let exe_dir = exe.parent().unwrap_or(&root).to_string_lossy().to_string();

    // Kill old processes
    let _ = Command::new("killall").args(["-9", "wineserver", "winedevice"]).output();
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Patch Steam DRM to run offline
    patch_game_for_offline(&exe, &app_handle).ok();

    let _ = app_handle.emit("steam:install-progress", format!("Launching: {}", exe.file_name().unwrap_or_default().to_string_lossy()));

    let mut child = Command::new(&wine)
        .env("WINEPREFIX", &prefix)
        .env("WINEDEBUG", "-all")
        .env("MVK_CONFIG_LOG_LEVEL", "0")
        .env("WINEDLLOVERRIDES", "dxgi=n,b;d3d11=n,b;d3d10=n,b;d3d9=n,b")
        .current_dir(&exe_dir)
        .arg(&exe.to_string_lossy().to_string())
        .spawn()
        .map_err(|e| format!("Failed to launch: {e}"))?;

    let pid = child.id();
    // Register with process supervisor so the UI can track it
    {
        let mut sup = state.process_supervisor.lock().unwrap();
        sup.register(crate::process_supervisor::GameProcess {
            pid,
            bottle_id: "steam".into(),
            steam_app_id: None,
            command: exe.to_string_lossy().to_string(),
            started_at: chrono::Utc::now(),
            status: crate::process_supervisor::ProcessStatus::Running,
            exit_code: None,
        });
    }

    let _ = app_handle.emit("steam:install-progress", "Game launched -- check your screen.");
    Ok(())
}

// ── Steam GUI ──────────────────────────────────────────────

#[tauri::command] pub async fn steam_status(state: tauri::State<'_, AppState>) -> Result<crate::steam_bridge::SteamStatus, String> { Ok(state.steam_bridge.get_status()) }
#[tauri::command] pub async fn steam_list_games(state: tauri::State<'_, AppState>) -> Result<Vec<SteamApp>, String> { state.steam_bridge.list_installed_games() }

#[tauri::command]
pub async fn steam_install(setup_path: String, app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let sb = state.steam_bridge.steam_bottle_path();
    if !sb.exists() { state.bottle_manager.create_bottle("Steam", "9.14-staging", None).await?; }
    let wine = state.wine_manager.resolve_wine_bin("9.14-staging")?;
    let p = sb.to_string_lossy().to_string();
    let o = Command::new(&wine).env("WINEPREFIX", &p).arg(&setup_path).arg("/S").output().map_err(|e| e.to_string())?;
    if !o.status.success() { return Err(String::from_utf8_lossy(&o.stderr).to_string()); }
    let _ = app.emit("steam:installed", ());
    Ok(())
}

#[tauri::command]
pub async fn steam_run_client(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let exe = state.steam_bridge.steam_exe_path();
    if !exe.exists() { return Err("Steam GUI not installed".into()); }
    let wine = state.wine_manager.resolve_wine_bin("9.14-staging")?;
    let p = state.steam_bridge.steam_bottle_path().to_string_lossy().to_string();
    Command::new(&wine).env("WINEPREFIX", &p).arg("explorer").arg("/desktop=Steam,1280x800").arg(&exe.to_string_lossy().to_string()).arg("-no-browser").arg("-no-cef-sandbox").arg("-cef-disable-gpu").arg("-cef-disable-gpu-compositing").spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn steam_launch_game(app_id: String, _bottle_id: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let games = state.steam_bridge.list_installed_games()?;
    let g = games.iter().find(|g| g.app_id == app_id).ok_or_else(|| format!("App {app_id} not found"))?;
    let exe = if g.install_dir.extension().map_or(false, |e| e == "exe") { g.install_dir.clone() } else { find_exe(&g.install_dir)? };
    let wine = state.wine_manager.resolve_wine_bin("9.14-staging")?;
    let p = state.steam_bridge.steam_bottle_path().to_string_lossy().to_string();
    let root = std::path::PathBuf::from("/");
    let dir = exe.parent().unwrap_or(&root).to_string_lossy().to_string();
    // Kill old, start Steam
    let _ = Command::new("killall").args(["-9", "wineserver", "winedevice"]).output();
    std::thread::sleep(std::time::Duration::from_secs(1));
    let sr = state.steam_bridge.steam_exe_path();
    if sr.exists() {
        Command::new(&wine).env("WINEPREFIX", &p).env("WINEDEBUG", "-all").env("MVK_CONFIG_LOG_LEVEL", "0").arg(&sr.to_string_lossy().to_string()).arg("-silent").arg("-no-browser").spawn().ok();
        std::thread::sleep(std::time::Duration::from_secs(15));
    }
    Command::new(&wine).env("WINEPREFIX", &p).env("WINEDEBUG", "-all").env("MVK_CONFIG_LOG_LEVEL", "0").env("WINEDLLOVERRIDES", "dxgi=n,b;d3d11=n,b;d3d10=n,b;d3d9=n,b").current_dir(&dir).arg(&exe.to_string_lossy().to_string()).spawn().map_err(|e| e.to_string())?;
    Ok(format!("launched:{app_id}"))
}

fn find_exe(dir: &std::path::PathBuf) -> Result<std::path::PathBuf, String> {
    for e in walkdir::WalkDir::new(dir).max_depth(3).into_iter().flatten() {
        if e.path().extension().map_or(false, |x| x == "exe") {
            let n = e.path().file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            if !n.contains("uninstall") && !n.contains("unins") && n != "steam.exe" { return Ok(e.path().to_path_buf()); }
        }
    }
    Err(format!("No exe in {}", dir.display()))
}
