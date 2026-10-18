#!/bin/bash
# E171 第 1 次重跑·修订 11：修订 8 那一趟第 0–10 步的一个单元，带复用（登记 research/prompts/e171-r1-prereg.md 修订 11 第 11.5、11.10 节）。
# 由 segment_one_units_revision_11.py 起：bash <这一份> <单元名>，环境变量由它给；退 0 是这个单元绿，别的是红。
# 照抄修订 8 单元脚本 segment_one_unit_revision_8.sh（那一份一个字节不改）；判法、阈值、线程、内存那几行一个字不动。改的只有：
#   device_step、driver_step、run_executor 三处，与第 3 步 O3 阳性对照（o3_bad_image_judge.py control）、研究一侧变异表两处读数进程：编排给了这一步的条目
#   （REUSE_ENTRIES 里「步名<TAB>条目」）就取用（reuse_library.py fetch：核 RC5、按角色硬链接到这一趟的名字），不起那个进程；判法照跑；
#   驱动行取用之后照旧起驱动、另带 --skip-finished，由驱动从取来的行目录重报产物与计时（不编、不跑装置与执行器；冻结行前后照核）；
#   每一步的读数（现跑的、取来的）记一行进 REUSE_READINGS（步名、fresh|reused、退出码、角色=路径…；现跑的驱动行另记 compiled=<编译用的线道目录> 给 RC4），
#   由编排在单元结束时照入库条件入库；
#   第 2b 步取用时不拷第 2 步导出（取来的那份导出目录已是拷贝加写入之后的整份）；
#   新加单元 reuse-selfproof（RP2：H7 那个装置二进制跑三次 stream-constants，reuse_library.py selfproof）。
# 环境：修订 8 那几个（REPO RESEARCH PREFIX SUFFIX TAG DUMPS TREE BIN E170_BIN SNAPSHOT_DIGEST MUTATIONS_SHA256 CONFIG THREADS CAP_SMALL CAP_LARGE WORK_ROOT
#   UNIT_LOG UNIT_PRODUCTS UNIT_DIRECTORIES，依赖的名字，Q5_REVERSE_STATES Q5_MAIN_STATES，PRINT_ONLY）加 REUSE_LIBRARY（H30）、REUSE_ENTRIES、REUSE_READINGS。
#   PRINT_ONLY=1：只打出这个单元会跑的每一条命令（取用的那几步打 fetch 那一条），一条都不跑。
set -u
UNIT=${1:?要单元名}
cd "$REPO" || exit 2
# 编排给的 RESEARCH 是这一份所在的研究目录（相对 REPO 的 RESEARCH_RELATIVE，自证里是它的绝对路径）；缺省写成那个字面路径
RESEARCH="${RESEARCH:-scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes}"
TIMING=$RESEARCH/device_timing.py
CHECK=$RESEARCH/check_product.py
JUDGE=$RESEARCH/frozen_tree_lines_revision_8.py
E170_SEED=85441ae491419411
PRINT_ONLY=${PRINT_ONLY:-0}
EXPECTED_EXIT=0
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一会话派出去的几个执行员各跑各的装置"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="E171 第一段修订 8 树：流三在里面的那几趟峰值表里没有这条命令，上限按登记推的给"
export PYTHONDONTWRITEBYTECODE=1

