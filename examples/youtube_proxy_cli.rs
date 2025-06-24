use clap::Parser;
use reqwest::Client;
use std::error::Error;
use std::sync::Arc;
use yt_transcript_rs::proxies::GenericProxyConfig;
use yt_transcript_rs::proxies::InvalidProxyConfig;
use yt_transcript_rs::video_data_fetcher::VideoDataFetcher;

/// Command line arguments for the YouTube proxy example
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// YouTube video ID to fetch
    #[arg(short, long, env = "YOUTUBE_VIDEO_ID")]
    video_id: String,

    /// Proxy host
    #[arg(long, env = "PROXY_HOST")]
    proxy_host: Option<String>,

    /// Proxy port
    #[arg(long, env = "PROXY_PORT")]
    proxy_port: Option<u16>,

    /// Proxy username
    #[arg(long, env = "PROXY_USERNAME")]
    proxy_username: Option<String>,

    /// Proxy password
    #[arg(long, env = "PROXY_PASSWORD")]
    proxy_password: Option<String>,
}

/// YouTube Proxy CLI Example
///
/// This example demonstrates how to:
/// 1. Fetch YouTube video information using a proxy
/// 2. Configure proxy settings via command line arguments or environment variables
/// 3. Handle proxy authentication and connection settings
/// 4. Use the VideoDataFetcher to retrieve video metadata
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize environment variables and logging
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = Args::parse();
    println!("Arguments: {:?}", args);

    // Configure proxy settings
    // Priority: Command line args > Environment variables
    let proxy_config = if args.proxy_host.is_some() {
        EnvProxyConfig::from_args(&args)?.into_boxed_proxy()
    } else {
        EnvProxyConfig::from_env()?.into_boxed_proxy()
    };

    // Configure HTTP client with proxy settings
    let client = {
        // Set up basic client configuration
        let mut builder = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36")
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::ACCEPT_LANGUAGE,
                    reqwest::header::HeaderValue::from_static("en-US"),
                );
                headers
            });

        // Configure proxy if available
        if let Some(proxy_config_ref) = &proxy_config {
            let proxy_map = proxy_config_ref.to_requests_dict();

            // Set up custom proxy configuration
            let proxies = reqwest::Proxy::custom(move |_url| {
                if let Some(http_proxy) = proxy_map.get("http") {
                    return Some(http_proxy.clone());
                }
                None
            });

            builder = builder.proxy(proxies);

            // Handle connection settings for proxy
            if proxy_config_ref.prevent_keeping_connections_alive() {
                builder = builder.connection_verbose(true).tcp_keepalive(None);

                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::CONNECTION,
                    reqwest::header::HeaderValue::from_static("close"),
                );
                builder = builder.default_headers(headers);
            }
        }

        builder.build()?
    };

    // Log configuration details
    println!("proxy_config: {:?}", proxy_config);
    println!("client: {:?}", client);

    // Fetch video information using the configured client
    let fetcher = Arc::new(VideoDataFetcher::new(client.clone()));
    let video_info = fetcher.fetch_video_infos(&args.video_id).await?;
    println!("Video title: {:?}", video_info);

    Ok(())
}

/// Configuration structure for proxy settings
#[derive(Debug, Clone)]
pub struct EnvProxyConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl EnvProxyConfig {
    /// Create proxy configuration from command line arguments
    /// Returns None if no proxy settings are provided, or an error if settings are incomplete
    pub fn from_args(args: &Args) -> Result<Option<Self>, InvalidProxyConfig> {
        match (
            &args.proxy_host,
            args.proxy_port,
            &args.proxy_username,
            &args.proxy_password,
        ) {
            (Some(host), Some(port), Some(username), Some(password)) => Ok(Some(Self {
                host: host.clone(),
                port,
                username: username.clone(),
                password: password.clone(),
            })),
            (None, None, None, None) => Ok(None),
            (host, port, username, password) => Err(InvalidProxyConfig(format!(
                "Invalid proxy config: host={:?}, port={:?}, username={:?}, password={:?}",
                host, port, username, password,
            ))),
        }
    }

    /// Create proxy configuration from environment variables
    /// Returns None if no proxy settings are found, or an error if settings are incomplete
    pub fn from_env() -> Result<Option<Self>, InvalidProxyConfig> {
        let host = std::env::var("PROXY_HOST").ok();
        let port_str = std::env::var("PROXY_PORT").ok();
        let username = std::env::var("PROXY_USERNAME").ok();
        let password = std::env::var("PROXY_PASSWORD").ok();

        match (host, port_str, username, password) {
            (Some(host), Some(port_str), Some(username), Some(password)) => {
                let port = match port_str.parse::<u16>() {
                    Ok(port) => port,
                    Err(_) => {
                        return Err(InvalidProxyConfig(format!("Invalid port: {}", port_str)));
                    }
                };

                Ok(Some(Self {
                    host,
                    port,
                    username,
                    password,
                }))
            }
            (None, None, None, None) => Ok(None),
            (host, port_str, username, password) => Err(InvalidProxyConfig(format!(
                "Invalid proxy config: host={:?}, port={:?}, username={:?}, password={:?}",
                host, port_str, username, password,
            ))),
        }
    }
}

/// Convert a ProxyConfig to a boxed trait object for use with yt_transcript_rs
pub fn get_proxy_config(
    config: Option<&EnvProxyConfig>,
) -> Option<Box<dyn yt_transcript_rs::proxies::ProxyConfig + Send + Sync>> {
    config
        .and_then(|p| GenericProxyConfig::try_from(p.clone()).ok())
        .map(
            |config| -> Box<dyn yt_transcript_rs::proxies::ProxyConfig + Send + Sync> {
                Box::new(config)
            },
        )
}

/// Trait to make Option<&ProxyConfig> easily convertible to Option<Box<dyn ProxyConfig + Send + Sync>>
pub trait IntoBoxedProxyConfig {
    fn into_boxed_proxy(
        self,
    ) -> Option<Box<dyn yt_transcript_rs::proxies::ProxyConfig + Send + Sync>>;
}

impl IntoBoxedProxyConfig for Option<EnvProxyConfig> {
    fn into_boxed_proxy(
        self,
    ) -> Option<Box<dyn yt_transcript_rs::proxies::ProxyConfig + Send + Sync>> {
        self.as_ref().and_then(|p| get_proxy_config(Some(p)))
    }
}

/// Implementation of TryFrom for converting EnvProxyConfig to GenericProxyConfig
impl TryFrom<EnvProxyConfig> for GenericProxyConfig {
    type Error = InvalidProxyConfig;

    fn try_from(config: EnvProxyConfig) -> Result<Self, Self::Error> {
        GenericProxyConfig::new(
            Some(format!(
                "http://{}:{}@{}:{}",
                config.username, config.password, config.host, config.port
            )),
            Some(format!(
                "https://{}:{}@{}:{}",
                config.username, config.password, config.host, config.port
            )),
        )
    }
}
