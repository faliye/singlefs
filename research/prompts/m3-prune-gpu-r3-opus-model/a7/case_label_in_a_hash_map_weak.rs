#![forbid(unsafe_code)]
mod guard { include!("guard_weak.rs"); }
pub fn attempt(label: guard::NodeLabel) -> usize { let mut map = std::collections::HashMap::new(); map.insert(label, 1u8); map.len() }
