#!/usr/bin/env python3
# admission: always 判的是此刻被判的仓里三语文件与术语表的样子，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本，除了 python3 之外没有环境要求
# gate-similar: .claude/singlefs-ai-sop/scripts/i18n-sync.sh 它判上游各语言仓之间的同步（兄弟目录里的仓、清单与溯源哈希）；这一份判本仓里登记的英、中、日三份文件，对象与输入都不同，不并进它
"""三语文件的判法：门禁 doc-text 的 trilingual-files、trilingual-structure、trilingual-changed-together、trilingual-glossary 四格调它，不单独成阶段。

三份：一份中文文件（默认语言）<名>.md 配 <名>.en.md（英语）与 <名>.ja.md（日语）。
登记表 .claude/trilingual-documents 一行一条「<模式>  # 理由」，模式指中文那一份（相对仓根；* 与 ? 不跨目录，** 跨目录），理由不许省。
名里可以带点；以 .<两个小写字母>.md 结尾的（.en.md、.ja.md 是译文，.fr.md 这类是别的语言）不是中文那一份，只有 .en.md 与 .ja.md 进组。

用法：
    lib-trilingual.py files <根>              登记的文件三份齐全；登记表本身写对了
    lib-trilingual.py structure <根>          每组三份的编号集合、链接目标集合、各级标题个数、表格行个数、围栏代码块个数逐项相同
    lib-trilingual.py changed-together <根>   标准输入给这次改动碰到的路径（一行一条）；碰了一组里的一份，三份都要碰
    lib-trilingual.py glossary <根>           术语表写对了，译文照术语表用词

退出码：0 都对；1 有不对的（逐条列出）；2 用法错；77 无对象可判（不是通过）；78 准入不满足（preflight）。

判法细节：
  files：登记表缺、读不出，或行没写理由、模式指不到任何文件、模式带语言后缀或不以 .md 结尾，判红；
        每组三份缺哪份点名哪份；反过来，登记模式所在目录里出现了 <名>.en.md 或 <名>.ja.md、而 <名>.md 符合某条登记模式，
        这一组同样要三份齐全（中文那份不在也判红）。
  structure：只比三份都在的组（缺份的归 files）。编号用 (?<![A-Za-z0-9_])(?:[DEC][0-9]+|I-[0-9]+(?:\\.[0-9]+)*)(?![0-9])，
        不用 \\b（日文字符算词字符，「ファイルE152」里 \\b 不成立）；链接目标取 ](目标) 里第一个空白之前的那一段，
        以 .en.md 或 .ja.md 结尾的目标先归成 <名>.md 再比（译文里链同一种语言的那一份，与中文链中文那一份算同一个目标；
        三语切换行 [中文](<名>.md) · [English](<名>.en.md) · [日本語](<名>.ja.md) 每份不链自己，归名之后三份都是 <名>.md）；
        标题取围栏外 ATX 写法（行首至多三个空格、一到六个 #、后接空白或行尾）的行，按级别分别数；表格行取围栏外去掉行首空白后以 | 起头的行；
        围栏取 ``` 或 ~~~ 起头的开围栏，数开围栏的个数。
  changed-together：路径是 <名>.md、<名>.en.md、<名>.ja.md 之一，而 <名>.md 符合某条登记模式，就算碰到了那一组（不看文件还在不在，删掉的也算碰）。
  glossary：术语表 .claude/kb/translation-glossary.md，单独一行 <!-- trilingual-glossary --> 紧挨着表，表头
        | 中文 | English | 日本語 | 说明 / Note / 注 |，分隔行，再是数据行（到第一个不以 | 起头的行为止）。
        表：每行四列，中文、English、日本語三列不空；中文列不重复；说明列可以空，不空就要恰好三段、用 <br> 分隔、每段不空。
        译文：每组三份齐全的，中文列的串在 zh 那份里原样出现，则 English 列的串要在 en 那份里出现（不分大小写），日本語列的串要在 ja 那份里原样出现。

弄坏开关 LIB_TRILINGUAL_BREAK=<项>（只给门禁 doc-text 的样本证红用，一次给一项）：
  registry-reason-ignored     登记表的行不要求理由：files-registry-red 里「没写理由」那一行找不到
  registry-unmatched-ignored  模式指不到任何文件不报：files-registry-red 里「指不到任何文件」那一行找不到
  registry-suffix-ignored     模式带语言后缀不报：files-registry-red 里「带语言后缀」那一行找不到
  reverse-ignored             不看目录里单有 .en.md / .ja.md 的那一组：files-red 里点名 2026-10-10 与 2026-10-12 的两行找不到
  ids-ignored                 不比编号集合：structure-red 里「编号集合」那一行找不到
  word-boundary               编号前后的限制换成 \\b：structure-green 里日文紧挨编号的写法对不上，判红
  links-ignored               不比链接目标集合：structure-red 里「链接目标集合」那一行找不到
  link-language-kept          链接目标里的 .en.md / .ja.md 不归成 <名>.md：structure-green 里 README 三份的三语切换行与 2026-09-28 三份各链同一种语言的下一份简报对不上，判红
  headings-ignored            不比标题个数：structure-red 里「标题」那一行找不到
  table-rows-ignored          不比表格行个数：structure-red 里「表格行」那一行找不到
  fences-ignored              不比围栏代码块个数：structure-red 里「围栏代码块」那一行找不到
  fence-blind                 围栏里的行也当正文数标题与表格行：structure-green 里围栏内的写法让个数对不上，判红
  partial-ignored             碰了一份就当三份都碰了：changed-together-red 里点名没碰全的那几行找不到
  glossary-marker-ignored     找不到标记不报：glossary-marker-red 判不出红
  glossary-header-ignored     表头对不上不报：glossary-table-red 里「表头」那一行找不到
  glossary-columns-ignored    列数不对、前三列有空不报：glossary-table-red 里「四列」「是空的」两行找不到
  glossary-duplicate-ignored  中文列重复不报：glossary-table-red 里「重复」那一行找不到
  glossary-note-ignored       说明列的三段不判：glossary-table-red 里「说明列」那一行找不到
  glossary-translation-ignored  不核译文用词：glossary-red 里点名缺哪份哪个译名的几行找不到
  glossary-case-sensitive     English 列按大小写比：glossary-green 里大小写不同的写法判红
  glossary-zh-gate-ignored    中文术语不在 zh 里也核译名：glossary-green 里只在别的组出现的术语让这一组判红；
                              glossary-red、glossary-table-red 汇总行里的处数也对不上
"""
import glob
import os
import posixpath
import re
import sys

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_OF_GATES = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_OF_GATES, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

