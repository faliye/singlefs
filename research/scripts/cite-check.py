#!/usr/bin/env python3
# admission: always 判的是此刻报告里的引文与被引文件此刻的样子，几秒跑完；每次交回、每次核查都现判
# run-condition: command git
"""报告里的引文核对：每处「文件:行号」加引号里的原文，那一行（或区间）逐字含这句、不是标题行、不是背景材料的行号。

用法：
    cite-check.py <报告.md> [<报告.md> …] [--root 目录 …] [--background 背景材料.md …] [--unchanged-since <epoch 秒>]
    cite-check.py --selftest          # CITE_CHECK_BREAK=<项> 时必须判红

认的两种写法（同一行里）：
    「原文」（`路径:行号`）  或  「原文」(`路径:行号`)        引号紧挨着括号里的位置
    `路径:行号`：「原文」    或  `路径:行号` 「原文」          位置之后紧跟引号
行号可以是区间（`路径:12-15`）。路径带目录的按 --root（默认仓根）逐个找；只写文件名的在仓里 `git ls-files` 按文件名找，恰好一个才判。
原文里的「…」与「……」是省略：切开的每一段要按次序出现在那一行（或区间）里。比对前两边都去掉 ** 与反引号、把连续空白并成一个空格。
判不了的不判、单列：路径认不出（不在、文件名不唯一）、行号超出文件、给了 --unchanged-since 而被引文件在那之后改过。
退出码：0 全部对上（或没有可判的）；1 有对不上的；2 用法错。
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

LOCATION = r"`?(?P<path>[A-Za-z0-9_./\-一-鿿]+\.[A-Za-z0-9]+):(?P<first>\d+)(?:[-–](?P<last>\d+))?`?"
QUOTE = r"「(?P<quote>[^「」]{2,400})」"
QUOTE_THEN_LOCATION = re.compile(QUOTE + r"\s*[（(]\s*(?:[^（）()`]{0,12}?)" + LOCATION)
LOCATION_THEN_QUOTE = re.compile(LOCATION + r"\s*[：:]?\s*" + QUOTE)
ELLIPSIS = re.compile(r"…+|\.\.\.")


def normalized(text):
    return re.sub(r"\s+", " ", text.replace("**", "").replace("`", "")).strip()


def citations_in(report_text):
    """→ [(报告行号, 路径, 首行, 末行, 原文)]。"""
    found = []
    for line_number, line in enumerate(report_text.splitlines(), 1):
        seen = set()
        for pattern in (QUOTE_THEN_LOCATION, LOCATION_THEN_QUOTE):
            for match in pattern.finditer(line):
                first = int(match.group("first"))
                last = int(match.group("last") or first)
                key = (match.group("path"), first, last, match.group("quote"))
                if key not in seen:
                    seen.add(key)
                    found.append((line_number, match.group("path"), first, last, match.group("quote")))
    return found


def tracked_files_by_basename(root):
    try:
        output = subprocess.run(["git", "-C", root, "ls-files"], capture_output=True, text=True, check=True).stdout
    except (OSError, subprocess.CalledProcessError):
        return {}
    by_name = {}
    for relative in output.splitlines():
        by_name.setdefault(os.path.basename(relative), []).append(os.path.join(root, relative))
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


def check_reports(report_paths, roots, background_paths, unchanged_since):
    by_basename = tracked_files_by_basename(roots[0])
    background_lines = [(path, open(path, encoding="utf-8", errors="replace").read().splitlines()) for path in background_paths] or None
    bad, skipped, total = [], [], 0
    for report_path in report_paths:
        for citation in citations_in(open(report_path, encoding="utf-8", errors="replace").read()):
            total += 1
            verdict, explanation = judge(citation, roots, by_basename, background_lines, unchanged_since)
            report_line, path, first, last, quote = citation
            span = f"{first}" if first == last else f"{first}-{last}"
            if verdict == "bad":
                bad.append(f"{os.path.basename(report_path)}:{report_line} 引 {path}:{span}「{quote[:60]}」：{explanation}")
            elif verdict == "skip":
                skipped.append(f"{os.path.basename(report_path)}:{report_line} 引 {path}:{span}：{explanation}")
    return total, bad, skipped


def run(arguments):
    reports, roots, backgrounds, unchanged_since = [], [], [], None
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
                unchanged_since = float(value)
            index += 2
            continue
        reports.append(argument)
        index += 1
    if not reports:
        print("✗ 没给要核的报告\n→ 怎么办：cite-check.py <报告.md> [--root 目录] [--background 背景材料.md]")
        return 2
    total, bad, skipped = check_reports(reports, roots or [REPOSITORY_ROOT], backgrounds, unchanged_since)
    for line in bad:
        print(f"  ✗ {line}")  # gate-lint:detail
    for line in skipped:
        print(f"  - 没判：{line}")
    if bad:
        print(f"  ✗ 核了 {total} 处引文，{len(bad)} 处对不上，{len(skipped)} 处没判（逐处列在上面）")  # gate-lint:summary
        print("  → 怎么办：按每处的提示改行号或引文——行号去被引文件里现查（grep -nF），不从背景材料里数；引文整行抄，不许摘句改字")
        return 1
    print(f"  ✓ 核了 {total} 处引文，对上 {total - len(skipped)} 处，没判 {len(skipped)} 处（逐处列在上面）")
    return 0


def selftest():
    work = tempfile.mkdtemp(prefix="cite-check-selftest-")
    try:
        subprocess.run(["git", "-C", work, "init", "-q"], check=True)
        os.makedirs(os.path.join(work, "kb"))
        with open(os.path.join(work, "kb", "rule.md"), "w", encoding="utf-8") as handle:
            handle.write("# 规则\n\n## 一节\n\n第一条：**先查** 再写。\n第二条：引文整行抄，不许摘句。\n")
        with open(os.path.join(work, "code.rs"), "w", encoding="utf-8") as handle:
            handle.write("fn main() {\n    let answer = 42;\n}\n")
        with open(os.path.join(work, "background.md"), "w", encoding="utf-8") as handle:
            handle.write("背景\n第二条：引文整行抄，不许摘句。\n")
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
        ]
        failures = []
        for label, text, want in cases:
            report = os.path.join(work, "report.md")
            with open(report, "w", encoding="utf-8") as handle:
                handle.write(f"报告\n{text}\n")
            total, bad, _ = check_reports([report], [work], [os.path.join(work, "background.md")], None)
            got = 1 if bad else 0
            if label == "行号是背景材料的" and bad and "背景材料" not in bad[0]:
                got = 0
            if label == "指到标题行" and bad and "标题行" not in bad[0]:
                got = 0
            if total != 1 or got != want:
                failures.append(f"{label}：应当{'判红' if want else '判绿'}，实际数到 {total} 处、{'判红' if got else '判绿'}（{bad}）")
        changed_report = os.path.join(work, "report.md")
        with open(changed_report, "w", encoding="utf-8") as handle:
            handle.write("「第一条：先查 再写」（`kb/rule.md:6`）\n")
        _, bad, skipped = check_reports([changed_report], [work], [], unchanged_since=0.0)
        if bad or not skipped:
            failures.append(f"被引文件在开工之后改过：应当只列进没判，实际对不上 {bad}、没判 {skipped}")
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 citations_in() / judge() 的判法；CITE_CHECK_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ cite-check 自检通过：对上的、位置在前、省略号、只写文件名、区间放行，错行、标题行、背景材料行号、超出文件判红，认不出的与开工后改过的只列不判（{len(cases) + 1} 种）")
        return 0
    finally:
        subprocess.run(["rm", "-rf", "--", work], check=False)


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(sys.argv[1:]))
