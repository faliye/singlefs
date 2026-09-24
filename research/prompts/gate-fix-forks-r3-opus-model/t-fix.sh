#!/usr/bin/env bash
# 本腿提的改法（只在这份模型上量过、被攻过零轮）：把快照树拷一份，定点改 75、61 号与共用脚本，再拿改过的那棵树当「快照树」
# 重跑 t1-scenes、t5-config-amend、t5-states，看打中的那几格还中不中、原来判对的格有没有变。每一处替换断言旧串恰好命中一次。
#   T1-B  75 号「决策正文引了它」按 doc-lint G 认的写法认（粗体、反引号、空格、半角括号），另起一条正则，不动 ③ 用的 E_REF
#   T1-D  基准里这个路径没有页时，按实验号在基准那棵树里找它原来那页的标题（搬家不算「改成已跑」）
#   T1-C  「决策文件在这一批里」收成「这一批在那份决策里新增或改写的行点到了这个实验号」
#   T5-F/G 75 号自己的新增行、61 号的「碰过的行」两处 git diff 带 --no-color --no-ext-diff
#   T5-S  共用脚本：HEAD 还没出生时基准取空树（与上游 lib.sh 的 changed_files 同一种处置）
# 用法：bash t-fix.sh <快照树> <真仓> <草稿目录>
set -uo pipefail
SNAP="$(cd "$1" && pwd)"; REPO="$(cd "$2" && pwd)"; D="$3"; HERE="$(cd "$(dirname "$0")" && pwd)"
rm -rf "$D"; mkdir -p "$D"; cp -a "$SNAP" "$D/tree"
python3 - "$D/tree" <<'PY'
import sys, pathlib
root = pathlib.Path(sys.argv[1])
def sub(rel, old, new):
    p = root / rel; t = p.read_text(encoding='utf-8'); assert t.count(old) == 1, (rel, old[:40], t.count(old))
    p.write_text(t.replace(old, new), encoding='utf-8')
S75 = '.claude/gate.d/75-decision-experiment-links.sh'
sub(S75, "D_REF = re.compile(r'(?<![A-Za-z0-9])D(\\d+)（')\n",
    "D_REF = re.compile(r'(?<![A-Za-z0-9])D(\\d+)（')\nMENTION_REF = re.compile(r'(?<![A-Za-z0-9._-])[*`]*E(\\d+)[*`]*\\s*[（(]')\n")
sub(S75, "'mentions': {int(e) for e in E_REF.findall('\\n'.join(body))}}", "'mentions': {int(e) for e in MENTION_REF.findall('\\n'.join(body))}}")
sub(S75, "def base_title(rel):\n", "def base_title(rel, number=None):\n")
sub(S75, "    shown = git('show', f'{base}:{rel}')\n    if shown.returncode != 0:\n        return None\n",
    "    shown = git('show', f'{base}:{rel}')\n    if shown.returncode != 0 and number is not None:\n"
    "        found = git('grep', '-h', '-m1', '-E', f'^## E{number} ', base, '--', '.claude/kb/experiments/')\n"
    "        hit = next((l[l.find('## E'):] for l in found.stdout.split('\\n') if '## E' in l), None)\n"
    "        return hit if found.returncode == 0 else None\n    if shown.returncode != 0:\n        return None\n")
sub(S75, "not ran_status(base_title(rel)):", "not ran_status(base_title(rel, number)):")
sub(S75, "                if info['path'] not in changed and decision not in reviewed:\n",
    "                touched_mention = info['path'] in changed and any(\n"
    "                    text and re.search(r'(?<![A-Za-z0-9])E%d(?![0-9])' % number, text) for _, text in added_lines(info['path']))\n"
    "                if not touched_mention and decision not in reviewed:\n")
sub(S75, "diff = git('diff', '--no-renames', '-U0', base, '--', rel).stdout", "diff = git('diff', '--no-renames', '--no-color', '--no-ext-diff', '-U0', base, '--', rel).stdout")
sub('.claude/gate.d/61-settled-same-file.sh', 'touched="$(git diff -U0 "$BASE" -- "$f" \\', 'touched="$(git diff --no-color --no-ext-diff -U0 "$BASE" -- "$f" \\')
sub('research/scripts/changed-paths.sh', '  local mode="$1" merge_base\n',
    '  local mode="$1" merge_base\n'
    '  if ! git rev-parse --verify -q HEAD >/dev/null 2>&1; then git hash-object -t tree /dev/null; return 0; fi   # HEAD 未出生：空树\n')
PY
echo "改过的树：$D/tree（改动见上面的脚本；共用脚本自证 $(cd "$D/tree" && nice -n 19 bash research/scripts/changed-paths.sh --selftest >/dev/null 2>&1; echo "退 $?")）"
for model in t1-scenes t5-config-amend t5-states; do
  echo; echo "######## $model（改过的树）"
  nice -n 19 bash "$HERE/$model.sh" "$D/tree" "$REPO" "$D/$model" 2>&1
done
echo; echo "######## 样本自检：原快照树与改过的树，各自的 gate.d 逐个样本（10 号两棵树一样红：它读的 lib-owed.py 不在快照清单里）"
for tree in "$SNAP" "$D/tree"; do
  echo "== $([[ "$tree" == "$SNAP" ]] && echo 原快照树 || echo 改过的树)"
  nice -n 19 bash "$SNAP/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" "$tree/.claude/gate.d" 2>&1 | grep -E '✗|判得对' | sed 's/^ *//' | sort
done
