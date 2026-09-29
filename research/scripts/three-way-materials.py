#!/usr/bin/env python3
# admission: always 每一次调都按此刻的正文与 kb 现文生成材料，上一次的结论不替这一次作保
# run-condition: command git
"""三方论证的材料：按正文引到的 kb 文件生成小节清单（默认全抄，索引表与历史节不抄）、抽附录、拼背景材料、写开工快照。

用法：
    three-way-materials.py <轮名> [--root 仓根] [--files kb文件 …] [--extra 文件:a-b …] [--no-history]
    three-way-materials.py --selftest      # THREE_WAY_MATERIALS_BREAK=skip-index-extra 时必须判红

它替掉的是材料员（three-way-materials）那份活：2026-09-20 起 8 天里 78 个材料员、每个 83 次 Bash、19 分钟在关键路径上，做的就是这条脚本链
（kb-sections.py 生成清单 → 逐行标抄 / 不抄 → checklist-specs.py 抽附录 → 拼背景材料 → 写快照）。它自己的判断只有清单上的抄 / 不抄，
而 `.claude/rules/three-way-inference.md` 要求整节抄、不摘句，默认全抄与规则同向。主 agent 要标例外，改清单再跑 checklist-specs.py。

清单怎么标（按 kb-sections.py 的行）：
- 开篇正文的行与文件顶上的标题：抄（checklist-specs.py 把两者合成一个行区间，不把整个文件抄进来）；
- 顶上标题的直接子节：抄，按自己的标题取；再往下的子节：抄、理由以「随」起头，由父节带到；
- `### 已定项` / `### 未定项` 的分项索引表：不抄，行区间（标题下一行到第一个 `####` 之前）作 --extra 按行取；它下面的 `####` 分项各按自己的标题取；
- `## 历史版本` 及其子节：不抄（正文只引现状；--no-history 之外的默认）。
读的文件：正文里写出的 `.claude/kb/…md` 路径、D / E 编号对应的决策与实验页、C 编号对应的欠账表、I 编号对应的不变量清单，加 --files 给的。
产出（都排他新建，已存在就停）：research/prompts/_<轮>-checklist.md、_<轮>-appendix.md、_<轮>-background.md、<轮>-snapshot/kb-sha256.txt。
退出码：0 做成；1 checklist-specs.py 或 quote-kb.py 判红（原样转打它们的输出，按那里的下一步改）；2 用法错、正文不在、产出已存在。
"""
import glob
import hashlib
import os
import re
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


def cited_files(body_text, root):
    """正文里引到的 kb 文件，按出现次序去重；不在的不列。"""
    found = []
    for path in KB_PATH.findall(body_text):
        found.append(path)
    for kind, number in NUMBER.findall(body_text):
        number = int(number)
        if kind == "D":
            found += sorted(glob.glob(os.path.join(root, ".claude", "kb", "decisions", f"{number:02d}-*.md")))
        elif kind == "E":
            found += sorted(glob.glob(os.path.join(root, ".claude", "kb", "experiments", f"{number}-*.md")))
            found += sorted(glob.glob(os.path.join(root, ".claude", "kb", "experiments", f"{number:02d}-*.md")))
        elif kind == "C":
            found.append(".claude/kb/checks-owed.md")
        elif kind == "I":
            found.append(".claude/kb/invariants.md")
    ordered = []
    for path in found:
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


