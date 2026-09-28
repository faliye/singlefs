#!/usr/bin/env bash
# gate-shrink-r1 云端攻方（Opus）的模型复跑入口。只读仓里的门禁脚本与文件；git 写与改动只发生在草稿目录里的样本仓与仓副本。
# 用法：bash rerun.sh <仓根> <草稿目录>
# 各件的挂钟（2026-09-28 在本机量的一次）：M1 约 13 秒；M2、M4、M5 各不到 1 秒；M3 约 2 秒；M6 对 research/results/ 全部 299 份约二三十分钟（原样正则那一支慢，慢就是要量的东西）；M7 约 3 分钟（doc-lint 占两分钟）。
# 74 号那一条（B4）不在这里：它要编 release 的 singlefs-harness，命令单列在报告开头。
set -uo pipefail
repository_root="$(cd "${1:?仓根}" && pwd)"
scratch="${2:?草稿目录}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$scratch"
export TMPDIR="$scratch/tmp"; mkdir -p "$TMPDIR"
echo "== M1 doc-kb 五道的样本在别的四道上判什么"; bash "$here/m1_want_cross_hits.sh" "$repository_root" "$scratch/m1"
echo "== M2 声称有 --selftest 的四种认法"; python3 "$here/m2_selftest_claim_recognizers.py" "$repository_root"
echo "== M3 --staged 的链接 worktree 上 evidence-in-repo 判什么"; bash "$here/m3_staged_worktree_evidence.sh" "$repository_root" "$scratch/m3"
echo "== M4 去编号之后的阶段次序"; bash "$here/m4_stage_order.sh"
echo "== M5 治理文档里按全名指门禁"; bash "$here/m5_gate_name_references.sh" "$repository_root" "$scratch/m5"
echo "== M7 B1、B2、B3、B8 的违规输入"; bash "$here/m7_copy_injections.sh" "$repository_root" "$scratch/m7"
echo "== M6 term-renames 的掩码正则（慢）"
( cd "$repository_root/research/results" && ls ) | sed 's|^|research/results/|' > "$scratch/m6-list.txt"
python3 "$here/m6_term_renames_mask_regex.py" "$repository_root" $(cat "$scratch/m6-list.txt")
echo "== M8 按暂存树现算 batch-scope"; python3 "$here/m8_batch_scope_on_staged.py" "$repository_root"
echo "== M9 research 脚本自证逐条计时与分批并行（约十分钟）"; bash "$here/m9_research_selftest_timing.sh" "$repository_root" "$scratch/m9"
echo "== M6b 照 sweep-term --check 的走法逐份计时，报读、掩码、检测三段的合计（约一刻钟以上）"; python3 -u "$here/m6b_sweep_term_phase_totals.py" "$repository_root"
