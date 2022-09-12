use std::{ops::Sub, iter::FromIterator};

pub struct Filename;

pub struct FilenameSet;
impl FilenameSet {
    pub fn difference<'a>(&'a self, other: &'a FilenameSet) -> FilenameSet {
        todo!();
    }
}
impl FromIterator<Filename> for FilenameSet {
    fn from_iter<T>(iter: T) -> Self
        where
            T: IntoIterator<Item = Filename> {
        todo!();
    }
}
impl<'a> Sub for &'a FilenameSet {
    type Output = FilenameSet;

    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}
impl<'a> IntoIterator for &'a FilenameSet {
    type Item = Filename;
    type IntoIter = FilenamesIter;
    fn into_iter(self) -> Self::IntoIter {
        todo!();
    }
}
pub struct FilenamesIter {
}
impl Iterator for FilenamesIter {
    type Item = Filename;
    fn next(&mut self) -> Option<Self::Item> {
        todo!();
    }
}

pub trait BackupModule {
    /// Returns the list of filenames that are backed up by
    /// this module.
    fn files_backed_up(&self) -> FilenameSet;
}

type Modules<'a> = &'a [&'a dyn BackupModule];

