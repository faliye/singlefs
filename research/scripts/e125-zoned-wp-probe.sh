#!/usr/bin/env bash
# E125：顺序区写指针之上读到什么 —— D22 未定项 6 甲问「只做寻址提示」臂欠的那次观测。
#
# ## 被引用条款逐字（verify-before-claiming.md「把定义句原样贴进注释」）
#
# - D22「根落在同一类，且两类布局需要的是同一种机制」：
#   「**读取次序是要求，不是实现细节**：必须**先逐个验证全部候选、再在有效者中按代号择新**」
# - D22 未定项 6：「崩溃后写指针可能领先于我们认定的提交点。
#   D20 承重面的表述目前覆盖不到它。」
# - D22 已定三：原地覆写的结构要带**整单元校验和**与**被实际检查的世代号**。
# - D20 七机制表：「在飞记录数上限 | 把窗口外的校验失败判为损坏而非撕裂 | —— 」
# - D22 已定项 2：「区域 r 落在 `r × P × chunk`，**P 素数且 `P > devs`**」；槽宽
#   「**等于挂载时探测的 `physical_block_size`**」。
# - E48：「`prime_stride`：区域 r 在 `r × P × chunk`，**P = 8191（素数）**」；chunk ∈ {64 KiB, 512 KiB, 4 MiB}。
# - invariants.md 的 zoned 缺口：「zoned 设备上一个 zone 里只要还有一块活数据就不能 reset」
# - tooling.md：「本机唯一的真实块设备是 `nvme0n1`……**没有真的旋转盘、没有真的 4Kn 盘、
#   没有真的 zoned 盘**，所有相关数字都只能是模拟出来的，引用时必须带这个口径」
#
# ## 判据（E125 正文跑前写死，跑完不许改）见 kb/experiments/125-顺序区写指针之上读到什么.md
#
# ## 口径
#
# 被测对象是 `null_blk`（内存后端）造出来的 host-managed zoned 设备，**不是真 ZNS 盘**。
# 本脚本只回答 null_blk 的行为；每个数引用时都要带上面 tooling.md 那句。
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO" || exit 2

# ── sudo：口令从仓根 .env 取（与 e72-devtable-probe.sh 同一形态）──
# 用 askpass 而不是 `printf | sudo -S`：后者会把口令占掉被调命令的 stdin，
# `sudo tee` 就会把口令当成要写的内容——实测踩过，configfs 静默不生效。
ASKPASS="$(mktemp "${TMPDIR:-/tmp}/singlefs-e125-askpass.XXXXXX")"
cat > "$ASKPASS" <<EOF
#!/bin/sh
. $REPO/.env
printf '%s\n' "\$SUDO_PASS_A"
EOF
chmod 700 "$ASKPASS"
export SUDO_ASKPASS="$ASKPASS"
sudo -A true 2>/dev/null || { echo "E7RESULT name=fatal reason=no_sudo"; rm -f "$ASKPASS"; exit 3; }
S() { sudo -A "$@"; }

CFG=/sys/kernel/config/nullb
DEVS=()          # 建过的 configfs 目录名，按字面量清理，不按模式匹配杀
WORK="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e125.XXXXXX")"
cleanup() {
  local d
  for d in "${DEVS[@]:-}"; do
    [ -n "$d" ] || continue
    printf '0' | S tee "$CFG/$d/power" >/dev/null 2>&1
    S rmdir "$CFG/$d" 2>/dev/null
  done
  rm -rf "${WORK:?}"
  rm -f "${ASKPASS:?}"
}
trap cleanup EXIT

N=0
emit() { printf 'E7RESULT %s\n' "$*"; N=$((N+1)); }
fatal() { emit "name=fatal reason=$1"; emit "name=count n=$N"; exit 4; }

