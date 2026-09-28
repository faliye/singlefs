#![forbid(unsafe_code)]
mod guard { include!("guard_strong.rs"); }
pub fn attempt(store: &guard::VerdictStore, label: &guard::NodeLabel) -> Option<guard::Verdict> { store.reuse(label) }
