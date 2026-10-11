#!/usr/bin/env python3
"""第 37 批代码轮第三轮攻方腿（Opus）的 D1 / D3 / D4 模型：只建「一次跑出哪几条对照」这一层，不编 Rust、不跑装置。

读（都只读）：
  1. 冻结副本（第三轮）里的 e165 装置：判据枚举与标签、`Criterion::family`、`is_judged_in_every_family`、`is_judged_under` 各臂、
     改后单测的 14 个单元（名字、族、档的写法、写死的集合、注释里点的 replay.sh 行与产物名）、`main` 起对照那一行；
  2. 第二轮冻结副本里的单测（五个调用、按标签并起来比），只用来判 D4「两行变异在第二轮单测下放过」；
  3. 仓里 scripts/experiments/replay.sh 第 699–880 行直接起装置的驱动（参数）、登记表里它们对的产物，与那些产物的
     `name=control` 判据集合、`name=verdict` 判定行（拿判定行独立核「这条判据在这一跑里判了」）。
对照读数一律当「跑了就红」，只管判据一级（读数一级不在这一轮的攻击面里）。
用法：d13_units.py <第三轮冻结副本根> <第二轮冻结副本根> <仓根>
"""
import re
import sys
from pathlib import Path

L, A, M = "L", "A", "M"
FAM = {"LegalHistory": L, "Adversarial": A, "MultiDisk": M}
SRC = "crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"
CALL_SITE = "        positive_control_readings(&selection.families, run_profile, thread_count);"
FILTER = "        if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {"
TEST_CALL = "            let readings = positive_control_readings(&[family], run_profile, thread_count);"
PRED = {
    "judges_j6qn": lambda p: p[0] in ("r2", "r3", "r5"),
    "judges_j7": lambda p: p[0] in ("r2", "r3"),
    "judges_j8": lambda p: p[0] == "r5",
    "splits_j6qn_by_i812": lambda p: p[0] in ("r3", "r5"),
    "compares_the_two_record_tables": lambda p: p[1] == "both",
    "base": lambda p: True,
}
# 判定行里证明「这一跑判了这条判据」的字段（name=verdict、不是 scope=run 的行）；多盘四条只认 M、M6、MC 域的行
EVIDENCE = {
    "J0": "j0_violations=", "J1": "j1_violations=", "J2": "j2_violations=", "J3": "j3_violations=", "J4": "j4_violations=",
    "J5a": "today_j5a_violations=", "J5b": "today_j5b_violations=", "J5a_vs_J5b": "today_j5a_violations=",
    "J6": "j6_premise_holds_violations=", "IA1": "ia1_violations=", "IA2": "ia2_violations=",
    "A0": "a0_violations=", "AS_whole_table": "as_whole_table_violations=",
    "AS_consecutive_gap_continues": "as_consecutive_gap_continues_violations=",
    "AS_consecutive_gap_stops": "as_consecutive_gap_stops_violations=",
    "A_mutual": "readings_differing_whole_table_vs_consecutive_gap_continues=", "A_structure": "a_structure_violations=",
    "A_limit": "a_limit_violations=", "J6Qn": "j6qn_premise_holds_violations=",
    "J6Qn_i812_split": "j6qn_premise_holds_i812_holds_violations=",
    "J7a_instance_table": "j7a_instance_table_violations=", "J7a_allocation_record_tree_root": "j7a_allocation_record_tree_root_violations=",
    "J7a_filesystem_identifier": "j7a_filesystem_identifier_violations=", "J7b": "j7b_violations=", "J7c": "j7c_violations=",
    "J8a": "j8a_instance_table_violations=", "J8a_unmount_marker": "j8a_unmount_marker_violations=", "J8b": "j8b_violations=",
    "J8c": "j8c_violations=", "T_table": "tables_differ=", "T_row": "rows_differ=",
    "multi_disk_any_copy": "j4_violations=", "J1_multi_disk": "j1_violations=", "J4_multi_disk": "j4_violations=",
    "J5_multi_disk": "today_j5a_violations=",
}
MULTI_DISK_LABELS = {"multi_disk_any_copy", "J1_multi_disk", "J4_multi_disk", "J5_multi_disk"}


