use anyhow::bail;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use reqwest::Url;

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct JenkinsArgs {
    #[arg(short, long)]
    pub profile: Option<String>,

    #[arg(long, default_value_t = false)]
    pub show_config_path: bool,

    #[arg(long, default_value_t = false)]
    pub show_config: bool,

    #[arg(short, long, env = "JENKINS_CLI_CONFIG_PATH")]
    pub config_path: Option<String>,

    #[command(subcommand)]
    pub action: Option<Action>,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    Run(RunArgs),
    Tail(TailArgs),
    Params(ParamsArgs),
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
pub struct TailArgs {
    #[arg()]
    pub job_url: String,
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
pub struct ParamsArgs {
    #[arg()]
    pub job_url: String,
}

fn parse_param(param: &str) -> Result<(String, String)> {
    if let Some((k, v)) = param.split_once('=') {
        return Ok((k.to_owned(), v.to_owned()));
    }
    bail!(format!(
        "Param argument {} doesn't have the form PARAM=VALUE",
        param
    ))
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
pub struct RunArgs {
    #[arg()]
    pub job_name: Url,

    #[arg(value_parser=parse_param)]
    pub params: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn clap_check() {
        JenkinsArgs::command().debug_assert();
    }
}
