#![forbid(unsafe_code)]
mod guard { include!("guard_weak.rs"); }
pub fn attempt(store: &guard::VerdictStore) -> Option<guard::Verdict> { store.reuse(&guard::PathWriteTableHash([0u8; 32])) }
