import re, subprocess, sys, os
repo = sys.argv[2]
out = sys.argv[1]
types = ["clerk","experiment-designer","experiment-runner","implementation-writer","investigator","kb-scribe","mutation-triage","prior-art","sweep","three-way-attack","three-way-defense","three-way-forward","three-way-materials","tooling-writer"]
work = os.path.expanduser("~/.local/share/singlefs/work")
for t in types:
    scratch = f"{work}/landing-audit-r2-opus-probe-{t}"
    sets = ["--set", f"草稿目录={scratch}", "--set", "认领用途=探针：现试骨架"]
    if t.startswith("three-way-") and t != "three-way-materials":
        sets += ["--set", "报告=research/prompts/probe-r9-opus-output.md"]
    sk = subprocess.run(["python3", "scripts/agents/dispatch-prompt.py", t, *sets], cwd=repo, capture_output=True, text=True)
    if sk.returncode != 0:
        print(t, "skeleton rc", sk.returncode, sk.stderr[:300]); continue
    lines = []
    for line in sk.stdout.split("\n"):
        if re.match(r"^(分片|续跑|多机)：<填", line):
            continue
        if line.startswith("预计挂钟：<填"):
            line = "预计挂钟：2 小时内，读几份文件"
        line = re.sub(r"<填 ([^>]*)>", lambda m: "探针内容" , line)
        lines.append(line)
    text = "\n".join(lines)
    path = os.path.join(out, t + ".txt")
    open(path, "w").write(text)
    ck = subprocess.run(["python3", "scripts/agents/dispatch-prompt.py", "--check", path, "--type", t, "--root", repo], cwd=repo, capture_output=True, text=True)
    print(f"== {t} rc={ck.returncode}")
    for l in (ck.stdout + ck.stderr).split("\n"):
        if "✗" in l:
            print("   ", l[:230])
