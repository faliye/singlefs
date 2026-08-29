#!/usr/bin/env bash
# 把 kb 里承重的外部逐行引用做成可重跑的检查。
#
# **它要拦的是「证据蒸发」**：kb 里一批引用曾标着「本机核实」，
# 而 2026-08-29 的 find 发现那批 PDF 已经不在本机了（checks-owed.md C38）。
# 源码这一侧同样会蒸发——路径变了、版本升了、树被删了，
# 而 kb 里的结论仍旧标着「已核实」。
#
# **判据**：每一条断言要么命中，要么判红。**取不到源码本身也判红**，
# 不许因为「树不在」就静默跳过——那正是 C38 的形状。
#
# 复跑：bash research/scripts/verify-citations.sh
# 源码固定点由环境变量覆盖：FS_REFS=/path bash research/scripts/verify-citations.sh
set -uo pipefail
REFS="${FS_REFS:-/home/fy5090/code/fs-refs}"
BCH="$REFS/linux-6.17/fs/bcachefs"
BTR="$REFS/linux-6.17/fs/btrfs"
ZFS="$REFS/zfs"
DOCS="$REFS/docs"
KERN="${KERNEL_TREE:-/home/fy5090/kbuild/linux-om}"

pass=0; fail=0
ck() { # ck <决策> <说的是什么> <文件> <ERE 模式>
  local d="$1" what="$2" f="$3" pat="$4"
  if [[ ! -f "$f" ]]; then
    printf '  ✗ %-6s %-42s 源码不在：%s\n' "$d" "$what" "$f"; fail=$((fail+1)); return
  fi
  if grep -qE "$pat" "$f"; then
    printf '  ✓ %-6s %s\n' "$d" "$what"; pass=$((pass+1))
  else
    printf '  ✗ %-6s %-42s 模式没命中：%s\n' "$d" "$what" "$f"; fail=$((fail+1))
  fi
}
ckn() { # ckn <决策> <说的是什么> <期望数> <实测命令>
  local d="$1" what="$2" want="$3"; shift 3
  local got; got=$(eval "$@" 2>/dev/null)
  if [[ "$got" == "$want" ]]; then printf '  ✓ %-6s %s（%s）\n' "$d" "$what" "$got"; pass=$((pass+1))
  else printf '  ✗ %-6s %-42s 期望 %s，实测 %s\n' "$d" "$what" "$want" "${got:-取不到}"; fail=$((fail+1)); fi
}

