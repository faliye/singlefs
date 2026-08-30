#!/usr/bin/env bash
# 复跑已入库的实验，把今天的输出和 research/results/ 里那份逐字节比对。
#
#   bash research/scripts/replay.sh [实验号...]      # 不给参数就全跑
#
# 为什么要有它：kb-discipline.md「所有历史数据都只是参考」——每条实测都绑在当时那个
# 构建上，要拿它支撑新结论，先复跑一遍确认它今天还成立。手工比对会被抄第二遍，所以做成脚本。
#
# 三条完整性闸（command-safety.md「结果抓取要有完整性闸」）：
#   1. 跑前删本轮输出，不许跨轮复用；
#   2. 收尾行必须是 `name=done emitted=N`，且 N 必须等于本轮实际 E7RESULT 行数
#      （口径：emitted 把 config 行与 done 行自己都算进去，2026-08-29 对 8 份入库产物逐份核过）；
#   3. 退出码非 0 一律判红，不许当成「跑过了」。
set -uo pipefail
cd "$(dirname "$0")/.."
OUT_DIR="${REPLAY_OUT:-${TMPDIR:-/tmp}/singlefs-replay-$$}"
mkdir -p "$OUT_DIR"
# E9 要一个真设备/文件当后端（O_DIRECT）。镜像一律落临时目录，不许进仓
# （command-safety.md「测试镜像一律放临时目录」）。
REPLAY_DEV="${REPLAY_DEV:-$OUT_DIR/e9.img}"
# e9-keylayout 用 O_DIRECT 打开这个后端，自己不建文件。tmpfs 不支持 O_DIRECT，
# 所以 REPLAY_OUT 要落在真文件系统上（本机 /tmp 是 ext4，够用）。
[[ -e "$REPLAY_DEV" ]] || truncate -s 512M "$REPLAY_DEV"
export REPLAY_DEV
# E45 要一个 64 MiB 的 O_DIRECT 后端量本机 fsync 率
REPLAY_DEV45="${REPLAY_DEV45:-$OUT_DIR/e45.img}"
[[ -e "$REPLAY_DEV45" ]] || truncate -s 64M "$REPLAY_DEV45"
export REPLAY_DEV45

# 实验号 | 二进制 | 参数 | 入库产物 | 判据（exact=应逐字节一致 / timing=含计时字段，只比结构）
TABLE=$(cat <<'TSV'
E14|e14-discrimination||e14-discrimination-2026-08-29.out|exact
E18|e18-branch||e18-branch-2026-08-28.out|exact
E19|e19-defer||e19-defer-2026-08-28.out|exact
E24|e24_journal_geom||e24-journal-geom-2026-08-29.out|exact
E25|e25_recovery||e25-recovery-2026-08-29.out|exact
E26|e26_journal_reserve||e26-journal-reserve-2026-08-29.out|exact
E27|e27_accounting||e27-accounting-2026-08-29.out|exact
E28|e28_d5_paths||e28-d5-paths-2026-08-29.out|exact
E29|e29_map_rebuild||e29-map-rebuild-2026-08-29.out|exact
E30|e30_blast_radius||e30-blast-radius-2026-08-29.out|exact
E31|e31_range_rebuild||e31-range-rebuild-2026-08-29.out|exact
E32|e32-aad-snapshot||e32-aad-snapshot-2026-08-29.out|exact
E33|e33-journal-timeline||e33-journal-timeline-2026-08-29.out|exact
E34|e34-pin-rules||e34-pin-rules-2026-08-29.out|exact
E36|e36-head-forms||e36-head-forms-2026-08-29.out|exact
E37|e37-slot-mapping||e37-slot-mapping-2026-08-29.out|exact
E38|e38-log-epoch||e38-log-epoch-2026-08-29.out|exact
E39|e39_accounting_cow||e39-accounting-cow-2026-08-29.out|exact
E40|e40_back_chain||e40-back-chain-2026-08-29.out|exact
E43|e43_txn_records||e43-txn-records-2026-08-29.out|exact
E45|e45_jsn_width|$REPLAY_DEV45|e45-jsn-width-2026-08-30.out|timing
E44|e44_ext_budget||e44-ext-budget-2026-08-30.out|exact
E42|e42_root_ring_geom||e42-root-ring-geom-2026-08-30.out|exact
E41|e41_csum_width||e41-csum-width-2026-08-30.out|exact
E47|e47_region_spacing||e47-region-spacing-2026-08-30.out|exact
E48|e48_ring_loss||e48-ring-loss-2026-08-30.out|exact
E49|e49_ring_placement||e49-ring-placement-2026-08-30.out|exact
E8|e8-split||e8-split-2026-08-28.out|exact
E9|@driver_e9||e9-keylayout-2026-08-28.out|exact
E16|e16-journal||e16-journal-2026-08-28.out|exact
E17|e17-merge||e17-merge-2026-08-29-repro.out|timing
E20|e20-fanout||e20-poscontrol-2026-08-29.out|timing
E21|e21-cpu|2048 5|e21-cpu-2026-08-28.out|timing
TSV
)

