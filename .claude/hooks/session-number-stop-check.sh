#!/usr/bin/env bash
# admission: always Claude Code 在主 agent 每一次收工时调它，每一条回复都要现判，上一次的结论不替这一次作保
# run-condition: command python3
# Stop hook：主 agent 收工时，这一轮最后一条给用户的回复里出现 mainagent 或「主 agent」后接括号里一到两个字母的叫法（例 mainagent(A)）的，拦下收工（退出 2，stderr 写命中的那几处与出路）。
#   用户看不出这个字母指的是哪个会话；给用户的回复里指别的会话要写它在 ListAgents 那一行的编号（例「(5b)」），连同它改了哪个文件、做了什么
#   （.claude/rules/writing-discipline-by-review.md「怎么说」）。子 agent 交给主 agent 的报告与会话之间的消息照旧用 mainagent 字母，这里不判。
# hook-events: Stop
# gate-similar: courtesy-phrase-check.sh 同挂 Stop、同样判最后一条回复里的文字，并且这里读它的 courtesy-phrase.py 里取最后一条回复的写法、会话记录与「已拦过一次不连拦」的状态文件；它判客套话与推测说法，词表与出路没有一条相同，它的 FENCED_BLOCK 与 QUOTED_SPANS 认不全 Markdown 的行内代码、缩进代码块、引用块与成对引号，这里自己认，它在上游副本里不许就地改，并不进去
# gate-similar: unanswered-message-check.sh 同挂 Stop，它判别的会话发来的消息回了没有、读会话记录里收到的消息；这里只读最后一条回复的文字，对象与判据都不同
# gate-similar: remind-rearm-stop-guard.sh 同挂 Stop，它读循环提醒的登记目录与 /proc；这里读回复的文字，输入没有一样相同
# gate-similar: completion-cleanup-guard.sh 同挂 Stop，它判认领的目录与仓副本有没有删；这里判回复里的会话叫法，对象不同
# selftest-serial: 自证的格都在内存里判、另起二十来次自己的入口，加起来不到十来秒，开并行省不下时间
# gate-overlap:copy-kept ask-user-placement-guard.sh 判不了时「放行但必记检出」（unjudged_pass）与平常跑拒弄坏开关（refuse_break_outside_selftest）两段照它的写法各一份、不带判据；抽成共用模块要改别的钩子，这一次不碰，交主 agent 定要不要抽
#
# 判法：取回复（钩子输入的 last_assistant_message；没有就读会话记录里最后一段回复，取法是上游 courtesy-phrase.py 的 last_reply），
#   先去掉不算叫法的内容（按 Markdown 认）：围栏代码块（三个以上的反引号或波浪线，同一种符号、不少于开头那一串的一行收尾，长的里面可以嵌短的，找不到收尾的不算）、
#   缩进代码块（前面是空行、缩进四列以上的行；列表项或列表续行后面的缩进行是列表内容，不算）、引用块（> 起头的行，前面可以有缩进与列表符号）、
#   行内代码（任意个反引号配对，可以跨行、不跨空行）、同一行里成对的「」『』“”‘’与 ASCII 双引号、单引号。
#   ASCII 引号开头的前面不是字母数字、后面不是空白，收尾的前面不是空白、后面不是字母数字（撇号与英寸号因此不配对）；行内代码开头的后面不是空白；
#   落单的引号与反引号不算引用，照字面留着，后面的叫法照判。
#   再把留下的文字归一：全角转半角（NFKC）、去零宽字符、去 Markdown 的转义与加粗；然后不分大小写找 mainagent 或「主 agent」（名字里的空白不计）
#   后接括号里的一到两个字母（括号半角全角都认，括号内可以换行，括号另起一行时只认大写字母；括号可省，省了括号只认大写字母），每找到一处记一处。有命中就退 2。
#   表格里的规则原文、路径与整句话同样要放进上面几种引用才放行，没有单独的口子。
#   子 agent 收工放行：hook_event_name 是 SubagentStop，或输入带 agent_id。这个钩子只注册在 Stop 上，子 agent 的收工到不了它，这一条是兜底。
# 不死循环：被拦回来的续跑（stop_hook_active 为 true）里，命中的整行连同回合（会话记录末尾最近的 promptId；输入里没有会话记录算同一回合）与上一次拦下时一字不差就放行；
#   换了整行或换了回合照样再拦。没有命中的收工清掉这一会话记下的指纹，之后的续跑里头一次写同样的叫法照拦。
#   状态目录默认 ${TMPDIR:-/tmp}/session-number-stop-check，SESSION_NUMBER_STOP_CHECK_STATE_DIRECTORY 可改。
# 判不了的照「放行但必记检出」：输入不是 JSON 对象、取不到回复、判的时候抛异常（含读不到上游 courtesy-phrase.py），
#   照常退 0，stderr 写一行卡在哪，检出记录（AGENT_HOOK_DETECTIONS，没设取同目录 lib_work_root.py 的 default_detections_path；写法在它的 record_unjudged_pass）
#   追加一行、结局「只记」，findings 以「会话编号收工检查没判、照常放行」起头；读不到 lib_work_root.py 时 stderr 另说一句没记成。
# 弄坏开关只在自证里认：平常跑（不是 --selftest，也不是自证起的子进程）设了 SESSION_NUMBER_STOP_CHECK_BREAK 就退 2，stderr 点名它只在自证里认；
#   自证起的子进程由环境里的 SESSION_NUMBER_STOP_CHECK_SELFTEST_MARK 认（指的那份文件在、内容是 SELFTEST_MARK_CONTENT）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红（「」里是失败行「✗ 自检：」之后那一段的开头）：
#   SESSION_NUMBER_STOP_CHECK_BREAK=letter-ignored  认不出 mainagent 加字母的写法，每一条回复都放行
#       「必拒：回复正文里写 mainagent(A)」「必拒：一行里两处」「必拒：写在列表项里」「必拒：全角括号也认」「必拒：首字母大写」「必拒：驼峰」「必拒：名字中间有空格」
#       「必拒：名字与括号之间有空格」「必拒：括号里两个大写字母」「必拒：括号里小写字母」「必拒：全角字母」「必拒：全角括号加全角字母」「必拒：括号内换行」
#       「必拒：名字与括号之间换行」「必拒：没有括号」「必拒：括号里的字母加粗」「必拒：括号加了转义」「必拒：名字里夹零宽空格」「必拒：写成主 agent」
#       「必拒：奇数个 ASCII 双引号夹住」「必拒：同一行落单的双引号」「必拒：落单的反引号夹住」「必拒：撇号不当引号」「必拒：引用块之后的正文」「必拒：围栏之后的正文」
#       「必拒：缩进不足四列的行」「必拒：列表项下缩进的子项」「必拒：表格里的规则原文不放进引号照拦」「必拒：路径不放进引号照拦」
#       「stdin：Stop 里写了字母拦下并给出路」「stdin：首次命中拦下」「stdin：续跑里换了说法照样拦」「stdin：没有回复时读会话记录」「stdin：换整行之前先拦一次」
#       「stdin：续跑里同一个名字换了整行照样拦」「stdin：干净回复之前先拦一次」「stdin：干净回复之后的续跑里头一次写同样的叫法照样拦」「stdin：非续跑里头一次命中拦下」
#       「stdin：非续跑里同一份命中再来一次照样拦」「stdin：回合头一次命中拦下」「stdin：换了回合的续跑里头一次写同样一行照样拦」
#   SESSION_NUMBER_STOP_CHECK_BREAK=quoted-spans-read  不去掉行内代码与成对引号里的内容，引用里的写法也判
#       「必放：反引号里」「必放：中文引号里」「必放：弯引号里」「必放：双反引号里」「必放：跨行的反引号里」「必放：『』里」「必放：‘’里」「必放：命令里的单引号里」
#       「必放：整句放进中文引号」「必放：整句放进双引号」「必放：整句放进反引号」「必放：表格里的规则原文放进中文引号」「必放：路径放进中文引号」
#   SESSION_NUMBER_STOP_CHECK_BREAK=fences-read  不去掉围栏代码块里的内容，代码块里的写法也判
#       「必放：代码块里」「必放：波浪线代码块」「必放：四反引号里嵌三反引号」
#   SESSION_NUMBER_STOP_CHECK_BREAK=indented-code-read  不去掉缩进代码块里的内容
#       「必放：四空格缩进的代码块」
#   SESSION_NUMBER_STOP_CHECK_BREAK=blockquote-read  不去掉引用块里的内容，引用块里的写法也判
#       「必拒：引用块之后的正文」「必放：引用块里的规则原文」「必放：列表项里的引用块」
#   SESSION_NUMBER_STOP_CHECK_BREAK=backtick-flank-unchecked  行内代码开头的反引号不看后面是不是空白，落单的反引号也往后配对
#       「必拒：落单的反引号夹住」
#   SESSION_NUMBER_STOP_CHECK_BREAK=quote-flank-unchecked  ASCII 引号不看两侧是什么字符，按出现的先后配对（撇号、英寸号也配）
#       「必拒：奇数个 ASCII 双引号夹住」「必拒：同一行落单的双引号」「必拒：撇号不当引号」
#   SESSION_NUMBER_STOP_CHECK_BREAK=fullwidth-unfolded  不做全角转半角，全角括号与全角字母认不出
#       「必拒：全角括号也认」「必拒：全角字母」「必拒：全角括号加全角字母」
#   SESSION_NUMBER_STOP_CHECK_BREAK=zero-width-kept  不去零宽字符，名字里夹零宽字符认不出
#       「必拒：名字里夹零宽空格」
#   SESSION_NUMBER_STOP_CHECK_BREAK=markdown-marks-kept  不去 Markdown 的转义与加粗，括号被转义、字母被加粗的认不出
#       「必拒：括号里的字母加粗」「必拒：括号加了转义」
#   SESSION_NUMBER_STOP_CHECK_BREAK=whitespace-kept  名字与括号里的空白与换行不压掉，名字中间、名字与括号之间、括号内的空白与换行认不出
#       「必拒：名字中间有空格」「必拒：名字与括号之间有空格」「必拒：括号内换行」「必拒：名字与括号之间换行」「必拒：没有括号」「必拒：写成主 agent」
#   SESSION_NUMBER_STOP_CHECK_BREAK=name-case-matters  区分名字的大小写，只认全小写的 mainagent
#       「必拒：首字母大写」「必拒：驼峰」
#   SESSION_NUMBER_STOP_CHECK_BREAK=lowercase-letter-ignored  括号里只认大写字母
#       「必拒：括号里小写字母」
#   SESSION_NUMBER_STOP_CHECK_BREAK=two-letters-ignored  只认一个字母，两个字母的认不出
#       「必拒：括号里两个大写字母」
#   SESSION_NUMBER_STOP_CHECK_BREAK=bare-letter-ignored  不认省了括号的写法
#       「必拒：没有括号」
#   SESSION_NUMBER_STOP_CHECK_BREAK=chinese-name-unseen  不认「主 agent」，只认 mainagent
#       「必拒：写成主 agent」
#   SESSION_NUMBER_STOP_CHECK_BREAK=first-hit-only  一条回复只记第一处命中
#       「必拒：一行里两处」
#   SESSION_NUMBER_STOP_CHECK_BREAK=subagent-judged  子 agent 收工也判
#       「stdin：SubagentStop 里写字母放行」「stdin：带 agent_id 的收工放行」
#   SESSION_NUMBER_STOP_CHECK_BREAK=loop-guard-off  续跑里同一份命中也照拦
#       「stdin：续跑里同一份命中不连拦」「stdin：同一回合的续跑里同一行不连拦」
#   SESSION_NUMBER_STOP_CHECK_BREAK=loop-guard-any-turn  不看是不是续跑，任何一次收工命中与上一次记下的相同就放行
#       「stdin：非续跑里同一份命中再来一次照样拦」
#   SESSION_NUMBER_STOP_CHECK_BREAK=shown-kept-on-clean  没有命中的收工不清这一会话记下的指纹
#       「stdin：干净回复之后的续跑里头一次写同样的叫法照样拦」
#   SESSION_NUMBER_STOP_CHECK_BREAK=fingerprint-names-only  指纹只记命中的名字，不记整行
#       「stdin：续跑里同一个名字换了整行照样拦」
#   SESSION_NUMBER_STOP_CHECK_BREAK=turn-unchecked  指纹不带回合，换了回合也算同一份
#       「stdin：换了回合的续跑里头一次写同样一行照样拦」
#   SESSION_NUMBER_STOP_CHECK_BREAK=input-unrecorded  输入不是 JSON 对象时只放行、不记检出
#       「stdin：坏 JSON 放行并记一条检出」「stdin：JSON 数组放行并记一条检出」
#   SESSION_NUMBER_STOP_CHECK_BREAK=reply-unrecorded  取不到回复时只放行、不记检出
#       「stdin：取不到回复放行并记一条检出」
#   SESSION_NUMBER_STOP_CHECK_BREAK=exception-unrecorded  判的时候抛异常不接住，照原样抛出（进程退 1，Claude Code 照常放行、不记检出）
#       「判的时候抛异常放行并记一条检出」
# 管不到的：回复当中、不在收工那一条里的写法；发给别的会话与子 agent 的消息；用别的说法指会话（会话名、两位短号、「那一路的会话」）；
#   mainagent 与括号之间隔着别的字的写法、括号里三个以上字母的写法；整句放进引号、反引号、引用块的叫法（规则原文要靠这些才引得了，不判里面是引用还是在指会话）；
#   跨行的引号（成对的引号只认同一行里的）；换成 ListAgents 编号之后写没写它改了哪个文件、做了什么，靠人。
#
#   session-number-stop-check.sh             # 从 stdin 读 hook 的 JSON
#   session-number-stop-check.sh --selftest  # 走一遍必拒与必放的回复，另有二十来例走真实的 stdin 入口
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（写成 `python3 - <<PY` 时程序占了标准输入，JSON 读不到，判定一律放行）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import datetime, importlib.util, json, os, re, subprocess, sys, tempfile, unicodedata

