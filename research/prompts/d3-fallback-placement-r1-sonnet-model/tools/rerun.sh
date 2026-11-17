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
  # 同一份冻结副本拷成两棵树：tree-today 不动（今天的政策），tree-patched 打 patch/b0-rewrite-set-leaves.patch（先取重写集的叶里的空槽）。
  # 打补丁之前核冻结副本里 allocator.rs、transaction.rs 的 sha256 就是补丁基于的那两份，对不上不跑。
  # 服务撞内存上限重起同一条命令时 --work 要仍是空的（模板拒非空的 --work）：每次尝试写在 $WORK.attempts/attempt.* 里，跑完才把日志与观测拷进 $WORK。
  local base_allocator_sha="36fe6af3e544c9bd56126e525393c749c9dd0cdc81fb01297e8623895e3e7499"
  local base_transaction_sha="19c50519ed1ede3a96298feb646ef7aac38a810596b9f80797f0cb28ccd016a1"
  local allocator_sha transaction_sha
  allocator_sha="$(sha256sum "$FROZEN/crates/singlefs-core/src/allocator.rs" | cut -d' ' -f1)"
  transaction_sha="$(sha256sum "$FROZEN/crates/singlefs-core/src/transaction.rs" | cut -d' ' -f1)"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  if [[ "$allocator_sha" != "$base_allocator_sha" || "$transaction_sha" != "$base_transaction_sha" ]]; then
    echo "name=base_check match=no allocator=$allocator_sha transaction=$transaction_sha"
    echo "  → 补丁基于 allocator.rs=$base_allocator_sha transaction.rs=$base_transaction_sha；冻结副本不是这两份，先重做补丁" >&2
    return 3
  fi
  echo "name=base_check match=yes"
  local attempts="$WORK.attempts" run_dir
  mkdir -p "$attempts"
  rm -rf "$attempts"/attempt.*/tree-today "$attempts"/attempt.*/tree-patched
  run_dir="$(mktemp -d "$attempts/attempt.XXXXXX")"
  mkdir -p "$run_dir/tree-today" "$run_dir/tree-patched"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$run_dir/tree-today/"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$run_dir/tree-patched/"
  local patch_rc=0
  (cd "$run_dir/tree-patched" && patch -p1 < "$MODEL/patch/b0-rewrite-set-leaves.patch") > "$run_dir/patch-apply.log" 2>&1 || patch_rc=$?
  echo "name=patch_apply rc=$patch_rc expected=0"
  [[ $patch_rc -eq 0 ]] || return 4
  cp "$MODEL/tests/d3_fallback_placement_forged_sweep.rs" "$run_dir/tree-today/crates/singlefs-verification/tests/d3_fallback_placement_forged_sweep.rs"
  cp "$MODEL/tests/d3_fallback_placement_forged_sweep.rs" "$run_dir/tree-patched/crates/singlefs-verification/tests/d3_fallback_placement_forged_sweep.rs"
  cp "$MODEL/tests/d3_fallback_placement_leaf_chain.rs" "$run_dir/tree-patched/crates/singlefs-verification/tests/d3_fallback_placement_leaf_chain.rs"
  local unit_rc=0 regression_rc=0 chain_rc=0 sweep_today_rc=0 sweep_patched_rc=0
  (cd "$run_dir/tree-patched" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 lib 测试 fallback_takes_the_lowest_free_slot_in_the_rewrite_set_leaves_first 与 fallback_in_the_rewrite_set_leaves_skips_slots_blocked_on_any_device_and_gives_way_when_the_leaves_are_full; 范围=2 个测试函数; 为什么=核补丁里新加的回落偏好单元测试' cargo test -p singlefs-core --lib -- fallback_takes_the_lowest_free_slot_in_the_rewrite_set_leaves_first fallback_in_the_rewrite_set_leaves_skips_slots_blocked_on_any_device_and_gives_way_when_the_leaves_are_full) \
    > "$run_dir/patched-unit.log" 2>&1 || unit_rc=$?
  (cd "$run_dir/tree-patched" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 commit_generated_units_fall_back_when_the_segment_runs_out; 范围=1 个测试目标; 为什么=补丁树上核现有的回落落点用例照旧绿' cargo test -p singlefs-verification --release --test commit_generated_units_fall_back_when_the_segment_runs_out) \
    > "$run_dir/patched-regression.log" 2>&1 || regression_rc=$?
  (cd "$run_dir/tree-patched" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 d3_fallback_placement_leaf_chain; 范围=1 个测试目标; 为什么=量三条回落臂在抽象叶几何上的固定点轮数与写放大' cargo test -p singlefs-verification --release --test d3_fallback_placement_leaf_chain -- --nocapture) \
    > "$run_dir/patched-leaf-chain.log" 2>&1 || chain_rc=$?
  (cd "$run_dir/tree-today" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 d3_fallback_placement_forged_sweep; 范围=1 个测试目标; 为什么=未打补丁的树上量今天政策在攻方造的空闲图上的固定点轮数' cargo test -p singlefs-verification --release --test d3_fallback_placement_forged_sweep -- --nocapture --test-threads 1) \
    > "$run_dir/today-forged-sweep.log" 2>&1 || sweep_today_rc=$?
  (cd "$run_dir/tree-patched" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 d3_fallback_placement_forged_sweep; 范围=1 个测试目标; 为什么=补丁树上量先取重写集的叶政策在同一批空闲图上的固定点轮数' cargo test -p singlefs-verification --release --test d3_fallback_placement_forged_sweep -- --nocapture --test-threads 1) \
    > "$run_dir/patched-forged-sweep.log" 2>&1 || sweep_patched_rc=$?
  grep -hF 'd3-fallback-placement:' "$run_dir/today-forged-sweep.log" | sed -E 's/ seconds=[0-9.]+//' > "$run_dir/observations-today-forged-sweep.txt" || true
  grep -hF 'd3-fallback-placement:' "$run_dir/patched-forged-sweep.log" | sed -E 's/ seconds=[0-9.]+//' > "$run_dir/observations-patched-forged-sweep.txt" || true
  grep -hF 'd3-fallback-placement:' "$run_dir/patched-leaf-chain.log" > "$run_dir/observations-patched-leaf-chain.txt" || true
  diff -u "$run_dir/observations-today-forged-sweep.txt" "$run_dir/observations-patched-forged-sweep.txt" > "$run_dir/forged-sweep-today-vs-patched.diff" || true
  echo "forged_sweep_lines today=$(wc -l < "$run_dir/observations-today-forged-sweep.txt") patched=$(wc -l < "$run_dir/observations-patched-forged-sweep.txt") differing_lines=$(grep -c '^[-+]d3-fallback-placement:' "$run_dir/forged-sweep-today-vs-patched.diff" || true)"
  grep -hE '^test result:|^test .* \.\.\. |^error(\[E[0-9]+\])?:|panicked at' "$run_dir"/patched-unit.log "$run_dir"/patched-regression.log "$run_dir"/patched-leaf-chain.log "$run_dir"/today-forged-sweep.log "$run_dir"/patched-forged-sweep.log || true
  echo "name=patched_unit_target rc=$unit_rc expected=0"
  echo "name=patched_regression_target rc=$regression_rc expected=0"
  echo "name=patched_leaf_chain_target rc=$chain_rc expected=0"
  echo "name=today_forged_sweep_target rc=$sweep_today_rc expected=0"
  echo "name=patched_forged_sweep_target rc=$sweep_patched_rc expected=0"
  rm -rf "$run_dir/tree-today" "$run_dir/tree-patched"
  cp "$run_dir"/*.log "$run_dir"/*.txt "$run_dir"/*.diff "$WORK/"
  if [[ $unit_rc -eq 0 && $regression_rc -eq 0 && $chain_rc -eq 0 && $sweep_today_rc -eq 0 && $sweep_patched_rc -eq 0 ]]; then
    echo "name=d3_fallback_placement_summary target_as_expected=yes"
    return 0
  fi
  echo "name=d3_fallback_placement_summary target_as_expected=no"
  return 1
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
