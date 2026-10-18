#!/usr/bin/env bash
# C439 正推腿：H1（漏判的形）与 H3（多会话）的历史脚本。每个历史一个函数 hist_<名> <候选>：建仓、走步骤、打出目标行进没进窗口。
# 用法：FROZEN=… WORK=… bash histories.sh [历史名…]      不给名字就全跑
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$HERE/lib.sh"

prelude() {   # prelude <历史名> <候选>：c0 之后各来一趟整轮绿（带与不带 --staged），乙再记一次起点；目标行 Lbad_A
  new_repo "$1-$2"
  if [[ "$2" == yi ]]; then git update-ref refs/sop/round-start HEAD; fi
  EXEMPT_FILE=""
  if [[ "$3" != no-initial-N ]]; then run_gate N "$2" 初始整轮; fi
  run_gate S "$2" 初始带暂存
  if [[ "$2" == ding ]]; then git notes --ref=sop-green add -f -m green HEAD; fi    # 丁的起点：基线提交先挂一次标记
}
report() {   # report <目标 tag…>：打出每个目标行是否进过窗口
  local t; for t in "$@"; do printf '    %-8s 进过窗口：%s\n' "$t" "$(ever_in_window "$t")"; done
}

# 1 原形：同一轮里先写（绕过门禁提交）、整轮全绿让 gate-ok 前移、再继续
hist_h1a_original() {
  prelude h1a "$1" with-N
  add_line a Lbad_A; commit_all c1-绕过
  add_line a Lok_1; run_gate N "$1" 整轮绿让gate-ok前移 ; commit_all c2-绕过
  add_line a Lok_2; stage_all; run_gate S "$1" 提交前带暂存
  commit_staged c3; add_line a Lok_3; run_gate N "$1" 之后不带暂存
}
# 1 原形的紧接版：整轮绿让 gate-ok 前移到 HEAD 之后马上带暂存再跑（中间没有别的提交）
hist_h1a_immediate() {
  prelude h1ai "$1" with-N
  add_line a Lbad_A; commit_all c1-绕过
  add_line a Lok_1; run_gate N "$1" 整轮绿让gate-ok前移
  add_line a Lok_2; stage_all; run_gate S "$1" 紧接着带暂存
}
# 1 原形的对照：同样但中途没有那一趟整轮绿
hist_h1a_nomove() {
  prelude h1a0 "$1" with-N
  add_line a Lbad_A; commit_all c1-绕过
  add_line a Lok_1; commit_all c2-绕过
  add_line a Lok_2; stage_all; run_gate S "$1" 提交前带暂存
  commit_staged c3; add_line a Lok_3; run_gate N "$1" 之后不带暂存
}
# 2 绕过门禁的提交：从没跑过不带 --staged 的整轮（gate-ok 不存在），两个绕过的提交之后才跑
hist_h1b_bypass() {
  prelude h1b "$1" no-initial-N
  add_line a Lbad_A; commit_all c1-绕过
  add_line a Lok_1; commit_all c2-绕过
  add_line a Lok_2; stage_all; run_gate S "$1" 提交前带暂存
  commit_staged c3; add_line a Lok_3; run_gate N "$1" 之后不带暂存
}
# 2 绕过的对照：只有一个绕过的提交
hist_h1b_bypass_one() {
  prelude h1b1 "$1" no-initial-N
  add_line a Lbad_A; commit_all c1-绕过
  add_line a Lok_2; stage_all; run_gate S "$1" 提交前带暂存
  commit_staged c3; add_line a Lok_3; run_gate N "$1" 之后不带暂存
}
# 3 一次暂存分几次提交：先暂存第一份、跑门禁、提交；后面几份直接分次提交，不再跑
hist_h1c_split() {
  prelude h1c "$1" no-initial-N
  add_line a Lok_p1; stage_all; run_gate S "$1" 第一份带暂存; commit_staged c1
  add_line b Lbad_A; stage_all; commit_staged c2-第二份没跑
  add_line a Lok_p3; stage_all; commit_staged c3-第三份没跑
  add_line a Lok_p4; stage_all; commit_staged c4-第四份没跑
  add_line a Lok_n; stage_all; run_gate S "$1" 下一批提交前带暂存
}
# 3 的另一种：全部暂存好再跑一趟，再分四次提交（目标行好的版本也要进窗口过一次）
hist_h1c_allstaged() {
  prelude h1cs "$1" no-initial-N
  add_line a Lok_p1; add_line b Lok_A; add_line a Lok_p3
  stage_all; run_gate S "$1" 全部暂存后一趟
  git reset -q; git add .claude/kb/a.md; commit_staged c1; git add -A; commit_staged c2
  add_line a Lok_n; stage_all; run_gate S "$1" 下一批提交前带暂存
}
# 4 另一个会话在两次门禁之间提交（共一个暂存区）：A 跑绿，B 暂存自己的错行并提交（带走 A 的暂存），A 再提交一次不重跑
hist_h1d_interleave() {
  prelude h1d "$1" no-initial-N
  add_line a Lok_A1; stage_all; run_gate S "$1" A带暂存
  add_line b Lbad_A; stage_all; commit_staged cB-绕过
  add_line a Lok_A2; stage_all; commit_staged cA2-没重跑
  add_line a Lok_A3; stage_all; run_gate S "$1" A下一批带暂存
}
# 5 全推出去之后：一个绕过的提交，推出去（上游指到 HEAD），之后两条路各跑一趟
hist_h1e_pushed_one() {
  prelude h1e1 "$1" no-initial-N
  add_line a Lbad_A; commit_all c1-绕过
  set_upstream HEAD
  add_line a Lok_n; run_gate N "$1" 推完不带暂存
  stage_all; run_gate S "$1" 推完带暂存
}
# 5 的变体：两个绕过的提交，推出去
hist_h1e_pushed_two() {
  prelude h1e2 "$1" no-initial-N
  add_line a Lbad_A; commit_all c1-绕过
  add_line a Lok_1; commit_all c2-绕过
  set_upstream HEAD
  add_line a Lok_n; run_gate N "$1" 推完不带暂存
  stage_all; run_gate S "$1" 推完带暂存
}
# 6 只跑不带 --staged 的整轮（staged-gate.md 许的「跑全量工作区」）：两个绕过的提交之后跑
hist_h1f_n_only() {
  prelude h1f "$1" with-N
  add_line a Lbad_A; commit_all c1-绕过
  add_line a Lok_1; commit_all c2-绕过
  add_line a Lok_n; run_gate N "$1" 只跑不带暂存
}
H1_HISTORIES=(h1a_original h1a_immediate h1a_nomove h1b_bypass h1b_bypass_one h1c_split h1c_allstaged h1d_interleave h1e_pushed_one h1e_pushed_two h1f_n_only)
H1_TARGET=(Lbad_A Lbad_A Lbad_A Lbad_A Lbad_A Lbad_A Lok_p3 Lbad_A Lbad_A Lbad_A Lbad_A)

