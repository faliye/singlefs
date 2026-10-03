#!/usr/bin/env bash
# 辩方 U4 探针：把钩子三份文件拷进「路径尾部不是 .claude/hooks」的平目录，直接跑它们的 --selftest，看草稿里自证起不起得来。
# 起得来或起不来都只读本仓；用法：bash probe-flat-draft.sh <草稿目录>（已存在拒绝）
set -uo pipefail
scratch="${1:?用法：probe-flat-draft.sh <草稿目录>}"
repository="${SINGLEFS_REPOSITORY:-/home/fy5090/code/singlefs}"
[[ ! -e "$scratch" ]] || { echo "草稿目录已存在：$scratch"; exit 2; }
mkdir -p "$scratch/flat/hooks"
cp "$repository/.claude/hooks/heavy-test-guard.sh" "$repository/.claude/hooks/lib_heavy_tests.py" "$repository/.claude/hooks/lib_shell_words.py" "$repository/.claude/hooks/selftest-registry.tsv" "$scratch/flat/hooks/"
ln -s "$repository/.claude/scripts" "$scratch/flat/scripts"
ln -s "$repository/.claude/singlefs-ai-sop" "$scratch/flat/singlefs-ai-sop"
echo "## lib_heavy_tests.py --selftest（平目录）"
nice -n 19 python3 "$scratch/flat/hooks/lib_heavy_tests.py" --selftest > "$scratch/lib.out" 2>&1; echo "退出码 $?"; head -3 "$scratch/lib.out" | cut -c1-200 | sed "s#$scratch#<草稿>#g"
echo "## heavy-test-guard.sh --selftest（平目录）"
nice -n 19 bash "$scratch/flat/hooks/heavy-test-guard.sh" --selftest > "$scratch/guard.out" 2>&1; echo "退出码 $?"; head -2 "$scratch/guard.out" | cut -c1-220 | sed "s#$scratch#<草稿>#g"
