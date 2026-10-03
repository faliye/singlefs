#!/usr/bin/env bash
# 从冻结副本里原样抽 E161 / E165 / E168 三份装置的线程数解析函数，拼成一个独立程序，量非 UTF-8 / 前后空白 / 0 / 空串的结局。
set -euo pipefail
TREE=${1:-/tmp/claude-1000/test-script-align-code-r3/tree}
OUT=${2:-/tmp/claude-1000/test-script-align-code-r3-sonnet/silent.rs}
BIN=$TREE/crates/singlefs-checker-tier/src/bin
# 抽法：从函数头那一行起、到之后第一个顶格 `}` 止；函数头的行号先用 grep -nF 核（不手数）
extract() { # 文件 函数头原文 期望行号
  grep -nF "$2" "$1" | grep -q "^$3:" || { echo "函数头不在第 $3 行：$2" >&2; exit 1; }
  awk -v start="$3" 'NR>=start { print } NR>start && /^}$/ { exit }' "$1" | sed 's/^fn /pub fn /'
}
{
  echo 'use std::env;'
  echo 'mod e161 { const THREADS_ENVIRONMENT_VARIABLE: &str = "E161_THREADS";'
  extract "$BIN/e161_crash_state_dedup_and_time_split.rs" 'fn threads_from_environment() -> usize {' 2637
  echo '}'
  echo 'mod e165 { use std::env; const THREADS_ENVIRONMENT_VARIABLE: &str = "E165_THREADS"; const SHARED_THREAD_CAP_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_THREAD_CAP";'
  extract "$BIN/e165_replay_bounded_enumeration.rs" 'fn thread_count_from_the_environment() -> usize {' 7442
  echo '}'
  echo 'mod e168 { const THREADS_ENVIRONMENT_VARIABLE: &str = "E168_THREADS"; const SHARED_THREAD_CAP_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_THREAD_CAP";'
  extract "$BIN/e168_delete_ring_window_borrow.rs" 'fn thread_count_from(' 587
  echo '}'
  cat <<'RS'
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
fn run(label: &str, f: impl FnOnce() -> usize) {
    match catch_unwind(AssertUnwindSafe(f)) { Ok(v) => println!("{label} -> OK {v}"), Err(_) => println!("{label} -> PANIC") }
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let raw = OsString::from_vec(vec![0x38, 0xff]);
    for (name, value) in [("non-utf8", Some(raw.clone())), ("lead-space", Some(OsString::from(" 4"))), ("zero", Some(OsString::from("0"))), ("empty", Some(OsString::from(""))), ("unset", None)] {
        for v in ["E161_THREADS", "E165_THREADS", "E168_THREADS", "SINGLEFS_THREAD_CAP"] { env::remove_var(v); }
        if let Some(v) = &value { env::set_var("E161_THREADS", v); env::set_var("E165_THREADS", v); }
        run(&format!("E161 {name:10}"), e161::threads_from_environment);
        run(&format!("E165 {name:10}"), e165::thread_count_from_the_environment);
        let s = value.as_ref().and_then(|v| v.to_str().map(str::to_string));
        run(&format!("E168 {name:10} (装置调用处是 std::env::var(..).ok()：非 UTF-8 先变 None)"), || e168::thread_count_from(s.as_deref(), None, 7));
    }
}
RS
} > "$OUT"
echo "wrote $OUT"
