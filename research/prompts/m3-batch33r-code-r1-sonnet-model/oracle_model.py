#!/usr/bin/env python3
"""正推腿的独立小模型（不依赖仓里的 Rust）：把 crash.rs 里 R1 的新判法与旧判法逐分支抄成 Python，
再照 D8 已定项 16 第 4 条写一份「规约」，对小范围里的每一张版本表、每个落到的根、每种读回结果穷举比对。
用法：python3 oracle_model.py   （输出行以 MODEL 开头，无随机、无耗时字段）
抄的对象（冻结副本 crates/singlefs-checker-tier/src/crash.rs）：
  新判法   classified_oracle_violation_for_the_version_table（第 1096 行起）、newest_older_version_if_it_has_a_file（第 1046 行起）
  旧判法   改动前的 (NoFile, None) 分支：版本表里任一更旧版有文件就违例（diff 里被删的那一段）
"""
import itertools

INSTANCES = (1, 2)
TXGS = (1, 2, 3, 4, 5)
ROOTS = [(inst, txg) for inst in INSTANCES for txg in TXGS]      # (实例, txg)
key = lambda root: (root[1], root[0])                              # 择根序：先 txg，再实例（D22 已定项 7）

# ---- 新判法（逐分支抄 crash.rs） -------------------------------------------------
def version_at_the_root(with_file, without_file, root):
    if root in with_file:
        return ("WithAFile", with_file[root])
    if root in without_file:
        return ("WithoutAFile", None)
    return ("NotInTheTable", None)

def newest_older_if_it_has_a_file(with_file, without_file, root):      # 第 1046–1072 行
    older_with = [r for r in with_file if key(r) < key(root)]
    newest_older_with = max(older_with, key=key) if older_with else None
    older_without = [r for r in without_file if key(r) < key(root)]
    newest_older_without = max(older_without, key=key) if older_without else None
    if newest_older_with is not None and newest_older_without is not None:
        return newest_older_with if key(newest_older_with) > key(newest_older_without) else None
    if newest_older_with is not None:
        return newest_older_with
    return None

def new_oracle(outcome, root, with_file, without_file, mutant=None):
    """outcome: ("NoFile",) / ("FileRead", content) / ("Failed",)。返回违例类名或 None。"""
    where = version_at_the_root(with_file, without_file, root)
    if outcome[0] == "Failed":
        return "ReadFailed"
    if outcome[0] == "NoFile":
        if where[0] == "NotInTheTable":
            older = newest_older_if_it_has_a_file_mutable(with_file, without_file, root, mutant)
            return "NoFileWhereAnOlderRootHasOne" if older is not None else None
        if where[0] == "WithoutAFile":
            return "NoFileWhereThisRootHasOne" if mutant == "row3106" else None
        return "NoFileWhereThisRootHasOne"
    # FileRead
    if where[0] in ("NotInTheTable", "WithoutAFile"):
        return "FileReadWhereThisRootHasNone"
    return "WrongContent" if outcome[1] != where[1] else None

def newest_older_if_it_has_a_file_mutable(with_file, without_file, root, mutant):
    """同 newest_older_if_it_has_a_file，按变异行改坏（对应 crates/mutations.tsv 第 16、3104、3105、3107 行）。"""
    pick = lambda items: (max(items, key=key) if items else None)
    older_with = [r for r in with_file if key(r) < key(root)]
    if mutant == "row16":
        older_with = []                                               # filter 里的比较换成 false
    if mutant == "row3107":
        newest_older_with = min(older_with, key=key) if older_with else None   # max_by_key -> min_by_key
    else:
        newest_older_with = pick(older_with)
    older_without = [r for r in without_file if key(r) < key(root)]
    newest_older_without = pick(older_without)
    if newest_older_with is not None and newest_older_without is not None:
        if mutant == "row3104":
            return newest_older_with                                  # 不看没有文件那一项
        return newest_older_with if key(newest_older_with) > key(newest_older_without) else None
    if newest_older_with is not None:
        if mutant == "row3105":
            return None                                               # (Some, None) => None
        return newest_older_with
    return None

# ---- 旧判法（diff 里被删的分支，文件从不消失的表） ---------------------------------
def old_oracle(outcome, root, with_file):
    where = ("WithAFile", with_file[root]) if root in with_file else ("None", None)
    if outcome[0] == "Failed":
        return "ReadFailed"
    if outcome[0] == "NoFile":
        if where[0] == "None":
            any_older = any(key(r) < key(root) for r in with_file)
            return "NoFileWhereAnOlderRootHasOne" if any_older else None
        return "NoFileWhereThisRootHasOne"
    if where[0] == "None":
        return "FileReadWhereThisRootHasNone"
    return "WrongContent" if outcome[1] != where[1] else None

