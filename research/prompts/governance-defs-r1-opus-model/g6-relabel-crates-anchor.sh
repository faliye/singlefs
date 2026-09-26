#!/usr/bin/env bash
# G6/kb-scribe 模型：D23 第 4 条从已定翻成未定时，照改后 kb-scribe.md 第 3 步跑 relabel-item.py（调它自己的 relabel()，root 指到草稿里的仓副本），
# 看 crates/ 里被改写的那一行是不是 crates/mutations.tsv 某条变异的锚点、33 号在副本里判不判红。只改副本，不碰主仓。
# 用法：bash g6-relabel-crates-anchor.sh <草稿目录>
set -uo pipefail
REPO=/home/user/singlefs
SCRATCH=${1:?草稿目录}
copy="$(mktemp -d -p "$SCRATCH" g6.XXXX)/repo"; mkdir -p "$copy"
tar -C "$REPO" --exclude=./target --exclude=./.git --exclude=./research/target -cf - . | tar -C "$copy" -xf -
decision="$copy/.claude/kb/decisions/23-journal的角色与格式.md"
# 翻状态：已定项表里删掉第 4 行，另起「### 未定项」一节放它（load_map 认表格行）
python3 - "$decision" <<'PY'
import re, sys
path = sys.argv[1]; text = open(path, encoding='utf-8').read()
row = re.search(r'^\|\s*4\s*\|.*\n', text, re.M)
text = text[:row.start()] + text[row.end():]
text = text.replace('\n## 历史版本', '\n### 未定项\n\n| 4 | 记录头能不能约束在单个原子单元内（模型里翻成未定） | 未定 |\n\n## 历史版本', 1)
open(path, 'w', encoding='utf-8').write(text)
PY
anchor_row=463
anchor_file="$(awk -F'\t' -v n=$anchor_row 'NR==n {print $2}' "$copy/crates/mutations.tsv")"
anchor_count() { python3 - "$copy/crates/mutations.tsv" "$anchor_row" "$copy" <<'PY'
import sys
table, row, root = sys.argv[1], int(sys.argv[2]), sys.argv[3]
fields = open(table, encoding='utf-8').read().split('\n')[row - 1].split('\t')
original = fields[2].replace('\\n', '\n')
print(open(f'{root}/{fields[1]}', encoding='utf-8').read().count(original))
PY
}
echo "crates/mutations.tsv 第 $anchor_row 行：变异名「$(awk -F'\t' -v n=$anchor_row 'NR==n {print $1}' "$copy/crates/mutations.tsv")」，文件 $anchor_file"
echo "翻状态之前锚点命中次数：$(anchor_count)"
( cd "$copy" && python3 - "$copy" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location('relabel', 'research/scripts/relabel-item.py'); module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
code, review = module.relabel(sys.argv[1], 'D23', 4, dry_run=False)
print('relabel 退出码', code)
PY
) | grep -E 'crates/|relabel 退出码|✓' | head -20
echo "翻状态之后锚点命中次数：$(anchor_count)"
echo "── 副本里跑 33 号（末 6 行）──"
( cd "$copy" && bash .claude/gate.d/33-mutation-tables.sh 2>&1 | tail -n 6 ); echo "33 号退出码=${PIPESTATUS[0]}"
