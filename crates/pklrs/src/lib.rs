pub use pklrs_derive::pkl;

pub mod codec;
pub mod de;
pub mod decoder;
pub mod error;
pub mod evaluator;
pub mod evaluator_options;
pub mod message;
pub mod module_source;
pub mod process;
pub mod reader;
pub mod ser;
pub mod types;
pub mod value;

pub use de::from_pkl_value;
pub use decoder::decode_pkl_binary;
pub use ser::to_pkl_value;
pub use error::{Error, Result};
pub use evaluator::{Evaluator, EvaluatorManager};
pub use evaluator_options::EvaluatorOptions;
pub use module_source::ModuleSource;
pub use reader::{ModuleReader, ResourceReader};
pub use types::{DataSize, DataSizeUnit, Duration, DurationUnit, IntSeq, Pair, PklRegex};
pub use value::{ObjectMember, PklValue};

/// One-shot evaluation of inline PKL text.
///
/// Creates a temporary evaluator, evaluates the text, and returns the result.
pub fn evaluate_text(text: &str) -> Result<PklValue> {
    let mut manager = EvaluatorManager::new()?;
    let opts = EvaluatorOptions::preconfigured();
    let evaluator = manager.new_evaluator(opts)?;
    let source = ModuleSource::text(text);
    let result = manager.evaluate_module(&evaluator, source);
    let _ = manager.close_evaluator(&evaluator);
    result
}
