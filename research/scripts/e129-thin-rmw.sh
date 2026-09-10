#!/usr/bin/env bash
# E129 判据 2 甲段：一次 512 字节的写会不会让设备把整个物理映射单元读改写一遍。
#
# 这一格答的是 C212（固定结构的写小于 io_min） 候选 ② 逐字欠的那条依据：
#   「那要给出为什么固定结构不怕设备内 RMW，而 E34（根环槽几何） 的真设备那一轮
#     **只证到 RAID5 写洞这一条机制**」
# ⇒ 本段找第二种机制：dm-thin 的**块内 COW**。块共享时对块里任意一个字节的写，
#   都会让 pool 把**整块**搬一遍——这正是 D2（RAID 条带策略）「写的粒度」说的「设备内部会做 read-modify-write」，
#   而它与 RAID5 写洞不是同一条机制（没有 parity，也不跨腿）。
#
# 判据（跑前写死，见 research/prompts/e129-preregistration.md）：
#   A 阳性对照（块共享）：对 thin 卷发一次 512 字节的写，数据设备上写入的扇区数
#     必须 ≥ 128（= 64 KiB 整块），而主机只发了 1 个扇区。
#   B 阴性对照（块独占，无快照）：同样一次 512 字节的写，数据设备上写入的扇区数
#     必须 < 128 —— 分不出这两格就说明计数器没在量 COW，整轮作废。
#   C 绝对值断言：块大小 128 扇区、64 KiB 是脚本写死的常量，
#     且实测的 pool io_min 必须等于它；对不上判红。
#
# 口径：计数器读的是**后备 loop 设备**的 `/sys/block/*/stat` 第 7 字段（写入扇区数），
#   它数的是块层真正下发到该设备的量，不是被测程序自报的（evidence-discipline「校验路径要独立」）。
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

W="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e129t.XXXXXX")"
POOL=sfs_e129_pool; TH=sfs_e129_thin; SN=sfs_e129_snap
LOOP_M=""; LOOP_D=""
cleanup() {
  for d in "$SN" "$TH" "$POOL"; do S dmsetup remove "$d" 2>/dev/null; done
  [ -n "$LOOP_M" ] && S losetup -d "$LOOP_M" 2>/dev/null
  [ -n "$LOOP_D" ] && S losetup -d "$LOOP_D" 2>/dev/null
  rm -rf "${W:?}"
}
trap cleanup EXIT
N=0; SPOOL=""; emit() { printf 'E7RESULT %s\n' "$*"; [ -n "$SPOOL" ] && printf '%s\n' "$*" >> "$SPOOL"; N=$((N+1)); }

SPOOL="$W/spool.txt"; : > "$SPOOL"
BLK_SECT=128            # 128 扇区 = 64 KiB：本段写死的物理映射单元宽度
BLK_BYTES=$(( BLK_SECT * 512 ))
DATA_MB=64; META_MB=8; VOL_SECT=$(( 16 * 1024 ))   # 8 MiB 的卷
ROUNDS=5
emit "name=config block_sectors=$BLK_SECT block_bytes=$BLK_BYTES data_mb=$DATA_MB rounds=$ROUNDS"

written_sectors() { awk '{print $7}' "/sys/block/$(basename "$1")/stat"; }

setup_pool() {
  truncate -s "${META_MB}M" "$W/meta.img"; truncate -s "${DATA_MB}M" "$W/data.img"
  LOOP_M="$(S losetup --find --show "$W/meta.img")"
  LOOP_D="$(S losetup --find --show "$W/data.img")"
  S dd if=/dev/zero of="$LOOP_M" bs=4096 count=1 status=none 2>/dev/null
  local data_sect; data_sect=$(( DATA_MB * 1024 * 1024 / 512 ))
  S dmsetup create "$POOL" --table "0 $data_sect thin-pool $LOOP_M $LOOP_D $BLK_SECT 0" \
    || { emit "name=fatal reason=pool_create_failed"; exit 4; }
  S dmsetup message "/dev/mapper/$POOL" 0 "create_thin 0"
  S dmsetup create "$TH" --table "0 $VOL_SECT thin /dev/mapper/$POOL 0"
}

