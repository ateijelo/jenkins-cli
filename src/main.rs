use anyhow::Result;
use clap::Parser;
use regex::Regex;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Sender};
use url::Url;

use jenkins_cli::profile::Profile;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long, env = "JENKINS_CLI_PROFILE", default_value = "default")]
    profile: String,

    #[arg(long, default_value_t = false)]
    show_config_path: bool,

    #[arg(long, default_value_t = false)]
    show_config: bool,

    #[arg(short, long)]
    config_path: Option<String>,
}

#[derive(Debug)]
struct NewTask(String, u32, Sender<NewTask>);

async fn _tail(
    job_path: String,
    job_number: u32,
    profile: Arc<Profile>,
    subjob_re: Arc<Regex>,
    tx: Sender<NewTask>,
) -> Result<()> {
    let client = Client::new();
    let url = _job_url(&job_path, job_number, &profile)?;
    let resp = client
        .get(url)
        .basic_auth(&profile.username, Some(&profile.password))
        .send()
        .await?;

    for line in resp.text().await?.lines() {
        if let Some(captures) = subjob_re.captures(line) {
            let job = captures.name("job_name").unwrap().as_str().to_owned();
            let number: u32 = captures.name("job_number").unwrap().as_str().parse()?;
            tx.send(NewTask(job, number, tx.clone())).await?;
        } else {
            println!("{job_path} #{job_number}: {line}");
        }
    }
    Ok(())
}

fn _job_url(job_path: &str, job_number: u32, profile: &Profile) -> Result<Url> {
    // "job/hello-pipeline/5/logText/progressiveText?start=0".to_owned(),
    let start = 0;
    let full_path = format!("job/{job_path}/{job_number}/logText/progressiveText?start={start}");
    Ok(Url::parse(&profile.url)?.join(&full_path)?)
}

async fn tail(job_path: &str, job_number: u32, profile: Profile) -> Result<()> {
    let (tx, mut rx) = channel(8);

    let subjob_re = Arc::new(
        Regex::new(r"Starting building: (?P<job_name>[^\s]+) #(?P<job_number>\d+)").unwrap(),
    );
    let profile = Arc::new(profile);
    tokio::spawn(_tail(
        job_path.to_owned(),
        job_number,
        profile.clone(),
        subjob_re.clone(),
        tx,
    ));

    while let Some(msg) = rx.recv().await {
        let NewTask(job_path, job_number, tx) = msg;
        // let url = _job_url(&job_path, job_number, &profile)?;
        tokio::spawn(_tail(
            job_path,
            job_number,
            profile.clone(),
            subjob_re.clone(),
            tx,
        ));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.show_config_path {
        println!(
            "{}",
            Profile::config_path(&args.config_path, &args.profile)?
                .to_str()
                .unwrap_or("")
        );
        std::process::exit(0);
    }

    let profile = Profile::new(&args.config_path, &args.profile)?;
    if args.show_config {
        println!("{}", profile);
    }

    tail("hello-pipeline", 5, profile).await?;
    Ok(())
}
