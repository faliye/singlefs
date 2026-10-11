#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 scripts/gate/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判
# gate-stage: QEMU 真设备上的新池新建文件、发布 B、第二个实例、发布 D 与抬 F（两块 virtio 盘、设备侧独立录制、漏一道屏障与走页缓存两个对照必须判红）
# gate-category: checker-tier 类
# gate-similar: checker-tier-layer0-replay.sh 它也验 crates 写路的崩溃一致性、也是 checker-tier 类，但在宿主上枚举崩溃状态、判录制流截断重放；这里起 QEMU 虚机在两块 virtio 盘上跑真负载、判设备侧独立录制与程序录制逐项相同，要 QEMU、/dev/kvm 与内核镜像，前提、登记的输入与失败方式都不同
# gate-similar: checker-tier-research-build-and-replay.sh 它的复跑那一格也读 E142 的产物，但比的是纯模型实验的整份输出；这里只从 E142 产物取段序列与反向链当虚机跑的对照，判的是真设备上收到的写
#
# gate-cell: device-streams QEMU 真设备上各档负载跑完，设备侧独立录制与程序录制逐项相同、两个对照红在该红的地方
# 参数解析、每格一个子 shell、汇总与退出码经 lib/stage-cells.sh。
#
# 「程序以为发了什么」与「盘上实际收到了什么」由两条不共享代码的路比：程序那一条是被测程序自己的录制器（宿主上同参数重跑，同字节）；
# 盘那一条是 QEMU 的 blklogwrites 过滤节点在来宾之外按 dm-log-writes 格式记下的每个写（带数据）与每个 FLUSH。
# 不声明 gate-covers：「最终判据」要真实负载 + 崩溃注入 + checker 全绿，这一道没有崩溃注入（欠在 C6（块层语义假设写错））。
#
# 用法：bash .claude/gate.d/checker-tier-qemu-device-streams.sh [选项] [项目根]；不带选项时六个模式并行各起一台虚机。
#   --list                  逐行打格名与判什么
#   --check device-streams  只跑这一格；格名写错退 2
#   --list-items            逐行打六个模式与各自判什么，不编译、不起虚机
#   --item <模式>           只跑点名的模式（给几次取并集；写错退 2、列出可点的模式）：不问整道的复用判定与改动范围，不写全绿标记
#   认不出的选项退 2。
# 读的环境变量：VM_LOG_MB（每块盘的设备侧日志 MiB，默认 4096）；GATE_QEMU_DEVICE_STREAMS_KEEP_WORK=1（判绿也留现场，重录样本时拷四件套用）；
#   SINGLEFS_GATE_FULL=1 由 stage-must-run.sh 与 change-touches-crates.sh 读（不复用、不按改动范围跳过）。
#
# 样本 fixtures/checker-tier-qemu-device-streams.sh/{red,direct-divergence-red,e142-row-missing-red,e142-row-duplicate-red,green}（stage-selftest.sh 跑，各放 .gate-cells 点名 device-streams）：
# 样本根放 .qemu-prerecorded，按 <模式>/out.txt、vm-exit、check.txt、check-exit 摆某一轮真跑的原样输出，本阶段不起虚机、不编译，
# 用同一段判定代码判（只供测试的开关：.claude/rules/fs-design.md 五条硬要求第 2 条）。预录档全过这一格退 77：退 0 与真起过虚机的一档在汇总里分不出。
#
# 弄坏开关（只给样本证红用）：GATE_QEMU_DEVICE_STREAMS_BREAK=direct-divergence-unread 让 direct 那一档只看退出码与宿主读回、不逐盘读 divergence=none 那一行，
#   direct-divergence-red（direct 的 check.txt 里盘 1 那一行改成 device=2、退出码仍是 0）判绿退 77、expect 里那几条 want 找不到。
#   GATE_QEMU_DEVICE_STREAMS_BREAK=e142-all-rows 让取 E142 产物的那一步退回「取复跑表里全部 E142 行的第 4 列」、不数行数：
#   green（复跑表里两行 E142，真仓现在的形态）取到两个文件名，红在「从 replay.sh 的 @driver_e142 那一行解析不出产物文件」、不是预期的退 77，expect 里的 want 对不上；
#   e142-row-missing-red（@driver_e142 那一行缺了）取到别的行的文件名，过了这一处，红不在「第 2 列是 @driver_e142 的 E142 行取到 0 行」。
#
# 准入：读的路径与三个前提（QEMU、/dev/kvm、内核镜像）都登记在 .claude/gate.d/stage-inputs.tsv 本阶段那一行，复用判定与改动范围都从它取，
# 那一行不在判红；判法在 scripts/gate/admission.py。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
ADMISSION_MODULE="$STAGE_REPOSITORY/scripts/gate/admission.py"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell device-streams cell_device_streams "QEMU 真设备上各档负载跑完，设备侧独立录制与程序录制逐项相同、两个对照红在该红的地方" \
  "照上面那句「怎么办」改；单跑一档：bash .claude/gate.d/checker-tier-qemu-device-streams.sh --item <模式>（模式用 --list-items 看）"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