# ── H3：多会话 ────────────────────────────────────────────
# M1 乙的起点被另一个会话挪动：A 开轮、A 的绕过提交里有错行，B 随后开自己的轮（同一条 ref），A 再跑
hist_m1_roundmove() {
  prelude m1 "$1" no-initial-N
  add_line a Lbad_A; commit_all c1-A绕过
  if [[ "$1" == yi ]]; then git update-ref refs/sop/round-start HEAD; fi    # B 开轮：覆盖同一条 ref
  add_line b Lok_B1; commit_all c2-B
  add_line a Lok_A2; stage_all; run_gate S "$1" A带暂存
}
# M2 丙/丁的最后写者赢：A 绿在含 a1 的树上，B 把 a1 退出暂存后绿在不含它的树上，A 再暂存 a1
hist_m2_lastwriter() {
  prelude m2 "$1" no-initial-N
  add_line a Lok_a1; stage_all; run_gate S "$1" A带暂存含a1
  git reset -q -- .claude/kb/a.md                     # B 把 A 的暂存退掉（共一个暂存区）
  add_line b Lok_b1; git add .claude/kb/b.md; run_gate S "$1" B带暂存不含a1
  git add .claude/kb/a.md; run_gate S "$1" A重新暂存a1
}
# M3 TOCTOU：A 跑绿后 B 往同一个暂存区放了错行，A 提交（带走 B 的错行），后面一个提交也没跑，再跑
hist_m3_toctou() {
  prelude m3 "$1" no-initial-N
  add_line a Lok_a1; stage_all; run_gate S "$1" A带暂存
  add_line b Lbad_A; git add .claude/kb/b.md               # B 在 A 跑完之后暂存
  commit_staged cA-带走了B的错行
  add_line a Lok_a2; stage_all; commit_staged cA2-没跑
  add_line a Lok_a3; stage_all; run_gate S "$1" 下一批带暂存
}
# M5 改写历史（amend / rebase 之后提交号全换）：乙的起点与丁的标记挂在旧提交上，丙记的是树
hist_m5_rewrite() {
  prelude m5 "$1" no-initial-N
  local old_c0; old_c0="$(git rev-parse HEAD)"
  add_line a Lbad_A; commit_all c1-绕过
  local old_head; old_head="$(git rev-parse HEAD)"
  git reset -q --hard "$old_c0"
  GIT_COMMITTER_DATE="2000-01-01T00:00:00" git commit -q --amend --allow-empty --no-edit --date "2000-01-01T00:00:00"   # c0 换了提交号，树不变  # clock-times:allow 自检造历史用的固定提交时刻，是被测输入
  git cherry-pick "$old_head" >/dev/null                                                                              # c1 跟着换号
  add_line a Lok_1; commit_all c2-绕过
  add_line a Lok_2; stage_all; run_gate S "$1" 提交前带暂存
}
# M4 窗口把别的会话已判绿的行算进来：A 绿在含 a1 的树上，B 的树不含 a1 且绿，A 再跑时 a1 又进窗口
# （同 M2；这里只数 a1 被判了几次）
H3_HISTORIES=(m1_roundmove m3_toctou m5_rewrite)
H3_TARGET=(Lbad_A Lbad_A Lbad_A)

