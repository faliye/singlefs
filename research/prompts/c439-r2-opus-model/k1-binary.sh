#!/usr/bin/env bash
# K1：页里有 NUL 字节（git 当二进制），庚二的窗口里一次都看不到它的新增行。用法：k1-binary.sh <冻结副本> <工具副本> <草稿根>
source "$(dirname "$0")/lib.sh" "$1" "$2" "$3"
new_repo "$WORK/repo-binary"
git update-ref refs/sop/staged-green "$(green_tree_commit "$(git write-tree)")"; echo "step=introduce staged-green=$(sg_tree | cut -c1-12)"
printf 'E7RESULT name=beta value=2\n\nraw: \0\n' > .claude/kb/experiments/901-y.md   # L：产物里是 value=1；一个 NUL 字节
git add -A; echo "step=stage_page_with_nul_and_L"
geng2_gate
echo "  full_judge（wu_cf 的全量判，同一棵树）："; judge_tree "$(git write-tree)" ""
git commit -qm page-with-nul; echo "step=commit"
printf 'E7RESULT name=beta value=2\n\nraw: (removed)\n' > .claude/kb/experiments/901-y.md; git add -A; echo "step=stage_remove_nul"
geng2_gate; git commit -qm remove-nul; echo "step=commit"
printf 'note\n' > .claude/kb/other.md; git add -A; echo "step=stage_unrelated"
geng2_gate; git commit -qm unrelated
echo "  full_judge（同一棵树）："; judge_tree "$(git write-tree)" ""
cd "$WORK"; rm -rf "$WORK/repo-binary"