say() { echo "$(TZ=Asia/Tokyo date +%F) [$UNIT] $*" | tee -a "$UNIT_LOG"; }
stop() { say "✗ 单元 $UNIT 红：$*"; exit 1; }
own() { echo "$PREFIX-$1$TAG$SUFFIX.out"; }            # 这个单元自己写出的产物
own_timing() { echo "$PREFIX-$1$TAG$SUFFIX-timing.out"; }
dep() { echo "$PREFIX-$1$SUFFIX.out"; }                # 依赖的单元的产物（名字已带它的 TAG）
dep_timing() { echo "$PREFIX-$1$SUFFIX-timing.out"; }
show() { printf '  $'; printf ' %q' "$@"; printf '\n'; }
x() { if [ "$PRINT_ONLY" = 1 ]; then show "$@"; return 0; fi; "$@"; }
must() { if [ "$PRINT_ONLY" = 1 ]; then show "$@"; return 0; fi; "$@" >> "$UNIT_LOG" 2>&1 || stop "判法不过：$*"; }
logged() { if [ "$PRINT_ONLY" = 1 ]; then show "$@"; return 0; fi; "$@" >> "$UNIT_LOG" 2>&1; }  # 跑一条、输出进单元日志，交回它的退出码
note_product() { [ "$PRINT_ONLY" = 1 ] || printf '%s\t%s\n' "$1" "$(grep -c '' "$1")" >> "$UNIT_PRODUCTS"; }
note_directory() { [ "$PRINT_ONLY" = 1 ] || printf '%s\n' "$1" >> "$UNIT_DIRECTORIES"; }
refuse_existing() { if [ "$PRINT_ONLY" != 1 ] && [ -e "$1" ]; then stop "已存在、不覆盖：$1"; fi; }
reuse_entry_for() {  # reuse_entry_for <步名>：编排给了这一步的条目就交回它（REUSE_ENTRIES：步名<TAB>条目），没给交回空
  [ -n "${REUSE_ENTRIES:-}" ] && [ -f "$REUSE_ENTRIES" ] || return 0
  awk -F'\t' -v name="$1" '$1 == name {print $2}' "$REUSE_ENTRIES"
}
note_reading() {  # note_reading <步名> <fresh|reused> <退出码> <角色>=<路径>…：这一步的读数记进 REUSE_READINGS，单元结束时由编排照入库条件入库
  [ "$PRINT_ONLY" = 1 ] && return 0
  [ -n "${REUSE_READINGS:-}" ] || return 0
  local IFS=$'\t'
  printf '%s\n' "$*" >> "$REUSE_READINGS"
}
fetch_into() {  # fetch_into <条目> <角色>=<路径>…：取用；条目里记的退出码放进 FETCHED_EXIT（PRINT_ONLY 时只打这一条命令、FETCHED_EXIT=0）
  local entry=$1 map; shift
  local words=(python3 -B "$RESEARCH/reuse_library.py" fetch --library "$REUSE_LIBRARY" --entry "$entry")
  for map in "$@"; do words+=(--map "$map"); done
  FETCHED_EXIT=0
  if [ "$PRINT_ONLY" = 1 ]; then show "${words[@]}"; return 0; fi
  FETCHED_EXIT=$("${words[@]}" 2>> "$UNIT_LOG")
}
option_value() {  # option_value <选项> <参数…>：参数里那个选项后面的值，没有交回空
  local option=$1; shift
  while [ $# -gt 0 ]; do [ "$1" = "$option" ] && { echo "${2:-}"; return 0; }; shift; done
}

snapshot_check() {  # 作废 Z5：冻结树快照摘要没变；修订 8 第 8.6 节另核树里 crates/mutations.tsv 等于 H8
  [ "$PRINT_ONLY" = 1 ] && return 0
  local digest mutations
  digest=$(python3 -B -c "import sys; sys.path.insert(0, '$RESEARCH'); import row_driver; print(row_driver.snapshot_digest('$TREE')[0])")
  [ "$digest" = "$SNAPSHOT_DIGEST" ] || stop "冻结树快照摘要变了（Z5）：$digest"
  mutations=$(sha256sum "$TREE/crates/mutations.tsv" | cut -d' ' -f1)
  [ "$mutations" = "$MUTATIONS_SHA256" ] || stop "树里 crates/mutations.tsv 变了（Z5，H8）：$mutations"
}
freeze_check() {  # 作废 Z2：O3 冻结行
  [ "$PRINT_ONLY" = 1 ] && return 0
  local differences
  differences=$(python3 -B -c "import sys; sys.path.insert(0, '$RESEARCH'); import frozen_o3; print(len(frozen_o3.freeze_differences('$REPO')))")
  [ "$differences" = "0" ] || stop "O3 冻结行对不上（Z2）：$differences 处"
}

# device_step <步名> <内存上限> <装置的路径> <装置参数…>：同修订 6 跑法；交回装置的退出码（250–254 是包装自己的结局，直接红）。PRINT_ONLY 时交回 EXPECTED_EXIT
device_step() {
  local name=$1 cap=$2 binary=$3; shift 3
  local product timing code entry directory
  product=$(own "$name"); timing=$(own_timing "$name")
  refuse_existing "$product"; refuse_existing "$timing"
  entry=$(reuse_entry_for "$name")
  directory=$(option_value --output-directory "$@"); [ -n "$directory" ] || directory=$(option_value --dump-directory "$@")
  if [ "$PRINT_ONLY" = 1 ]; then
    if [ -n "$entry" ]; then fetch_into "$entry" "product=$product" "timing=$timing" ${directory:+"directory=$directory"}
    else show nice -n 19 bash scripts/hosts/run-with-memory-cap.sh "$cap" bash scripts/hosts/capped.sh "$THREADS" python3 -B "$TIMING" run -- "$binary" "$@"; fi
    echo "    → 产物 $product，计时 $timing"; return "$EXPECTED_EXIT"
  fi
  snapshot_check
  if [ -n "$entry" ]; then
    fetch_into "$entry" "product=$product" "timing=$timing" ${directory:+"directory=$directory"} || stop "$name 取用不成（条目坏了已挪进隔离目录，或目标已在；见 $UNIT_LOG）"
    code=$FETCHED_EXIT
    note_reading "$name" reused "$code" "product=$product" "timing=$timing" ${directory:+"directory=$directory"}
  else
    nice -n 19 bash scripts/hosts/run-with-memory-cap.sh "$cap" bash scripts/hosts/capped.sh "$THREADS" \
      python3 -B "$TIMING" run -- "$binary" "$@" > "$UNIT_LOG.$name.stdout" 2> "$UNIT_LOG.$name.stderr"
    code=$?
    ( set -o noclobber; grep '^E7RESULT' "$UNIT_LOG.$name.stdout" > "$product"; grep '^E7TIMING' "$UNIT_LOG.$name.stdout" > "$timing" )
    note_reading "$name" fresh "$code" "product=$product" "timing=$timing" ${directory:+"directory=$directory"}
  fi
  note_product "$product"; note_product "$timing"
  snapshot_check
  say "  $name 退出码 $code，产物 $product（$(grep -c '' "$product") 行），计时 $(tail -n 1 "$timing")"
  if [ "$code" -ge 250 ] && [ "$code" -le 254 ]; then stop "$name 撞了内存包装自己的结局 $code（Z3：那一次不算，交主 agent 重跑）"; fi
  return "$code"
}

# driver_step <步名> <基线 -timing 产物> <驱动参数…>：逐行驱动一趟（同修订 6 跑法；工作根是这个单元自己的）
driver_step() {
  local name=$1 baseline_timing=$2; shift 2
  local product timing code entry row
  product=$(own "$name"); timing=$(own_timing "$name")
  refuse_existing "$product"; refuse_existing "$timing"
  entry=$(reuse_entry_for "$name")
  row=$(option_value --output-root "$@")/row-0
  if [ -n "$entry" ]; then  # 取来的行目录放进输出根，驱动照旧起、带 --skip-finished 从它重报（不编、不跑装置与执行器）
    fetch_into "$entry" "row=$row" || stop "$name 取用不成（条目坏了已挪进隔离目录，或目标已在；见 $UNIT_LOG）"
    set -- "$@" --skip-finished
  fi
  if [ "$PRINT_ONLY" = 1 ]; then
    show nice -n 19 python3 -B "$RESEARCH/row_driver.py" --repo-root "$REPO" --work-root "$WORK_ROOT" --snapshot-from "$REPO/$TREE" \
      --baseline-timing "$REPO/$baseline_timing" --timing-output "$REPO/$timing" "$@"
    echo "    → 产物 $product"; return 0
  fi
  snapshot_check
  ( set -o noclobber; nice -n 19 python3 -B "$RESEARCH/row_driver.py" --repo-root "$REPO" --work-root "$WORK_ROOT" \
      --snapshot-from "$REPO/$TREE" --baseline-timing "$REPO/$baseline_timing" --timing-output "$REPO/$timing" "$@" \
      > "$product" 2> "$UNIT_LOG.$name.stderr" )
  code=$?
  if [ -n "$entry" ]; then note_reading "$name" reused "$code" "row=$row"; else note_reading "$name" fresh "$code" "row=$row" "compiled=$WORK_ROOT/lane0"; fi
  note_product "$product"; [ -e "$timing" ] && note_product "$timing"
  snapshot_check
  say "  $name 驱动退出码 $code，$(grep '^name=row_time_limit' "$product")"
  return "$code"
}

# ───── 第 0 步：冻结树产物（修订 8 第 8.3 节那份判法）、stream-constants、S3/S11 流一带探针自检 ─────
unit_step_0() {
  local frozen
  frozen=$(own frozen-tree); refuse_existing "$frozen"
  logged python3 -B "$JUDGE" step-0 --config "$CONFIG" --output "$frozen" || stop "S16 ①②③、A12、Z5 或 H7、H8 触发（见 $frozen 的 verdict 行）"
  note_product "$frozen"
  device_step stream-constants "$CAP_SMALL" "$BIN" stream-constants --streams 1,2,3
  [ $? -eq 0 ] || stop "stream-constants 退出码不是 0（S1 有不过的，看 name=s1 行）"
  must python3 -B "$CHECK" verdict "$(own stream-constants)" s1_failed_checks=0
  for stream in new_pool_file_creation fixed_script_through_e_then_unmount first_file_unlinked_then_inodes_across_one_leaf; do
    must python3 -B "$CHECK" line "$(own stream-constants)" s1 check="${stream}_allocator_carries_the_root_ring_of_make_filesystem" holds=true
  done
  must python3 -B "$CHECK" line "$(own stream-constants)" s1 check=third_stream_split_publish local='Some(234)' holds=true
  note_directory "$DUMPS/s3-s11-baseline-stream-1-with-probe$TAG"
  device_step s3-s11-baseline-stream-1-with-probe "$CAP_SMALL" "$BIN" baseline --streams 1 --writable-mount-probe --output-directory "$DUMPS/s3-s11-baseline-stream-1-with-probe$TAG"
  [ $? -eq 0 ] || stop "S3/S11 自检那一趟退出码不是 0"
  must python3 -B "$CHECK" verdict "$(own s3-s11-baseline-stream-1-with-probe)" s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 \
    s3_o2_judgement_mismatches=0 s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o1_reference_filesystem_red_states=0 \
    o2_red_states=0 probed_states='>0' witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0
  must python3 -B "$TIMING" check "$(own_timing s3-s11-baseline-stream-1-with-probe)"
}

# ───── 第 1 步：S2 新装置基线（流一、流二、不带探针）；不接 S15 ⑤ ─────
unit_step_1() {
  local stem=s2-device-baseline-streams-1-2
  note_directory "$DUMPS/$stem$TAG"
  device_step "$stem" "$CAP_SMALL" "$BIN" baseline --streams 1,2 --output-directory "$DUMPS/$stem$TAG"
  [ $? -eq 0 ] || stop "装置退出码不是 0"
  must python3 -B "$CHECK" verdict "$(own $stem)" s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 \
    s3_o2_judgement_mismatches=0 s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 \
    o1_reference_filesystem_red_states=0 o2_red_states=0 recovery_panic_states=0
  must python3 -B "$TIMING" check "$(own_timing $stem)"
}

# ───── 第 2 步：E170 导出装置 dump（修订 8 树编的），与第 1 步逐文件比（S2） ─────
unit_step_2() {
  # 逐文件比的那一份 s2_compare.py 写在仓外的草稿目录里，本机与 git 里都已经没有（登记 research/prompts/e171-r1-prereg.md 修订 11 的登记修订）：
  # 这一步不起导出装置、不退回别的比法，显式报错退 2
  say "✗ 单元 $UNIT：s2 比对脚本 s2_compare.py 未入库，这一步不能跑"
  say "  → 怎么办：把逐文件比的脚本写进研究目录、登记进跑前登记，再把这一步改回现跑；在那之前第 2 步的 S2 读数记缺，交主 agent"
  exit 2
}

# ───── 第 2b 步（修订 8 新加）：第 2 步导出的拷贝上现导 E170 第 2 次重跑坏状态，种子 85441ae491419411（E170 第 2 次重跑登记 Z3 那一句） ─────
unit_step_2b() {
  local directory=$DUMPS/e170-bad-states-second-run$TAG
  refuse_existing "$directory"
  if [ -z "$(reuse_entry_for e170-bad-states-second-run)" ]; then  # 取用时不拷：取来的导出目录是拷贝加写入之后的整份
    x cp -r "$DUMPS/$S2_E170_EXPORT" "$directory" || stop "拷第 2 步导出没成"
  fi
  note_directory "$directory"
  device_step e170-bad-states-second-run "$CAP_SMALL" "$E170_BIN" bad-states-second-run --dump-directory "$directory" --seed-hex "$E170_SEED"
  [ $? -eq 0 ] || stop "E170 导出装置 bad-states-second-run 退出码不是 0"
  refuse_existing "$(own e170-bad-states-second-run-p1-f-base-digest)"
  must python3 -B "$JUDGE" p1f-base-digest --directory "$directory" --output "$(own e170-bad-states-second-run-p1-f-base-digest)"
  note_product "$(own e170-bad-states-second-run-p1-f-base-digest)"
}

# ───── 第 3 步：阳性对照，基线那一趟（登记 5.2、第七节 A8）；十项各一个单元 ─────
control() {  # control <步名> <内存上限> <装置参数…>：期望退 0
  local name=$1
  note_directory "$DUMPS/$name$TAG"
  device_step "$@"; [ $? -eq 0 ] || stop "$name 装置退出码不是 0"
}
unit_step_3() {
  case "$1" in
    o1-comparison)
      control positive-control-o1-comparison "$CAP_SMALL" "$BIN" baseline --streams 2 --positive-control O1-对拍 --output-directory "$DUMPS/positive-control-o1-comparison$TAG"
      must python3 -B "$CHECK" verdict "$(own positive-control-o1-comparison)" o1_comparison_red_calls=1 s3_o2_judgement_mismatches=0 ;;
    o1-comparison-unmount)
      control positive-control-o1-comparison-unmount "$CAP_SMALL" "$BIN" baseline --streams 2 --positive-control O1-对拍-卸载 --output-directory "$DUMPS/positive-control-o1-comparison-unmount$TAG"
      must python3 -B "$CHECK" verdict "$(own positive-control-o1-comparison-unmount)" o1_comparison_red_calls=1 s3_o2_judgement_mismatches=0
      must python3 -B "$CHECK" line "$(own positive-control-o1-comparison-unmount)" call stream=fixed_script_through_e_then_unmount label=unmount o1_comparison=prefix:disagrees ;;
    o2)
      control positive-control-o2 "$CAP_SMALL" "$BIN" baseline --streams 1 --positive-control O2 --output-directory "$DUMPS/positive-control-o2$TAG"
      must python3 -B "$CHECK" verdict "$(own positive-control-o2)" o2_red_states=1 s3_o2_judgement_mismatches=0 ;;
    return-value)
      control positive-control-return-value "$CAP_SMALL" "$BIN" baseline --streams 2 --positive-control 交回值 --output-directory "$DUMPS/positive-control-return-value$TAG"
      must python3 -B "$CHECK" calls_differ "$(dep "$S2_BASELINE")" "$(own positive-control-return-value)" return_value_sha256 fixed_script_through_e_then_unmount 2
      must python3 -B "$CHECK" call_o1 "$(own positive-control-return-value)" fixed_script_through_e_then_unmount 2 agrees ;;
    memory-object)
      control positive-control-memory-object "$CAP_SMALL" "$BIN" baseline --streams 2 --positive-control 内存对象 --output-directory "$DUMPS/positive-control-memory-object$TAG"
      must python3 -B "$CHECK" calls_differ "$(dep "$S2_BASELINE")" "$(own positive-control-memory-object)" memory_objects_sha256 fixed_script_through_e_then_unmount 2 ;;
    panic)
      # 基线模式下流二截在第 3 次调用，S1 的段长、写数那几条对不上本地常量，装置退 3（第一段报告第 8 节：登记预期内）
      note_directory "$DUMPS/positive-control-panic$TAG"
      EXPECTED_EXIT=3
      device_step positive-control-panic "$CAP_SMALL" "$BIN" baseline --streams 2 --positive-control panic --output-directory "$DUMPS/positive-control-panic$TAG"
      [ $? -eq 3 ] || stop "panic 对照预期退 3（截断的流对不上 S1 本地常量）"
      EXPECTED_EXIT=0
      must python3 -B "$CHECK" line "$(own positive-control-panic)" stream ending=prefix:stopped_by_a_panic:call_3 exported=true \
        comparison_judged_calls_before_the_first_panic=1 sampled_states='>0'
      must python3 -B "$CHECK" verdict "$(own positive-control-panic)" s3_o2_judgement_mismatches=0 streams_not_exported=0 ;;
    witness)
      control positive-control-witness "$CAP_SMALL" "$BIN" baseline --streams 1 --writable-mount-probe --positive-control 见证 --output-directory "$DUMPS/positive-control-witness$TAG"
      must python3 -B "$CHECK" verdict "$(own positive-control-witness)" counter_branch_states=1 counter_branch_states_judged_legal=0 \
        witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0 ;;
    o3-p1)
      # O3：流一全持久态叠 P1 补丁交冻结的执行器，要判「不在」（不是装置，没有 -timing）；P1-F 取第 2b 步在修订 8 树上现导的那一份
      local product entry code
      product=$(own positive-control-o3-p1); refuse_existing "$product"
      entry=$(reuse_entry_for positive-control-o3-p1)
      freeze_check
      if [ -n "$entry" ]; then
        fetch_into "$entry" "product=$product" || stop "O3 阳性对照取用不成（见 $UNIT_LOG）"
        code=$FETCHED_EXIT
        note_reading positive-control-o3-p1 reused "$code" "product=$product"
      else
        logged nice -n 19 bash scripts/hosts/run-with-memory-cap.sh "$CAP_SMALL" bash scripts/hosts/capped.sh "$THREADS" python3 -B "$RESEARCH/o3_bad_image_judge.py" control \
          --repo-root "$REPO" --dumps "$DUMPS/$S2_BASELINE" --e170-dumps "$DUMPS/$E170_SECOND_RUN" --row-name P1-F --output "$product" \
          || { note_reading positive-control-o3-p1 fresh "$?" "product=$product"; stop "O3 阳性对照没跑完（日志 $UNIT_LOG）"; }
        note_reading positive-control-o3-p1 fresh 0 "product=$product"
      fi
      note_product "$product"
      freeze_check
      must python3 -B "$CHECK" line "$product" o3_control_verdict strict=not_in base_digest_matches=true red_strict=true ;;
    o1-crash-judge)
      # 流三在里面的两样（每条流批 1 第一个读回了文件的态各一，恰 3 态）
      control positive-control-o1-crash-judge "$CAP_LARGE" "$BIN" baseline --streams 1,2,3 --positive-control O1-崩溃判官 --output-directory "$DUMPS/positive-control-o1-crash-judge$TAG"
      must python3 -B "$CHECK" verdict "$(own positive-control-o1-crash-judge)" o1_crash_judge_red_states=3 o1_reference_filesystem_red_states=0 s3_o2_judgement_mismatches=0 ;;
    reference-filesystem)
      control positive-control-reference-filesystem "$CAP_LARGE" "$BIN" baseline --streams 1,2,3 --positive-control RefFS --output-directory "$DUMPS/positive-control-reference-filesystem$TAG"
      must python3 -B "$CHECK" verdict "$(own positive-control-reference-filesystem)" o1_reference_filesystem_red_states=3 o1_crash_judge_red_states=0 s3_o2_judgement_mismatches=0 ;;
    *) stop "第 3 步没有这一项：$1" ;;
  esac
}