# 建一台 null_blk zoned 设备。$1=名字 其余=attr=value
mk() {
  local name="$1"; shift
  S mkdir -p "$CFG/$name" || return 1
  DEVS+=("$name")
  local kv
  for kv in "$@"; do
    printf '%s' "${kv#*=}" | S tee "$CFG/$name/${kv%%=*}" >/dev/null || return 1
  done
  printf '1' | S tee "$CFG/$name/power" >/dev/null || return 1
  # 等设备节点出现
  local i
  for i in $(seq 1 50); do [ -b "/dev/$name" ] && return 0; sleep 0.1; done
  return 1
}

# blkzone report 的一行 → 关心的字段。$1=设备 $2=区序号
zline() { S blkzone report -o "$(( $2 * ZSECT ))" -c 1 "$1" 2>/dev/null; }

# ────────────────────────────────────────────────────────────────
# 装置一：4 个区 × 64 MiB，前 1 个常规区，pbs 4096
# ────────────────────────────────────────────────────────────────
ZMB=64; NCONV=1; SIZEMB=256; BS=4096
mk e125a "blocksize=$BS" "memory_backed=1" "zoned=1" "zone_size=$ZMB" \
        "zone_nr_conv=$NCONV" "size=$SIZEMB" || fatal cannot_create_e125a
DEV=/dev/e125a
ZBYTES=$((ZMB*1024*1024)); ZSECT=$((ZBYTES/512))

PBS=$(cat /sys/block/e125a/queue/physical_block_size)
LBS=$(cat /sys/block/e125a/queue/logical_block_size)
NZ=$(cat /sys/block/e125a/queue/nr_zones)
emit "name=config dev=$DEV zone_bytes=$ZBYTES nr_zones=$NZ nr_conv=$NCONV pbs=$PBS lbs=$LBS backend=null_blk_memory"
# 槽宽 = 挂载时探测的 physical_block_size（D22 已定项 2）
SLOT=$PBS
[ "$SLOT" = 4096 ] || fatal "slot_width_not_4096:$SLOT"
[ "$ZBYTES" = 67108864 ] || fatal "zone_bytes_not_64MiB:$ZBYTES"

# ── 阴性对照：空顺序区 wp == start，cond == EMPTY ──
Z1=1                       # 区 1 是第一个顺序区（区 0 是常规区）
L="$(zline $DEV $Z1)"
WP0=$(sed -n 's/.*wptr \(0x[0-9a-f]*\).*/\1/p' <<<"$L")
CND0=$(sed -n 's/.*zcond: *\([0-9]*\).*/\1/p' <<<"$L")
emit "name=neg_control zone=$Z1 wptr=$WP0 zcond=$CND0"
[ "$((WP0))" -eq 0 ] || fatal "neg_control_wp_not_zero:$WP0"
[ "$CND0" = 1 ] || fatal "neg_control_cond_not_empty:$CND0"

# ── 阳性对照：写 M 条槽宽记录，wp 必须 == M × 槽宽 ──
M=8
python3 - "$WORK/rec.bin" "$SLOT" "$M" <<'PY'
import sys
p,w,m=sys.argv[1],int(sys.argv[2]),int(sys.argv[3])
buf=bytearray()
for i in range(m):
    r=bytearray(b'\xA5')*w
    r[0:8]=b'SFSROOT\x00'          # 特征字节：重置之后拿它判「痕迹还在不在」
    r[8:12]=i.to_bytes(4,'little')
    buf+=r
open(p,'wb').write(bytes(buf))
PY
ZOFF=$((Z1*ZBYTES))
S dd if="$WORK/rec.bin" of=$DEV bs=$SLOT count=$M seek=$((ZOFF/SLOT)) \
     oflag=direct conv=notrunc status=none || fatal write_failed
