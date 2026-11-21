#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（AskUserQuestion）：放置类的弹窗（东西放哪个包、哪个目录，要不要新开包）问句没点名位置标准、选项没写「标准：」或「偏离：」的，执行前拒绝（退出 2，stderr 写哪一问缺什么与出路）。
# hook-events: PreToolUse:AskUserQuestion
# gate-similar: ask-user-claim-guard.sh 同挂 PreToolUse[AskUserQuestion]、同读每一问的问句与选项说明；它逐句判断言词有没有出处（证据纪律），这里按问判放置类问题有没有对着 .claude/rules/directory-layout.md「crates/：各包放什么」列选项（目录规矩），词表、判据与出路没有一条相同；并进去一份钩子要同时管两份规则、两套弄坏开关与自证搅在一起，改一边要重证另一边
# gate-similar: runner-dispatch-guard.sh 也在 PreToolUse 上判一段给人看的中文，但挂 Agent|Task、判派发提示；弹窗不经它，并进去要让它多认一个工具
# selftest-serial: 自证的格都在内存里判、另起几次自己的入口，一共不到一秒，开并行省不下时间
# gate-overlap:copy-kept ask-user-claim-guard.sh 判不了时「放行但必记检出」（unjudged_pass）、钩子入口（main）与平常跑拒弄坏开关那几段照它的写法各一份、不带判据；抽成共用模块要改 ask-user-claim-guard.sh，这一次不碰别的钩子，交主 agent 定要不要抽
#
# 判法：扫 tool_input.questions 里每一问的 question 与每个选项的 label、description（header 不扫），这几段各判各的：
#   ① 放置类：有一段（「」括着的原文先去掉，轮名、引的原话不算）里有下面三样之一：
#      问处词 WHERE_WORDS 之一紧跟在放置词 PLACEMENT_WORDS 之一或单字 WHERE_SINGLE_LEADS 之一后面（中间可隔空白）；
#      放置词之后、同一小句里（到 CLAUSE_STOPS 之一或换行为止）有落点词 TARGET_WORDS 之一；
#      开包词 OPEN_PACKAGE_WORDS 之一之后、同一小句里隔 OPEN_PACKAGE_GAP_MAXIMUM 个字以内有「包」。
#      这几组词、小句的断处与「包」不算整词的那几串 NON_PACKAGE_COMPOUNDS，就是 .claude/rules/directory-layout.md「crates/：各包放什么」里
#      「放置类问题怎么认」那一条列的（RULE_WORD_LABELS：规则里每组前面的名字与那一组，同一份、同一个次序），自证逐组对过（rule_word_list_mismatches）；
#      那一条写的隔字上限、「弹窗问放置类问题」那一条写的原话字数下限，自证对着 OPEN_PACKAGE_GAP_MAXIMUM、QUOTE_MINIMUM 核（rule_phrase_mismatches）。
#      认法：「包」按整词认，落在 NON_PACKAGE_COMPOUNDS（包括、包含……）里的那个「包」不算；
#      英文词（crate）前后不接字母、数字与下划线，带复数 s 的照认、不分大小写；词里的空格认任意空白（「哪个 crate」「哪个crate」都算）。
#      不是放置类的问一律放行：「日志在哪个目录里找」「每个文件落在哪个包」的问处词前面不是放置词；「复跑的输出放在草稿目录」
#      「放在提交前做，还是对验证包跑」放置词后面同一小句里没有落点词；「这一行放进 kb 吗」没有落点词；「新开一趟合入后验证再对验证包跑」的「包」隔得远；
#      「这一条放进哪个包括出处的表里」里的「包」不是整词。
#   ② 放置类的问要齐下面四样，缺一样那一问记一条（种类名在括号里）：
#      问句里写了 RULE_FILE_NAME（directory-layout.md）与小节名 SECTION_TITLE（crates/：各包放什么）（citation）；
#      每个选项的说明去掉开头空白之后以「标准：」或「偏离：」起头，说明缺了也算没起头（prefix）；
#      每个「偏离：」选项的说明里至少有一处「」括着的原话，去掉空白至少 QUOTE_MINIMUM 个字，是那一节正文（去掉空白比）里的一段（deviation）；
#      至少一个选项以「标准：」起头（standard）。
#   那一节正文从这份钩子旁边的 ../rules/directory-layout.md 现读：标题行是「## crates/：各包放什么」，到下一个「## 」为止。
#   有一条记的就退 2，stderr 逐问列出缺什么；都齐就放行。
# 判不了的照「放行但必记检出」：输入不是 JSON 对象、tool_input.questions 不是列表、判的时候抛异常、遇到放置类的问而那一节正文读不出（文件不在、没有那个标题），
#   照常退 0，stderr 写一行卡在哪，检出记录（AGENT_HOOK_DETECTIONS，没设取同目录 lib_work_root.py 的 default_detections_path；写法在它的 record_unjudged_pass）
#   追加一行、结局「只记」，findings 以「放置类弹窗闸没判、照常放行」起头；读不到 lib_work_root.py 时 stderr 另说一句没记成。
# 弄坏开关只在自证里认：平常跑（不是 --selftest，也不是自证起的子进程）设了 ASK_USER_PLACEMENT_GUARD_BREAK 就退 2，stderr 点名它只在自证里认；
#   自证起的子进程由环境里的 ASK_USER_PLACEMENT_GUARD_SELFTEST_MARK 认（指的那份文件在、内容是 SELFTEST_MARK_CONTENT）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红（「」里是失败行「✗ 自检：」之后那一段的开头）：
#   ASK_USER_PLACEMENT_GUARD_BREAK=placement-words-ignored  认不出放置类，每一问都放行
#       「必拒：放置类问句没点名规则那一节」「必拒：选项说明没以「标准：」「偏离：」起头」「必拒：「偏离：」没引规则原话」
#       「必拒：「偏离：」引的话规则里没有」「必拒：没有一个「标准：」选项」「必拒：放置类的词只在选项说明里也认」「必拒：原话那一问」
#       「必拒：选项说明里放进 crates/ 路径也认」「必拒：放在哪个包」「必拒：偏离只写在选项标题里」「必拒：放进哪个目录」
#       「必拒：放进哪个 crate」「必拒：问处词前是单字归（归哪个包）」「必拒：开包词紧挨着包（要不要新开包）」
#       「必拒：放到实验包还是验证包」「必拒：新开一个包」「必拒：单开包还是并进验证包」「必拒：归哪一个包」「必拒：移到 crates/ 下的路径」
#       「必拒：挪进哪个子目录」「必拒：放哪个包（对照）」
#       「stdin：选项说明没起头的拒绝」「判不了：放置类而规则那一节读不出放行并记一条检出」
#   ASK_USER_PLACEMENT_GUARD_BREAK=placement-target-ignored  放置词加落点词那一支不认，只认问处词与开包词
#       「必拒：放置类问句没点名规则那一节」「必拒：选项说明没以「标准：」「偏离：」起头」「必拒：「偏离：」没引规则原话」
#       「必拒：「偏离：」引的话规则里没有」「必拒：没有一个「标准：」选项」「必拒：放置类的词只在选项说明里也认」「必拒：原话那一问」
#       「必拒：选项说明里放进 crates/ 路径也认」「必拒：偏离只写在选项标题里」「必拒：放到实验包还是验证包」「必拒：移到 crates/ 下的路径」
#       「stdin：选项说明没起头的拒绝」「判不了：放置类而规则那一节读不出放行并记一条检出」
#   ASK_USER_PLACEMENT_GUARD_BREAK=target-words-unrequired  有放置词就算放置类，不另要同一小句里的落点词
#       「必放：有放置词而没有落点词（这一行放进 kb 吗）」「必放：放进 kb 而说明写「包括」」「必放：放在当时间用（放在提交前做，还是对验证包跑）」
#       「必放：输出放在草稿目录还是 research/results/」「必放：放进之后同一小句的包括不算落点」「必放：放进哪个包括出处的表不算问处词」
#       「stdin：放进 kb 的问句放行」
#   ASK_USER_PLACEMENT_GUARD_BREAK=span-punctuation-unchecked  小句不断在标点上，放置词与开包词往后一直看到那一段末尾
#       「必放：放在当时间用（放在提交前做，还是对验证包跑）」
#   ASK_USER_PLACEMENT_GUARD_BREAK=where-words-ignored  问处词那一支不认，只认放置词加落点词与开包词
#       「必拒：放进哪个目录」「必拒：问处词前是单字归（归哪个包）」「必拒：归哪一个包」「必拒：挪进哪个子目录」「必拒：放哪个包（对照）」
#   ASK_USER_PLACEMENT_GUARD_BREAK=where-lead-unrequired  问处词前面不另要放置词或单字，出现就算
#       「必放：问处词前不是放置词（日志在哪个目录里找）」「必放：问处词前不是放置词（每个文件落在哪个包）」
#   ASK_USER_PLACEMENT_GUARD_BREAK=single-lead-words-ignored  问处词前面只认放置词，单字「放」「归」不认
#       「必拒：问处词前是单字归（归哪个包）」「必拒：归哪一个包」「必拒：放哪个包（对照）」
#   ASK_USER_PLACEMENT_GUARD_BREAK=open-package-words-ignored  开包词那一支不认
#       「必拒：开包词紧挨着包（要不要新开包）」「必拒：新开一个包」
#   ASK_USER_PLACEMENT_GUARD_BREAK=open-package-gap-unchecked  开包词后面同一小句里隔多远的「包」都算
#       「必放：新开之后隔得远的包不算开包」
#   ASK_USER_PLACEMENT_GUARD_BREAK=label-unscanned  选项标题不扫，只扫问句与选项说明
#       「必拒：偏离只写在选项标题里」
#   ASK_USER_PLACEMENT_GUARD_BREAK=package-whole-word-unchecked  「包」按子串认，「包括」「包含」里的也算
#       「必放：放进之后同一小句的包括不算落点」「必放：放进哪个包括出处的表不算问处词」
#   ASK_USER_PLACEMENT_GUARD_BREAK=quoted-text-scanned  「」括着的原文也扫
#       「必放：问句里括起来的轮名带搬进实验包不认」
#   ASK_USER_PLACEMENT_GUARD_BREAK=rule-word-lists-unchecked  规则那一条列的词与钩子词表不对（rule_word_list_mismatches 一律交空）
#       「样本规则少一个放置词判得出与钩子词表对不上」
#   ASK_USER_PLACEMENT_GUARD_BREAK=rule-phrases-unchecked  规则写的原话字数下限、开包隔字上限与钩子常量不对（rule_phrase_mismatches 一律交空）
#       「样本规则把原话字数下限写成别的数判得出」
#   ASK_USER_PLACEMENT_GUARD_BREAK=rule-citation-unchecked  问句不点名规则与小节也放行
#       「必拒：放置类问句没点名规则那一节」「必拒：放置类的词只在选项说明里也认」「必拒：原话那一问」
#       「必拒：选项说明里放进 crates/ 路径也认」「必拒：放在哪个包」「必拒：偏离只写在选项标题里」「必拒：放进哪个目录」
#       「必拒：放进哪个 crate」「必拒：问处词前是单字归（归哪个包）」「必拒：开包词紧挨着包（要不要新开包）」
#       「必拒：放到实验包还是验证包」「必拒：新开一个包」「必拒：单开包还是并进验证包」「必拒：归哪一个包」「必拒：移到 crates/ 下的路径」
#       「必拒：挪进哪个子目录」「必拒：放哪个包（对照）」
#   ASK_USER_PLACEMENT_GUARD_BREAK=option-prefix-unchecked  选项说明不以「标准：」「偏离：」起头也放行
#       「必拒：选项说明没以「标准：」「偏离：」起头」「必拒：原话那一问」「stdin：选项说明没起头的拒绝」「必拒：放在哪个包」
#       「必拒：偏离只写在选项标题里」「必拒：放进哪个目录」「必拒：放进哪个 crate」「必拒：问处词前是单字归（归哪个包）」「必拒：开包词紧挨着包（要不要新开包）」
#       「必拒：放到实验包还是验证包」「必拒：新开一个包」「必拒：单开包还是并进验证包」「必拒：归哪一个包」「必拒：移到 crates/ 下的路径」
#       「必拒：挪进哪个子目录」「必拒：放哪个包（对照）」
#   ASK_USER_PLACEMENT_GUARD_BREAK=deviation-rule-unchecked  「偏离：」不引那一节的原话也放行
#       「必拒：「偏离：」没引规则原话」「必拒：「偏离：」引的话规则里没有」
#   ASK_USER_PLACEMENT_GUARD_BREAK=standard-option-unchecked  一个「标准：」选项都没有也放行
#       「必拒：没有一个「标准：」选项」「必拒：原话那一问」「必拒：放在哪个包」「必拒：偏离只写在选项标题里」「必拒：放进哪个目录」
#       「必拒：放进哪个 crate」「必拒：问处词前是单字归（归哪个包）」「必拒：开包词紧挨着包（要不要新开包）」
#       「必拒：放到实验包还是验证包」「必拒：新开一个包」「必拒：单开包还是并进验证包」「必拒：归哪一个包」「必拒：移到 crates/ 下的路径」
#       「必拒：挪进哪个子目录」「必拒：放哪个包（对照）」
#   ASK_USER_PLACEMENT_GUARD_BREAK=input-unrecorded  输入不是 JSON 对象时只放行、不记检出
#       「stdin：坏 JSON 放行并记一条检出（结局只记、会话号读不到）」「stdin：JSON 数组放行并记一条检出（结局只记、会话号读不到）」
#   ASK_USER_PLACEMENT_GUARD_BREAK=questions-unrecorded  tool_input.questions 不是列表时只放行、不记检出
#       「stdin：questions 不是列表放行并记一条检出（结局只记、会话号照输入）」
#   ASK_USER_PLACEMENT_GUARD_BREAK=exception-unrecorded  判的时候抛异常不接住，照原样抛出（进程退 1，Claude Code 照常放行、不记检出）
#       「判的时候抛异常放行并记一条检出（结局只记、会话号照输入）」
#   ASK_USER_PLACEMENT_GUARD_BREAK=section-unrecorded  放置类而规则那一节读不出时只放行、不记检出
#       「判不了：放置类而规则那一节读不出放行并记一条检出」
# 管不到的：换了说法的放置类问题认不出：词表外的「落在哪里」「搁哪」，问处词前面不是放置词的「该在哪个包里」，
#   放置词与落点词之间隔了标点的「放到产品、验证两个包之一」；放置类的话只写在「」里的（「」整段不认）认不出；
#   放置词后面同一小句里有落点词而问的不是放置的事（不隔标点的「放在提交前对验证包跑一趟」）、问处词前面那个字碰巧是「放」「归」的（「回归哪个包」）照判；
#   「标准：」那个选项是不是真照标准、「偏离：」引的那一条是不是它真偏离的那一条，靠人。
#
#   ask-user-placement-guard.sh             # 从 stdin 读 hook 的 JSON
#   ask-user-placement-guard.sh --selftest  # 走一遍必拒与必放的弹窗，另有几例走真实的 stdin 入口
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（写成 `python3 - <<PY` 时程序占了标准输入，JSON 读不到，判定一律放行）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, subprocess, sys, tempfile

