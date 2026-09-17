#!/usr/bin/env bash
# PreToolUse hook（Write、Edit）：项目 subagent 写文件时，目标必须落在它登记的写范围里。
#
# 为什么：执行类 agent（implementation-writer、kb-scribe、experiment-runner）越界写，
# 不该等人翻 diff 才发现（records/2026-09-16-subagent拆分提案.md 第五节，用户 2026-09-17 定走项目 settings hook）。
# 定义文件上挂 hook 在这个仓里整段不注册（文件夹没被标成受信任，同一份计划第十二节实测），
# 而项目 settings 的 hook 生效、输入里带 agent_type，所以闸挂在这里、按 agent_type 查表。
#
# 判法：输入里没有 agent_type（主 agent）放行；agent_type 没有项目定义（内置 agent）放行；
# 有项目定义而表里没有它的写范围，拒绝；目标路径规范化之后命中它的任一模式才放行，否则退出码 2，stderr 交给模型。
# 表在同目录的 agent-write-scope.tsv。拦不住 Bash 里的写，定义的「没做什么」照写。
#
#   agent-write-scope.sh             # 从 stdin 读 hook 的 JSON
#   agent-write-scope.sh --selftest  # 在临时目录里走一遍放行与拒绝的各种情形；AGENT_WRITE_SCOPE_DISABLE_CHECK=1 时自检必须判红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
# 写成 `python3 - <<PY` 时程序占了标准输入，JSON 读不到，判定一律放行——2026-09-17 自己写出来、用 stdin 形态实测撞到的。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, shutil, subprocess, sys, tempfile

def glob_to_regex(pattern):
    parts, index = [], 0
    while index < len(pattern):
        if pattern.startswith("**", index):
            parts.append(".*"); index += 2
        elif pattern[index] == "*":
            parts.append("[^/]*"); index += 1
        else:
            parts.append(re.escape(pattern[index])); index += 1
    return re.compile("^" + "".join(parts) + "$")

def load_scopes(table_path):
    scopes = {}
    for line in open(table_path, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) >= 2 and fields[0].strip() and fields[1].strip():
            scopes.setdefault(fields[0].strip(), []).append(fields[1].strip())
    return scopes

def decide(hook_input, project_root, table_path):
    """返回 (0, None) 放行；(2, 说明) 拒绝。"""
    agent_type = hook_input.get("agent_type")
    if not agent_type:
        return 0, None
    if not os.path.isfile(os.path.join(project_root, ".claude", "agents", f"{agent_type}.md")):
        return 0, None
    if os.environ.get("AGENT_WRITE_SCOPE_DISABLE_CHECK") == "1":
        return 0, None
    scopes = load_scopes(table_path)
    patterns = scopes.get(agent_type, [])
    file_path = (hook_input.get("tool_input") or {}).get("file_path") or ""
    if not patterns:
        return 2, (f"✗ {agent_type} 在写范围表里没有登记，按拒绝处理：{file_path}\n"
                   f"→ 怎么办：在 .claude/hooks/agent-write-scope.tsv 按它定义的「写范围」一节补上路径模式；补之前这件活交回主 agent。")
    absolute = os.path.normpath(file_path if os.path.isabs(file_path) else os.path.join(project_root, file_path))
    root = os.path.normpath(project_root)
    relative = os.path.relpath(absolute, root) if absolute == root or absolute.startswith(root + os.sep) else None
    for pattern in patterns:
        target = absolute if pattern.startswith("/") else relative
        if target is not None and glob_to_regex(pattern).match(target):
            return 0, None
    return 2, (f"✗ {agent_type} 的写范围不含 {absolute}\n"
               f"→ 它的写范围：{'、'.join(patterns)}（.claude/hooks/agent-write-scope.tsv）。\n"
               f"→ 怎么办：范围外的改动不自己做，写进报告交回主 agent，由主 agent 改或派该写的 agent。")