L="$(zline $DEV $Z1)"
WP1=$(sed -n 's/.*wptr \(0x[0-9a-f]*\).*/\1/p' <<<"$L")
WPB=$(( WP1 * 512 ))
emit "name=pos_control zone=$Z1 records=$M slot=$SLOT wptr_sectors=$((WP1)) wp_bytes=$WPB expect_bytes=$((M*SLOT))"
[ "$WPB" -eq "$((M*SLOT))" ] || fatal "pos_control_wp_mismatch:$WPB!=$((M*SLOT))"
# ⚠️ 上面那条是**互比**：M 同时驱动写入与期望值，改 M 两边一起动，断言照样成立
# （实测：把 M 从 8 改成 7，那条一声不吭）。test-discipline.md 要求每条互比旁边
# 有一条把绝对值钉死的，绝对值由独立算术给出：8 条 × 4096 字节 = 32768。
[ "$WPB" -eq 32768 ] || fatal "pos_control_wp_absolute:$WPB!=32768"
# 判据 1 要读的字节数同样钉死：67108864 − 32768 = 67076096。
[ $(( ZBYTES - WPB )) -eq 67076096 ] || fatal "above_wp_span_absolute:$((ZBYTES-WPB))!=67076096"

# ── 判据 1：读 [wp, zone_end) ──
ABOVE=$((ZOFF+WPB)); NBLK=$(( (ZBYTES-WPB)/SLOT ))
S dd if=$DEV bs=$SLOT count=$NBLK skip=$((ABOVE/SLOT)) iflag=direct \
     status=none > "$WORK/above.bin" 2>"$WORK/above.err"
RC1=$?
GOT=$(stat -c %s "$WORK/above.bin" 2>/dev/null || echo 0)
if [ $RC1 -ne 0 ]; then
  emit "name=crit1_above_wp outcome=read_failed rc=$RC1 errno_text=$(tr -d '\n' < "$WORK/above.err" | tr ' ' '_')"
else
  python3 - "$WORK/above.bin" <<'PY' > "$WORK/above.sum"
import sys
b=open(sys.argv[1],'rb').read()
s=set(b)
nz=next((i for i,v in enumerate(b) if v!=0), -1)
print(f"len={len(b)} distinct={len(s)} uniform_byte={('0x%02x'%b[0]) if len(s)==1 else 'none'} "
      f"first_nonzero_off={nz} first_nonzero_val={('0x%02x'%b[nz]) if nz>=0 else 'none'}")
PY
  emit "name=crit1_above_wp outcome=read_ok rc=0 bytes_requested=$((NBLK*SLOT)) bytes_got=$GOT $(cat "$WORK/above.sum")"
fi

# ── 判据 2：区重置之后 wp、内容、以及特征字节还在不在 ──
S blkzone reset -o $((Z1*ZSECT)) -c 1 $DEV >/dev/null 2>&1 || fatal reset_failed
L="$(zline $DEV $Z1)"
WP2=$(sed -n 's/.*wptr \(0x[0-9a-f]*\).*/\1/p' <<<"$L")
CND2=$(sed -n 's/.*zcond: *\([0-9]*\).*/\1/p' <<<"$L")
S dd if=$DEV bs=$SLOT count=$M skip=$((ZOFF/SLOT)) iflag=direct status=none > "$WORK/afterreset.bin" 2>/dev/null
RC2=$?
python3 - "$WORK/afterreset.bin" <<'PY' > "$WORK/ar.sum"
import sys
b=open(sys.argv[1],'rb').read()
s=set(b)
print(f"len={len(b)} distinct={len(s)} uniform_byte={('0x%02x'%b[0]) if len(s)==1 and b else 'none'} "
      f"marker_found_at={b.find(b'SFSROOT')}")
PY
emit "name=crit2_after_reset rc=$RC2 wptr_sectors=$((WP2)) wp_bytes=$((WP2*512)) zcond=$CND2 $(cat "$WORK/ar.sum")"

# ── 判据 5：常规区 vs 顺序区的 type，以及常规区总字节 ──
TYPES=""
for ((z=0; z<NZ; z++)); do
  t=$(zline $DEV $z | sed -n 's/.*type: *\([0-9]*\).*/\1/p')
  TYPES="$TYPES$t"
