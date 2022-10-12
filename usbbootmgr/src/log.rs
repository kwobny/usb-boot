use std::{fmt, iter};

// This function processes the input given by the user
// to any of the logging functions and transforms it to be suitable
// for logging.
// Treatment of newlines at the end:
//     n_f = max(1, n_i - 1),
//     where n_i = the number of trailing newlines in the input,
//     and n_f = the number of trailing newlines in the printed
//     output.
//     In other words: remove a newline from the end, but if
//     the result has less than 1 trailing newline, make the result
//     have 1 newline.
//     Another description:
//     1 or less newlines -> 1 newline.
//     more than 1 newlines -> Remove a newline.
// Returns None if there is no content to be logged.
fn process_input(mut content: String) -> Option<String> {
    let mut chars = content.chars();
    match (chars.next_back(), chars.next_back()) {
        (Some('\n'), Some('\n')) => {
            // There is more than one newline character at the end.
            // Remove one newline character.
            content.pop();
        },
        (Some('\n'), _) => {
            // There is only one newline character at the end. Do nothing.
        },
        _ => {
            // There are no newline characters at the end.
            // Add a newline character.
            content.push('\n');
        },
    }
    Some(content)
}

pub trait ToLogMessage {
    fn normal(self) -> Option<LogMessage>
        where
            Self: Sized {
        process_input(self.raw().0).map(LogMessage)
    }
    fn raw(self) -> LogMessage;
}

/// This struct represents a possible form of a
/// message to be logged.
#[derive(Debug)]
pub struct LogMessage(String);
impl ToLogMessage for LogMessage {
    fn normal(self) -> Option<LogMessage>
        where
            Self: Sized {
        Some(self)
    }
    fn raw(self) -> LogMessage {
        self
    }
}
impl ToLogMessage for String {
    fn raw(self) -> LogMessage {
        LogMessage(self)
    }
}
impl ToLogMessage for &str {
    fn raw(self) -> LogMessage {
        LogMessage(self.to_owned())
    }
}
impl ToLogMessage for fmt::Arguments<'_> {
    fn raw(self) -> LogMessage {
        LogMessage(fmt::format(self))
    }
}
impl ToLogMessage for usize {
    fn normal(self) -> Option<LogMessage>
        where
            Self: Sized {
        Some(self.raw())
    }
    fn raw(self) -> LogMessage {
        let mut ret = String::with_capacity(self);
        ret.extend(iter::repeat('\n').take(self));
        LogMessage(ret)
    }
}
impl ToLogMessage for () {
    fn normal(self) -> Option<LogMessage>
        where
            Self: Sized {
        Some(self.raw())
    }
    fn raw(self) -> LogMessage {
        ToLogMessage::raw(1)
    }
}

/// This function logs an info message to the console.
/// The provided message can be anything that can
/// be converted to a [`LogMessage`]. Look at the documentation
/// for that to see more information about message types.
///
/// This function transforms the provided message a bit:
/// Terminating newline behavior:
///     1 or less newlines -> 1 newline.
///     more than 1 newlines -> Remove a newline.
///     The reason for this behavior is to make it easier
///     to type multiline log messages. Without this, you
///     would have to type the end quote or a backslash in
///     the same line as the last line of the message. With this,
///     you can now put the end quote on a separate line.
pub fn info<T: ToLogMessage>(message: T) {
    message.normal().map(|x| print!("{}", x.0));
}
/// Same as [`info`], except logs an error instead of info.
/// Look there for more information about the semantics
/// of this function.
pub fn error<T: ToLogMessage>(message: T) {
    message.normal().map(|x| eprint!("{}", x.0));
}
