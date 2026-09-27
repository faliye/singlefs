#!/usr/bin/env bash
# 在草稿目录里搭一个小工作区，量 libtest / cargo / rustc 真怎么跑（被判的闸与准入模块判得对不对，要拿真跑的样子比）：
#   SEM-P3a  -- --include-ignored <名> -- --list：第二个 -- 之后的 --list 是过滤词，标 ignore 的用例照跑
#   SEM-P3b  -- --ignored -- --list：只剩过滤词 --list，一条都不跑（对照）
#   SEM-P10a cargo test --config <runner> …：--config 写在 test 之后，cargo 照认，runner 替测试二进制加 --include-ignored
#   SEM-P10b cargo test … --config=<runner>
#   SEM-P9   .cargo/config.toml 里 runner = "sh ../../tools/r.sh"：runner 进程的当前目录是包目录（crates/pkg），参数里的相对路径从那里解
#   SEM-P4   debug 下标 ignore、release 下 include!("common/release_case.rs") 进来一份不标的同名用例：cargo test --release 不带 --ignored 照跑它
#   SEM-P6   use core::include_str as grab; grab!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/mutations.tsv")) 编得过、读到那份文件
# 用法：bash verify_semantics.sh <草稿目录>；经 research/scripts/run-with-memory-cap.sh 起（跑编译出来的代码）。
set -uo pipefail
work="${1:?草稿目录}/semantics"
rm -rf -- "${work:?}"
mkdir -p "$work/crates/pkg/src" "$work/crates/pkg/tests/common" "$work/.cargo" "$work/tools"
export CARGO_TARGET_DIR="$work/target"
cat > "$work/Cargo.toml" <<'TOML'
[workspace]
members = ["crates/pkg"]
resolver = "2"
TOML
cat > "$work/crates/pkg/Cargo.toml" <<'TOML'
[package]
name = "pkg"
version = "0.0.0"
edition = "2021"
TOML
echo 'pub fn one() -> u32 { 1 }' > "$work/crates/pkg/src/lib.rs"
cat > "$work/crates/pkg/tests/own_case.rs" <<'RS'
#[test]
#[ignore]
fn the_case() { std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/ran.own_case"), "").unwrap(); }
#[test]
fn quick() {}
RS
cat > "$work/crates/pkg/tests/inc_case.rs" <<'RS'
#[cfg(debug_assertions)]
#[test]
#[ignore]
fn the_case() { println!("DEBUG_MARKED_COPY"); }
#[cfg(not(debug_assertions))]
include!("common/release_case.rs");
RS
echo '#[test] fn the_case() { std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/ran.inc_case_release"), "").unwrap(); }' > "$work/crates/pkg/tests/common/release_case.rs"
echo 'mutation-table-content' > "$work/crates/mutations.tsv"
cat > "$work/crates/pkg/tests/renamed_include.rs" <<'RS'
use core::include_str as grab;
const TABLE: &str = grab!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/mutations.tsv"));
#[test]
fn reads_it() { println!("RENAMED_INCLUDE_READ={}", TABLE.trim()); }
RS
cat > "$work/tools/r.sh" <<'SH'
echo "RUNNER_SCRIPT_RAN cwd=$(pwd)" >&2
exec "$@"
SH
printf '#!/bin/sh\necho "ADD_IGNORED_RUNNER cwd=$(pwd)" >&2\nexec "$@" --include-ignored\n' > "$work/add-ignored.sh"
chmod +x "$work/add-ignored.sh"
cd "$work" || exit 1
ran() { if [[ -e "$work/crates/pkg/ran.$1" ]]; then rm -f "$work/crates/pkg/ran.$1"; echo ran; else echo not-run; fi; }
say() { printf 'SEM\t%s\t%s\t%s\n' "$1" "$2" "$3"; }
run() { local log="$1"; shift; "$@" > "$log" 2>&1; echo "$?"; }
nice -n 19 cargo build --tests -q -p pkg > "$work/build.log" 2>&1 || { echo "build failed"; tail -20 "$work/build.log"; exit 1; }

rc=$(run p3c.log cargo test -q -p pkg --test own_case)
say P3c-control "$(ran own_case)" "rc=$rc；$(grep -m1 '^test result' p3c.log)"
rc=$(run p3a.log cargo test -q -p pkg --test own_case -- --include-ignored the_case -- --list)
say P3a "$(ran own_case)" "rc=$rc；$(grep -m1 '^test result' p3a.log)"
rc=$(run p3b.log cargo test -q -p pkg --test own_case -- --ignored -- --list)
say P3b "$(ran own_case)" "rc=$rc；$(grep -m1 '^test result' p3b.log)"
rc=$(run p10a.log cargo test -q --config "target.x86_64-unknown-linux-gnu.runner=\"$work/add-ignored.sh\"" -p pkg --test own_case)
say P10a "$(ran own_case)" "rc=$rc；$(grep -m1 -o 'ADD_IGNORED_RUNNER.*' p10a.log | sed "s#$work#<小仓>#")；$(grep -m1 'test result' p10a.log)"
rc=$(run p10b.log cargo test -q -p pkg --test own_case "--config=target.x86_64-unknown-linux-gnu.runner=\"$work/add-ignored.sh\"")
say P10b "$(ran own_case)" "rc=$rc；$(grep -m1 'test result' p10b.log)"
printf '[target.x86_64-unknown-linux-gnu]\nrunner = "sh ../../tools/r.sh"\n' > "$work/.cargo/config.toml"
rc=$(run p9.log cargo test -q -p pkg --test own_case)
say P9 "$(grep -m1 -o 'RUNNER_SCRIPT_RAN cwd=.*' p9.log | sed "s#$work#<小仓>#" || echo runner-not-seen)" "rc=$rc"
rm -f "$work/.cargo/config.toml"
rc=$(run p4.log cargo test -q --release -p pkg --test inc_case)
say P4 "$(ran inc_case_release)" "rc=$rc；$(grep -m1 'test result' p4.log)"
rc=$(run p6.log cargo test -q -p pkg --test renamed_include -- --nocapture)
say P6 "$(grep -m1 -o 'RENAMED_INCLUDE_READ=.*' p6.log || echo not-read)" "rc=$rc"
rc=$(run p9b.log env CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="sh ../../tools/r.sh" cargo test -q -p pkg --test own_case)
say P9b "$(grep -m1 -o 'RUNNER_SCRIPT_RAN cwd=.*' p9b.log | sed "s#$work#<小仓>#" || echo runner-not-seen)" "rc=$rc（环境变量写法 CARGO_TARGET_…_RUNNER=\"sh ../../tools/r.sh\"）"