REGISTRY = ".claude/trilingual-documents"
GLOSSARY = ".claude/kb/translation-glossary.md"
GLOSSARY_MARKER = "<!-- trilingual-glossary -->"
GLOSSARY_HEADER = ["中文", "English", "日本語", "说明 / Note / 注"]
GLOSSARY_COLUMN_NAMES = ["中文", "English", "日本語"]
LANGUAGES = (("zh", ""), ("en", ".en"), ("ja", ".ja"))
MINIMUM_REASON_CHARACTERS = 4
DETAIL_LIMIT = 200
BROKEN = os.environ.get("LIB_TRILINGUAL_BREAK", "")

IDENTIFIER_BODY = r"(?:[DEC][0-9]+|I-[0-9]+(?:\.[0-9]+)*)(?![0-9])"
IDENTIFIER = re.compile(r"(?<![A-Za-z0-9_])" + IDENTIFIER_BODY)
IDENTIFIER_WORD_BOUNDARY = re.compile(r"\b" + IDENTIFIER_BODY)
LINK_TARGET = re.compile(r"\]\(([^)]*)\)")
FENCE_OPEN = re.compile(r"^ {0,3}(`{3,}|~{3,})")
FENCE_CLOSE = re.compile(r"^ {0,3}(`{3,}|~{3,})[ \t]*$")
HEADING = re.compile(r"^ {0,3}(#{1,6})(?:[ \t]|$)")
LANGUAGE_FILE = re.compile(r"^(?P<name>.+)\.(?P<language>en|ja)\.md$")
LINK_LANGUAGE_SUFFIX = re.compile(r"\.(?:en|ja)\.md(?=$|[#?])")
LANGUAGE_SUFFIXED = re.compile(r"\.[a-z]{2}\.md$")
CHANGED_FILE = re.compile(r"^(?P<directory>(?:.*/)?)(?P<name>[^/]+?)(?P<suffix>\.en|\.ja)?\.md$")
TABLE_SEPARATOR_CELL = re.compile(r"^:?-+:?$")
UNESCAPED_PIPE = re.compile(r"(?<!\\)\|")
GLOB_CHARACTERS = re.compile(r"[*?]")


