use super::*;

const TEST_CONFIG: &str = "src/test_config.toml";

#[test]
#[ignore]
fn interact_with_user_interactive() {
    let result = interact_with_user(TEST_CONFIG);
    println!("{:#?}", result);
}

#[test]
#[ignore]
fn config_parse_interactive() {
    let parsed = config_parsing::parse_config(TEST_CONFIG);
    println!("{:#?}", parsed);
}

#[test]
#[ignore]
fn cli_parse_interactive() {
    let parsed = cmdline_parsing::Cli::try_parse();
    println!("{:#?}", parsed);
}
