#!/usr/bin/env python3
# admission: always 判的是此刻这份规格与它点名的 kb 文件此刻的样子，几秒跑完；每次派书记员之前都现判
# run-condition: none 只读规格文件与仓里的 kb，不碰别的环境
"""kb 写回规格检查：派书记员（kb-scribe）之前，规格原文本身会不会让门禁红。

用法：
    kb-spec-check.py <规格.json 或 规格.md> [--root 仓根]
    kb-spec-check.py --selftest          # KB_SPEC_CHECK_BREAK=<项> 时必须判红

规格的两种写法（kb-spec-drafter 两份都写，主 agent 手写的用哪种都行）：
  JSON：[{"file": "<仓根起的路径>", "old": "<旧串>", "new": "<新串>", "basis": "<依据>"}, …]，或 {"entries": [同上]}
  markdown：每条一段——一行「文件：<路径>」、一行「旧串：」后跟一个围栏块、一行「新串：」后跟一个围栏块、一行「依据：…」；
            围栏用三个及以上的反引号或波浪号，块里原样是要替换的文字。插一行就把锚点那一整行当旧串、锚点加新行当新串。
判五样，任一样不成立就判红（退 1），逐条列出：
  ① 旧串在目标文件里恰好出现 1 次（与 replace-once.py 同义）；文件不在也红；
  ② 新串里每个已登记的编号（kb 里 `<!-- doc-lint:registry name-col=N -->` 登记表的第一列与「## 编号 简称 —— 状态」登记标题）
     写成「编号（简称）」、简称与登记位逐字一致（忽略 ** 与反引号）；登记行自己的第一格不算引用。与已登记编号同形、却指别的东西的记号
     （判决里的段名写成 E10、N2）也在这一条里红；
  ③ 新串比旧串多出来的位置指代、时间指代与自称（doc-lint 那三项词表的子集：如上所述、上述、下表、本轮、这一轮、本条、本决策……）；
     时间指代在同一行带 YYYY-MM-DD 日期时不算，自称只在行首或标点之后才算；
  ④ 改了决策正文里「**依据**」段、增删了引的实验号的，同一份规格里要有那个实验页（.claude/kb/experiments/<号>-*.md）的一条改动
     （门禁 doc-experiments 的 decision-links 格那条双向检查的另一侧）；
  ⑤ 写决策变更史（.claude/kb/decisions-history.md）的，新串里要有「### 」或「#### 」标题行与「快查·改前：」「快查·改后：」两行。
②③ 只判新串比旧串多出来的：旧串里原有的不怪这份规格。认不出规格（两种写法都不是、一条都没有）退 2。
"""
import glob
import json
import os
import re
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402
BROKEN = os.environ.get("KB_SPEC_CHECK_BREAK", "")

REGISTRY_MARK = re.compile(r"<!--\s*doc-lint:registry\s+name-col=(\d+)\s*-->")
NOT_NUMBERS_MARK = re.compile(r"<!--\s*doc-lint:not-numbers\s+(.*?)\s*-->")
REGISTRY_HEADING = re.compile(r"^#{2,4}\s+([A-Z]{1,3}-?\d+(?:\.\d+)?)\s+(.+?)\s+——")
NUMBER_TOKEN = re.compile(r"(?<![A-Za-z0-9_\-./])([A-Z]{1,3}-?\d+(?:\.\d+)?)(?![A-Za-z0-9_])")
FOLLOWING_NAME = re.compile(r"[\s*`]*（([^（）]*)）")
POSITION_WORDS = ["如上所述", "同上", "见上文", "前面提到", "前述", "下面会说", "见下节", "见上表", "上面那张表", "上述", "下文",
                  "下表", "以下是", "逐条如下：", "见上方", "见下方", "见下一段", "同上一行"]
TIME_WORDS = ["本轮", "这一轮", "上一轮", "前一轮", "上轮"]
SELF_WORDS = ["本条", "本决策", "本实验", "该决策", "该实验", "这条不变量", "本节", "本章节", "本表", "本文档"]
SELF_WORD_START = re.compile(r"(?:^|[。，；：！？、（(「\s>|*])(" + "|".join(SELF_WORDS) + ")", re.M)
DATE = re.compile(r"\d{4}-\d{2}-\d{2}")
EVIDENCE_LABEL = "**依据**"
EXPERIMENT_NUMBER = re.compile(r"(?<![A-Za-z0-9_\-])E(\d+)(?![0-9])")


def normalized_name(text):
    return re.sub(r"\s+", "", text.replace("**", "").replace("`", ""))


