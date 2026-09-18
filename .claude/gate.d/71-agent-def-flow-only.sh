#!/usr/bin/env bash
# gate-stage: agent 定义只写怎么做，不写记录与解释
#
# 管三处：`.claude/agents/*.md`、`.claude/agent-common.md`、`.claude/main-agent.md`。
# 规则在 `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」：定义只写步骤、约束、判据、交付；
# 为什么与是什么写进 `records/` 或 `.claude/kb/`，定义里不写、也不留解释的尾巴，要用到时写成一条读取指令
# （「开工先读：`.claude/kb/<某份>`「某节」」）。来历与第一次清理的经过在 `records/2026-09-16-subagent拆分提案.md` 第二十八节。
#
# 判五条，任一条不成立判红：
#   ① 记录小节：标题行的文字里带「历史」或「变更史」。
#   ② 带日期的行没指路：「」引号之外出现 `20\d\d-\d\d-\d\d` 的行，同一行没有一条指到文件的 `records/…` 或 `.claude/kb/…` 路径。
#      不认：「」里的日期——引一条规则小节名时名字里自带的日期（「多轮：…（2026-09-06 用户明令）」）是引用，不是记录；
#      只提到目录（`.claude/kb/` 后面直接是反引号或空白）或别处的脚本路径不算指路。
#   ③ 解释性段落：段落的头一行（剥掉列表记号、`>`、`**`、⚠️ 之后）以下面三类之一开头——
#      日期（一段以日期起头就是在记一件事）；
#      标签词 为什么 / 依据 / 理由 / 实测 / 经过 / 原因 / 来历 / 背景 / 历史 / 沿革 / 前情，后面紧跟 ：:（(，,。、 空白或「是」；
#      连词 因为 / 之所以。
#      不认：标签词后面直接接别的字（`背景材料路径（…）` 是输入项）。
#   ④ 解释性半句：在剥掉「」之后的行上判（引的小节名里带这些词不认，与 ② 同）；行内在 （(，,；;。 或 —— 之后（中间可隔空白与一个日期）紧接着
#      实测 / 试跑 / 踩过 / 撞上 / 撞过、因为 / 之所以，或 为什么 / 依据 / 理由 / 原因 / 经过 后跟 ：:或空白。
#      段落中间的一行（不是段落头一行）若按 ③ 的认法起头，也记在这一条里：「依据：…」接在角色句下一行，渲染出来是同一段的后半句。
#      不认：顿号之后（「写结论、依据与没做什么」是列举）；标签词后面直接接别的字（「为什么这么定」是名词短语）；
#      日期与这些词之间隔了别的字（「（2026-09-17 两次试跑…）」）——这一种由 ② 管。
#   ⑤ 词法说明：在剥掉「」之后的行上找下面五种，命中一处记一处——
#      「实测」后八个字以内跟数字（`实测[^，。；,;（）()「」]{0,8}\d`：实测数）；
#      轮名后十二个字以内跟一个分数（`[a-z][a-z0-9]*(?:-[a-z0-9]+)*-r\d+[^，。；）)]{0,12}\d+\s*/\s*\d+`：拿轮名替日期的实测数）；
#      「今天」后跟数字（`今天\s*\d`：现状数）；
#      （(，,；;。：: 或 —— 之后（中间可隔空白）紧接着 免得 / 以免 / 为了 / 所以（目的与因果的尾巴）；
#      「计划第 N 节」（`计划第[一二三四五六七八九十]+节`：不带文件路径的经过指路）。
#
# 都不判的：围栏（``` 或 ~~~）里的行；表格行（以 | 起头）不判 ③ ④ ⑤，② 照判。
# 段落怎么切：列表项（`- `、`1. `、`3b. `）各自成段，到下一个列表项、空行、标题、表格或围栏为止；不是列表的连续非空行算一段。
# 判不到的：这几条只认上面写的字面，下面这些认不出，靠三方审核与人看（72 号）——
#   ④ ⑤ 的那些词不在标点之后（「负载下实测过两次」「派你是为了…」）、在顿号之后，或「实测」「今天」后面跟的不是阿拉伯数字；
#   日期的替身（轮名、「今天」）后面跟的不是数或分数；表格行里的解释，带不带这些词都认不出；
#   括注、冒号、逗号引出的为什么（「（两道门禁同时跑，重阶段互相拖）」「：它不读共用约束」「，那类读数被抢了 CPU 就不作数」）；
#   格言（「发散是探索该有的样子」「延迟不是丢掉」）。
# 会误判的：说明与指令字面逐字相同的，这一道分不开——标点后当动词用的「试跑」、交付字段与列名里标点后的「实测」「依据：」「理由：」、
#   以「依据：」起头的输入项、标题里的「变更史」、反引号里写成日期的格式例子；是指令而被判红的，换一个不带这些词的说法。
#
# 判别力：`fixtures/71-agent-def-flow-only.sh/` 红样本五条各犯一处（⑤ 的五种各一行）、分布在三处被管文件里；
# 绿样本放了每条的「不认」形态，含一条点名的小节名里带「实测」的读取指令、一行带 ⑤ 字面的表格行。
#
#   bash .claude/gate.d/71-agent-def-flow-only.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
python3 - <<'PY'
import glob, os, re, sys

