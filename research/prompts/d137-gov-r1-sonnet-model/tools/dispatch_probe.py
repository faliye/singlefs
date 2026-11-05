#!/usr/bin/env python3
"""派发闸：设计员写重跑登记，准入判法（假的 admission.py）退各种码，钩子怎么判。"""
import sys, os, warnings, json
warnings.filterwarnings("ignore")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import feed
LEG = feed.LEG
THREAD = open(os.path.join(feed.TREE, ".claude/heavy-thread-default")).read().split()[0:0] or None
tl = [l.split()[0] for l in open(os.path.join(feed.TREE, ".claude/heavy-thread-default")) if l.strip() and not l.startswith("#")][0]
draft = os.path.join(LEG, "workroot", "dispatch-probe")
os.makedirs(draft, exist_ok=True)
def prompt(extra="", subagent="experiment-designer", body="写重跑登记 research/prompts/e142-r19-prereg.md。"):
    return (f"# 派发：{subagent}\n\n重型测试：不跑\n批次：records/2026-10-29-一百三十七节那一轮调度记录.md#4\n草稿目录：{draft}\n"
            f"认领：bash scripts/hosts/work-directory-ownership.sh claim {draft} --purpose '探针'\n被测条款：无\n岔路单：重跑\n预计挂钟：2 小时内\n"
            f"线程上限：{tl}\n\n## 关的是哪几条\n\n{body}\n\n## 出口\n\n交回。\n" + extra)
def run(label, code_script, extra="", **kw):
    env = {"RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT": os.path.join(LEG, "stubs", code_script)} if code_script else {}
    rc, err, notes = feed.feed("runner-dispatch-guard.sh", None, env=env, tool="Agent", tool_input={"subagent_type": "experiment-designer", "prompt": prompt(extra)}, **kw)
    first = [l for l in err.splitlines() if "准入" in l][:1]
    print("%-28s rc=%s notes=%d %s" % (label, rc, len(notes), (first[0][:90] if first else (err.strip().splitlines()[0][:90] if err.strip() else ""))))
    for n in notes:
        if any("准入" in f for f in n.get("findings", [])): print("      检出:", [f[:100] for f in n["findings"]], n.get("outcome"))
if __name__ == "__main__":
    for name in ["adm0.py", "adm2.py", "adm3.py", "adm77.py", "adm1.py", "adm4.py", "adm5.py", "adm124.py", "adm250.py", "admhang.py", "nonexistent.py"]:
        run("退出码 " + name, name, timeout=100)
    run("退 2 + 准入已判", "adm2.py", extra="准入已判：换了一条臂\n")
    run("退 3 + 准入已判", "adm3.py", extra="准入已判：换了一条臂\n")
    run("不是重跑登记", "adm2.py", extra="")