HOOK_NAME = "ask-user-placement-guard.sh"
BREAK_VARIABLE = "ASK_USER_PLACEMENT_GUARD_BREAK"
SELFTEST_MARK_VARIABLE = "ASK_USER_PLACEMENT_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "ask-user-placement-guard.sh --selftest 起的子进程"
UNJUDGED_PREFIX = "放置类弹窗闸没判、照常放行"
INPUT_PREFIX_BYTES = 200
# 这几组词与 .claude/rules/directory-layout.md「crates/：各包放什么」里「放置类问题怎么认」那一条列的同一份、同一个次序（RULE_WORD_LABELS 是规则里每组前面的那个名字；自证逐组对）
WHERE_WORDS = ("哪个包", "哪一个包", "哪个目录", "哪个子目录", "哪个 crate")          # 问处词：紧跟在放置词或单字后面才算
PLACEMENT_WORDS = ("放在", "放进", "放到", "搬到", "搬进", "移到", "挪到", "挪进", "住进", "并进")   # 放置词：同一小句里后面有落点词才算
WHERE_SINGLE_LEADS = ("放", "归")                                                   # 单字：只在问处词前面才算
TARGET_WORDS = ("包", "crate")                                                      # 落点词
CLAUSE_STOPS = ("，", "。", "？", "！", "；", "：", "、", ",", "?", "!", ";")       # 小句断在这几个标点与换行上
OPEN_PACKAGE_WORDS = ("新开", "单开", "另开")                                       # 开包词：后面隔 OPEN_PACKAGE_GAP_MAXIMUM 个字以内有「包」才算
OPEN_PACKAGE_GAP_MAXIMUM = 2
NON_PACKAGE_COMPOUNDS = ("包括", "包含", "包装", "包裹", "包办", "包容", "打包")   # 这几串里的「包」不是整词
RULE_WORD_LABELS = (("问处词", WHERE_WORDS), ("放置词", PLACEMENT_WORDS), ("单字", WHERE_SINGLE_LEADS), ("落点词", TARGET_WORDS),
                    ("小句断在", CLAUSE_STOPS), ("开包词", OPEN_PACKAGE_WORDS), ("不算整词的有", NON_PACKAGE_COMPOUNDS))
