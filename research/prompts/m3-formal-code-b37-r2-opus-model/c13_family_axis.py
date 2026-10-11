#!/usr/bin/env python3
"""第 37 批代码轮第二轮攻方腿（Opus）的 C1 / C3 / C4 模型：只建「一次跑出哪几条对照」这一层，不编 Rust、不跑装置。

读三样东西（都只读）：
  1. 冻结副本里的 e165 装置源码：判据枚举（变体 → 标签）、`Criterion::family` 的分族、`is_judged_under` 的六条臂、
     `positive_control_readings` 的族过滤那一行、改后单测里写死的五份集合与五个 (档, 族) 调用；
  2. 仓里 `scripts/experiments/replay.sh` 起 e165 装置的驱动（参数）与它登记的产物名；
  3. 那几份产物里 `name=control criterion=` 的判据集合——拿它独立核模型算出的「今天每条正式跑有哪几条对照」。
然后：
  A. C1：单测的五个调用里，哪几个是 `main` 根本起不出来的族组合；正式跑按档并起来的对照集与单测写死的集合逐档比；
  B. C3：族这一维、盘次序、奇偶、标签相撞几类改坏，单测放不放过、正式跑哪几条驱动少了对照；
  C. C4：单测五个调用逐个去掉，四行变异里锚在这条单测上的三行还死不死。
对照读数一律当「跑了就红」（只管覆盖、不管数值），这是模型的限度。

用法：c13_family_axis.py <冻结副本根> <仓根>
"""
import itertools
import re
import sys
from pathlib import Path

L, A, M = "L", "A", "M"
FAMILY_OF_ENUM = {"LegalHistory": L, "Adversarial": A, "MultiDisk": M}
REACHABLE_FAMILY_SETS = [frozenset({L, A}), frozenset({L}), frozenset({A}), frozenset({M})]  # parse_the_command_line 的四种
SOURCE = "crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"
MUST_BE_PRESENT = [
    "        if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {",
    "        if !criterion.is_judged_under(run_profile) {",
    "    let mut families = vec![Family::LegalHistory, Family::Adversarial];",
    "                    families = vec![Family::MultiDisk];",
    "            Self::TableComparisonSeesADifferentTable | Self::TableComparisonSeesADifferentRow",
    "            let readings = positive_control_readings(families, run_profile, thread_count);",
    "            let controlled: std::collections::BTreeSet<&str> = readings",
    "                .map(|reading| reading.criterion.label())",
    "                controlled_in_some_profile.contains(criterion.label()),",
    "        \"r3\" | \"r4\" => Ok((",
]


def between(text, start, end):
    i = text.index(start)
    return text[i:text.index(end, i)]


def parse_criteria(src):
    block = between(src, "    enum Criterion {", "\n    }\n}")
    pairs = re.findall(r"(\w+) => \"([^\"]+)\",", block)
    return [v for v, _ in pairs], dict(pairs)


def parse_family(src):
    block = between(src[src.index("impl Criterion {"):], "    const fn family(self) -> Family {", "    const fn is_judged_in_every_family")
    out, pending = {}, []
    for token in re.finditer(r"Self::(\w+)|=> Family::(\w+)", block):
        if token.group(1):
            pending.append(token.group(1))
        else:
            for v in pending:
                out[v] = FAMILY_OF_ENUM[token.group(2)]
            pending = []
    return out


def parse_arms(src):
    block = between(src, "    const fn is_judged_under(self, run_profile: RunProfile) -> bool {", "\n    }\n}")
    out, pending = {}, []
    for token in re.finditer(r"Self::(\w+)|=> (?:\{\s*)?(run_profile\.(\w+)\(\)|true)", block):
        if token.group(1):
            pending.append(token.group(1))
        else:
            arm = "base" if token.group(2) == "true" else token.group(3)
            for v in pending:
                out[v] = arm
            pending = []
    return out


PREDICATES = {
    "judges_j6qn": lambda p: p["set"] in ("R2", "R3", "R5"),
    "judges_j7": lambda p: p["set"] in ("R2", "R3"),
    "judges_j8": lambda p: p["set"] == "R5",
    "splits_j6qn_by_i812": lambda p: p["set"] in ("R3", "R5"),
    "compares_the_two_record_tables": lambda p: p["both"],
    "base": lambda p: True,
    "never": lambda p: False,
}


