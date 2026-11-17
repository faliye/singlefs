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
#   RERUN_TEMPLATE_BREAK=root-check-by-wrapper  判是不是本仓看 scripts/hosts/run-with-memory-cap.sh 在不在（不看 task-server/task）
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

# ── 这一腿要跑的步骤：腿把这一段换成自己的 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 直接起（线程由 task-server 给整条套的 capped.sh <N> 定，这里不再套），
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
rerun_steps() {
  # 冻结副本经 minimal-copy.py 拷进 $WORK.attempts 下这一次尝试的目录，先核两份被判产品文件与开工快照的 sha256，
  # 再把 probes/ 里那一条探针放进副本验证包的 tests/ 当新测试目标，先在产品代码上跑（产品臂），
  # 再在同一份副本上把停着删意图的破例改成照样判门（变异臂：`&& !is_this_drop_the_exception` 换成恒真），
  # 跑同一条探针与停着删意图那两份已有测试目标。观测行（m4-close-batch2-r3-attack: 开头）落进 $WORK/observations.txt。
  # 服务撞内存上限重起同一条命令时 --work 要仍是空的（模板拒非空的 --work）：每次尝试写在 $WORK.attempts/attempt.* 里，跑完才把日志与观测拷进 $WORK。
  local attempts="$WORK.attempts" run_dir copy
  mkdir -p "$attempts"
  rm -rf "$attempts"/attempt.*/copy
  run_dir="$(mktemp -d "$attempts/attempt.XXXXXX")"
  copy="$run_dir/copy"
  python3 "$REPO_ROOT/scripts/hosts/minimal-copy.py" "$FROZEN" "$copy" --with-test-inputs > "$run_dir/minimal-copy.log" 2>&1
  local snapshot_rc=0
  (cd "$copy" && sha256sum -c "$REPO_ROOT/research/prompts/m4-close-batch2-code-r3-snapshot/crates-sha256.txt") > "$run_dir/snapshot-check.txt" 2>&1 || snapshot_rc=$?
  cat "$run_dir/snapshot-check.txt"
  echo "name=m4_close_batch2_r3_attack_snapshot rc=$snapshot_rc expected=0"
  if [[ $snapshot_rc -ne 0 ]]; then
    rm -rf "$copy"
    cp "$run_dir/snapshot-check.txt" "$WORK/"
    echo "name=m4_close_batch2_r3_attack_summary ran=no why=snapshot"
    return 1
  fi
  cp "$MODEL/probes/attack_stopped_drop_exception_and_the_explicit_entry.rs" \
    "$copy/crates/singlefs-verification/tests/attack_stopped_drop_exception_and_the_explicit_entry.rs"
  local product_rc=0
  (cd "$copy" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 attack_stopped_drop_exception_and_the_explicit_entry（攻方探针，副本里新加）; 范围=1 个测试目标; 为什么=攻方腿核停着删意图的破例与显式入口的预算在产品代码上的样子' cargo test -p singlefs-verification --test attack_stopped_drop_exception_and_the_explicit_entry -- --nocapture) \
    > "$run_dir/probe-product.log" 2>&1 || product_rc=$?
  grep -aF "m4-close-batch2-r3-attack:" "$run_dir/probe-product.log" | sed 's/^/arm=product /' > "$run_dir/observations.txt" || true
  grep -aE "^test result:|^test .* \.\.\. |^error(\[E[0-9]+\])?:|panicked at" "$run_dir/probe-product.log" || true
  echo "name=m4_close_batch2_r3_attack_probe arm=product rc=$product_rc expected=0"
  local mutated_rc=0
  python3 - "$copy/crates/singlefs-core/src/mounted_session.rs" > "$run_dir/mutation.txt" 2>&1 <<'PY' || mutated_rc=$?
import sys
path = sys.argv[1]
original = "        if budget != 0 && is_the_rollback_gate_judged && !is_this_drop_the_exception {\n"
mutated = "        if budget != 0 && is_the_rollback_gate_judged && (true || !is_this_drop_the_exception) {\n"
with open(path, encoding="utf-8") as source:
    text = source.read()
if text.count(original) != 1:
    print(f"mutation_anchor_hits={text.count(original)}")
    sys.exit(3)
with open(path, "w", encoding="utf-8") as target:
    target.write(text.replace(original, mutated))
print("mutation_anchor_hits=1")
PY
  cat "$run_dir/mutation.txt"
  echo "name=m4_close_batch2_r3_attack_mutation rc=$mutated_rc expected=0"
  local mutant_probe_rc=skipped
  local -A existing_rc=()
  if [[ $mutated_rc -eq 0 ]]; then
    mutant_probe_rc=0
    (cd "$copy" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 attack_stopped_drop_exception_and_the_explicit_entry（攻方探针，变异副本）; 范围=1 个测试目标; 为什么=攻方腿核探针抓不抓得到破例照样判门' cargo test -p singlefs-verification --test attack_stopped_drop_exception_and_the_explicit_entry -- --nocapture) \
      > "$run_dir/probe-mutant.log" 2>&1 || mutant_probe_rc=$?
    grep -aF "m4-close-batch2-r3-attack:" "$run_dir/probe-mutant.log" | sed 's/^/arm=exception-judged-at-the-gate /' >> "$run_dir/observations.txt" || true
    grep -aE "^test result:|^test .* \.\.\. |^error(\[E[0-9]+\])?:|panicked at" "$run_dir/probe-mutant.log" || true
    echo "name=m4_close_batch2_r3_attack_probe arm=exception-judged-at-the-gate rc=$mutant_probe_rc expected=nonzero"
    existing_rc[compaction_stopped_intent_drop_judges_the_rollback_gate]=0
    (cd "$copy" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 compaction_stopped_intent_drop_judges_the_rollback_gate（变异副本）; 范围=1 个测试目标; 为什么=攻方腿核已有测试抓不抓得到停着删意图的破例照样判门' cargo test -p singlefs-verification --test compaction_stopped_intent_drop_judges_the_rollback_gate) \
      > "$run_dir/existing-compaction_stopped_intent_drop_judges_the_rollback_gate.log" 2>&1 || existing_rc[compaction_stopped_intent_drop_judges_the_rollback_gate]=$?
    grep -aE "^test result:|^error(\[E[0-9]+\])?:" "$run_dir/existing-compaction_stopped_intent_drop_judges_the_rollback_gate.log" || true
    echo "name=m4_close_batch2_r3_attack_existing arm=exception-judged-at-the-gate target=compaction_stopped_intent_drop_judges_the_rollback_gate rc=${existing_rc[compaction_stopped_intent_drop_judges_the_rollback_gate]} expected=0"
    existing_rc[compaction_on_the_cpu_while_stopped_after_a_gpu_failure]=0
    (cd "$copy" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 compaction_on_the_cpu_while_stopped_after_a_gpu_failure（变异副本）; 范围=1 个测试目标; 为什么=攻方腿核已有测试抓不抓得到停着删意图的破例照样判门' cargo test -p singlefs-verification --test compaction_on_the_cpu_while_stopped_after_a_gpu_failure) \
      > "$run_dir/existing-compaction_on_the_cpu_while_stopped_after_a_gpu_failure.log" 2>&1 || existing_rc[compaction_on_the_cpu_while_stopped_after_a_gpu_failure]=$?
    grep -aE "^test result:|^error(\[E[0-9]+\])?:" "$run_dir/existing-compaction_on_the_cpu_while_stopped_after_a_gpu_failure.log" || true
    echo "name=m4_close_batch2_r3_attack_existing arm=exception-judged-at-the-gate target=compaction_on_the_cpu_while_stopped_after_a_gpu_failure rc=${existing_rc[compaction_on_the_cpu_while_stopped_after_a_gpu_failure]} expected=0"
  fi
  cat "$run_dir/observations.txt"
  rm -rf "$copy"
  cp "$run_dir"/*.log "$run_dir/observations.txt" "$run_dir/snapshot-check.txt" "$run_dir/mutation.txt" "$WORK/"
  local product_tests_ok=no explicit_ok=no exception_ok=no mutant_caught=no survives=no
  grep -aqE '^test result: ok\. 4 passed' "$run_dir/probe-product.log" && product_tests_ok=yes
  grep -aqE '^test attack_the_explicit_entry_.* \.\.\. ok' "$run_dir/probe-product.log" && explicit_ok=yes
  [[ "$(grep -acE '^test attack_the_stopped_drop_after_.* \.\.\. ok' "$run_dir/probe-product.log")" == 3 ]] && exception_ok=yes
  if [[ -f "$run_dir/probe-mutant.log" ]] && [[ "$(grep -acE '^test attack_the_stopped_drop_after_.* \.\.\. FAILED' "$run_dir/probe-mutant.log")" == 3 ]]; then
    mutant_caught=yes
  fi
  if [[ "${existing_rc[compaction_stopped_intent_drop_judges_the_rollback_gate]:-x}" == 0 && "${existing_rc[compaction_on_the_cpu_while_stopped_after_a_gpu_failure]:-x}" == 0 ]]; then
    survives=yes
  fi
  echo "name=m4_close_batch2_r3_attack_summary product_probe_all_ok=$product_tests_ok exception_drops_through_the_closed_gate=$exception_ok explicit_entry_drops_with_the_budget_spent=$explicit_ok mutant_caught_by_the_probe=$mutant_caught mutant_survives_the_existing_targets=$survives"
  # 退出码只说跑没跑成：快照核过、产品臂编成跑完（有 test result 行）、变异打上、变异臂三条都编成跑完。读数看上面那一行。
  grep -aq '^test result:' "$run_dir/probe-product.log" || return 1
  [[ $mutated_rc -eq 0 ]] || return 1
  grep -aq '^test result:' "$run_dir/probe-mutant.log" || return 1
  grep -aq '^test result:' "$run_dir/existing-compaction_stopped_intent_drop_judges_the_rollback_gate.log" || return 1
  grep -aq '^test result:' "$run_dir/existing-compaction_on_the_cpu_while_stopped_after_a_gpu_failure.log" || return 1
  return 0
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
