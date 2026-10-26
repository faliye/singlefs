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
# 辩方腿的后置分类原型：池级 checker 原样判每一张镜像，各候选在原型里从镜像字段后置分类（不改 walk.rs）。
# 全跑：S = 4、5、7、8 各一批，作业数 S=4 共 52、S=5 共 58、S=7 共 8、S=8 共 93（S=8 的 Z2 两个洞格拆 27 片，每个洞的历史一片）；
# 单件限时 RERUN_JOB_LIMIT 秒（默认 3000）。本机负载高时每件很慢：这一条整条放后台、经内存包装跑，或把 $MODEL/tools/peer-batch.sh 经第二台跑。
# RERUN_QUICK=1：只跑 S=4 的格 A 与格 G 和给定状态原型（约几分钟），只核这几格的汇总。RERUN_THREADS 取线程上限（默认 31）。
# 末尾对 $MODEL/out/final/ 里的 summary-S<S>.txt 与 given-states.txt 逐行比（作业分片的次序不影响汇总：汇总按 ROW / SCAN 行聚合）。
jobs_for() {   # jobs_for <S>：打出这个 S 的作业，一行一个
  local s="$1" k
  case "$s" in
    4|5) bash "$MODEL/tools/job-list.sh" "$s" ;;
    7) bash "$MODEL/tools/job-list.sh" 7 a g ;;
    8)
      bash "$MODEL/tools/job-list.sh" 8 a b f g floor alt
      for ((k = 0; k < 27; k++)); do echo "8:z2_two_holes_in_the_ring:$k/27"; done ;;
  esac
}

summarize_and_compare() {   # summarize_and_compare <S>
  local s="$1" merged="$WORK/all-S$s.txt" file
  : > "$merged"
  for file in $(ls "$WORK/batch-S$s/out/$s"-*.txt | sort); do
    sed 's/^test [a-z0-9_]* \.\.\. //' "$file" | grep -E '^(ROW|SCAN) ' >> "$merged"
  done
  python3 "$MODEL/tools/summary.py" "$merged" > "$WORK/summary-S$s.txt"
  if diff "$MODEL/out/final/summary-S$s.txt" "$WORK/summary-S$s.txt" > "$WORK/summary-S$s.diff"; then
    echo "name=summary-S$s same_as=out/final/summary-S$s.txt lines=$(wc -l < "$WORK/summary-S$s.txt")"
  else
    echo "name=summary-S$s DIFFERS see=$WORK/summary-S$s.diff"
  fi
}

rerun_steps() {
  local tree="$WORK/tree" classifier given s limit="${RERUN_JOB_LIMIT:-3000}" threads="${RERUN_THREADS:-31}" batch
  local -a jobs list_of_s
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  mkdir -p "$tree/.claude"
  rsync -a --exclude target --exclude .git "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$tree/"
  cp "$FROZEN/.claude/heavy-thread-default" "$tree/.claude/"
  cp "$MODEL/tests/i718_r3_independent.rs" "$MODEL/tests/i718_r3_given_states.rs" "$tree/crates/singlefs-harness/tests/"
  python3 "$MODEL/tools/slots-from-env.py" "$tree/crates/singlefs-harness/tests/common_root_ring_holes/mod.rs"
  echo "name=walk_rs_sha256 $(sha256sum "$tree/crates/singlefs-checker/src/walk.rs" | cut -d' ' -f1)（原型不改它；开工快照 judged-sha256.txt 里是 1ec7571f…）"
  ( cd "$tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 i718_r3_independent 与 i718_r3_given_states; 范围=2 个测试目标; 为什么=复跑辩方腿后置分类原型（release 只编）' \
      CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "$threads" \
      cargo test --release -p singlefs-harness --test i718_r3_independent --test i718_r3_given_states --no-run > "$WORK/build.out" 2>&1 )
  classifier="$(ls -t "$WORK"/target/release/deps/i718_r3_independent-* | grep -v '\.d$' | head -n 1)"
  given="$(ls -t "$WORK"/target/release/deps/i718_r3_given_states-* | grep -v '\.d$' | head -n 1)"
  echo "name=binaries classifier=$classifier given=$given"
  list_of_s=(4 5 7 8)
  if [[ "${RERUN_QUICK:-}" == 1 ]]; then list_of_s=(4); fi
  for s in "${list_of_s[@]}"; do
    mapfile -t jobs < <(jobs_for "$s")
    if [[ "${RERUN_QUICK:-}" == 1 ]]; then mapfile -t jobs < <(bash "$MODEL/tools/job-list.sh" 4 a g); fi
    batch="$WORK/batch-S$s"
    mkdir -p "$batch/tools"
    cp "$MODEL/tools/run-jobs.sh" "$batch/tools/"
    if [[ "$s" == 4 ]]; then given_binary="$given"; else given_binary=none; fi
    bash "$MODEL/tools/peer-batch.sh" "$batch" "$classifier" "$given_binary" "$limit" "${jobs[@]}" > "$WORK/batch-S$s.log" 2>&1
    tail -n 3 "$WORK/batch-S$s.log"
    summarize_and_compare "$s"
  done
  cat "$WORK"/batch-S4/out/given-*.txt | sed 's/^test [a-z0-9_]* \.\.\. //' | grep '^GIVEN' | sort > "$WORK/given-states.txt"
  sort "$MODEL/out/final/given-states.txt" > "$WORK/given-states.expected"
  if diff "$WORK/given-states.expected" "$WORK/given-states.txt" > "$WORK/given-states.diff"; then
    echo "name=given-states same_as=out/final/given-states.txt lines=$(wc -l < "$WORK/given-states.txt")"
  else
    echo "name=given-states DIFFERS see=$WORK/given-states.diff"
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
