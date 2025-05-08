use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

/// # InvalidProxyConfig
///
/// Error type for invalid proxy configurations.
///
/// This error is returned when a proxy configuration is deemed invalid,
/// such as when required fields are missing or values are not in the expected format.
#[derive(Debug, thiserror::Error)]
#[error("Invalid proxy configuration: {0}")]
pub struct InvalidProxyConfig(pub String);

/// # ProxyConfig
///
/// Trait for defining proxy configurations to route YouTube requests through proxies.
///
/// This trait provides methods for configuring how HTTP requests are routed through
/// proxies, which is essential for bypassing geographical restrictions or IP blocks
/// that YouTube might impose.
///
/// ## Implementing Types
///
/// The library provides two built-in implementations:
/// - `GenericProxyConfig`: For standard HTTP/HTTPS proxies
/// - `WebshareProxyConfig`: For Webshare's rotating residential proxies
///
/// ## Custom Implementations
///
/// You can implement this trait for your own proxy providers by:
/// 1. Creating a struct with your proxy configuration details
/// 2. Implementing the required methods to generate proxy URLs
/// 3. Pass your custom implementation to `YouTubeTranscriptApi::new`
///
/// ## Example
///
/// ```rust,no_run
/// # use std::any::Any;
/// # use std::collections::HashMap;
/// # use yt_transcript_rs::proxies::{ProxyConfig, InvalidProxyConfig};
/// #[derive(Debug)]
/// struct MyCustomProxy {
///     server: String,
///     port: u16,
///     username: String,
///     password: String,
/// }
///
/// impl ProxyConfig for MyCustomProxy {
///     fn to_requests_dict(&self) -> HashMap<String, String> {
///         let url = format!(
///             "http://{}:{}@{}:{}",
///             self.username, self.password, self.server, self.port
///         );
///         
///         let mut map = HashMap::new();
///         map.insert("http".to_string(), url.clone());
///         map.insert("https".to_string(), url);
///         map
///     }
///     
///     fn prevent_keeping_connections_alive(&self) -> bool {
///         false // We want persistent connections
///     }
///     
///     fn retries_when_blocked(&self) -> i32 {
///         3 // Retry up to 3 times if blocked
///     }
///     
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
/// }
/// ```
pub trait ProxyConfig: Debug + Send + Sync {
    /// Converts the proxy configuration to a reqwest-compatible proxy URL map.
    ///
    /// This method should return a HashMap with keys "http" and/or "https"
    /// containing the formatted proxy URLs for each protocol.
    ///
    /// # Returns
    ///
    /// * `HashMap<String, String>` - Map of protocol names to proxy URLs
    ///
    /// # Expected Format
    ///
    /// The URL format typically follows: `protocol://[username:password@]host:port`
    fn to_requests_dict(&self) -> HashMap<String, String>;

    /// Controls whether HTTP connections should be closed after each request.
    ///
    /// Setting this to `true` ensures you get a new connection (and potentially
    /// a new IP address) for each request, which is useful for rotating proxies
    /// to prevent IP-based blocking.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` to close connections after each request, `false` to reuse connections
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns `false`, which keeps connections alive.
    fn prevent_keeping_connections_alive(&self) -> bool {
        false
    }

    /// Specifies how many retries to attempt when a request is blocked by YouTube.
    ///
    /// When using rotating residential proxies, this allows the library to
    /// retry the request with different IPs when YouTube blocks a request.
    ///
    /// # Returns
    ///
    /// * `i32` - The number of retries to attempt when blocked (0 = no retries)
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns `0`, meaning no retries.
    fn retries_when_blocked(&self) -> i32 {
        0
    }

    /// Type conversion for dynamic dispatch and type identification.
    ///
    /// This method is used internally to determine the concrete type of the proxy
    /// configuration, which is needed for specific error handling and behavior.
    ///
    /// # Returns
    ///
    /// * `&dyn Any` - Reference to the concrete type as `Any`
    fn as_any(&self) -> &dyn Any;
}

