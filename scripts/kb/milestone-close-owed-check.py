#!/usr/bin/env python3
# admission: always 判的是此刻的欠账表与里程碑文件，几十毫秒读完；两份有一份改了都要现判，上一次的结论不替这一次作保
# run-condition: none 只读仓里的几份文本，除了 python3 之外没有环境要求，不起子进程
"""里程碑收口时欠账表里还有没有点名它的开着的账（用户 2026-10-22 指出里程碑三带着出口里还开着的欠账收了口）。

用法：
    milestone-close-owed-check.py <里程碑号> [仓根]     判这一个里程碑，不看它收没收口（收口之前先跑它）；号写 3 或 三
    milestone-close-owed-check.py --closed [仓根]       只判段表已写收口的里程碑（门禁 doc-process-records 的 milestone-close-owed 格调这一种）
    milestone-close-owed-check.py --selftest            MILESTONE_CLOSE_OWED_BREAK=<项> 时点名的格必须红
仓根不给就是这份脚本所在的仓。

判法（照文件现状读，不看历史）：
  里程碑文件：.claude/kb/milestone/<两位号>-<名>-txn.md（译文 .en.md、.ja.md 不算）。
  收没收口：文件里第一张段表（表头第一格是「段」、有一列叫「状态」）里，有一行的状态格（去掉 * 与空白）以「结束」或「已完结」起头，就算收口。
    没有段表的里程碑文件（例如只在文件头写「状态：判定结束」的）不算收口，--closed 不判它。
  欠着的账：.claude/kb/checks-owed.md 里「已还清」那一行整行标题之前、首列是 C<编号> 的表格行（切法与门禁共用 .claude/gate.d/lib-owed.py 的 PAID_HEADING、OWED_ROW）；
    认不出「已还清」标题判红，不对着一张认不出的表判。
  点名：一行按没转义的「|」切成六格，只看第五格「前置」里第一个「去向：」之后的那一段（同一格里写在「去向：」之前的说明与出处不算，
    一格里没有「去向：」的不点名任何里程碑）；切不成六格的整行都算。认两种写法：
    ① 「里程碑」后接号（阿拉伯数字或中文数字，中间可以有空白）：后面紧跟「·」的，最后一个中文数字是项号、不算进里程碑号
       （「里程碑三二·5」是里程碑三的二·5、「里程碑十四一·1」是里程碑十四的一·1）；前面紧挨着「-」的是带日期的文件名（records/2026-10-18-里程碑三收尾….md），不算；
    ② 里程碑文件名：<两位号>-<小写字母>-txn.md（带不带 .en / .ja、带不带目录都算）。
  点名这个里程碑的行数为 0 才绿；不为 0 逐行列出编号、简称与点名处前后的原文。

退出码：0 没有点名它的开着的账；1 有（逐行列出），或欠账表、里程碑文件读不了；2 用法错、找不到这个号的里程碑文件；
  77 --closed 下没有一个里程碑的段表写了收口（无对象可判）。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（格名是 selftest() 里那一格，自证打「✗ 自证 <格名>：…」）：
  MILESTONE_CLOSE_OWED_BREAK=item-suffix-kept        「里程碑三二·5」不拆项号，读成三十二
      「✗ 自证 both-forms-red：」
      「✗ 自证 closed-only-red：」
  MILESTONE_CLOSE_OWED_BREAK=file-form-ignored       不认里程碑文件名那种写法
      「✗ 自证 both-forms-red：」
  MILESTONE_CLOSE_OWED_BREAK=filename-dates-counted  带日期的文件名里的「里程碑三」也算点名
      「✗ 自证 both-forms-red：」、「✗ 自证 rerouted-green：」
      「✗ C5（带日期的文件名）：…（原话在 records/2026-10-18-里程碑三收尾与近期欠账.md）」
      「✗ 自证 closed-green：」
  MILESTONE_CLOSE_OWED_BREAK=whole-row-scanned       整行都看，不只看前置那一格
      「✗ 自证 both-forms-red：」、「✗ 自证 rerouted-green：」
      「✗ C1（中文号带项号）：…1 | 中文号带项号 | 拦 | 红 | 去向：里程碑三二·5（样本） | 出处 |」
      「✗ C2（文件名）：…向：`.claude/kb/milestone/03-third-txn.md`「待优化」第 27 行 | 出处 |」
      「✗ C3（阿拉伯号）：…C3 | 阿拉伯号 | 拦 | 红 | 去向：里程碑 3 | 出处 |」
      「✗ C6（只在出处）：…|b` | 红 | 去向：里程碑十四一·1 | 里程碑三二·4 的原话 |」
      「✗ 自证 closed-green：」、「✗ 自证 elsewhere-named-green：」
      「✗ C8（别处点名）：…」、「✗ C9（没写去向）：…」
  MILESTONE_CLOSE_OWED_BREAK=paid-rows-counted       「已还清」之后的行也算欠着的
      「✗ 自证 both-forms-red：」、「✗ 自证 rerouted-green：」
      「✗ C90（还清的）：…| C90 | 还清的 | 去向：里程碑三二·1 | 样本日期 |」
      「✗ 自证 closed-green：」、「✗ 自证 elsewhere-named-green：」
  MILESTONE_CLOSE_OWED_BREAK=status-ignored          --closed 不看段表，每个里程碑都判
      「✗ 自证 closed-only-red：」、「✗ 自证 none-closed：」
      「✗ 里程碑四（.claude/kb/milestone/04-fourth-txn.md）（段表还没写收口）：欠账表开着的 7 行里有 2 行的」
      「✗ C5（带日期的文件名）：…里程碑四一·2（原话在 records/2026-10-18-里程碑三收尾」
      「✗ C7（去向开着的里程碑）：…去向：里程碑四」
      「✗ 里程碑十三（.claude/kb/milestone/13-thirteenth-txn.md）（段表还没写收口）：欠账表开着的 7 行里有」
      「✗ C4（十三）：…去向：里程碑十三」
      「✗ 自证 closed-green：」
      「✗ 里程碑四（.claude/kb/milestone/04-fourth-txn.md）（段表还没写收口）：欠账表开着的 3 行里有 1 行的」
      「✗ 里程碑十三（.claude/kb/milestone/13-thirteenth-txn.md）（段表还没写收口）：欠账表开着的 3 行里有」
  MILESTONE_CLOSE_OWED_BREAK=paid-heading-optional   认不出「已还清」标题时不判红，把全文的行当欠着的判
      「✗ 自证 no-paid-heading-red：」
      「✗ C90（还清的）：…| C90 | 还清的 | 去向：里程碑三二·1 | 样本日期 |」
  MILESTONE_CLOSE_OWED_BREAK=prerequisite-whole-cell  前置那一格整格都看，不只看「去向：」之后（改前的读法）
      「✗ 自证 elsewhere-named-green：」
      「✗ C8（别处点名）：…里程碑三二·4 那一段先做的半件已合入；去向：里程碑四一·3」
      「✗ C9（没写去向）：…等 `.claude/kb/milestone/03-third-txn.md` 那一段先做」
"""
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
DESTINATION_MARK = "去向："       # 前置格里只看它之后的那一段（文件头「判法」的「点名」）
OWED_COLUMNS = 6
CLOSED_STATUS = ("结束", "已完结")
CHINESE_DIGITS = {"一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}
MILESTONE_WORD = re.compile(r"(?<!-)里程碑\s*([0-9]+|[一二三四五六七八九十]+)(·?)")
MILESTONE_WORD_WITH_FILENAMES = re.compile(r"里程碑\s*([0-9]+|[一二三四五六七八九十]+)(·?)")
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


def named_milestones(text):
    """→ {里程碑号: 第一次点名处的下标}（文件头「判法」的「点名」两种写法）。"""
    found = {}
    word = MILESTONE_WORD_WITH_FILENAMES if "filename-dates-counted" in BROKEN else MILESTONE_WORD
    for match in word.finditer(text):
        run = match.group(1)
        if "item-suffix-kept" in BROKEN:
            number = chinese_number(run)
            if number:
                found.setdefault(number, match.start())
            continue
        if not run.isdigit() and match.group(2) == "·" and len(run) > 1:
            run = run[:-1]
        number = chinese_number(run)
        if number is None and not run.isdigit():   # 号后面紧跟别的中文数字而没有「·」的，取认得出的最长前缀
            number = next((chinese_number(run[:length]) for length in (3, 2, 1) if length < len(run) and chinese_number(run[:length])), None)
        if number:
            found.setdefault(number, match.start())
    if "file-form-ignored" not in BROKEN:
        for match in MILESTONE_FILENAME.finditer(text):
            found.setdefault(int(match.group(1)), match.start())
    return found


def owed_rows(root):
    """→ ([(编号, 简称, 要看的那一段原文)], 问题)。只取开着的那张表（切法见 load_owed_library）。"""
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
        if len(cells) != OWED_COLUMNS or "whole-row-scanned" in BROKEN:
            scanned = line
        elif "prerequisite-whole-cell" in BROKEN:
            scanned = cells[PREREQUISITE_COLUMN]
        else:
            scanned = cells[PREREQUISITE_COLUMN].partition(DESTINATION_MARK)[2]   # 没有「去向：」的格交空串，不点名任何里程碑
        rows.append((row.group(1), row.group(2), scanned))
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


def judge(root, number, relative, closing, say):
    """判一个里程碑：→ 0 绿、1 红。"""
    label = f"里程碑{chinese_name(number)}（{relative}）"
    rows, problems = owed_rows(root)
    if problems:
        for problem in problems:
            say(f"  ✗ {problem}")  # gate-lint:detail
        say(f"  ✗ {label}：欠账表读不了，判不了还有没有点名它的开着的账")  # gate-lint:summary
        say("  → 怎么办：欠账表按「### 已还清」整行标题切成开着与还清两段（.claude/gate.d/lib-owed.py）；把那一行标题找回来，或把 --root 指到仓根再跑")
        return 1
    named = []
    for name, short_name, text in rows:
        position = named_milestones(text).get(number)
        if position is not None:
            snippet = text[max(0, position - SNIPPET_RADIUS):position + SNIPPET_RADIUS * 2].replace("\n", " ").strip()
            named.append((name, short_name, snippet))
    state = f"段表第 {closing[0]} 行状态写「{closing[1][:40]}」" if closing else "段表还没写收口"
    if named:
        say(f"  ✗ {label}（{state}）：欠账表开着的 {len(rows)} 行里有 {len(named)} 行的前置格「去向：」之后点名它：")  # gate-lint:summary
        for name, short_name, snippet in named:
            say(f"  ✗ {name}（{short_name}）：…{snippet}…")  # gate-lint:detail
        say(f"  → 怎么办：收口之前把这 {len(named)} 行逐行了结：做完的照 .claude/kb/checks-owed.md 表尾的写法挪进「已还清」，没做完的把去向改到还开着的里程碑"
            f"（前置格「去向：」之后不再写「里程碑{chinese_name(number)}」或 {os.path.basename(relative)}），改不了去向的就别收口，把段表那一格的「结束 / 已完结」撤回；"
            f"改完跑 python3 scripts/kb/milestone-close-owed-check.py {number} 到绿")
        return 1
    say(f"  ✓ {label}（{state}）：欠账表开着的 {len(rows)} 行里没有一行的前置格「去向：」之后点名它（认「里程碑<号>」与「{number:02d}-*-txn.md」两种写法）")
    return 0


def run(arguments, say=print):
    """命令行入口（自证也走这一份）：→ 退出码。"""
    if not arguments or len(arguments) > 2:
        say("  ✗ 参数不对")
        say("  → 怎么办：milestone-close-owed-check.py <里程碑号> [仓根] | --closed [仓根] | --selftest")
        return 2
    root = os.path.abspath(arguments[1]) if len(arguments) == 2 else REPOSITORY_ROOT
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

    def world(name, owed_rows_text, closed=(3,), paid_heading=True):
        root = os.path.join(work, name)
        os.makedirs(os.path.join(root, MILESTONE_DIRECTORY))
        owed = ["# 欠的检查清单", "", "| # | 简称 | 要拦什么 | 怎么拦（会红的形态） | 前置 | 出处 |", "|---|---|---|---|---|---|"]
        owed += owed_rows_text
        owed += ["", "### 已还清" if paid_heading else "### 别的", "", "| # | 简称 | 怎么还的 | 还清日期 |", "|---|---|---|---|",
                 "| C90 | 还清的 | 去向：里程碑三二·1 | 样本日期 |", ""]
        open(os.path.join(root, OWED_TABLE), "w", encoding="utf-8").write("\n".join(owed))
        for number, title in ((3, "third"), (4, "fourth"), (13, "thirteenth")):
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
    finally:
        shutil.rmtree(work, ignore_errors=True)
    cells = ("both-forms-red", "rerouted-green", "elsewhere-named-green", "closed-only-red", "closed-green", "none-closed", "no-paid-heading-red", "thirteen", "usage")
    for cell, failure in failures:
        print(f"  ✗ 自证 {cell}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自证 {len(cells)} 格里 {len(failures)} 格判错")  # gate-lint:summary
        print("  → 怎么办：看 named_milestones / owed_rows / closing_row 那一处判法；MILESTONE_CLOSE_OWED_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ milestone-close-owed-check 自证通过（{len(cells)} 格）：中文号带项号、文件名、阿拉伯号三种点名都认，带日期的文件名、出处一格、前置格里「去向：」之前与没写「去向：」的、已还清的行与别的里程碑号不算，"
          "--closed 只判段表写了收口的（译文与别的表不算），都没收口退 77，认不出「已还清」标题判红")
    return 0


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(sys.argv[1:]))
