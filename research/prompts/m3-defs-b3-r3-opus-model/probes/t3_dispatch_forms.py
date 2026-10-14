#!/usr/bin/env python3
"""T3-B：⑭ 的「多机」一行只按字面点名两个词、不许两个现算词，别的现算说法与「只用一台」照放；预计挂钟一行只认开头。
用法：python3 -I t3_dispatch_forms.py <树> <工作目录> <标签>。打印 name=… 行。"""
import os, subprocess, sys

tree, work, label = sys.argv[1], sys.argv[2], sys.argv[3]
os.makedirs(work, exist_ok=True)
SHARDS, RESUME = "按登记的取样点切片，片数取多机配置的台数", "同一条命令重起，按产物里已写完的片号跳过"
CASES = [
    # (格名, 预计挂钟, 多机一行或 None)
    ("divide-by-running", "超过 2 小时，按上一次同规模的用时推",
     "台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值除以此刻同时在跑的重活数"),
    ("one-host-nproc", "超过 2 小时，按上一次同规模的用时推",
     "只在本机一台跑，不读 --emit-hosts，也不读 host-resources.py；线程按 nproc 取，不用 .claude/heavy-thread-default"),
    ("one-host-nproc-registered-words", "超过 2 小时，按上一次同规模的用时推",
     "只在本机一台跑，不读 --emit-hosts；线程按 nproc 取，不用 .claude/heavy-thread-default"),
    ("wall-clock-first-segment", "2 小时内（第一段）；第二段超过 2 小时，另派", None),
    ("control-registered", "超过 2 小时，按上一次同规模的用时推",
     "台数从 layer0-shard-configuration-check.sh --emit-hosts 读、每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量"),
]
for name, wall_clock, hosts in CASES:
    prompt = os.path.join(work, f"prompt-{name}.md")
    lines = ["# 派发：experiment-runner", "", "重型测试：不跑", "线程上限：31", f"草稿目录：{work}/runner-{name}/", f"报告：{work}/runner-{name}/report.md",
             "这一段回答的岔路：岔路单第 1 行", f"预计挂钟：{wall_clock}"]
    if hosts is not None:
        lines += [f"分片：{SHARDS}", f"续跑：{RESUME}", f"多机：{hosts}"]
    lines += ["", "## 关的是哪几条", "", "照跑前登记跑产物。", ""]
    with open(prompt, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines))
    guarded = subprocess.run(["bash", os.path.join(tree, ".claude", "hooks", "runner-dispatch-guard.sh"), "--check-prompt", prompt,
                              "--type", "experiment-runner", "--root", tree], capture_output=True, text=True,
                             env=dict(os.environ, CLAUDE_PROJECT_DIR=tree, AGENT_HOOK_DETECTIONS=os.path.join(work, "detections.jsonl")))
    text = (guarded.stderr + guarded.stdout).replace(work, "<WORK>").replace(tree, "<TREE>")
    crosses = [line.strip()[:160] for line in text.splitlines() if line.strip().startswith("✗")]
    print(f"name=dispatch-14-form tree={label} case={name} rc={guarded.returncode} refusals={len(crosses)} first={crosses[0] if crosses else '-'}")
