"""汇总原型的 c834_state 行：核每个（候选，前缀）的状态一个不落，再按（前缀，状态类，候选，次序）数结局。用法：python3 aggregate.py <输出文件…>"""
import collections, re, sys

expected = {}
rows = collections.defaultdict(set)
table = collections.Counter()
examples = {}
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8"):
        if line.startswith("name=c834_history"):
            fields = dict(item.split("=", 1) for item in line.split() if "=" in item)
            for faults in ("y2", "tree_and_mapping", "none"):
                expected[(fields["candidate"], fields["prefix"] + "/" + faults)] = int(fields["state_count"])
        if not line.startswith("name=c834_state"):
            continue
        fields = dict(re.findall(r"(\w+)=(\S+|\[[^\]]*\])", line))
        per_device = re.search(r"per_device=(\[[^\]]*\])", line).group(1)
        faults = fields.get("m2_faults", "y2")
        key = (fields["candidate"], fields["prefix"] + "/" + faults)
        rows[key].add((int(fields["ordinal"]), fields["order"]))
        instances = [int(x) for x in re.findall(r"\(\d+, (\d+)\)", per_device)]
        if len(set(instances)) > 1:
            state_class = "number_on_one_device_only"
        elif instances[0] == 1:
            state_class = "no_number_taken"
        elif int(fields["r_units_overwritten_in_state"]) > 0:
            state_class = "number_taken_and_r_units_overwritten"
        elif int(fields["later_records"]) > 0:
            state_class = "number_taken_records_written_r_units_intact"
        else:
            state_class = "number_taken_no_record_r_units_intact"
        if "m2=refused" in line:
            outcome = "refused"
        else:
            harmed = int(fields["r_units_overwritten_before_row_root"]) > 0
            outcome = "mounted_overwrote_r_units" if harmed else "mounted_r_units_kept"
            outcome += ":red=" + fields["red_before_row_root"]
        judge = re.search(r"judge=(\S+)", line).group(1)
        verdict = "not_reached" if judge == "not_reached" else ",".join(sorted(set(re.findall(r"verdict=(\w+)", judge))))
        cell = (fields["prefix"] + "/" + faults, state_class, fields["candidate"], fields["order"], verdict, outcome)
        table[cell] += 1
        examples.setdefault(cell, int(fields["ordinal"]))
for key, count in sorted(expected.items()):
    got = len(rows[key])
    if got == 0:
        continue
    print(f"name=c834_coverage candidate={key[0]} prefix={key[1]} state_count={count} state_order_rows={got} complete={got == 2 * count}")
for cell, count in sorted(table.items()):
    prefix, state_class, candidate, order, verdict, outcome = cell
    print(f"name=c834_cell prefix={prefix} class={state_class} candidate={candidate} order={order} judge={verdict} outcome={outcome} states={count} example_ordinal={examples[cell]}")
