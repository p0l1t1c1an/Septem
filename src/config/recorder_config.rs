use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct RecorderConfig {
    write_delay: u64,
    productive: Vec<String>,
}

impl RecorderConfig {
    pub fn productive(&self) -> &Vec<String> {
        &self.productive
    }

    pub fn write_delay(&self) -> u64 {
        self.write_delay
    }
}
