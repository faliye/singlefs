#!/usr/bin/env bash
# 草稿：照 89 号的判法（退出码 + want 片段）对一个阶段跑它的 red / green 样本；阶段脚本可以指到一份改过的副本。
#   fixture-check.sh <样本目录（fixtures/<阶段文件名>）> <阶段脚本>
set -uo pipefail
fixtures="$1"; stage="$2"
for kind in red green; do
  sample="$fixtures/$kind"
  want_exit="$(sed -n 's/^exit=//p' "$sample/expect")"
  work="$(mktemp -d)"
  cp -a "$sample/." "$work/"
  if [[ -f "$work/setup.sh" ]]; then
    ( cd "$work" && env -u GATE_BASE -u GATE_STAGED_FROM bash setup.sh >/dev/null 2>&1 ) || { echo "  $kind: setup.sh 没跑成"; rm -rf "${work:?}"; continue; }
  fi
  output="$(cd "$work" && env -u GATE_BASE -u GATE_STAGED_FROM bash "$stage" "$work" 2>&1)"; got=$?
  rm -rf "${work:?}"
  verdict="判得对"
  [[ "$got" == "$want_exit" ]] || { verdict="判错"; echo "  $kind: 期望退出 $want_exit，实测 $got"; }
  while IFS= read -r want; do
    [[ -z "$want" ]] && continue
    grep -qF -- "$want" <<<"$output" || { verdict="判错"; echo "  $kind: 输出里找不到「$want」"; }
  done < <(sed -n 's/^want=//p' "$sample/expect")
  echo "  $kind: 退出码 $got（期望 $want_exit）→ $verdict"
  echo "  ---- $kind 原样输出 ----"
  sed 's/^/  | /' <<<"$output"
done
