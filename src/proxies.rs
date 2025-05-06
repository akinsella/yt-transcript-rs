use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

/// Error type for invalid proxy configurations
#[derive(Debug, thiserror::Error)]
#[error("Invalid proxy configuration: {0}")]
pub struct InvalidProxyConfig(pub String);

/// Trait for proxy configurations
pub trait ProxyConfig: Debug + Send + Sync {
    /// Convert to a reqwest-compatible proxy HashMap
    fn to_requests_dict(&self) -> HashMap<String, String>;

    /// Should we prevent keeping HTTP connections alive?
    /// This is useful for rotating proxies to ensure you get a new IP for each request.
    fn prevent_keeping_connections_alive(&self) -> bool {
        false
    }

    /// How many retries should we attempt when a request is blocked?
    /// When using rotating residential proxies, this allows retrying with different IPs.
    fn retries_when_blocked(&self) -> i32 {
        0
    }

    /// Type conversion for dynamic dispatch
    fn as_any(&self) -> &dyn Any;
}

/// A generic proxy configuration for HTTP/HTTPS/SOCKS proxies
#[derive(Debug, Clone)]
pub struct GenericProxyConfig {
    pub http_url: Option<String>,
    pub https_url: Option<String>,
}

impl GenericProxyConfig {
    pub fn new(
        http_url: Option<String>,
        https_url: Option<String>,
    ) -> Result<Self, InvalidProxyConfig> {
        if http_url.is_none() && https_url.is_none() {
            return Err(InvalidProxyConfig(
                "GenericProxyConfig requires you to define at least one of the two: http or https"
                    .to_string(),
            ));
        }

        Ok(Self {
            http_url,
            https_url,
        })
    }
}

impl ProxyConfig for GenericProxyConfig {
    fn to_requests_dict(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        let http = match &self.http_url {
            Some(url) => url.clone(),
            None => self.https_url.clone().unwrap_or_default(),
        };

        let https = match &self.https_url {
            Some(url) => url.clone(),
            None => self.http_url.clone().unwrap_or_default(),
        };

        map.insert("http".to_string(), http);
        map.insert("https".to_string(), https);

        map
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Webshare proxy configuration for rotating residential proxies
#[derive(Debug, Clone)]
pub struct WebshareProxyConfig {
    pub proxy_username: String,
    pub proxy_password: String,
    pub domain_name: String,
    pub proxy_port: u16,
    pub retries: i32,
}

impl WebshareProxyConfig {
    pub const DEFAULT_DOMAIN_NAME: &'static str = "p.webshare.io";
    pub const DEFAULT_PORT: u16 = 80;

    pub fn new(
        proxy_username: String,
        proxy_password: String,
        retries_when_blocked: i32,
        domain_name: Option<String>,
        proxy_port: Option<u16>,
    ) -> Self {
        Self {
            proxy_username,
            proxy_password,
            domain_name: domain_name.unwrap_or_else(|| Self::DEFAULT_DOMAIN_NAME.to_string()),
            proxy_port: proxy_port.unwrap_or(Self::DEFAULT_PORT),
            retries: retries_when_blocked,
        }
    }

    pub fn url(&self) -> String {
        format!(
            "http://{}-rotate:{}@{}:{}/",
            self.proxy_username, self.proxy_password, self.domain_name, self.proxy_port
        )
    }
}

impl ProxyConfig for WebshareProxyConfig {
    fn to_requests_dict(&self) -> HashMap<String, String> {
        let url = self.url();
        let mut map = HashMap::new();

        map.insert("http".to_string(), url.clone());
        map.insert("https".to_string(), url);

        map
    }

    fn prevent_keeping_connections_alive(&self) -> bool {
        true
    }

    fn retries_when_blocked(&self) -> i32 {
        self.retries
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
