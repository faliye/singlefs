#!/usr/bin/env bash
# 第三轮攻方腿模型：choose_slot 的工作根比对（merge-and-verify-snapshot-lib.sh 第 409–416 行）在 SINGLEFS_DATA_ROOT、SINGLEFS_WORK_ROOT、
# 多机配置 SHARED_DATA_ROOT 的各种组合下，拦不拦得住「槽那一层与这一台编译根配不上对」、会不会把配得上对的那一趟误拒。
# 用法：slot-root-pairing.sh <仓根> <冻结副本> <输出目录> <模型目录>（由 tools/rerun.sh 起，整条经统一任务服务交）
#   冻结副本里的 merge-and-verify-snapshot-lib.sh 与 merge-and-verify.sh 要与第三轮开工快照逐字节相同，不同退 3。
#   两臂：intact 是冻结的原样；fix-configured-root 只把第 410 行取缺省工作根那一条换成 probe/configured-default-work-root.py（只看多机配置）。
#   每一格在 <输出目录>/s/<格名>/ 下造假数据根 data（或 data/work 是指向 work-real 的符号链接）、指向它的 data-link、另一个数据根 other-data、
#   另一个工作根 other-work、假编译根 build 与假多机配置（SHARED_DATA_ROOT 是 data 或 data-link，BUILD_ROOT_BY_HOST 这一台指到 build）；
#   配得上对的槽那一层（SHARED_DATA_ROOT/work/merge-and-verify）里放空着的 slot.idle 与留着的 slot.kept，build 下两槽各一份编译目录。
#   调 choose_slot 之前在同一份环境里取一遍编译根、工作根与缺省工作根：编译根或工作根取不到、有一样不落在这一格的根下，就不调（不碰真的工作根与编译根）。
# 输出：每格一行 name=slot_root_pairing …；每臂一行 name=slot_root_pairing_summary …；先跑头 4 格估全量用时，估出超过 2400 秒退 3、不跑全量。
# 退出码：0 跑完；2 用法错；3 冻结副本对不上或估时超限。
set -uo pipefail
(($# == 4)) || { echo "用法：slot-root-pairing.sh <仓根> <冻结副本> <输出目录> <模型目录>" >&2; exit 2; }
repo="$1" frozen="$2" out="$3" model="$4"
expected_snapshot_lib=5aaad87e488aa2e97476c48fed2f708ed6c2b2f6e7f877b3b4d3d6684f735fce
expected_main_script=7ff6c0a3b79a382cfc9b6fa01cf5042419cefe4ce35764425e393d7bd2eca94d
seen_snapshot_lib="$(sha256sum < "$frozen/merge-and-verify-snapshot-lib.sh" | cut -d' ' -f1)"
seen_main_script="$(sha256sum < "$frozen/merge-and-verify.sh" | cut -d' ' -f1)"
if [[ "$seen_snapshot_lib" != "$expected_snapshot_lib" || "$seen_main_script" != "$expected_main_script" ]]; then
  echo "  ✗ 冻结副本与第三轮开工快照不同：snapshot-lib $seen_snapshot_lib、merge-and-verify.sh $seen_main_script" >&2
  exit 3
fi
lib_directory="$out/lib"
mkdir -p "$lib_directory" "$out/s"
for name in .claude/hooks/lib_work_root.py scripts/hosts/lib_repository_copy.py scripts/hosts/multi_host_addresses.py scripts/hosts/run_build_directory.py; do
  echo "name=repo_library path=$name sha256=$(sha256sum < "$repo/$name" | cut -d' ' -f1)"
done
echo "name=frozen_scripts snapshot_lib=$seen_snapshot_lib main_script=$seen_main_script helper=$(sha256sum < "$model/probe/configured-default-work-root.py" | cut -d' ' -f1)"

# 冻结的 merge-and-verify.sh 里挑槽、取编译根的两段 python（单引号包着的多行赋值，里面没有单引号），抽出来 source
awk -v q="'" '$0 == "SLOT_BUILD_ROOT_PROGRAM=" q || $0 == "SLOT_HOLDER_PROGRAM=" q { on = 1; print; next } on { print; if ($0 == q) on = 0 }' \
  "$frozen/merge-and-verify.sh" > "$lib_directory/slot-programs.sh"
# shellcheck source=/dev/null
source "$lib_directory/slot-programs.sh"
[[ "$SLOT_BUILD_ROOT_PROGRAM" == *BUILD_ROOT_BY_HOST* && "$SLOT_HOLDER_PROGRAM" == *"SLOT held"* ]] \
  || { echo "  ✗ 没从冻结的 merge-and-verify.sh 抽出 SLOT_BUILD_ROOT_PROGRAM 与 SLOT_HOLDER_PROGRAM" >&2; exit 3; }
cp -- "$frozen/merge-and-verify-snapshot-lib.sh" "$lib_directory/intact.sh"
python3 - "$lib_directory/intact.sh" "$lib_directory/fix-configured-root.sh" <<'PY'
import sys
source, target = sys.argv[1], sys.argv[2]
text = open(source, encoding="utf-8").read()
old = 'default_work_root="$(python3 "$root/.claude/hooks/lib_work_root.py" shared work)"'
new = 'default_work_root="$(python3 "$PROBE_DEFAULT_HELPER" "$root")"'
if text.count(old) != 1:
    print(f"  ✗ 冻结的 snapshot-lib 里取缺省工作根那一条命中 {text.count(old)} 次，不是 1 次", file=sys.stderr)
    sys.exit(3)
open(target, "w", encoding="utf-8").write(text.replace(old, new))
PY
(( $? == 0 )) || exit 3
PROBE_DEFAULT_HELPER="$model/probe/configured-default-work-root.py"; export PROBE_DEFAULT_HELPER

# 入口脚本里 choose_slot 用到的全局（冻结的 merge-and-verify.sh 第 1149–1160 行的值）；仓里的库只读、照用
root="$repo"
RUN_BUILD_DIRECTORY="$repo/scripts/hosts/run_build_directory.py"
SLOT_ROOT_NAME="merge-and-verify"; SLOT_BUILD_DIRECTORY_NAME="merge-and-verify-slots"; SLOT_RECORD_NAME="slot"; SLOT_HOLDER_SECONDS=120
die() { echo "  ✗ $1"; echo "  → 怎么办：$2"; exit "${3:-2}"; }

# value_of <格根> <记号>：记号换成路径（unset 打空）
value_of() {
  case "$2" in
    unset) printf '' ;;
    data) printf '%s' "$1/data" ;;
    data-link) printf '%s' "$1/data-link" ;;
    other-data) printf '%s' "$1/other-data" ;;
    data-work) printf '%s' "$1/data/work" ;;
    data-link-work) printf '%s' "$1/data-link/work" ;;
    real-work) printf '%s' "$(realpath -m -- "$1/data/work")" ;;
    other-data-work) printf '%s' "$1/other-data/work" ;;
    other-work) printf '%s' "$1/other-work" ;;
    *) echo "  ✗ 认不出的记号 $2" >&2; exit 2 ;;
  esac
}

