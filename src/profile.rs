use config::Config;

pub struct Profile {
    pub username: String,
    pub password: String,
    pub url: String,
}

impl Profile {
    pub fn new(profile: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::builder()
            .add_source(config::File::with_name("jenkins-cli.toml"))
            // .add_source(config::Environment::with_prefix("JENKINS_CLI"))
            .build()?;

        let url: String = config.get(&format!("{}.url", profile))?;
        let username: String = config.get(&format!("{}.username", profile))?;
        let password: String = config.get(&format!("{}.password", profile))?;

        Ok(Self {
            url,
            username,
            password,
        })
    }
}
