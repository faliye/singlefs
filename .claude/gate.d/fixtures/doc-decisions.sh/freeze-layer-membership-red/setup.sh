#!/usr/bin/env bash
# 这一道从 crates/singlefs-format/src/lib.rs 现读树 ID 常量；样本在临时目录里造一份，仓里不放 .rs。
set -e
mkdir -p crates/singlefs-format/src
cat > crates/singlefs-format/src/lib.rs <<'RS'
pub const TREE_IDENTIFIER_EXTENT: u64 = 11;
pub const TREE_IDENTIFIER_INODE: u64 = 12;
pub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 13;
pub const TREE_IDENTIFIER_ACCOUNTING: u64 = 14;
pub const TREE_IDENTIFIER_CENTRAL_MAPPING: u64 = 15;
pub const TREE_IDENTIFIER_LIVELIST: u64 = 16;
pub const TREE_IDENTIFIER_SPARSE_SIDE_TABLE: u64 = 17;
pub const TREE_IDENTIFIER_DEADLIST: u64 = 18;
pub const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 11;
pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 19;
RS
