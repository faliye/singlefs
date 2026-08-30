#!/usr/bin/env bash
# E56：条带映射的算术在真设备上对不对。
#
# E47 / E49 的全部几何结论压在一条算术上：逻辑地址 p 落在第 `(p / chunk) % devs` 块盘。
# 那条算术**只读过 Linux 源码（raid0.c / raid5.c），从没在设备上量过**。
# 本脚本在**真的条带设备**（dm-stripe，若可用再加 md/raid0）上写标记、回查落在哪块盘。
#
# 纪律（`.claude/singlefs-ai-sop/rules/command-safety.md`）：
#   - 镜像落 TMPDIR 不落仓库；设备名全部来自变量并先打印；不硬编码 /dev/sdX。
#   - 口令从 .env 读进变量、用 `sudo -S` 走 stdin，**不打印**；
#     ⚠️ `sudo -S` 会吃掉 stdin ⇒ 要写的数据一律走文件，不走管道（第一版栽在这里）。
#   - EXIT 清理：dmsetup remove / mdadm --stop / losetup -d，全用字面量名字。
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

W="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-stripe.XXXXXX")"
DM="singlefs_stripe_probe"
LOOPS=()
cleanup() {
  S dmsetup remove "$DM" 2>/dev/null
  for l in "${LOOPS[@]:-}"; do [ -n "$l" ] && S losetup -d "$l" 2>/dev/null; done
  rm -rf "$W"
}
trap cleanup EXIT

emit() { printf 'E7RESULT %s\n' "$*"; N=$((N+1)); }
N=0
DISK_MB=64
emit "name=config disk_mb=$DISK_MB workdir_kind=tmpdir target=dm-stripe"

# 建 4 个 loop
for i in 0 1 2 3; do
  truncate -s "${DISK_MB}M" "$W/d$i.img"
  L="$(S losetup --find --show "$W/d$i.img")" || { emit "name=fatal reason=losetup_failed"; exit 4; }
  LOOPS+=("$L")
done
emit "name=loops devices=${LOOPS[*]}"

probe_one() {  # $1=devs $2=chunk_bytes
  local devs="$1" chunk="$2" cs=$(( $2 / 512 )) tbl sect ok=0 tot=0 k pred hit i mark
  sect=$(( DISK_MB * 1024 * 1024 / 512 ))
  tbl="0 $(( sect * devs )) striped $devs $cs"
  for ((i=0;i<devs;i++)); do tbl="$tbl ${LOOPS[$i]} 0"; done
  S dmsetup create "$DM" --table "$tbl" 2>/dev/null || { emit "name=skip devs=$devs chunk=$chunk reason=dmsetup_failed"; return; }
  for k in 0 1 2 3 4 5 7 9 13 17 23 31; do
    mark="M${devs}_${chunk}_${k}_$$"
    printf '%s' "$mark" > "$W/mark.bin"
    S dd if="$W/mark.bin" of="/dev/mapper/$DM" bs=512 seek=$(( k * chunk / 512 )) conv=notrunc status=none 2>/dev/null
    S sync
    pred=$(( k % devs )); hit=""
    for ((i=0;i<devs;i++)); do grep -qa "$mark" "$W/d$i.img" 2>/dev/null && hit="$hit$i"; done
    tot=$((tot+1)); [ "$hit" = "$pred" ] && ok=$((ok+1))
    emit "name=map devs=$devs chunk=$chunk k=$k predicted=$pred observed=${hit:-none} match=$([ "$hit" = "$pred" ] && echo 1 || echo 0)"
  done
  emit "name=map_summary devs=$devs chunk=$chunk matched=$ok total=$tot"
  # 素数步长：区域 r 在 r*P*chunk，P 素数且 > devs ⇒ 归属 (r*P)%devs 两两不同
  local P=13 r regions=$devs distinct
  local -a seen=()
  for ((r=0;r<regions;r++)); do
    mark="R${devs}_${chunk}_${r}_$$"
    printf '%s' "$mark" > "$W/mark.bin"
    S dd if="$W/mark.bin" of="/dev/mapper/$DM" bs=512 seek=$(( r * P * chunk / 512 )) conv=notrunc status=none 2>/dev/null
    S sync
    hit=""
    for ((i=0;i<devs;i++)); do grep -qa "$mark" "$W/d$i.img" 2>/dev/null && hit="$hit$i"; done
    seen+=("$hit")
    emit "name=prime_stride devs=$devs chunk=$chunk region=$r prime=$P predicted=$(( (r * P) % devs )) observed=${hit:-none}"
  done
  distinct=$(printf '%s\n' "${seen[@]}" | sort -u | wc -l)
  emit "name=prime_summary devs=$devs chunk=$chunk regions=$regions distinct_devices=$distinct all_distinct=$([ "$distinct" -eq "$regions" ] && echo 1 || echo 0)"
  # 阳性对照：非素数步长（8 = 2^3）在 devs=4 上必须碰撞
  if [ "$devs" -eq 4 ]; then
    local -a seen2=()
    for ((r=0;r<4;r++)); do
      mark="C${chunk}_${r}_$$"
      printf '%s' "$mark" > "$W/mark.bin"
      S dd if="$W/mark.bin" of="/dev/mapper/$DM" bs=512 seek=$(( r * 8 * chunk / 512 )) conv=notrunc status=none 2>/dev/null
      S sync
      hit=""
      for ((i=0;i<4;i++)); do grep -qa "$mark" "$W/d$i.img" 2>/dev/null && hit="$hit$i"; done
      seen2+=("$hit")
    done
    distinct=$(printf '%s\n' "${seen2[@]}" | sort -u | wc -l)
    emit "name=poscontrol_composite_stride chunk=$chunk stride=8 distinct_devices=$distinct expect=1 collides=$([ "$distinct" -eq 1 ] && echo 1 || echo 0)"
  fi
  S dmsetup remove "$DM" 2>/dev/null
}

for devs in 2 3 4; do
  for chunk in 65536 524288; do
    probe_one "$devs" "$chunk"
  done
done
emit "name=done emitted=$((N+1))"
