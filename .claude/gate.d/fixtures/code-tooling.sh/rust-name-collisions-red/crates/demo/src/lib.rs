mod ledger;
mod journal;
mod inventory;
pub mod inline_pair {
    pub mod reading {
        pub fn shared_inline() {}
    }
    pub mod writing {
        fn shared_inline() {}
    }
}
