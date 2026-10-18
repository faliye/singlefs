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
  # 冻结副本拷成 tree-candidates，装上候选开关；九段测试放进 harness 的 tests；模型拷成 model；编译、跑九段、跑丁的探针、按件分批跑全量、
  # 汇总，最后与 $MODEL/out/ 逐行比（比时去掉 example_ordinal 这类多线程下不确定的字段）。整条约 40 分钟（本机忙时更长）。
  local cap="$REPO_ROOT/scripts/hosts/capped.sh" threads="${RERUN_THREADS:-31}" per_batch="${RERUN_PIECES_PER_BATCH:-29}"
  local candidate status=0
  mkdir -p "$WORK/tree-candidates/.claude" "$WORK/runs" "$WORK/runs-mapping" "$WORK/rollback" "$WORK/out" "$WORK/probe"
  cp -r "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$WORK/tree-candidates/"
  cp "$FROZEN/.claude/heavy-thread-default" "$WORK/tree-candidates/.claude/"
  python3 "$MODEL/patch/apply_candidates.py" "$WORK/tree-candidates"
  cp "$MODEL/tests/sonnet-nine-histories.rs" "$WORK/tree-candidates/crates/singlefs-harness/tests/c834_order_of_devices_decides_the_newer_root_judgement.rs"
  rsync -a --exclude target --exclude out "$MODEL/" "$WORK/model/"
  # 九段 × 候选（测试在候选读法下可以红，那正是要看的：退出码不当作这一步出错）
  (cd "$WORK/tree-candidates" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$cap" "$threads" cargo test -p singlefs-harness --test c834_order_of_devices_decides_the_newer_root_judgement --no-run)
  for candidate in today later_root_or_record k6 mapping_root_only pointer_slots_overwritten pointer_slots_fixed always_refuse; do
    (cd "$WORK/tree-candidates" && C834_CANDIDATE="$candidate" CARGO_TARGET_DIR="$WORK/target" nice -n 19 \
      bash "$cap" "$threads" cargo test -p singlefs-harness --test c834_order_of_devices_decides_the_newer_root_judgement -- --nocapture --test-threads=1 \
      > "$WORK/out/nine-$candidate.log" 2>&1) || true
  done
  python3 "$MODEL/tools/nine-matrix.py" "$WORK/out" today later_root_or_record k6 mapping_root_only pointer_slots_overwritten pointer_slots_fixed always_refuse > "$WORK/out/nine-matrix.txt"
  # 崩溃状态全量的原型
  (cd "$WORK/model" && nice -n 19 bash "$cap" "$threads" cargo build --release --target-dir "$WORK/target-model")
  local binary="$WORK/target-model/release/c834-attack-model"
  for candidate in today k6 pointer_slots_overwritten pointer_slots_fixed mapping_root_only; do
    C834_CANDIDATE="$candidate" nice -n 19 bash "$cap" "$threads" "$binary" root_units_and_record run 16385 16400 1 none > "$WORK/probe/probe-ding-$candidate.out" 2> /dev/null
  done
  python3 "$MODEL/tools/ding-summary.py" "$WORK/probe" today k6 pointer_slots_overwritten pointer_slots_fixed mapping_root_only > "$WORK/out/ding-summary.txt"
  python3 "$MODEL/tools/pieces-r2.py" 600 > "$WORK/pieces-r2.txt"
  python3 "$MODEL/tools/mapping-pieces.py" > "$WORK/pieces-mapping.txt"
  local list batches batch
  for list in pieces-r2:runs pieces-mapping:runs-mapping; do
    batches=$(( ($(wc -l < "$WORK/${list%%:*}.txt") + per_batch - 1) / per_batch ))
    for ((batch = 0; batch < batches; batch++)); do
      nice -n 19 bash "$cap" "$threads" bash "$MODEL/tools/run-batch-faults.sh" "$binary" "$WORK/${list%%:*}.txt" "$batch" "$per_batch" "$WORK/${list##*:}"
    done
  done
  for candidate in k6 pointer_slots_fixed later_root_or_record today mapping_root_only; do
    C834_CANDIDATE="$candidate" nice -n 19 bash "$cap" "$threads" "$binary" acquisition_rolled_back run 0 17 1 y2 > "$WORK/rollback/$candidate.out"
  done
  python3 "$MODEL/tools/aggregate.py" "$WORK"/runs/*.out > "$WORK/out/aggregate-all.txt"
  python3 "$MODEL/tools/aggregate.py" "$WORK"/runs-mapping/*.out > "$WORK/out/aggregate-mapping-tail.txt"
  python3 "$MODEL/tools/aggregate.py" "$WORK"/rollback/*.out > "$WORK/out/aggregate-rollback.txt"
  local name
  for name in aggregate-all aggregate-mapping-tail aggregate-rollback nine-matrix ding-summary; do
    if diff <(sed -E 's/ example_ordinal=[0-9]+//' "$MODEL/out/$name.txt") <(sed -E 's/ example_ordinal=[0-9]+//' "$WORK/out/$name.txt") > "$WORK/out/$name.diff"; then
      echo "name=rerun_compare $name=same"
    else
      echo "name=rerun_compare $name=differs（见 $WORK/out/$name.diff）"
      status=1
    fi
  done
  return "$status"
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
