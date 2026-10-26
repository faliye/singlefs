#!/usr/bin/env python3
# admission: always 判的是此刻这份规格与它点名的 kb 文件此刻的样子，几秒跑完；每次派书记员之前都现判
# run-condition: command git
"""kb 写回规格检查：派书记员（kb-scribe）之前，规格原文本身会不会让门禁红。

用法：
    kb-spec-check.py <规格.json 或 规格.md> [<规格> …] [--root 仓根] [--allow-uncommitted <路径> …]
    kb-spec-check.py --selftest          # KB_SPEC_CHECK_BREAK=<项> 时必须判红
    kb-spec-check.py --help              # 打这一段用法（-h 同），退 0
  一次给几份同批规格（并行写回员按 kb 文件切开写的那几份）时，除了各份各自判，再做跨规格比对（⑦）。
  --allow-uncommitted：这一次确认「依据」里这份只在工作区的文件会跟这一批一起提交，放行它（⑥）；可重复。

规格的两种写法（kb-spec-drafter 两份都写，主 agent 手写的用哪种都行）：
  JSON：[{"file": "<仓根起的路径>", "old": "<旧串>", "new": "<新串>", "basis": "<依据>"}, …]，或 {"entries": [同上]}
  markdown：每条一段——一行「文件：<路径>」、一行「旧串：」后跟一个围栏块、一行「新串：」后跟一个围栏块、一行「依据：…」；
            围栏用三个及以上的反引号或波浪号，块里原样是要替换的文字。插一行就把锚点那一整行当旧串、锚点加新行当新串。
判七样，任一样不成立就判红（退 1），逐条列出：
  ① 旧串在目标文件里恰好出现 1 次（与 replace-once.py 同义）；文件不在也红；
  ② 新串里每个已登记的编号（kb 里 `<!-- doc-lint:registry name-col=N -->` 登记表的第一列与「## 编号 简称 —— 状态」登记标题）
     写成「编号（简称）」、简称与登记位逐字一致（忽略 ** 与反引号）；登记行自己的第一格不算引用。与已登记编号同形、却指别的东西的记号
     （判决里的段名写成 E10、N2）也在这一条里红。
     目标是译文（<名>.en.md、<名>.ja.md）的，编号后面照 `.claude/kb/translation-glossary.md` 的写法用半角括号写译名「E184 (译名)」
     （括号前的空白不限，译名里可以再套半角括号）：只判形态，不跟同语言的登记位比（译文的登记位可能正是这份规格要改的、也可能还没译到）。
     红的有三种：编号后面没有括号；写了全角括号（中文简称的写法，术语表不用全角括号）；.en.md 括号里的译名带中日文字符（照中文简称原样抄了；
     .ja.md 的译名本来就是日文，不判这一样）。中文那一份照旧只认全角括号加逐字一致的简称；
  ③ 新串比旧串多出来的位置指代、时间指代与自称（doc-lint 那三项词表的子集：如上所述、上述、下表、本轮、这一轮、本条、本决策……）；
     时间指代在同一行带 YYYY-MM-DD 日期时不算，自称只在行首或标点之后才算；
  ④ 改了决策正文里「**依据**」段、增删了引的实验号的，同一份规格里要有那个实验页（.claude/kb/experiments/<号>-*.md）的一条改动
     （门禁 doc-experiments 的 decision-links 格那条双向检查的另一侧）；
  ⑤ 写决策变更史（.claude/kb/decisions-history.md）的，新串里要有「### <日期>」行或顶格「- 」变更项，不许有「#### 」子条目、「快查·」两行与「**现状**：」行（.claude/rules/changelog-format.md）；
  ⑥ 「依据」里点名的路径（从仓根起、带目录层的；写了「:行号」的连行号）：已跟踪、工作区与暂存区都和 HEAD 相同的放行；
     已跟踪而工作区或暂存区与 HEAD 不同（别的会话还没提交的改动）、又没登记在 `.claude/batch-scope`（这一批的改动范围）、
     也没被 --allow-uncommitted 点名放行的，判红、点名是哪一条哪一个路径（写了行号的，那几行在 HEAD 里存在、且工作区与暂存区的同一行与 HEAD 相同才放行）；
     暂存区里的新文件（HEAD 里没有）同样判红；未跟踪的新文件（这一批新写的判决与证据）放行，输出里另列一行「依据是未跟踪的新文件 N 份：<名单>」；
     三处都没有的路径不判；仓根不是 git 仓而依据里有路径，判红（不静默放行）。判法在 lib_basis_paths.py；
  ⑦ 跨规格：命令行给了两份以上规格时，各份新串里新加的句子（比旧串多出来的）中，同一个已登记编号所在的那一句里的状态词不一致，判红，点名两份规格与两句。
     状态词只认下面 STATUS_WORDS 那一张表（已实现 / 未实现 / 已还清 / 未还清 / 已定 / 未定 / 半定），句子按换行与「。；;」切；一句里有几个编号时，
     状态词算给它前面最近的那个编号（前面没有编号、整句只有一个编号时算给它）；译文规格不参与（状态词是中文的）。
     冲突的算法：「完成」一族里已实现、已还清算已完成，未实现、未还清算未完成，一方已完成、一方未完成才冲突；「定」一族里已定、未定、半定三者互不相同。
     一次只给一份规格时不做这一样。
②③ 只判新串比旧串多出来的：旧串里原有的不怪这份规格。认不出规格（两种写法都不是、一条都没有）退 2；
不认的选项、--root 缺值、规格文件读不了也退 2（写哪一份、怎么办，不抛 traceback）。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一例的标签，括号里是那条「✗ 自检：」的开头）：
  KB_SPEC_CHECK_BREAK=any-hits  check_entries() 不判旧串恰好命中一次（①）
      旧串 0 次（「✗ 自检：旧串 0 次：应当判红」）
  KB_SPEC_CHECK_BREAK=any-name  reference_problems() 编号后面跟什么简称都算对（②）
      简称与登记位不一致（「✗ 自检：简称与登记位不一致：应当判红」）
      连带红的格：「✗ 自检：嵌套括号简称写错：应当判红」、「✗ 自检：嵌套括号简称只写了外层：应当判红」
  KB_SPEC_CHECK_BREAK=dated-time  pointer_problems() 同一行带日期的时间指代也判红（③）
      时间指代带日期不算（「✗ 自检：时间指代带日期不算：应当判绿」）
  KB_SPEC_CHECK_BREAK=flat-parens  following_name() 简称里一见嵌套括号就认不出（②，改前 FOLLOWING_NAME 的样子）
      简称带嵌套括号（「✗ 自检：简称带嵌套括号：应当判绿」）
  KB_SPEC_CHECK_BREAK=no-history  check_entries() 不判决策变更史的形态（⑤）
      变更史用旧形态（「✗ 自检：变更史用旧形态：应当判红」）
  KB_SPEC_CHECK_BREAK=no-pairing  check_entries() 不判依据段与实验页的配对（④）
      依据段加实验号没配对（「✗ 自检：依据段加实验号没配对：应当判红」）
  KB_SPEC_CHECK_BREAK=no-pointers  check_entries() 不调 pointer_problems()（③）
      新加位置指代（「✗ 自检：新加位置指代：应当判红」）、新加时间指代（「✗ 自检：新加时间指代：应当判红」）
  KB_SPEC_CHECK_BREAK=no-references  check_entries() 不调 reference_problems()（②）
      编号没带简称（「✗ 自检：编号没带简称：应当判红」）、简称与登记位不一致（「✗ 自检：简称与登记位不一致：应当判红」）、
      判决段名撞了登记编号（「✗ 自检：判决段名撞了登记编号：应当判红」）
      连带红的格：「✗ 自检：嵌套括号简称写错：应当判红」、「✗ 自检：嵌套括号简称只写了外层：应当判红」、「✗ 自检：嵌套括号简称括号没配平：应当判红」、「✗ 自检：译文写全角中文简称：应当判红」、「✗ 自检：译文写全角日文译名：应当判红」、「✗ 自检：译文编号没带译名：应当判红」、「✗ 自检：译文译名带中文简称原样：应当判红」、「✗ 自检：中文写半角括号简称：应当判红」
  KB_SPEC_CHECK_BREAK=translations-as-registry  load_registry() 把译文（.en.md、.ja.md）也当登记位读，译名先于中文简称被认到（②，改前的读法）
      译文登记位简称与中文不同（「✗ 自检：译文登记位简称与中文不同：应当判绿」），连带 selftest() 里别的引用了登记编号的绿例（干净的一条、简称带嵌套括号、依据段加实验号带了配对）
      连带红的格：「✗ 自检：干净的一条：应当判绿」、「✗ 自检：简称带嵌套括号：应当判绿」、「✗ 自检：依据段加实验号带了配对：应当判绿」、「✗ 自检：命令行 ['…spec-done.json', '--root', '…'] 应当退 0」、「✗ 规格 1 条（1 份）里有 1 处会让门禁红」
  KB_SPEC_CHECK_BREAK=raw-arguments  run() 不先判 --help、不认的选项、--root 缺值与读不了的规格（改前的样子）
      命令行那几格（「✗ 自检：命令行 ['--help'] 应当退 0」起的六条）
      连带红的格：「✗ 自检：命令行 ['-h'] 应当退 0」、「✗ 自检：命令行 ['--bogus', '…'] 应当退 2」、「✗ 自检：命令行 ['…', '--root', '…'] 应当退 2」、「✗ 自检：命令行 ['…spec.md', '--root'] 应当退 2」
  KB_SPEC_CHECK_BREAK=translations-as-chinese  reference_problems() 译文也按中文那一份判（只认全角括号加逐字一致的简称，②改前的样子）
      译文写半角译名（「✗ 自检：译文写半角译名：应当判绿」）、译文译名带嵌套半角括号（「✗ 自检：译文译名带嵌套半角括号：应当判绿」）；
      译文写全角中文简称（「✗ 自检：译文写全角中文简称：应当判红」）也变绿（中文那一支认得逐字一致的全角简称）
  KB_SPEC_CHECK_BREAK=translation-fullwidth-ok  reference_problems() 译文里编号后面写全角括号也放行（②）
      译文写全角中文简称（「✗ 自检：译文写全角中文简称：应当判红」）、译文写全角日文译名（「✗ 自检：译文写全角日文译名：应当判红」）
  KB_SPEC_CHECK_BREAK=translation-chinese-name-ok  reference_problems() 英语版半角括号里的译名带中日文字符也放行（②）
      译文译名带中文简称原样（「✗ 自检：译文译名带中文简称原样：应当判红」）
  KB_SPEC_CHECK_BREAK=basis-unchecked  check_entries() 不核「依据」里的路径有没有别的会话没提交的改动（⑥）
      依据已跟踪且有没进这一批的改动（「✗ 自检：依据已跟踪且有没进这一批的改动：应当判红」）、依据引的行号被改了（「✗ 自检：依据引的行号被改了：应当判红」）、
      依据是暂存区里的新文件（「✗ 自检：依据是暂存区里的新文件：应当判红」）、依据已跟踪且只暂存区有改动（「✗ 自检：依据已跟踪且只暂存区有改动：应当判红」）、
      命令行那一格（「✗ 自检：命令行 … 应当退 1、说「依据是别的会话还没提交的改动」」）
      连带红的格：「✗ 自检：依据引的行号范围里有一行被改了：应当判红」、「✗ 自检：依据引的行号只在工作区：应当判红」、「✗ 自检：依据引的行号只暂存区改了：应当判红」、「✗ 自检：依据里的未跟踪新文件没按名单列出」、「✗ 自检：命令行 ['…spec-untracked.json', '--root', '…'] 应当退 0」
  KB_SPEC_CHECK_BREAK=cross-spec-unchecked  cross_spec_problems() 不比各份规格里同一编号的状态词（⑦）
      跨规格状态词不一致（「✗ 自检：跨规格状态词不一致：应当判红」）、跨规格定的状态词不一致（「✗ 自检：跨规格定的状态词不一致：应当判红」）、
      点名两份规格的那一格（「✗ 自检：跨规格状态词不一致：应当点名两份规格」起）、命令行给两份规格的那一格
      连带红的格：「✗ 自检：跨规格不变量编号状态不一致：应当判红」、「✗ 自检：命令行 ['…spec-done.json', '…spec-open.json', '--root', '…'] 应当退 1」
  KB_SPEC_CHECK_BREAK=cross-spec-any-word  cross_spec_problems() 状态词不同就判红，不分「已实现与已还清同属已完成」这类一族（⑦）
      跨规格状态词同族一致（「✗ 自检：跨规格状态词同族一致：应当判绿」）
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
sys.path.insert(0, SCRIPT_DIRECTORY)
import lib_basis_paths  # noqa: E402
BROKEN = os.environ.get("KB_SPEC_CHECK_BREAK", "")

REGISTRY_MARK = re.compile(r"<!--\s*doc-lint:registry\s+name-col=(\d+)\s*-->")
NOT_NUMBERS_MARK = re.compile(r"<!--\s*doc-lint:not-numbers\s+(.*?)\s*-->")
REGISTRY_HEADING = re.compile(r"^#{2,4}\s+([A-Z]{1,3}-?\d+(?:\.\d+)?)\s+(.+?)\s+——")
NUMBER_TOKEN = re.compile(r"(?<![A-Za-z0-9_\-./])([A-Z]{1,3}-?\d+(?:\.\d+)?)(?![A-Za-z0-9_])")
FOLLOWING_NAME_PREFIX = re.compile(r"[\s*`]*（")
POSITION_WORDS = ["如上所述", "同上", "见上文", "前面提到", "前述", "下面会说", "见下节", "见上表", "上面那张表", "上述", "下文",
                  "下表", "以下是", "逐条如下：", "见上方", "见下方", "见下一段", "同上一行"]
TIME_WORDS = ["本轮", "这一轮", "上一轮", "前一轮", "上轮"]
SELF_WORDS = ["本条", "本决策", "本实验", "该决策", "该实验", "这条不变量", "本节", "本章节", "本表", "本文档"]
SELF_WORD_START = re.compile(r"(?:^|[。，；：！？、（(「\s>|*])(" + "|".join(SELF_WORDS) + ")", re.M)
DATE = re.compile(r"\d{4}-\d{2}-\d{2}")
EVIDENCE_LABEL = "**依据**"
EXPERIMENT_NUMBER = re.compile(r"(?<![A-Za-z0-9_\-])E(\d+)(?![0-9])")
# 译文：<名>.en.md、<名>.ja.md（登记在 .claude/trilingual-documents，规矩在 .claude/skills/trilingual/SKILL.md）。登记位只认中文那一份；
# 判法与 .claude/gate.d/lib-trilingual.py 的 LANGUAGE_SUFFIXED 同口径：带两个小写字母语言后缀的 .md 不是中文那一份。
TRANSLATION_SUFFIX = re.compile(r"\.[a-z]{2}\.md$")
ASCII_FOLLOWING_PREFIX = re.compile(r"[\s*`]*\(")
CJK_CHARACTER = re.compile(r"[぀-ヿ㐀-鿿]")
# 跨规格比对认的状态词（⑦）：词 → (一族, 这一族里的取值)。完成一族里已实现与已还清同值（都算已完成）；定一族里三个词各是各的。
# 加词时同时想清楚它和别的词冲不冲突，再往这里加一行；最长的先匹配（未实现不能被当成「实现」，已定不能吃掉已定项）。
STATUS_WORDS = {
    "已实现": ("完成", "已完成"), "已还清": ("完成", "已完成"),
    "未实现": ("完成", "未完成"), "未还清": ("完成", "未完成"),
    "已定": ("定", "已定"), "未定": ("定", "未定"), "半定": ("定", "半定"),
}
STATUS_WORD = re.compile("|".join(sorted((re.escape(word) for word in STATUS_WORDS), key=len, reverse=True)) + r"(?!项)")
INVARIANT_NUMBER = re.compile(r"I-\d+(?:\.\d+)*")   # 不变量编号登记在 crates/*/invariants/ 下，kb 的登记表里没有；跨规格比对照样认它
QUOTED_SPAN = re.compile(r"「[^「」\n]*」|『[^『』\n]*』")   # 引原话（旧状态、别处的说法）里的状态词不算这一句自己的说法
SENTENCE_SEPARATOR = re.compile(r"[。；;\n]")


def following_name(line, position):
    """line[position:] 起（跳过空白、星号、反引号）跟着一对配平的全角括号：→ 括号里的简称，简称里可以再套括号；没有、或括号没配平 → None。"""
    opening = FOLLOWING_NAME_PREFIX.match(line, position)
    if not opening:
        return None
    depth, start = 1, opening.end()
    for cursor in range(start, len(line)):
        if line[cursor] == "（":
            if BROKEN == "flat-parens":   # 弄坏开关：简称里不许再有括号（改前的样子）
                return None
            depth += 1
        elif line[cursor] == "）":
            depth -= 1
            if depth == 0:
                return line[start:cursor]
    return None


def following_ascii_name(line, position):
    """译文用：line[position:] 起（跳过空白、星号、反引号）跟着一对配平的半角括号 → 括号里的译名，里面可以再套半角括号；没有、或没配平 → None。"""
    opening = ASCII_FOLLOWING_PREFIX.match(line, position)
    if not opening:
        return None
    depth, start = 1, opening.end()
    for cursor in range(start, len(line)):
        if line[cursor] == "(":
            depth += 1
        elif line[cursor] == ")":
            depth -= 1
            if depth == 0:
                return line[start:cursor]
    return None


def normalized_name(text):
    return re.sub(r"\s+", "", text.replace("**", "").replace("`", ""))


def load_registry(root):
    """→ ({编号: 简称}, {不算编号的记号})。登记位只认中文那一份：译文（<名>.en.md、<名>.ja.md）里的简称是译名，不是登记位。"""
    registry, not_numbers = {}, set()
    for path in sorted(glob.glob(os.path.join(root, ".claude", "kb", "**", "*.md"), recursive=True)):
        if TRANSLATION_SUFFIX.search(os.path.basename(path)) and BROKEN != "translations-as-registry":   # 弄坏开关：译文也当登记位（改前的读法）
            continue
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


def translation_reference_violation(token, line, position, language):
    """译文里一处已登记编号的写法：→ 不对的说明，对的 → None。形态判（半角括号、里面有译名），不跟同语言的登记位比。"""
    ascii_name = following_ascii_name(line, position)
    if ascii_name is not None and ascii_name.strip():
        if language == "en" and CJK_CHARACTER.search(ascii_name) and BROKEN != "translation-chinese-name-ok":
            return f"{token} ({ascii_name}) → 英语版的译名里不该有中日文字符（照中文简称原样抄了）；写英文译名"
        return None
    fullwidth = following_name(line, position)
    if fullwidth is not None:
        if BROKEN == "translation-fullwidth-ok":
            return None
        return f"{token}（{fullwidth}）→ 译文里编号后面不用全角括号（那是中文简称的写法）；写成「{token} (<译名>)」，见 .claude/kb/translation-glossary.md"
    return f"{token} → 译文里编号后面没带半角括号译名；写成「{token} (<译名>)」，见 .claude/kb/translation-glossary.md"


def reference_problems(new, old, registry, not_numbers, language=None):
    """language 是译文的语言后缀（en、ja），中文那一份传 None。"""
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
                if language is not None and BROKEN != "translations-as-chinese":
                    problem = translation_reference_violation(token, line, match.end(), language)
                    if problem is not None:
                        found.append(problem)
                    continue
                following = following_name(line, match.end())
                if following is not None and (normalized_name(following) == normalized_name(registry[token]) or BROKEN == "any-name"):
                    continue
                written = following if following is not None else "没带简称"
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


def status_claims_of(sentence, registry, not_numbers):
    """一句话里的状态说法：[(编号, 状态词)]。编号后面紧跟的全角简称括注先遮掉（简称里的字不是状态），「」『』引的原话也遮掉，再按逗号与顿号切成分句；
    状态词算给它所在分句里、它前面最近的那个已登记编号，分句里它前面没有编号就不算（「字段表未定」改成……已定 这类引原话的写法不当成编号的状态）。"""
    masked, last = [], 0
    for match in NUMBER_TOKEN.finditer(sentence):
        if match.start() < last:
            continue
        following = following_name(sentence, match.end())
        if following is not None:
            opening = sentence.index("（", match.end())
            end = opening + 1 + len(following) + 1
            masked.append(sentence[last:match.end()])
            masked.append("·" * (end - match.end()))
            last = end
    masked.append(sentence[last:])
    claims = []
    unquoted = QUOTED_SPAN.sub(lambda match: "·" * len(match.group(0)), "".join(masked))
    for clause in re.split(r"[，,、]", unquoted):
        numbers = [(match.start(), match.group(1)) for match in NUMBER_TOKEN.finditer(clause)
                   if (match.group(1) in registry or INVARIANT_NUMBER.fullmatch(match.group(1))) and match.group(1) not in not_numbers]
        for word_match in STATUS_WORD.finditer(clause):
            before = [number for position, number in numbers if position < word_match.start()]
            if before:
                claims.append((before[-1], word_match.group(0)))
    return claims


def cross_spec_problems(specs, registry, not_numbers):
    """⑦ 跨规格：specs 是 [(规格名, 条目列表)]。同一个已登记编号在不同规格的新句子里，状态词落在同一族而取值不一致 → 点名两份规格与两句。"""
    if BROKEN == "cross-spec-unchecked":
        return []
    claims = {}   # 编号 → [(规格下标, 条目序号, 文件, 句子, 状态词)]
    for spec_index, (_name, entries) in enumerate(specs):
        for entry_number, entry in enumerate(entries, 1):
            if TRANSLATION_SUFFIX.search(os.path.basename(entry["file"])) or entry.get("new") is None:
                continue
            old_sentences = {sentence.strip() for sentence in SENTENCE_SEPARATOR.split(entry.get("old") or "")}
            for sentence in SENTENCE_SEPARATOR.split(entry["new"]):
                if sentence.strip() in old_sentences or not sentence.strip():
                    continue
                for owner, word in status_claims_of(sentence, registry, not_numbers):
                    claims.setdefault(owner, []).append((spec_index, entry_number, entry["file"], sentence.strip(), word))
    problems, reported = [], set()
    for number, found in sorted(claims.items()):
        for first_index, first in enumerate(found):
            for second in found[first_index + 1:]:
                if first[0] == second[0] or first[4] == second[4]:
                    continue
                family_first, value_first = STATUS_WORDS[first[4]]
                family_second, value_second = STATUS_WORDS[second[4]]
                if family_first != family_second or (value_first == value_second and BROKEN != "cross-spec-any-word"):
                    continue
                key = (number, first[0], second[0], first[4], second[4])
                if key in reported:
                    continue
                reported.add(key)
                problems.append(
                    f"跨规格状态不一致：{number}{'（' + registry[number] + '）' if number in registry else ''}在 {specs[first[0]][0]} 第 {first[1]} 条（{first[2]}）写「{first[4]}」"
                    f"（那一句：{first[3][:80]}），在 {specs[second[0]][0]} 第 {second[1]} 条（{second[2]}）写「{second[4]}」（那一句：{second[3][:80]}）")
    return problems


def check_entries(entries, root, allowed=(), untracked_out=None):
    """untracked_out 传一个列表，依据里引的未跟踪新文件按出现次序追加进去（不重复）。"""
    registry, not_numbers = load_registry(root)
    problems = []
    targets = [entry["file"] for entry in entries]
    committed = lib_basis_paths.CommittedState(root)
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
            translation = TRANSLATION_SUFFIX.search(os.path.basename(entry["file"]))
            language = translation.group(0)[1:3] if translation else None
            problems += [f"{label}：编号引用不对：{item}" for item in reference_problems(entry["new"], entry["old"], registry, not_numbers, language)]
        if BROKEN != "basis-unchecked":
            basis_problems, basis_untracked = lib_basis_paths.basis_check(root, entry.get("basis") or "", allowed, committed)
            problems += [f"{label}：依据是别的会话还没提交的改动，不能当现状依据：{item}（{lib_basis_paths.ADVICE}）" for item in basis_problems]
            if untracked_out is not None:
                untracked_out += [path for path in basis_untracked if path not in untracked_out]
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
    guarded = BROKEN != "raw-arguments"   # 弄坏开关：--help、不认的选项、读不了的文件都不先判
    if guarded and arguments[:1] in (["--help"], ["-h"]):
        print(__doc__)
        return 0
    allowed = []
    while "--allow-uncommitted" in arguments:
        position = arguments.index("--allow-uncommitted")
        if position + 1 >= len(arguments) or arguments[position + 1].startswith("-"):
            print("✗ --allow-uncommitted 后面没给路径\n→ 怎么办：--allow-uncommitted <从仓根起的路径>（可重复）；用法跑 kb-spec-check.py --help")
            return 2
        allowed.append(arguments[position + 1])
        arguments = arguments[:position] + arguments[position + 2:]
    if "--root" in arguments:
        position = arguments.index("--root")
        if guarded and position + 1 >= len(arguments):
            print("✗ --root 后面没给值\n→ 怎么办：kb-spec-check.py <规格.json 或 规格.md> [<规格> …] [--root 仓根]")
            return 2
        root = os.path.abspath(arguments[position + 1])
        arguments = arguments[:position] + arguments[position + 2:]
    options = [argument for argument in arguments if argument.startswith("-")]
    if guarded and options:
        print(f"✗ 不认的选项：{' '.join(options)}\n→ 怎么办：只认 --root、--allow-uncommitted（用法跑 kb-spec-check.py --help）")
        return 2
    if not arguments:
        print("✗ 要给一份规格文件\n→ 怎么办：kb-spec-check.py <规格.json 或 规格.md> [<规格> …] [--root 仓根]")
        return 2
    for path in arguments:
        if guarded and not os.path.isfile(path):
            print(f"✗ 读不了规格 {path}（文件不在，或不是普通文件）\n→ 怎么办：核对路径（相对路径相对当前工作目录）；规格写法见 kb-spec-check.py --help「规格的两种写法」")
            return 2
    specs = []
    for path in arguments:
        entries = load_spec(path)
        if not entries:
            print(f"✗ 认不出规格 {path}：两种写法都不是，或一条都没有\n→ 怎么办：照这个脚本文件头「规格的两种写法」写，或派 kb-spec-drafter 起草")
            return 2
        specs.append((path, entries))
    many = len(specs) > 1
    problems, untracked = [], []
    for path, entries in specs:
        problems += [f"规格 {path}：{problem}" if many else problem for problem in check_entries(entries, root, allowed, untracked)]
    if many:
        registry, not_numbers = load_registry(root)
        problems += cross_spec_problems(specs, registry, not_numbers)
    total = sum(len(entries) for _, entries in specs)
    note = lib_basis_paths.untracked_note(untracked)
    if note:
        print(f"  ℹ {note}")
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    if problems:
        print(f"  ✗ 规格 {total} 条（{len(specs)} 份）里有 {len(problems)} 处会让门禁红（逐处列在上面）")  # gate-lint:summary
        print("  → 怎么办：按每处改规格原文再跑一次这个脚本；改不动的（要判断的）交主 agent，不派书记员")
        return 1
    print(f"  ✓ 规格 {total} 条（{len(specs)} 份）：旧串各恰好 1 次，编号引用、指代、依据配对、依据没有别的会话还没提交的改动、变更史形态都过"
          f"{'，跨规格同一编号的状态词一致' if many else ''}（查了 {total} 条）")
    return 0


def selftest():
    work = tempfile.mkdtemp(prefix="kb-spec-check-selftest-")
    try:
        kb = os.path.join(work, ".claude", "kb")
        os.makedirs(os.path.join(kb, "decisions"))
        os.makedirs(os.path.join(kb, "experiments"))
        open(os.path.join(kb, "checks-owed.md"), "w", encoding="utf-8").write(
            "# 欠的检查\n\n<!-- doc-lint:registry name-col=2 -->\n| 编号 | 简称 | 内容 |\n|---|---|---|\n| C120 | 锚点腐化 | 旧行 |\n| C121 | 另一笔 | 行 |\n| C122 | 对拍（HashMap 理想文件系统） | 简称里套着括号 |\n")
        open(os.path.join(kb, "decisions", "03-空间分配.md"), "w", encoding="utf-8").write(
            "## D3 空间分配 —— 已定\n\n**依据**：E10（样本实验） 证明了 x。\n")
        open(os.path.join(kb, "experiments", "10-样本实验.md"), "w", encoding="utf-8").write("## E10 样本实验 —— 已跑\n\n### 影响的决策\n| D3 | 支撑 | 不受影响 |\n")
        open(os.path.join(kb, "experiments", "11-另一实验.md"), "w", encoding="utf-8").write("## E11 另一实验 —— 已跑\n\n### 影响的决策\n")
        open(os.path.join(kb, "decisions-history.md"), "w", encoding="utf-8").write("# 决策变更史\n\n## D3（空间分配）\n\n锚点行\n")
        # 译文：登记表与决策标题照中文那份抄一遍，简称是译名（与中文不同）；登记位只认中文那一份，规格里写中文简称必须判绿
        open(os.path.join(kb, "checks-owed.en.md"), "w", encoding="utf-8").write(
            "# Owed checks\n\n<!-- doc-lint:registry name-col=2 -->\n| No. | Name | Content |\n|---|---|---|\n| C120 | Anchor rot | old row |\n| C121 | Another entry | row |\n| C122 | Mirror test (HashMap ideal file system) | nested parentheses |\n")
        open(os.path.join(kb, "checks-owed.ja.md"), "w", encoding="utf-8").write(
            "# 未実施の検査\n\n<!-- doc-lint:registry name-col=2 -->\n| 番号 | 名称 | 内容 |\n|---|---|---|\n| C120 | アンカー腐敗 | 旧行 |\n| C121 | 別の一件 | 行 |\n| C122 | 照合（HashMap 理想ファイルシステム） | 括弧入れ子 |\n")
        open(os.path.join(kb, "decisions", "03-space-allocation.en.md"), "w", encoding="utf-8").write(
            "## D3 Space allocation —— Settled\n\n**Basis**: E10 (sample experiment) shows x.\n")
        # 依据：work 当一个 git 仓，上面写的都提交进 HEAD；之后再写的文件只在工作区里
        import subprocess

        def git(*arguments):
            subprocess.run(["git", "-C", work, "-c", "user.email=selftest@example.invalid", "-c", "user.name=selftest", *arguments],
                           check=True, capture_output=True)
        os.makedirs(os.path.join(work, "docs"))
        open(os.path.join(work, "docs", "committed.md"), "w", encoding="utf-8").write("提交过的一行\n")
        open(os.path.join(work, "docs", "grown.md"), "w", encoding="utf-8").write("第一行\n第二行\n")
        open(os.path.join(work, "docs", "edited.md"), "w", encoding="utf-8").write("甲\n乙\n丙\n")
        open(os.path.join(work, "docs", "staged.md"), "w", encoding="utf-8").write("子\n丑\n")
        open(os.path.join(work, "docs", "scoped-tracked.md"), "w", encoding="utf-8").write("原来的一行\n")
        git("init", "-q")
        git("add", "-A")
        git("commit", "-q", "-m", "样本：HEAD")
        open(os.path.join(work, "docs", "worktree-only.md"), "w", encoding="utf-8").write("只在工作区的一行\n")
        os.makedirs(os.path.join(work, "research", "prompts"))
        for judgment in ("batch-judge-5.md", "r1-main-verification.md"):   # 这一批新写、还没跟踪的判决与证据
            open(os.path.join(work, "research", "prompts", judgment), "w", encoding="utf-8").write("这一批的判决\n")
        open(os.path.join(work, "docs", "scoped.md"), "w", encoding="utf-8").write("登记进这一批范围的一行\n")
        open(os.path.join(work, "docs", "grown.md"), "w", encoding="utf-8").write("第一行\n第二行\n第三行\n第四行\n第五行\n")
        open(os.path.join(work, "docs", "edited.md"), "w", encoding="utf-8").write("甲\n乙改过\n丙\n")   # 已跟踪，第 2 行被别人改了
        open(os.path.join(work, "docs", "scoped-tracked.md"), "w", encoding="utf-8").write("这一批改过的一行\n")
        # 只暂存区与 HEAD 不同：暂存了第 2 行的改动与一个新文件，再把工作区那份改回 HEAD 的样子
        open(os.path.join(work, "docs", "staged.md"), "w", encoding="utf-8").write("子\n丑改过\n")
        open(os.path.join(work, "docs", "staged-new.md"), "w", encoding="utf-8").write("别人暂存的新文件\n")
        git("add", "docs/staged.md", "docs/staged-new.md")
        open(os.path.join(work, "docs", "staged.md"), "w", encoding="utf-8").write("子\n丑\n")
        open(os.path.join(work, ".claude", "batch-scope"), "w", encoding="utf-8").write("# 样本\ndocs/scoped.md\t# 这一批的改动\ndocs/scoped-tracked.md\t# 这一批的改动\n")

        def entry(file, old, new, basis="样本"):
            return {"file": file, "old": old, "new": new, "basis": basis}
        owed = ".claude/kb/checks-owed.md"
        owed_en, owed_ja = ".claude/kb/checks-owed.en.md", ".claude/kb/checks-owed.ja.md"
        cases = [
            ("干净的一条", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行，见 C121（另一笔） |")], 0),
            ("简称带嵌套括号", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 见 C122（对拍（HashMap 理想文件系统））和 C121（另一笔） |")], 0),
            ("嵌套括号简称写错", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 见 C122（对拍（别的名字）） |")], 1),
            ("嵌套括号简称只写了外层", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 见 C122（对拍） |")], 1),
            ("嵌套括号简称括号没配平", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 见 C122（对拍（HashMap 理想文件系统） |")], 1),
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
            ("译文登记位简称与中文不同", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 见 C121（另一笔） 与 C120（锚点腐化），决策见 D3（空间分配） |")], 0),
            ("变更史新形态", [entry(".claude/kb/decisions-history.md", "锚点行",
                                    "锚点行\n\n### 2026-09-26\n\n- 已定项 1：样本\n  - **结论**：乙。\n")], 0),
            # 译文：编号后面用半角括号写译名（.claude/kb/translation-glossary.md），不用全角括号；只判形态
            ("译文写半角译名", [entry(owed_en, "| C120 | Anchor rot | old row |", "| C120 | Anchor rot | new row, see C121 (Another entry) |"),
                               entry(owed_ja, "| C120 | アンカー腐敗 | 旧行 |", "| C120 | アンカー腐敗 | 新しい行、C121 (別の一件) を参照 |")], 0),
            ("译文译名带嵌套半角括号", [entry(owed_en, "| C120 | Anchor rot | old row |", "| C120 | Anchor rot | see C122 (Mirror test (HashMap ideal file system)) |")], 0),
            ("译文写全角中文简称", [entry(owed_en, "| C120 | Anchor rot | old row |", "| C120 | Anchor rot | see C121（另一笔） |")], 1),
            ("译文写全角日文译名", [entry(owed_ja, "| C120 | アンカー腐敗 | 旧行 |", "| C120 | アンカー腐敗 | C121（別の一件）を参照 |")], 1),
            ("译文编号没带译名", [entry(owed_en, "| C120 | Anchor rot | old row |", "| C120 | Anchor rot | see C121 |")], 1),
            ("译文译名带中文简称原样", [entry(owed_en, "| C120 | Anchor rot | old row |", "| C120 | Anchor rot | see C121 (另一笔) |")], 1),
            ("中文写半角括号简称", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 见 C121 (另一笔) |")], 1),
            # 依据：已跟踪而工作区或暂存区与 HEAD 不同（别的会话还没提交的改动）、又没登记在 .claude/batch-scope 的判红；
            # 与 HEAD 相同的、登记在这一批范围里的、显式放行的、未跟踪的新文件（这一批新写的判决与证据）判绿
            ("依据在 HEAD 里且与 HEAD 相同", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/committed.md 第 1 行")], 0),
            ("依据已跟踪且有没进这一批的改动", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/edited.md 写了 x")], 1),
            ("依据已跟踪且只暂存区有改动", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/staged.md 写了 x")], 1),
            ("依据是暂存区里的新文件", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/staged-new.md 写了 x")], 1),
            ("依据是未跟踪的新文件", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/worktree-only.md 写了 x")], 0),
            ("依据是未跟踪的新判决文件（写回员引这一批的判决）", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "判决 research/prompts/batch-judge-5.md:1 与 research/prompts/r1-main-verification.md")], 0),
            ("依据登记在这一批范围里（未跟踪）", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/scoped.md 写了 x")], 0),
            ("依据登记在这一批范围里（已跟踪且改过）", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/scoped-tracked.md 写了 x")], 0),
            ("依据引的行号没被改", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/grown.md:2")], 0),
            ("依据引的行号范围没被改", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/edited.md:3-3")], 0),
            ("依据引的行号被改了", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/edited.md:2")], 1),
            ("依据引的行号范围里有一行被改了", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/edited.md:1-2")], 1),
            ("依据引的行号只在工作区", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/grown.md:5")], 1),
            ("依据引的行号只暂存区改了", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/staged.md:2")], 1),
            ("依据引的行号只暂存区有别处改动", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/staged.md:1")], 0),
            ("依据里的路径哪儿都没有不判", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "research/prompts/archived-long-ago.md 与 `docs/*.md`")], 0),
        ]
        failures = []
        for label, entries, want in cases:
            problems = check_entries(entries, work)
            if (1 if problems else 0) != want:
                failures.append(f"{label}：应当{'判红' if want else '判绿'}，实际 {problems or '判绿'}")
        if check_entries([entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/edited.md 写了 x")], work, allowed=["docs/edited.md"]):
            failures.append("依据已跟踪且有改动、命令行 --allow-uncommitted 点名放行：应当判绿")
        if check_entries([entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/edited.md 写了 x")], work, allowed=["docs/"]):
            failures.append("依据已跟踪且有改动、命令行 --allow-uncommitted 以目录（docs/）放行：应当判绿")
        named_untracked = []
        check_entries([entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/worktree-only.md 与 docs/committed.md 与 docs/worktree-only.md:1")], work, untracked_out=named_untracked)
        if named_untracked != ["docs/worktree-only.md"]:
            failures.append(f"依据里的未跟踪新文件没按名单列出（该是 docs/worktree-only.md 一份、不重复）：{named_untracked}")
        # 跨规格：同一个编号在几份同批规格的新句子里状态词不一致。规格各写一条，文件不同（并行写回员按 kb 文件切开写）
        def spec_of(name, new, file=owed):
            return (name, [entry(file, "| C120 | 锚点腐化 | 旧行 |", new)])
        registry, not_numbers = load_registry(work)
        cross_cases = [
            ("跨规格状态词不一致", [spec_of("规格甲", "| C120 | 锚点腐化 | 见 C121（另一笔）已还清 |"),
                                    spec_of("规格乙", "| C120 | 锚点腐化 | C121（另一笔）未实现 |", ".claude/kb/decisions/03-空间分配.md")], 1),
            ("跨规格定的状态词不一致", [spec_of("规格甲", "D3（空间分配）已定"), spec_of("规格乙", "D3（空间分配）半定")], 1),
            ("跨规格状态词同族一致", [spec_of("规格甲", "| C120 | 锚点腐化 | 见 C121（另一笔）已还清 |"), spec_of("规格乙", "| C120 | 锚点腐化 | C121（另一笔）已实现 |")], 0),
            ("跨规格同一句里两个编号各写各的状态", [spec_of("规格甲", "C120（锚点腐化）已还清，C121（另一笔）未实现"),
                                                 spec_of("规格乙", "| C120 | 锚点腐化 | C121（另一笔）未实现 |")], 0),
            ("跨规格不变量编号状态不一致", [spec_of("规格甲", "I-4.4（意图只描述目标态） 的字段白名单判法已实现"), spec_of("规格乙", "I-4.4（意图只描述目标态） 的字段白名单判法未实现")], 1),
            ("跨规格引原话里编号前的旧状态词不算", [spec_of("规格甲", "「字段表未定」改成字段表已定（D3（空间分配） 已定项 2）"), spec_of("规格乙", "D3（空间分配）已定")], 0),
            ("跨规格引号里的旧状态词不算", [spec_of("规格甲", "I-8.4（重放幂等） 已实现、22 条状态列写着「未实现」"), spec_of("规格乙", "I-8.4（重放幂等） 已实现")], 0),
            ("跨规格不同编号各写各的状态", [spec_of("规格甲", "C120（锚点腐化）已还清"), spec_of("规格乙", "C121（另一笔）未实现")], 0),
            ("跨规格旧串里原有的句子不算", [("规格甲", [entry(owed, "| C120 | 锚点腐化 | 旧行 |\nC121（另一笔）未实现", "| C120 | 锚点腐化 | 新行 |\nC121（另一笔）未实现")]),
                                           spec_of("规格乙", "C121（另一笔）已还清")], 0),
        ]
        for label, specs, want in cross_cases:
            problems = cross_spec_problems(specs, registry, not_numbers)
            if (1 if problems else 0) != want:
                failures.append(f"{label}：应当{'判红' if want else '判绿'}，实际 {problems or '判绿'}")
        named = cross_spec_problems(cross_cases[0][1], registry, not_numbers)
        if cross_cases[0][1] and (len(named) != 1 or "规格甲" not in named[0] or "规格乙" not in named[0] or "已还清" not in named[0] or "未实现" not in named[0]):
            failures.append(f"跨规格状态词不一致：应当点名两份规格与两句话里的状态词，实际 {named}")
        markdown = os.path.join(work, "spec.md")
        open(markdown, "w", encoding="utf-8").write("## 条 1\n文件：`.claude/kb/checks-owed.md`\n旧串：\n```\n| C120 | 锚点腐化 | 旧行 |\n```\n新串：\n~~~\n| C120 | 锚点腐化 | 新行 |\n~~~\n依据：样本\n")
        parsed = load_spec(markdown)
        if len(parsed) != 1 or parsed[0]["old"] != "| C120 | 锚点腐化 | 旧行 |" or parsed[0]["new"] != "| C120 | 锚点腐化 | 新行 |":
            failures.append(f"markdown 规格没认对：{parsed}")
        # 命令行：--help / -h 打文件头的用法退 0；不认的选项退 2；规格读不了报文件名与出路、不抛 traceback
        import subprocess
        cli_cases = [(["--help"], 0, "用法："), (["-h"], 0, "用法："), (["--bogus", markdown], 2, "不认的选项：--bogus"),
                     ([os.path.join(work, "nothere.json"), "--root", work], 2, f"读不了规格 {os.path.join(work, 'nothere.json')}"),
                     ([work, "--root", work], 2, f"读不了规格 {work}"), ([markdown, "--root"], 2, "--root 后面没给值")]
        import json
        spec_files = {}
        for name, specification in (("spec-done.json", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 见 C121（另一笔）已还清 |")]),
                                    ("spec-open.json", [entry(".claude/kb/decisions/03-空间分配.md", "## D3 空间分配 —— 已定", "## D3 空间分配 —— 已定\n\nC121（另一笔）未实现")]),
                                    ("spec-worktree.json", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/edited.md 写了 x")]),
                                    ("spec-untracked.json", [entry(owed, "| C120 | 锚点腐化 | 旧行 |", "| C120 | 锚点腐化 | 新行 |", "docs/worktree-only.md 写了 x")])):
            spec_files[name] = os.path.join(work, name)
            open(spec_files[name], "w", encoding="utf-8").write(json.dumps(specification, ensure_ascii=False))
        cli_cases += [([spec_files["spec-done.json"], "--root", work], 0, "（1 份）"),
                      ([spec_files["spec-done.json"], spec_files["spec-open.json"], "--root", work], 1, "跨规格状态不一致"),
                      ([spec_files["spec-worktree.json"], "--root", work], 1, "依据是别的会话还没提交的改动"),
                      ([spec_files["spec-untracked.json"], "--root", work], 0, "依据是未跟踪的新文件 1 份：docs/worktree-only.md"),
                      ([spec_files["spec-worktree.json"], "--root", work, "--allow-uncommitted", "docs/edited.md"], 0, "（1 份）"),
                      ([spec_files["spec-worktree.json"], "--root", work, "--allow-uncommitted"], 2, "--allow-uncommitted 后面没给路径")]
        for arguments, want_code, want_text in cli_cases:
            child = subprocess.run([sys.executable, os.path.abspath(__file__), *arguments], capture_output=True, text=True)
            said = child.stdout + child.stderr
            if child.returncode != want_code or want_text not in said or "Traceback" in said or (want_code and "→ 怎么办" not in said):
                failures.append(f"命令行 {arguments} 应当退 {want_code}、说「{want_text}」（拒绝时带「→ 怎么办」、不抛 traceback），实际退 {child.returncode}：{said[-300:]}")
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 check_entries() / reference_problems() / pointer_problems() / parse_markdown_spec() 的判法；KB_SPEC_CHECK_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ kb-spec-check 自检通过：旧串次数、编号简称、段名撞登记、位置与时间指代、依据配对、变更史形态各有红绿样本，译文不当登记位，markdown 规格认得出（{len(cases) + 1} 种）；"
              f"译文编号写半角译名判绿、写全角中文简称判红；依据已跟踪且工作区或暂存区有没进这一批的改动判红（引的行号被改、暂存区里的新文件同样红）、与 HEAD 相同的、登记在 batch-scope 里的、未跟踪的新文件判绿（未跟踪的另列一行）；跨规格同一编号状态词不一致判红（{len(cross_cases)} 种）；"
              f"命令行 {len(cli_cases)} 种：--help、-h 退 0，不认的选项、读不了的规格、--root 缺值退 2 带出路，两份规格的跨规格比对与 --allow-uncommitted")
        return 0
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(sys.argv[1:]))
