#!/usr/bin/env python3
"""攻方 m3-batch33s-code-r1 S5 模型：把 CPU 的 crash::classified_oracle_violation_for_the_version_table 与 GPU 的
crash_judge_recovery.wgsl classify_oracle 逐句抄成 Python，在小根域上穷举版本表（每个根：不在表 / 有文件 / 没有文件）、落到的根、
盘上最新持久根与恢复结局，比两边判的违例种类；再核崩溃放量复用键的截法（只留 (txg, 实例) 不超过上界的行）不改判定。
纯算术，不读仓。用法：python3 -I s5_oracle_model.py  → 打印 S5_ROW 行（无耗时字段）。"""
import itertools

TXGS = [1, 2, 3]
INSTANCES = [0, 1]
ROOTS = [(t, i) for t in TXGS for i in INSTANCES]          # (txg, 实例)
EXTRA_ROOTS = [(4, 0), (0, 1)]                               # 不在任何表里的根（更新、更旧）
OUTCOMES = ["failed", "no_file", "file_match", "file_mismatch"]


def cpu(table, effective, newest, outcome):
    """table: dict root -> 'with' | 'without'（with 的内容一律当「与 file_match 相同」）。"""
    if effective is None:
        return "NoRootChosen"
    if newest is not None and effective < newest:
        return "RecoveredToAnOlderRoot"
    with_roots = [r for r, k in table.items() if k == "with"]
    without_roots = [r for r, k in table.items() if k == "without"]
    if effective in with_roots:
        at = "with"
    elif effective in without_roots:
        at = "without"
    else:
        at = "absent"
    if outcome == "failed":
        return "ReadFailed"
    if outcome == "no_file":
        if at == "absent":
            older_with = [r for r in with_roots if r < effective]
            older_without = [r for r in without_roots if r < effective]
            w = max(older_with) if older_with else None
            wo = max(older_without) if older_without else None
            if w is not None and wo is not None:
                return "NoFileWhereAnOlderRootHasOne" if w > wo else None
            if w is not None:
                return "NoFileWhereAnOlderRootHasOne"
            return None
        if at == "without":
            return None
        return "NoFileWhereThisRootHasOne"
    if at in ("absent", "without"):
        return "FileReadWhereThisRootHasNone"
    return None if outcome == "file_match" else "WrongContent"


def gpu(rows, effective, newest, outcome):
    """rows: 期望版本表按 crash_judge_tables 的次序（带文件的在前、没有文件的接在后面），每行 (txg, 实例, 第 5 字)。"""
    if effective is None:
        return "NoRootChosen"
    e_txg, e_inst = effective
    if newest is not None:
        n_txg, n_inst = newest
        if e_txg < n_txg or (e_txg == n_txg and e_inst < n_inst):
            return "RecoveredToAnOlderRoot"
    version = None
    for index, (txg, inst, _w5) in enumerate(rows):
        if inst == e_inst and txg == e_txg:
            version = index
            break
    has_a_file = version is not None and rows[version][2] == 0
    if outcome == "failed":
        return "ReadFailed"
    if outcome == "no_file":
        if version is None:
            newest_older = None
            for index, (txg, inst, _w5) in enumerate(rows):
                is_older = txg < e_txg or (txg == e_txg and inst < e_inst)
                if newest_older is None:
                    newer = True
                else:
                    o_txg, o_inst = rows[newest_older][0], rows[newest_older][1]
                    newer = o_txg < txg or (o_txg == txg and o_inst < inst)
                if is_older and newer:
                    newest_older = index
            if newest_older is not None and rows[newest_older][2] == 0:
                return "NoFileWhereAnOlderRootHasOne"
            return None
        if not has_a_file:
            return None
        return "NoFileWhereThisRootHasOne"
    if version is None or not has_a_file:
        return "FileReadWhereThisRootHasNone"
    return None if outcome == "file_match" else "WrongContent"


def rows_of(table, order):
    with_rows = [(r[0], r[1], 0) for r in order if table.get(r) == "with"]
    without_rows = [(r[0], r[1], 1) for r in order if table.get(r) == "without"]
    return with_rows + without_rows


def main():
    effectives = [None] + ROOTS + EXTRA_ROOTS
    newests = [None] + ROOTS
    cases = 0
    disagreements = 0
    first_disagreement = None
    key_cases = 0
    key_changes = 0
    first_key_change = None
    for kinds in itertools.product(["absent", "with", "without"], repeat=len(ROOTS)):
        table = {r: k for r, k in zip(ROOTS, kinds) if k != "absent"}
        for order in (ROOTS, list(reversed(ROOTS))):
            rows = rows_of(table, order)
            for effective, newest, outcome in itertools.product(effectives, newests, OUTCOMES):
                cases += 1
                a = cpu(table, effective, newest, outcome)
                b = gpu(rows, effective, newest, outcome)
                if a != b:
                    disagreements += 1
                    if first_disagreement is None:
                        first_disagreement = (table, order, effective, newest, outcome, a, b)
        # 复用键的截法：上界 highest 取任一根；落到的根不超过它；截掉 (txg, 实例) 大于上界的行之后判定不变
        for highest in ROOTS + EXTRA_ROOTS:
            cut = {r: k for r, k in table.items() if r <= highest}
            for effective in [e for e in effectives if e is not None and e <= highest] + [None]:
                for newest, outcome in itertools.product(newests, OUTCOMES):
                    key_cases += 1
                    if cpu(table, effective, newest, outcome) != cpu(cut, effective, newest, outcome):
                        key_changes += 1
                        if first_key_change is None:
                            first_key_change = (table, highest, effective, newest, outcome)
    print(f"S5_ROW name=cpu_vs_gpu_oracle cases={cases} disagreements={disagreements} first={first_disagreement}")
    print(f"S5_ROW name=reuse_key_cut_keeps_the_verdict cases={key_cases} changes={key_changes} first={first_key_change}")
    # 自证：GPU 若照改之前的「任一更旧版有文件」判（不看最新那一项是不是没有文件），这里必须分出不同
    def gpu_before(rows, effective, newest, outcome):
        verdict = gpu(rows, effective, newest, outcome)
        if outcome == "no_file" and effective is not None and (newest is None or not effective < newest):
            if all(not (r[1] == effective[1] and r[0] == effective[0]) for r in rows):
                if any((r[0], r[1]) < effective and r[2] == 0 for r in rows):
                    return "NoFileWhereAnOlderRootHasOne"
        return verdict
    probe = {(1, 0): "with", (2, 0): "without"}
    a = cpu(probe, (3, 0), None, "no_file")
    b = gpu_before(rows_of(probe, ROOTS), (3, 0), None, "no_file")
    print(f"S5_ROW name=selfproof_old_any_older_rule_is_caught cpu={a} gpu_before={b} caught={a != b}")


main()
