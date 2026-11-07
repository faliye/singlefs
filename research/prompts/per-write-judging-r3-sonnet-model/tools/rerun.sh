#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条交 task-server 跑，工作目录随意；不套内存包装、不写线程：上限、线程与机器都由服务定）：
#   task-server/task submit -- bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 task-server/task（判是不是本仓）
#     --frozen  冻结副本（腿开工时拷出的那棵树）：要是一个目录
#     --work    新的草稿目录：不在、或是空目录；跑出来的全写在这里
#     --model   模型目录（原型、变异表、out/）：默认是本脚本的上一层——模型目录整份拷走时它跟着走；只拷这一份脚本时用它指过去
# 退出码：0 跑完；2 用法错（缺参数、仓根不像本仓、冻结副本不在、草稿目录已存在且非空、模型目录不在），stderr 列用法；
#         别的是 rerun_steps 里出错那一步的退出码。--selftest：0 全过，1 有格红。
#
# 准入与运行条件不写、不调 preflight：这份模板拷进模型目录、草稿目录跑，不从自身路径就找不到 .claude/scripts/preflight.sh；
# 环境要求（仓根、冻结副本、草稿目录）由 parse_arguments 自己判，不满足退 2。
#
# 弄坏开关（只给 --selftest 证红用）：
#   RERUN_TEMPLATE_BREAK=root-from-self-path  仓根改回从本脚本路径往上推四层（旧 rerun.sh 的写法），--repo 不认
#       「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」那一格红
#       连带红的格：「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
#   RERUN_TEMPLATE_BREAK=root-check-by-wrapper  判是不是本仓照改前看 scripts/hosts/run-with-memory-cap.sh 在不在（不看 task-server/task）
#       「✗ 自检：--repo 指到只有内存包装、没有 task-server/task 的旧仓应当退 2」那一格红
#       连带红的格（自证里的假仓只有 task-server/task、没有内存包装，被当成不是本仓）：「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」、「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条交 task-server/task submit -- bash <这份脚本> … 跑" >&2
  cat >&2 <<'USAGE'
用法：rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
      rerun.sh --selftest
USAGE
  exit 2
}

parse_arguments() {
  REPO_ROOT="" FROZEN="" WORK="" MODEL=""
  while (($#)); do
    case "$1" in
      --repo|--frozen|--work|--model)
        (($# >= 2)) || refuse "$1 后面要跟一个路径"
        case "$1" in
          --repo) REPO_ROOT="$2" ;;
          --frozen) FROZEN="$2" ;;
          --work) WORK="$2" ;;
          --model) MODEL="$2" ;;
        esac
        shift 2 ;;
      *) refuse "认不出参数：$1" ;;
    esac
  done
  [[ -n "$REPO_ROOT" ]] || refuse "缺 --repo <仓根>"
  [[ -n "$FROZEN" ]] || refuse "缺 --frozen <冻结副本>"
  [[ -n "$WORK" ]] || refuse "缺 --work <新的草稿目录>"
  if [[ "${RERUN_TEMPLATE_BREAK:-}" == root-from-self-path ]]; then
    REPO_ROOT="$(dirname "${BASH_SOURCE[0]}")/../../../.."
  fi
  local repository_marker="task-server/task"
  [[ "${RERUN_TEMPLATE_BREAK:-}" != root-check-by-wrapper ]] || repository_marker="scripts/hosts/run-with-memory-cap.sh"
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/$repository_marker" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 $repository_marker）"
  REPO_ROOT="$(cd "$REPO_ROOT" && pwd)"
  [[ -d "$FROZEN" ]] || refuse "--frozen $FROZEN 不是目录"
  FROZEN="$(cd "$FROZEN" && pwd)"
  MODEL="${MODEL:-$(dirname "${BASH_SOURCE[0]}")/..}"
  [[ -d "$MODEL" ]] || refuse "模型目录 $MODEL 不在（只拷了这一份脚本的，用 --model 指过去）"
  MODEL="$(cd "$MODEL" && pwd)"
  if [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
    refuse "--work $WORK 已存在且非空：给一个新的草稿目录，不在旧输出上接着写"
  fi
  mkdir -p "$WORK"
  WORK="$(cd "$WORK" && pwd)"
}

