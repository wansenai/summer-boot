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
	NEED
}

pub struct Ssl {

    enabled: Option<bool>,

    ciphers: Vec<String>,

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
            enabled: true,
            protocol: String::from("TLS"),
            ciphers,
            enabled_protocols,
            key_alias,
            key_passowrd,
            key_store,
            key_store_password,
            key_store_type,
            trust_store,
            trust_store_password,
            trust_store_type,
            trust_store_provider,
            certificate,
            certificate_private_key,
            trust_certificate,
            trust_certificate_private_key,
        }
    }
}