# build_layout <格根> <plain|work-symlink> <配置里的 SHARED_DATA_ROOT 记号>：造这一格的假根；打配得上对的槽那一层
build_layout() {
  local base="$1" layout="$2" layer name
  mkdir -p "$base/data" "$base/other-data/work" "$base/other-work" "$base/tree" "$base/logs" "$base/build"
  if [[ "$layout" == work-symlink ]]; then
    mkdir -p "$base/work-real"; ln -s -- "$base/work-real" "$base/data/work"; layer="$base/work-real/merge-and-verify"
  else
    mkdir -p "$base/data/work"; layer="$base/data/work/merge-and-verify"
  fi
  ln -s -- "$base/data" "$base/data-link"
  for name in slot.idle slot.kept; do
    mkdir -p "$layer/$name/repo" "$base/build/merge-and-verify-slots/$name/target/debug"
    : > "$layer/$name/used"
    printf 'artifact\n' > "$base/build/merge-and-verify-slots/$name/target/debug/artifact"
  done
  touch -d '-1 day' "$layer/slot.kept/used"
  printf 'log: %s\ncreated: %s\n' "$base/logs" "$(TZ=Asia/Tokyo date +%F)" > "$layer/slot.kept/kept"
  printf '%s\n' "BUILD_ROOT_BY_HOST=127.0.0.1=$base/build 192.0.2.81=/selftest/build-root-of-peer" \
    "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/cargo-of-this 192.0.2.81=/selftest/cargo-of-peer" "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 192.0.2.81=2G" \
    "QUIESCE_STOP_COMMAND=" "QUIESCE_STOPPED_CHECK_COMMAND=" "QUIESCE_START_COMMAND=" "QUIESCE_STARTED_CHECK_COMMAND=" "ENABLE_ACROSS_MACHINES=1" "ENABLE_GPU=0" \
    "NFS_SHARED_ROOTS=$base" "SHARED_DATA_ROOT=$(value_of "$base" "$3")" "HOSTS=127.0.0.1 192.0.2.81" > "$base/multi-host.env"
  printf '%s\n' "$layer"
}