# 计时字段：换机器、换负载就会变，比对时抹掉。抹掉的是**值**不是**字段名**——
# 字段整个消失属于结构变化，仍然会被抓。
strip_timing() {
  sed -E 's/(per_sec_milli|median_per_sec_milli|spread_bp|min|max|ratio_bp|years_at_sync1|years_at_sync8|sync1_per_sec_milli|nosync_per_sec_milli|elapsed_ns|ns_per_op|ns_per_lookup|lookups_per_s|ns_small|ns_big|ratio|best_ns|t1_ns|t16_ns|mibs|mib_per_s|entries_per_s|gbps|peak_gbps|speedup|threads16_speedup|dev|secs)=[^ ]*/\1=X/g'
}

# ── 结论区间断言 ──────────────────────────────────────────────────────────
# 计时实验复跑不出同样的字节，抹掉计时之后的「结构一致」又几乎什么都不证明——
# 三条臂一起漂到别处，结构照样一致（test-discipline.md「只让多条臂互相比，
# 测不出所有臂一起错」）。所以每个计时实验都要有一条把 kb 里那个数钉住的断言。
# 断言不中**不等于代码坏了**，也可能是 kb 里那个区间该改了——两种都要人来判，所以判红。
claim() { # claim <实验> <说的是什么> <实测值> <下界> <上界>
  local exp="$1" what="$2" got="$3" lo="$4" hi="$5"
  if [[ -z "$got" ]]; then
    printf '  ✗ %-5s %-46s 读不到这个值\n' "$exp" "$what"; return 1
  fi
  if awk -v g="$got" -v l="$lo" -v h="$hi" 'BEGIN{exit !(g>=l && g<=h)}'; then
    printf '  ✓ %-5s %-46s %s（kb 记的区间 %s–%s）\n' "$exp" "$what" "$got" "$lo" "$hi"; return 0
  fi
  printf '  ✗ %-5s %-46s %s 落在 kb 记的 %s–%s 之外\n' "$exp" "$what" "$got" "$lo" "$hi"; return 1
}
fld() { sed -n "s/.*$2=\([0-9.]*\).*/\1/p" "$1" | tail -1; }   # 取某行最后一个匹配字段

