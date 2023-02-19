use anyhow::Result;
use clap::Parser;

use jenkins_cli::cli::JenkinsArgs;
use jenkins_cli::profile::Profile;
use jenkins_cli::tail::tail;

#[tokio::main()]
async fn main() -> Result<()> {
    let args = JenkinsArgs::parse();
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

    match args.action {
        jenkins_cli::cli::Action::Run => todo!(),
        jenkins_cli::cli::Action::Tail(tail_args) => {
            tail(&tail_args.job_name, tail_args.job_number, profile).await?
        }
        jenkins_cli::cli::Action::Params => todo!(),
    }

    // tail("hello-pipeline", 10, profile).await?;
    Ok(())
}