def load_registry(root):
    """→ ({编号: 简称}, {不算编号的记号})。"""
    registry, not_numbers = {}, set()
    for path in glob.glob(os.path.join(root, ".claude", "kb", "**", "*.md"), recursive=True):
        lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
        for index, line in enumerate(lines):
            for words in NOT_NUMBERS_MARK.findall(line):
                not_numbers.update(words.split())
            heading = REGISTRY_HEADING.match(line)
            if heading:
                registry.setdefault(heading.group(1), heading.group(2).strip())
            mark = REGISTRY_MARK.search(line)
            if not mark:
                continue
            column = int(mark.group(1))
            table_started = False
            for row in lines[index + 1:]:
                if not row.startswith("|"):
                    if table_started or row.strip():
                        break
                    continue
                table_started = True
                cells = [cell.strip() for cell in row.strip().strip("|").split("|")]
                number = cells[0].replace("**", "").replace("`", "").strip() if cells else ""
                if len(cells) >= column and NUMBER_TOKEN.fullmatch(number):
                    registry.setdefault(number, cells[column - 1].replace("**", "").replace("`", "").strip())
    return registry, not_numbers


def fenced_block(lines, start):
    """lines[start] 起往后找第一个围栏块：→ (块里的文字, 块之后的下一行下标)；找不到 → (None, start)。"""
    index = start
    while index < len(lines) and not lines[index].strip():
        index += 1
    if index >= len(lines):
        return None, start
    opening = re.match(r"^\s*(`{3,}|~{3,})", lines[index])
    if not opening:
        return None, start
    fence = opening.group(1)
    body = []
    for cursor in range(index + 1, len(lines)):
        if lines[cursor].strip().startswith(fence) and lines[cursor].strip().strip(fence[0]) == "":
            return "\n".join(body), cursor + 1
        body.append(lines[cursor])
    return None, start


def parse_markdown_spec(text):
    entries, lines, index, current = [], text.splitlines(), 0, None
    while index < len(lines):
        line = lines[index].strip()
        if line.startswith("文件：") or line.startswith("文件:"):
            current = {"file": line.split("：", 1)[-1].split(":", 1)[-1].strip().strip("`").strip(), "old": None, "new": None, "basis": ""}
            entries.append(current)
        elif current is not None and (line.startswith("旧串：") or line.startswith("新串：")):
            block, after = fenced_block(lines, index + 1)
            current["old" if line.startswith("旧串") else "new"] = block
            if block is not None:
                index = after
                continue
        elif current is not None and line.startswith("依据："):
            current["basis"] = line[len("依据："):].strip()
        index += 1
    return entries


def load_spec(path):
    text = open(path, encoding="utf-8", errors="replace").read()
    try:
        data = json.loads(text)
        entries = data.get("entries") if isinstance(data, dict) else data
        if isinstance(entries, list):
            return [{"file": str(item.get("file", "")), "old": item.get("old"), "new": item.get("new"), "basis": item.get("basis", "")}
                    for item in entries if isinstance(item, dict)]
    except ValueError:
        pass
    return parse_markdown_spec(text)


def reference_problems(new, old, registry, not_numbers):
    def violations(text):
        found = []
        for line in (text or "").splitlines():
            first_cell = line.strip().strip("|").split("|")[0].strip().replace("**", "").replace("`", "") if line.strip().startswith("|") else None
            if REGISTRY_HEADING.match(line):
                continue
            for match in NUMBER_TOKEN.finditer(line):
                token = match.group(1)
                if token not in registry or token in not_numbers or token == first_cell:
                    continue
                following = FOLLOWING_NAME.match(line, match.end())
                if following and (normalized_name(following.group(1)) == normalized_name(registry[token]) or BROKEN == "any-name"):
                    continue
                written = following.group(1) if following else "没带简称"
                found.append(f"{token}（{written}）→ 登记位写的是「{token}（{registry[token]}）」")
        return found
    old_violations = violations(old)
    fresh = []
    for violation in violations(new):
        if violation in old_violations:
            old_violations.remove(violation)
        else:
            fresh.append(violation)
    return fresh


