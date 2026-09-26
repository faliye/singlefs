#!/usr/bin/env bash
# PreToolUse hook（Agent）：派 experiment-runner 时，派发提示必须说清这一段为哪几行岔路而跑。另两条：派发提示要子 agent 跑重型测试的拒绝；
# 派 kb-scribe、implementation-writer 时，它要改的文件在一轮还没判完的三方开工快照里的拒绝。
#
# 为什么：2026-09-17 两个计数实验派了十一次执行员，每次交回主 agent 只问「还缺什么」就接着派；
# 岔路 1、3、4、7 能判的数在开跑四个多小时时已经齐了，之后两个多小时的数不改变任何选择
# （records/2026-09-17-已分配口径三方与两个实验.md 第六节）。规则写在 .claude/rules/three-way-inference.md
# 「交岔路时写岔路单，派实验时带上它」；这道闸让「续派之前先对着岔路单判够不够」跳不过去。
#
# 判法：不是派 experiment-runner 的放行；派发提示写着「只修锚点」的放行（那种活没有登记与岔路）。
#   ① 派发提示里没有「这一段回答的岔路：<非空>」一行，拒绝；
#   ② 登记对应的实验已经有实验页（.claude/kb/experiments/<号>-*.md，第一段交回才会有）算续做：
#      没有「上一段岔路表里还差：<非空>」一行、或写的是「无 / 没有」，拒绝——一行都不差就该交岔路表，不续派。
# 实验号从派发提示里的 research/prompts/e<号>-preregistration.md 或 e<号>-r<n>-prereg.md 取。
#
# 另一条，管派任何 agent：派发提示里有一句要它跑重型测试的，拒绝。用户 2026-09-24 定「subagent任务派发的门禁里面写上 禁止跑0层测试等这种重测试
# 就跑自己相关的测试就好了」「项目内 全量崩溃 qemu herd7 等重型的测试任务 禁止平时调用 仅仅在每次代码提交时候跑 或者要求时候再跑」，
# 更正「崩溃验证员和门禁分诊员 跑各自的部分就好了」；执行时的闸是 heavy-test-guard.sh，这一条让派发那一刻就说不出口。
#   只在「动词的宾语就是重型阶段」时才算要它跑。判法逐行、逐句、逐分句，全是字面规则，不用模型（每条规则的词表是下面同名的常量）：
#   ① 引用不判：一行里被「」『』“”‘’或 ASCII 双引号括起来的片段先换成 □；反引号里的代码不看引号（那里的引号是命令的一部分），
#      一行里没闭合的开引号不算，原样留着判。
#   ② 按。！？；!?; 切句；句子里出现转述的开头（REPORTED_SPEECH_OPENER：「……说，」「……说：」「报告里写 / 报告说」「报告：」「报告……」「原句」「原话」「转述」），
#      从那里到句末不判。
#   ③ 切分句：全角括号里的每一段单独成一个分句，括号外的前后两半拼回去；都再按，与「, 」切。
#   ④ 分句里有否定词（NEGATION：不、别、禁止、严禁、没法、没有办法、无法、不能、不许、勿，「没 / 没有」只在紧挨动词时算；「不变量」「不同」「别人」「判别」
#      这类复合词、「别忘了」「不要漏」与「有没有」「是不是」这类正反问不算），或分句开头是别人（THIRD_PARTY_SUBJECT_AT_CLAUSE_START：主 agent、别的会话、用户、另外两个验证员）
#      而分句里没有「你」，整个分句不判。
#   ⑤ 动词（RUN_VERB）：跑、复跑、重跑、执行、运行、bash、起。「跑前」「实验复跑」「可复跑命令」「执行员」「运行时」「一起」「起来」「从……起」不算；
#      动词后面紧跟「的」是定语（「只在提交时才执行的重型阶段」「跑出的标记」），前面紧挨「正在 / 在 / 在后台」是在说正在发生的事，都不算。
#   ⑥ 宾语在动词后面：到下一个动词为止，重型阶段的名字前面至多 OBJECT_LEAD_LIMIT 个非 ASCII 字、没有 OBJECT_LEAD_BREAK（看、确认、证明、时、前、后、再……），
#      不是 > / >> 的重定向目标；名字后面紧跟 EXCLUDED_AFTER_NAME（之外、时、里、「的」加判定段这类别的东西）的，宾语是别的东西，不算。
#   ⑦ 宾语在动词前面（跑、复跑、重跑、执行、运行；「QEMU 跑一遍」「`check.sh` 交回前跑一次」「把 54 号跑一遍」）：两者之间至多 TOPIC_GAP_LIMIT 个字、
#      没有 TOPIC_GAP_BREAK（只在、才、由、归、是、被、把、改、看……）；名字前面紧挨「被」（施事）或「照 / 按 / 依」（介词宾语）、动词后面紧跟代词宾语（「跑它」）的不算。
#   ⑧ `cargo test --all` / `--workspace` 这种命令字面本身就是要跑，不要动词。
#   重型阶段的名字：层 0（层 0 / 0 层 / layer0 / 54 号）、QEMU（qemu / vm-bench / 55 号）、herd7（herd7 / lkmm / 57 号）、crates 变异整表
#   （crates/mutations.tsv / 变异整表 / 59 号）、全量测试（check.sh / 全量测试）、整轮门禁（gate.sh / gate-staged.sh / 整轮门禁）、全部实验复跑（87 号）、E152 装置。
#   `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，提示里要它跑不拦。
#   crash-verifier 放行 QEMU、herd7、crates 变异整表那几句（层 0 不归它：快档在整轮门禁里，全量由主 agent 跑），gate-triage 放行整轮门禁与全部实验复跑那几句（它们各自那一份，执行时还要带
#   SINGLEFS_HEAVY_TESTS=commit / user-request，由 heavy-test-guard.sh 判）；其余一律拒。
#   会误拒：名字在前、动词在后、中间没有 TOPIC_GAP_BREAK 的说明句（「54 号在提交时跑全量」「54 号跑全量要四十分钟」），改写成「只在……才跑」或用「」括起来；
#   否定词在括号外、重型阶段的名字在括号里的（「不跑重型测试（check.sh、`cargo test --workspace`）」）：按 ③ 括号里单独成分句，否定词不在那个分句里，照拒；
#   否定词与名字写进同一个分句（「不跑 check.sh、`cargo test --workspace`」），或只指清单出处不列名字。
#   会漏：同一分句里另有否定词的指令（「不改代码直接跑 54 号」）、宾语是代词的（「跑它」）、名字离动词太远的、没有动词的清单行（「验证负载：……层 0 各流快档」）；
#   这些执行时 heavy-test-guard.sh 照拒。
#
# 第三条，管派 kb-scribe 与 implementation-writer：它要改的文件落在一轮还没判完的三方开工快照里，拒绝。规矩是 .claude/rules/implementation-workflow.md
# 「代码轮派腿之前记一份开工快照」：腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。为什么：2026-09-25 m2-safety-r1 的腿还在跑时派了书记员，
# 规格改的两份 kb 都在那一轮的快照里，这条之前只靠主 agent 记得（records/2026-09-16-subagent拆分提案.md 第四十节第 27 行）。
#   ① 开着的轮：research/prompts/<轮>-snapshot/ 在、research/prompts/_<轮>-body.md 在、research/prompts/<轮>-main-verification.md 不在；
#      有快照、没有正文的（普查一类，不是三方轮）不算。
#   ② 快照清单：快照目录里文件名含 sha256 的每一份，按「<哈希>  <路径>」逐行读（哈希是 64 位十六进制，路径前的 * 去掉）；路径从第一段 .claude/ 或 crates/ 起取，
#      ./.claude/…、tree/crates/…、defs/.claude/… 都归一成仓库根相对路径。不是这个形状的行、路径里没有 .claude/ 或 crates/ 的行跳过，不判红。
#   ③ 要改的文件：派发提示原文，加上提示里点名的 /tmp/claude-<uid>/ 下的规格文件（是文件就读进来；不在的、是目录的跳过）。快照清单里的哪条路径在这些文字里
#      出现（前后不连着别的路径字符；项目根起的绝对路径与 ./ 前缀先去掉），就算要改它。
#   ④ 有交集就拒绝（退出码 2，stderr 列出哪一轮、哪几个文件、各出现在提示还是哪份规格里）。放行口：派发提示里有一行以「快照冲突已判：<轮名> <理由>」开头，
#      轮名就是撞上的那一轮、理由非空；撞了几轮就要几行。
#   ⑤ 这一条自己出错（快照清单或规格读不了、别的异常）不拒绝：放行，错误写进 stderr。hook 的 JSON 解析不了同样放行、写 stderr。
#   会误拒：只为读、不为改而点名的快照文件也算（实现员提示里「压着的条款」列的 kb 路径、「别的会话在改、你不碰」的 crates 路径），用放行行写明。
#   会漏：只写目录或只写文件名、不写到仓库根相对路径的；规格放在 /tmp/claude-<uid>/ 之外的；规格文件里再指到的下一层文件。
#
#   runner-dispatch-guard.sh             # 从 stdin 读 hook 的 JSON
#   runner-dispatch-guard.sh --selftest  # 在临时目录里走一遍放行与拒绝；RUNNER_DISPATCH_GUARD_DISABLE_CHECK=1（三条都关）、
#                                        # RUNNER_DISPATCH_GUARD_DISABLE_HEAVY_TEST=1（只关重型测试那一条）或 RUNNER_DISPATCH_GUARD_DISABLE_SNAPSHOT=1
#                                        # （只关快照那一条）时自检必须判红；RUNNER_DISPATCH_GUARD_BREAK=snapshot-no-normalize / snapshot-spec-unread /
#                                        # snapshot-closed-round-open / snapshot-body-optional / snapshot-valve-ignored / snapshot-valve-loose /
#                                        # snapshot-any-agent / snapshot-loose-lines / snapshot-no-edges / snapshot-unreadable-denies /
#                                        # snapshot-unreadable-aborts / snapshot-missing-spec-denies / json-error-silent / json-error-denies
#                                        # 各自也必须让它红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import glob, json, os, re, shutil, subprocess, sys, tempfile