def profile(criteria_set, both=False, reversed_order=False, parity="even"):
    return {"set": criteria_set, "both": both, "reversed": reversed_order, "parity": parity}


def name_of(p):
    return p["set"].lower() + ("+both" if p["both"] else "") + ("+reversed" if p["reversed"] else "") + \
        ("+odd" if p["parity"] == "odd" else "")


class Model:
    """今天的源码读出来的那一份；改坏 = 换掉 family / arm / filter / every_family 之一。"""

    def __init__(self, variants, labels, family, arms):
        self.variants, self.labels = variants, labels
        self.family, self.arms = dict(family), dict(arms)
        self.every_family = {"TableComparisonSeesADifferentTable", "TableComparisonSeesADifferentRow"}
        self.filter_family = None          # None：照 criterion.family()；"ALL"：族一律放过；否则写死成那一族
        self.guard = {}                    # 臂 → 额外条件（盘次序、奇偶）
        self.label_override = {}

    def copy(self):
        other = Model(self.variants, self.labels, self.family, self.arms)
        other.every_family = set(self.every_family)
        other.filter_family, other.guard, other.label_override = self.filter_family, dict(self.guard), dict(self.label_override)
        return other

    def label(self, v):
        return self.label_override.get(v, self.labels[v])

    def judged(self, v, p):
        arm = self.arms[v]
        return PREDICATES[arm](p) and self.guard.get(arm, lambda q: True)(p)

    def family_passes(self, v, families):
        if v in self.every_family:
            return True
        if self.filter_family == "ALL":
            return True
        fam = self.family[v] if self.filter_family is None else self.filter_family
        return fam in families

    def controls(self, families, p):
        """positive_control_readings：交回有对照的判据（变体名）。"""
        return {v for v in self.variants if self.family_passes(v, families) and self.judged(v, p)}


def parse_test(src, labels):
    body = between(src, "fn every_criterion_has_a_positive_control_and_every_control_is_red()", "\n    }\n\n")
    arrays = {}
    for name, inner in re.findall(r"let (\w+) = \[(.*?)\];", body, re.S):
        found = re.findall(r"Criterion::(\w+)", inner)
        if found:
            arrays[name] = found
    expanded = dict(arrays)
    r2_expr = re.search(r"let r2_criteria: Vec<Criterion> =\s*\[(.*?)\]\.concat\(\);", body, re.S).group(1)
    expanded["r2_criteria"] = [v for ref in re.findall(r"&(\w+)", r2_expr) for v in expanded[ref]]
    family_vars = {"legal_and_adversarial": frozenset({L, A}), "all_families": frozenset({L, A, M})}
    for name, inner in re.findall(r"let (\w+) = \[(Family::[^\]]*)\];", body):
        family_vars[name] = frozenset(FAMILY_OF_ENUM[f] for f in re.findall(r"Family::(\w+)", inner))
    profiles_block = between(body, "let profiles:", "let mut controlled_in_some_profile")
    entries = []
    for match in re.finditer(r"\(\s*\"([^\"]+)\",\s*&(\w+),\s*(RunProfile::[^,]*?(?:\([^)]*\))?(?:\s*\.with_table_source\(RecordTableSource::\w+\))?),\s*(.*?),\s*\),", profiles_block, re.S):
        name, fam_var, prof_expr, reg_expr = match.groups()
        prof_expr = re.sub(r"\s+", "", prof_expr)
        criteria_set = {"RunProfile::R1": "R1"}.get(prof_expr, None) or "R" + re.search(r"RunProfile::r(\d)", prof_expr).group(1)
        p = profile(criteria_set, both="RecordTableSource::Both" in prof_expr)
        refs = re.findall(r"(\w+)(?:\[\.\.\]|\.to_vec\(\)|\.clone\(\))", reg_expr) or re.findall(r"&(\w+)", reg_expr)
        refs = [r for r in refs if r in expanded] + [r for r in re.findall(r"&(\w+)", reg_expr) if r in expanded and r not in refs]
        registered = [v for ref in dict.fromkeys(refs) for v in expanded[ref]]
        entries.append((name, family_vars[fam_var], p, registered))
    return entries


