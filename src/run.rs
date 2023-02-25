use anyhow::Result;
// use regex::Regex;
use reqwest::Client;
// use std::{sync::Arc, time::Duration};
use url::Url;

use crate::profile::Profile;

fn _job_url(job_path: &str, profile: &Profile) -> Result<Url> {
    // "job/hello-params/build"
    let full_path = format!("job/{job_path}/buildWithParameters");
    Ok(Url::parse(&profile.url)?.join(&full_path)?)
}

pub async fn run(job_path: &str, job_params: &[(String, String)], profile: Profile) -> Result<()> {
    let client = Client::new();

    let full_path = if !job_params.is_empty() {
        format!("job/{job_path}/buildWithParameters")
    } else {
        format!("job/{job_path}/build")
    };

    let url = Url::parse(&profile.url)?.join(&full_path)?;
    // let url = _job_url(job_path, &profile)?;
    let resp = client
        .post(url)
        .basic_auth(&profile.username, Some(&profile.password))
        .send()
        .await?;

    println!("{:?}", resp.headers());
    println!("{:?}", resp.text().await?);
    println!("{:?}", job_params);

    Ok(())
}
