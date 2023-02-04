use config::Config;
use directories::ProjectDirs;
use std::{collections::HashMap, fmt::Display, path::PathBuf};

pub struct Profile {
    pub username: String,
    pub password: String,
    pub url: String,
    pub aliases: HashMap<String, String>,
}

type Err = Box<dyn std::error::Error>;

impl Profile {
    pub fn config_path(path: &Option<String>, profile: &str) -> Result<PathBuf, Err> {
        if let Some(p) = path {
            // if a config file path is provided, we use it
            return Ok(PathBuf::from(p));
        }
        if let Some(dirs) = ProjectDirs::from("", "", "jenkins-cli") {
            // if no config file was provided, but a standard profile exists
            // then we use that
            return Ok(dirs.config_dir().join(format!("{}.toml", profile)));
        }
        Err("No config path provided, and no standard config directory found".into())
    }

    pub fn new(config_path: &Option<String>, profile: &str) -> Result<Self, Err> {
        let mut builder =
            Config::builder().add_source(config::Environment::with_prefix("JENKINS_CLI"));

        let cfg_path = Self::config_path(config_path, profile)?;
        // println!("cfg_path: {:?}", cfg_path);

        if config_path.is_none() {
            if cfg_path.exists() {
                builder = builder.add_source(config::File::from(cfg_path));
            }
        } else {
            builder = builder.add_source(config::File::from(cfg_path));
        }

        let config = builder.build()?;

        let url: String = config.get("url")?;
        let username: String = config.get("username")?;
        let password: String = config.get("password")?;
        let aliases = config
            .get_table("aliases")?
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect();

        Ok(Self {
            url,
            username,
            password,
            aliases,
        })
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "url: {}", self.url)?;
        writeln!(f, "username: {}", self.username)?;
        writeln!(f, "password: {}", "*".repeat(self.password.len()))?;
        write!(f, "aliases: {:?}", self.aliases)
    }
}