def test_passes(model, entries):
    """改后单测：五个调用各自「对照集（按标签）= 写死的那一份（按标签）」且读数全红（模型里跑了就红），五档并起来罩全表（按标签）。"""
    union = set()
    for name, families, p, registered in entries:
        controlled = {model.label(v) for v in model.controls(families, p)}
        if controlled != {model.label(v) for v in registered}:
            return False, name
        union |= controlled
    for v in model.variants:
        if model.label(v) not in union:
            return False, "罩全表"
    return True, None


def old_test_passes(model):
    """改前单测（改动全文 `-` 那几行）：r1 段只判读数红；r2 段只判没有拆行；r3、r5、r3+both 三档全族并起来罩全表。"""
    r2 = model.controls(frozenset({L, A, M}), profile("R2"))
    if "J6QnSplitByI812" in r2:
        return False
    union = set()
    for p in (profile("R3"), profile("R5"), profile("R3", both=True)):
        union |= model.controls(frozenset({L, A, M}), p)
    return all(v in union for v in model.variants)


def parse_drivers(repo):
    text = (repo / "scripts/experiments/replay.sh").read_text()
    artifact_of = dict(re.findall(r"^E165\|@(driver_e165\w*)\|\|([^|]+)\|exact$", text, re.M))
    drivers = []
    for name, body in re.findall(r"^(driver_e165\w*)\(\) \{\n(.*?)\n\}", text, re.M | re.S):
        command = re.search(r"e165_replay_bounded_enumeration -- ([^|]*?) \| grep", body)
        if command is None:
            continue                       # 经 e165-replay-mutation-kill-bounds.sh 起的驱动：产物里没有 name=control 行
        args = command.group(1)
        tokens = args.split()
        opt = {tokens[i]: tokens[i + 1] for i in range(0, len(tokens) - 1, 2)}
        family = opt.get("--family", "all")
        if family in ("root_choice", "root_choice_faults"):
            continue                       # 不走 positive_control_readings（另一套判据表）
        families = {"all": frozenset({L, A}), "legal": frozenset({L}), "adversarial": frozenset({A}),
                    "multi_disk": frozenset({M})}[family]
        prof = opt.get("--profile", "r1")
        criteria_set = {"r1": "R1", "r2": "R2", "r3": "R3", "r4": "R3", "r5": "R5"}[prof]
        p = profile(criteria_set, both=opt.get("--table") == "both", reversed_order=opt.get("--disk-order") == "reversed",
                    parity=opt.get("--marker-parity", "even"))
        drivers.append((name, families, p, artifact_of[name]))
    return drivers


def artifact_controls(repo, artifact):
    text = (repo / "research/results" / artifact).read_text()
    return set(re.findall(r"name=control criterion=(\S+)", text))


def production_gaps(base, mutated, drivers):
    """逐条驱动：今天有对照、改坏后没了的判据（删），与改坏后多出来的（加）。"""
    out = []
    for name, families, p, _ in drivers:
        before = {base.label(v) for v in base.controls(families, p)}
        after = {mutated.label(v) for v in mutated.controls(families, p)}
        if before != after:
            out.append((name, sorted(before - after), sorted(after - before)))
    return out


def entry_is_red(model, entry):
    """单测里一个调用自己那一段（集合相等）判不判红，不算最后「罩全表」那一条。"""
    _, families, p, registered = entry
    return {model.label(v) for v in model.controls(families, p)} != {model.label(v) for v in registered}


def union_losses(base, mutated, drivers):
    """按 --profile 把正式跑并起来：今天这一档的正式跑里有对照、改坏后这一档哪一条正式跑里都没有的判据。"""
    before = per_profile_union(base, drivers, lambda p: p["set"])
    after = per_profile_union(mutated, drivers, lambda p: p["set"])
    return [(k.lower(), sorted(before[k] - after.get(k, set()))) for k in before if before[k] - after.get(k, set())]


def per_profile_union(model, drivers, key):
    union = {}
    for _, families, p, _ in drivers:
        union.setdefault(key(p), set()).update(model.label(v) for v in model.controls(families, p))
    return union


def fmt_fams(families):
    return "+".join(f for f in (L, A, M) if f in families)


