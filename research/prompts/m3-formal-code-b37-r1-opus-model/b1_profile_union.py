#!/usr/bin/env python3
"""第 37 批代码轮第一轮攻方腿（Opus）的 B1 / B4 模型：只建「哪一档跑哪几条对照」这一层的逻辑，不跑装置、不编 Rust。

读冻结副本里的 e165 装置源码：
  1. 核 `is_judged_under` 六条臂与 judges_* 五个谓词的原文还是模型里写死的那样（对不上就退 3，模型作废）；
  2. 从单测 `every_criterion_has_a_positive_control_and_every_control_is_red` 的正文里抽出它调 `positive_control_readings` 的每一处
     （族、档、并进哪一份读数）；
  3. 拿「只删不加」的变异（某条臂在某几档上从判变成不判）逐个过一遍：这条单测放不放过、replay.sh 里的正式跑会不会有一条判据被判而没有对照；
  4. 把 `crates/mutations.tsv` 里点名这条单测的几行逐条施加在单测正文上，再抽一遍档，看并起来的读数缺哪几条判据。
对照读数一律当「在判它的那一档里算就红」（模型只管覆盖，不管数值）；这是这份模型的限度。

用法：b1_profile_union.py <冻结副本根> <mutations.tsv>
"""
import itertools
import re
import sys
from pathlib import Path

L, A, M = "L", "A", "M"
# (标签, 族, 臂)；次序同 Criterion 枚举（e165 源码 enum Criterion）。
CRITERIA = [
    ("J0", L, "base"), ("J1", L, "base"), ("J2", L, "base"), ("J3", L, "base"), ("J4", L, "base"),
    ("J5a", L, "base"), ("J5b", L, "base"), ("J5a_vs_J5b", L, "base"), ("J6", L, "base"),
    ("IA1", L, "base"), ("IA2", L, "base"),
    ("A0", A, "base"), ("AS_whole_table", A, "base"), ("AS_consecutive_gap_continues", A, "base"),
    ("AS_consecutive_gap_stops", A, "base"), ("A_mutual", A, "base"), ("A_structure", A, "base"), ("A_limit", A, "base"),
    ("J6Qn", L, "j6qn"),
    ("J7a_instance_table", L, "j7"), ("J7a_allocation_record_tree_root", L, "j7"), ("J7a_filesystem_identifier", L, "j7"),
    ("J7b", L, "j7"), ("J7c", L, "j7"),
    ("J8a", L, "j8"), ("J8a_unmount_marker", L, "j8"), ("J8b", L, "j8"), ("J8c", L, "j8"),
    ("multi_disk_any_copy", M, "j6qn"), ("J1_multi_disk", M, "j6qn"), ("J4_multi_disk", M, "j6qn"), ("J5_multi_disk", M, "j6qn"),
    ("T_table", L, "t"), ("T_row", L, "t"),
    ("J6Qn_i812_split", A, "split"),
]
ALL_LABELS = [c[0] for c in CRITERIA]
EVERY_FAMILY = {"T_table", "T_row"}  # is_judged_in_every_family

# 判据集 → judges_* 谓词（源码 judges_j6qn / judges_j7 / judges_j8 / splits_j6qn_by_i812）。
PRED = {
    "j6qn": {"R1": False, "R2": True, "R3": True, "R5": True},
    "j7": {"R1": False, "R2": True, "R3": True, "R5": False},
    "j8": {"R1": False, "R2": False, "R3": False, "R5": True},
    "split": {"R1": False, "R2": False, "R3": True, "R5": True},
}
# R1 档不认 --table（split_off_the_profile_options 里 "r1" 带 --table 报错），所以没有 r1+both 这一档。
CLASSES = [("R1", False)] + [(s, both) for s in ("R2", "R3", "R5") for both in (False, True)]


def original_arm(arm, cls):
    criteria_set, both = cls
    if arm == "base":
        return True
    if arm == "t":
        return both
    return PRED[arm][criteria_set]


