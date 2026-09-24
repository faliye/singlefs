#!/usr/bin/env bash
# T6：五行红样本之后，还有哪些 check-segment-registry.py 的变异让仓里的脚本退化，而 47 号的 --selftest 与 52 号样本一起放过。
# 第二轮报过的 M0–M8 不再报（这里只跑 M1 当对照）；N1–N10 是这一轮的新变异，各配一处埋在真输入里、原脚本判红的错。
# 用法：bash t6-mutants.sh <快照树> <真仓> <草稿目录> [<改过的红样本目录>]
#   快照树给被判的脚本、52 号与样本；真仓给脚本读的其余输入（E142 产物、第八节点名的 .rs），只读。
#   第四个参数给了，就把 52 号的红样本换成那一份再跑一遍（改法，见 t6-fixture-v2.sh）。
set -uo pipefail
SNAP="$(cd "$1" && pwd)"; REPO="$(cd "$2" && pwd)"; D="$3"; NEW_RED="${4:-}"
rm -rf "$D"; mkdir -p "$D"
unset GATE_BASE GATE_DIFF_BASE GATE_STAGED_FROM
SCRIPT=research/scripts/check-segment-registry.py
LAYOUT=.claude/kb/layout/01-first-txn.md
PRODUCT="research/results/$(awk -F'|' '$1=="E142"{print $4}' "$SNAP/research/scripts/replay.sh")"
RS=$(awk '/^## 八、/{f=1;next} f&&/^## /{exit} f' "$SNAP/$LAYOUT" | grep -oE '`[^`]+\.rs`' | tr -d '`' | sort -u)
from() { [[ -e "$SNAP/$1" ]] && echo "$SNAP/$1" || echo "$REPO/$1"; }   # 快照里有的取快照，没有的取真仓
mkroot() { local R="$1" f; mkdir -p "$R"; for f in "$LAYOUT" research/scripts/replay.sh "$PRODUCT" $RS "$SCRIPT"; do mkdir -p "$R/$(dirname "$f")"; cp "$(from "$f")" "$R/$f"; done; }
mkroot "$D/base"; mkdir -p "$D/base/.claude/gate.d/fixtures"
cp "$SNAP/.claude/gate.d/52-segment-registry.sh" "$D/base/.claude/gate.d/"
cp -r "$SNAP/.claude/gate.d/fixtures/52-segment-registry.sh" "$D/base/.claude/gate.d/fixtures/"
[[ -z "$NEW_RED" ]] || { rm -rf "$D/base/.claude/gate.d/fixtures/52-segment-registry.sh/red"; cp -r "$NEW_RED" "$D/base/.claude/gate.d/fixtures/52-segment-registry.sh/red"; }
echo "现场：$PRODUCT；第八节点名的 .rs $(wc -w <<<"$RS") 份；输入的 sha256："
(cd "$D/base" && sha256sum "$LAYOUT" "$PRODUCT" $RS "$SCRIPT" .claude/gate.d/52-segment-registry.sh | sed 's/^/  /')

