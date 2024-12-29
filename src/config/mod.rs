use std::{fs::{self, File}, io::{self, Write}, path::Path, process::exit};
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use serde_yml::Value;
use simple_home_dir::home_dir;
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct CauthConfigRaw {
  database_url: String,
}

pub struct CauthConfig {
  pub db_conn: PgPool,
}

#[derive(Debug)]
pub enum CauthParseError {
  /// Returned when the file cannot be created/opened
  FileError,
  /// Returned when the config content's cannot be parsed into config
  ParseError,
  /// Returned when the database is unreachable under the database_url provided
  DatabaseError
}

impl CauthConfig {
  pub fn parse_or_edit() -> Self {
    let config = Self::parse();

    match config {
      Ok(config) => return config,
      Err(_) => {
        Self::edit();

        return Self::parse_or_edit();
      }
    };
  }

  pub fn parse() -> Result<Self, CauthParseError> {
    let config_file = Self::get_config_file()?;
    let config_content = match io::read_to_string(config_file) {
      Ok(content) => content,
      Err(_) => return Err(CauthParseError::FileError)
    };
    let config_raw = match serde_yml::from_str::<CauthConfigRaw>(&config_content) {
      Ok(config) => config,
      Err(_) => return Err(CauthParseError::ParseError)
    };

    let db_conn = match block_on(PgPool::connect(&config_raw.database_url)) {
      Ok(db_conn) => db_conn,
      Err(_) => return Err(CauthParseError::DatabaseError)
    };

    let config = CauthConfig {
      db_conn
    };
    
    return Ok(config);
  }

  pub fn edit() {
    let _ = Self::write_template();

    let fullpath = Self::get_config_full_path();
    let _ = open::that(fullpath).unwrap_or_else(|_| {
      println!("Cannot open the config file, make sure to check permissions of the {} directory.", Self::get_config_path());

      exit(0);
    });
  }

  fn get_config_path() -> String {
    return format!(
      "{}/.cauth",
      home_dir().unwrap().to_str().unwrap()
    );
  }

  fn get_config_full_path() -> String {
    let directory_path = Self::get_config_path();

    return format!(
      "{}/config.yml",
      directory_path
    );
  }

  fn get_config_file() -> Result<File, CauthParseError> {
    let config_dir_path_str = Self::get_config_path();
    let config_dir_path = Path::new(&config_dir_path_str);

    if !config_dir_path.exists() {
      match fs::create_dir_all(config_dir_path) {
        Ok(_) => (),
        Err(_) => return Err(CauthParseError::FileError)
      };
    }

    let config_path_str = Self::get_config_full_path();
    let mut binding = File::options();
    let config_file_options = binding
      .create(true)
      .read(true)
      .write(true)
      .truncate(false);

    let config_file = match config_file_options.open(config_path_str) {
      Ok(file) => file,
      Err(_) => return Err(CauthParseError::FileError)
    };

    return Ok(config_file);
  }

  fn write_template() {
    let mut file = Self::get_config_file().unwrap();
    let filepath = Self::get_config_full_path();
    let content = fs::read_to_string(Path::new(&filepath)).unwrap();

    let parsed = serde_yml::from_str::<serde_yml::Value>(content.as_str())
      .unwrap_or(Value::default());

    let default_map = serde_yml::Mapping::default();
    let mut as_map = parsed
      .as_mapping()
      .unwrap_or(&default_map)
      .clone();

    // this block sets the default comments of CauthConfigRaw if a value is not present in the config.
    // i will replace that with a macro soon
    if !as_map.contains_key("database_url") {
      // we don't want to overwrite the existing value
      let _ = as_map.insert("database_url".into(), serde_yml::to_value("Your database url..").unwrap());
    }

    let as_string = serde_yml::to_string(&as_map).unwrap();
    let _ = file.write(as_string.as_bytes());
  }
}
