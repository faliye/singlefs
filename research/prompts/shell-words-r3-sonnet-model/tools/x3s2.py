"""x3s2.py <登记目录> : 第二轮正推腿乙 1943 案例此刻的 (重型闸, 检出钩子) 退出码 vs 腿登记的第二轮值，逐条列出不同的。"""
import json, glob, sys
reg = {}
for f in sorted(glob.glob(sys.argv[1] + "/*.jsonl")):
    for l in open(f):
        r = json.loads(l); reg[r["id"]] = (r["rc_h"], r["rc_d"], r["exp_h"], r["exp_d"])
now = {r["id"]: (r["rc_h"], r["rc_d"], r["exp_h"], r["exp_d"]) for r in json.load(open("res/s2.now.json"))}
diff = [k for k in sorted(set(now) | set(reg)) if now.get(k) != reg.get(k)]
print("此刻", len(now), "条，登记", len(reg), "条，不同", len(diff), "条")
for k in diff: print(k, "登记(h,d,应h,应d)=", reg.get(k), "此刻=", now.get(k))
