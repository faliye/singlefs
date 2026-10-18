#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条经内存包装跑，工作目录随意）：
#   bash scripts/hosts/run-with-memory-cap.sh <上限> bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 scripts/hosts/run-with-memory-cap.sh（判是不是本仓）
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
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条经 scripts/hosts/run-with-memory-cap.sh 跑" >&2
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
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 scripts/hosts/run-with-memory-cap.sh）"
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
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 经 "$REPO_ROOT/scripts/hosts/capped.sh" 起，
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
rerun_steps() {
  # RERUN_STEPS（可选，空格分隔的步骤名）只跑其中几步；不设就全跑。CARGO_TARGET_DIR 不设就用 $WORK/target。
  local wanted="${RERUN_STEPS:-all}"
  local tree="$WORK/tree" out="$WORK/out"
  mkdir -p "$tree" "$out"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$tree/"
  (cd "$tree" && patch -p1 --quiet < "$MODEL/src/probe-instrumentation.diff")
  local tests="$tree/crates/singlefs-harness/tests"
  cat "$tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs" "$MODEL/src/tests/forward_probe_q1.tail.rs" > "$tests/forward_probe_q1.rs"
  cat "$tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs" "$MODEL/src/tests/forward_probe_rollback.tail.rs" > "$tests/forward_probe_rollback.rs"
  cp "$MODEL/src/tests/forward_probe_q1b.rs" "$tests/forward_probe_q1b.rs"
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$WORK/target}"
  local capped="$REPO_ROOT/scripts/hosts/capped.sh"

  want() { [[ "$wanted" == all || " $wanted " == *" $1 "* ]]; }
  # cargo_test <日志> <环境赋值…> -- <cargo test 的参数…>：线程上限取脚本内部的 default（登记值），整条已在外层内存包装里
  cargo_test() {
    local log="$1"; shift
    local -a assignments=()
    while [[ "$1" != "--" ]]; do assignments+=("$1"); shift; done
    shift
    (cd "$tree" && env "${assignments[@]}" nice -n 19 bash "$capped" default cargo test --offline "$@") > "$log" 2>&1 || echo "exit_nonzero=$?" >> "$log"
  }
  # 只留结论行（PROBE 行、test result 行），去掉耗时字段，写进 $out/<名>.txt
  digest() {
    local log="$1" name="$2"
    { grep -E 'PROBE-|^test result' "$log" || true; } \
      | sed -E 's/^test [A-Za-z0-9_:]+ \.\.\. //; s/ finished in [0-9.]+s//; s/ build=[0-9.a-zµ]+ total=[0-9.a-zµ]+//' | { grep -v -e '^PROBE-Q3 call=' -e '^PROBE-Q8 rollback' || true; } > "$out/$name.txt"
    echo "name=digest step=$name lines=$(wc -l < "$out/$name.txt")"
  }

  want q1 && { cargo_test "$out/q1.log" -- -p singlefs-harness --test forward_probe_q1 probe_q1 -- --nocapture --test-threads 1; digest "$out/q1.log" q1; }
  want q1b && { cargo_test "$out/q1b.log" -- -p singlefs-harness --test forward_probe_q1b -- --nocapture --test-threads 1; digest "$out/q1b.log" q1b; }
  want q1b_checker_red && { cargo_test "$out/q1b_checker_red.log" PROBE_CHECKER_RED_ON_BOTH_BITS=1 -- -p singlefs-harness --test forward_probe_q1b -- --nocapture --test-threads 1; digest "$out/q1b_checker_red.log" q1b_checker_red; }
  want matrix && { cargo_test "$out/matrix.log" -- -p singlefs-harness --test forward_probe_rollback probe_matrix -- --nocapture --test-threads 1; digest "$out/matrix.log" matrix; }
  want crash_isolating && { cargo_test "$out/crash_isolating.log" -- -p singlefs-harness --test forward_probe_rollback probe_crash_isolating_publish -- --nocapture --test-threads 1; digest "$out/crash_isolating.log" crash_isolating; }
  want crash_rollback && { cargo_test "$out/crash_rollback.log" -- -p singlefs-harness --test forward_probe_rollback probe_crash_rollback_publish -- --nocapture --test-threads 1; digest "$out/crash_rollback.log" crash_rollback; }
  want asymmetric && { cargo_test "$out/asymmetric.log" -- -p singlefs-harness --test forward_probe_rollback -- probe_asymmetric probe_symmetric_control --nocapture --test-threads 1; digest "$out/asymmetric.log" asymmetric; }
  want q4 && { cargo_test "$out/q4.log" -- -p singlefs-harness --test forward_probe_rollback probe_q4 -- --nocapture --test-threads 1; digest "$out/q4.log" q4; }
  want remount && { cargo_test "$out/remount.log" -- -p singlefs-harness --test forward_probe_rollback probe_bit_survives -- --nocapture --test-threads 1; digest "$out/remount.log" remount; }
  want q8_model && { cargo_test "$out/q8_model.log" -- -p singlefs-harness --lib probe_q8 -- --nocapture; digest "$out/q8_model.log" q8_model; }
  # Q4 的四个读法变体 × 现有四个目标（现有测试分不分得开两种读法）
  local targets=(rollback_by_a_forward_publish rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references release_checksum_quarantine checker_quarantine_flag_exempts_only_flagged_unreferenced_records)
  local -a test_args=()
  local target
  for target in "${targets[@]}"; do test_args+=(--test "$target"); done
  local mode
  for mode in inference bit_any bit_all bit_any_and_unreferenced inference_without_guard; do
    want "variant_$mode" && { cargo_test "$out/variant_$mode.log" "PROBE_Q4=$mode" -- -p singlefs-harness "${test_args[@]}" -- --test-threads 8 --nocapture; digest "$out/variant_$mode.log" "variant_$mode"; }
  done
  # 在池级 checker 里「已释放位与隔离位同置」判红之后，现有六个目标有没有新红；同时数 PROBE-Q3（隔离位记录被根引用）的出现次数
  local wide=("${targets[@]}" fault_injection_fast_tier random_histories)
  test_args=()
  for target in "${wide[@]}"; do test_args+=(--test "$target"); done
  want regression_with_checker_red && { cargo_test "$out/regression_with_checker_red.log" PROBE_CHECKER_RED_ON_BOTH_BITS=1 -- -p singlefs-harness "${test_args[@]}" -- --test-threads 8 --nocapture
    { grep -E '^test result' "$out/regression_with_checker_red.log" | sed -E 's/ finished in [0-9.]+s//'
      echo "name=count probe_q3_union_lines=$(grep -c 'PROBE-Q3 call=union' "$out/regression_with_checker_red.log") probe_q3_newest_lines=$(grep -c 'PROBE-Q3 call=newest' "$out/regression_with_checker_red.log")"
      echo "name=count rollbacks_reaching_the_clear_step=$(grep -c 'PROBE-Q8 rollback' "$out/regression_with_checker_red.log") of_them_with_isolated_units=$(grep -c 'PROBE-Q8 rollback isolated_units=[1-9]' "$out/regression_with_checker_red.log")"; } > "$out/regression_with_checker_red.txt"
    echo "name=digest step=regression_with_checker_red lines=$(wc -l < "$out/regression_with_checker_red.txt")"; }

  # 与模型目录里 out/ 逐行比
  local differing=0 compared=0 file
  for file in "$out"/*.txt; do
    compared=$((compared + 1))
    if [[ -f "$MODEL/out/$(basename "$file")" ]]; then
      if ! diff -q "$file" "$MODEL/out/$(basename "$file")" > /dev/null; then
        differing=$((differing + 1)); echo "name=rerun_differs file=$(basename "$file")"; diff "$file" "$MODEL/out/$(basename "$file")" | head -n 6
      fi
    else
      echo "name=rerun_no_baseline file=$(basename "$file")"
    fi
  done
  echo "name=rerun_compare compared=$compared differing=$differing"
}

# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  mkdir -p "$scratch/fake-repo/.git" "$scratch/fake-repo/scripts/hosts" "$scratch/frozen/crates/target" \
    "$scratch/elsewhere/r1-opus-model/tools" "$scratch/cwd"
  : > "$scratch/fake-repo/scripts/hosts/run-with-memory-cap.sh"
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
  echo "  ✓ 自检通过（查了 $checked 格）：拷进与仓无关的临时目录、从别的目录起，传参跑通；不传参、缺 --frozen、--repo 不是仓、--work 非空都退 2 并说明"
  return 0
}

if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
