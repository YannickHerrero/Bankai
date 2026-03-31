use std::fmt;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub access_token: Option<String>,
}

#[derive(Debug)]
pub enum TokenError {
    ConfigDir,
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigDir => write!(f, "could not determine config directory"),
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Parse(e) => write!(f, "parse error: {e}"),
            Self::Serialize(e) => write!(f, "serialize error: {e}"),
        }
    }
}

pub fn config_path() -> Result<PathBuf, TokenError> {
    let dir = dirs::config_dir().ok_or(TokenError::ConfigDir)?;
    Ok(dir.join("bankai").join("config.toml"))
}

pub fn load_config() -> Result<Config, TokenError> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = fs::read_to_string(&path).map_err(TokenError::Io)?;
    toml::from_str(&contents).map_err(TokenError::Parse)
}

pub fn save_config(config: &Config) -> Result<(), TokenError> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(TokenError::Io)?;
    }
    let contents = toml::to_string_pretty(config).map_err(TokenError::Serialize)?;
    fs::write(&path, contents).map_err(TokenError::Io)
}
