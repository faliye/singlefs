#!/usr/bin/env python3
"""B1：种类码与树 ID「错开」在三个候选下还剩什么判别力。

错开的来历（records/2026-09-13-总审核.md 第 270 行）：「错开树 ID 让「种类写成树 ID」在新池新建文件就红」——
它的读者是钉字节的测试与读树表的一方，不是格式本身。这里对新池新建文件那一版的树表（中央映射树不进树表）量四种写错：
  W1 某一条条目的种类字段写成自己树 ID 的低 2 字节（单条写错）
  W2 全部条目的种类字段都写成自己树 ID 的低 2 字节（写者系统性写错）
  W3 读者按「树 ID == 种类常量」找条目（查找写错）
  W4 某一条条目的树 ID 写成自己的种类码（单条写错，反方向）
每种写错的结局：bytes_differ（钉字节的测试红）、refused（读者拒：种类没登记 / 同一种类出现两次 / 树 ID 不升序）、
silent（读者照常读下去、读成别的树）。recovery.rs 的读者：种类没登记 ⇒ 拒（user_visible_tree_root_pointers），
同一种类两次 ⇒ 拒（tree_table_entries_each_kind_at_most_once），树 ID 不按发号次序升序 ⇒ 拒（I-9.16）。
"""

NAMES = ["extent", "inode", "alloc", "accounting", "livelist", "sparse", "deadlist",
         "deleted_inodes", "tombstone", "lineage", "logged_ops"]

def table(kinds, base):
    # 发号次序：extent、inode、分配记录、记账、中央映射（不进树表）、livelist、稀疏旁表、deadlist、四棵新树
    order = [0, 1, 2, 3, None, 4, 5, 6, 7, 8, 9, 10]
    rows = []
    for offset, idx in enumerate(order):
        if idx is not None and idx < len(kinds):
            rows.append((NAMES[idx], kinds[idx], base + offset))
    return rows

CANDIDATES = {
    "old_8_trees": (table([1, 2, 3, 4, 6, 7, 8], 11), [1, 2, 3, 4, 5, 6, 7, 8]),
    "A_append_9_12": (table([1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12], 11), list(range(1, 13))),
    "B_segment_0x8000": (table([1, 2, 3, 4, 6, 7, 8, 0x8000, 0x8001, 0x8002, 0x8003], 11),
                         [1, 2, 3, 4, 5, 6, 7, 8, 0x8000, 0x8001, 0x8002, 0x8003]),
    "C_ids_from_65536": (table([1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12], 65536), list(range(1, 13))),
    "C_ids_from_65547": (table([1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12], 65547), list(range(1, 13))),
}

def reader(rows, registered):
    kinds = [k for _, k, _ in rows]
    if any(k not in registered for k in kinds):
        return "refused_unregistered"
    if len(set(kinds)) != len(kinds):
        return "refused_kind_twice"
    if any(rows[i][2] >= rows[i + 1][2] for i in range(len(rows) - 1)):
        return "refused_order"
    return "silent"

def main():
    for name, (rows, registered) in CANDIDATES.items():
        own_equal = [n for n, k, t in rows if k == t]
        w1 = {}
        for i, (n, k, t) in enumerate(rows):
            mutated = list(rows)
            mutated[i] = (n, t & 0xFFFF, t)
            outcome = "same_bytes" if (t & 0xFFFF) == k else reader(mutated, registered)
            w1[outcome] = w1.get(outcome, 0) + 1
        w2_rows = [(n, t & 0xFFFF, t) for n, k, t in rows]
        w2 = "same_bytes" if w2_rows == rows else reader(w2_rows, registered)
        w3_silent = [f"{n}->{m}" for n, k, _ in rows for m, _, t in rows if t == k]
        w4 = {}
        for i, (n, k, t) in enumerate(rows):
            mutated = list(rows)
            mutated[i] = (n, k, k)
            outcome = "same_bytes" if k == t else reader(mutated, registered)
            w4[outcome] = w4.get(outcome, 0) + 1
        print(f"E name=kind_alias cand={name} entries={len(rows)} own_kind_equals_own_id={own_equal} "
              f"W1={sorted(w1.items())} W2={w2} W3_lookup_returns_other_tree={w3_silent} W4={sorted(w4.items())}")
    # 乙 能不能「永远不撞」：树 ID 水位只增；第一个文件版本每次从水位连号发 12 个，一次快照 1 个（D8 已定项 8 ②）
    for segment in (0x100, 0x8000, 0xFFF0):
        # 第 k 次重发第一个文件版本发出 11 + 12k .. 22 + 12k；第 k 个快照取 22 + k
        reissues = -(-(segment - 22) // 12)
        print(f"E name=kind_segment_reached segment_start={segment} first_file_reissues_until_a_tree_id_reaches_it={reissues} "
              f"or_snapshots={segment - 22}")

if __name__ == "__main__":
    main()
