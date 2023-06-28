use anyhow::Result;
use reqwest::Client;
use reqwest::Url;
use serde::Deserialize;
use serde_json::Value;

use crate::{config::JenkinsConfig, job::JobBuild};

#[derive(Debug, Deserialize)]
struct WorkflowRun {
    actions: Vec<Action>,
}

#[derive(Debug, Deserialize)]
struct Action {
    #[serde(default)]
    _class: String,
    #[serde(default)]
    parameters: Vec<ParameterValue>,
}

#[derive(Debug, Deserialize)]
struct ParameterValue {
    _class: String,
    name: String,
    value: Value,
}

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
        .await?;

    let run: WorkflowRun = resp.json().await?;
    for action in run.actions {
        if action._class != "hudson.model.ParametersAction" {
            continue;
        }
        for parameter in action.parameters {
            println!("{}={}", parameter.name, parameter.value);
        }
    }

    Ok(())
}
