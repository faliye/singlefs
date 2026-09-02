#!/usr/bin/env bash
# E34 真设备那一半：根环槽几何在真 parity 阵列上的失败耦合。
#
# ## 判据（跑前写死，跑完不许改；N=5 轮，判「打掉」要全打掉、判「完好」要全完好）
#
# V1 几何前提：md/raid5（4 腿、chunk 64 KiB）必须报 minimum_io_size == 65536
#    且 physical_block_size == 512 —— E34 的「阵列上 io_min 轻易是 64 KiB」第一次在本仓真设备上量。
# V2 阳性对照（机制存在）：对槽 A 注入撕裂（数据腿上 A 的扇区被改、parity 没跟着改，
#    = 掉电落在数据写与 parity 写之间），再 fail 掉**同一 stripe 行、同扇区位**的受害腿，
#    降级读该位置 —— 那里的数据从未被写过，必须被重建成垃圾。5/5 轮打掉。
#    抓不到 ⇒ 机制没复现，V4/V5 的「完好」全是空话，整轮作废。
# V3 阴性对照：同流程**不注入撕裂**，fail 同一条腿，受害位置必须完好。5/5 轮完好。
#    坏了 ⇒ 是 harness 弄坏的，整轮作废。
# V4 已定几何的区域解耦：撕裂打在 region 0 槽 0，fail 掉 region 1 / region 2 所在腿，
#    region 1 / 2 的槽（素数步长 11 × chunk / 22 × chunk 处）必须完好。
# V5 区域内邻槽：撕裂打在 A（扇区位 0），fail 掉 A/B 共用的那条腿，
#    同 chunk 内邻槽 B（扇区位 1）必须完好 —— parity 按扇区位逐行算，行 1 没被撕裂。
#
# ## 口径（它答得了什么、答不了什么）
#
# - 撕裂 = 绕过 md 直接改数据腿上 A 的扇区。它模拟的是「数据落了、parity 没落」这个盘上状态，
#   **不是真的断电**——loop 设备断不了电。状态等价，路径不等价。
# - 注入前先 mdadm --stop、注入后**降级重组**（少一条腿 --run）再读：
#   ① 贴近真实序列（掉电 → 重启 → 少盘拉起）；② 绕开 md 的 stripe cache——
#   第一版不停阵列，重建用的是缓存里撕裂前的数据，V2 判红整轮作废（该红就红，是对照抓的）。
# - **同一 stripe 行、同扇区位上放两个不同标记**：全零盘上单标记行的 parity 块
#   就是标记的逐字拷贝，grep 会把 parity 腿误认成数据腿（第一版 V4 因此 fail 错腿、结论空转）。
#   两标记异或后 parity 是垃圾 ⇒ grep 唯一命中数据腿，且 leg_of 断言恰好一条腿，多一条就弃轮。
# - 注入后回读确认撕裂真的落了（command-safety.md「改完回读」）。
# - 它演示的耦合是 **RAID5 写洞**：耦合单元是「同 stripe 行 × 同扇区位」跨腿，
#   宽度 = (devs−1) × chunk，**不是单个 io_min 单元**。
#   E34 主张 1 的另一条机制（SSD 内部对整个映射单元的 RMW）**在 loop 上注入不了，仍欠**。
# - 镜像落 TMPDIR；设备名全部来自变量并打印；口令从 .env 读、不打印；
#   sudo -S 吃 stdin ⇒ 数据走文件不走管道；EXIT 清理按字面量名字。
set -uo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)/.."
cd "$REPO" || exit 2

set -a; . ./.env 2>/dev/null; set +a
PASS=""
for v in "${SUDO_PASS_A:-}" "${SUDO_PASS_B:-}"; do
  [ -n "$v" ] || continue
  printf '%s\n' "$v" | sudo -S -p '' true 2>/dev/null && { PASS="$v"; break; }
done
[ -n "$PASS" ] || { echo "E7RESULT name=fatal reason=no_sudo"; exit 3; }
S() { printf '%s\n' "$PASS" | sudo -S -p '' "$@"; }

W="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e34.XXXXXX")"
MD_NAME="sfs_e34_probe"
MD_DEV="/dev/md/$MD_NAME"
LOOPS=()
cleanup() {
  S mdadm --stop "$MD_DEV" 2>/dev/null
  for l in "${LOOPS[@]:-}"; do [ -n "$l" ] && S losetup -d "$l" 2>/dev/null; done
  rm -rf "$W"
}
trap cleanup EXIT

N=0
emit() { printf 'E7RESULT %s\n' "$*"; N=$((N+1)); }

