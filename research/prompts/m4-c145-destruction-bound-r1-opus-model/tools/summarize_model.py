#!/usr/bin/env python3
"""把模型的 jsonl 摘成一行一种（调度、core 变体）：状态数、带未完成意图的、释放侧的错、纪元 ≥ 意图 txg 的、各 checker 上界的红与差、首红深度。
用法：summarize_model.py <jsonl>…"""
import json
import sys

for path in sys.argv[1:]:
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            d = json.loads(line)
            print(
                d["mode"], d["core_hi"], "states", d["states"], "open", d["states_with_an_open_intent"],
                "bugs", d["release_bugs"], "epoch>=I", d["entries_with_epoch_at_or_above_the_intent_txg"],
                "red", d["checker_red_states"], "diff", d["verdict_differs_from_infinity"],
                "reddepth", d["first_checker_red_depth"],
            )
