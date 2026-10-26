"""K2 交回闸 ⑤：定义自己教的几种跑法与几种跑完了的写法，造假会话记录（改一份主工作区 crates/ 下的 .rs，再按给定的命令与工具结果跑两条命名检查），
逐棵树喂 handback-guard.sh 入口，看退出码。工具结果照 Claude Code 的样子写：退 0 的不是错误、原样输出；退非 0 的 is_error、开头「Exit code N」；
输出太长退 0 的落成 persisted-output（只留开头预览）。命令不真跑，判的是钩子读记录的那一层。
用法：python3 -I naming_forms.py <草稿目录> <仓根（只读，当 CLAUDE_PROJECT_DIR）> <树名=树根>…"""
import json, os, shutil, subprocess, sys
scratch, root = sys.argv[1:3]
trees = [argument.split("=", 1) for argument in sys.argv[3:]]
SCOPE = ("SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; "
         "范围=4 格; 为什么=交回前核自己改动里的命名'")
STAGE = f"{SCOPE} bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names {root}"
LINT = f"bash .claude/singlefs-ai-sop/scripts/naming-lint.sh {root}"
STAGE_RED = "  ✗ vague-names：别的文件里 3 处\n── 汇总\nvague-names：红\n✗ code-source-discipline：4 格里 1 格红"
LINT_DONE = "naming-lint: 查了 812 份 .rs，命中 2 处（都不在这一批改的文件里）"
BIG = "<persisted-output>\nOutput too large (61.2KB). Full output saved to: /home/x/tool-results/b.txt\n\nPreview (first 2KB):\n" + "  ✗ 一行命中\n" * 40 + "...\n</persisted-output>"
def result(code, output):
    return (code, output)
CASES = [
    ("两条各自单独跑、判红退 1（对照）", [(STAGE, (1, STAGE_RED)), (LINT, (1, LINT_DONE))], 0),
    ("前面加 nice -n 19（共用约束「跑脚本、跑模型一律加 nice -n 19」）、退 1", [(f"{SCOPE} nice -n 19 bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names {root}", (1, STAGE_RED)), (f"nice -n 19 {LINT}", (1, LINT_DONE))], 0),
    ("经 two-logs.sh 包着落盘（共用约束「要落盘就经 two-logs.sh」）、退 1", [(f"{SCOPE} bash scripts/hosts/two-logs.sh /home/fy5090/.local/share/singlefs/work/x/naming-stage -- bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names {root}", (1, STAGE_RED)), (f"bash scripts/hosts/two-logs.sh /home/fy5090/.local/share/singlefs/work/x/naming-lint -- {LINT}", (1, LINT_DONE))], 0),
    ("输出落进草稿目录文件、再读汇总行（共用约束「长命令的输出落进草稿目录的文件，只读汇总行」）", [(f"{STAGE} > /home/fy5090/.local/share/singlefs/work/x/stage.txt 2>&1; tail -n 5 /home/fy5090/.local/share/singlefs/work/x/stage.txt", (0, STAGE_RED)), (f"{LINT} > /home/fy5090/.local/share/singlefs/work/x/lint.txt 2>&1; tail -n 3 /home/fy5090/.local/share/singlefs/work/x/lint.txt", (0, LINT_DONE))], 0),
    ("接 | tail -n 20、输出里有收尾汇总行（第十一批没收的那一类）", [(f"{STAGE} 2>&1 | tail -n 20", (0, STAGE_RED)), (f"{LINT} 2>&1 | tail -n 20", (0, LINT_DONE))], 0),
    ("接 | tail -n 20、后面 ; echo rc=${PIPESTATUS[0]}（打出的是命名检查自己的退出码）", [(f"{STAGE} 2>&1 | tail -n 20; echo rc=${{PIPESTATUS[0]}}", (0, STAGE_RED + "\nrc=1")), (f"{LINT} 2>&1 | tail -n 20; echo rc=${{PIPESTATUS[0]}}", (0, LINT_DONE + "\nrc=1"))], 0),
    ("set -o pipefail 之后接 | tail -n 20、整条退 1", [(f"set -o pipefail; {STAGE} 2>&1 | tail -n 20", (1, STAGE_RED)), (f"set -o pipefail; {LINT} 2>&1 | tail -n 20", (1, LINT_DONE))], 0),
    ("( 两条 ; echo rc=$? ) | tail -n 5（rc 行走管道照样打出来）", [(f"( {STAGE} 2>&1; echo rc=$? ) | tail -n 5", (0, STAGE_RED + "\nrc=1")), (f"( {LINT} 2>&1; echo rc=$? ) | tail -n 5", (0, LINT_DONE + "\nrc=1"))], 0),
    ("timeout 230 包着、; echo rc=$?、输出太长落成 persisted-output（rc 行在预览之外）", [(f"timeout 230 {STAGE} 2>&1; echo rc=$?", (0, BIG)), (f"timeout 230 {LINT} 2>&1; echo rc=$?", (0, BIG))], 0),
    ("两条用 && 连着、前一条退 1（后一条没跑），整条退 1", [(f"{STAGE} && {LINT}", (1, STAGE_RED))], 2),
    ("两条接 || true（退出码被吃掉，对照）", [(f"{STAGE} || true", (0, STAGE_RED)), (f"{LINT} || true", (0, LINT_DONE))], 2),
]
base = os.path.join(scratch, "forms")
shutil.rmtree(base, ignore_errors=True)
for number, (label, calls, expected) in enumerate(CASES, 1):
    session = os.path.join(base, f"case{number}", "session")
    os.makedirs(os.path.join(session, "subagents"))
    open(session + ".jsonl", "w").close()
    records = [{"type": "user", "timestamp": "2026-10-23T00:00:00Z", "cwd": root, "message": {"role": "user", "content": "派发提示"}},
               {"type": "assistant", "cwd": root, "message": {"role": "assistant", "content": [{"type": "tool_use", "id": "e1", "name": "Edit", "input": {"file_path": f"{root}/crates/singlefs-harness/src/lib.rs"}}]}},
               {"type": "user", "cwd": root, "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "e1", "content": "ok"}]}}]
    for index, (command, (code, output)) in enumerate(calls):
        records.append({"type": "assistant", "cwd": root, "message": {"role": "assistant", "content": [{"type": "tool_use", "id": f"b{index}", "name": "Bash", "input": {"command": command}}]}})
        block = {"type": "tool_result", "tool_use_id": f"b{index}", "content": output if code == 0 else f"Exit code {code}\n{output}"}
        if code != 0:
            block["is_error"] = True
        records.append({"type": "user", "cwd": root, "message": {"role": "user", "content": [block]}})
    with open(os.path.join(session, "subagents", "agent-aform0000000000.jsonl"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(json.dumps(record, ensure_ascii=False) for record in records) + "\n")
    hook_input = {"hook_event_name": "PreToolUse", "tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "tooling-writer",
                  "agent_id": "aform0000000000", "transcript_path": session + ".jsonl", "cwd": root}
    verdicts = []
    for name, tree in trees:
        done = subprocess.run(["bash", os.path.join(tree, ".claude", "hooks", "handback-guard.sh")], input=json.dumps(hook_input), capture_output=True,
                              text=True, env={**os.environ, "CLAUDE_PROJECT_DIR": root}, timeout=120)
        verdicts.append(f"{name} 退 {done.returncode}")
    print(f"{label}：{'，'.join(verdicts)}（真跑完了没有：{'跑完了，该放行（0）' if expected == 0 else '没跑完或认不出，该拒（2）'}）")
shutil.rmtree(base, ignore_errors=True)