DISK_MB=64
CHUNK_K=64
CHUNK=$(( CHUNK_K * 1024 ))
DEVS=4
emit "name=config devs=$DEVS chunk=$CHUNK disk_mb=$DISK_MB level=raid5 rounds=5"

# ── 槽位（数组内字节偏移）。槽宽按 D22 已定 = pbs = 512。 ──────────────
OFF_A=0                       # region 0 槽 0：撕裂打在这里
OFF_B_INTRA=512               # 同 chunk 邻槽（扇区位 1）
OFF_VICTIM=$CHUNK             # 同 stripe 行、下一条数据腿、扇区位 0：写洞受害位
OFF_R1=$(( 11 * CHUNK ))      # region 1 槽 0（素数步长 P=11，stripe 行 3）
OFF_R2=$(( 22 * CHUNK ))      # region 2 槽 0（stripe 行 7）
# 同行同扇区位的陪衬标记：让每行 parity = 两标记异或 = 垃圾，grep 唯一命中数据腿
OFF_C_B=$(( CHUNK + 512 ))    # 陪 B_INTRA（行 0、扇区位 1）
OFF_C_R1=$(( 10 * CHUNK ))    # 陪 R1（行 3、扇区位 0）
OFF_C_R2=$(( 21 * CHUNK ))    # 陪 R2（行 7、扇区位 0）

fresh_array() {
  S mdadm --stop "$MD_DEV" 2>/dev/null
  for l in "${LOOPS[@]:-}"; do [ -n "$l" ] && S losetup -d "$l" 2>/dev/null; done
  LOOPS=()
  local i
  for i in 0 1 2 3; do
    rm -f "$W/d$i.img"; truncate -s "${DISK_MB}M" "$W/d$i.img"
    local L; L="$(S losetup --find --show "$W/d$i.img")" || return 1
    LOOPS+=("$L")
  done
  S mdadm --create "$MD_DEV" --level=5 --raid-devices=$DEVS --chunk=$CHUNK_K \
      --assume-clean --run --quiet "${LOOPS[@]}" 2>/dev/null || return 1
  S udevadm settle 2>/dev/null
  return 0
}

# 写一个 512 字节标记块到数组偏移；标记内容进文件（sudo -S 吃 stdin，不走管道）
put_mark() { # $1=偏移字节 $2=标记串
  printf '%-512s' "$2" > "$W/mark.bin"
  S dd if="$W/mark.bin" of="$MD_DEV" bs=512 seek=$(( $1 / 512 )) conv=notrunc,fsync oflag=direct status=none
}
# 降级读数组偏移处 512 字节，回显开头 32 字节
get_mark() { # $1=偏移字节
  S dd if="$MD_DEV" of="$W/read.bin" bs=512 skip=$(( $1 / 512 )) count=1 iflag=direct status=none
  head -c 32 "$W/read.bin"
}
# 标记在哪条腿：逐个 backing 文件找（观测，不算 layout）。
# **必须恰好一条腿**——命中两条说明 parity 里还躺着逐字拷贝（陪衬标记没起效），弃轮。
leg_of() { # $1=标记串 → 打印腿号或空
  local i hits=() 
  for i in 0 1 2 3; do
    grep -aqF "$1" "$W/d$i.img" 2>/dev/null && hits+=("$i")
  done
  [ "${#hits[@]}" -eq 1 ] || return 1
  echo "${hits[0]}"
}
# 撕裂注入：把某条腿 backing 文件里标记串所在扇区改成垃圾（parity 不动）
tear() { # $1=腿号 $2=标记串
  local off
  off=$(grep -aboF "$2" "$W/d$1.img" | head -1 | cut -d: -f1) || return 1
  [ -n "$off" ] || return 1
  off=$(( off / 512 * 512 ))
  printf '%-512s' "TORN_GARBAGE_$$" > "$W/torn.bin"
  # 直写 loop 设备（O_DIRECT），绕过 md，也绕开 backing 文件页缓存的别名问题
  S dd if="$W/torn.bin" of="${LOOPS[$1]}" bs=512 seek=$(( off / 512 )) conv=notrunc,fsync oflag=direct status=none || return 1
  # 回读确认：原标记必须已经不在这条腿上（改完回读，替换必须断言命中）
  grep -aqF "$2" "$W/d$1.img" 2>/dev/null && return 1
  return 0
}
fail_leg() { # $1=腿号
  S mdadm "$MD_DEV" --fail "${LOOPS[$1]}" --quiet 2>/dev/null &&
  S mdadm "$MD_DEV" --remove "${LOOPS[$1]}" --quiet 2>/dev/null
  S sh -c 'echo 3 > /proc/sys/vm/drop_caches'
}

