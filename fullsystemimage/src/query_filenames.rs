mod filename_set;

pub struct QueryArgs<'a> {
    expression: Expression<'a>,
}

pub fn query_filenames(args: QueryArgs) {
}

pub trait QueryModule {
    fn value(&self) -> QueryModuleOutput;
}
impl<T> QueryModule for T
    where
        T: filename_set::FilenamesProducer {
    fn value(&self) -> QueryModuleOutput {
        QueryModuleOutput::Filenames(self)
    }
}

pub enum QueryModuleOutput<'a> {
    Filenames(&'a dyn filename_set::FilenamesProducer),
    Expression(Expression<'a>),
}
pub type Expression<'a> = &'a dyn Iterator<Item = filename_set::Term<'a>>;
