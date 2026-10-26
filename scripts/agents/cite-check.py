#!/usr/bin/env python3
# admission: always 判的是此刻报告里的引文与被引文件此刻的样子，几秒跑完；每次交回、每次核查都现判
# run-condition: command git
"""报告里的引文核对：每处「文件:行号」加引号里的原文，那一行（或区间）逐字含这句、不是标题行、不是背景材料的行号。

用法：
    cite-check.py <报告.md> [<报告.md> …] [--root 目录 …] [--background 背景材料.md …] [--unchanged-since <epoch 秒>]
    cite-check.py --selftest          # CITE_CHECK_BREAK=<项> 时必须判红
    cite-check.py --help              # 打这一段用法（-h 同），退 0

认的两种写法（同一行里）：
    「原文」（`路径:行号`）  或  「原文」(`路径:行号`)        引号紧挨着括号里的位置
    `路径:行号`：「原文」    或  `路径:行号` 「原文」          位置之后紧跟引号
行号可以是区间（`路径:12-15`）；位置也可以写成小节：`路径@标题`（标题不带井号、逐字，原文要在那一节里，到同级或更高一级的下一个标题为止；
标题命中不是恰好一次判红）——引规则与 kb 的条款优先这样写，不用为一个行号再读一遍文件。路径带目录的按 --root（默认仓根）逐个找；只写文件名的按文件名找，恰好一个才判：--root 是 git 仓根的在 `git ls-files` 里找，不是 git 仓根的（snapshot-tree.py 铺的树、草稿目录）按目录逐层找实际文件（不进 .git、不跟目录链接）；几个 --root 的合起来找。
原文里的「…」与「……」是省略：切开的每一段要按次序出现在那一行（或区间）里。比对前两边都去掉 ** 与反引号、把连续空白并成一个空格。
判不了的不判、单列：路径认不出（不在、文件名不唯一）、行号超出文件、给了 --unchanged-since 而被引文件在那之后改过。
裸行号（紧挨着没有引文的 `路径:行号` 或 `路径:起-止`，路径的认法与上面同一份）也核，只核行号在不在文件里：
    文件找得到而行号（区间的止）大于文件行数 → 判红，说「`路径` 只有 N 行，引的是第 M 行——多半是背景材料附录里的行号」；
    行号在范围内 → 没有引文可比，列进「没判」并单独计数（行首写「没判（裸行号，没有引文可比）」）；
    路径认不出、给了 --unchanged-since 而文件改过 → 与带引文的一样列进「没判」。
    「…」引号里的字不扫裸行号；已经有引文的 `路径:行号` 不重复算。
    报数分开写：「核了 A 处引文、B 处裸行号」，A、B 各自计数。
退出码：0 全部对上（或没有可判的）；1 有对不上的；2 用法错（不认的选项、选项缺值、报告或背景材料读不了）。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一例的标签，括号里是那条「✗ 自检：」的开头）：
  CITE_CHECK_BREAK=always-ok  judge() 与 judge_section() 不比原文，每处引文都判对上
      行号错一行（「✗ 自检：行号错一行：应当判红」）、指到标题行（「✗ 自检：指到标题行：应当判红」）、
      行号是背景材料的（「✗ 自检：行号是背景材料的：应当判红」）、小节写法：原文不在那一节（「✗ 自检：小节写法：原文不在那一节：应当判红」）、
      非 git 根上只写文件名、行号错（「✗ 自检：非 git 根上只写文件名、行号错：应当判红」）
  CITE_CHECK_BREAK=ignore-changed  judge() 不看 --unchanged-since，开工之后改过的被引文件照判
      被引文件在开工之后改过（「✗ 自检：被引文件在开工之后改过：应当只列进没判」）
  CITE_CHECK_BREAK=no-background-check  judge() 不比背景材料，行号是背景材料的不单报
      行号是背景材料的（「✗ 自检：行号是背景材料的：应当判红」）
  CITE_CHECK_BREAK=no-heading-check  judge() 不单判行号指到标题行
      指到标题行（「✗ 自检：指到标题行：应当判红」）
  CITE_CHECK_BREAK=title-any-section  judge_section() 不要求标题恰好命中一次，原文在整份文件里找
      小节写法：标题不在（「✗ 自检：小节写法：标题不在：应当判红」）
  CITE_CHECK_BREAK=bare-line-refs-ignored  citations_in() 不收裸行号（改前的样子），超出文件的裸行号既不判红也不数
      裸行号超出文件（「✗ 自检：裸行号超出文件：应当判红」）、裸行号区间的止超出文件（「✗ 自检：裸行号区间的止超出文件：应当判红」）、
      裸行号只写文件名、仓里唯一（「✗ 自检：裸行号只写文件名、仓里唯一：应当判红」）、
      裸行号在范围内（「✗ 自检：裸行号在范围内：应当数 1 处裸行号、列进没判」）、裸行号区间在范围内（「✗ 自检：裸行号区间在范围内：应当数 1 处裸行号、列进没判」）、
      裸行号路径认不出（「✗ 自检：裸行号路径认不出：应当数 1 处裸行号、列进没判」）、命令行那两格裸行号报数（「✗ 自检：命令行 […/bare-red.md …」「✗ 自检：命令行 […/bare-green.md …」）
  CITE_CHECK_BREAK=raw-arguments  run() 不先判 --help、不认的选项、读不了的文件与写错的 --unchanged-since（改前的样子）
      命令行那几格（「✗ 自检：命令行 ['--help'] 应当退 0」起的六条）
      连带红的格：「✗ 自检：命令行 ['-h'] 应当退 0」、「✗ 自检：命令行 ['--bogus', '…'] 应当退 2」、「✗ 自检：命令行 ['…nothere.md'] 应当退 2」、「✗ 自检：命令行 ['…', '--background', '…'] 应当退 2」、「✗ 自检：命令行 ['…', '--unchanged-since', '昨天'] 应当退 2」
  CITE_CHECK_BREAK=non-git-root-unindexed  tracked_files_by_basename() 在不是 git 仓根的 --root 上不按目录找文件名、交空表（改前的样子：只写文件名的引文一律认不出、只列进没判；
      records/2026-10-23-subagent拆分提案.md 第一百二十二节 ①）
      非 git 根上只写文件名的引文行号错（「✗ 自检：非 git 根上只写文件名、行号错：应当判红」）、非 git 根上只写文件名的引文对上（「✗ 自检：非 git 根上只写文件名、对上：应当判绿且判了」）
"""
import os
import re
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402
BROKEN = os.environ.get("CITE_CHECK_BREAK", "")

