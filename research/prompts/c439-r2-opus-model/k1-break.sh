#!/usr/bin/env bash
# K1：调用方环境里留着一个弄坏开关（LIB_CHANGED_PATHS_BREAK=fallback，changed-paths.sh 第 50、76 行），
# gate_diff_base 不认 GATE_BASE，庚二的窗口被收窄，而守卫只看 GATE_BASE 与 --force。用法同 k1-binary.sh
source "$(dirname "$0")/lib.sh" "$1" "$2" "$3"
for arm in control break_switch_left_in_environment; do
  echo "=== arm=$arm"
  new_repo "$WORK/repo-break"
  git update-ref refs/sop/staged-green "$(green_tree_commit "$(git write-tree)")"; echo "step=introduce"
  printf '\nE7RESULT name=beta value=2\n' >> .claude/kb/experiments/900-x.md; git add -A; git commit -qm bypass; echo "step=commit_L_without_gate"
  printf 'note\n' > .claude/kb/other.md; git add -A; echo "step=stage_unrelated"
  if [[ $arm == control ]]; then geng2_gate; else geng2_gate LIB_CHANGED_PATHS_BREAK=fallback; fi
  git commit -qm unrelated 2>/dev/null; echo "step=commit"
  printf 'note2\n' >> .claude/kb/other.md; git add -A; echo "step=stage_unrelated"
  geng2_gate; git commit -qm unrelated2 2>/dev/null
  cd "$WORK"; rm -rf "$WORK/repo-break"
done
