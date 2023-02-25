#[test]
fn test_cli() {
    trycmd::TestCases::new()
        // .default_bin_name("jenkins")
        .case("tests/cmd/*.toml")
        .case("README.md");
}
