#!/usr/bin/env python3
"""会话编号收工检查的探针：同一组回复文本，各判三遍——规则说该怎样（want）、照钩子源码里的正则重做一遍判法（model）、真起钩子喂 Stop 输入（real）。

用法：stop_check_cases.py --repo <仓根> --work <工作目录> [--model-only]
  model：从仓里两份源码的文本里按行取出 FENCED_BLOCK、QUOTED_SPANS、LETTER_NAME 三条正则（只解析、不执行项目代码），按钩子的 judged_reply_text 与 letter_hits 的次序重做。
  real：起 bash <仓根>/.claude/hooks/session-number-stop-check.sh，输入 JSON 走标准输入，状态目录与检出记录都写 <工作目录>，每次起放进自己的进程组。
输出：每例一行 name=case 起头；末行 name=done。退出码 0 跑完；2 用法错。
"""
import ast
import json
import os
import re
import subprocess
import sys

HOOK_RELATIVE = ".claude/hooks/session-number-stop-check.sh"
UPSTREAM_RELATIVE = ".claude/singlefs-ai-sop/scripts/courtesy-phrase.py"


def regex_from_source(path, name):
    """文件里 `NAME = re.compile(<字面串>)` 那一行的正则。"""
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            matched = re.match(rf"{name} = re\.compile\((.*)\)\s*$", line)
            if matched:
                return re.compile(ast.literal_eval(matched.group(1)))
    raise SystemExit(f"{path} 里找不到 {name}")


def model_blocked(reply, fenced, quoted, letter):
    text = quoted.sub("", fenced.sub("", reply))
    return any(letter.search(line) for line in text.splitlines())


# (标签, 回复, 规则说该不该拦, 备注)；want 取 block 或 pass
SINGLE_CASES = (
    ("正文里写", "mainagent(A) 改了 scripts/x.py。", "block", ""),
    ("全角括号", "mainagent（A）改了 x.py。", "block", ""),
    ("半角全角混写", "mainagent(A）改了 x.py。", "block", ""),
    ("列表项", "已交回：\n- mainagent(C) 改了 c.py", "block", ""),
    ("表格单元格", "| 会话 | 改动 |\n|---|---|\n| mainagent(B) | x.py |", "block", ""),
    ("引用块", "> mainagent(C) 改了 c.py", "block", ""),
    ("加粗", "**mainagent(A)** 改了 x.py。", "block", ""),
    ("圆括号里", "（mainagent(A) 在改 x.py）", "block", ""),
    ("链接文字里", "[mainagent(A)](x.md) 改了 x.py。", "block", ""),
    ("单引号里", "他们写 'mainagent(A)' 改了 x.py。", "block", "单引号不在放行的几种里"),
    ("闭合的代码块之后", "```\n输出\n```\nmainagent(A) 改了 x.py。", "block", ""),
    ("反引号之后同一行", "`x.py` 是 mainagent(A) 改的。", "block", ""),
    ("两个「」之间", "「甲」与 mainagent(A) 与「乙」。", "block", ""),
    ("ASCII 双引号成对之外", '他说 "好" 之后 mainagent(A) 改了 x.py。', "block", ""),
    ("奇数个 ASCII 双引号夹住", '屏幕 5" 宽，mainagent(A) 改了 x.py，后文 "再说"。', "block", "引号配对错位"),
    ("落单的反引号夹住", "用 ` 转义，mainagent(A) 改了 x.py，另见 `y`。", "block", "反引号配对错位"),
    ("首字母大写", "Mainagent(A) 改了 x.py。", "block", "规则字面是小写；用户照样看不出是谁"),
    ("驼峰", "MainAgent(A) 改了 x.py。", "block", "同上"),
    ("中间有空格的名字", "main agent(A) 改了 x.py。", "block", "同上"),
    ("名字与括号之间有空格", "mainagent (A) 改了 x.py。", "block", "同上"),
    ("两个大写字母", "mainagent(AB) 改了 x.py。", "block", "文件头自认管不到"),
    ("全角字母", "mainagent(Ａ) 改了 x.py。", "block", "同上"),
    ("括号内换行", "mainagent(\nA) 改了 x.py。", "block", "文件头自认管不到"),
    ("反引号", "规则里的写法是 `mainagent(A)`。", "pass", ""),
    ("双反引号", "规则里的写法是 ``mainagent(A)``。", "pass", "两个反引号写的行内代码"),
    ("跨行的反引号", "规则里的写法是 `mainagent(A)\n` 这种。", "pass", "行内代码跨行"),
    ("中文引号", "不写「mainagent(A)」这类字母。", "pass", ""),
    ("弯引号", "不写“mainagent(A)”这类字母。", "pass", ""),
    ("ASCII 双引号", '不写 "mainagent(A)" 这类字母。', "pass", ""),
    ("三反引号代码块", "输出：\n```\nmainagent(A) 改了 x\n```\n共 1 行。", "pass", ""),
    ("带语言的代码块", "```text\nmainagent(A) 改了 x\n```", "pass", ""),
    ("波浪线代码块", "~~~\nmainagent(A) 改了 x\n~~~", "pass", ""),
    ("四空格缩进的代码块", "输出：\n\n    mainagent(A) 改了 x\n\n共 1 行。", "pass", "缩进代码块"),
    ("四反引号里嵌三反引号", "````\n```\nx\n```\nmainagent(A) 改了 x\n````", "pass", "嵌套的围栏"),
    ("只写编号", "(5b) 改了 scripts/x.py，做了一次换上。", "pass", ""),
    ("子 agent 字母", "subagent(A) 报告了红。", "pass", ""),
    ("占位写法", "把 mainagent(<字母>) 换成编号。", "pass", ""),
    ("只有名字没有括号", "mainagent 的规则写在 agent-common.md。", "pass", ""),
)

