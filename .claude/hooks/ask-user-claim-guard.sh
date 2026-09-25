#!/usr/bin/env bash
# PreToolUse hook（AskUserQuestion）：弹窗问句里把推出来的话写成事实的，执行前拒绝（退出 2，stderr 写是哪一句与出路）。
# hook-events: PreToolUse
# gate-similar: runner-dispatch-guard.sh 同样在 PreToolUse 上逐句判一段给人看的中文、引号里的片段换成占位不判；但它挂 Agent|Task、判派发提示有没有点名岔路与有没有要子 agent 跑重型测试（动词加宾语），这里挂 AskUserQuestion、判断言词加出处，两边的句子判据没有一条相同。引号遮掉那一步两边各一份、写法不同（它逐字符用栈、不遮反引号，这里先遮反引号再按成对的引号反复替换）；抽成共用模块要改 runner-dispatch-guard.sh，这一次不碰别的钩子，交主 agent 定要不要抽
# gate-similar: write-guard.sh 同样在执行前按字面拒一类写法（撇号类角标），但挂 Write|Edit、判的是落进文件的字与写哪个文件；弹窗问句不落盘，它看不见，并进去要让它多认一个工具、多一套与写文件无关的判据
# gate-similar: continuation-guard.sh 挂 PreToolUse[SendMessage]，判收件的子 agent 在会话记录里交回过没有、被中断过没有；这里判弹窗里的句子，两边的对象与输入没有交集
# gate-overlap:copy-kept continuation-guard.sh main() 那十行是钩子入口（认 --selftest、从 stdin 读 hook 的 JSON、读不出就放行、拒绝时把说明写进 stderr），不带判据；抽成共用模块要同时改 continuation-guard.sh 与 runner-dispatch-guard.sh，这一次的任务不许碰别的钩子，交主 agent 定要不要抽
# gate-overlap:copy-kept runner-dispatch-guard.sh 同 continuation-guard.sh 那一条：main() 入口十行三个钩子各一份，抽共用模块要改它，这一次不碰别的钩子
#
# 为什么：2026-09-23 主 agent 问用户 alloc-basis 岔路 7 时写「可达状态里 defer 一项不可能为 0……重新搭环境也造不出来」，后半句是推的；
# 用户据此定了案，2026-09-25 E156 第 3 次重跑在可达状态上读到 0（records/2026-09-16-subagent拆分提案.md 第四十节第 25 行）。
# main-agent.md「一轮怎么开、怎么收」第 5 条要弹窗里每句事实写出处、推的写明「推的，没量过」；这道闸让两样都没写的断言句送不到用户面前。
#
# 判法：扫 tool_input.questions 里每一问的 question 与每个 options[].description（header、label 不扫），按句切——
#   。；！？与换行断句；反引号里的不断句（代码里的分号不是句末），反引号没闭合时一直到行尾算一句。每一句：
#   ① 找断言词：不可能、造不出、从来不、从来没有、一定、必然、永远不、绝不会、恒为。
#      找之前先把反引号里的代码、引号里的片段（「」『』“”‘’与 ASCII 双引号，成对的才算）换成 □：那是写代码或引别人的话，不是自己断言；
#      「不一定」「一定程度」不算断言词（前者就是没把握，后者是量词）。一个断言词都没有的句子放行。
#   ② 句子里有「推的」「没量过」「推测」「估计」「粗估」之一的放行。
#   ③ 句子里有出处的放行，出处是下面任一样（在原句上找，引号与反引号里的也算）：
#      反引号里的路径（带 / 的，或带登记过的扩展名 PATH_EXTENSIONS 的文件名）或命令（全 ASCII、至少两段，第一段是小写命令名、
#      ./ 开头的路径或 VAR= 赋值）；文件:行号（文件带 / 或带登记过的扩展名，冒号全角半角都认）；research/results/ 下的文件名；
#      name= 开头的产物行；「实测」「量过」「产物」「输出」前后 MEASUREMENT_WINDOW 个字以内带着一个数或路径，
#      而且那个数或路径与这个词之间没有隔着断言词（「输出一定为 0」里 0 与「输出」之间隔着「一定」，不算）。
#   ①有、②③都没有的句子逐句列出，退出 2；一句都没有就放行。stdin 读不出 JSON、questions 不是列表的放行。
# 管不到的：出处是不是真的、数是不是那个产物里的、「推的」写得如不如实——这几样靠人（verify-before-claiming.md）；
#   换个说法（「肯定」「绝对」「百分之百」）就过去了，词表只认上面九个；出处写在前一句、断言写在后一句的照拒。
#
#   ask-user-claim-guard.sh             # 从 stdin 读 hook 的 JSON
#   ask-user-claim-guard.sh --selftest  # 走一遍必拒与必放的句子，另有三例走真实的 stdin 入口
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（写成 `python3 - <<PY` 时程序占了标准输入，JSON 读不到，判定一律放行）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, subprocess, sys