teardown_pool() {
  for d in "$SN" "$TH" "$POOL"; do S dmsetup remove "$d" 2>/dev/null; done
  [ -n "$LOOP_M" ] && S losetup -d "$LOOP_M" 2>/dev/null; LOOP_M=""
  [ -n "$LOOP_D" ] && S losetup -d "$LOOP_D" 2>/dev/null; LOOP_D=""
  rm -f "$W/meta.img" "$W/data.img"
}

# 一格：shared=1 建快照（块共享）、shared=0 不建（块独占）
one_cell() {
  local round="$1" shared="$2"
  setup_pool
  # 把第一个块整块写实，A 在偏移 0、B（邻居）在偏移 32768
  S dd if=/dev/urandom of="/dev/mapper/$TH" bs="$BLK_BYTES" count=1 oflag=direct status=none 2>/dev/null
  S blockdev --flushbufs "/dev/mapper/$TH" 2>/dev/null
  if [ "$shared" = 1 ]; then
    S dmsetup message "/dev/mapper/$POOL" 0 "create_snap 1 0"
    S dmsetup create "$SN" --table "0 $VOL_SECT thin /dev/mapper/$POOL 1"
  fi
  local io_min pbs
  io_min="$(cat "/sys/block/$(basename "$(readlink -f "/dev/mapper/$TH")")/queue/minimum_io_size" 2>/dev/null)"
  pbs="$(cat "/sys/block/$(basename "$(readlink -f "/dev/mapper/$TH")")/queue/physical_block_size" 2>/dev/null)"
  local before after delta
  before="$(written_sectors "$LOOP_D")"
  # 只写 512 字节
  S dd if=/dev/urandom of="/dev/mapper/$TH" bs=512 count=1 oflag=direct status=none 2>/dev/null
  S blockdev --flushbufs "/dev/mapper/$TH" 2>/dev/null
  after="$(written_sectors "$LOOP_D")"
  delta=$(( after - before ))
  emit "name=cell round=$round shared=$shared host_sectors=1 data_dev_sectors=$delta thin_io_min=$io_min thin_pbs=$pbs"
  teardown_pool
}

for r in $(seq 1 "$ROUNDS"); do one_cell "$r" 1; one_cell "$r" 0; done

# ── 判决：共享格必须放大到整块（≥128 扇区），独占格必须没放大（<128）。
# 两格分不开 ⇒ 计数器没在量 COW，整轮作废（判据 2 B）。
shared_small=$(awk -F'data_dev_sectors=' '/shared=1/{split($2,a," "); if (a[1]+0 < 128) n++} END{print n+0}' "$SPOOL")
excl_big=$(awk -F'data_dev_sectors=' '/shared=0/{split($2,a," "); if (a[1]+0 >= 128) n++} END{print n+0}' "$SPOOL")
# ⚠️ **绝对值断言**：阈值 128 是写死的，它对「块大小往上改」不敏感
# （`.claude/rules/mutation-sampling.md` 第三类）⇒ 另钉一条：每一格实测的
# thin io_min 必须等于本段声明的块宽 65536，对不上说明被测几何已经不是声明的那个。
cells=$(grep -c 'name=cell ' "$SPOOL" || true)
iomin_ok=$(grep -c "thin_io_min=$BLK_BYTES " "$SPOOL" || true)
emit "name=verdict shared_not_amplified=$shared_small exclusive_amplified=$excl_big cells=$cells cells_with_declared_iomin=$iomin_ok declared_block_bytes=$BLK_BYTES"
if [ "$iomin_ok" -ne "$cells" ]; then
  emit "name=fatal reason=geometry_mismatch detail=实测 io_min 与本段声明的块宽对不上"
  printf 'E7RESULT name=done emitted=%s\n' "$((N+1))"; exit 5
fi
if [ "$shared_small" -gt 0 ] || [ "$excl_big" -gt 0 ]; then
  emit "name=fatal reason=controls_failed detail=共享格与独占格分不开，计数器没在量块内COW"
  printf 'E7RESULT name=done emitted=%s\n' "$((N+1))"; exit 5
fi
emit "name=done emitted=$((N+1))"