LOCATION = r"`?(?P<path>[A-Za-z0-9_./\-一-鿿]+\.[A-Za-z0-9]+)(?::(?P<first>\d+)(?:[-–](?P<last>\d+))?|@(?P<title>[^`」（）()\n]{2,80}?))`?"
QUOTE = r"「(?P<quote>[^「」]{2,400})」"
QUOTE_THEN_LOCATION = re.compile(QUOTE + r"\s*[（(]\s*(?:[^（）()`]{0,12}?)" + LOCATION)
LOCATION_THEN_QUOTE = re.compile(LOCATION + r"\s*[：:]?\s*" + QUOTE)
ELLIPSIS = re.compile(r"…+|\.\.\.")
# 裸行号：`路径:行号` 或 `路径:起-止`，路径的认法与 LOCATION 同一份；扩展名要以字母起头（不收 0.5:1、1.2:3 这类数字写法），路径前不许紧挨路径字符与冒号（不收 URL 里的 host.com:8080）
BARE_LINE_REF = re.compile(r"(?<![A-Za-z0-9_./\-一-鿿:@])`?(?P<path>[A-Za-z0-9_./\-一-鿿]+\.[A-Za-z][A-Za-z0-9]*):(?P<first>\d+)(?:[-–](?P<last>\d+))?")


def normalized(text):
    return re.sub(r"\s+", " ", text.replace("**", "").replace("`", "")).strip()


def citations_in(report_text):
    """→ [(报告行号, 路径, 首行, 末行, 原文)]；裸行号（没有紧挨的引文）的原文是 None。"""
    found = []
    for line_number, line in enumerate(report_text.splitlines(), 1):
        seen = set()
        located = set()   # 带引文的位置里路径起头的下标，裸行号扫描时跳过这些
        for pattern in (QUOTE_THEN_LOCATION, LOCATION_THEN_QUOTE):
            for match in pattern.finditer(line):
                located.add(match.start("path"))
                if match.group("title") is not None:
                    first, last = match.group("title").strip(), None   # 小节写法：first 放标题、last 空
                else:
                    first = int(match.group("first"))
                    last = int(match.group("last") or first)
                key = (match.group("path"), first, last, match.group("quote"))
                if key not in seen:
                    seen.add(key)
                    found.append((line_number, match.group("path"), first, last, match.group("quote")))
        if BROKEN == "bare-line-refs-ignored":   # 弄坏开关：裸行号不认（改前的样子）
            continue
        outside_quotes = re.sub(QUOTE, lambda quoted: " " * len(quoted.group(0)), line)   # 引号里的字不扫；长度不变，下标对得上
        for match in BARE_LINE_REF.finditer(outside_quotes):
            if match.start("path") in located:
                continue
            first = int(match.group("first"))
            last = int(match.group("last") or first)
            key = (match.group("path"), first, last, None)
            if key not in seen:
                seen.add(key)
                found.append((line_number, match.group("path"), first, last, None))
    return found


