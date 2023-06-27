use anyhow::Result;
use reqwest::Client;
use reqwest::Url;

use crate::{config::JenkinsConfig, job::JobBuild};

pub async fn params(job: String, config: JenkinsConfig) -> Result<()> {
    let client = Client::new();

    let url = if job.starts_with('/') {
        config.profile()?.url()?.join(&job)?
    } else {
        Url::parse(&job)?
    };

    let job = JobBuild::new(&url)?;
    let url = job.params_path()?;
    let profile = config.profile()?;
    let resp = client
        .get(url)
        .basic_auth(&profile.username, Some(&profile.password))
        .send()
        .await?
        .text()
        .await?;

    println!("{:?}", resp);

    Ok(())
}
