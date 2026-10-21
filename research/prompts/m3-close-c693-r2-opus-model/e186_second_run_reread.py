"""m3-close-c693-r2 攻方腿：E186 第二次跑的判决在三种读法下重判（只读两份入库产物，纯算术）。

读法：
  as_registered   第二次跑登记的判据原样（dangling-u5，B1、W2、Q2 计），从第二次跑的格行重算，必须复出产物第 186276-186278 行。
  u5_excluded     B1 的排除理由（登记第 29 行「每条非现任臂都要改它，计进去等于先给现任臂一票」）同样用到 U5（D8 已定项 12
                  定案里点名 ㊀ 那类行的半句）：U5 不计。实现：第二次跑的格行里 clauses_false 含 U5 的，第 6 位减 1。
                  自证：这一读法下的 172800 格向量必须逐格等于第一次跑产物的 vector_q2_counted（vacuous U5 从不为假）。
  per_statistic   在上面两种读法上各加一条记账臂「按统计量」（第一轮攻方 W1-戊：有台阶行例外的统计量不带这一代那一行，
                  其余统计量照 Carry 带），向量由同一点的格行派生，不是装置的一次跑：
                    记账臂起作用的 flags 格（Skip、RejectEntry、PerBit 在 registered=4）：r_tree_dim=1 时等于 NoCarry 那一格；
                    r_tree_dim=2 时等于 Carry 那一格、第 7 位减 h_new（水位那一统计量每个不认识的头少写这一代那一行）；
                    其余 flags 格（Ignore、ReadOnly、PerBit 在 registered=0）等于 Carry 那一格（这些格里三条记账臂本来全等或按 Carry 写）。
                  只在被判点（合法点、h_new>0）上派生：(s,lpc)=(0,0) 时水位那一元组没有台阶行，这一派生不成立。
形态与后果照第二次跑登记第十节那张表（form=…）。

弄坏开关：E186_REREAD_BREAK=keep_u5  u5_excluded 读法里不减 U5 ⇒ 与第一次跑的逐格比对必须出现不等、自证退 3。
"""
import os
import sys
from collections import Counter, defaultdict

FIRST, SECOND = sys.argv[1], sys.argv[2]
BREAK = os.environ.get("E186_REREAD_BREAK", "")
FLAGS = ["Ignore", "Skip", "RejectEntry", "ReadOnly", "PerBit"]
ACCOUNTING = ["Carry", "NoCarry", "KeepWithRoot"]


def fields(line):
    out = {}
    for token in line.split()[1:]:
        if "=" in token:
            key, value = token.split("=", 1)
            out[key] = value
    return out


def load(path):
    points, cells = {}, {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            if " name=cell " in line:
                f = fields(line)
                vector = tuple(int(x) for x in f["vector_q2_counted"].split(","))
                u5_false = any(part.startswith("U5:") for part in f["clauses_false"].split("+"))
                cells[(int(f["point"]), f["arm"])] = (vector, u5_false)
            elif " name=point_verdict " in line:
                f = fields(line)
                points[int(f["point"])] = {k: f[k] for k in ("geometry", "h_new", "registered", "s", "lpc", "r_tree_dim", "legal")}
    return points, cells


def acting(flags, registered):
    return flags in ("Skip", "RejectEntry") or (flags == "PerBit" and registered == 4)


def winners(vectors):
    best = min(vectors.values())
    return {cell for cell, vector in vectors.items() if vector == best}


def form_of(sets):
    if not sets:
        return "no_judged_points"
    distinct = set(sets)
    with_carry = [("Carry" in s) for s in sets]
    if len(distinct) == 1:
        only = next(iter(distinct))
        if only == frozenset({"Carry"}):
            return "unique_carry"
        if len(only) == 1:
            return "unique_other_arm"
        return "tie_with_carry" if "Carry" in only else "tie_without_carry"
    if all(with_carry):
        return "depends_on_point_carry_everywhere"
    if not any(with_carry):
        return "depends_on_point_carry_nowhere"
    return "depends_on_point_mixed"


CONSEQUENCE = {"unique_carry": "carry_stands", "tie_with_carry": "carry_stands_tie_known_to_user",
               "depends_on_point_carry_everywhere": "carry_stands", "unique_other_arm": "flips",
               "tie_without_carry": "flips_choice_to_user", "depends_on_point_carry_nowhere": "flips_choice_to_user",
               "depends_on_point_mixed": "left_to_user", "no_judged_points": "none"}


def main():
    points1, cells1 = load(FIRST)
    points2, cells2 = load(SECOND)
    assert points1 == points2 and len(points2) == 11520 and len(cells2) == 172800, "两次跑的点集与格数"
    # 自证：u5_excluded 读法下第二次跑的格向量逐格等于第一次跑的格向量
    mismatches = 0
    for key, (vector, u5_false) in cells2.items():
        adjusted = list(vector)
        if u5_false and BREAK != "keep_u5":
            adjusted[5] -= 1
        if tuple(adjusted) != cells1[key][0]:
            mismatches += 1
    print(f"E7RESULT name=selftest check=u5_excluded_equals_first_run cells={len(cells2)} mismatches={mismatches}")
    selftest_failed = mismatches != 0

    def vectors_at(point, reading, with_per_statistic):
        p = points2[point]
        h_new, registered, r = int(p["h_new"]), int(p["registered"]), int(p["r_tree_dim"])
        out = {}
        for flags in FLAGS:
            for accounting in ACCOUNTING:
                vector, u5_false = cells2[(point, f"{flags}_{accounting}")]
                vector = list(vector)
                if reading == "u5_excluded" and u5_false:
                    vector[5] -= 1
                out[f"{flags}_{accounting}"] = tuple(vector)
            if with_per_statistic:
                if acting(flags, registered):
                    if r == 1:
                        derived = out[f"{flags}_NoCarry"]
                    else:
                        carry = list(out[f"{flags}_Carry"])
                        carry[6] -= h_new
                        derived = tuple(carry)
                else:
                    derived = out[f"{flags}_Carry"]
                out[f"{flags}_PerStatistic"] = derived
        return out

    for reading in ("as_registered", "u5_excluded"):
        for with_per_statistic in (False, True):
            arm_set = "three_arms" if not with_per_statistic else "three_arms_plus_per_statistic"
            groups = defaultdict(list)
            for point, p in sorted(points2.items()):
                if p["legal"] != "1" or int(p["h_new"]) == 0:
                    continue
                w = winners(vectors_at(point, reading, with_per_statistic))
                accounting_set = frozenset(cell.split("_", 1)[1] for cell in w)
                groups[f"legal_r_tree_dim_{p['r_tree_dim']}"].append(accounting_set)
                groups["legal_pooled"].append(accounting_set)
            for group in ("legal_r_tree_dim_1", "legal_r_tree_dim_2", "legal_pooled"):
                sets = groups[group]
                tally = Counter("+".join(sorted(s, key=(ACCOUNTING + ["PerStatistic"]).index)) for s in sets)
                rendered = "|".join(f"{k}:{v}" for k, v in sorted(tally.items(), key=lambda kv: (-kv[1], kv[0])))
                form = form_of(sets)
                print(f"E7RESULT name=q1_form reading={reading} arms={arm_set} group={group} judged_points={len(sets)} "
                      f"form={form} consequence={CONSEQUENCE[form]} winner_sets={rendered}")
    print("E7RESULT name=done")
    return 3 if selftest_failed else 0


if __name__ == "__main__":
    sys.exit(main())
