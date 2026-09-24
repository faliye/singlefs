#!/usr/bin/env bash
# T5：64 号两条判据的漏判与误判。每种写法各放一份阶段文件，跑真仓现在的 64 号，看它点名哪几份。
# 用法：bash t5-64.sh <真仓根> <草稿目录>   只读真仓；阶段文件全写在草稿目录下的假仓里。
set -uo pipefail
REPO="$(cd "$1" && pwd)"; R="$2/fake"; rm -rf "$R"; mkdir -p "$R/.claude/gate.d"; G="$R/.claude/gate.d"
w() { cat > "$G/$1"; }
# ── 漏判：自己算了基准或列了会转义的路径，64 号不点名 ──
w 01-三点号隐含merge-base.sh <<'X'
changed="$(git -c core.quotepath=false diff --name-only @{u}... --)"
X
w 02-基准写死HEAD的上一个.sh <<'X'
changed="$(git -c core.quotepath=false diff --name-only HEAD~1 --)"
X
w 03-参数列表换行引号开头.py <<'X'
import subprocess
names = subprocess.run(['git', 'diff',
                        '--name-only', base, '--'], capture_output=True, text=True).stdout
X
w 04-quotepath写成true.sh <<'X'
changed="$(git -c core.quotepath=true ls-files --others --exclude-standard)"
X
w 05-包装函数名就叫git.py <<'X'
import subprocess
def git(*args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], capture_output=True, text=True)
untracked = subprocess.run(['git', 'ls-files', '--others', '--exclude-standard'], capture_output=True, text=True).stdout
X
w 06-同一行有test的-z.sh <<'X'
[[ -z "${SKIP:-}" ]] && git ls-files --others --exclude-standard > "$list"
X
w 07-printf开头.sh <<'X'
printf '%s\n' "$(git ls-files --others --exclude-standard)" > "$list"
X
w 08-上游短写.sh <<'X'
base="$(git rev-parse @{u})"
X
# ── 误判：没算改动范围基准，64 号照样点名 ──
w 21-嵌套跑样本前清环境.sh <<'X'
unset GATE_BASE GATE_STAGED_FROM
X
w 22-核登记的提交在历史里.sh <<'X'
git merge-base --is-ancestor "$recorded_commit" HEAD || bad "登记的提交不在当前历史里"
X
w 23-报上次全绿在哪.sh <<'X'
last_green="$(git rev-parse -q --verify refs/sop/gate-ok || true)"
X
out="$(cd "$R" && bash "$REPO/.claude/gate.d/64-change-range-single-source.sh" "$R" 2>&1)"; rc=$?
echo "64 号退出码 $rc"
for f in "$G"/*; do
  b="$(basename "$f")"; n="$(grep -cF "$b:" <<<"$out")"
  printf '  %-28s 64 号点名 %s 行\n' "$b" "$n"
done
