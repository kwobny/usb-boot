use std::str::CharIndices;

#[derive(Clone, Debug)]
pub struct SplitStrings<'a> {
    internal_iter: CharIndices<'a>,
    is_exhausted: bool,

    // Whether or not the iterator is currently in a segment of quoted
    // characters.
    // Some(x) indicates iterator is, and the character inside it indicates
    // the specific quotation mark that surrounds the quoted area.
    // None indicates the iterator is not currently in a quoted area.
    currently_quoted: Option<char>,
    whole_str: &'a str,
    // The byte offset of the first character in the current substring.
    // None means iterator is currently in a segment of spaces.
    substring_beginning: Option<usize>,
}
impl<'a> Iterator for SplitStrings<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_exhausted {
            return None;
        }
        loop {
            // Get the next character. If there are no more
            // characters, then return the remaining characters
            // at the end of the whole string.
            let (i, ch) = match self.internal_iter.next() {
                Some(x) => x,
                None => {
                    self.is_exhausted = true;
                    // If iterator was in an area of spaces, then
                    // return None. But if not, then return the remaining
                    // substring at the end of the whole string.
                    return self.substring_beginning.map(|x| &self.whole_str[x..]);
                },
            };

            if ch == ' ' && self.currently_quoted.is_none() {
                if let Some(last_split) = self.substring_beginning {
                    let first = &self.whole_str[last_split..i];
                    self.substring_beginning = None;
                    return Some(first);
                }
                continue;
            }

            if self.substring_beginning.is_none() {
                self.substring_beginning = Some(i);
            }

            if let '"'|'\'' = ch {
                match self.currently_quoted {
                    Some(x) => if x == ch {
                        self.currently_quoted = None;
                    },
                    None => self.currently_quoted = Some(ch),
                }
            }
        }
    }
}
/// Splits a string at every unquoted space in the string.
/// Unquoted means that if a space is inside quotation marks,
/// the string will not be split on that space.
/// Returns an iterator over the split pieces of the string,
/// starting from the front and going to the end.
/// The substrings returned by the iterator will not contain any
/// unquoted spaces.
pub fn split_at_unquoted_spaces(string: &str) -> SplitStrings {
    SplitStrings {
        internal_iter: string.char_indices(),
        is_exhausted: false,

        currently_quoted: None,
        whole_str: string,
        substring_beginning: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_at_unquoted_spaces() {
        let simple_case = r#"   asdfdji   ewaj"   " dfsfde=5"#;
        let simple_case_expected = ["asdfdji", r#"ewaj"   ""#, "dfsfde=5"];

        let testing_everything = r#"    root=UUID=lolololol  tcp_handler 893s zxvv=289 additional_args="single sysrq_always_on=1   fdsaew kjk" dsfder   kernel=/boot/vmlinuz-asdf --single-quoted='asdei "dcxie     " fjid' enclave="cxerdsd 'fds "ewdsji  " fews' dsfds"  --lol=" xczc"#;
        let testing_everything_expected = [
            "root=UUID=lolololol", "tcp_handler", "893s", "zxvv=289", r#"additional_args="single sysrq_always_on=1   fdsaew kjk""#,
            "dsfder", "kernel=/boot/vmlinuz-asdf", r#"--single-quoted='asdei "dcxie     " fjid'"#, r#"enclave="cxerdsd 'fds "ewdsji"#,
            r#"" fews' dsfds""#, r#"--lol=" xczc"#,
        ];

        let quotes_inside_each_other = r#"--asdf=""""jkn ""  "" ewvj 'hello goodbyte' cnvvie="tty3 9cx jszv="32"" 32f  unpaired_quote="asdf eiwo cxbk    ids  "#;
        let quotes_inside_each_other_expected = [r#"--asdf=""""jkn"#, "\"\"", "\"\"", "ewvj", "'hello goodbyte'", r#"cnvvie="tty3 9cx jszv="32"""#, "32f", "unpaired_quote=\"asdf eiwo cxbk    ids  "];

        let test_cases: &[(&str, &[&str])] = &[
            (simple_case, &simple_case_expected),
            (testing_everything, &testing_everything_expected),
            (quotes_inside_each_other, &quotes_inside_each_other_expected),
        ];
        for (input, expected) in test_cases {
            assert_eq!(split_at_unquoted_spaces(input).collect::<Vec<_>>().as_slice(), *expected);
        }
    }
}

/// Tests whether there are any two elements in the slice that are equal
/// to each other.
/// Returns true if every element in the slice is unique, i.e. there are no two elements in
/// the slice that are equal to each other.
/// Returns false if there are at least two elements in the slice that are equal to each other.
pub fn elements_are_unique<T: Eq>(elements: &[T]) -> bool {
    for base in 0..(elements.len()-1) {
        let compare_to = &elements[base];
        for elem in &elements[base+1..] {
            if *elem == *compare_to {
                return false;
            }
        }
    }
    true
}