NONE_WORDS = {"", "无", "没有", "无。", "-", "—", "空", "none"}

def labelled_value(prompt, label):
    match = re.search(label + r"[：:]\s*(.*)", prompt)
    return None if match is None else match.group(1).strip()

SENTENCE_BOUNDARY = re.compile(r"[。！？；!?;]+")
# 分句：逗号切；全角括号里的一段单独成一个分句，括号外的前后两半拼回一句
CLAUSE_BOUNDARY = re.compile(r"[，（）]|,\s")
FULL_WIDTH_PARENTHETICAL = re.compile(r"（[^（）]*）")
# 引号：开 → 关。ASCII 双引号开关同字；ASCII 单引号不算（英文撇号与 shell 引号都用它）
QUOTE_CLOSING = {"「": "」", "『": "』", "“": "”", "‘": "’", '"': '"'}
QUOTED_FRAGMENT_PLACEHOLDER = "□"
# 转述的开头：从这里到句末是别人的话，不判
REPORTED_SPEECH_OPENER = re.compile(
    r"(?<![如是话再来])说\s*[，：,:]"
    r"|(?:报告|记录|判决|交回|原文)(?:里|中|上)?(?:说|称|提到|写着|写(?![进到入成回清明]))"
    r"|报告\s*(?:[：:]|…)"
    r"|原句|原话|转述")
RUN_VERB = re.compile(r"(?<!实验)(?<!可)复跑(?!命令|登记)|重跑|(?<![复重])跑(?!前)|执行(?!员)|运行(?!时)|(?<![\w./-])bash(?![\w-])"
                      r"|(?<![一看想说对引提发谈兴升崛])起(?![来码点初因见草名作])")
# 名字在前、动词在后时认的动词（bash 与「起」的宾语只在后面）
TOPIC_VERBS = {"复跑", "重跑", "跑", "执行", "运行"}
FROM_TIME_BEFORE_QI = re.compile(r"从[^，：；。]{0,20}$")
ATTRIBUTIVE_AFTER_VERB = re.compile(r"(?:完|好|过|了|出|到)?的")
# 否定：分句里有这些词就整句不判。「不变量」「不同」「别人」「判别」这类复合词与「别忘了」「不要漏」不算，「有没有」「是不是」这类正反问先换掉
NEGATION = re.compile(r"没有办法|没法|无法|不能|不许|禁止|严禁|勿"
                      r"|不(?!变|同|止|少|仅|但|管|论|过|断|久|然|得不|要忘|要漏)"
                      r"|(?<![分区特级类个识告差性派辨鉴判甄离送道])别(?![人的处家名称忘漏])"
                      r"|没有?(?=\s*(?:复跑|重跑|跑|执行|运行))")
A_NOT_A_QUESTION = re.compile(r"([\u4e00-\u9fff])([不没])\1")
# 动词前紧挨着「正在 / 在 / 在后台」是在说正在发生的事（「54 号正在跑层 0 全量」「有一个层 0 全量在后台跑」），不是要它跑；「现在」不算
PROGRESSIVE_BEFORE_VERB = re.compile(r"(?:正在|(?<!现)在)(?:\s*后台)?(?:\s|`[^`]*`)*$")
# 名字在前、动词在后：两者之间至多这么多个字，而且没有 TOPIC_GAP_BREAK 里那些另起一件事、换了施事或说「只在……才」的词
TOPIC_GAP_LIMIT = 20
TOPIC_GAP_BREAK = re.compile(r"只在|才|看|确认|证明|检查|核对|核实|对照|贴|写|读|改|交给|报告|然后|由|归(?!属)|属于"
                             r"|(?<![但于就总还凡要若正倒硬])是|被|让|叫|把|等|负责|用来")
# 动词后面紧跟代词宾语（「都要跑它」）：前面的名字不是它的宾语
PRONOUN_OBJECT_AFTER_VERB = re.compile(r"\s*(?:它|他|这|那|自己)")
# 名字前面紧挨「照 / 按 / 依」：名字是介词的宾语（「照门禁 59 号那一套判据跑」），不是要跑的东西
PREPOSITION_BEFORE_NAME = re.compile(r"(?:照|按|依|像)(?:照)?\s*(?:门禁\s*)?$")
OBJECT_LEAD_LIMIT = 6
OBJECT_LEAD_BREAK = re.compile(r"时|之后|以后|后|前|看|确认|证明|检查|核对|核实|对照|贴|写进|写到|读|改|交回|报告|然后|再|除")
# 名字后面接这些，它就不是宾语本身：「54 号之外的阶段」「跑整轮门禁时」「crates/mutations.tsv 里的两行」「54 号的判定段」（「的」后面是全量、快档、用例这类仍算）
EXCLUDED_AFTER_NAME = re.compile(r"[`\s]*(?:之外|以外|时|的时候|里|中|的(?!\s*(?:全量|快档|整表|变异表|快用例|用例|测试|装置|全部|所有)))")
# 名字前面紧挨 > / >>：那是 shell 重定向的目标（往 crates/mutations.tsv 追加一行），不是要跑的东西
REDIRECTION_BEFORE_NAME = re.compile(r">{1,2}\s*$")
# 名字在前、动词在后，名字前面紧挨「被」：名字是施事（「变异会被门禁 59 号复跑」），不是宾语
PASSIVE_BEFORE_NAME = re.compile(r"被\s*(?:门禁\s*)?$")
# 分句开头是别人（主 agent、别的会话、用户、另外两个验证员），分句里又没有「你」：在说别人做什么，不是要它跑
THIRD_PARTY_SUBJECT_AT_CLAUSE_START = re.compile(r"^[\s*_#>\-\d.、]*(?:主\s*agent|另一个会话|别的会话|其他会话|用户|崩溃验证员|crash-verifier|门禁分诊员?|gate-triage)")
# (类, 认的名字)
HEAVY_TEST_NAMES = [
    ("层 0", re.compile(r"层\s*0(?!\d)|(?<![\d.])0\s*层|layer\s*0(?!\d)|(?<!\d)54\s*号|54-layer0", re.I)),
    ("QEMU", re.compile(r"qemu|vm-bench|(?<!\d)55\s*号|55-qemu", re.I)),
    ("herd7", re.compile(r"herd7|lkmm|(?<!\d)57\s*号|57-lkmm", re.I)),
    ("crates 变异整表", re.compile(r"crates/mutations\.tsv|变异整表|(?<!\d)59\s*号|59-crates")),
    ("全量测试", re.compile(r"(?<![\w-])check\.sh|全量测试")),
    ("整轮门禁", re.compile(r"(?<![\w-])gate\.sh|gate-staged\.sh|整轮门禁")),
    ("全部实验复跑", re.compile(r"(?<!\d)87\s*号|87-replay")),
    ("E152 装置", re.compile(r"e152(?!-tables|-stage-root)", re.I)),
]
FULL_TEST_COMMAND = re.compile(r"cargo\s+test\b[^\n]*?--(?:all|workspace)(?![\w-])")
DISPATCH_HEAVY_OWN_SHARE = {
    "crash-verifier": {"QEMU", "herd7", "crates 变异整表"},
    "gate-triage": {"整轮门禁", "全部实验复跑"},
}

