"""汇总原型的 c834_state 行：核每段（历史，读故障）的（状态，次序）一个不落，再按
（历史/读故障，R 此前的样子，state_red，候选，次序，判，结局）数状态。例子序号取那一格最小的序号（与件的先后无关）。
R 此前的样子：intact（R 写的单元一份不少）、degraded_walkable（有几份被盖、每个单元还剩一份、池级 checker 不红 I-7.2）、
gone（有单元两份都没了）、degraded_red72（每个单元还剩一份而 I-7.2 红）。
结局：refused:<拒因>；mounted_r_kept（这次挂载写行根落盘前没新写掉 R 的单元）；mounted_overwrote:gone=<那一刻两份都没了的单元数>:red=<那一刻红的>。
用法：python3 aggregate.py [--states N] <输出文件…>。--states N：只取每段历史序号 0..N-1 那一截（M1 写行根落盘之前的崩溃状态，
序号 N-1 是写行根之前全落的那一个，即入库 urJ 的盘面），覆盖核对按 N 算；序号 ≥ N 的行不进汇总。"""
import collections, re, sys

CANDIDATES = ["today", "pointer_per_kind", "mapping_root_only", "pointer_pair", "mapping_pair", "always_refuse"]
expected = {}
seen = collections.defaultdict(set)
table = collections.Counter()
examples = {}
arguments = sys.argv[1:]
states_cap = None
if arguments[:1] == ["--states"]:
    states_cap = int(arguments[1])
    arguments = arguments[2:]
for path in arguments:
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
        if states_cap is not None and ordinal >= states_cap:
            continue
        seen[key].add((ordinal, fields["order"]))
        red = fields["state_red"].split("+")
        if fields["r_copies_overwritten"] == "0":
            before = "intact"
        elif fields["r_units_gone"] != "0":
            before = "gone"
        elif "I-7.2" in red:
            before = "degraded_red72"
        else:
            before = "degraded_walkable"
        if fields["r_in_ring"] != "true":
            before += "_r_not_in_ring"
        for cell in fields["cands"].split(";"):
            candidate, verdict, outcome = cell.split(":", 2)
            if outcome.startswith("refused"):
                summary = "refused:" + outcome.split(":")[2]
            else:
                parts = dict(item.split("=", 1) for item in outcome.split(":")[1:] if "=" in item)
                if parts["new"] == "0":
                    summary = "mounted_r_kept"
                else:
                    summary = f"mounted_overwrote:gone={parts['gone']}:red={parts['red']}"
            names = CANDIDATES if candidate == "all" else [candidate]
            for name in names:
                verdict_name = "not_reached" if candidate == "all" else verdict
                cell_key = (key[0] + "/" + key[1], before, fields["state_red"], name, fields["order"], verdict_name, summary)
                table[cell_key] += 1
                examples[cell_key] = min(examples.get(cell_key, ordinal), ordinal)
for (history, faults), states in sorted(seen.items()):
    count = states_cap if states_cap is not None else expected.get(history, -1)
    print(f"name=c834_coverage history={history} faults={faults} state_count={count} state_order_rows={len(states)} complete={len(states) == 2 * count}")
for cell, count in sorted(table.items()):
    history, before, state_red, candidate, order, verdict, outcome = cell
    print(f"name=c834_cell history={history} before={before} state_red={state_red} candidate={candidate} order={order} judge={verdict} outcome={outcome} states={count} example_ordinal={examples[cell]}")
