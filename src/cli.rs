use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug)]
#[command()]
pub struct JenkinsArgs {
    #[arg(short, long, env = "JENKINS_CLI_PROFILE", default_value = "default")]
    pub profile: String,

    #[arg(long, default_value_t = false)]
    pub show_config_path: bool,

    #[arg(long, default_value_t = false)]
    pub show_config: bool,

    #[arg(short, long)]
    pub config_path: Option<String>,

    #[command(subcommand)]
    pub action: Action
}

#[derive(Subcommand, Debug)]
pub enum Action {
    Run,
    Tail(TailArgs),
    Params,
}

#[derive(Args, Debug)]
pub struct TailArgs {
    #[arg(short = 'j', long)]
    pub job_name: String,

    #[arg(short = 'n', long)]
    pub job_number: u32,
}
