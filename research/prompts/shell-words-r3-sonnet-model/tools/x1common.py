"""x1common.py : X1 造样本的公共部分。每条样本 {tag, item, cmd, hook, agent, [bg], [cwd]}。
重型闸（hook=guard，agent=main）的重活 P = cargo test --all；检出钩子（hook=det，agent=clerk）的 P = detached-run.sh start。"""
import json, sys
GP = "cargo test --all"
DP = "bash scripts/hosts/detached-run.sh start --label m -- sleep 1"
def mk(item, tag, cmd, hook="guard", **kw):
    d = dict(item=item, tag=tag, cmd=cmd, hook=hook, agent="main" if hook == "guard" else "clerk")
    d.update(kw); return d
def both(item, tag, tpl, gp=GP, dp=DP):
    """模板里 {P} 换成两种重活，各出一条。"""
    return [mk(item, tag, tpl.replace("{P}", gp)), mk(item, tag, tpl.replace("{P}", dp), "det")]
def dump(path, cases):
    seen, uniq = set(), []
    for c in cases:
        k = (c["cmd"], c["hook"], c["agent"], c.get("cwd"), bool(c.get("bg")))
        if k in seen: continue
        seen.add(k); uniq.append(c)
    json.dump(uniq, open(path, "w"), ensure_ascii=False); print(path, len(uniq))
