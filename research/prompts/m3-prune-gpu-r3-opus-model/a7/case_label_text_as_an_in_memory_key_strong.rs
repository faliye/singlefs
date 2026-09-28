#![forbid(unsafe_code)]
mod guard { include!("guard_strong.rs"); }
pub fn attempt(label: &guard::NodeLabel, verdict: guard::Verdict) -> usize { let mut map = std::collections::HashMap::new(); map.insert(label.to_string(), verdict.0); map.len() }