def between(text, start, end):
    i = text.index(start)
    return text[i:text.index(end, i)]


def die(code, msg):
    print(msg)
    sys.exit(code)


class Table:
    """源码里的判据表与 positive_control_readings 那段循环；改坏 = 把改过的源码整份重读一遍。"""

    def __init__(self, src):
        block = between(src, "    enum Criterion {", "\n    }\n}")
        pairs = re.findall(r"(\w+) => \"([^\"]+)\",", block)
        self.variants, self.labels = [v for v, _ in pairs], dict(pairs)
        fam_block = between(src[src.index("impl Criterion {"):], "    const fn family(self) -> Family {", "    const fn is_judged_in_every_family")
        self.family, pending = {}, []
        for t in re.finditer(r"Self::(\w+)|=> Family::(\w+)", fam_block):
            if t.group(1):
                pending.append(t.group(1))
            else:
                self.family.update({v: FAM[t.group(2)] for v in pending})
                pending = []
        every_block = between(src, "    const fn is_judged_in_every_family(self) -> bool {", "\n    }\n")
        self.every = set(re.findall(r"Self::(\w+)", every_block))
        arm_block = between(src, "    const fn is_judged_under(self, run_profile: RunProfile) -> bool {", "\n    }\n}")
        self.arms, pending = {}, []
        for t in re.finditer(r"Self::(\w+)|=> (?:\{\s*)?([^,{}]+?)\s*(?:,|\})", arm_block):
            if t.group(1):
                pending.append(t.group(1))
            else:
                self.arms.update({v: rust_to_python(t.group(2)) for v in pending})
                pending = []
        pcr = between(src, "fn positive_control_readings(", "\n}\n")
        loop = re.search(r"for criterion in Criterion::ALL(?:\.iter\(\)\.take\((\d+)\))? \{", pcr)
        self.take = int(loop.group(1)) if loop.group(1) else None
        self.skips = [rust_to_python(c) for c in re.findall(r"if (.*?) \{\n\s*continue;", pcr, re.S)]
        if set(self.family) != set(self.variants) or set(self.arms) != set(self.variants) or len(self.skips) != 2:
            die(3, "✗ 判据表或对照循环读出来的形状对不上，模型作废")

    def judged(self, v, prof):
        return eval(self.arms[v], {"P": PRED, "p": prof})

    def controls(self, families, prof):
        """positive_control_readings：逐行过两道 continue，交回有对照的判据标签集合。families 是有序的列表或集合。"""
        families = list(families) if not isinstance(families, list) else families
        out = set()
        for v in self.variants[: self.take]:
            env = {"P": PRED, "p": prof, "families": families, "v": v, "fam": self.family.get,
                   "every": self.every, "judged": lambda c: self.judged(c, prof)}
            if any(eval(cond, env) for cond in self.skips):
                continue
            out.add(self.labels[v])
        return out


def rust_to_python(expr):
    """is_judged_under 的臂与对照循环里 continue 的条件：只认这几种写法，认不出的整个模型作废。"""
    e = " ".join(expr.split())
    subs = [(r"families\.contains\(&criterion\.family\(\)\)", "(fam(v) in families)"),
            (r"families\.contains\(&Family::(\w+)\)", lambda m: f"({FAM[m.group(1)]!r} in families)"),
            (r"families\[0\] == criterion\.family\(\)", "(families[:1] == [fam(v)])"),
            (r"criterion\.is_judged_in_every_family\(\)", "(v in every)"),
            (r"criterion\.is_judged_under\(run_profile\)", "judged(v)"),
            (r"\*criterion == Criterion::(\w+)", lambda m: f"(v == {m.group(1)!r})"),
            (r"run_profile\.(\w+)\(\)", lambda m: f"P[{m.group(1)!r}](p)"),
            (r"\btrue\b", "True"), (r"\bfalse\b", "False"), (r"&&", " and "), (r"\|\|", " or "), (r"!", " not ")]
    for pattern, repl in subs:
        e = re.sub(pattern, repl, e)
    if re.search(r"[A-Za-z_]+::|&|\.\w+\(", e.replace("P[", "")):
        die(3, f"✗ 认不出的写法：{expr}")
    return e