# ---- 规约（D8 已定项 16 第 4 条的读法，独立于上面两份写法）-------------------------
def spec(outcome, root, with_file, without_file):
    """表按择根序排成一条时间线；落到的根在表里就是那一项的状态，不在表里就继承比它旧的最新一项的状态，没有更旧的项就是没有文件。
    读回内容只在「落到的根本身是带文件的一项」时合法（表里没有的根拿不出该读的内容）。"""
    if outcome[0] == "Failed":
        return "ReadFailed"
    entries = [(key(r), "F", r) for r in with_file] + [(key(r), "N", r) for r in without_file]
    entries.sort()
    if root in with_file:
        state = "F"
    elif root in without_file:
        state = "N"
    else:
        older = [e for e in entries if e[0] < key(root)]
        state = older[-1][1] if older else "N"
    in_table = root in with_file or root in without_file
    if outcome[0] == "NoFile":
        if state == "F":
            return "NoFileWhereThisRootHasOne" if in_table else "NoFileWhereAnOlderRootHasOne"
        return None
    if root in with_file:
        return "WrongContent" if outcome[1] != with_file[root] else None
    return "FileReadWhereThisRootHasNone"

def outcomes_for(with_file):
    contents = sorted(set(with_file.values())) + ["foreign"]
    return [("NoFile",), ("Failed",)] + [("FileRead", c) for c in contents]

def tables(allow_without):
    labels = ("absent", "F", "N") if allow_without else ("absent", "F")
    for assignment in itertools.product(labels, repeat=len(ROOTS)):
        with_file = {r: f"c{inst}{txg}" for r, a in zip(ROOTS, assignment) if a == "F" for inst, txg in [r]}
        without_file = {r: None for r, a in zip(ROOTS, assignment) if a == "N"}
        yield with_file, without_file

def main():
    # 1. 文件从不消失的表：新判法（without 为空）与旧判法逐项相同（含违例类）
    judged = diff = 0
    for with_file, without_file in tables(False):
        for root in ROOTS:
            for outcome in outcomes_for(with_file):
                judged += 1
                if new_oracle(outcome, root, with_file, {}) != old_oracle(outcome, root, with_file):
                    diff += 1
    print(f"MODEL equivalence_new_vs_old tables_with_a_file_only judged={judged} differing={diff}")
    # 2. 全部表（含没有文件的一组）：新判法与规约逐项相同
    judged = diff = 0
    first_diff = None
    read_in_without_group = read_in_without_group_accepted = 0
    for with_file, without_file in tables(True):
        for root in ROOTS:
            for outcome in outcomes_for(with_file):
                judged += 1
                got, want = new_oracle(outcome, root, with_file, without_file), spec(outcome, root, with_file, without_file)
                if got != want:
                    diff += 1
                    first_diff = first_diff or (sorted(with_file), sorted(without_file), root, outcome, got, want)
                if outcome[0] == "FileRead" and root in without_file:
                    read_in_without_group += 1
                    read_in_without_group_accepted += got is None
    print(f"MODEL new_vs_spec_all_tables judged={judged} differing={diff} first_diff={first_diff}")
    print(f"MODEL file_read_on_a_root_in_the_without_a_file_group judged={read_in_without_group} accepted_as_ok={read_in_without_group_accepted}")
    # 3. 变异行：每一行在函数一级、对规约有没有不同（有不同才可能被判器那一层看见）
    for mutant in ("row16", "row3104", "row3105", "row3106", "row3107"):
        judged = diff = 0
        for with_file, without_file in tables(True):
            for root in ROOTS:
                for outcome in outcomes_for(with_file):
                    judged += 1
                    if new_oracle(outcome, root, with_file, without_file, mutant) != spec(outcome, root, with_file, without_file):
                        diff += 1
        print(f"MODEL mutant={mutant} differing_from_spec={diff} of={judged}")
    # 4. 新流那张表：A、B、C 带文件（实例 1 第 3、4、5 代），意图（第 6 代）没有文件，各批（第 7–9 代）不进表
    with_file = {(1, 3): "A", (1, 4): "B", (1, 5): "C"}
    without_file = {(1, 6): None}
    for root_txg in range(1, 11):
        root = (1, root_txg)
        row = []
        for outcome in [("NoFile",), ("FileRead", "A"), ("FileRead", "B"), ("FileRead", "C"), ("FileRead", "foreign")]:
            row.append(str(new_oracle(outcome, root, with_file, without_file)))
        print(f"MODEL unlink_stream_table txg={root_txg} NoFile/A/B/C/foreign -> " + " | ".join(row))
    for mutant in ("row3104", "row3105", "row3107"):
        hits = []
        for root_txg in (7, 8, 9):
            got = new_oracle(("NoFile",), (1, root_txg), with_file, without_file, mutant)
            hits.append(f"txg{root_txg}:{got}")
        print(f"MODEL unlink_stream_batch_roots_NoFile mutant={mutant} " + " ".join(hits))

if __name__ == "__main__":
    main()