def tracked_files_by_basename(root):
    """Git 仓索引已跟踪文件；普通快照目录索引实际文件，不跟随目录链接。"""
    relatives = None
    try:
        top = subprocess.run(["git", "-C", root, "rev-parse", "--show-toplevel"], capture_output=True, check=True).stdout
        if os.path.realpath(os.fsdecode(top).rstrip("\n")) == os.path.realpath(root):
            output = subprocess.run(["git", "-C", root, "ls-files", "-z"], capture_output=True, check=True).stdout
            relatives = list(dict.fromkeys(os.fsdecode(path) for path in output.split(b"\0") if path))
    except (OSError, subprocess.CalledProcessError):
        pass
    if relatives is None and BROKEN == "non-git-root-unindexed":   # 弄坏开关：非 git 根上不按目录找（改前的样子）
        return {}
    if relatives is None:
        relatives = []
        for directory, directories, files in os.walk(root):
            directories[:] = [name for name in directories if name != ".git"]
            relatives.extend(os.path.relpath(os.path.join(directory, name), root) for name in files)
    by_name = {}
    for relative in relatives:
        absolute = os.path.join(root, relative)
        if os.path.isfile(absolute):
            by_name.setdefault(os.path.basename(relative), []).append(absolute)
    return by_name


def resolve(path, roots, by_basename):
    """→ (绝对路径, None) 或 (None, 认不出的原因)。"""
    if os.path.isabs(path):
        return (path, None) if os.path.isfile(path) else (None, "文件不在")
    if "/" in path:
        for root in roots:
            candidate = os.path.join(root, path)
            if os.path.isfile(candidate):
                return candidate, None
        return None, "在给的根下都找不到"
    matches = by_basename.get(path, [])
    if len(matches) == 1:
        return matches[0], None
    return None, ("仓里没有这个文件名" if not matches else f"仓里有 {len(matches)} 个同名文件，只写文件名认不出是哪个")


def quote_is_in(quote, text):
    fragments = [normalized(part) for part in ELLIPSIS.split(quote)]
    fragments = [part for part in fragments if part]
    haystack = normalized(text)
    position = 0
    for fragment in fragments:
        found = haystack.find(fragment, position)
        if found < 0:
            return False
        position = found + len(fragment)
    return bool(fragments)


def judge(citation, roots, by_basename, background_lines, unchanged_since):
    """→ ('ok' | 'bad' | 'skip', 说明)。"""
    _, path, first, last, quote = citation
    absolute, why = resolve(path, roots, by_basename)
    if absolute is None:
        return "skip", f"路径认不出（{why}）"
    if unchanged_since is not None and os.path.getmtime(absolute) > unchanged_since and BROKEN != "ignore-changed":
        return "skip", "被引文件在开工之后改过，行号可能是开工那一刻的"
    lines = open(absolute, encoding="utf-8", errors="replace").read().splitlines()
    if quote is None:   # 裸行号：没有引文可比，只核行号在不在文件里
        if first < 1 or first > last:
            return "bad", f"裸行号写法不对（第 {first} 行到第 {last} 行）"
        if last > len(lines):
            return "bad", f"`{path}` 只有 {len(lines)} 行，引的是第 {last} 行——多半是背景材料附录里的行号"
        return "bare-skip", "行号在文件范围内，没有引文可比"
    if last is None:   # `路径@标题`：原文要在那一节里
        return judge_section(lines, first, quote)
    if first < 1 or last > len(lines) or first > last:
        return "bad", f"行号超出文件（文件共 {len(lines)} 行）"
    cited = "\n".join(lines[first - 1:last])
    if BROKEN == "always-ok" or quote_is_in(quote, cited):
        return "ok", ""
    if first == last and cited.lstrip().startswith("#") and BROKEN != "no-heading-check":
        return "bad", "行号指到标题行，原文不在那一行"
    hints = [str(index) for index, line in enumerate(lines, 1) if quote_is_in(quote, line)][:3]
    background_hint = ""
    if background_lines is not None and BROKEN != "no-background-check":
        for name, texts in background_lines:
            if 1 <= first <= len(texts) and quote_is_in(quote, texts[first - 1]):
                background_hint = f"；这个行号在背景材料 {name} 里那一行正好是这句——写的是背景材料的行号"
                break
    where = f"；原文在这份文件第 {', '.join(hints)} 行" if hints else "；这份文件里找不到这句"
    return "bad", f"那一行不含这句{where}{background_hint}"


