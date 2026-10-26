"""把闭窗口两版（改前 UTC 切天 / 今天东京切天）逐个子 agent 对齐，拆出 2144 与 2125 的差、各桶的差各来自哪一处改动。
用法：decompose.py <改前 agents.jsonl> <今天 agents.jsonl>"""
import collections, datetime, json, re, sys
old = {json.loads(l)["agent"]: json.loads(l) for l in open(sys.argv[1])}
new = {json.loads(l)["agent"]: json.loads(l) for l in open(sys.argv[2])}
UTC = datetime.timezone.utc
def t(s): return datetime.datetime.fromisoformat(s)
only_old, only_new, both = set(old) - set(new), set(new) - set(old), set(old) & set(new)
print("name=counts old", len(old), "new", len(new), "only_old", len(only_old), "only_new", len(only_new), "both", len(both))
edge_top = (datetime.datetime(2026, 10, 21, 15, tzinfo=UTC), datetime.datetime(2026, 10, 22, tzinfo=UTC))
edge_bottom = (datetime.datetime(2026, 9, 30, 15, tzinfo=UTC), datetime.datetime(2026, 10, 1, tzinfo=UTC))
print("name=only_old_in_top_edge", sum(edge_top[0] <= t(old[a]["start"]) < edge_top[1] for a in only_old), "of", len(only_old))
print("name=only_new_in_bottom_edge", sum(edge_bottom[0] <= t(new[a]["start"]) < edge_bottom[1] for a in only_new), "of", len(only_new))
tot_old = sum(a["cost"] for a in old.values()); tot_new = sum(a["cost"] for a in new.values())
print("name=cost old %.2f new %.2f only_old %.2f (%.2f%% of old) only_new %.2f (%.2f%% of new)" % (tot_old / 1e6, tot_new / 1e6,
      sum(old[a]["cost"] for a in only_old) / 1e6, 100 * sum(old[a]["cost"] for a in only_old) / tot_old,
      sum(new[a]["cost"] for a in only_new) / 1e6, 100 * sum(new[a]["cost"] for a in only_new) / tot_new))
def buckets(rows): return collections.Counter(r["bucket"] for r in rows)
bo, bn = buckets(old.values()), buckets(new.values())
bo_only, bn_only = buckets(old[a] for a in only_old), buckets(new[a] for a in only_new)
changed = collections.Counter((old[a]["bucket"], new[a]["bucket"]) for a in both if old[a]["bucket"] != new[a]["bucket"])
print("name=bucket_table  bucket old new | minus_only_old plus_only_new | changed_in_both_net")
for b in sorted(set(bo) | set(bn)):
    net = sum(c for (x, y), c in changed.items() if y == b) - sum(c for (x, y), c in changed.items() if x == b)
    ok = bo[b] - bo_only[b] + bn_only[b] + net == bn[b]
    print(f"  {b:38s} {bo[b]:5d} {bn[b]:5d} | -{bo_only[b]:3d} +{bn_only[b]:3d} | {net:+4d} {'ok' if ok else 'MISMATCH'}")
print("name=changed_in_both", dict(changed))
for (x, y), c in changed.items():
    sample = sorted(a for a in both if old[a]["bucket"] == x and new[a]["bucket"] == y)[:3]
    for a in sample:
        o, n = old[a], new[a]
        print(f"    {a} {x}->{y} type={o['type']} written {o['written']}->{n['written']} landed {o['landed']}->{n['landed']} used_main {o['used_main']}->{n['used_main']} used_any {o['used_any']}->{n['used_any']}")
# 返工字样：313 对 327
ro = sum(a["rework"] for a in old.values()); rn = sum(a["rework"] for a in new.values())
ro_only = sum(old[a]["rework"] for a in only_old); rn_only = sum(new[a]["rework"] for a in only_new)
flip = collections.Counter((old[a]["rework"], new[a]["rework"]) for a in both if old[a]["rework"] != new[a]["rework"])
print("name=rework old", ro, "new", rn, "only_old", ro_only, "only_new", rn_only, "flip_in_both", dict((f"{k[0]}->{k[1]}", v) for k, v in flip.items()))
for a in sorted(a for a in both if old[a]["rework"] != new[a]["rework"]):
    print("    flip", a, old[a]["rework"], "->", new[a]["rework"], repr(new[a]["description"][:70]))
