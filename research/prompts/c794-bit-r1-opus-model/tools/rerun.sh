#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。（本份：c794-bit-r1 攻方腿）
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
  # C794 盘上隔离位第一轮攻方腿：冻结副本拷进 $WORK/tree，打 patches/attack-switches.diff（读法开关、原型二进制、取样计数），
  # 编原型与 harness 用例，逐个读法跑镜像场景与随机历史取样，输出写进 $WORK/out，与 $MODEL/out 逐份比。
  local cap="$REPO_ROOT/scripts/hosts/capped.sh"
  local threads="${RERUN_THREADS:-31}"
  mkdir -p "$WORK/tree" "$WORK/out"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  (cd "$WORK/tree" && patch -p1 --quiet < "$MODEL/patches/attack-switches.diff")
  export CARGO_TARGET_DIR="$WORK/target"
  (cd "$WORK/tree" && nice -n 19 bash "$cap" "$threads" cargo build --release -p singlefs-checker-tier --bin c794_attack_probe)
  (cd "$WORK/tree" && nice -n 19 bash "$cap" "$threads" cargo test --release -p singlefs-harness --no-run --test random_histories \
    --test checker_quarantine_flag_exempts_only_flagged_unreferenced_records \
    --test rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references --test release_checksum_quarantine)
  local probe="$WORK/target/release/c794_attack_probe"
  run_probe() { local name="$1"; shift; local arguments="$1"; shift
    # shellcheck disable=SC2086
    env "$@" nice -n 19 "$probe" $arguments 2>&1 | grep -E '^(Q|COUNT)' > "$WORK/out/$name.txt"; }
  run_probe q1 q1 C794_NONE=1
  run_probe q2-clear q2-image C794_NONE=1
  run_probe q2-keep q2-image C794_ROLLBACK_KEEPS_BIT=1
  run_probe q2-keep-q3-newest q2-image C794_ROLLBACK_KEEPS_BIT=1 C794_Q3=newest
  run_probe q2-clear-q3-newest q2-image C794_Q3=newest
  run_probe q5-per-record q5-image C794_NONE=1
  run_probe q5-per-unit q5-image C794_EXEMPT_READING=per_unit
  run_probe q3-every q5-image C794_Q3=every
  run_probe q3-newest q5-image C794_Q3=newest
  run_probe q4-inference-clean q4 C794_NONE=1
  run_probe q4-inference-corrupt "q4 corrupt" C794_NONE=1
  run_probe q4-bit-clean q4 C794_ISOLATED_BY_BIT=1
  run_probe q4-bit-corrupt "q4 corrupt" C794_ISOLATED_BY_BIT=1
  run_probe counts "count quarantine-publish" C794_NONE=1
  run_probe counts-rollback "count rollback-publish" C794_NONE=1
  run_probe counts-after-rollback "count after-rollback-overwrite" C794_NONE=1
  local mutant test_name
  for mutant in C794_NONE C794_MUTANT_NEW_RECORD_BIT C794_ROLLBACK_KEEPS_BIT; do
    for test_name in checker_quarantine_flag_exempts_only_flagged_unreferenced_records \
        rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references release_checksum_quarantine; do
      (cd "$WORK/tree" && env "$mutant=1" nice -n 19 bash "$cap" "$threads" cargo test --release -p singlefs-harness --test "$test_name" 2>&1 || true) \
        | grep -E '^test .*(FAILED|ok)$|^test result' | sed -E 's/finished in [0-9.]+s//' | sort > "$WORK/out/targeted-$mutant-$test_name.txt"
    done
    for test_name in random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation \
        rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms; do
      (cd "$WORK/tree" && env "$mutant=1" nice -n 19 bash "$cap" "$threads" cargo test --release -p singlefs-harness --test random_histories \
        -- --ignored --exact "$test_name" --nocapture 2>&1 || true) > "$WORK/out/rh-$mutant-$test_name.log"
      { grep -c C794_QUARANTINE_EVENT "$WORK/out/rh-$mutant-$test_name.log" || true; grep -E '^历史 |^  模型对拍|^test result' "$WORK/out/rh-$mutant-$test_name.log" | sed -E 's/finished in [0-9.]+s//'; } \
        > "$WORK/out/rh-$mutant-$test_name.txt"
      rm -f "$WORK/out/rh-$mutant-$test_name.log"
    done
  done
  local file differing=0
  for file in "$MODEL"/out/*.txt; do
    if ! diff -q "$file" "$WORK/out/$(basename "$file")" > /dev/null; then
      echo "name=rerun_differs file=$(basename "$file")"; differing=$((differing + 1))
    fi
  done
  echo "name=rerun_done compared=$(ls "$MODEL"/out/*.txt | wc -l) differing=$differing work=$WORK"
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
