#!/usr/bin/env bash
# T7：31、52 号改成在 cd 之前取阶段自己的位置之后，哪一种调用方式取错位置。
# 现场：A = 快照树的拷贝（被判的 31、52 号与它们调的脚本）；B = 一份诱饵拷贝，脚本与生成器换成只打「DECOY」的；
#       F31、F52 = 两道阶段自己的 green 样本（外来的项目根）。对的结果：用 A 的脚本判 F，判绿。
# 每种调用方式跑快照版与 HEAD 版（git show 真仓，放在 A 的同一目录下，位置逻辑只看自己所在的目录）。
# 用法：bash t7-invocations.sh <快照树> <真仓> <草稿目录>
set -uo pipefail
SNAP="$(cd "$1" && pwd)"; REPO="$(cd "$2" && pwd)"; T="$3"; rm -rf "$T"; mkdir -p "$T/bin"
unset GATE_BASE GATE_DIFF_BASE GATE_STAGED_FROM CDPATH
cp -a "$SNAP" "$T/A"
for s in 31-blocking-verdict 52-segment-registry; do git -C "$REPO" show "HEAD:.claude/gate.d/$s.sh" > "$T/A/.claude/gate.d/$s.head.sh"; done
cp -a "$SNAP" "$T/B"
printf 'print("DECOY-52")\n' > "$T/B/research/scripts/check-segment-registry.py"
printf 'import sys\nprint("DECOY-31", file=sys.stderr)\nsys.exit(3)\n' > "$T/B/.claude/scripts/gen-decision-items.py"
cp -a "$SNAP/.claude/gate.d/fixtures/52-segment-registry.sh/green" "$T/F52"
cp -a "$REPO/.claude/gate.d/fixtures/31-blocking-verdict.sh/green" "$T/F31"   # 31 号的样本不在快照清单里，取真仓工作区那一份
ln -s "$T/A/.claude/gate.d/52-segment-registry.sh" "$T/bin/stage52"; ln -s "$T/A/.claude/gate.d/31-blocking-verdict.sh" "$T/bin/stage31"
ln -s "$T/A/.claude/gate.d/52-segment-registry.head.sh" "$T/bin/stage52h"; ln -s "$T/A/.claude/gate.d/31-blocking-verdict.head.sh" "$T/bin/stage31h"
ln -s "$T/A" "$T/Alink"
classify() {  # 输出 → 用的是哪一份
  if grep -q DECOY <<<"$1"; then echo 诱饵B; elif grep -qE '找不到 .*(check-segment-registry|gen-decision-items)' <<<"$1"; then echo 找不到;
  elif grep -qE '比对了 2 处登记|2 条未定项两把尺都判过' <<<"$1"; then echo 仓A判绿; else echo "其他：$(head -1 <<<"$1" | cut -c1-40)"; fi
}
try() {  # $1 = 名字，$2 = cwd，其余 = 命令（N = 阶段号、V = 空（快照版）或 .head（HEAD 版）由调用方展开）
  local name="$1" dir="$2"; shift 2
  local out rc=0; out="$(cd "$dir" && "$@" 2>&1)" || rc=$?
  printf '%s 退%s' "$(classify "$out")" "$rc"
}
row() {  # $1 = 名字，$2 = cwd，$3 = 命令模板（__S__ 换成阶段文件名，__F__ 换成外来根，__B__ 换成 bin 下的链接名）
  local line="  $(printf '%-44s' "$1")" s f v b cmd
  for s in 31-blocking-verdict 52-segment-registry; do
    f="$T/F${s%%-*}"
    for v in '' .head; do
      b="stage${s%%-*}$([[ -n "$v" ]] && echo h)"
      cmd="${3//__S__/$s$v.sh}"; cmd="${cmd//__F__/$f}"; cmd="${cmd//__B__/$b}"
      line+=" | ${s%%-*}$([[ -n "$v" ]] && echo HEAD || echo 快照) $(try "$1" "$2" bash -c "$cmd")"
    done
  done
  echo "$line"
}
echo "（每格：用的是哪一份脚本 / 生成器，退出码）"
row '绝对路径，另给项目根'                   /     'bash '"$T"'/A/.claude/gate.d/__S__ __F__'
row '在 A 里用相对路径，另给项目根'          "$T/A" 'bash .claude/gate.d/__S__ __F__'
row '在 B 里用 ../A 的相对路径，另给项目根'  "$T/B" 'bash ../A/.claude/gate.d/__S__ __F__'
row '经 bin/ 下的符号链接调，另给项目根'     /     'bash '"$T"'/bin/__B__ __F__'
row '经指向 A 的目录链接调，另给项目根'      /     'bash '"$T"'/Alink/.claude/gate.d/__S__ __F__'
row 'bash -c 里 source，另给项目根'          "$T/A" 'source .claude/gate.d/__S__ __F__'
row '从标准输入喂给 bash -s，另给项目根'     "$T/A" 'bash -s __F__ < .claude/gate.d/__S__'
row '导出 CDPATH=B，在 A 里相对路径调'       "$T/A" 'CDPATH='"$T"'/B bash .claude/gate.d/__S__ __F__'
row '导出 CDPATH=B，绝对路径调'              /     'CDPATH='"$T"'/B bash '"$T"'/A/.claude/gate.d/__S__ __F__'
