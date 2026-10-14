#!/usr/bin/env python3
# admission: always 每一次调都按此刻的正文与 kb 现文生成材料，上一次的结论不替这一次作保
# run-condition: command git
"""三方论证的材料：按正文引到的 kb 文件生成小节清单（默认全抄，索引表与历史节不抄）、抽附录、拼背景材料、写开工快照。

用法：
    three-way-materials.py <轮名> [--root 仓根] [--files kb文件 …] [--extra 文件:a-b …] [--no-history]
    three-way-materials.py <轮名> --resume [同样的参数]   # checklist-specs.py 判红、改好清单之后接着做
    three-way-materials.py --selftest      # THREE_WAY_MATERIALS_BREAK=skip-index-extra 时必须判红

它替掉的是材料员（three-way-materials）那份活：2026-09-20 起 8 天里 78 个材料员、每个 83 次 Bash、19 分钟在关键路径上，做的就是这条脚本链
（kb-sections.py 生成清单 → 逐行标抄 / 不抄 → checklist-specs.py 抽附录 → 拼背景材料 → 写快照）。它自己的判断只有清单上的抄 / 不抄，
而 `.claude/rules/three-way-inference.md` 要求整节抄、不摘句，默认全抄与规则同向。主 agent 要标例外，改清单再跑 checklist-specs.py。

清单怎么标（按 kb-sections.py 的行）：
- 开篇正文的行与文件顶上的标题：抄（checklist-specs.py 把两者合成一个行区间，不把整个文件抄进来）；
- 顶上标题的直接子节：抄，按自己的标题取；再往下的子节：抄、理由以「随」起头，由父节带到；
- `### 已定项` / `### 未定项` 的分项索引表：不抄，行区间（标题下一行到第一个 `####` 之前）作 --extra 按行取；它下面的 `####` 分项各按自己的标题取；
- `## 历史版本` 及其子节：不抄（正文只引现状；--no-history 之外的默认）。
读的文件：正文里写出的 `.claude/kb/…md` 路径、D / E 编号对应的决策与实验页、C 编号对应的欠账表、I-<n>.<m>（与类号 I-<n>）所在的不变量类文件
（`.claude/kb-directories` 登记的目录里含 `| I-<n>.` 表行的那几份，类号找不到文件的退回那个目录的 README.md），加 --files 给的；
认编号与 I 号指到哪份文件的读法都是 quote-kb.py 的 cited_kb_sources(…, loose=True)（I 号经它的 invariant_files_for），与 quote-kb.py --cited 判清单全不全是同一个函数（这里用宽读法，认出的是 --cited 的超集）；
正文里 `<!-- doc-lint:not-numbers 记号 … -->` 声明的记号不当编号（代码围栏里的声明不算）。
产出（都排他新建，已存在就停）：research/prompts/_<轮>-checklist.md、_<轮>-appendix.md、_<轮>-background.md、<轮>-snapshot/kb-sha256.txt。
--resume：清单已在、另三样都还没有时，沿用那份清单（人改过的照用），正文或 --files 引到而清单里没有的文件各补一张（默认全抄），
  再跑 checklist-specs.py 抽附录、拼背景材料、写快照。checklist-specs.py 判红时，说明里给的就是这一条能照抄的 --resume 命令。
退出码：0 做成；1 checklist-specs.py 或 quote-kb.py 判红（原样转打它们的输出，按那里的下一步改清单，再照抄给出的 --resume 命令）；2 用法错、正文不在、产出已存在。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句的开头，括号里是那条「✗ 自检：」的开头）：
  THREE_WAY_MATERIALS_BREAK=skip-index-extra  mark_checklist() 把分项索引表标「不抄」却不把它的行区间交给 --extra
      附录（「✗ 自检：附录应当含索引表行、分项正文、细节、别的一节与开篇，不含历史节」）、
      背景材料（「✗ 自检：背景材料应当含正文与附录，快照应当列那份 kb 文件」）
  THREE_WAY_MATERIALS_BREAK=ignore-not-numbers  cited_files() 不认正文的 `<!-- doc-lint:not-numbers … -->`，声明过的 E1 照当实验号拉
      not-numbers（「✗ 自检：not-numbers 声明了 E1」）
  THREE_WAY_MATERIALS_BREAK=skip-declared-kind  cited_files() 正文声明了哪个字母的记号，就把这个字母的编号全跳过（没声明的 E165 也不拉）
      not-numbers（「✗ 自检：not-numbers 声明了 E1」）、没声明的照拉（「✗ 自检：没声明的 E165 照拉」）、
      两边同一份读法（「✗ 自检：材料拉的文件不是 --cited 要的超集」）
  THREE_WAY_MATERIALS_BREAK=own-number-reading  cited_files() 不调 quote-kb.py 的 cited_kb_sources，走回改前自己抄的一份读法（认不出 I-7.4）
      不变量号（「✗ 自检：正文点名 I-7.4」）、两边同一份读法（「✗ 自检：材料拉的文件不是 --cited 要的超集」）、
      续做（「✗ 自检：checklist-specs.py 判红之后照抄给出的 --resume」：样本正文也点名了 I-7.4）
  THREE_WAY_MATERIALS_BREAK=no-resume  build() 不认 --resume，checklist-specs.py 判红时只给改前那条只补附录的命令
      续做（「✗ 自检：checklist-specs.py 判红之后照抄给出的 --resume」）
  QUOTE_KB_BREAK=invariants-to-readme  quote-kb.py 的开关（I 号一律退回不变量目录的 README.md），读法是那一份，这里同样红
      不变量号（「✗ 自检：正文点名 I-7.4」）、续做（「✗ 自检：checklist-specs.py 判红之后照抄给出的 --resume」：样本正文也点名了 I-7.4）
"""
import contextlib
import glob
import hashlib
import importlib.util
import io
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
BROKEN = os.environ.get("THREE_WAY_MATERIALS_BREAK", "")
KB_PATH = re.compile(r"\.claude/kb/[A-Za-z0-9_./\-一-鿿]+\.md")
NUMBER = re.compile(r"(?<![A-Za-z0-9])([DECI])(\d+)(?![0-9])")
ROW = re.compile(r"^\|\s*(.+?)\s*\|\s*(.*?)\s*\|\s*(.*?)\s*\|$")   # kb-sections.py 的空单元格写成「| |」，格与格之间只有一个空格
HEADING = re.compile(r"^(#{1,6}) ")
INDEX_TABLE_TITLES = ("### 已定项", "### 未定项")
HISTORY_TITLE = "历史版本"


