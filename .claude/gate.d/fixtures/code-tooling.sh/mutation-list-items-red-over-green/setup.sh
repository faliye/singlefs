#!/usr/bin/env bash
# mutation-list-items 格记红撤绿样本的现场：与 mutation-list-items-red 同一个小工作区（一个包 tiny，一条普通测试 adds、一条只在 gpu 特性下编的测试 gpu_only，
# 变异表两行都改 lib.rs、成一组），变异表由同目录的 rows.tsv 抄进 crates/mutations.tsv；照这一版逐字拷一份上一版进 .unit-verdict-sample/prior/
# （门禁拿它照判绿把同一把键的那一组记进库），再 git init。crates-mutation-rows.py --list-items 在它上面只起 git 与 rustc -vV，不编不跑。
set -euo pipefail
mkdir -p crates/tiny/src
printf '[workspace]\nmembers = ["crates/tiny"]\nresolver = "2"\n' > Cargo.toml
printf '[package]\nname = "tiny"\nversion = "0.1.0"\nedition = "2021"\n\n[features]\ngpu = []\n' > crates/tiny/Cargo.toml
printf 'pub fn add(left: u32, right: u32) -> u32 { left + right }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() {\n        assert_eq!(super::add(2, 3), 5);\n    }\n\n    #[cfg(feature = "gpu")]\n    #[test]\n    fn gpu_only() {}\n}\n' > crates/tiny/src/lib.rs
cp rows.tsv crates/mutations.tsv
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -dR --preserve=timestamps,links Cargo.toml crates "$prior/"
git init -q . && printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude && git add -A
