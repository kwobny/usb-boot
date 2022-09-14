use crate::query_filenames::QueryModule;

#[derive(Clone, Copy)]
pub struct ImageModule<'a> {
    query_filenames: Option<&'a dyn QueryModule>,
}
pub struct ModuleDatabase {
}
impl ModuleDatabase {
    pub fn get_from_string(&self, name: &str) -> ImageModule {
        todo!();
    }
}

pub fn builtin_modules() -> ModuleDatabase {
    todo!();
}
