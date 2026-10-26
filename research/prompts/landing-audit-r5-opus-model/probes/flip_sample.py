import json, os, random, sys
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
sys.path.insert(0, sys.argv[4])
import hookload
corpus, cur_hooks, orig_hooks = sys.argv[1:4]
cur = hookload.load(os.path.join(cur_hooks, "runner-dispatch-guard.sh"), cur_hooks, "c")
orig = hookload.load(os.path.join(orig_hooks, "runner-dispatch-guard.sh"), orig_hooks, "o")
flips = []
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    p, k = item["prompt"], item["type"] or ""
    if p and orig.report_file_name_verdict(k, p) is not None and cur.report_file_name_verdict(k, p) is None:
        flips.append(item)
random.seed(20261023)
for item in random.sample(flips, 12):
    lines = [l.strip()[:150] for l in item["prompt"].split("\n") if "草稿目录" in l or "report" in l.lower()]
    print("==", item["agent"], item["type"], (item["first_time"] or "")[:10]); print("   " + "\n   ".join(lines[:4]))
print(len(flips))
