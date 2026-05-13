//! Interactive TUI wizard. Entry point: `wizard::run`.

pub mod answers;
pub mod flow;
pub mod strings;

use crate::error::RuntimeError;
use crate::options::Options;
use crate::wizard::answers::to_options;
use crate::wizard::strings::select;

/// Run the wizard. Returns a fully populated `Options` ready for the
/// existing `Config::new + start()` pipeline.
pub fn run(lang_override: Option<&str>) -> Result<Options, RuntimeError> {
    let strings = select(lang_override);
    let answers = flow::collect(strings)?;
    to_options(answers)
}
