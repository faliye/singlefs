#!/usr/bin/env bash
# 模型：crash-verifier 定义第 6 步的指纹，在别的会话于阶段跑的时候改源码的几种序列上，变不变。
# 对照：按内容算的指纹（tracked + 未跟踪、非忽略文件逐个 sha256，缺失文件记名字）。
# 只在临时 git 仓里跑，不碰 singlefs 仓。每个情形从同一个起点建一个新仓，互不影响。
#   bash fingerprint-blindspots.sh
# 输出每行：情形名 step6_变没变 content_变没变 判定（漏报 / 误报 / 两边都报 / 两边都不报）；末行 name=done。
set -euo pipefail
BASE="$(mktemp -d "${TMPDIR:-/tmp}/cv-fingerprint.XXXXXX")"
trap 'rm -rf "${BASE:?}"' EXIT

step6_fingerprint() {   # crash-verifier.md 第 6 步原文的两条命令
  printf '%s|%s' \
    "$(git diff -- crates litmus | sha256sum | cut -d' ' -f1)" \
    "$(git ls-files --others --exclude-standard -- crates litmus | sha256sum | cut -d' ' -f1)"
}
content_fingerprint() { # 对照：按内容
  git ls-files -co --exclude-standard -z -- crates litmus | sort -zu |
    while IFS= read -r -d '' path; do
      if [[ -e "$path" ]]; then sha256sum -- "$path"; else printf 'deleted  %s\n' "$path"; fi
    done | sha256sum | cut -d' ' -f1
}
new_repo() {  # 起点：一个已提交的 lib.rs、一个未提交修改的 recovery.rs、一个未跟踪的 mount.rs（与 2026-09-17 真仓 crates/ 的形状同型）
  local dir="$BASE/$1"; mkdir -p "$dir"; cd "$dir"
  git init -q -b master .
  git config user.email model@example.invalid; git config user.name model
  mkdir -p crates/core/src litmus
  printf 'pub mod mount;\npub mod recovery;\n' > crates/core/src/lib.rs
  printf 'pub fn choose_root() -> u64 { 1 }\n' > crates/core/src/recovery.rs
  printf 'x\n' > litmus/a.litmus
  git add -A; git commit -qm base
  printf 'pub fn choose_root() -> u64 { 2 }\n' > crates/core/src/recovery.rs   # 别的会话开跑前就有的未提交修改
  printf 'pub fn mount() -> u64 { 1 }\n' > crates/core/src/mount.rs            # 别的会话开跑前就有的未跟踪新文件
}
verdict() { # $1 step6 变没变  $2 content 变没变
  case "$1/$2" in
    same/changed) echo "漏报：源码变了，第6步指纹不变" ;;
    changed/same) echo "误报：源码没变，第6步指纹变了" ;;
    changed/changed) echo "两边都报" ;;
    same/same) echo "两边都不报" ;;
  esac
}
run_case() { # $1 情形名  $2 阶段跑的时候别的会话做的事（在仓里 eval）
  new_repo "$1"
  local s6_start c_start s6_end c_end s6 c
  s6_start="$(step6_fingerprint)"; c_start="$(content_fingerprint)"
  eval "$2"
  s6_end="$(step6_fingerprint)"; c_end="$(content_fingerprint)"
  [[ "$s6_start" == "$s6_end" ]] && s6=same || s6=changed
  [[ "$c_start" == "$c_end" ]] && c=same || c=changed
  printf 'name=case case=%s step6=%s content=%s verdict=%s\n' "$1" "$s6" "$c" "$(verdict "$s6" "$c")"
}
run_case S1_untracked_content_edit 'printf "pub fn mount() -> u64 { 99 }\n" > crates/core/src/mount.rs'
run_case S2_clean_file_edit_then_add 'printf "pub mod mount;\npub mod recovery;\npub mod extra;\n" > crates/core/src/lib.rs; git add crates/core/src/lib.rs'
run_case S3_clean_file_edit_add_commit 'printf "pub mod mount;\npub mod recovery;\npub mod extra;\n" > crates/core/src/lib.rs; git add crates/core/src/lib.rs; git commit -qm other-session'
run_case S4_modified_file_edit_add_control 'printf "pub fn choose_root() -> u64 { 3 }\n" > crates/core/src/recovery.rs; git add crates/core/src/recovery.rs'
run_case C1_unstaged_edit_control 'printf "pub fn choose_root() -> u64 { 4 }\n" > crates/core/src/recovery.rs'
run_case C2_new_untracked_file_control 'printf "pub fn table() {}\n" > crates/core/src/instance_table.rs'
run_case C3_stage_only_no_content_change 'git add crates/core/src/recovery.rs'
run_case C4_nothing_happens ':'
echo "name=done cases=8"
