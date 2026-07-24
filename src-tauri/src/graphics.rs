use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::{AppConfig, GraphicsBackend};

/// Describes an available graphics translation backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsBackendInfo {
    pub backend: GraphicsBackend,
    pub name: String,
    pub description: String,
    pub installed: bool,
    pub dll_paths: Vec<PathBuf>,
}

pub struct GraphicsManager {
    config: AppConfig,
}

impl GraphicsManager {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// List all detected graphics backends with their installation status.
    pub fn detect_backends(&self) -> Vec<GraphicsBackendInfo> {
        let runtimes = &self.config.runtimes_dir;

        vec![
            GraphicsBackendInfo {
                installed: self.check_d3dmetal(&runtimes),
                description:
                    "DirectX 11/12 → Metal. Best performance on Apple Silicon. \
                     Part of Apple's Game Porting Toolkit."
                        .into(),
                name: "D3DMetal (Apple GPTK)".into(),
                backend: GraphicsBackend::D3DMetal,
                dll_paths: vec![
                    runtimes.join("d3dmetal").join("libdxgi.dylib"),
                    runtimes.join("d3dmetal").join("libd3d11.dylib"),
                    runtimes.join("d3dmetal").join("libd3d12.dylib"),
                ],
            },
            GraphicsBackendInfo {
                installed: self.check_dxvk(&runtimes) && self.check_moltenvk(&runtimes),
                description:
                    "DirectX 9/10/11 → Vulkan → Metal. Fully open source, \
                     good D3D9/10/11 compatibility."
                        .into(),
                name: "DXVK + MoltenVK".into(),
                backend: GraphicsBackend::DXVK,
                dll_paths: vec![
                    runtimes.join("dxvk").join("x64").join("dxgi.dll"),
                    runtimes.join("dxvk").join("x64").join("d3d11.dll"),
                    runtimes.join("dxvk").join("x64").join("d3d10.dll"),
                    runtimes.join("dxvk").join("x64").join("d3d9.dll"),
                ],
            },
            GraphicsBackendInfo {
                installed: true, // WineD3D is built into Wine.
                description:
                    "Wine's built-in DirectX → OpenGL translation. \
                     Always available but slow on macOS (OpenGL 4.1 limit)."
                        .into(),
                name: "WineD3D (built-in)".into(),
                backend: GraphicsBackend::WineD3D,
                dll_paths: vec![],
            },
        ]
    }

    /// Determine the best available backend for a given DirectX feature level.
    pub fn best_backend_for_d3d(&self, d3d_version: u8) -> GraphicsBackend {
        let backends = self.detect_backends();

        match d3d_version {
            12 => {
                if backends[0].installed {
                    GraphicsBackend::D3DMetal
                } else if backends[1].installed {
                    GraphicsBackend::DXVK
                } else {
                    GraphicsBackend::WineD3D
                }
            }
            11 | 10 => {
                if backends[0].installed {
                    GraphicsBackend::D3DMetal
                } else if backends[1].installed {
                    GraphicsBackend::DXVK
                } else {
                    GraphicsBackend::WineD3D
                }
            }
            9 => {
                if backends[1].installed {
                    GraphicsBackend::DXVK
                } else {
                    GraphicsBackend::WineD3D
                }
            }
            _ => GraphicsBackend::WineD3D,
        }
    }

    /// Build the WINEDLLOVERRIDES string for a given backend configuration.
    pub fn build_dll_overrides(
        &self,
        backend: GraphicsBackend,
        overrides: &DllOverrideConfig,
    ) -> String {
        let mut parts = Vec::new();

        match backend {
            GraphicsBackend::D3DMetal | GraphicsBackend::DXVK => {
                if overrides.dxgi {
                    parts.push("dxgi=n,b");
                }
                if overrides.d3d10 {
                    parts.push("d3d10=n,b");
                }
                if overrides.d3d11 {
                    parts.push("d3d11=n,b");
                }
                if overrides.d3d12 {
                    parts.push("d3d12=n,b");
                    parts.push("d3d12core=n,b");
                }
                if overrides.d3d9 {
                    parts.push("d3d9=n,b");
                }
            }
            GraphicsBackend::WineD3D => {
                // Use Wine's built-in DLLs.
                parts.push("dxgi=b;d3d11=b;d3d10=b;d3d9=b");
            }
        }

        if parts.is_empty() {
            "".into()
        } else {
            parts.join(";")
        }
    }

    fn check_d3dmetal(&self, runtimes: &PathBuf) -> bool {
        runtimes.join("d3dmetal").join("libdxgi.dylib").exists()
            && runtimes.join("d3dmetal").join("libd3d12.dylib").exists()
    }

    fn check_dxvk(&self, runtimes: &PathBuf) -> bool {
        runtimes.join("dxvk").join("x64").join("dxgi.dll").exists()
    }

    fn check_moltenvk(&self, runtimes: &PathBuf) -> bool {
        runtimes
            .join("moltenvk")
            .join("libMoltenVK.dylib")
            .exists()
    }
}

/// Which DLLs to override as native.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DllOverrideConfig {
    pub dxgi: bool,
    pub d3d9: bool,
    pub d3d10: bool,
    pub d3d11: bool,
    pub d3d12: bool,
}

impl Default for DllOverrideConfig {
    fn default() -> Self {
        Self {
            dxgi: true,
            d3d9: false,
            d3d10: true,
            d3d11: true,
            d3d12: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_backends_always_finds_wine3d() {
        let config = AppConfig::default();
        let mgr = GraphicsManager::new(config);
        let backends = mgr.detect_backends();
        assert_eq!(backends.len(), 3);
        // WineD3D is always installed.
        assert!(backends[2].installed);
    }

    #[test]
    fn test_best_backend_fallback() {
        let config = AppConfig::default();
        let mgr = GraphicsManager::new(config);
        // Without D3DMetal or DXVK installed, should fall back to WineD3D.
        assert_eq!(mgr.best_backend_for_d3d(12), GraphicsBackend::WineD3D);
        assert_eq!(mgr.best_backend_for_d3d(9), GraphicsBackend::WineD3D);
    }

    #[test]
    fn test_dll_overrides() {
        let config = AppConfig::default();
        let mgr = GraphicsManager::new(config);
        let overrides = mgr.build_dll_overrides(
            GraphicsBackend::D3DMetal,
            &DllOverrideConfig::default(),
        );
        assert!(overrides.contains("dxgi=n,b"));
        assert!(overrides.contains("d3d12=n,b"));
        // d3d9 should not be in overrides by default.
        assert!(!overrides.contains("d3d9=n,b"));
    }
}
