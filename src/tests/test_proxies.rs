use crate::proxies::*;

#[test]
fn test_generic_proxy_config() {
    // Test with both HTTP and HTTPS
    let proxy_config = GenericProxyConfig::new(
        Some("http://proxy.example.com:8080".to_string()),
        Some("https://secure-proxy.example.com:8443".to_string()),
    )
    .unwrap();

    let requests_dict = proxy_config.to_requests_dict();

    assert_eq!(
        requests_dict.get("http").unwrap(),
        "http://proxy.example.com:8080"
    );
    assert_eq!(
        requests_dict.get("https").unwrap(),
        "https://secure-proxy.example.com:8443"
    );

    // By default, keep-alive should be enabled
    assert!(!proxy_config.prevent_keeping_connections_alive());

    // By default, retries should be 0
    assert_eq!(proxy_config.retries_when_blocked(), 0);
}

#[test]
fn test_generic_proxy_config_http_only() {
    // Test with HTTP only
    let proxy_config =
        GenericProxyConfig::new(Some("http://proxy.example.com:8080".to_string()), None).unwrap();

    let requests_dict = proxy_config.to_requests_dict();

    assert_eq!(
        requests_dict.get("http").unwrap(),
        "http://proxy.example.com:8080"
    );
    assert_eq!(
        requests_dict.get("https").unwrap(),
        "http://proxy.example.com:8080"
    );
}

#[test]
fn test_generic_proxy_config_https_only() {
    // Test with HTTPS only
    let proxy_config = GenericProxyConfig::new(
        None,
        Some("https://secure-proxy.example.com:8443".to_string()),
    )
    .unwrap();

    let requests_dict = proxy_config.to_requests_dict();

    assert_eq!(
        requests_dict.get("http").unwrap(),
        "https://secure-proxy.example.com:8443"
    );
    assert_eq!(
        requests_dict.get("https").unwrap(),
        "https://secure-proxy.example.com:8443"
    );
}

#[test]
fn test_generic_proxy_config_invalid() {
    // Test with neither HTTP nor HTTPS
    let proxy_config = GenericProxyConfig::new(None, None);
    assert!(proxy_config.is_err());

    let error = proxy_config.unwrap_err();
    assert!(format!("{}", error).contains("requires you to define at least one"));
}

#[test]
fn test_webshare_proxy_config() {
    // Test with default values
    let proxy_config =
        WebshareProxyConfig::new("user123".to_string(), "pass456".to_string(), 5, None, None);

    let requests_dict = proxy_config.to_requests_dict();

    // Check that both HTTP and HTTPS are set to the same URL
    let expected_url = "http://user123-rotate:pass456@p.webshare.io:80/";
    assert_eq!(requests_dict.get("http").unwrap(), expected_url);
    assert_eq!(requests_dict.get("https").unwrap(), expected_url);

    // Check URL generation
    assert_eq!(proxy_config.url(), expected_url);

    // Keep-alive should be disabled
    assert!(proxy_config.prevent_keeping_connections_alive());

    // Retries should be set to 5
    assert_eq!(proxy_config.retries_when_blocked(), 5);
}

#[test]
fn test_webshare_proxy_config_custom() {
    // Test with custom domain and port
    let proxy_config = WebshareProxyConfig::new(
        "user123".to_string(),
        "pass456".to_string(),
        10,
        Some("custom.webshare.io".to_string()),
        Some(8080),
    );

    let requests_dict = proxy_config.to_requests_dict();

    // Check that both HTTP and HTTPS are set to the same URL
    let expected_url = "http://user123-rotate:pass456@custom.webshare.io:8080/";
    assert_eq!(requests_dict.get("http").unwrap(), expected_url);
    assert_eq!(requests_dict.get("https").unwrap(), expected_url);

    // Check URL generation
    assert_eq!(proxy_config.url(), expected_url);

    // Retries should be set to 10
    assert_eq!(proxy_config.retries_when_blocked(), 10);
}
