use clap::Parser;
use reqwest::Client;

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

mod profile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use profile::Profile;

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

    let client = Client::new();
    let resp = client
        .get(profile.url)
        .basic_auth(profile.username, Some(profile.password))
        .send()
        .await?;

    println!("{:#?}", resp);
    Ok(())
}
