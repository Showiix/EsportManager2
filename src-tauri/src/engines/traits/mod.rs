//! 选手特性系统
//!
//! 特性影响选手在不同情境下的表现，通过修改 ability/stability/condition 实现
//! 完全解耦，不影响核心模拟引擎

pub mod engine;
pub mod modifiers;
pub mod types;

#[cfg(test)]
mod tests;

pub use engine::TraitEngine;
pub use modifiers::{TraitContext, TraitModifiers};
pub use types::TraitType;