plant() {  # $1 = 目标根，$2 = 缺陷名；每一处只改一行里的一处，断言恰好命中一次
  python3 - "$1" "$2" "$LAYOUT" "$PRODUCT" <<'PY'
import sys, pathlib
root, defect, layout_rel, product_rel = sys.argv[1:5]
layout = pathlib.Path(root, layout_rel); product = pathlib.Path(root, product_rel)
mkfs_test = pathlib.Path(root, 'crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs')
zero_test = pathlib.Path(root, 'crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs')
def edit_line(path, line_prefix, old, new):
    lines = path.read_text(encoding='utf-8').split('\n')
    hits = [i for i, l in enumerate(lines) if l.startswith(line_prefix) and old in l]
    assert len(hits) == 1 and lines[hits[0]].count(old) == 1, (defect, len(hits))
    lines[hits[0]] = lines[hits[0]].replace(old, new)
    path.write_text('\n'.join(lines), encoding='utf-8')
def edit_all(path, old, new):
    text = path.read_text(encoding='utf-8'); assert old in text, defect; path.write_text(text.replace(old, new), encoding='utf-8')
if defect == '取号行种类':        edit_line(layout, '| 第一次可写挂载取号', '`[system_configuration_slot×2]`', '`[system_configuration_slot×3]`')
elif defect == '整条流状态数':    edit_line(layout, '⚠️ 发布与下一次发布之间没有屏障', '262165 个状态', '262166 个状态')
elif defect == '第二条流两段对调': edit_line(layout, '⚠️ 第二条流', '的段序列 `2+2+1+2+2+1+18+2+1+18', '的段序列 `2+2+2+1+2+1+18+2+1+18')
elif defect == '第二条流写数':    edit_line(layout, '⚠️ 第二条流', '279 次写', '280 次写')
elif defect == '用例丢了状态数':  edit_all(zero_test, '2104413', '2104414')
elif defect == '钉 mkfs 的用例搬走': mkfs_test.rename(mkfs_test.with_name('first_transaction_mkfs_moved.rs'))
elif defect == '钉 mkfs 的用例函数改名': edit_all(mkfs_test, 'fn recorded_stream_matches_the_registered_mkfs_segment_sequence(', 'fn recorded_stream_matches_mkfs(')
elif defect == 'mkfs 行出处丢了用例': edit_line(layout, '| mkfs 种根', '`crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs` 的 ', '')
elif defect == '取号行段序列丢了反引号': edit_line(layout, '| 第一次可写挂载取号', '⇒ `2`，2 次操作', '⇒ 2，2 次操作')
elif defect == '产物取号行没有 kinds':
    edit_line(product, 'E7RESULT name=segments path=instance_acquisition ', ' kinds=[system_configuration_slot×2]', '')
elif defect == '声明多出一条路径':
    edit_line(layout, '一条线的提交协议', '`path=instance_acquisition / warm_up / transaction / post_mkfs_stream`', '`path=instance_acquisition / warm_up / transaction / post_mkfs_stream / extra`')
elif defect != '无': raise SystemExit('unknown defect ' + defect)
PY
}
mutate() {  # $1 = 目标脚本，$2 = 变异名；旧串必须恰好命中一次
  python3 - "$1" "$2" <<'PY'
import sys
path, name = sys.argv[1:3]
text = open(path, encoding='utf-8').read()
I8, I12, I16 = ' ' * 8, ' ' * 12, ' ' * 16
table = {
  'M1 产物行不比种类（第二轮，对照）': (I8 + "if entry.step_kinds_text != product_entry['step_kinds_text']:\n", I8 + 'if False:\n'),
  'N1 整条流不比状态数': (I8 + 'crash_state_count=merged_crash_state_count,\n', I8 + 'crash_state_count=None,\n'),
  'N2 第二条流不核用例里的数组': (I8 + 'if array_count == 0:\n', I8 + 'if False:\n'),
  'N3 第二条流不核写数': (I8 + 'if sum(segment_lengths) != claimed_operation_count:\n', I8 + 'if False:\n'),
  'N4 第二条流不核用例里的状态数': (I8 + 'if str(claimed_crash_state_count) not in normalized_test_text:\n', I8 + 'if False:\n'),
  'N5 钉活代码的用例文件不在就放过': ("        return [f'{entry.row_label}：出处指的用例文件 {file_match.group(1)} 不存在']\n", '        return []\n'),
  'N6 用例里找不到函数就放过': ("        return [f'{entry.row_label}：出处指的用例文件里找不到 `fn {function_match.group(1)}`']\n", '        return []\n'),
  'N7 出处既无 path 也无用例就放过': (I12 + 'if pinned is None:\n' + I16 + 'mismatches.append(', I12 + 'if pinned is None:\n' + I16 + 'continue\n' + I16 + 'mismatches.append('),
  'N8 不是预想却没有段序列就放过': (I8 + 'if entry.segment_sequence_text is None:\n' + I12 + 'mismatches.append(', I8 + 'if entry.segment_sequence_text is None:\n' + I12 + 'continue\n' + I12 + 'mismatches.append('),
  'N9 产物行没有 kinds 就放过': (I8 + "if product_entry['step_kinds_text'] is None:\n" + I12 + 'mismatches.append(', I8 + "if product_entry['step_kinds_text'] is None:\n" + I12 + 'continue\n' + I12 + 'mismatches.append('),
  'N10 声明路径剩不下恰好一条也照认第一条': ('    if len(remaining_paths) != 1:\n', '    if len(remaining_paths) < 1:\n'),
}
old, new = table[name]
assert text.count(old) == 1, (name, text.count(old))
open(path, 'w', encoding='utf-8').write(text.replace(old, new))
PY
}
judge52() { local rc=0; (cd "$2" && nice -n 19 bash "$1/.claude/gate.d/52-segment-registry.sh" "$2" >/dev/null 2>&1) || rc=$?; echo "$rc"; }
echo
printf '%-36s | 47 号 --selftest | 52 号样本自检（判错几条） | 埋错（原脚本 52 号 → 变异后 52 号）\n' 变异
while IFS='|' read -r mutant defect; do
  M="$D/m"; rm -rf "$M"; cp -r "$D/base" "$M"
  [[ "$mutant" == 无 ]] || mutate "$M/$SCRIPT" "$mutant"
  st=0; (cd "$M" && nice -n 19 python3 "$SCRIPT" --selftest >/dev/null 2>&1) || st=$?
  fx_out="$(nice -n 19 bash "$SNAP/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" "$M/.claude/gate.d" 2>&1)"; fx=$?
  B="$D/defect"; rm -rf "$B"; mkroot "$B"; plant "$B" "$defect"
  printf '%-36s | 退 %s | 退 %s（%s 条）| 「%s」：%s → %s\n' "$mutant" "$st" "$fx" "$(grep -cE '✗ .*(期望|找不到)' <<<"$fx_out")" \
    "$defect" "$(judge52 "$D/base" "$B")" "$(judge52 "$M" "$B")"
done <<'LIST'
无|无
M1 产物行不比种类（第二轮，对照）|取号行种类
N1 整条流不比状态数|整条流状态数
N2 第二条流不核用例里的数组|第二条流两段对调
N3 第二条流不核写数|第二条流写数
N4 第二条流不核用例里的状态数|用例丢了状态数
N5 钉活代码的用例文件不在就放过|钉 mkfs 的用例搬走
N6 用例里找不到函数就放过|钉 mkfs 的用例函数改名
N7 出处既无 path 也无用例就放过|mkfs 行出处丢了用例
N8 不是预想却没有段序列就放过|取号行段序列丢了反引号
N9 产物行没有 kinds 就放过|产物取号行没有 kinds
N10 声明路径剩不下恰好一条也照认第一条|声明多出一条路径
LIST
