#!/usr/bin/env python3
"""T2：⑯ 只看点名那一层目录自己的锁。子目录的持锁者还在跑时，把它的父目录派给另一个子 agent 照放；
之后新来的那一个照共用约束「建仓副本之前先删不再用的副本」在自己的草稿目录里删子目录，Bash 里的删没有闸。
用法：python3 -I t2_nested.py <树> <工作目录> <标签>。全部在 <工作目录> 里：假工作根、假主会话记录、派发闸状态目录、检出记录。"""
import datetime, json, os, subprocess, sys

tree, work, label = sys.argv[1], sys.argv[2], sys.argv[3]
root = os.path.join(work, "work-root")
os.makedirs(root, exist_ok=True)
transcript = os.path.join(work, "session.jsonl")
open(transcript, "w").close()
environment = dict(os.environ, CLAUDE_PROJECT_DIR=tree, SINGLEFS_WORK_ROOT=root,
                   DISPATCH_GUARD_STATE_ROOT=os.path.join(work, "dispatch-state"), AGENT_HOOK_DETECTIONS=os.path.join(work, "detections.jsonl"))

def prompt_for(directory, note):
    return (f"# 派发：clerk\n\n重型测试：不跑\n线程上限：31\n预计挂钟：2 小时内\n草稿目录：{directory}/\n报告：{directory}/report.md\n\n{note}\n")

def dispatch(directory, note):
    text = prompt_for(directory, note)
    hook_input = {"tool_name": "Agent", "session_id": "probe-session", "transcript_path": transcript,
                  "tool_input": {"subagent_type": "clerk", "description": note, "prompt": text}}
    done = subprocess.run(["bash", os.path.join(tree, ".claude", "hooks", "runner-dispatch-guard.sh")], input=json.dumps(hook_input),
                          capture_output=True, text=True, env=environment)
    return done, text

def launched(agent_id, text):
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.000Z")
    with open(transcript, "a", encoding="utf-8") as handle:
        handle.write(json.dumps({"timestamp": stamp, "type": "user",
                                 "toolUseResult": {"status": "async_launched", "agentId": agent_id, "prompt": text, "resolvedModel": "claude-sonnet-5"}},
                                ensure_ascii=False) + "\n")

def write(agent_id, path):
    hook_input = {"tool_name": "Write", "session_id": "probe-session", "agent_id": agent_id, "agent_type": "clerk", "transcript_path": transcript,
                  "tool_input": {"file_path": path, "content": "x\n"}}
    done = subprocess.run(["bash", os.path.join(tree, ".claude", "hooks", "write-guard.sh")], input=json.dumps(hook_input),
                          capture_output=True, text=True, env=environment)
    lock_line = next((line.strip()[:120] for line in done.stderr.splitlines() if "写者锁" in line and "✗" in line), "-")
    return done.returncode, lock_line.replace(work, "<WORK>")

def lock_exists(directory):
    return os.path.isfile(os.path.join(directory, ".writer-lock"))

child = os.path.join(root, "e900", "segment-1")
parent = os.path.join(root, "e900")
first, first_text = dispatch(child, "第一段：在子目录里建仓副本、跑装置")
print(f"name=dispatch tree={label} step=child-first rc={first.returncode} lock_child={lock_exists(child)}")
launched("a1111111111111111", first_text)          # 子目录的持锁者已起、还在跑（刚派、没交回）
os.makedirs(os.path.join(child, "repo-copy"), exist_ok=True)
control, _ = dispatch(child, "同一子目录再派一件")
print(f"name=dispatch tree={label} step=same-child-again rc={control.returncode}")
second, second_text = dispatch(parent, "汇总：读各段产物写汇总表")
print(f"name=dispatch tree={label} step=parent-while-child-running rc={second.returncode} lock_parent={lock_exists(parent)} lock_child_kept={lock_exists(child)}")
launched("a2222222222222222", second_text)
for who, path in (("parent-holder-writes-own", os.path.join(parent, "summary.md")),
                  ("parent-holder-writes-child", os.path.join(child, "repo-copy", "notes.md")),
                  ("child-holder-writes-child", os.path.join(child, "repo-copy", "notes.md"))):
    agent = "a1111111111111111" if who.startswith("child") else "a2222222222222222"
    rc, line = write(agent, path)
    print(f"name=write tree={label} case={who} rc={rc} lock_refusal={line}")
