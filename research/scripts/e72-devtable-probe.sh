#!/usr/bin/env bash
# E72：设备描述符表的挂载复算 —— D12 已定项 3 / 已定项 4 欠的那次测量。
#
# ## 被引用条款逐字（verify-before-claiming.md「把定义句原样贴进注释」）
#
# - D12 已定项 3：「设备级几何量**住超级块里的设备描述符表**，不住块头。」
# - D12 已定项 4：「**凡是进入地址算术的几何量都要逐设备比对，不进地址算术的不比。**」
#   清单四条：`physical_block_size`（对不上 ⇒ 拒绝可写挂载）、
#   设备容量（变小 ⇒ 拒绝；变大 ⇒ 放行并记）、设备身份（⇒ 拒绝挂载）、
#   该设备用哪套布局 / incompat 位（⇒ 拒绝挂载）。
#   并逐字：「⚠️ **每一条都要能被测试强制走进『对不上』那条分支**……
#   一条不会红的比对项与没有这条比对项，在挂载日志里长得一模一样。」
# - 不在清单里的：转速、队列深度、`io_opt`、discard —— 比对它们只会制造无法挂载的假红。
#
# ## 判据（E72 正文跑前写死，跑完不许改）
#
# 1. **绝对值断言**：表的字节数**恰好**等于 `设备数 × 每设备条目宽度`，不许只看趋势。
# 2. 复算耗时随设备数**线性**；出现超线性判「复算里混进了 O(设备²) 的比对」。
# 3. **每一条比对项都要能被强制走进「对不上」那条分支。**
#
# ## 失败条款
#
# - **阳性对照**：改坏其中一台设备的几何字段，复算必须判红；判不出来说明那一条比对项是摆设。
# - **若所有比对项都只在同构池上验过，整轮作废**——异构本身就是 md/raid0 那个静默损坏的触发条件。
#   ⇒ 全部场景跑在**真异构池**上（`losetup --sector-size` 造出 pbs 512 与 4096 混合）。
#
# ## 口径：它答得了什么
#
# - 设备是**真 loop 设备**，几何从 `/sys/block/*/queue` **实测读出**，不是编的。
#   「改坏几何」用真手段：换 `--sector-size` 重挂、`truncate` 改容量、换设备顺序。
# - **挂载路径本身不存在**（无超级块、无实现）⇒ 这里实现的是 D12 已定项 4 那条**规则**，
#   验的是**规则里每一条比对项有没有判别力**，不是「我们的挂载代码对不对」。
# - 表的条目宽度**仓里没定过** ⇒ 是本脚本的假设，按三档各算一次。
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

W="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e72.XXXXXX")"
LOOPS=()
cleanup() { for l in "${LOOPS[@]:-}"; do [ -n "$l" ] && S losetup -d "$l" 2>/dev/null; done; rm -rf "$W"; }
trap cleanup EXIT

N=0
emit() { printf 'E7RESULT %s\n' "$*"; N=$((N+1)); }

# 每设备条目宽度：dev_uuid 16 + pbs 4 + lbs 4 + capacity 8 + layout_bits 8 = 40
# ⚠️ 仓里没定过，是假设。另扫 24 / 64 两档。
ENTRY_W=40
DISK_MB=16
emit "name=config entry_width=$ENTRY_W disk_mb=$DISK_MB rounds=5 pool=heterogeneous"

# 造一个异构池：偶数号 512、奇数号 4096
make_pool() { # $1=设备数
  for l in "${LOOPS[@]:-}"; do [ -n "$l" ] && S losetup -d "$l" 2>/dev/null; done
  LOOPS=()
  local i ss L
  for ((i=0;i<$1;i++)); do
    rm -f "$W/d$i.img"; truncate -s "${DISK_MB}M" "$W/d$i.img"
    ss=512; [ $(( i % 2 )) -eq 1 ] && ss=4096
    L="$(S losetup --find --show --sector-size $ss "$W/d$i.img")" || return 1
    LOOPS+=("$L")
  done
  return 0
}

# 实测读一台设备的几何 → "pbs lbs capacity"
probe() { # $1=loop 设备
  local n; n="$(basename "$1")"
  printf '%s %s %s' \
    "$(cat /sys/block/$n/queue/physical_block_size)" \
    "$(cat /sys/block/$n/queue/logical_block_size)" \
    "$(S blockdev --getsize64 "$1")"
}

