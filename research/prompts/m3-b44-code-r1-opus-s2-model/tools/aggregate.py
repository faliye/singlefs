#!/usr/bin/env python3
"""把 run 模式的输出（name=k1_state 行）按（历史、读故障、交盘次序、凭据、结局类、判错标记、R 已没而拒）归组数状态，例子序号取那一组最小的。
survey 的输出（name=k1_landed 行）另按（历史、读故障、凭据、结局类）归组。用法：aggregate.py <输出文件…>"""
import re
import sys
from collections import defaultdict

FIELD = re.compile(r"(\w+)=(\S+)")


def outcome_class(outcome: str) -> str:
    if outcome.startswith("refused:"):
        return ":".join(outcome.split(":")[:3])
    new = re.search(r":new=(\d+)", outcome).group(1)
    gone = re.search(r":gone=(\d+)", outcome).group(1)
    red = re.search(r":red_after=(\S+)", outcome).group(1)
    return f"mounted:new={new}:gone={gone}:red_after={red}"


groups = defaultdict(lambda: [0, None])
seen_states = defaultdict(set)
for path in sys.argv[1:]:
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            if not line.startswith("name=k1_state "):
                continue
            fields = dict(FIELD.findall(line))
            key = (
                fields["history"], fields["faults"], fields["order"], fields["clause_credential"],
                f"r_units_gone={fields['r_units_gone']}", outcome_class(fields["outcome"]),
                f"wrong={fields['wrong']}", f"refused_with_r_gone={fields['refused_with_r_gone']}",
            )
            ordinal = int(fields["ordinal"])
            seen_states[(fields["history"], fields["faults"], fields["order"])].add(ordinal)
            group = groups[key]
            group[0] += 1
            group[1] = ordinal if group[1] is None else min(group[1], ordinal)
for key in sorted(groups):
    count, example = groups[key]
    history, faults, order, credential, gone, outcome, wrong, refused = key
    print(f"name=k1_cell history={history} faults={faults} order={order} clause_credential={credential} {gone} outcome={outcome} {wrong} {refused} states={count} example_ordinal={example}")
for (history, faults, order), states in sorted(seen_states.items()):
    print(f"name=k1_coverage history={history} faults={faults} order={order} states={len(states)} smallest={min(states)} largest={max(states)}")
