"""K3 交回闸 ②：真会话里三方腿交回正文（SubagentHandback 的 message）点名的报告，改前（交回闸自己那份正则）与改后（lib_three_way_legs.leg_report_mentions）认出的是不是同一组；
另数改后认出的那一段前面接着别的目录（不是仓根）的份数。数法：一次交回调用一次（被拒后重交的各算一次）。
用法：python3 -I leg_handbacks.py <语料.jsonl> <改后钩子目录> <改前钩子目录> <仓根> <agent 号清单>。只读。"""
import collections, json, os, re, sys
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hookload
corpus, cur_hooks, orig_hooks, root, id_list = sys.argv[1:6]
wanted = set(open(id_list, encoding="utf-8").read().split())
cur = hookload.load(os.path.join(cur_hooks, "handback-guard.sh"), cur_hooks, "cur_handback")
orig = hookload.load(os.path.join(orig_hooks, "handback-guard.sh"), orig_hooks, "orig_handback")
legs = cur.three_way_legs.three_way_leg_types(root) | {"three-way-verifier"}
counts, examples = collections.Counter(), collections.defaultdict(list)
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    if item["agent"] not in wanted or item["type"] not in legs:
        continue
    for record in item["records"]:
        for block in record["message"]["content"]:
            if block.get("type") != "tool_use" or block.get("name") != "SubagentHandback":
                continue
            message = str(block["input"].get("message") or "")
            before = sorted(set(orig.LEG_REPORT_MENTION.findall(message))) if hasattr(orig, "LEG_REPORT_MENTION") else []
            after = cur.leg_reports_named(message)
            prefixed = [m for m in cur.three_way_legs.leg_report_mentions(message)
                        if m.start() > 0 and message[m.start() - 1] == "/" and not message[:m.start()].endswith(root.rstrip("/") + "/")]
            key = "两边相同" if before == after else ("改前认出、改后一份没认出" if before and not after else "两边不同")
            counts[key] += 1
            if prefixed:
                counts["改后认出的那一段前面接着仓根之外的目录"] += 1
                examples["改后认出的那一段前面接着仓根之外的目录"].append(f"{item['agent']}：{message[max(0, prefixed[0].start() - 60):prefixed[0].end()]!r}")
            if key != "两边相同" and len(examples[key]) < 6:
                examples[key].append(f"{item['agent']}（{item['type']}）：改前 {before} 改后 {after}")
for key in sorted(counts):
    print(f"{key}：{counts[key]} 次；例：{' ｜ '.join(examples[key][:6])}")