def remove_quoted_fragments(line):
    """一行里引号括起来的片段换成 □。反引号里的代码不看引号；没闭合的开引号不算，原样留着。"""
    kept = []
    open_quotes = []  # [(该用哪个字关, 这段引号在 kept 里的起点)]
    inside_code = False
    for character in line:
        if inside_code:
            kept.append(character)
            inside_code = character != "`"
            continue
        if character == "`":
            inside_code = True
            kept.append(character)
            continue
        closing_depth = next((depth for depth in range(len(open_quotes) - 1, -1, -1) if open_quotes[depth][0] == character), None)
        if closing_depth is not None:
            fragment_start = open_quotes[closing_depth][1]
            del open_quotes[closing_depth:]
            kept.append(character)
            if not open_quotes:
                del kept[fragment_start:]
                kept.append(QUOTED_FRAGMENT_PLACEHOLDER)
            continue
        if character in QUOTE_CLOSING:
            open_quotes.append((QUOTE_CLOSING[character], len(kept)))
        kept.append(character)
    return "".join(kept)

def is_topic_gap(gap):
    """名字在前、动词在后时两者之间这一段：不长于 TOPIC_GAP_LIMIT，没有 TOPIC_GAP_BREAK。"""
    return len(gap) <= TOPIC_GAP_LIMIT and TOPIC_GAP_BREAK.search(gap) is None

def is_counted_verb(clause, verb):
    """RUN_VERB 的一处命中是不是要它跑的动词：「从……起」不是；后面紧跟「的」的是定语；前面紧挨「正在 / 在」的是在说正在发生的事。"""
    if verb.group() == "起" and FROM_TIME_BEFORE_QI.search(clause[:verb.start()]):
        return False
    if PROGRESSIVE_BEFORE_VERB.search(clause[:verb.start()]):
        return False
    return ATTRIBUTIVE_AFTER_VERB.match(clause, verb.end()) is None

def heavy_names_in(text):
    """text 里每一处重型阶段的名字：[(起点, 终点, 类)]，按起点排。"""
    return sorted((match.start(), match.end(), category) for category, names in HEAVY_TEST_NAMES for match in names.finditer(text))

def heavy_objects_after_verb(object_span):
    """动词后面那一段里，算得上宾语的重型阶段的类：名字前面至多 OBJECT_LEAD_LIMIT 个非 ASCII 字、没有 OBJECT_LEAD_BREAK、不是重定向目标，
    名字后面不是 EXCLUDED_AFTER_NAME。"""
    categories = []
    for start, end, category in heavy_names_in(object_span):
        lead = object_span[:start]
        if OBJECT_LEAD_BREAK.search(lead) or sum(1 for character in lead if ord(character) > 127 and not character.isspace()) > OBJECT_LEAD_LIMIT:
            break
        if EXCLUDED_AFTER_NAME.match(object_span, end) is None and REDIRECTION_BEFORE_NAME.search(lead) is None:
            categories.append(category)
    return categories

def clause_heavy_requests(clause):
    """一个分句里真要跑的重型阶段的类，按出现的次序。分句里有否定词、或开头是别人在做的，整句不判。"""
    clause = A_NOT_A_QUESTION.sub(lambda match: match.group(1) + QUOTED_FRAGMENT_PLACEHOLDER + match.group(1), clause)
    if NEGATION.search(clause):
        return []
    if THIRD_PARTY_SUBJECT_AT_CLAUSE_START.search(clause) and "你" not in clause:
        return []
    verbs = [verb for verb in RUN_VERB.finditer(clause) if is_counted_verb(clause, verb)]
    names_in_clause = heavy_names_in(clause)
    categories = []
    for index, verb in enumerate(verbs):
        object_end = verbs[index + 1].start() if index + 1 < len(verbs) else len(clause)
        categories += heavy_objects_after_verb(clause[verb.end():object_end])
        if verb.group() in TOPIC_VERBS and PRONOUN_OBJECT_AFTER_VERB.match(clause, verb.end()) is None:
            categories += [category for start, end, category in names_in_clause
                           if end <= verb.start() and is_topic_gap(clause[end:verb.start()])
                           and EXCLUDED_AFTER_NAME.match(clause, end) is None
                           and PASSIVE_BEFORE_NAME.search(clause[:start]) is None
                           and PREPOSITION_BEFORE_NAME.search(clause[:start]) is None]
    if FULL_TEST_COMMAND.search(clause):
        categories.append("全量测试")
    return categories

def clauses_of(sentence):
    """一句切成分句：全角括号里的每一段单独成句（由内往外剥），括号外剩下的前后拼回去，再都按逗号切。"""
    parentheticals = []
    outer = sentence
    match = FULL_WIDTH_PARENTHETICAL.search(outer)
    while match is not None:
        parentheticals.append(match.group()[1:-1])
        outer = outer[:match.start()] + " " + outer[match.end():]
        match = FULL_WIDTH_PARENTHETICAL.search(outer)
    return [clause for text in [outer] + parentheticals for clause in CLAUSE_BOUNDARY.split(text)]

def heavy_test_requests(prompt, subagent_type):
    """派发提示里要这个 agent 跑、而不是它自己那一份的重型测试：[(类, 句子)]。句子里引号括起来的片段已换成 □。"""
    own_share = DISPATCH_HEAVY_OWN_SHARE.get(subagent_type, set())
    found = []
    for line in prompt.split("\n"):
        for sentence in SENTENCE_BOUNDARY.split(remove_quoted_fragments(line)):
            opener = REPORTED_SPEECH_OPENER.search(sentence)
            judged = sentence if opener is None else sentence[:opener.start()]
            outside = [category for clause in clauses_of(judged)
                       for category in clause_heavy_requests(clause) if category not in own_share]
            if outside:
                found.append((outside[0], sentence.strip()))
    return found

