//! 攻方原型（m3-batch63-code-r1 opus）：冻结副本的 `verdict_store.rs` 逐字拷进来当一个模块编（gen.py 生成 `verdict_store_under_test.rs`），
//! 它的 `use crate::crash_identity::…`、`pub use crate::crash_judge_conclusions::…` 照下面两个转出模块解析到 checker 档库里的同名类型。
#![allow(dead_code, unused_imports, clippy::all)]

pub mod crash_identity {
    pub use singlefs_checker_tier::crash_identity::*;
}

pub mod crash_judge_conclusions {
    pub use singlefs_checker_tier::crash_judge_conclusions::*;
}

#[path = "verdict_store_under_test.rs"]
pub mod verdict_store;