# ───── 第 4 步：阳性对照，空变异那一趟（逐行驱动、流一流二；A8 第二半）；十项各一个单元 ─────
empty_control() {  # empty_control <名> <对照> <驱动另加的参数…>
  local slug=$1 control_name=$2; shift 2
  note_directory "$DUMPS/empty-control-$slug$TAG"
  driver_step "empty-control-$slug-driver" "$(dep_timing "$S2_BASELINE")" --empty-mutation --keep-dumps --positive-control "$control_name" \
    --lanes 1 --threads "$THREADS" --memory-cap "$CAP_SMALL" --output-root "$REPO/$DUMPS/empty-control-$slug$TAG" "$@"
  [ $? -eq 0 ] || stop "empty-control-$slug 驱动退出码不是 0"
  must python3 -B "$CHECK" rows "$(own "empty-control-$slug-driver")"
  EMPTY_DEVICE_OUT="$DUMPS/empty-control-$slug$TAG/row-0/device.out"
}
unit_step_4() {
  case "$1" in
    o1-comparison) empty_control o1-comparison O1-对拍 --streams 1,2
      must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" o1_comparison_red_calls=1 s3_o2_judgement_mismatches=0 ;;
    o1-comparison-unmount) empty_control o1-comparison-unmount O1-对拍-卸载 --streams 1,2
      must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" o1_comparison_red_calls=1 s3_o2_judgement_mismatches=0 ;;
    o1-crash-judge) empty_control o1-crash-judge O1-崩溃判官 --streams 1,2
      must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" o1_crash_judge_red_states=2 o1_reference_filesystem_red_states=0 ;;
    reference-filesystem) empty_control reference-filesystem RefFS --streams 1,2
      must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" o1_reference_filesystem_red_states=2 o1_crash_judge_red_states=0 ;;
    o2) empty_control o2 O2 --streams 1,2
      must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" o2_red_states=1 ;;
    return-value) empty_control return-value 交回值 --streams 1,2
      must python3 -B "$CHECK" calls_differ "$(dep "$S2_BASELINE")" "$EMPTY_DEVICE_OUT" return_value_sha256 fixed_script_through_e_then_unmount 2
      must python3 -B "$CHECK" call_o1 "$EMPTY_DEVICE_OUT" fixed_script_through_e_then_unmount 2 agrees ;;
    memory-object) empty_control memory-object 内存对象 --streams 1,2
      must python3 -B "$CHECK" calls_differ "$(dep "$S2_BASELINE")" "$EMPTY_DEVICE_OUT" memory_objects_sha256 fixed_script_through_e_then_unmount 2 ;;
    panic) empty_control panic panic --streams 1,2
      must python3 -B "$CHECK" line "$EMPTY_DEVICE_OUT" stream stream=fixed_script_through_e_then_unmount ending=prefix:stopped_by_a_panic:call_3 \
        exported=true comparison_judged_calls_before_the_first_panic=1 sampled_states='>0' ;;
    witness) empty_control witness 见证 --streams 1 --writable-mount-probe
      must python3 -B "$CHECK" verdict "$EMPTY_DEVICE_OUT" counter_branch_states=1 counter_branch_states_judged_legal=0 \
        witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0 ;;
    o3-p1) empty_control o3-p1 O3 --streams 1,2 --o3-control-bad-states "$REPO/$DUMPS/$E170_SECOND_RUN"
      must python3 -B "$CHECK" line "$DUMPS/empty-control-o3-p1$TAG/row-0/o3-control.out" o3_control_verdict strict=not_in base_digest_matches=true red_strict=true ;;
    *) stop "第 4 步没有这一项：$1" ;;
  esac
}