/// # GenericProxyConfig
///
/// A generic proxy configuration for standard HTTP/HTTPS proxies.
///
/// This configuration allows you to specify separate proxies for HTTP and HTTPS
/// requests, or use the same proxy for both. It's suitable for most standard
/// proxy services.
///
/// ## Features
///
/// - Support for separate HTTP and HTTPS proxies
/// - Simple configuration with minimal required fields
/// - Compatible with most proxy services
///
/// ## Example Usage
///
/// ```rust,no_run
/// # use yt_transcript_rs::proxies::GenericProxyConfig;
/// # use yt_transcript_rs::YouTubeTranscriptApi;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a proxy configuration with the same proxy for both HTTP and HTTPS
/// let proxy = GenericProxyConfig::new(
///     Some("http://username:password@proxy.example.com:8080".to_string()),
///     None
/// )?;
///
/// // Use it with the YouTube Transcript API
/// let api = YouTubeTranscriptApi::new(
///     None,
///     Some(Box::new(proxy)),
///     None
/// )?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GenericProxyConfig {
    /// URL for HTTP proxy (format: "http://[username:password@]host:port")
    pub http_url: Option<String>,
    /// URL for HTTPS proxy (format: "https://[username:password@]host:port")
    pub https_url: Option<String>,
}

impl GenericProxyConfig {
    /// Creates a new generic proxy configuration.
    ///
    /// You can specify different proxies for HTTP and HTTPS requests, or use
    /// the same proxy for both by specifying only one. At least one of the
    /// proxy URLs must be provided.
    ///
    /// # Parameters
    ///
    /// * `http_url` - Optional URL for HTTP proxy
    /// * `https_url` - Optional URL for HTTPS proxy
    ///
    /// # Returns
    ///
    /// * `Result<Self, InvalidProxyConfig>` - A new proxy configuration or an error
    ///
    /// # Errors
    ///
    /// Returns `InvalidProxyConfig` if both `http_url` and `https_url` are `None`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::proxies::GenericProxyConfig;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Configure different proxies for HTTP and HTTPS
    /// let proxy = GenericProxyConfig::new(
    ///     Some("http://user:pass@proxy1.example.com:8080".to_string()),
    ///     Some("http://user:pass@proxy2.example.com:8443".to_string())
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
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
    /// Converts the generic proxy configuration to a reqwest-compatible dictionary.
    ///
    /// If either HTTP or HTTPS URL is missing, the other is used as a fallback.
    ///
    /// # Returns
    ///
    /// * `HashMap<String, String>` - Map with "http" and "https" keys and their proxy URLs
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

/// # WebshareProxyConfig
///
/// Specialized proxy configuration for Webshare's rotating residential proxies.
///
/// Webshare provides residential proxies that rotate IPs automatically, which is
/// extremely useful for accessing YouTube without being blocked. This configuration
/// is optimized for Webshare's API format.
///
/// ## Features
///
/// - Automatic IP rotation for each request
/// - Configurable retry mechanism for handling blocks
/// - Optimized for Webshare's proxy service
///
/// ## Important Note
///
/// For reliable YouTube access, use Webshare's "Residential" proxies, not the
/// "Proxy Server" or "Static Residential" options, as YouTube often blocks those IPs.
///
/// ## Example Usage
///
/// ```rust,no_run
/// # use yt_transcript_rs::proxies::WebshareProxyConfig;
/// # use yt_transcript_rs::YouTubeTranscriptApi;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a Webshare proxy configuration with credentials
/// let proxy = WebshareProxyConfig::new(
///     "your_username".to_string(),
///     "your_password".to_string(),
///     5,                             // Retry up to 5 times if blocked
///     None,                          // Use default domain
///     None                           // Use default port
/// );
///
/// // Use it with the YouTube Transcript API
/// let api = YouTubeTranscriptApi::new(
///     None,
///     Some(Box::new(proxy)),
///     None
/// )?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct WebshareProxyConfig {
    /// Your Webshare proxy username
    pub proxy_username: String,
    /// Your Webshare proxy password
    pub proxy_password: String,
    /// The proxy domain name (default: "p.webshare.io")
    pub domain_name: String,
    /// The port number to use (default: 80)
    pub proxy_port: u16,
    /// Number of retries to attempt when blocked
    pub retries: i32,
}

