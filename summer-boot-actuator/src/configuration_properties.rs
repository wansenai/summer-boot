//!
//! Configuration properties
//! 
pub struct ConfigurationProperties {
    pub keys_sanitize : Vec<String>,
    pub additional_keys_sanitize : Vec<String>,
}

impl ConfigurationProperties {

    pub fn getKeysSanitize() -> Vec<String> {
        ConfigurationProperties.keys_sanitize
    }

    pub fn setKeysSanitize(keys_sanitize: Vec<String>) {
        ConfigurationProperties.keys_sanitize = keys_sanitize;
    }

    pub fn getAdditionalKeysSanitize() -> Vec<String> {
        ConfigurationProperties.additional_keys_sanitize
    }

    pub fn setAdditionalKeysSanitize(additional_keys_sanitize: Vec<String>) {
        ConfigurationProperties.additional_keys_sanitize = additional_keys_sanitize;
    }
}