# ───── 第 5 步：O3 基线（冻结的 v2-487 执行器判第 1 步导出的流一、流二） ─────
run_executor() {  # run_executor <产物步名> <导出目录> <流,…> <内存上限>
  local product streams_python entry code
  product=$(own "$1"); refuse_existing "$product"
  streams_python=$(printf "'%s'," ${3//,/ })
  entry=$(reuse_entry_for "$1")
  freeze_check
  if [ -n "$entry" ]; then
    fetch_into "$entry" "product=$product" || stop "执行器那一步取用不成（见 $UNIT_LOG）"
    code=$FETCHED_EXIT
    note_reading "$1" reused "$code" "product=$product"
  else
    logged python3 -B -c "import sys; sys.path.insert(0, '$RESEARCH'); import frozen_o3; code, wall = frozen_o3.run_executor('$REPO', '$REPO/$2', [$streams_python], $THREADS, '$REPO/$product', '$4', '$UNIT_LOG.$1.log'); print('executor_exit', code, 'wall_ns', wall); sys.exit(code)" \
      || { note_reading "$1" fresh "$?" "product=$product"; stop "执行器没退 0（日志 $UNIT_LOG.$1.log）"; }
    note_reading "$1" fresh 0 "product=$product"
  fi
  note_product "$product"
  freeze_check
}
unit_step_5() {
  run_executor o3-baseline-streams-1-2 "$DUMPS/$S2_BASELINE" new_pool_file_creation,fixed_script_through_e_then_unmount "$CAP_SMALL"
  must python3 -B "$CHECK" o3 "$(own o3-baseline-streams-1-2)" new_pool_file_creation,fixed_script_through_e_then_unmount
}

# ───── 第 6 步：Q5 反向取样点（截到第 2 次调用的返回点，末段全枚举 K 取法数那么多态，H14）；不接 S15 ④ ─────
unit_step_6() {
  local stem=q5-reverse-at-the-return-point-of-call-2
  note_directory "$DUMPS/q5-reverse$TAG"
  device_step "$stem" "$CAP_SMALL" "$BIN" confirmed-loss-form --truncate-at-the-return-point --output-directory "$DUMPS/q5-reverse$TAG"
  [ $? -eq 0 ] || stop "装置退出码不是 0（3：S1、S3、S13 或 q5_not_run，看 verdict 行）"
  must python3 -B "$CHECK" verdict "$(own $stem)" s1_failed_checks=0 s3_o2_judgement_mismatches=0 s4_digest_mismatches=0 s13=false unattributed_panic_states=0
  must python3 -B "$CHECK" line "$(own $stem)" q5_shape confirmed_call_returned_before_the_truncation=true in_flight_call_returned=false
  must python3 -B "$CHECK" line "$(own $stem)" q5 states="$Q5_REVERSE_STATES" a10_states_equal_the_product=true
}

# ───── 第 7–9 步：带流三、带探针的基线两次（第一次的进程挂钟是逐行的 T，H13）与它上面的 O3 ─────
baseline_checks() {  # 阴性（A1 的 O1 三列与 O2）、S1、S3、S5、S9、S11（同修订 6 跑法）
  must python3 -B "$CHECK" verdict "$(own "$1")" s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 \
    s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o1_reference_filesystem_red_states=0 o2_red_states=0 \
    probed_states='>0' witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0
  must python3 -B "$CHECK" line "$(own "$1")" stream stream=first_file_unlinked_then_inodes_across_one_leaf ending=completed exported=true model_stopped_at_call=none
  must python3 -B "$TIMING" check "$(own_timing "$1")"
}
unit_step_7() {  # 不接 S15 ⑥⑤；A1 照旧（baseline_checks）。同时在跑的单元由编排在这一单元绿时另记一行进它的 -timing 产物（只描述）
  note_directory "$DUMPS/baseline-run1$TAG"
  device_step baseline-run1 "$CAP_LARGE" "$BIN" baseline --streams 1,2,3 --writable-mount-probe --output-directory "$DUMPS/baseline-run1$TAG"
  local code=$?
  [ "$code" -eq 0 ] || stop "装置退出码 $code 不是 0（3：停机条款，看 verdict 行）"
  baseline_checks baseline-run1
}
unit_step_8() {
  run_executor o3-baseline-run1 "$DUMPS/$BASELINE_RUN1" new_pool_file_creation,fixed_script_through_e_then_unmount,first_file_unlinked_then_inodes_across_one_leaf "$CAP_LARGE"
  if [ "$PRINT_ONLY" = 1 ]; then
    show python3 -B "$CHECK" o3 "$(own o3-baseline-run1)" new_pool_file_creation,fixed_script_through_e_then_unmount
    show python3 -B "$CHECK" o3 "$(own o3-baseline-run1)" first_file_unlinked_then_inodes_across_one_leaf; return 0
  fi
  python3 -B "$CHECK" o3 "$(own o3-baseline-run1)" new_pool_file_creation,fixed_script_through_e_then_unmount >> "$UNIT_LOG" 2>&1 \
    || stop "O3 在基线上有「不在」、出错、摘要对不上，或流一流二没态（F2 或执行器缺陷）"
  python3 -B "$CHECK" o3 "$(own o3-baseline-run1)" first_file_unlinked_then_inodes_across_one_leaf >> "$UNIT_LOG" 2>&1 \
    || say "  注意：O3 在流三上一态都没有（F7，比对器记「到不了」；不停）"
}
unit_step_9() {
  note_directory "$DUMPS/baseline-run2$TAG"
  device_step baseline-run2 "$CAP_LARGE" "$BIN" baseline --streams 1,2,3 --writable-mount-probe --output-directory "$DUMPS/baseline-run2$TAG"
  [ $? -eq 0 ] || stop "装置退出码不是 0"
  baseline_checks baseline-run2
}
unit_step_9_z1() {  # 作废 Z1：第 7、9 步两次基线的导出逐字节比（check_product.py z1，同修订 6 跑法第 9 步末行）
  must python3 -B "$CHECK" z1 "$DUMPS/$BASELINE_RUN1" "$DUMPS/$BASELINE_RUN2"
}

# ───── 第 10 步：Q5 主取样点两次（Z7），每次一个单元；q5 states= 取 H14；不接 S15 ④ ─────
Q5_MAIN=q5-main-before-the-first-barrier-of-call-3
unit_step_10() {  # unit_step_10 <1|2>
  local run=$1 directory product
  if [ "$run" = 1 ]; then directory=$DUMPS/q5-main$TAG; else directory=$DUMPS/q5-main-run-2$TAG; fi
  note_directory "$directory"
  device_step "$Q5_MAIN-run-$run" "$CAP_SMALL" "$BIN" confirmed-loss-form --output-directory "$directory"
  [ $? -eq 0 ] || stop "第 $run 次退出码不是 0（3：S13、S1、S3 或 q5_not_run，看 verdict 行）"
  product=$(own "$Q5_MAIN-run-$run")
  must python3 -B "$CHECK" verdict "$product" s1_failed_checks=0 s3_o2_judgement_mismatches=0 s4_digest_mismatches=0 s13=false unattributed_panic_states=0
  must python3 -B "$CHECK" line "$product" q5_equivalence_pairing mismatching_states=0 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 \
    read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
  must python3 -B "$CHECK" line "$product" q5_piece_panic_control holds=true mismatching_states=0 s13=false   # F9 不触发
  must python3 -B "$CHECK" line "$product" q5_equivalence input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 \
    recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
  must python3 -B "$CHECK" line "$product" q5 states="$Q5_MAIN_STATES" a10_states_equal_the_product=true
}
unit_step_10_z7() {  # 作废 Z7：两份结果产物逐字节同（结果产物里只有 E7RESULT 行，耗时在 -timing 那一份）
  must cmp "$(dep "$Q5_MAIN_RUN_1")" "$(dep "$Q5_MAIN_RUN_2")"
  [ "$PRINT_ONLY" = 1 ] || say "  Z7：两次逐字节相同"
}

# ───── 研究一侧变异表（第 0 步判 research_side_differs=true 时才排：修订 8 第 8.3 节末表，修订 1 第 1 行的做法；产物名带 -r8） ─────
unit_research_mutations() {
  local product entry code
  product=$(own research-mutations); refuse_existing "$product"
  entry=$(reuse_entry_for research-mutations)
  if [ -n "$entry" ]; then
    fetch_into "$entry" "product=$product" || stop "研究一侧变异表取用不成（见 $UNIT_LOG）"
    code=$FETCHED_EXIT
    note_reading research-mutations reused "$code" "product=$product"
    [ "$PRINT_ONLY" = 1 ] || python3 -B "$CHECK" verdict "$product" survived=0 timed_out=0 restored_and_baseline_green=true >> "$UNIT_LOG" 2>&1 \
      || stop "研究一侧变异表没过（见 $product 的 verdict 行）"
  else
    x mkdir -p "$WORK_ROOT"
    [ "$PRINT_ONLY" = 1 ] && show cd "$RESEARCH"
    ( { [ "$PRINT_ONLY" = 1 ] || cd "$RESEARCH"; } && logged nice -n 19 bash "$REPO/scripts/hosts/capped.sh" "$THREADS" python3 -B mutation_driver.py \
        --table "$REPO/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes.tsv" --scratch "$WORK_ROOT" --output "$REPO/$product" )
    code=$?
    note_reading research-mutations fresh "$code" "product=$product"
    [ "$code" -eq 0 ] || stop "研究一侧变异表没过（见 $product 的 verdict 行）"
  fi
  note_product "$product"
}

# ───── RP2（修订 11 第 11.8 节）：H7 那个装置二进制跑三次 stream-constants（只打 S1 常量），复用关入库、复用开取来、另一个目录现跑照 RC1 比 ─────
unit_reuse_selfproof() {
  local product
  product=$(own reuse-selfproof); refuse_existing "$product"
  x mkdir -p "$WORK_ROOT"
  logged nice -n 19 python3 -B "$RESEARCH/reuse_library.py" selfproof --library "$REUSE_LIBRARY/selfproof" --binary "$BIN" --scratch "$WORK_ROOT/selfproof" \
    --output "$product" --threads "$THREADS" --memory-cap "$CAP_SMALL" --repo "$REPO" || stop "RP2 不过（见 $product 的 verdict 行）"
  note_product "$product"
}

case "$UNIT" in
  step-0) unit_step_0 ;;
  step-1) unit_step_1 ;;
  step-2) unit_step_2 ;;
  step-2b) unit_step_2b ;;
  step-3-*) unit_step_3 "${UNIT#step-3-}" ;;
  step-4-*) unit_step_4 "${UNIT#step-4-}" ;;
  step-5) unit_step_5 ;;
  step-6) unit_step_6 ;;
  step-7) unit_step_7 ;;
  step-8) unit_step_8 ;;
  step-9) unit_step_9 ;;
  step-9-z1) unit_step_9_z1 ;;
  step-10-run-1) unit_step_10 1 ;;
  step-10-run-2) unit_step_10 2 ;;
  step-10-z7) unit_step_10_z7 ;;
  research-mutations) unit_research_mutations ;;
  reuse-selfproof) unit_reuse_selfproof ;;
  *) echo "认不得的单元：$UNIT" >&2; exit 2 ;;
esac
# gate-lint:nocount 一次只跑点名的一个单元，不扫一批对象；单元里判了多少写在它自己的输出里
[ "$PRINT_ONLY" = 1 ] || say "✓ 单元 $UNIT 绿"
exit 0
