use std::{env, fmt, net::SocketAddr};

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_FRONTEND_DIST_DIR: &str = "frontend/dist";
const DEFAULT_RATE_LIMIT_REQUESTS_PER_WINDOW: u32 = 5;
const DEFAULT_RATE_LIMIT_WINDOW_SECONDS: u64 = 60;

#[derive(Clone)]
pub struct Config {
    pub host: std::net::IpAddr,
    pub port: u16,
    pub frontend_dist_dir: String,
    pub email_proxy_url: Option<String>,
    pub email_app_token: Option<String>,
    pub rate_limit_requests_per_window: u32,
    pub rate_limit_window_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            host: read_parsed(
                "HOST",
                DEFAULT_HOST
                    .parse::<std::net::IpAddr>()
                    .map_err(|source| ConfigError {
                        variable: "HOST",
                        value: DEFAULT_HOST.to_string(),
                        message: source.to_string(),
                    })?,
            )?,
            port: read_parsed("PORT", DEFAULT_PORT)?,
            frontend_dist_dir: read_optional("FRONTEND_DIST_DIR")?
                .unwrap_or_else(|| DEFAULT_FRONTEND_DIST_DIR.to_string()),
            email_proxy_url: read_optional("MCTAI_EMAIL_URL")?,
            email_app_token: read_optional("MCTAI_EMAIL_APP_TOKEN")?,
            rate_limit_requests_per_window: read_parsed(
                "RATE_LIMIT_REQUESTS_PER_WINDOW",
                DEFAULT_RATE_LIMIT_REQUESTS_PER_WINDOW,
            )?,
            rate_limit_window_seconds: read_parsed(
                "RATE_LIMIT_WINDOW_SECONDS",
                DEFAULT_RATE_LIMIT_WINDOW_SECONDS,
            )?,
        })
    }

    pub fn bind_address(&self) -> SocketAddr {
        SocketAddr::from((self.host, self.port))
    }

    pub fn email_proxy_configured(&self) -> bool {
        self.email_proxy_url.is_some() && self.email_app_token.is_some()
    }
}

fn read_optional(name: &'static str) -> Result<Option<String>, ConfigError> {
    match env::var(name) {
        Ok(value) if !value.trim().is_empty() => Ok(Some(value)),
        Ok(_) | Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(value)) => Err(ConfigError {
            variable: name,
            value: value.to_string_lossy().into_owned(),
            message: "value is not valid unicode".to_string(),
        }),
    }
}

fn read_parsed<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
    T::Err: fmt::Display,
{
    let Some(raw_value) = read_optional(name)? else {
        return Ok(default);
    };

    raw_value.parse::<T>().map_err(|source| ConfigError {
        variable: name,
        value: raw_value,
        message: source.to_string(),
    })
}

#[derive(Debug)]
pub struct ConfigError {
    variable: &'static str,
    value: String,
    message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid value {:?} for {}: {}",
            self.value, self.variable, self.message
        )
    }
}

impl std::error::Error for ConfigError {}
