use std::{collections::{HashMap, hash_map::Entry}, fs};

/// This is a struct that represents a path in the filesystem.
/// All filenames are relative to the root directory.
/// A filename that refers to a directory encompasses everything
/// in that directory.
#[derive(Clone, Debug)]
pub struct Filename {
    // This vector does not contain any '/' characters.
    // An empty components vector indicates the filename
    // refers to the root directory itself.
    components: Vec<String>,
}
impl Filename {
    /// Construct a filename from a string.
    /// Returns Err if the path does not have a leading slash.
    pub fn from_str(path: String) -> Result<Filename, ()> {
        let mut iter = path.chars();
        let first_char = iter.next().ok_or(())?;
        if first_char != '/' {
            return Err(());
        }

        let remaining = iter.as_str();
        let components = remaining.split('/')
            .filter(|x| x.len() > 0)
            .map(str::to_owned).collect();
        Ok(Filename {
            components,
        })
    }
    pub fn to_string(&self) -> String {
        let mut acc = String::new();
        for component in self.components.iter() {
            acc.push('/');
            acc.push_str(&component);
        }
        acc
    }
}

pub struct FilenameSet;
impl FilenameSet {
    pub fn from_iter<T>(iter: T) -> Self
        where
            T: IntoIterator<Item = Filename> {
        todo!();
    }
}
impl FilenameSet {
    pub(super) fn evaluate_expression<'a>(
        options: EvaluateOptions,
        expression: impl IntoIterator<Item = Term<'a>>,
    ) -> FilenameSet {
        todo!();
    }

    pub(super) fn iter(&self) -> FilenamesIter {
        todo!();
    }
}
pub(super) struct EvaluateOptions {
    pub(super) allow_duplicate_addition: bool,
    pub(super) allow_nonpresent_subtraction: bool,
}
pub(super) struct FilenamesIter {
}
impl Iterator for FilenamesIter {
    type Item = Filename;
    fn next(&mut self) -> Option<Self::Item> {
        todo!();
    }
}

impl FilenameSet {
    fn sum(left: FilenameSet, right: FilenameSet) {
    }
}

pub trait FilenamesProducer {
    fn produce(&self) -> FilenameSet;
}

pub enum Sign {
    Positive,
    Negative,
}
pub struct Term<'a> {
    pub sign: Sign,
    pub filenames: &'a dyn FilenamesProducer,
}

#[derive(Clone, Debug)]
struct FileTree {
    // A node that's a leaf means
    // that this file tree includes everything
    // under the root directory itself.
    root: Node,
}

#[derive(Clone, Copy, Debug)]
enum LeafKind {
    File,
    Directory,
}
#[derive(Clone, Debug)]
enum Node {
    Stem(Stem),
    Leaf(LeafKind),
}
type Stem = HashMap<String, Node>;

impl FileTree {
    /// Construct a file tree from a set of filenames.
    /// The file tree is the union of the filenames.
    fn from_filenames(
        iter: impl IntoIterator<Item = Filename>
    ) -> Result<FileTree, ()> {
        let mut root: HashMap<String, Node> = HashMap::new();
        for filename in iter {
            // If the filename has no components, that means
            // the filename refers to the root directory.
            if filename.components.len() == 0 {
                return Ok(FileTree {
                    root: Node::Leaf(LeafKind::Directory),
                })
            }

            // Loop through each component in the filename.
            let mut curr_stem = &mut root;
            let mut curr_directory = String::new();
            let mut components_iter = filename.components.into_iter();
            let mut current_component = components_iter.next().unwrap();
            loop {
                let next_component = components_iter.next();
                if next_component.is_none() {
                    // Current component is a leaf.

                    // Check if the filename is a directory.
                    let combined_path = format!(
                        "{}/{}",
                        curr_directory, current_component,
                    );
                    let metadata = fs::symlink_metadata(combined_path)
                        .map_err(|_| ())?;
                    let is_dir = metadata.is_dir();

                    // Insert a leaf node with a leaf kind
                    // depending on whether the filename refers
                    // to a directory.
                    let leaf_kind = match is_dir {
                        true => LeafKind::Directory,
                        false => LeafKind::File,
                    };
                    curr_stem.insert(current_component, Node::Leaf(leaf_kind));

                    break;
                } else {
                    // Current component is a directory.

                    // Push the current component onto the current directory.
                    curr_directory.push('/');
                    curr_directory.push_str(&current_component);

                    curr_stem = match curr_stem.entry(current_component) {
                        // The current component is not in the stem yet.
                        // Make a new stem and set the current stem to it.
                        Entry::Vacant(vacant) => {
                            let new_node = Node::Stem(HashMap::new());
                            match vacant.insert(new_node) {
                                Node::Stem(x) => x,
                                Node::Leaf(_) => panic!(),
                            }
                        },
                        // The current component is in the stem.
                        // Set the current stem to the existing stem.
                        Entry::Occupied(occupied) => match occupied.into_mut() {
                            Node::Stem(x) => x,
                            Node::Leaf(leaf) => match leaf {
                                // If the existing node is a directory leaf,
                                // then stop looping because this leaf
                                // already contains the filename.
                                LeafKind::Directory => break,
                                LeafKind::File => return Err(()),
                            },
                        },
                    };
                }

                current_component = next_component.unwrap();
            }
        }

        Ok(FileTree {
            root: Node::Stem(root),
        })
    }
    fn iterate(&self) -> impl Iterator<Item = Vec<&str>> {
        struct FileTreeIter {
        }
        impl Iterator for FileTreeIter {
            type Item = Vec<&str>;
            fn next(&mut self) -> Option<Self::Item> {
                todo!();
            }
        }

        todo!();
    }

    fn difference_relaxed(whole: impl Iterator<Item = Filename>, subtract: FileTree) -> FileTree {
        todo!();
    }
    fn difference_strict(whole: FileTree, subtract: FileTree) -> Result<FileTree, ()> {
        // Check that all stuff in subtract is also in whole.
        // If there is a filename/path that's only in subtract,
        // return Err.
        let lol = match (whole.root, subtract.root) {
            (Node::Leaf(_), Node::Leaf(_)) =>
                return Ok(FileTree {
                    root: Node::Stem(HashMap::new()),
                }),
            (Node::Stem(_), Node::Leaf(_)) =>
                return Err(()),
        };
        let subtract_tree_stack = vec![subtract.root.iter()];
        let whole_tree_stack = Vec::new();
        loop {
            let subtract_stack_top = subtract.last().unwrap();
            break;
        }
        todo!();
    }
}
