#!/usr/bin/env python3
"""攻方腿模型：门禁 code-tooling 的 agent-write-scope 格「Stop 上有命令指向 session-number-stop-check.sh」那条判据，
在注册挪了地方、换了命令形态时判得对不对（正文第 7 问）。

用法：
  python3 gate_variants.py build <被判树> <运行目录>       从被判树的 agent-write-scope-green 样本拷出各变体，改 .claude/settings.json，写 <运行目录>/variants/<名>/
  python3 gate_variants.py summarize <运行目录>            读 rerun.sh 逐个跑出的 <运行目录>/gate-<名>.out 与 .rc，逐变体打一行
每个变体一行：name=gate-<名> gate_wants=<0|1> gate_exit=<门禁退出码> named_line=<yes|no> verdict=<ok|hit|undetermined>
  gate_wants：钩子在这一形态下真会在主 agent 收工时判（照 hook_probes.py 的 7-hook-side 读数与 Claude Code 的起法）就该绿（0），不会就该红（1）；
  undetermined：要 Claude Code 自己的语义才定得了该绿该红（Stop 上的 matcher、hook 的 type、timeout），只打门禁的结局。
"""
import json
import os
import shutil
import sys

FIXTURE = os.path.join(".claude", "gate.d", "fixtures", "code-tooling.sh", "agent-write-scope-green")
SCRIPT = "session-number-stop-check.sh"
REGISTERED = 'bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh'
NAMED_LINE = "没注册会话编号收工检查"

# (名, gate_wants, 怎么改)。gate_wants None 是 undetermined。
VARIANTS = (
    ("green", 0, "原样"),
    ("removed", 1, "Stop 里删掉这一条（与红样本同）"),
    ("moved-to-subagentstop", 1, "挪到 SubagentStop"),
    ("moved-to-userpromptsubmit", 1, "挪到 UserPromptSubmit"),
    ("mention-only-echo", 1, "命令写成 echo 这个路径"),
    ("stop-with-matcher", None, "Stop 那一条加 matcher Bash"),
    ("selftest-argument", 1, "命令末尾加 --selftest"),
    ("or-true", 1, "命令末尾加 || true"),
    ("python3-interpreter", 1, "解释器写成 python3"),
    ("sh-interpreter", 1, "解释器写成 sh"),
    ("other-file-same-name", 1, "路径指到别处一份同名文件"),
    ("type-prompt", None, "type 写成 prompt、command 照留"),
    ("timeout-one-second", None, "timeout 写成 1"),
    ("disable-all-hooks", None, "settings.json 顶层 disableAllHooks 为 true"),
    ("cd-then-relative", 0, "cd 到项目根再相对路径起"),
    ("env-bash", 0, "前面加 env"),
    ("assignment-prefix", 0, "前面加 LC_ALL=C.UTF-8"),
    ("bash-option", 0, "bash 带 -e"),
    ("direct-exec", 0, "不写解释器、直接起（钩子有执行位）"),
)

COMMANDS = {
    "mention-only-echo": "echo .claude/hooks/session-number-stop-check.sh",
    "selftest-argument": REGISTERED + " --selftest",
    "or-true": REGISTERED + " || true",
    "python3-interpreter": 'python3 "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh',
    "sh-interpreter": 'sh "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh',
    "other-file-same-name": 'bash "$CLAUDE_PROJECT_DIR"/.claude/gate.d/fixtures/stub/session-number-stop-check.sh',
    "cd-then-relative": 'cd "$CLAUDE_PROJECT_DIR" && bash .claude/hooks/session-number-stop-check.sh',
    "env-bash": 'env bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh',
    "assignment-prefix": 'LC_ALL=C.UTF-8 bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh',
    "bash-option": 'bash -e "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh',
    "direct-exec": '"$CLAUDE_PROJECT_DIR"/.claude/hooks/session-number-stop-check.sh',
}


def split_entry(settings):
    """把 Stop 里指向这个钩子的那一条摘出来，交 (摘掉之后的 settings, 那一条 hook)。"""
    stop = settings["hooks"]["Stop"]
    for entry in stop:
        for hook in list(entry.get("hooks") or []):
            if hook.get("command") == REGISTERED:
                entry["hooks"].remove(hook)
                return settings, hook
    raise SystemExit("绿样本的 Stop 里找不到原样的注册命令：样本变了，模型要改")


def build(tree, work):
    source = os.path.join(tree, FIXTURE)
    for name, _, _ in VARIANTS:
        target = os.path.join(work, "variants", name)
        shutil.copytree(source, target, symlinks=True)
        path = os.path.join(target, ".claude", "settings.json")
        with open(path, encoding="utf-8") as handle:
            settings = json.load(handle)
        if name != "green":
            settings, hook = split_entry(settings)
            if name in COMMANDS:
                hook["command"] = COMMANDS[name]
                settings["hooks"]["Stop"].append({"hooks": [hook]})
            elif name.startswith("moved-to-"):
                event = {"moved-to-subagentstop": "SubagentStop", "moved-to-userpromptsubmit": "UserPromptSubmit"}[name]
                settings["hooks"].setdefault(event, []).append({"hooks": [hook]})
            elif name == "stop-with-matcher":
                settings["hooks"]["Stop"].append({"matcher": "Bash", "hooks": [hook]})
            elif name == "type-prompt":
                hook["type"] = "prompt"
                hook["prompt"] = "回复里有没有 mainagent 加字母"
                settings["hooks"]["Stop"].append({"hooks": [hook]})
            elif name == "timeout-one-second":
                hook["timeout"] = 1
                settings["hooks"]["Stop"].append({"hooks": [hook]})
            elif name == "disable-all-hooks":
                settings["disableAllHooks"] = True
                settings["hooks"]["Stop"].append({"hooks": [hook]})
        with open(path, "w", encoding="utf-8") as handle:
            json.dump(settings, handle, ensure_ascii=False, indent=2)
            handle.write("\n")
        print(f"built={name}")
    return 0


def summarize(work):
    wrong = 0
    for name, want, change in VARIANTS:
        try:
            with open(os.path.join(work, f"gate-{name}.rc"), encoding="utf-8") as handle:
                code = handle.read().strip()
            with open(os.path.join(work, f"gate-{name}.out"), encoding="utf-8", errors="replace") as handle:
                named = NAMED_LINE in handle.read()
        except OSError:
            print(f"name=gate-{name} gate_wants={want} gate_exit=- named_line=- verdict=control-broken change={change}")
            wrong += 1
            continue
        if want is None:
            verdict = "undetermined"
        elif name in ("green", "removed"):
            matched = (code == "0") == (want == 0) and named == (want == 1)
            verdict = "control-ok" if matched else "control-broken"
            wrong += 0 if matched else 1
        else:
            verdict = "ok" if (code == "0") == (want == 0) and named == (want == 1) else "hit"
        print(f"name=gate-{name} gate_wants={want if want is not None else '?'} gate_exit={code} "
              f"named_line={'yes' if named else 'no'} verdict={verdict} change={change}")
    return 1 if wrong else 0


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "build":
        sys.exit(build(sys.argv[2], sys.argv[3]))
    if len(sys.argv) == 3 and sys.argv[1] == "summarize":
        sys.exit(summarize(sys.argv[2]))
    print(__doc__, file=sys.stderr)
    sys.exit(2)
