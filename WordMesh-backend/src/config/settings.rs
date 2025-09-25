use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use std::env;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub neo4j: Neo4jSettings,
    pub application: ApplicationSettings,
    pub jwt: JwtSettings,
    pub auth: AuthSettings,
    pub logging: LoggingSettings,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub max_connections: u32,
    #[serde(default = "DatabaseSettings::default_connect_timeout_seconds")]
    pub connect_timeout_seconds: u64,
}

impl DatabaseSettings {
    #[allow(dead_code)]
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn connect_options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
            .database(&self.database_name)
    }

    fn default_connect_timeout_seconds() -> u64 {
        5
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Neo4jSettings {
    pub uri: String,
    pub username: String,
    pub password: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct JwtSettings {
    pub secret: String,
    pub issuer: String,
    #[serde(default = "JwtSettings::default_access_ttl_minutes")]
    pub access_token_ttl_minutes: i64,
    #[serde(default = "JwtSettings::default_refresh_ttl_hours")]
    pub refresh_token_ttl_hours: i64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct AuthSettings {
    #[serde(default = "AuthSettings::default_enabled")]
    pub enabled: bool,
    pub jwt: AuthJwtSettings,
    pub password: AuthPasswordSettings,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct AuthJwtSettings {
    #[serde(default = "AuthJwtSettings::default_algorithm")]
    pub algorithm: String,
    #[serde(default = "AuthJwtSettings::default_access_ttl_secs")]
    pub access_ttl_secs: u64,
    #[serde(default = "AuthJwtSettings::default_refresh_ttl_secs")]
    pub refresh_ttl_secs: u64,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub private_key: Option<String>,
    #[serde(default)]
    pub public_key: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct AuthPasswordSettings {
    #[serde(default = "AuthPasswordSettings::default_min_length")]
    pub min_length: u8,
    #[serde(default = "AuthPasswordSettings::default_require_complexity")]
    pub require_complexity: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub level: String,
}

impl Settings {
    #[allow(dead_code)]
    pub fn load() -> Result<Self, config::ConfigError> {
        // 获取运行环境
        let environment = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());

        // 构建配置，支持多种配置源
        let config = config::Config::builder()
            // 1. 默认配置文件
            .add_source(config::File::with_name("config/default").required(false))
            // 2. 环境特定配置文件
            .add_source(config::File::with_name(&format!("config/{}", environment)).required(false))
            // 3. 环境变量覆盖
            .add_source(config::Environment::with_prefix("WORDMESH").separator("_"))
            .build()?;

        let settings: Settings = config.try_deserialize()?;
        settings.validate()?;
        Ok(settings)
    }

    #[allow(dead_code)]
    pub fn load_for_environment(env: &str) -> Result<Self, config::ConfigError> {
        // 构建指定环境的配置
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::File::with_name(&format!("config/{}", env)).required(false))
            .add_source(config::Environment::with_prefix("WORDMESH").separator("_"))
            .build()?;

        let settings: Settings = config.try_deserialize()?;
        settings.validate()?;
        Ok(settings)
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), config::ConfigError> {
        self.auth.validate()?;
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database: DatabaseSettings {
                username: "wordmesh".to_string(),
                password: "wordmesh123".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                database_name: "wordmesh_dev".to_string(),
                max_connections: 5,
                connect_timeout_seconds: DatabaseSettings::default_connect_timeout_seconds(),
            },
            neo4j: Neo4jSettings {
                uri: "bolt://localhost:7687".to_string(),
                username: "neo4j".to_string(),
                password: "wordmesh123".to_string(),
            },
            application: ApplicationSettings {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            jwt: JwtSettings {
                secret: "your-secret-key".to_string(),
                issuer: "wordmesh-backend".to_string(),
                access_token_ttl_minutes: JwtSettings::default_access_ttl_minutes(),
                refresh_token_ttl_hours: JwtSettings::default_refresh_ttl_hours(),
            },
            auth: AuthSettings::default(),
            logging: LoggingSettings {
                level: "info".to_string(),
            },
        }
    }
}

impl JwtSettings {
    fn default_access_ttl_minutes() -> i64 {
        60
    }

    fn default_refresh_ttl_hours() -> i64 {
        24
    }
}

impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            enabled: AuthSettings::default_enabled(),
            jwt: AuthJwtSettings::default(),
            password: AuthPasswordSettings::default(),
        }
    }
}

impl AuthSettings {
    fn default_enabled() -> bool {
        true
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), config::ConfigError> {
        self.jwt.validate()?;
        self.password.validate()?;
        Ok(())
    }
}

impl Default for AuthJwtSettings {
    fn default() -> Self {
        Self {
            algorithm: AuthJwtSettings::default_algorithm(),
            access_ttl_secs: AuthJwtSettings::default_access_ttl_secs(),
            refresh_ttl_secs: AuthJwtSettings::default_refresh_ttl_secs(),
            secret: None,
            private_key: None,
            public_key: None,
        }
    }
}

impl AuthJwtSettings {
    fn default_algorithm() -> String {
        "HS256".to_string()
    }

    fn default_access_ttl_secs() -> u64 {
        3600
    }

    fn default_refresh_ttl_secs() -> u64 {
        604800
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), config::ConfigError> {
        let algorithm = self.algorithm.to_uppercase();
        match algorithm.as_str() {
            "HS256" => {
                if self.secret.as_ref().map_or(true, |s| s.trim().is_empty()) {
                    return Err(config::ConfigError::Message(
                        "auth.jwt.secret is required when using HS256".into(),
                    ));
                }
            }
            "RS256" => {
                if self.private_key.as_ref().map_or(true, |s| s.trim().is_empty())
                    || self.public_key.as_ref().map_or(true, |s| s.trim().is_empty())
                {
                    return Err(config::ConfigError::Message(
                        "auth.jwt.private_key and auth.jwt.public_key are required when using RS256".into(),
                    ));
                }
            }
            other => {
                return Err(config::ConfigError::Message(format!(
                    "unsupported auth.jwt.algorithm: {}",
                    other
                )));
            }
        }

        if self.access_ttl_secs == 0 {
            return Err(config::ConfigError::Message(
                "auth.jwt.access_ttl_secs must be greater than 0".into(),
            ));
        }

        // refresh_ttl_secs 为 0 表示关闭刷新功能；否则必须大于 access ttl
        if self.refresh_ttl_secs != 0 && self.refresh_ttl_secs <= self.access_ttl_secs {
            return Err(config::ConfigError::Message(
                "auth.jwt.refresh_ttl_secs must be greater than access_ttl_secs".into(),
            ));
        }

        Ok(())
    }
}

impl AuthPasswordSettings {
    fn default_min_length() -> u8 {
        8
    }

    fn default_require_complexity() -> bool {
        false
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), config::ConfigError> {
        if self.min_length < 8 {
            return Err(config::ConfigError::Message(
                "auth.password.min_length must be at least 8".into(),
            ));
        }
        Ok(())
    }
}

impl Default for AuthPasswordSettings {
    fn default() -> Self {
        Self {
            min_length: AuthPasswordSettings::default_min_length(),
            require_complexity: AuthPasswordSettings::default_require_complexity(),
        }
    }
}
