use clap::Parser;
use reqwest::Client;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long, default_value = "default")]
    profile: String,
}

mod profile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use profile::Profile;
    let args = Args::parse();
    let profile = Profile::new(&args.profile)?;

    let client = Client::new();
    let resp = client
        .get(profile.url)
        .basic_auth(profile.username, Some(profile.password))
        .send()
        .await?;

    println!("{:#?}", resp);
    Ok(())
}
