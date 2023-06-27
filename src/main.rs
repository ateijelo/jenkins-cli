use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;

use jenkins_cli::cli::JenkinsArgs;
use jenkins_cli::config::JenkinsConfig;
use jenkins_cli::run::run;
use jenkins_cli::tail::tail;
use jenkins_cli::params::params;

#[tokio::main()]
async fn main() -> Result<()> {
    let args = JenkinsArgs::parse();

    if args.show_config_path {
        println!(
            "{}",
            JenkinsConfig::config_path(&args.config_path)?
                .to_str()
                .unwrap_or("")
        );
        return Ok(());
    }

    let mut config = JenkinsConfig::new(&args.config_path)?;

    if args.show_config {
        println!("{:?}", config);
        return Ok(());
    }

    if let Some(p) = args.profile {
        config.select_profile(&p);
    }

    if let Some(action) = args.action {
        match action {
            jenkins_cli::cli::Action::Run(run_args) => {
                let params = HashMap::from_iter(run_args.params);
                run(&run_args.job_name, &params, config).await?
            }
            jenkins_cli::cli::Action::Tail(tail_args) => {
                tail(tail_args.job_url, config).await?
            }
            jenkins_cli::cli::Action::Params(params_args) => {
                params(params_args.job_url, config).await?
            }
        }
        return Ok(());
    }

    Ok(())
}
