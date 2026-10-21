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

# ── 这一腿要跑的步骤：m4-batch2-code-r1 正推腿（Sonnet）──
# 五步，全在 $WORK/tree（冻结副本的拷贝）里，编译目录在 $WORK/target，cargo 经 capped.sh 8 起（整条脚本要再经 run-with-memory-cap.sh 跑）：
#   s1 探针：两个现成测试文件原样拷一份、后面接 probes/ 下追加的测试，打印 P1（合法两对快照条目五个读者的结局）、P2（码 14 比码 15 多一条）、P3（必红世界红项表）
#   s2 变异 A：harness 的 discard 两行没证红的变异（录制器不数结局、切段归 UnitWrite）
#   s3 变异 B：recovery.rs 的条数相等判成「码 14 < 码 15」（码 14 比码 15 多一条的半边不判）
#   s4 变异 C：region_scan.rs 的钉住 × 隔离优先级对调、隔离槽有条目指着也判隔离，跑核心 lib 的 region_scan 用例
#   s5 变异 C 续：同一组变异跑 harness 的 compaction_plan 用例
# 每步的结局经 tools/summarize.sh 摘成「测试名 ... ok / FAILED、test result、PROBE 行」排序后的几行，与 $MODEL/out/expected.txt 逐行 diff。
rerun_steps() {
  mkdir -p "$WORK/tree" "$WORK/logs"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  export CARGO_TARGET_DIR="$WORK/target"
  local tests="$WORK/tree/crates/singlefs-harness/tests"
  local snapshot_test=tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds.rs
  cp "$tests/$snapshot_test" "$tests/probe_forward_snapshot.rs"
  cat "$MODEL/probes/append_to_snapshot_test.rs" >> "$tests/probe_forward_snapshot.rs"
  cp "$tests/scan_rebuild_agrees_with_the_walked_index.rs" "$tests/probe_forward_scan_rebuild.rs"
  cat "$MODEL/probes/append_to_scan_rebuild_test.rs" >> "$tests/probe_forward_scan_rebuild.rs"
  : > "$WORK/summary.txt"

  run_step s1 test --offline -p singlefs-harness --test probe_forward_snapshot --test probe_forward_scan_rebuild -- --nocapture --test-threads 2

  local lib="$WORK/tree/crates/singlefs-harness/src/lib.rs"
  local segments="$WORK/tree/crates/singlefs-harness/src/segments.rs"
  sed -i 's/self.0.borrow_mut().discard_outcome_counts.note(outcome);/let _ = (self, outcome);/' "$lib"
  sed -i 's/Discard => StepKind::ZeroFill,/Discard => StepKind::UnitWrite,/' "$segments"
  run_step s2 test --offline -p singlefs-harness --test block_devices_record_and_apply_discards -- --test-threads 2
  cp "$FROZEN/crates/singlefs-harness/src/lib.rs" "$lib"
  cp "$FROZEN/crates/singlefs-harness/src/segments.rs" "$segments"

  local recovery="$WORK/tree/crates/singlefs-core/src/recovery.rs"
  sed -i 's/if snapshot_inode_root_entries.len() != snapshot_extent_root_entries.len() {/if snapshot_inode_root_entries.len() < snapshot_extent_root_entries.len() {/' "$recovery"
  run_step s3 test --offline -p singlefs-harness --test probe_forward_snapshot -- --test-threads 2
  cp "$FROZEN/crates/singlefs-core/src/recovery.rs" "$recovery"

  local scan="$WORK/tree/crates/singlefs-core/src/region_scan.rs"
  sed -i 's/if pinned_slots.range(covered_slots.clone()).next().is_some() {/if pinned_slots.range(covered_slots.clone()).next().is_some() \&\& quarantined_slots.range(covered_slots.clone()).next().is_none() {/' "$scan"
  sed -i 's/if entries_pointing_here.is_empty() \&\& quarantined_slots.range(covered_slots).next().is_some() {/if quarantined_slots.range(covered_slots).next().is_some() {/' "$scan"
  run_step s4 test --offline -p singlefs-core --lib region_scan:: -- --test-threads 2
  run_step s5 test --offline -p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses -- --test-threads 2
  cp "$FROZEN/crates/singlefs-core/src/region_scan.rs" "$scan"

  echo "name=sonnet_forward_rerun summary=$WORK/summary.txt"
  if diff -u "$MODEL/out/expected.txt" "$WORK/summary.txt"; then echo "name=sonnet_forward_rerun_matches_expected result=same"; else echo "name=sonnet_forward_rerun_matches_expected result=differs"; fi
}

# run_step <节名> <cargo 参数…>：在 $WORK/tree 里经 capped.sh 8 跑 cargo，日志进 $WORK/logs，摘要追加进 $WORK/summary.txt；测试红不中止
run_step() {
  local step="$1"; shift
  (cd "$WORK/tree"; bash "$REPO_ROOT/scripts/hosts/capped.sh" 8 nice -n 19 cargo "$@") > "$WORK/logs/$step.log" 2>&1 || true
  { echo "== $step =="; bash "$MODEL/tools/summarize.sh" "$WORK/logs/$step.log"; } >> "$WORK/summary.txt"
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
