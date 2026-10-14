#!/usr/bin/env python3
"""V1：四处类型表逐个比——派发闸两张表（经 dispatch-prompt.py 自己的 guard_type_tables 解析）、骨架两张表、
共用约束 agent-common.md 与 main-agent.md 文字里列的两串类型名、各定义里「输入」一节带不带预计挂钟与线程上限两行。
用法：tables.py <冻结副本根>"""
import importlib.util, os, re, sys
root = os.path.realpath(sys.argv[1])
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("dispatch_prompt_under_test", os.path.join(root, "scripts/agents/dispatch-prompt.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
guard_thread, guard_long, problem = module.guard_type_tables()
print("guard_type_tables 问题:", problem)
print("派发闸 线程上限表:", sorted(guard_thread))
print("派发闸 预计挂钟表:", sorted(guard_long))
print("骨架 HEAVY_WORK_TYPES:", sorted(module.HEAVY_WORK_TYPES))
print("骨架 LONG_RUN_TYPES:", sorted(module.LONG_RUN_TYPES))
print("线程表一致:", set(module.HEAVY_WORK_TYPES) == guard_thread, " 挂钟表一致:", set(module.LONG_RUN_TYPES) == guard_long)
def listed(path, start_pat, end_pat):
    text = open(os.path.join(root, path), encoding="utf-8").read()
    start = re.search(start_pat, text)
    if start is None:
        return None
    segment = text[start.end():]
    end = re.search(end_pat, segment)
    segment = segment[:end.start()] if end else segment
    return sorted(set(re.findall(r"`([a-z][a-z-]+)`", segment)))
agent_common_14 = listed(".claude/agent-common.md", r"派 (?=`implementation-writer`、`investigator`、`tooling-writer`、`clerk`)", r"没写「预计挂钟")
agent_common_15 = listed(".claude/agent-common.md", r"或派 (?=`implementation-writer`、`investigator`、`tooling-writer`、`experiment-runner`)", r"没写「线程上限：」一行又没有「线程上限已判：<理由>」一行顶着（⑮）|没写这一行（⑮）")
main_agent_15 = listed(".claude/main-agent.md", r"(?:一半」「除以」、中文数词、「分之」「合计」「减」「各」或两个及以上数字的、整数后面接逗号或别的话的|「线程上限：」一行里在一个整数之外还有别的字的)，派 ", r"没写「线程上限：」一行")
main_agent_14 = listed(".claude/main-agent.md", r"⑭ 判的类型：", r"，派这几种都写")
for label, found, expected in [("agent-common.md ⑭ 串", agent_common_14, guard_long), ("agent-common.md ⑮ 串", agent_common_15, guard_thread),
                               ("main-agent.md ⑮ 串", main_agent_15, guard_thread), ("main-agent.md ⑭ 串", main_agent_14, guard_long)]:
    print(f"{label}: {found} 与派发闸{'相同' if found is not None and set(found) == expected else '不同'}")
for name in sorted(os.listdir(os.path.join(root, ".claude/agents"))):
    text = open(os.path.join(root, ".claude/agents", name), encoding="utf-8").read()
    kind = name[:-3]
    has_wall, has_thread = bool(re.search(r"^- 预计挂钟：", text, re.M)), bool(re.search(r"^- 线程上限：", text, re.M))
    print(f"定义 {kind:<24} 输入里有预计挂钟行={has_wall} 线程上限行={has_thread}   派发闸要预计挂钟={kind in guard_long} 要线程上限={kind in guard_thread}")
