use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{channel, Sender},
    task::JoinSet,
};
use url::Url;

use crate::profile::Profile;

#[derive(Debug)]
struct NewTask(String, u32, Sender<NewTask>);

fn _job_url(job_path: &str, job_number: u32, start: u32, profile: &Profile) -> Result<Url> {
    let full_path = format!("job/{job_path}/{job_number}/logText/progressiveText?start={start}");
    Ok(Url::parse(&profile.url)?.join(&full_path)?)
}

async fn _tail(
    job_path: String,
    job_number: u32,
    profile: Arc<Profile>,
    subjob_re: Arc<Regex>,
    tx: Sender<NewTask>,
) -> Result<()> {
    let client = Client::new();
    let mut start = 0;
    loop {
        let url = _job_url(&job_path, job_number, start, &profile)?;
        let resp = client
            .get(url)
            .basic_auth(&profile.username, Some(&profile.password))
            .send()
            .await?;

        let more_data = resp.headers().get("x-more-data").cloned();
        let text_size = resp.headers().get("x-text-size").cloned();

        for line in resp.text().await?.lines() {
            if let Some(captures) = subjob_re.captures(line) {
                let job = captures.name("job_name").unwrap().as_str().to_owned();
                let job = job.replace(" » ", "/job/");
                let number: u32 = captures.name("job_number").unwrap().as_str().parse()?;
                tx.send(NewTask(job, number, tx.clone())).await?;
            }
            println!("{} #{job_number}: {line}", job_path.replace("/job/", " » "));
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

pub async fn tail(job_path: &str, job_number: u32, profile: Profile) -> Result<()> {
    let (tx, mut rx) = channel(8);

    let subjob_re = Arc::new(
        Regex::new(r"^Starting building: (?P<job_name>.+) #(?P<job_number>\d+)$").unwrap(),
    );
    let profile = Arc::new(profile);
    let mut tasks = JoinSet::new();
    tasks.spawn(_tail(
        job_path.to_owned(),
        job_number,
        profile.clone(),
        subjob_re.clone(),
        tx,
    ));

    while let Some(msg) = rx.recv().await {
        let NewTask(job_path, job_number, tx) = msg;
        tasks.spawn(_tail(
            job_path,
            job_number,
            profile.clone(),
            subjob_re.clone(),
            tx,
        ));
    }

    while let Some(result) = tasks.join_next().await {
        result??;
    }

    Ok(())
}
