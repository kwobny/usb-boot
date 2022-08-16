use std::collections::VecDeque;

use regex::Regex;
use lazy_static::lazy_static;

#[derive(Clone, Copy, Debug)]
pub enum OptionArgumentKind {
    Required,
    Optional,
    None,
}
pub enum OptionName {
    Long(String),
    Short(char),
}
pub struct CmdlineOption {
    pub name: OptionName,
    pub argument_kind: OptionArgumentKind,
}
pub struct OptionMatcher {
    pub possible_options: Vec<CmdlineOption>,
    pub long_option_prefix: String,
    pub short_option_prefix: String,
}

pub enum Argument {
    Positional(String),
    Option {
        name: OptionName,
        argument: Option<String>,
    },
}

pub enum LexError {
    UnknownOption,
    NoArgumentProvidedForOption,
    UnnecessaryArgumentProvidedForOption,
}

struct LongOption {
    name: String, // Name excluding long option prefix.
    argument_kind: OptionArgumentKind,
}
struct ShortOption {
    name: char,
    argument_kind: OptionArgumentKind,
}
pub struct ArgumentsLexer<T> {
    long_options: Vec<LongOption>,
    long_option_prefix: String,
    short_options: Vec<ShortOption>,
    short_option_prefix: String,

    args_iter: T,
    exhausted: bool,
}

/// Possible ways to specify arguments to options:
/// 1. option argument
/// 2. option=argument (only for long options)
/// 3. oargument (only for short options)
/// Form 1 is not available for optional arguments.
pub fn lex_arguments<T>(
    option_matcher: OptionMatcher,
    args: T,
) -> ArgumentsLexer<T> where
    T: Iterator<Item = String>,
{
    let long_options = option_matcher.possible_options.iter()
        .filter_map(|option| match &option.name {
            OptionName::Long(name) => Some(LongOption {
                name: name.clone(),
                argument_kind: option.argument_kind,
            }),
            OptionName::Short(_)=> None,
        })
        .collect();
    let short_options = option_matcher.possible_options.iter()
        .filter_map(|option| match &option.name {
            OptionName::Short(name) => Some(ShortOption {
                name: name.clone(),
                argument_kind: option.argument_kind,
            }),
            OptionName::Long(_)=> None,
        })
        .collect();

    ArgumentsLexer {
        long_options,
        long_option_prefix: option_matcher.long_option_prefix,
        short_options,
        short_option_prefix: option_matcher.short_option_prefix,

        args_iter: args,
        exhausted: false,
    }
}
impl<T> Iterator for ArgumentsLexer<T> where
    T: Iterator<Item = String>,
{
    type Item = Result<Argument, LexError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }
        let next_elem = self.iter_next();
        if let Some(Err(_)) = next_elem {
            self.exhausted = true;
        } else if next_elem.is_none() {
            self.exhausted = true;
        }
        return next_elem;
    }
}
impl<T> ArgumentsLexer<T> where
    T: Iterator<Item = String>,
{
    fn iter_next(&mut self) -> Option<Result<Argument, LexError>> {
        let argument = self.args_iter.next()?;

        // Test for long option.
        let mut potential_option_argument = None;
        let potential_option_name = match argument.split_once('=') {
            Some((name, argument)) => {
                potential_option_argument = Some(argument);
                name
            },
            None => &argument,
        };
        
        let matches_long_option_prefix = potential_option_name
            .strip_prefix(&self.long_option_prefix);

        if let Some(option_without_prefix) = matches_long_option_prefix {
            let is_long_option = self.long_options.iter()
                .find(|long_option| long_option.name == option_without_prefix);
            let long_option = match is_long_option {
                None => return Some(Err(LexError::UnknownOption)),
                Some(x) => x,
            };

            let option_name = OptionName::Long(long_option.name.clone());
            return match &long_option.argument_kind {
                kind@(OptionArgumentKind::Required|OptionArgumentKind::Optional) => {
                    let option_argument = match potential_option_argument {
                        Some(x) => x.to_string(),
                        None => match kind {
                            OptionArgumentKind::Optional => return Some(Err(LexError::NoArgumentProvidedForOption)),
                            OptionArgumentKind::Required => match self.args_iter.next() {
                                Some(x) => x,
                                None => return Some(Err(LexError::NoArgumentProvidedForOption)),
                            },
                            _ => panic!(),
                        }
                    };
                    Some(Ok(Argument::Option {
                        name: option_name,
                        argument: Some(option_argument),
                    }))
                },
                OptionArgumentKind::None => {
                    if potential_option_argument.is_some() {
                        return Some(Err(LexError::UnnecessaryArgumentProvidedForOption));
                    }
                    Some(Ok(Argument::Option {
                        name: option_name,
                        argument: None,
                    }))
                },
            }
        }

        // Test for short option.
        let matches_short_option_prefix = argument
            .strip_prefix(&self.short_option_prefix)
            .and_then(|x| if x == "" { None } else { Some(x) });

        // Note: without_prefix is not empty.
        if let Some(excluding_prefix) = matches_short_option_prefix {
            let mut options = VecDeque::new();
            let mut chars_iter = excluding_prefix.chars().peekable();
            loop {
                let next_char = match chars_iter.next() {
                    None => break,
                    Some(x) => x,
                };
                let char_is_option = self.short_options.iter()
                    .find(|short_option| short_option.name == next_char);
                if let Some(short_option) = char_is_option {
                    options.push_back(Argument::Option {
                        name: OptionName::Short(short_option.name),
                        argument: None,
                    });
                } else {
                }
            }
        }

        todo!();
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    GiveHelp,
    ChangeKernel,
    UpdateKernel,
}
impl Operation {
    fn from_name(name: &str) -> Result<Operation, ()> {
        use Operation::*;
        match name {
            "help" => Ok(GiveHelp),
            "change-kernel" => Ok(ChangeKernel),
            "update-kernel" => Ok(UpdateKernel),
            _ => Err(()),
        }
    }
}
pub enum ParseError {
    NoCommandGiven,
    UnrecognizedCommand,
}
pub fn parse_args(args: impl Iterator<Item = String>) -> Result<Operation, ParseError> {
    let mut command_specific_options = Vec::new();
    let mut positional_arguments = VecDeque::new();

    for arg in args {
        lazy_static! {
            static ref OPTION_RE: Regex =
                Regex::new("").unwrap();
            static ref ESCAPED_POSITIONAL_RE: Regex =
                Regex::new("").unwrap();
        }
        match &*arg {
            "-h"|"--help" => {
                return Ok(Operation::GiveHelp);
            }
            _ => {
                if OPTION_RE.is_match(&arg) {
                    command_specific_options.push(arg);
                }
                else if ESCAPED_POSITIONAL_RE.is_match(&arg) {
                    let mut temp = arg.chars();
                    temp.next();
                    positional_arguments.push_back(temp.as_str().to_string());
                }
                else {
                    positional_arguments.push_back(arg);
                }
            }
        }
    }

    let command_str = positional_arguments.pop_front()
        .ok_or(ParseError::NoCommandGiven)?;
    let command_choice = Operation::from_name(&command_str)
        .map_err(|_| ParseError::UnrecognizedCommand)?;

    todo!();
}
