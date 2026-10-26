"""K3 ⑥ 探针：造子 agent 会话记录，喂草稿里拷的 handback-guard.sh，看哪些形状被当成「真跑过命名检查」。"""
import json, os, shutil, subprocess, sys
S = sys.argv[1]   # 草稿目录（里面有 tree/）；sys.argv[2] 是仓根（只读，拷实现员定义进假仓）
hook = os.path.join(S, "tree", ".claude", "hooks", "handback-guard.sh")
fake = os.path.join(S, "fake-repo")
os.makedirs(os.path.join(fake, ".git"), exist_ok=True)
os.makedirs(os.path.join(fake, "crates", "core", "src"), exist_ok=True)
os.makedirs(os.path.join(fake, ".claude", "agents"), exist_ok=True)
shutil.copy(os.path.join(sys.argv[2], ".claude", "agents", "implementation-writer.md"), os.path.join(fake, ".claude", "agents"))
wt = os.path.join(S, "fake-worktree")
os.makedirs(os.path.join(wt, "crates", "core", "src"), exist_ok=True)
open(os.path.join(wt, ".git"), "w").write("gitdir: /nonexistent\n")
session = os.path.join(S, "fake-session")
os.makedirs(os.path.join(session, "subagents"), exist_ok=True)
STAGE = ("SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; "
         "范围=4 格; 为什么=交回前核自己改动里的命名' bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names {root}")
LINT = "bash .claude/singlefs-ai-sop/scripts/naming-lint.sh {root}"
ok, red, timeout_red = (False, "ok"), (True, "Exit code 1\n  ✗ 命中 1 处"), (True, "Exit code 124")
src = os.path.join(fake, "crates", "core", "src", "lib.rs")
def edited(path):
    return ("Edit", {"file_path": path, "old_string": "a", "new_string": "b"}, ok, fake)
def bash(cmd, outcome=ok, cwd=None):
    return ("Bash", {"command": cmd}, outcome, cwd or fake)
cases = {
    "对照：改了 .rs 没跑": [edited(src)],
    "对照：两条都真跑": [edited(src), bash(STAGE.format(root=fake)), bash(LINT.format(root=fake), red)],
    "false && 两条（一条都没执行，退 1）": [edited(src), bash("false && " + STAGE.format(root=fake), red), bash("false && " + LINT.format(root=fake), red)],
    "true || 两条（一条都没执行，退 0）": [edited(src), bash("true || " + STAGE.format(root=fake)), bash("true || " + LINT.format(root=fake))],
    "if false; then 两条; fi": [edited(src), bash("if false; then " + STAGE.format(root=fake) + "; " + LINT.format(root=fake) + "; fi")],
    "bash -n 只查语法": [edited(src), bash(STAGE.replace("bash .claude", "bash -n .claude").format(root=fake)), bash(LINT.replace("bash .claude", "bash -n .claude").format(root=fake))],
    "timeout 0.1 包着、被杀退 124": [edited(src), bash("timeout 0.1 " + STAGE.format(root=fake), timeout_red), bash("timeout 0.1 " + LINT.format(root=fake), timeout_red)],
    "两条写进函数定义不调用": [edited(src), bash("f() { " + STAGE.format(root=fake) + "; " + LINT.format(root=fake) + "; }")],
    "根写成 \"$CLAUDE_PROJECT_DIR\"（真跑了两条）": [edited(src), bash(STAGE.format(root='"$CLAUDE_PROJECT_DIR"')), bash(LINT.format(root='"$CLAUDE_PROJECT_DIR"'), red)],
    "worktree 里改、根写主工作区（应拒）": [("Edit", {"file_path": os.path.join(wt, "crates", "core", "src", "lib.rs"), "old_string": "a", "new_string": "b"}, ok, fake),
                                           bash(STAGE.format(root=fake)), bash(LINT.format(root=fake), red)],
}
outputs = []
for number, (label, records) in enumerate(cases.items()):
    agent = f"probe{number}"
    with open(os.path.join(session, "subagents", f"agent-{agent}.jsonl"), "w", encoding="utf-8") as handle:
        handle.write(json.dumps({"type": "user", "timestamp": "2026-10-23T00:00:00+00:00", "message": {"role": "user", "content": "# 派发"}}, ensure_ascii=False) + "\n")
        for index, (tool, arguments, outcome, cwd) in enumerate(records):
            handle.write(json.dumps({"type": "assistant", "cwd": cwd, "message": {"role": "assistant", "content": [{"type": "tool_use", "id": f"c{index}", "name": tool, "input": arguments}]}}, ensure_ascii=False) + "\n")
            result = {"type": "tool_result", "tool_use_id": f"c{index}", "content": outcome[1]}
            if outcome[0]:
                result["is_error"] = True
            handle.write(json.dumps({"type": "user", "message": {"role": "user", "content": [result]}}, ensure_ascii=False) + "\n")
    hook_input = {"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "implementation-writer", "agent_id": agent,
                  "transcript_path": session + ".jsonl"}
    done = subprocess.run(["bash", hook], input=json.dumps(hook_input), capture_output=True, text=True, env={**os.environ, "CLAUDE_PROJECT_DIR": fake}, timeout=60)
    first = next((l for l in done.stderr.splitlines() if l.startswith("✗")), "")
    print(f"{label}：退 {done.returncode}" + (f"；{first[:120]}" if done.returncode else ""))
