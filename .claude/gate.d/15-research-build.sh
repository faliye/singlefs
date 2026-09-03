#!/usr/bin/env bash
# gate-stage: research 构建与单测
#
# **为什么要单开一条**：共享门禁的「构建与单测」阶段只看 `crates/`，
# 而本工程还没有 `crates/`，于是那一阶段恒报「项目尚无 Rust 代码，本阶段不适用」。
# 与此同时，**kb 里几乎每一条实测结论都由 `research/` 下那些实验二进制背书**
# （160 个单测），而它们**从来没有被门禁碰过**。
#
# ⚠️ **实测踩过（2026-08-29）**：`Cargo.toml` 里留了一个指向已删源码的 `[[bin]]`，
# `cargo test` 直接报 `can't find bin`——**而门禁全绿**。
# 一个连编都编不过的证据仓库，比没有证据更糟：它看起来还在。
set -uo pipefail
R=research
[[ -f "$R/Cargo.toml" ]] || { echo "  ✓ 没有 $R/Cargo.toml，本阶段无对象可判"; exit 0; }
command -v cargo >/dev/null || { echo "  ✗ 没有 cargo，装不了就没法验 research 的证据"; exit 1; }

out="$(cd "$R" && cargo test --release 2>&1)"
rc=$?
if (( rc != 0 )); then
  echo "  ✗ research 的构建或单测没过（cargo test 退出码 $rc）"
  grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'
  echo "     → 怎么办：先修好——kb 里的实测结论全靠它们背书，编不过就等于那些数字今天没有来源。"
  exit 1
fi
n="$(grep '^test result' <<<"$out" | sed 's/test result: ok\. \([0-9]*\) passed.*/\1/' | paste -sd+ | bc)"
b="$(grep -c '^test result: ok' <<<"$out")"
echo "  ✓ research 构建通过，$b 个测试批次、共 ${n:-?} 个单测全绿"

# ── 本地腿的字词损坏闸，它自己会不会红 ──
# 三方论证的本地腿靠 `ask-local.sh` 里那道闸挡损坏输出，而那道闸此前没人验过。
# 实测（2026-09-03）：一份含 `inaccessibleisabled` 的输出被判绿，差点当成证据用掉。
OOV="$R/scripts/oov-check.py"
if [[ -f "$OOV" ]]; then
  if ! out="$(python3 "$OOV" --selftest 2>&1)"; then
    echo "$out" | sed 's/^/  /'
    echo "     → 怎么办：本地腿的损坏闸判错了样本，修 splice_of 的规则再跑。"
    echo "                闸不准 ⇒ 三方论证里那一腿的输出可信度归零。"
    exit 1
  fi
  echo "$out" | sed 's/^  /  /'
fi
