use std::borrow::Cow;

pub type PossibleCompletions<'a> = Cow<'a, [&'a str]>;
pub type OptionArgCompleter<'a> = &'a dyn FnOnce() -> PossibleCompletions<'a>;

#[derive(Clone, Copy)]
pub enum CmdlineOptionKind<'a> {
    None,
    Required(OptionArgCompleter<'a>),
}

#[derive(Clone)]
pub struct CmdlineOption<'a> {
    name: Cow<'a, str>,
    option_kind: CmdlineOptionKind<'a>,
}
#[derive(Clone, Debug)]
pub struct PositionalArg {
}

type Options<'a> = Cow<'a, [CmdlineOption<'a>]>;

#[derive(Clone)]
pub struct Subcommand<'a> {
    name: Cow<'a, str>,
    possible_options: Options<'a>,
    positional_args: Cow<'a, [PositionalArg]>,
}

#[derive(Clone)]
pub struct CompleteConfig<'a> {
    initial_options: Options<'a>,
    subcommands: Cow<'a, [Subcommand<'a>]>,
}

struct ArgsState<'a> {
    is_expecting_option_argument: bool,
    subcommand: Option<(&'a Subcommand<'a>, u8)>,
    possible_options: &'a [CmdlineOption<'a>],
    possible_subcommands: &'a [Subcommand<'a>],
}
enum SubcommandState<'a> {
    BeforeSubcommand {
        possible_subcommands: &'a [Subcommand<'a>],
    },
    DuringSubcommand {
        subcommand: &'a Subcommand<'a>,
        current_positional_argument: u32,
    },
}
struct CleanState<'a> {
    subcommand: SubcommandState<'a>,
    possible_options: &'a [CmdlineOption<'a>],
}
enum ArgsState<'a> {
    ExpectingOptionArgument(CleanState<'a>),
    Clean(CleanState<'a>),
    InvalidSyntax,
}
impl<'a> ArgsState<'a> {
    fn new(
        config: &'a CompleteConfig,
    ) -> ArgsState<'a> {
        todo!();
    }
}
// The role of this function is to provide any prior information
// that is necessary to parse the final, incomplete argument.
// This function must not be called on the final, incomplete argument.
fn fold_args(state: &mut ArgsState, current_arg: &str) {
    // Check if the previous option was expecting an argument.
    let clean_state = match state {
        ArgsState::ExpectingOptionArgument(previous_state) => {
            *state = ArgsState::Clean(previous_state);
            return;
        },
        ArgsState::Clean(x) => x,
        ArgsState::
    }

    // The state is a clean slate.

    // Check if the current argument is an option. If it is, then set
    // the expecting option argument state to true if the option expects
    // an argument.
    if let Some(found) = clean_state.possible_options.iter().find(
        |x| x.name == current_arg
    ) {
        *state = match found.option_kind {
            CmdlineOptionKind::None => state,
            CmdlineOptionKind::Required(_) =>
                ArgsState::ExpectingOptionArgument(clean_state),
        };
        return;
    }

    // Current arg is not an option, so it must be a positional
    // argument or subcommand.

    match clean_state.subcommand {
        SubcommandState::DuringSubcommand {
        } => {
            // Current argument is supposed to be a positional argument to
            // a subcommand.
            *index += 1;
        },
        SubcommandState::BeforeSubcommand {
            possible_subcommands,
        } => {
            // Current argument is supposed to be a subcommand.
            if let Some(found) = state.possible_subcommands.iter().find(
                |x| x.name == current_arg
            ) {
                state.subcommand = Some((found, 0));
                state.possible_options = &found.possible_options;
                return;
            } else {
                // Current argument is not a subcommand.
                // This is invalid syntax, but for now just continue like it's
                // all right.
                return;
            }
        },
    }

    todo!();
}

/// This function returns the possible word completions
/// given a set of command line arguments.
/// Parameters:
/// - config: A struct to configure how the function behaves.
/// - args: An iterator of arguments. The last argument in the iterator
///   is expected to be the partially typed word.
pub fn complete_command(
    config: CompleteConfig,
    args: impl IntoIterator<Item = String>,
) -> PossibleCompletions {
    let mut args = args.into_iter().peekable();
    let mut args_state = ArgsState::new(&config);

    while args.peek().is_some() {
        let next_arg = args.next().unwrap();
        fold_args(&mut args_state, &next_arg);
    }
    todo!();
}
