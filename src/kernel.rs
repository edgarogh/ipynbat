use serde::de::IgnoredAny;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// Specification: https://jupyter-client.readthedocs.io/en/stable/kernels.html#kernel-specs
#[derive(Debug, serde::Deserialize)]
pub struct KernelSpec {
    pub argv: Vec<String>,

    pub display_name: String,

    pub language: String,

    #[serde(default)]
    pub interrupt_mode: InterruptMode,

    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub metadata: Option<IgnoredAny>,
}

#[derive(Debug, serde::Deserialize)]
pub enum InterruptMode {
    #[serde(rename = "signal")]
    Signal,

    #[serde(rename = "message")]
    Message,
}

impl Default for InterruptMode {
    fn default() -> Self {
        Self::Signal
    }
}

impl Display for KernelSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)
    }
}

impl KernelSpec {
    #[cfg(target_os = "linux")]
    pub fn directories() -> [PathBuf; 1] {
        [dirs::home_dir()
            .unwrap()
            .join(".local/share/jupyter/kernels")] // TODO
    }

    pub fn find(name: &str) -> KernelSpec {
        let kernel_path = dirs::home_dir()
            .unwrap()
            .join(".local/share/jupyter/kernels")
            .join(name)
            .join("kernel.json");

        serde_json::from_str::<KernelSpec>(&std::fs::read_to_string(kernel_path).unwrap()).unwrap()
    }
}
