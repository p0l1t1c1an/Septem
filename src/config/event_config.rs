use serde_derive::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct EventConfig {
    delay: u64,
}

impl Default for EventConfig {
    fn default() -> Self {
        Self { delay: 500 }
    }
}

impl EventConfig {
    pub fn delay(&self) -> u64 {
        self.delay
    }
}
