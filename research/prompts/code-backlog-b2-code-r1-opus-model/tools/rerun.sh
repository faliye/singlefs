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

# ── 这一腿要跑的步骤（代码欠账批甲簇代码轮攻方腿，Opus）──
# 冻结副本拷两份：插桩那一份（OPUS-K2 / OPUS-B3 打印、K1 直调用例、K2 / K3 两份 harness 原型）与次序互换变异那一份；
# 跑 core 影子账模块、K2 / K3 原型、46 个 harness 目标、随机历史（快档带忽略的与大档两段），变异那一份跑 core 全库与 46 个目标带忽略的。
# 每段的计数行写进 $WORK/tally.txt，最后与 $MODEL/out/tally.txt 逐行比（去掉耗时）。
count() { local n; n=$(grep -o -- "$1" "$2" | wc -l) || true; echo "$n"; }
tally() {   # tally <名> <日志>
  local L="$2" passed failed
  passed=$(grep '^test result' "$L" | awk '{p+=$4} END {print p+0}') || true
  failed=$(grep '^test result' "$L" | awk '{f+=$6} END {print f+0}') || true
  echo "name=$1 tests_passed=$passed tests_failed=$failed b3_calls=$(count 'OPUS-B3 abandoned' "$L") b3_new_only_nonempty=$(count 'new_only=\[(' "$L") b3_old_only_nonempty=$(count 'old_only=\[(' "$L") b3_with_abandoned=$(count 'OPUS-B3 abandoned_roots=[1-9]' "$L") b3_strong=$(count 'strong_roots=[1-9]' "$L") b3_named=$(count 'named=[1-9]' "$L") b3_ring_bad=$(count 'ring_bad=[1-9]' "$L") k2_rollbacks=$(count 'OPUS-K2 rollback' "$L") k2_isolated_nonempty=$(count 'isolated_units=\[(' "$L")" | tee -a "$WORK/tally.txt"
}
cargo_capped() {   # cargo_capped <目录> <编译目录> <cargo 参数…>：经 capped.sh 起（外层整条已经经内存包装）
  local tree="$1" target="$2"; shift 2
  (cd "$tree" && CARGO_TARGET_DIR="$target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "${OPUS_THREADS:-31}" cargo "$@")
}
rerun_steps() {
  local main="$WORK/tree-main" swap="$WORK/tree-swap" args
  mkdir -p "$main" "$WORK/out"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$main/"
  python3 -I "$MODEL/src/instrument.py" "$main/crates/singlefs-core/src/mount.rs" "$MODEL/src/k1-tests.rs"
  cp "$MODEL/src/opus_k2_isolation_made_by_a_rollback_release.rs" "$MODEL/src/opus_k3_refusal_order_and_allocator_state.rs" "$main/crates/singlefs-harness/tests/"
  rsync -a "$main/" "$swap/"
  python3 -I "$MODEL/src/swap.py" "$swap/crates/singlefs-core/src/mount.rs"
  args=$(sed 's/^/--test /' "$MODEL/src/harness-selected.txt" | tr '\n' ' ')
  cargo_capped "$main" "$WORK/target-main" test -p singlefs-core --lib -- shadow_ledger_exempts_and_isolates_slot_by_slot_tests --nocapture --test-threads 1 > "$WORK/out/k1-main.log" 2>&1 || true
  tally k1-main "$WORK/out/k1-main.log"
  cargo_capped "$main" "$WORK/target-main" test -p singlefs-harness --test opus_k2_isolation_made_by_a_rollback_release --test opus_k3_refusal_order_and_allocator_state -- --nocapture > "$WORK/out/k2-k3.log" 2>&1 || true
  tally k2-k3 "$WORK/out/k2-k3.log"
  # shellcheck disable=SC2086
  cargo_capped "$main" "$WORK/target-main" test -p singlefs-harness $args --no-fail-fast -- --nocapture > "$WORK/out/run-main-harness.log" 2>&1 || true
  tally main-harness-46 "$WORK/out/run-main-harness.log"
  cargo_capped "$main" "$WORK/target-main" test --release -p singlefs-harness --test random_histories --test unlink_in_histories_judged_by_the_model --test rollback_by_a_forward_publish --no-fail-fast -- --include-ignored --nocapture --skip large_tier --skip shrink_one_failing_seed > "$WORK/out/run-release-random.log" 2>&1 || true
  tally main-random-fast-and-ignored "$WORK/out/run-release-random.log"
  local combo weights devices
  for combo in rollback:unit-area-384 reuse:unit-area-240; do
    weights=${combo%%:*} devices=${combo##*:}
    SINGLEFS_RANDOM_HISTORY_SEEDS=2000 SINGLEFS_RANDOM_HISTORY_FIRST_SEED=7100000000 SINGLEFS_RANDOM_HISTORY_WEIGHTS=$weights \
      SINGLEFS_RANDOM_HISTORY_DEVICES=$devices SINGLEFS_RANDOM_HISTORY_SPACE_ADMISSION=skipped SINGLEFS_RANDOM_HISTORY_SHRINK=none \
      cargo_capped "$main" "$WORK/target-main" test --release -p singlefs-harness --test random_histories -- --ignored --nocapture large_tier > "$WORK/out/large-$weights-$devices.log" 2>&1 || true
    tally "main-large-$weights-$devices" "$WORK/out/large-$weights-$devices.log"
  done
  cargo_capped "$swap" "$WORK/target-swap" test -p singlefs-core --lib -- --nocapture --test-threads 1 > "$WORK/out/swap-core-lib.log" 2>&1 || true
  tally swap-core-lib "$WORK/out/swap-core-lib.log"
  # shellcheck disable=SC2086
  cargo_capped "$swap" "$WORK/target-swap" test --release -p singlefs-harness $args --no-fail-fast -- --include-ignored --skip large_tier --skip shrink_one_failing_seed > "$WORK/out/swap-harness.log" 2>&1 || true
  tally swap-harness-46-with-ignored "$WORK/out/swap-harness.log"
  grep -h -E '^OPUS-K1-|^OPUS-K2-|^OPUS-K3 ' "$WORK"/out/k1-main.log "$WORK"/out/k2-k3.log "$WORK"/out/swap-core-lib.log | sort > "$WORK/opus-lines.txt"
  echo "name=rerun_compare tally_same=$(diff -q "$WORK/tally.txt" "$MODEL/out/tally.txt" > /dev/null && echo yes || echo no) lines_same=$(diff -q "$WORK/opus-lines.txt" "$MODEL/out/opus-lines.txt" > /dev/null && echo yes || echo no)"
  rm -rf "${WORK:?}/target-main" "${WORK:?}/target-swap"
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