present() { if [[ -e "$1" ]]; then echo 在; else echo 删了; fi; }

# run_case <臂> <plain|work-symlink> <配置记号> <SINGLEFS_DATA_ROOT 记号> <SINGLEFS_WORK_ROOT 记号> <登记几槽>：造一格、调一次 choose_slot、打一行
run_case() {
  local arm="$1" layout="$2" cfg="$3" data="$4" work="$5" limit="$6"
  local name="$arm.$layout.cfg-$cfg.data-$data.work-$work.slots-$limit"
  local base="$out/s/$name" started=$SECONDS layer base_real cfg_value data_value work_value work_used paired used chosen intact_default
  local took fallback outcome idle_target kept_target idle_slot kept_slot stray=0 safe verdict expected_ok observed_accept predicted_accept prediction directory
  mkdir -p "$base"; base_real="$(cd "$base" && pwd -P)"
  layer="$(build_layout "$base" "$layout" "$cfg")"
  cfg_value="$(value_of "$base" "$cfg")"; data_value="$(value_of "$base" "$data")"; work_value="$(value_of "$base" "$work")"
  (
    cd "$base" || exit 1
    unset SINGLEFS_DATA_ROOT SINGLEFS_WORK_ROOT MERGE_AND_VERIFY_SLOT_TABLE MERGE_AND_VERIFY_SLOT_BUILD_ROOT CARGO_TARGET_DIR CARGO_INCREMENTAL \
      SINGLEFS_WORK_ROOT_BREAK MULTI_HOST_RUN_PEER_SCRATCH MERGE_AND_VERIFY_BREAK TASK_SERVER_TASK TASK_SERVER_ATTEMPT
    export SINGLEFS_MULTI_HOST_CONFIG="$base/multi-host.env"
    [[ -z "$data_value" ]] || export SINGLEFS_DATA_ROOT="$data_value"
    [[ -z "$work_value" ]] || export SINGLEFS_WORK_ROOT="$work_value"
    # shellcheck source=/dev/null
    source "$lib_directory/$arm.sh"
    # 不碰真的：同一份环境里先取编译根、工作根与两种缺省工作根，有一样不落在这一格的根下就不调
    guard_build="$(slot_build_root)" || guard_build=""
    guard_work="$(python3 "$root/.claude/hooks/lib_work_root.py")" || guard_work=""
    guard_default="$(python3 "$root/.claude/hooks/lib_work_root.py" shared work)" || guard_default=""
    guard_configured="$(python3 "$PROBE_DEFAULT_HELPER" "$root")" || guard_configured=""
    under() { [[ -z "$1" || "$(realpath -m -- "$1")/" == "$base_real/"* ]]; }
    if [[ -z "$guard_build" || -z "$guard_work" ]] || ! under "$guard_build" || ! under "$guard_work" || ! under "$guard_default" || ! under "$guard_configured"; then
      printf '编译根「%s」、工作根「%s」、缺省工作根「%s」、配置里的工作根「%s」有取不到或不落在这一格的根下的，没调 choose_slot\n' \
        "$guard_build" "$guard_work" "$guard_default" "$guard_configured" > "$base/guard-refused"
      exit 0
    fi
    SLOT_TABLE="$base/slots-of-this-case.conf"; SLOT_LIMIT="$limit"; SLOT_TARGET_LIMIT_GIB=1000; SOURCE_TREE="$base/tree"
    SLOT_DIRECTORY=""; SLOT_TARGET=""; SLOT_FALLBACK_REASON=""; SLOT_ROOT_DIRECTORY=""; SLOT_HOLDER_PID=""; SLOT_HOLDER_FD=""; SNAPSHOT_RELEASE_TEXT=""
    choose_slot "$base/logs"
    printf '%s\n' "$SLOT_DIRECTORY" > "$base/took"; printf '%s\n' "$SLOT_FALLBACK_REASON" > "$base/fallback"
    release_slot
  ) > "$base/choose.out" 2>&1
  took="$(cat "$base/took" 2> /dev/null)"; fallback="$(cat "$base/fallback" 2> /dev/null)"
  if [[ -f "$base/guard-refused" ]]; then outcome=guard-refused
  elif [[ ! -f "$base/fallback" ]]; then outcome=died
  elif [[ -n "$took" ]]; then outcome=took
  elif [[ "$fallback" == *"不是这一台缺省的那一个"* || "$fallback" == *"取不到这一台缺省的工作根"* ]]; then outcome=refused-default
  elif [[ "$fallback" == *"的 slots 是 0" ]]; then outcome=slots-zero
  else outcome=other; fi
  idle_target="$(present "$base/build/merge-and-verify-slots/slot.idle/target/debug/artifact")"
  kept_target="$(present "$base/build/merge-and-verify-slots/slot.kept/target/debug/artifact")"
  idle_slot="$(present "$layer/slot.idle")"; kept_slot="$(present "$layer/slot.kept/kept")"
  for directory in "$base/other-data/work/merge-and-verify"/slot.* "$base/other-work/merge-and-verify"/slot.*; do [[ -d "$directory" ]] && stray=$((stray + 1)); done
  # 配得上对：用的槽那一层就是多机配置 SHARED_DATA_ROOT 下 work 的那一层（编译根从同一份配置读）
  work_used="${work_value:-${data_value:-$cfg_value}/work}"
  paired="$(realpath -m -- "$cfg_value/work")"; used="$(realpath -m -- "$work_used")"
  [[ "$paired" == "$used" ]] && safe=yes || safe=no
  expected_ok=no
  if [[ "$safe" == yes && "$limit" != 0 ]]; then
    chosen="$(dirname -- "$took")"
    [[ "$outcome" == took && "$chosen" == "$(realpath -m -- "$layer")" && "$idle_target$kept_target$kept_slot" == 在在在 && "$stray" == 0 ]] && expected_ok=yes
  elif [[ "$safe" == yes ]]; then
    [[ "$outcome" == slots-zero && "$idle_slot$idle_target" == 删了删了 && "$kept_slot$kept_target" == 在在 && "$stray" == 0 ]] && expected_ok=yes
  else
    [[ "$outcome" == refused-default && "$idle_slot$idle_target$kept_slot$kept_target" == 在在在在 && "$stray" == 0 ]] && expected_ok=yes
  fi
  if [[ "$outcome" == guard-refused ]]; then verdict=guard-refused
  elif [[ "$kept_target" == 删了 || ( "$idle_target" == 删了 && ! ( "$safe" == yes && "$limit" == 0 ) ) ]]; then verdict=deleted-paired-targets
  elif [[ "$safe" == yes && "$outcome" == refused-default ]]; then verdict=false-refusal
  elif [[ "$safe" == no && ( "$outcome" == took || "$outcome" == slots-zero ) ]]; then verdict=accepted-unpaired
  elif [[ "$expected_ok" == yes ]]; then verdict=ok
  else verdict=other; fi
  # 跑之前写下的推测：intact 收「工作根 = (SINGLEFS_DATA_ROOT 或配置)/work」的，fix-configured-root 收「工作根 = 配置/work」的
  intact_default="$(realpath -m -- "${data_value:-$cfg_value}/work")"
  if [[ "$arm" == intact ]]; then [[ "$used" == "$intact_default" ]] && predicted_accept=yes || predicted_accept=no
  else predicted_accept="$safe"; fi
  [[ "$outcome" == took || "$outcome" == slots-zero ]] && observed_accept=yes || observed_accept=no
  [[ "$predicted_accept" == "$observed_accept" ]] && prediction=match || prediction=miss
  printf 'name=slot_root_pairing arm=%s layout=%s cfg=%s data=%s work=%s slots=%s safe=%s outcome=%s idle_slot=%s idle_target=%s kept_slot=%s kept_target=%s stray_slots=%s verdict=%s prediction=%s seconds=%s\n' \
    "$arm" "$layout" "$cfg" "$data" "$work" "$limit" "$safe" "$outcome" "$idle_slot" "$idle_target" "$kept_slot" "$kept_target" "$stray" "$verdict" "$prediction" "$((SECONDS - started))"
  LAST_VERDICT="$verdict"; LAST_PREDICTION="$prediction"
}

