#!/usr/bin/env bash
# gate-stage: QEMU 真设备上的第一个事务、发布 B、第二个实例、发布 D 与抬 F（两块 virtio 盘、设备侧独立录制、漏一道屏障与走页缓存两个对照必须判红）
# 不声明 gate-covers：共享清单 2026-09-16 起没有「QEMU 真实负载」这一项（QEMU 移交给本工程），而「最终判据」要的是真实负载 + 崩溃注入 + checker 全绿，
# 这一道今天只有真实负载与设备侧录制、没有崩溃注入，不冒充覆盖它。
#
# C6（块层语义假设写错）要的：「程序以为发了什么」与「盘上实际收到了什么」由两条不共享代码的路比。
#   程序那一条 = 被测程序自己的录制器（宿主上同参数重跑，同字节）；
#   盘那一条   = QEMU 的 blklogwrites 过滤节点在来宾之外按 dm-log-writes 格式记下的每个写（带数据）与每个 FLUSH。
# 六次虚机跑（并行）：
#   direct                          O_DIRECT 真写路：设备侧日志逐项等于程序的录制流（末尾至多一个关机 FLUSH），
#                                   来宾块层的 FLUSH 数 = 屏障 + FUA 写数，冷重开读回文件，段序列与 E142 产物逐字相同，宿主从盘镜像再读回一次
#   skip-first-transaction-barrier  盘 0 漏掉「单元 → journal 记录」那道屏障：设备侧比对必须判红，且红在盘 0
#   page-cache                      读写走页缓存（阳性对照）：回写合并 / 重排写，设备侧比对必须判红
#   second-transaction              第一个事务之后同一个进程里覆盖写一次（发布 B，里程碑「第二个事务」步 1）：冷重开择 (1, 4) 读回第二版
#   second-instance                 发布 B 之后同一对盘冷重开、可写挂载（取号 → 写行 txg 5 → 暖机 txg 6、7）再发布 C（步 3）：冷重开择 (2, 8) 读回第三版
#   raise-rollback-floor            second-instance 那条路走完，同一次挂载里再发布 D（txg 9，第四版），把 F 抬到上限 3、推两次空发布 txg 10、11
#                                   （增补 2 收口表第 58 行）：冷重开择 (2, 11) 读回第四版
#
# 设备侧比对每一档都逐项比到这一档最后那次发布：`first_transaction_device_log_check` 的第一个参数是模式，宿主照模式重跑
# 发布 B / 可写挂载与发布 C / 发布 D 与抬 F（`name=host_rerun` 行列出重跑了哪几段）。所以后三档照 direct 判：退出码 0、两块盘都没有分歧。
#
# 判别力样本 fixtures/55-qemu-first-transaction.sh/{red,green}（89 号跑）拿预录输出喂：样本目录里放一个 `.qemu-prerecorded` 标记文件，再按 `<模式>/out.txt`、`vm-exit`、
# `check.txt`、`check-exit` 摆好某一轮真跑留下来的原样输出，本阶段就不起虚机、不编译，只拿同一段判定代码判它们
# （`.claude/rules/fs-design.md` 五条硬要求第 2 条：只供测试的开关）。预录档判全过退 3，不退 0——
# 退 0 会和真起过虚机的那一档在汇总里长得一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

fail() { echo "  ✗ $1"; echo "     → 怎么办：$2"; exit 1; }

MODES=(direct skip-first-transaction-barrier page-cache second-transaction second-instance raise-rollback-floor)

# 冷重开之后应当择到的根。前三个模式停在第一个事务（实例 1、txg 3）；发布 B 走到 txg 4；
# 第二个实例再走一遍「取号 → 写行 → 暖机 ×2 → 发布 C」，落在实例 2 的 txg 8；
# 抬 F 那一档接着发布 D（txg 9）、抬 F 推两次空发布（txg 10、11），落在实例 2 的 txg 11。
cold_root_of() {
  case "$1" in
    direct|skip-first-transaction-barrier|page-cache) printf '1:3' ;;
    second-transaction)                               printf '1:4' ;;
    second-instance)                                  printf '2:8' ;;
    raise-rollback-floor)                             printf '2:11' ;;
    *)                                                printf '' ;;
  esac
}

# 这个模式的输出里必须逐条出现的行，一行一条基本正则。六个模式共有的那几条（段序列、事务计数、
# 冷重开、块层 FLUSH 数）在判定循环里另查，这里只列各模式独有的——没有这几条，后加的三档就等于没跑。
required_lines_of() {
  case "$1" in
    second-transaction|second-instance|raise-rollback-floor)
      cat <<'PATTERNS'
