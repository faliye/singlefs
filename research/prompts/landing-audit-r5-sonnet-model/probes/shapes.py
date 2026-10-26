"""探针：造实现员的会话记录，喂旧（tree_old）、新（tree_new）、只认退 0（tree_only0）三棵交回闸，看命名检查几种写法放不放。用法：python3 -I shapes.py <树目录> <仓根> <假仓放哪>"""
import datetime, json, os, shutil, subprocess, sys
W = sys.argv[1]   # 树目录（里面有 tree_old、tree_new、tree_only0）
REPO = sys.argv[2]   # 仓根（只读，拷实现员定义进假仓）
S = sys.argv[3]   # 假仓与假会话记录放哪（新的草稿目录里的子目录）
if os.path.exists(S) and os.listdir(S):
    sys.exit(f"{S} 已存在且非空：给一个新的目录")
fake = S + "/fake-repo"
os.makedirs(fake + "/.git", exist_ok=True)
os.makedirs(fake + "/crates/core/src", exist_ok=True)
os.makedirs(fake + "/.claude/agents", exist_ok=True)
shutil.copy(REPO + "/.claude/agents/implementation-writer.md", fake + "/.claude/agents")
session = S + "/fake-session"
os.makedirs(session + "/subagents", exist_ok=True)
CSD = ("SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; 范围=4 格; "
       "为什么=交回前核自己改动里的命名' {pre}bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names {root}")
LINT = "{pre}bash .claude/singlefs-ai-sop/scripts/naming-lint.sh {root}"
src = fake + "/crates/core/src/lib.rs"
def run(label, commands, hooks):
    out = {}
    for tree in hooks:
        agent = "p" + str(sum(ord(c) for c in label) * 1000 + sum(ord(c) for c in tree))
        with open(f"{session}/subagents/agent-{agent}.jsonl", "w", encoding="utf-8") as h:
            h.write(json.dumps({"type": "user", "timestamp": datetime.datetime(2026, 10, 23).isoformat() + "+00:00", "message": {"role": "user", "content": "# 派发"}}) + "\n")
            calls = [("Edit", {"file_path": src, "old_string": "a", "new_string": "b"}, (False, "ok"))] + [("Bash", {"command": c}, r) for c, r in commands]
            for i, (tool, args, (err, text)) in enumerate(calls):
                h.write(json.dumps({"type": "assistant", "cwd": fake, "message": {"role": "assistant", "content": [{"type": "tool_use", "id": f"c{i}", "name": tool, "input": args}]}}, ensure_ascii=False) + "\n")
                res = {"type": "tool_result", "tool_use_id": f"c{i}", "content": text}
                if err: res["is_error"] = True
                h.write(json.dumps({"type": "user", "message": {"role": "user", "content": [res]}}, ensure_ascii=False) + "\n")
        hook_input = {"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "implementation-writer", "agent_id": agent, "transcript_path": session + ".jsonl"}
        done = subprocess.run(["bash", f"{W}/tree_{tree}/.claude/hooks/handback-guard.sh"], input=json.dumps(hook_input), capture_output=True, text=True,
                              env={**os.environ, "CLAUDE_PROJECT_DIR": fake}, timeout=60)
        out[tree] = done.returncode
    print(f"{label}：" + "  ".join(f"{t}=退{rc}" for t, rc in out.items()))
ok = (False, "ok")
red1 = (True, "Exit code 1\n  ✗ 命中")
trees = ["old", "new", "only0"]
r = fake
# 1 简单写法，两条各自一次 Bash，lint 判红退 1
run("两条各一次，lint 判红退 1", [(CSD.format(pre="", root=r), ok), (LINT.format(pre="", root=r), red1)], trees)
# 2 真会话里 4 份交回的写法：timeout 包着、输出落文件、echo rc 收尾，整条退 0
run("timeout 包着、重定向到文件、echo rc 收尾（整条退 0，lint 其实红）", [(CSD.format(pre="timeout 230 nice -n 19 ", root=r) + " > /x/csd.out 2>&1; echo \"csd rc=$?\"", ok), (LINT.format(pre="timeout 230 nice -n 19 ", root=r) + " > /x/lint.out 2>&1; echo \"rc=$?\"; wc -l < /x/lint.out", ok)], trees)
# 3 同上但去掉 timeout
run("重定向到文件、echo rc 收尾，没有 timeout（整条退 0）", [(CSD.format(pre="nice -n 19 ", root=r) + " > /x/csd.out 2>&1; echo \"csd rc=$?\"", ok), (LINT.format(pre="nice -n 19 ", root=r) + " > /x/lint.out 2>&1; echo \"rc=$?\"; wc -l < /x/lint.out", ok)], trees)
# 4 timeout 包着、就是最后一条、退 0
run("timeout 包着、就是整条的最后一条（退 0）", [(CSD.format(pre="timeout 230 ", root=r), ok), (LINT.format(pre="timeout 230 ", root=r), ok)], trees)
# 5 两条用 ; 连在一条里，后一条判红退 1
run("两条用 ; 连在一条里，后一条判红退 1", [(CSD.format(pre="", root=r) + "; " + LINT.format(pre="", root=r), red1)], trees)
