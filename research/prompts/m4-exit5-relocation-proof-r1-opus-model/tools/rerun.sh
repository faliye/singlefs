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

# ── 这一腿要跑的步骤：腿把这一段换成自己的 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 经 "$REPO_ROOT/scripts/hosts/capped.sh" 起，
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
# 这一腿：Verus 反例（两份冻结证明原样拷来、只加一份攻方文件；基线与两份必红变体）、harness 副本上的对拍原型（带整理与去掉整理两份历史）、
# 单元闭包与核内占比。环境：VERUS_BIN 不设取本机 formal-tools 那一份；整条外面经 run-with-memory-cap.sh 包一层，里面不再套。
# 末了把 $WORK/summary.txt 与 $MODEL/out/summary.txt 逐行比，不同退 1。
rerun_steps() {
  mkdir -p "$WORK/tree"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  local verus="${VERUS_BIN:-$HOME/.local/share/singlefs/formal-tools/verus/verus-x86-linux/verus}"
  local summary="$WORK/summary.txt" rc
  : > "$summary"
  # 一、Verus
  mkdir -p "$WORK/verus"
  cp "$FROZEN/crates/singlefs-replay-prefix/src/gc_crash_proofs.rs" "$FROZEN/crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs" \
    "$MODEL/verus/attack_gc_protocol_and_p4.rs" "$WORK/verus/"
  printf 'use vstd::prelude::*;\nmod gc_crash_proofs;\nmod compaction_plan_proofs;\nverus! {\nfn main() {}\n}\n' > "$WORK/verus/baseline.rs"
  sed 's/    assert(!gc_view_matches(before, s5.disk, logical));/    assert(gc_view_matches(before, s5.disk, logical));/' \
    "$WORK/verus/attack_gc_protocol_and_p4.rs" > "$WORK/verus/must_red_rollback.rs"
  sed 's/    assert(moved(3) != slots(3)) by { assert(moved(3)\[0\] != slots(3)\[0\]); }/    assert(moved(3) == slots(3));/' \
    "$WORK/verus/attack_gc_protocol_and_p4.rs" > "$WORK/verus/must_red_overlap.rs"
  cmp -s "$WORK/verus/attack_gc_protocol_and_p4.rs" "$WORK/verus/must_red_rollback.rs" && { echo "必红变体没改到"; return 1; }
  cmp -s "$WORK/verus/attack_gc_protocol_and_p4.rs" "$WORK/verus/must_red_overlap.rs" && { echo "必红变体没改到"; return 1; }
  local name
  for name in baseline attack_gc_protocol_and_p4 must_red_rollback must_red_overlap; do
    if (cd "$WORK/verus" && nice -n 19 "$verus" "$name.rs") > "$WORK/verus/$name.out" 2>&1; then rc=0; else rc=$?; fi
    echo "name=verus file=$name rc_is_zero=$([[ $rc -eq 0 ]] && echo yes || echo no) $(grep -o 'verification results:: .*' "$WORK/verus/$name.out" | head -n 1)" | tee -a "$summary"
  done
  # 二、harness 副本上的对拍原型
  cp "$MODEL/harness-twin/gc_twin_rollback_and_floor.rs" "$WORK/tree/crates/singlefs-harness/tests/"
  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 gc_twin_rollback_and_floor; 范围=1 个测试目标; 为什么=攻方腿复跑自己的对拍原型' \
    CARGO_TARGET_DIR="$WORK/build" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 \
    cargo test --offline -p singlefs-harness --test gc_twin_rollback_and_floor --no-run) > "$WORK/build-twin.txt" 2>&1
  local binary
  binary="$(grep -o '(/[^)]*gc_twin_rollback_and_floor-[0-9a-f]*)' "$WORK/build-twin.txt" | tr -d '()' | head -n 1)"
  [[ -x "$binary" ]] || { echo "没编出对拍原型"; tail -n 20 "$WORK/build-twin.txt"; return 1; }
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 gc_twin_rollback_and_floor; 范围=1 个测试目标; 为什么=攻方腿复跑自己的对拍原型' \
    nice -n 19 "$binary" --nocapture --test-threads 2 > "$WORK/twin.txt" 2>&1
  grep '^name=twin ' "$WORK/twin.txt" | sort > "$WORK/twin-lines.txt"
  grep '^name=reachable ' "$WORK/twin.txt" | sort > "$WORK/reachable-lines.txt"
  grep '^name=twin-summary\|^name=reachable-summary' "$WORK/twin.txt" | sort | tee -a "$summary"
  echo "name=twin-differing-by-kind rollback=$(grep -c 'rollback-ring-root.*same_outcome_class=false\|rollback-ring-root.*same_file=false' "$WORK/twin-lines.txt") raise_floor=$(grep -c 'raise-floor.*same_outcome_class=false' "$WORK/twin-lines.txt") in_cases_without_a_batch=$(grep 'batches=0)' "$WORK/twin-lines.txt" | grep -c 'same_outcome_class=false\|same_file=false')" | tee -a "$summary"
  # 三、单元闭包与核内占比
  (cd "$WORK/tree" && CARGO_TARGET_DIR="$WORK/build" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 \
    cargo build --release --offline -p singlefs-unit-graph --bin unit-graph) > "$WORK/build-unit-graph.txt" 2>&1
  local entry file
  for entry in transaction::publish_relocation_batch mount::roll_back_by_a_forward_publish; do
    file="$WORK/closure-${entry//::/__}.txt"
    "$WORK/build/release/unit-graph" closure "$WORK/tree" "singlefs_core::$entry" --toolchain-line 'host: x86_64-unknown-linux-gnu' --threads 31 \
      > "$file" 2> "$file.err"
    awk -F'\t' -v entry="$entry" '$1=="fn"{split($3,a,":"); split(a[2],b,"-"); lines=b[2]-b[1]+1; f++; L+=lines; if (a[1] ~ /singlefs-replay-prefix/){kf++; kl+=lines}} END {print "name=closure entry=" entry " fn=" f " fn_lines=" L " kernel_fn=" kf " kernel_lines=" kl}' "$file" | tee -a "$summary"
  done
  for name in mount::mapping_keys_whose_target_locations_differ_from_the_current_mapping mount::version_to_publish_the_rollback_after \
      transaction::relocation_checked_against_the_version gc_crash_proofs::released_record_can_be_reclaimed; do
    echo "name=membership fn=$name in_relocation_publish=$(grep -c "	singlefs_[a-z_]*::$name	" "$WORK/closure-transaction__publish_relocation_batch.txt") in_rollback=$(grep -c "	singlefs_[a-z_]*::$name	" "$WORK/closure-mount__roll_back_by_a_forward_publish.txt")" | tee -a "$summary"
  done
  awk -F'\t' '$1=="fn" && ($2 ~ /::relocation_checked_against_the_version$|::allocate_relocation_destination$|::persist_publish_writes$|::prepare_the_version_publish_relocating_units$|::settle_the_allocation_record_tree$/) {split($3,a,":"); split(a[2],b,"-"); s+=b[2]-b[1]+1; n++} END {print "name=lifted-upper-bound fns=" n " body_lines=" s}' \
    "$WORK/closure-transaction__publish_relocation_batch.txt" | tee -a "$summary"
  if diff "$MODEL/out/summary.txt" "$summary" > "$WORK/summary.diff" 2>&1; then
    echo "name=rerun-compare out/summary.txt same=yes"
  else
    echo "name=rerun-compare out/summary.txt same=no"; cat "$WORK/summary.diff"; return 1
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
