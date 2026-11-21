#!/usr/bin/env python3
"""攻方腿探针：放置类弹窗钩子（.claude/hooks/ask-user-placement-guard.sh）对几种问句各判成什么。
每一问的选项都写成中性的「是」「否」（不带四组词），只看问句本身。钩子退 2 是拦、退 0 是放。
检出记录写进 <运行目录>/placement-detections.jsonl（AGENT_HOOK_DETECTIONS），不写工作根别处。
用法：python3 placement_probe.py <冻结树根> <运行目录>
每一问打一行 name=placement_<问名> exit=<n> kind=<误拦样本|漏拦样本|对照> placement_by_intent=<yes|no> verdict=<right|wrong>"""
import json
import os
import subprocess
import sys

root, work = sys.argv[1], sys.argv[2]
hook = os.path.join(root, ".claude", "hooks", "ask-user-placement-guard.sh")
# (问名, 问句, 按意思是不是问 crates/ 下东西放哪个包或目录)
QUESTIONS = (
    ("scheduling-put-at-with-package", "合入后验证放在提交前做，还是现在就对验证包跑一趟？", "no"),
    ("result-file-into-scratch-directory", "复跑的输出放在草稿目录还是 research/results/ 下？", "no"),
    ("batch-title-unquoted", "产品GPU包测试与E202装置归位那一批先关哪几件？", "no"),
    ("where-is-the-log-directory", "这一趟的日志在哪个目录里找？", "no"),
    ("put-to-package", "E202 的装置放到实验包还是验证包？", "yes"),
    ("open-one-new-package", "要不要为 E202 新开一个包？", "yes"),
    ("separate-package-or-merge", "配对键那个包该单开包，还是并进验证包？", "yes"),
    ("which-one-package", "E202 归哪一个包？", "yes"),
    ("move-into-crate-path", "E202 的装置移到 crates/singlefs-implementation-experiments 行吗？", "yes"),
    ("which-subdirectory", "这批装置挪进哪个子目录？", "yes"),
    ("control-put-where-package", "E202 放哪个包？", "yes"),
    ("control-whole-package-run", "整包跑哪几个包？", "no"),
)

environment = dict(os.environ)
environment["AGENT_HOOK_DETECTIONS"] = os.path.join(work, "placement-detections.jsonl")
for name, question, intent in QUESTIONS:
    hook_input = {"tool_name": "AskUserQuestion", "session_id": "attack-probe",
                  "tool_input": {"questions": [{"question": question, "header": "探针", "multiSelect": False,
                                                "options": [{"label": "是", "description": "是"}, {"label": "否", "description": "否"}]}]}}
    completed = subprocess.run(["bash", hook], input=json.dumps(hook_input, ensure_ascii=False), capture_output=True, text=True,
                               env=environment, cwd=root, start_new_session=True)
    blocked = completed.returncode == 2
    right = blocked == (intent == "yes")
    kind = "对照" if name.startswith("control-") else ("漏拦样本" if intent == "yes" else "误拦样本")
    print(f"name=placement_{name} exit={completed.returncode} kind={kind} placement_by_intent={intent} verdict={'right' if right else 'wrong'}")