RULE_WORD_RUN = "((?:「[^「」]+」)+)"   # 规则里一组词：紧挨着的几个「…」
QUOTED_SPAN = re.compile(r"「[^「」]*」")   # 认放置类之前先去掉的「」原文（不嵌套的那一层）
ASCII_WORD_CHARACTER = "A-Za-z0-9_"
RULE_FILE_NAME = "directory-layout.md"
SECTION_TITLE = "crates/：各包放什么"
SECTION_HEADING = "## " + SECTION_TITLE
STANDARD_PREFIX = "标准："
DEVIATION_PREFIX = "偏离："
QUOTE_MINIMUM = 4
# 规则那一节里要写着的两句（去掉空白比），数取钩子的常量：「弹窗问放置类问题」那一条的原话字数下限、「放置类问题怎么认」那一条的开包隔字上限
RULE_PHRASES = (("「偏离：」引原话的字数下限", f"至少 {QUOTE_MINIMUM} 个字"), ("开包词后面的隔字上限", f"隔 {OPEN_PACKAGE_GAP_MAXIMUM} 个字以内"))
QUOTED = re.compile(r"「([^「」]+)」")
KIND_DESCRIPTIONS = {
    "citation": f"问句没点名 .claude/rules/{RULE_FILE_NAME}「{SECTION_TITLE}」",
    "prefix": f"选项说明没以「{STANDARD_PREFIX}」或「{DEVIATION_PREFIX}」起头",
    "deviation": f"「{DEVIATION_PREFIX}」选项没用「」引「{SECTION_TITLE}」里它偏离的那一条的原话（至少 {QUOTE_MINIMUM} 个字）",
    "standard": f"一个以「{STANDARD_PREFIX}」起头的选项都没有",
}

def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")

def without_spaces(text):
    return "".join(text.split())

def section_text(rules_path):
    """规则文件里「## crates/：各包放什么」那一节的正文（标题之后到下一个「## 」为止）；文件读不出、没有那个标题交 None。"""
    try:
        lines = open(rules_path, encoding="utf-8").read().split("\n")
    except OSError:
        return None
    for index, line in enumerate(lines):
        if line.strip() == SECTION_HEADING:
            body = []
            for following in lines[index + 1:]:
                if following.startswith("## "):
                    break
                body.append(following)
            return "\n".join(body)
    return None

def word_pattern(word):
    """一个词的正则：空格认任意空白；英文词前后不接字母、数字与下划线、带复数 s 也认、不分大小写。"""
    body = r"\s*".join(re.escape(part) for part in word.split(" "))
    if re.search(f"[{ASCII_WORD_CHARACTER}]$", word):
        body += "s?" + f"(?![{ASCII_WORD_CHARACTER}])"
    if re.match(f"[{ASCII_WORD_CHARACTER}]", word):
        body = f"(?<![{ASCII_WORD_CHARACTER}])" + body
    return re.compile(body, re.IGNORECASE)