def profile_of_expression(expr):
    """单测里手写的档：RunProfile::R1 / RunProfile::r2(even) 加 with_table_source / with_reported_disk_order。"""
    m = re.search(r"RunProfile::(R1|r2|r3|r5)(?:\((\w+)\))?", expr)
    level = "r1" if m.group(1) == "R1" else m.group(1)
    parity = {"even": "even", "odd": "odd", "parity": "even", None: "even"}[m.group(2)]
    table = "both" if "RecordTableSource::Both" in expr else ("scan" if "RecordTableSource::Scanned" in expr else "direct")
    order = "reversed" if "SecondDeviceBeforeFirst" in expr else "normal"
    return (level, table, order, parity)


def profile_of_argv(words):
    """照 split_off_the_profile_options：档不给取 r1；奇偶、建表法、盘次序不给取 even、direct、normal。"""
    opt = {k: words[words.index(k) + 1] for k in ("--profile", "--marker-parity", "--table", "--disk-order") if k in words}
    level = {"r4": "r3"}.get(opt.get("--profile", "r1"), opt.get("--profile", "r1"))
    return (level, opt.get("--table", "direct"), opt.get("--disk-order", "normal"), opt.get("--marker-parity", "even"))


def families_of_argv(words):
    """照 parse_the_command_line：不给 --family 是合法、对抗两族；all 同；legal / adversarial / multi_disk 各一族。"""
    if "--family" not in words:
        return frozenset({L, A})
    return {"legal": frozenset({L}), "adversarial": frozenset({A}), "multi_disk": frozenset({M}),
            "all": frozenset({L, A})}.get(words[words.index("--family") + 1])


UNIT_RE = re.compile(
    r"// `scripts/replay\.sh` 第 ([^；]+) 行；产物 `([^`]+)`，(\d+) 条\n\s*\(\n\s*\"([^\"]+)\",\n"
    r"\s*Family::(\w+),\n(.*?),\n\s*&\[(.*?)\],\n\s*\),", re.S)


def parse_units(src, table):
    body = between(src, "fn every_criterion_has_a_positive_control_and_every_control_is_red()", "\n    }\n\n")
    units = []
    for lines, artifact, count, name, fam, prof, crit in UNIT_RE.findall(body):
        pinned = [table.labels[v] for v in re.findall(r"Criterion::(\w+)", crit)]
        if len(pinned) != int(count):
            die(3, f"✗ 单元 {name} 注释写 {count} 条、字面 {len(pinned)} 条")
        units.append({"name": name, "family": FAM[fam], "profile": profile_of_expression(prof),
                      "pinned": set(pinned), "replay_lines": [int(x) for x in re.findall(r"\d+", lines)], "artifact": artifact})
    declared = int(re.search(r"formal_run_units: \[\(&str, Family, RunProfile, &\[Criterion\]\); (\d+)\]", body).group(1))
    if len(units) != declared:
        die(3, f"✗ 单测声明 {declared} 个单元、模型读出 {len(units)} 个，模型作废")
    return units


def new_test(table, units):
    """改后单测：逐单元 PCR(&[族], 档) 与写死的集合比（按 Criterion 比，标签互不相同时与按标签比等价）；交回第一个红的单元或 None。"""
    covered = set()
    for u in units:
        got = table.controls({u["family"]}, u["profile"])
        if got != u["pinned"]:
            return u["name"]
        covered |= got
    missing = [table.labels[v] for v in table.variants if table.labels[v] not in covered]
    return f"罩全表缺 {missing}" if missing else None