def quote_kb_module():
    """quote-kb.py 当模块读进来：认编号（cited_kb_sources）与 not-numbers 声明（not_numbers_declared）的读法只在那里一份。"""
    spec = importlib.util.spec_from_file_location("quote_kb", os.path.join(SCRIPT_DIRECTORY, "..", "kb", "quote-kb.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def not_numbers_declared(body_text):
    """正文里 `<!-- doc-lint:not-numbers … -->` 声明的记号；读法只在 quote-kb.py 的 not_numbers_declared 一处（它的 --cited 用同一份）。"""
    return quote_kb_module().not_numbers_declared(body_text)


def own_number_reading(body_text, root):
    """弄坏开关 own-number-reading 才走：改前这里自己抄的一份读法（I 编号只认 I<n>，认不出 I-7.4）；I 号指到哪份文件照 quote-kb.py 的 invariant_files_for。"""
    found = list(KB_PATH.findall(body_text))
    declared = not_numbers_declared(body_text)
    quote_kb = quote_kb_module()
    for kind, number in NUMBER.findall(body_text):
        if f"{kind}{number}" in declared:
            continue
        number = int(number)
        if kind == "D":
            found += sorted(glob.glob(os.path.join(root, ".claude", "kb", "decisions", f"{number:02d}-*.md")))
        elif kind == "E":
            found += sorted(glob.glob(os.path.join(root, ".claude", "kb", "experiments", f"{number}-*.md")))
        elif kind == "C":
            found.append(".claude/kb/checks-owed.md")
        elif kind == "I":
            found += quote_kb.invariant_files_for(f"I-{number}", root)[0]
    return [(path, path) for path in found]


def cited_files(body_text, root):
    """正文里引到的 kb 文件，显式路径在前、编号在后，按出现次序去重；不在的不列。
    读法是 quote-kb.py 的 cited_kb_sources(…, loose=True)：与 --cited 判清单全不全（严格读法）同一份函数，宽读法认出的是超集。
    `doc-lint:not-numbers` 声明的记号不当编号。"""
    if BROKEN == "own-number-reading":
        sources = own_number_reading(body_text, root)
    else:
        quote_kb = quote_kb_module()
        # 弄坏开关 ignore-not-numbers：把正文里的 not-numbers 声明抹掉再读，声明过的记号照当编号
        read_text = quote_kb.NOT_NUMBERS_MARK.sub("", body_text) if BROKEN == "ignore-not-numbers" else body_text
        sources = quote_kb.cited_kb_sources(read_text, root, loose=True)
        if BROKEN == "skip-declared-kind":   # 弄坏开关：声明了哪个字母的记号，这个字母的编号全跳过
            kinds = {token[0] for token in not_numbers_declared(body_text)}
            sources = [(path, token) for path, token in sources if token == path or token[0] not in kinds]
    ordered = []
    for path, _token in sources:
        relative = os.path.relpath(path, root) if os.path.isabs(path) else path
        if relative not in ordered and os.path.isfile(os.path.join(root, relative)):
            ordered.append(relative)
    return ordered


def index_table_range(lines, heading_text):
    """`### 已定项` 这类索引表的行区间：标题下一行起，到第一个 #### 之前（1 起数）。找不到返回 None。"""
    for index, line in enumerate(lines):
        if line.strip() == heading_text:
            start = index + 2
            end = len(lines)
            for later in range(index + 1, len(lines)):
                if lines[later].startswith("#"):
                    end = later
                    break
            return (start, end) if end >= start else None
    return None


def mark_checklist(section_text, file_lines, keep_history):
    """给 kb-sections.py 的一份清单填抄 / 不抄与理由；返回 (填好的文本, --extra 取法列表, 抄的行数, 不抄的行数)。"""
    output, extras, copied, skipped = [], [], 0, 0
    stack = []   # [(级别, 种类)]，种类 ∈ copy / follow / skip-index / skip-history
    top_level = None
    for line in section_text.split("\n"):
        matched = ROW.match(line)
        if not matched or matched.group(1) in ("小节", "---"):
            output.append(line)
            continue
        title = matched.group(1)
        heading = HEADING.match(title)
        if not heading:   # 开篇正文那一行
            output.append(f"| {title} | 抄 | 开篇正文，与顶上标题合成行区间 |")
            copied += 1
            continue
        level = len(heading.group(1))
        if top_level is None:
            top_level = level
        while stack and stack[-1][0] >= level:
            stack.pop()
        parent = stack[-1] if stack else None
        bare = title.strip()
        if HISTORY_TITLE in bare and not keep_history:
            kind, mark, reason = "skip-history", "不抄", "历史节，正文只引现状"
        elif parent and parent[1] == "skip-history":
            kind, mark, reason = "skip-history", "不抄", "历史节的子节"
        elif bare in INDEX_TABLE_TITLES:
            kind, mark, reason = "skip-index", "不抄", "分项索引表，用 --extra 按行区间取"
            span = index_table_range(file_lines, bare)
            if span and BROKEN != "skip-index-extra":
                extras.append(span)
        elif level == top_level:
            kind, mark, reason = "copy", "抄", "顶上标题，与开篇正文合成行区间"
        elif parent is None or parent[0] == top_level or parent[1] == "skip-index":
            kind, mark, reason = "copy", "抄", "默认全抄（脚本生成）"
        else:
            kind, mark, reason = "follow", "抄", "随父节带到"
        stack.append((level, kind))
        output.append(f"| {title} | {mark} | {reason} |")
        if mark == "抄":
            copied += 1
        else:
            skipped += 1
    return "\n".join(output), extras, copied, skipped


def sha256_of(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


CHECKLIST_SECTION = re.compile(r"^### 小节清单：`([^`]+)`", re.M)


def resume_command(round_name, root, extra_files, extra_ranges, keep_history):
    """续做那一条能照抄的命令：同样的参数加 --resume。"""
    script = os.path.join(SCRIPT_DIRECTORY, "three-way-materials.py")
    shown = os.path.relpath(script)
    words = ["python3", shown if not shown.startswith("..") else script, round_name, "--resume"]
    if os.path.realpath(root) != os.path.realpath(REPOSITORY_ROOT):
        words += ["--root", root]
    if extra_files:
        words += ["--files", *extra_files]
    if extra_ranges:
        words += ["--extra", *extra_ranges]
    if keep_history:
        words.append("--no-history")
    return " ".join(shlex.quote(word) for word in words)


def counted_marks(checklist_text):
    """清单里标「抄」与「不抄」的行数（续做时按清单现文数）。"""
    marks = [ROW.match(line).group(2) for line in checklist_text.split("\n") if ROW.match(line) and ROW.match(line).group(1) not in ("小节", "---")]
    return sum(1 for mark in marks if mark == "抄"), sum(1 for mark in marks if mark == "不抄")


def build(round_name, root, extra_files, extra_ranges, keep_history, quiet=False, resume=False):
    prompts = os.path.join(root, "research", "prompts")
    body_path = os.path.join(prompts, f"_{round_name}-body.md")
    if not os.path.isfile(body_path):
        return 2, [f"  ✗ 正文不在：{os.path.relpath(body_path, root)}", "  → 怎么办：先写正文 research/prompts/_<轮>-body.md，再跑这条脚本"]
    body_text = open(body_path, encoding="utf-8").read()
    files = cited_files(body_text, root)
    for path in extra_files:
        relative = os.path.relpath(path, root) if os.path.isabs(path) else path
        if relative not in files and os.path.isfile(os.path.join(root, relative)):
            files.append(relative)
    outputs = {name: os.path.join(prompts, f"_{round_name}-{name}.md") for name in ("checklist", "appendix", "background")}
    snapshot_path = os.path.join(prompts, f"{round_name}-snapshot", "kb-sha256.txt")
    resume = resume and BROKEN != "no-resume"
    continue_with = resume_command(round_name, root, extra_files, extra_ranges, keep_history)
    existing_text = None
    if resume:
        if not os.path.isfile(outputs["checklist"]):
            return 2, [f"  ✗ --resume 要接着做的清单不在：{os.path.relpath(outputs['checklist'], root)}",
                       "  → 怎么办：这一轮还没生成过清单，去掉 --resume 从头跑"]
        later = [path for path in (outputs["appendix"], outputs["background"], snapshot_path) if os.path.exists(path)]
        if later:
            return 2, [f"  ✗ 产出已存在：{'、'.join(os.path.relpath(path, root) for path in later)}（--resume 只接着做附录、背景材料与快照，这几样要还没有）",
                       "  → 怎么办：这一轮的材料已经做成过；要重拼就先把它们挪到草稿目录（清单留着）再跑同一条 --resume（原样保存的证据不许覆盖）"]
        existing_text = open(outputs["checklist"], encoding="utf-8").read()
        for listed in CHECKLIST_SECTION.findall(existing_text):
            if listed not in files and os.path.isfile(os.path.join(root, listed)):
                files.append(listed)
    else:
        existing = [path for path in [*outputs.values(), snapshot_path] if os.path.exists(path)]
        if existing == [outputs["checklist"]]:
            return 2, [f"  ✗ 产出已存在：{os.path.relpath(outputs['checklist'], root)}（上一次停在 checklist-specs.py 判红，只留下了清单）",
                       f"  → 怎么办：改好清单之后接着做（附录、背景材料与 kb-sha256.txt 一起补齐，清单不重生成）：{continue_with}",
                       "    要从头重生成清单，先把它挪到草稿目录再跑同一条命令"]
        if existing:
            return 2, [f"  ✗ 产出已存在：{'、'.join(os.path.relpath(path, root) for path in existing)}",
                       "  → 怎么办：这一轮的材料已经生成过；要重拼就先把它们挪到草稿目录再跑（原样保存的证据不许覆盖）"]
    if not files:
        return 2, ["  ✗ 正文里一个 kb 文件、一个 D / E / C / I-<n>.<m> 编号都没引到，也没给 --files", "  → 怎么办：正文里写出要引的 kb 路径或编号，或者用 --files 点名"]
    listed_already = set(CHECKLIST_SECTION.findall(existing_text)) if existing_text is not None else set()
    checklist_parts, extras, copied, skipped, appended = [], list(extra_ranges), 0, 0, []
    for relative in files:
        result = subprocess.run([sys.executable, os.path.join(SCRIPT_DIRECTORY, "..", "kb", "kb-sections.py"), relative], cwd=root, capture_output=True, text=True)
        if result.returncode != 0:
            return 1, [f"  ✗ kb-sections.py 对 {relative} 判红（退出码 {result.returncode}）：", result.stdout[-800:], result.stderr[-400:],
                       "  → 怎么办：按它给的下一步改那份 kb 文件或把它从 --files 里去掉"]
        file_lines = open(os.path.join(root, relative), encoding="utf-8").read().split("\n")
        marked, spans, copied_here, skipped_here = mark_checklist(result.stdout, file_lines, keep_history)
        extras += [f"{relative}:{start}-{end}" for start, end in spans]
        if relative in listed_already:
            continue
        checklist_parts.append(marked.rstrip("\n"))
        appended.append(relative)
        copied += copied_here
        skipped += skipped_here
    if existing_text is None:
        checklist_text = "\n\n".join(checklist_parts) + "\n"
        with open(outputs["checklist"], "x", encoding="utf-8") as handle:
            handle.write(checklist_text)
    else:
        if checklist_parts:   # 清单里缺的几张（正文后来引到的、或被删掉的）补到末尾，默认全抄
            with open(outputs["checklist"], "a", encoding="utf-8") as handle:
                handle.write(("" if existing_text.endswith("\n") else "\n") + "\n" + "\n\n".join(checklist_parts) + "\n")
        checklist_text = open(outputs["checklist"], encoding="utf-8").read()
        copied, skipped = counted_marks(checklist_text)
    command = [sys.executable, os.path.join(SCRIPT_DIRECTORY, "checklist-specs.py"), os.path.relpath(outputs["checklist"], root),
               "--cited", os.path.relpath(body_path, root), "--out", os.path.relpath(outputs["appendix"], root)]
    for extra in extras:
        command += ["--extra", extra]
    result = subprocess.run(command, cwd=root, capture_output=True, text=True)
    if result.returncode != 0:
        lines = [f"  ✗ checklist-specs.py 判红（退出码 {result.returncode}）：", result.stdout[-1200:], result.stderr[-600:]]
        if BROKEN == "no-resume":
            return 1, lines + [f"  → 怎么办：按上面它给的下一步改清单 {os.path.relpath(outputs['checklist'], root)}，再手动跑同一条命令：{' '.join(command)}"]
        return 1, lines + [f"  → 怎么办：按上面它给的下一步改清单 {os.path.relpath(outputs['checklist'], root)}（它说缺清单的，那几张这条命令会自己补上），"
                           f"改完照抄这一条接着做，附录、背景材料与 kb-sha256.txt 一起补齐、清单不重生成：",
                           f"      {continue_with}"]
    appendix_text = open(outputs["appendix"], encoding="utf-8").read()
    with open(outputs["background"], "x", encoding="utf-8") as handle:
        handle.write(body_text.rstrip("\n") + "\n\n" + checklist_text.rstrip("\n") + "\n\n" + appendix_text)
    os.makedirs(os.path.dirname(snapshot_path), exist_ok=True)
    with open(snapshot_path, "x", encoding="utf-8") as handle:
        for relative in files:
            handle.write(f"{sha256_of(os.path.join(root, relative))}  {relative}\n")
    resumed = (f"（--resume 接着做：沿用已有的清单" + (f"，补了 {len(appended)} 张：{'、'.join(appended)}" if appended else "") + "）") if existing_text is not None else ""
    lines = [f"  ✓ 材料做成：读了 {len(files)} 份 kb 文件，清单抄 {copied} 行、不抄 {skipped} 行，--extra {len(extras)} 条{resumed}",
             *(f"    {os.path.relpath(path, root)}  sha256 {sha256_of(path)}" for path in [outputs['checklist'], outputs['appendix'], outputs['background'], snapshot_path]),
             f"    checklist-specs.py：{result.stdout.strip().splitlines()[-1] if result.stdout.strip() else '（无输出）'}"]
    return 0, lines


def selftest():
    work = tempfile.mkdtemp(prefix="three-way-materials-selftest-")
    failures, checked = [], 0
    try:
        subprocess.run(["git", "init", "-q", work], check=True)
        os.makedirs(os.path.join(work, ".claude", "kb", "decisions"))
        os.makedirs(os.path.join(work, "research", "prompts"))
        decision = "\n".join([
            "## D7 样本 —— 已定", "", "开篇第一句。开篇第二句。", "", "### 已定项", "", "| # | 一句话 |", "|---|---|",
            "| 1 | 索引表第一行 INDEXROW |", "", "#### 已定项 1：第一项", "", "**定案**：第一项正文 BODYONE。", "",
            "##### 第一项的细节", "", "细节正文 DETAIL。", "", "### 别的一节", "", "别的正文 OTHER。", "",
            "## 历史版本", "", "### 2026-09-01", "- 曾经的事 HISTORY。", ""])
        open(os.path.join(work, ".claude", "kb", "decisions", "07-样本.md"), "w", encoding="utf-8").write(decision)
        open(os.path.join(work, "research", "prompts", "_t-r1-body.md"), "w", encoding="utf-8").write(
            "# t-r1 正文\n\n判 D7（样本） 已定项 1 站不站得住，见 `.claude/kb/decisions/07-样本.md`。\n")
        code, lines = build("t-r1", work, [], [], keep_history=False, quiet=True)
        checked += 1
        if code != 0:
            failures.append(f"样本仓上应当做成（退出码 0），实际 {code}：\n" + "\n".join(lines))
        else:
            checklist = open(os.path.join(work, "research", "prompts", "_t-r1-checklist.md"), encoding="utf-8").read()
            appendix = open(os.path.join(work, "research", "prompts", "_t-r1-appendix.md"), encoding="utf-8").read()
            background = open(os.path.join(work, "research", "prompts", "_t-r1-background.md"), encoding="utf-8").read()
            snapshot = open(os.path.join(work, "research", "prompts", "t-r1-snapshot", "kb-sha256.txt"), encoding="utf-8").read()
            wanted_marks = ["| ### 已定项 | 不抄 | 分项索引表", "| #### 已定项 1：第一项 | 抄 | 默认全抄", "| ##### 第一项的细节 | 抄 | 随父节带到",
                            "| ## 历史版本 | 不抄 | 历史节", "| ### 2026-09-01 | 不抄 | 历史节的子节", "| ## D7 样本 —— 已定 | 抄 | 顶上标题"]
            missing = [piece for piece in wanted_marks if piece not in checklist]
            checked += 1
            if missing:
                failures.append(f"清单标法缺 {missing}：\n{checklist}")
            checked += 1
            if not all(piece in appendix for piece in ("INDEXROW", "BODYONE", "DETAIL", "OTHER", "开篇第一句")) or "HISTORY" in appendix:
                failures.append(f"附录应当含索引表行、分项正文、细节、别的一节与开篇，不含历史节：\n{appendix}")
            checked += 1
            if "t-r1 正文" not in background or "INDEXROW" not in background or "07-样本.md" not in snapshot:
                failures.append("背景材料应当含正文与附录，快照应当列那份 kb 文件")
            checked += 1
            code_again, lines_again = build("t-r1", work, [], [], keep_history=False, quiet=True)
            if code_again != 2 or "产出已存在" not in "\n".join(lines_again):
                failures.append(f"产出已存在时应当退 2 并说明，实际 {code_again}：{lines_again[:1]}")
        # 正文用 doc-lint:not-numbers 声明了 E1（txg 顶格那一域，不是实验号）：不拉 E1 的实验页；没声明的 E165 照拉
        os.makedirs(os.path.join(work, ".claude", "kb", "experiments"))
        for name in ("1-节点大小vs可恢复性.md", "165-恢复重放有界穷举.md"):
            open(os.path.join(work, ".claude", "kb", "experiments", name), "w", encoding="utf-8").write(f"## {name[:-3]} —— 样本\n\n开篇 {name[:3]}。\n")
        open(os.path.join(work, "research", "prompts", "_t-r2-body.md"), "w", encoding="utf-8").write(
            "# t-r2 正文\n\n<!-- doc-lint:not-numbers E1 -->\n\n判 E1（txg 顶格那一域） 站不站得住，压在 E165（恢复重放有界穷举） 上。\n")
        code_declared, lines_declared = build("t-r2", work, [], [], keep_history=False, quiet=True)
        snapshot_path = os.path.join(work, "research", "prompts", "t-r2-snapshot", "kb-sha256.txt")
        snapshot = open(snapshot_path, encoding="utf-8").read() if os.path.isfile(snapshot_path) else ""
        checked += 1
        if code_declared != 0 or "1-节点大小vs可恢复性.md" in snapshot:
            failures.append(f"not-numbers 声明了 E1：应当做成（退出码 0）且不拉 E1 的实验页，实际 {code_declared}，快照：{snapshot!r}；{' '.join(lines_declared)[-300:]}")
        checked += 1
        if code_declared != 0 or "165-恢复重放有界穷举.md" not in snapshot:
            failures.append(f"没声明的 E165 照拉：应当做成（退出码 0）且快照列 E165 的实验页，实际 {code_declared}，快照：{snapshot!r}；{' '.join(lines_declared)[-300:]}")
        # 不变量清单按类拆在登记的 kb 目录里（README.md 加类文件，形态照 crates/singlefs-checker/invariants/）：
        # 正文只点名 I-7.4 与 I-8.3，清单里自动带上那两份类文件、不带 README.md，四样产出齐
        invariants_directory = os.path.join("crates", "sample", "invariants")
        os.makedirs(os.path.join(work, invariants_directory))
        open(os.path.join(work, ".claude", "kb-directories"), "w", encoding="utf-8").write(
            f".claude/kb  # 样本：决策、实验仍在这里\n{invariants_directory}  # 样本：不变量按类拆在这里\n")
        for name, text in (("README.md", "# 不变量清单\n\n开篇 INVARIANTHEAD。\n\n## 哪类放哪\n\n| 文件 | 类 |\n|---|---|\n| 02-根环.md | I-7 |\n| 03-journal.md | I-8 |\n"),
                           ("02-根环.md", "## I-7 根环 —— 类\n\n开篇 ROOTRING。\n\n| 编号 | 简称 | 陈述 |\n|---|---|---|\n| I-7.4 | 样本 | 正文 INVARIANT。 |\n"),
                           ("03-journal.md", "## I-8 journal —— 类\n\n开篇 JOURNALHEAD。\n\n| 编号 | 简称 | 陈述 |\n|---|---|---|\n| I-8.3 | 样本 | 正文 JOURNALROW。 |\n")):
            open(os.path.join(work, invariants_directory, name), "w", encoding="utf-8").write(text)
        root_ring_file = f"{invariants_directory}/02-根环.md"
        open(os.path.join(work, "research", "prompts", "_t-r4-body.md"), "w", encoding="utf-8").write(
            "# t-r4 正文\n\n判 I-7.4（样本不变量） 与 I-8.3（样本不变量） 站不站得住。\n")
        code_invariant, lines_invariant = build("t-r4", work, [], [], keep_history=False, quiet=True)
        produced = [os.path.join(work, "research", "prompts", name) for name in
                    ("_t-r4-checklist.md", "_t-r4-appendix.md", "_t-r4-background.md", os.path.join("t-r4-snapshot", "kb-sha256.txt"))]
        snapshot = open(produced[3], encoding="utf-8").read() if os.path.isfile(produced[3]) else ""
        appendix = open(produced[1], encoding="utf-8").read() if os.path.isfile(produced[1]) else ""
        snapshot_files = sorted(line.split("  ", 1)[1] for line in snapshot.split("\n") if "  " in line)
        checked += 1
        if code_invariant != 0 or not all(os.path.isfile(path) for path in produced) \
                or snapshot_files != [root_ring_file, f"{invariants_directory}/03-journal.md"] \
                or "INVARIANT。" not in appendix or "JOURNALROW" not in appendix:
            failures.append(f"正文点名 I-7.4 与 I-8.3：应当做成（退出码 0）、四样产出齐、快照只列 02-根环.md 与 03-journal.md 两份类文件（不带 README.md）、"
                            f"附录含两份的表行，实际 {code_invariant}，快照：{snapshot!r}；{' '.join(lines_invariant)[-400:]}")
        # 两边同一份读法：同一段正文，这里拉的文件是 quote-kb.py --cited 要的（在的那几份）的超集
        quote_kb = quote_kb_module()
        bodies = [open(os.path.join(work, "research", "prompts", f"_t-r{index}-body.md"), encoding="utf-8").read() for index in (1, 2, 4)]
        bodies.append("压在 I-7.4（样本不变量） 与 D7（样本） 上，见 C1（欠账） 与 E165（样本）。\n")
        uncovered = []
        for body in bodies:
            wanted = {path for path in quote_kb.cited_kb_files(body, work) if os.path.isfile(os.path.join(work, path))}
            missing = sorted(wanted - set(cited_files(body, work)))
            if missing:
                uncovered.append(f"{body.strip()[:40]!r} 缺 {missing}")
        checked += 1
        if uncovered:
            failures.append(f"材料拉的文件不是 --cited 要的超集（两边读法不是同一份）：{'；'.join(uncovered)}")
        # 续做：kb 文件里有两个同名的「### 细节」，默认全抄时 checklist-specs.py 判红（标题命中 2 次）；把它们改成「不抄」、
        # 再删掉清单里 02-根环.md 那一张，照抄判红说明里给的 --resume 命令跑一次：缺的那张补上，附录、背景材料、快照齐
        open(os.path.join(work, ".claude", "kb", "decisions", "08-重名.md"), "w", encoding="utf-8").write(
            "## D8 重名 —— 已定\n\n开篇 DUPHEAD。\n\n### 细节\n\n甲的细节 DUPA。\n\n### 细节\n\n乙的细节 DUPB。\n\n### 别的\n\n别的正文 DUPOTHER。\n")
        open(os.path.join(work, "research", "prompts", "_t-r5-body.md"), "w", encoding="utf-8").write(
            "# t-r5 正文\n\n判 D8（重名） 站不站得住，压在 I-7.4（样本不变量） 上。\n")
        code_first, lines_first = build("t-r5", work, [], [], keep_history=False, quiet=True)
        checklist_path = os.path.join(work, "research", "prompts", "_t-r5-checklist.md")
        given = [line.strip() for line in lines_first if "--resume" in line and line.strip().startswith("python3 ")]
        resumed_code, resumed_output = None, ""
        if code_first == 1 and given and os.path.isfile(checklist_path):
            text = open(checklist_path, encoding="utf-8").read()
            text = text.replace("| ### 细节 | 抄 | 默认全抄（脚本生成） |", "| ### 细节 | 不抄 | 重名，样本里不抄 |")
            text = text.split(f"\n\n### 小节清单：`{root_ring_file}`")[0] + "\n"
            open(checklist_path, "w", encoding="utf-8").write(text)
            words = shlex.split(given[0])
            captured = io.StringIO()
            with contextlib.redirect_stdout(captured):
                resumed_code = main(words[2:])
            resumed_output = captured.getvalue()
        produced = [os.path.join(work, "research", "prompts", name) for name in
                    ("_t-r5-appendix.md", "_t-r5-background.md", os.path.join("t-r5-snapshot", "kb-sha256.txt"))]
        appendix = open(produced[0], encoding="utf-8").read() if os.path.isfile(produced[0]) else ""
        background = open(produced[1], encoding="utf-8").read() if os.path.isfile(produced[1]) else ""
        snapshot = open(produced[2], encoding="utf-8").read() if os.path.isfile(produced[2]) else ""
        checklist = open(checklist_path, encoding="utf-8").read() if os.path.isfile(checklist_path) else ""
        checked += 1
        if not (code_first == 1 and given and resumed_code == 0 and f"补了 1 张：{root_ring_file}" in resumed_output
                and f"### 小节清单：`{root_ring_file}`" in checklist and "DUPOTHER" in appendix and "INVARIANT。" in appendix
                and "DUPA" not in appendix and "t-r5 正文" in background and "INVARIANT。" in background
                and "08-重名.md" in snapshot and root_ring_file in snapshot):
            failures.append(f"checklist-specs.py 判红之后照抄给出的 --resume 命令跑一次，应当补上缺的那张清单、附录、背景材料与快照一起齐；"
                            f"第一次退 {code_first}、给的命令 {given}、续做退 {resumed_code}：{resumed_output[-300:]}；{' '.join(lines_first)[-300:]}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")   # gate-lint:detail
    if failures:
        print("    → 怎么办：看 mark_checklist()、cited_files() 与 build()；THREE_WAY_MATERIALS_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {checked} 项）：索引表不抄而按行区间补抄、分项与细节按自己的标题或随父节抄、历史节不抄、"
          "附录与背景材料含该抄的每一段、快照列了 kb 文件、产出已存在时拒绝覆盖、not-numbers 声明的记号不当编号而没声明的照拉、"
          "只点名 I-7.4 与 I-8.3 时带上那两份不变量类文件、认编号与 quote-kb.py --cited 同一份读法、checklist-specs.py 判红后照抄 --resume 一次补齐")
    return 0


def main(argv):
    if argv[:1] == ["--selftest"]:
        return selftest()
    if not argv or argv[0].startswith("--"):
        print("  ✗ 用法：three-way-materials.py <轮名> [--root 仓根] [--files kb文件 …] [--extra 文件:a-b …] [--no-history] [--resume]\n"
              "  → 怎么办：第一个参数写轮名（正文是 research/prompts/_<轮名>-body.md）")
        return 2
    round_name, root, files, extras, keep_history, resume = argv[0], REPOSITORY_ROOT, [], [], False, False
    rest = argv[1:]
    while rest:
        flag = rest.pop(0)
        if flag == "--root" and rest:
            root = os.path.abspath(rest.pop(0))
        elif flag == "--files":
            while rest and not rest[0].startswith("--"):
                files.append(rest.pop(0))
        elif flag == "--extra":
            while rest and not rest[0].startswith("--"):
                extras.append(rest.pop(0))
        elif flag == "--no-history":
            keep_history = True
        elif flag == "--resume":
            resume = True
        else:
            print(f"  ✗ 认不出参数 {flag}\n  → 怎么办：只认 --root、--files、--extra、--no-history、--resume")
            return 2
    code, lines = build(round_name, root, files, extras, keep_history, resume=resume)
    print("\n".join(lines))
    return code


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
