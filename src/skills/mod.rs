//! Skills Module
//!
//! Function registry with dispatch and built-ins.
//! Replaces "pantheon" with honest function registration.

pub mod registry;
pub mod dispatch;
pub mod builtin;

pub use registry::{SkillRegistry, Skill, SkillError};
pub use dispatch::{Dispatcher, Invocation, InvocationResult};
pub use builtin::Builtins;