# 全格：两臂 × 两种布局 × 配置两种写法 × SINGLEFS_DATA_ROOT 四种 × SINGLEFS_WORK_ROOT 六种 × 登记 3 槽与 0 槽
specifications=()
for layout in plain work-symlink; do
  for cfg in data data-link; do
    for data in unset data data-link other-data; do
      for work in unset data-work data-link-work real-work other-data-work other-work; do
        for limit in 3 0; do
          for arm in intact fix-configured-root; do specifications+=("$arm $layout $cfg $data $work $limit"); done
        done
      done
    done
  done
done
total=${#specifications[@]}
declare -A counts=()
estimate_after=4 index=0 started_all=$SECONDS
for specification in "${specifications[@]}"; do
  read -r arm layout cfg data work limit <<< "$specification"
  run_case "$arm" "$layout" "$cfg" "$data" "$work" "$limit"
  counts["$arm.cases"]=$(( ${counts["$arm.cases"]:-0} + 1 ))
  counts["$arm.$LAST_VERDICT"]=$(( ${counts["$arm.$LAST_VERDICT"]:-0} + 1 ))
  [[ "$LAST_PREDICTION" == match ]] || counts["$arm.prediction_miss"]=$(( ${counts["$arm.prediction_miss"]:-0} + 1 ))
  index=$((index + 1))
  if ((index == estimate_after)); then
    elapsed=$((SECONDS - started_all)); projected=$(( (elapsed + 1) * total / estimate_after ))
    echo "name=slot_root_pairing_estimate first=$estimate_after seconds=$elapsed total_cases=$total projected_seconds=$projected"
    if ((projected > 2400)); then
      echo "  ✗ 按头 $estimate_after 格估全量 $projected 秒，超过 2400 秒：不跑全量，先缩格数" >&2
      exit 3
    fi
  fi
done
for arm in intact fix-configured-root; do
  echo "name=slot_root_pairing_summary arm=$arm cases=${counts["$arm.cases"]:-0} ok=${counts["$arm.ok"]:-0} deleted_paired_targets=${counts["$arm.deleted-paired-targets"]:-0}" \
    "false_refusal=${counts["$arm.false-refusal"]:-0} accepted_unpaired=${counts["$arm.accepted-unpaired"]:-0} other=${counts["$arm.other"]:-0}" \
    "guard_refused=${counts["$arm.guard-refused"]:-0} prediction_miss=${counts["$arm.prediction_miss"]:-0}"
done
# 跑完核没有留下挑槽的进程（它的参数里带这一格的槽那一层）：只读 /proc、只报
sleep 1
leftover=0
for process in /proc/[0-9]*; do
  command_line="$(tr '\0' ' ' < "$process/cmdline" 2> /dev/null)" || continue
  [[ "$command_line" == *"$out/s/"* ]] && leftover=$((leftover + 1))
done
echo "name=slot_root_pairing_leftover_processes count=$leftover seconds=$((SECONDS - started_all))"
exit 0