def pointer_problems(new, old):
    fresh = []
    def counts(text, words):
        return {word: (text or "").count(word) for word in words}
    for word, count in counts(new, POSITION_WORDS).items():
        if count > counts(old, [word])[word]:
            fresh.append(f"位置指代「{word}」")
    def undated_time(text):
        total = {word: 0 for word in TIME_WORDS}
        for line in (text or "").splitlines():
            if DATE.search(line) and BROKEN != "dated-time":
                continue
            for word in TIME_WORDS:
                total[word] += len(re.findall(r"(?<!样)" + word, line))
        return total
    old_time = undated_time(old)
    for word, count in undated_time(new).items():
        if count > old_time[word]:
            fresh.append(f"时间指代「{word}」（这一行没有日期）")
    old_self = [match.group(1) for match in SELF_WORD_START.finditer(old or "")]
    for match in SELF_WORD_START.finditer(new or ""):
        if match.group(1) in old_self:
            old_self.remove(match.group(1))
        else:
            fresh.append(f"自称「{match.group(1)}」")
    return fresh


def check_entries(entries, root):
    registry, not_numbers = load_registry(root)
    problems = []
    targets = [entry["file"] for entry in entries]
    for number, entry in enumerate(entries, 1):
        label = f"第 {number} 条（{entry['file']}）"
        if entry.get("old") is None or entry.get("new") is None:
            problems.append(f"{label}：没认出旧串或新串的围栏块")
            continue
        path = os.path.join(root, entry["file"])
        if not os.path.isfile(path):
            problems.append(f"{label}：目标文件不在")
        else:
            hits = open(path, encoding="utf-8", errors="replace").read().count(entry["old"])
            if hits != 1 and BROKEN != "any-hits":
                problems.append(f"{label}：旧串在文件里出现 {hits} 次，要恰好 1 次")
        if BROKEN != "no-references":
            problems += [f"{label}：编号引用不对：{item}" for item in reference_problems(entry["new"], entry["old"], registry, not_numbers)]
        if BROKEN != "no-pointers":
            problems += [f"{label}：新串里有{item}，换成名字或日期锚" for item in pointer_problems(entry["new"], entry["old"])]
        if entry["file"].startswith(".claude/kb/decisions/") and (EVIDENCE_LABEL in entry["old"] or EVIDENCE_LABEL in entry["new"]) and BROKEN != "no-pairing":
            changed = set(EXPERIMENT_NUMBER.findall(entry["new"])) ^ set(EXPERIMENT_NUMBER.findall(entry["old"]))
            for experiment in sorted(changed, key=int):
                if not any(re.match(rf"\.claude/kb/experiments/0*{experiment}-", target) for target in targets):
                    problems.append(f"{label}：依据段增删了 E{experiment}，规格里却没有它实验页「### 影响的决策」那一行的改动（门禁 doc-experiments 的 decision-links 格的另一侧）")
        if entry["file"] == ".claude/kb/decisions-history.md" and BROKEN != "no-history":
            new = entry["new"]
            if (not re.search(r"^(?:### 20\d\d-\d\d-\d\d|- )", new, re.M) or "快查·" in new
                    or re.search(r"^#### ", new, re.M) or re.search(r"^\*\*现状\*\*：", new, re.M)):
                problems.append(f"{label}：写变更史要有「### 日期」行或顶格「- 」变更项，不写「#### 」子条目、快查两行与现状行（.claude/rules/changelog-format.md）")
    return problems


def run(arguments):
    root = REPOSITORY_ROOT
    if "--root" in arguments:
        position = arguments.index("--root")
        root = os.path.abspath(arguments[position + 1])
        arguments = arguments[:position] + arguments[position + 2:]
    if len(arguments) != 1:
        print("✗ 要给一份规格文件\n→ 怎么办：kb-spec-check.py <规格.json 或 规格.md> [--root 仓根]")
        return 2
    entries = load_spec(arguments[0])
    if not entries:
        print(f"✗ 认不出规格 {arguments[0]}：两种写法都不是，或一条都没有\n→ 怎么办：照这个脚本文件头「规格的两种写法」写，或派 kb-spec-drafter 起草")
        return 2
    problems = check_entries(entries, root)
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    if problems:
        print(f"  ✗ 规格 {len(entries)} 条里有 {len(problems)} 处会让门禁红（逐处列在上面）")  # gate-lint:summary
        print("  → 怎么办：按每处改规格原文再跑一次这个脚本；改不动的（要判断的）交主 agent，不派书记员")
        return 1
    print(f"  ✓ 规格 {len(entries)} 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 {len(entries)} 条）")
    return 0


