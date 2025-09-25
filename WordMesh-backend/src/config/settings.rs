use serde::Deserialize;
use std::env;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub neo4j: Neo4jSettings,
    pub application: ApplicationSettings,
    pub jwt: JwtSettings,
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
}

impl DatabaseSettings {
    #[allow(dead_code)]
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
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
    pub expiration_hours: i64,
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

        config.try_deserialize()
    }

    #[allow(dead_code)]
    pub fn load_for_environment(env: &str) -> Result<Self, config::ConfigError> {
        // 构建指定环境的配置
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::File::with_name(&format!("config/{}", env)).required(false))
            .add_source(config::Environment::with_prefix("WORDMESH").separator("_"))
            .build()?;

        config.try_deserialize()
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
                expiration_hours: 24,
            },
            logging: LoggingSettings {
                level: "info".to_string(),
            },
        }
    }
}
