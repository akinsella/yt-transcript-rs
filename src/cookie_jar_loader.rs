use crate::errors::CookieError;
use reqwest::cookie::Jar;
use std::path::Path;
use std::sync::Arc;

/// Responsible for loading and managing cookie jars from files
pub struct CookieJarLoader;

impl CookieJarLoader {
    /// Load a cookie jar from a file path
    ///
    /// This expects cookies in Netscape format and validates the file contents
    pub fn load_cookie_jar(cookie_path: &Path) -> Result<Jar, CookieError> {
        if !cookie_path.exists() {
            return Err(CookieError::PathInvalid(cookie_path.display().to_string()));
        }

        let content = std::fs::read_to_string(cookie_path)
            .map_err(|_| CookieError::PathInvalid(cookie_path.display().to_string()))?;

        if content.trim().is_empty() {
            return Err(CookieError::Invalid(cookie_path.display().to_string()));
        }

        // Parse the cookie file (expected to be in Netscape format)
        let jar = Jar::default();
        let cookie_lines = content
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty());

        let mut has_cookies = false;

        for line in cookie_lines {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 7 {
                let domain = parts[0];
                let path = parts[2];
                let secure = parts[3] == "TRUE";
                let name = parts[5];
                let value = parts[6];

                let cookie = format!("{}={}", name, value);
                let url = format!(
                    "{}://{}{}",
                    if secure { "https" } else { "http" },
                    domain,
                    path
                );

                jar.add_cookie_str(&cookie, &url.parse().unwrap());
                has_cookies = true;
            }
        }

        if !has_cookies {
            return Err(CookieError::Invalid(cookie_path.display().to_string()));
        }

        Ok(jar)
    }

    /// Create an Arc-wrapped cookie jar from a file path
    pub fn create_cookie_jar(cookie_path: &Path) -> Result<Arc<Jar>, CookieError> {
        let jar = Self::load_cookie_jar(cookie_path)?;
        Ok(Arc::new(jar))
    }
}
