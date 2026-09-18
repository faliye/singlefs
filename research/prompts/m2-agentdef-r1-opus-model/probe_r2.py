#!/usr/bin/env python3
"""m2-agentdef-r1 Opus 攻方腿：改法 R2（本腿自己提的，被攻过零轮）——② 不看反引号里的日期。
量它修不修得了 FP7（时刻格式的例子），会不会让两套样本与清理前原件上的判定掉。
用法：python3 probe_r2.py <仓根> <工作目录>
"""
import glob, os, re, shutil, subprocess, sys

ROOT, WORK = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
GATE = ".claude/gate.d/71-agent-def-flow-only.sh"
FIXTURES = ".claude/gate.d/fixtures/71-agent-def-flow-only.sh"
BEFORE = "research/prompts/m2-agent-def-cleanup/before"
original = open(os.path.join(ROOT, GATE), encoding="utf-8").read()
R2_OLD = "            if not DATE.search(without_quoted(line)):\n"
R2_NEW = "            if not DATE.search(re.sub(r\"`[^`]*`\", \"\", without_quoted(line))):\n"
assert original.count(R2_OLD) == 1
patched = original.replace(R2_OLD, R2_NEW)


def run(root, text):
    path = os.path.join(WORK, "gate-r2.sh")
    open(path, "w", encoding="utf-8").write(text)
    result = subprocess.run(["bash", path, root], capture_output=True, text=True)
    out = result.stdout + result.stderr
    dated = re.search(r"✗ (\d+) 行写了日期", out)
    return result.returncode, int(dated.group(1)) if dated else 0, out


def root_with(name, sources):
    root = os.path.join(WORK, name)
    shutil.rmtree(root, ignore_errors=True)
    os.makedirs(os.path.join(root, ".claude/agents"))
    for source in sources:
        shutil.copy(source, os.path.join(root, ".claude/agents", os.path.basename(source)))
    return root


today = sorted(glob.glob(os.path.join(ROOT, ".claude/agents/*.md")))
fp7 = root_with("fp7", today)
target = os.path.join(fp7, ".claude/agent-common.md")
text = open(target, encoding="utf-8").read()
old, new = "报告里的时刻写清是哪个时区。", "报告里的时刻写清是哪个时区（写成 `2026-09-18 14:08 JST` 这种形式）。"
assert text.count(old) == 1
open(target, "w", encoding="utf-8").write(text.replace(old, new))
before = root_with("before", sorted(glob.glob(os.path.join(ROOT, BEFORE, "agents/*.md")))
                   + [os.path.join(ROOT, BEFORE, "agent-common.md"), os.path.join(ROOT, BEFORE, "main-agent.md")])
for label, gate_text in (("original", original), ("R2", patched)):
    for name, root in (("FP7", fp7), ("before-files", before)):
        code, dated, _ = run(root, gate_text)
        print(f"{label}\t{name}\texit={code}\t②={dated}")
    for kind in ("red", "green"):
        work = os.path.join(WORK, f"fixture-{kind}")
        shutil.rmtree(work, ignore_errors=True)
        shutil.copytree(os.path.join(ROOT, FIXTURES, kind), work)
        subprocess.run(["bash", "setup.sh"], cwd=work, check=True, capture_output=True)
        code, dated, out = run(work, gate_text)
        expect = open(os.path.join(ROOT, FIXTURES, kind, "expect"), encoding="utf-8").read().splitlines()
        want_exit = int([row for row in expect if row.startswith("exit=")][0][5:])
        missing = [row[5:] for row in expect if row.startswith("want=") and row[5:] not in out]
        print(f"{label}\tfixture-{kind}\tpass={code == want_exit and not missing}\texit={code}\tmissing={missing}")