HOOK_NAME = "session-number-stop-check.sh"
BREAK_VARIABLE = "SESSION_NUMBER_STOP_CHECK_BREAK"
SELFTEST_MARK_VARIABLE = "SESSION_NUMBER_STOP_CHECK_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "session-number-stop-check.sh --selftest 起的子进程"
STATE_DIRECTORY_VARIABLE = "SESSION_NUMBER_STOP_CHECK_STATE_DIRECTORY"
UNJUDGED_PREFIX = "会话编号收工检查没判、照常放行"
INPUT_PREFIX_BYTES = 200
RULE_CITATION = ".claude/rules/writing-discipline-by-review.md「怎么说」"
PROMPT_IDENTIFIER = re.compile(r'"promptId":\s*"([^"]+)"')
TURN_READ_WINDOWS = (1 << 18, 1 << 22)

def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")

def load_courtesy(hook_dir):
    """上游 courtesy-phrase.py（取最后一条回复、会话记录与「拦过一次」的状态文件都在它那里）；读不到抛异常，由调用方记检出。"""
    sys.dont_write_bytecode = True
    specification = importlib.util.spec_from_file_location("courtesy_phrase", os.path.join(hook_dir, "..", "singlefs-ai-sop", "scripts", "courtesy-phrase.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module

SENTINEL = "\x00"   # 去掉的引用与代码留下的占位：名字与括号之间隔着它就连不上
ZERO_WIDTH_CHARACTERS = dict.fromkeys(map(ord, "\u200b\u200c\u200d\u200e\u200f\u2060\ufeff\u00ad"))
MARKDOWN_ESCAPE = re.compile(r"\\([!-/:-@\[-`{-~])")
FENCE_LINE = re.compile(r"^[ \t]*(`{3,}|~{3,})(.*)$")
BLOCKQUOTE_LINE = re.compile(r"^[ \t]*(?:(?:[-*+]|\d{1,9}[.)])[ \t]+)*>")
LIST_ITEM_LINE = re.compile(r"^[ \t]*(?:[-*+]|\d{1,9}[.)])[ \t]")
BACKTICK_RUN = re.compile(r"`+")
BLANK_LINE_BREAK = re.compile(r"\n[ \t]*\n")
PAIRED_QUOTES = re.compile(r"「[^」\n]*」|『[^』\n]*』|“[^”\n]*”|‘(?:[^’\n]|’(?=[A-Za-z0-9]))*’(?![A-Za-z0-9])")

def blank_fenced_blocks(text):
    """围栏代码块（三个以上的反引号或波浪线开头，同一种符号、不少于开头那一串的一行收尾；长的围栏里可以嵌短的）整块换成空行；找不到收尾的围栏不算围栏。"""
    lines = text.split("\n")
    kept, position = [], 0
    while position < len(lines):
        opening = FENCE_LINE.match(lines[position])
        if opening and not (opening.group(1)[0] == "`" and "`" in opening.group(2)):
            marker = opening.group(1)
            closing = re.compile("^[ \\t]*" + re.escape(marker[0]) + "{" + str(len(marker)) + ",}[ \\t]*$")
            end = next((later for later in range(position + 1, len(lines)) if closing.match(lines[later])), None)
            if end is not None:
                kept.extend([""] * (end - position + 1))
                position = end + 1
                continue
        kept.append(lines[position])
        position += 1
    return "\n".join(kept)

def indent_width(line):
    width = 0
    for char in line:
        if char == " ":
            width += 1
        elif char == "\t":
            width += 4 - width % 4
        else:
            break
    return width

def blank_indented_code(text):
    """缩进代码块：前面是空行（或在开头）、缩进不少于四列的行起，到下一行不缩进的非空行止；紧跟在列表项或列表续行（缩进两列以上）后面的缩进行是列表的内容，不算代码。"""
    lines = text.split("\n")
    kept = list(lines)
    in_code, previous_blank, context = False, True, None
    for position, line in enumerate(lines):
        if not line.strip():
            previous_blank = True
            continue
        starts = previous_blank and (context is None or not (LIST_ITEM_LINE.match(context) or indent_width(context) >= 2))
        if indent_width(line) >= 4 and (in_code or starts):
            kept[position] = ""
            in_code = True
        else:
            in_code = False
            context = line
        previous_blank = False
    return "\n".join(kept)

def blank_blockquotes(text):
    """引用块：以 > 起头的行（前面可以有缩进与列表符号）整行换成空行。"""
    return "\n".join("" if BLOCKQUOTE_LINE.match(line) else line for line in text.split("\n"))

def blank_code_spans(text):
    """行内代码：n 个反引号开头、同样 n 个反引号收尾，中间可以跨行、不跨空行，整段换成一个占位符（换行留着，行号不变）。
    开头那一串反引号后面是空白、前面有反斜杠的不开头；找不到收尾的照字面留着，落单的反引号因此不算引用（弄坏开关 backtick-flank-unchecked 下后面是空白的也开头）。"""
    runs = [(match.start(), match.end()) for match in BACKTICK_RUN.finditer(text)]
    pieces, kept_from, index = [], 0, 0
    while index < len(runs):
        start, end = runs[index]
        opens = not (start > 0 and text[start - 1] == "\\") and (broken("backtick-flank-unchecked") or (end < len(text) and not text[end].isspace()))
        closing = None
        if opens:
            paragraph_break = BLANK_LINE_BREAK.search(text, end)
            paragraph_end = paragraph_break.start() if paragraph_break else len(text)
            for later in range(index + 1, len(runs)):
                if runs[later][0] >= paragraph_end:
                    break
                if runs[later][1] - runs[later][0] == end - start:
                    closing = later
                    break
        if closing is not None:
            pieces.append(text[kept_from:start])
            pieces.append(SENTINEL + "\n" * text.count("\n", start, runs[closing][1]))
            kept_from = runs[closing][1]
            index = closing + 1
        else:
            index += 1
    pieces.append(text[kept_from:])
    return "".join(pieces)

def blank_ascii_quotes(line):
    """同一行里成对的 ASCII 双引号与单引号：开头的引号前面不是字母数字、后面不是空白，收尾的引号前面不是空白、后面不是字母数字
    （撇号 don't 与 12" 的英寸号因此不配对；弄坏开关 quote-flank-unchecked 下不看两侧，按出现的先后配对）。"""
    unchecked = broken("quote-flank-unchecked")
    spans, opened = [], {}
    for index, char in enumerate(line):
        if char not in "\"'":
            continue
        before = line[index - 1] if index else " "
        after = line[index + 1] if index + 1 < len(line) else " "
        can_open = unchecked or (not (before.isascii() and before.isalnum()) and not after.isspace())
        can_close = unchecked or (not before.isspace() and not (after.isascii() and after.isalnum()))
        if char in opened and can_close:
            spans.append((opened.pop(char), index + 1))
        elif char not in opened and can_open:
            opened[char] = index
    merged = []
    for start, end in sorted(spans):
        if merged and start <= merged[-1][1]:
            merged[-1] = (merged[-1][0], max(merged[-1][1], end))
        else:
            merged.append((start, end))
    pieces, position = [], 0
    for start, end in merged:
        pieces += [line[position:start], SENTINEL]
        position = end
    return "".join(pieces) + line[position:]

def blank_paired_quotes(text):
    """同一行里成对的「」『』“”‘’与 ASCII 双引号、单引号（落单的不算）整段换成占位符。"""
    return "\n".join(blank_ascii_quotes(PAIRED_QUOTES.sub(SENTINEL, line)) for line in text.split("\n"))

def normalized(text):
    """认名字之前把回复归一：全角转半角、去零宽字符、去 Markdown 的转义与加粗（弄坏开关 fullwidth-unfolded、zero-width-kept、markdown-marks-kept 各留下一样）。"""
    if not broken("fullwidth-unfolded"):
        text = unicodedata.normalize("NFKC", text)
    if not broken("zero-width-kept"):
        text = text.translate(ZERO_WIDTH_CHARACTERS)
    if not broken("markdown-marks-kept"):
        text = MARKDOWN_ESCAPE.sub(r"\1", text).replace("*", "").replace("__", "")
    return text

def judged_reply_text(reply):
    """去掉围栏代码块、缩进代码块、引用块、行内代码与成对的引号里的内容，再归一（弄坏开关 fences-read、indented-code-read、blockquote-read、quoted-spans-read 各留下一种）。"""
    text = reply.replace("\r\n", "\n").replace("\r", "\n")
    if not broken("fences-read"):
        text = blank_fenced_blocks(text)
    if not broken("indented-code-read"):
        text = blank_indented_code(text)
    if not broken("blockquote-read"):
        text = blank_blockquotes(text)
    if not broken("quoted-spans-read"):
        text = blank_paired_quotes(blank_code_spans(text))
    return normalized(text)

def letter_name_pattern():
    """mainagent 或「主 agent」（不分大小写，名字里的空白不计）后接括号里一到两个字母（括号内可以换行，括号可省，省了括号只认大写字母）；弄坏开关各去掉其中一样。"""
    blank = "" if broken("whitespace-kept") else r"[ \t]*"
    blank_or_break = "" if broken("whitespace-kept") else r"[ \t]*(?:\n[ \t]*)?"
    names = ["(?<![A-Za-z])main" + blank + "agent"]
    if not broken("chinese-name-unseen"):
        names.append("主" + blank + "agent")
    name = "(?:" + "|".join(names) + ")"
    if not broken("name-case-matters"):
        name = "(?i:" + name + ")"
    count = "{1}" if broken("two-letters-ignored") else "{1,2}"

    def parenthesised(letters):
        return r"\(" + blank_or_break + letters + count + blank_or_break + r"\)"

    bracketed = blank + parenthesised("[A-Z]" if broken("lowercase-letter-ignored") else "[A-Za-z]")
    if not broken("whitespace-kept"):
        bracketed += r"|[ \t]*\n[ \t]*" + parenthesised("[A-Z]")   # 括号另起一行时只认大写字母，免得把换行后的 (a) 列表标记当成字母
    bare = "" if broken("bare-letter-ignored") else "|" + blank + "[A-Z]" + count + "(?![A-Za-z0-9])"
    return re.compile(name + "(?:" + bracketed + bare + ")")

def letter_hits(reply):
    """回复里写了 mainagent 加字母的几处：[(命中的名字，空白压掉, 那一行的原文)]。"""
    if broken("letter-ignored"):
        return []
    original_lines = reply.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    text = judged_reply_text(reply)
    found = []
    for match in letter_name_pattern().finditer(text):
        line = original_lines[min(text.count("\n", 0, match.start()), len(original_lines) - 1)]
        found.append((re.sub(r"\s+", "", match.group(0)), line.strip()))
    return found[:1] if broken("first-hit-only") else found

def from_subagent(hook_input):
    """子 agent 的收工：事件是 SubagentStop，或输入带 agent_id（弄坏开关 subagent-judged 下一概当主 agent）。"""
    if broken("subagent-judged"):
        return False
    return hook_input.get("hook_event_name") == "SubagentStop" or bool(hook_input.get("agent_id"))

def current_turn(hook_input):
    """这一回合（用户这一次输入起到收工）的标识：会话记录末尾最近的 promptId，Stop 钩子拦回来的续跑与拦它的那条回复同一个。
    输入里没有会话记录、读不到、末尾几 MB 里没有 promptId 的交空串（弄坏开关 turn-unchecked 下一概交空串）。"""
    path = hook_input.get("transcript_path")
    if broken("turn-unchecked") or not isinstance(path, str) or not os.path.isfile(path):
        return ""
    try:
        size = os.path.getsize(path)
        with open(path, "rb") as handle:
            for window in TURN_READ_WINDOWS:
                handle.seek(max(0, size - window))
                found = PROMPT_IDENTIFIER.findall(handle.read().decode("utf-8", "replace"))
                if found:
                    return found[-1]
    except OSError:
        return ""
    return ""

def shown_state_file(courtesy, hook_input):
    """这一会话记「上一次拦下的是什么」的状态文件。"""
    state_directory = os.environ.get(STATE_DIRECTORY_VARIABLE) or os.path.join(os.environ.get("TMPDIR") or "/tmp", "session-number-stop-check")
    return courtesy.SESSION_TRANSCRIPT.shown_state_file(state_directory, hook_input, (("session_id", "no-session"),))

def forget_shown(courtesy, hook_input):
    """没有命中的回复：清掉这一会话记下的指纹（弄坏开关 shown-kept-on-clean 下不清）。"""
    if broken("shown-kept-on-clean"):
        return
    try:
        os.remove(shown_state_file(courtesy, hook_input))
    except OSError:
        pass

def already_shown(courtesy, hook_input, hits):
    """被拦回来的续跑里命中的整行连同回合与上一次一字不差：放行。其余情形记下指纹、照拦
    （弄坏开关 fingerprint-names-only 下指纹只记名字、不记整行，loop-guard-any-turn 下不看是不是续跑）。"""
    if broken("loop-guard-any-turn"):
        hook_input = dict(hook_input, stop_hook_active=True)
    transcript = courtesy.SESSION_TRANSCRIPT
    state_file = shown_state_file(courtesy, hook_input)
    fingerprint = transcript.fingerprint_of([current_turn(hook_input)] + [name if broken("fingerprint-names-only") else line for name, line in hits])
    repeated = hook_input.get("stop_hook_active") is True and transcript.stored_fingerprint(state_file) == fingerprint
    transcript.store_fingerprint(state_file, fingerprint)
    return repeated and not broken("loop-guard-off")

def blocked_message(hits):
    listed = "\n".join(f"    {name}：{line[:80]}" for name, line in hits)
    return (f"✗ 最后一条回复里有 {len(hits)} 处 mainagent 加字母的叫法，用户看不出那个字母指的是哪个会话：\n"
            f"{listed}\n"
            f"→ 怎么办：把 mainagent(<字母>) 换成它在 ListAgents 那一行的编号（例「(5b)」），连同它改了哪个文件、做了什么，重写这一条回复；"
            f"依据 {RULE_CITATION}。确实是在引用这种写法，放进反引号、围栏代码块、引用块或成对的引号里（落单的引号与反引号不算）。")

def decide(hook_input, hook_dir):
    """返回 (退出码, stderr 说明或 None)；取不到回复交 (None, 原因)；读不到上游脚本等的异常由调用方接。"""
    if from_subagent(hook_input):
        return 0, None
    courtesy = load_courtesy(hook_dir)
    reply = courtesy.last_reply(hook_input)
    if reply is None:
        return None, "输入里没有 last_assistant_message，会话记录里也取不到回复"
    hits = letter_hits(reply)
    if not hits:
        forget_shown(courtesy, hook_input)
        return 0, None
    if already_shown(courtesy, hook_input, hits):
        return 0, None
    return 2, blocked_message(hits)

SAMPLE_REPLIES = (
    # (说明, 回复, 应当命中的处数)
    ("必拒：回复正文里写 mainagent(A)", "mainagent(A) 改了 scripts/x.py，做了一次换上。", 1),
    ("必拒：一行里两处", "mainagent(A) 改了 a.py，mainagent(B) 改了 b.py。", 2),
    ("必拒：写在列表项里", "已交回：\n- mainagent(C) 改了 c.py\n- 另一份已推", 1),
    ("必拒：全角括号也认", "mainagent（D）改了 d.py。", 1),
    ("必拒：首字母大写", "Mainagent(A) 改了 x.py。", 1),
    ("必拒：驼峰", "MainAgent(A) 改了 x.py。", 1),
    ("必拒：名字中间有空格", "main agent(A) 改了 x.py。", 1),
    ("必拒：名字与括号之间有空格", "mainagent (A) 改了 x.py。", 1),
    ("必拒：括号里两个大写字母", "mainagent(AB) 改了 x.py。", 1),
    ("必拒：括号里小写字母", "mainagent(a) 改了 x.py。", 1),
    ("必拒：全角字母", "mainagent(Ａ) 改了 x.py。", 1),
    ("必拒：全角括号加全角字母", "mainagent（Ａ）改了 x.py。", 1),
    ("必拒：括号内换行", "mainagent(\nA) 改了 x.py。", 1),
    ("必拒：名字与括号之间换行", "这一处是 mainagent\n(A) 改的：x.py。", 1),
    ("必拒：没有括号", "mainagent A 改了 x.py。", 1),
    ("必拒：括号里的字母加粗", "mainagent(**A**) 改了 x.py。", 1),
    ("必拒：括号加了转义", "mainagent\\(A\\) 改了 x.py。", 1),
    ("必拒：名字里夹零宽空格", "main​agent(A) 改了 x.py。", 1),
    ("必拒：写成主 agent", "主 agent(A) 改了 x.py。", 1),
    ("必拒：奇数个 ASCII 双引号夹住", "屏幕 5\" 宽，mainagent(A) 改了 x.py，后文 \"再说\"。", 1),
    ("必拒：同一行落单的双引号", "x.py 里那一行写的是 12\" 屏，mainagent(A) 改了它，输出 \"ok\"。", 1),
    ("必拒：落单的反引号夹住", "用 ` 转义，mainagent(A) 改了 x.py，另见 `y`。", 1),
    ("必拒：撇号不当引号", "it's fine, mainagent(A) 改了 x.py, don't worry。", 1),
    ("必拒：引用块之后的正文", "> 规则原文：写 mainagent(A) 这类字母。\n\nmainagent(A) 改了 x.py。", 1),
    ("必拒：围栏之后的正文", "```\n输出\n```\nmainagent(A) 改了 x.py。", 1),
    ("必拒：缩进不足四列的行", "输出：\n\n  mainagent(A) 改了 x\n\n共 1 行。", 1),
    ("必拒：列表项下缩进的子项", "- 已交回\n\n    - mainagent(C) 改了 c.py", 1),
    ("必拒：表格里的规则原文不放进引号照拦", "| 写给谁 | 指别的会话怎么写 |\n|---|---|\n| 会话之间的消息 | mainagent(A) |\n| 给用户的回复 | ListAgents 那一行的编号 |", 1),
    ("必拒：路径不放进引号照拦", "那份草稿在 ~/.local/share/singlefs/work/notes/mainagent(A)-message.txt，是发给别的会话的原文。", 1),
    ("必放：反引号里", "规则里的写法是 `mainagent(A)`。", 0),
    ("必放：中文引号里", "不写「mainagent(A)」这类字母。", 0),
    ("必放：代码块里", "输出：\n```\nmainagent(A) 改了 x\n```\n共 1 行。", 0),
    ("必放：只写 ListAgents 的编号", "(5b) 改了 scripts/x.py，做了一次换上。", 0),
    ("必放：弯引号里", "规则里写的是“mainagent(A)”。", 0),
    ("必放：subagent 字母不判", "subagent(A) 报告了红。", 0),
    ("必放：双反引号里", "规则里的写法是 ``mainagent(A)``，括号里只放大写字母。", 0),
    ("必放：跨行的反引号里", "规则里的写法是 `mainagent(A)\n` 这种。", 0),
    ("必放：波浪线代码块", "~~~\nmainagent(A) 改了 x\n~~~", 0),
    ("必放：四反引号里嵌三反引号", "日志原文：\n````\n```\nx\n```\nmainagent(A) 改了 x\n````\n共 1 段。", 0),
    ("必放：四空格缩进的代码块", "日志原文：\n\n    mainagent(A) 改了 x\n\n共 1 行。", 0),
    ("必放：引用块里的规则原文", "规则原文：\n\n> 会话之间的消息指别的会话写 mainagent(A)、mainagent(B)，给用户的回复写编号。\n\n所以给你的汇报里只写编号。", 0),
    ("必放：列表项里的引用块", "- > mainagent(C) 改了 c.py", 0),
    ("必放：『』里", "规则里的写法是『mainagent(A)』，括号里只放大写字母。", 0),
    ("必放：‘’里", "规则里的写法是‘mainagent(A)’，括号里只放大写字母。", 0),
    ("必放：命令里的单引号里", "跑了 grep -nF 'mainagent(A)' .claude/agent-common.md，命中 1 行，在「写」一节。", 0),
    ("必放：整句放进中文引号", "「mainagent(A) 改了 scripts/x.py，做了一次换上。」", 0),
    ("必放：整句放进双引号", "\"mainagent(A) 改了 scripts/x.py，做了一次换上。\"", 0),
    ("必放：整句放进反引号", "`mainagent(A) 改了 scripts/x.py，做了一次换上。`", 0),
    ("必放：表格里的规则原文放进中文引号", "| 写给谁 | 指别的会话怎么写 |\n|---|---|\n| 会话之间的消息 | 「mainagent(A)」 |\n| 给用户的回复 | ListAgents 那一行的编号 |", 0),
    ("必放：路径放进中文引号", "那份草稿在「~/.local/share/singlefs/work/notes/mainagent(A)-message.txt」，是发给别的会话的原文。", 0),
    ("必放：占位写法", "把 mainagent(<字母>) 换成编号。", 0),
    ("必放：只有名字没有括号", "mainagent 的规则写在 agent-common.md。", 0),
    ("必放：泛指主 agent", "交给主 agent 去办，子 agent 的报告由它读。", 0),
)
LETTER_REPLY = "mainagent(A) 改了 x.py。"
CHANGED_LINE_REPLY = "好的。mainagent(A) 还改了 y.py，没写编号。"
CLEAN_REPLY = "(5b) 改了 x.py。"

def run_hook(script, hook_input, environment, raw=None):
    return subprocess.run(["bash", script], input=raw if raw is not None else json.dumps(hook_input, ensure_ascii=False),
                          capture_output=True, text=True, env=environment)

def selftest(hook_dir):
    results = []
    for label, reply, want_count in SAMPLE_REPLIES:
        results.append((label, want_count, len(letter_hits(reply))))
    script = os.path.join(hook_dir, HOOK_NAME)
    scratch = tempfile.mkdtemp(prefix="session-number-stop-check-selftest-")
    mark = os.path.join(scratch, "selftest-mark")   # 自证起的子进程带这份标记，照样认调用方设的弄坏开关
    with open(mark, "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_MARK_CONTENT + "\n")
    detections = os.path.join(scratch, "detections.jsonl")   # 检出记在自证自己的临时目录，不碰真的检出记录
    transcript = os.path.join(scratch, "session.jsonl")
    with open(transcript, "w", encoding="utf-8") as handle:
        handle.write(json.dumps({"type": "assistant", "timestamp": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                                 "message": {"role": "assistant", "content": [{"type": "text", "text": LETTER_REPLY}]}}, ensure_ascii=False) + "\n")
    turn_transcripts = {}   # 两个回合各一份只有一条用户输入的会话记录，promptId 是回合的标识
    for turn in ("blocked-turn", "later-turn"):
        turn_transcripts[turn] = os.path.join(scratch, f"{turn}.jsonl")
        with open(turn_transcripts[turn], "w", encoding="utf-8") as handle:
            handle.write(json.dumps({"type": "user", "promptId": turn, "message": {"role": "user", "content": "继续"}}, ensure_ascii=False) + "\n")
    environment = dict(os.environ, **{SELFTEST_MARK_VARIABLE: mark, "AGENT_HOOK_DETECTIONS": detections,
                                      STATE_DIRECTORY_VARIABLE: os.path.join(scratch, "state")})

    def recorded():
        try:
            with open(detections, encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except OSError:
            return []

    for label, hook_input, want_code, want_in_stderr in (
            ("stdin：Stop 里写了字母拦下并给出路", {"hook_event_name": "Stop", "session_id": "s-main", "last_assistant_message": LETTER_REPLY}, 2, ("ListAgents", "(5b)", "writing-discipline-by-review.md")),
            ("stdin：只写编号的放行", {"hook_event_name": "Stop", "session_id": "s-clean", "last_assistant_message": CLEAN_REPLY}, 0, ()),
            ("stdin：SubagentStop 里写字母放行", {"hook_event_name": "SubagentStop", "session_id": "s-sub", "agent_id": "a1", "agent_type": "tooling-writer", "last_assistant_message": LETTER_REPLY}, 0, ()),
            ("stdin：带 agent_id 的收工放行", {"session_id": "s-agent", "agent_id": "a2", "last_assistant_message": LETTER_REPLY}, 0, ()),
            ("stdin：首次命中拦下", {"hook_event_name": "Stop", "session_id": "s-loop", "last_assistant_message": LETTER_REPLY}, 2, ("mainagent(A)",)),
            ("stdin：续跑里同一份命中不连拦", {"hook_event_name": "Stop", "session_id": "s-loop", "stop_hook_active": True, "last_assistant_message": LETTER_REPLY}, 0, ()),
            ("stdin：续跑里换了说法照样拦", {"hook_event_name": "Stop", "session_id": "s-loop", "stop_hook_active": True, "last_assistant_message": "mainagent(B) 改了 y.py。"}, 2, ("mainagent(B)",)),
            ("stdin：没有回复时读会话记录", {"hook_event_name": "Stop", "session_id": "s-transcript", "transcript_path": transcript}, 2, ("mainagent(A)",)),
            ("stdin：换整行之前先拦一次", {"hook_event_name": "Stop", "session_id": "s-line", "last_assistant_message": LETTER_REPLY}, 2, ("mainagent(A)",)),
            ("stdin：续跑里同一个名字换了整行照样拦", {"hook_event_name": "Stop", "session_id": "s-line", "stop_hook_active": True, "last_assistant_message": CHANGED_LINE_REPLY}, 2, ("mainagent(A)",)),
            ("stdin：干净回复之前先拦一次", {"hook_event_name": "Stop", "session_id": "s-forget", "last_assistant_message": LETTER_REPLY}, 2, ("mainagent(A)",)),
            ("stdin：续跑里改成编号放行", {"hook_event_name": "Stop", "session_id": "s-forget", "stop_hook_active": True, "last_assistant_message": CLEAN_REPLY}, 0, ()),
            ("stdin：干净回复之后的续跑里头一次写同样的叫法照样拦", {"hook_event_name": "Stop", "session_id": "s-forget", "stop_hook_active": True, "last_assistant_message": LETTER_REPLY}, 2, ("mainagent(A)",)),
            ("stdin：非续跑里头一次命中拦下", {"hook_event_name": "Stop", "session_id": "s-again", "last_assistant_message": LETTER_REPLY}, 2, ("mainagent(A)",)),
            ("stdin：非续跑里同一份命中再来一次照样拦", {"hook_event_name": "Stop", "session_id": "s-again", "last_assistant_message": LETTER_REPLY}, 2, ("mainagent(A)",)),
            ("stdin：回合头一次命中拦下", {"hook_event_name": "Stop", "session_id": "s-turn", "transcript_path": turn_transcripts["blocked-turn"], "last_assistant_message": LETTER_REPLY}, 2, ("mainagent(A)",)),
            ("stdin：同一回合的续跑里同一行不连拦", {"hook_event_name": "Stop", "session_id": "s-turn", "stop_hook_active": True, "transcript_path": turn_transcripts["blocked-turn"], "last_assistant_message": LETTER_REPLY}, 0, ()),
            ("stdin：换了回合的续跑里头一次写同样一行照样拦", {"hook_event_name": "Stop", "session_id": "s-turn", "stop_hook_active": True, "transcript_path": turn_transcripts["later-turn"], "last_assistant_message": LETTER_REPLY}, 2, ("mainagent(A)",))):
        completed = run_hook(script, hook_input, environment)
        stderr_ok = all(text in completed.stderr for text in want_in_stderr) if want_in_stderr else completed.stderr == ""
        results.append((label, (want_code, True), (completed.returncode, stderr_ok)))
    # 判不了的照「放行但必记检出」：退 0、stderr 写一行、检出记录多一行（结局只记）
    for label, hook_input, raw, want_session in (
            ("stdin：坏 JSON 放行并记一条检出", None, "{不是 JSON", "读不到"),
            ("stdin：JSON 数组放行并记一条检出", None, "[]", "读不到"),
            ("stdin：取不到回复放行并记一条检出", {"hook_event_name": "Stop", "session_id": "s-none"}, None, "s-none")):
        before = len(recorded())
        completed = run_hook(script, hook_input, environment, raw)
        added = recorded()[before:]
        results.append((label, (0, True, 1, "只记", want_session, True),
                        (completed.returncode, HOOK_NAME in completed.stderr, len(added), added[0].get("outcome") if added else None,
                         added[0].get("session_id") if added else None,
                         bool(added) and all(finding.startswith(UNJUDGED_PREFIX) for finding in added[0].get("findings") or []))))
    saved_detections = os.environ.get("AGENT_HOOK_DETECTIONS")
    os.environ["AGENT_HOOK_DETECTIONS"] = detections
    # 判的时候抛异常：在进程里把判法换成一个抛异常的，照钩子入口调 judge_input
    original_letter_hits = globals()["letter_hits"]
    def raising_letter_hits(reply):
        raise RuntimeError("自证造的异常")
    globals()["letter_hits"] = raising_letter_hits
    before = len(recorded())
    try:
        got_code = judge_input(json.dumps({"hook_event_name": "Stop", "session_id": "s-raise", "last_assistant_message": LETTER_REPLY}).encode("utf-8"), hook_dir)
    except Exception as error:   # 弄坏开关 exception-unrecorded：照原样抛出
        got_code = f"抛出 {type(error).__name__}"
    finally:
        globals()["letter_hits"] = original_letter_hits
    added = recorded()[before:]
    results.append(("判的时候抛异常放行并记一条检出", (0, 1, "只记", "s-raise"),
                    (got_code, len(added), added[0].get("outcome") if added else None, added[0].get("session_id") if added else None)))
    if saved_detections is None:
        os.environ.pop("AGENT_HOOK_DETECTIONS", None)
    else:
        os.environ["AGENT_HOOK_DETECTIONS"] = saved_detections
    # 平常跑（不带标记、不是 --selftest）设了弄坏开关：退 2、点名它只在自证里认
    outside = run_hook(script, {"hook_event_name": "Stop", "session_id": "s-outside", "last_assistant_message": CLEAN_REPLY},
                       dict({key: value for key, value in os.environ.items() if key != SELFTEST_MARK_VARIABLE},
                            **{BREAK_VARIABLE: "input-unrecorded", "AGENT_HOOK_DETECTIONS": detections}))
    results.append(("平常跑设了弄坏开关退 2、点名它只在自证里认", (2, True),
                    (outside.returncode, any(BREAK_VARIABLE in line and "自证" in line for line in outside.stderr.splitlines()))))
    for name in sorted(os.listdir(scratch), reverse=True):
        path = os.path.join(scratch, name)
        if os.path.isdir(path):
            for inner in os.listdir(path):
                os.remove(os.path.join(path, inner))
            os.rmdir(path)
        else:
            os.remove(path)
    os.rmdir(scratch)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 letter_hits() / letter_name_pattern() / judged_reply_text() 与它调的 blank_*()、normalized() / from_subagent() / already_shown() / forget_shown() / current_turn() / decide() 的判法与 stdin 入口，改完再跑 --selftest")
        return 1
    print(f"  ✓ 自检通过（查了 {len(results)} 种）：mainagent 与「主 agent」后接括号里一到两个字母（大小写、全角、空白换行、加粗转义、零宽字符、省括号各种写法）的拦，"
          "行内代码、围栏与缩进代码块、引用块、成对引号里的与只写 ListAgents 编号的放行，落单的引号与反引号不算引用；"
          "子 agent 收工放行、续跑里同一整行同一回合的命中不连拦、换了整行或回合照拦、非续跑照拦、没有命中的回复清指纹、没有回复时读会话记录；"
          "输入不是 JSON 对象、取不到回复、判的时候抛异常照常放行并记一条检出；平常跑设了弄坏开关退 2")
    return 0

def unjudged_pass(hook_input, problem, switch, hook_dir):
    """判不了：照「放行但必记检出」stderr 写一行、检出记录追加一行（结局只记），交退出码 0。弄坏开关 switch 下只打 stderr、不记。"""
    print(f"! {HOOK_NAME}：{problem}，这一次没判、照常放行", file=sys.stderr)
    if broken(switch):
        return 0
    try:
        sys.dont_write_bytecode = True
        if hook_dir not in sys.path:
            sys.path.insert(0, hook_dir)
        import lib_work_root
        _, said = lib_work_root.record_unjudged_pass(HOOK_NAME, hook_input, "收工回复的会话编号检查", [f"{UNJUDGED_PREFIX}：{problem}"])
    except Exception as error:   # 文件不在、语法错都算读不到
        said = f"检出没记成：读不到同目录的 lib_work_root.py（{error!r}）"
    print(f"  {said}", file=sys.stderr)
    return 0

def judge_input(raw, hook_dir):
    """hook 的原始输入（字节）→ 退出码。判不了的几种照「放行但必记检出」（文件头「判不了的」一段）。"""
    try:
        hook_input = json.loads(raw.decode("utf-8"))
    except ValueError as error:
        return unjudged_pass(None, f"输入不是合法的 JSON（{error}；读到 {len(raw)} 字节，前 {INPUT_PREFIX_BYTES} 字节：{raw[:INPUT_PREFIX_BYTES].decode('utf-8', 'replace')!r}）",
                             "input-unrecorded", hook_dir)
    if not isinstance(hook_input, dict):
        return unjudged_pass(None, f"输入顶层是 {type(hook_input).__name__}，不是 JSON 对象", "input-unrecorded", hook_dir)
    try:
        code, message = decide(hook_input, hook_dir)
    except Exception as error:
        if broken("exception-unrecorded"):   # 弄坏开关：不接住，照原样抛出
            raise
        return unjudged_pass(hook_input, f"判的时候抛异常（{type(error).__name__}: {error}）", "exception-unrecorded", hook_dir)
    if code is None:
        return unjudged_pass(hook_input, message, "reply-unrecorded", hook_dir)
    if code:
        print(message, file=sys.stderr)
    return code

def refuse_break_outside_selftest():
    """弄坏开关只在自证里认：平常跑（不是 --selftest，也不是自证起的子进程）设了 SESSION_NUMBER_STOP_CHECK_BREAK 交 2，点名它只在自证里认；别的交 0。"""
    if not os.environ.get(BREAK_VARIABLE) or sys.argv[2:] == ["--selftest"]:
        return 0
    mark = os.environ.get(SELFTEST_MARK_VARIABLE, "")
    try:
        if mark and open(mark, encoding="utf-8").read().strip() == SELFTEST_MARK_CONTENT:
            return 0
    except OSError:
        pass
    print(f"✗ {HOOK_NAME}：{BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 是弄坏开关，只在自证（{HOOK_NAME} --selftest 与它起的子进程）里认；平常跑设了就拒，这一次没判", file=sys.stderr)
    print(f"→ 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 {BREAK_VARIABLE}=<项> bash .claude/hooks/{HOOK_NAME} --selftest（开关写在文件头）", file=sys.stderr)
    return 2

def main():
    hook_dir = sys.argv[1]
    refused = refuse_break_outside_selftest()
    if refused:
        return refused
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    return judge_input(sys.stdin.buffer.read(), hook_dir)

sys.exit(main())
PY
