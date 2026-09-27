#!/usr/bin/env bash
# 攻方改法 F1 F2 F3 F5 F10（只在我的模型上量过、被攻过零轮）套到 hooks 的副本上：<草稿目录>/fixcopy/ 里拷真仓 .claude/hooks 的三份，
# .claude/scripts、.claude/singlefs-ai-sop、.claude/gate.d 与 research/ 链到真仓；lib_heavy_tests.py 打 fix-lib-heavy-tests.diff。真仓不动。
# 用法：bash make_fix_copy.sh <仓根> <草稿目录>；打印副本里 heavy-test-guard.sh 的路径。
set -euo pipefail
repository="$1"; scratch="$2"; here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
copy="$scratch/fixcopy"
rm -rf -- "${copy:?}"
mkdir -p "$copy/.claude/hooks"
cp -p "$repository/.claude/hooks/heavy-test-guard.sh" "$repository/.claude/hooks/lib_shell_words.py" "$repository/.claude/hooks/lib_heavy_tests.py" "$copy/.claude/hooks/"
for name in scripts singlefs-ai-sop gate.d; do ln -s "$repository/.claude/$name" "$copy/.claude/$name"; done
ln -s "$repository/research" "$copy/research"
patch -s "$copy/.claude/hooks/lib_heavy_tests.py" < "$here/fix-lib-heavy-tests.diff"
echo "$copy/.claude/hooks/heavy-test-guard.sh"
