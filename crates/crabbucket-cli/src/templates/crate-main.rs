//! Builds this site.
//!
//! The site is a crate rather than a bare content directory because it uses
//! its own design system, and a design system is a crate you depend on.

use std::path::Path;
use std::process::ExitCode;

// The one line to change if your theme calls its Theme implementation
// something other than `Standard`.
use @THEME_IDENT@::Standard as Design;

fn main() -> ExitCode {
    match crabbucket::build(Path::new("."), &Design) {
        Ok(report) => {
            // Report implements Display, and says everything it has to say --
            // including any warnings and skipped drafts.
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("@NAME@: {}", err.render(std::io::IsTerminal::is_terminal(&std::io::stderr())));
            ExitCode::FAILURE
        }
    }
}