def package_character_is_whole_word(text, index):
    """text[index] 是「包」：不落在 NON_PACKAGE_COMPOUNDS 的哪一串里就是整词（弄坏开关 package-whole-word-unchecked 下一律算整词）。"""
    if broken("package-whole-word-unchecked"):
        return True
    for compound in NON_PACKAGE_COMPOUNDS:
        offset = compound.index("包")
        if index >= offset and text[index - offset:index - offset + len(compound)] == compound:
            return False
    return True

def word_spans(text, word):
    """text 里 word 的每一处 (起, 止)：照 word_pattern 找，命中的那一段里每个「包」都要是整词（package_character_is_whole_word）。"""
    return [(found.start(), found.end()) for found in word_pattern(word).finditer(text)
            if all(package_character_is_whole_word(text, index) for index in range(found.start(), found.end()) if text[index] == "包")]

def clause_end(text, start):
    """从 start 起这一小句到哪里为止：下一个 CLAUSE_STOPS 之一或换行的位置，都没有到 text 末尾（弄坏开关 span-punctuation-unchecked 下一律到末尾）。"""
    if broken("span-punctuation-unchecked"):
        return len(text)
    for index in range(start, len(text)):
        if text[index] in CLAUSE_STOPS or text[index] == "\n":
            return index
    return len(text)

def where_word_after_lead(text):
    """有问处词紧跟在放置词或单字后面（中间可隔空白）。弄坏开关 where-lead-unrequired 下问处词出现就算，single-lead-words-ignored 下单字不认。"""
    leads = PLACEMENT_WORDS + (() if broken("single-lead-words-ignored") else WHERE_SINGLE_LEADS)
    for word in WHERE_WORDS:
        for start, _ in word_spans(text, word):
            if broken("where-lead-unrequired") or text[:start].rstrip().endswith(leads):
                return True
    return False

def placement_word_reaches_target(text):
    """有放置词，它后面同一小句里有落点词（弄坏开关 target-words-unrequired 下有放置词就算）。"""
    for word in PLACEMENT_WORDS:
        for _, end in word_spans(text, word):
            if broken("target-words-unrequired"):
                return True
            stop = clause_end(text, end)
            if any(end <= start and finish <= stop for target in TARGET_WORDS for start, finish in word_spans(text, target)):
                return True
    return False

def opens_a_package(text):
    """有开包词，它后面同一小句里隔 OPEN_PACKAGE_GAP_MAXIMUM 个字以内有整词的「包」（弄坏开关 open-package-gap-unchecked 下同一小句里多远都算）。"""
    for word in OPEN_PACKAGE_WORDS:
        for _, end in word_spans(text, word):
            stop = clause_end(text, end)
            last = stop if broken("open-package-gap-unchecked") else min(stop, end + OPEN_PACKAGE_GAP_MAXIMUM + 1)
            if any(text[index] == "包" and package_character_is_whole_word(text, index) for index in range(end, last)):
                return True
    return False

def placement_texts(question):
    """认放置类要扫的几段：问句、每个选项的 label 与 description（弄坏开关 label-unscanned 下不扫 label），「」括着的原文先去掉（quoted-text-scanned 下不去）。"""
    options = [option for option in question.get("options") or [] if isinstance(option, dict)]
    texts = [question.get("question")] + [option.get("description") for option in options]
    if not broken("label-unscanned"):
        texts += [option.get("label") for option in options]
    texts = [text for text in texts if isinstance(text, str)]
    return texts if broken("quoted-text-scanned") else [QUOTED_SPAN.sub(" ", text) for text in texts]

def is_placement(question):
    """放置类（文件头判法 ①）：有一段里问处词紧跟在放置词或单字后面、放置词后面同一小句里有落点词、或开包词后面隔得近有「包」。
    弄坏开关 where-words-ignored、placement-target-ignored、open-package-words-ignored 各关掉一支，placement-words-ignored 三支都关。"""
    if broken("placement-words-ignored"):
        return False
    for text in placement_texts(question):
        if not broken("where-words-ignored") and where_word_after_lead(text):
            return True
        if not broken("placement-target-ignored") and placement_word_reaches_target(text):
            return True
        if not broken("open-package-words-ignored") and opens_a_package(text):
            return True
    return False

def rule_word_list_mismatches(section):
    """规则那一节里每组词（RULE_WORD_LABELS 的名字后面紧挨着的几个「…」）与钩子的词表逐组比，交对不上的几句；都对得上交 []。
    弄坏开关 rule-word-lists-unchecked 下一律交 []。"""
    if broken("rule-word-lists-unchecked"):
        return []
    found = []
    for label, words in RULE_WORD_LABELS:
        run = re.search(re.escape(label) + RULE_WORD_RUN, section)
        listed = tuple(QUOTED.findall(run.group(1))) if run else None
        if listed != words:
            found.append(f"规则里「{label}」{'那一组读不出' if listed is None else '列的是 ' + '、'.join(listed)}，钩子的是 {'、'.join(words)}")
    return found

def rule_phrase_mismatches(section):
    """规则那一节里（去掉空白比）没写 RULE_PHRASES 的哪几句，交对不上的几句；都写了交 []。弄坏开关 rule-phrases-unchecked 下一律交 []。"""
    if broken("rule-phrases-unchecked"):
        return []
    flattened_section = without_spaces(section)
    return [f"规则里没写{name}「{phrase}」（钩子的常量）" for name, phrase in RULE_PHRASES if without_spaces(phrase) not in flattened_section]

def quotes_the_section(description, section):
    """「偏离：」之后有一处「」括着的原话（去掉空白至少 QUOTE_MINIMUM 个字）是那一节正文里的一段。"""
    flattened_section = without_spaces(section)
    return any(len(without_spaces(quote)) >= QUOTE_MINIMUM and without_spaces(quote) in flattened_section
               for quote in QUOTED.findall(description[len(DEVIATION_PREFIX):]))

def question_problems(question, section):
    """一问放置类的问 → [(种类, 哪里)]；种类见 KIND_DESCRIPTIONS。"""
    found = []
    text = question.get("question") if isinstance(question.get("question"), str) else ""
    if not (RULE_FILE_NAME in text and SECTION_TITLE in text) and not broken("rule-citation-unchecked"):
        found.append(("citation", "问句"))
    options = [option for option in question.get("options") or [] if isinstance(option, dict)]
    standard_count = 0
    for option in options:
        place = f"选项「{option.get('label') or ''}」"
        description = (option.get("description") if isinstance(option.get("description"), str) else "").lstrip()
        if description.startswith(STANDARD_PREFIX):
            standard_count += 1
        elif description.startswith(DEVIATION_PREFIX):
            if not quotes_the_section(description, section) and not broken("deviation-rule-unchecked"):
                found.append(("deviation", place))
        elif not broken("option-prefix-unchecked"):
            found.append(("prefix", place))
    if standard_count == 0 and not broken("standard-option-unchecked"):
        found.append(("standard", "选项"))
    return found