impl WebshareProxyConfig {
    /// Default domain name for Webshare proxies
    pub const DEFAULT_DOMAIN_NAME: &'static str = "p.webshare.io";
    /// Default port for Webshare proxies
    pub const DEFAULT_PORT: u16 = 80;

    /// Creates a new Webshare proxy configuration.
    ///
    /// This configuration is specifically designed for Webshare's rotating proxy service.
    /// It automatically adds the rotation feature to your proxy.
    ///
    /// # Parameters
    ///
    /// * `proxy_username` - Your Webshare proxy username
    /// * `proxy_password` - Your Webshare proxy password
    /// * `retries_when_blocked` - Number of retries to attempt if blocked (recommended: 3-5)
    /// * `domain_name` - Optional custom domain name (default: "p.webshare.io")
    /// * `proxy_port` - Optional custom port (default: 80)
    ///
    /// # Returns
    ///
    /// * `Self` - A new Webshare proxy configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::proxies::WebshareProxyConfig;
    /// // Basic configuration
    /// let proxy = WebshareProxyConfig::new(
    ///     "username".to_string(),
    ///     "password".to_string(),
    ///     3,     // Retry 3 times
    ///     None,  // Use default domain
    ///     None   // Use default port
    /// );
    ///
    /// // Custom domain and port
    /// let proxy_custom = WebshareProxyConfig::new(
    ///     "username".to_string(),
    ///     "password".to_string(),
    ///     5,
    ///     Some("custom.webshare.io".to_string()),
    ///     Some(8080)
    /// );
    /// ```
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

    /// Generates the complete proxy URL for Webshare.
    ///
    /// This formats the proxy URL with rotation enabled by appending "-rotate"
    /// to the username, which tells Webshare to provide a new IP for each request.
    ///
    /// # Returns
    ///
    /// * `String` - The formatted proxy URL
    ///
    /// # Example (internal)
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::proxies::WebshareProxyConfig;
    /// # fn example() {
    /// let proxy = WebshareProxyConfig::new(
    ///     "user123".to_string(),
    ///     "pass456".to_string(),
    ///     3,
    ///     None,
    ///     None
    /// );
    ///
    /// // Generates: "http://user123-rotate:pass456@p.webshare.io:80/"
    /// let url = proxy.url();
    /// # }
    /// ```
    pub fn url(&self) -> String {
        format!(
            "http://{}-rotate:{}@{}:{}/",
            self.proxy_username, self.proxy_password, self.domain_name, self.proxy_port
        )
    }
}

impl ProxyConfig for WebshareProxyConfig {
    /// Converts the Webshare proxy configuration to a reqwest-compatible dictionary.
    ///
    /// Uses the same URL for both HTTP and HTTPS requests.
    ///
    /// # Returns
    ///
    /// * `HashMap<String, String>` - Map with "http" and "https" keys and proxy URLs
    fn to_requests_dict(&self) -> HashMap<String, String> {
        let url = self.url();
        let mut map = HashMap::new();

        map.insert("http".to_string(), url.clone());
        map.insert("https".to_string(), url);

        map
    }

    /// Always returns `true` to ensure connection rotation.
    ///
    /// Webshare rotating proxies work best when a new connection is established
    /// for each request, ensuring you get a fresh IP address each time.
    ///
    /// # Returns
    ///
    /// * `bool` - Always `true` for Webshare proxies
    fn prevent_keeping_connections_alive(&self) -> bool {
        true
    }

    /// Returns the configured number of retries.
    ///
    /// This determines how many times the library will retry a request with
    /// a new IP address if YouTube blocks the request.
    ///
    /// # Returns
    ///
    /// * `i32` - The number of retries to attempt
    fn retries_when_blocked(&self) -> i32 {
        self.retries
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