def parse_r2_round_test(src_r2, table):
    """第二轮单测（改动全文 `-` 那几行）：五个调用，r1 取合法、对抗两族，余四个取三族；集合按标签比。"""
    body = between(src_r2, "fn every_criterion_has_a_positive_control_and_every_control_is_red()", "\n    }\n\n")
    arrays = {n: [table.labels[v] for v in re.findall(r"Criterion::(\w+)", inner)]
              for n, inner in re.findall(r"let (\w+) = \[(.*?)\];", body, re.S) if "Criterion::" in inner}
    base, j6md, j7, j8, t, split = (arrays[k] for k in ("criteria_of_every_profile", "j6qn_and_multi_disk", "j7", "j8",
                                                        "table_comparison", "split"))
    r2c = base + j6md + j7
    lam = {L, A, M}
    return [("r1", {L, A}, ("r1", "direct", "normal", "even"), set(base)),
            ("r2", lam, ("r2", "direct", "normal", "even"), set(r2c)),
            ("两份表都取的 r2", lam, ("r2", "both", "normal", "even"), set(r2c + t)),
            ("r3", lam, ("r3", "direct", "normal", "even"), set(r2c + split)),
            ("r5", lam, ("r5", "direct", "normal", "even"), set(base + j6md + split + j8))]


def r2_round_test(table, calls):
    covered = set()
    for name, fams, prof, reg in calls:
        got = table.controls(fams, prof)
        if got != reg:
            return name
        covered |= got
    return None if all(table.labels[v] in covered for v in table.variants) else "罩全表"


def parse_drivers(replay):
    """replay.sh 第 699–880 行里直接起装置（cargo run … e165_replay_bounded_enumeration）的驱动；root_choice 两族另走一套、不调对照，不算。"""
    lines = replay.split("\n")
    registered = dict(re.findall(r"^E165\|@(\w+)\|\|([^|]+)\|exact$", replay, re.M))
    drivers, current = [], None
    for number in range(699, 881):
        text = lines[number - 1]
        head = re.match(r"^(driver_e165\w*)\(\) \{", text)
        if head:
            current = head.group(1)
        m = re.search(r"--bin e165_replay_bounded_enumeration -- (.*?) \| grep", text)
        if m:
            words = m.group(1).split()
            fam = words[words.index("--family") + 1] if "--family" in words else "(不给)"
            if fam.startswith("root_choice"):
                continue
            drivers.append({"line": number, "driver": current, "words": words, "families": families_of_argv(words),
                            "profile": profile_of_argv(words), "artifact": registered.get(current)})
    return drivers


def artifact_rows(repo, name):
    return (repo / "research/results" / name).read_text(encoding="utf-8").split("\n")


def control_set(rows):
    return {m.group(1) for r in rows for m in [re.search(r"name=control criterion=(\S+)", r)] if m}


def judged_evidence(rows, label):
    """这一跑的判定行里有几行带这条判据的字段（不是 scope=run 的 name=verdict 行；多盘四条只数 M、M6、MC 域）。"""
    token = EVIDENCE[label]
    hits = [r for r in rows if "name=verdict " in r and "scope=run" not in r and f" {token}" in r]
    if label in MULTI_DISK_LABELS:
        hits = [r for r in hits if re.search(r" domain=(M|M6|MC) ", r)]
    return len(hits)


def main_controls(table, call_text, driver):
    """main 第 17769 行那一次调用（可被改坏）：交回这条驱动的正式跑打出的对照集。"""
    m = re.match(r"\s*positive_control_readings\((.*), (.*), thread_count\);$", call_text)
    fam_expr, prof_expr = m.group(1), m.group(2)
    order = {frozenset({L, A}): [L, A]}.get(driver["families"], sorted(driver["families"]))
    families = order if fam_expr == "&selection.families" else [FAM[x] for x in re.findall(r"Family::(\w+)", fam_expr)]
    prof = driver["profile"] if prof_expr == "run_profile" else profile_of_expression(prof_expr)
    return table.controls(families, prof)


def fmt(s):
    return "[" + ", ".join(sorted(s)) + "]"


def apply_row(src, old, new):
    old, new = old.replace("\\n", "\n"), new.replace("\\n", "\n")
    return src.count(old), src.count(new), src.replace(old, new, 1)