ASSERTION_WORDS = ("不可能", "造不出", "从来不", "从来没有", "一定", "必然", "永远不", "绝不会", "恒为")
# 字面上带着断言词、说的却不是断言：先换掉再找
NOT_ASSERTION_PHRASES = ("不一定", "一定程度")
HEDGE_WORDS = ("推的", "没量过", "推测", "估计", "粗估")
MEASUREMENT_WORDS = ("实测", "量过", "产物", "输出")
MEASUREMENT_WINDOW = 16
SENTENCE_ENDINGS = "。；！？"
PLACEHOLDER = "□"
PATH_EXTENSIONS = ("rs", "md", "sh", "py", "tsv", "csv", "out", "txt", "log", "json", "jsonl", "toml", "yaml", "yml",
                   "conf", "litmus", "cat", "diff", "patch", "lock", "img")
ASCII_PATH_CHARACTER = r"[A-Za-z0-9_.-]"
EXTENSION_ALTERNATION = "|".join(PATH_EXTENSIONS)
# 路径：带一个 /，或以登记过的扩展名收尾的文件名
PATH_LIKE = re.compile(
    rf"{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{ASCII_PATH_CHARACTER}+"
    rf"|(?<![A-Za-z0-9_.-])[A-Za-z0-9_-]{ASCII_PATH_CHARACTER}*\.(?:{EXTENSION_ALTERNATION})(?![A-Za-z0-9])")
FILE_AND_LINE = re.compile(
    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{ASCII_PATH_CHARACTER}+"
    rf"|[A-Za-z0-9_-]{ASCII_PATH_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))[:：]\d+")
RESULTS_FILE = re.compile(rf"research/results/{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]")
PRODUCT_LINE = re.compile(r"(?<![A-Za-z0-9_])name=[^\s`'\"]")
CODE_SPAN = re.compile(r"`[^`\n]*`")
QUOTED_SPAN = re.compile(r"「[^「」]*」|『[^『』]*』|“[^“”]*”|‘[^‘’]*’|\"[^\"\n]*\"")
COMMAND_FIRST_WORD = re.compile(r"[A-Za-z_][A-Za-z0-9_]*=\S*|\.{0,2}/\S+|[a-z][a-z0-9_.+-]*")

def sentences_of(text):
    """按。；！？与换行切句；反引号里的不切，换行总是切。"""
    sentences, current, inside_code = [], [], False
    for character in text:
        if character == "\n":
            sentences.append("".join(current))
            current, inside_code = [], False
            continue
        current.append(character)
        if character == "`":
            inside_code = not inside_code
        elif not inside_code and character in SENTENCE_ENDINGS:
            sentences.append("".join(current))
            current = []
    sentences.append("".join(current))
    return [sentence.strip() for sentence in sentences if sentence.strip()]

def without_code_and_quotations(sentence):
    """反引号里的代码与成对引号里的片段换成 □；嵌套的由外层一次吞掉，没闭合的开引号原样留着。"""
    masked = CODE_SPAN.sub(PLACEHOLDER, sentence)
    while True:
        replaced = QUOTED_SPAN.sub(PLACEHOLDER, masked)
        if replaced == masked:
            return masked
        masked = replaced

def assertion_spans(text):
    """[(起, 止, 断言词)]，按出现位置排；「不一定」「一定程度」先换成等长的占位，位置不变。"""
    for phrase in NOT_ASSERTION_PHRASES:
        text = text.replace(phrase, PLACEHOLDER * len(phrase))
    spans = []
    for word in ASSERTION_WORDS:
        spans += [(match.start(), match.end(), word) for match in re.finditer(re.escape(word), text)]
    return sorted(spans)

def is_path_or_command(code):
    code = code.strip()
    if PATH_LIKE.search(code):
        return True
    words = code.split()
    return len(words) >= 2 and code.isascii() and COMMAND_FIRST_WORD.fullmatch(words[0]) is not None