def controls(families, cls, judged):
    """positive_control_readings：族过滤（T 两条每族都带）+ is_judged_under。返回有对照的判据标签集合。"""
    return {lab for lab, fam, arm in CRITERIA
            if (fam in families or lab in EVERY_FAMILY) and judged(arm, cls)}


def verdict_judged(families, cls):
    """正式跑里判据被判的集合：域的判定行直接用 judges_* / compares（emit_the_verdict_of_a_domain），不经 is_judged_under。"""
    return controls(families, cls, original_arm)


# replay.sh 里起 e165 的正式跑（第 331–355 行那几条驱动；r3 root_choice、r4 root_choice_faults 不在这张判据表里，空集）。
FORMAL_RUNS = [
    ("driver_e165 (r1, legal)", ("R1", False), {L}),
    ("driver_e165_light (r1, legal)", ("R1", False), {L}),
    ("driver_e165_adversarial (r1)", ("R1", False), {A}),
    ("driver_e165_segment2 (r2, segment2, L+A)", ("R2", False), {L, A}),
    ("driver_e165_r2_segment3_multi_disk", ("R2", False), {M}),
    ("driver_e165_r2_segment4_table_both_legal", ("R2", True), {L}),
    ("driver_e165_r2_segment4_table_both_adversarial", ("R2", True), {A}),
    ("driver_e165_r2_segment4_table_both_multi_disk", ("R2", True), {M}),
    ("driver_e165_r2_segment4_reversed_disk_order_legal", ("R2", True), {L}),
    ("driver_e165_r2_segment4_reversed_disk_order_multi_disk", ("R2", True), {M}),
    ("driver_e165_r3_segment1_adversarial", ("R3", False), {A}),
    ("driver_e165_r5_segment3_legal", ("R5", False), {L}),
    ("driver_e165_r5_segment3_adversarial", ("R5", False), {A}),
    ("driver_e165_r5_segment3_multi_disk", ("R5", False), {M}),
    ("driver_e165_r5_segment3_odd_marker_legal", ("R5", False), {L}),
    ("driver_e165_r5_segment3_odd_marker_adversarial", ("R5", False), {A}),
]

SOURCE_LINES_THAT_MUST_BE_PRESENT = [
    "            Self::J6QnSplitByI812 => run_profile.splits_j6qn_by_i812(),",
    "                run_profile.compares_the_two_record_tables()",
    "            | Self::J7cUnmountMarkerWhenNothingIsApplied => run_profile.judges_j7(),",
    "            | Self::J8cNothingAppliedReturnsTheChosenRoot => run_profile.judges_j8(),",
    "            | Self::MultiDiskAppliedSetAndLandedVersion => run_profile.judges_j6qn(),",
    "            | Self::ALimit => true,",
    "            CriteriaSet::RegisteredBeforeR2 | CriteriaSet::WithJ6QnAndJ8SplitByI812 => false,",
    "            CriteriaSet::WithJ6QnAndJ8SplitByI812 => true,",
    "            CriteriaSet::RegisteredBeforeR2 | CriteriaSet::WithJ6QnAndJ7 => false,",
    "        matches!(self.table_source, RecordTableSource::Both)",
    "        if !criterion.is_judged_under(run_profile) {",
    "        if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {",
]

TEST_NAME = "fn every_criterion_has_a_positive_control_and_every_control_is_red()"
FAMILY_VARS = {"&families": {L, A}, "&all_families": {L, A, M},
               "&[Family::LegalHistory, Family::Adversarial, Family::MultiDisk]": {L, A, M}}


def test_body(source):
    start = source.index(TEST_NAME)
    end = source.index("\n    }\n", start)
    return source[start:end]