def selftest(hook_dir):
    work = tempfile.mkdtemp()
    try:
        os.makedirs(os.path.join(work, ".claude", "agents"))
        for name in ("kb-scribe", "implementation-writer", "experiment-runner", "unscoped-writer"):
            open(os.path.join(work, ".claude", "agents", f"{name}.md"), "w").write("---\n")
        table = os.path.join(work, "scope.tsv")
        shutil.copyfile(os.path.join(hook_dir, "agent-write-scope.tsv"), table)
        def case(agent, path, want):
            hook_input = {"tool_name": "Edit", "tool_input": {"file_path": path}}
            if agent: hook_input["agent_type"] = agent
            got, _ = decide(hook_input, work, table)
            return (agent, path, want, got)
        cases = [
            case(None, f"{work}/crates/a.rs", 0),
            case("general-purpose", f"{work}/crates/a.rs", 0),
            case("kb-scribe", f"{work}/.claude/kb/decisions/01-x.md", 0),
            case("kb-scribe", f"{work}/crates/a.rs", 2),
            case("kb-scribe", f"{work}/records/x.md", 2),
            case("implementation-writer", f"{work}/crates/singlefs-core/src/a.rs", 0),
            case("implementation-writer", f"{work}/crates/../.claude/kb/x.md", 2),
            case("implementation-writer", f"{work}/research/x.md", 2),
            case("implementation-writer", "/tmp/claude-1000/agents/implementation-writer/report.md", 0),
            case("implementation-writer", "/etc/passwd", 2),
            case("experiment-runner", f"{work}/research/prompts/e12-preregistration.md", 0),
            case("experiment-runner", f"{work}/research/prompts/_e12-body.md", 2),
            case("experiment-runner", f"{work}/research/scripts/replay.sh", 0),
            case("experiment-runner", f"{work}/research/scripts/mutate.sh", 2),
            case("unscoped-writer", f"{work}/anything.md", 2),
        ]
        # 走真实入口：把脚本当子进程、从标准输入喂 JSON，防「判定函数对、入口读不到输入」
        script = os.path.join(hook_dir, "agent-write-scope.sh")
        def stdin_case(label, hook_input, want):
            completed = subprocess.run(["bash", script], input=json.dumps(hook_input), capture_output=True, text=True,
                                       env=dict(os.environ, CLAUDE_PROJECT_DIR=work))
            return (label, hook_input.get("tool_input", {}).get("file_path"), want, completed.returncode)
        cases += [
            stdin_case("stdin:主 agent", {"tool_name": "Write", "tool_input": {"file_path": f"{work}/crates/a.rs", "content": "x"}}, 0),
            stdin_case("stdin:kb-scribe 越界", {"tool_name": "Edit", "agent_type": "kb-scribe", "tool_input": {"file_path": f"{work}/crates/a.rs"}}, 2),
            stdin_case("stdin:kb-scribe 范围内", {"tool_name": "Edit", "agent_type": "kb-scribe", "tool_input": {"file_path": f"{work}/.claude/kb/x.md"}}, 0),
            stdin_case("stdin:大内容越界", {"tool_name": "Write", "agent_type": "implementation-writer", "tool_input": {"file_path": f"{work}/research/big.md", "content": "y" * 300000}}, 2),
        ]
    finally:
        shutil.rmtree(work)
    failures = [item for item in cases if item[2] != item[3]]
    for agent, path, want, got in failures:
        print(f"  ✗ 自检：agent={agent} 路径={path} 应当返回 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide() 的判法；AGENT_WRITE_SCOPE_DISABLE_CHECK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过：主 agent 与内置 agent 放行、范围内放行、范围外与 .. 绕路与未登记的项目 agent 拒绝（查了 {len(cases)} 种情形）")
    return 0

hook_dir = sys.argv[1]
if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
    sys.exit(selftest(hook_dir))
raw = sys.stdin.read()
try:
    hook_input = json.loads(raw)
except ValueError:
    sys.exit(0)
project_root = os.environ.get("CLAUDE_PROJECT_DIR") or hook_input.get("cwd") or os.getcwd()
code, message = decide(hook_input, project_root, os.path.join(hook_dir, "agent-write-scope.tsv"))
if code:
    print(message, file=sys.stderr)
sys.exit(code)
PY