def report_problems(summary, details, advice):
    """红：汇总一行、明细逐行（超过上限的折叠）、出路。"""
    print(f"  ✗ {summary}")   # gate-lint:summary
    for detail in details[:DETAIL_LIMIT]:
        print(f"      {detail}")   # gate-lint:detail
    if len(details) > DETAIL_LIMIT:
        print(f"      （另有 {len(details) - DETAIL_LIMIT} 条没列出）")   # gate-lint:detail
    print(f"     → 怎么办：{advice[0]}")
    for line in advice[1:]:
        print(f"               {line}")
    return 1


def read_text(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        return handle.read()


def pattern_regex(pattern):
    """登记模式 → 正则：* 与 ? 不跨目录，** 跨目录，其余按字面。"""
    pieces, index = [], 0
    while index < len(pattern):
        if pattern.startswith("**/", index):
            pieces.append("(?:.*/)?")
            index += 3
        elif pattern.startswith("**", index):
            pieces.append(".*")
            index += 2
        elif pattern[index] == "*":
            pieces.append("[^/]*")
            index += 1
        elif pattern[index] == "?":
            pieces.append("[^/]")
            index += 1
        else:
            pieces.append(re.escape(pattern[index]))
            index += 1
    return re.compile("^" + "".join(pieces) + "$")


def load_registry():
    """→ (条目列表, 登记表自己的毛病列表)；登记表不在返回 (None, [])。条目：行号、模式、理由、正则、匹配到的英语文件（排序）。"""
    if not os.path.isfile(REGISTRY):
        return None, []
    entries, problems = [], []
    for line_number, raw in enumerate(read_text(REGISTRY).split("\n"), 1):
        text = raw.strip()
        if not text or text.startswith("#"):
            continue
        pattern, separator, reason = text.partition("#")
        pattern, reason = pattern.strip(), reason.strip()
        where = f"{REGISTRY}:{line_number}"
        if BROKEN != "registry-reason-ignored" and (not separator or len(reason) < MINIMUM_REASON_CHARACTERS):
            problems.append(f"{where}：模式 {pattern} 没写理由（一行是「<模式>  # 理由」，理由至少 {MINIMUM_REASON_CHARACTERS} 个字）")
        pattern = posixpath.normpath(pattern) if pattern else pattern
        base = posixpath.basename(pattern)
        if not pattern or pattern.startswith("/") or pattern.split("/")[0] == "..":
            problems.append(f"{where}：模式 {pattern or '（空）'} 要写成相对仓根的路径")
            continue
        if not base.endswith(".md") or (LANGUAGE_SUFFIXED.search(base) and BROKEN != "registry-suffix-ignored"):
            problems.append(f"{where}：模式 {pattern} 带语言后缀或不以 .md 结尾，登记的是中文那一份 <名>.md")
            continue
        matches = sorted(path for path in glob.glob(pattern, root_dir=".", recursive=True)
                         if os.path.isfile(path) and not LANGUAGE_SUFFIXED.search(posixpath.basename(path)))
        if not matches and BROKEN != "registry-unmatched-ignored":
            problems.append(f"{where}：模式 {pattern} 指不到任何文件")
        entries.append({"line": line_number, "pattern": pattern, "regex": pattern_regex(pattern), "matches": matches})
    return entries, problems


def registered(entries, chinese_path):
    return any(entry["regex"].match(chinese_path) for entry in entries)


def language_paths(directory, name):
    return {language: posixpath.join(directory, name + suffix + ".md") for language, suffix in LANGUAGES}


def build_groups(entries):
    """登记模式指到的中文文件各成一组；模式所在目录里单有 .en.md 或 .ja.md 的也成一组。→ 按组名排序的列表。"""
    keys = set()
    directories = set()
    for entry in entries:
        for path in entry["matches"]:
            directory, base = posixpath.split(path)
            keys.add((directory, base[:-3]))
            directories.add(directory)
        if not GLOB_CHARACTERS.search(posixpath.dirname(entry["pattern"])):
            directories.add(posixpath.dirname(entry["pattern"]))
    if BROKEN != "reverse-ignored":
        for directory in sorted(directories):
            try:
                names = os.listdir(directory or ".")
            except OSError:
                continue
            for file_name in names:
                found = LANGUAGE_FILE.match(file_name)
                if found and registered(entries, posixpath.join(directory, found.group("name") + ".md")):
                    keys.add((directory, found.group("name")))
    groups = []
    for directory, name in sorted(keys):
        paths = language_paths(directory, name)
        groups.append({"key": posixpath.join(directory, name), "paths": paths,
                       "missing": [language for language, _ in LANGUAGES if not os.path.isfile(paths[language])]})
    return groups


def require_registry():
    entries, problems = load_registry()
    if entries is None:
        report_problems(f"没有三语文件登记表 {REGISTRY}",
                        [],
                        [f"建 {REGISTRY}：一行一条「<模式>  # 理由」，模式指中文那一份（例 README.md、briefs/*.md），理由不许省；"
                         "四格都从它取对象。"])
        return None, None
    return entries, problems


def judge_files():
    entries, registry_problems = require_registry()
    if entries is None:
        return 1
    groups = build_groups(entries)
    details = list(registry_problems)
    incomplete = 0
    for group in groups:
        if group["missing"]:
            incomplete += 1
            names = "、".join(f"{group['paths'][language]}（{language}）" for language in group["missing"])
            details.append(f"{group['key']}：缺 {names}")
    if details:
        advice = []
        if incomplete:
            advice.append("缺哪份补哪份：<名>.md 是中文（默认），<名>.en.md 是英语，<名>.ja.md 是日语，三份同时在、同时改；")
        if registry_problems:
            advice.append(f"登记表 {REGISTRY} 一行一条「<模式>  # 理由」，模式指中文那一份（相对仓根）、要指到文件、理由不许省；"
                          "不再登记的整行删掉。")
        return report_problems(f"三语文件有 {len(details)} 处不对（登记表 {len(registry_problems)} 处，三份没配齐的 {incomplete} 组，共 {len(groups)} 组）",
                               details, advice)
    if not groups:
        print("  ! 登记表里没有一条模式指到文件，这一格无对象可判")
        return 77
    print(f"  ✓ 查了登记表 {len(entries)} 条，指到 {len(groups)} 组，每组三份（{len(groups) * 3} 份文件）都在；"
          "登记目录里单有 .en.md 或 .ja.md 的也归在这些组里")
    return 0


def analyze(text):
    """→ 编号集合、链接目标集合（.en.md / .ja.md 归成 .md）、各级标题个数、表格行个数、围栏个数。"""
    identifier = IDENTIFIER_WORD_BOUNDARY if BROKEN == "word-boundary" else IDENTIFIER
    identifiers = set(identifier.findall(text))
    links = set()
    for target in LINK_TARGET.findall(text):
        words = target.split()
        if words:
            link = words[0].strip("<>")
            if BROKEN != "link-language-kept":
                link = LINK_LANGUAGE_SUFFIX.sub(".md", link)
            links.add(link)
    headings = {level: 0 for level in range(1, 7)}
    table_rows = fences = 0
    open_fence = None
    for line in text.splitlines():
        if open_fence is not None:
            closing = FENCE_CLOSE.match(line)
            if closing and closing.group(1)[0] == open_fence[0] and len(closing.group(1)) >= open_fence[1]:
                open_fence = None
            if BROKEN != "fence-blind":
                continue
        else:
            opening = FENCE_OPEN.match(line)
            if opening:
                fences += 1
                open_fence = (opening.group(1)[0], len(opening.group(1)))
                continue
        heading = HEADING.match(line)
        if heading:
            headings[len(heading.group(1))] += 1
        if line.lstrip().startswith("|"):
            table_rows += 1
    return {"ids": identifiers, "links": links, "headings": headings, "table_rows": table_rows, "fences": fences}


def set_difference_note(name, sets):
    union = set().union(*sets.values())
    lacks = []
    for language, _ in LANGUAGES:
        missing = sorted(union - sets[language])
        if missing:
            shown = "、".join(missing[:8]) + (f" 等 {len(missing)} 个" if len(missing) > 8 else "")
            lacks.append(f"{language} 缺 {shown}")
    sizes = "、".join(f"{language} {len(sets[language])} 个" for language, _ in LANGUAGES)
    return f"{name}不同：{sizes}；{'；'.join(lacks)}"


def compare_structure(infos):
    """三份的分析结果逐项比，→ 不同项的说明列表。"""
    notes = []
    languages = [language for language, _ in LANGUAGES]
    if BROKEN != "ids-ignored" and len({frozenset(infos[language]["ids"]) for language in languages}) > 1:
        notes.append(set_difference_note("编号集合", {language: infos[language]["ids"] for language in languages}))
    if BROKEN != "links-ignored" and len({frozenset(infos[language]["links"]) for language in languages}) > 1:
        notes.append(set_difference_note("链接目标集合", {language: infos[language]["links"] for language in languages}))
    if BROKEN != "headings-ignored":
        for level in range(1, 7):
            counts = {language: infos[language]["headings"][level] for language in languages}
            if len(set(counts.values())) > 1:
                notes.append(f"{'#' * level} 标题个数不同：" + "，".join(f"{language} {counts[language]}" for language in languages))
    for key, label, switch in (("table_rows", "表格行个数", "table-rows-ignored"), ("fences", "围栏代码块个数", "fences-ignored")):
        if BROKEN == switch:
            continue
        counts = {language: infos[language][key] for language in languages}
        if len(set(counts.values())) > 1:
            notes.append(f"{label}不同：" + "，".join(f"{language} {counts[language]}" for language in languages))
    return notes


def complete_groups(entries):
    groups = build_groups(entries)
    return [group for group in groups if not group["missing"]], [group for group in groups if group["missing"]]


def judge_structure():
    entries, _ = require_registry()
    if entries is None:
        return 1
    judged, skipped = complete_groups(entries)
    details = []
    for group in judged:
        infos = {language: analyze(read_text(group["paths"][language])) for language, _ in LANGUAGES}
        for note in compare_structure(infos):
            details.append(f"{group['key']}：{note}")
    if details:
        different = len({detail.split("：", 1)[0] for detail in details})
        return report_problems(f"三份的结构对不上：{different} 组里 {len(details)} 项不同（查了 {len(judged)} 组）",
                               details,
                               ["三份说的是同一件事，编号、链接目标、各级标题个数、表格行个数、围栏代码块个数都要一样：照点名的组和项，"
                                "把落后的那几份改到与改了的那份一致（改动是另一份多出来的内容就补译，是误删就恢复）；",
                                "围栏代码块里的 # 开头行与 | 开头行不算标题与表格行。"])
    if not judged:
        print(f"  ! 没有三份齐全的组可比（缺份的 {len(skipped)} 组归 trilingual-files），这一格无对象可判")
        return 77
    print(f"  ✓ 查了 {len(judged)} 组三份的结构：编号集合、链接目标集合、各级标题个数、表格行个数、围栏代码块个数都对得上；"
          f"缺份没比的 {len(skipped)} 组归 trilingual-files")
    return 0


def judge_changed_together(changed_paths):
    entries, _ = require_registry()
    if entries is None:
        return 1
    touched = {}
    for path in changed_paths:
        found = CHANGED_FILE.match(posixpath.normpath(path)) if path else None
        if not found or (found.group("suffix") is None and LANGUAGE_SUFFIXED.search(path)):
            continue
        chinese = found.group("directory") + found.group("name") + ".md"
        if not registered(entries, chinese):
            continue
        language = {None: "zh", ".en": "en", ".ja": "ja"}[found.group("suffix")]
        touched.setdefault(found.group("directory") + found.group("name"), set()).add(language)
    if not touched:
        print("  ! 这次改动没碰到登记的三语文件，这一格无对象可判")
        return 77
    details = []
    for key in sorted(touched):
        have = [language for language, _ in LANGUAGES if language in touched[key]]
        lacking = [language for language, _ in LANGUAGES if language not in touched[key]]
        if lacking and BROKEN != "partial-ignored":
            directory, name = posixpath.split(key)
            paths = language_paths(directory, name)
            details.append(f"{key}：碰了 {'、'.join(paths[language] for language in have)}，没碰 {'、'.join(paths[language] for language in lacking)}")
    if details:
        return report_problems(f"这次改动碰了 {len(touched)} 组三语文件，其中 {len(details)} 组没碰全三份",
                               details,
                               ["改一份就要三份一起改：把没碰的那几份照改了的那份同步译过去，再重跑；",
                                "这一组其实不该改动的，把误碰的那一份改回去（git diff 看碰了什么）。"])
    print(f"  ✓ 这次改动碰到的 {len(touched)} 组三语文件，每组三份都碰了")
    return 0


def split_table_row(line):
    text = line.strip()
    if text.startswith("|"):
        text = text[1:]
    if text.endswith("|") and not text.endswith("\\|"):
        text = text[:-1]
    return [cell.strip().replace("\\|", "|") for cell in UNESCAPED_PIPE.split(text)]


def parse_glossary(text):
    """→ (可用于核译文的行 [(行号, 中文, English, 日本語)], 毛病 [(行号或 None, 说明)])。"""
    lines = text.split("\n")
    problems, terms = [], []
    markers = [index for index, line in enumerate(lines) if line.strip() == GLOSSARY_MARKER]
    if not markers:
        if BROKEN != "glossary-marker-ignored":
            problems.append((None, f"找不到单独一行的 {GLOSSARY_MARKER} 标记"))
        return terms, problems
    if len(markers) > 1:
        problems.append((markers[1] + 1, f"标记 {GLOSSARY_MARKER} 出现了 {len(markers)} 次，只许一处"))
    marker = markers[0]
    if marker + 1 >= len(lines) or not lines[marker + 1].lstrip().startswith("|"):
        problems.append((marker + 1, "标记下面不是表：表头要紧挨着标记那一行"))
        return terms, problems
    header = split_table_row(lines[marker + 1])
    if header != GLOSSARY_HEADER and BROKEN != "glossary-header-ignored":
        problems.append((marker + 2, f"表头不对：要 | {' | '.join(GLOSSARY_HEADER)} |，实际 | {' | '.join(header)} |"))
    separator_line = lines[marker + 2] if marker + 2 < len(lines) else ""
    separator = split_table_row(separator_line) if separator_line.lstrip().startswith("|") else []
    if not separator or not all(TABLE_SEPARATOR_CELL.match(cell) for cell in separator):
        problems.append((marker + 3, "表头下面不是分隔行（| --- | --- | --- | --- |），标记下面不是表"))
        return terms, problems
    first_seen = {}
    row_count = 0
    index = marker + 3
    while index < len(lines) and lines[index].lstrip().startswith("|"):
        number = index + 1
        cells = split_table_row(lines[index])
        index += 1
        row_count += 1
        if BROKEN != "glossary-columns-ignored":
            if len(cells) != 4:
                problems.append((number, f"表每行要四列，这一行有 {len(cells)} 列"))
                continue
            for name, cell in zip(GLOSSARY_COLUMN_NAMES, cells[:3]):
                if not cell:
                    problems.append((number, f"{name}列是空的"))
        elif len(cells) != 4:
            continue
        if cells[0]:
            if cells[0] in first_seen and BROKEN != "glossary-duplicate-ignored":
                problems.append((number, f"中文列「{cells[0]}」与第 {first_seen[cells[0]]} 行重复"))
            first_seen.setdefault(cells[0], number)
        note = cells[3]
        if note and BROKEN != "glossary-note-ignored":
            segments = note.split("<br>")
            if len(segments) != 3 or any(not segment.strip() for segment in segments):
                problems.append((number, f"说明列不空就要恰好三段、用 <br> 分隔、每段不空，这一行有 {len(segments)} 段"
                                         f"（空段 {sum(1 for segment in segments if not segment.strip())} 段）"))
        if cells[0] and cells[1] and cells[2]:
            terms.append((number, cells[0], cells[1], cells[2]))
    if row_count == 0:
        problems.append((marker + 3, "表里一行数据都没有"))
    return terms, problems


def judge_glossary():
    entries, _ = require_registry()
    if entries is None:
        return 1
    if not os.path.isfile(GLOSSARY):
        return report_problems(f"没有翻译术语表 {GLOSSARY}", [],
                               [f"建 {GLOSSARY}：表上方单独一行 {GLOSSARY_MARKER}，紧挨着表，表头 | {' | '.join(GLOSSARY_HEADER)} |；"
                                "译文照它用词，这一格才有对象可判。"])
    terms, problems = parse_glossary(read_text(GLOSSARY))
    details = [f"{GLOSSARY}:{number}：{message}" if number else f"{GLOSSARY}：{message}" for number, message in problems]
    judged, skipped = complete_groups(entries)
    hits = 0
    for group in judged:
        texts = {language: read_text(group["paths"][language]) for language, _ in LANGUAGES}
        english_folded = texts["en"] if BROKEN == "glossary-case-sensitive" else texts["en"].casefold()
        for number, chinese, english, japanese in terms:
            if chinese not in texts["zh"] and BROKEN != "glossary-zh-gate-ignored":
                continue
            hits += 1
            if BROKEN == "glossary-translation-ignored":
                continue
            wanted_english = english if BROKEN == "glossary-case-sensitive" else english.casefold()
            if wanted_english not in english_folded:
                details.append(f"{group['key']}：中文「{chinese}」在 zh 里出现，en 里找不到「{english}」（术语表第 {number} 行）")
            if japanese not in texts["ja"]:
                details.append(f"{group['key']}：中文「{chinese}」在 zh 里出现，ja 里找不到「{japanese}」（术语表第 {number} 行）")
    if details:
        return report_problems(f"翻译术语表与译文有 {len(details)} 处不对（表自己 {len(problems)} 处；查了 {len(judged)} 组、{len(terms)} 条术语）",
                               details,
                               [f"表的毛病照点名的行改 {GLOSSARY}（每行四列，前三列不空，中文列不重复，说明列空着或恰好三段用 <br> 分隔）；",
                                "译文的毛病改译文：zh 里用了某个术语，en 和 ja 里就要用术语表给的译名，三份都改；",
                                "术语表里的译名本身要改的，改表，再把三份里用到它的地方一起改。"])
    if not judged:
        print(f"  ! 没有三份齐全的组可核译文（缺份的 {len(skipped)} 组归 trilingual-files）；术语表本身 {len(terms)} 条写对了，这一格无对象可判")
        return 77
    print(f"  ✓ 术语表 {len(terms)} 条写对了；{len(judged)} 组三份里中文术语在 zh 出现 {hits} 次，对应的英文、日文译名都在 en、ja 里；"
          f"缺份没核的 {len(skipped)} 组归 trilingual-files")
    return 0


COMMANDS = ("files", "structure", "changed-together", "glossary")


def main(arguments):
    if len(arguments) != 2 or arguments[0] not in COMMANDS:
        print(f"  ✗ 用法：lib-trilingual.py {{{'|'.join(COMMANDS)}}} <项目根>", file=sys.stderr)   # gate-lint:detail
        print("     → 怎么办：给一个命令和项目根；changed-together 另从标准输入读这次改动碰到的路径。", file=sys.stderr)
        return 2
    command, root = arguments
    try:
        os.chdir(root)
    except OSError as error:
        print(f"  ✗ 进不去项目根 {root}：{error}")   # gate-lint:summary
        print("     → 怎么办：第二个参数给项目根（仓的顶层目录）。")
        return 2
    if command == "files":
        return judge_files()
    if command == "structure":
        return judge_structure()
    if command == "glossary":
        return judge_glossary()
    return judge_changed_together([line.strip() for line in sys.stdin.read().split("\n") if line.strip()])


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