# ── H5：判别力自证（丙对今天）：两份会红的样本，选定取法必须进窗口，今天的取法必须漏掉 ──────
h5_selftest() {
  local sample cand expected got fails=0 checks=0
  for sample in h1a_original h1b_bypass; do
    for cand in bing_s bing_sn today; do
      "hist_$sample" "$cand" >/dev/null 2>&1; got="$(ever_in_window Lbad_A | awk "{print \$1}")"
      if [[ "$cand" == today ]]; then expected=no; else expected=yes; fi
      checks=$((checks+1))
      if [[ "$got" == "$expected" ]]; then echo "PASS $sample / $(cand_name "$cand")：Lbad_A 进过窗口=$got"; else echo "FAIL $sample / $(cand_name "$cand")：期望 $expected，实测 $got"; fails=$((fails+1)); fi
    done
  done
  echo "h5 自证查了 $checks 项，红 $fails 项"; ((fails == 0))
}

run_histories() {   # run_histories <名字数组> <目标数组>
  local -n hist_names="$1"; local -n hist_targets="$2"; local i cand
  for i in "${!hist_names[@]}"; do
    echo "== ${hist_names[$i]}（目标行 ${hist_targets[$i]}）"
    for cand in "${CANDS[@]}"; do
      "hist_${hist_names[$i]}" "$cand" >/dev/null 2>&1
      printf '  %-10s %s\n' "$(cand_name "$cand")" "$(ever_in_window "${hist_targets[$i]}")"
    done
  done
}
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  if [[ "${1:-}" == h5 ]]; then h5_selftest
  elif (($#)); then
    for name in "$@"; do
      for cand in "${CANDS[@]}"; do echo "── $name / $(cand_name "$cand")"; "hist_$name" "$cand" >/dev/null 2>&1; show_runs; done
    done
  else
    run_histories H1_HISTORIES H1_TARGET; run_histories H3_HISTORIES H3_TARGET
  fi
fi
