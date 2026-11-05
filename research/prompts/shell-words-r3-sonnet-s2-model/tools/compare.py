#!/usr/bin/env python3
"""rerun 的对比：新跑的 jsonl 与随报告带的 out/ 里同 id 的逐条比（该判、钩子判都要一样）。不一样的件数写在末行，供 rerun 退出码用。"""
import glob, json, os, sys
ref_dir, new_dir = sys.argv[1], sys.argv[2]


def load(d):
    rows = {}
    for p in sorted(glob.glob(os.path.join(d, "*.jsonl"))):
        for line in open(p):
            r = json.loads(line); rows[r["id"]] = r
    return rows


ref, new = load(ref_dir), load(new_dir)
common = sorted(set(ref) & set(new))
diff = [i for i in common if (ref[i]["want_reject"], ref[i]["hook"]) != (new[i]["want_reject"], new[i]["hook"])]
for i in diff[:30]:
    print("不一致", i, "报告里", ref[i]["want_reject"], ref[i]["hook"], "新跑", new[i]["want_reject"], new[i]["hook"])
print(f"对比 {len(common)} 条（新跑 {len(new)}，报告里 {len(ref)}），不一致 {len(diff)}")
sys.exit(1 if diff or not common else 0)
