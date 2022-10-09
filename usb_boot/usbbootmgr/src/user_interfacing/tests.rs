use std::iter;

use super::*;

const TEST_CONFIG: &str = "src/test_config.toml";
const EXECUTABLE_NAME: &str = "lol";

fn get_cmdline_args() -> Result<
    impl IntoIterator<Item = impl Into<OsString> + Clone>, (),
> {
    let mut args_iter = env::args_os();
    while let Some(next_arg) = args_iter.next() {
        if next_arg == "--" {
            return Ok(iter::once(EXECUTABLE_NAME.to_string().into())
                      .chain(args_iter));
        }
    }
    Err(())
}

#[test]
#[ignore]
fn interact_with_user_interactive() {
    let cmdline = get_cmdline_args().expect("invalid cmdline arguments");
    let result = interact_with_user_provided_cmdline(TEST_CONFIG, cmdline);
    println!("{:#?}", result);
    if let Err(x) = result {
        if let UserInteractError::CliIOError { source: details } |
            UserInteractError::InvalidCommandLineArguments { details } = x
        {
            details.print().unwrap();
        }
    }
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
    let cmdline = get_cmdline_args().expect("invalid cmdline arguments");
    let parsed = cmdline_parsing::Cli::try_parse_from(cmdline);
    println!("{:#?}", parsed);
}