def main():
    root, repo = Path(sys.argv[1]), Path(sys.argv[2])
    src = (root / SOURCE).read_text()
    for line in MUST_BE_PRESENT:
        if (line + "\n") not in src:
            print(f"✗ 源码里找不到模型依赖的一行：{line!r}")
            sys.exit(3)
    variants, labels = parse_criteria(src)
    family, arms = parse_family(src), parse_arms(src)
    if set(family) != set(variants) or set(arms) != set(variants) or len(variants) != 35:
        print("✗ 分族或臂没抽全")
        sys.exit(3)
    base = Model(variants, labels, family, arms)
    entries = parse_test(src, labels)
    drivers = parse_drivers(repo)
    print(f"== 判据 {len(variants)} 条；单测调用 {len(entries)} 个；replay.sh 里直接起装置、走判据表的正式跑 {len(drivers)} 条")

    print("== 核一：模型算的今天每条正式跑的对照集，与那条驱动登记的产物里 name=control 的判据集合逐条比")
    mismatches = 0
    for name, families, p, artifact in drivers:
        predicted = {labels[v] for v in base.controls(families, p)}
        observed = artifact_controls(repo, artifact)
        same = predicted == observed
        mismatches += not same
        print(f"  {name:58s} {name_of(p):15s} {fmt_fams(families):5s} 模型 {len(predicted):2d} 产物 {len(observed):2d} {'相同' if same else '不同'}")
    print(f"   不同 {mismatches} 条")
    if mismatches:
        sys.exit(4)

    print("== C1：单测五个调用（族组合 main 起不起得出 / 正式跑里有没有同档同族的一条）")
    for name, families, p, registered in entries:
        reachable = families in REACHABLE_FAMILY_SETS
        same_run = [d[0] for d in drivers if d[1] == families and name_of(d[2]) == name_of(p)]
        print(f"  {name:16s} {name_of(p):8s} {fmt_fams(families):6s} main 起得出：{'是' if reachable else '否'}；同档同族的正式跑：{same_run or '无'}；写死 {len(registered)} 条")
    print("== C1：正式跑按 (判据集, 两份表) 并起来的对照集，对单测写死的那一份")
    union = per_profile_union(base, drivers, lambda p: (p["set"], p["both"]))
    for name, families, p, registered in entries:
        got = union.get((p["set"], p["both"]), set())
        reg = {labels[v] for v in registered}
        missing = sorted(reg - got, key=lambda lab: [labels[v] for v in variants].index(lab))
        print(f"  {name:16s} 写死 {len(reg):2d}；正式跑并起来 {len(got):2d}；写了而正式跑这一档没有对照的 {len(missing)} 条：{missing}")
    union_by_profile = per_profile_union(base, drivers, lambda p: p["set"])
    for criteria_set in ("R1", "R2", "R3", "R5"):
        print(f"  只按 --profile 并：{criteria_set.lower()} {len(union_by_profile.get(criteria_set, set()))} 条")

    ok, where = test_passes(base, entries)
    print(f"== 原样：改后单测 {'过' if ok else '红在 ' + str(where)}；改前单测 {'过' if old_test_passes(base) else '红'}")

    def sweep(title, mutants, show=4):
        passed_with_loss = passed_add_only = passed_no_change = caught = passed_with_union_loss = 0
        examples, union_examples, unchanged = [], [], []
        for desc, mutant in mutants:
            ok, _ = test_passes(mutant, entries)
            gaps = production_gaps(base, mutant, drivers)
            lost = [g for g in gaps if g[1]]
            if not ok:
                caught += 1
            elif lost:
                passed_with_loss += 1
                if len(examples) < show:
                    examples.append((desc, lost))
                losses = union_losses(base, mutant, drivers)
                if losses:
                    passed_with_union_loss += 1
                    union_examples.append((desc, losses))
            elif gaps:
                passed_add_only += 1
            else:
                passed_no_change += 1
                unchanged.append(desc)
        print(f"== {title}：共 {len(mutants)} 个；单测红 {caught}；单测过而某条正式跑少了对照 {passed_with_loss}；"
              f"单测过、正式跑只多不少 {passed_add_only}；单测过、正式跑不变 {passed_no_change}")
        print(f"   单测过而某一档（按 --profile 把它的正式跑并起来）整档少了对照的 {passed_with_union_loss} 个：")
        for desc, losses in union_examples:
            print(f"     {desc} ⇒ {losses}")
        if unchanged:
            print(f"   单测过、正式跑不变的：{unchanged}")
        for desc, lost in examples:
            first = lost[0]
            print(f"  例：{desc} ⇒ 单测过；少了对照的正式跑 {len(lost)} 条，第一条 {first[0]} 少 {first[1]}")

    family_moves = []
    for v in variants:
        for f in (L, A, M):
            if f != family[v]:
                m = base.copy()
                m.family[v] = f
                family_moves.append((f"{labels[v]} 的族 {family[v]} → {f}", m))
    sweep("甲 `Criterion::family` 里一条判据挪到另一族", family_moves)

    filters = []
    for desc, setting in (("族过滤那一行去掉族条件（每族都跑）", "ALL"), ("族过滤写死成 &Family::LegalHistory", L),
                          ("族过滤写死成 &Family::Adversarial", A), ("族过滤写死成 &Family::MultiDisk", M)):
        m = base.copy()
        m.filter_family = setting
        filters.append((desc, m))
    m = base.copy()
    m.every_family = set()
    filters.append(("`is_judged_in_every_family` 交回 false", m))
    m = base.copy()
    m.every_family = set(variants)
    filters.append(("`is_judged_in_every_family` 交回 true", m))
    sweep("乙 族过滤那一行与 `is_judged_in_every_family`", filters, show=6)

    guards = []
    for arm in [a for a in PREDICATES if a != "never"]:
        for desc, guard in (("盘次序反过来时不判", lambda p: not p["reversed"]), ("奇偶取 odd 时不判", lambda p: p["parity"] != "odd")):
            m = base.copy()
            m.guard[arm] = guard
            guards.append((f"臂 {arm} 加「{desc}」", m))
    sweep("丙 `is_judged_under` 一条臂多一个看盘次序或奇偶的条件", guards, show=12)

    collisions = []
    for v in variants:
        for w in variants:
            if v != w:
                m = base.copy()
                m.label_override[v] = labels[w]
                collisions.append((f"{labels[v]} 的标签改成 {labels[w]}", m))
    passed = sum(test_passes(m, entries)[0] for _, m in collisions)
    print(f"== 丁 判据标签撞上另一条的（单测按标签比集合）：共 {len(collisions)} 个；单测过 {passed}")
    combo = base.copy()
    combo.label_override["J8aUnmountMarkerFromTheNewRootSegment"] = "J8a"
    combo.arms["J8aUnmountMarkerFromTheNewRootSegment"] = "never"
    ok, _ = test_passes(combo, entries)
    print(f"  丁 两步：J8a_unmount_marker 标签改成 J8a，同时它那一臂在 r5 档不判 ⇒ 单测 {'过' if ok else '红'}；"
          f"正式跑少了对照的驱动 {sum(1 for g in production_gaps(base, combo, drivers) if g[1])} 条")

    print("== C4：变异表里锚在这条单测上的三行（施加在 `is_judged_under` 上），改后 / 改前单测，与单测逐个去掉一个调用之后")
    rows = {
        "第 3383 行 基础臂 → !judges_j8()": ("base", lambda p: not PREDICATES["judges_j8"](p)),
        "第 3407 行 T 臂 → compares && splits": ("compares_the_two_record_tables",
                                              lambda p: PREDICATES["compares_the_two_record_tables"](p) and PREDICATES["splits_j6qn_by_i812"](p)),
        "第 3496 行 基础臂 → judges_j6qn()": ("base", lambda p: PREDICATES["judges_j6qn"](p)),
    }
    for desc, (arm, new) in rows.items():
        m = base.copy()
        m.guard[arm] = new
        ok, where = test_passes(m, entries)
        alone = [e[0] for e in entries if entry_is_red(m, e)]
        print(f"  {desc}：改后单测 {'过' if ok else '红在 ' + where}；改前单测 {'过' if old_test_passes(m) else '红'}；"
              f"单独就能判红的调用 {alone}")
    for i, entry in enumerate(entries):
        reduced = entries[:i] + entries[i + 1:]
        if not test_passes(base, reduced)[0]:
            print(f"  去掉「{entry[0]}」那个调用：没改坏的代码上单测就红（「罩全表」那一条兜住，这一个调用去不掉）")
            continue
        survivors = []
        for desc, (arm, new) in rows.items():
            m = base.copy()
            m.guard[arm] = new
            if test_passes(m, reduced)[0]:
                survivors.append(desc.split(" ")[1])
        print(f"  去掉「{entry[0]}」那个调用：三行里还活的 {survivors or '无'}")
    only_one = {
        "拆行臂 → splits && judges_j8()（r3 档少拆行对照）": ("splits_j6qn_by_i812", lambda p: p["set"] == "R5"),
        "T 臂 → compares || (judges_j7 && !splits)（r2 档多跑 T 对照）": ("compares_the_two_record_tables",
                                                                      lambda p: p["both"] or p["set"] == "R2"),
    }
    for desc, (arm, pred) in only_one.items():
        PREDICATES[f"x_{arm}"] = pred
        m = base.copy()
        m.arms = {v: (f"x_{arm}" if a == arm else a) for v, a in m.arms.items()}
        alone = [e[0] for e in entries if entry_is_red(m, e)]
        print(f"  {desc}：单测 {'过' if test_passes(m, entries)[0] else '红'}；单独判得红的调用 {alone}；"
              f"正式跑变了的驱动 {[g[0] for g in production_gaps(base, m, drivers)]}")


    print("== 点名的一行写法（锚点在源码里各命中几次、单测过不过、正式跑少了对照的驱动、按 --profile 并起来整档少的）")
    named = []
    m = base.copy()
    for v in variants:
        if family[v] == A:
            m.family[v] = L
    named.append(("第 6193 行 `| Self::J6QnSplitByI812 => Family::Adversarial,` 换成 `=> Family::LegalHistory,`",
                  "            | Self::J6QnSplitByI812 => Family::Adversarial,\n", m))
    m = base.copy()
    for v in variants:
        if family[v] == M:
            m.family[v] = L
    named.append(("第 6197 行 `| Self::MultiDiskAppliedSetAndLandedVersion => Family::MultiDisk,` 换成 `=> Family::LegalHistory,`",
                  "            | Self::MultiDiskAppliedSetAndLandedVersion => Family::MultiDisk,\n", m))
    m = base.copy()
    m.every_family = set()
    named.append(("第 6203–6206 行 `is_judged_in_every_family` 的 matches! 换成 `false`",
                  "            Self::TableComparisonSeesADifferentTable | Self::TableComparisonSeesADifferentRow\n        )\n", m))
    m = base.copy()
    m.filter_family = L
    named.append(("第 7758 行 `&criterion.family()` 换成 `&Family::LegalHistory`",
                  "        if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {\n", m))
    for desc, anchor, mutant in named:
        ok, _ = test_passes(mutant, entries)
        lost = [g for g in production_gaps(base, mutant, drivers) if g[1]]
        print(f"  {desc}：锚点命中 {src.count(anchor.replace(chr(92) + 'n', chr(10)))} 次；单测 {'过' if ok else '红'}；"
              f"少了对照的驱动 {len(lost)} 条 {[g[0] for g in lost]}；整档少的 {union_losses(base, mutant, drivers)}")

    print("== 改法甲（被攻过零轮，只在这份模型上量过）：单测改成照正式跑的 (档, 族) 逐条钉，期望集写死成今天那条驱动产物里的判据集合")
    fix_entries = [(name, families, p, [v for v in variants if labels[v] in artifact_controls(repo, artifact)])
                   for name, families, p, artifact in drivers]
    print(f"   钉 {len(fix_entries)} 个 (档, 族)；原样 {'过' if test_passes(base, fix_entries)[0] else '红'}")
    groups = {"甲 分族挪动": family_moves, "乙 族过滤": filters, "丙 盘次序 / 奇偶": guards,
              "C4 三行": [(d, (lambda a, n: (lambda mm: (mm.guard.__setitem__(a, n), mm)[1])(base.copy()))(arm, new))
                         for d, (arm, new) in rows.items()]}
    for title, mutants in groups.items():
        escaped = [d for d, mm in mutants if test_passes(mm, entries)[0] and any(g[1] for g in production_gaps(base, mm, drivers))]
        still = [d for d in escaped if test_passes(dict(mutants)[d], fix_entries)[0]]
        print(f"   {title}：今天的单测放过且正式跑少了对照的 {len(escaped)} 个，改法甲下仍放过 {len(still)} 个 {still}")
    collided = [mm for _, mm in collisions]
    print(f"   丁 标签相撞：改法甲按标签比时放过 {sum(test_passes(mm, fix_entries)[0] for mm in collided)} / {len(collided)}"
          f"（改用 BTreeSet<Criterion> 比，标签相撞不改集合；两步那一例改法甲下 {'过' if test_passes(combo, fix_entries)[0] else '红'}）")


if __name__ == "__main__":
    main()
