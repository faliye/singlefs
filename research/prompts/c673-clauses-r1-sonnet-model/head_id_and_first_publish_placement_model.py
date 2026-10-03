#!/usr/bin/env python3
"""C673 K1 / K2 的算术小模型（只用标准库，不读 crates）：
 部分 A（K1）：树表头 ID 两种读法（字面 65537 / 同一张树表里 inode 树条目的树 ID）在几张树表上的判定。
   树 ID、种类、头 ID 的取值表抄自 D8 已定项 8 / 已定项 9 / 已定项 11 与 core `FileVersionTreeIdentifiers::issued_from_watermark`
   （偏移 = 登记的 65536.. 常量减 mkfs 水位 65536，水位 + 偏移）。
 部分 B（K2）：用户数据落点的首次适配算术（单元区起点 U 下 mkfs 占 U..U+1 与 U+2，开放段 = 最低的 64 槽对齐全空段，
   用户数据取开放段之外最低的偶数空槽对），以及 E171 第 15 行那条变异（扫描起点 + 64 槽）给出的值。
输出一行一条 `name=… 键=值`。不碰盘、不编译、不跑重型测试。
"""
import math

MKFS_WATERMARK = 65536
# 发号次序里各棵树离水位的偏移（D8 已定项 11 登记 + D18 已定项 24 第 6 条）：种类码、名字、偏移
ISSUE = [  # (kind, name, offset_from_watermark)；中央映射树不进树表，种类码 5 只给偏移 4
    (1, "extent", 0), (2, "inode", 1), (3, "allocation", 2), (4, "accounting", 3),
    (None, "central_mapping", 4),
    (6, "livelist", 5), (7, "sparse", 6), (8, "deadlist", 7),
    (9, "deleted_inodes", 8), (10, "tombstone", 9), (11, "lineage", 10), (12, "logged_ops", 11),
]
HEAD_OWN = {2}                  # 写自己的树 ID
HEAD_SERVES = {1, 9, 10}        # 写 inode 树的树 ID
# 其余写 0（D8 已定项 8 头 ID 取值句，D5 已定项 9）


def table(watermark):
    entries = []
    for kind, name, off in ISSUE:
        if kind is None:
            continue
        entries.append({"kind": kind, "name": name, "tree": watermark + off})
    inode_tree = next(e["tree"] for e in entries if e["kind"] == 2)
    for e in entries:
        e["head"] = inode_tree if (e["kind"] in HEAD_OWN or e["kind"] in HEAD_SERVES) else 0
    return entries


def judge_literal(entries):
    """字面读法：每条的头 ID 等于登记表里写死的 65537（或 0）。"""
    bad = []
    for e in entries:
        want = 65537 if (e["kind"] in HEAD_OWN or e["kind"] in HEAD_SERVES) else 0
        if e["head"] != want:
            bad.append(e["name"])
    return bad


def judge_relative(entries):
    """相对读法：自己写自己 / 写同一张树表里 inode 树条目的树 ID（0 条或多条 inode 树条目的处理见 n_inode）。"""
    inode_ids = [e["tree"] for e in entries if e["kind"] == 2]
    bad = []
    for e in entries:
        if e["kind"] in HEAD_OWN:
            ok = e["head"] == e["tree"]
        elif e["kind"] in HEAD_SERVES:
            ok = e["head"] in inode_ids          # 唯一一条时就是「等于它」；0 条时恒红；多条时放宽到「是其中之一」
        elif e["kind"] in (3, 4, 6, 7, 8, 11, 12):
            ok = e["head"] == 0
        else:
            continue
        if not ok:
            bad.append(e["name"])
    return bad


def show(name, entries):
    print(f"name=k1_table label={name} entries={len(entries)} "
          f"inode_tree={next((e['tree'] for e in entries if e['kind'] == 2), None)} "
          f"literal_red={judge_literal(entries)} relative_red={judge_relative(entries)}")


# 干净的表：mkfs 水位发的、回退到无文件那一版之后水位 65548 再发的、再来一轮 65560
for w in (MKFS_WATERMARK, 65548, 65560):
    show(f"clean_issued_from_{w}", table(w))
# E171 第 271 行变异：deleted_inodes 头 ID 65537 -> 0（mkfs 水位那张表）
t = table(MKFS_WATERMARK)
next(e for e in t if e["kind"] == 9)["head"] = 0
show("e171_row271_deleted_inodes_head_zero", t)
# 同一变异落在水位 65548 发的那张表上（0 与 inode 树 ID 65549 不等：两种读法都红，但字面读法在干净表上先误红）
t = table(65548)
next(e for e in t if e["kind"] == 9)["head"] = 0
show("e171_row271_on_reissued_table", t)
# inode 树条目头 ID 写成 65537，而这张表的 inode 树是 65549（回退再建之后的「抄旧值」坏实现）
t = table(65548)
next(e for e in t if e["kind"] == 2)["head"] = 65537
show("reissued_table_inode_head_copies_the_old_65537", t)
# 谱系树头 ID 写成 inode 树的树 ID（应为 0）
t = table(MKFS_WATERMARK)
next(e for e in t if e["kind"] == 11)["head"] = 65537
show("lineage_head_not_zero", t)
# 0 条 inode 树条目（拿掉 inode 树条目，其余照旧）与两条 inode 树条目
t = [e for e in table(MKFS_WATERMARK) if e["kind"] != 2]
show("no_inode_tree_entry", t)
t = table(MKFS_WATERMARK) + [{"kind": 2, "name": "inode_second", "tree": 65600, "head": 65600}]
show("two_inode_tree_entries", t)
show("empty_table", [])

# ---- 部分 B：用户数据落点 ----
SLOT_BYTES = 16384
JOURNAL_RING_START_SLOT = 1024     # singlefs-format 常量（16 MiB / 16 KiB）
SEG = 64


def unit_area_start(ring_bytes):
    return JOURNAL_RING_START_SLOT + math.ceil(ring_bytes / SLOT_BYTES)


def first_fit_user_data(U, scan_start_offset=0):
    """第一个文件版本里 t1 先于任何提交内生块：这一刻没有开放的聚簇段（段在第一个提交内生块 bump 时才开，D3 已定项 10 ①），
    用户数据取单元区内最低的偶数空槽对；之后开放段 = 最低的 64 槽对齐全空段（t1 已占了第一段，所以是 U + 64）。"""
    allocated = {U, U + 1, U + 2}     # m1 实例表 2 槽、m2 树表 1 槽（layout/01 零节）
    cand = U + scan_start_offset
    while cand in allocated or cand + 1 in allocated:
        cand += 2
    t1 = cand
    allocated |= {t1, t1 + 1}
    seg = U
    while any(s in allocated for s in range(seg, seg + SEG)):
        seg += SEG
    return t1, seg


for label, ring in (("default_768MiB", 805306368), ("128MiB", 128 * 1024 * 1024), ("1GiB", 1024 ** 3)):
    U = unit_area_start(ring)
    t1, open_seg = first_fit_user_data(U)
    t1_mut, _ = first_fit_user_data(U, scan_start_offset=SEG)
    print(f"name=k2_first_fit label={label} unit_area_start={U} aligned_64={U % SEG == 0} "
          f"open_segment_start={open_seg} t1={t1} t1_minus_U={t1 - U} mutated_scan_from_U_plus_64_t1={t1_mut}")
