#!/bin/bash
# 多机跑法（2026-10-06，用户定案「分片 双机62核 给80g」「分片 断点重启 自适应多机跑」）：这一份由原跑法 run_segment_one_on_the_revision_6_tree.sh（869d2c0a…）
# 生成，第 1 行到第 350 行（前提、续跑入口、第 0–10 步）只改了这几行注释，第 11–13 步换成多机编排 rows_orchestrator.py（文件头写分配、派发、合并、跨机对照、续跑）。
# 起法：detached-run.sh start --wall-clock over-2h --label … -- env RUN_SUFFIX=-r6-r3 RUN_DATE=2026-10-05 FROM_STEP=11 RESUME_TAG=<新的> RESUME_LOG=… RESUME_MANIFEST=… bash <这一份>；同一条命令就是续跑命令。
# long-run-shards: 第 11–13 步按行切：每行一个单线道逐行驱动，行从按推的用时最长先做的队列里派到各台空出来的槽（rows_orchestrator.py 文件头「行与派发」）
# long-run-resume: 同一条命令再起就是续跑：第 0–10 步经 resume_check.py 核过才跳过；第 11–13 步只补行目录里还没有 row-result.tsv 的行，别的台被停时取回的行先收进来、写到一半的重做
# long-run-hosts: 台从 scripts/crash/layer0-shard-configuration-check.sh --emit-hosts 读，每台槽数与每槽线程从 scripts/hosts/host-resources.py --json 现查现算（rows_orchestrator.py 文件头「资源与分配」），不写死
# E171 第 1 次重跑第一段·修订 6 树：照登记修订 6 第 6.4 节从第 0 步起全部重跑，次序照修订 2 第 2.7 节（登记 research/prompts/e171-r1-prereg.md，修订 6、修订 7）。
# 主 agent 用脱离单元起，在仓根跑：bash scripts/hosts/detached-run.sh start --label e171-r1-s1-revision-6 -- bash /home/fy5090/.local/share/singlefs/work/e171-r1-segment1-rev6/run_segment_one_on_the_revision_6_tree.sh
# 形照第一段新树那一份按序脚本（原在 /tmp，整机重启后逐字节复原在 /home/fy5090/.local/share/singlefs/work/e171-r1-segment1-rev6/exact-reconstruction/run_segment_one_on_the_new_tree.sh）；改的：指修订 6 树与它编出的两个二进制、
# 产物名带 RUN_SUFFIX（默认 -r6）、导出目录另起、第 0 步把冻结树产物与 stream-constants、S3/S11 自检做成产物、第 6、10 步换成修订 5 第 5.6 节的判法，
# 第 1、6、7、10 步之后接 S15 ⑤、④、⑥⑤、④ 的比对（修订 6 第 6.6 节；比法 s15_compare.py）。一步红就停、后面的不跑，进度追加进 $LOGS/sequence.log。
# 环境变量：RUN_SUFFIX（默认 -r6；同一天再跑给 -r6-r2 这类）、FROM_STEP（从第几步起，默认 0；前面的步当作已跑完，它们的产物要在）。
# 续跑（2026-10-06 整机重启后重建时加）：FROM_STEP 大于 0 时必须给 RUN_DATE（上一趟的日期，产物名与导出目录照它）、RESUME_TAG（例 -resume1）、
# RESUME_LOG（上一趟的日志）、RESUME_MANIFEST（resume_check.py make 做的清单）；开跑先由 resume_check.py check 核「第 0 至 FROM_STEP-1 步
# 在日志里都记了绿、日志点名的产物都在且行数相同、清单里每个文件 sha256 未变」，不过就停。RESUME_TAG 只接进第 11–13 步的产物名、
# 逐行输出根与日志目录（上一趟停在第 11 步留下的半份产物与行目录原样留着，不覆盖）；第 1–10 步的名字不带它，从那几步续跑撞名照旧停。
set -u
REPO=/home/fy5090/code/singlefs
cd "$REPO" || exit 2
SCRATCH=/home/fy5090/.local/share/singlefs/work/e171-r1-segment1-rev6
DATE=${RUN_DATE:-$(TZ=Asia/Tokyo date +%F)}
SUFFIX=${RUN_SUFFIX:--r6}
FROM_STEP=${FROM_STEP:-0}
RESUME_TAG=${RESUME_TAG:-}
PREFIX=research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-$DATE-s1
DUMPS=scripts/experiments/target/e171-r1-dumps/revision-6-$DATE${SUFFIX#-r6}
LOGS=$SCRATCH/sequence-$DATE$SUFFIX$RESUME_TAG
WORK_ROOT=$SCRATCH/rows-work$RESUME_TAG
TREE=scripts/experiments/target/e171-r1-frozen-tree-revision-6
BIN=$TREE/target/release/e171_three_oracle_collusion_after_the_oracle_fixes
E170_BIN=$TREE/target/release/e170_o3_spec_executor_export
DEVICE_SOURCE=crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
RESEARCH=scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes
TIMING=$RESEARCH/device_timing.py
# 编排器、check_product.py（与编排器同目录的 peer_side.py、 由编排器按它自己的目录取）用仓里入库的那一份（与草稿里的逐字节相同）；
# s2_compare.py、s15_compare.py、frozen_tree_lines_revision_6.py、resume_check.py 没入库，照旧在草稿里
CHECK=$RESEARCH/check_product.py
S2_COMPARE=$SCRATCH/s2_compare.py
S15=$SCRATCH/s15_compare.py
FROZEN_TREE_LINES=$SCRATCH/frozen_tree_lines_revision_6.py
RESUME_CHECK=$SCRATCH/resume_check.py
E170_BAD_STATES=scripts/experiments/target/e170-r2-today-dumps/e170-bad-states-second-run
# S15 ④⑤⑥ 比的旧那几份（修订 6 第 6.6 节）
OLD_FIRST_SEGMENT_DUMPS=scripts/experiments/target/e171-r1-dumps/newtree-2026-10-05
OLD_Q5_RESULTS=research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5
OLD_Q5_DUMPS=scripts/experiments/target/e171-r1-dumps
# 修订 6 树（2026-10-05 拍）与它 release 编出的两个装置；仓里装置换进修订 7 那份之后应为 REPO_DEVICE_SHA256
SNAPSHOT_DIGEST=7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4
BIN_SHA256=6057769a543a629b0974ce759676df76cd9ad6e6a7e6c8941ec00641eb1d6036
E170_BIN_SHA256=e67b8b012fab36cfa5a9ccbfa3efd5bbe321fa9179c9a6a864a637e1939ed46d
REPO_DEVICE_SHA256=ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df
# 线程：派发给的上限 10，逐行两条线道各 5（修订 2 第 2.7 节）。内存：流三在里面的 6G，只有流一流二的 4G
THREADS=10
LANES=2
THREADS_PER_LANE=5
CAP_SMALL=4G
CAP_LARGE=6G
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一会话派出去的几个执行员各跑各的装置"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="E171 第一段修订 6 树：流三在里面的那几趟峰值表里没有这条命令，上限按登记推的给"
export PYTHONDONTWRITEBYTECODE=1
mkdir -p "$LOGS" "$DUMPS"
STEP=0

say() { echo "$(TZ=Asia/Tokyo date +%F) $*" | tee -a "$LOGS/sequence.log"; }
stop() { say "✗ 停在第 $STEP 步：$*"; exit 1; }
product_of() { echo "$PREFIX-$1$SUFFIX.out"; }
timing_of() { echo "$PREFIX-$1$SUFFIX-timing.out"; }
begin() { STEP=$1; shift; if [ "$STEP" -lt "$FROM_STEP" ]; then say "跳过第 $STEP 步（FROM_STEP=$FROM_STEP）：$*"; return 1; fi; say "▶ 第 $STEP 步：$*"; date +%s > "$LOGS/step-$STEP.started"; return 0; }
finish() { say "✓ 第 $STEP 步绿，挂钟 $(( $(date +%s) - $(cat "$LOGS/step-$STEP.started") )) 秒"; }
must() { "$@" >> "$LOGS/sequence.log" 2>&1 || stop "判法不过：$*"; }

snapshot_check() {  # 作废 Z5：每趟开头与收尾核冻结树的快照摘要
  local digest
  digest=$(python3 -B -c "import sys; sys.path.insert(0, '$RESEARCH'); import row_driver; print(row_driver.snapshot_digest('$TREE')[0])")
  [ "$digest" = "$SNAPSHOT_DIGEST" ] || stop "冻结树快照摘要变了（Z5）：$digest"
}
freeze_check() {  # 作废 Z2：O3 冻结行
  local differences
  differences=$(python3 -B -c "import sys; sys.path.insert(0, '$RESEARCH'); import frozen_o3; print(len(frozen_o3.freeze_differences('$REPO')))")
  [ "$differences" = "0" ] || stop "O3 冻结行对不上（Z2）：$differences 处"
}
# s15 <比法> <步名> <参数…>：S15 的一项比对，结果进新产物 <步名>；触发就停（两边都查，既不作废也不当结果，交主 agent）
s15() {
  local how=$1 name=$2; shift 2
  [ -e "$(product_of "$name")" ] && stop "产物已存在：$(product_of "$name")"
  python3 -B "$S15" "$how" "$(product_of "$name")" "$@" >> "$LOGS/sequence.log" 2>&1 || stop "S15 触发（$name，见 $(product_of "$name") 的 verdict 行）"
}

# device_step <步名> <内存上限> <装置的路径> <装置参数…>：装置一趟，结果行与计时行各落一份新产物；交回装置的退出码（250–254 是包装自己的结局，直接停）
device_step() {
  local name=$1 cap=$2 binary=$3; shift 3
  local product timing code
  product=$(product_of "$name"); timing=$(timing_of "$name")
  [ -e "$product" ] || [ -e "$timing" ] && stop "产物已存在：$product 或 $timing（同一天再跑给 RUN_SUFFIX）"
  snapshot_check
  nice -n 19 bash scripts/hosts/run-with-memory-cap.sh "$cap" bash scripts/hosts/capped.sh "$THREADS" \
    python3 -B "$TIMING" run -- "$binary" "$@" > "$LOGS/$name.stdout" 2> "$LOGS/$name.stderr"
  code=$?
  echo "$code" > "$LOGS/$name.exit"
  ( set -o noclobber; grep '^E7RESULT' "$LOGS/$name.stdout" > "$product"; grep '^E7TIMING' "$LOGS/$name.stdout" > "$timing" )
  snapshot_check
  say "  $name 退出码 $code，产物 $product（$(grep -c '' "$product") 行），计时 $(tail -n 1 "$timing")"
  if [ "$code" -ge 250 ] && [ "$code" -le 254 ]; then stop "$name 撞了内存包装自己的结局 $code（Z3：那一次不算，交主 agent 重跑）"; fi
  return "$code"
}

# driver_step <步名> <基线 -timing 产物> <驱动参数…>：逐行驱动一趟（stdout 进产物，--timing-output 进 -timing 产物）；交回驱动的退出码
driver_step() {
  local name=$1 baseline_timing=$2; shift 2
  local product timing code
  product=$(product_of "$name"); timing=$(timing_of "$name")
  [ -e "$product" ] || [ -e "$timing" ] && stop "产物已存在：$product 或 $timing"
  snapshot_check
  ( set -o noclobber; nice -n 19 python3 -B "$RESEARCH/row_driver.py" --repo-root "$REPO" --work-root "$WORK_ROOT" \
      --snapshot-from "$REPO/$TREE" --baseline-timing "$REPO/$baseline_timing" --timing-output "$REPO/$timing" "$@" \
      > "$product" 2> "$LOGS/$name.stderr" )
  code=$?
  echo "$code" > "$LOGS/$name.exit"
  snapshot_check
  say "  $name 驱动退出码 $code，$(grep '^name=row_time_limit' "$product")"
  return "$code"
}

# ───── 第 0 步：前提（每趟都核）与冻结树产物、stream-constants、S3/S11 的流一带探针自检（修订 6 第 6.4 节步表第 0 行） ─────
say "开跑：DATE=$DATE SUFFIX=$SUFFIX FROM_STEP=$FROM_STEP 导出 $DUMPS 日志 $LOGS"
snapshot_check
[ "$(sha256sum "$BIN" | cut -d' ' -f1)" = "$BIN_SHA256" ] || stop "新装置二进制不是修订 6 树编出的那一份：$BIN"
[ "$(sha256sum "$E170_BIN" | cut -d' ' -f1)" = "$E170_BIN_SHA256" ] || stop "E170 导出装置二进制不是修订 6 树编出的那一份：$E170_BIN"
[ "$(sha256sum "$DEVICE_SOURCE" | cut -d' ' -f1)" = "$REPO_DEVICE_SHA256" ] || stop "仓里装置不是修订 7 那一份（$REPO_DEVICE_SHA256）：常量还没换进主工作区，或仓里装置又改过（修订 6 第 6.3 节第 3 条、S15 ②）"
freeze_check
[ -d "$E170_BAD_STATES/new_pool_file_creation" ] || stop "O3 阳性对照要的 E170 bad-states 导出不在：$E170_BAD_STATES"
for old in "$OLD_FIRST_SEGMENT_DUMPS/s2-device-baseline-streams-1-2" "$OLD_FIRST_SEGMENT_DUMPS/baseline-run1" "$OLD_Q5_DUMPS/q5-revision-5-reverse" \
           "$OLD_Q5_DUMPS/q5-revision-5-main" "$OLD_Q5_DUMPS/q5-revision-5-main-run-2" "$OLD_Q5_RESULTS-reverse.out" "$OLD_Q5_RESULTS-main-run-1.out" "$OLD_Q5_RESULTS-main-run-2.out"; do
  [ -e "$old" ] || stop "S15 要比的旧那一份不在：$old"
done
say "✓ 前提：修订 6 树摘要、两个二进制与仓里装置的 sha256、O3 冻结行、E170 bad-states 导出、S15 要比的旧导出与旧产物都在"
if [ "$FROM_STEP" -gt 0 ]; then  # 续跑入口：前面的步只在 resume_check.py check 放行时才当作已绿
  [ -n "${RUN_DATE:-}" ] && [ -n "$RESUME_TAG" ] && [ -n "${RESUME_LOG:-}" ] && [ -n "${RESUME_MANIFEST:-}" ] \
    || stop "FROM_STEP=$FROM_STEP 续跑要给 RUN_DATE、RESUME_TAG、RESUME_LOG、RESUME_MANIFEST"
  python3 -B "$RESUME_CHECK" check --repo "$REPO" --log "$RESUME_LOG" --prefix "$PREFIX" --suffix="$SUFFIX" --dumps "$DUMPS" \
    --before-step "$FROM_STEP" --manifest "$RESUME_MANIFEST" >> "$LOGS/sequence.log" 2>&1 \
    || stop "续跑核对不过（见 $LOGS/sequence.log 里 resume_check 那几行）：第 0 至 $((FROM_STEP - 1)) 步不能当作已绿"
  say "✓ 续跑核对：第 0 至 $((FROM_STEP - 1)) 步在 $RESUME_LOG 里都记了绿，产物行数与日志同，清单 $RESUME_MANIFEST 里的 sha256 都没变"
fi
if begin 0 "冻结树产物（S15 ①②③）、stream-constants、S3/S11 流一带探针自检"; then
  [ -e "$(product_of frozen-tree)" ] && stop "产物已存在：$(product_of frozen-tree)"
  python3 -B "$FROZEN_TREE_LINES" "$(product_of frozen-tree)" >> "$LOGS/sequence.log" 2>&1 || stop "S15 ①②③ 或 A12 触发（见 $(product_of frozen-tree) 的 verdict 行）"
  device_step stream-constants "$CAP_SMALL" "$BIN" stream-constants --streams 1,2,3
  [ $? -eq 0 ] || stop "stream-constants 退出码不是 0（S1 有不过的，看 name=s1 行）"
  must python3 -B "$CHECK" verdict "$(product_of stream-constants)" s1_failed_checks=0
  for stream in new_pool_file_creation fixed_script_through_e_then_unmount first_file_unlinked_then_inodes_across_one_leaf; do
    must python3 -B "$CHECK" line "$(product_of stream-constants)" s1 check="${stream}_allocator_carries_the_root_ring_of_make_filesystem" holds=true
  done
  must python3 -B "$CHECK" line "$(product_of stream-constants)" s1 check=third_stream_split_publish local='Some(234)' holds=true
  # stream-constants 只算常量、不跑流，计时产物里没有逐流时间，不核 device_timing（登记末尾主 agent 认定 2026-10-05 第 5 条）
  device_step s3-s11-baseline-stream-1-with-probe "$CAP_SMALL" "$BIN" baseline --streams 1 --writable-mount-probe --output-directory "$DUMPS/s3-s11-baseline-stream-1-with-probe"
  [ $? -eq 0 ] || stop "S3/S11 自检那一趟退出码不是 0"
  must python3 -B "$CHECK" verdict "$(product_of s3-s11-baseline-stream-1-with-probe)" s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 \
    s3_o2_judgement_mismatches=0 s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o1_reference_filesystem_red_states=0 \
    o2_red_states=0 probed_states='>0' witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0
  must python3 -B "$TIMING" check "$(timing_of s3-s11-baseline-stream-1-with-probe)"
  finish
fi

# ───── 第 1 步：S2 新装置基线（流一、流二、不带探针；它的进程挂钟是第二段的 T，也是这一段空变异对照的 T）；之后 S15 ⑤ ─────
S2_BASELINE=s2-device-baseline-streams-1-2
if begin 1 "S2 新装置基线 --streams 1,2，S15 ⑤"; then
  device_step "$S2_BASELINE" "$CAP_SMALL" "$BIN" baseline --streams 1,2 --output-directory "$DUMPS/$S2_BASELINE"
  [ $? -eq 0 ] || stop "装置退出码不是 0"
  must python3 -B "$CHECK" verdict "$(product_of $S2_BASELINE)" s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 \
    s3_o2_judgement_mismatches=0 s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 \
    o1_reference_filesystem_red_states=0 o2_red_states=0 recovery_panic_states=0
  must python3 -B "$TIMING" check "$(timing_of $S2_BASELINE)"
  s15 five s15-5-step-1 "$DUMPS/$S2_BASELINE" "$OLD_FIRST_SEGMENT_DUMPS/s2-device-baseline-streams-1-2"
  finish
fi

# ───── 第 2 步：S2 的另一半——同一棵冻结树上 E170 导出装置 dump，与第 1 步逐文件比 ─────
if begin 2 "S2：E170 导出装置 dump 与新装置基线逐文件比"; then
  device_step s2-e170-export-dump "$CAP_SMALL" "$E170_BIN" dump --output-directory "$DUMPS/s2-e170-export"
  [ $? -eq 0 ] || stop "E170 导出装置退出码不是 0（3 是 S6：跑不起来，交主 agent）"
  [ -e "$(product_of s2)" ] && stop "产物已存在：$(product_of s2)"
  must python3 -B "$S2_COMPARE" "$DUMPS/s2-e170-export" "$DUMPS/$S2_BASELINE" "$(product_of s2)"
  must python3 -B "$CHECK" verdict "$(product_of s2)" s2_differing_files=0 s2_compared_files=18
  finish
fi
# ───── 第 3 步：阳性对照全表，基线那一趟（登记 5.2、第七节 A8；流三在里面的两样排最后） ─────
if begin 3 "阳性对照（基线那一趟）"; then
  control() {  # control <步名> <内存上限> <装置参数…>：期望退 0
    device_step "$@"; [ $? -eq 0 ] || stop "$1 装置退出码不是 0"
  }
  control positive-control-o1-comparison "$CAP_SMALL" "$BIN" baseline --streams 2 --positive-control O1-对拍 --output-directory "$DUMPS/positive-control-o1-comparison"
  must python3 -B "$CHECK" verdict "$(product_of positive-control-o1-comparison)" o1_comparison_red_calls=1 s3_o2_judgement_mismatches=0
  control positive-control-o1-comparison-unmount "$CAP_SMALL" "$BIN" baseline --streams 2 --positive-control O1-对拍-卸载 --output-directory "$DUMPS/positive-control-o1-comparison-unmount"
  must python3 -B "$CHECK" verdict "$(product_of positive-control-o1-comparison-unmount)" o1_comparison_red_calls=1 s3_o2_judgement_mismatches=0
  must python3 -B "$CHECK" line "$(product_of positive-control-o1-comparison-unmount)" call stream=fixed_script_through_e_then_unmount label=unmount o1_comparison=prefix:disagrees
  control positive-control-o2 "$CAP_SMALL" "$BIN" baseline --streams 1 --positive-control O2 --output-directory "$DUMPS/positive-control-o2"
  must python3 -B "$CHECK" verdict "$(product_of positive-control-o2)" o2_red_states=1 s3_o2_judgement_mismatches=0
  control positive-control-return-value "$CAP_SMALL" "$BIN" baseline --streams 2 --positive-control 交回值 --output-directory "$DUMPS/positive-control-return-value"
  must python3 -B "$CHECK" calls_differ "$(product_of $S2_BASELINE)" "$(product_of positive-control-return-value)" return_value_sha256 fixed_script_through_e_then_unmount 2
  must python3 -B "$CHECK" call_o1 "$(product_of positive-control-return-value)" fixed_script_through_e_then_unmount 2 agrees
  control positive-control-memory-object "$CAP_SMALL" "$BIN" baseline --streams 2 --positive-control 内存对象 --output-directory "$DUMPS/positive-control-memory-object"
  must python3 -B "$CHECK" calls_differ "$(product_of $S2_BASELINE)" "$(product_of positive-control-memory-object)" memory_objects_sha256 fixed_script_through_e_then_unmount 2
  # panic：基线模式下流二截在第 3 次调用，S1 的段长、写数那几条对不上本地常量，装置退 3（第一段报告第 8 节：登记预期内）
  device_step positive-control-panic "$CAP_SMALL" "$BIN" baseline --streams 2 --positive-control panic --output-directory "$DUMPS/positive-control-panic"
  [ $? -eq 3 ] || stop "panic 对照预期退 3（截断的流对不上 S1 本地常量）"
  must python3 -B "$CHECK" line "$(product_of positive-control-panic)" stream ending=prefix:stopped_by_a_panic:call_3 exported=true \
    comparison_judged_calls_before_the_first_panic=1 sampled_states='>0'
  must python3 -B "$CHECK" verdict "$(product_of positive-control-panic)" s3_o2_judgement_mismatches=0 streams_not_exported=0
  control positive-control-witness "$CAP_SMALL" "$BIN" baseline --streams 1 --writable-mount-probe --positive-control 见证 --output-directory "$DUMPS/positive-control-witness"
  must python3 -B "$CHECK" verdict "$(product_of positive-control-witness)" counter_branch_states=1 counter_branch_states_judged_legal=0 \
    witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0
  # O3：流一全持久态叠 P1 补丁交冻结的执行器，要判「不在」（不是装置，没有 -timing）
  freeze_check
  [ -e "$(product_of positive-control-o3-p1)" ] && stop "产物已存在：$(product_of positive-control-o3-p1)"
  nice -n 19 bash scripts/hosts/run-with-memory-cap.sh "$CAP_SMALL" bash scripts/hosts/capped.sh "$THREADS" python3 -B "$RESEARCH/o3_bad_image_judge.py" control \
    --repo-root "$REPO" --dumps "$DUMPS/$S2_BASELINE" --e170-dumps "$E170_BAD_STATES" --row-name P1-F --output "$(product_of positive-control-o3-p1)" \
    > "$LOGS/positive-control-o3-p1.log" 2>&1 || stop "O3 阳性对照没跑完（日志 $LOGS/positive-control-o3-p1.log）"
  freeze_check
  must python3 -B "$CHECK" line "$(product_of positive-control-o3-p1)" o3_control_verdict strict=not_in base_digest_matches=true red_strict=true
  # 流三在里面的两样（每条流批 1 第一个读回了文件的态各一，恰 3 态）
  control positive-control-o1-crash-judge "$CAP_LARGE" "$BIN" baseline --streams 1,2,3 --positive-control O1-崩溃判官 --output-directory "$DUMPS/positive-control-o1-crash-judge"
  must python3 -B "$CHECK" verdict "$(product_of positive-control-o1-crash-judge)" o1_crash_judge_red_states=3 o1_reference_filesystem_red_states=0 s3_o2_judgement_mismatches=0
  control positive-control-reference-filesystem "$CAP_LARGE" "$BIN" baseline --streams 1,2,3 --positive-control RefFS --output-directory "$DUMPS/positive-control-reference-filesystem"
  must python3 -B "$CHECK" verdict "$(product_of positive-control-reference-filesystem)" o1_reference_filesystem_red_states=3 o1_crash_judge_red_states=0 s3_o2_judgement_mismatches=0
  finish
fi

# ───── 第 4 步：阳性对照，空变异那一趟（逐行驱动、流一流二，A8 第二半：崩溃判官与 RefFS 各 2 态，其余同基线那一趟） ─────
if begin 4 "阳性对照（空变异那一趟，经逐行驱动）"; then
  empty_control() {  # empty_control <名> <对照> <驱动另加的参数…>
    local slug=$1 control_name=$2; shift 2
    driver_step "empty-control-$slug-driver" "$(timing_of $S2_BASELINE)" --empty-mutation --keep-dumps --positive-control "$control_name" \
      --lanes 1 --threads "$THREADS" --memory-cap "$CAP_SMALL" --output-root "$REPO/$DUMPS/empty-control-$slug" "$@"
    [ $? -eq 0 ] || stop "empty-control-$slug 驱动退出码不是 0"
    must python3 -B "$CHECK" rows "$(product_of "empty-control-$slug-driver")"
    EMPTY_DEVICE_OUT="$DUMPS/empty-control-$slug/row-0/device.out"
  }
  empty_control o1-comparison O1-对拍 --streams 1,2
  must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" o1_comparison_red_calls=1 s3_o2_judgement_mismatches=0
  empty_control o1-comparison-unmount O1-对拍-卸载 --streams 1,2
  must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" o1_comparison_red_calls=1 s3_o2_judgement_mismatches=0
  empty_control o1-crash-judge O1-崩溃判官 --streams 1,2
  must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" o1_crash_judge_red_states=2 o1_reference_filesystem_red_states=0
  empty_control reference-filesystem RefFS --streams 1,2
  must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" o1_reference_filesystem_red_states=2 o1_crash_judge_red_states=0
  empty_control o2 O2 --streams 1,2
  must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" o2_red_states=1
  empty_control return-value 交回值 --streams 1,2
  must python3 -B "$CHECK" calls_differ "$(product_of $S2_BASELINE)" "$EMPTY_DEVICE_OUT" return_value_sha256 fixed_script_through_e_then_unmount 2
  must python3 -B "$CHECK" call_o1 "$EMPTY_DEVICE_OUT" fixed_script_through_e_then_unmount 2 agrees
  empty_control memory-object 内存对象 --streams 1,2
  must python3 -B "$CHECK" calls_differ "$(product_of $S2_BASELINE)" "$EMPTY_DEVICE_OUT" memory_objects_sha256 fixed_script_through_e_then_unmount 2
  empty_control panic panic --streams 1,2
  must python3 -B "$CHECK" line "$EMPTY_DEVICE_OUT" stream stream=fixed_script_through_e_then_unmount ending=prefix:stopped_by_a_panic:call_3 \
    exported=true comparison_judged_calls_before_the_first_panic=1 sampled_states='>0'
  empty_control witness 见证 --streams 1 --writable-mount-probe
  must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" counter_branch_states=1 counter_branch_states_judged_legal=0 \
    witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0
  empty_control o3-p1 O3 --streams 1,2 --o3-control-bad-states "$REPO/$E170_BAD_STATES"
  must python3 -B "$CHECK" line "$DUMPS/empty-control-o3-p1/row-0/o3-control.out" o3_control_verdict strict=not_in base_digest_matches=true red_strict=true
  finish
fi

# ───── 第 5 步：O3 基线（冻结的 v2-487 执行器判第 1 步导出的流一、流二批 1 批 2；A1 的 O3 两列） ─────
if begin 5 "O3 基线（流一、流二）"; then
  freeze_check
  [ -e "$(product_of o3-baseline-streams-1-2)" ] && stop "产物已存在：$(product_of o3-baseline-streams-1-2)"
  python3 -B -c "import sys; sys.path.insert(0, '$RESEARCH'); import frozen_o3; code, wall = frozen_o3.run_executor('$REPO', '$REPO/$DUMPS/$S2_BASELINE', ['new_pool_file_creation', 'fixed_script_through_e_then_unmount'], $THREADS, '$REPO/$(product_of o3-baseline-streams-1-2)', '$CAP_SMALL', '$LOGS/o3-baseline-streams-1-2.log'); print('executor_exit', code, 'wall_ns', wall); sys.exit(code)" \
    >> "$LOGS/sequence.log" 2>&1 || stop "执行器没退 0（日志 $LOGS/o3-baseline-streams-1-2.log）"
  freeze_check
  must python3 -B "$CHECK" o3 "$(product_of o3-baseline-streams-1-2)" new_pool_file_creation,fixed_script_through_e_then_unmount
  finish
fi


# ───── 第 6 步：Q5 反向取样点（修订 5 第 5.6 节第 3 条 a：截到第 2 次调用的返回点，末段全枚举 72 态）；之后 S15 ④ ─────
Q5_REVERSE=q5-reverse-at-the-return-point-of-call-2
if begin 6 "Q5 反向取样点，S15 ④"; then
  device_step "$Q5_REVERSE" "$CAP_SMALL" "$BIN" confirmed-loss-form --truncate-at-the-return-point --output-directory "$DUMPS/q5-reverse"
  [ $? -eq 0 ] || stop "装置退出码不是 0（3：S1、S3、S13 或 q5_not_run，看 verdict 行）"
  must python3 -B "$CHECK" verdict "$(product_of $Q5_REVERSE)" s1_failed_checks=0 s3_o2_judgement_mismatches=0 s4_digest_mismatches=0 s13=false unattributed_panic_states=0
  must python3 -B "$CHECK" line "$(product_of $Q5_REVERSE)" q5_shape confirmed_call_returned_before_the_truncation=true in_flight_call_returned=false
  must python3 -B "$CHECK" line "$(product_of $Q5_REVERSE)" q5 states=72 a10_states_equal_the_product=true
  s15 four s15-4-step-6 "$(product_of $Q5_REVERSE)" "$OLD_Q5_RESULTS-reverse.out" "$DUMPS/q5-reverse" "$OLD_Q5_DUMPS/q5-revision-5-reverse"
  finish
fi

# ───── 第 7–9 步：带流三、带探针的基线两次（第一次的进程挂钟是这一段逐行的 T）与它上面的 O3 ─────
baseline_checks() {  # 阴性（A1 的 O1 三列与 O2）、S1（含修订 6 那条根环表）、S3、S5（基线模式退 0 已含）、S9、S11
  must python3 -B "$CHECK" verdict "$(product_of "$1")" s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 \
    s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o1_reference_filesystem_red_states=0 o2_red_states=0 \
    probed_states='>0' witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0
  must python3 -B "$CHECK" line "$(product_of "$1")" stream stream=first_file_unlinked_then_inodes_across_one_leaf ending=completed exported=true model_stopped_at_call=none
  must python3 -B "$TIMING" check "$(timing_of "$1")"
}
if begin 7 "带流三、带探针的基线第一次，S15 ⑥ 与流一流二的 S15 ⑤"; then
  device_step baseline-run1 "$CAP_LARGE" "$BIN" baseline --streams 1,2,3 --writable-mount-probe --output-directory "$DUMPS/baseline-run1"
  BASELINE_RUN1_EXIT=$?
  # S15 先比（导出在就比），再按退出码与 A1 判：装置退 3 或 A1 不符时，S15 ⑥ 的读数照样留在产物里，交主 agent 两边查
  if [ -d "$DUMPS/baseline-run1/first_file_unlinked_then_inodes_across_one_leaf" ]; then
    s15 six s15-6-step-7 "$DUMPS/baseline-run1/first_file_unlinked_then_inodes_across_one_leaf" "$OLD_FIRST_SEGMENT_DUMPS/baseline-run1/first_file_unlinked_then_inodes_across_one_leaf"
    s15 five s15-5-step-7 "$DUMPS/baseline-run1" "$OLD_FIRST_SEGMENT_DUMPS/baseline-run1"
  fi
  [ "$BASELINE_RUN1_EXIT" -eq 0 ] || stop "装置退出码 $BASELINE_RUN1_EXIT 不是 0（3：停机条款，看 verdict 行）"
  [ -e "$(product_of s15-6-step-7)" ] || stop "流三导出不在，S15 ⑥ 没比"
  baseline_checks baseline-run1
  finish
fi
if begin 8 "O3 判基线第一次的三条流"; then
  freeze_check
  [ -e "$(product_of o3-baseline-run1)" ] && stop "产物已存在：$(product_of o3-baseline-run1)"
  python3 -B -c "import sys; sys.path.insert(0, '$RESEARCH'); import frozen_o3; code, wall = frozen_o3.run_executor('$REPO', '$REPO/$DUMPS/baseline-run1', ['new_pool_file_creation', 'fixed_script_through_e_then_unmount', 'first_file_unlinked_then_inodes_across_one_leaf'], $THREADS, '$REPO/$(product_of o3-baseline-run1)', '$CAP_LARGE', '$LOGS/o3-baseline-run1.log'); print('executor_exit', code, 'wall_ns', wall); sys.exit(code)" \
    >> "$LOGS/sequence.log" 2>&1 || stop "执行器没退 0（日志 $LOGS/o3-baseline-run1.log）"
  freeze_check
  python3 -B "$CHECK" o3 "$(product_of o3-baseline-run1)" new_pool_file_creation,fixed_script_through_e_then_unmount >> "$LOGS/sequence.log" 2>&1 \
    || stop "O3 在基线上有「不在」、出错、摘要对不上，或流一流二没态（F2 或执行器缺陷）"
  python3 -B "$CHECK" o3 "$(product_of o3-baseline-run1)" first_file_unlinked_then_inodes_across_one_leaf >> "$LOGS/sequence.log" 2>&1 \
    || say "  注意：O3 在流三上一态都没有（F7，比对器记「到不了」；不停）"
  finish
fi
if begin 9 "带流三、带探针的基线第二次（作废 Z1 快查）"; then
  device_step baseline-run2 "$CAP_LARGE" "$BIN" baseline --streams 1,2,3 --writable-mount-probe --output-directory "$DUMPS/baseline-run2"
  [ $? -eq 0 ] || stop "装置退出码不是 0"
  baseline_checks baseline-run2
  must python3 -B "$CHECK" z1 "$DUMPS/baseline-run1" "$DUMPS/baseline-run2"
  finish
fi

# ───── 第 10 步：Q5 主取样点（修订 5 第 5.6 节第 3 条 b、c：同一棵树连跑两次，Z7；判法照第 7 条主取样点那几行）；两次各接 S15 ④ ─────
Q5_MAIN=q5-main-before-the-first-barrier-of-call-3
q5_main_checks() {
  local product
  product=$(product_of "$1")
  must python3 -B "$CHECK" verdict "$product" s1_failed_checks=0 s3_o2_judgement_mismatches=0 s4_digest_mismatches=0 s13=false unattributed_panic_states=0
  must python3 -B "$CHECK" line "$product" q5_equivalence_pairing mismatching_states=0 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 \
    read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
  must python3 -B "$CHECK" line "$product" q5_piece_panic_control holds=true mismatching_states=0 s13=false   # F9 不触发
  must python3 -B "$CHECK" line "$product" q5_equivalence input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 \
    recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
  must python3 -B "$CHECK" line "$product" q5 states=1207959552 a10_states_equal_the_product=true
}
if begin 10 "Q5 主取样点两次（Z7），S15 ④"; then
  device_step "$Q5_MAIN-run-1" "$CAP_SMALL" "$BIN" confirmed-loss-form --output-directory "$DUMPS/q5-main"
  [ $? -eq 0 ] || stop "第 1 次退出码不是 0（3：S13、S1、S3 或 q5_not_run，看 verdict 行）"
  q5_main_checks "$Q5_MAIN-run-1"
  s15 four s15-4-step-10-run-1 "$(product_of "$Q5_MAIN-run-1")" "$OLD_Q5_RESULTS-main-run-1.out" "$DUMPS/q5-main" "$OLD_Q5_DUMPS/q5-revision-5-main"
  device_step "$Q5_MAIN-run-2" "$CAP_SMALL" "$BIN" confirmed-loss-form --output-directory "$DUMPS/q5-main-run-2"
  [ $? -eq 0 ] || stop "第 2 次退出码不是 0"
  q5_main_checks "$Q5_MAIN-run-2"
  # 作废 Z7：两份结果产物逐字节同（结果产物里只有 E7RESULT 行，耗时在 -timing 那一份）
  cmp "$(product_of "$Q5_MAIN-run-1")" "$(product_of "$Q5_MAIN-run-2")" >> "$LOGS/sequence.log" 2>&1 || stop "Z7：主取样点两次的结果行不逐字节相同"
  say "  Z7：两次逐字节相同"
  s15 four s15-4-step-10-run-2 "$(product_of "$Q5_MAIN-run-2")" "$OLD_Q5_RESULTS-main-run-2.out" "$DUMPS/q5-main-run-2" "$OLD_Q5_DUMPS/q5-revision-5-main-run-2"
  finish
fi

# ───── 第 11–13 步：多机编排（rows_orchestrator.py；现查各台资源、逐行分到各台、取回、按写死的规则合成原名产物、check_product.py rows、跨机一致对照、第 13 步） ─────
# 判法、阈值、限时规则与每行驱动的参数照上面原跑法第 11–13 步（改的只有 --lanes、--threads、--rows、输出根与工作根，另加驱动自带的 --skip-finished）。
# 被停之后同一条命令（FROM_STEP=11、同一个 RESUME_TAG）再起：第 0–10 步照旧经续跑核对跳过，第 11–13 步由编排器只补没做完的行。
ORCHESTRATOR=$RESEARCH/rows_orchestrator.py
[ "$FROM_STEP" -le 11 ] || stop "多机跑法的第 11–13 步只从第 11 步起（FROM_STEP=$FROM_STEP）：续跑由编排器只补没做完的行，同一条命令（FROM_STEP=11）再起即可"
[ -n "$RESUME_TAG" ] || stop "多机跑法要给 RESUME_TAG（行输出根、产物名与分片目录接它，不与上一趟撞名）"
STEP=11
say "▶ 第 11–13 步：多机编排 $ORCHESTRATOR（行输出根 $DUMPS/segment1-rows$RESUME_TAG，分片 $DUMPS/segment1-rows$RESUME_TAG-shards）"
nice -n 19 python3 -B "$ORCHESTRATOR" run --repo "$REPO" --prefix "$PREFIX" --suffix="$SUFFIX" --resume-tag="$RESUME_TAG" --dumps "$DUMPS" \
  --tree "$TREE" --snapshot-digest "$SNAPSHOT_DIGEST" --baseline-timing "$(timing_of baseline-run1)" --check "$CHECK" --logs "$LOGS" \
  --work-root "$WORK_ROOT" &
ORCHESTRATOR_PID=$!
trap 'say "收到停止信号：等编排器收尾（停各台在跑的行、取回别的台已做完的行）"' TERM INT HUP
while :; do  # 信号打断 wait 时编排器还在：接着等它收完尾，拿它真的退出码
  wait "$ORCHESTRATOR_PID"; ORCHESTRATOR_EXIT=$?
  kill -0 "$ORCHESTRATOR_PID" 2>/dev/null || break
done
trap - TERM INT HUP
case "$ORCHESTRATOR_EXIT" in
  0) ;;
  78) say "✗ 停在第 11–13 步：运行条件不满足（见上面 ✗ 那一行；修好那台或改多机配置，再用同一条命令起）"; exit 78 ;;
  143) say "✗ 停在第 11–13 步：被停（同一条命令再起就是续跑）"; exit 143 ;;
  *) stop "多机编排退 $ORCHESTRATOR_EXIT（见上面 ✗ 那一行）" ;;
esac
STEP=done
say "✓ 全部步骤跑完。比对器（segment_one_comparator.py）归下一段执行员；产物前缀 $PREFIX，行输出根 $DUMPS/segment1-rows$RESUME_TAG，日志 $LOGS"
exit 0