# 第三条：派这两类 agent 时，它要改的文件不许落在一轮还没判完的三方开工快照里
SNAPSHOT_GUARDED_AGENTS = {"kb-scribe", "implementation-writer"}
SNAPSHOT_DIRECTORY_SUFFIX = "-snapshot"
SNAPSHOT_LIST_NAME_MARK = "sha256"
# 快照清单的一行：64 位十六进制哈希、空白、路径（sha256sum 二进制模式在路径前加 *）
SNAPSHOT_LIST_LINE = re.compile(r"^\s*[0-9a-fA-F]{64}\s+\*?(\S.*?)\s*$")
# 清单里的路径从第一段 .claude/ 或 crates/ 起取（./.claude/…、tree/crates/…、defs/.claude/… 都归一成仓库根相对路径）
REPOSITORY_PATH_START = re.compile(r"(?:^|/)((?:\.claude|crates)/)")
# 一条路径在文字里「出现」：前后不连着路径字符（后面的 . 只在它后面还跟着路径字符时才算连着，句末的点不算）
PATH_LEFT_EDGE = r"(?<![A-Za-z0-9_\-./~])"
PATH_RIGHT_EDGE = r"(?![A-Za-z0-9_\-/]|\.[A-Za-z0-9_])"
DOT_SLASH_BEFORE_REPOSITORY_PATH = re.compile(PATH_LEFT_EDGE + r"\./(?=\.claude/|crates/)")
# 提示里点名的规格文件：/tmp/claude-<uid>/ 下的路径，到空白、引号、括号或中文标点为止
SCRATCH_FILE_MENTION = re.compile(r"/tmp/claude-\d+/[^\s`'\"，。；：、（）()「」『』“”‘’《》<>\[\]{}|,;*]+")
SPECIFICATION_READ_LIMIT_BYTES = 4 * 1024 * 1024
# 放行口：一行以「快照冲突已判：<轮名> <理由>」开头（行首可以有列表记号与粗体星号）；只在这一行里找，不跨到下一行
SNAPSHOT_CONFLICT_RULED_LINE = re.compile(r"^[ \t>*_\-]*快照冲突已判[* \t]*[：:][* \t]*([^\s，,：:；;。]+)[ \t，,：:；;。]+(.*?)[ \t]*$", re.M)
SNAPSHOT_CONFLICT_RULED_ANYWHERE = re.compile(r"快照冲突已判[*\s]*[：:][*\s]*([^\s，,：:；;。]*)")
SNAPSHOT_CONFLICTS_SHOWN_LIMIT = 20

def break_switch_is(name):
    """自检用的弄坏开关：RUNNER_DISPATCH_GUARD_BREAK 等于 name 时，那一处判法故意走错，自检必须判红。"""
    return os.environ.get("RUNNER_DISPATCH_GUARD_BREAK") == name

def open_three_way_rounds(project_root):
    """还没判完的三方轮：[(轮名, 快照目录)]。research/prompts/<轮>-snapshot/ 在、_<轮>-body.md 在、<轮>-main-verification.md 不在。"""
    prompts_directory = os.path.join(project_root, "research", "prompts")
    if not os.path.isdir(prompts_directory):
        return []
    rounds = []
    for entry_name in sorted(os.listdir(prompts_directory)):
        snapshot_directory = os.path.join(prompts_directory, entry_name)
        if not entry_name.endswith(SNAPSHOT_DIRECTORY_SUFFIX) or not os.path.isdir(snapshot_directory):
            continue
        round_name = entry_name[:-len(SNAPSHOT_DIRECTORY_SUFFIX)]
        has_body = os.path.isfile(os.path.join(prompts_directory, f"_{round_name}-body.md"))
        has_verdict = os.path.exists(os.path.join(prompts_directory, f"{round_name}-main-verification.md"))
        if round_name and (has_body or break_switch_is("snapshot-body-optional")) and (not has_verdict or break_switch_is("snapshot-closed-round-open")):
            rounds.append((round_name, snapshot_directory))
    return rounds

def repository_path_of_snapshot_line(line):
    """快照清单的一行归一成的仓库根相对路径；不是「<哈希>  <路径>」的形状、或路径里没有 .claude/ 与 crates/ 起头的一段，返回 None。"""
    if break_switch_is("snapshot-loose-lines"):
        fields = line.split()
        return fields[-1] if fields else None
    match = SNAPSHOT_LIST_LINE.match(line)
    if match is None:
        return None
    listed_path = match.group(1)
    if break_switch_is("snapshot-no-normalize"):
        return listed_path
    start = REPOSITORY_PATH_START.search(listed_path)
    if start is None:
        return None
    normalized = os.path.normpath(listed_path[start.start(1):])
    return None if normalized.startswith("..") else normalized

def snapshot_listed_paths(snapshot_directory, notes):
    """快照目录里文件名含 sha256 的每份清单读出来的路径：{仓库根相对路径: 清单文件}。认不出的行跳过；读不了的清单记进 notes、跳过。"""
    listed = {}
    for list_name in sorted(os.listdir(snapshot_directory)):
        if SNAPSHOT_LIST_NAME_MARK not in list_name:
            continue
        list_path = os.path.join(snapshot_directory, list_name)
        try:
            with open(list_path, encoding="utf-8", errors="replace") as list_file:
                lines = list_file.read().splitlines()
        except OSError as error:
            if break_switch_is("snapshot-unreadable-aborts"):
                raise
            if break_switch_is("snapshot-unreadable-denies"):
                listed[f"<读不了的清单 {list_name}>"] = list_path
            notes.append(f"! 派发闸（快照那一条）读不了快照清单 {list_path}：{error}；这一份跳过，没当成撞快照")
            continue
        for line in lines:
            repository_path = repository_path_of_snapshot_line(line)
            if repository_path is not None:
                listed.setdefault(repository_path, list_path)
    return listed

def specification_texts(prompt, notes):
    """提示里点名的 /tmp/claude-<uid>/ 下的规格文件：[(路径, 内容)]。不在的、是目录的跳过；在而读不了的记进 notes、跳过。"""
    texts = []
    seen = set()
    for mention in SCRATCH_FILE_MENTION.finditer(prompt):
        candidate = mention.group().rstrip(".:")
        if not os.path.isfile(candidate):
            # 路径后面紧跟中文（「spec.md里」）：去掉尾巴上的非 ASCII 字再看一次
            candidate = re.sub(r"[^\x00-\x7f]+$", "", candidate).rstrip(".:")
        if candidate in seen:
            continue
        seen.add(candidate)
        if not os.path.isfile(candidate):
            if break_switch_is("snapshot-missing-spec-denies") and not os.path.isdir(candidate):
                texts.append((candidate, "<不在的规格>"))
            continue
        try:
            with open(candidate, "rb") as specification_file:
                texts.append((candidate, specification_file.read(SPECIFICATION_READ_LIMIT_BYTES).decode("utf-8", errors="replace")))
        except OSError as error:
            notes.append(f"! 派发闸（快照那一条）读不了派发提示点名的规格 {candidate}：{error}；这一份没收进要改的文件")
    return texts

def mentioned_repository_paths(text, candidate_paths, project_root):
    """candidate_paths 里哪几条在 text 里出现：前后不连着路径字符；项目根起的绝对路径与 ./ 前缀先去掉。"""
    root_prefix = project_root.rstrip("/")
    if root_prefix:
        text = text.replace(root_prefix + "/", "")
    text = DOT_SLASH_BEFORE_REPOSITORY_PATH.sub("", text)
    if break_switch_is("snapshot-no-edges"):
        return {path for path in candidate_paths if path in text}
    return {path for path in candidate_paths if re.search(PATH_LEFT_EDGE + re.escape(path) + PATH_RIGHT_EDGE, text)}

def ruled_snapshot_conflicts(prompt):
    """派发提示里写了「快照冲突已判：<轮名> <理由>」的轮名（理由非空、不是 <理由> 这种占位）。"""
    if break_switch_is("snapshot-valve-ignored"):
        return set()
    if break_switch_is("snapshot-valve-loose"):
        return {"*"} if SNAPSHOT_CONFLICT_RULED_ANYWHERE.search(prompt) else set()
    ruled = set()
    for match in SNAPSHOT_CONFLICT_RULED_LINE.finditer(prompt):
        reason = match.group(2).strip()
        if reason not in NONE_WORDS and not (reason.startswith("<") and reason.endswith(">")):
            ruled.add(match.group(1))
    return ruled

