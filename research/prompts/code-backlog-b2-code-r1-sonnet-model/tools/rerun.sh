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

# ── 这一腿要跑的步骤 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 经 "$REPO_ROOT/scripts/hosts/capped.sh" 起，
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
rerun_steps() {
  local tree="$WORK/tree" out="$WORK/out" tsv="$MODEL/mutations-b3-c572.tsv"
  mkdir -p "$tree" "$out"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$tree/"
  export CARGO_TARGET_DIR="$WORK/target"
  local mount_rs="crates/singlefs-core/src/mount.rs"
  # 一步：cargo test 经线程上限包装，只留 test 行与 result 行（去掉耗时），写进 out/<名>.txt
  cargo_step() {   # cargo_step <名> <cargo test 的参数> [<测试二进制的参数>]
    local name="$1" cargo_arguments="$2" test_arguments="${3:-}"
    bash "$REPO_ROOT/scripts/hosts/capped.sh" default bash -c "cd '$tree' && cargo test --no-fail-fast $cargo_arguments -- --nocapture $test_arguments" 2>&1 \
      | grep -E '^test |^test result|^P3 outcome|^PROBE-|^error' | sed -E 's/; finished in [0-9.]+s//; s/ \.\.\. / => /' | sort > "$out/$name.txt" || true
    echo "== $name"; cat "$out/$name.txt"
  }
  reset_mount() { cp "$FROZEN/$mount_rs" "$tree/$mount_rs"; }
  local core_filter="-p singlefs-core --lib shadow_ledger_exempts_and_isolates_slot_by_slot_tests"

  # S1 交付的六条 B3 用例（冻结副本原样）
  reset_mount; cargo_step s1-b3-delivered "$core_filter"
  # S2 追加表里 B3 五处各改回按起点槽，看哪几条红
  for row in 0 1 2 3 4; do
    reset_mount; python3 -I "$MODEL/tools/mutate.py" "$tree" "$tsv" "$row"
    cargo_step "s2-b3-mutation-$row" "$core_filter"
  done
  # S3 探针 P1（次序）、P3（分配位置着的槽被两条被抛弃根罩着）：交付的样子；次序对调；第三处改回起点槽
  reset_mount; python3 -I "$MODEL/tools/apply-core-probes.py" "$tree" "$MODEL/tools/core-probes.rs.txt"
  cargo_step s3-probes-delivered "-p singlefs-core --lib probe_p"
  reset_mount; python3 -I "$MODEL/tools/apply-core-probes.py" "$tree" "$MODEL/tools/core-probes.rs.txt"; python3 -I "$MODEL/tools/mutate.py" "$tree" extra order
  cargo_step s3-probes-order-swapped "-p singlefs-core --lib shadow_ledger_exempts_and_isolates_slot_by_slot_tests"
  reset_mount; python3 -I "$MODEL/tools/apply-core-probes.py" "$tree" "$MODEL/tools/core-probes.rs.txt"; python3 -I "$MODEL/tools/mutate.py" "$tree" "$tsv" 2
  cargo_step s3-probes-site3-back-to-start-slot "-p singlefs-core --lib probe_p"
  # S4 harness：交付的新用例文件与回退文件（含慢档）
  reset_mount
  cargo_step s4-harness-delivered "-p singlefs-harness --test rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references --test rollback_by_a_forward_publish" "--include-ignored"
  # S5 C572 那一步去掉
  reset_mount; python3 -I "$MODEL/tools/mutate.py" "$tree" "$tsv" 5
  cargo_step s5-c572-step-removed "-p singlefs-harness --test rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references"
  # S6 K2 探针（共用单元带坏份不验）：交付的样子；去掉「现行那一版引用着就不验」这一道
  reset_mount; cp "$MODEL/src/probe_k2_shared_unit_is_not_verified.rs" "$tree/crates/singlefs-harness/tests/"
  cargo_step s6-k2-probes-delivered "-p singlefs-harness --test probe_k2_shared_unit_is_not_verified"
  reset_mount; python3 -I "$MODEL/tools/mutate.py" "$tree" extra guard
  cargo_step s6-k2-probes-guard-dropped "-p singlefs-harness --test probe_k2_shared_unit_is_not_verified --test rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references --test rollback_by_a_forward_publish" "--include-ignored"
  # S7 整行替换的八行：每行改进树里、按那一行自己的参数跑，记点名的测试红不红（第 1 行即「步 4：影子账不豁免候选根引用的槽」在 harness 慢档那条上不红）
  local replaced="$MODEL/mutations-replaced.tsv" row
  for row in 0 1 2 3 4 5 6 7; do
    reset_mount; python3 -I "$MODEL/tools/mutate.py" "$tree" "$replaced" "$row" > "$out/s7-row$row.info"
    local named_test row_arguments
    named_test="$(sed -n "$((row + 1))p" "$replaced" | cut -f6)"; row_arguments="$(sed -n "$((row + 1))p" "$replaced" | cut -f5)"
    bash "$REPO_ROOT/scripts/hosts/capped.sh" default bash -c "cd '$tree' && cargo test --no-fail-fast $row_arguments" > "$out/s7-row$row.log" 2>&1 || true
    if grep -qE "^test .*${named_test} \.\.\. FAILED" "$out/s7-row$row.log"; then echo "RED" > "$out/s7-row$row.txt"; elif grep -qE "^test .*${named_test} \.\.\. ok" "$out/s7-row$row.log"; then echo "NOT-RED(named test ran and passed)" > "$out/s7-row$row.txt"; else echo "NOT-RUN-OR-INVALID" > "$out/s7-row$row.txt"; fi
    echo "== s7 row $row ($(head -c 60 "$out/s7-row$row.info" | tr -d '\n')...): $(cat "$out/s7-row$row.txt")"
  done
  # S8 第 1 行（步 4 不豁免候选根）换一个取样点：交付的 core 用例里那条候选根用例
  reset_mount; python3 -I "$MODEL/tools/mutate.py" "$tree" "$replaced" 1 > /dev/null
  cargo_step s8-row1-at-the-core-unit-tests "$core_filter"
  # 与模型目录里 out/ 逐行比
  if [[ -d "$MODEL/out" ]] && ls "$MODEL/out"/*.txt > /dev/null 2>&1; then
    local differing=0 f
    for f in "$MODEL/out"/*.txt; do
      if ! diff -q "$f" "$out/$(basename "$f")" > /dev/null; then differing=$((differing + 1)); echo "不同：$(basename "$f")"; fi
    done
    echo "name=rerun_compare differing=$differing"
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
