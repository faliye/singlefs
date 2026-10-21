#!/usr/bin/env python3
"""第三轮辩方腿（Q4）：前两轮判决第二节引的数，对着两轮四条腿交的输出文件与冻结副本现算一遍。

用法：python3 -I check-numbers.py --prompts <仓根>/research/prompts --frozen-r1 <第一轮冻结副本> --frozen-r2 <第二轮冻结副本> --repo <仓根>
每个检查打一行 `name=check id=<编号> source=<出处> expected=<判决或报告里的数> actual=<现算的数> result=ok|differ`；
末行 `name=check_total ok=<n> differ=<m>`。只读输入，不写任何文件。
"""
import argparse
import os
import re
import sys

checks = []


def add(identifier, source, expected, actual):
    checks.append((identifier, source, expected, actual, "ok" if str(expected) == str(actual) else "differ"))


def read(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        return handle.read()


def lines_of(path):
    return read(path).splitlines()


def kv_numbers(line):
    return dict(re.findall(r"(\w+)=(\S+)", line))


def line_has(path, number, needle):
    rows = lines_of(path)
    if number > len(rows):
        return False
    return needle in rows[number - 1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--prompts", required=True)
    parser.add_argument("--frozen-r1", required=True)
    parser.add_argument("--frozen-r2", required=True)
    parser.add_argument("--repo", required=True)
    args = parser.parse_args()
    p = args.prompts
    r1_opus = os.path.join(p, "m4-segment-code-r1-opus-model")
    r1_sonnet = os.path.join(p, "m4-segment-code-r1-sonnet-model")
    r2_opus = os.path.join(p, "m4-segment-code-r2-opus-model")
    r2_sonnet = os.path.join(p, "m4-segment-code-r2-sonnet-model")

    # ── 第一轮判决第二节 ──
    sweep = lines_of(os.path.join(r1_opus, "out/remount-sweep.txt"))
    add("r1-T5-cells", "r1-opus out/remount-sweep.txt 行数", 16, len(sweep))
    add("r1-T5-hits", "r1-opus out/remount-sweep.txt 里 ending=NewFinding 的格", 6, sum(1 for row in sweep if "ending=NewFinding" in row))
    rerun_opus_r1 = read(os.path.join(r1_opus, "tools/rerun.sh"))
    keep_2022 = re.search(r"ATTACK_SEED_INDEX=2022 .*?ATTACK_KEEP=([0-9,]+)", rerun_opus_r1, re.S).group(1)
    add("r1-T4-steps", "r1-opus tools/rerun.sh 种子下标 2022 的 ATTACK_KEEP 个数", 5, len(keep_2022.split(",")))
    i35 = read(os.path.join(r1_opus, "out/i35-seed2022.txt"))
    add("r1-T4-red", "r1-opus out/i35-seed2022.txt 含 I-3.5 红与 HIT true", True, ("RED Operation(4) I-3.5" in i35) and ("HIT true" in i35))
    keep_1055 = re.search(r"ATTACK_SEED_INDEX=1055 .*?ATTACK_KEEP=([0-9,]+)", rerun_opus_r1, re.S).group(1)
    add("r1-T6-steps", "r1-opus tools/rerun.sh 种子下标 1055 的 ATTACK_KEEP 个数", 13, len(keep_1055.split(",")))
    seed1055 = read(os.path.join(r1_opus, "out/compaction-seed1055-trace.txt"))
    add("r1-T6-ending", "r1-opus out/compaction-seed1055-trace.txt 的 ENDING 是整理发布对意图做的事不合", True, "这一次整理发布对意图做的事" in seed1055)
    expected = read(os.path.join(r1_sonnet, "out/expected.txt"))
    add("r1-T6-segs0-3310", "r1-sonnet out/expected.txt 签发开新段那格 segs0", True,
        "case=issue_publish_opens_a_new_segment segs0_written_in_the_intent=3310 accounting_row_11_written_by_the_issue_publish=3309" in expected)
    add("r1-T7-comparisons", "r1-sonnet out/expected.txt s01 的 checked_comparisons", True, "checked_comparisons=32000" in expected)
    add("r1-T8-harness-killers", "r1-sonnet out/expected.txt s03 变异 M2 下失败的用例数", True,
        "step=s03-mutation-m2-discard-every-noted-segment failed_tests=3" in expected)
    add("r1-T9-M4-not-killed", "r1-sonnet out/expected.txt s15a、s15b 变异 M4 下仍绿", True,
        "step=s15a-mutation-m4-cross-check exit=0" in expected and "step=s15b-mutation-m4-harness-plan-tests exit=0" in expected)
    add("r1-T11-4346", "r1-sonnet out/expected.txt s05a 红、s05b 30 条原有单测绿", True,
        "step=s05a-mutation-m3-model-probe exit=101" in expected and "test result: ok. 30 passed; 0 failed" in expected)
    t11o = read(os.path.join(r1_opus, "out/t11-original.txt"))
    t11m = read(os.path.join(r1_opus, "out/t11-mutated.txt"))
    add("r1-T11-orig-mutated", "r1-opus out/t11-original.txt 2 passed、t11-mutated.txt 1 failed", True,
        "2 passed; 0 failed" in t11o and "1 passed; 1 failed" in t11m)
    # 第一轮判决第一节、第二节引的行（第一轮冻结副本）
    f1 = args.frozen_r1
    for ident, rel, number, needle in [
        ("r1-anchor-T5", "crates/singlefs-core/src/mount.rs", 8187, "central_mapping_tree: current.central_mapping_tree.clone(),"),
        ("r1-anchor-T9", "crates/singlefs-core/src/compaction_plan.rs", 902, "span_slots: 1,"),
        ("r1-anchor-T1", "crates/singlefs-core/src/mounted_session.rs", 1390, "| UserChange::CreateSnapshot => {"),
        ("r1-anchor-T10", "crates/singlefs-core/src/recovery.rs", 2968, "盘上读来的坏条目当损坏处置，可写挂载拒"),
        ("r1-anchor-T2-a", "crates/singlefs-core/src/transaction.rs", 4843, "SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue"),
        ("r1-anchor-T2-b", "crates/singlefs-core/src/transaction.rs", 4994, "pub enum SnapshotLifetimeVerdict"),
        ("r1-anchor-T8-row", "crates/mutations.tsv", 4453, "回收记下的段不论整段可不可再分配都发 discard"),
        ("r1-anchor-T11-row", "crates/mutations.tsv", 4346, "快照模型对拍：「非空」不看快照条目"),
        ("r1-anchor-4491-row", "crates/mutations.tsv", 4491, "合取 7 被拒仍算空间不够"),
    ]:
        add(ident, f"第一轮冻结副本 {rel}:{number}", True, line_has(os.path.join(f1, rel), number, needle))
    # 第一轮判决 T2 缺项：SnapshotLifetime 在核心源码里的命中数（判决只说「只命中枚举与它的两个臂」）
    hits = 0
    for root, _dirs, files in os.walk(os.path.join(f1, "crates/singlefs-core/src")):
        for name in files:
            if name.endswith(".rs"):
                hits += sum(1 for row in lines_of(os.path.join(root, name)) if "SnapshotLifetime" in row)
    add("r1-T2-grep-hits", "第一轮冻结副本核心源码里含 SnapshotLifetime 的行数（r1 判决「只命中寿命判定的枚举与它的两个臂」）", 11, hits)

    # ── 第二轮判决第二节 ──
    s = lambda name: os.path.join(r2_sonnet, "out", name)
    def probe_rows(name):
        rows = [row for row in lines_of(s(name)) if row.startswith("PROBE")]
        not_completed = sum(1 for row in rows if "ending=Completed" not in row)
        found = [re.search(r"reds=(\d+)", row) for row in rows]
        reds = sum(int(m.group(1)) for m in found if m)
        red_cells = sum(1 for m in found if m and int(m.group(1)) > 0)
        return len(rows), not_completed, reds, red_cells
    n, nc, reds, cells = probe_rows("f1-r1-core.txt")
    add("r2-R1-scenarios", "r2-sonnet out/f1-r1-core.txt PROBE 行数", 49, n)
    add("r2-R1-not-completed", "r2-sonnet out/f1-r1-core.txt 没走完的格（判决写「24 格红」）", 24, nc)
    add("r2-R1-reds", "r2-sonnet out/f1-r1-core.txt 池级 checker 违例条数", 108, reds)
    add("r2-R1-cells-with-reds", "r2-sonnet out/f1-r1-core.txt 有违例的格数（判决写「24 格红」，按有池级 checker 违例的格读）", 24, cells)
    n, nc, reds, cells = probe_rows("f1-r2-core.txt")
    add("r2-R1-after-fix", "r2-sonnet out/f1-r2-core.txt 没走完的格 + 违例", "0+0", f"{nc}+{reds}")
    n, nc, reds, cells = probe_rows("f5-r1-core.txt")
    add("r2-R5-before", "r2-sonnet out/f5-r1-core.txt 没走完的格（判决「6/6 不合」）", 6, nc)
    n, nc, reds, cells = probe_rows("f5-r2-core.txt")
    add("r2-R5-after", "r2-sonnet out/f5-r2-core.txt 没走完 + 违例", "0+0", f"{nc}+{reds}")
    add("r2-R6-lines", "r2-sonnet out/f6-reference-filesystem-snapshot-sets.txt 行数（判决「20 行探针」）", 20,
        len(lines_of(s("f6-reference-filesystem-snapshot-sets.txt"))))
    arithmetic = read(s("arithmetic.txt"))
    add("r2-R3-boundary", "r2-sonnet out/arithmetic.txt 的 (3310,3310) 与 (3310,3309)", True,
        "(3310 vs 3310)=false" in arithmetic and "(3310 vs 3309)=true" in arithmetic)
    seeds = read(s("seeds.txt"))
    for seed in ("940015", "940105", "960004", "950093"):
        add(f"r2-R2-seed-{seed}", f"r2-sonnet out/seeds.txt 含 seed_offset={seed} 的 NewFinding", True,
            re.search(rf"seed_offset={seed} ending=NewFinding", seeds) is not None)
    add("r2-R1-seed950093-9-vs-11", "r2-sonnet out/seeds.txt 950093 一行含「上限 Some(9)」与「实现报上限 11」", True,
        re.search(r"seed_offset=950093.*Some\(9\).*11", seeds) is not None)
    t4a = lines_of(s("t4-snapshot-seeds-r1-checker.txt"))
    t4b = lines_of(s("t4-snapshot-seeds-r2-checker.txt"))
    add("r2-T4-seeds-before", "r2-sonnet out/t4-snapshot-seeds-r1-checker.txt 四个种子的 reds", "13,0,8,2",
        ",".join(re.search(r"reds=(\d+)", row).group(1) for row in t4a))
    add("r2-T4-seeds-after", "r2-sonnet out/t4-snapshot-seeds-r2-checker.txt 四个种子的 reds", "0,0,0,0",
        ",".join(re.search(r"reds=(\d+)", row).group(1) for row in t4b))

    # 攻方第二轮
    o = lambda name: os.path.join(r2_opus, "out", name)
    sweep2 = [row for row in lines_of(o("rollback-shapes-sweep.txt")) if row.startswith("CASE")]
    add("r2-R1-240-cells", "r2-opus out/rollback-shapes-sweep.txt CASE 行数", 240, len(sweep2))
    add("r2-R1-240-hits", "同上 hit=true 的格", 0, sum(1 for row in sweep2 if "hit=true" in row))
    empty = lines_of(o("empty-file-set-snapshots.txt"))
    cases = sum(int(re.search(r"cases=(\d+)", row).group(1)) for row in empty if row.startswith("SWEEP"))
    add("r2-R5-115", "r2-opus out/empty-file-set-snapshots.txt 两条 SWEEP 的 cases 之和", 115, cases)
    add("r2-R5-hits", "同上 hits 之和", 0, sum(int(re.search(r"hits=(\d+)", row).group(1)) for row in empty if row.startswith("SWEEP")))
    total_seeds = total_problems = 0
    by_weights = {}
    for name in ("random-summaries.txt", "random-summaries-small-devices-and-later.txt"):
        for row in lines_of(o(name)):
            if row.startswith("ATTACK_SUMMARY"):
                d = kv_numbers(row)
                total_seeds += int(d["seeds"])
                total_problems += int(d["problems"])
    add("r2-R8-2924", "r2-opus out/random-summaries*.txt 的 seeds 之和", 2924, total_seeds)
    add("r2-R8-problems", "同上 problems 之和（报告「原样模型上 15 段 + R7 那一形 2 段」）", 17, total_problems)
    # 310 段：with-rollback 种子下标 [400,555) 与 [600,755)
    summary_rows = [kv_numbers(row) for row in lines_of(o("random-summaries.txt")) if row.startswith("ATTACK_SUMMARY") and "挂着时整理夹着回退" in row]
    def in_range(row):
        start = int(row["seed_start"])
        return 400 <= start < 555 or 600 <= start < 755
    chosen = [row for row in summary_rows if in_range(row)]
    add("r2-R1-310-seeds", "r2-opus out/random-summaries.txt 里 with-rollback 种子下标 [400,555)∪[600,755) 的 seeds 之和", 310, sum(int(r["seeds"]) for r in chosen))
    add("r2-R1-310-problems", "同上 problems 之和（判决「310 段中 3 段」）", 3, sum(int(r["problems"]) for r in chosen))
    add("r2-R1-310-steps", "同上 steps_checked 之和（报告「12636 张」）", 12636, sum(int(r["steps_checked"]) for r in chosen))
    all_with_rollback = sum(int(r["seeds"]) for r in summary_rows)
    add("r2-R7-382", "同上文件里 with-rollback 的 seeds 之和（报告 R7「4 GiB 上 382 段里 1 段」）", 382, all_with_rollback)
    model_fix = read(o("model-rollback-relocations-before-and-after.txt"))
    sections = re.split(r"^== ", model_fix, flags=re.M)
    def section_sum(section, key):
        return sum(int(kv_numbers(row)[key]) for row in section.splitlines() if row.startswith("ATTACK_SUMMARY"))
    original = next(x for x in sections if x.startswith("原样模型"))
    fixed = next(x for x in sections if x.startswith("副本上改了模型回退那一步"))
    add("r2-R1-496-original", "r2-opus out/model-rollback-relocations-before-and-after.txt 原样模型 seeds/problems", "496/12", f"{section_sum(original, 'seeds')}/{section_sum(original, 'problems')}")
    add("r2-R1-496-fixed", "同上 改了模型回退那一步 seeds/problems", "496/0", f"{section_sum(fixed, 'seeds')}/{section_sum(fixed, 'problems')}")
    margins = [int(m.group(2)) - int(m.group(1)) for m in re.finditer(r"fixed_point_slots_written=(\d+) ckpt_cost=(\d+)", read(o("checkpoint-cost-margins.txt")))]
    add("r2-R7-margin", "r2-opus out/checkpoint-cost-margins.txt 余量最小/最大（判决「528–534 槽」）", "528-534", f"{min(margins)}-{max(margins)}")
    for seed, fname in (("297", "trace297.txt"), ("6318", "seed6318-small-device.txt"), ("476", "trace476.txt")):
        add(f"r2-R2-seed-{seed}", f"r2-opus out/{fname} 含 NewFinding", True, "NewFinding" in read(o(fname)))
    # 第二轮判决引的行（第二轮冻结副本与 kb）
    f2 = args.frozen_r2
    add("r2-anchor-model-2750", f"第二轮冻结副本 crates/singlefs-harness/src/model.rs:2750", True,
        line_has(os.path.join(f2, "crates/singlefs-harness/src/model.rs"), 2750, "let mut relocations_carried = previous.user_visible_root_relocations.as_ref().clone();"))
    kb = os.path.join(args.repo, ".claude/kb/decisions")
    add("r2-quote-D23-378", ".claude/kb/decisions/23-journal的角色与格式.md:378 含「管理员回退是挂着时的一次向前发布」与「不从 R_old 那棵账载入」", True,
        line_has(os.path.join(kb, "23-journal的角色与格式.md"), 378, "**管理员回退是挂着时的一次向前发布**") and line_has(os.path.join(kb, "23-journal的角色与格式.md"), 378, "不从 R_old 那棵账载入"))
    add("r2-quote-D3-146", ".claude/kb/decisions/03-空间分配.md:146 含「只拦前台写入类发布」", True,
        line_has(os.path.join(kb, "03-空间分配.md"), 146, "只拦前台写入类发布"))
    add("r1-quote-D8-108", ".claude/kb/decisions/08-核心索引结构.md:108 含「提示类」「不挡挂载」", True,
        line_has(os.path.join(kb, "08-核心索引结构.md"), 108, "**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载"))
    gc = os.path.join(p, "m4-gc-r1-main-verification.md")
    add("r2-quote-G3-34", "research/prompts/m4-gc-r1-main-verification.md:34 含「写路径改成以映射为准」", True, line_has(gc, 34, "写路径改成以映射为准"))
    add("r2-quote-G4-35", "同上:35 含「目标态到达与最后一批同发布删」（判决 G4 那一行，用户定合取 7 只拦前台）", True, line_has(gc, 35, "用户定合取 7 只拦前台写入类发布"))
    add("r2-quote-G5-36", "同上:36 含「按 64 槽对齐的整段发」", True, line_has(gc, 36, "按 64 槽对齐的整段发"))

    for ident, source, expected_value, actual, result in checks:
        print(f"name=check id={ident} source={source} expected={expected_value} actual={actual} result={result}")
    ok = sum(1 for c in checks if c[4] == "ok")
    print(f"name=check_total ok={ok} differ={len(checks) - ok}")


if __name__ == "__main__":
    sys.exit(main())