def profile_of(expression):
    expression = re.sub(r"\s+", "", expression)
    both = ".with_table_source(RecordTableSource::Both)" in expression
    for token, criteria_set in (("RunProfile::R1", "R1"), ("RunProfile::r2(", "R2"),
                                ("RunProfile::r3(", "R3"), ("RunProfile::r5(", "R5")):
        if expression.startswith(token):
            return (criteria_set, both)
    raise SystemExit(f"认不出的档：{expression}")


def calls_in(body):
    """单测里每一处 positive_control_readings 调用：(并进哪一份, 族, 档)。"""
    out = []
    for match in re.finditer(r"(let r1_readings =|let r2_readings =|let mut readings =|readings\.extend\()", body):
        rest = body[match.end():]
        call = re.match(r"\s*positive_control_readings\(\s*(&[^,]*?|&\[[^\]]*\]),\s*(RunProfile::.*?),\s*thread_count,?\s*\)", rest, re.S)
        if not call:
            raise SystemExit(f"认不出的调用：{rest[:120]!r}")
        role = {"let r1_readings =": "r1", "let r2_readings =": "r2"}.get(match.group(1), "union")
        families = FAMILY_VARS[re.sub(r"\s+", " ", call.group(1)).strip()]
        out.append((role, families, profile_of(call.group(2))))
    return out


def test_passes(calls, judged, criteria_filter=None):
    """单测三段：r1 段每个读数红（模型里一律红）；r2 段没有拆行；并起来的读数罩住 Criterion::ALL。返回 (过不过, 缺的判据)。"""
    union = set()
    for role, families, cls in calls:
        got = controls(families, cls, judged)
        if criteria_filter is not None:
            got &= criteria_filter
        if role == "r2" and "J6Qn_i812_split" in got:
            return False, ["r2 段：读数里有 J6Qn_i812_split"]
        if role == "union":
            union |= got
    missing = [lab for lab in ALL_LABELS if lab not in union]
    return (not missing), missing


def formal_gaps(judged):
    gaps = []
    for name, cls, families in FORMAL_RUNS:
        lacking = sorted(verdict_judged(families, cls) - controls(families, cls, judged), key=ALL_LABELS.index)
        if lacking:
            gaps.append((name, lacking))
    return gaps


def describe_class(cls):
    return cls[0].lower() + ("+both" if cls[1] else "")


