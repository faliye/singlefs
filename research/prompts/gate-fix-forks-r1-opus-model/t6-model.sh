#!/usr/bin/env bash
# T6 模型：52 号的两种样本形态（甲：样本里放脚本拷贝；乙：阶段按自己的位置调仓里的脚本、--root 指合成样本），
# 在「仓里的脚本 main() 不传退出码」这一处变异下，门禁的样本自检（上游 stage-selftest.sh）抓不抓得到。
# 只读真仓；一切现场建在 $1（草稿目录）下的临时目录里。用法：bash t6-model.sh <真仓根> <草稿目录>
set -uo pipefail
REPO="$(cd "$1" && pwd)"; DRAFT="$2"
work="$(mktemp -d "$DRAFT/t6.XXXXXX")"
SCRIPT_REL=research/scripts/check-segment-registry.py
LAYOUT_REL=.claude/kb/layout/01-first-txn.md
PRODUCT=e142-first-txn-dry-run-2026-09-22-mkfs-zero-fill-resync.out
mapfile -t PINNED < <(awk '/^## 八、/,/^## 还剩/' "$REPO/$LAYOUT_REL" | grep -oE '`[^`]+\.rs`' | tr -d '`' | sort -u)

put_real_inputs() {  # $1 = 目标根；把 52 读的真输入拷进去
  mkdir -p "$1/.claude/kb/layout" "$1/research/scripts" "$1/research/results"
  cp "$REPO/$LAYOUT_REL" "$1/$LAYOUT_REL"
  cp "$REPO/research/scripts/replay.sh" "$1/research/scripts/replay.sh"
  cp "$REPO/research/results/$PRODUCT" "$1/research/results/$PRODUCT"
  for p in "${PINNED[@]}"; do [[ -f "$REPO/$p" ]] && { mkdir -p "$1/$(dirname "$p")"; cp "$REPO/$p" "$1/$p"; }; done
}
mutate_layout() {  # $1 = layout 文件；用脚本自己的 mutate_one_segment_number 改坏一个数字
  python3 - "$REPO/$SCRIPT_REL" "$1" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("csr", sys.argv[1]); m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
text = open(sys.argv[2], encoding='utf-8').read()
out = m.mutate_one_segment_number(text)[0]
open(sys.argv[2], 'w', encoding='utf-8').write(out)
PY
}
make_repo() {  # $1 = 假仓根；$2 = 甲|乙；$3 = clean|M
  local r="$1"
  mkdir -p "$r/.claude/gate.d/fixtures/52-segment-registry.sh"/{green,red} "$r/research/scripts"
  cp "$REPO/$SCRIPT_REL" "$r/$SCRIPT_REL"
  if [[ "$3" == M ]]; then
    python3 - "$r/$SCRIPT_REL" <<'PY'
import sys
p = sys.argv[1]; t = open(p, encoding='utf-8').read()
old = '    sys.exit(exit_code)\n'
assert t.count(old) == 1, t.count(old)
open(p, 'w', encoding='utf-8').write(t.replace(old, '    sys.exit(0)\n'))
PY
  fi
  local stage="$r/.claude/gate.d/52-segment-registry.sh"
  cp "$REPO/.claude/gate.d/52-segment-registry.sh" "$stage"
  local fx="$r/.claude/gate.d/fixtures/52-segment-registry.sh"
  if [[ "$2" == 甲 ]]; then
    for k in green red; do put_real_inputs "$fx/$k"; cp "$REPO/$SCRIPT_REL" "$fx/$k/$SCRIPT_REL"; done   # 样本里的脚本拷贝取「造样本那天」的干净版
    mutate_layout "$fx/red/$LAYOUT_REL"
  else
    python3 - "$stage" <<'PY'
import sys
p = sys.argv[1]; t = open(p, encoding='utf-8').read()
old_a = '[[ -f research/scripts/check-segment-registry.py ]] || { echo "  ! 找不到 research/scripts/check-segment-registry.py，本阶段跳过"; exit 77; }\n'
old_b = 'python3 research/scripts/check-segment-registry.py --root "$ROOT" || exit 1\n'
assert t.count(old_a) == 1 and t.count(old_b) == 1
new_a = 'SCRIPT="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/check-segment-registry.py"\n[[ -f "$SCRIPT" ]] || { echo "  ✗ 找不到 $SCRIPT"; echo "     → 怎么办：它随仓走，从 git 找回来。"; exit 1; }\n'
new_b = 'python3 "$SCRIPT" --root "$ROOT" || exit 1\n'
open(p, 'w', encoding='utf-8').write(t.replace(old_a, new_a).replace(old_b, new_b))
PY
    for k in green red; do   # 合成的最小输入：layout 八节（一行表 + 声明 + 整条流提示）、replay.sh 一行、产物两行
      mkdir -p "$fx/$k/.claude/kb/layout" "$fx/$k/research/scripts" "$fx/$k/research/results"
      printf 'E142|@x||syn.out|exact\n' > "$fx/$k/research/scripts/replay.sh"
      printf 'E7RESULT name=segments path=a operations=3 segments=2+1 closed_form=5 kinds=[x×2]|[y]\nE7RESULT name=segments path=b operations=2 segments=1+1 closed_form=3 kinds=[x]|[y]\n' > "$fx/$k/research/results/syn.out"
      n=2; [[ $k == red ]] && n=3
      cat > "$fx/$k/$LAYOUT_REL" <<MD
## 八、合成登记表

产物的 \`name=segments\` 两行（\`path=a / b\`）。

| 路径 | 段序列 | 出处 | 层 0 |
|---|---|---|---|
| 甲路径 | \`${n}+1\`，3 次操作、5 个崩溃状态，种类 \`[x×2]\\|[y]\` | 产物 \`name=segments path=a\` | 无 |

⚠️ 整条流 \`1+1\`、3 个状态（种类 \`[x]|[y]\`）。

## 九、下一节
MD
    done
  fi
  printf 'exit=0\n' > "$fx/green/expect"
  printf 'exit=1\nwant=对不上\n' > "$fx/red/expect"
}
for form in 甲 乙; do
  for variant in clean M; do
    r="$work/$form-$variant"; make_repo "$r" "$form" "$variant"
    out="$(bash "$REPO/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" "$r/.claude/gate.d" 2>&1)"; rc=$?
    echo "== 形态 $form，仓里脚本 $variant：stage-selftest 退出码 $rc"
    printf '%s\n' "$out" | grep -E '52-segment|样本判错|判得都对' | sed 's/^/   /'
  done
done
# 真跑一遍：仓里脚本是 M 时，52 号对一份改坏了的真 layout 判什么；47 号跑的 --selftest 判什么。
r="$work/real-M"; mkdir -p "$r/research/scripts" "$r/.claude/gate.d"
put_real_inputs "$r"; cp "$REPO/$SCRIPT_REL" "$r/$SCRIPT_REL"; cp "$REPO/.claude/gate.d/52-segment-registry.sh" "$r/.claude/gate.d/"
python3 - "$r/$SCRIPT_REL" <<'PY'
import sys
p = sys.argv[1]; t = open(p, encoding='utf-8').read()
open(p, 'w', encoding='utf-8').write(t.replace('    sys.exit(exit_code)\n', '    sys.exit(0)\n'))
PY
python3 "$r/$SCRIPT_REL" --selftest >/dev/null 2>&1; echo "== 仓里脚本 M：--selftest（47 号跑的那条）退出码 $?"
mutate_layout "$r/$LAYOUT_REL"
out="$(bash "$r/.claude/gate.d/52-segment-registry.sh" "$r" 2>&1)"; rc=$?
echo "== 仓里脚本 M、layout 改坏一个段序列数字：52 号退出码 $rc；输出含「对不上」：$(grep -c '对不上' <<<"$out")"
echo "现场：$work"
