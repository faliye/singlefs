#!/usr/bin/env bash
# mutation-list-items 格构建环境不进键样本的现场：reuse-three-questions 的同一个小工作区（一个包 tiny，两份源文件 lib.rs、second.rs，各一条普通测试），
# 另写 .cargo/config.toml 与 Cargo.lock（unrelated.sh 改这两份：构建环境与依赖版本不在这一格的键里，要复用）。变异表由同目录的 rows.tsv 抄进 crates/mutations.tsv
# （两行各改一份文件，按改的文件分成两组）。源文件由这里现写；crates-mutation-rows.py --list-items 在它上面只起 git，不编不跑。
set -euo pipefail
mkdir -p crates/tiny/src .cargo
printf '[workspace]\nmembers = ["crates/tiny"]\nresolver = "2"\n' > Cargo.toml
printf '[package]\nname = "tiny"\nversion = "0.1.0"\nedition = "2021"\n' > crates/tiny/Cargo.toml
printf 'pub mod second;\n\npub fn add(left: u32, right: u32) -> u32 { left + right }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() {\n        assert_eq!(super::add(2, 3), 5);\n    }\n\n    #[test]\n    fn doubles() {\n        assert_eq!(super::second::double(4), 8);\n    }\n}\n' > crates/tiny/src/lib.rs
printf 'pub fn double(value: u32) -> u32 { value * 2 }\n' > crates/tiny/src/second.rs
printf '[build]\nrustflags = ["-Cdebuginfo=0"]\n' > .cargo/config.toml
printf 'version = 4\n\n[[package]]\nname = "tiny"\nversion = "0.1.0"\n' > Cargo.lock
cp rows.tsv crates/mutations.tsv
git init -q . && git add -A