check_claims() {
  local exp="$1" f="$2" bad=0 v w x y
  case "$exp" in
  E17)
    # kb 记的是 30.9–31.8M 条目/秒（三轮独立运行）。留 1% 余量给机器状态波动，
    # 超出就是该改 kb 那个区间了。
    v=$(grep 'arm=single' "$f" | sed -n 's/.*entries_per_s=\([0-9]*\).*/\1/p')
    claim E17 "单线程合并吞吐（条目/秒）" "$v" 30591000 32118000 || bad=1
    # 散射是并行度天花板：阳性对照 32 线程必须明显快过合并臂 32 线程，
    # 否则「散射吃掉 55%」这条读数没有判别力。
    w=$(grep 'arm=parallel threads=32' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    x=$(grep 'name=poscontrol threads=32' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    if awk -v a="$x" -v b="$w" 'BEGIN{exit !(a > b*1.5)}'; then
      printf '  ✓ %-5s %-46s 对照 %s× vs 合并臂 %s×\n' E17 "散射吃掉的那一半仍在（阳性对照更快）" "$x" "$w"
    else
      printf '  ✗ %-5s %-46s 对照 %s× 没比合并臂 %s× 快出 1.5 倍\n' E17 "阳性对照失去判别力" "$x" "$w"; bad=1
    fi ;;
  E20)
    # kb 的承重结论：超 L3 那一档 16 KiB 是唯一最小点，且 8 KiB 反常地差（五轮稳定，未解释）。
    for n in 2048 4096 8192 16384 32768 65536; do
      eval "v$n=\$(grep \"name=e20 node_bytes=$n keys=8388608 entry_bytes=40 \" '$f' | sed -n 's/.*ns_per_lookup=\\([0-9.]*\\).*/\\1/p')"
    done
    if awk -v a="$v16384" -v b="$v2048" -v c="$v4096" -v d="$v8192" -v e="$v32768" -v g="$v65536" \
         'BEGIN{exit !(a>0 && a<b && a<c && a<d && a<e && a<g)}'; then
      printf '  ✓ %-5s %-46s 16K=%s ns，其余 %s/%s/%s/%s/%s\n' E20 "超 L3 档 16 KiB 仍是唯一最小点" "$v16384" "$v2048" "$v4096" "$v8192" "$v32768" "$v65536"
    else
      printf '  ✗ %-5s %-46s 16K=%s，2K/4K/8K/32K/64K=%s/%s/%s/%s/%s\n' E20 "16 KiB 不再是最小点 ⇒ kb 那条要改" "$v16384" "$v2048" "$v4096" "$v8192" "$v32768" "$v65536"; bad=1
    fi
    if awk -v d="$v8192" -v c="$v4096" 'BEGIN{exit !(d>c)}'; then
      printf '  ✓ %-5s %-46s 8K=%s > 4K=%s\n' E20 "8 KiB 的未解释拐点又复现一次" "$v8192" "$v4096"
    else
      printf '  ✗ %-5s %-46s 8K=%s ≤ 4K=%s ⇒ kb 记的「五轮稳定」不再成立\n' E20 "8 KiB 拐点这次没出现" "$v8192" "$v4096"; bad=1
    fi ;;
  E45)
    # 本机 fsync 率：换机器会变，但**量级**要稳住，否则寿命折算整个塌掉
    v=$(grep 'name=arm arm=Sync1' "$f" | sed -n 's/.*median_per_sec_milli=\([0-9]*\).*/\1/p')
    claim E45 "本机 fsync 率（每秒千分之一次）" "$v" 500000 20000000 || bad=1
    # 阳性对照：不 fsync 必须至少快一倍，否则 fdatasync 没到设备
    w=$(grep 'name=poscontrol' "$f" | sed -n 's/.*ok=\([a-z]*\).*/\1/p')
    if [[ "$w" == true ]]; then printf '  ✓ %-5s %-46s\n' E45 "阳性对照：fdatasync 确实到了设备"
    else printf '  ✗ %-5s %-46s ok=%s\n' E45 "阳性对照失败 ⇒ fdatasync 没到设备，整轮作废" "$w"; bad=1; fi
    # 48 位计数器在本机速率下的寿命：这是「48 位够不够」那条结论的落点
    x=$(grep 'name=lifetime bits=48' "$f" | sed -n 's/.*years_at_sync1=\([0-9]*\).*/\1/p')
    claim E45 "48 位计数器在本机撑多少年" "$x" 500 50000 || bad=1
    # 加宽 jsn 到 12 字节的代价：0..=100 项里一格都不该多占
    y=$(grep 'name=width unit=512 jsn_bytes=12 ' "$f" | sed -n 's/.*cost_unit_count_0_100=\([0-9]*\).*/\1/p')
    claim E45 "jsn 8→12 在 512 单元下多占几格" "$y" 0 0 || bad=1 ;;
  E21)
    # kb 的承重结论：CPU 扫描撞内存带宽墙（约 65 GB/s），16 线程几乎不加速 ⇒ GPU 传输地板已经更慢。
    v=$(grep 'name=scaling arm=bandwidth' "$f" | sed -n 's/.*peak_gbps=\([0-9.]*\).*/\1/p')
    claim E21 "CPU 扫描峰值带宽（GB/s）" "$v" 55 75 || bad=1
    w=$(grep 'name=scaling arm=bandwidth' "$f" | sed -n 's/.*threads16_speedup=\([0-9.]*\).*/\1/p')
    claim E21 "带宽受限：16 线程几乎不加速" "$w" 1.0 1.4 || bad=1
    # 阳性对照：同样 16 线程，计算受限的那条必须大幅加速，否则「不加速」分不清是带宽墙还是没跑起来
    x=$(grep 'name=poscontrol arm=compute' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    claim E21 "阳性对照（计算受限）16 线程加速" "$x" 10 20 || bad=1 ;;
  esac
  return $bad
}

# E9 的入库产物是 25 次运行拼起来的（5 种子 × 5 改名档），而这个循环从没被写进 kb。
# 2026-08-29 审计时按产物里的 config 行反推出来，重建结果与入库产物**逐字节一致**。
driver_e9() {
  local r s
  for r in 0 500 2000 5000 20000; do
    for s in 3 7 11 13 17; do
      ./target/release/e9-keylayout "$REPLAY_DEV" "$s" interleave 8 "$r" || return 1
    done
  done
}

