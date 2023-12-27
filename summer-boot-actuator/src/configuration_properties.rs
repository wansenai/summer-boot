//!
//! Configuration properties
//! 
pub struct ConfigurationProperties {
    pub keys_sanitize: Vec<String>,
    pub additional_keys_sanitize: Vec<String>,
}

impl ConfigurationProperties {
    pub fn new() -> ConfigurationProperties {
        ConfigurationProperties {
            keys_sanitize: Vec::new(),
            additional_keys_sanitize: Vec::new(),
        }
    }

    pub fn get_keys_sanitize(&self) -> &Vec<String> {
        &self.keys_sanitize
    }

    pub fn set_keys_sanitize(&mut self, keys_sanitize: Vec<String>) {
        self.keys_sanitize = keys_sanitize;
    }

    pub fn get_additional_keys_sanitize(&self) -> &Vec<String> {
        &self.additional_keys_sanitize
    }

    pub fn set_additional_keys_sanitize(&mut self, additional_keys_sanitize: Vec<String>) {
        self.additional_keys_sanitize = additional_keys_sanitize;
    }
}
