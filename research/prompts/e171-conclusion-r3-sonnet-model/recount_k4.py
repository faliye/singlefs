"""e171-conclusion-r3 正推腿（Sonnet）K4：第二轮判决引的数，从第二轮攻方模型目录 out/ 与 E171 产物里重数。
只读，不写仓。用法（仓根）：
  python3 research/prompts/e171-conclusion-r3-sonnet-model/recount_k4.py
数的东西：
  1 W 的 15 / 63 态与 76 条流 19 361 态（out/witness-consistency-all.out）
  2 61 个见证拒与第二截判 legal（out/refusal-judge-summary.out）
  3 RefFS 27 / 27 与 36 态（out/judge-summary.out）
  4 第 125 行 D = 38（out/drive-s4-m125.tsv 与产物 q1_denominator）
  5 O3 独有 29 张（产物 q2_row 逐行重数，与 q2_family 两行比）
"""
import collections
import os

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
OUT = os.path.join(ROOT, "research/prompts/e171-conclusion-r2-opus-model/out")
ARTIFACT = os.path.join(ROOT, "research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out")


def fields(line):
    return dict(kv.split("=", 1) for kv in line.split() if "=" in kv)


def section(title):
    print(f"\n== {title}")


section("1 W（witness-consistency-all.out）")
total_states = 0
labels = 0
w_by_row = collections.Counter()
base_states = 0
for line in open(os.path.join(OUT, "witness-consistency-all.out")):
    d = fields(line)
    labels += 1
    total_states += int(d["states"])
    w_by_row[d["label"]] += int(d["w_true"])
    if d["label"] in ("base1", "base2"):
        base_states += int(d["states"])
print(f"labels={labels} states={total_states}")
print("w_true_nonzero=" + str({k: v for k, v in w_by_row.items() if v}))
print("row242 w_true=%d row269 w_true=%d" % (w_by_row["r242new"] + w_by_row["r242fix"], w_by_row["r269new"] + w_by_row["r269fix"]))
print("baseline states=%d w_true=%d" % (base_states, w_by_row["base1"] + w_by_row["base2"]))

section("2 见证拒（refusal-judge-summary.out，core=s4）")
witness_reasons = {"AgainstTheRecordAtTheWitnessedCounter", "AgainstTheSelectedVersionsLastRecord", "Undecidable"}
torn_reason = "SomeSystemConfigurationSlotUnreadOrUnverified"
by_row = collections.Counter()
torn = collections.Counter()
verdicts = collections.Counter()
for line in open(os.path.join(OUT, "refusal-judge-summary.out")):
    d = fields(line)
    if d["core"] != "s4" or not d["mount1"].startswith("refused:"):
        continue
    reason = d["mount1"].split(":", 1)[1]
    row = d["label"][:4]
    verdicts[d["second_cut_verdict"]] += int(d["count"])
    if reason in witness_reasons:
        by_row[row] += int(d["count"])
    elif reason == torn_reason:
        torn[row] += int(d["count"])
    else:
        print("unclassified reason", reason)
print("witness_class_refusals_by_row=" + str(dict(by_row)) + " total=%d" % sum(by_row.values()))
print("torn_system_configuration_refusals_by_row=" + str(dict(torn)))
print("second_cut_verdicts_over_all_s4_refusals=" + str(dict(verdicts)))

section("3 RefFS（judge-summary.out）")
sums = collections.Counter()
for line in open(os.path.join(OUT, "judge-summary.out")):
    d = fields(line)
    if "judge_s4_v2" not in d["label"]:
        continue
    sums[(d["label"], d["o3_strict"], d["reffs"])] += int(d["count"])
for key, value in sorted(sums.items()):
    print(key, value)
content_red = sum(v for (l, o, r), v in sums.items() if l.startswith("content") and r == "red")
content_o3_red = sum(v for (l, o, r), v in sums.items() if l.startswith("content") and o == "not_in")
warmup_o3_red = sum(v for (l, o, r), v in sums.items() if l.startswith("warmup") and o == "not_in")
warmup_green = sum(v for (l, o, r), v in sums.items() if l.startswith("warmup") and o == "not_in" and r == "green")
print(f"warmup: o3_not_in={warmup_o3_red} of which reffs_green={warmup_green}")
print(f"content: o3_not_in={content_o3_red} reffs_red_total={content_red} reffs_red_while_o3_in={content_red - content_o3_red}")

section("4 第 125 行（drive-s4-m125.tsv 与产物）")
calls = []
for line in open(os.path.join(OUT, "drive-s4-m125.tsv")):
    parts = line.rstrip("\n").split("\t")
    if len(parts) > 3 and parts[1] == "call":
        calls.append((int(parts[2]), parts[3], parts[-1]))
print("calls with o2_full_persisted=I-3.11:", [c for c, _, v in calls if v == "o2_full_persisted=I-3.11"])
print("last call:", calls[-1])
for number, line in enumerate(open(ARTIFACT, encoding="utf-8"), 1):
    if line.startswith("E7RESULT name=q1_denominator ") or line.startswith("E7RESULT name=untouched_but_red "):
        print(number, line.strip()[:260])
d = fields([l for l in open(ARTIFACT, encoding="utf-8") if l.startswith("E7RESULT name=q1_denominator ")][0])
print("counted_D=%s self_crash=%s -> D with row 125 = %d; with 155,161 as well = %d" % (d["counted_D"], d["self_crash"], int(d["counted_D"]) + 1, int(d["counted_D"]) + 3))

section("5 Q2 O3 独有（产物 q2_row 逐行重数）")
rows = [fields(l) for l in open(ARTIFACT, encoding="utf-8") if l.startswith("E7RESULT name=q2_row ")]
counter = collections.Counter()
kinds = collections.defaultdict(set)
for d in rows:
    if d["o2"] == "green" and d["o3_strict"] == "not_in":
        counter[d["family"]] += 1
        kinds[d["family"]].add(d["bad_name"].split(":")[1] if d["family"] == "B" else d["bad_name"].split("-")[0])
print("o3_only_strict by family (recount from q2_row):", dict(counter), "total", sum(counter.values()))
print("patch kinds among them:", {k: sorted(v) for k, v in kinds.items()})
for number, line in enumerate(open(ARTIFACT, encoding="utf-8"), 1):
    if line.startswith("E7RESULT name=q2_family family=P ") or line.startswith("E7RESULT name=q2_family family=B "):
        d = fields(line)
        print(number, "family=%s rows=%s o3_only_strict=%s" % (d["family"], d["rows"], d["o3_only_strict"]))
