#!/usr/bin/env bash
# T5：64 号三条判据在以后的写法上漏判什么、误拦什么（第二轮攻方报过的 11 种写法不再报）。
# 每一格一个临时根，.claude/gate.d/ 下只放一份样本阶段，跑快照树里的 64 号；「该」是这一格按 64 号头部判据的字面该得的结果。
# 用法：bash t5-64.sh <快照树> <草稿目录>
set -uo pipefail
SNAP="$(cd "$1" && pwd)"; D="$2"; rm -rf "$D"; mkdir -p "$D"
S64="$SNAP/.claude/gate.d/64-change-range-single-source.sh"
case_() {  # $1 = 名字，$2 = 该（红 / 绿），$3 = 扩展名，stdin = 样本阶段正文
  local root="$D/$1" rc=0 out
  mkdir -p "$root/.claude/gate.d"; cat > "$root/.claude/gate.d/50-sample.$3"
  out="$(nice -n 19 bash "$S64" "$root" 2>&1)" || rc=$?
  local got; got=$([[ $rc -eq 1 ]] && echo 红 || echo "绿(退$rc)")
  printf '  %-34s 该%s 实测%s%s\n' "$1" "$2" "$got" "$([[ "${got:0:1}" == "$2" ]] && echo '' || echo '   ← 判错')"
}
echo "== 漏判：阶段自己算基准 / 列路径不带 quotepath / 不判共用取法的退出码，64 号绿"
case_ 'python 用 os.getenv 读 GATE_BASE' 红 py <<'EOF'
import os, subprocess
base = os.getenv('GATE_BASE') or 'HEAD'
changed = subprocess.run(['git', '-c', 'core.quotepath=false', 'diff', '--name-only', base], capture_output=True, text=True).stdout
EOF
case_ '写死远端跟踪分支（origin 的 master）当基准' 红 sh <<'EOF'
changed="$(git -c core.quotepath=false diff --name-only origin/master -- )" || exit 1
EOF
case_ '写死 HEAD^ 当基准' 红 sh <<'EOF'
changed="$(git -c core.quotepath=false diff --name-only HEAD^ -- )" || exit 1
EOF
case_ '写死 HEAD~（不带数字，就是 HEAD~1）' 红 sh <<'EOF'
changed="$(git -c core.quotepath=false diff --name-only HEAD~ -- )" || exit 1
EOF
case_ 'python 拼出来的 HEAD~1' 红 py <<'EOF'
import subprocess
changed = subprocess.run(['git', '-c', 'core.quotepath=false', 'diff', '--name-only', 'HEAD' + '~1'], capture_output=True, text=True).stdout
EOF
case_ 'git status --porcelain 列路径、不带 quotepath' 红 sh <<'EOF'
changed="$(git status --porcelain | cut -c4-)" || exit 1
EOF
case_ 'git diff --numstat 列路径、不带 quotepath' 红 sh <<'EOF'
base="$(gate_diff_base gate)"
changed="$(git diff --numstat "$base" -- | cut -f3)" || exit 1
EOF
case_ '共用取法后接 || true' 红 sh <<'EOF'
changed="$(gate_changed_paths "$base" untracked)" || true
EOF
case_ '同一行的 || 属于后一条命令' 红 sh <<'EOF'
changed="$(gate_changed_paths "$base" untracked)"; [[ -n "$changed" ]] || echo "这次什么都没改"
EOF
echo
echo "== 误拦：判了退出码、或者只是文字里提到，64 号红"
case_ '调用折行，|| 在下一行' 绿 sh <<'EOF'
changed="$(gate_changed_paths "$base" \
  untracked)" || { echo "  ✗ 取不到"; echo "     → 修仓"; exit 1; }
EOF
case_ '下一行取 $? 判退出码' 绿 sh <<'EOF'
changed="$(gate_changed_paths "$base" untracked)"
status=$?
if (( status != 0 )); then echo "  ✗ 取不到"; exit 1; fi
EOF
case_ 'set -e 下的赋值（失败即退出）' 绿 sh <<'EOF'
set -euo pipefail
changed="$(gate_changed_paths "$base" untracked)"
EOF
case_ 'python 多行文档字符串中间一行' 绿 sh <<'EOF'
python3 - <<'PY'
def load(path):
    """读新增行。
    行来自共用脚本 gate_added_lines 交来的「路径<TAB>行文」，一行一条。
    """
    return open(path).read()
PY
EOF
case_ 'heredoc 里的用法说明' 绿 sh <<'EOF'
usage() {
  cat <<'TXT'
取法：changed=$(gate_changed_paths <基准> untracked)，基准见 merge-base 那一段
TXT
}
EOF