fail() { echo "  ✗ $1"; echo "     → 怎么办：$2"; exit 1; }

MODES=(direct skip-new-pool-file-creation-barrier page-cache file-overwrite second-instance raise-rollback-floor)

list_items=0; root_argument=""; named_modes=()
while (($#)); do
  case "$1" in
    --list-items) list_items=1; shift ;;
    --item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then
        echo "  ✗ --item 后面缺模式名"
        echo "     → 怎么办：写成 --item <模式>（例 --item direct），几档就给几次；可点的模式：${MODES[*]}"
        exit 2
      fi
      named_modes+=("$2"); shift 2 ;;
    --item=*) named_modes+=("${1#--item=}"); shift ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check device-streams、--list-items、--item <模式> 与一个项目根。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
if ((list_items)); then
  printf '%s\t%s\n' \
    direct "O_DIRECT 真写路：设备侧日志逐项等于程序的录制流，冷重开读回文件，宿主从盘镜像再读回一次" \
    skip-new-pool-file-creation-barrier "对照：盘 0 漏掉「单元 → journal 记录」那道屏障，设备侧比对必须红在盘 0" \
    page-cache "对照：读写走页缓存，设备侧比对必须判红" \
    file-overwrite "新池新建文件之后覆盖写一次（发布 B），冷重开择 (1, 4)" \
    second-instance "发布 B 之后冷重开、可写挂载再发布 C，冷重开择 (2, 8)" \
    raise-rollback-floor "second-instance 走完再发布 D、把 F 抬到 3，冷重开择 (2, 11)"
  exit 0
fi
if ((${#named_modes[@]})); then
  unknown_modes=()
  for named in "${named_modes[@]}"; do
    found=0
    for mode in "${MODES[@]}"; do [[ "$mode" == "$named" ]] && found=1; done
    ((found)) || unknown_modes+=("$named")
  done
  if ((${#unknown_modes[@]})); then
    echo "  ✗ --item 点名的模式不认得：${unknown_modes[*]}"
    echo "     → 怎么办：模式照 bash .claude/gate.d/checker-tier-qemu-device-streams.sh --list-items 列的写：${MODES[*]}"
    exit 2
  fi
  # 按 MODES 的次序只留点名的那几档（判定循环与汇总都按 MODES 走）
  kept_modes=()
  for mode in "${MODES[@]}"; do
    for named in "${named_modes[@]}"; do [[ "$mode" == "$named" ]] && { kept_modes+=("$mode"); break; }; done
  done
  MODES=("${kept_modes[@]}")
fi
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || exit 2

# 冷重开之后应当择到的根。前三个模式停在新池新建文件（实例 1、txg 3）；发布 B 走到 txg 4；
# 第二个实例再走一遍「取号 → 写行 → 暖机 ×2 → 发布 C」，落在实例 2 的 txg 8；
# 抬 F 那一档接着发布 D（txg 9）、抬 F 推两次空发布（txg 10、11），落在实例 2 的 txg 11。
cold_root_of() {
  case "$1" in
    direct|skip-new-pool-file-creation-barrier|page-cache) printf '1:3' ;;
    file-overwrite)                               printf '1:4' ;;
    second-instance)                                  printf '2:8' ;;
    raise-rollback-floor)                             printf '2:11' ;;
    *)                                                printf '' ;;
  esac
}

# 这个模式的输出里必须逐条出现的行，一行一条基本正则。六个模式共有的那几条（段序列、事务计数、
# 冷重开、块层 FLUSH 数）在判定循环里另查，这里只列各模式独有的——没有这几条，后加的三档就等于没跑。
required_lines_of() {
  case "$1" in
    file-overwrite|second-instance|raise-rollback-floor)
      # file_overwrite 单元写 24：kb layout/01-first-txn.md 第八节第二条流「A、B、C 各 24」（来宾单测不钉 B 的段序列）
      cat <<'PATTERNS'
name=file_overwrite root_txg=4 transaction=2 released=
name=file_overwrite .*segments=24+2+1+2 closed_form=
name=publish_writes publish=file_overwrite txg=4 write_calls=
name=publish_writes_against_device window=file_overwrite publishes=1 .*matches=true
PATTERNS
      ;;
    *) : ;;
  esac
  case "$1" in
    second-instance|raise-rollback-floor)
      # third_transaction 单元写 28：来宾单测 new_pool_file_creation_on_device.rs 的 second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches 钉的值
      cat <<'PATTERNS'