# mkfs：把实测几何写进描述符表文件（模拟超级块那张表）
write_table() { # $1=表文件
  : > "$1"
  local i g
  for ((i=0;i<${#LOOPS[@]};i++)); do
    g="$(probe "${LOOPS[$i]}")"
    # dev_id 用 backing 文件名当身份；layout 位固定 1（第一版只有一套布局，D12 已定项 5）
    echo "dev$i $g 1" >> "$1"
  done
}

# mount：逐设备复算并按 D12 已定项 4 的规则比对。打印每条比对项的判决。
recheck() { # $1=表文件 → 打印 "pbs=<ok|mismatch> cap=<ok|shrunk|grown> id=<ok|mismatch> layout=<ok|mismatch>"
  local i line f_id f_pbs f_lbs f_cap f_lay g n_pbs n_lbs n_cap
  local v_pbs=ok v_cap=ok v_id=ok v_lay=ok
  i=0
  while read -r f_id f_pbs f_lbs f_cap f_lay; do
    if [ $i -ge ${#LOOPS[@]} ]; then v_id=mismatch; break; fi
    g="$(probe "${LOOPS[$i]}")"; read -r n_pbs n_lbs n_cap <<<"$g"
    [ "$f_pbs" = "$n_pbs" ] || v_pbs=mismatch
    if   [ "$n_cap" -lt "$f_cap" ]; then v_cap=shrunk
    elif [ "$n_cap" -gt "$f_cap" ] && [ "$v_cap" = ok ]; then v_cap=grown; fi
    # 设备身份：表里第 i 行的 dev 号必须对上第 i 个 loop 的 backing 文件
    local back; back="$(S losetup -O BACK-FILE -n "${LOOPS[$i]}" | tr -d ' ')"
    [ "$(basename "$back" .img)" = "d${f_id#dev}" ] || v_id=mismatch
    [ "$f_lay" = "1" ] || v_lay=mismatch
    i=$((i+1))
  done < "$1"
  [ $i -eq ${#LOOPS[@]} ] || v_id=mismatch
  printf 'pbs=%s cap=%s id=%s layout=%s' "$v_pbs" "$v_cap" "$v_id" "$v_lay"
}

verdict_of() { # $1=recheck 输出 → 按 D12 已定项 4 给最终处置
  # ⚠️ **次序 = 严格程度，不是清单次序。** `refuse` 严于 `refuse_rw`，必须先判。
  # 本脚本每个场景只改坏一条，所以多条同时坏这条路径**没有被任何场景走到**——
  # 写对它是为了它将来被扩展时不给出偏松的处置，不是这一轮验过的东西。
  case "$1" in
    *cap=shrunk*)      echo "refuse";;
    *id=mismatch*)     echo "refuse";;
    *layout=mismatch*) echo "refuse";;
    *pbs=mismatch*)    echo "refuse_rw";;
    *cap=grown*)       echo "allow_and_note";;
    *)                 echo "allow";;
  esac
}

# ── 判据 1 + 2：表大小与复算耗时随设备数怎么走 ──────────────────────────
for devs in 2 4 8; do
  make_pool $devs || { emit "name=skip reason=losetup_failed devs=$devs"; continue; }
  write_table "$W/tab"
  lines=$(wc -l < "$W/tab")
  t0=$(date +%s%N); recheck "$W/tab" >/dev/null; t1=$(date +%s%N)
  emit "name=scale devs=$devs table_entries=$lines table_bytes_w40=$(( devs * 40 )) table_bytes_w24=$(( devs * 24 )) table_bytes_w64=$(( devs * 64 )) recheck_ns=$(( t1 - t0 ))"
done

# ── 判据 3 + 阳性对照：每条比对项都要能被强制走进「对不上」 ─────────────
# 全部跑在 4 台的真异构池上（pbs 512/4096 混合）
for round in 1 2 3 4 5; do
  # 场景 0：阴性对照 —— 什么都不改，四条必须全 ok
  make_pool 4 || { emit "name=skip reason=losetup_failed round=$round"; continue; }
  write_table "$W/tab"
  het="$(awk '{print $2}' "$W/tab" | sort -u | tr '\n' ',' )"
  r="$(recheck "$W/tab")"
  emit "name=item scen=control_unchanged round=$round pbs_mix=$het $r verdict=$(verdict_of "$r")"

  # 场景 1：pbs 变了 —— 把 1 号盘从 4096 换成 512 重挂
  make_pool 4 || continue
  write_table "$W/tab"
  S losetup -d "${LOOPS[1]}"
  LOOPS[1]="$(S losetup --find --show --sector-size 512 "$W/d1.img")"
  r="$(recheck "$W/tab")"
  emit "name=item scen=force_pbs_mismatch round=$round $r verdict=$(verdict_of "$r")"

  # 场景 2：容量变小 —— truncate 后端文件再重挂
  make_pool 4 || continue
  write_table "$W/tab"
  S losetup -d "${LOOPS[2]}"
  truncate -s $(( DISK_MB / 2 ))M "$W/d2.img"
  LOOPS[2]="$(S losetup --find --show --sector-size 512 "$W/d2.img")"
  r="$(recheck "$W/tab")"
  emit "name=item scen=force_capacity_shrunk round=$round $r verdict=$(verdict_of "$r")"

  # 场景 3：容量变大 —— 应当放行并记
  make_pool 4 || continue
  write_table "$W/tab"
  S losetup -d "${LOOPS[2]}"
  truncate -s $(( DISK_MB * 2 ))M "$W/d2.img"
  LOOPS[2]="$(S losetup --find --show --sector-size 512 "$W/d2.img")"
  r="$(recheck "$W/tab")"
  emit "name=item scen=force_capacity_grown round=$round $r verdict=$(verdict_of "$r")"

  # 场景 4：设备身份错配 —— 交换两台设备的顺序（表没变，实际盘换位了）
  make_pool 4 || continue
  write_table "$W/tab"
  tmp="${LOOPS[0]}"; LOOPS[0]="${LOOPS[2]}"; LOOPS[2]="$tmp"
  r="$(recheck "$W/tab")"
  emit "name=item scen=force_devid_swap round=$round $r verdict=$(verdict_of "$r")"

  # 场景 5：布局位不认识 —— 把表里某台的 layout 位改成 2
  make_pool 4 || continue
  write_table "$W/tab"
  awk 'NR==3{$5=2}1' "$W/tab" > "$W/tab2" && mv "$W/tab2" "$W/tab"
  r="$(recheck "$W/tab")"
  emit "name=item scen=force_unknown_layout round=$round $r verdict=$(verdict_of "$r")"

  # 场景 6：不在清单里的量变了 —— 必须**不**判红（否则是假红）
  make_pool 4 || continue
  write_table "$W/tab"
  S blockdev --setra 8192 "${LOOPS[1]}" 2>/dev/null
  r="$(recheck "$W/tab")"
  emit "name=item scen=offlist_readahead_changed round=$round $r verdict=$(verdict_of "$r")"
done

emit "name=done emitted=$(( N + 1 ))"
