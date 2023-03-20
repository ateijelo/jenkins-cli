use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use reqwest::Url;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{channel, Sender},
    task::JoinSet,
};

use crate::{config::JenkinsConfig, job::JobBuild};

lazy_static! {
    static ref SUB_BUILD: Regex =
        Regex::new(r"^Starting building: (?P<job_name>.+) #(?P<job_number>\d+)$").unwrap();
}

#[derive(Debug)]
struct NewTask(Url, Sender<NewTask>);

async fn _tail(job: Url, config: Arc<JenkinsConfig>, tx: Sender<NewTask>) -> Result<()> {
    let client = Client::new();
    let mut start = 0;
    let profile = config.profile()?;

    let build = JobBuild::new(&job)?;
    loop {
        let resp = client
            .get(build.log_path(start)?)
            .basic_auth(&profile.username, Some(&profile.password))
            .send()
            .await?;

        let more_data = resp.headers().get("x-more-data").cloned();
        let text_size = resp.headers().get("x-text-size").cloned();

        for line in resp.text().await?.lines() {
            if let Some(captures) = SUB_BUILD.captures(line) {
                let job = captures.name("job_name").unwrap().as_str().to_owned();
                let job = job.replace(" » ", "/job/");
                let number: u32 = captures.name("job_number").unwrap().as_str().parse()?;

                tx.send(NewTask(
                    profile.url()?.join(&format!("/job/{}/{}", job, number))?,
                    tx.clone(),
                ))
                .await?;
            }
            println!("{build}: {line}");
        }

        let mut more = false;
        if let Some(md) = more_data {
            if let Some(ts) = text_size {
                if md == "true" {
                    more = true;
                }
                start = ts.to_str()?.parse()?;
            }
        }

        if !more {
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

pub async fn tail(job: String, config: JenkinsConfig) -> Result<()> {
    let (tx, mut rx) = channel(8);

    let url = if job.starts_with('/') {
        config.profile()?.url()?.join(&job)?
    } else {
        Url::parse(&job)?
    };

    let cfg = Arc::new(config);
    let mut tasks = JoinSet::new();
    tasks.spawn(_tail(url.clone(), cfg.clone(), tx));

    while let Some(msg) = rx.recv().await {
        let NewTask(url, tx) = msg;
        tasks.spawn(_tail(url.clone(), cfg.clone(), tx));
    }

    while let Some(result) = tasks.join_next().await {
        result??;
    }

    Ok(())
}
