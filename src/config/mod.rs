use std::{fs::{self, File}, io::{self, Write}, path::Path};
use serde::{Deserialize, Serialize};
use simple_home_dir::home_dir;
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct CauthConfigRaw {
    database_url: String,
}

pub struct CauthConfig {
    pub db_conn: PgPool,
}

enum CauthConfigError {
    /// Returned when the file cannot be created/opened
    FileError,
    /// Returned when the config content's cannot be parsed into config
    ParseError,
    /// Returned when the database is unreachable under the database_url provided
    DatabaseError
}

impl CauthConfig {
    pub async fn parse() -> Result<Self, CauthConfigError> {
        let config_file = Self::get_config_file()?;
        let config_content = match io::read_to_string(config_file) {
            Ok(content) => content,
            Err(_) => return Err(CauthConfigError::FileError)
        };
        let config_raw = match serde_yml::from_str::<CauthConfigRaw>(&config_content) {
            Ok(config) => config,
            Err(_) => return Err(CauthConfigError::ParseError)
        };

        let db_conn = match PgPool::connect(&config_raw.database_url).await {
            Ok(db_conn) => db_conn,
            Err(_) => return Err(CauthConfigError::DatabaseError)
        };

        let config = CauthConfig {
            db_conn
        };
        
        return Ok(config);
    }

    pub fn save(new_data: CauthConfigRaw) -> Result<(), CauthConfigError> {
        let mut config_file = Self::get_config_file()?;
        let serialized = match serde_yml::to_string(&new_data) {
            Ok(serialized) => serialized,
            Err(_) => return Err(CauthConfigError::ParseError)
        };

        match config_file.write(serialized.as_bytes()) {
            Ok(_) => (),
            Err(_) => return Err(CauthConfigError::FileError)
        };

        return Ok(());
    }

    fn get_config_file() -> Result<File, CauthConfigError> {
        let config_dir_path_str = format!(
            "{}/.cauth",
            home_dir().unwrap().to_str().unwrap()
        );
        let config_dir_path = Path::new(&config_dir_path_str);

        if !config_dir_path.exists() {
            match fs::create_dir_all(config_dir_path) {
                Ok(_) => (),
                Err(_) => return Err(CauthConfigError::FileError)
            };
        }

        let config_path_str = format!(
            "{}/config.yml",
            config_dir_path_str
        );
        let mut binding = File::options();
        let config_file_options = binding
            .create(true)
            .read(true)
            .write(true)
            .truncate(false);

        let config_file = match config_file_options.open(config_path_str) {
            Ok(file) => file,
            Err(_) => return Err(CauthConfigError::FileError)
        };

        return Ok(config_file);
    }
}
