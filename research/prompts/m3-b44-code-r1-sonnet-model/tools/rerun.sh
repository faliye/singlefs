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
  # 正推腿 m3-b44-code-r1：冻结副本上跑 core 库测试基线与三个 harness 测试目标的基线，再逐行把变异表里点名的那几行改坏、跑点名的测试、还原。
  # RERUN_ROWS 可缩小行范围（默认全部）；RERUN_THREADS 默认 31（派发提示的「线程上限」）。每行汇总打进 $WORK/summary.txt，
  # 与 $MODEL/out/summary.txt 逐行比。RERUN_SKIP_BASELINE 设了就不跑基线（只跑变异行）；RERUN_TARGET_DIR 指一个已有的编译目录（分段跑时复用）。变异表读 $FROZEN/crates/mutations.tsv，被改的文件只在 $WORK/tree 里。
  local threads="${RERUN_THREADS:-31}"
  local rows="${RERUN_ROWS-3810 3812 3813 3814 3815 3816 3817 3818 3821 3822 3823 3824 3825 40 3858 3859 3860 3861}"
  local capped="$REPO_ROOT/scripts/hosts/capped.sh"
  local table="$FROZEN/crates/mutations.tsv"
  local row target
  mkdir -p "$WORK/tree" "$WORK/logs"
  # --no-times：拷出来的文件取当下的修改时间。保留冻结副本的旧修改时间时，共用的编译目录里别的树留下的产物会被 cargo 当成还新鲜（量到过：基线红在别的变异留下的产物上）。
  rsync -a --no-times --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  export CARGO_TARGET_DIR="${RERUN_TARGET_DIR:-$WORK/target}"
  : > "$WORK/summary.txt"
  if [[ -z "${RERUN_SKIP_BASELINE:-}" ]]; then
  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-core --lib 目标; 范围=1 个测试目标; 为什么=核正推腿各格的 core 库用例在冻结副本上基线全绿' \
    bash "$capped" "$threads" cargo test --offline -p singlefs-core --lib) > "$WORK/logs/baseline-core-lib.log" 2>&1 || true
  echo "baseline core-lib: $(grep -m1 '^test result' "$WORK/logs/baseline-core-lib.log" | sed 's/; finished in [0-9.]*s//')" >> "$WORK/summary.txt"
  for target in newer_unreadable_root_passes_only_when_both_copies_of_one_pointer_are_newer \
                system_configuration_slot_with_a_foreign_node_size_is_not_chosen \
                opening_a_container_past_the_largest_number_returns_an_error; do
    (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的一个测试目标; 范围=1 个测试目标; 为什么=核正推腿 K1 K3 K6 三格点名的 harness 用例在冻结副本上基线全绿' \
      bash "$capped" "$threads" cargo test --offline -p singlefs-harness --test "$target") > "$WORK/logs/baseline-$target.log" 2>&1 || true
    echo "baseline harness-$target: $(grep -m1 '^test result' "$WORK/logs/baseline-$target.log" | sed 's/; finished in [0-9.]*s//')" >> "$WORK/summary.txt"
  done
  fi
  for row in $rows; do
    bash "$capped" "$threads" python3 "$MODEL/tools/mutrow.py" "$table" "$WORK/tree" "$row" "$WORK/logs/mut-$row.log" >> "$WORK/summary.txt" || true
  done
  # 探针（K4）：RERUN_PROBE=new 用冻结副本的代码，RERUN_PROBE=old 把码 2 码 3 指针写者的断言临时改回改前的 `<=`（只在 $WORK/tree 里改）。
  # 探针造一份坏镜像（墓碑树根里指向第一片容器的子指针，两条位置条目同一块盘，读得通），挂载后发一次要重写根的 unlink 意图。
  if [[ -n "${RERUN_PROBE:-}" ]]; then
    cp "$MODEL/probe/probe_equal_device_pointer_in_a_copied_container_tree_entry.rs" "$WORK/tree/crates/singlefs-harness/tests/"
    if [[ "$RERUN_PROBE" == old ]]; then
      python3 - "$WORK/tree/crates/singlefs-core/src/pointer.rs" <<'PY'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
before = "            self.location_entries_ascend_by_device().is_ok(),\n"
assert text.count(before) == 1
open(path, "w", encoding="utf-8").write(text.replace(before, "            self.locations[0].device <= self.locations[1].device,\n"))
PY
    fi
    (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的探针目标 probe_equal_device_pointer_in_a_copied_container_tree_entry; 范围=1 个测试目标; 为什么=K4 探针：盘上读来的容器树子指针两条位置条目同盘时，被重写的根照抄它，看新旧断言的结局' \
      bash "$capped" "$threads" cargo test --offline -p singlefs-harness --test probe_equal_device_pointer_in_a_copied_container_tree_entry -- --nocapture) > "$WORK/logs/probe-$RERUN_PROBE.log" 2>&1 || true
    grep -h '^probe ' "$WORK/logs/probe-$RERUN_PROBE.log" | sed "s/^/probe-$RERUN_PROBE: /" >> "$WORK/summary.txt"
  fi
  cat "$WORK/summary.txt"
  if [[ -f "$MODEL/out/summary.txt" ]]; then
    if diff <(sort "$MODEL/out/summary.txt") <(sort "$WORK/summary.txt") > "$WORK/summary.diff"; then
      echo "name=rerun_compare same"
    else
      cat "$WORK/summary.diff"
      echo "name=rerun_compare differ"
    fi
  else
    echo "name=rerun_compare no_reference"
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
