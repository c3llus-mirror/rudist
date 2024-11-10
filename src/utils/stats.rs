#[derive(Default)]
pub struct Stats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub total_commands: usize,
    pub total_memory: usize,
}