def placement_questions(tool_input):
    """[(第几问, 那一问)]：放置类的问。"""
    questions = tool_input.get("questions") if isinstance(tool_input, dict) else None
    return [(number, question) for number, question in enumerate(questions if isinstance(questions, list) else [], 1)
            if isinstance(question, dict) and is_placement(question)]

def flagged(tool_input, section):
    """[(第几问, 种类, 哪里)]：放置类的问缺的那几样。"""
    return [(number, kind, place) for number, question in placement_questions(tool_input) for kind, place in question_problems(question, section)]

def decide(hook_input, section):
    """返回 (0, None) 放行；(2, 说明) 拒绝。"""
    findings = flagged(hook_input.get("tool_input") or {}, section)
    if not findings:
        return 0, None
    numbers = sorted({number for number, _, _ in findings})
    listed = "\n".join(f"   第 {number} 问的{place}：{KIND_DESCRIPTIONS[kind]}" for number, kind, place in findings)
    def listed_words(words):
        return "".join("「" + word + "」" for word in words)
    return 2, (f"✗ 弹窗里有 {len(numbers)} 问是放置类问题（问句、选项标题或选项说明的一段里，问处词{listed_words(WHERE_WORDS)}之一"
               f"紧跟在放置词或单字{listed_words(WHERE_SINGLE_LEADS)}后面，或放置词{listed_words(PLACEMENT_WORDS)}之一后面同一小句里有落点词{listed_words(TARGET_WORDS)}之一，"
               f"或开包词{listed_words(OPEN_PACKAGE_WORDS)}之一后面隔 {OPEN_PACKAGE_GAP_MAXIMUM} 个字以内有「包」；「」括着的原文不算），"
               f"写法不合 .claude/rules/{RULE_FILE_NAME}「{SECTION_TITLE}」，缺 {len(findings)} 样：\n"
               f"{listed}\n"
               f"→ 怎么办：问句里写 `.claude/rules/{RULE_FILE_NAME}`「{SECTION_TITLE}」；每个选项的说明以「{STANDARD_PREFIX}」或「{DEVIATION_PREFIX}」起头，"
               f"照那一节的位置标准放的写「{STANDARD_PREFIX}」，「{DEVIATION_PREFIX}」后面用「」照抄它偏离的那一条的原话（至少 {QUOTE_MINIMUM} 个字）；"
               f"至少一个选项以「{STANDARD_PREFIX}」起头。")

SAMPLE_SECTION = (
    "\n- 一个包的集成测试放它自己的 `tests/`，单元测试放 `src/` 里的 `#[cfg(test)]`；不为测试另开包。\n"
    "- 实验装置二进制只放实验装置包；没有包依赖实验装置包。\n"
    "- 产品包不放二进制（`src/bin/`、`[[bin]]`），装置二进制尤其不放。\n")
CITED = f"E202 的装置放哪？位置标准见 `.claude/rules/{RULE_FILE_NAME}`「{SECTION_TITLE}」"
STANDARD_OPTION = {"label": "实验包", "description": "标准：住进实验包，「实验装置二进制只放实验装置包」"}
DEVIATION_OPTION = {"label": "测试包", "description": "偏离：「不为测试另开包」——整份搬进测试包"}
NEUTRAL_OPTIONS = ({"label": "是", "description": "是"}, {"label": "否", "description": "否"})
# (说明, 问句, 该不该拦)：选项写成中性的 NEUTRAL_OPTIONS、只靠问句判的几问；该拦的三样都缺（没点名规则那一节、选项没起头、没有「标准：」）
MODEL_QUESTIONS = (
    ("必放：放在当时间用（放在提交前做，还是对验证包跑）", "合入后验证放在提交前做，还是现在就对验证包跑一趟？", False),
    ("必放：输出放在草稿目录还是 research/results/", "复跑的输出放在草稿目录还是 research/results/ 下？", False),
    ("必放：调度记录标题没加引号带归位", "产品GPU包测试与E202装置归位那一批先关哪几件？", False),
    ("必放：问处词前不是放置词（日志在哪个目录里找）", "这一趟的日志在哪个目录里找？", False),
    ("必拒：放到实验包还是验证包", "E202 的装置放到实验包还是验证包？", True),
    ("必拒：新开一个包", "要不要为 E202 新开一个包？", True),
    ("必拒：单开包还是并进验证包", "配对键那个包该单开包，还是并进验证包？", True),
    ("必拒：归哪一个包", "E202 归哪一个包？", True),
    ("必拒：移到 crates/ 下的路径", "E202 的装置移到 crates/singlefs-implementation-experiments 行吗？", True),
    ("必拒：挪进哪个子目录", "这批装置挪进哪个子目录？", True),
    ("必拒：放哪个包（对照）", "E202 放哪个包？", True),
    ("必放：整包跑哪几个包（对照）", "整包跑哪几个包？", False),
)

