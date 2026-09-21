#!/usr/bin/env bash
# 把 kb 里承重的外部逐行引用做成可重跑的检查。
#
# **它要拦的是「证据蒸发」**：kb 里一批引用曾标着「本机核实」，
# 而 2026-08-29 的 find 发现那批 PDF 已经不在本机了（checks-owed.md C38）。
# 源码这一侧同样会蒸发——路径变了、版本升了、树被删了，
# 而 kb 里的结论仍旧标着「已核实」。
#
# **判据一**：每一条断言要么命中，要么判红。**取不到源码本身也判红**，
# 不许因为「树不在」就静默跳过——那正是 C38 的形状。
#
# **判据二（checks-owed.md C44）**：每条断言还要说清它核的是哪棵树。
# 一部分引用核的是 fetch-refs.sh 按 URL + sha256 固定下来的那份拷贝，
# 另一部分核的是由环境变量默认值指到的本机内核树——后者不在固定点里，
# 随时会被升级、改动或删掉，而两类此前混在同一个绿色汇总里，读的人分不出来。
# ⇒ 树名写在每条断言的**第二格**，路径相对那棵树的根写；树名认不出就判红；
# 末尾按树分组报数，非固定点的那几类单独标出来。
#
# 树名与实际读到的路径出自同一格，分叉不了：ck / ckdoc 的路径由树根拼出来，
# ckn 的命令只能拿 %ROOT% 指那棵树的根，命令里出现别的树的根照样判红。
# 这就是为什么没有第二张「哪条断言属于哪棵树」的表——两处手抄会分叉。
#
# 复跑：bash research/scripts/verify-citations.sh
# 自证：bash research/scripts/verify-citations.sh --selftest（合成两棵假树，不碰本机的真树）
# 源码固定点由环境变量覆盖：FS_REFS=/path bash research/scripts/verify-citations.sh
set -uo pipefail
SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
REFS="${FS_REFS:-/home/fy5090/code/fs-refs}"
KERN="${KERNEL_TREE:-/home/fy5090/kbuild/linux-om}"

# ── 树登记表：一行一棵树，树名 | 根路径 | 固定点还是非固定点 ──────────────
# 固定点＝research/scripts/fetch-refs.sh 按 URL + sha256 取回、放在 $FS_REFS 底下的那一份。
# 非固定点＝由环境变量默认值指到的本机树：它随时会被别的活动升级、改动或删掉，
# 所以它上面的「已核实」只在这台机器的今天成立。
# 这张表登记的是**树**，不是断言——断言归哪棵树由它自己那一格说了算。
TREE_LIST=$(cat <<TSV
refs-linux|$REFS/linux-6.17|固定点
refs-zfs|$REFS/zfs|固定点
refs-docs|$REFS/docs|固定点
host-kernel|$KERN|非固定点
TSV
)
declare -A TREE_ROOT=() TREE_PINNED=() TREE_PASS=() TREE_FAIL=()
TREE_ORDER=()
while IFS='|' read -r tree_name tree_root tree_pinned; do
  [[ -z "$tree_name" ]] && continue
  TREE_ROOT["$tree_name"]="$tree_root"
  TREE_PINNED["$tree_name"]="$tree_pinned"
  TREE_PASS["$tree_name"]=0
  TREE_FAIL["$tree_name"]=0
  TREE_ORDER+=("$tree_name")
done <<<"$TREE_LIST"

