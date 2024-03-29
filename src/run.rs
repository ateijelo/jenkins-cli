use anyhow::{bail, Result};
use reqwest::{Client, Response};
use serde::Deserialize;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use url::Url;

use crate::{config::JenkinsConfig, job::Job, tail::tail};

#[derive(Deserialize, Debug)]
struct QueueResponse {
    why: Option<String>,
    task: Option<Task>,
    executable: Option<Executable>,
    timestamp: Option<u128>,
}

#[derive(Deserialize, Debug)]
struct Task {
    name: String,
}

#[derive(Deserialize, Debug)]
struct Executable {
    number: u32,
    url: String,
}

async fn resp_error(resp: Response, msg: &str) -> Result<String> {
    Ok(format!(
        "{}: status: {:?}, headers: {:?}, body: {:?}",
        msg,
        resp.status(),
        resp.headers().clone(),
        resp.text().await?
    ))
}

pub async fn run(job: &Url, params: &HashMap<String, String>, config: JenkinsConfig) -> Result<()> {
    let client = Client::new();

    let job = Job::new(job)?;
    let full_path = job.build_path(params);
    let profile = config.profile()?;
    let url = profile.url()?.join(&full_path)?;
    let resp = client
        .post(url)
        .basic_auth(&profile.username, Some(&profile.password))
        .form(params)
        .send()
        .await?;

    if resp.status() != 201 {
        bail!(resp_error(resp, "Unexpected response").await?);
    }

    if resp.headers().get("location").is_none() {
        bail!(resp_error(resp, "Location header missing in response").await?);
    }

    for i in 1..10 {
        let loc = resp.headers().get("location").unwrap().to_str()?;
        println!("Waiting on queue item: {}...", loc);
        let loc = Url::parse(loc)?.join("api/json")?;
        let queue_resp: QueueResponse = client
            .get(loc)
            .basic_auth(&profile.username, Some(&profile.password))
            .send()
            .await?
            .json()
            .await?;

        if let Some(why) = queue_resp.why {
            println!("{}", why);
            if let Some(ts) = queue_resp.timestamp {
                let unix_now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
                if ts > unix_now {
                    let until =
                        tokio::time::Instant::now() + Duration::from_millis((ts - unix_now) as u64);
                    tokio::time::sleep_until(until).await;
                    continue;
                }
            }
        }

        if let Some(task) = queue_resp.task {
            if let Some(exec) = queue_resp.executable {
                println!("Tailing job {} #{}:", task.name, exec.number);
                tail(exec.url, config).await?;
                break;
            }
        }

        tokio::time::sleep(Duration::from_secs(i)).await;
    }

    Ok(())
}
