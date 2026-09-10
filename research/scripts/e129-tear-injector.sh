#!/usr/bin/env bash
# E129 判据 1：位置确定的撕裂注入器今天造不造得出。
#
# D2（RAID 条带策略）「写的粒度」那一节末尾逐字：
#   「⚠️ 本机无法用故障注入证伪（`dm-flakey` 造不出撕裂）……证伪等崩溃点重放 harness。」
# 前半句就 flakey 本身而言是对的（它造的是**字节损坏**，不是撕裂）；
# 本脚本问的是后半句：换一个构造，撕裂造不造得出。
#
# 构造：dm-linear 把一台设备拼成两段——前段落在正常 loop 上、后段是 dm-error。
# 对**跨边界**的一次 O_DIRECT 写，撕裂 = ① 写报错 ② 边界之前已落盘 ③ 边界之后没落盘。
#
# 判据与作废条款在 research/prompts/e129-preregistration.md，写在本文件之前。
#
# 口径（它答得了什么、答不了什么）：
# - 它造的是「前缀持久、后缀未持久」这个**盘上状态**，不是真的断电——loop 断不了电。
#   状态等价，路径不等价。这一句与 E34（根环槽几何） 真设备那一半同一条限制。
# - 边界由 dm 表**写死**，所以撕裂点是确定的、可复核的；真断电的撕裂点是随机的。
#   ⇒ 它买到的是「能不能构造出撕裂态去验一条不变量」，不是「撕裂多久发生一次」。
# - 全部构造建在 TMPDIR 下的 loop 文件上，一个字节都不碰 nvme0n1 的分区。
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

W="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e129.XXXXXX")"
DM_NAME="sfs_e129_tear"
DM_DEV="/dev/mapper/$DM_NAME"
LOOP=""
cleanup() {
  S dmsetup remove "$DM_NAME" 2>/dev/null
  [ -n "$LOOP" ] && S losetup -d "$LOOP" 2>/dev/null
  rm -rf "${W:?}"
}
trap cleanup EXIT

N=0
SPOOL="$W/spool.txt"; : > "$SPOOL"
emit() { printf 'E7RESULT %s\n' "$*"; printf '%s\n' "$*" >> "$SPOOL"; N=$((N+1)); }

SEC=512
GOOD_SECTORS=8          # 前段 8 扇区 = 4096 字节
ERR_SECTORS=8           # 后段 8 扇区 = 4096 字节
BOUNDARY=$(( GOOD_SECTORS * SEC ))
BACKING_MB=8
ROUNDS=5

emit "name=config good_sectors=$GOOD_SECTORS err_sectors=$ERR_SECTORS boundary=$BOUNDARY rounds=$ROUNDS sector=$SEC"

# ── 建装置 ───────────────────────────────────────────────────────────
truncate -s "${BACKING_MB}M" "$W/backing.img"
LOOP="$(S losetup --find --show "$W/backing.img")" || { emit "name=fatal reason=losetup_failed"; exit 4; }
emit "name=device loop=$LOOP backing=$W/backing.img"

# 表：0..GOOD → loop；GOOD..GOOD+ERR → error
printf '0 %d linear %s 0\n%d %d error\n' "$GOOD_SECTORS" "$LOOP" "$GOOD_SECTORS" "$ERR_SECTORS" > "$W/table"
S dmsetup create "$DM_NAME" "$W/table" || { emit "name=fatal reason=dmsetup_failed"; exit 4; }
S dmsetup table "$DM_NAME" | sed 's/^/E7RESULT name=dm_table row=/'
N=$((N+2))

# 实测这台合成设备的几何（判据 3 的一半：报出来，与期望比）
for f in logical_block_size physical_block_size minimum_io_size optimal_io_size; do
  v="$(cat "/sys/block/$(basename "$(readlink -f "$DM_DEV")")/queue/$f" 2>/dev/null || echo -)"
  emit "name=geometry field=$f value=$v"
done

