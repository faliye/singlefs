#!/usr/bin/env bash
# K2 ⑤：naming-lint.sh 接 | head 时被 SIGPIPE 杀掉（没跑完），整条退 0。在 <草稿目录>/sigpipe-root 造一棵 4000 个单字母加数字函数名的小树，跑完删掉。
# 用法：bash sigpipe.sh <草稿目录> <仓根（只读：从这里起 naming-lint.sh）>
set -uo pipefail
work="$1"; repo="$2"; fake="$work/sigpipe-root"
rm -rf "${fake:?}"; mkdir -p "$fake/.git" "$fake/crates/core/src" "$fake/.claude"
python3 -I -c "import sys; open(sys.argv[1], 'w').write(''.join(f'pub fn x{i}() {{}}\n' for i in range(4000)))" "$fake/crates/core/src/lib.rs"
cd "$repo" || exit 2
bash .claude/singlefs-ai-sop/scripts/naming-lint.sh "$fake" > "$work/sigpipe-full.txt" 2>&1; full=$?
echo "整条跑完：退 $full，输出 $(wc -l < "$work/sigpipe-full.txt") 行"
bash .claude/singlefs-ai-sop/scripts/naming-lint.sh "$fake" 2>&1 | head -2 > /dev/null; echo "接 | head -2：PIPESTATUS=${PIPESTATUS[*]}（naming-lint 退 141 是被 SIGPIPE 杀掉，整条的退出码是 head 的）"
rm -rf "${fake:?}" "$work/sigpipe-full.txt"