def judge_section(lines, title, quote):
    """标题逐字命中恰好一行；那一节从标题行起、到同级或更高一级的下一个标题之前；原文要在节里。"""
    heads = [(index, len(line) - len(line.lstrip("#"))) for index, line in enumerate(lines)
             if line.startswith("#") and line.lstrip("#").strip() == title]
    if len(heads) != 1 and BROKEN != "title-any-section":
        return "bad", (f"标题「{title}」在这份文件里命中 {len(heads)} 次（要恰好 1 次）" + ("" if heads else "；标题逐字抄，不带井号"))
    start, level = heads[0] if heads else (0, 0)
    end = len(lines)
    for index in range(start + 1, len(lines)):
        if lines[index].startswith("#") and (len(lines[index]) - len(lines[index].lstrip("#"))) <= level:
            end = index
            break
    section = "\n".join(lines[start:end]) if BROKEN != "title-any-section" else "\n".join(lines)
    if BROKEN == "always-ok" or quote_is_in(quote, section):
        return "ok", ""
    hints = [str(index) for index, line in enumerate(lines, 1) if quote_is_in(quote, line)][:3]
    return "bad", f"「{title}」那一节（第 {start + 1}-{end} 行）里不含这句" + (f"；原文在这份文件第 {', '.join(hints)} 行" if hints else "；这份文件里找不到这句")


def check_reports(report_paths, roots, background_paths, unchanged_since):
    by_basename = {}
    for root in roots:
        for name, paths in tracked_files_by_basename(root).items():
            matches = by_basename.setdefault(name, [])
            for path in paths:
                if os.path.realpath(path) not in {os.path.realpath(match) for match in matches}:
                    matches.append(path)
    background_lines = [(path, open(path, encoding="utf-8", errors="replace").read().splitlines()) for path in background_paths] or None
    bad, skipped, total, bare_total, bare_unjudged = [], [], 0, 0, 0
    for report_path in report_paths:
        for citation in citations_in(open(report_path, encoding="utf-8", errors="replace").read()):
            if citation[4] is None:
                bare_total += 1
            else:
                total += 1
            verdict, explanation = judge(citation, roots, by_basename, background_lines, unchanged_since)
            report_line, path, first, last, quote = citation
            span = f"@{first}" if last is None else (f"{first}" if first == last else f"{first}-{last}")
            joiner = "" if span.startswith("@") else ":"
            shown = "" if quote is None else f"「{quote[:60]}」"
            if verdict == "bad":
                bad.append(f"{os.path.basename(report_path)}:{report_line} 引 {path}{joiner}{span}{shown}：{explanation}")
            elif verdict == "skip":
                skipped.append(f"没判：{os.path.basename(report_path)}:{report_line} 引 {path}{joiner}{span}：{explanation}")
            elif verdict == "bare-skip":
                bare_unjudged += 1
                skipped.append(f"没判（裸行号，没有引文可比）：{os.path.basename(report_path)}:{report_line} 引 {path}{joiner}{span}：{explanation}")
    return total, bare_total, bad, skipped, bare_unjudged


