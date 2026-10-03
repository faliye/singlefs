#!/usr/bin/env bash
# E2：Verus 判定还取决于 tool-versions 那一行没看的东西吗。直接起 Verus 验一份三行的玩具文件（不是被判的 crate；
# lib_heavy_tests.py 文件头：直接起的 verus 不算重型），每次经内存包装与 capped.sh 4；不跑门禁、不跑判法脚本。
# 用法：bash e2-verus-toy.sh <仓根> <草稿目录>
#   V1 原样；V2 VERUS_Z3_PATH 设成空串；V3 VERUS_Z3_PATH=z3（裸名）且 PATH 最前面放一个记日志再转给真 z3 的 z3；
#   V4 安装目录顶层文件拷一份（A），V5 同一份只把 libverus_builtin_macros.so 换成截断的（B）。
set -uo pipefail
REPO="${1:?仓根}"; WORK="${2:?草稿目录}"
INSTALL="${SINGLEFS_VERUS_ROOT:-${XDG_DATA_HOME:-$HOME/.local/share}/$(awk -F'\t' '$1=="release"{print $5}' "$REPO/.claude/gate.d/verus-pins.tsv")}"
mkdir -p "$WORK/toy" "$WORK/path-z3"
printf 'use vstd::prelude::*;\nverus! {\nproof fn one_plus_one() ensures 1 + 1 == 2 {}\n}\n' > "$WORK/toy/toy.rs"
printf '#!/bin/sh\necho "PATH 里的 z3 被起了：$*" >> "%s/z3-calls.log"\nexec "%s/z3" "$@"\n' "$WORK" "$INSTALL" > "$WORK/path-z3/z3"; chmod +x "$WORK/path-z3/z3"
if [[ ! -d "$WORK/verus-A" ]]; then
  mkdir -p "$WORK/verus-A"; find "$INSTALL" -maxdepth 1 -type f -exec cp -p {} "$WORK/verus-A/" \;
  mkdir -p "$WORK/verus-B"; cp -p "$WORK/verus-A/"* "$WORK/verus-B/"
  head -c 4096 "$INSTALL/libverus_builtin_macros.so" > "$WORK/verus-B/libverus_builtin_macros.so"
fi
run() {  # $1 标签 $2 安装目录，其余是 env 赋值
  local label="$1" dir="$2"; shift 2
  : > "$WORK/z3-calls.log"
  local out rc
  out="$(cd "$WORK/toy" && env "$@" RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方腿的玩具 Verus 探针，与同一会话派出去的别的 agent 各跑各的" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="同一个键下几种安装与 z3 换着跑，峰值各不相同" bash "$REPO/research/scripts/run-with-memory-cap.sh" 500M bash "$REPO/research/scripts/capped.sh" 4 nice -n 19 \
         "$dir/verus" --no-cheating --crate-type=lib --crate-name toy --num-threads 2 --output-json --rlimit 10 toy.rs 2>"$WORK/$label.err")"; rc=$?
  printf '%s\t退 %s\tstdout：%s\tstderr 末行：%s\tPATH 里的 z3 被起了 %s 次\n' "$label" "$rc" "$(printf '%s' "$out" | tr -d '\n' | cut -c1-160)" \
    "$(grep -v '^\s*$' "$WORK/$label.err" | tail -1 | cut -c1-160)" "$(grep -c . "$WORK/z3-calls.log")"
}
run V1-原样 "$INSTALL" -u VERUS_Z3_PATH
run V2-空串 "$INSTALL" VERUS_Z3_PATH=
run V3-裸名z3 "$INSTALL" VERUS_Z3_PATH=z3 PATH="$WORK/path-z3:$PATH"
run V4-拷贝A "$WORK/verus-A" -u VERUS_Z3_PATH
run V5-拷贝B-宏库截断 "$WORK/verus-B" -u VERUS_Z3_PATH
