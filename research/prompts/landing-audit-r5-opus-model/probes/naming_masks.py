"""K2 交回闸 ⑤：工具退出码 0 却不是命名检查自己的退出码的几种写法（管道、; echo $?、|| true），与真会话里最常见的 timeout 包着再 ; echo $? 的写法，
造子 agent 会话记录喂改前、改后两棵树里的 handback-guard.sh。工具结果的内容照真跑时会有的样子写（rc=…、被 SIGPIPE 截断）。
用法：python3 -I naming_masks.py <草稿目录（下面有 tree-cur、tree-orig）> <仓根（只读：拷实现员定义进假仓）>"""
import json, os, shutil, subprocess, sys
S, R = sys.argv[1], sys.argv[2]
fake, session = os.path.join(S, "masks-repo"), os.path.join(S, "masks-session")
for path in (fake, session):
    shutil.rmtree(path, ignore_errors=True)
os.makedirs(os.path.join(fake, ".git")); os.makedirs(os.path.join(fake, "crates", "core", "src")); os.makedirs(os.path.join(fake, ".claude", "agents"))
shutil.copy(os.path.join(R, ".claude", "agents", "implementation-writer.md"), os.path.join(fake, ".claude", "agents"))
os.makedirs(os.path.join(session, "subagents"))
STAGE = ("SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; "
         "范围=4 格; 为什么=交回前核自己改动里的命名' bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names " + fake)
LINT = "bash .claude/singlefs-ai-sop/scripts/naming-lint.sh " + fake
def ok(text): return (False, text)
edit = ("Edit", {"file_path": os.path.join(fake, "crates", "core", "src", "lib.rs"), "old_string": "a", "new_string": "b"}, ok("ok"))
def bash(command, outcome): return ("Bash", {"command": command}, outcome)
cases = [
    ("误放：两条都接 | head -2，命名检查被 SIGPIPE 杀掉（真跑是 PIPESTATUS=141 0），工具退 0", 2,
     [edit, bash(f"{STAGE} 2>&1 | head -2", ok("── vague-names")), bash(f"{LINT} 2>&1 | head -2", ok("\n══ 命名纪律 ══"))]),
    ("误放：两条都退 2（用法错）而后面接 ; echo rc=$?，工具退 0", 2,
     [edit, bash(f"{STAGE} > /tmp/n1 2>&1; echo rc=$?", ok("rc=2")), bash(f"{LINT} > /tmp/n2 2>&1; echo rc=$?", ok("rc=2"))]),
    ("误放：两条都接 || true，真退 3（naming-lint 没有 .rs 可查）也退 0", 2,
     [edit, bash(f"{STAGE} || true", ok("! 本次无对象可判")), bash(f"{LINT} || true", ok("没有 .rs 可查"))]),
    ("误拒：真会话里的写法 timeout 230 包着、输出落文件、; echo rc=$?（rc=1 是判红跑完），工具退 0", 0,
     [edit, bash(f"D=/tmp/x; {STAGE.replace('bash .claude', 'timeout 230 nice -n 19 bash .claude')} > $D/a.txt 2>&1; echo rc=$?; tail -n 12 $D/a.txt", ok("rc=1\n  ✗ …")),
      bash(f"D=/tmp/x; timeout 230 nice -n 19 {LINT} > $D/b.txt 2>&1; echo rc=$?; tail -n 5 $D/b.txt", ok("rc=1\n  ✗ 3449 处名字不合命名纪律"))]),
    ("对照：同一写法去掉 timeout，工具退 0", 0,
     [edit, bash(f"D=/tmp/x; {STAGE} > $D/a.txt 2>&1; echo rc=$?; tail -n 12 $D/a.txt", ok("rc=1")),
      bash(f"D=/tmp/x; nice -n 19 {LINT} > $D/b.txt 2>&1; echo rc=$?; tail -n 5 $D/b.txt", ok("rc=1"))]),
]
for number, (label, want, records) in enumerate(cases):
    agent = f"mask{number}"
    with open(os.path.join(session, "subagents", f"agent-{agent}.jsonl"), "w", encoding="utf-8") as handle:
        handle.write(json.dumps({"type": "user", "timestamp": "2026-10-23T00:00:00+00:00", "message": {"role": "user", "content": "# 派发"}}, ensure_ascii=False) + "\n")
        for index, (tool, arguments, outcome) in enumerate(records):
            handle.write(json.dumps({"type": "assistant", "cwd": fake, "message": {"role": "assistant", "content": [{"type": "tool_use", "id": f"c{index}", "name": tool, "input": arguments}]}}, ensure_ascii=False) + "\n")
            result = {"type": "tool_result", "tool_use_id": f"c{index}", "content": outcome[1]}
            if outcome[0]:
                result["is_error"] = True
            handle.write(json.dumps({"type": "user", "message": {"role": "user", "content": [result]}}, ensure_ascii=False) + "\n")
    hook_input = {"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "implementation-writer", "agent_id": agent, "transcript_path": session + ".jsonl"}
    codes = []
    for tree in ("tree-orig", "tree-cur"):
        done = subprocess.run(["bash", os.path.join(S, tree, ".claude", "hooks", "handback-guard.sh")], input=json.dumps(hook_input), capture_output=True, text=True,
                              env={**os.environ, "CLAUDE_PROJECT_DIR": fake}, timeout=60)
        codes.append(done.returncode)
    print(f"{label}：改前退 {codes[0]}，改后退 {codes[1]}（判得对的应当是 {want}）")
shutil.rmtree(fake, ignore_errors=True)
shutil.rmtree(session, ignore_errors=True)