def run(arguments):
    reports, roots, backgrounds, unchanged_since = [], [], [], None
    guarded = BROKEN != "raw-arguments"   # 弄坏开关：--help、不认的选项、读不了的文件都不先判
    if guarded and arguments[:1] in (["--help"], ["-h"]):
        print(__doc__)
        return 0
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument in ("--root", "--background", "--unchanged-since"):
            if index + 1 >= len(arguments):
                print(f"✗ {argument} 后面没给值\n→ 怎么办：照文件头的用法补上")
                return 2
            value = arguments[index + 1]
            if argument == "--root":
                roots.append(os.path.abspath(value))
            elif argument == "--background":
                backgrounds.append(value)
            else:
                try:
                    unchanged_since = float(value)
                except ValueError:
                    if not guarded:
                        raise
                    print(f"✗ --unchanged-since 要一个 epoch 秒数，给的是「{value}」\n→ 怎么办：写开工时刻的秒数，例如 $(date +%s) 当时记下的那个数")
                    return 2
            index += 2
            continue
        if guarded and argument.startswith("-"):
            print(f"✗ 不认的选项：{argument}\n→ 怎么办：只认 --root、--background、--unchanged-since（用法跑 cite-check.py --help）")
            return 2
        reports.append(argument)
        index += 1
    if not reports:
        print("✗ 没给要核的报告\n→ 怎么办：cite-check.py <报告.md> [--root 目录] [--background 背景材料.md]")
        return 2
    unreadable = [path for path in reports + backgrounds if guarded and not os.path.isfile(path)]
    if unreadable:
        print(f"✗ 读不了 {'、'.join(unreadable)}（文件不在，或不是普通文件）\n→ 怎么办：核对路径（相对路径相对当前工作目录）；报告与背景材料都要给文件，不给目录")
        return 2
    total, bare_total, bad, skipped, bare_unjudged = check_reports(reports, roots or [REPOSITORY_ROOT], backgrounds, unchanged_since)
    for line in bad:
        print(f"  ✗ {line}")  # gate-lint:detail
    for line in skipped:
        print(f"  - {line}")
    unjudged_note = f"其中裸行号没有引文可比 {bare_unjudged} 处；逐处列在上面"
    if bad:
        print(f"  ✗ 核了 {total} 处引文、{bare_total} 处裸行号，{len(bad)} 处对不上，{len(skipped)} 处没判（{unjudged_note}）")  # gate-lint:summary
        print("  → 怎么办：按每处的提示改行号或引文——行号去被引文件里现查（grep -nF），不从背景材料里数；引文整行抄，不许摘句改字")
        return 1
    print(f"  ✓ 核了 {total} 处引文、{bare_total} 处裸行号，对上 {total + bare_total - len(skipped)} 处，没判 {len(skipped)} 处（{unjudged_note}）")
    return 0