def selftest(hook_dir):
    # (说明, tool_input, 应当记的种类集合——空集就是应当放行)
    cases = [
        ("必放：不是放置类的问句", {"questions": [{"question": "岔路 7 选哪一条？", "options": [{"label": "甲", "description": "加一条会红的用例"}]}]}, set()),
        ("必放：放置类，问句点名规则那一节，一个「标准：」一个引了原话的「偏离：」",
         {"questions": [{"question": CITED, "options": [STANDARD_OPTION, DEVIATION_OPTION]}]}, set()),
        ("必放：几问里只判放置类那一问", {"questions": [{"question": "岔路 3 选哪一条？", "options": [{"label": "甲", "description": "照旧"}]},
                                                    {"question": CITED, "options": [STANDARD_OPTION]}]}, set()),
        ("必放：说明开头有空白照认起头", {"questions": [{"question": CITED, "options": [
            {"label": "实验包", "description": "  " + STANDARD_OPTION["description"]}]}]}, set()),
        ("必拒：放置类问句没点名规则那一节", {"questions": [{"question": "E202 的装置放哪？", "options": [STANDARD_OPTION, DEVIATION_OPTION]}]}, {"citation"}),
        ("必拒：选项说明没以「标准：」「偏离：」起头", {"questions": [{"question": CITED, "options": [
            STANDARD_OPTION, {"label": "测试包", "description": "整份搬进测试包"}]}]}, {"prefix"}),
        ("必拒：「偏离：」没引规则原话", {"questions": [{"question": CITED, "options": [
            STANDARD_OPTION, {"label": "测试包", "description": "偏离：放进测试包，偏离第二条"}]}]}, {"deviation"}),
        ("必拒：「偏离：」引的话规则里没有", {"questions": [{"question": CITED, "options": [
            STANDARD_OPTION, {"label": "测试包", "description": "偏离：「测试可以另开一个包」"}]}]}, {"deviation"}),
        ("必拒：没有一个「标准：」选项", {"questions": [{"question": CITED, "options": [
            DEVIATION_OPTION, {"label": "产品包", "description": "偏离：「产品包不放二进制」——留在产品包"}]}]}, {"standard"}),
        ("必拒：放置类的词只在选项说明里也认", {"questions": [{"question": "E202 怎么处理？", "options": [
            {"label": "实验包", "description": "标准：搬进实验包，「实验装置二进制只放实验装置包」"}]}]}, {"citation"}),
        ("必拒：原话那一问", {"questions": [{"question": "E202 的装置怎么办？", "options": [
            {"label": "整份搬进测试包", "description": "整份搬进测试包"}, {"label": "留在原处", "description": "留在产品包里"}]}]},
         {"citation", "prefix", "standard"}),
        ("必放：有放置词而没有落点词（这一行放进 kb 吗）", {"questions": [{"question": "这一行放进 kb 吗？", "options": [
            {"label": "放进", "description": "写进 .claude/kb/checks-owed.md 末尾"}, {"label": "不放", "description": "留在这一轮的调度记录里"}]}]}, set()),
        ("必拒：选项说明里放进 crates/ 路径也认", {"questions": [{"question": "这份测试放进哪里？", "options": [
            {"label": "自己的 tests/", "description": "标准：放进 crates/singlefs-verification/tests/"}]}]}, {"citation"}),
        ("必放：没有 questions", {}, set()),
        ("必放：问实验包起什么名字（没有放置词与问处词）", {"questions": [{"question": "实验包起什么名字？", "options": [
            {"label": "singlefs-implementation-experiments", "description": "照第二轮正推腿推荐的名字"}, {"label": "singlefs-experiments", "description": "短一些"}]}]}, set()),
        ("必放：问句里括起来的轮名带搬进实验包不认", {"questions": [{"question": "「E202 装置搬进实验包」这一轮的规则改动先单独提交吗？", "options": [
            {"label": "先提交", "description": "规则与门禁先进一次提交"}, {"label": "等拆包落地", "description": "和拆包一起提交"}]}]}, set()),
        ("必放：放进 kb 而说明写「包括」", {"questions": [{"question": "这条欠账放进 .claude/kb/checks-owed.md 吗？", "options": [
            {"label": "放进", "description": "包括出处与判据两列"}, {"label": "不放", "description": "留在这一轮的调度记录"}]}]}, set()),
        ("必放：「哪个包括」不算问处词", {"questions": [{"question": "这几条里哪个包括出处？", "options": [
            {"label": "甲", "description": "写了出处与判据"}]}]}, set()),
        ("必放：只问测试范围、选项里点名测试包与实验包（整包跑哪几个包）", {"questions": [{"question": (
            "拆包两簇合入后验证列项退 3：列得出的 37 项：实验包 23、产品 GPU 包 10、验证包 / 函数单元图 / 测试支撑 crate / 测试包各 --lib 1 项。"
            "验证包被改的只有 Cargo.toml 的 [dependencies]/[package] 与搬走的 src/bin 装置，它的测试一份没改。整包跑哪几个包？"), "options": [
            {"label": "点 37 项加三包整包（推荐）", "description": "再整包跑产品 GPU 包、测试支撑 crate、实验包三个包；测试包下一步就删，不整包。"},
            {"label": "五包都整包", "description": "上面三包之外，验证包与测试包也整包跑。"},
            {"label": "只点 37 项不整包", "description": "判不出的那 17 处全部留给提交时的整轮门禁；另外 build 会把五个包都编一遍。"}]}]}, set()),
        ("必拒：放在哪个包", {"questions": [{"question": "E202 的装置放在哪个包？", "options": [
            {"label": "整份搬到 -tests 那个包", "description": "跟十个测试目标一起搬到 singlefs-background-gpu-tests 的 src/bin/"},
            {"label": "留在产品 GPU 包", "description": "留在 singlefs-background-gpu 的 src/bin/"}]}]}, {"citation", "prefix", "standard"}),
        ("必拒：偏离只写在选项标题里", {"questions": [{"question": "E202 的装置怎么办？", "options": [
            {"label": "整份搬进测试包", "description": "跟十个测试目标放一起"}, {"label": "留在原处", "description": "不动"}]}]}, {"citation", "prefix", "standard"}),
        ("必拒：放进哪个目录", {"questions": [{"question": "E202 的装置放进哪个目录？", "options": [
            {"label": "scripts/experiment-apparatus/", "description": "跟 E163 的装置放一起"}, {"label": "research/", "description": "跟产物放一起"}]}]},
         {"citation", "prefix", "standard"}),
        ("必拒：放进哪个 crate", {"questions": [{"question": "这十个测试目标放进哪个 crate？", "options": [
            {"label": "新开一个 -tests crate", "description": "依赖产品 GPU crate 与验证 crate"}, {"label": "放回产品 GPU crate", "description": "它自己的 tests/"}]}]},
         {"citation", "prefix", "standard"}),
        ("必拒：问处词前是单字归（归哪个包）", {"questions": [{"question": "E202 的装置归哪个包？", "options": [
            {"label": "实验包", "description": "跟 E185 的装置放一起"}, {"label": "测试包", "description": "跟十个测试目标放一起"}]}]}, {"citation", "prefix", "standard"}),
        ("必拒：开包词紧挨着包（要不要新开包）", {"questions": [{"question": "这十个测试要不要新开包？", "options": [
            {"label": "新开", "description": "另起一个 -tests 包"}, {"label": "不开", "description": "住产品 GPU 包自己的 tests/"}]}]}, {"citation", "prefix", "standard"}),
    ]
    unready = {"citation", "prefix", "standard"}
    cases += [(label, {"questions": [{"question": question, "options": [dict(option) for option in NEUTRAL_OPTIONS]}]}, unready if blocked else set())
              for label, question, blocked in MODEL_QUESTIONS]
    cases += [
        ("必放：问处词前不是放置词（每个文件落在哪个包）", {"questions": [{"question": (
            "crate-cross-references 格的复用键收『包目录、文件相对路径』，与 D13（验证路线） 已定项 17 的「一律不进键」相抵；"
            "它判的就是每个文件落在哪个包，键里留不留路径？"), "options": [
            {"label": "D13 开第二个例外", "description": "判文件归属包的这一格，键里收包目录与相对仓根的路径，只用来判归属"},
            {"label": "键里去掉路径", "description": "文件换了归属包而键不变，复用会交出错的判定"}]}]}, set()),
        ("必放：放进之后同一小句的包括不算落点", {"questions": [{"question": "这条欠账放进 checks-owed.md 包括出处吗？", "options": [
            dict(option) for option in NEUTRAL_OPTIONS]}]}, set()),
        ("必放：放进哪个包括出处的表不算问处词", {"questions": [{"question": "这一条放进哪个包括出处的表里？", "options": [
            dict(option) for option in NEUTRAL_OPTIONS]}]}, set()),
        ("必放：新开之后隔得远的包不算开包", {"questions": [{"question": "要不要新开一趟合入后验证再对验证包跑？", "options": [
            dict(option) for option in NEUTRAL_OPTIONS]}]}, set()),
    ]
    results = []
    for label, tool_input, want in cases:
        results.append((label, want, {kind for _, kind, _ in flagged(tool_input, SAMPLE_SECTION)}))
    repository_section = section_text(os.path.join(hook_dir, "..", "rules", RULE_FILE_NAME))
    results.append((f"规则文件里有「{SECTION_HEADING}」那一节、样本引的原话都在里面", (True, True),
                    (repository_section is not None,
                     repository_section is not None and all(quotes_the_section(option["description"], repository_section) for option in (DEVIATION_OPTION,)))))
    results.append(("规则那一条列的各组词与钩子词表相同", [],
                    rule_word_list_mismatches(repository_section) if repository_section is not None else ["规则那一节读不出"]))
    sample_word_line = "；".join(label + "".join(f"「{word}」" for word in (words[:-1] if words is PLACEMENT_WORDS else words))
                                for label, words in RULE_WORD_LABELS)
    results.append(("样本规则少一个放置词判得出与钩子词表对不上", True, bool(rule_word_list_mismatches(sample_word_line))))
    results.append(("规则写的原话字数下限与开包隔字上限与钩子常量相同", [],
                    rule_phrase_mismatches(repository_section) if repository_section is not None else ["规则那一节读不出"]))
    sample_phrase_section = f"「偏离：」后面的原话至少 {QUOTE_MINIMUM + 1} 个字；开包词后面隔 {OPEN_PACKAGE_GAP_MAXIMUM} 个字以内有「包」"
    results.append(("样本规则把原话字数下限写成别的数判得出", True, bool(rule_phrase_mismatches(sample_phrase_section))))
    script = os.path.join(hook_dir, HOOK_NAME)
    scratch = tempfile.mkdtemp(prefix="ask-user-placement-guard-selftest-")
    mark = os.path.join(scratch, "selftest-mark")   # 自证起的子进程带这份标记，照样认调用方设的弄坏开关
    with open(mark, "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_MARK_CONTENT + "\n")
    detections = os.path.join(scratch, "detections.jsonl")   # 检出记在自证自己的临时目录，不碰真的检出记录
    environment = dict(os.environ, **{SELFTEST_MARK_VARIABLE: mark, "AGENT_HOOK_DETECTIONS": detections})

    def recorded():
        try:
            with open(detections, encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except OSError:
            return []

    for label, tool_input, want_code, want_in_stderr in (
            ("stdin：选项说明没起头的拒绝", cases[5][1], 2, "选项说明没以「标准：」或「偏离：」起头"),
            ("stdin：写齐了的放置类放行", cases[1][1], 0, ""),
            ("stdin：不是放置类的放行", cases[0][1], 0, ""),
            ("stdin：放进 kb 的问句放行", cases[11][1], 0, "")):
        completed = subprocess.run(["bash", script], input=json.dumps({"tool_name": "AskUserQuestion", "tool_input": tool_input}),
                                   capture_output=True, text=True, env=environment)
        stderr_ok = want_in_stderr in completed.stderr if want_in_stderr else completed.stderr == ""
        results.append((label, (want_code, True), (completed.returncode, stderr_ok)))
    # 判不了的照「放行但必记检出」：退 0、stderr 写一行、检出记录多一行（结局只记）
    for label, raw, want_session in (
            ("stdin：坏 JSON 放行并记一条检出（结局只记、会话号读不到）", "{不是 JSON", "读不到"),
            ("stdin：JSON 数组放行并记一条检出（结局只记、会话号读不到）", "[]", "读不到"),
            ("stdin：questions 不是列表放行并记一条检出（结局只记、会话号照输入）",
             json.dumps({"tool_name": "AskUserQuestion", "session_id": "s-shape", "tool_input": {"questions": "E202 放哪"}}), "s-shape")):
        before = len(recorded())
        completed = subprocess.run(["bash", script], input=raw, capture_output=True, text=True, env=environment)
        added = recorded()[before:]
        results.append((label, (0, True, 1, "只记", want_session, True),
                        (completed.returncode, HOOK_NAME in completed.stderr, len(added), added[0].get("outcome") if added else None,
                         added[0].get("session_id") if added else None,
                         bool(added) and all(finding.startswith(UNJUDGED_PREFIX) for finding in added[0].get("findings") or []))))
    saved_detections = os.environ.get("AGENT_HOOK_DETECTIONS")
    os.environ["AGENT_HOOK_DETECTIONS"] = detections
    # 判的时候抛异常：在进程里把判法换成一个抛异常的，照钩子入口调 judge_input
    original_flagged = globals()["flagged"]
    def raising_flagged(tool_input, section):
        raise RuntimeError("自证造的异常")
    globals()["flagged"] = raising_flagged
    before = len(recorded())
    try:
        got_code = judge_input(json.dumps({"tool_name": "AskUserQuestion", "session_id": "s-raise", "tool_input": cases[4][1]}).encode("utf-8"), hook_dir)
    except Exception as error:   # 弄坏开关 exception-unrecorded：照原样抛出
        got_code = f"抛出 {type(error).__name__}"
    finally:
        globals()["flagged"] = original_flagged
    added = recorded()[before:]
    results.append(("判的时候抛异常放行并记一条检出（结局只记、会话号照输入）", (0, 1, "只记", "s-raise"),
                    (got_code, len(added), added[0].get("outcome") if added else None, added[0].get("session_id") if added else None)))
    # 放置类而规则那一节读不出：钩子目录换成一个旁边没有规则文件的临时目录（lib_work_root.py 照样从真的钩子目录读）
    before = len(recorded())
    got_code = judge_input(json.dumps({"tool_name": "AskUserQuestion", "session_id": "s-section", "tool_input": cases[4][1]}).encode("utf-8"),
                           hook_dir, rules_path=os.path.join(scratch, "no-such-rules.md"))
    added = recorded()[before:]
    results.append(("判不了：放置类而规则那一节读不出放行并记一条检出", (0, 1, "只记", "s-section"),
                    (got_code, len(added), added[0].get("outcome") if added else None, added[0].get("session_id") if added else None)))
    if saved_detections is None:
        os.environ.pop("AGENT_HOOK_DETECTIONS", None)
    else:
        os.environ["AGENT_HOOK_DETECTIONS"] = saved_detections
    # 平常跑（不带标记、不是 --selftest）设了弄坏开关：退 2、点名它只在自证里认
    outside = subprocess.run(["bash", script], input=json.dumps({"tool_name": "AskUserQuestion", "tool_input": cases[1][1]}), capture_output=True, text=True,
                             env=dict({key: value for key, value in os.environ.items() if key != SELFTEST_MARK_VARIABLE},
                                      **{BREAK_VARIABLE: "input-unrecorded", "AGENT_HOOK_DETECTIONS": detections}))
    results.append(("平常跑设了弄坏开关退 2、点名它只在自证里认", (2, True),
                    (outside.returncode, any(BREAK_VARIABLE in line and "自证" in line for line in outside.stderr.splitlines()))))
    for name in os.listdir(scratch):
        os.remove(os.path.join(scratch, name))
    os.rmdir(scratch)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 is_placement() / question_problems() / quotes_the_section() / section_text() 的判法与 stdin 入口，改完再跑 --selftest")
        return 1
    print(f"  ✓ 自检通过（查了 {len(results)} 种）：放置类问题（一段里问处词紧跟在放置词或单字后面、放置词后面同一小句里有落点词、开包词后面隔得近有「包」；"
          "「」原文不算）问句不点名规则那一节、选项说明不以「标准：」「偏离：」起头、「偏离：」不引那一节的原话或引的不在那一节里、一个「标准：」选项都没有的拒，"
          "不是放置类的（问处词前面不是放置词、放置词后面同一小句里没有落点词、开包词离「包」远、「包括」里的包、「」里的轮名、问名字的）与写齐了的放行；"
          "规则文件里有那一节、那一条列的各组词与钩子词表相同、写的原话字数下限与开包隔字上限与钩子常量相同，少一个词、数写错判得出；四例走真实的 stdin 入口；"
          "输入不是 JSON 对象、questions 不是列表、判的时候抛异常、规则那一节读不出照常放行并记一条检出；平常跑设了弄坏开关退 2")
    return 0

def unjudged_pass(hook_input, problem, switch, hook_dir):
    """判不了：照「放行但必记检出」stderr 写一行、检出记录追加一行（结局只记），交退出码 0。弄坏开关 switch 下只打 stderr、不记。"""
    print(f"! {HOOK_NAME}：{problem}，这一次没判、照常放行", file=sys.stderr)
    if broken(switch):
        return 0
    tool_input = hook_input.get("tool_input") if isinstance(hook_input, dict) else None
    header = tool_input.get("questions") if isinstance(tool_input, dict) else None
    try:
        sys.dont_write_bytecode = True
        if hook_dir not in sys.path:
            sys.path.insert(0, hook_dir)
        import lib_work_root
        _, said = lib_work_root.record_unjudged_pass(HOOK_NAME, hook_input, f"AskUserQuestion：{str(header)[:300]}", [f"{UNJUDGED_PREFIX}：{problem}"])
    except Exception as error:   # 文件不在、语法错都算读不到
        said = f"检出没记成：读不到同目录的 lib_work_root.py（{error!r}）"
    print(f"  {said}", file=sys.stderr)
    return 0

def judge_input(raw, hook_dir, rules_path=None):
    """hook 的原始输入（字节）→ 退出码。判不了的几种照「放行但必记检出」（文件头「判不了的」一段）。rules_path 不给取钩子旁边的 ../rules/directory-layout.md。"""
    try:
        hook_input = json.loads(raw.decode("utf-8"))
    except ValueError as error:
        return unjudged_pass(None, f"输入不是合法的 JSON（{error}；读到 {len(raw)} 字节，前 {INPUT_PREFIX_BYTES} 字节：{raw[:INPUT_PREFIX_BYTES].decode('utf-8', 'replace')!r}）",
                             "input-unrecorded", hook_dir)
    if not isinstance(hook_input, dict):
        return unjudged_pass(None, f"输入顶层是 {type(hook_input).__name__}，不是 JSON 对象", "input-unrecorded", hook_dir)
    tool_input = hook_input.get("tool_input")
    if not isinstance(tool_input, dict) or not isinstance(tool_input.get("questions"), list):
        shape = type(tool_input.get("questions")).__name__ if isinstance(tool_input, dict) else f"tool_input 是 {type(tool_input).__name__}"
        return unjudged_pass(hook_input, f"tool_input.questions 不是列表（{shape}），一问都没判", "questions-unrecorded", hook_dir)
    rules_path = rules_path or os.path.join(hook_dir, "..", "rules", RULE_FILE_NAME)
    try:
        section = None
        if placement_questions(tool_input):
            section = section_text(rules_path)
            if section is None:
                return unjudged_pass(hook_input, f"有放置类的问，而 {rules_path} 里读不出「{SECTION_HEADING}」那一节", "section-unrecorded", hook_dir)
        code, message = decide(hook_input, section or "")
    except Exception as error:
        if broken("exception-unrecorded"):   # 弄坏开关：不接住，照原样抛出
            raise
        return unjudged_pass(hook_input, f"判的时候抛异常（{type(error).__name__}: {error}）", "exception-unrecorded", hook_dir)
    if code:
        print(message, file=sys.stderr)
    return code

def refuse_break_outside_selftest():
    """弄坏开关只在自证里认：平常跑（不是 --selftest，也不是自证起的子进程）设了 ASK_USER_PLACEMENT_GUARD_BREAK 交 2，点名它只在自证里认；别的交 0。"""
    if not os.environ.get(BREAK_VARIABLE) or sys.argv[2:] == ["--selftest"]:
        return 0
    mark = os.environ.get(SELFTEST_MARK_VARIABLE, "")
    try:
        if mark and open(mark, encoding="utf-8").read().strip() == SELFTEST_MARK_CONTENT:
            return 0
    except OSError:
        pass
    print(f"✗ {HOOK_NAME}：{BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 是弄坏开关，只在自证（{HOOK_NAME} --selftest 与它起的子进程）里认；平常跑设了就拒，这一次没判", file=sys.stderr)
    print(f"→ 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 {BREAK_VARIABLE}=<项> bash .claude/hooks/{HOOK_NAME} --selftest（开关写在文件头）", file=sys.stderr)
    return 2

def main():
    hook_dir = sys.argv[1]
    refused = refuse_break_outside_selftest()
    if refused:
        return refused
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    return judge_input(sys.stdin.buffer.read(), hook_dir)

sys.exit(main())
PY
