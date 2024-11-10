use crate::protocol::resp::types::RESPType;
use crate::engine::Engine;
use crate::protocol::resp::parser::parse_resp;

pub struct Handler{
    engine: Engine
}

impl Handler {
    pub fn new(
        engine: Engine,
    ) -> Self {
        Self {
            engine: engine,
        }
    }
}
impl Handler {
    pub fn handle(&mut self, buffer: &[u8]) -> RESPType {
        let resp: RESPType = match parse_resp(buffer) {
            Ok((resp, _)) => resp,
            Err(e) => return RESPType::Error(e)
        };

        match self.engine.process_command(resp) {
            Ok(result) => RESPType::SimpleString(result),
            Err(e) => RESPType::Error(e)
        }
    }
}