def selftest():
    work = tempfile.mkdtemp(prefix="cite-check-selftest-")
    plain_roots = []   # 非 git 根那两格的临时目录（不在 work 里：work 是 git 仓）
    try:
        subprocess.run(["git", "-C", work, "init", "-q"], check=True)
        os.makedirs(os.path.join(work, "kb"))
        with open(os.path.join(work, "kb", "rule.md"), "w", encoding="utf-8") as handle:
            handle.write("# 规则\n\n## 一节\n\n第一条：**先查** 再写。\n第二条：引文整行抄，不许摘句。\n")
        with open(os.path.join(work, "code.rs"), "w", encoding="utf-8") as handle:
            handle.write("fn main() {\n    let answer = 42;\n}\n")
        with open(os.path.join(work, "background.md"), "w", encoding="utf-8") as handle:
            handle.write("背景\n第二条：引文整行抄，不许摘句。\n")
        os.makedirs(os.path.join(work, "crates", "singlefs-checker", "invariants"))
        invariants = "crates/singlefs-checker/invariants/README.md"   # 不到 300 行的小文件：背景材料附录里的 1638 行指不到它
        with open(os.path.join(work, invariants), "w", encoding="utf-8") as handle:
            handle.write("".join(f"第 {number} 行\n" for number in range(1, 21)))
        subprocess.run(["git", "-C", work, "add", "-A"], check=True)
        cases = [
            ("对上的引文", "「第一条：先查 再写」（`kb/rule.md:5`）", 0),
            ("位置在前的写法", "`kb/rule.md:6`：「引文整行抄，不许摘句」", 0),
            ("省略号切开按次序", "「第二条：…不许摘句」（`kb/rule.md:6`）", 0),
            ("只写文件名、仓里唯一", "「let answer = 42」（`code.rs:2`）", 0),
            ("行号区间", "「先查…整行抄」（`kb/rule.md:5-6`）", 0),
            ("行号错一行", "「第一条：先查 再写」（`kb/rule.md:6`）", 1),
            ("指到标题行", "「引文整行抄」（`kb/rule.md:3`）", 1),
            ("行号是背景材料的", "「第二条：引文整行抄」（`kb/rule.md:2`）", 1),
            ("行号超出文件", "「先查」（`kb/rule.md:99`）", 1),
            ("认不出的路径不判", "「随便」（`nothere/x.md:3`）", 0),
            ("小节写法：原文在那一节里", "「引文整行抄，不许摘句」（`kb/rule.md@一节`）", 0),
            ("小节写法：位置在前", "`kb/rule.md@一节`：「第一条：先查 再写」", 0),
            ("小节写法：标题不在", "「先查」（`kb/rule.md@没有的节`）", 1),
            ("小节写法：原文不在那一节", "「let answer = 42」（`kb/rule.md@一节`）", 1),
        ]
        failures = []
        for label, text, want in cases:
            report = os.path.join(work, "result.md")
            with open(report, "w", encoding="utf-8") as handle:
                handle.write(f"报告\n{text}\n")
            total, _, bad, _, _ = check_reports([report], [work], [os.path.join(work, "background.md")], None)
            got = 1 if bad else 0
            if label == "行号是背景材料的" and bad and "背景材料" not in bad[0]:
                got = 0
            if label == "指到标题行" and bad and "标题行" not in bad[0]:
                got = 0
            if total != 1 or got != want:
                failures.append(f"{label}：应当{'判红' if want else '判绿'}，实际数到 {total} 处、{'判红' if got else '判绿'}（{bad}）")
        # 非 git 根（snapshot-tree.py 铺的树、草稿目录这类）上只写文件名的引文：按目录找文件名，恰好一个才判（开关 non-git-root-unindexed 下认不出、只列进没判，两格都红）
        plain_root = tempfile.mkdtemp(prefix="cite-check-plain-root-")
        plain_roots.append(plain_root)
        os.makedirs(os.path.join(plain_root, "deep", "er"))
        with open(os.path.join(plain_root, "deep", "er", "laid.rs"), "w", encoding="utf-8") as handle:
            handle.write("fn laid() {\n    let tree = 7;\n}\n")
        plain_report = os.path.join(plain_root, "result.md")
        for label, text, want in (("非 git 根上只写文件名、行号错", "「let tree = 7」（`laid.rs:3`）", 1),
                                  ("非 git 根上只写文件名、对上", "「let tree = 7」（`laid.rs:2`）", 0)):
            with open(plain_report, "w", encoding="utf-8") as handle:
                handle.write(f"报告\n{text}\n")
            total, _, bad, skipped, _ = check_reports([plain_report], [plain_root], [], None)
            judged = total == 1 and not skipped
            if want and not bad:
                failures.append(f"{label}：应当判红，实际数到 {total} 处、对不上 {bad}、没判 {skipped}")
            if not want and (bad or not judged):
                failures.append(f"{label}：应当判绿且判了（不是只列进没判），实际数到 {total} 处、对不上 {bad}、没判 {skipped}")
        changed_report = os.path.join(work, "result.md")
        with open(changed_report, "w", encoding="utf-8") as handle:
            handle.write("「第一条：先查 再写」（`kb/rule.md:6`）\n")
        _, _, bad, skipped, _ = check_reports([changed_report], [work], [], unchanged_since=0.0)
        if bad or not skipped:
            failures.append(f"被引文件在开工之后改过：应当只列进没判，实际对不上 {bad}、没判 {skipped}")
        # 裸行号：(标签, 报告行, 应当判红, 引文数, 裸行号数, 其中没有引文可比数, 红的说明里该有的字)
        bare_cases = [
            ("裸行号超出文件", f"背景材料里的 `{invariants}:1638` 写了这条", True, 0, 1, 0, ["只有 20 行", "第 1638 行", "背景材料附录里的行号"]),
            ("裸行号区间的止超出文件", f"见 `{invariants}:15-30`", True, 0, 1, 0, ["只有 20 行", "第 30 行"]),
            ("裸行号只写文件名、仓里唯一", "见 code.rs:99", True, 0, 1, 0, ["只有 3 行", "第 99 行"]),
            ("裸行号在范围内", f"见 `{invariants}:7`", False, 0, 1, 1, []),
            ("裸行号区间在范围内", f"见 `{invariants}:7-9`", False, 0, 1, 1, []),
            ("裸行号路径认不出", "见 `nothere/x.md:3`", False, 0, 1, 0, []),
            ("带引文的不当裸行号再数", "「第一条：先查 再写」（`kb/rule.md:5`）", False, 1, 0, 0, []),
            ("位置在前带引文的不当裸行号再数", "`kb/rule.md:6`：「引文整行抄，不许摘句」", False, 1, 0, 0, []),
            ("引号里的行号不扫", "「见 kb/rule.md:99」", False, 0, 0, 0, []),
            ("网址端口与小数比不当裸行号", "见 http://example.com:8080 与比值 0.5:1", False, 0, 0, 0, []),
        ]
        for label, text, want_bad, want_total, want_bare, want_unjudged, want_words in bare_cases:
            bare_report = os.path.join(work, "bare-report.md")
            with open(bare_report, "w", encoding="utf-8") as handle:
                handle.write(f"报告\n{text}\n")
            total, bare_total, bad, skipped, unjudged = check_reports([bare_report], [work], [], None)
            said = " ".join(bad)
            if bool(bad) != want_bad or (total, bare_total, unjudged) != (want_total, want_bare, want_unjudged) or any(word not in said for word in want_words):
                failures.append(f"{label}：应当{'判红且说清实际行数' if want_bad else f'数 {want_bare} 处裸行号、列进没判 {want_unjudged} 处'}，"
                                f"实际数到 {total} 处引文、{bare_total} 处裸行号、没有引文可比 {unjudged} 处、{'判红' if bad else '判绿'}（{bad}）")
        # 命令行：--help / -h 打文件头的用法退 0；不认的选项退 2；报告读不了报文件名与出路、不抛 traceback
        bare_red_report = os.path.join(work, "bare-red.md")
        with open(bare_red_report, "w", encoding="utf-8") as handle:
            handle.write(f"`{invariants}:1638`\n")
        bare_green_report = os.path.join(work, "bare-green.md")
        with open(bare_green_report, "w", encoding="utf-8") as handle:
            handle.write(f"`{invariants}:7`\n")
        cli_cases = [([bare_red_report, "--root", work], 1, "核了 0 处引文、1 处裸行号，1 处对不上"),
                     ([bare_green_report, "--root", work], 0, "核了 0 处引文、1 处裸行号，对上 0 处，没判 1 处（其中裸行号没有引文可比 1 处"),
                     (["--help"], 0, "用法："), (["-h"], 0, "用法："), (["--bogus", changed_report], 2, "不认的选项：--bogus"),
                     ([os.path.join(work, "nothere.md")], 2, f"读不了 {os.path.join(work, 'nothere.md')}"),
                     ([changed_report, "--background", work], 2, f"读不了 {work}"),
                     ([changed_report, "--unchanged-since", "昨天"], 2, "--unchanged-since 要一个 epoch 秒数")]
        for arguments, want_code, want_text in cli_cases:
            child = subprocess.run([sys.executable, os.path.abspath(__file__), *arguments], capture_output=True, text=True)
            said = child.stdout + child.stderr
            if child.returncode != want_code or want_text not in said or "Traceback" in said or (want_code and "→ 怎么办" not in said):
                failures.append(f"命令行 {arguments} 应当退 {want_code}、说「{want_text}」（拒绝时带「→ 怎么办」、不抛 traceback），实际退 {child.returncode}：{said[-300:]}")
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 citations_in() / judge() / judge_section() / tracked_files_by_basename() 的判法；CITE_CHECK_BREAK（always-ok、no-heading-check、no-background-check、ignore-changed、title-any-section、bare-line-refs-ignored、non-git-root-unindexed）设着的话这里本来就该红")
            return 1
        print(f"  ✓ cite-check 自检通过：对上的、位置在前、省略号、只写文件名、区间放行，错行、标题行、背景材料行号、超出文件判红，认不出的与开工后改过的只列不判（{len(cases) + 1} 种）；"
              "非 git 根上只写文件名的按目录找到、照判（行号错判红、对上判绿，2 种）；"
              f"裸行号 {len(bare_cases)} 种：超出文件与区间的止超出文件判红并报实际行数，在范围内的列进没判单独计数，带引文的、引号里的、网址端口不当裸行号；"
              f"命令行 {len(cli_cases)} 种：--help、-h 退 0，不认的选项、读不了的报告与背景材料、写错的 --unchanged-since 退 2 带出路")
        return 0
    finally:
        subprocess.run(["rm", "-rf", "--", work, *plain_roots], check=False)


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(sys.argv[1:]))