ONLY=("$@")
want() { [[ ${#ONLY[@]} -eq 0 ]] && return 0; local e; for e in "${ONLY[@]}"; do [[ "$e" == "$1" ]] && return 0; done; return 1; }

cargo build --release --manifest-path e7-index-bench/Cargo.toml >/dev/null 2>&1 || { echo "replay: 构建失败" >&2; exit 2; }

pass=0; drift=0; timing_only=0; broken=0; claim_bad=0
CLAIM_QUEUE=()
printf '%-5s %-24s %-10s %s\n' 实验 二进制 判定 说明
printf '%s\n' "-------------------------------------------------------------------------"
while IFS='|' read -r exp bin args stored kind; do
  [[ -z "$exp" ]] && continue
  want "$exp" || continue
  args="${args//\$REPLAY_DEV45/$REPLAY_DEV45}"  # 先换长的，否则前缀会被短的吃掉
  args="${args//\$REPLAY_DEV/$REPLAY_DEV}"   # 表里写字面量 $REPLAY_DEV，这里才展开
  fresh="$OUT_DIR/$exp.out"
  rm -f "$fresh"                                    # 闸 1：不许跨轮复用
  if [[ "$bin" == @* ]]; then
    "${bin#@}" >"$fresh" 2>"$OUT_DIR/$exp.err"
  else
    # shellcheck disable=SC2086
    ./target/release/"$bin" $args >"$fresh" 2>"$OUT_DIR/$exp.err"
  fi
  rc=$?
  if [[ $rc -ne 0 ]]; then                          # 闸 3
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "退出码 $rc，见 $OUT_DIR/$exp.err"; broken=$((broken+1)); continue
  fi
  # 闸 2：**逐段**核。产物可能是多次运行拼起来的（E9 就是 25 段），
  # 只看最后一个 name=done 会让前 24 段的缺行全部漏过去。
  gate2=$(awk '/^E7RESULT/{n++}
               /^E7RESULT name=done emitted=/{
                 split($0,a,"emitted="); e=a[2]+0
                 segs++
                 if (e != n) { print "第 " segs " 段收尾行说 " e " 条，实收 " n " 条"; bad=1; exit }
                 n=0
               }
               END{ if (!bad) { if (segs==0) print "没有收尾行 name=done"; else if (n>0) print "最后一段没有收尾行，尾巴 " n " 条" } }' "$fresh")
  if [[ -n "$gate2" ]]; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "$gate2"; broken=$((broken+1)); continue
  fi
  if diff -q "$fresh" "results/$stored" >/dev/null 2>&1; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 字节一致 "$stored"; pass=$((pass+1))
    CLAIM_QUEUE+=("$exp|$fresh"); continue
  fi
  if diff -q <(strip_timing <"$fresh") <(strip_timing <"results/$stored") >/dev/null 2>&1; then
    if [[ "$kind" == timing ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 仅计时不同 "$stored（结构一致，符合声明）"; timing_only=$((timing_only+1))
      CLAIM_QUEUE+=("$exp|$fresh")
    else
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 判据写错 "$stored 声明 exact 却只在抹掉计时后才一致"; drift=$((drift+1))
    fi
    continue
  fi
  n=$(diff <(strip_timing <"$fresh") <(strip_timing <"results/$stored") | grep -c '^[<>]')
  printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 对不上 "$stored，$n 行不同 → diff $fresh results/$stored"
  drift=$((drift+1))
done <<<"$TABLE"

printf '%s\n' "-------------------------------------------------------------------------"
echo "结论区间断言（计时实验复跑不出同样的字节，靠这些把 kb 里的数钉住）："
for q in "${CLAIM_QUEUE[@]}"; do
  check_claims "${q%%|*}" "${q#*|}" || claim_bad=$((claim_bad+1))
done
printf '%s\n' "-------------------------------------------------------------------------"
echo "字节一致 $pass ／ 仅计时不同 $timing_only ／ 对不上 $drift ／ 跑不了 $broken ／ 结论断言不中 $claim_bad"
echo "本轮输出：$OUT_DIR"
if [[ $drift -eq 0 && $broken -eq 0 && $claim_bad -eq 0 ]]; then
  # 全绿且用的是自动分配的临时目录 ⇒ 收拾掉。有一条不绿就留着，上面的提示指着它。
  [[ -z "${REPLAY_OUT:-}" ]] && rm -rf "$OUT_DIR"
  exit 0
fi
exit 1
