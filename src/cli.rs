use anyhow::bail;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct JenkinsArgs {
    #[arg(short, long, env = "JENKINS_CLI_PROFILE", default_value = "default")]
    pub profile: String,

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
    Params,
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
pub struct TailArgs {
    #[arg(short, short = 'j', long)]
    pub job_name: String,

    #[arg(short, short = 'n', long)]
    pub job_number: u32,
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
    #[arg(short, long)]
    pub job_name: String,

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