name=second_transaction root_txg=4 transaction=2 released=
name=second_transaction .*segments=16+2+1+2 closed_form=
name=publish_writes publish=second_transaction txg=4 write_calls=
name=publish_writes_against_device window=second_transaction publishes=1 .*matches=true
PATTERNS
      ;;
    *) : ;;
  esac
  case "$1" in
    second-instance|raise-rollback-floor)
      cat <<'PATTERNS'
name=writable_mount instance=2 chosen_root=1:4 rows_written=1 row_publish_root=2:5 warm_up_txgs=6,7 nanoseconds=
name=publish_writes publish=instance_row txg=5 write_calls=
name=publish_writes publish=warm_up txg=6 write_calls=
name=publish_writes publish=warm_up txg=7 write_calls=
name=publish_writes_against_device window=writable_mount publishes=3 .*matches=true
name=third_transaction root_txg=8 transaction=1 released=
name=third_transaction .*segments=16+2+1+2 closed_form=
name=publish_writes publish=third_transaction txg=8 write_calls=
name=publish_writes_against_device window=third_transaction publishes=1 .*matches=true
PATTERNS
      ;;
    *) : ;;
  esac
  case "$1" in
    raise-rollback-floor)
      cat <<'PATTERNS'
name=fourth_transaction root_txg=9 transaction=2 released=
name=fourth_transaction .*segments=16+2+1+2 closed_form=
name=publish_writes publish=fourth_transaction txg=9 write_calls=
name=publish_writes_against_device window=fourth_transaction publishes=1 .*matches=true
name=raise_rollback_floor floor_before=0 requested_floor=3 ceiling=3 publishes=2 root_txgs=10,11 
name=recover_cold_rollback_floor chosen_root=2:11 rollback_floor_on_disk=3$
name=raise_rollback_floor .*segments=8+2+1+10+2+1+2 closed_form=
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
    direct|skip-first-transaction-barrier|page-cache)
      printf '%s\n' 'name=second_transaction ' 'name=writable_mount ' 'name=third_transaction ' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
    second-transaction)
      printf '%s\n' 'name=writable_mount ' 'name=third_transaction ' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
    second-instance)
      printf '%s\n' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
    *) : ;;
  esac
}

# 宿主检查照这个模式重跑的段（它的 `name=host_rerun` 行里 windows= 那一串，段名与 first_transaction_device_log_check.rs
# 的 ProgramWindow::name 相同）：第一个事务之后那几次发布的写有没有进逐项比对，以这一行为准。只登记第一个事务之后还有发布的三档。
host_rerun_windows_of() {
  case "$1" in
    second-transaction)   printf 'mkfs,instance_acquisition,warm_up,first_transaction,second_transaction' ;;
    second-instance)      printf 'mkfs,instance_acquisition,warm_up,first_transaction,second_transaction,reopen_and_writable_mount,third_transaction' ;;
    raise-rollback-floor) printf 'mkfs,instance_acquisition,warm_up,first_transaction,second_transaction,reopen_and_writable_mount,third_transaction,fourth_transaction,raise_rollback_floor' ;;
    *)                    printf '' ;;
  esac
}

prerecorded=0
[[ -f "$ROOT/.qemu-prerecorded" ]] && prerecorded=1

if ((prerecorded == 0)); then
  # 这次改动没碰这道阶段判的东西就退 77（本次未跑），不退 0——`exit 0` 的跳过在汇总里与「判过了」
  # 一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。这是 C8（范围判定）的粗粒度前身：
  # 它只摘得掉「零行代码的改动」，摘不出别的，C8 照旧欠着。本阶段自己的脚本也算输入：判定代码改了要重跑。
  # 先问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
  # 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
  # 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
  reuse_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "$ROOT" "$(basename "$0")")"
  reuse_rc=$?
  if [[ "$reuse_rc" != 0 ]]; then
    echo "  ! 本阶段跳过（复用上一次整轮全绿的判定）：$reuse_reason"
    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。"
    exit 77
  fi
  scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" crates/ research/scripts/vm-bench.sh .claude/gate.d/55-qemu-first-transaction.sh)"
  scope_rc=$?
  if [[ "$scope_rc" != 0 ]]; then
    echo "  ! 本阶段跳过（这次改动没碰它判的东西）：$scope_reason"
    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；判据与前缀见 research/scripts/change-touches-crates.sh。"
    exit 77
  fi
  [[ -f Cargo.toml && -d crates/singlefs-harness ]] || { echo "  ! 没有 crates/singlefs-harness，本阶段跳过（步 0 之前没有装置）"; exit 77; }

  command -v qemu-system-x86_64 >/dev/null || fail "qemu-system-x86_64 缺失" "装 QEMU（qemu-system-x86），见 .claude/kb/vm-harness.md「三个前置」。"
  [[ -r /dev/kvm && -w /dev/kvm ]] || fail "/dev/kvm 不可读写" "按 .claude/kb/vm-harness.md「三个前置」查 kvm 组成员身份；不许用 setfacl 补。"
  bash research/scripts/vm-kernel.sh --check >/dev/null 2>&1 || fail "找不到可读的内核镜像" "跑 bash research/scripts/vm-kernel.sh，按它打印的路径设 SINGLEFS_KERNEL。"

  if ! cargo build --release --target x86_64-unknown-linux-musl -p singlefs-harness --bin first_transaction_on_device >/dev/null 2>&1; then
    fail "虚机二进制（musl 静态）编不过" "单跑 cargo build --release --target x86_64-unknown-linux-musl -p singlefs-harness --bin first_transaction_on_device 看报错。"
  fi
  if ! cargo build -p singlefs-harness --bin first_transaction_device_log_check >/dev/null 2>&1; then
    fail "宿主一侧的设备日志检查编不过" "单跑 cargo build -p singlefs-harness --bin first_transaction_device_log_check 看报错。"
  fi