targets = sorted(glob.glob(".claude/agents/*.md"))
for extra in (".claude/agent-common.md", ".claude/main-agent.md"):
    if os.path.isfile(extra):
        targets.append(extra)
if not targets:
    print("  ! 这个仓里没有 .claude/agents/*.md、.claude/agent-common.md 或 .claude/main-agent.md，本阶段无对象可判")
    sys.exit(77)

DATE = re.compile(r"20\d\d-\d\d-\d\d")
POINTER = re.compile(r"(?:records|\.claude/kb)/[^\s`'\"）)]+")
QUOTED = re.compile(r"「[^「」]*」")

def without_quoted(text):
    previous = None
    while previous != text:
        previous, text = text, QUOTED.sub("", text)
    return text
HEADING = re.compile(r"^\s{0,3}#{1,6}\s+(.*)$")
FENCE = re.compile(r"^\s*(?:```|~~~)")
TABLE = re.compile(r"^\s*\|")
LIST_MARKER = re.compile(r"^[\s>]*(?:[-*+]\s+|\d+[a-z]?\s*[.)、]\s*)")
DECORATION = re.compile(r"^(?:[\s>*_`~]|⚠️|⚠|✗|✓|！|!)+")
PARAGRAPH_LABELS = ["为什么", "依据", "理由", "实测", "经过", "原因", "来历", "背景", "历史", "沿革", "前情"]
PARAGRAPH_CONJUNCTIONS = ["因为", "之所以"]
PARAGRAPH_LABEL_DELIMITERS = "：:（(，,。、 \t是"
HALF_SENTENCE = re.compile(
    r"(?:[（(，,；;。]|——)\s*(?:20\d\d-\d\d-\d\d\s*)?"
    r"(实测|试跑|踩过|撞上|撞过|因为|之所以|(?:为什么|依据|理由|原因|经过)(?=[：:\s]))"
)
LEXICAL_EXPLANATIONS = [
    ("「实测」后跟数", re.compile(r"实测[^，。；,;（）()「」]{0,8}\d")),
    ("轮名后跟分数", re.compile(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*-r\d+[^，。；）)]{0,12}\d+\s*/\s*\d+")),
    ("「今天」后跟数", re.compile(r"今天\s*\d")),
    ("标点后的目的或因果", re.compile(r"(?:[（(，,；;。：:]|——)\s*(?:免得|以免|为了|所以)")),
    ("「计划第 N 节」", re.compile(r"计划第[一二三四五六七八九十]+节")),
]

def paragraph_opener(first_line):
    text = DECORATION.sub("", LIST_MARKER.sub("", first_line, count=1))
    if DATE.match(text):
        return "日期"
    for word in PARAGRAPH_CONJUNCTIONS:
        if text.startswith(word):
            return word
    for word in PARAGRAPH_LABELS:
        if text.startswith(word):
            rest = text[len(word):]
            if rest == "" or rest[0] in PARAGRAPH_LABEL_DELIMITERS:
                return word
    return None

history_sections, dated_without_pointer, explanatory_paragraphs, explanatory_half_sentences = [], [], [], []
lexical_explanations = []
scanned_lines = dated_lines = dated_only_in_quotes = dated_with_pointer = lexical_lines = 0

for path in targets:
    lines = open(path, encoding="utf-8").read().split("\n")
    in_fence = False
    fenced = [False] * len(lines)
    breaks_paragraph = [False] * len(lines)
    for index, line in enumerate(lines):
        if FENCE.match(line):
            fenced[index] = breaks_paragraph[index] = True
            in_fence = not in_fence
        elif in_fence:
            fenced[index] = breaks_paragraph[index] = True
        elif not line.strip() or HEADING.match(line) or TABLE.match(line):
            breaks_paragraph[index] = True
    starts_paragraph = [
        not breaks_paragraph[index] and (index == 0 or breaks_paragraph[index - 1] or bool(LIST_MARKER.match(line)))
        for index, line in enumerate(lines)
    ]
    for index, line in enumerate(lines):
        if fenced[index]:
            continue
        scanned_lines += 1
        location = f"{path}:{index + 1}"
        heading = HEADING.match(line)
        if heading and ("历史" in heading.group(1) or "变更史" in heading.group(1)):
            history_sections.append(f"{location}：{line.strip()}")
        if DATE.search(line):
            dated_lines += 1
            if not DATE.search(without_quoted(line)):
                dated_only_in_quotes += 1
            elif POINTER.search(line):
                dated_with_pointer += 1
            else:
                dated_without_pointer.append(f"{location}：{line.strip()[:90]}")
        if not breaks_paragraph[index] and not starts_paragraph[index]:
            opener = paragraph_opener(line)
            if opener:
                explanatory_half_sentences.append(f"{location}（段落中间一行以「{opener}」起头）：{line.strip()[:90]}")
        unquoted_line = without_quoted(line)
        if not breaks_paragraph[index]:
            for match in HALF_SENTENCE.finditer(unquoted_line):
                start = max(0, match.start() - 12)
                explanatory_half_sentences.append(f"{location}（「{match.group(1)}」）：…{unquoted_line[start:match.end() + 24].strip()}…")
        if not TABLE.match(line):
            lexical_lines += 1
            for kind, pattern in LEXICAL_EXPLANATIONS:
                for match in pattern.finditer(unquoted_line):
                    start = max(0, match.start() - 12)
                    lexical_explanations.append(f"{location}（{kind}「{match.group(0).strip()}」）：…{unquoted_line[start:match.end() + 24].strip()}…")
    index = 0
    while index < len(lines):
        if not starts_paragraph[index]:
            index += 1
            continue
        end = index + 1
        while end < len(lines) and not breaks_paragraph[end] and not starts_paragraph[end]:
            end += 1
        opener = paragraph_opener(lines[index])
        if opener:
            explanatory_paragraphs.append(f"{path}:{index + 1}（以「{opener}」起头，{end - index} 行）：{lines[index].strip()[:90]}")
        index = end

failed = False
if history_sections:
    failed = True
    print(f"  ✗ {len(history_sections)} 个记录小节（标题带「历史」或「变更史」）——定义是流程，不是说明：")  # gate-lint:summary
    for entry in history_sections:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：把这一节整段搬进 records/2026-09-16-subagent拆分提案.md 或对应 kb，定义里删掉这一节。")
if dated_without_pointer:
    failed = True
    print(f"  ✗ {len(dated_without_pointer)} 行写了日期却没在同一行指路（实测与经过写在别处，这里只指过去）：")  # gate-lint:summary
    for entry in dated_without_pointer:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：把经过写进 records/2026-09-16-subagent拆分提案.md 或对应 kb，定义里只留指令；")
    print("               日期是引的规则小节名的一部分，就把小节名放进「」里原样引。")
if explanatory_paragraphs:
    failed = True
    print(f"  ✗ {len(explanatory_paragraphs)} 处解释性段落（以日期、「为什么」「依据」「实测」「经过」「因为」这类起头）：")  # gate-lint:summary
    for entry in explanatory_paragraphs:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：把经过写进 records/2026-09-16-subagent拆分提案.md 或对应 kb，定义里删掉这一段；")
    print("               是要读的规则或 kb，改成一条读取指令「开工先读：`文件`「小节」」。")
if explanatory_half_sentences:
    failed = True
    print(f"  ✗ {len(explanatory_half_sentences)} 处解释性半句（指令后面挂着「实测」「因为」「依据」这类尾巴）：")  # gate-lint:summary
    for entry in explanatory_half_sentences:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：指令留下，尾巴删掉；尾巴里的经过写进 records/2026-09-16-subagent拆分提案.md 或对应 kb，")
    print("               要读的规则或 kb 改成一条读取指令「照 `文件`「小节」办」。")
if lexical_explanations:
    failed = True
    print(f"  ✗ {len(lexical_explanations)} 处词法说明（「实测」「今天」后跟数、轮名后跟分数、标点后的「免得」「以免」「为了」「所以」、「计划第 N 节」）：")  # gate-lint:summary
    for entry in lexical_explanations:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：数、经过与现状删掉，要留的写进 records/2026-09-16-subagent拆分提案.md 或对应 kb；目的与因果的尾巴删掉；")
    print("               执行者要照它分支的前提不删，改写成一条指令（条件 → 动作）。")
if failed:
    sys.exit(1)
print(f"  ✓ 定义只写怎么做（{len(targets)} 份文件 {scanned_lines} 行；{dated_lines} 行带日期：{dated_only_in_quotes} 行的日期只在「」引的小节名里，"
      f"{dated_with_pointer} 行同行指路；记录小节 0、解释性段落 0、解释性半句 0；词法说明判了 {lexical_lines} 行（围栏与表格行不判），命中 0）")
PY
