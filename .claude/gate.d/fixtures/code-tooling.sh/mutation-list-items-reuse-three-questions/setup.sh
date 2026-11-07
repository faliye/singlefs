#!/usr/bin/env bash
# mutation-list-items 格复用三问样本的现场：一个小工作区（一个包 tiny，两份源文件 lib.rs、second.rs，各一条普通测试），变异表由同目录的 rows.tsv 抄进 crates/mutations.tsv
# （两行各改一份文件，按改的文件分成两组）。源文件由这里现写；crates-mutation-rows.py --list-items 在它上面只起 git 与 rustc -vV，不编不跑。
set -euo pipefail
mkdir -p crates/tiny/src
printf '[workspace]\nmembers = ["crates/tiny"]\nresolver = "2"\n' > Cargo.toml
printf '[package]\nname = "tiny"\nversion = "0.1.0"\nedition = "2021"\n' > crates/tiny/Cargo.toml
printf 'pub mod second;\n\npub fn add(left: u32, right: u32) -> u32 { left + right }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() {\n        assert_eq!(super::add(2, 3), 5);\n    }\n\n    #[test]\n    fn doubles() {\n        assert_eq!(super::second::double(4), 8);\n    }\n}\n' > crates/tiny/src/lib.rs
printf 'pub fn double(value: u32) -> u32 { value * 2 }\n' > crates/tiny/src/second.rs
cp rows.tsv crates/mutations.tsv
git init -q . && git add -A