fi

PRODUCT="research/results/$(awk -F'|' '$1=="E142"{print $4}' research/scripts/replay.sh)"
[[ -f "$PRODUCT" ]] || fail "从 replay.sh 的 E142 行解析不出产物文件（得到 $PRODUCT）" "看 research/scripts/replay.sh 里 E142 那一行的第 4 列。"
# 反向链从产物里现取，不写死：写死的那个数在格式常量一改就过期，而这道闸红起来像是写路错了。
# 实测 2026-09-16 树表条目 148 → 200：产物已经是新链值，闸里还写着旧的，红在一个与写路无关的地方。
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
  work="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-gate55.XXXXXX")"
  # 判绿才清理：判红时出路点名的 check.txt、out.txt 都在 $work 底下，退出时一删，人照着出路去看就扑空。
  # 判红（退出码不是 0）把现场留下、把路径印出来，删不删由看的人定；盘镜像是 vm-bench.sh 用 truncate 建的稀疏文件，留下只占真写进去的那些。
  trap 'if (($? == 0)); then rm -rf "${work:?}"; else echo "     ! 判红，现场没清：$work（出路里点名的文件都在这底下，看完自己删）"; fi' EXIT
  missing=()
  BIN="target/x86_64-unknown-linux-musl/release/first_transaction_on_device"
  CHECK="target/debug/first_transaction_device_log_check"
  pids=()
  for mode in "${MODES[@]}"; do
    mkdir -p "$work/$mode"
    ( VM_DISKS=2 VM_DISK_MB=4096 VM_BLKLOGWRITES_DIR="$work/$mode" bash research/scripts/vm-bench.sh "$BIN" "$mode" >"$work/$mode/out.txt" 2>&1
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
    fail "虚机跑 $mode 没过（vm-bench.sh 的退出码或条数闸）" "单跑：VM_DISKS=2 VM_DISK_MB=4096 VM_BLKLOGWRITES_DIR=<目录> bash research/scripts/vm-bench.sh <musl 二进制> $mode"
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
    || fail "虚机跑 $mode 的第一个事务计数或反向链与产物不符" "产物 name=root_record 行的 back_chain 是 $EXPECTED_BACK_CHAIN；两个运行时计数要是 0。"
  checks=$((checks + 1))
  while IFS= read -r pattern; do
    [[ -n "$pattern" ]] || continue
    grep -aq "$pattern" "$out" || fail "虚机跑 $mode 的输出里没有「$pattern」" "这一档该跑的那次发布没跑、或者结果行改了字段名；先看 $mode 的 out.txt 有哪些 name= 行，再对 crates/singlefs-harness/src/bin/first_transaction_on_device.rs 里这一档打的行。"
    checks=$((checks + 1))
  done < <(required_lines_of "$mode")
  while IFS= read -r pattern; do
    [[ -n "$pattern" ]] || continue
    grep -aq "$pattern" "$out" && fail "虚机跑 $mode 的输出里出现了不该有的「$pattern」" "这一档不该跑那次发布：确认送进虚机的模式参数就是 $mode，别的模式的负载跑进来了这一档的判据就全落空。"
    checks=$((checks + 1))
  done < <(forbidden_lines_of "$mode")
  # 来宾块层的 FLUSH 数 = 程序真正交给设备的屏障 + FUA 写（来宾内核是第三条独立的路）
  while IFS= read -r line; do
    barriers="$(sed -n 's/.* barriers=\([0-9]*\) .*/\1/p' <<<"$line")"
    fua="$(sed -n 's/.* force_unit_access_writes=\([0-9]*\) .*/\1/p' <<<"$line")"
    flushes="$(sed -n 's/.* flush_ios=\([0-9NA]*\).*/\1/p' <<<"$line")"
    [[ "$flushes" =~ ^[0-9]+$ && "$flushes" == "$((barriers + fua))" ]] \
      || fail "虚机跑 $mode 的来宾块层 FLUSH 数（$flushes）不等于屏障 + FUA 写（$barriers + $fua）" "看 name=device_calls 行；读不到（NA）也算红——读不到不等于读到 0。"
    checks=$((checks + 1))
  done < <(grep -ao 'E7RESULT name=device_calls .*' "$out" | tr -d '\r')
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
judged_device_logs=0
for mode in "${MODES[@]}"; do
  case "$mode" in
    direct)
      [[ "$(verdict direct)" == 0 ]] || { sed 's/^/        /' "$work/direct/check.txt"
        fail "direct：设备侧日志与程序的录制流对不上" "上面 divergence= 指着第一处不一致的下标、程序以为的与盘上收到的；先判是写路的错还是比对口径的错。"; }
      grep -q 'name=host_recover outcome=file_read root=1:3 content_matches=true' "$work/direct/check.txt" \
        || fail "direct：宿主从虚机写出的盘镜像上读不回文件" "看 $work/direct/check.txt 的 name=host_recover 行。"
      checks=$((checks + 2))
      ;;
    skip-first-transaction-barrier)
      [[ "$(verdict skip-first-transaction-barrier)" == 1 ]] && grep -q 'name=device_log device=0 .*divergence=at=' "$work/skip-first-transaction-barrier/check.txt" \
        && grep -q 'name=device_log device=1 .*divergence=none' "$work/skip-first-transaction-barrier/check.txt" \
        || { sed 's/^/        /' "$work/skip-first-transaction-barrier/check.txt"
             fail "对照 skip-first-transaction-barrier 没有红在盘 0：这道比对分不出漏了一道屏障" "比对失去判别力，之前所有「对得上」作废；看 device_log.rs 的 expected_device_events 与解析。"; }
      checks=$((checks + 3))
      ;;
    page-cache)
      [[ "$(verdict page-cache)" == 1 ]] \
        || { sed 's/^/        /' "$work/page-cache/check.txt"
             fail "对照 page-cache 没有判红：走页缓存的回写与 O_DIRECT 的逐个写在这道比对里分不出来" "比对失去判别力；看设备侧日志里写的条数与长度。"; }
      checks=$((checks + 1))
      ;;
    second-transaction|second-instance|raise-rollback-floor)
      # 宿主检查照模式重跑到这一档最后那次发布，这几段与第一个事务一样逐项比，照 direct 判：退出码 0、两块盘都没有分歧，
      # 且 `name=host_rerun` 列出的段就是这一档该跑的那几段（少一段，那一段的写就没进比对）。
      [[ "$(verdict "$mode")" == 0 ]] || { sed 's/^/        /' "$work/$mode/check.txt"
        fail "$mode：设备侧日志与程序的录制流对不上" "上面 divergence_window= 指着第一处不一致落在哪一段、divergence= 指着下标与两侧各是什么，先判是写路的错还是比对口径的错；退出码 2 是用法错或宿主重跑失败，看 check.txt 末尾那句。"; }
      checks=$((checks + 1))
      for device in 0 1; do
        grep -q "name=device_log device=$device .* divergence_window=none divergence=none" "$work/$mode/check.txt" \
          || { sed 's/^/        /' "$work/$mode/check.txt"
               fail "$mode 盘 $device：宿主检查退出 0，却没有这块盘 divergence=none 的 name=device_log 行" "退出码与结果行对不上，先查 crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs 的 main 与 compare_one_device。"; }
        checks=$((checks + 1))
      done
      host_rerun_windows="$(host_rerun_windows_of "$mode")"
      [[ -n "$host_rerun_windows" ]] || fail "模式 $mode 没有登记宿主检查该重跑的段" "在 host_rerun_windows_of 里给它补一行；不补，宿主少重跑一段也看不出来。"
      grep -q "name=host_rerun mode=$mode windows=$host_rerun_windows operations_by_window=" "$work/$mode/check.txt" \
        || { sed 's/^/        /' "$work/$mode/check.txt"
             fail "$mode：宿主检查重跑的段不是 $host_rerun_windows" "看 check.txt 的 name=host_rerun 行；段名单对不上，先查送给 first_transaction_device_log_check 的第一个参数是不是 $mode。"; }
      checks=$((checks + 1))
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
  exit 3
fi
echo "  ✓ QEMU 真设备：${#MODES[@]} 次虚机跑（${MODES[*]}）、$checks 项检查全过；direct、second-transaction、second-instance、raise-rollback-floor 设备侧逐项对得上（宿主照模式重跑到这一档最后那次发布），两个对照都红在该红的地方"
echo "     ! 这一道没罩到：$uncovered"