def main():
    root, mutations_path = Path(sys.argv[1]), Path(sys.argv[2])
    source_path = root / "crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"
    source = source_path.read_text()
    for line in SOURCE_LINES_THAT_MUST_BE_PRESENT:
        if source.count(line + "\n") < 1:
            print(f"✗ 源码里找不到模型依赖的一行：{line!r}")
            sys.exit(3)
    body = test_body(source)
    calls = calls_in(body)
    print("== 单测里调 positive_control_readings 的每一处（并进哪一份 / 族 / 档）")
    for role, families, cls in calls:
        print(f"  {role:5s} {''.join(sorted(families)):4s} {describe_class(cls)}")
    ok, missing = test_passes(calls, original_arm)
    print(f"== 原样：单测 {'过' if ok else '红'}；缺 {missing}")
    print(f"== 原样：正式跑里被判而没有对照的 = {formal_gaps(original_arm)}")
    # 每一档各自罩哪几条（族取 L+A+M；R1 档只能 L+A）。
    print("== 每一档（全族）有对照的判据数，与哪几条三档并集里有、这一档没有")
    for cls in CLASSES:
        fams = {L, A} if cls[0] == "R1" else {L, A, M}
        got = controls(fams, cls, original_arm)
        print(f"  {describe_class(cls):8s} {len(got):2d} 条；没有：{[l for l in ALL_LABELS if l not in got]}")
    # 冗余：并集只要 r5 与 r3+both 两档就罩全。
    two = controls({L, A, M}, ("R5", False), original_arm) | controls({L, A, M}, ("R3", True), original_arm)
    r3_direct = controls({L, A, M}, ("R3", False), original_arm)
    print(f"== r5 ∪ r3+both 罩全 {len(two)}/{len(ALL_LABELS)}；r3 直接建表那一档有、而 r3+both 没有的 = "
          f"{sorted(r3_direct - controls({L, A, M}, ('R3', True), original_arm))}")

    # 只删不加的变异：一条臂在某几档上从「判」变成「不判」。
    arms = ["base", "j6qn", "j7", "j8", "t", "split"]
    total = caught = escaped_with_gap = escaped_without_gap = 0
    caught_by_fix = escaped_caught_by_fix = 0
    examples = []
    fix_profiles = [cls for cls in CLASSES]
    for arm in arms:
        true_classes = [cls for cls in CLASSES if original_arm(arm, cls)]
        for size in range(1, len(true_classes) + 1):
            for dropped in itertools.combinations(true_classes, size):
                dropped = set(dropped)

                def judged(a, cls, arm=arm, dropped=dropped):
                    return original_arm(a, cls) and not (a == arm and cls in dropped)

                total += 1
                ok, _ = test_passes(calls, judged)
                gaps = formal_gaps(judged)
                # 改法（被攻过零轮）：每一档（全族）各自的对照集等于按登记写死的那一份（这里写死的就是 original_arm）。
                fix_red = any(controls({L, A} if c[0] == "R1" else {L, A, M}, c, judged)
                              != controls({L, A} if c[0] == "R1" else {L, A, M}, c, original_arm) for c in fix_profiles)
                if not ok:
                    caught += 1
                elif gaps:
                    escaped_with_gap += 1
                    escaped_caught_by_fix += fix_red
                    if len(examples) < 6 and size <= 3:
                        examples.append((arm, sorted(describe_class(c) for c in dropped), gaps))
                else:
                    escaped_without_gap += 1
                if gaps and fix_red:
                    caught_by_fix += 1
    print(f"== 只删不加的变异：共 {total} 个；单测红 {caught}；单测放过且某条正式跑有判据没对照 {escaped_with_gap}；"
          f"单测放过、正式跑也不缺 {escaped_without_gap}")
    print(f"== 单测放过且正式跑有缺口的 {escaped_with_gap} 个里，被「每档各自对照集钉死」的改法判红的 {escaped_caught_by_fix} 个"
          f"（正式跑有缺口的变异全体里判红 {caught_by_fix} 个）")
    # 改法只钉单测今天跑的那几档够不够：分两种钉法各数一遍。
    family_of = lambda c: {L, A} if c[0] == "R1" else {L, A, M}
    for pin_name, pins in (("钉单测今天跑的 r1、r2、r3、r5、r3+both", [("R1", False), ("R2", False), ("R3", False), ("R5", False), ("R3", True)]),
                           ("钉正式跑用的 r1、r2、r2+both、r3、r5", [("R1", False), ("R2", False), ("R2", True), ("R3", False), ("R5", False)])):
        escaped = pinned_red = 0
        for arm in arms:
            true_classes = [cls for cls in CLASSES if original_arm(arm, cls)]
            for size in range(1, len(true_classes) + 1):
                for dropped in itertools.combinations(true_classes, size):
                    dropped = set(dropped)

                    def judged(a, cls, arm=arm, dropped=dropped):
                        return original_arm(a, cls) and not (a == arm and cls in dropped)

                    ok, _ = test_passes(calls, judged)
                    if ok and formal_gaps(judged):
                        escaped += 1
                        pinned_red += any(controls(family_of(c), c, judged) != controls(family_of(c), c, original_arm) for c in pins)
        print(f"   {pin_name}：放过且有缺口的 {escaped} 个里判红 {pinned_red} 个")
    for arm, dropped, gaps in examples:
        print(f"  例：臂 {arm} 在 {dropped} 上不判 ⇒ 单测过；正式跑缺口 {gaps[0][0]}: {gaps[0][1]}"
              + (f" 等 {len(gaps)} 条驱动" if len(gaps) > 1 else ""))
    # 点名的单一写法：基础那一臂 `=> true` 改成 `=> !run_profile.judges_j8()`；J6Qn 与多盘那一臂改成 judges_j8()；T 那一臂加 && splits。
    named = {
        "基础臂 `=> true` → `=> !run_profile.judges_j8()`": ("base", lambda c: not PRED["j8"][c[0]]),
        "基础臂 `=> true` → `=> !run_profile.judges_j7()`": ("base", lambda c: not PRED["j7"][c[0]]),
        "J6Qn/多盘臂 `judges_j6qn()` → `judges_j8()`": ("j6qn", lambda c: PRED["j8"][c[0]]),
        "T 臂 `compares…()` → `compares…() && splits_j6qn_by_i812()`": ("t", lambda c: c[1] and PRED["split"][c[0]]),
        "J7 臂 `judges_j7()` → `judges_j7() && compares…()`": ("j7", lambda c: PRED["j7"][c[0]] and c[1]),
    }
    print("== 点名的几种写法")
    for name, (arm, new_pred) in named.items():
        def judged(a, cls, arm=arm, new_pred=new_pred):
            return new_pred(cls) if a == arm else original_arm(a, cls)
        ok, missing = test_passes(calls, judged)
        gaps = formal_gaps(judged)
        print(f"  {name}：单测 {'过' if ok else '红，缺 ' + str(missing)}；正式跑缺口 {len(gaps)} 条驱动"
              + (f"，第一条 {gaps[0][0]} 缺 {gaps[0][1]}" if gaps else ""))

    # B4：变异表里点名这条单测的行，施加在单测正文上再抽档。
    print("== crates/mutations.tsv 里点名这条单测的行")
    for number, row in enumerate(mutations_path.read_text().splitlines(), start=1):
        fields = row.split("\t")
        if len(fields) != 6 or fields[5] != "every_criterion_has_a_positive_control_and_every_control_is_red":
            continue
        original, replacement = fields[2].replace("\\n", "\n"), fields[3].replace("\\n", "\n")
        hits = source.count(original)
        mutated_source = source.replace(original, replacement, 1)
        mutated_calls = calls_in(test_body(mutated_source))
        if "Criterion::ALL.iter().take(1)" in replacement:
            ok, missing = test_passes(mutated_calls, original_arm, criteria_filter={ALL_LABELS[0]})
        elif "Criterion::J7cUnmountMarkerWhenNothingIsApplied" in replacement:
            ok, missing = test_passes(mutated_calls, lambda a, c: original_arm(a, c), criteria_filter=set(ALL_LABELS) - {"J7c"})
        else:
            ok, missing = test_passes(mutated_calls, original_arm)
        print(f"  第 {number} 行「{fields[0]}」：原文命中 {hits} 次；施加后单测 {'过（变异活）' if ok else '红'}；缺 {missing}")
    # 单测正文自己的几种改坏（不是 crates 变异罩得住的那一类）。
    print("== 单测正文自己改坏")
    first_r3 = "        let mut readings = positive_control_readings(\n            &all_families,\n            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),\n"
    variants = {
        "第一档 r3 直接建表换成 r5": first_r3.replace("RunProfile::r3(", "RunProfile::r5("),
        "第一档 r3 直接建表换成 r3+both": first_r3.replace(
            "RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),\n",
            "RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence).with_table_source(RecordTableSource::Both),\n"),
    }
    for name, replacement in variants.items():
        hits = source.count(first_r3)
        mutated_calls = calls_in(test_body(source.replace(first_r3, replacement, 1)))
        ok, missing = test_passes(mutated_calls, original_arm)
        print(f"  {name}：原文命中 {hits} 次；单测 {'过' if ok else '红'}；缺 {missing}")


if __name__ == "__main__":
    main()
