use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Adapter {
    /// GPU name, e.g. `"Apple M1 Pro"` or `"NVIDIA GeForce RTX 4090"`
    pub name: String,
    /// Device class: one of `"discrete_gpu"`, `"integrated_gpu"`, `"virtual_gpu"`, `"cpu"` (software), `"other"`
    pub device_type: &'static str,
    /// Graphics API: one of `"vulkan"`, `"metal"`, `"dx12"`, `"gl"`, `"browser_webgpu"`, `"noop"`
    pub backend: &'static str,
}

#[cfg(feature = "wgpu")]
impl From<&wgpu_types::AdapterInfo> for Adapter {
    fn from(info: &wgpu_types::AdapterInfo) -> Self {
        Self {
            name: info.name.clone(),
            device_type: match info.device_type {
                wgpu_types::DeviceType::Other => "other",
                wgpu_types::DeviceType::IntegratedGpu => "integrated_gpu",
                wgpu_types::DeviceType::DiscreteGpu => "discrete_gpu",
                wgpu_types::DeviceType::VirtualGpu => "virtual_gpu",
                wgpu_types::DeviceType::Cpu => "cpu",
            },
            backend: info.backend.to_str(),
        }
    }
}
