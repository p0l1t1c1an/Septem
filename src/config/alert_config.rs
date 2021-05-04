
use serde_derive::Deserialize;

// Todo: Add Enum and alert type for config
// It can be a pop up message or play audio
// Rn, I will just make it println! a message

#[derive(Clone, Deserialize, Debug)]
pub struct AlertConfig {
    delay: u64,
    productive_time: f64,
    unproductive_time: f64,
    message: String,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            delay: 500,
            productive_time: 5.0,    // Resets at 5 minutes
            unproductive_time: 20.0, // Prints message at 5 minutes
            message: "You have been wasting time.\nPlease start being productive.".to_owned(),
        }
    }
}

impl AlertConfig {
    pub fn delay(&self) -> u64 {
        self.delay
    }

    pub fn productive_time(&self) -> f64 {
        self.productive_time
    }

    pub fn unproductive_time(&self) -> f64 {
        self.unproductive_time
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

