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
  mkdir -p "$WORK/tree" "$WORK/logs"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  # 冻结副本根上只有 crates、Cargo.toml、Cargo.lock：补仓里的 .cargo 与登记的线程数文件（harness 的 include_str! 要它）。
  mkdir -p "$WORK/tree/.claude"
  cp "$REPO_ROOT/.claude/heavy-thread-default" "$WORK/tree/.claude/heavy-thread-default"
  cp -r "$REPO_ROOT/.cargo" "$WORK/tree/.cargo"
  find "$WORK/tree/crates" -type f -exec touch {} +
  export CARGO_TARGET_DIR="$WORK/target"
  local tests="$WORK/tree/crates/singlefs-harness/tests"
  local core_src="$WORK/tree/crates/singlefs-core/src"
  local harness_src="$WORK/tree/crates/singlefs-harness/src"
  cp "$MODEL"/probes/probe_forward_*.rs "$tests/"
  # 整理意图那份探针：拿冻结副本里的原用例文件当底（辅助函数照用），去掉原来的 #[test]，再接探针。
  sed 's/^#\[test\]$//' "$FROZEN/crates/singlefs-harness/tests/compaction_intent_is_logged_and_resumed_after_a_crash.rs" > "$tests/probe_forward_intent_paths.rs"
  cat "$MODEL/probes/append_to_intent_paths.rs" >> "$tests/probe_forward_intent_paths.rs"
  cp -r "$FROZEN/crates/singlefs-harness/tests/common_pointer_head_corruption" "$tests/" 2>/dev/null || true
  : > "$WORK/summary.txt"

  # 一步：run_step <步名> <范围声明> <cargo 的参数…>。输出落 logs/<步名>.log，summary 记退出码、探针打出的 name= 行（排序）、测试结果行（去掉用时）。
  run_step() {
    local name="$1" scope="$2" rc=0
    shift 2
    if [[ -n "${RERUN_ONLY:-}" && " $RERUN_ONLY " != *" $name "* ]]; then return 0; fi
    (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE="$scope" bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 8G cargo "$@" > "$WORK/logs/$name.log" 2>&1) || rc=$?
    {
      echo "step=$name exit=$rc"
      sed -n 's/.*\(name=probe_forward.*\)$/\1/p' "$WORK/logs/$name.log" | LC_ALL=C sort
      grep -E '^test result' "$WORK/logs/$name.log" | sed -E 's/^(test result: [A-Za-z]+\. [0-9]+ passed; [0-9]+ failed);.*/\1/'
      echo "step=$name failed_tests=$(grep -cE '^test .* \.\.\. FAILED' "$WORK/logs/$name.log" || true)"
    } >> "$WORK/summary.txt"
  }
  # 改一处源码：replace_once <文件> <原文> <替换文>（原文必须恰好命中一次，否则退出）；restore <文件> 换回冻结副本那份。
  replace_once() {
    python3 -I - "$1" "$2" "$3" <<'PY'
import sys
path, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(path).read()
assert text.count(old) == 1, f"原文在 {path} 里命中 {text.count(old)} 次"
open(path, "w").write(text.replace(old, new))
PY
  }
  restore() { cp "$FROZEN/${1#"$WORK/tree/"}" "$1"; }

  # ── 对照基线：已入库的几个用例在冻结副本上是绿的 ──
  run_step s00a-baseline-relocation "种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test compaction_relocation_batches_publish_through_the_mapping; 范围=1 个测试目标; 为什么=正推腿复跑搬迁一批的已入库用例，作为探针的对照基线" test --offline -p singlefs-harness --test compaction_relocation_batches_publish_through_the_mapping
  run_step s00b-baseline-snapshot-creation "种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test snapshot_creation_publish_and_the_entries_every_later_publish_carries; 范围=1 个测试目标; 为什么=正推腿复跑建快照的已入库用例，作为探针的对照基线" test --offline -p singlefs-harness --test snapshot_creation_publish_and_the_entries_every_later_publish_carries
  run_step s00c-baseline-compaction-runs "种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test compaction_runs_stop_and_discard_after_the_publish; 范围=1 个测试目标; 为什么=正推腿复跑整理运行与 discard 的已入库用例，作为探针的对照基线" test --offline -p singlefs-harness --test compaction_runs_stop_and_discard_after_the_publish
  run_step s00d-baseline-snapshot-release "种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test snapshot_release_and_destroy_through_the_deadlist; 范围=1 个测试目标; 为什么=正推腿复跑释放分流与删快照的已入库用例，作为探针的对照基线" test --offline -p singlefs-harness --test snapshot_release_and_destroy_through_the_deadlist

  # ── T7：empty_segments 增量维护对着全扫记录；变异 M1（复活不加回计数）要让探针红 ──
  run_step s01-allocator-empty-segments "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_allocator; 范围=1 个测试目标; 为什么=分配器 empty_segments 增量维护对着全扫记录重算" test --offline -p singlefs-harness --test probe_forward_allocator -- --nocapture
  replace_once "$core_src/allocator.rs" "        self.note_slots_under_an_unreleased_record(
            start,
            end,
            UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord,
        );
    }

    /// 隔离一个只被被抛弃根引用" "        /* M1 */
    }

    /// 隔离一个只被被抛弃根引用"
  run_step s02-mutation-m1-resurrection-does-not-count "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_allocator; 范围=1 个测试目标; 为什么=变异 M1（复活不加回未释放记录计数）下探针要红，证探针有判别力" test --offline -p singlefs-harness --test probe_forward_allocator
  restore "$core_src/allocator.rs"

  # ── T8：变异 M2（回收记下的段一律发 discard，mutations.tsv 第 4453 行）下 harness 层三条用例红 ──
  replace_once "$core_src/allocator.rs" "            if self.segment_is_reusable_as_a_whole(segment_index) {
                segments_to_discard.push" "            if true {
                segments_to_discard.push"
  run_step s03-mutation-m2-discard-every-noted-segment "种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test compaction_runs_stop_and_discard_after_the_publish 里按名字筛的 3 条; 范围=1 个测试目标; 为什么=变异 M2 下不超时的 harness 层用例红不红" test --offline -p singlefs-harness --test compaction_runs_stop_and_discard_after_the_publish -- a_segment_with_a_slot_still_allocated_is_not_discarded a_segment_holding_an_isolated_slot_is_not_discarded a_segment_held_until_the_floor_takes_effect_is_discarded_only_after_the_hold_is_released
  restore "$core_src/allocator.rs"

  # ── T11：模型 + 真实现对同一段快照历史的抬 F 上限；变异 M3（mutations.tsv 第 4346 行）下模型探针红、原有 30 条 model 单测仍绿 ──
  python3 -I - "$harness_src/model.rs" "$MODEL/probes/append_to_model_tests.rs" <<'PY'
import sys
path, extra = sys.argv[1], sys.argv[2]
text = open(path).read().rstrip("\n")
assert text.endswith("}")
open(path, "w").write(text[:-1] + open(extra).read() + "}\n")
PY
  run_step s04-model-ceiling-probe "种类=target; 跑什么=cargo test -p singlefs-harness 的 --lib 目标里按名字筛的 1 个探针; 范围=1 个测试目标; 为什么=模型对同一段快照历史的抬 F 上限，未变异时应当是 10" test --offline -p singlefs-harness --lib -- probe_forward_two_roots --nocapture
  replace_once "$harness_src/model.rs" "            snapshot_roots: self
                .snapshots
                .iter()
                .map(|snapshot| {" "            snapshot_roots: self
                .snapshots
                .iter()
                .filter(|_| false)
                .map(|snapshot| {"
  run_step s05a-mutation-m3-model-probe "种类=target; 跑什么=cargo test -p singlefs-harness 的 --lib 目标里按名字筛的 1 个探针; 范围=1 个测试目标; 为什么=变异 M3（非空不看快照条目）下模型探针要红，上限变成 9" test --offline -p singlefs-harness --lib -- probe_forward_two_roots --nocapture
  run_step s05b-mutation-m3-shipped-model-tests "种类=target; 跑什么=cargo test -p singlefs-harness 的 --lib 目标里 model 模块的原有单测; 范围=1 个测试目标; 为什么=变异 M3 下原有的 model 单测仍然全绿，证这行变异没被杀" test --offline -p singlefs-harness --lib -- model:: --skip probe_forward
  restore "$harness_src/model.rs"
  run_step s06-real-ceiling-probe "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_ceiling; 范围=1 个测试目标; 为什么=真实现对同一段快照历史的抬 F 上限，应当是建第一个快照那一版的 txg" test --offline -p singlefs-harness --test probe_forward_ceiling -- --nocapture
  run_step s07-snapshot-root-relocation "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_snapshot_root_relocation; 范围=1 个测试目标; 为什么=整理搬只被快照引用的根单元时码 14 15 条目的根指针与非空判法" test --offline -p singlefs-harness --test probe_forward_snapshot_root_relocation -- --nocapture
  run_step s08-reference-filesystem "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_reference_filesystem; 范围=1 个测试目标; 为什么=RefFS 在建快照与删快照写了之后报错再崩溃时快照集合怎么定" test --offline -p singlefs-harness --test probe_forward_reference_filesystem -- --nocapture

  # ── T1 T3 T4：快照条目随各类发布续上、销毁中的快照、读树表的读者 ──
  run_step s09-snapshot-cells "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_snapshot_cells; 范围=1 个测试目标; 为什么=快照条目随各类发布续上与销毁中的快照在建快照回退读快照上被当成什么" test --offline -p singlefs-harness --test probe_forward_snapshot_cells -- --nocapture --test-threads 1
  run_step s10-readers "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_readers; 范围=1 个测试目标; 为什么=读树表的各个读者对带码 14 15 码 8 码 12 的树表放行还是拒" test --offline -p singlefs-harness --test probe_forward_readers -- --nocapture

  # ── T5 T6 T10：搬迁与删快照、ckpt_cost、segs0、意图三条路、logged_ops 读者 ──
  run_step s11-relocation "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_relocation; 范围=1 个测试目标; 为什么=被快照钉住的单元搬走再删快照放的是目的地，整理的一批写的保留池一类单元对 ckpt_cost" test --offline -p singlefs-harness --test probe_forward_relocation -- --nocapture --test-threads 1
  run_step s12-segs0 "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_segs0; 范围=1 个测试目标; 为什么=意图里的 segs0 与签发那次发布写下的记账第 11 项差不差" test --offline -p singlefs-harness --test probe_forward_segs0 -- --nocapture
  run_step s13-intent-paths "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_intent_paths; 范围=1 个测试目标; 为什么=带活整理意图的新镜像在只读挂载可写挂载回退三条路上各被当成什么，以及坏条目" test --offline -p singlefs-harness --test probe_forward_intent_paths -- --nocapture --test-threads 1 probe_forward
  run_step s14-logged-ops-decode "种类=target; 跑什么=cargo test -p singlefs-harness 的探针 probe_forward_logged_ops_decode; 范围=1 个测试目标; 为什么=logged_ops 根的读者对多于一条意图不升序保留种类码补零非零各拒成什么" test --offline -p singlefs-harness --test probe_forward_logged_ops_decode -- --nocapture

  # ── T9：变异 M4（选段数记录条数不数槽数）下，小输入对拍与 harness 层选段用例都不红 ──
  replace_once "$core_src/compaction_plan.rs" "        // 迭代上界是这条记录的跨度；只数落在整段里的槽。
        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {" "        // 迭代上界是这条记录的跨度；只数落在整段里的槽。
        for slot in record.slot.0..record.slot.0 + 1 {"
  run_step s15a-mutation-m4-cross-check "种类=target; 跑什么=cargo test -p singlefs-core 的 --lib 目标里按名字筛的 compaction_plan::tests; 范围=1 个测试目标; 为什么=变异 M4（选段数记录条数不数槽数）下选段同证过的核逐项对拍还绿不绿" test --offline -p singlefs-core --lib -- compaction_plan::tests
  run_step s15b-mutation-m4-harness-plan-tests "种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test compaction_plan_chooses_regions_and_batches_by_the_clauses; 范围=1 个测试目标; 为什么=变异 M4 下 harness 层选段用例还绿不绿" test --offline -p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses
  restore "$core_src/compaction_plan.rs"

  echo "name=sonnet_forward_rerun summary=$WORK/summary.txt"
  if [[ -z "${RERUN_ONLY:-}" ]]; then
    if diff -u "$MODEL/out/expected.txt" "$WORK/summary.txt"; then echo "name=sonnet_forward_rerun_matches_expected result=same"; else echo "name=sonnet_forward_rerun_matches_expected result=differs"; fi
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