# (标签, 步骤列表, 每步想要的结局)；每步是 (钩子输入里的几项, 回复)
SEQUENCE_CASES = (
    ("续跑里同一份命中换了句子", [({"stop_hook_active": False}, "mainagent(A) 改了 x.py。", "block"),
                         ({"stop_hook_active": True}, "好的。mainagent(A) 还改了 y.py，没写编号。", "block")], "规则要求拦；文件头设计成同一份命中不连拦"),
    ("续跑里改成编号", [({"stop_hook_active": False}, "mainagent(A) 改了 x.py。", "block"),
                    ({"stop_hook_active": True}, "(5b) 改了 x.py。", "pass")], ""),
    ("子 agent 收工事件", [({"hook_event_name": "SubagentStop", "agent_id": "probe-agent"}, "mainagent(A) 改了 x.py。", "pass")], ""),
    ("输入带 agent_id", [({"agent_id": "probe-agent"}, "mainagent(A) 改了 x.py。", "pass")], ""),
)


def run_real(repo, work, session, extra, reply):
    hook_input = {"hook_event_name": "Stop", "session_id": session, "last_assistant_message": reply}
    hook_input.update(extra)
    environment = dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "detections.jsonl"),
                       SESSION_NUMBER_STOP_CHECK_STATE_DIRECTORY=os.path.join(work, "state"))
    for name in ("SESSION_NUMBER_STOP_CHECK_BREAK", "SESSION_NUMBER_STOP_CHECK_SELFTEST_MARK"):
        environment.pop(name, None)
    child = subprocess.Popen(["bash", os.path.join(repo, HOOK_RELATIVE)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             text=True, env=environment, cwd=repo, start_new_session=True)
    try:
        _, error_text = child.communicate(json.dumps(hook_input, ensure_ascii=False), timeout=120)
    except subprocess.TimeoutExpired:
        os.killpg(child.pid, 15)
        child.communicate()
        return "timeout", ""
    return {0: "pass", 2: "block"}.get(child.returncode, f"exit{child.returncode}"), error_text


def verdict_word(blocked):
    return "block" if blocked else "pass"


def main(arguments):
    repo = work = None
    model_only = False
    position = 0
    while position < len(arguments):
        if arguments[position] == "--repo" and position + 1 < len(arguments):
            repo, position = arguments[position + 1], position + 2
        elif arguments[position] == "--work" and position + 1 < len(arguments):
            work, position = arguments[position + 1], position + 2
        elif arguments[position] == "--model-only":
            model_only, position = True, position + 1
        else:
            print(__doc__, file=sys.stderr)
            return 2
    if not repo or not work:
        print(__doc__, file=sys.stderr)
        return 2
    os.makedirs(work, exist_ok=True)
    fenced = regex_from_source(os.path.join(repo, UPSTREAM_RELATIVE), "FENCED_BLOCK")
    quoted = regex_from_source(os.path.join(repo, UPSTREAM_RELATIVE), "QUOTED_SPANS")
    letter = regex_from_source(os.path.join(repo, HOOK_RELATIVE), "LETTER_NAME")
    total = model_differs = real_differs = 0
    for index, (label, reply, want, note) in enumerate(SINGLE_CASES):
        total += 1
        model = verdict_word(model_blocked(reply, fenced, quoted, letter))
        real = "skipped" if model_only else run_real(repo, work, f"probe-single-{index}", {}, reply)[0]
        model_differs += model != want
        real_differs += (not model_only) and real != want
        print(f"name=case label={label} want={want} model={model} real={real} note={note or '-'}")
    for index, (label, steps, note) in enumerate(SEQUENCE_CASES):
        for step_index, (extra, reply, want) in enumerate(steps):
            total += 1
            real = "skipped" if model_only else run_real(repo, work, f"probe-sequence-{index}", extra, reply)[0]
            real_differs += (not model_only) and real != want
            print(f"name=case label={label}/步骤{step_index} want={want} real={real} note={note or '-'}")
    print(f"name=done cases={total} model_differs_from_want={model_differs} real_differs_from_want={real_differs}")
    return 0


sys.exit(main(sys.argv[1:]))
