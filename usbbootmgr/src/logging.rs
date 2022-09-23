use std::fmt;

// Returns None if there is no content to be printed.
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
    Some(content)
}

#[derive(Debug)]
pub struct PrintContent(String);
impl From<String> for PrintContent {
    fn from(contents: String) -> Self {
        PrintContent(contents)
    }
}
impl From<fmt::Arguments<'_>> for PrintContent {
    fn from(contents: fmt::Arguments) -> Self {
        PrintContent(fmt::format(contents))
    }
}

pub fn info<T: Into<PrintContent>>(contents: T) {
    let contents = match process_input(contents.into().0) {
        None => return,
        Some(x) => x,
    };
    print!("{contents}");
}
pub fn error(content: fmt::Arguments) {
    eprintln!("{content}");
}
