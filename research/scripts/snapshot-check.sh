#!/usr/bin/env bash
# admission: always 每一次调都拿此刻的工作区与 HEAD 比快照，上一次的结论不替这一次作保
# run-condition: command git sha256sum
# 三方开工快照的核对：快照里每个文件此刻还是不是那一版；不是的，说清是工作区未提交的改动还是快照之后的提交。
#
#   snapshot-check.sh <轮名> [--root 仓根]    # 读 research/prompts/<轮名>-snapshot/ 下每份 *sha256*.txt，逐行 sha256sum -c，对不上的再比 HEAD
#   snapshot-check.sh --selftest             # 造一个小仓走三条路；SNAPSHOT_CHECK_BREAK=ignore-mismatch 时必须判红
#
# 为什么：.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」要主 agent 写判决前拿快照 sha256sum -c 一遍，对不上的倒推；
# 2026-09-20 起主 agent 为这类核对敲了几百次 grep / sha256sum，每次都在 50 万上下文里。这一条命令把它做完，判决里抄它的输出。
# 输出每行一个文件：一致 / 工作区改了（HEAD 与快照一致，是别的会话未提交的改动）/ 提交改了（HEAD 也不是快照那一版）/ 文件不在。
# 退出码：0 全部一致；1 有对不上的；2 用法错、快照目录不在。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

die() { echo "  ✗ $1"; echo "  → 怎么办：$2"; exit "${3:-2}"; }

check_snapshot() {   # $1 仓根 $2 轮名；stdout 逐行结论，返回 0 全一致、1 有对不上
  local root="$1" round="$2" dir list line digest path actual head_digest checked=0 mismatched=0 verdict
  dir="$root/research/prompts/$round-snapshot"
  [[ -d "$dir" ]] || die "快照目录不在：$dir" "开工快照由 three-way-materials.py 写在 research/prompts/<轮>-snapshot/；没有就没法核" 2
  for list in "$dir"/*sha256*.txt; do
    [[ -f "$list" ]] || continue
    while IFS= read -r line || [[ -n "$line" ]]; do
      [[ "$line" =~ ^([0-9a-fA-F]{64})[[:space:]]+\*?(.+)$ ]] || continue
      digest="${BASH_REMATCH[1]}"; path="${BASH_REMATCH[2]}"; path="${path#./}"
      checked=$((checked+1))
      if [[ ! -f "$root/$path" ]]; then
        verdict="文件不在"
      else
        actual="$(sha256sum "$root/$path" | cut -c1-64)"
        if [[ "$actual" == "$digest" || "${SNAPSHOT_CHECK_BREAK:-}" == "ignore-mismatch" ]]; then
          verdict="一致"
        else
          head_digest="$(git -C "$root" show "HEAD:$path" 2>/dev/null | sha256sum | cut -c1-64)"
          if [[ "$head_digest" == "$digest" ]]; then verdict="工作区改了（HEAD 与快照一致：别的会话未提交的改动，腿引的行按 HEAD 那一版核）"
          else verdict="提交改了（HEAD 也不是快照那一版：快照之后有提交动过它，倒推要看 git log -- $path）"; fi
        fi
      fi
      [[ "$verdict" == "一致" ]] || mismatched=$((mismatched+1))
      echo "  $verdict  $path  快照 ${digest:0:12}"
    done < "$list"
  done
  [[ $checked -gt 0 ]] || die "快照目录里一行 sha256 都没读到：$dir" "快照清单每行是「<sha256>  <路径>」（sha256sum 的原样输出）" 2
  if [[ $mismatched -eq 0 ]]; then echo "  ✓ 快照 $round：$checked 个文件都与开工时一致"; return 0; fi
  echo "  ✗ 快照 $round：$checked 个文件里 $mismatched 个对不上（逐个列在上面）"
  echo "  → 怎么办：工作区改了的按 HEAD 那一版核腿引的行；提交改了的倒推出快照时的原样再核；判决开头写明哪个文件、被改了几处、腿引的行落没落在那几处"
  return 1
}

selftest() {
  local work failures=0 checked=0 out rc
  work="$(mktemp -d)"
  git -C "$work" init -q && git -C "$work" config user.email t@t && git -C "$work" config user.name t
  mkdir -p "$work/.claude/kb" "$work/research/prompts/r1-snapshot"
  printf 'a\n' > "$work/.claude/kb/one.md"; printf 'b\n' > "$work/.claude/kb/two.md"; printf 'c\n' > "$work/.claude/kb/three.md"
  git -C "$work" add -A && git -C "$work" commit -qm one
  (cd "$work" && sha256sum .claude/kb/one.md .claude/kb/two.md .claude/kb/three.md > research/prompts/r1-snapshot/kb-sha256.txt)
  out="$(check_snapshot "$work" r1)"; rc=$?; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '3 个文件都与开工时一致' <<<"$out"; then echo "  ✗ 自检：全一致时应当退 0，实际 $rc：$out"; failures=1; fi   # gate-lint:detail
  printf 'b2\n' > "$work/.claude/kb/two.md"; git -C "$work" add .claude/kb/two.md; git -C "$work" commit -qm two   # 提交改了（只提交这一份）
  printf 'a2\n' > "$work/.claude/kb/one.md"                    # 工作区改了、没提交
  rm "$work/.claude/kb/three.md"                                # 文件不在
  out="$(check_snapshot "$work" r1)"; rc=$?; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -q '工作区改了.*one.md' <<<"$out" || ! grep -q '提交改了.*two.md' <<<"$out" || ! grep -q '文件不在.*three.md' <<<"$out"; then
    echo "  ✗ 自检：三种对不上应当各判各的、退 1，实际 $rc：$out"; failures=1; fi   # gate-lint:detail
  out="$(check_snapshot "$work" nothere 2>&1)"; rc=$?; checked=$((checked+1))
  if [[ $rc -ne 2 ]]; then echo "  ✗ 自检：快照目录不在应当退 2，实际 $rc"; failures=1; fi   # gate-lint:detail
  rm -rf "${work:?}"
  if [[ $failures -ne 0 ]]; then echo "    → 怎么办：看 check_snapshot() 三种判定；SNAPSHOT_CHECK_BREAK=ignore-mismatch 设着的话这里本来就该红"; return 1; fi
  echo "  ✓ 自检通过（查了 $checked 种）：全一致退 0；工作区改了、提交改了、文件不在各判各的并退 1；快照目录不在退 2"
  return 0
}

if [[ "${1:-}" == "--selftest" ]]; then selftest; exit $?; fi
[[ $# -ge 1 && "$1" != --* ]] || die "用法：snapshot-check.sh <轮名> [--root 仓根]" "给三方轮名（快照在 research/prompts/<轮名>-snapshot/）"
round="$1"; root="$(cd "$here/../.." && pwd)"; shift
while [[ $# -gt 0 ]]; do case "$1" in --root) root="$(cd "${2:?}" && pwd)"; shift 2 ;; *) die "认不出参数 $1" "只认 --root <仓根>" ;; esac; done
check_snapshot "$root" "$round"