def build(round_name, root, extra_files, extra_ranges, keep_history, quiet=False):
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
    if not files:
        return 2, ["  ✗ 正文里一个 kb 文件、一个 D / E / C / I 编号都没引到，也没给 --files", "  → 怎么办：正文里写出要引的 kb 路径或编号，或者用 --files 点名"]
    outputs = {name: os.path.join(prompts, f"_{round_name}-{name}.md") for name in ("checklist", "appendix", "background")}
    snapshot_path = os.path.join(prompts, f"{round_name}-snapshot", "kb-sha256.txt")
    existing = [path for path in [*outputs.values(), snapshot_path] if os.path.exists(path)]
    if existing:
        return 2, [f"  ✗ 产出已存在：{'、'.join(os.path.relpath(path, root) for path in existing)}",
                   "  → 怎么办：这一轮的材料已经生成过；要重拼就先把它们挪到草稿目录再跑（原样保存的证据不许覆盖）"]
    checklist_parts, extras, copied, skipped = [], list(extra_ranges), 0, 0
    for relative in files:
        result = subprocess.run([sys.executable, os.path.join(SCRIPT_DIRECTORY, "kb-sections.py"), relative], cwd=root, capture_output=True, text=True)
        if result.returncode != 0:
            return 1, [f"  ✗ kb-sections.py 对 {relative} 判红（退出码 {result.returncode}）：", result.stdout[-800:], result.stderr[-400:],
                       "  → 怎么办：按它给的下一步改那份 kb 文件或把它从 --files 里去掉"]
        file_lines = open(os.path.join(root, relative), encoding="utf-8").read().split("\n")
        marked, spans, copied_here, skipped_here = mark_checklist(result.stdout, file_lines, keep_history)
        checklist_parts.append(marked.rstrip("\n"))
        extras += [f"{relative}:{start}-{end}" for start, end in spans]
        copied += copied_here
        skipped += skipped_here
    checklist_text = "\n\n".join(checklist_parts) + "\n"
    with open(outputs["checklist"], "x", encoding="utf-8") as handle:
        handle.write(checklist_text)
    command = [sys.executable, os.path.join(SCRIPT_DIRECTORY, "checklist-specs.py"), os.path.relpath(outputs["checklist"], root),
               "--cited", os.path.relpath(body_path, root), "--out", os.path.relpath(outputs["appendix"], root)]
    for extra in extras:
        command += ["--extra", extra]
    result = subprocess.run(command, cwd=root, capture_output=True, text=True)
    if result.returncode != 0:
        return 1, [f"  ✗ checklist-specs.py 判红（退出码 {result.returncode}）：", result.stdout[-1200:], result.stderr[-600:],
                   f"  → 怎么办：按上面它给的下一步改清单 {os.path.relpath(outputs['checklist'], root)}，再手动跑同一条命令：{' '.join(command)}"]
    appendix_text = open(outputs["appendix"], encoding="utf-8").read()
    with open(outputs["background"], "x", encoding="utf-8") as handle:
        handle.write(body_text.rstrip("\n") + "\n\n" + checklist_text.rstrip("\n") + "\n\n" + appendix_text)
    os.makedirs(os.path.dirname(snapshot_path), exist_ok=True)
    with open(snapshot_path, "x", encoding="utf-8") as handle:
        for relative in files:
            handle.write(f"{sha256_of(os.path.join(root, relative))}  {relative}\n")
    lines = [f"  ✓ 材料做成：读了 {len(files)} 份 kb 文件，清单抄 {copied} 行、不抄 {skipped} 行，--extra {len(extras)} 条",
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
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")   # gate-lint:detail
    if failures:
        print("    → 怎么办：看 mark_checklist() 与 build()；THREE_WAY_MATERIALS_BREAK=skip-index-extra 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {checked} 项）：索引表不抄而按行区间补抄、分项与细节按自己的标题或随父节抄、历史节不抄、"
          "附录与背景材料含该抄的每一段、快照列了 kb 文件、产出已存在时拒绝覆盖")
    return 0


def main(argv):
    if argv[:1] == ["--selftest"]:
        return selftest()
    if not argv or argv[0].startswith("--"):
        print("  ✗ 用法：three-way-materials.py <轮名> [--root 仓根] [--files kb文件 …] [--extra 文件:a-b …] [--no-history]\n"
              "  → 怎么办：第一个参数写轮名（正文是 research/prompts/_<轮名>-body.md）")
        return 2
    round_name, root, files, extras, keep_history = argv[0], REPOSITORY_ROOT, [], [], False
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
        else:
            print(f"  ✗ 认不出参数 {flag}\n  → 怎么办：只认 --root、--files、--extra、--no-history")
            return 2
    code, lines = build(round_name, root, files, extras, keep_history)
    print("\n".join(lines))
    return code


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
