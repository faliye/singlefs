#!/usr/bin/env python3
# admission: always 判的是此刻的欠账表与里程碑文件，几十毫秒读完；两份有一份改了都要现判，上一次的结论不替这一次作保
# run-condition: none 只读仓里的几份文本，除了 python3 之外没有环境要求，不起子进程
"""里程碑收口时欠账表里还有没有点名它的开着的账（出口里还开着欠账的里程碑不许收口），
与欠账表去向的写法（去向挪到别的里程碑时，写法要可以检查得出）。

用法：
    milestone-close-owed-check.py <里程碑号> [仓根]          判这一个里程碑，不看它收没收口（收口之前先跑它）；号写 3 或 三
    milestone-close-owed-check.py --closed [仓根]            只判段表已写收口的里程碑（门禁 doc-process-records 的 milestone-close-owed 格调这一种）
    milestone-close-owed-check.py --destination-form [仓根]  判欠账表开着的每一行的去向写法（门禁 doc-process-records 的 owed-destination-form 格调这一种）
    milestone-close-owed-check.py --selftest                 MILESTONE_CLOSE_OWED_BREAK=<项> 时点名的格必须红
仓根不给就是这份脚本所在的仓。

判法（照文件现状读，不看历史）：
  里程碑文件：.claude/kb/milestone/<两位号>-<名>-txn.md（译文 .en.md、.ja.md 不算）。
  收没收口：文件里第一张段表（表头第一格是「段」、有一列叫「状态」）里，有一行的状态格（去掉 * 与空白）以「结束」或「已完结」起头，就算收口。
    没有段表的里程碑文件（例如只在文件头写「状态：判定结束」的）不算收口，--closed 不判它。
  欠着的账：.claude/kb/checks-owed.md 里「已还清」那一行整行标题之前、首列是 C<编号> 的表格行（切法与门禁共用 .claude/gate.d/lib-owed.py 的 PAID_HEADING、OWED_ROW）；
    认不出「已还清」标题判红，不对着一张认不出的表判。
  去向段：一行按没转义的「|」切成六格，取第五格「前置」里第一个「去向：」之后的那一段（同一格里写在「去向：」之前的说明与出处不算）；
    切不成六格的取整行里第一个「去向：」之后那一段；没有「去向：」的行没有去向段，不点名任何里程碑。
  落点：去向段开头（跳过空白）只认三种形态；几个落点用「、」连着写在开头算几个，第一个不是落点的字（括注、分号、句号、逗号接说明）之后都是说明：
    ① 里程碑文件：`.claude/kb/milestone/<两位号>-<小写字母>-txn.md`（目录可省，反引号可有可无，可带 .en / .ja）；
    ② 「里程碑」后接号（阿拉伯数字或中文数字，中间可以有空白）：后面紧跟「·」的，最后一个中文数字是项号、不算进里程碑号
       （「里程碑三二·5」是里程碑三的二·5、「里程碑十四一·1」是里程碑十四的一·1）；
    ③ 「未排」起头：不点名任何里程碑，后面都是说明。
    ①② 后面可接作业点：空白、「的」、「<节名>」、<中文数字>·<数字>、「出口 <数字>」任意连写（「「八　接手里程碑四挪来的…」八·4」整段是作业点，节名里的号不算点名）；
    作业点之后紧跟「、」再接一个落点的，算下一个落点。
  点名：去向段开头 ①② 落点的里程碑号；说明里的「里程碑<号>」、文件名、括注里的「从里程碑三挪来」都不算。
    点名这个里程碑的行数为 0 才绿；不为 0 逐行列出编号、简称与点名处前后的原文。
  写法（--destination-form）：开着的每一行，有去向段的，开头不是三种落点之一判红；①② 点名的里程碑在 .claude/kb/milestone/ 下找不到也判红
    （② 按号找 <两位号>-*-txn.md，① 按写的文件名找那一份）；红行列编号、简称与去向段开头 30 字。
    成功句报判了几行、三种形态各几行（按开头第一个落点算）、没写「去向：」的几行（只报数，不判红）。

退出码：0 没有点名它的开着的账、去向写法都对；1 有（逐行列出），或欠账表、里程碑文件读不了；2 用法错、找不到这个号的里程碑文件；
  77 --closed 下没有一个里程碑的段表写了收口，或 --destination-form 下树里没有欠账表、开着的行一行写了「去向：」的都没有（无对象可判）。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（格名是 selftest() 里那一格，自证打「✗ 自证 <格名>：…」）：
  MILESTONE_CLOSE_OWED_BREAK=item-suffix-kept        「里程碑三二·5」不拆项号，号认不出就不是落点
      「✗ 自证 both-forms-red：」、「✗ 自证 closed-only-red：」、「✗ C2（文件名）：」、「✗ C3（阿拉伯号）：」
      「✗ 自证 explanation-not-named：」、「✗ 自证 section-name-not-named：」
      「✗ 自证 destination-form-green：」、「✗ C12（号带项号再括注）：去向段开头不是三种落点之一」、「✗ C14（两个落点）：去向段开头不是三种落点之一」
      「✗ 自证 destination-form-red：」、「✗ C21（先写挪出）：」、「✗ C22（括注起头）：」、「✗ C23（号带项号）：」
      「✗ 自证 destination-missing-file-red：」、「✗ C31（没有那份的号）：去向段开头不是三种落点之一」、「✗ C34（都在）：」
      「✗ C32（文件名写错）：落点」、「✗ C33（后一个落点没有那份）：落点」
  MILESTONE_CLOSE_OWED_BREAK=file-form-ignored       不认里程碑文件名那种落点（与整段读法里的文件名）
      「✗ 自证 both-forms-red：」、「✗ C1（中文号带项号）：」、「✗ C3（阿拉伯号）：」
      「✗ 自证 section-name-not-named：」、「✗ 自证 file-landing-named：」
      「✗ 自证 destination-form-green：」、「✗ C11（括注里挪来）：去向段开头不是三种落点之一」、「✗ C13（节名里的号）：去向段开头不是三种落点之一」
      「✗ 自证 destination-form-red：」、「✗ C21（先写挪出）：」、「✗ C22（括注起头）：」、「✗ C26（文件带目录）：去向段开头不是三种落点之一」
      「✗ 自证 destination-missing-file-red：」、「✗ C32（文件名写错）：去向段开头不是三种落点之一」
      「✗ C31（没有那份的号）：落点「里程碑七」」、「✗ C33（后一个落点没有那份）：落点「里程碑十七」」
  MILESTONE_CLOSE_OWED_BREAK=whole-row-scanned       整行都看、任何一处都算点名，不只看去向段开头的落点
      「✗ 自证 both-forms-red：」、「✗ C1（中文号带项号）：」、「✗ C2（文件名）：」、「✗ C3（阿拉伯号）：」、「✗ C6（只在出处）：」
      「✗ 自证 rerouted-green：」、「✗ 自证 elsewhere-named-green：」、「✗ C8（别处点名）：」、「✗ C9（没写去向）：」、「✗ 自证 closed-green：」
      「✗ 自证 explanation-not-named：」、「✗ C11（括注里挪来）：」、「✗ C12（号带项号再括注）：」、「✗ C14（两个落点）：」、「✗ C15（未排）：」、「✗ C16（没写去向的说明）：」
      「✗ 自证 section-name-not-named：」、「✗ C13（节名里的号）：」
  MILESTONE_CLOSE_OWED_BREAK=paid-rows-counted       「已还清」之后的行也算欠着的
      「✗ 自证 both-forms-red：」、「✗ C1（中文号带项号）：」、「✗ C2（文件名）：」、「✗ C3（阿拉伯号）：」、「✗ C90（还清的）：」
      「✗ 自证 rerouted-green：」、「✗ 自证 elsewhere-named-green：」、「✗ 自证 closed-green：」
      「✗ 自证 explanation-not-named：」、「✗ C14（两个落点）：」
      「✗ 自证 destination-form-green：」、「✗ 自证 destination-form-red：」、「✗ C21（先写挪出）：」、「✗ C22（括注起头）：」、「✗ 自证 destination-no-object：」
  MILESTONE_CLOSE_OWED_BREAK=status-ignored          --closed 不看段表，每个里程碑都判
      「✗ 自证 closed-only-red：」、「✗ C1（中文号带项号）：」、「✗ C2（文件名）：」、「✗ C3（阿拉伯号）：」
      「✗ 里程碑四（.claude/kb/milestone/04-fourth-txn.md）（段表还没写收口）：」、「✗ C5（带日期的文件名）：」、「✗ C7（去向开着的里程碑）：」
      「✗ 里程碑十三（.claude/kb/milestone/13-thirteenth-txn.md）（段表还没写收口）：」、「✗ C4（十三）：」
      「✗ 自证 closed-green：」、「✗ 自证 none-closed：」
  MILESTONE_CLOSE_OWED_BREAK=paid-heading-optional   认不出「已还清」标题时不判红，把全文的行当欠着的判
      「✗ 自证 no-paid-heading-red：」、「✗ C90（还清的）：」
  MILESTONE_CLOSE_OWED_BREAK=prerequisite-whole-cell  前置那一格整格都看、任何一处都算点名（收窄到「去向：」之后之前的读法）
      「✗ 自证 elsewhere-named-green：」、「✗ C8（别处点名）：」、「✗ C9（没写去向）：」
      「✗ 自证 explanation-not-named：」、「✗ C11（括注里挪来）：」、「✗ C12（号带项号再括注）：」、「✗ C14（两个落点）：」、「✗ C15（未排）：」、「✗ C16（没写去向的说明）：」
      「✗ 自证 section-name-not-named：」、「✗ C13（节名里的号）：」
  MILESTONE_CLOSE_OWED_BREAK=whole-segment-scanned   去向段整段都看、任何一处都算点名（只认落点之前的读法：括注、说明、节名里的号都算）
      「✗ 自证 explanation-not-named：」、「✗ C11（括注里挪来）：」、「✗ C12（号带项号再括注）：」、「✗ C14（两个落点）：」、「✗ C15（未排）：」
      「✗ 自证 section-name-not-named：」、「✗ C13（节名里的号）：」
  MILESTONE_CLOSE_OWED_BREAK=form-unchecked          --destination-form 不判去向段开头是不是落点
      「✗ 自证 destination-form-red：」
  MILESTONE_CLOSE_OWED_BREAK=landing-file-unchecked  --destination-form 不核落点点名的里程碑文件在不在
      「✗ 自证 destination-missing-file-red：」
"""
import collections
import importlib.util
import os
import re
import shutil
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BROKEN = set(filter(None, os.environ.get("MILESTONE_CLOSE_OWED_BREAK", "").split(",")))
OWED_TABLE = os.path.join(".claude", "kb", "checks-owed.md")
MILESTONE_DIRECTORY = os.path.join(".claude", "kb", "milestone")
MILESTONE_FILE = re.compile(r"^([0-9]{2})-[a-z]+-txn\.md$")
PREREQUISITE_COLUMN = 4          # 「| # | 简称 | 要拦什么 | 怎么拦 | 前置 | 出处 |」的第五格
DESTINATION_MARK = "去向："       # 前置格里只看它之后的那一段（文件头「判法」的「去向段」）
OWED_COLUMNS = 6
CLOSED_STATUS = ("结束", "已完结")
CHINESE_DIGITS = {"一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}
BLANKS = " \t　"
# 去向段开头的落点（文件头「判法」的「落点」三种形态）
LANDING_FILE = re.compile(r"(`?)((?:\.claude/kb/milestone/)?(([0-9]{2})-[a-z]+-txn(?:\.en|\.ja)?\.md))\1")
LANDING_MILESTONE = re.compile(r"里程碑[ \t　]*([0-9]+|[一二三四五六七八九十]+)")
LANDING_UNSCHEDULED = "未排"
LANDING_SEPARATOR = "、"
WORK_POINT = re.compile(r"(?:[ \t　]*的?[ \t　]*(?:「[^」\n]*」|[一二三四五六七八九十]+·[0-9]+|出口[ \t　]*[0-9]+))*")
DESTINATION_FORMS = (("file", "里程碑文件"), ("milestone", "「里程碑<号>」"), ("unscheduled", "「未排」"))
DESTINATION_HEAD_LENGTH = 30
# 弄坏开关的「整段都算」读法：段里任何一处「里程碑<号>」与里程碑文件名都算点名
MILESTONE_WORD = re.compile(r"(?<!-)里程碑\s*([0-9]+|[一二三四五六七八九十]+)(·?)")
MILESTONE_FILENAME = re.compile(r"(?<![0-9])([0-9]{2})-[a-z]+-txn(?:\.en|\.ja)?\.md")
UNESCAPED_PIPE = re.compile(r"(?<!\\)\|")
SNIPPET_RADIUS = 24


def load_owed_library():
    """欠账表开着与已还清的切法：门禁共用的 .claude/gate.d/lib-owed.py（这份脚本所在的仓里那一份），不另抄一份。"""
    path = os.path.join(REPOSITORY_ROOT, ".claude", "gate.d", "lib-owed.py")
    specification = importlib.util.spec_from_file_location("lib_owed_for_milestone_close", path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


OWED_LIBRARY = load_owed_library()


def chinese_number(text):
    """一到九十九的中文数字 → 整数；认不出交 None。"""
    if text.isdigit():
        return int(text)
    if len(text) == 1:
        return 10 if text == "十" else CHINESE_DIGITS.get(text)
    if "十" not in text or text.count("十") > 1:
        return None
    tens, _, ones = text.partition("十")
    if tens and tens not in CHINESE_DIGITS or ones and ones not in CHINESE_DIGITS or len(tens) > 1 or len(ones) > 1:
        return None
    return (CHINESE_DIGITS[tens] if tens else 1) * 10 + (CHINESE_DIGITS[ones] if ones else 0)


def milestone_number_argument(text):
    number = chinese_number(text.strip())
    return number if number and number > 0 else None


def skip_blanks(text, position):
    while position < len(text) and text[position] in BLANKS:
        position += 1
    return position


def read_landing(segment, position):
    """segment[position:] 开头的一个 ①② 落点 → (形态, 里程碑号, 落点结束处, 文件名或 None)；不是落点交 None。"""
    if "file-form-ignored" not in BROKEN:
        found = LANDING_FILE.match(segment, position)
        if found:
            return "file", int(found.group(4)), found.end(), found.group(3)
    found = LANDING_MILESTONE.match(segment, position)
    if not found:
        return None
    run, end = found.group(1), found.end()
    if run.isdigit():
        number = int(run)
    elif "item-suffix-kept" in BROKEN:
        number = chinese_number(run)
    elif segment[end:end + 1] == "·" and len(run) > 1:   # 最后一个中文数字是项号：「里程碑三二·5」→ 三，作业点从「二·5」起
        number, end = chinese_number(run[:-1]), end - 1
    else:
        number = chinese_number(run)
        if number is None:   # 号后面紧跟别的中文数字而没有「·」的，取认得出的最长前缀
            length = next((length for length in (3, 2, 1) if length < len(run) and chinese_number(run[:length])), None)
            if length:
                number, end = chinese_number(run[:length]), found.start(1) + length
    if not number:
        return None
    return "milestone", number, end, None


def destination_landings(segment):
    """去向段 → (形态, [(里程碑号, 落点在段里的下标, 落点原文, 文件名或 None)])（文件头「判法」的「落点」）。
    形态是 file / milestone / unscheduled（按开头第一个落点），开头不是三种落点之一交 None。"""
    position = skip_blanks(segment, 0)
    if segment.startswith(LANDING_UNSCHEDULED, position):
        return "unscheduled", []
    form, landings = None, []
    while True:
        landing = read_landing(segment, position)
        if landing is None:
            break
        kind, number, end, filename = landing
        form = form or kind
        landings.append((number, position, segment[position:end], filename))
        end = WORK_POINT.match(segment, end).end()
        if not segment.startswith(LANDING_SEPARATOR, end):
            break
        position = skip_blanks(segment, end + len(LANDING_SEPARATOR))
    return form, landings


def mentioned_milestones(text):
    """弄坏开关退回的读法：→ {里程碑号: 第一次提到处的下标}，text 里任何一处「里程碑<号>」与里程碑文件名都算。"""
    found = {}
    for match in MILESTONE_WORD.finditer(text):
        run = match.group(1)
        if "item-suffix-kept" in BROKEN:
            number = chinese_number(run)
            if number:
                found.setdefault(number, match.start())
            continue
        if not run.isdigit() and match.group(2) == "·" and len(run) > 1:
            run = run[:-1]
        number = chinese_number(run)
        if number is None and not run.isdigit():
            number = next((chinese_number(run[:length]) for length in (3, 2, 1) if length < len(run) and chinese_number(run[:length])), None)
        if number:
            found.setdefault(number, match.start())
    if "file-form-ignored" not in BROKEN:
        for match in MILESTONE_FILENAME.finditer(text):
            found.setdefault(int(match.group(1)), match.start())
    return found


def named_milestones(row):
    """一行开着的账 → ({里程碑号: 点名处在原文里的下标}, 原文)（文件头「判法」的「点名」）。"""
    _, _, segment, line, cell = row
    if "whole-row-scanned" in BROKEN:
        return mentioned_milestones(line), line
    if cell is not None and "prerequisite-whole-cell" in BROKEN:
        return mentioned_milestones(cell), cell
    if segment is None:
        return {}, ""
    if "whole-segment-scanned" in BROKEN:
        return mentioned_milestones(segment), segment
    found = {}
    for number, position, _, _ in destination_landings(segment)[1]:
        found.setdefault(number, position)
    return found, segment


def owed_rows(root):
    """→ ([(编号, 简称, 去向段或 None, 整行, 前置格或 None)], 问题)。只取开着的那张表（切法见 load_owed_library）。"""
    path = os.path.join(root, OWED_TABLE)
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except OSError as error:
        return [], [f"读不了欠账表 {OWED_TABLE}：{error}"]
    if not any(OWED_LIBRARY.PAID_HEADING.match(line) for line in lines) and "paid-heading-optional" not in BROKEN:
        return [], [f"{OWED_TABLE} 里认不出「已还清」那一行整行标题，分不出哪些账开着"]
    rows = []
    for line in lines:
        if OWED_LIBRARY.PAID_HEADING.match(line) and "paid-rows-counted" not in BROKEN:
            break
        if OWED_LIBRARY.HISTORY_HEADING.match(line):
            break
        row = OWED_LIBRARY.OWED_ROW.match(line)
        if not row:
            continue
        cells = UNESCAPED_PIPE.split(line.strip())[1:-1]
        cell = cells[PREREQUISITE_COLUMN] if len(cells) == OWED_COLUMNS else None
        holder = line if cell is None else cell
        segment = holder.partition(DESTINATION_MARK)[2] if DESTINATION_MARK in holder else None
        rows.append((row.group(1), row.group(2), segment, line, cell))
    return rows, []


def milestone_files(root):
    """→ {里程碑号: 相对仓根的路径}（只取中文那一份）。"""
    directory = os.path.join(root, MILESTONE_DIRECTORY)
    found = {}
    for name in sorted(os.listdir(directory)) if os.path.isdir(directory) else []:
        match = MILESTONE_FILE.match(name)
        if match:
            found.setdefault(int(match.group(1)), os.path.join(MILESTONE_DIRECTORY, name))
    return found


def closing_row(root, relative):
    """段表里写了收口的那一行：→ (行号, 状态格原文)；没有段表或没写收口交 None。"""
    lines = open(os.path.join(root, relative), encoding="utf-8").read().splitlines()
    status_column = None
    for number, line in enumerate(lines, 1):
        if not line.startswith("|"):
            if status_column is not None:
                return None
            continue
        cells = [cell.strip() for cell in UNESCAPED_PIPE.split(line.strip())[1:-1]]
        if status_column is None:
            if cells and cells[0] == "段" and "状态" in cells:
                status_column = cells.index("状态")
            continue
        if set(line.replace("|", "").strip()) <= set("-: "):
            continue
        if status_column < len(cells) and cells[status_column].replace("*", "").strip().startswith(CLOSED_STATUS):
            return number, cells[status_column]
    return None


def chinese_name(number):
    digits = "零一二三四五六七八九"
    if number < 10:
        return digits[number]
    return ("" if number < 20 else digits[number // 10]) + "十" + (digits[number % 10] if number % 10 else "")


def unreadable_owed_table(problems, label, say):
    for problem in problems:
        say(f"  ✗ {problem}")  # gate-lint:detail
    say(f"  ✗ {label}：欠账表读不了")  # gate-lint:summary
    say("  → 怎么办：欠账表按「### 已还清」整行标题切成开着与还清两段（.claude/gate.d/lib-owed.py）；把那一行标题找回来，或把仓根参数指到仓根再跑")
    return 1


def judge(root, number, relative, closing, say):
    """判一个里程碑：→ 0 绿、1 红。"""
    label = f"里程碑{chinese_name(number)}（{relative}）"
    rows, problems = owed_rows(root)
    if problems:
        return unreadable_owed_table(problems, f"{label}：判不了还有没有点名它的开着的账", say)
    named = []
    for row in rows:
        found, text = named_milestones(row)
        position = found.get(number)
        if position is not None:
            snippet = text[max(0, position - SNIPPET_RADIUS):position + SNIPPET_RADIUS * 2].replace("\n", " ").strip()
            named.append((row[0], row[1], snippet))
    state = f"段表第 {closing[0]} 行状态写「{closing[1][:40]}」" if closing else "段表还没写收口"
    if named:
        say(f"  ✗ {label}（{state}）：欠账表开着的 {len(rows)} 行里有 {len(named)} 行的前置格「去向：」开头的落点点名它：")  # gate-lint:summary
        for name, short_name, snippet in named:
            say(f"  ✗ {name}（{short_name}）：…{snippet}…")  # gate-lint:detail
        say(f"  → 怎么办：收口之前把这 {len(named)} 行逐行了结：做完的照 .claude/kb/checks-owed.md 表尾的写法挪进「已还清」，没做完的把去向改到还开着的里程碑"
            f"（「去向：」之后开头的落点不再写「里程碑{chinese_name(number)}」或 {os.path.basename(relative)}，挪出来的经过写在落点之后的括注里），改不了去向的就别收口，把段表那一格的「结束 / 已完结」撤回；"
            f"改完跑 python3 scripts/kb/milestone-close-owed-check.py {number} 到绿")
        return 1
    say(f"  ✓ {label}（{state}）：欠账表开着的 {len(rows)} 行里没有一行的前置格「去向：」之后点名它"
        f"（只认去向段开头的落点「里程碑<号>」与「{number:02d}-*-txn.md」，落点之后的说明不算）")
    return 0


def destination_form(root, say):
    """--destination-form：→ 0 绿、1 红、77 无对象（文件头「判法」的「写法」）。"""
    if not os.path.isfile(os.path.join(root, OWED_TABLE)):
        say(f"  ! 树里没有 {OWED_TABLE}，去向写法这一格无对象可判")
        return 77
    rows, problems = owed_rows(root)
    if problems:
        return unreadable_owed_table(problems, "判不了去向的写法", say)
    directory = os.path.join(root, MILESTONE_DIRECTORY)
    present = set(os.listdir(directory)) if os.path.isdir(directory) else set()
    files = milestone_files(root)
    counts = collections.Counter()
    without, judged, malformed, missing = 0, 0, [], []
    for name, short_name, segment, _, _ in rows:
        if segment is None:
            without += 1
            continue
        judged += 1
        form, landings = destination_landings(segment)
        head = segment.strip()[:DESTINATION_HEAD_LENGTH]
        if form is None:
            if "form-unchecked" not in BROKEN:
                malformed.append(f"{name}（{short_name}）：去向段开头不是三种落点之一：「{head}」")
            continue
        counts[form] += 1
        if "landing-file-unchecked" in BROKEN:
            continue
        for number, _, text, filename in landings:
            if filename is not None and filename not in present:
                missing.append(f"{name}（{short_name}）：落点「{text}」在 {MILESTONE_DIRECTORY}/ 下找不到 {filename}：「{head}」")
            elif filename is None and number not in files:
                missing.append(f"{name}（{short_name}）：落点「{text}」在 {MILESTONE_DIRECTORY}/ 下找不到 {number:02d}-*-txn.md：「{head}」")
    tally = "、".join(f"{label} {counts[form]} 行" for form, label in DESTINATION_FORMS)
    if malformed or missing:
        say(f"  ✗ 欠账表开着的 {len(rows)} 行里写了「去向：」的 {judged} 行：{len(malformed)} 行去向段开头不是三种落点之一，"
            f"{len(missing)} 处落点点名的里程碑文件找不到")  # gate-lint:summary
        for problem in malformed + missing:
            say(f"  ✗ {problem}")  # gate-lint:detail
        say("  → 怎么办：把去向段开头改成三种落点之一，说明挪到落点之后：① `.claude/kb/milestone/<两位号>-<名>-txn.md` 后接作业点（「四·1」「「节名」八·4」）；"
            "② 「里程碑<号>」后接作业点（「里程碑三二·5」）；③ 「未排」起头；几个落点用「、」连着写在开头，挪来挪去的经过写进落点之后的括注；"
            "落点的文件找不到的，核里程碑号与文件名；改完跑 python3 scripts/kb/milestone-close-owed-check.py --destination-form 到绿")
        return 1
    if not judged:
        say(f"  ! 欠账表开着的 {len(rows)} 行里没有一行写了「去向：」，去向写法这一格无对象可判")
        return 77
    say(f"  ✓ 欠账表开着的 {len(rows)} 行里写了「去向：」的判了 {judged} 行，去向段开头都是落点、点名的里程碑文件都在："
        f"{tally}（按开头第一个落点算）；没写「去向：」的 {without} 行（只报数，不判）")
    return 0


def run(arguments, say=print):
    """命令行入口（自证也走这一份）：→ 退出码。"""
    if not arguments or len(arguments) > 2:
        say("  ✗ 参数不对")
        say("  → 怎么办：milestone-close-owed-check.py <里程碑号> [仓根] | --closed [仓根] | --destination-form [仓根] | --selftest")
        return 2
    root = os.path.abspath(arguments[1]) if len(arguments) == 2 else REPOSITORY_ROOT
    if arguments[0] == "--destination-form":
        return destination_form(root, say)
    files = milestone_files(root)
    if arguments[0] == "--closed":
        judged = []
        for number, relative in files.items():
            closing = closing_row(root, relative)
            if closing or "status-ignored" in BROKEN:
                judged.append((number, relative, closing))
        if not judged:
            say(f"  ! {MILESTONE_DIRECTORY} 下 {len(files)} 份里程碑文件的段表都没写「结束 / 已完结」，这一格无对象可判")
            return 77
        codes = [judge(root, number, relative, closing, say) for number, relative, closing in judged]
        red = sum(1 for code in codes if code)
        if red:
            return 1
        say(f"  ✓ 段表写了收口的里程碑 {len(judged)} 个都没有点名它的开着的账（里程碑文件共 {len(files)} 份，其余 {len(files) - len(judged)} 份段表没写收口、不判）")
        return 0
    number = milestone_number_argument(arguments[0])
    if number is None:
        say(f"  ✗ 认不出里程碑号「{arguments[0]}」")
        say(f"  → 怎么办：写阿拉伯数字或中文数字，例如 3 或 三")
        return 2
    if number not in files:
        say(f"  ✗ {MILESTONE_DIRECTORY} 下没有 {number:02d}-*-txn.md")
        say(f"  → 怎么办：核里程碑号；仓根不是这份脚本所在的仓时把仓根写在号后面")
        return 2
    return judge(root, number, files[number], closing_row(root, files[number]), say)


def selftest():
    """造几份小仓（欠账表与里程碑文件），逐格判；MILESTONE_CLOSE_OWED_BREAK 设着的话点名的格本来就该红。"""
    work = tempfile.mkdtemp(prefix="milestone-close-owed-selftest-")
    failures = []
    titles = {3: "third", 4: "fourth", 5: "fifth", 8: "eighth", 9: "ninth", 13: "thirteenth"}

    def world(name, owed_rows_text, closed=(3,), paid_heading=True, milestones=(3, 4, 13)):
        root = os.path.join(work, name)
        os.makedirs(os.path.join(root, MILESTONE_DIRECTORY))
        owed = ["# 欠的检查清单", "", "| # | 简称 | 要拦什么 | 怎么拦（会红的形态） | 前置 | 出处 |", "|---|---|---|---|---|---|"]
        owed += owed_rows_text
        owed += ["", "### 已还清" if paid_heading else "### 别的", "", "| # | 简称 | 怎么还的 | 还清日期 |", "|---|---|---|---|",
                 "| C90 | 还清的 | 去向：里程碑三二·1 | 样本日期 |", ""]
        open(os.path.join(root, OWED_TABLE), "w", encoding="utf-8").write("\n".join(owed))
        for number in milestones:
            title = titles[number]
            status = "结束（样本：用户定）" if number in closed else "做了一半"
            text = (f"# 样本里程碑 {number}\n\n| 段 | 做什么 | 为什么排在这 | 状态 |\n|---|---|---|---|\n"
                    f"| 0 开工 | 清单 | 先定 | 做了 |\n| 1 收尾 | 收口 | 最后 | **{status}** |\n\n## 别的\n\n| 段 | 状态 |\n|---|---|\n| x | 结束 |\n")
            open(os.path.join(root, MILESTONE_DIRECTORY, f"{number:02d}-{title}-txn.md"), "w", encoding="utf-8").write(text)
            open(os.path.join(root, MILESTONE_DIRECTORY, f"{number:02d}-{title}-txn.en.md"), "w", encoding="utf-8").write("| 段 | 状态 |\n|---|---|\n| x | 结束 |\n")
        return root

    named_rows = [
        "| C1 | 中文号带项号 | 拦 | 红 | 去向：里程碑三二·5（样本） | 出处 |",
        "| C2 | 文件名 | 拦 | 红 | 去向：`.claude/kb/milestone/03-third-txn.md`「待优化」第 27 行 | 出处 |",
        "| C3 | 阿拉伯号 | 拦 | 红 | 去向：里程碑 3 | 出处 |",
    ]
    other_rows = [
        "| C4 | 十三 | 拦 | 红 | 去向：里程碑十三 | 出处 |",
        "| C5 | 带日期的文件名 | 拦 | 红 | 去向：里程碑四一·2（原话在 records/2026-10-18-里程碑三收尾与近期欠账.md） | 出处 |",
        "| C6 | 只在出处 | 拦 `a\\|b` | 红 | 去向：里程碑十四一·1 | 里程碑三二·4 的原话 |",
    ]
    elsewhere_rows = [
        "| C8 | 别处点名 | 拦 | 红 | 里程碑三二·4 那一段先做的半件已合入；去向：里程碑四一·3 | 出处 |",
        "| C9 | 没写去向 | 拦 | 红 | 等 `.claude/kb/milestone/03-third-txn.md` 那一段先做 | 出处 |",
    ]
    # 去向段开头的落点才算点名：说明里的号、作业点的节名都不算，「、」连写的几个落点都算
    landing_rows = [
        "| C11 | 括注里挪来 | 拦 | 红 | 去向：`09-ninth-txn.md` 四·1（从里程碑三挪来） | 出处 |",
        "| C12 | 号带项号再括注 | 拦 | 红 | 去向：里程碑八一·1（write buffer 前端，用户定从里程碑三挪来） | 出处 |",
        "| C13 | 节名里的号 | 拦 | 红 | 去向：`05-fifth-txn.md`「八　接手里程碑四挪来的删除与验证」八·4 | 出处 |",
        "| C14 | 两个落点 | 拦 | 红 | 去向：里程碑三二·5、`04-fourth-txn.md` 八·7 | 出处 |",
        "| C15 | 未排 | 拦 | 红 | 去向：未排进里程碑三，等主 agent 排 | 出处 |",
        "| C16 | 没写去向的说明 | 拦 | 红 | 等里程碑三那一段先做 | 出处 |",
    ]
    form_red_rows = [
        "| C21 | 先写挪出 | 拦 | 红 | 去向：挪出里程碑三，归 `09-ninth-txn.md` 四·1 | 出处 |",
        "| C22 | 括注起头 | 拦 | 红 | 去向：（从里程碑三挪来）里程碑八一·1 | 出处 |",
        "| C23 | 号带项号 | 拦 | 红 | 去向：里程碑八一·1 | 出处 |",
        "| C24 | 没写去向 | 拦 | 红 | 等里程碑三那一段先做 | 出处 |",
        "| C25 | 未排 | 拦 | 红 | 去向：未排 | 出处 |",
        "| C26 | 文件带目录 | 拦 | 红 | 去向：`.claude/kb/milestone/09-ninth-txn.md` 二·3 | 出处 |",
    ]
    missing_file_rows = [
        "| C31 | 没有那份的号 | 拦 | 红 | 去向：里程碑七一·1 | 出处 |",
        "| C32 | 文件名写错 | 拦 | 红 | 去向：`.claude/kb/milestone/09-ninety-txn.md` 二·3 | 出处 |",
        "| C33 | 后一个落点没有那份 | 拦 | 红 | 去向：里程碑八、里程碑十七 | 出处 |",
        "| C34 | 都在 | 拦 | 红 | 去向：里程碑八一·1、`09-ninth-txn.md` 二·3 | 出处 |",
    ]
    landing_milestones = (3, 4, 5, 8, 9, 13)

    def expect(cell, arguments, wanted_code, wanted, unwanted=()):
        said = []
        code = run(arguments, say=said.append)
        text = "\n".join(said)
        missing = [piece for piece in wanted if piece not in text]
        extra = [piece for piece in unwanted if piece in text]
        if code != wanted_code or missing or extra:
            failures.append((cell, f"期望退 {wanted_code}、实际退 {code}；找不到 {missing}；不该出现 {extra}；输出：{text}"))

    try:
        red = world("both-forms-red", named_rows + other_rows)
        expect("both-forms-red", ["3", red], 1, ["有 3 行的前置", "✗ C1（", "✗ C2（", "✗ C3（"], ["✗ C4（", "✗ C5（", "✗ C6（", "✗ C90（"])
        green = world("rerouted-green", other_rows)
        expect("rerouted-green", ["三", green], 0, ["开着的 3 行里没有一行的前置格「去向：」之后点名它"], ["✗"])
        elsewhere = world("elsewhere-named-green", elsewhere_rows)
        expect("elsewhere-named-green", ["3", elsewhere], 0, ["开着的 2 行里没有一行的前置格「去向：」之后点名它"], ["✗"])
        both = world("closed-only-red", named_rows + other_rows + ["| C7 | 去向开着的里程碑 | 拦 | 红 | 去向：里程碑四 | 出处 |"])
        expect("closed-only-red", ["--closed", both], 1, ["里程碑三（", "✗ C1（"], ["里程碑四（", "里程碑十三（", "✗ C7（"])
        closed_green = world("closed-green", other_rows)
        expect("closed-green", ["--closed", closed_green], 0, ["段表写了收口的里程碑 1 个", "其余 2 份段表没写收口"])
        none_closed = world("none-closed", named_rows, closed=())
        expect("none-closed", ["--closed", none_closed], 77, ["3 份里程碑文件的段表都没写"])
        no_heading = world("no-paid-heading-red", other_rows, paid_heading=False)
        expect("no-paid-heading-red", ["3", no_heading], 1, ["认不出「已还清」"])
        expect("thirteen", ["13", red], 1, ["✗ C4（"], ["✗ C3（"])
        expect("usage", ["99", red], 2, ["没有 99-*-txn.md"])
        landing = world("landing-only-named", landing_rows, closed=(), milestones=landing_milestones)
        expect("explanation-not-named", ["3", landing], 1, ["有 1 行的前置", "✗ C14（"], ["✗ C11（", "✗ C12（", "✗ C13（", "✗ C15（", "✗ C16（"])
        expect("section-name-not-named", ["4", landing], 1, ["有 1 行的前置", "✗ C14（"], ["✗ C13（"])
        expect("file-landing-named", ["5", landing], 1, ["有 1 行的前置", "✗ C13（"], ["✗ C11（", "✗ C14（"])
        expect("destination-form-green", ["--destination-form", landing], 0,
               ["判了 5 行，去向段开头都是落点、点名的里程碑文件都在：里程碑文件 2 行、「里程碑<号>」 2 行、「未排」 1 行", "没写「去向：」的 1 行"], ["✗"])
        form_red = world("destination-form-red", form_red_rows, closed=(), milestones=landing_milestones)
        expect("destination-form-red", ["--destination-form", form_red], 1,
               ["写了「去向：」的 5 行：2 行去向段开头不是三种落点之一，0 处", "✗ C21（先写挪出）：去向段开头不是三种落点之一：「挪出里程碑三，归", "✗ C22（括注起头）"],
               ["✗ C23（", "✗ C24（", "✗ C25（", "✗ C26（"])
        missing_file = world("destination-missing-file-red", missing_file_rows, closed=(), milestones=landing_milestones)
        expect("destination-missing-file-red", ["--destination-form", missing_file], 1,
               ["0 行去向段开头不是三种落点之一，3 处落点点名的里程碑文件找不到", "✗ C31（", "找不到 07-*-txn.md", "✗ C32（", "找不到 09-ninety-txn.md", "✗ C33（", "找不到 17-*-txn.md"],
               ["✗ C34（"])
        no_destination = world("destination-no-object", elsewhere_rows[1:], closed=())
        expect("destination-no-object", ["--destination-form", no_destination], 77, ["开着的 1 行里没有一行写了「去向：」"])
    finally:
        shutil.rmtree(work, ignore_errors=True)
    cells = ("both-forms-red", "rerouted-green", "elsewhere-named-green", "closed-only-red", "closed-green", "none-closed", "no-paid-heading-red", "thirteen", "usage",
             "explanation-not-named", "section-name-not-named", "file-landing-named", "destination-form-green", "destination-form-red",
             "destination-missing-file-red", "destination-no-object")
    for cell, failure in failures:
        print(f"  ✗ 自证 {cell}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自证 {len(cells)} 格里 {len(failures)} 格判错")  # gate-lint:summary
        print("  → 怎么办：看 destination_landings / named_milestones / owed_rows / closing_row / destination_form 那一处判法；MILESTONE_CLOSE_OWED_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ milestone-close-owed-check 自证通过（{len(cells)} 格）：去向段开头的中文号带项号、文件名（带不带目录）、阿拉伯号、「、」连写的几个落点都算点名，"
          "落点之后的括注与说明、作业点节名里的号、「未排」、出处一格、前置格里「去向：」之前与没写「去向：」的、已还清的行与别的里程碑号不算，"
          "--closed 只判段表写了收口的（译文与别的表不算），都没收口退 77，认不出「已还清」标题判红；"
          "--destination-form 判去向段开头不是三种落点之一的与落点文件找不到的，没写去向的只报数、一行都没写退 77")
    return 0


BREAK_VARIABLES = ("MILESTONE_CLOSE_OWED_BREAK",)


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；门禁样本根（当前目录有 .gate-cells 或 .gate-break-sample、没设 GATE_STAGED_FROM）里由阶段转来的；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    sample_root = (os.path.exists(".gate-cells") or os.path.exists(".gate-break-sample")) and not os.environ.get("GATE_STAGED_FROM")
    if not setting or arguments[:1] == ["--selftest"] or sample_root or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）和门禁样本根里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/kb/milestone-close-owed-check.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(sys.argv[1:]))
