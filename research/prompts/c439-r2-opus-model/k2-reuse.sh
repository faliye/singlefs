#!/usr/bin/env bash
# K2：庚二下 stage-must-run 的复用一侧。判法用 scripts/gate/admission.py gate-reuse 本体（工具副本，sha256 见 tools-sha256.txt），
# 阶段本身不跑：阶段在那一趟判出什么照它文件头写的行为摆进去（见各段注释）。用法同 k1-binary.sh
source "$(dirname "$0")/lib.sh" "$1" "$2" "$3"
reuse() {   # reuse <阶段>：打 admission.py gate-reuse 的判定（退 0 要跑、退 10 可跳过）
  local out rc; out="$(SINGLEFS_STAGED_TREE="$(git write-tree)" python3 "$TOOLS/scripts/gate/admission.py" gate-reuse "$PWD" "$1" 2>&1)"; rc=$?
  echo "  reuse stage=$1 rc=$rc verdict=$([[ $rc == 10 ]] && echo may_skip || echo must_run) reason=${out:0:160}"
}
advance() { git update-ref refs/sop/staged-green "$(green_tree_commit "$(git write-tree)")"; echo "  staged-green -> $(sg_tree | cut -c1-12)（$1）"; }
base_layout() {
  mkdir -p .claude/gate.d .claude/scripts litmus crates
  printf 'checker-tier-lkmm.sh\tlitmus/ .claude/scripts/lkmm.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/\n' > .claude/gate.d/stage-inputs.tsv
  printf 'checker-tier-crates-mutation-replay.sh\tcrates/ Cargo.toml\n' >> .claude/gate.d/stage-inputs.tsv
  printf '#!/usr/bin/env bash\n# 假阶段：只占登记里的名字\n' > .claude/gate.d/checker-tier-lkmm.sh
  printf '#!/usr/bin/env bash\n# 假阶段：只占登记里的名字\n' > .claude/gate.d/checker-tier-crates-mutation-replay.sh
  printf 'source "$(dirname "${BASH_SOURCE[0]}")/../singlefs-ai-sop/scripts/lib.sh"\n' > .claude/scripts/lkmm.sh
  printf 'C a\n{}\nP0(int *x) { WRITE_ONCE(*x, 1); }\nexists (x=1)\n' > litmus/a.litmus
  printf 'pub fn one() -> u32 { 1 }\n' > crates/a.rs; printf '[workspace]\n' > Cargo.toml
  mkdir -p .claude/singlefs-ai-sop/scripts   # 规范副本（被 .gitignore 挡着）：源码摘要要它的 comment-only-change.py 与 preflight.py
  cp "$TOOLS/.claude/singlefs-ai-sop/scripts/comment-only-change.py" "$TOOLS/.claude/singlefs-ai-sop/scripts/preflight.py" .claude/singlefs-ai-sop/scripts/
  git add -A; git commit -qm stages
}
echo "=== shape=ignored_registered_input（checker-tier-lkmm 登记的 .claude/singlefs-ai-sop/scripts/lib.sh 被 .gitignore 挡着）"
new_repo "$WORK/repo-reuse"; base_layout
mkdir -p .claude/singlefs-ai-sop/scripts; printf 'herd7_flags=v1\n' > .claude/singlefs-ai-sop/scripts/lib.sh
echo "  step=introduce lib.sh=v1 tracked=$(git ls-files .claude/singlefs-ai-sop | wc -l)"
reuse checker-tier-lkmm.sh; advance "整轮全绿：这一道在含 lib.sh v1 的临时树上判过"
printf 'herd7_flags=v2\n' > .claude/singlefs-ai-sop/scripts/lib.sh; printf 'note\n' > .claude/kb/other.md; git add -A
echo "  step=working_tree_lib.sh_v2（里层照冻结 gate.sh 第 91–93 行把它拷进临时树）; stage_unrelated"
reuse checker-tier-lkmm.sh
cd "$WORK"; rm -rf "$WORK/repo-reuse"
for arm in geng2_as_written stop_on_not_run; do
  echo "=== shape=not_run_rows_then_reuse arm=$arm"
  new_repo "$WORK/repo-reuse"; base_layout
  reuse checker-tier-crates-mutation-replay.sh; advance "整轮全绿，变异表每条都抓到"
  printf 'pub fn one() -> u32 { 2 }\n' > crates/a.rs; git add -A; echo "  step=stage_crates_change"
  reuse checker-tier-crates-mutation-replay.sh
  echo "  stage_run: 1 条变异点名的测试打 NOT_RUN_NO_GPU_ADAPTER 记本次未跑，整道退 0、全绿标记不写（scripts/mutation/crates-mutation-rows.py 文件头「本次未跑」）；gate.sh 记 PASS、本次未跑进汇总"
  if [[ $arm == geng2_as_written ]]; then advance "退 0、没人给 GATE_BASE、没带 --force"; else echo "  staged-green 不动（有本次未跑）"; fi
  git commit -qm crates-change; printf 'note\n' > .claude/kb/other.md; git add -A; echo "  step=stage_unrelated（有卡的那一趟）"
  reuse checker-tier-crates-mutation-replay.sh
  cd "$WORK"; rm -rf "$WORK/repo-reuse"
done
