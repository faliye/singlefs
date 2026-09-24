#!/usr/bin/env bash
# T6（乙形态的第二次抽样）：仓里 check-segment-registry.py 的变异，逐个看三道东西判不判得出——
#   47 号跑的 --selftest、乙形态样本（上游 stage-selftest 跑 52 号的 red / green）、52 号对着一份埋了错的真输入。
# 用法：bash t6-mutants.sh <真仓根> <草稿目录>   只读真仓；拷一份最小现场到草稿目录（layout、replay.sh、E142 产物、第八节点名的 .rs、52 号与样本）。
set -uo pipefail
REPO="$(cd "$1" && pwd)"; D="$2"; rm -rf "$D"; mkdir -p "$D"
unset GATE_BASE GATE_DIFF_BASE GATE_STAGED_FROM
SCRIPT=research/scripts/check-segment-registry.py
LAYOUT=.claude/kb/layout/01-first-txn.md
PRODUCT="research/results/$(awk -F'|' '$1=="E142"{print $4}' "$REPO/research/scripts/replay.sh")"
RS=$(awk '/^## 八、/{f=1;next} f&&/^## /{exit} f' "$REPO/$LAYOUT" | grep -oE '`[^`]+\.rs`' | tr -d '`' | sort -u)
mkroot() { local R="$1" f; mkdir -p "$R"; for f in "$LAYOUT" research/scripts/replay.sh "$PRODUCT" $RS "$SCRIPT"; do mkdir -p "$R/$(dirname "$f")"; cp "$REPO/$f" "$R/$f"; done; }
mkroot "$D/base"
mkdir -p "$D/base/.claude/gate.d/fixtures"; cp "$REPO/.claude/gate.d/52-segment-registry.sh" "$D/base/.claude/gate.d/"
cp -r "$REPO/.claude/gate.d/fixtures/52-segment-registry.sh" "$D/base/.claude/gate.d/fixtures/"
echo "现场：$PRODUCT；第八节点名的 .rs $(wc -w <<<"$RS") 份"

# 埋错：只动一行里的一处，python 断言恰好命中一次
plant() {  # $1 = 目标根，$2 = 缺陷名
  python3 - "$1" "$2" "$LAYOUT" "$PRODUCT" <<'PY'
import sys, pathlib
root, defect, layout_rel, product_rel = sys.argv[1:5]
layout = pathlib.Path(root, layout_rel); product = pathlib.Path(root, product_rel)
def edit_line(path, line_prefix, old, new):
    lines = path.read_text(encoding='utf-8').split('\n')
    hits = [i for i, l in enumerate(lines) if l.startswith(line_prefix) and old in l]
    assert len(hits) == 1 and lines[hits[0]].count(old) == 1, (defect, len(hits))
    lines[hits[0]] = lines[hits[0]].replace(old, new)
    path.write_text('\n'.join(lines), encoding='utf-8')
row = '| 第一次可写挂载取号'
if defect == '取号行种类':      edit_line(layout, row, '`[system_configuration_slot×2]`', '`[system_configuration_slot×3]`')
elif defect == '取号行操作数':  edit_line(layout, row, '2 次操作', '3 次操作')
elif defect == '取号行状态数':  edit_line(layout, row, '4 个崩溃状态', '5 个崩溃状态')
elif defect == '取号行没写种类': edit_line(layout, row, '，种类 `[system_configuration_slot×2]`', '')
elif defect == '整条流状态数':  edit_line(layout, '⚠️ 发布与下一次发布之间没有屏障', '262165 个状态', '262166 个状态')
elif defect == '普通发布段序列': edit_line(layout, '| 普通发布（第一个事务）', '`16+2+1+2`', '`16+2+1+3`')
elif defect == '产物缺取号行':
    lines = product.read_text(encoding='utf-8').split('\n')
    keep = [l for l in lines if not l.startswith('E7RESULT name=segments path=instance_acquisition ')]
    assert len(keep) == len(lines) - 1; product.write_text('\n'.join(keep), encoding='utf-8')
elif defect != '无': raise SystemExit('unknown defect ' + defect)
PY
}
mutate() {  # $1 = 目标脚本，$2 = 变异名
  python3 - "$1" "$2" <<'PY'
import sys
path, name = sys.argv[1:3]
text = open(path, encoding='utf-8').read()
table = {
  'M0 main 不传退出码（第一轮的变异）': ('    sys.exit(exit_code)\n', '    sys.exit(0)\n'),
  'M1 产物行不比种类': ("        if entry.step_kinds_text != product_entry['step_kinds_text']:\n", '        if False:\n'),
  'M2 产物行不比操作数': ("        if entry.operation_count is not None and entry.operation_count != product_entry['operation_count']:\n", '        if False:\n'),
  'M3 产物行不比状态数': ("        if entry.crash_state_count is not None and entry.crash_state_count != product_entry['closed_form_crash_state_count']:\n", '        if False:\n'),
  'M4 不比整条流那句': ('    return table_entries + [merged_stream_entry]\n', '    return table_entries\n'),
  'M5 产物里没有这条 path 就跳过': ('        if product_entry is None:\n', '        if product_entry is None:\n            continue\n'),
  'M6 没写种类也放过': ('        if entry.step_kinds_text is None:\n', '        if False:\n'),
  'M7 只比第一行与整条流': ('    for entry in entries:\n', '    for entry in entries[:1] + entries[-1:]:\n'),
  'M8 产物行不比段序列': ("        if entry.segment_sequence_text != product_entry['segment_sequence_text']:\n", '        if False:\n'),
}
old, new = table[name]
assert text.count(old) == 1, (name, text.count(old))
open(path, 'w', encoding='utf-8').write(text.replace(old, new))
PY
}
judge52() { local out rc=0; out="$(bash "$1/.claude/gate.d/52-segment-registry.sh" "$2" 2>&1)" || rc=$?; echo "$rc"; }
while IFS='|' read -r mutant defect; do
  M="$D/m"; rm -rf "$M"; cp -r "$D/base" "$M"
  [[ "$mutant" == 无 ]] || mutate "$M/$SCRIPT" "$mutant"
  st=0; (cd "$M" && python3 "$SCRIPT" --selftest >/dev/null 2>&1) || st=$?
  fx_out="$(nice -n 19 bash "$REPO/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" "$M/.claude/gate.d" 2>&1)"; fx=$?
  fx_wrong="$(grep -cE '✗ .*(期望|找不到)' <<<"$fx_out")"
  B="$D/defect"; rm -rf "$B"; mkroot "$B"; plant "$B" "$defect"
  clean_rc=$(judge52 "$D/base" "$B"); m_rc=$(judge52 "$M" "$B")
  printf '%-30s | 47 --selftest 退 %s | 样本自检退 %s（判错 %s 条）| 埋错「%s」：原脚本 52 号退 %s，变异后 52 号退 %s\n' \
    "$mutant" "$st" "$fx" "$fx_wrong" "$defect" "$clean_rc" "$m_rc"