def snapshot_conflict_verdict(subagent_type, prompt, project_root):
    """第三条。返回 (0, 给 stderr 的提醒或 None) 放行；(2, 说明) 拒绝。这一条自己出错一律放行，错误写进说明。"""
    if os.environ.get("RUNNER_DISPATCH_GUARD_DISABLE_SNAPSHOT") == "1":
        return 0, None
    if subagent_type not in SNAPSHOT_GUARDED_AGENTS and not break_switch_is("snapshot-any-agent"):
        return 0, None
    notes = []
    try:
        rounds = open_three_way_rounds(project_root)
        if not rounds:
            return 0, None
        sources = [("派发提示", prompt)]
        if not break_switch_is("snapshot-spec-unread"):
            sources += [(f"规格 {path}", text) for path, text in specification_texts(prompt, notes)]
        ruled = ruled_snapshot_conflicts(prompt)
        conflicts = []  # [(轮名, 快照目录, [(路径, 出现在哪, 清单文件)])]
        for round_name, snapshot_directory in rounds:
            listed = snapshot_listed_paths(snapshot_directory, notes)
            hits = {}
            for source_name, text in sources:
                for path in sorted(mentioned_repository_paths(text, listed, project_root)):
                    hits.setdefault(path, (source_name, listed[path]))
            if break_switch_is("snapshot-unreadable-denies"):
                hits.update({path: ("快照清单", list_path) for path, list_path in listed.items() if path.startswith("<读不了的清单")})
            if break_switch_is("snapshot-missing-spec-denies"):
                hits.update({f"<不在的规格 {path}>": ("派发提示", "-") for path, text in sources[1:] if text == "<不在的规格>"})
            if hits and round_name not in ruled and "*" not in ruled:
                conflicts.append((round_name, snapshot_directory, [(path, *hits[path]) for path in sorted(hits)]))
    except Exception as error:  # 挂在每一次派发上的闸，坏了不能把所有派发都挡住
        notes.append(f"! 派发闸（快照那一条）自己出错，这一次放行：{type(error).__name__}: {error}")
        return 0, "\n".join(notes)
    if not conflicts:
        return 0, ("\n".join(notes) or None)
    listing = []
    for round_name, snapshot_directory, hits in conflicts:
        listing.append(f"  三方轮 {round_name}（开工快照 {os.path.relpath(snapshot_directory, project_root)}/，"
                       f"research/prompts/{round_name}-main-verification.md 还不在）：")
        for path, source_name, list_path in hits[:SNAPSHOT_CONFLICTS_SHOWN_LIMIT]:
            listing.append(f"    {path}（出现在{source_name}；快照清单 {os.path.basename(list_path)}）")
        if len(hits) > SNAPSHOT_CONFLICTS_SHOWN_LIMIT:
            listing.append(f"    （另有 {len(hits) - SNAPSHOT_CONFLICTS_SHOWN_LIMIT} 个）")
    round_names = "、".join(round_name for round_name, _, _ in conflicts)
    valve_lines = "；".join(f"「快照冲突已判：{round_name} <理由>」" for round_name, _, _ in conflicts)
    return 2, (f"✗ 派 {subagent_type} 要改的文件落在还没判完的三方轮 {round_names} 的开工快照里：\n" + "\n".join(listing) + "\n"
               "→ 规矩：.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」：腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。\n"
               f"→ 怎么办：等 {round_names} 的判决写完再派，或把这几处改动并进判决；判过这次派发不碰那一轮的证据（例如只读不改），"
               f"在派发提示里单起一行写{valve_lines}（理由不许空）。"
               + ("\n" + "\n".join(notes) if notes else ""))

def decide(hook_input, project_root):
    """返回 (0, None 或给 stderr 的提醒) 放行；(2, 说明) 拒绝。"""
    if os.environ.get("RUNNER_DISPATCH_GUARD_DISABLE_CHECK") == "1":
        return 0, None
    tool_input = hook_input.get("tool_input") or {}
    subagent_type = tool_input.get("subagent_type") or "general-purpose"
    prompt = tool_input.get("prompt") or ""
    requests = [] if os.environ.get("RUNNER_DISPATCH_GUARD_DISABLE_HEAVY_TEST") == "1" else heavy_test_requests(prompt, subagent_type)
    if requests:
        category, sentence = requests[0]
        more = f"（另有 {len(requests) - 1} 句）" if len(requests) > 1 else ""
        placeholder_note = f"（{QUOTED_FRAGMENT_PLACEHOLDER} 是引号里、已按引用不判的片段）" if QUOTED_FRAGMENT_PLACEHOLDER in sentence else ""
        return 2, (f"✗ 派 {subagent_type} 的提示要它跑「{category}」：「{sentence[:120]}」{placeholder_note}{more}\n"
                   "→ 判法：句子里「跑 / 执行 / 复跑 / 重跑 / 运行 / bash / 起」的宾语是重型阶段，或写了「`cargo test --all`」这种命令字面；"
                   "否定句、引号里的、「……说，」「报告里写」「原句」之后的转述、重型阶段只是同句另一个名词的，都不拦（判法细节在这个 hook 的文件头）。\n"
                   "→ 规矩：重型测试（层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置）只在提交代码时、或用户要求时跑；"
                   "子 agent 只跑自己动到的测试二进制、fmt / clippy / build 与 54、55、57、59、87 之外的门禁阶段；crash-verifier 只跑 55、57、59 号那几道，"
                   "gate-triage 只跑 gate.sh 整轮与 87 号（执行时 .claude/hooks/heavy-test-guard.sh 也拒）。\n"
                   "→ 怎么办：真要它跑就删掉这一句；不是要它跑的，写成否定句（「不跑层 0」），否定词与名字放在同一个分句里（名字别放进否定词后面的全角括号，括号里单独成分句），转述别人的原话用「」括起来；提交时的重阶段派 crash-verifier、整轮门禁派 gate-triage；"
                   "提交之外任务确实要跑，先弹窗问用户，用户同意了由主 agent 带 SINGLEFS_HEAVY_TESTS=user-request 跑。")
    snapshot_code, snapshot_message = snapshot_conflict_verdict(subagent_type, prompt, project_root)
    if snapshot_code:
        return snapshot_code, snapshot_message
    if subagent_type != "experiment-runner":
        return 0, snapshot_message
    if "只修锚点" in prompt:
        return 0, None
    answered = labelled_value(prompt, "这一段回答的岔路")
    if answered is None or answered in NONE_WORDS:
        return 2, ("✗ 派 experiment-runner 的提示里没写「这一段回答的岔路：…」。\n"
                   "→ 怎么办：对着岔路单（research/prompts/<轮>-forks.md）写明这一段的量让哪几行够判；"
                   "说不出是哪一行，这一段就不该派（.claude/rules/three-way-inference.md「交岔路时写岔路单，派实验时带上它」）。")
    number_match = re.search(r"research/prompts/e(\d+)-(?:preregistration|r\d+-prereg)\.md", prompt)
    if number_match is None:
        return 0, None
    pages = glob.glob(os.path.join(project_root, ".claude", "kb", "experiments", f"{int(number_match.group(1))}-*.md"))
    if not pages:
        return 0, None
    remaining = labelled_value(prompt, "上一段岔路表里还差")
    if remaining is None:
        return 2, (f"✗ E{number_match.group(1)} 已经有实验页，这是续做，派发提示里却没写「上一段岔路表里还差：…」。\n"
                   "→ 怎么办：先读上一段交回的岔路表，把还开着的行点名写进提示；一行都不差就停下交岔路表，不续派。")
    if remaining in NONE_WORDS:
        return 2, (f"✗ E{number_match.group(1)} 上一段岔路表里一行都不差了，不续派。\n"
                   "→ 怎么办：按岔路单交岔路表给用户；剩下的量在实验页标「够判后未跑」，状态写「已跑（够判）」。")
    return 0, None

