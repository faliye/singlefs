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
#       连带红的格：「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
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
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引；cargo 与 verus 都经 "$REPO_ROOT/scripts/hosts/capped.sh" 31 起（线程上限 31）。
# 每一步的输出去掉耗时之后写进 $WORK/out/<名>.txt，与 $MODEL/out/<名>.txt 逐行比；对不上的件数就是退出码（0 = 全对上）。
MISMATCHES=0

capped() { bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 nice -n 19 "$@"; }

compare_with_reference() {   # compare_with_reference <名>
  if diff -u "$MODEL/out/$1.txt" "$WORK/out/$1.txt" > "$WORK/out/$1.diff"; then
    echo "name=rerun_compare item=$1 result=same"
  else
    echo "name=rerun_compare item=$1 result=DIFFERENT"; cat "$WORK/out/$1.diff"
    MISMATCHES=$((MISMATCHES + 1))
  fi
}

closure_table() {   # closure_table <仓副本> <输出名>：八个入口的闭包规模与核内占比
  local tree="$1" name="$2"
  bash "$MODEL/tools/closures.sh" "$WORK/build/ug/release/unit-graph" "$tree" "$WORK/out/closures-$name" \
    singlefs_core::transaction::publish_relocation_batch singlefs_core::transaction::relocation_checked_against_the_version \
    singlefs_core::transaction::prepare_the_version_publish_relocating_units singlefs_core::transaction::persist_publish_writes \
    singlefs_core::mounted_session::run_compaction_on singlefs_core::compaction_plan::plan_batch \
    singlefs_core::compaction_plan::choose_region singlefs_core::recovery::replay_journal \
    | sed -E 's/，[0-9]+ 线程，[0-9.]+ 秒$//' > "$WORK/out/closure-$name.txt"
}

verus_count() {   # verus_count <crate 目录> <输出名>
  THREADS=31 capped bash "$MODEL/tools/verus-run.sh" "$1" "$WORK/out/verus-$2" > "$WORK/out/verus-$2.raw" 2>&1 || true
  cat "$WORK/out/verus-$2.raw" > "$WORK/out/verus-$2.txt"
}

rerun_steps() {
  mkdir -p "$WORK/out" "$WORK/build" "$WORK/tree"
  rsync -a --exclude target --exclude .git "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$WORK/tree/"
  mkdir -p "$WORK/tree/.claude" && cp "$FROZEN/.claude/heavy-thread-default" "$WORK/tree/.claude/" 2>/dev/null || true
  rsync -a "$WORK/tree/" "$WORK/tree-base/"
  cp "$MODEL/src/relocation_execution_proofs.rs" "$WORK/tree/crates/singlefs-replay-prefix/src/"
  (cd "$WORK/tree" && patch -p1 < "$MODEL/patches/lib-mod-line.patch" && patch -p1 < "$MODEL/patches/core-glue-item1.patch")
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"

  # 1 闭包规模与核内占比：冻结副本原样（base）与接上条目判定核的副本（after）
  CARGO_TARGET_DIR="$WORK/build/ug" capped cargo build --release --offline --manifest-path "$WORK/tree-base/Cargo.toml" -p singlefs-unit-graph --bin unit-graph > "$WORK/out/build-unit-graph.raw" 2>&1
  closure_table "$WORK/tree-base" base;  compare_with_reference closure-base
  closure_table "$WORK/tree" after;      compare_with_reference closure-after

  # 2 Verus：整个 crate 直接起（不经门禁脚本）。base 88 条（gate 登记 88），after 加原型模块
  verus_count "$WORK/tree-base/crates/singlefs-replay-prefix" base; compare_with_reference verus-base
  verus_count "$WORK/tree/crates/singlefs-replay-prefix" after;     compare_with_reference verus-after

  # 3 手工必红世界：八行，各改坏一处，Verus 必须报证明错
  capped python3 -I "$MODEL/tools/red-worlds.py" "$WORK/tree/crates/singlefs-replay-prefix" "$MODEL/tools/red-worlds.tsv" "$WORK/out" > "$WORK/out/red-worlds.txt" 2>&1
  compare_with_reference red-worlds

  # 4 core 直验探针：七个小文件，各取 core 里一处写法的形状
  cp -r "$MODEL/probes" "$WORK/probes"
  capped bash "$MODEL/tools/probe.sh" "$WORK/probes" > "$WORK/out/probes.txt" 2>&1
  compare_with_reference probes

  # 5 闭包里 fn 用到的写法静态扫描（按子串，不是解析）
  python3 -I "$MODEL/tools/closure_constructs.py" "$WORK/tree-base" "$WORK/out/closures-base/closure-publish_relocation_batch.txt" > "$WORK/out/constructs.txt"
  compare_with_reference constructs

  # 6 接上条目判定核的 singlefs-core 能编过（cargo check，无新警告）
  CARGO_TARGET_DIR="$WORK/build/check" capped cargo check --offline --manifest-path "$WORK/tree/Cargo.toml" -p singlefs-core > "$WORK/out/core-check.raw" 2>&1
  echo "core_check_rc=$? warnings=$(grep -c '^warning' "$WORK/out/core-check.raw")" > "$WORK/out/core-check.txt"
  compare_with_reference core-check

  rm -rf "$WORK/build" "$WORK/tree-base" "$WORK/tree"
  echo "name=rerun_result mismatches=$MISMATCHES"
  return "$MISMATCHES"
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
