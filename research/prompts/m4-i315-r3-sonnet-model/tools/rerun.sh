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
  # 副本取自 $FROZEN 的 crates（本轮的数是 HEAD 的导出树上量的：git archive HEAD 解到一个目录当 --frozen）；被判的四份对开工快照，对不上照跑、打出来
  mkdir -p "$WORK/tree/.claude" "$WORK/logs" "$WORK/sums"
  rsync -a --exclude target --exclude .git "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$WORK/tree/"
  cp "$FROZEN/.claude/heavy-thread-default" "$WORK/tree/.claude/"
  (cd "$WORK/tree" && sha256sum -c "$REPO_ROOT/research/prompts/m4-i315-r3-snapshot/tree-sha256.txt") || echo "name=snapshot_mismatch（冻结副本与开工快照对不上，照跑）"
  python3 -I "$MODEL/tools/apply_arms_r3.py" "$WORK/tree"
  python3 -I "$MODEL/tools/apply_defects_r3.py" "$WORK/tree"
  python3 -I "$MODEL/tools/apply_common_r3.py" "$WORK/tree"
  cp "$MODEL/tools/r3_probe.rs" "$MODEL/tools/r3_worlds.rs" "$MODEL/tools/r3_fixtures.rs" "$WORK/tree/crates/singlefs-harness/tests/"
  local threads="${RERUN_THREADS:-31}"
  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test r3_probe、--test r3_worlds、--test r3_fixtures 与 --test fault_injection_fast_tier; 范围=4 个测试目标; 为什么=只编辩方腿副本里的三个探针目标与入库快档目标，复跑前先编出来' \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "$threads" cargo test --release -p singlefs-harness --test r3_probe --test r3_worlds --test r3_fixtures --test fault_injection_fast_tier --no-run) > "$WORK/logs/build.log" 2>&1
  compare_text() {   # compare_text <标签> <生成的文件>：与 out/<标签>.sum 逐行比
    if diff -q "$2" "$MODEL/out/$1.sum" > /dev/null; then echo "name=rerun_compare config=$1 same_as_out=true"; else echo "name=rerun_compare config=$1 same_as_out=false"; fi
  }
  slice() {   # slice <标签> <步数> <段数> <种子偏移> <缺陷> <权重名>
    RUN_SLICE_NO_CAP=1 bash "$MODEL/tools/run_slice.sh" "$REPO_ROOT" "$WORK/tree" "$WORK/logs" "$1" "$2" "$3" "$4" "$5" "$threads" "$6" > /dev/null
    grep -E '^(POINT|CLASS|NOENT)' "$WORK/logs/run-$1.log" > "$WORK/sums/$1.lines"
    R3_SHOW=200 python3 -I "$MODEL/tools/r3_summarize.py" "$WORK/sums/$1.lines" > "$WORK/sums/$1.sum"
    compare_text "$1" "$WORK/sums/$1.sum"
    grep -E '^(TALLY today|TALLY exempt|TALLY gen |TALLY cmp|TALLY both |TALLY jrn )' "$WORK/sums/$1.sum" || true
  }
  # 入库快档（副本里它不读任何臂，等于入库那一条原样跑）
  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test fault_injection_fast_tier 里的快档一条; 范围=1 个测试目标里 1 条用例; 为什么=复跑入库快档 24 段，对第二轮判决 K2 引的快档数' \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "$threads" cargo test --release -p singlefs-harness --test fault_injection_fast_tier -- --ignored --exact fault_injection_fast_tier_returns_errors_instead_of_panicking --nocapture) > "$WORK/logs/fast-tier-in-repo.log" 2>&1
  grep -F '新发现 0' "$WORK/logs/fast-tier-in-repo.log" | sed 's/^/name=in_repo_fast_tier /' || echo "name=in_repo_fast_tier 新发现不是 0"
  slice fast-20x24 20 24 0 none broad
  slice k21 60 1 1282 none broad
  slice leak_once-60x31-o2000 60 31 2000 leak_once broad
  slice extra_leak-20x24 20 24 0 extra_leak broad
  slice one_device-20x24 20 24 0 one_device broad
  slice leak_every-20x24 20 24 0 leak_every broad
  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test r3_worlds; 范围=1 个测试目标; 为什么=复跑辩方腿固定盘面十二个世界在九个臂下的判定，只跑自己的探针目标' \
    DEFENSE_DUMP=1 nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "$threads" cargo test --release -p singlefs-harness --test r3_worlds -- --nocapture) > "$WORK/logs/run-worlds.log" 2>&1
  python3 -I "$MODEL/tools/r3_worlds_table.py" "$WORK/logs/run-worlds.log" --all --facts > "$WORK/sums/worlds.sum"
  compare_text worlds "$WORK/sums/worlds.sum"
  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test r3_fixtures; 范围=1 个测试目标; 为什么=复跑辩方腿副本里入库固定盘面在九个臂下的判定，只跑自己的探针目标' \
    DEFENSE_DUMP=1 nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "$threads" cargo test --release -p singlefs-harness --test r3_fixtures -- --nocapture) > "$WORK/logs/run-fixtures.log" 2>&1
  grep -h "^SCEN" "$WORK/logs/run-fixtures.log" | sed 's/ 全部违例=.*//' | sort > "$WORK/sums/fixtures.sum"
  compare_text fixtures "$WORK/sums/fixtures.sum"
  # 放大档与别的比重（挂钟长，RERUN_FULL=1 才跑）
  if [[ "${RERUN_FULL:-0}" == 1 ]]; then
    local offset
    for offset in 1000 1050 1100 1150 1200 1250; do
      slice "big-o$offset" 60 50 "$offset" none broad
    done
    cat "$WORK"/sums/big-o*.lines > "$WORK/sums/big-all.lines"
    R3_SHOW=0 python3 -I "$MODEL/tools/r3_summarize.py" "$WORK/sums/big-all.lines" > "$WORK/sums/big-all.sum"
    compare_text big-all "$WORK/sums/big-all.sum"
    slice extra_leak-60x31-o2000 60 31 2000 extra_leak broad
    slice leak_every-60x31-o2000 60 31 2000 leak_every broad
    slice w-rollback 60 40 3000 none rollback
    slice w-reuse 60 25 3000 none reuse
    slice w-compaction 60 25 3000 none compaction
    slice w-snapshot 50 20 3000 none snapshot
  fi
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
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