# 2026-09-25 主 agent 派「修派发闸」时写的说明（第 3–25 行逐字）：引了三句误拦原句与四句该拦例句，旧判法把它整条拦下
DISPATCH_SPECIFICATION_EXCERPT = """还 `records/2026-09-16-subagent拆分提案.md` 第四十节第 18 行后半那笔欠账：派发闸只要看到提示里点了重型阶段、又不是否定句就拦，连转述别人报告、写「为什么不」的句子也拦。

## 2026-09-25 被误拦的原句（逐字，都该放行）

1. 「理由照实写：59 号属于只在提交时才执行的重型阶段，改它的实现在提交之前没有办法证明没改坏，所以两边各留一份」——在写一行说明的理由，没有要谁去跑。
2. 「还有一处旧判法要一起看：实二四的报告……第 3 条说，55 号还按「宿主只重跑第一个事务」判 `second-transaction` 和 `second-instance`」——「重跑」的宾语是「第一个事务」，而且在转述别人报告里的判法。
3. 「理由照实写：59 号是重型阶段，抽成共用库之后没法在提交之外跑它来证明没改坏」——「没法……跑」是否定。
4. 这份说明本身：主 agent 派发「修派发闸」时，提示里引了上面三句与下面的该拦例句，被整条拦下。

## 要做的

只在「跑 / 执行 / 复跑 / 重跑 / 起」这一类动词的宾语就是重型阶段时才判。重型阶段：层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、87 号、E152 装置，以及 54、55、57、59、87 号。下面几种不判：
- 否定：不、别、禁止、没法、没有办法、无法、不能、不许；
- 转述或引用：「……说」「……报告」「原句」、被「」或引号括起来的片段；
- 宾语是别的东西，重型阶段只是同一句里的另一个名词。

判法要写得出来、可复核：按句切（句号、分号、换行），在句内找动词与宾语的相对位置；不许用模型判。

收窄之后该拦的照拦。该拦例句：
- 「跑 54 号层 0 全量」
- 「提交前起 QEMU 跑一遍」
- 「执行 gate.sh」
- 「复跑 crates 变异整表」"""