# ── V1：几何。只建一次量一次（几何不随轮变）。 ───────────────────────
fresh_array || { emit "name=fatal reason=mdadm_create_failed"; exit 4; }
MD_SYS="$(readlink -f "$MD_DEV")"; MD_SYS="/sys/block/$(basename "$MD_SYS")/queue"
IO_MIN=$(cat "$MD_SYS/minimum_io_size" 2>/dev/null || echo NA)
PBS=$(cat "$MD_SYS/physical_block_size" 2>/dev/null || echo NA)
LBS=$(cat "$MD_SYS/logical_block_size" 2>/dev/null || echo NA)
IO_OPT=$(cat "$MD_SYS/optimal_io_size" 2>/dev/null || echo NA)
emit "name=v1_geometry io_min=$IO_MIN pbs=$PBS lbs=$LBS io_opt=$IO_OPT expect_io_min=$CHUNK v1_pass=$([ "$IO_MIN" = "$CHUNK" ] && [ "$PBS" = "512" ] && echo 1 || echo 0)"

run_scenario() { # $1=轮号 $2=场景名 $3=受害偏移 $4=注入撕裂(0/1) $5=fail哪个标记的腿
  local round="$1" scen="$2" v_off="$3" do_tear="$4" fail_mark="$5"
  fresh_array || { emit "name=fatal reason=mdadm_create_failed round=$round"; exit 4; }
  # ⚠️ 每个标记两侧加 ~ 定界，且全程 grep -F：否则 "B_1_9" 会命中 companion
  #   "CB_1_9" 里的子串，leg_of 误判成两条腿而弃轮（V5 第一版栽在这里）。
  local MA="~A_${round}_$$~" MB="~B_${round}_$$~" MV="~V_${round}_$$~"
  local M1="~R1_${round}_$$~" M2="~R2_${round}_$$~"
  put_mark $OFF_A       "$MA"
  put_mark $OFF_B_INTRA "$MB"
  put_mark $OFF_VICTIM  "$MV"
  put_mark $OFF_R1      "$M1"
  put_mark $OFF_R2      "$M2"
  # 同行同位的陪衬标记：把每行 parity 变成异或垃圾，数据腿的 grep 才唯一
  put_mark $OFF_C_B  "~CB_${round}_$$~"
  put_mark $OFF_C_R1 "~C1_${round}_$$~"
  put_mark $OFF_C_R2 "~C2_${round}_$$~"
  S sync
  # 掉电序列：先停干净（superblock 落盘），再在停机状态注入，再少一条腿拉起
  S mdadm --stop "$MD_DEV" 2>/dev/null
  local legA legV
  legA="$(leg_of "$MA")" || { emit "name=skip scen=$scen round=$round reason=legA_ambiguous"; return; }
  local vmark
  case "$fail_mark" in
    V)  vmark="$MV";;
    R1) vmark="$M1";;
    R2) vmark="$M2";;
    B)  vmark="$MB";;
  esac
  legV="$(leg_of "$vmark")" || { emit "name=skip scen=$scen round=$round reason=legV_ambiguous"; return; }
  if [ "$do_tear" = "1" ]; then
    tear "$legA" "$MA" || { emit "name=skip scen=$scen round=$round reason=tear_failed"; return; }
  fi
  # 降级重组：受害腿不给它
  local members=() i
  for i in 0 1 2 3; do [ "$i" != "$legV" ] && members+=("${LOOPS[$i]}"); done
  S mdadm --assemble "$MD_DEV" --run --quiet "${members[@]}" 2>/dev/null || {
    emit "name=skip scen=$scen round=$round reason=degraded_assemble_failed"; return; }
  S udevadm settle 2>/dev/null
  S sh -c 'echo 3 > /proc/sys/vm/drop_caches'
  local got intact
  got="$(get_mark "$v_off")"
  case "$got" in "$vmark"*) intact=1;; *) intact=0;; esac
  emit "name=scenario scen=$scen round=$round legA=$legA legV=$legV torn=$do_tear victim_intact=$intact vacuous=$([ "$legA" = "$legV" ] && [ "$fail_mark" != "B" ] && echo 1 || echo 0)"
}

for round in 1 2 3 4 5; do
  run_scenario $round v2_writehole_hits  $OFF_VICTIM  1 V    # 期望 victim_intact=0
  run_scenario $round v3_no_tear_control $OFF_VICTIM  0 V    # 期望 1
  run_scenario $round v4_region1_survives $OFF_R1     1 R1   # 期望 1
  run_scenario $round v4_region2_survives $OFF_R2     1 R2   # 期望 1
  run_scenario $round v5_intra_slot_survives $OFF_B_INTRA 1 B # 期望 1
done

emit "name=done emitted=$(( N + 1 ))"
