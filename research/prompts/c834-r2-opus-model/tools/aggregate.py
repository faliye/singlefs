"""汇总原型的 c834_state 行：核每段（历史，读故障）的状态一个不落，再按（历史/读故障，状态类，候选，次序，判，结局）数状态。
例子序号取那一格里最小的序号（与件的先后无关，复跑逐行可比）。用法：python3 aggregate.py <输出文件…>"""
import collections, re, sys

expected = {}
seen = collections.defaultdict(set)
table = collections.Counter()
examples = {}
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8"):
        if line.startswith("name=c834_history"):
            fields = dict(re.findall(r"(\w+)=(\S+)", line))
            expected[fields["history"]] = int(fields["state_count"])
            continue
        if not line.startswith("name=c834_state"):
            continue
        fields = dict(re.findall(r"(\w+)=(\S+)", line))
        key = (fields["history"], fields["faults"])
        ordinal = int(fields["ordinal"])
        seen[key].add((ordinal, fields["order"]))
        per_device = [max(int(x) for x in slots.split("/")) for _, slots in (item.split(":") for item in fields["per_device"].split(","))]
        later = [x for x in fields["later_records"].strip("[]").split(",") if x]
        if len(set(per_device)) > 1:
            state_class = "number_on_one_device_only"
        elif per_device[0] == 1:
            state_class = "no_number_taken"
        elif int(fields["r_units_overwritten_in_state"]) > 0:
            state_class = "r_units_already_overwritten"
        elif later:
            state_class = "later_records_r_units_intact"
        else:
            state_class = "number_taken_no_record_r_units_intact"
        if fields["r_in_ring"] != "true":
            state_class += "_r_not_in_ring"
        for cell in fields["cands"].split(";"):
            candidate, verdict, outcome = cell.split(":", 2)
            if outcome.startswith("refused"):
                summary = "refused:" + outcome.split(":")[2]
            else:
                parts = dict(item.split("=", 1) for item in outcome.split(":")[1:] if "=" in item)
                summary = ("mounted_overwrote_r_units" if parts["new"] != "0" else "mounted_r_units_kept") + ":red=" + parts["red"]
            candidates = ["today", "k3", "k6", "pointer_r1", "pointer_per_kind", "always_refuse"] if candidate == "all" else [candidate]
            for name in candidates:
                verdict_name = "not_reached" if candidate == "all" else verdict
                cell_key = (key[0] + "/" + key[1], state_class, fields["state_red"], name, fields["order"], verdict_name, summary)
                table[cell_key] += 1
                examples[cell_key] = min(examples.get(cell_key, ordinal), ordinal)
for (history, faults), states in sorted(seen.items()):
    count = expected.get(history, -1)
    print(f"name=c834_coverage history={history} faults={faults} state_count={count} state_order_rows={len(states)} complete={len(states) == 2 * count}")
for cell, count in sorted(table.items()):
    history, state_class, state_red, candidate, order, verdict, outcome = cell
    print(f"name=c834_cell history={history} class={state_class} state_red={state_red} candidate={candidate} order={order} judge={verdict} outcome={outcome} states={count} example_ordinal={examples[cell]}")
