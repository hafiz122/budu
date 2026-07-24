use std::path::PathBuf;

use crate::config::AppConfig;

/// Known runtime dependency that can be installed into a Wine bottle.
#[derive(Debug, Clone)]
pub struct Runtime {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: RuntimeCategory,
    /// Paths cached globally, relative to runtimes_dir.
    pub cached_path: Option<PathBuf>,
    /// Whether this runtime is cached locally.
    pub cached: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCategory {
    /// Microsoft Visual C++ Redistributable.
    VCRun,
    /// .NET Framework.
    DotNet,
    /// DirectX runtime DLLs.
    DirectX,
    /// Translation layer (DXVK, MoltenVK, D3DMetal).
    Graphics,
    /// Other system components.
    System,
}

impl RuntimeCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::VCRun => "vcrun",
            Self::DotNet => "dotnet",
            Self::DirectX => "directx",
            Self::Graphics => "graphics",
            Self::System => "system",
        }
    }
}

pub struct RuntimeManager {
    config: AppConfig,
}

impl RuntimeManager {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// List all known runtimes and their cache status.
    pub fn list_runtimes(&self) -> Vec<Runtime> {
        let runtimes_dir = &self.config.runtimes_dir;

        vec![
            Runtime {
                id: "vcrun2022".into(),
                name: "Visual C++ 2015-2022".into(),
                description: "Required by most modern Windows games.".into(),
                category: RuntimeCategory::VCRun,
                cached: runtimes_dir.join("vcrun2022").exists(),
                cached_path: Some(runtimes_dir.join("vcrun2022")),
            },
            Runtime {
                id: "vcrun2019".into(),
                name: "Visual C++ 2019".into(),
                description: "Required by older games.".into(),
                category: RuntimeCategory::VCRun,
                cached: runtimes_dir.join("vcrun2019").exists(),
                cached_path: Some(runtimes_dir.join("vcrun2019")),
            },
            Runtime {
                id: "dotnet48".into(),
                name: ".NET Framework 4.8".into(),
                description: "Required by .NET-based games and tools.".into(),
                category: RuntimeCategory::DotNet,
                cached: runtimes_dir.join("dotnet48").exists(),
                cached_path: Some(runtimes_dir.join("dotnet48")),
            },
            Runtime {
                id: "directx_jun2010".into(),
                name: "DirectX June 2010".into(),
                description: "Legacy DirectX 9/10/11 DLLs not shipped with Wine.".into(),
                category: RuntimeCategory::DirectX,
                cached: runtimes_dir.join("directx_jun2010").exists(),
                cached_path: Some(runtimes_dir.join("directx_jun2010")),
            },
        ]
    }

    /// Check whether a specific runtime is cached locally.
    pub fn is_cached(&self, runtime_id: &str) -> bool {
        self.config.runtimes_dir.join(runtime_id).exists()
    }

    /// Symlink cached runtime DLLs into a bottle's system32 directory.
    pub fn install_to_bottle(
        &self,
        bottle_id: &str,
        runtime_id: &str,
    ) -> Result<(), String> {
        let runtime_path = self
            .config
            .runtimes_dir
            .join(runtime_id);
        if !runtime_path.exists() {
            return Err(format!(
                "Runtime '{runtime_id}' is not cached. Download it first."
            ));
        }

        let system32 = self
            .config
            .bottles_dir
            .join(bottle_id)
            .join("drive_c")
            .join("windows")
            .join("system32");

        std::fs::create_dir_all(&system32)
            .map_err(|e| format!("Failed to create system32: {e}"))?;

        // Symlink each DLL from the runtime cache into system32.
        for entry in
            std::fs::read_dir(&runtime_path).map_err(|e| format!("Read runtime dir: {e}"))?
        {
            let entry = entry.map_err(|e| format!("Dir entry: {e}"))?;
            let src = entry.path();
            let dst = system32.join(
                src.file_name()
                    .ok_or_else(|| "Invalid filename".to_string())?,
            );

            // Skip if already present.
            if dst.exists() {
                continue;
            }

            #[cfg(target_os = "macos")]
            std::os::unix::fs::symlink(&src, &dst)
                .map_err(|e| format!("Failed to symlink DLL: {e}"))?;

            #[cfg(not(target_os = "macos"))]
            std::fs::copy(&src, &dst)
                .map_err(|e| format!("Failed to copy DLL: {e}"))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_runtimes() {
        let config = AppConfig::default();
        let mgr = RuntimeManager::new(config);
        let runtimes = mgr.list_runtimes();
        assert_eq!(runtimes.len(), 4);
    }
}