# ── 三格：跨边界 / 全在好段（阳性对照）/ 全在错段（阴性对照）────────────
# 每格的判定都要三件事：写报不报错、前缀落没落、后缀落没落。
python3 - "$W" > "$W/pattern.bin" <<'PY'
import sys, os
# 每 512 字节写入它自己的偏移 + 一个固定 magic，逐字节可核（判据 3）
buf=bytearray()
for off in range(0, 8192, 512):
    blk = bytearray(512)
    blk[0:8] = off.to_bytes(8,'little')
    blk[8:16] = b'E129MAGC'
    for i in range(16,512): blk[i] = (off//512 + i) & 0xff
    buf += blk
sys.stdout.buffer.write(bytes(buf))
PY

one_round() {
  local round="$1"
  # 每轮先把 backing 前 8 KiB 清零，确保「落没落」判得出来
  S dd if=/dev/zero of="$LOOP" bs=4096 count=2 oflag=direct status=none 2>/dev/null

  # ---- 格 1：跨边界写 8192 字节 ----
  local rc_span
  S dd if="$W/pattern.bin" of="$DM_DEV" bs=8192 count=1 oflag=direct status=none 2>/dev/null
  rc_span=$?
  # 前缀（0..4095）应落在 loop 上；后缀无处可落
  S dd if="$LOOP" of="$W/rb_span.bin" bs=4096 count=2 iflag=direct status=none 2>/dev/null
  local pre_ok post_ok
  pre_ok=$(cmp -s -n "$BOUNDARY" "$W/rb_span.bin" "$W/pattern.bin" && echo 1 || echo 0)
  # 后缀那 4096 字节在 loop 上对应的位置**不该**被写（它属于 error 段，没有后备）
  post_ok=$(python3 -c "
import sys
d=open('$W/rb_span.bin','rb').read()[$BOUNDARY:$BOUNDARY+4096]
print(1 if d==b'\x00'*4096 else 0)")
  emit "name=span round=$round write_rc=$rc_span prefix_persisted=$pre_ok suffix_absent=$post_ok"

  # ---- 格 2：阳性对照，整写落在好段 ----
  S dd if=/dev/zero of="$LOOP" bs=4096 count=2 oflag=direct status=none 2>/dev/null
  local rc_good
  S dd if="$W/pattern.bin" of="$DM_DEV" bs=4096 count=1 oflag=direct status=none 2>/dev/null
  rc_good=$?
  S dd if="$LOOP" of="$W/rb_good.bin" bs=4096 count=1 iflag=direct status=none 2>/dev/null
  local good_ok
  good_ok=$(cmp -s -n "$BOUNDARY" "$W/rb_good.bin" "$W/pattern.bin" && echo 1 || echo 0)
  emit "name=poscontrol round=$round write_rc=$rc_good all_persisted=$good_ok"

  # ---- 格 3：阴性对照，整写落在错段 ----
  S dd if=/dev/zero of="$LOOP" bs=4096 count=2 oflag=direct status=none 2>/dev/null
  local rc_err
  S dd if="$W/pattern.bin" of="$DM_DEV" bs=4096 count=1 seek=1 oflag=direct status=none 2>/dev/null
  rc_err=$?
  S dd if="$LOOP" of="$W/rb_err.bin" bs=4096 count=2 iflag=direct status=none 2>/dev/null
  local nothing_ok
  nothing_ok=$(python3 -c "
d=open('$W/rb_err.bin','rb').read()
print(1 if d==b'\x00'*len(d) else 0)")
  emit "name=negcontrol round=$round write_rc=$rc_err nothing_persisted=$nothing_ok"
}

for r in $(seq 1 "$ROUNDS"); do one_round "$r"; done

# ── 判决（判据写死在 research/prompts/e129-preregistration.md，跑完不许改）──
# 阳性对照：全在好段 ⇒ 不报错、全落盘。阴性对照：全在错段 ⇒ 报错、一个字节不落。
# 任一格不符 ⇒ 装置分不出差别，整轮作废。
bad=0
pos_bad=$(grep -c 'name=poscontrol .*write_rc=[^0]' "$SPOOL" 2>/dev/null || true)
pos_bad2=$(grep -c 'name=poscontrol .*all_persisted=0' "$SPOOL" 2>/dev/null || true)
neg_bad=$(grep -c 'name=negcontrol .*write_rc=0 ' "$SPOOL" 2>/dev/null || true)
neg_bad2=$(grep -c 'name=negcontrol .*nothing_persisted=0' "$SPOOL" 2>/dev/null || true)
span_ok=$(grep -c 'name=span .*prefix_persisted=1 suffix_absent=1' "$SPOOL" 2>/dev/null || true)
bad=$(( pos_bad + pos_bad2 + neg_bad + neg_bad2 ))
emit "name=verdict poscontrol_bad=$((pos_bad+pos_bad2)) negcontrol_bad=$((neg_bad+neg_bad2)) span_torn_rounds=$span_ok/$ROUNDS"
if [ "$bad" -gt 0 ]; then
  emit "name=fatal reason=controls_failed detail=对照分不出差别，整轮作废"
  printf 'E7RESULT name=done emitted=%s\n' "$((N+1))"; exit 5
fi
emit "name=done emitted=$((N+1))"
