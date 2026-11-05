#!/usr/bin/env python3
"""G2：「判不了的（输入不是 JSON 对象……）照常放行并记一条检出（只记）」逐个钩子喂坏输入。"""
import os, sys, json, signal, subprocess, warnings
warnings.filterwarnings("ignore")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import feed
HOOKS = ["heavy-test-guard.sh", "bash-command-detector.sh", "runner-dispatch-guard.sh", "write-guard.sh", "handback-guard.sh", "ask-user-claim-guard.sh", "continuation-guard.sh", "completion-cleanup-guard.sh", "run-scope-guard.sh", "timer-only-remind-guard.sh"]
rows = []
for hook in HOOKS:
    path = os.path.join(feed.TREE, ".claude/hooks", hook)
    if not os.path.exists(path): print("%s\t缺文件" % hook); continue
    for label, payload in (("非 JSON 文本", "garbage not json"), ("JSON 数组", "[]"), ("空输入", "")):
        before = sum(1 for _ in open(feed.DETECT)) if os.path.exists(feed.DETECT) else 0
        e = {k: v for k, v in os.environ.items() if not k.startswith(("SINGLEFS_", "CLAUDE_", "AGENT_HOOK"))}
        e.update({"CLAUDE_PROJECT_DIR": feed.TREE, "AGENT_HOOK_DETECTIONS": feed.DETECT, "SINGLEFS_WORK_ROOT": feed.LEG + "/workroot"})
        p = subprocess.Popen(["setsid", "--wait", "bash", path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=feed.SANDBOX, env=e, text=True, preexec_fn=os.setpgrp)
        try: out, err = p.communicate(payload, timeout=60); rc = p.returncode
        except subprocess.TimeoutExpired:
            os.killpg(p.pid, signal.SIGKILL); out, err = p.communicate(); rc = 124
        try: os.killpg(p.pid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError): pass
        after = sum(1 for _ in open(feed.DETECT)) if os.path.exists(feed.DETECT) else 0
        print("%s\t%s\trc=%s\t检出+%d" % (hook, label, rc, after - before))