def number_or_path_in(fragment):
    return re.search(r"\d", fragment) is not None or PATH_LIKE.search(fragment) is not None

def measurement_backed(sentence):
    """「实测」「量过」「产物」「输出」前后 MEASUREMENT_WINDOW 个字以内有数或路径，中间没隔着断言词。"""
    assertions = assertion_spans(sentence)
    for word in MEASUREMENT_WORDS:
        for match in re.finditer(re.escape(word), sentence):
            after_end = min(len(sentence), match.end() + MEASUREMENT_WINDOW)
            after_end = min([start for start, _, _ in assertions if match.end() <= start < after_end] + [after_end])
            before_start = max(0, match.start() - MEASUREMENT_WINDOW)
            before_start = max([end for _, end, _ in assertions if before_start < end <= match.start()] + [before_start])
            if number_or_path_in(sentence[match.end():after_end]) or number_or_path_in(sentence[before_start:match.start()]):
                return True
    return False

def has_provenance(sentence):
    if any(is_path_or_command(match.group(0)[1:-1]) for match in CODE_SPAN.finditer(sentence)):
        return True
    if FILE_AND_LINE.search(sentence) or RESULTS_FILE.search(sentence) or PRODUCT_LINE.search(sentence):
        return True
    return measurement_backed(sentence)

def unsupported_claims(text):
    """[(句子, [断言词])]：带断言词、同一句里没有出处、也没写明是推的那些句子。"""
    found = []
    for sentence in sentences_of(text):
        words = sorted({word for _, _, word in assertion_spans(without_code_and_quotations(sentence))}, key=ASSERTION_WORDS.index)
        if not words or any(hedge in sentence for hedge in HEDGE_WORDS) or has_provenance(sentence):
            continue
        found.append((sentence, words))
    return found

def places_to_scan(tool_input):
    """[(给人看的位置, 文字)]：每一问的问句与每个选项的说明。"""
    places = []
    questions = tool_input.get("questions") if isinstance(tool_input, dict) else None
    for question_number, question in enumerate(questions if isinstance(questions, list) else [], 1):
        if not isinstance(question, dict):
            continue
        places.append((f"第 {question_number} 问的问句", question.get("question")))
        for option in question.get("options") if isinstance(question.get("options"), list) else []:
            if isinstance(option, dict):
                places.append((f"第 {question_number} 问选项「{option.get('label') or ''}」的说明", option.get("description")))
    return [(place, text) for place, text in places if isinstance(text, str)]

def flagged(tool_input):
    return [(place, sentence, words) for place, text in places_to_scan(tool_input) for sentence, words in unsupported_claims(text)]

def decide(hook_input):
    """返回 (0, None) 放行；(2, 说明) 拒绝。"""
    findings = flagged(hook_input.get("tool_input") or {})
    if not findings:
        return 0, None
    listed = "\n".join(f"   {place}：{sentence}（断言词：{'、'.join(words)}）" for place, sentence, words in findings)
    return 2, (f"✗ 弹窗里有 {len(findings)} 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：\n"
               f"{listed}\n"
               "→ 怎么办：给这句写出处（产物整行、命令与输出、文件:行号）；推出来的写明『推的，没量过』。"
               "出处与断言要写在同一句里（按。；！？与换行断句）；只是转述别人的原话，放进「」里。")

