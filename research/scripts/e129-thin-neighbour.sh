#!/usr/bin/env bash
# E129 判据 2 乙段：块内 COW 被打断时，同一物理映射单元里的**邻居**怎么坏。
#
# 甲段（e129-thin-rmw.sh）证了机制存在：块共享时一次 512 字节的写让数据设备写 129 扇区。
# 乙段问的是 D2（RAID 条带策略）「写的粒度」的实质主张：
#   「掉电可能**损坏同一映射单元里已经持久的邻居数据**」
# ⇒ 把判据 1 的撕裂注入器（dm-linear: good | error）垫到 pool 的**数据设备**下面，
#   让 COW 的后半截落不下去，再读邻居 B。
#
# 判据（跑前写死）：
#   A 邻居坏不坏：写 A（偏移 0，512 字节）之后读 B（偏移 32768，512 字节），
#     与写之前钉死的已知模式逐字节比。
#   B **坏的形态**要分清三种，不许合成一句：
#     silent  —— 读得出来，但内容与写前不同（静默损坏）
#     eio     —— 读不出来，返回错误（可检出）
#     intact  —— 逐字节完好
#   C 阴性对照：同样流程但**不垫**错误段，B 必须 intact。坏了 ⇒ 是 harness 弄坏的，整格作废。
#   D 绝对值断言：B 的写前内容是脚本生成的固定模式，比对报出第一处不同的偏移。
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

W="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e129n.XXXXXX")"
POOL=sfs_e129n_pool; TH=sfs_e129n_thin; SN=sfs_e129n_snap; DATA=sfs_e129n_data
LOOP_M=""; LOOP_D=""
cleanup() {
  for d in "$SN" "$TH" "$POOL" "$DATA"; do S dmsetup remove "$d" 2>/dev/null; done
  [ -n "$LOOP_M" ] && S losetup -d "$LOOP_M" 2>/dev/null
  [ -n "$LOOP_D" ] && S losetup -d "$LOOP_D" 2>/dev/null
  rm -rf "${W:?}"
}
trap cleanup EXIT
N=0; SPOOL=""; emit() { printf 'E7RESULT %s\n' "$*"; [ -n "$SPOOL" ] && printf '%s\n' "$*" >> "$SPOOL"; N=$((N+1)); }

SPOOL="$W/spool.txt"; : > "$SPOOL"
BLK_SECT=128; BLK_BYTES=$((BLK_SECT*512))
DATA_MB=16; META_MB=8; VOL_SECT=$((16*1024))
B_OFF=32768                       # 邻居 B 在块内的偏移
GOOD_SECT=$(( 128 + 64 ))         # 数据设备前 96 KiB 好：块 0 整块 + 块 1 的前半
ROUNDS=5
emit "name=config block_sectors=$BLK_SECT b_offset=$B_OFF good_sectors=$GOOD_SECT rounds=$ROUNDS"

