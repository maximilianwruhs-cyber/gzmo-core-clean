//! ETL Module
//!
//! Nightly batch processing for knowledge extraction.
//! Replaces "dreams" with explicit extract/verify/promote pipeline.

pub mod extract;
pub mod verify;
pub mod promote;

pub use extract::{Extractor, Extraction, Relation, ExtractorError};
pub use verify::{Verifier, VerificationResult};
pub use promote::{Promoter, PromotionResult};
