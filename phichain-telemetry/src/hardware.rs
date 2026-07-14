use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Hardware {
    /// CPU brand string, e.g. `"Apple M1 Pro"` or `"Intel(R) Core(TM) i7-..."`
    pub cpu: String,
    /// Logical core count
    pub core_count: usize,
    /// Total installed RAM in bytes
    pub memory: u64,
}

impl Hardware {
    /// Collect hardware info from the current host.
    ///
    /// Internally builds a fresh `sysinfo::System` with a full refresh, which is not cheap (tens of ms on a cold cache).
    /// Call once per session and cache the result.
    pub fn collect() -> Self {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();
        let cpu = system
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default();
        Self {
            cpu,
            core_count: system.cpus().len(),
            memory: system.total_memory(),
        }
    }
}