name=writable_mount instance=2 chosen_root=1:4 rows_written=1 row_publish_root=2:5 warm_up_txgs=6,7 nanoseconds=
name=publish_writes publish=instance_row txg=5 write_calls=
name=publish_writes publish=warm_up txg=6 write_calls=
name=publish_writes publish=warm_up txg=7 write_calls=
name=publish_writes_against_device window=writable_mount publishes=3 .*matches=true
name=third_transaction root_txg=8 transaction=1 released=
name=third_transaction .*segments=28+2+1+2 closed_form=
name=publish_writes publish=third_transaction txg=8 write_calls=
name=publish_writes_against_device window=third_transaction publishes=1 .*matches=true
PATTERNS
      ;;
    *) : ;;
  esac
  case "$1" in
    raise-rollback-floor)
      # fourth_transaction 单元写 28：来宾单测 new_pool_file_creation_on_device.rs 的 raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count 钉的值
      # raise_rollback_floor：kb layout/01-first-txn.md 第八节第二条流「抬 F 与卸载开头先写的系统配置与上一次的轮换各自成段（2、2）」「抬 F 与卸载推的空发布各 16」，D 的轮换算在 D 自己的窗口里
      cat <<'PATTERNS'
name=fourth_transaction root_txg=9 transaction=2 released=
name=fourth_transaction .*segments=28+2+1+2 closed_form=
name=publish_writes publish=fourth_transaction txg=9 write_calls=
name=publish_writes_against_device window=fourth_transaction publishes=1 .*matches=true
name=raise_rollback_floor floor_before=0 requested_floor=3 ceiling=3 publishes=2 root_txgs=10,11 
name=recover_cold_rollback_floor chosen_root=2:11 rollback_floor_on_disk=3$
name=raise_rollback_floor .*segments=2+16+2+1+2+16+2+1+2 closed_form=
name=publish_writes publish=raise_rollback_floor txg=10 write_calls=
name=publish_writes publish=raise_rollback_floor txg=11 write_calls=
name=publish_writes_against_device window=raise_rollback_floor publishes=2 .*matches=true
PATTERNS
      ;;
    *) : ;;
  esac
}

# 这个模式的输出里不许出现的行：模式参数真的换掉了负载，而不是每一档都跑同一条路。
# 少了这一半，把 MODES 里的新模式换成 `direct` 也照样全绿。
forbidden_lines_of() {
  case "$1" in
    direct|skip-new-pool-file-creation-barrier|page-cache)
      printf '%s\n' 'name=file_overwrite ' 'name=writable_mount ' 'name=third_transaction ' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
    file-overwrite)
      printf '%s\n' 'name=writable_mount ' 'name=third_transaction ' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
    second-instance)
      printf '%s\n' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
    *) : ;;
  esac
}

# 宿主检查照这个模式重跑的段（它的 `name=host_rerun` 行里 windows= 那一串，段名与 new_pool_file_creation_device_log_check.rs
# 的 ProgramWindow::name 相同）：新池新建文件之后那几次发布的写有没有进逐项比对，以这一行为准。只登记新池新建文件之后还有发布的三档。
host_rerun_windows_of() {
  case "$1" in
    file-overwrite)   printf 'mkfs,instance_acquisition,warm_up,new_pool_file_creation,file_overwrite' ;;
    second-instance)      printf 'mkfs,instance_acquisition,warm_up,new_pool_file_creation,file_overwrite,reopen_and_writable_mount,third_transaction' ;;
    raise-rollback-floor) printf 'mkfs,instance_acquisition,warm_up,new_pool_file_creation,file_overwrite,reopen_and_writable_mount,third_transaction,fourth_transaction,raise_rollback_floor' ;;
    *)                    printf '' ;;
  esac
}