mkpattern() { python3 -c "
import sys
buf=bytearray()
for off in range(0,$BLK_BYTES,512):
    b=bytearray(512); b[0:8]=off.to_bytes(8,'little'); b[8:16]=b'E129NBR '
    for i in range(16,512): b[i]=(off//512+i)&0xff
    buf+=b
sys.stdout.buffer.write(bytes(buf))" > "$1"; }

one_cell() {
  local round="$1" inject="$2"
  truncate -s "${META_MB}M" "$W/meta.img"; truncate -s "${DATA_MB}M" "$W/data.img"
  LOOP_M="$(S losetup --find --show "$W/meta.img")"
  LOOP_D="$(S losetup --find --show "$W/data.img")"
  S dd if=/dev/zero of="$LOOP_M" bs=4096 count=1 status=none 2>/dev/null
  local data_sect=$(( DATA_MB*1024*1024/512 ))
  if [ "$inject" = 1 ]; then
    printf '0 %d linear %s 0\n%d %d error\n' "$GOOD_SECT" "$LOOP_D" "$GOOD_SECT" "$((data_sect-GOOD_SECT))" > "$W/dt"
  else
    printf '0 %d linear %s 0\n' "$data_sect" "$LOOP_D" > "$W/dt"
  fi
  S dmsetup create "$DATA" "$W/dt" || { emit "name=fatal reason=data_dm_failed"; return; }
  S dmsetup create "$POOL" --table "0 $data_sect thin-pool $LOOP_M /dev/mapper/$DATA $BLK_SECT 0" \
    || { emit "name=fatal reason=pool_failed"; return; }
  S dmsetup message "/dev/mapper/$POOL" 0 "create_thin 0"
  S dmsetup create "$TH" --table "0 $VOL_SECT thin /dev/mapper/$POOL 0"

  mkpattern "$W/pat.bin"
  S dd if="$W/pat.bin" of="/dev/mapper/$TH" bs="$BLK_BYTES" count=1 oflag=direct status=none 2>/dev/null
  S blockdev --flushbufs "/dev/mapper/$TH" 2>/dev/null
  # 建快照 ⇒ 块共享 ⇒ 下一次写触发整块 COW（分配到块 1，横跨 good/error 边界）
  S dmsetup message "/dev/mapper/$POOL" 0 "create_snap 1 0"
  S dmsetup create "$SN" --table "0 $VOL_SECT thin /dev/mapper/$POOL 1"

  # ⚠️ **判别子**：光看「邻居完好」分不出「机制是安全的」和「注入根本没打到 COW」。
  # 所以这一步同时记两样：数据设备上真正写进去了多少扇区，以及 pool 用了几个块。
  # 注入格若 cow_sectors 为 0，就是没打到，那一格的「完好」作废。
  local before_sect after_sect cow_sectors pool_before pool_after
  before_sect="$(awk '{print $7}' "/sys/block/$(basename "$LOOP_D")/stat")"
  pool_before="$(S dmsetup status "/dev/mapper/$POOL" 2>/dev/null | awk '{print $6}')"  # $6 = data_used/data_total

  # 只写 A（偏移 0，512 字节）
  local wrc
  S dd if=/dev/urandom of="/dev/mapper/$TH" bs=512 count=1 oflag=direct status=none 2>/dev/null
  wrc=$?
  S blockdev --flushbufs "/dev/mapper/$TH" 2>/dev/null
  after_sect="$(awk '{print $7}' "/sys/block/$(basename "$LOOP_D")/stat")"
  cow_sectors=$(( after_sect - before_sect ))
  pool_after="$(S dmsetup status "/dev/mapper/$POOL" 2>/dev/null | awk '{print $6}')"

  # 读邻居 B
  local rrc verdict first_diff
  S dd if="/dev/mapper/$TH" of="$W/b.bin" bs=512 count=1 skip=$((B_OFF/512)) iflag=direct status=none 2>/dev/null
  rrc=$?
  if [ "$rrc" -ne 0 ]; then
    verdict=eio; first_diff=-1
  else
    first_diff="$(python3 -c "
got=open('$W/b.bin','rb').read()
want=open('$W/pat.bin','rb').read()[$B_OFF:$B_OFF+512]
print(next((i for i,(a,b) in enumerate(zip(got,want)) if a!=b), -1) if got!=want else -1)")"
    if [ "$first_diff" = "-1" ]; then verdict=intact; else verdict=silent; fi
  fi
  emit "name=neighbour round=$round inject=$inject write_rc=$wrc read_rc=$rrc verdict=$verdict first_diff=$first_diff cow_sectors=$cow_sectors pool_data_before=$pool_before pool_data_after=$pool_after"

  for d in "$SN" "$TH" "$POOL" "$DATA"; do S dmsetup remove "$d" 2>/dev/null; done
  S losetup -d "$LOOP_M" 2>/dev/null; LOOP_M=""
  S losetup -d "$LOOP_D" 2>/dev/null; LOOP_D=""
  rm -f "$W/meta.img" "$W/data.img"
}

for r in $(seq 1 "$ROUNDS"); do one_cell "$r" 1; one_cell "$r" 0; done

# ── 判决。两条，缺一条这一格的「邻居完好」就什么也不证明：
# ① 注入格必须真的把 COW 撕开：0 < cow_sectors < 128。为 0 说明注入没打到 COW，
#    等于 128 说明没撕开 —— 两种都让「完好」失去意义（判据 2 C 的判别力那一半）。
# ② 阴性对照（inject=0）必须 verdict=intact。坏了就是 harness 弄坏的。
tear_bad=$(awk -F'cow_sectors=' '/inject=1/{split($2,a," "); c=a[1]+0; if (c<=0 || c>=128) n++} END{print n+0}' "$SPOOL")
neg_bad=$(awk '/inject=0/ && !/verdict=intact/{n++} END{print n+0}' "$SPOOL")
emit "name=verdict inject_not_torn=$tear_bad negcontrol_damaged=$neg_bad"
if [ "$tear_bad" -gt 0 ] || [ "$neg_bad" -gt 0 ]; then
  emit "name=fatal reason=controls_failed detail=注入没撕开COW或阴性对照被弄坏，整轮作废"
  printf 'E7RESULT name=done emitted=%s\n' "$((N+1))"; exit 5
fi
emit "name=done emitted=$((N+1))"
