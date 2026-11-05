"""文档整理所用的顶层代码围栏状态，不是完整 Markdown 解析器。

记录打开围栏的字符与长度；只有同字符、长度足够、尾部只有空白的行能关闭。
支持零到三个前导空格；反引号围栏的信息串不能含反引号。
不解析列表、引用块内部的容器缩进。fenced_flags 包含开闭围栏行，未闭合时保护到文末。
"""
import re

FENCE = re.compile(r'^ {0,3}(`{3,}|~{3,})(.*)$')


def advance_fence(line, state):
    match = FENCE.match(line)
    if not match:
        return state
    marker, tail = match.groups()
    if not state:
        if marker[0] == '`' and '`' in tail:
            return None
        return marker[0], len(marker)
    if marker[0] == state[0] and len(marker) >= state[1] and not tail.strip():
        return None
    return state


def fenced_flags(lines):
    state = None
    flags = []
    for line in lines:
        previous = state
        state = advance_fence(line, state)
        flags.append(bool(previous or state))
    return flags
