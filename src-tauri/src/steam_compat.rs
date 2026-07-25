use std::fs;
use std::path::{Path, PathBuf};

const SHIM_MARKER: &[u8] = b"GameRunner SteamWebHelper Compatibility Shim v1";
const ORIGINAL_NAME: &str = "steamwebhelper.gamerunner-original.exe";
const STEAM_UPDATE_POLICY: &str =
    "BootStrapperInhibitAll=enable\nBootStrapperForceSelfUpdate=disable\n";

pub fn resolve_shim(resource_dir: Option<&Path>) -> Result<PathBuf, String> {
    let development =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../runtime/dist/steamwebhelper-shim.exe");
    let candidates = resource_dir
        .map(|dir| dir.join("runtime/steamwebhelper-shim.exe"))
        .into_iter()
        .chain(std::iter::once(development));

    candidates
        .filter(|path| path.is_file())
        .find(|path| file_contains_marker(path).unwrap_or(false))
        .ok_or_else(|| {
            "Steam compatibility shim is missing. Rebuild Budu with \
             `bash scripts/build-steam-shim.sh`."
                .into()
        })
}

pub fn install_for_steam(steam_executable: &Path, shim: &Path) -> Result<PathBuf, String> {
    if !steam_executable.is_file() {
        return Err(format!(
            "Steam executable not found: {}",
            steam_executable.display()
        ));
    }
    if !file_contains_marker(shim)? {
        return Err(format!(
            "Invalid Steam compatibility shim: {}",
            shim.display()
        ));
    }

    let steam_dir = steam_executable
        .parent()
        .ok_or_else(|| "Steam executable has no parent directory".to_string())?;
    install_update_policy(steam_dir)?;
    let helper = [
        steam_dir.join("bin/cef/cef.win64/steamwebhelper.exe"),
        steam_dir.join("bin/cef/cef.win7x64/steamwebhelper.exe"),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .ok_or_else(|| format!("SteamWebHelper was not found below {}", steam_dir.display()))?;
    let backup = helper.with_file_name(ORIGINAL_NAME);

    if file_contains_marker(&helper)? {
        if backup.is_file() {
            if fs::read(&helper)
                .map_err(|error| format!("Failed to read {}: {error}", helper.display()))?
                != fs::read(shim)
                    .map_err(|error| format!("Failed to read {}: {error}", shim.display()))?
            {
                atomic_copy(shim, &helper)?;
            }
            return Ok(helper);
        }
        return Err(format!(
            "SteamWebHelper is patched but its original backup is missing: {}",
            backup.display()
        ));
    }

    atomic_copy(&helper, &backup)?;
    atomic_copy(shim, &helper)?;
    Ok(helper)
}

fn install_update_policy(steam_dir: &Path) -> Result<(), String> {
    let policy = steam_dir.join("steam.cfg");
    if fs::read_to_string(&policy).ok().as_deref() == Some(STEAM_UPDATE_POLICY) {
        return Ok(());
    }

    let temporary = policy.with_extension(format!("gamerunner-tmp-{}", std::process::id()));
    fs::write(&temporary, STEAM_UPDATE_POLICY)
        .map_err(|error| format!("Failed to stage {}: {error}", policy.display()))?;
    fs::rename(&temporary, &policy).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("Failed to replace {}: {error}", policy.display())
    })
}

fn file_contains_marker(path: &Path) -> Result<bool, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
    Ok(bytes
        .windows(SHIM_MARKER.len())
        .any(|window| window == SHIM_MARKER))
}

