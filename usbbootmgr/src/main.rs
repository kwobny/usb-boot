use std::io::{self, Write};
use std::process::{ExitCode, Termination};

struct MainReturn(Result<ExitCode, anyhow::Error>);
impl Termination for MainReturn {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(x) => x,
            Err(err) => {
                let _ = writeln!(io::stderr(), "Error: {err:?}");
                ExitCode::FAILURE
            },
        }
    }
}
impl From<Result<ExitCode, anyhow::Error>> for MainReturn {
    fn from(lol: Result<ExitCode, anyhow::Error>) -> Self {
        MainReturn(lol)
    }
}

fn main() -> MainReturn {
    MainReturn::from(usbbootmgr::run())
}