done
CONVB=$((NCONV*ZBYTES))
emit "name=crit5_zone_types types=$TYPES conv_zones=$NCONV conv_bytes=$CONVB"
# 未写过的常规区读到什么（与顺序区对照——判据 1 的解读要用）
S dd if=$DEV bs=$SLOT count=1 skip=0 iflag=direct status=none > "$WORK/conv.bin" 2>/dev/null
python3 - "$WORK/conv.bin" <<'PY' > "$WORK/conv.sum"
import sys
b=open(sys.argv[1],'rb').read(); s=set(b)
print(f"distinct={len(s)} uniform_byte={('0x%02x'%b[0]) if len(s)==1 and b else 'none'}")
PY
emit "name=crit5_conv_unwritten $(cat "$WORK/conv.sum")"

# 常规区覆不覆盖 {0, P·chunk, 2P·chunk}（「一律落常规区」臂要求的三个点）
P=8191
for CH in 65536 524288 4194304; do
  P1=$((P*CH)); P2=$((2*P*CH))
  C1=$([ $P1 -lt $CONVB ] && echo yes || echo no)
  C2=$([ $P2 -lt $CONVB ] && echo yes || echo no)
  emit "name=crit5_coverage chunk=$CH point1=$P1 point2=$P2 conv_bytes=$CONVB p1_inside=$C1 p2_inside=$C2"
done

# ── 判据 4：区边界对齐，跨装置对账（纯算术，与 D22 那九格逐格比）──
for CH in 65536 524288 4194304; do
  PC=$((P*CH))
  for ZS in 67108864 134217728 268435456; do
    R=$((PC % ZS))
    Q=$(python3 -c "print(f'{$PC/$ZS:.6f}')")
    emit "name=crit4_align chunk=$CH p_times_chunk=$PC zone_size=$ZS remainder=$R quotient=$Q divides=$([ $R -eq 0 ] && echo yes || echo no)"
    [ $R -ne 0 ] || fatal "crit4_unexpected_divides:chunk=$CH:zs=$ZS"
  done
done

# ────────────────────────────────────────────────────────────────
# 判据 3：zone_full=1 造不造得出「WP 领先于我们认定的提交点」
# ────────────────────────────────────────────────────────────────
mk e125b "blocksize=$BS" "memory_backed=1" "zoned=1" "zone_size=$ZMB" \
        "zone_nr_conv=0" "zone_full=1" "size=$SIZEMB" || fatal cannot_create_e125b
DEVB=/dev/e125b
NZB=$(cat /sys/block/e125b/queue/nr_zones)
for ((z=0; z<NZB; z++)); do
  L="$(S blkzone report -o $((z*ZSECT)) -c 1 $DEVB 2>/dev/null)"
  st=$(sed -n 's/.*start: *\(0x[0-9a-f]*\).*/\1/p' <<<"$L")
  wp=$(sed -n 's/.*wptr \(0x[0-9a-f]*\).*/\1/p' <<<"$L")
  cp=$(sed -n 's/.*cap \(0x[0-9a-f]*\).*/\1/p' <<<"$L")
  cn=$(sed -n 's/.*zcond: *\([0-9]*\).*/\1/p' <<<"$L")
  emit "name=crit3_zone_full zone=$z start_bytes=$(( st * 512 )) wp_bytes=$(( wp * 512 )) cap_bytes=$(( cp * 512 )) zcond=$cn wp_eq_cap=$([ $((wp)) -eq $((cp)) ] && echo yes || echo no)"
done
# 这台盘上，一个「本工程认定的提交点」是 0（一个字节都没写过）而 WP 在末尾
emit "name=crit3_verdict committed_bytes=0 wp_bytes=$(( $(sed -n 's/.*wptr \(0x[0-9a-f]*\).*/\1/p' <<<"$(S blkzone report -o 0 -c 1 $DEVB)") * 512 )) lead_constructed=yes"

emit "name=count n=$N"
