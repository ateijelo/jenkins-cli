use std::fs::File;
use std::io::Write;

use anyhow::Result;
use tempdir::TempDir;
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

// fn mount_

#[tokio::test]
async fn test_cli() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/job/hello/1/logText/progressiveText"))
        .and(query_param("start", "0"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("abcd")
                .append_header("x-more-data", "true")
                .append_header("x-text-size", "4"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/job/hello/1/logText/progressiveText"))
        .and(query_param("start", "4"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("efgh")
                .append_header("x-more-data", "false")
                .append_header("x-text-size", "8"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let dir = TempDir::new("jenkins-cli-tests")?;
    let cfg_path = dir.path().join("cfg.toml");
    let mut cfg_file = File::create(&cfg_path)?;

    writeln!(
        cfg_file,
        r#"
        url = "{}"
        username = "test"
        password = "test"
        "#,
        &mock_server.uri()
    )?;

    cfg_file.flush()?;

    trycmd::TestCases::new()
        .env("JENKINS_CLI_CONFIG_PATH", cfg_path.to_str().unwrap())
        .case("tests/cmd/*.toml")
        .case("README.md");
    Ok(())
}
