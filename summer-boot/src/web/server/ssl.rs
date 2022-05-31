use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum ClientAuth {
    /**
     * Client authentication is not wanted
     */
    NONE,
    /**
     * Client authentication is wanted but not mandatory.
     */
    WANT,
    /**
     * Client authentication is needed and mandatory.
     */
    NEED,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ssl {

    enabled: Option<bool>,

    ciphers: Vec<String>,

    client_auth: ClientAuth,

    enabled_protocols: Vec<String>,

    key_alias: Option<String>,

    key_passowrd: Option<String>,

    key_store: Option<String>,

    key_store_password: Option<String>,

    key_store_type: Option<String>,

    trust_store: Option<String>,

    trust_store_password: Option<String>,

    trust_store_type: Option<String>,

    trust_store_provider: Option<String>,

    certificate: Option<String>,

    certificate_private_key: Option<String>,

    trust_certificate: Option<String>,

    trust_certificate_private_key: Option<String>,

    protocol: Option<String>,
}

impl Ssl {

    pub(crate) fn new(ssl_config: Ssl) -> Self {
        Ssl {
            enabled:                            Some(true),
            protocol:                           Some(String::from("TLS")),
            ciphers:                            ssl_config.ciphers,
            client_auth:                        ssl_config.client_auth,
            enabled_protocols:                  ssl_config.enabled_protocols,
            key_alias:                          ssl_config.key_alias,
            key_passowrd:                       ssl_config.key_passowrd,
            key_store:                          ssl_config.key_store,                      
            key_store_password:                 ssl_config.key_store_password,
            key_store_type:                     ssl_config.key_store_type,
            trust_store:                        ssl_config.trust_store,
            trust_store_password:               ssl_config.trust_store_password,
            trust_store_type:                   ssl_config.trust_store_type,
            trust_store_provider:               ssl_config.trust_store_provider,
            certificate:                        ssl_config.certificate,
            certificate_private_key:            ssl_config.certificate_private_key,
            trust_certificate:                  ssl_config.trust_certificate,
            trust_certificate_private_key:      ssl_config.trust_certificate_private_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssl_config() {
        let ssl_config = Ssl {
            enabled: Some(false),
            protocol: Some(String::from("TLS")),
            ciphers:  Vec::new(),
            client_auth:  ClientAuth::NONE,
            enabled_protocols: Vec::new(),
            key_alias: None,
            key_passowrd: None,
            key_store: None,
            key_store_password: None,
            key_store_type: None,
            trust_store: None,
            trust_store_password: None,
            trust_store_type: None,
            trust_store_provider: None,
            certificate: None,
            certificate_private_key: None,
            trust_certificate: None,
            trust_certificate_private_key: None,
        };

        println!("ssl config : {:?}", Ssl::new(ssl_config));
    }
}