echo "══ 外部引用复核 ══"
echo "  源码固定点：$REFS"
echo
echo "── bcachefs（Linux 6.17）──"
ckn D8  "btree 树数"                 21 "grep -oE '^\	x\\([a-z_]+,[[:space:]]+[0-9]+' <(awk '/^#define BCH_BTREE_IDS\\(\\)/{f=1} f{print}' '$BCH/bcachefs_format.h') | awk -F, '{gsub(/[ \t]/,\"\",\$2);print \$2}' | sort -nu | wc -l"
ck  D19 "bch_extent_ptr 是 offset:44/dev:8/gen:8" "$BCH/extents_format.h" 'offset:44,.*$|dev:8'
ck  D19 "crc128 的 offset 是 13 位"   "$BCH/extents_format.h" 'CRC128_SIZE_MAX[[:space:]]+\(1U << 13\)'
ck  D9  "crc64 的 80 位由 csum_hi:16 + csum_lo 拼出" "$BCH/extents_format.h" 'csum_hi:16'
ckn D9  "对 crypto_aead/setauthsize 零命中" 0 "grep -rE 'crypto_aead|setauthsize' '$BCH' | wc -l"
ck  D9  "MAC 截断靠 memcpy bch_crc_bytes"  "$BCH/checksum.c" 'memcpy\(&ret, digest, bch_crc_bytes\[type\]\)'
ck  D9  "元数据加密时恒用 128 位"      "$BCH/checksum.h" 'bch2_meta_checksum_type'
ck  D11 "accounting 树格式级绑 write buffer" "$BCH/bcachefs_format.h" 'x\(accounting,[[:space:]]+20,'
ck  D11 "记账更新是 delta"             "$BCH/disk_accounting_format.h" 'updates are _deltas_'
ck  D11 "运行时与 GC 重建共用同一行"    "$BCH/disk_accounting.h" 'this_cpu_add\(e->v\[gc\]\[i\], a\.v->d\[i\]\)'
ck  D11 "btree_gc 自陈重建侧不幂等"     "$BCH/btree_gc.c" 'not idempotant'
ck  D18 "BTREE_NODE_SEQ 判谁覆盖谁"     "$BCH/btree_gc.c" 'BTREE_NODE_SEQ\(cur->data\) > BTREE_NODE_SEQ\(prev->data\)'
ck  D22 "btree 指针不带校验和"          "$BCH/extents_format.h" "Btree pointers don't carry around checksums"
ck  D12 "bucket 寻址显式带设备"         "$BCH/buckets.h" 'div_u64\(s, ca->mi\.bucket_size\)'
ck  D1  "BUCKET_GC_GEN_MAX 是 96"       "$BCH/alloc_background.h" 'BUCKET_GC_GEN_MAX[[:space:]]+96U'
echo
echo "── OpenZFS master ──"
ck  D4  "blkptr_t 是 128 字节"          "$ZFS/include/sys/spa.h" 'SPA_BLKPTRSHIFT[[:space:]]+7'
ck  D18 "预留区是 blk_prop2 + blk_pad"  "$ZFS/include/sys/spa.h" 'uint64_t[[:space:]]+blk_prop2;'
ck  D18 "BP_GET_REWRITE 用 blk_prop2 最高位" "$ZFS/include/sys/spa.h" 'BF64_GET\(\(bp\)->blk_prop2, 63, 1\)'
ck  D9  "密文校验和不论密钥在不在都能查损坏" "$ZFS/include/sys/spa.h" 'whether or not the$'
ck  D4  "加密块最多 2 副本"             "$ZFS/include/sys/spa.h" 'encrypted blocks can only have 2 copies'
ck  D9  "ZIO_DATA_MAC_LEN 是 16"        "$ZFS/include/sys/zio.h" 'ZIO_DATA_MAC_LEN[[:space:]]+16'
ck  D5  "deadlist 并到更新的那一侧"      "$ZFS/module/zfs/dsl_destroy.c" 'Merge our deadlist into next'
ck  D5  "分叉点快照不许销毁"            "$ZFS/module/zfs/dsl_destroy.c" "Can't delete a branch point"
ck  D16 "TXG_DEFER_SIZE 是 2"           "$ZFS/include/sys/txg.h" 'TXG_DEFER_SIZE[[:space:]]+2'
ck  D17 "vdev 后端签名里带 txg"          "$ZFS/include/sys/vdev_impl.h" 'vdev_asize_func_t\(vdev_t \*vd, uint64_t psize, uint64_t txg\)'
ck  D20 "ZFS 自陈设备不做单扇区原子覆写"  "$ZFS/module/zfs/vdev_label.c" 'even though it is required to'
ck  D23 "ZIL 有 zh_claim_txg"           "$ZFS/include/sys/zil.h" 'zh_claim_txg'
echo
echo "── btrfs（Linux 6.17）──"
ckn D17 "btrfs_is_zoned 的文件数"        22 "grep -rn 'btrfs_is_zoned' '$BTR' | awk -F: '{print \$1}' | sort -u | wc -l"
ckn D17 "btrfs_is_zoned 的总处数"        93 "grep -rn 'btrfs_is_zoned' '$BTR' | wc -l"
ckn D17 "其中落在事务日志路径的"          5 "grep -c 'btrfs_is_zoned' '$BTR/tree-log.c'"
ckn D17 "其中落在 ENOSPC 准入的"          6 "grep -c 'btrfs_is_zoned' '$BTR/space-info.c'"
echo
echo "── 外部文献（已重新固定的）──"
ck  D9  "RFC 8439 的 Tag truncation MUST NOT" "$DOCS/rfc8439.txt" 'Tag truncation'
ck  D9  "RFC 8439 的 2\^128 possible tags"    "$DOCS/rfc8439.txt" '2\^128'
echo
echo "── 本机内核树（$(sed -n 's/^VERSION = //p' "$KERN/Makefile" 2>/dev/null).$(sed -n 's/^PATCHLEVEL = //p' "$KERN/Makefile" 2>/dev/null)）──"
ck  D2  "io_min 取 phys_bs，与 physical_block_size 不同" "$KERN/drivers/nvme/host/core.c" 'lim->io_min = phys_bs;'
ck  D2  "physical_block_size 取 min(phys_bs, atomic_bs)" "$KERN/drivers/nvme/host/core.c" 'lim->physical_block_size = min\(phys_bs, atomic_bs\)'
ck  D20 "原子宽度要 ATOMICS 位且 nawupf 非零" "$KERN/drivers/nvme/host/core.c" 'NVME_NS_FEAT_ATOMICS\) && id->nawupf'
ckn D9  "MaxInvalids 全内核零命中"        0 "grep -rl 'MaxInvalids' '$KERN' 2>/dev/null | wc -l"
ck  D9  "dm-integrity 的 number_of_mismatches 只自增不比阈值" "$KERN/drivers/md/dm-integrity.c" 'atomic64_inc\(&ic->number_of_mismatches\)'
ck  D9  "dm-integrity 的 recalc_sector 可被清零" "$KERN/Documentation/admin-guide/device-mapper/dm-integrity.rst" 'set recalc_sector to zero'
ck  D9  "fscrypt 明写不防离线篡改"        "$KERN/Documentation/filesystems/fscrypt.rst" 'manipulate the filesystem offline'
ck  D17 "dm-verity 的信任锚"              "$KERN/Documentation/admin-guide/device-mapper/verity.rst" 'no other authenticity'
ck  D22 "dm-flakey 造不出撕裂"            "$KERN/Documentation/admin-guide/device-mapper/dm-flakey.rst" 'corrupt_bio_byte'
ck  D23 "XFS_MIN_LOG_FACTOR 存在"         "$KERN/fs/xfs/libxfs/xfs_log_format.h" 'XFS_MIN_LOG_FACTOR'
ck  D23 "XLOG_MAX_ICLOGS 存在"            "$KERN/fs/xfs/libxfs/xfs_log_format.h" 'XLOG_MAX_ICLOGS'
ck  D12 "btrfs 的 zoned incompat 位"      "$KERN/include/uapi/linux/btrfs.h" 'BTRFS_FEATURE_INCOMPAT_ZONED'
ck  D12 "XFS 的 zoned incompat 位"        "$KERN/fs/xfs/libxfs/xfs_format.h" 'XFS_SB_FEAT_INCOMPAT_ZONED'
ck  D12 "md/raid0 的布局 feature bit"     "$KERN/include/uapi/linux/raid/md_p.h" 'MD_FEATURE_RAID0_LAYOUT'
ck  D14 "erofs 无条件只读"                "$KERN/fs/erofs/super.c" 'SB_RDONLY'
ck  D14 "btrfs 的默认内联上限"            "$KERN/fs/btrfs/fs.h" 'BTRFS_DEFAULT_MAX_INLINE'
ck  D18 "btrfs 用 transid 抓丢写/错向写"   "$KERN/fs/btrfs/disk-io.c" 'wrong place'
ck  D18 "f2fs 把 checkpoint CRC 塞进 cp_ver" "$KERN/fs/f2fs/node.h" 'cur_cp_crc'
ck  D18 "ocfs2 的 metaecc"                "$KERN/fs/ocfs2/ocfs2_fs.h" 'ocfs2_block_check'
ck  D19 "btrfs 的 sys_chunk_array 定长"    "$KERN/include/uapi/linux/btrfs_tree.h" 'BTRFS_SYSTEM_CHUNK_ARRAY_SIZE'
ck  D9  "ext4 把 uuid 折进 s_csum_seed"    "$KERN/fs/ext4/super.c" 's_csum_seed'
ck  D13 "iomap 契约里没有事务概念"          "$KERN/include/linux/iomap.h" 'iomap_begin'

echo
echo "══ 结果：$pass 条命中，$fail 条未命中 ══"
if ((fail)); then
  echo "  → 怎么办：未命中不等于 kb 写错了，也可能是源码固定点没了或版本变了。"
  echo "    逐条查：源码在不在（不在就重新固定）、模式还在不在（不在就现查改 kb 并记进 decisions-history）。"
  echo "    ⚠️ 不许因为「树不在」就把这条当跳过——那正是 checks-owed.md C38 要拦的证据蒸发。"
  exit 1
fi
echo "  ✓ 全部命中"