# ── 这一腿要跑的步骤（第三轮辩方腿）──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引；cargo 每一条写成字面命令（线程由 task-server 给整条套的 capped.sh <N> 定）。
# 续跑：被服务撞内存上限重排时整条命令原样重起：状态（仓副本、各步输出、完成记号）放在 "$WORK.state"，"$WORK" 到最后一步之前一直是空目录
# （parse_arguments 对非空的 --work 退 2，写进去就接不上了）；已完成的步按 "$WORK.state/done/<步名>" 记号跳过，没完成的步重跑、覆盖它自己的输出。
# 选步：环境变量 PROBE_STEPS（逗号分隔，默认 census-unlink,census-pairing,control）；逐态全量的两步要另给 PROBE_EXHAUST（段号或 段号@片号/片数，逗号分隔），
# 对拍三条流那一步再给 PROBE_EXHAUST_FLOW（quarantined、applied-by-records、last-batch 之一）；造坏另给 PROBE_EXHAUST_FAULT（flip:<段内位置>、copy:<组号>、group:<组号>，可用 + 连几个）；按阶全量另给 PROBE_EXHAUST_MAX_GROUPS（至多几个组里有写落下）。
rerun_steps() {
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  local state="${WORK}.state"
  mkdir -p "$state/done"
  has_step() { [[ ",${PROBE_STEPS:-census-unlink,census-pairing,control}," == *",$1,"* ]]; }
  local unlink_test="$state/tree/crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs"
  local pairing_test="$state/tree/crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs"
  if has_step exhaust-unlink || has_step exhaust-pairing; then
    [[ -n "${PROBE_EXHAUST:-}" ]] || { echo "逐态全量的步要设 PROBE_EXHAUST（段号或 段号@片号/片数，逗号分隔）" >&2; return 2; }
  fi
  if has_step exhaust-pairing; then
    [[ -n "${PROBE_EXHAUST_FLOW:-}" ]] || { echo "exhaust-pairing 要设 PROBE_EXHAUST_FLOW（quarantined、applied-by-records、last-batch 之一）" >&2; return 2; }
  fi
  if [[ ! -f "$state/done/tree" ]]; then
    rm -rf "${state:?}/tree"
    mkdir -p "$state/tree"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$state/tree/"
    [[ -f "$unlink_test" && -f "$pairing_test" ]] || { echo "冻结副本里没有两份测试文件" >&2; return 2; }
    if grep -q 'per_write_judging_named_slots' "$unlink_test" "$pairing_test"; then
      echo "冻结副本里的测试文件已带第三轮探针，换一份没追加过的树" >&2
      return 2
    fi
    cat "$MODEL/probe/named_slots_classifier.rs.in" "$MODEL/probe/exhaustive.rs.in" "$MODEL/probe/named_slots_unlink.rs.in" >> "$unlink_test"
    cat "$MODEL/probe/named_slots_classifier.rs.in" "$MODEL/probe/exhaustive.rs.in" "$MODEL/probe/named_slots_pairing.rs.in" >> "$pairing_test"
    touch "$state/done/tree"
  fi
  cd "$state/tree"
  if has_step census-unlink && [[ ! -f "$state/done/census-unlink" ]]; then
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_enumeration_unlink_after_the_multi_record_publish_stream 里第三轮辩方探针的点名槽普查测试; 范围=1 个测试; 为什么=删除流逐个单元段读 E6 点名的槽与点名它的记录，并在几条造坏的臂上看收窄后的 E6n 拒不拒，只读写表不判崩溃状态' cargo test --release -p singlefs-verification --test crash_enumeration_unlink_after_the_multi_record_publish_stream -- --exact per_write_judging_named_slots_tests::probe_named_slot_census_over_the_unlink_stream_and_the_misordered_arms --nocapture --test-threads 1 > "$state/out-census-unlink.txt" 2>&1
    touch "$state/done/census-unlink"
  fi
  if has_step census-pairing && [[ ! -f "$state/done/census-pairing" ]]; then
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 带 gpu 特性的测试目标 crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state 里第三轮辩方探针的点名槽普查测试; 范围=1 个测试; 为什么=对拍三条十亿级流逐个单元段读 E6 点名的槽，只录流、不上卡、不判状态' cargo test --release -p singlefs-verification --features gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact probe_named_slot_census_over_the_three_pairing_flows --nocapture --test-threads 1 > "$state/out-census-pairing.txt" 2>&1
    touch "$state/done/census-pairing"
  fi
  if has_step control && [[ ! -f "$state/done/control" ]]; then
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_enumeration_unlink_after_the_multi_record_publish_stream 里第三轮辩方探针的阳性对照测试; 范围=1 个测试; 为什么=逐态全量对照的同一个比较器在第二轮量到有差的两组同被盖的九十一对上要报红，至多一千四百多个状态的 CPU 判定' cargo test --release -p singlefs-verification --test crash_enumeration_unlink_after_the_multi_record_publish_stream -- --exact per_write_judging_named_slots_tests::probe_exhaustive_comparer_flags_the_planted_two_group_pairs --nocapture --test-threads 1 > "$state/out-control.txt" 2>&1
    touch "$state/done/control"
  fi
  if has_step exhaust-unlink && [[ ! -f "$state/done/exhaust-unlink" ]]; then
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_enumeration_unlink_after_the_multi_record_publish_stream 里第三轮辩方探针的逐态全量对照测试（删除流，段与片由 PROBE_EXHAUST 给）; 范围=1 个测试; 为什么=把候选乙的并合成与增减合成对这些段每个状态的真判逐个比，一个状态不漏' cargo test --release -p singlefs-verification --test crash_enumeration_unlink_after_the_multi_record_publish_stream -- --exact per_write_judging_named_slots_tests::probe_exhaustive_comparison_over_the_requested_segments_of_the_unlink_stream --nocapture --test-threads 1 > "$state/out-exhaust-unlink.txt" 2>&1
    touch "$state/done/exhaust-unlink"
  fi
  if has_step exhaust-pairing && [[ ! -f "$state/done/exhaust-pairing" ]]; then
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 带 gpu 特性的测试目标 crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state 里第三轮辩方探针的逐态全量对照测试（对拍三条流之一，流、段与片由 PROBE_EXHAUST_FLOW 与 PROBE_EXHAUST 给）; 范围=1 个测试; 为什么=把候选乙的并合成与增减合成对这些段每个状态的 CPU 真判逐个比，一个状态不漏，不上卡' cargo test --release -p singlefs-verification --features gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact probe_exhaustive_comparison_over_the_requested_segments_of_a_pairing_flow --nocapture --test-threads 1 > "$state/out-exhaust-pairing.txt" 2>&1
    touch "$state/done/exhaust-pairing"
  fi
  # 汇总：每个输出文件里每一行含 PROBE_ 的都进 out.txt（用 -o 取从第一个 PROBE_ 起的整行尾，测试名前缀不丢行）；行首不是 PROBE_ 的另数一遍报出来
  local joined="$state/out.txt"
  : > "$joined"
  local file probe_lines=0 at_line_start=0 count
  for file in "$state"/out-*.txt; do
    [[ -f "$file" ]] || continue
    count="$(grep -c 'PROBE_' "$file" || true)"
    probe_lines=$((probe_lines + count))
    count="$(grep -c '^PROBE_' "$file" || true)"
    at_line_start=$((at_line_start + count))
    grep -ho 'PROBE_.*$' "$file" >> "$joined" || true
  done
  local joined_lines
  joined_lines="$(wc -l < "$joined")"
  echo "name=rerun_output file=$joined lines=$joined_lines probe_lines_in_the_per_step_files=$probe_lines probe_lines_at_line_start=$at_line_start complete=$([[ "$joined_lines" -eq "$probe_lines" ]] && echo true || echo false)"
  [[ "$joined_lines" -eq "$probe_lines" ]] || return 1
  cp "$joined" "$WORK/out.txt"
  for file in "$state"/out-*.txt; do
    [[ -f "$file" ]] && cp "$file" "$WORK/"
  done
  echo "name=rerun_state tree=$state/tree（仓副本，核完要删）"
  sha256sum "$WORK"/out*.txt
  if [[ -f "$MODEL/out/out.txt" ]]; then
    if diff "$MODEL/out/out.txt" "$WORK/out.txt" > "$WORK/out.diff"; then
      echo "name=rerun_compare same=true against=$MODEL/out/out.txt"
    else
      echo "name=rerun_compare same=false against=$MODEL/out/out.txt diff=$WORK/out.diff"
      return 1
    fi
  else
    echo "name=rerun_compare same=not_compared reason=模型目录里还没有 out/out.txt"
  fi
}

# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  export RERUN_TEMPLATE_SELFTEST_MARK="rerun-template-selftest"   # 自证里拷出去起的子进程（环境原样继承）靠它认：弄坏开关只在自证里认
  mkdir -p "$scratch/fake-repo/.git" "$scratch/fake-repo/task-server" "$scratch/old-repo/.git" "$scratch/old-repo/scripts/hosts" "$scratch/frozen/crates/target" \
    "$scratch/elsewhere/r1-opus-model/tools" "$scratch/cwd"
  : > "$scratch/fake-repo/task-server/task"
  : > "$scratch/old-repo/scripts/hosts/run-with-memory-cap.sh"
  echo 冻结副本里的文件 > "$scratch/frozen/crates/marker.txt"
  echo 编译产物 > "$scratch/frozen/crates/target/junk"
  copy="$scratch/elsewhere/r1-opus-model/tools/rerun.sh"
  cp "${BASH_SOURCE[0]}" "$copy"

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1") \
      > "$scratch/run1.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 0 ]] || ! grep -qF "name=rerun_roots repo=$scratch/fake-repo frozen=$scratch/frozen model=$scratch/elsewhere/r1-opus-model " "$scratch/run1.out" \
      || [[ ! -f "$scratch/work1/tree/crates/marker.txt" || -e "$scratch/work1/tree/crates/target" ]]; then
    failures+=("拷进与仓无关的临时目录、传参应当跑通（退 0、仓根取 --repo、冻结副本拷进草稿目录、target 不拷），实际退 $rc：$(tail -n 4 "$scratch/run1.out")")
  fi

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy") > "$scratch/run2.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "用法：rerun.sh --repo" "$scratch/run2.out"; then
    failures+=("不传参应当退 2 并列出用法，实际退 $rc：$(tail -n 4 "$scratch/run2.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --work "$scratch/work3" > "$scratch/run3.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "缺 --frozen" "$scratch/run3.out"; then
    failures+=("缺 --frozen 应当退 2 并点名缺它，实际退 $rc：$(tail -n 4 "$scratch/run3.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/cwd" --frozen "$scratch/frozen" --work "$scratch/work4" > "$scratch/run4.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run4.out"; then
    failures+=("--repo 指到不是仓的目录应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run4.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/old-repo" --frozen "$scratch/frozen" --work "$scratch/work6" > "$scratch/run6.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run6.out"; then
    failures+=("--repo 指到只有内存包装、没有 task-server/task 的旧仓应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run6.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1" > "$scratch/run5.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "已存在且非空" "$scratch/run5.out"; then
    failures+=("--work 指到已有输出的目录应当退 2、不在旧输出上接着写，实际退 $rc：$(tail -n 4 "$scratch/run5.out")")
  fi

  if ((${#failures[@]})); then
    local failure
    for failure in "${failures[@]}"; do
      echo "  ✗ 自检：$failure"   # gate-lint:detail
    done
    echo "    → 怎么办：看 parse_arguments() 与 rerun_steps()；RERUN_TEMPLATE_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 格）：拷进与仓无关的临时目录、从别的目录起，传参跑通；不传参、缺 --frozen、--repo 不是仓、--repo 是没有 task-server/task 的旧仓、--work 非空都退 2 并说明"
  return 0
}

# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（RERUN_TEMPLATE_SELFTEST_MARK 对得上）而变量非空，参数解析之前就退 2（不管值是什么）。
if [[ -n "${RERUN_TEMPLATE_BREAK:-}" && ! ( "$#" -eq 1 && "${1:-}" == --selftest ) && "${RERUN_TEMPLATE_SELFTEST_MARK:-}" != rerun-template-selftest ]]; then
  echo "  ✗ RERUN_TEMPLATE_BREAK=${RERUN_TEMPLATE_BREAK} 是弄坏开关，只在自证（bash <这份脚本> --selftest）里认，平常跑设了就拒，这一趟什么都没做" >&2
  echo "     → 怎么办：去掉 RERUN_TEMPLATE_BREAK 再跑；要证红就跑 RERUN_TEMPLATE_BREAK=<项> bash scripts/agents/three-way-rerun-template.sh --selftest" >&2
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
