use std::fmt;

// This function processes the input given by the user
// to any of the logging functions and transforms it to be suitable
// for logging.
// Treatment of newlines at the end:
//     No newline -> Add a newline at the end.
//     1 newline -> Do nothing.
//     2 or more newlines -> Remove a newline.
// Returns None if there is no content to be logged.
fn process_input(mut content: String) -> Option<String> {
    let mut chars = content.chars();
    match (chars.next_back(), chars.next_back()) {
        (None, _) => {
            // There is no content to be printed.
            return None;
        },
        (Some('\n'), Some('\n')) => {
            // There is more than one newline character at the end.
            // Remove one newline character.
            content.pop();
        },
        (Some('\n'), _) => {
            // There is only one newline character at the end. Do nothing.
        },
        (Some(_), _) => {
            // There are no newline characters at the end.
            // Add a newline character.
            content.push('\n');
        },
    }
    // true -> return Some.
    // false -> return None.
    let result: bool = loop {
        let last_char = match chars.next_back() {
            Some(x) => x,
            // There is no content to be printed.
            None => break false,
        };

        if last_char == '\n' {
            let second_to_last = chars.next_back();
            if let Some('\n') = second_to_last {
                // There is more than one newline character at the end.
                // Remove one newline character.
                content.pop();
            } else {
                // There is only one newline character at the end. Do nothing.
            }
            break true;
        }

        let remaining_str = chars.as_str();
        let last_newline = remaining_str.rfind('\n');
        let add_newline = if let Some(last_newline) = last_newline {
            let asdf = &remaining_str[(last_newline+1)..];
            let lol = asdf.trim_start_matches(['\t', ' ']);
            !(lol.len() == 0)
        } else {
            true
        };
        if add_newline {
            content.push('\n');
        }

        break true;
    };
    Some(content)
}

/// This struct represents a possible form of a
/// message to be logged.
#[derive(Debug)]
pub struct LogMessage(String);
impl From<String> for LogMessage {
    fn from(contents: String) -> Self {
        LogMessage(contents)
    }
}
impl From<&str> for LogMessage {
    fn from(contents: &str) -> Self {
        LogMessage(contents.to_owned())
    }
}
impl From<fmt::Arguments<'_>> for LogMessage {
    fn from(contents: fmt::Arguments) -> Self {
        LogMessage(fmt::format(contents))
    }
}

/// This function logs an info message to the console.
/// The provided message can be anything that can
/// be converted to a [`LogMessage`]. Look at the documentation
/// for that to see more information about message types.
///
/// This function transforms the provided message a bit:
/// Terminating newline behavior:
///     No newline at the end -> Add a newline.
///     1 newline at the end -> Do nothing.
///     2 or more newlines at the end -> Remove 1 newline.
///     The reason for this behavior is to make it easier
///     to type multiline log messages. Without this, you
///     would have to type the end quote or a backslash in
///     the same line as the last line of the message. With this,
///     you can now put the end quote on a separate line.
pub fn info<T: Into<LogMessage>>(message: T) {
    let contents = match process_input(message.into().0) {
        None => return,
        Some(x) => x,
    };
    print!("{contents}");
}
/// Same as [`info`], except logs an error instead of info.
/// Look there for more information about the semantics
/// of this function.
pub fn error<T: Into<LogMessage>>(message: T) {
    let contents = match process_input(message.into().0) {
        None => return,
        Some(x) => x,
    };
    eprintln!("{contents}");
}