def selftest(hook_dir):
    work = tempfile.mkdtemp()
    other_directories = []  # 快照那一条另起的临时目录，finally 里一起删
    try:
        os.makedirs(os.path.join(work, ".claude", "kb", "experiments"))
        open(os.path.join(work, ".claude", "kb", "experiments", "153-样本.md"), "w").write("## E153 样本 —— 部分已跑\n")
        first = "跑前登记：research/prompts/e160-preregistration.md\n"
        again = "跑前登记：research/prompts/e153-preregistration.md\n"
        def case_in(project_root, label, subagent_type, prompt, want):
            hook_input = {"tool_name": "Agent", "tool_input": {"subagent_type": subagent_type, "prompt": prompt}}
            return (label, want, decide(hook_input, project_root)[0])
        def case(label, subagent_type, prompt, want):
            return case_in(work, label, subagent_type, prompt, want)
        # 快照那一条的样本仓：r-open 开着（有正文、没判决）；r-closed 已判完；r-nobody 有快照没正文（普查一类）；r-odd 开着，清单里全是认不出的行
        def write_sample(project_root, relative_path, content):
            path = os.path.join(project_root, relative_path)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w", encoding="utf-8") as sample_file:
                sample_file.write(content)
        digest = "0123456789abcdef" * 4
        write_sample(work, "research/prompts/_r-open-body.md", "# r-open 正文\n")
        write_sample(work, "research/prompts/r-open-snapshot/kb-sha256.txt",
                     f"{digest}  ./.claude/kb/checks-owed.md\n{digest}  ./.claude/kb/decisions/18-块里携带什么信息.md\n")
        write_sample(work, "research/prompts/r-open-snapshot/crates-sha256.txt",
                     f"{digest}  tree/crates/singlefs-core/src/mount.rs\n{digest}  defs/.claude/agents/crash-verifier.md\n")
        write_sample(work, "research/prompts/_r-closed-body.md", "# r-closed 正文\n")
        write_sample(work, "research/prompts/r-closed-snapshot/sha256sums.txt", f"{digest}  .claude/kb/decisions/03-空间分配.md\n")
        write_sample(work, "research/prompts/r-closed-main-verification.md", "# r-closed 判决\n")
        write_sample(work, "research/prompts/r-nobody-snapshot/sources.sha256", f"{digest}  crates/singlefs-core/src/allocator.rs\n")
        write_sample(work, "research/prompts/_r-odd-body.md", "# r-odd 正文\n")
        write_sample(work, "research/prompts/r-odd-snapshot/odd-sha256.txt",
                     f"快照时刻 14:09:00 UTC\n# 注释\n{digest}  README.md\n{digest}  research/prompts/r-odd-model.md\nnot-a-hash  .claude/kb/pitfalls.md\n")
        # 另一个样本仓：一份清单读不了（悬空的符号链接），另一份读得了
        unreadable_work = tempfile.mkdtemp()
        other_directories.append(unreadable_work)
        write_sample(unreadable_work, "research/prompts/_r-broken-body.md", "# r-broken 正文\n")
        write_sample(unreadable_work, "research/prompts/r-broken-snapshot/kb-sha256.txt", f"{digest}  .claude/kb/checks-owed.md\n")
        os.symlink(os.path.join(unreadable_work, "不在的文件"), os.path.join(unreadable_work, "research/prompts/r-broken-snapshot/gone-sha256.txt"))
        # 规格文件放在 /tmp/claude-<uid>/ 下（派发闸只认那里）
        scratch_root = f"/tmp/claude-{os.getuid()}"
        os.makedirs(scratch_root, exist_ok=True)
        specification_directory = tempfile.mkdtemp(prefix="runner-dispatch-guard-selftest-", dir=scratch_root)
        other_directories.append(specification_directory)
        specification_path = os.path.join(specification_directory, "spec.md")
        write_sample(specification_directory, "spec.md", "1. 文件：.claude/kb/checks-owed.md\n   旧串：| C120 | …\n   新串：| C120 | …\n")
        missing_specification_path = os.path.join(specification_directory, "missing-spec.md")
        collided = "规格：改 `.claude/kb/checks-owed.md` 里 C120 那一行。\n"
        forwarded_refusal = decide({"tool_input": {"subagent_type": "kb-scribe", "prompt": collided}}, work)[1] or ""
        cases = [
            case("不是执行员", "experiment-designer", "随便", 0),
            case("只修锚点", "experiment-runner", "只修锚点：表 e153", 0),
            case("第一段点名了岔路", "experiment-runner", first + "这一段回答的岔路：岔路 1、岔路 4\n", 0),
            case("第一段没点名岔路", "experiment-runner", first, 2),
            case("第一段岔路写无", "experiment-runner", first + "这一段回答的岔路：无\n", 2),
            case("续做点名了还差的行", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：岔路 1 的崩溃支线\n", 0),
            case("续做没写还差", "experiment-runner", again + "这一段回答的岔路：岔路 1\n", 2),
            case("续做还差写无", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：无\n", 2),
            case("重跑登记也算续做", "experiment-runner", "research/prompts/e153-r2-prereg.md\n这一段回答的岔路：岔路 3\n", 2),
            # 重型测试：肯定句拒、否定句放；crash-verifier / gate-triage 只放自己那一份
            case("重型:实现员跑层 0 快档", "implementation-writer", "改完交回前跑层 0 快档一次。", 2),
            case("重型:实现员跑 cargo test --all", "implementation-writer", "交回前 `cargo test --all` 贴末尾。", 2),
            case("重型:实现员 bash check.sh", "implementation-writer", "最后 bash .claude/scripts/check.sh 贴末尾原样输出。", 2),
            case("重型:通用 agent 跑 layer0", None, "跑 first_transaction_step_seven_layer0 的全量", 2),
            case("重型:执行员跑 gate.sh", "experiment-runner", first + "这一段回答的岔路：岔路 1\n收尾跑 gate.sh --staged。\n", 2),
            case("重型:书记员跑 QEMU", "kb-scribe", "写完跑 55 号 QEMU。", 2),
            case("重型:实现员跑 herd7", "implementation-writer", "改完跑 herd7 看 litmus。", 2),
            case("重型:实现员复跑 crates 变异整表", "implementation-writer", "复跑 crates/mutations.tsv 整表。", 2),
            case("重型:崩溃验证员跑整轮门禁", "crash-verifier", "跑层 0 全量，再跑 gate.sh --staged。", 2),
            case("重型:门禁分诊跑层 0", "gate-triage", "直接跑 54 号 --full。", 2),
            case("重型:通用 agent 跑 E152", "general-purpose", "跑 E152 八家对比。", 2),
            case("重型:否定句「不跑层 0」", "implementation-writer", "不跑层 0，只跑动到的测试二进制。", 0),
            case("重型:否定句「别跑」", "implementation-writer", "别跑 `cargo test --all`。", 0),
            case("重型:否定句「禁止」", "implementation-writer", "禁止跑 check.sh。", 0),
            case("重型:否定句「不用」", "implementation-writer", "层 0 全量不用跑，提交时统一跑。", 0),
            case("重型:只提到、没要它跑", "implementation-writer", "层 0 归 crash-verifier；check.sh 那一套 lint 下的 clippy 要过。", 0),
            case("重型:--all-targets 不是 --all", "implementation-writer", "跑 `cargo build --offline --all-targets`。", 0),
            case("重型:轻阶段谁都能跑", "experiment-runner", first + "这一段回答的岔路：岔路 1\n跑完再跑 bash .claude/gate.d/12-no-prime-marks.sh。\n", 0),
            case("重型:崩溃验证员跑自己那几道", "crash-verifier", "提交流程里跑 55 号 QEMU、57 号 herd7、59 号变异整表，命令带 SINGLEFS_HEAVY_TESTS=commit。", 0),
            case("重型:崩溃验证员跑层 0 不归它", "crash-verifier", "提交流程里跑 54 号 --full，命令带 SINGLEFS_HEAVY_TESTS=commit。", 2),
            case("重型:门禁分诊跑整轮", "gate-triage", "带 SINGLEFS_HEAVY_TESTS=commit 跑 gate.sh --staged。", 0),
            # 收窄：只在动词的宾语就是重型阶段时才拦。2026-09-25 被误拦的原句逐字，都该放行
            case("收窄:原句一「只在提交时才执行的」是定语", "general-purpose",
                 "理由照实写：59 号属于只在提交时才执行的重型阶段，改它的实现在提交之前没有办法证明没改坏，所以两边各留一份", 0),
            case("收窄:原句二 转述报告、「重跑」的宾语是第一个事务", "general-purpose",
                 "还有一处旧判法要一起看：实二四的报告……第 3 条说，55 号还按「宿主只重跑第一个事务」判 `second-transaction` 和 `second-instance`", 0),
            case("收窄:原句三「没法……跑它」", "general-purpose", "理由照实写：59 号是重型阶段，抽成共用库之后没法在提交之外跑它来证明没改坏", 0),
            case("收窄:派发说明本身（引了上面三句与该拦例句）", "general-purpose", DISPATCH_SPECIFICATION_EXCERPT, 0),
            # 该拦例句照拦
            case("收窄:拦「跑 54 号层 0 全量」", "implementation-writer", "跑 54 号层 0 全量", 2),
            case("收窄:拦「提交前起 QEMU 跑一遍」", "implementation-writer", "提交前起 QEMU 跑一遍", 2),
            case("收窄:拦「执行 gate.sh」", "implementation-writer", "执行 gate.sh", 2),
            case("收窄:拦「复跑 crates 变异整表」", "implementation-writer", "复跑 crates 变异整表", 2),
            # 各一句否定形态：放行
            case("收窄:否定「不许跑 54 号」", "implementation-writer", "不许跑 54 号层 0 全量", 0),
            case("收窄:否定「别起 QEMU 跑一遍」连着两个动词", "implementation-writer", "提交前别起 QEMU 跑一遍", 0),
            case("收窄:否定「不能执行 gate.sh」", "implementation-writer", "不能执行 gate.sh", 0),
            case("收窄:否定「没法在提交之外复跑」", "implementation-writer", "没法在提交之外复跑 crates 变异整表", 0),
            # 各一句引号形态：放行
            case("收窄:引号「」", "implementation-writer", "该拦例句「跑 54 号层 0 全量」", 0),
            case("收窄:引号“”", "implementation-writer", "旧提示里那句“提交前起 QEMU 跑一遍”已删", 0),
            case("收窄:ASCII 双引号", "implementation-writer", '例句 "执行 gate.sh" 照拦', 0),
            case("收窄:引号『』", "implementation-writer", "例句『复跑 crates 变异整表』", 0),
            # 每条收窄规则各自的边：转述、别的宾语、定语放行；否定只管紧跟的动词，照拦
            case("收窄:转述「报告说，」之后", "implementation-writer", "实二四报告说，要复跑 crates 变异整表", 0),
            case("收窄:转述之前的那半照判", "implementation-writer", "跑 54 号层 0 全量，报告里写退出码", 2),
            case("收窄:宾语是单测，54 号只是同句另一个名词", "implementation-writer", "跑一遍自己的单测确认 54 号层 0 没被改坏", 0),
            case("收窄:宾语是 54 号之外的阶段", "implementation-writer", "跑 54 号之外的门禁阶段", 0),
            case("收窄:「只在提交时才跑」是说明", "implementation-writer", "54 号只在提交时才跑全量", 0),
            case("收窄:「check.sh 交回前跑一次」照拦", "implementation-writer", "验证负载：只跑动到的测试二进制，`check.sh` 交回前跑一次", 2),
            case("收窄:定语「才跑的那一道」", "implementation-writer", "59 号是只在提交时才跑的那一道", 0),
            case("收窄:命令字面在前、否定在后", "implementation-writer", "`cargo test --all` 不用跑", 0),
            case("收窄:否定只管它那个分句", "implementation-writer", "不跑 12 号，跑 54 号层 0 全量", 2),
            case("收窄:「正在跑」是在说正在发生的事", "implementation-writer", "宿主上门禁 54 号正在跑层 0 全量，你的复跑一律 nice -n 19", 0),
            case("收窄:「在后台跑」是在说正在发生的事", "implementation-writer", "此刻有一个层 0 全量（门禁 54 号）在后台跑，`nice` 起", 0),
            case("收窄:拒绝信息原样转给别人不再被拦", "general-purpose", decide({"tool_input": {"subagent_type": "implementation-writer", "prompt": "跑 54 号层 0 全量"}}, work)[1], 0),
            case("收窄:「现在跑」照拦", "implementation-writer", "现在跑 54 号层 0 全量", 2),
            case("收窄:「……时」是时间状语", "implementation-writer", "12 号只在收尾跑整轮门禁时判", 0),
            case("收窄:宾语是「54 号的判定段」", "three-way-attack", "在仓的副本里真跑 54 号的判定段，用合成日志与假 cargo", 0),
            case("收窄:「54 号的全量」照拦", "implementation-writer", "跑 54 号的全量", 2),
            case("收窄:重定向目标不是要跑的", "implementation-writer", "追加那张表只许用单条 bash `printf '…\\n' >> crates/mutations.tsv`", 0),
            case("收窄:被动句里名字是施事", "general-purpose", "变异会不会被门禁 59 号复跑", 0),
            case("收窄:分句开头是主 agent", "three-way-attack", "主 agent 收尾跑 54 号 `--full`、整轮门禁只核全绿标记", 0),
            case("收窄:主 agent 要你跑照拦", "implementation-writer", "主 agent 要你跑 54 号层 0 全量", 2),
            case("收窄:「跑出的标记」是定语", "three-way-attack", "收尾在工作区上跑出的标记在 `gate.sh --staged` 的临时 worktree 里对得上", 0),
            case("收窄:名字在前隔着并列与地点，照拦", "implementation-writer", "`check.sh` 与登记给你的阶段在副本上跑", 2),
            case("收窄:括号拆开之后前后两半照判", "implementation-writer", "`check.sh` 与阶段归属表登记给你的阶段（33、53、74 号）都要跑，贴末行", 2),
            case("收窄:括号里的否定只管括号里", "implementation-writer", "跑 `nice -n 19 bash .claude/scripts/check.sh`（别抢主工作区 target 锁），原样抄判定行", 2),
            case("收窄:名字在前隔着「由」是别人跑", "implementation-writer", "gate.sh 由 gate-triage 跑", 0),
            case("收窄:「照 59 号那一套判据」是介词宾语", "mutation-triage", "照门禁 59 号那一套判据跑", 0),
            case("收窄:「把层 0 重放改成多线程跑」宾语是改", "implementation-writer", "把层 0 崩溃点重放改成多线程跑", 0),
            case("收窄:e152-tables.py 不是 E152 装置", "general-purpose", "跑完再跑 `python3 research/scripts/e152-tables.py --selftest`", 0),
            case("收窄:「层 0 每个崩溃状态都要跑它」宾语是它", "implementation-writer", "层 0 每个崩溃状态都要跑它", 0),
            case("收窄:「变异表里两行」不是整表", "three-way-forward", "在副本上把 `crates/mutations.tsv` 里新加的两行各跑一遍", 0),
            case("收窄:gate-reuse-check.sh 不是 check.sh", "implementation-writer", "单跑一次 `bash .claude/singlefs-ai-sop/scripts/claude-hooks/gate-reuse-check.sh`", 0),
            case("收窄:「别忘了跑」不是否定", "implementation-writer", "别忘了跑 gate.sh", 2),
            case("收窄:「别人」不是否定", "implementation-writer", "交给别人之前跑 54 号层 0 全量", 2),
            case("收窄:「不变量」不是否定、名字在前", "implementation-writer", "改了 checker 不变量就连层 0 快档一起跑", 2),
            case("收窄:把字句", "implementation-writer", "把 54 号跑一遍", 2),
            case("收窄:没闭合的引号照判", "implementation-writer", "「跑 54 号层 0 全量", 2),
            case("收窄:反引号里的引号是命令", "implementation-writer", '执行 `bash "$ROOT/.claude/gate.d/54-layer0-replay.sh" --full`', 2),
            # 快照：派书记员 / 实现员时要改的文件落在还没判完的三方轮的开工快照里，拒绝
            case("快照:书记员撞开着的轮（清单写 ./.claude/）", "kb-scribe", collided, 2),
            case("快照:实现员撞开着的轮（清单写 tree/crates/）", "implementation-writer", "改 crates/singlefs-core/src/mount.rs 的挂载准入，带测试。", 2),
            case("快照:提示写项目根起的绝对路径、中文文件名", "kb-scribe", f"改 {work}/.claude/kb/decisions/18-块里携带什么信息.md 已定项 11 的射程。", 2),
            case("快照:清单写 defs/.claude/ 也归一", "kb-scribe", "规格：改 `.claude/agents/crash-verifier.md` 的写范围一节。", 2),
            case("快照:规格文件里点名的照拒（路径后面紧跟中文）", "kb-scribe", f"逐条规格在 {specification_path}里，照做。报告写到同一目录。", 2),
            case("快照:路径后面连着别的路径字符不算", "implementation-writer", "对照 crates/singlefs-core/src/mount.rs.orig 那份旧拷贝。", 0),
            case("快照:撞的是已判完的轮，放行", "kb-scribe", "规格：改 `.claude/kb/decisions/03-空间分配.md` 已定项 2。", 0),
            case("快照:有快照没正文的不是三方轮，放行", "implementation-writer", "改 crates/singlefs-core/src/allocator.rs 的分配顺序。", 0),
            case("快照:有放行行，放行", "kb-scribe", collided + "快照冲突已判：r-open 只读它核 C120 的行号，不改那一行\n", 0),
            case("快照:放行行写的是别的轮，照拒", "kb-scribe", collided + "快照冲突已判：r-closed 只读\n", 2),
            case("快照:放行行没写理由，照拒", "kb-scribe", collided + "快照冲突已判：r-open\n", 2),
            case("快照:放行行没写理由、下一行不算它的理由，照拒", "kb-scribe", collided + "快照冲突已判：r-open\n报告写到草稿目录。\n", 2),
            case("快照:放行行理由是占位，照拒", "kb-scribe", collided + "- 快照冲突已判：r-open <理由>\n", 2),
            case("快照:拒绝信息原样转贴，句中的放行模板不算", "kb-scribe", forwarded_refusal, 2),
            case("快照:派别的 agent 放行（通用）", "general-purpose", collided, 0),
            case("快照:派别的 agent 放行（实验设计员）", "experiment-designer", collided, 0),
            case("快照:清单里认不出的行不拒", "kb-scribe", "规格：改 README.md、research/prompts/r-odd-model.md 与 `.claude/kb/pitfalls.md`。", 0),
            case("快照:规格文件不存在不拒", "kb-scribe", f"逐条规格在 {missing_specification_path}，草稿目录 {specification_directory}/，报告写 {specification_directory}/report.md。", 0),
            case_in(unreadable_work, "快照:一份清单读不了，不拒", "kb-scribe", "规格：改 `.claude/kb/pitfalls.md`。", 0),
            case_in(unreadable_work, "快照:一份清单读不了，读得了的那份撞了照拒", "kb-scribe", collided, 2),
        ]
        script = os.path.join(hook_dir, "runner-dispatch-guard.sh")
        def run_hook(project_root, stdin_text):
            return subprocess.run(["bash", script], input=stdin_text, capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=project_root))
        snapshot_refusal = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": collided}}))
        cases.append(("stdin:快照撞了拒绝（退出码 2）", 2, snapshot_refusal.returncode))
        cases.append(("stdin:快照拒绝的 stderr 写了轮名、文件与出路", 1,
                      int(all(needle in snapshot_refusal.stderr for needle in ("r-open", ".claude/kb/checks-owed.md", "等 r-open 的判决写完再派", "快照冲突已判：r-open")))))
        broken_json = run_hook(work, "{不是 JSON")
        cases.append(("stdin:JSON 解析不了放行", 0, broken_json.returncode))
        cases.append(("stdin:JSON 解析不了时 stderr 写了错误", 1, int("JSON" in broken_json.stderr)))
        unreadable_list = run_hook(unreadable_work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": "规格：改 `.claude/kb/pitfalls.md`。"}}))
        cases.append(("stdin:快照清单读不了放行", 0, unreadable_list.returncode))
        cases.append(("stdin:快照清单读不了时 stderr 写了是哪一份", 1, int("读不了快照清单" in unreadable_list.stderr and "gone-sha256.txt" in unreadable_list.stderr)))
        for label, prompt, want in (("stdin:续做没写还差", again + "这一段回答的岔路：岔路 1\n", 2),
                                    ("stdin:第一段点名了岔路", first + "这一段回答的岔路：岔路 1\n", 0)):
            completed = subprocess.run(["bash", script],
                                       input=json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "experiment-runner", "prompt": prompt}}),
                                       capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=work))
            cases.append((label, want, completed.returncode))
        heavy = subprocess.run(["bash", script],
                               input=json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "implementation-writer", "prompt": "交回前跑层 0 快档。"}}),
                               capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=work))
        cases.append(("stdin:派实现员跑层 0 拒绝（退出码 2）", 2, heavy.returncode))
        cases.append(("stdin:拒绝时 stderr 写了出路", 1, int("→ 怎么办" in heavy.stderr and "弹窗" in heavy.stderr)))
    finally:
        shutil.rmtree(work)
        for directory in other_directories:
            shutil.rmtree(directory, ignore_errors=True)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide()、heavy_test_requests() 与 snapshot_conflict_verdict() 的判法；RUNNER_DISPATCH_GUARD_DISABLE_CHECK、_DISABLE_HEAVY_TEST、"
              "_DISABLE_SNAPSHOT 或 _BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(cases)} 种）：非执行员与只修锚点放行，没点名岔路、续做没写还差或还差写无的拒绝；派发提示里动词的宾语是越出自己那一份的重型阶段就拒，"
          "否定句、引号里的、转述、宾语是别的东西、定语、轻阶段、crash-verifier 与 gate-triage 各自那一份放行；派书记员或实现员时要改的文件在没判完的三方快照里就拒，"
          "已判完的轮、没正文的快照、写了放行行的、派别的 agent 的、认不出的清单行、不在的规格与读不了的清单放行")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except ValueError as error:
        if not break_switch_is("json-error-silent"):
            print(f"! 派发闸读不懂 hook 的 JSON，这一次放行：{error}", file=sys.stderr)
        return 2 if break_switch_is("json-error-denies") else 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    code, message = decide(hook_input, project_root)
    if message:
        print(message, file=sys.stderr)
    return code

sys.exit(main())
PY
