use std::fs::File;
use std::io::Write;

use anyhow::Result;
use tempdir::TempDir;
use wiremock::{
    http::HeaderValue,
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

struct TestEnv {
    mock_server: MockServer,
    _temp_dir: TempDir,
    _cfg_file: File,
    cfg_path: String,
}

async fn setup_test() -> Result<TestEnv> {
    let mock_server = MockServer::start().await;

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

    Ok(TestEnv {
        mock_server,
        _temp_dir: dir,
        _cfg_file: cfg_file,
        cfg_path: cfg_path.to_str().unwrap().to_owned(),
    })
}

async fn mount_tail_with_more_data(mock_server: &MockServer) {
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
        .mount(mock_server)
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
        .mount(mock_server)
        .await;
}

async fn mount_job(mock_server: &MockServer, job_path: &str, output: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/job/{job_path}/logText/progressiveText")))
        .and(query_param("start", "0"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(output)
                .append_header("x-more-data", "false")
                .append_header(
                    "x-text-size",
                    HeaderValue::from_bytes(format!("{}", output.len()).into_bytes()).unwrap(),
                ),
        )
        .mount(mock_server)
        .await;
}

#[tokio::test]
async fn test_cli() -> Result<()> {
    let mock_server = MockServer::start().await;

    mount_tail_with_more_data(&mock_server).await;

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

#[tokio::test]
async fn test_tail_with_subjobs() -> Result<()> {
    let testenv = setup_test().await?;

    let mainjob = concat!(
        "AAAA\n",
        "Scheduling project: subjob1\n",
        "AAAA\n",
        "Starting building: subjob1 #10\n",
        "Scheduling project: Folder A » subjob2\n",
        "Starting building: Folder A » subjob2 #20\n",
        "Scheduling project: Folder A » Folder B » subjob3\n",
        "Starting building: Folder A » Folder B » subjob3 #30\n",
    );

    let subjob1 = "BBBB";
    let subjob2 = "CCCC";
    let subjob3 = "DDDD";

    mount_job(&testenv.mock_server, "mainjob/1", mainjob).await;
    mount_job(&testenv.mock_server, "subjob1/10", subjob1).await;
    mount_job(&testenv.mock_server, "Folder%20A/job/subjob2/20", subjob2).await;
    mount_job(
        &testenv.mock_server,
        "Folder%20A/job/Folder%20B/job/subjob3/30",
        subjob3,
    )
    .await;

    let mut cmd = assert_cmd::Command::cargo_bin("jenkins").unwrap();
    let output = cmd
        .args(["tail", "-j", "mainjob", "-n", "1"])
        .env("JENKINS_CLI_CONFIG_PATH", testenv.cfg_path)
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("mainjob #1: AAAA"));
    assert!(stdout.contains("subjob1 #10: BBBB"));
    assert!(stdout.contains("Folder A » subjob2 #20: CCCC"));
    assert!(stdout.contains("Folder A » Folder B » subjob3 #30: DDDD"));

    Ok(())
}

#[tokio::test]
async fn test_that_subjob_regex_is_anchored_to_beginning_of_line() -> Result<()> {
    let testenv = setup_test().await?;

    mount_job(
        &testenv.mock_server,
        "hello/1",
        "This is not a subjob Starting building: hello #1",
    )
    .await;

    let mut cmd = assert_cmd::Command::cargo_bin("jenkins").unwrap();
    let assert = cmd
        .args(["tail", "-j", "hello", "-n", "1"])
        .env("JENKINS_CLI_CONFIG_PATH", testenv.cfg_path)
        .assert();
    assert
        .success()
        .stdout("hello #1: This is not a subjob Starting building: hello #1\n");

    Ok(())
}
