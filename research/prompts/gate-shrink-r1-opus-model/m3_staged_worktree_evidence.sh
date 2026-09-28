#!/usr/bin/env bash
# 模型 M3（gate-shrink-r1 云端攻方）：「扫仓的门禁只在提交前由 gate-staged.sh 起的整轮跑」之后，
# 34 号 evidence-in-repo 那一格（kb 里拿 /tmp 路径当依据、装置改了没留新产物）还有没有人判。
# 做法：拿 34 号自己的红样本 evidence-in-repo-red，照 stage-selftest.sh 拷进临时目录、跑 setup.sh，
#   ① 在样本仓本身上跑 --check evidence-in-repo（与样本自检同一种跑法）；
#   ② 照 gate.sh --staged 的做法（worktree add --detach HEAD，再把 git diff --cached 套上去）建一个链接 worktree，
#      在它上面跑同一格。git 写只发生在临时目录里的样本仓，不碰本仓。
# 用法：bash m3_staged_worktree_evidence.sh <仓根> <输出目录>
set -uo pipefail
repository_root="$(cd "${1:?仓根}" && pwd)"
output_directory="${2:?输出目录}"
mkdir -p "$output_directory"
stage="$repository_root/.claude/gate.d/34-doc-experiment-pages-and-products.sh"
sample="$repository_root/.claude/gate.d/fixtures/34-doc-experiment-pages-and-products.sh/evidence-in-repo-red"
work_directory="$(mktemp -d "${TMPDIR:-/tmp}/m3-work.XXXXXX")"
cp -a "$sample/." "$work_directory/sample/"
( cd "$work_directory/sample" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash setup.sh >/dev/null 2>&1 ) || { echo "setup.sh 没跑成"; exit 1; }
plain_exit=0
( cd "$work_directory/sample" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE -u GATE_NOT_RUN_FILE bash "$stage" "$work_directory/sample" --force --check evidence-in-repo ) > "$output_directory/m3-plain.log" 2>&1 || plain_exit=$?
# 照 gate.sh --staged：暂存全部改动，HEAD 上建链接 worktree，再把暂存区的差套上去
git -C "$work_directory/sample" add -A >/dev/null 2>&1
git -C "$work_directory/sample" worktree add --detach "$work_directory/staged" HEAD >/dev/null 2>&1 || { echo "worktree add 失败"; exit 1; }
git -C "$work_directory/sample" diff --cached --binary | git -C "$work_directory/staged" apply --index >/dev/null 2>&1 || { echo "套暂存区的差失败"; exit 1; }
staged_exit=0
( cd "$work_directory/staged" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE -u GATE_NOT_RUN_FILE bash "$stage" "$work_directory/staged" --force --check evidence-in-repo ) > "$output_directory/m3-staged.log" 2>&1 || staged_exit=$?
tmp_evidence_file="$(grep -rlF '/tmp/claude-1000/sample-round/report.md' "$work_directory/staged/.claude/kb" 2>/dev/null | head -1)"
echo "plain_exit=$plain_exit staged_exit=$staged_exit tmp_evidence_line_present_in_staged_tree=$([[ -n "$tmp_evidence_file" ]] && echo yes || echo no)"
echo "plain /tmp lines: $(grep -c '/tmp/' "$output_directory/m3-plain.log")  staged /tmp lines: $(grep -c '/tmp/claude-1000/sample' "$output_directory/m3-staged.log")"
git -C "$work_directory/sample" worktree remove --force "$work_directory/staged" >/dev/null 2>&1
rm -rf "${work_directory:?}"