fn atomic_copy(source: &Path, destination: &Path) -> Result<(), String> {
    let temporary = destination.with_extension(format!("gamerunner-tmp-{}", std::process::id()));
    fs::copy(source, &temporary).map_err(|error| {
        format!(
            "Failed to stage {} from {}: {error}",
            destination.display(),
            source.display()
        )
    })?;
    fs::rename(&temporary, destination).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("Failed to replace {}: {error}", destination.display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_shim(path: &Path) {
        fs::write(path, [b"MZ".as_slice(), SHIM_MARKER].concat()).unwrap();
    }

    #[test]
    fn installs_and_preserves_original_helper() {
        let tmp = TempDir::new().unwrap();
        let steam = tmp.path().join("Steam/steam.exe");
        let helper = tmp
            .path()
            .join("Steam/bin/cef/cef.win64/steamwebhelper.exe");
        let shim = tmp.path().join("shim.exe");
        fs::create_dir_all(helper.parent().unwrap()).unwrap();
        fs::write(&steam, b"steam").unwrap();
        fs::write(&helper, b"original-v1").unwrap();
        write_shim(&shim);

        assert_eq!(install_for_steam(&steam, &shim).unwrap(), helper);
        assert!(file_contains_marker(&helper).unwrap());
        assert_eq!(
            fs::read(helper.with_file_name(ORIGINAL_NAME)).unwrap(),
            b"original-v1"
        );
        assert_eq!(
            fs::read_to_string(tmp.path().join("Steam/steam.cfg")).unwrap(),
            STEAM_UPDATE_POLICY
        );
    }

    #[test]
    fn refreshes_backup_after_a_steam_update() {
        let tmp = TempDir::new().unwrap();
        let steam = tmp.path().join("Steam/steam.exe");
        let helper = tmp
            .path()
            .join("Steam/bin/cef/cef.win64/steamwebhelper.exe");
        let shim = tmp.path().join("shim.exe");
        fs::create_dir_all(helper.parent().unwrap()).unwrap();
        fs::write(&steam, b"steam").unwrap();
        fs::write(&helper, b"original-v1").unwrap();
        write_shim(&shim);
        install_for_steam(&steam, &shim).unwrap();

        fs::write(&helper, b"original-v2").unwrap();
        install_for_steam(&steam, &shim).unwrap();

        assert!(file_contains_marker(&helper).unwrap());
        assert_eq!(
            fs::read(helper.with_file_name(ORIGINAL_NAME)).unwrap(),
            b"original-v2"
        );
    }

    #[test]
    fn refreshes_an_outdated_shim_without_touching_the_original() {
        let tmp = TempDir::new().unwrap();
        let steam = tmp.path().join("Steam/steam.exe");
        let helper = tmp
            .path()
            .join("Steam/bin/cef/cef.win64/steamwebhelper.exe");
        let shim = tmp.path().join("shim.exe");
        fs::create_dir_all(helper.parent().unwrap()).unwrap();
        fs::write(&steam, b"steam").unwrap();
        fs::write(&helper, [b"MZ-old".as_slice(), SHIM_MARKER].concat()).unwrap();
        fs::write(helper.with_file_name(ORIGINAL_NAME), b"original-helper").unwrap();
        write_shim(&shim);

        install_for_steam(&steam, &shim).unwrap();

        assert_eq!(fs::read(&helper).unwrap(), fs::read(&shim).unwrap());
        assert_eq!(
            fs::read(helper.with_file_name(ORIGINAL_NAME)).unwrap(),
            b"original-helper"
        );
    }

    #[test]
    fn rejects_unmarked_shim() {
        let tmp = TempDir::new().unwrap();
        let steam = tmp.path().join("Steam/steam.exe");
        let helper = tmp
            .path()
            .join("Steam/bin/cef/cef.win64/steamwebhelper.exe");
        let shim = tmp.path().join("shim.exe");
        fs::create_dir_all(helper.parent().unwrap()).unwrap();
        fs::write(&steam, b"steam").unwrap();
        fs::write(&helper, b"original").unwrap();
        fs::write(&shim, b"not a shim").unwrap();

        assert!(install_for_steam(&steam, &shim)
            .unwrap_err()
            .contains("Invalid"));
    }

    #[test]
    fn resolves_packaged_shim_before_development_fallback() {
        let tmp = TempDir::new().unwrap();
        let packaged = tmp.path().join("runtime/steamwebhelper-shim.exe");
        fs::create_dir_all(packaged.parent().unwrap()).unwrap();
        write_shim(&packaged);

        assert_eq!(resolve_shim(Some(tmp.path())).unwrap(), packaged);
    }

    #[test]
    fn repairs_the_steam_update_policy() {
        let tmp = TempDir::new().unwrap();
        let steam_dir = tmp.path().join("Steam");
        fs::create_dir_all(&steam_dir).unwrap();
        fs::write(steam_dir.join("steam.cfg"), "stale").unwrap();

        install_update_policy(&steam_dir).unwrap();

        assert_eq!(
            fs::read_to_string(steam_dir.join("steam.cfg")).unwrap(),
            STEAM_UPDATE_POLICY
        );
    }
}
