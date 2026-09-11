#!/usr/bin/env bash
# 取实验号并当场占住：查号与建第一个文件在同一步里做，建文件用排他方式（noclobber）。
#
#   bash research/scripts/claim-experiment.sh --next              # 打印下一个没人用的实验号
#   bash research/scripts/claim-experiment.sh E139 简称            # 占住 E139：建 research/prompts/e139-preregistration.md
#   bash research/scripts/claim-experiment.sh --selftest
#
# 为什么要有它：2026-09-11 两个会话前后几分钟都取了 E137——一边查过号、隔几轮对话才写文件，
# 另一边在这中间写了同名的未提交跑前登记，被 Write 整份盖掉（没进过 git），靠对话记录逐字复原。
# 查号与写文件分两步，中间就有窗口；这里把两步并成一步，写文件用 O_EXCL（set -C），
# 两个会话同时占同一个号时只有一个成功。
#
# 查号的范围：跑前登记、实验源码、产物、变异表、实验脚本、kb 正文与索引行——哪一处出现过这个号都算用过。
# 自证会红：--selftest 在临时目录里摆几处已用的号，确认 --next、「已用的号被拒」「同一个号第二次占被拒」；
# 再用 CLAIM_SKIP_USED_CHECK=1 关掉查号，已有源码的号被占成功、selftest 判红。
set -uo pipefail

used_numbers() {
  local root="$1"
  {
    ls "$root/research/prompts" 2>/dev/null | sed -n 's/^e\([0-9][0-9]*\)[-_].*/\1/p'
    ls "$root/research/scripts" 2>/dev/null | sed -n 's/^e\([0-9][0-9]*\)[-_].*/\1/p'
    ls "$root/research/e7-index-bench/src/bin" 2>/dev/null | sed -n 's/^e\([0-9][0-9]*\)_.*/\1/p'
    ls "$root/research/results" 2>/dev/null | sed -n 's/^e\([0-9][0-9]*\)-.*/\1/p'
    ls "$root/research/mutations" 2>/dev/null | sed -n 's/^e\([0-9][0-9]*\)_.*/\1/p'
    ls "$root/.claude/kb/experiments" 2>/dev/null | sed -n 's/^\([0-9][0-9]*\)-.*/\1/p'
    sed -n 's/^| E\([0-9][0-9]*\)（.*/\1/p' "$root/.claude/kb/experiments.md" 2>/dev/null
  } | sort -un
}

next_number() {
  local highest
  highest="$(used_numbers "$1" | tail -1)"
  echo "E$(( ${highest:-0} + 1 ))"
}

claim() {
  local root="$1" tag="$2" short_name="$3" number file
  if [[ ! "$tag" =~ ^E([0-9]+)$ ]]; then
    echo "  ✗ 实验号要写成 E<数字>，拿到的是「$tag」"
    echo "    → 例：bash research/scripts/claim-experiment.sh E139 某某的代价；不知道用几号先跑 --next"
    return 2
  fi
  number="${BASH_REMATCH[1]}"
  if [[ -z "$short_name" ]]; then
    echo "  ✗ 没给简称"
    echo "    → 简称写进跑前登记的标题；编号在别处引用时要带着它"
    return 2
  fi
  if [[ "${CLAIM_SKIP_USED_CHECK:-0}" != 1 ]] && grep -qx "$number" <<<"$(used_numbers "$root")"; then
    echo "  ✗ E$number 已经有人用了"
    echo "    → 取下一个：bash research/scripts/claim-experiment.sh --next"
    return 1
  fi
  file="$root/research/prompts/e$number-preregistration.md"
  mkdir -p "$root/research/prompts"
  if ! ( set -C; printf '# E%s 跑前登记：%s\n\n写于 %s JST，装置写之前。\n' "$number" "$short_name" \
         "$(TZ=Asia/Tokyo date '+%Y-%m-%d %H:%M')" > "$file" ) 2>/dev/null; then
    echo "  ✗ $file 已经存在（刚被别的会话占了）"
    echo "    → 取下一个：bash research/scripts/claim-experiment.sh --next"
    return 1
  fi
  echo "  ✓ 占住 E$number：$file"
}

selftest() {
  local root next rc_used rc_first rc_again
  root="$(mktemp -d)"
  mkdir -p "$root/research/prompts" "$root/research/e7-index-bench/src/bin" "$root/.claude/kb/experiments"
  : > "$root/research/prompts/e5-preregistration.md"
  : > "$root/research/e7-index-bench/src/bin/e6_sample.rs"
  : > "$root/.claude/kb/experiments/7-样本.md"
  printf '| E%s（%s） | 已跑 |\n' 8 样本 > "$root/.claude/kb/experiments.md"   # 拼出来写：源码里不留编号加简称的字面，免得被当成对 E8 的引用
  next="$(next_number "$root")"
  claim "$root" E6 样本 >/dev/null; rc_used=$?
  claim "$root" E9 样本 >/dev/null; rc_first=$?
  claim "$root" E9 样本 >/dev/null; rc_again=$?
  rm -rf "${root:?}"
  if [[ "${CLAIM_SKIP_USED_CHECK:-0}" == 1 ]]; then
    if [[ $rc_used -ne 0 ]]; then echo "selftest: 关掉查号之后已有源码的 E6 仍被拒 —— 检查坏了"; return 1; fi
    echo "selftest: 关掉查号确认判红（已有源码的 E6 被占成功）"; return 0
  fi
  [[ "$next" == E9 ]] || { echo "selftest: --next 该是 E9，拿到 $next"; return 1; }
  [[ $rc_used -eq 1 ]] || { echo "selftest: E6 已有源码却没被拒"; return 1; }
  [[ $rc_first -eq 0 ]] || { echo "selftest: E9 没人用却没占成"; return 1; }
  [[ $rc_again -eq 1 ]] || { echo "selftest: E9 第二次占也成功了"; return 1; }
  echo "selftest: 通过（--next 取到 E9；已有源码的 E6 被拒；E9 第一次占成、第二次被拒）"
}

case "${1:-}" in
  --selftest) selftest; exit $? ;;
  --next) next_number "${CLAIM_ROOT:-$(git rev-parse --show-toplevel)}"; exit 0 ;;
  "") echo "  ✗ 缺参数"; echo "    → bash research/scripts/claim-experiment.sh --next，或 … E<数字> 简称"; exit 2 ;;
  *) claim "${CLAIM_ROOT:-$(git rev-parse --show-toplevel)}" "$1" "${*:2}"; exit $? ;;
esac