def selftest():
    work = tempfile.mkdtemp(prefix="kb-spec-check-selftest-")
    try:
        kb = os.path.join(work, ".claude", "kb")
        os.makedirs(os.path.join(kb, "decisions"))
        os.makedirs(os.path.join(kb, "experiments"))
        open(os.path.join(kb, "checks-owed.md"), "w", encoding="utf-8").write(
            "# 欠的检查\n\n<!-- doc-lint:registry name-col=2 -->\n| 编号 | 简称 | 内容 |\n|---|---|---|\n| C120 | 锚点腐化 | 旧行 |\n| C121 | 另一笔 | 行 |\n")
        open(os.path.join(kb, "decisions", "03-空间分配.md"), "w", encoding="utf-8").write(
            "## D3 空间分配 —— 已定\n\n**依据**：E10（样本实验） 证明了 x。\n")
        open(os.path.join(kb, "experiments", "10-样本实验.md"), "w", encoding="utf-8").write("## E10 样本实验 —— 已跑\n\n### 影响的决策\n| D3 | 支撑 | 不受影响 |\n")
        open(os.path.join(kb, "experiments", "11-另一实验.md"), "w", encoding="utf-8").write("## E11 另一实验 —— 已跑\n\n### 影响的决策\n")
        open(os.path.join(kb, "decisions-history.md"), "w", encoding="utf-8").write("# 决策变更史\n\n## D3（空间分配）\n\n锚点行\n")
        def entry(file, old, new):
            return {"file": file, "old": old, "new": new, "basis": "样本"}
        owed = ".claude/kb/checks-owed.md"
        cases = [
            ("干净的一条", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行，见 C121（另一笔） |")], 0),
            ("旧串 0 次", [entry(owed, "不在的行", "新")], 1),
            ("编号没带简称", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行，见 C121 |")], 1),
            ("简称与登记位不一致", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 见 C121（别的名字） |")], 1),
            ("判决段名撞了登记编号", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 判决 E10 段 |")], 1),
            ("新加位置指代", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 见上述那一行 |")], 1),
            ("新加时间指代", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 本轮还清 |")], 1),
            ("时间指代带日期不算", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 2026-09-26 这一轮还清 |")], 0),
            ("依据段加实验号没配对", [entry(".claude/kb/decisions/03-空间分配.md", "**依据**：E10（样本实验） 证明了 x。",
                                             "**依据**：E10（样本实验） 证明了 x；E11（另一实验） 证明了 y。")], 1),
            ("依据段加实验号带了配对", [entry(".claude/kb/decisions/03-空间分配.md", "**依据**：E10（样本实验） 证明了 x。",
                                               "**依据**：E10（样本实验） 证明了 x；E11（另一实验） 证明了 y。"),
                                         entry(".claude/kb/experiments/11-另一实验.md", "### 影响的决策\n", "### 影响的决策\n| D3（空间分配） 已定项 1 | 支撑 | 2026-09-26 改了 |\n")], 0),
            ("变更史用旧形态", [entry(".claude/kb/decisions-history.md", "锚点行",
                                      "锚点行\n\n### 2026-09-26\n\n#### 已定项 1：样本\n\n> 快查·改前：甲。\n>\n> 快查·改后：乙。\n")], 1),
            ("变更史新形态", [entry(".claude/kb/decisions-history.md", "锚点行",
                                    "锚点行\n\n### 2026-09-26\n\n- 已定项 1：样本\n  - **结论**：乙。\n")], 0),
        ]
        failures = []
        for label, entries, want in cases:
            problems = check_entries(entries, work)
            if (1 if problems else 0) != want:
                failures.append(f"{label}：应当{'判红' if want else '判绿'}，实际 {problems or '判绿'}")
        markdown = os.path.join(work, "spec.md")
        open(markdown, "w", encoding="utf-8").write("## 条 1\n文件：`.claude/kb/checks-owed.md`\n旧串：\n```\n| C120 | 锚点腐化 | 旧行 |\n```\n新串：\n~~~\n| C120 | 锚点腐化 | 新行 |\n~~~\n依据：样本\n")
        parsed = load_spec(markdown)
        if len(parsed) != 1 or parsed[0]["old"] != "| C120 | 锚点腐化 | 旧行 |" or parsed[0]["new"] != "| C120 | 锚点腐化 | 新行 |":
            failures.append(f"markdown 规格没认对：{parsed}")
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 check_entries() / reference_problems() / pointer_problems() / parse_markdown_spec() 的判法；KB_SPEC_CHECK_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ kb-spec-check 自检通过：旧串次数、编号简称、段名撞登记、位置与时间指代、依据配对、变更史形态各有红绿样本，markdown 规格认得出（{len(cases) + 1} 种）")
        return 0
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(sys.argv[1:]))