done <<'LIST'
无|无
M0 main 不传退出码（第一轮的变异）|普通发布段序列
M1 产物行不比种类|取号行种类
M2 产物行不比操作数|取号行操作数
M3 产物行不比状态数|取号行状态数
M4 不比整条流那句|整条流状态数
M5 产物里没有这条 path 就跳过|产物缺取号行
M6 没写种类也放过|取号行没写种类
M7 只比第一行与整条流|普通发布段序列
M8 产物行不比段序列|普通发布段序列
LIST

# ── 改法（本腿提的，只在这份模型上量过、被攻过零轮）：红样本每条比对分支各放一行错，expect 逐条 want ──
echo
echo "== 改法：乙形态的红样本改成五行、每行一种错（段序列 / 种类 / 操作数 / 状态数 / 产物里没有这条 path）"
P="$D/base/.claude/gate.d/fixtures/52-segment-registry.sh/red"
cat > "$P/.claude/kb/layout/01-first-txn.md" <<'MD'
## 八、合成登记表

五行加整条流（`path=a / b / c / d / e / m`）。

| 路径 | 段序列 | 出处 | 层 0 |
|---|---|---|---|
| 甲路径 | `3+1`，3 次操作、5 个崩溃状态，种类 `[x×2]\|[y]` | 产物 `name=segments path=a` | 无 |
| 乙路径 | `1+1`，2 次操作、3 个崩溃状态，种类 `[x]\|[z]` | 产物 `name=segments path=b` | 无 |
| 丙路径 | `1+1`，3 次操作、3 个崩溃状态，种类 `[x]\|[y]` | 产物 `name=segments path=c` | 无 |
| 丁路径 | `1+1`，2 次操作、4 个崩溃状态，种类 `[x]\|[y]` | 产物 `name=segments path=d` | 无 |
| 戊路径 | `1+1`，2 次操作、3 个崩溃状态，种类 `[x]\|[y]` | 产物 `name=segments path=e` | 无 |

⚠️ 整条流 `1+1`、3 个状态（种类 `[x]|[y]`）。

## 九、下一节
MD
{ printf 'E7RESULT name=segments path=a operations=3 segments=2+1 closed_form=5 kinds=[x×2]|[y]\n'
  for p in b c d m; do printf 'E7RESULT name=segments path=%s operations=2 segments=1+1 closed_form=3 kinds=[x]|[y]\n' "$p"; done; } > "$P/research/results/syn.out"
cat > "$P/expect" <<'EX'
exit=1
want=对不上，共 5 处
want=甲路径（path=a）：kb 写的段序列是 `3+1`，产物里是 `2+1`
want=乙路径（path=b）：kb 写的种类是
want=丙路径（path=c）：kb 写的操作次数是 3，产物里 operations 是 2
want=丁路径（path=d）：kb 写的崩溃状态数是 4，产物里 closed_form 是 3
want=戊路径：登记表引用 path=e，产物 syn.out 里没有这个 path 的 name=segments 行
EX
while IFS='|' read -r mutant; do
  M="$D/m"; rm -rf "$M"; cp -r "$D/base" "$M"; [[ "$mutant" == 无 ]] || mutate "$M/$SCRIPT" "$mutant"
  fx_out="$(nice -n 19 bash "$REPO/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" "$M/.claude/gate.d" 2>&1)"; fx=$?
  printf '  %-30s | 改过的样本自检退 %s（判错 %s 条）\n' "$mutant" "$fx" "$(grep -cE '✗ .*(期望|找不到)' <<<"$fx_out")"
done <<'LIST'
无
M0 main 不传退出码（第一轮的变异）
M1 产物行不比种类
M2 产物行不比操作数
M3 产物行不比状态数
M4 不比整条流那句
M5 产物里没有这条 path 就跳过
M7 只比第一行与整条流
M8 产物行不比段序列
LIST