def selftest(hook_dir):
    original = "可达状态里 defer 一项不可能为 0……重新搭环境也造不出来"
    one_sourced_one_not = "实测 `research/results/e156-r3.out:12` 这一格 defer=0。重新搭环境也造不出别的值"
    # (说明, tool_input, 应当被拒的句子——空列表就是应当放行)
    cases = [
        ("必拒：原话那句", {"questions": [{"question": original, "options": [{"label": "采纳", "description": "自证用造的基底"}]}]}, [original]),
        ("必拒：「一定会红」无出处", {"questions": [{"question": "这条改动一定会红，要不要先回退？", "options": []}]}, ["这条改动一定会红，要不要先回退？"]),
        ("必拒：选项说明里的「从来不」", {"questions": [{"question": "岔路 3 选哪一条？", "options": [
            {"label": "甲", "description": "这条路径从来不会被走到"}, {"label": "乙", "description": "加一条会红的用例"}]}]},
         ["这条路径从来不会被走到"]),
        ("必拒：两句里只有一句有出处，拒没出处的那句", {"questions": [{"question": one_sourced_one_not, "options": []}]},
         ["重新搭环境也造不出别的值"]),
        ("必拒：「输出」与数之间隔着断言词", {"questions": [{"question": "输出一定为 0", "options": []}]}, ["输出一定为 0"]),
        ("必拒：反引号里只有标识符不算出处", {"questions": [{"question": "`defer` 恒为正", "options": []}]}, ["`defer` 恒为正"]),
        ("必拒：出处在前一行、断言在后一行", {"questions": [{"question": "见 `crates/core/src/mount.rs:120`\n它必然先于回收", "options": []}]},
         ["它必然先于回收"]),
        ("必放：带 research/results/x.out:12 的", {"questions": [{"question": "`research/results/x.out:12` 那一行 defer 恒为 1，一定不为 0", "options": []}]}, []),
        ("必放：带「推的，没量过」的", {"questions": [{"question": original + "（推的，没量过）", "options": []}]}, []),
        ("必放：没有断言词的", {"questions": [{"question": "岔路 7 选哪一条？", "options": [{"label": "采纳", "description": "自证用造的基底"}]}]}, []),
        ("必放：断言词在反引号里的", {"questions": [{"question": "这道闸拦 `不可能` 这类词，要不要把 `一定` 也留在词表里？", "options": []}]}, []),
        ("必放：断言词在「」里被当成引用的", {"questions": [{"question": "判决里原话是「重新搭环境也造不出来」，这句要撤回吗？", "options": []}]}, []),
        ("必放：「不一定」是没把握", {"questions": [{"question": "换一个取样点不一定会红", "options": []}]}, []),
        ("必放：实测带着数", {"questions": [{"question": "实测 5 轮都红，一定会红", "options": []}]}, []),
        ("必放：name= 开头的产物行", {"questions": [{"question": "那一行是 name=e156 defer=0，所以它不可能恒为正", "options": []}]}, []),
        ("必放：反引号里的命令", {"questions": [{"question": "`cargo test -p singlefs-core mount` 全绿，这条路径一定走到了", "options": []}]}, []),
        ("必放：反引号里的路径", {"questions": [{"question": "回收写在 `crates/core/src/mount.rs` 的挂载路径里，defer 一定会被清掉", "options": []}]}, []),
        ("必放：不在反引号里的文件:行号", {"questions": [{"question": "见 transaction.rs：120，回收一定排在提交之后", "options": []}]}, []),
        ("必放：research/results/ 下的文件名", {"questions": [{"question": "research/results/e156-r3.out 那一格必然是 0", "options": []}]}, []),
        ("必放：没有 questions", {}, []),
    ]
    results = []
    for label, tool_input, want in cases:
        got = [sentence for _, sentence, _ in flagged(tool_input)]
        results.append((label, want, got))
    script = os.path.join(hook_dir, "ask-user-claim-guard.sh")
    for label, tool_input, want_code, want_in_stderr in (
            ("stdin：原话那句拒绝", cases[0][1], 2, "重新搭环境也造不出来"),
            ("stdin：两句里拒没出处的那句", cases[3][1], 2, "重新搭环境也造不出别的值"),
            ("stdin：带出处的放行", cases[7][1], 0, "")):
        completed = subprocess.run(["bash", script], input=json.dumps({"tool_name": "AskUserQuestion", "tool_input": tool_input}),
                                   capture_output=True, text=True)
        stderr_ok = want_in_stderr in completed.stderr if want_in_stderr else completed.stderr == ""
        results.append((label, (want_code, True), (completed.returncode, stderr_ok)))
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 sentences_of() / without_code_and_quotations() / assertion_spans() / has_provenance() 的判法与 stdin 入口，改完再跑 --selftest")
        return 1
    print(f"  ✓ 自检通过（查了 {len(results)} 种）：带断言词、同一句里没有出处也没写「推的」的句子拒绝，只拒没出处的那一句；"
          "出处（反引号里的路径或命令、文件:行号、research/results/ 下的文件、name= 产物行、实测带数）、「推的，没量过」、"
          "反引号与「」里的断言词、「不一定」放行；三例走真实的 stdin 入口")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except ValueError:
        return 0
    if not isinstance(hook_input, dict):
        return 0
    code, message = decide(hook_input)
    if code:
        print(message, file=sys.stderr)
    return code

sys.exit(main())
PY
