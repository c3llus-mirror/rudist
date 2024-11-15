mod executor;
mod router;

pub use executor::Executor;
pub use router::Router;

use crate::protocol::resp::types::RESPType;

pub struct Engine {
    router: Router,
    executor: Executor,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            executor: Executor::new(),
        }
    }

    pub fn process_command(&mut self, command: RESPType) -> Result<String, String> {
        let cmd = self.router.route(&command).map_err(|e| e.to_string())?;
        self.executor.execute(cmd).map_err(|e| e.to_string())
    }

    pub fn active_expire_cycle_fast(&mut self) {
        self.executor.active_expire_cycle_fast();
    }

    pub fn active_expire_cycle_slow(&mut self) {
        self.executor.active_expire_cycle_slow();
    }
}