def main():
    root, root_r2, repo = (Path(x) for x in sys.argv[1:4])
    src = (root / SRC).read_text(encoding="utf-8")
    src_r2 = (root_r2 / SRC).read_text(encoding="utf-8")
    for needle in (CALL_SITE, FILTER, TEST_CALL):
        if src.count(needle) != 1:
            die(3, f"✗ 源码里这一行不是恰好一次：{needle.strip()}")
    table = Table(src)
    units = parse_units(src, table)
    drivers = parse_drivers((repo / "scripts/experiments/replay.sh").read_text(encoding="utf-8"))
    rows = {d["artifact"]: artifact_rows(repo, d["artifact"]) for d in drivers}

    print("== 核一：模型算的每条正式跑的对照集 = 那条驱动在 replay.sh 登记行里对的产物的 name=control 集合")
    bad = 0
    for d in drivers:
        got, want = main_controls(table, CALL_SITE, d), control_set(rows[d["artifact"]])
        bad += got != want
        print(f"  第 {d['line']} 行 {d['driver']} 族 {fmt(d['families'])} 档 {'/'.join(d['profile'])}：{len(got)} 条，产物 {len(want)} 条，"
              f"{'相同' if got == want else '不同 ' + fmt(got ^ want)}")
    if bad:
        die(4, f"✗ 核一 {bad} 条不同，模型作废")

    print("== D1：14 个单元对 replay.sh 直接起装置的驱动（root_choice 两族除外）")
    print(f"  驱动 {len(drivers)} 条，单元 {len(units)} 个")
    for u in units:
        mine = [d for d in drivers if d["families"] == frozenset({u["family"]}) and d["profile"] == u["profile"]]
        lines = [d["line"] for d in mine]
        registered = sorted({d["artifact"] for d in mine})
        cited_rows = artifact_rows(repo, u["artifact"])
        twin = {r: (u["artifact"].replace("-r1-legal-", "-r1-light-legal-") if "light" in r else u["artifact"]) for r in registered}
        same_bytes = all((repo / "research/results" / r).read_bytes() == (repo / "research/results" / twin[r]).read_bytes()
                         for r in registered)
        print(f"  {u['name']}：驱动行 {lines}（注释写 {u['replay_lines']}，{'对' if lines == u['replay_lines'] else '不对'}）；"
              f"写死 {len(u['pinned'])} 条 = 注释点的产物 {'是' if u['pinned'] == control_set(cited_rows) else '否'}"
              f" = 登记行对的产物 {'是' if all(u['pinned'] == control_set(rows[r]) for r in registered) else '否'}；"
              f"注释点的产物{'就是' if registered == [u['artifact']] else '不是'}登记行对的那一份"
              f"{'' if registered == [u['artifact']] else '（登记 ' + '、'.join(registered) + '；与 2026-10-04 那一份（轻档对轻档）逐字节' + ('相同' if same_bytes else '不同') + '）'}")
    orphans = [d["line"] for d in drivers if not any(d["families"] == frozenset({u["family"]}) and d["profile"] == u["profile"] for u in units)]
    print(f"  没有单元的驱动：{orphans or '无'}")
    print(f"  今天的源码上改后单测：{'过' if new_test(table, units) is None else '红在 ' + new_test(table, units)}")
    groups = {}
    for u in units:
        groups.setdefault((u["family"], frozenset(u["pinned"])), []).append(u["name"])
    print("  写死的集合相同的单元（单元里手写的档、奇偶、建表法、盘次序写成同组另一个的，集合不变、单测照过）：")
    for names in groups.values():
        if len(names) > 1:
            print("    " + " ≡ ".join(names))

    print("== D3-甲：main 里把命令行接到对照的那一次调用（单测不经它，单测直接调 positive_control_readings）")
    print(f"  单测的调用：{TEST_CALL.strip()}")
    print(f"  main 的调用：{CALL_SITE.strip()}")
    call_mutations = [
        ("族写死成合法一族", "&selection.families", "&[Family::LegalHistory]"),
        ("族写死成对抗一族", "&selection.families", "&[Family::Adversarial]"),
        ("族换成空", "&selection.families", "&[]"),
        ("档写死成 r1", "run_profile, thread_count", "RunProfile::R1, thread_count"),
        ("档写死成 r5", "run_profile, thread_count", "RunProfile::r5(MarkerParity::EvenVersionsWrittenByTheUnmountSequence), thread_count"),
    ]
    for title, old, new in call_mutations:
        mutated_call = CALL_SITE.replace(old, new, 1)
        _, _, mutated_src = apply_row(src, CALL_SITE, mutated_call)
        verdict = new_test(Table(mutated_src), parse_units(mutated_src, Table(mutated_src)))
        hit, judged_lost = [], 0
        for d in drivers:
            got, want = main_controls(table, mutated_call, d), control_set(rows[d["artifact"]])
            if got != want:
                lost = want - got
                ev = {c: judged_evidence(rows[d["artifact"]], c) for c in sorted(lost)}
                judged_lost += sum(1 for c in ev if ev[c] > 0)
                hit.append((d, lost, got - want, ev))
        print(f"  {title}（`{old}` → `{new}`）：单测 {'过' if verdict is None else '红'}；对照集变了的正式跑 {len(hit)} / {len(drivers)} 条；"
              f"少了的 (驱动, 判据) 里这一跑判定行照有的 {judged_lost} 对")
        for d, lost, extra, ev in hit[:3]:
            print(f"    第 {d['line']} 行 {d['driver']}：少 {len(lost)} 条 {fmt(lost)}；多 {fmt(extra) if extra else '无'}；"
                  f"少的每条在这一跑判定行里的行数 {min(ev.values()) if ev else '-'}–{max(ev.values()) if ev else '-'}")

    print("== D3 对照：判据表与对照循环这一侧，单测还放不放过「正式跑对照集变了」的改坏（只为说明缺口不在这一侧）")
    alternatives = ["base", "judges_j6qn", "judges_j7", "judges_j8", "splits_j6qn_by_i812", "compares_the_two_record_tables"]
    tried = escaped = 0
    examples = []
    groups_of_arm = {}
    for v in table.variants:
        groups_of_arm.setdefault(table.arms[v], []).append(v)
    candidates = []
    for v in table.variants:
        for fam in (L, A, M):
            if fam != table.family[v]:
                candidates.append((f"{table.labels[v]} 挪进 {fam} 族", ("family", v, fam)))
        candidates.append((f"{table.labels[v]} {'不再' if v in table.every else '改成'}每族都跑", ("every", v, None)))
    for arm, members in groups_of_arm.items():
        for alt in alternatives:
            for neg in ("", " not "):
                expr = f"{neg}P[{alt!r}](p)" if alt != "base" else ("True" if not neg else "False")
                if expr != arm:
                    candidates.append((f"臂 {arm} 换成 {expr.strip()}", ("arm", members, expr)))
    for title, (kind, who, what) in candidates:
        mutated = Table(src)
        if kind == "family":
            mutated.family[who] = what
        elif kind == "every":
            mutated.every ^= {who}
        else:
            mutated.arms.update({v: what for v in who})
        changed = [d["line"] for d in drivers if mutated.controls(d["families"], d["profile"]) != control_set(rows[d["artifact"]])]
        if not changed:
            continue
        tried += 1
        if new_test(mutated, units) is None:
            escaped += 1
            examples.append((title, changed))
    print(f"  改坏 {len(candidates)} 种里让某条正式跑的对照集变了的 {tried} 种，其中改后单测放过的 {escaped} 种")
    for title, changed in examples[:5]:
        print(f"    放过：{title} ⇒ 变了的驱动行 {changed}")

    print("== D3-乙：照产物钉——每个单元的产物里判定行有、这一跑却没有对照的判据（按判定行字段数，不是模型推的）")
    for u in units:
        d = next(d for d in drivers if d["families"] == frozenset({u["family"]}) and d["profile"] == u["profile"])
        judged = {lab for lab in EVIDENCE if judged_evidence(rows[d["artifact"]], lab) > 0}
        gap = judged - u["pinned"]
        cover = [w["name"] for w in units if w is not u and w["profile"][0] == u["profile"][0] and w["profile"][3] == u["profile"][3]
                 and gap and gap <= w["pinned"]]
        print(f"  {u['name']}：判了没对照 {len(gap)} 条{' ' + fmt(gap) if gap else ''}；同档同奇偶、一个单元就罩住它们的："
              f"{'、'.join(cover) if cover else ('—' if not gap else '无')}")

    pcr = between(src, "fn positive_control_readings(", "\n}\n")
    uses = [i for i, line in enumerate(pcr.split("\n")) if "thread_count" in line and "thread_count: usize" not in line]
    print(f"== D3 线程数：positive_control_readings 里 thread_count 只出现在 {len(uses)} 行，"
          f"{'都在两道 continue 之后' if all(i > pcr.split(chr(10)).index(next(l for l in pcr.split(chr(10)) if 'is_judged_under' in l)) for i in uses) else '有在 continue 之前的'}")
    print(f"   --only、--bounds、--max-n 不进这次调用（main 第 17769 行的参数只有族、档、线程数）")

    print("== D4：mutations.tsv 里点名这条单测的 e165 行，在改后单测与第二轮单测下")
    calls_r2 = parse_r2_round_test(src_r2, table)
    tsv = (root / "crates/mutations.tsv").read_text(encoding="utf-8").split("\n")
    for number in (1462, 1625, 3383, 3407, 3496, 3498, 3499):
        cols = tsv[number - 1].split("\t")
        old_count, new_count, mutated_src = apply_row(src, cols[2], cols[3])
        mutated = Table(mutated_src)
        red_new = new_test(mutated, parse_units(mutated_src, mutated))
        red_r2 = r2_round_test(mutated, calls_r2)
        formal = [d["line"] for d in drivers if mutated.controls(d["families"], d["profile"]) != control_set(rows[d["artifact"]])]
        print(f"  第 {number} 行 {cols[0][:28]}…：原文命中 {old_count}、替换文命中 {new_count}；改后单测 {'红在 ' + red_new if red_new else '过'}；"
              f"第二轮单测 {'红在 ' + red_r2 if red_r2 else '过'}；对照集变了的正式跑 {formal}")
    print(f"  第二轮单测在今天的源码上：{'过' if r2_round_test(table, calls_r2) is None else '红'}")

    print("== D4 挂钟：每个单元取它那条正式跑的 positive_controls 计时（产物 -timing.out，7 线程）")
    total = 0.0
    for u in units:
        timing = (repo / "research/results" / u["artifact"].replace(".out", "-timing.out")).read_text(encoding="utf-8")
        seconds = float(re.search(r"scope=positive_controls elapsed_seconds=([\d.]+) threads=(\d+)", timing).group(1))
        total += seconds
        print(f"  {u['name']}：{seconds:.3f} 秒")
    print(f"  合计 {total:.3f} 秒（单测里各单元串行跑，线程取本机 available_parallelism）")
    old_calls = [("r1", ["r1-legal-family", "r1-adversarial-family"]),
                 ("r2", ["r2-legal-family", "r2-adversarial-family", "r2-multi-disk"]),
                 ("两份表都取的 r2", ["r2-table-both-legal-family", "r2-table-both-adversarial-family", "r2-table-both-multi-disk"]),
                 ("r3（合法、多盘两族没有 r3 产物，借 r2 直接建表那两份的计时）", ["r3-adversarial-family", "r2-legal-family", "r2-multi-disk"]),
                 ("r5", ["segment3-r5-legal-family", "segment3-r5-adversarial-family", "segment3-r5-multi-disk"])]
    old_total = 0.0
    for name, parts in old_calls:
        seconds = 0.0
        for part in parts:
            stem = part if part.startswith("segment3-") else "segment1-" + part
            timing = (repo / f"research/results/e165-replay-bounded-enumeration-2026-10-04-r5-{stem}-timing.out").read_text(encoding="utf-8")
            seconds += float(re.search(r"scope=positive_controls elapsed_seconds=([\d.]+)", timing).group(1))
        old_total += seconds
        print(f"  第二轮单测的 {name} 调用（几族的计时相加）：{seconds:.3f} 秒")
    print(f"  第二轮单测合计 {old_total:.3f} 秒；改后 / 第二轮 = {total / old_total:.3f}")


if __name__ == "__main__":
    main()
