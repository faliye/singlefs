"""K3 交回闸 ⑤ 端到端：把几份真子 agent 会话记录（原文件逐字节拷贝）放进假主会话目录，喂改前、改后两棵树里的 handback-guard.sh，看退出码。
用法：python3 -I naming_e2e.py <草稿目录> <仓根（只读，当 CLAUDE_PROJECT_DIR）> <项目记录目录> <agent 号…>
树：<草稿目录>/tree-cur 与 <草稿目录>/tree-orig（rerun.sh 建）。拷贝放 <草稿目录>/e2e/，跑完删。"""
import glob, hashlib, json, os, shutil, subprocess, sys
S, R, projects = sys.argv[1:4]
agents = sys.argv[4:]
fake = os.path.join(S, "e2e", "session")
for agent in agents:
    found = glob.glob(os.path.join(projects, "*", "subagents", f"agent-{agent}.jsonl"))
    assert len(found) == 1, (agent, found)
    meta = json.load(open(found[0][:-len(".jsonl")] + ".meta.json", encoding="utf-8"))
    shutil.rmtree(os.path.join(S, "e2e"), ignore_errors=True)
    os.makedirs(os.path.join(fake, "subagents"))
    open(fake + ".jsonl", "w").close()
    copy = os.path.join(fake, "subagents", f"agent-{agent}.jsonl")
    shutil.copyfile(found[0], copy)
    digest = hashlib.sha256(open(copy, "rb").read()).hexdigest()[:16]
    hook_input = {"hook_event_name": "PreToolUse", "tool_name": "SubagentHandback", "tool_input": {"message": "交回"},
                  "agent_type": meta["agentType"], "agent_id": agent, "transcript_path": fake + ".jsonl", "cwd": R}
    outcome = []
    for tree in ("tree-orig", "tree-cur"):
        hook = os.path.join(S, tree, ".claude", "hooks", "handback-guard.sh")
        done = subprocess.run(["bash", hook], input=json.dumps(hook_input), capture_output=True, text=True,
                              env={**os.environ, "CLAUDE_PROJECT_DIR": R}, timeout=120)
        first = next((line for line in done.stderr.split("\n") if line.startswith("✗")), "")
        outcome.append(f"{'改前' if tree == 'tree-orig' else '改后'}退 {done.returncode}{('：' + first[:90]) if first else ''}")
    print(f"{agent}（{meta['agentType']}，记录 sha256 {digest}…）：" + "；".join(outcome))
shutil.rmtree(os.path.join(S, "e2e"), ignore_errors=True)