# ── 格 device-streams（在自己的子 shell 里跑，fail 与 exit 只结束这一格）──
cell_device_streams() {
prerecorded=0
[[ -f "$ROOT/.qemu-prerecorded" ]] && prerecorded=1

if ((prerecorded == 0)); then
 if ((${#named_modes[@]})); then
  echo "  · 只跑点名的 ${#MODES[@]} 档（${MODES[*]}）：不问整道的复用判定与改动范围，跑完不写全绿标记"
 else
  # 先问能不能复用上一次整轮全绿的判定，再问这次改动碰没碰它判的东西（前缀含本阶段脚本自己）；两样都退 77（本次未跑），不退 0。
  # 判法在 scripts/gate/stage-must-run.sh 与 change-touches-crates.sh 的文件头。范围判定只摘得掉零行代码的改动，细的那一半欠在 C8（门禁范围判不出来）。
  source "$(cd "$(dirname "$0")/../.." && pwd)/scripts/gate/stage-run-or-skip.sh" || { echo "  ✗ source 不进 scripts/gate/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
  stage_run_or_skip "$(cd "$(dirname "$0")/../.." && pwd)/scripts/gate/stage-must-run.sh" "复用上一次整轮全绿的判定" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")"
  # 改动范围的前缀：登记表本阶段那一行的路径（admission.py paths，不另抄一份），加本阶段脚本自己
  qemu_registered_input_paths=()
  if registered_paths_text="$(python3 "$ADMISSION_MODULE" paths "$ROOT" "$(basename "$0")" 2>/dev/null)"; then
    mapfile -t qemu_registered_input_paths <<< "$registered_paths_text"
  fi
  if [[ ${#qemu_registered_input_paths[@]} -eq 0 || -z "${qemu_registered_input_paths[0]}" ]]; then
    fail "$ROOT/.claude/gate.d/stage-inputs.tsv 里没有 $(basename "$0") 这一行（或读不到这份表）：判不出这一道读哪些路径" \
         "在 .claude/gate.d/stage-inputs.tsv 里给本阶段登记它读的路径（制表符分隔，照别的行写）。"
  fi
  stage_run_or_skip "$STAGE_REPOSITORY/scripts/gate/change-touches-crates.sh" "这次改动没碰它判的东西" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 ${qemu_registered_input_paths[*]} 加本阶段脚本，判据见 scripts/gate/change-touches-crates.sh。" \
    -- "$ROOT" "${qemu_registered_input_paths[@]}" ".claude/gate.d/$(basename "$0")"
 fi
  [[ -f Cargo.toml && -d crates/singlefs-harness ]] || { echo "  ! 没有 crates/singlefs-harness，本阶段跳过（步 0 之前没有装置）"; exit 77; }

  # 三个前置（QEMU、/dev/kvm、内核镜像）登记在 stage-inputs.tsv 本阶段那一行第三列，没齐判红
  python3 "$ADMISSION_MODULE" gate-preconditions "$STAGE_REPOSITORY" "$(basename "$0")" || exit 1

  if ! cargo build --release --target x86_64-unknown-linux-musl -p singlefs-checker-tier --bin new_pool_file_creation_on_device >/dev/null 2>&1; then
    fail "虚机二进制（musl 静态）编不过" "单跑 cargo build --release --target x86_64-unknown-linux-musl -p singlefs-checker-tier --bin new_pool_file_creation_on_device 看报错。"
  fi
  if ! cargo build -p singlefs-checker-tier --bin new_pool_file_creation_device_log_check >/dev/null 2>&1; then
    fail "宿主一侧的设备日志检查编不过" "单跑 cargo build -p singlefs-checker-tier --bin new_pool_file_creation_device_log_check 看报错。"
  fi
fi

# 复跑表里 E142 有不止一行（第 2 列 @driver_e142 与 @driver_e142_p8g_crates_dump 等）：只取第 2 列恰是 @driver_e142 的那一行的第 4 列，不取全部 E142 行。
e142_row_filter='$1=="E142" && $2=="@driver_e142"'
[[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" == e142-all-rows ]] && e142_row_filter='$1=="E142"'
e142_product_names="$(awk -F'|' "$e142_row_filter"'{print $4}' scripts/experiments/replay.sh)"
e142_product_row_count="$(grep -c . <<<"$e142_product_names")"
if [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" != e142-all-rows && "$e142_product_row_count" != 1 ]]; then
  fail "scripts/experiments/replay.sh 里第 2 列是 @driver_e142 的 E142 行取到 $e142_product_row_count 行（要恰好 1 行；0 行是这一行被删或改了名，多于 1 行是被重复登记）" \
       "看 scripts/experiments/replay.sh 里 E142|@driver_e142| 开头的行：留一行，第 4 列写这一道对照的 E142 产物文件名；驱动改了名，这里的 @driver_e142 跟着改。"
fi
PRODUCT="research/results/$e142_product_names"
[[ -f "$PRODUCT" ]] || fail "从 replay.sh 的 @driver_e142 那一行解析不出产物文件（得到 $PRODUCT）" "看 scripts/experiments/replay.sh 里 E142|@driver_e142| 那一行的第 4 列，对应的文件在不在 research/results/ 下。"
# 反向链从产物里现取，不写死：写死的数在格式常量一改就过期，红起来像是写路错了。
EXPECTED_BACK_CHAIN="$(sed -n 's/.*name=root_record .*back_chain=\([0-9][0-9]*\).*/\1/p' "$PRODUCT" | head -1)"
[[ -n "$EXPECTED_BACK_CHAIN" ]] || fail "产物里取不出 name=root_record 的 back_chain（$PRODUCT）" "看 $PRODUCT 有没有 name=root_record 那一行、它的 back_chain 字段还在不在。"

if ((prerecorded)); then
  # 预录档：不起虚机、不编译，只判样本目录里摆好的那几个模式。被判的模式现算，不写死——
  # 写死一份清单，样本里少摆一个模式就会整档静默不判（`show-me-test.md`「跳过清单要与被扫集合出自同一份数据、现算」）。
  work="$ROOT"
  present=()
  missing=()
  for mode in "${MODES[@]}"; do
    if [[ -f "$work/$mode/out.txt" && -f "$work/$mode/vm-exit" && -f "$work/$mode/check.txt" && -f "$work/$mode/check-exit" ]]; then
      present+=("$mode")
    else
      missing+=("$mode")
    fi
  done
  ((${#present[@]})) || fail "预录档：$work 下一个模式的四件套（out.txt / vm-exit / check.txt / check-exit）都没摆全" "按 <模式>/out.txt、vm-exit、check.txt、check-exit 放某一轮真跑留下来的原样输出；模式名取 ${MODES[*]} 里的。"
  MODES=("${present[@]}")
else
  work="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-checker-tier-qemu-device-streams.XXXXXX")"
  # 设备侧日志（blklogwrites）连数据记下每块盘收到的每个写，要装得下 mkfs 写零的 768 MiB 与之后的写：默认取一块数据盘的大小（4096 MiB）；
  # vm-bench.sh 自己的 256 MiB 装不下，direct 那几档会在 mkfs 里撞 I/O error。镜像是稀疏文件，只占真写进去的那些。
  # 数据盘或写路再变大，先看判红现场里 log<d>.img 的 du -m。
  QEMU_DEVICE_LOG_MEGABYTES="${VM_LOG_MB:-4096}"
  [[ "$QEMU_DEVICE_LOG_MEGABYTES" =~ ^[1-9][0-9]*$ ]] || fail "VM_LOG_MB 写成了「$QEMU_DEVICE_LOG_MEGABYTES」，vm-bench.sh 要的是正整数（MiB）" "写成正整数，例 VM_LOG_MB=4096；不设就用一块数据盘的大小 4096。"
  # 判绿才清理：判红时出路点名的 check.txt、out.txt 都在 $work 底下，留着现场、印出路径，删不删由看的人定。
  trap 'stage_exit_code=$?; if ((stage_exit_code == 0)) && [[ "${GATE_QEMU_DEVICE_STREAMS_KEEP_WORK:-0}" != 1 ]]; then rm -rf "${work:?}"; elif ((stage_exit_code == 0)); then echo "     ! GATE_QEMU_DEVICE_STREAMS_KEEP_WORK=1，判绿的现场留着：$work（看完自己删）"; else echo "     ! 判红，现场没清：$work（出路里点名的文件都在这底下，看完自己删）"; fi' EXIT
  missing=()
  BIN="target/x86_64-unknown-linux-musl/release/new_pool_file_creation_on_device"
  CHECK="target/debug/new_pool_file_creation_device_log_check"
  pids=()
  for mode in "${MODES[@]}"; do
    mkdir -p "$work/$mode"
    ( VM_DISKS=2 VM_DISK_MB=4096 VM_LOG_MB="$QEMU_DEVICE_LOG_MEGABYTES" VM_BLKLOGWRITES_DIR="$work/$mode" bash scripts/devices/vm-bench.sh "$BIN" "$mode" >"$work/$mode/out.txt" 2>&1
      echo "$?" >"$work/$mode/vm-exit" ) &
    pids+=("$!")
  done
  for pid in "${pids[@]}"; do wait "$pid"; done   # 逐个 wait 取退出码：每台虚机把退出码写进 $work/<模式>/vm-exit，紧接着按 MODES 的顺序逐个读
fi

checks=0
for mode in "${MODES[@]}"; do
  out="$work/$mode/out.txt"
  if [[ "$(cat "$work/$mode/vm-exit")" != 0 ]]; then
    tail -15 "$out" | sed 's/^/        /'
    fail "虚机跑 $mode 没过（vm-bench.sh 的退出码或条数闸）" "单跑：VM_DISKS=2 VM_DISK_MB=4096 VM_LOG_MB=$QEMU_DEVICE_LOG_MEGABYTES VM_BLKLOGWRITES_DIR=<目录> bash scripts/devices/vm-bench.sh <musl 二进制> $mode"
  fi
  checks=$((checks + 1))
  if ! diff <(grep -ao 'E7RESULT name=segments .*' "$PRODUCT" | tr -d '\r') <(grep -ao 'E7RESULT name=segments .*' "$out" | tr -d '\r') >/dev/null; then
    fail "虚机跑 $mode 的段序列与 E142 产物的 name=segments 行不一致" "对照 $PRODUCT 与虚机输出的 name=segments 行；真设备上的写路发出的步骤与装置不同，先查是哪一段。"
  fi
  checks=$((checks + 1))
  cold_root="$(cold_root_of "$mode")"
  [[ -n "$cold_root" ]] || fail "模式 $mode 没有登记冷重开该择到的根" "在 cold_root_of 里给它补一行；新加模式不补这一行，冷重开那一条就恒不判。"
  grep -aq "name=recover_cold outcome=file_read root=$cold_root content_matches=true" "$out" \
    || fail "虚机跑 $mode 冷重开之后没按 ($cold_root) 读回文件" "看 $mode 的 name=recover_cold 行与来宾的 stderr（vm-bench.sh 保留现场用 VM_KEEP=1）；根对不上先查这一档该走到哪一次发布。"
  checks=$((checks + 1))
  grep -aq "name=transaction key_order_mismatches=0 root_txg=3 back_chain=$EXPECTED_BACK_CHAIN" "$out" \
    || fail "虚机跑 $mode 的新池新建文件计数或反向链与产物不符" "产物 name=root_record 行的 back_chain 是 $EXPECTED_BACK_CHAIN；两个运行时计数要是 0。"
  checks=$((checks + 1))
  while IFS= read -r pattern; do
    [[ -n "$pattern" ]] || continue
    grep -aq "$pattern" "$out" || fail "虚机跑 $mode 的输出里没有「$pattern」" "这一档该跑的那次发布没跑、或者结果行改了字段名；先看 $mode 的 out.txt 有哪些 name= 行，再对 crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs 里这一档打的行。"
    checks=$((checks + 1))
  done < <(required_lines_of "$mode")
  while IFS= read -r pattern; do
    [[ -n "$pattern" ]] || continue
    grep -aq "$pattern" "$out" && fail "虚机跑 $mode 的输出里出现了不该有的「$pattern」" "这一档不该跑那次发布：确认送进虚机的模式参数就是 $mode，别的模式的负载跑进来了这一档的判据就全落空。"
    checks=$((checks + 1))
  done < <(forbidden_lines_of "$mode")
  # 来宾块层的 FLUSH 数 = 程序真正交给设备的屏障 + FUA 写（来宾内核是第三条独立的路）。
  # 一行 device_calls 都没有时循环一圈不转，这一条判据就没判：按红记，不按「没有对不上的」放过。
  device_calls_lines=0
  while IFS= read -r line; do
    device_calls_lines=$((device_calls_lines + 1))
    barriers="$(sed -n 's/.* barriers=\([0-9]*\) .*/\1/p' <<<"$line")"
    fua="$(sed -n 's/.* force_unit_access_writes=\([0-9]*\) .*/\1/p' <<<"$line")"
    flushes="$(sed -n 's/.* flush_ios=\([0-9NA]*\).*/\1/p' <<<"$line")"
    [[ "$flushes" =~ ^[0-9]+$ && "$flushes" == "$((barriers + fua))" ]] \
      || fail "虚机跑 $mode 的来宾块层 FLUSH 数（$flushes）不等于屏障 + FUA 写（$barriers + $fua）" "看 name=device_calls 行；读不到（NA）也算红——读不到不等于读到 0。"
    checks=$((checks + 1))
  done < <(grep -ao 'E7RESULT name=device_calls .*' "$out" | tr -d '\r')
  ((device_calls_lines > 0)) \
    || fail "虚机跑 $mode 的输出里一行 name=device_calls 都没有：来宾块层的 FLUSH 数没法与屏障 + FUA 写对账" "看 $mode 的 out.txt：虚机二进制没打这一行、或者行名改了；读不到不等于对得上。"
  if ((prerecorded == 0)); then
    geometry="$(grep -ao 'E7RESULT name=geometry .*' "$out" | tr -d '\r' | head -1)"
    bytes="$(sed -n 's/.*device_bytes=\([0-9]*\).*/\1/p' <<<"$geometry")"
    physical="$(sed -n 's/.*physical_block_size=\([0-9]*\).*/\1/p' <<<"$geometry")"
    minimum="$(sed -n 's/.*minimum_io=\([0-9]*\).*/\1/p' <<<"$geometry")"
    disks=()
    [[ "$mode" == direct ]] && disks=("$work/$mode/disk0.img" "$work/$mode/disk1.img")
    "$CHECK" "$mode" "$work/$mode/log0.img" "$work/$mode/log1.img" "$bytes" "$physical" "$minimum" "${disks[@]}" >"$work/$mode/check.txt" 2>&1
    echo "$?" >"$work/$mode/check-exit"
  fi
done

verdict() { cat "$work/$1/check-exit"; }
# 宿主检查退出 0 的档，两块盘各要有一行 divergence_window=none divergence=none 的 name=device_log：退出码与结果行两处都判，
# 只看退出码的话宿主检查漏比一块盘、或那块盘的行没打出来，都照样判绿
require_each_device_without_divergence() {
  local mode="$1" device
  for device in 0 1; do
    grep -q "name=device_log device=$device .* divergence_window=none divergence=none" "$work/$mode/check.txt" \
      || { sed 's/^/        /' "$work/$mode/check.txt"
           fail "$mode 盘 $device：宿主检查退出 0，却没有这块盘 divergence=none 的 name=device_log 行" "退出码与结果行对不上，先查 crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs 的 main 与 compare_one_device。"; }
    checks=$((checks + 1))
  done
}
judged_device_logs=0
matched_device_log_modes=(); control_modes=()
for mode in "${MODES[@]}"; do
  case "$mode" in
    direct)
      [[ "$(verdict direct)" == 0 ]] || { sed 's/^/        /' "$work/direct/check.txt"
        fail "direct：设备侧日志与程序的录制流对不上" "上面 divergence= 指着第一处不一致的下标、程序以为的与盘上收到的；先判是写路的错还是比对口径的错。"; }
      grep -q 'name=host_recover outcome=file_read root=1:3 content_matches=true' "$work/direct/check.txt" \
        || fail "direct：宿主从虚机写出的盘镜像上读不回文件" "看 $work/direct/check.txt 的 name=host_recover 行。"
      checks=$((checks + 2))
      [[ "${GATE_QEMU_DEVICE_STREAMS_BREAK:-}" == direct-divergence-unread ]] || require_each_device_without_divergence direct
      matched_device_log_modes+=("$mode")
      ;;
    skip-new-pool-file-creation-barrier)
      [[ "$(verdict skip-new-pool-file-creation-barrier)" == 1 ]] && grep -q 'name=device_log device=0 .*divergence=at=' "$work/skip-new-pool-file-creation-barrier/check.txt" \
        && grep -q 'name=device_log device=1 .*divergence=none' "$work/skip-new-pool-file-creation-barrier/check.txt" \
        || { sed 's/^/        /' "$work/skip-new-pool-file-creation-barrier/check.txt"
             fail "对照 skip-new-pool-file-creation-barrier 没有红在盘 0：这道比对分不出漏了一道屏障" "比对失去判别力，之前所有「对得上」作废；看 device_log.rs 的 expected_device_events 与解析。"; }
      checks=$((checks + 3))
      control_modes+=("$mode")
      ;;
    page-cache)
      [[ "$(verdict page-cache)" == 1 ]] \
        || { sed 's/^/        /' "$work/page-cache/check.txt"
             fail "对照 page-cache 没有判红：走页缓存的回写与 O_DIRECT 的逐个写在这道比对里分不出来" "比对失去判别力；看设备侧日志里写的条数与长度。"; }
      checks=$((checks + 1))
      control_modes+=("$mode")
      ;;
    file-overwrite|second-instance|raise-rollback-floor)
      # 宿主检查照模式重跑到这一档最后那次发布，这几段与新池新建文件一样逐项比，照 direct 判：退出码 0、两块盘都没有分歧，
      # 且 `name=host_rerun` 列出的段就是这一档该跑的那几段（少一段，那一段的写就没进比对）。
      [[ "$(verdict "$mode")" == 0 ]] || { sed 's/^/        /' "$work/$mode/check.txt"
        fail "$mode：设备侧日志与程序的录制流对不上" "上面 divergence_window= 指着第一处不一致落在哪一段、divergence= 指着下标与两侧各是什么，先判是写路的错还是比对口径的错；退出码 2 是用法错或宿主重跑失败，看 check.txt 末尾那句。"; }
      checks=$((checks + 1))
      require_each_device_without_divergence "$mode"
      host_rerun_windows="$(host_rerun_windows_of "$mode")"
      [[ -n "$host_rerun_windows" ]] || fail "模式 $mode 没有登记宿主检查该重跑的段" "在 host_rerun_windows_of 里给它补一行；不补，宿主少重跑一段也看不出来。"
      grep -q "name=host_rerun mode=$mode windows=$host_rerun_windows operations_by_window=" "$work/$mode/check.txt" \
        || { sed 's/^/        /' "$work/$mode/check.txt"
             fail "$mode：宿主检查重跑的段不是 $host_rerun_windows" "看 check.txt 的 name=host_rerun 行；段名单对不上，先查送给 new_pool_file_creation_device_log_check 的第一个参数是不是 $mode。"; }
      checks=$((checks + 1))
      matched_device_log_modes+=("$mode")
      ;;
    *) fail "模式 $mode 没有登记设备侧日志的判据" "在这个 case 里给它补一支；不补就等于这一档的设备侧比对一个字都不判。" ;;
  esac
  judged_device_logs=$((judged_device_logs + 1))
done

[[ "$judged_device_logs" == "${#MODES[@]}" ]] || fail "判了 $judged_device_logs 档设备侧日志，而这一轮跑了 ${#MODES[@]} 档" "两个数对不上就有一档被整个跳过；看上面那个 for 循环。"
uncovered="崩溃注入：${#MODES[@]} 档都没有"
if ((prerecorded)); then
  echo "  ✓ 预录档（.qemu-prerecorded，没起虚机）：判了 ${#MODES[@]} 档 ${MODES[*]}、$checks 项检查全过；样本里没摆的 ${#missing[@]} 档：${missing[*]:-无}"
  echo "     → 这一档不算真跑过 QEMU：真跑要在仓顶层不带 .qemu-prerecorded 跑一遍本阶段。"
  exit 77
fi
echo "  ✓ QEMU 真设备：${#MODES[@]} 次虚机跑（${MODES[*]}）、$checks 项检查全过；${matched_device_log_modes[*]} 设备侧逐项对得上（宿主照模式重跑到这一档最后那次发布），${#control_modes[@]} 个对照（${control_modes[*]}）都红在该红的地方"
echo "     ! 这一道没罩到：$uncovered"
# 全绿标记（git common-dir）供整轮门禁与下一趟复用；写不成只报不红（方向是多跑）。只跑了点名的几档（--item）不写：标记说的是整道。
if ((${#named_modes[@]})); then
  echo "     ! 只跑了点名的 ${#MODES[@]} 档，不是整道结论：没写全绿标记"
elif ! python3 "$ADMISSION_MODULE" stage-marker-write "$ROOT" "$(basename "$0")" >/dev/null; then
  echo "  ! 没写成这一道的全绿标记（admission.py stage-marker-write 没成）：下一趟这一道照跑，不会复用"
fi
}

stage_cells_run "$ROOT"
