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
  # 冻结副本拷成 tree，装上候选开关成 tree-candidates；模型拷成 model；编译；跑九段、三份件表（甲至戊与丁的两种新读故障、缩短的三次挂载、丁只藏 R 的记录）、
  # 入库 13 份 harness 用例在四个候选下；汇总后与 $MODEL/out/ 逐行比（汇总里的例子序号取那一格最小的序号，与件的先后无关，不用去字段）。
  local cap="$REPO_ROOT/scripts/hosts/capped.sh" threads="${RERUN_THREADS:-31}"
  mkdir -p "$WORK/tree" "$WORK/runs"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  cp -a "$WORK/tree" "$WORK/tree-candidates"
  python3 "$MODEL/patch/apply_candidates.py" "$WORK/tree-candidates"
  rsync -a --exclude target --exclude out "$MODEL/" "$WORK/model/"
  (cd "$WORK/model" && nice -n 19 bash "$cap" "$threads" cargo build --release --target-dir "$WORK/target")
  local binary="$WORK/target/release/c834-r2-attack-model"
  nice -n 19 bash "$cap" "$threads" "$binary" nine > "$WORK/runs/nine.txt"
  python3 - "$WORK/runs" <<'PIECES'
import sys
out = sys.argv[1]
lists = {
    "first": [("tree_table_and_record", 20516, "tree_and_mapping"), ("root_units_and_record", 20516, "none"), ("clean", 20511, "y2"),
              ("tree_table_and_record", 20516, "y2"), ("acquisition_rolled_back", 17, "y2"), ("root_units_and_record", 20516, "y2")],
    "three": [("three_mount_tail", 8227, "y2"), ("three_mount_tail", 8227, "y2_m0_record")],
    "partial": [("root_units_and_record", 20516, "record_only")],
}
for name, histories in lists.items():
    rows = [f"{history} {start} {min(start + 600, count)} {faults}" for history, count, faults in histories for start in range(0, count, 600)]
    open(f"{out}/pieces-{name}.txt", "w").write("\n".join(rows) + "\n")
PIECES
  local list batches batch
  for list in first three partial; do
    mkdir -p "$WORK/runs/$list"
    batches=$(( ($(wc -l < "$WORK/runs/pieces-$list.txt") + threads - 1) / threads ))
    for ((batch = 0; batch < batches; batch++)); do
      bash "$cap" "$threads" bash "$MODEL/tools/run-batch.sh" "$binary" "$WORK/runs/pieces-$list.txt" "$batch" "$threads" "$WORK/runs/$list" > "$WORK/runs/$list/batch-$batch.log"
      if grep -q 'rc=[1-9]' "$WORK/runs/$list/batch-$batch.log"; then
        echo "name=rerun_piece_failed list=$list batch=$batch"
        return 1
      fi
    done
    python3 "$MODEL/tools/aggregate.py" "$WORK"/runs/$list/*.out > "$WORK/runs/agg-$list.txt"
  done
  python3 "$MODEL/tools/derive.py" "$WORK"/runs/first/*.out "$WORK"/runs/three/*.out "$WORK"/runs/partial/*.out > "$WORK/runs/derived.txt"
  local candidate tests
  tests=$(cd "$WORK/tree-candidates" && grep -ln 'common_unwitnessed_publish' crates/singlefs-harness/tests/*.rs | xargs -n1 basename | sed 's/\.rs$//' | sed 's/^/--test /' | paste -sd' ')
  tests="$tests --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping"
  for candidate in today k6 pointer_per_kind pointer_r1; do
    # shellcheck disable=SC2086
    (cd "$WORK/tree-candidates" && C834_CANDIDATE="$candidate" nice -n 19 bash "$cap" "$threads" cargo test -p singlefs-harness $tests --no-fail-fast --target-dir "$WORK/target-tests") > "$WORK/runs/existing-$candidate.log" 2>&1 || true
    echo "candidate=$candidate binaries=$(grep -c '^test result' "$WORK/runs/existing-$candidate.log") passed=$(grep '^test result' "$WORK/runs/existing-$candidate.log" | awk '{s+=$4} END{print s}') failed=$(grep '^test result' "$WORK/runs/existing-$candidate.log" | awk '{s+=$6} END{print s}') ignored=$(grep '^test result' "$WORK/runs/existing-$candidate.log" | awk '{s+=$8} END{print s}') failed_tests=$(grep -E '\.\.\. FAILED' "$WORK/runs/existing-$candidate.log" | awk '{print $2}' | sort | paste -sd, -)"
  done > "$WORK/runs/existing-tests-summary.txt"
  local file differs=0
  for file in nine.txt agg-first.txt agg-three.txt agg-partial.txt derived.txt existing-tests-summary.txt; do
    if diff -q "$MODEL/out/$file" "$WORK/runs/$file" > /dev/null; then
      echo "name=rerun_file file=$file same"
    else
      echo "name=rerun_file file=$file differs"
      differs=1
    fi
  done
  if ((differs)); then
    echo "name=rerun_compare differs"
    return 1
  fi
  echo "name=rerun_compare all_same"
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