# 两个根都来自环境变量，一棵树的根落进另一棵底下时，「这条读的是哪棵树」就分不出来了。
for tree_name in "${TREE_ORDER[@]}"; do
  for other_tree in "${TREE_ORDER[@]}"; do
    [[ "$tree_name" == "$other_tree" ]] && continue
    if [[ "${TREE_ROOT[$other_tree]}" == "${TREE_ROOT[$tree_name]}" || "${TREE_ROOT[$other_tree]}" == "${TREE_ROOT[$tree_name]}"/* ]]; then
      echo "  ✗ 树登记表坏了：$other_tree 的根落在 $tree_name 的根底下，一条断言读的是哪棵树就分不出来了"
      echo "  → 怎么办：改 FS_REFS / KERNEL_TREE，让每棵树的根互不包含；登记表在本脚本的 TREE_LIST。"
      exit 2
    fi
  done
done

pass=0; fail=0; unmarked=0; tally_dropped=0

score() { # score <树名> <pass 或 fail> —— 分组计数与总计在同一次调用里一起加，两个数出自同一份数据
  local tree="$1" outcome="$2"
  if [[ "$outcome" == pass ]]; then
    pass=$((pass + 1)); TREE_PASS["$tree"]=$(( ${TREE_PASS[$tree]} + 1 ))
  else
    fail=$((fail + 1)); TREE_FAIL["$tree"]=$(( ${TREE_FAIL[$tree]} + 1 ))
  fi
  # 只供自证使用的开关（.claude/rules/fs-design.md「每条分支必须能被测试强制进入」）：
  # 漏记**一格**分组计数，末尾那道「分组报数与被扫集合对得上」必须由绿转红。
  if [[ "${VERIFY_CITATIONS_BREAK_TALLY:-}" == "1" && "$tally_dropped" == 0 && "$outcome" == pass ]]; then
    TREE_PASS["$tree"]=$(( ${TREE_PASS[$tree]} - 1 )); tally_dropped=1
  fi
}

arity_red() { # arity_red <函数名> <要几格> <实际收到的那几格…> —— 格数不对就是没标树来源，判红
  local fn="$1" want="$2"; shift 2
  echo "  ✗ $fn 的格数不对：要 $want 格（决策、树来源、说的是什么、树内路径…），实际 $# 格 ⇒ 多半是没标树来源：$*"  # gate-lint:detail
  fail=$((fail + 1)); unmarked=$((unmarked + 1))
}

mark_tree() { # mark_tree <树名> <决策> <说的是什么> —— 认得返回 0；认不出当场判红，返回 1
  local tree="$1" d="$2" what="$3"
  [[ -n "${TREE_ROOT[$tree]:-}" ]] && return 0
  printf '  ✗ %-6s %-42s 没标树来源，或标了一棵没登记的树：「%s」\n' "$d" "$what" "$tree"  # gate-lint:detail
  fail=$((fail + 1)); unmarked=$((unmarked + 1)); return 1
}

rel_in_tree() { # rel_in_tree <树名> <决策> <说的是什么> <树内相对路径>
  local tree="$1" d="$2" what="$3" rel="$4"
  [[ "$rel" != /* && "$rel" != *..* ]] && return 0
  printf '  ✗ %-6s %-42s 路径越出标着的那棵树 %s：%s\n' "$d" "$what" "$tree" "$rel"  # gate-lint:detail
  score "$tree" fail; return 1
}

cmd_in_tree() { # cmd_in_tree <树名> <决策> <说的是什么> <已把 %ROOT% 展开过的命令>
  local tree="$1" d="$2" what="$3" cmd="$4" other_tree
  if [[ "$cmd" != *"${TREE_ROOT[$tree]}"* ]]; then
    printf '  ✗ %-6s %-42s 命令没读标着的那棵树 %s（%%ROOT%% 忘了写？）\n' "$d" "$what" "$tree"  # gate-lint:detail
    score "$tree" fail; return 1
  fi
  for other_tree in "${TREE_ORDER[@]}"; do
    [[ "$other_tree" == "$tree" ]] && continue
    if [[ "$cmd" == *"${TREE_ROOT[$other_tree]}"* ]]; then
      printf '  ✗ %-6s %-42s 标的树是 %s，命令里却读了 %s\n' "$d" "$what" "$tree" "$other_tree"  # gate-lint:detail
      score "$tree" fail; return 1
    fi
  done
  return 0
}

ck() { # ck <决策> <树名> <说的是什么> <树内相对路径> <ERE 模式>
  (( $# == 5 )) || { arity_red ck 5 "$@"; return; }
  local d="$1" tree="$2" what="$3" rel="$4" pat="$5" f
  mark_tree "$tree" "$d" "$what" || return
  rel_in_tree "$tree" "$d" "$what" "$rel" || return
  f="${TREE_ROOT[$tree]}/$rel"
  if [[ ! -f "$f" ]]; then
    printf '  ✗ %-6s %-42s 源码不在：%s\n' "$d" "$what" "$f"; score "$tree" fail; return  # gate-lint:detail
  fi
  if grep -qE "$pat" "$f"; then
    printf '  ✓ %-6s %s\n' "$d" "$what"; score "$tree" pass
  else
    printf '  ✗ %-6s %-42s 模式没命中：%s\n' "$d" "$what" "$f"; score "$tree" fail  # gate-lint:detail
  fi
}

ckn() { # ckn <决策> <树名> <说的是什么> <期望数> <实测命令，拿 %ROOT% 指那棵树的根>
  (( $# == 5 )) || { arity_red ckn 5 "$@"; return; }
  local d="$1" tree="$2" what="$3" want="$4" cmd="$5" got
  mark_tree "$tree" "$d" "$what" || return
  cmd="${cmd//%ROOT%/${TREE_ROOT[$tree]}}"
  cmd_in_tree "$tree" "$d" "$what" "$cmd" || return
  got=$(eval "$cmd" 2>/dev/null)
  if [[ "$got" == "$want" ]]; then printf '  ✓ %-6s %s（%s）\n' "$d" "$what" "$got"; score "$tree" pass
  else printf '  ✗ %-6s %-42s 期望 %s，实测 %s\n' "$d" "$what" "$want" "${got:-取不到}"; score "$tree" fail; fi  # gate-lint:detail
}

# ── 外部文献（PDF）──────────────────────────────────────────────────────
# checks-owed.md C38 欠的另一半：源码那一侧 2026-08-29 已还，文献这一侧当时只还了 RFC 8439。
# 取回方式 research/scripts/fetch-refs.sh（含 URL 与 sha256），抽取器 research/scripts/pdf-text.py。
# ⚠️ 抽取器只解 FlateDecode + 单字节字体。抽出乱码要当**抽取失败**处理，不许当成「原文没这句」——
#    ZFS On-Disk Specification 就是这种（CID 字体），所以它不在那张表里，见 fetch-refs.sh 的说明。
PDFTXT_CACHE="${TMPDIR:-/tmp}/singlefs-pdftext-$(id -u)"
mkdir -p "$PDFTXT_CACHE"
pdftxt() { # pdftxt <树名> <文献名> —— 抽一次缓存一次，打印缓存路径；抽不出就打印空
  local tree="$1" name="$2" pdf="" out=""
  pdf="${TREE_ROOT[$tree]}/$name"; out="$PDFTXT_CACHE/${name%.pdf}.txt"
  [[ -f "$pdf" ]] || return 1
  if [[ ! -s "$out" || "$pdf" -nt "$out" ]]; then
    python3 "$(dirname "$SELF")/pdf-text.py" "$pdf" >"$out" 2>/dev/null || return 1
  fi
  printf '%s' "$out"
}
ckdoc() { # ckdoc <决策> <树名> <说的是什么> <文献名> <ERE 模式>
  (( $# == 5 )) || { arity_red ckdoc 5 "$@"; return; }
  local d="$1" tree="$2" what="$3" name="$4" pat="$5" txt="" flat=""
  mark_tree "$tree" "$d" "$what" || return
  rel_in_tree "$tree" "$d" "$what" "$name" || return
  if [[ ! -f "${TREE_ROOT[$tree]}/$name" ]]; then
    printf '  ✗ %-6s %-42s 文献不在本机：%s ⇒ 跑 research/scripts/fetch-refs.sh\n' "$d" "$what" "${TREE_ROOT[$tree]}/$name"  # gate-lint:detail
    score "$tree" fail; return
  fi
  txt="$(pdftxt "$tree" "$name")" || { printf '  ✗ %-6s %-42s 抽取失败：%s\n' "$d" "$what" "$name"; score "$tree" fail; return; }  # gate-lint:detail
  # 原文按栏排版，一句话常被折成多行、还会在断行处加连字符（per-\nformance）⇒
  # 先接行、去掉断行连字符、再把空白压成单空格。**只在这份规范化文本上匹配**，
  # 所以模式里不要指望能对上原文的换行。
  # 末段不许是 `grep -q`（门禁阶段 41）：前段是整篇正文，输出大，最容易吃 SIGPIPE
  # ⇒ 一条**能核实**的引用会被随机读成核不到。先落到变量再判。
  flat=$(tr '\n' ' ' <"$txt" | sed -E 's/([a-z])- ([a-z])/\1\2/g' | tr -s ' ')
  if grep -qE "$pat" <<<"$flat"; then
    printf '  ✓ %-6s %s\n' "$d" "$what"; score "$tree" pass
  else
    printf '  ✗ %-6s %-42s 模式没命中：%s\n' "$d" "$what" "$name"; score "$tree" fail  # gate-lint:detail
  fi
}
ckdocn() { # ckdocn <决策> <树名> <说的是什么> <文献名> <ERE 模式> <期望命中数>
  (( $# == 6 )) || { arity_red ckdocn 6 "$@"; return; }
  local d="$1" tree="$2" what="$3" name="$4" pat="$5" want="$6" txt="" got=""
  mark_tree "$tree" "$d" "$what" || return
  rel_in_tree "$tree" "$d" "$what" "$name" || return
  if [[ ! -f "${TREE_ROOT[$tree]}/$name" ]]; then
    printf '  ✗ %-6s %-42s 文献不在本机：%s\n' "$d" "$what" "${TREE_ROOT[$tree]}/$name"; score "$tree" fail; return  # gate-lint:detail
  fi
  txt="$(pdftxt "$tree" "$name")" || { printf '  ✗ %-6s %-42s 抽取失败：%s\n' "$d" "$what" "$name"; score "$tree" fail; return; }  # gate-lint:detail
  got=$(grep -oiE "$pat" "$txt" | wc -l)
  if [[ "$got" == "$want" ]]; then printf '  ✓ %-6s %s（%s 次）\n' "$d" "$what" "$got"; score "$tree" pass
  else printf '  ✗ %-6s %-42s 期望 %s 次，实测 %s 次\n' "$d" "$what" "$want" "$got"; score "$tree" fail; fi  # gate-lint:detail
}

# ── 断言表：每条的第二格是它核的那棵树，路径相对那棵树的根 ───────────────
builtin_assertions() {
echo "── bcachefs（Linux 6.17）──"
ckn D8  refs-linux  "btree 树数"                 21 "grep -oE '^\	x\\([a-z_]+,[[:space:]]+[0-9]+' <(awk '/^#define BCH_BTREE_IDS\\(\\)/{f=1} f{print}' '%ROOT%/fs/bcachefs/bcachefs_format.h') | awk -F, '{gsub(/[ \t]/,\"\",\$2);print \$2}' | sort -nu | wc -l"
ck  D19 refs-linux  "bch_extent_ptr 是 offset:44/dev:8/gen:8" fs/bcachefs/extents_format.h 'offset:44,.*$|dev:8'
ck  D19 refs-linux  "crc128 的 offset 是 13 位"   fs/bcachefs/extents_format.h 'CRC128_SIZE_MAX[[:space:]]+\(1U << 13\)'
ck  D9  refs-linux  "crc64 的 80 位由 csum_hi:16 + csum_lo 拼出" fs/bcachefs/extents_format.h 'csum_hi:16'
ckn D9  refs-linux  "对 crypto_aead/setauthsize 零命中" 0 "grep -rE 'crypto_aead|setauthsize' '%ROOT%/fs/bcachefs' | wc -l"
ck  D9  refs-linux  "MAC 截断靠 memcpy bch_crc_bytes"  fs/bcachefs/checksum.c 'memcpy\(&ret, digest, bch_crc_bytes\[type\]\)'
ck  D9  refs-linux  "元数据加密时恒用 128 位"      fs/bcachefs/checksum.h 'bch2_meta_checksum_type'
ck  D11 refs-linux  "accounting 树格式级绑 write buffer" fs/bcachefs/bcachefs_format.h 'x\(accounting,[[:space:]]+20,'
ck  D11 refs-linux  "记账更新是 delta"             fs/bcachefs/disk_accounting_format.h 'updates are _deltas_'
ck  D11 refs-linux  "运行时与 GC 重建共用同一行"    fs/bcachefs/disk_accounting.h 'this_cpu_add\(e->v\[gc\]\[i\], a\.v->d\[i\]\)'
ck  D11 refs-linux  "btree_gc 自陈重建侧不幂等"     fs/bcachefs/btree_gc.c 'not idempotant'
ck  D18 refs-linux  "BTREE_NODE_SEQ 判谁覆盖谁"     fs/bcachefs/btree_gc.c 'BTREE_NODE_SEQ\(cur->data\) > BTREE_NODE_SEQ\(prev->data\)'
ck  D22 refs-linux  "btree 指针不带校验和"          fs/bcachefs/extents_format.h "Btree pointers don't carry around checksums"
ck  D12 refs-linux  "bucket 寻址显式带设备"         fs/bcachefs/buckets.h 'div_u64\(s, ca->mi\.bucket_size\)'
ck  D1  refs-linux  "BUCKET_GC_GEN_MAX 是 96"       fs/bcachefs/alloc_background.h 'BUCKET_GC_GEN_MAX[[:space:]]+96U'
echo
echo "── OpenZFS master ──"
ck  D4  refs-zfs    "blkptr_t 是 128 字节"          include/sys/spa.h 'SPA_BLKPTRSHIFT[[:space:]]+7'
ck  D18 refs-zfs    "预留区是 blk_prop2 + blk_pad"  include/sys/spa.h 'uint64_t[[:space:]]+blk_prop2;'
ck  D18 refs-zfs    "BP_GET_REWRITE 用 blk_prop2 最高位" include/sys/spa.h 'BF64_GET\(\(bp\)->blk_prop2, 63, 1\)'
ck  D9  refs-zfs    "密文校验和不论密钥在不在都能查损坏" include/sys/spa.h 'whether or not the$'
ck  D4  refs-zfs    "加密块最多 2 副本"             include/sys/spa.h 'encrypted blocks can only have 2 copies'
ck  D9  refs-zfs    "ZIO_DATA_MAC_LEN 是 16"        include/sys/zio.h 'ZIO_DATA_MAC_LEN[[:space:]]+16'
ck  D5  refs-zfs    "deadlist 并到更新的那一侧"      module/zfs/dsl_destroy.c 'Merge our deadlist into next'
ck  D5  refs-zfs    "分叉点快照不许销毁"            module/zfs/dsl_destroy.c "Can't delete a branch point"
ck  D16 refs-zfs    "TXG_DEFER_SIZE 是 2"           include/sys/txg.h 'TXG_DEFER_SIZE[[:space:]]+2'
ck  D17 refs-zfs    "vdev 后端签名里带 txg"          include/sys/vdev_impl.h 'vdev_asize_func_t\(vdev_t \*vd, uint64_t psize, uint64_t txg\)'
ck  D20 refs-zfs    "ZFS 自陈设备不做单扇区原子覆写"  module/zfs/vdev_label.c 'even though it is required to'
ck  D20 refs-zfs    "ZFS 的 4 个 label"                include/sys/vdev_impl.h '#define[[:space:]]+VDEV_LABELS[[:space:]]+4'
ck  D20 refs-zfs    "ZFS 的 128 KiB uberblock 环"      include/sys/vdev_impl.h '#define[[:space:]]+VDEV_UBERBLOCK_RING[[:space:]]+\(128 << 10\)'
ck  D20 refs-zfs    "ZFS 槽号 = txg % 槽数（轮换）"     module/zfs/vdev_label.c 'ub->ub_txg - \(RRSS_GET_STATE'
ck  D23 refs-zfs    "ZIL 有 zh_claim_txg"           include/sys/zil.h 'zh_claim_txg'
echo
echo "── btrfs（Linux 6.17）──"
ckn D17 refs-linux  "btrfs_is_zoned 的文件数"        22 "grep -rn 'btrfs_is_zoned' '%ROOT%/fs/btrfs' | awk -F: '{print \$1}' | sort -u | wc -l"
ckn D17 refs-linux  "btrfs_is_zoned 的总处数"        93 "grep -rn 'btrfs_is_zoned' '%ROOT%/fs/btrfs' | wc -l"
ckn D17 refs-linux  "其中落在事务日志路径的"          5 "grep -c 'btrfs_is_zoned' '%ROOT%/fs/btrfs/tree-log.c'"
ckn D17 refs-linux  "其中落在 ENOSPC 准入的"          6 "grep -c 'btrfs_is_zoned' '%ROOT%/fs/btrfs/space-info.c'"
echo
echo "── 外部文献（已重新固定的）──"
ck  D9  refs-docs   "RFC 8439 的 Tag truncation MUST NOT" rfc8439.txt 'Tag truncation'
ck  D9  refs-docs   "RFC 8439 的 2\^128 possible tags"    rfc8439.txt '2\^128'
echo
echo "── 本机内核树（$(sed -n 's/^VERSION = //p' "${TREE_ROOT[host-kernel]}/Makefile" 2>/dev/null).$(sed -n 's/^PATCHLEVEL = //p' "${TREE_ROOT[host-kernel]}/Makefile" 2>/dev/null)）──"
ck  D2  host-kernel "io_min 取 phys_bs，与 physical_block_size 不同" drivers/nvme/host/core.c 'lim->io_min = phys_bs;'
ck  D2  host-kernel "physical_block_size 取 min(phys_bs, atomic_bs)" drivers/nvme/host/core.c 'lim->physical_block_size = min\(phys_bs, atomic_bs\)'
ck  D20 host-kernel "原子宽度要 ATOMICS 位且 nawupf 非零" drivers/nvme/host/core.c 'NVME_NS_FEAT_ATOMICS\) && id->nawupf'
ckn D9  host-kernel "MaxInvalids 全内核零命中"        0 "grep -rl 'MaxInvalids' '%ROOT%' 2>/dev/null | wc -l"
ck  D9  host-kernel "dm-integrity 的 number_of_mismatches 只自增不比阈值" drivers/md/dm-integrity.c 'atomic64_inc\(&ic->number_of_mismatches\)'
ck  D9  host-kernel "dm-integrity 的 recalc_sector 可被清零" Documentation/admin-guide/device-mapper/dm-integrity.rst 'set recalc_sector to zero'
ck  D9  host-kernel "fscrypt 明写不防离线篡改"        Documentation/filesystems/fscrypt.rst 'manipulate the filesystem offline'
ck  D17 host-kernel "dm-verity 的信任锚"              Documentation/admin-guide/device-mapper/verity.rst 'no other authenticity'
ck  D22 host-kernel "dm-flakey 造不出撕裂"            Documentation/admin-guide/device-mapper/dm-flakey.rst 'corrupt_bio_byte'
ck  D23 host-kernel "XFS_MIN_LOG_FACTOR 存在"         fs/xfs/libxfs/xfs_log_format.h 'XFS_MIN_LOG_FACTOR'
ck  D23 host-kernel "XLOG_MAX_ICLOGS 存在"            fs/xfs/libxfs/xfs_log_format.h 'XLOG_MAX_ICLOGS'
ck  D23 host-kernel "XFS 的 h_prev_block 是 32 位块号"  fs/xfs/libxfs/xfs_log_format.h '__be32[[:space:]]+h_prev_block'
ck  D23 host-kernel "XFS 用 h_cycle 分辨圈次"          fs/xfs/libxfs/xfs_log_format.h '__be32[[:space:]]+h_cycle;'
ck  D23 host-kernel "XFS 每个基本块带 cycle 戳"        fs/xfs/libxfs/xfs_log_format.h 'h_cycle_data\[XLOG_CYCLE_DATA_SIZE\]'
ck  D23 host-kernel "XFS 恢复靠 cycle 跃变找头"        fs/xfs/xfs_log_recover.c 'first_half_cycle'
ck  D23 host-kernel "jbd2 恢复后号跳过断点"            fs/jbd2/recovery.c 'j_transaction_sequence'
ck  D23 host-kernel "XFS 的事务可跨多条记录"          fs/xfs/libxfs/xfs_log_format.h 'XLOG_CONTINUE_TRANS'
ck  D23 host-kernel "XFS 用 END_TRANS 标事务末条"      fs/xfs/libxfs/xfs_log_format.h 'XLOG_END_TRANS'
ck  D23 host-kernel "XFS 自陈单个 region 装不下就拆"   fs/xfs/libxfs/xfs_log_format.h 'split up into'
ck  D23 host-kernel "jbd2 的事务由多块拼成"            fs/jbd2/commit.c 'jbd2_journal_get_descriptor_buffer'
ck  D23 host-kernel "XFS 的 CRC 覆盖载荷"             fs/xfs/xfs_log.c 'crc32c\(crc, dp, size\)'
ck  D23 host-kernel "jbd2 每个数据块自带校验和"        fs/jbd2/commit.c 'jbd2_block_tag_csum_set'
ck  D23 host-kernel "XFS 默认 iclog 32 KiB"           fs/xfs/libxfs/xfs_log_format.h 'XLOG_BIG_RECORD_BSIZE	\(32\*1024\)'
ck  D23 host-kernel "jbd2 描述块塞满 tag 为止"         fs/jbd2/commit.c 'space_left < tag_bytes'
ck  D23 refs-linux  "bcachefs 的 csum 是 jset 首字段"  fs/bcachefs/bcachefs_format.h 'struct jset \{'
ck  D22 host-kernel "XFS 日志下限 10 MiB / 512 块"     fs/xfs/libxfs/xfs_fs.h 'XFS_MIN_LOG_BYTES[[:space:]]+\(10 \* 1024 \* 1024ULL\)'
ck  D22 host-kernel "XFS 日志上限 2 GiB"               fs/xfs/libxfs/xfs_fs.h 'XFS_MAX_LOG_BYTES'
ck  D22 host-kernel "jbd2 日志下限 1024 块"            include/linux/jbd2.h 'JBD2_MIN_JOURNAL_BLOCKS 1024'
ck  D12 host-kernel "btrfs 的 zoned incompat 位"      include/uapi/linux/btrfs.h 'BTRFS_FEATURE_INCOMPAT_ZONED'
ck  D12 host-kernel "XFS 的 zoned incompat 位"        fs/xfs/libxfs/xfs_format.h 'XFS_SB_FEAT_INCOMPAT_ZONED'
ck  D12 host-kernel "md/raid0 的布局 feature bit"     include/uapi/linux/raid/md_p.h 'MD_FEATURE_RAID0_LAYOUT'
ck  D14 host-kernel "erofs 无条件只读"                fs/erofs/super.c 'SB_RDONLY'
ck  D14 host-kernel "btrfs 的默认内联上限"            fs/btrfs/fs.h 'BTRFS_DEFAULT_MAX_INLINE'
ck  D18 host-kernel "btrfs 用 transid 抓丢写/错向写"   fs/btrfs/disk-io.c 'wrong place'
ck  D18 host-kernel "f2fs 把 checkpoint CRC 塞进 cp_ver" fs/f2fs/node.h 'cur_cp_crc'
ck  D18 host-kernel "ocfs2 的 metaecc"                fs/ocfs2/ocfs2_fs.h 'ocfs2_block_check'
ck  D19 host-kernel "btrfs 的 sys_chunk_array 定长"    include/uapi/linux/btrfs_tree.h 'BTRFS_SYSTEM_CHUNK_ARRAY_SIZE'
ck  D9  host-kernel "ext4 把 uuid 折进 s_csum_seed"    fs/ext4/super.c 's_csum_seed'
ck  D13 host-kernel "iomap 契约里没有事务概念"          include/linux/iomap.h 'iomap_begin'
echo
echo "══ 外部文献逐字 ══"
ckdoc D18 refs-docs "OSTEP §45.6：身份校验抓不到丢失写"  ostep-45-file-integrity.pdf \
      'the old block likely has a matching checksum'
ckdoc D18 refs-docs "NetApp FAST.20：lost write 占 13.54%" netapp-fast20-ssd-reliability.pdf \
      'Lost Writes 13 ?: ?54%'
ckdoc D18 refs-docs "NetApp FAST.20：靠 WAFL 块签名发现"   netapp-fast20-ssd-reliability.pdf \
      'signature, ?which includes attributes ?and ?version number'
ckdoc D9  refs-docs "NaCl §9：一次伪造即可解出 r"          naclcrypto.pdf \
      'by polynomial root-finding, easily determine ClampP'
ckdoc D9  refs-docs "SP 800-38D §5.2.1.2 的五个 t 取值"     nist-sp800-38d.pdf \
      '128, 120, 112, 104, or 96'
ckdoc D9  refs-docs "SP 800-38D 的 shall not（七选一）"     nist-sp800-38d.pdf \
      'shall not support values for t'
ckdocn D9 refs-docs "MaxInvalids 属 38B 不属 38D（38B 侧）"  nist-sp800-38b.pdf 'MaxInvalids' 2
ckdocn D9 refs-docs "MaxInvalids 属 38B 不属 38D（38D 侧）"  nist-sp800-38d.pdf 'MaxInvalids' 0
ckdoc D8  refs-docs "FAST18：全路径索引的 Achilles heel"   betrfs-fast18-fullpath.pdf \
      "Achilles' heel of full-path indexing"
ckdoc D8  refs-docs "FAST18：改名代价从子树大小降到深度"   betrfs-fast18-fullpath.pdf \
      'proportional to the size of the subtree to the depth of the subtree'
ckdocn D8 refs-docs "FAST18 全文无 checksum"               betrfs-fast18-fullpath.pdf 'checksum' 0
ckdoc D10 refs-docs "FAST18：git 负载把 ext4 scan 退化 15 倍" betrfs-fast18-fullpath.pdf \
      'degrade ext4 scan performance by up to 15'
ckdoc D10 refs-docs "FAST17：老化用连续 git checkout 内核树" betrfs-fast17-senescence.pdf \
      'successive git checkouts of the Linux kernel source'
ckdoc D10 refs-docs "FAST17：老化的 BetrFS 胜过别家未老化" betrfs-fast17-senescence.pdf \
      "Other than Btrfs, BetrFS's aged performance is better than the other file systems' unaged performance"
}

# ── 自证 ───────────────────────────────────────────────────────────────
# 合成两棵假树跑整条链路，不碰本机那两棵真树（真树在别人机器上多半不在，
# 自证要能在任何机器上跑）。断言表由 VERIFY_CITATIONS_ASSERTIONS 顶替，
# 那是只供自证使用的开关（.claude/rules/fs-design.md「每条分支必须能被测试强制进入」）。
st_checked=0; st_failed=0
st_case() { # st_case <这一格叫什么> <要的结局：绿 或 红> <实际退出码> <整份输出> <输出里要有的话>
  local title="$1" want="$2" rc="$3" out="$4" needle="$5" ok=1
  st_checked=$((st_checked + 1))
  if [[ "$want" == 绿 ]]; then (( rc == 0 )) || ok=0; else (( rc != 0 )) || ok=0; fi
  [[ "$out" == *"$needle"* ]] || ok=0
  (( ok )) && return 0
  echo "  ✗ 自证第 $st_checked 格「$title」没过：要判$want、实测退出码 $rc；要在输出里看到「$needle」"  # gate-lint:detail
  printf '%s\n' "$out" | sed 's/^/      /'  # gate-lint:detail
  st_failed=$((st_failed + 1))
}
st_run() { # st_run <合成树目录> <断言表文件> [额外的环境赋值…] —— 打印整份输出，退出码转手
  local work="$1" assertions="$2"; shift 2
  env FS_REFS="$work/refs" KERNEL_TREE="$work/kern" VERIFY_CITATIONS_ASSERTIONS="$assertions" "$@" bash "$SELF" 2>&1
}
selftest() {
  local work="" out="" rc=0
  work="$(mktemp -d "${TMPDIR:-/tmp}/verify-citations-selftest-XXXXXX")"
  mkdir -p "$work/refs/linux-6.17/fs/bcachefs" "$work/refs/zfs/include/sys" "$work/refs/docs" "$work/kern/fs/xfs"
  printf '#define BUCKET_GC_GEN_MAX\t96U\n' >"$work/refs/linux-6.17/fs/bcachefs/alloc_background.h"
  printf '#define SPA_BLKPTRSHIFT\t7\n'     >"$work/refs/zfs/include/sys/spa.h"
  printf '#define XLOG_MAX_ICLOGS\t8\n'     >"$work/kern/fs/xfs/xfs_log_format.h"

  cat >"$work/green.sh" <<'GREEN_EOF'
ck  D1  refs-linux  "合成：固定点树上的一条"   fs/bcachefs/alloc_background.h 'BUCKET_GC_GEN_MAX'
ckn D1  refs-linux  "合成：固定点树上的计数"   1 "grep -c 'BUCKET_GC_GEN_MAX' '%ROOT%/fs/bcachefs/alloc_background.h'"
ck  D4  refs-zfs    "合成：另一棵固定点树"     include/sys/spa.h 'SPA_BLKPTRSHIFT'
ck  D23 host-kernel "合成：本机内核树上的一条" fs/xfs/xfs_log_format.h 'XLOG_MAX_ICLOGS'
GREEN_EOF
  cat >"$work/unmarked.sh" <<'UNMARKED_EOF'
ck  D1  refs-linux  "合成：固定点树上的一条"   fs/bcachefs/alloc_background.h 'BUCKET_GC_GEN_MAX'
ck  D23 "合成：依赖本机内核树却忘了标树来源"   fs/xfs/xfs_log_format.h 'XLOG_MAX_ICLOGS'
UNMARKED_EOF
  cat >"$work/unknown-tree.sh" <<'UNKNOWN_EOF'
ck  D23 没登记过的树名 "合成：标了一棵没登记的树" fs/xfs/xfs_log_format.h 'XLOG_MAX_ICLOGS'
UNKNOWN_EOF
  cat >"$work/missing-file.sh" <<'MISSING_EOF'
ck  D23 host-kernel "合成：源码不在" fs/xfs/根本没有这个文件.h 'XLOG_MAX_ICLOGS'
MISSING_EOF
  cat >"$work/missing-pattern.sh" <<'PATTERN_EOF'
ck  D23 host-kernel "合成：模式不命中" fs/xfs/xfs_log_format.h 'XLOG_这个串不在文件里'
PATTERN_EOF
  cat >"$work/cross-tree.sh" <<CROSS_EOF
ckn D1  refs-linux  "合成：标的是一棵，命令读了另一棵" 1 "grep -c 'BUCKET_GC_GEN_MAX' '%ROOT%/fs/bcachefs/alloc_background.h' '$work/kern/fs/xfs/xfs_log_format.h'"
CROSS_EOF
  cat >"$work/no-root.sh" <<'NOROOT_EOF'
ckn D1  refs-linux  "合成：命令里忘了写 %ROOT%" 1 "printf '1\n'"
NOROOT_EOF

  # ① ~ ③ 基线：全命中判绿，而且分组报数逐棵树对得上（数是现算的，一条断言都不许漏进漏出）
  out="$(st_run "$work" "$work/green.sh")"; rc=$?
  st_case "基线：固定点树那一组的数"   绿 "$rc" "$out" 'refs-linux（固定点） 2 条：命中 2，未命中 0'
  st_case "基线：非固定点树那一组的数" 绿 "$rc" "$out" 'host-kernel（非固定点） 1 条：命中 1，未命中 0'
  st_case "基线：一条断言都没有的树也要列出来" 绿 "$rc" "$out" 'refs-docs（固定点） 0 条：命中 0，未命中 0'
  # ④ ⑤ ⑥ C44 逐字要的那一格：依赖非固定点树的引用没标树来源 ⇒ 判红。
  # 三格分开钉：那一行自己判红（两种缺法各一格），以及末尾那道 C44 汇总闸自己。
  # 合成一格是不够的——缺标注的那条同时也被当成一条未命中，光看退出码分不出是哪道闸红的。
  out="$(st_run "$work" "$work/unmarked.sh")"; rc=$?
  st_case "旧写法少一格，那一行判红" 红 "$rc" "$out" '多半是没标树来源'
  st_case "C44 汇总闸自己判红"       红 "$rc" "$out" '没标树来源（checks-owed.md C44 要它必须标）'
  out="$(st_run "$work" "$work/unknown-tree.sh")"; rc=$?
  st_case "标了一棵没登记的树，那一行判红" 红 "$rc" "$out" '没标树来源，或标了一棵没登记的树'
  # ⑦ 分组报数与被扫集合对不上 ⇒ 判红（开关只漏记一格，钉的是一个 1 的差）
  out="$(st_run "$work" "$work/green.sh" VERIFY_CITATIONS_BREAK_TALLY=1)"; rc=$?
  st_case "分组报数算错判红" 红 "$rc" "$out" '分组报数与被扫集合对不上'
  # ⑧ ⑨ 原有的两条判据一条都没放松
  out="$(st_run "$work" "$work/missing-file.sh")"; rc=$?
  st_case "文件不在仍判红" 红 "$rc" "$out" '源码不在'
  out="$(st_run "$work" "$work/missing-pattern.sh")"; rc=$?
  st_case "模式不命中仍判红" 红 "$rc" "$out" '模式没命中'
  # ⑩ ⑪ 标注与命令实际读的树不许分叉：读了别的树判红，一棵都没读也判红
  out="$(st_run "$work" "$work/cross-tree.sh")"; rc=$?
  st_case "命令读了别的树判红" 红 "$rc" "$out" '命令里却读了 host-kernel'
  out="$(st_run "$work" "$work/no-root.sh")"; rc=$?
  st_case "命令一棵树都没读判红" 红 "$rc" "$out" '命令没读标着的那棵树'

  rm -rf "${work:?}"
  if (( st_failed )); then
    echo "    → 怎么办：上面逐格列出了要什么、实测什么。分组报数那几格看本脚本末尾那道"
    echo "      「分组报数与被扫集合对不上」；树来源那几格看 mark_tree 与 cmd_in_tree。"
    exit 1
  fi
  echo "  ✓ 引文复核自证通过（查了 $st_checked 格：分组报数三格、缺树来源标注三格、分组报数算错、源码不在、模式没命中、命令跨树、命令没读标着的树）"
  exit 0
}
if [[ "${1:-}" == "--selftest" ]]; then selftest; fi

# ── 跑 ─────────────────────────────────────────────────────────────────
echo "══ 外部引用复核 ══"
for tree_name in "${TREE_ORDER[@]}"; do
  printf '  %s（%s）根：%s\n' "$tree_name" "${TREE_PINNED[$tree_name]}" "${TREE_ROOT[$tree_name]}"
done
echo
if [[ -n "${VERIFY_CITATIONS_ASSERTIONS:-}" ]]; then
  # 只供自证使用：拿一份合成的断言表顶替内置那一份
  # shellcheck source=/dev/null
  source "$VERIFY_CITATIONS_ASSERTIONS"
else
  builtin_assertions
fi

echo
echo "══ 按每条引用依赖的那棵树分组 ══"
tallied=0
summary=""
for tree_name in "${TREE_ORDER[@]}"; do
  tree_total=$(( ${TREE_PASS[$tree_name]} + ${TREE_FAIL[$tree_name]} ))
  tallied=$(( tallied + tree_total ))
  summary="$summary $tree_name $tree_total"
  printf '  %s（%s） %d 条：命中 %d，未命中 %d  根：%s\n' \
    "$tree_name" "${TREE_PINNED[$tree_name]}" "$tree_total" "${TREE_PASS[$tree_name]}" "${TREE_FAIL[$tree_name]}" "${TREE_ROOT[$tree_name]}"
done
echo "  没标树来源：$unmarked 条"
for tree_name in "${TREE_ORDER[@]}"; do
  [[ "${TREE_PINNED[$tree_name]}" == 非固定点 ]] || continue
  tree_total=$(( ${TREE_PASS[$tree_name]} + ${TREE_FAIL[$tree_name]} ))
  echo "  ⚠️ $tree_name 上的 $tree_total 条核的不是固定点：${TREE_ROOT[$tree_name]} 不在 fetch-refs.sh 的固定点里，"
  echo "     随时会被升级、改动或删掉 ⇒ 这一批的「已核实」只在这台机器的今天成立。"
done

if (( tallied + unmarked != pass + fail )); then
  echo "  ✗ 分组报数与被扫集合对不上：分组合计 $tallied 条 + 没标树来源 $unmarked 条 ≠ 断言总数 $(( pass + fail )) 条"
  echo "  → 怎么办：分组计数与总计由 score() 在同一次调用里一起加，对不上说明有一条路径绕过了 score()。"
  echo "    逐个看 ck / ckn / ckdoc / ckdocn 里每一处 return 之前记没记分。"
  exit 1
fi
if (( unmarked )); then
  echo "  ✗ 有 $unmarked 条断言没标树来源（checks-owed.md C44 要它必须标）"
  echo "  → 怎么办：每条断言的第二格写树名，树名登记在本脚本的 TREE_LIST 里（$(printf '%s ' "${TREE_ORDER[@]}")）；"
  echo "    路径相对那棵树的根写，ckn 的命令拿 %ROOT% 指那棵树的根。"
  exit 1
fi
echo
echo "══ 结果：$pass 条命中，$fail 条未命中；分组合计 $tallied 条，与断言总数一致 ══"
if ((fail)); then
  echo "  → 怎么办：未命中不等于 kb 写错了，也可能是源码固定点没了或版本变了。"
  echo "    逐条查：源码在不在（不在就重新固定）、模式还在不在（不在就现查改 kb 并记进 decisions-history）。"
  echo "    ⚠️ 不许因为「树不在」就把这条当跳过——那正是 checks-owed.md C38 要拦的证据蒸发。"
  exit 1
fi
echo "  ✓ 全部命中（$pass 条，按树分组：$summary）"
