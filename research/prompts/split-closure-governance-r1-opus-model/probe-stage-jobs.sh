#!/usr/bin/env bash
# 探针：门禁入口的格并行缺省取法 stage_cells_default_jobs（.claude/gate.d/lib/stage-cells.sh）与登记位点名的 shell 取法
#   heavy_thread_default（scripts/hosts/heavy-threads.sh），在同几份登记文件、同几个 SINGLEFS_THREAD_CAP 下各交什么。
#   判的是 .claude/heavy-thread-exempt 新登的五行 STAGE_CELLS_JOBS 是不是「确实不是跑法」：两边逐格同值才算只是认不出包装。
# 用法：probe-stage-jobs.sh <冻结副本> <输出目录>
#   冻结副本里要有 .claude/gate.d/lib/stage-cells.sh 与 scripts/hosts/heavy-threads.sh；输出目录要已存在。
# 输出：每格一行 name=stage_jobs_case case=<格名> cap=<值或 unset> stage_out=<值> stage_rc=<码> heavy_out=<值> heavy_rc=<码> agree=<yes|no>，
#   末行 name=stage_jobs_summary cases=<格数> disagree=<两边不同值或不同退出码的格数>。
# 退出码：0 跑完（只打读数，不判对错）；2 用法错、冻结副本缺文件或抽不出函数；3 有样本进程没收干净。
set -euo pipefail

if (($# != 2)); then
  echo "用法：probe-stage-jobs.sh <冻结副本> <输出目录>" >&2
  exit 2
fi
frozen="$1"
output="$2"
stage_library="$frozen/.claude/gate.d/lib/stage-cells.sh"
heavy_library="$frozen/scripts/hosts/heavy-threads.sh"
[[ -f "$stage_library" && -f "$heavy_library" && -d "$output" ]] || { echo "  ✗ 冻结副本缺 stage-cells.sh 或 heavy-threads.sh，或输出目录不在" >&2; exit 2; }

# 只抽 stage_cells_default_jobs 那一个函数（整份 source stage-cells.sh 会带进别的全局设置）；抽出来放回同一相对路径，函数里 BASH_SOURCE 推的登记位置不变
function_text="$(awk '/^stage_cells_default_jobs\(\) \{$/{inside=1} inside{print} inside && /^\}$/{exit}' "$stage_library")"
[[ "$function_text" == *heavy-thread-default* ]] || { echo "  ✗ 从 $stage_library 抽不出 stage_cells_default_jobs" >&2; exit 2; }
echo "name=stage_jobs_sources stage_cells_sha256=$(sha256sum "$stage_library" | cut -c1-64) heavy_threads_sha256=$(sha256sum "$heavy_library" | cut -c1-64) function_sha256=$(printf '%s\n' "$function_text" | sha256sum | cut -c1-64)"

leftover=0
cases=0
disagree=0

# run_side <输出前缀> <cap 或 unset> <库> <函数名>：在新会话里 source 那份库、调那个函数，标准输出、标准错误、退出码各写一份
run_side() {
  local prefix="$1" cap="$2" library="$3" function_name="$4" pid rc
  if [[ "$cap" == unset ]]; then
    setsid --wait env -u SINGLEFS_THREAD_CAP bash -c 'source "$1"; "$2"' probe-side "$library" "$function_name" >"$prefix.out" 2>"$prefix.err" &
    pid=$!
  else
    setsid --wait env "SINGLEFS_THREAD_CAP=$cap" bash -c 'source "$1"; "$2"' probe-side "$library" "$function_name" >"$prefix.out" 2>"$prefix.err" &
    pid=$!
  fi
  if wait "$pid"; then rc=0; else rc=$?; fi
  if kill -0 -- "-$pid" 2>/dev/null; then
    leftover=$((leftover + 1))
  fi
  printf '%s\n' "$rc" > "$prefix.rc"
}

# one_case <格名> <cap 或 unset> <登记文件内容，MISSING 是不建>
one_case() {
  local name="$1" cap="$2" registration="$3" tree stage_out stage_rc heavy_out heavy_rc agree
  tree="$output/stage-jobs/$name"
  mkdir -p "$tree/.claude/gate.d/lib" "$tree/scripts/hosts"
  printf '%s\n' "$function_text" > "$tree/.claude/gate.d/lib/stage-cells.sh"
  cp "$heavy_library" "$tree/scripts/hosts/heavy-threads.sh"
  if [[ "$registration" != MISSING ]]; then
    printf '%b\n' "$registration" > "$tree/.claude/heavy-thread-default"
  fi
  run_side "$tree/stage" "$cap" "$tree/.claude/gate.d/lib/stage-cells.sh" stage_cells_default_jobs
  run_side "$tree/heavy" "$cap" "$tree/scripts/hosts/heavy-threads.sh" heavy_thread_default
  stage_out="$(tr '\n' ' ' < "$tree/stage.out")"; stage_rc="$(<"$tree/stage.rc")"
  heavy_out="$(tr '\n' ' ' < "$tree/heavy.out")"; heavy_rc="$(<"$tree/heavy.rc")"
  agree=yes
  if [[ "$stage_out" != "$heavy_out" || "$stage_rc" != "$heavy_rc" ]]; then agree=no; disagree=$((disagree + 1)); fi
  cases=$((cases + 1))
  echo "name=stage_jobs_case case=$name cap=$cap stage_out=${stage_out% } stage_rc=$stage_rc heavy_out=${heavy_out% } heavy_rc=$heavy_rc agree=$agree"
}

one_case registered-value unset '31  # 样本理由'
one_case cap-five-over-registered 5 '31  # 样本理由'
one_case cap-not-a-number abc '31  # 样本理由'
one_case registration-missing unset MISSING
one_case registration-not-a-number unset 'abc  # 样本理由'
one_case registration-zero unset '0  # 样本理由'
one_case registration-without-reason unset '7'
one_case registration-without-space-before-hash unset '7# 样本理由'
one_case malformed-line-before-a-number unset 'abc  # 写坏的一行\n7  # 样本理由'
echo "name=stage_jobs_summary cases=$cases disagree=$disagree leftover_groups=$leftover"
if ((leftover)); then
  echo "  ✗ 有 $leftover 个样本进程组跑完还有进程" >&2
  exit 3
fi
