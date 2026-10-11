#!/usr/bin/env bash
# 绿样本现场：crate-sources-reachable 那一格——scripts/experiments 下一份平铺的 crate：库、登记的 bin、经 #[path] 引的模块、
# 经 mod 引的子目录模块、经 include_str! 当文本读的载荷，每一份 .rs 都有着落。
set -e
mkdir -p scripts/experiments/first scripts/experiments/shared
printf '[package]\nname = "sample"\nversion = "0.1.0"\n\n[lib]\npath = "lib.rs"\n\n[[bin]]\nname = "first"\npath = "first.rs"\n' > scripts/experiments/Cargo.toml
printf 'mod shared;\n' > scripts/experiments/lib.rs
printf 'pub fn helper() {}\n' > scripts/experiments/shared/mod.rs
printf '#[path = "by_path.rs"]\nmod by_path;\nconst PAYLOAD: &str = include_str!("first/payload.rs");\nfn main() {}\n' > scripts/experiments/first.rs
printf '\n' > scripts/experiments/by_path.rs
printf 'mod never_followed;\n' > scripts/experiments/first/payload.rs
git init -q . && git add -A
