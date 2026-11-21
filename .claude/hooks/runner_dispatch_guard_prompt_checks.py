# 派发闸 runner-dispatch-guard.sh 的 python 部分之一：共用库的读入、提示里标签行与转述句与代码块的认法、定义里要的输入、写范围、主会话记录里的派发与在跑的子 agent（撞文件登记、并发上限）、写回员规格、实验设计员准入、开工快照冲突、报告文件名与禁读清单与三方腿报告。
# 入口 runner-dispatch-guard.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 RUNNER_DISPATCH_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
from __main__ import LINE_START

import glob, hashlib, importlib.util, json, os, re, shutil, subprocess, sys, tempfile, time
from datetime import datetime, timedelta, timezone

# 这个钩子自己所在的 .claude/hooks：写者锁的库按它找，不按派发提示里的仓根找。部分文件与入口同目录（引导把 argv[1] 放在 sys.path 最前再 import 它们），
# 同目录的 lib_*.py 按名字 import（入口已把它放进 sys.path）或按文件路径载入，路径都从这一个取；算键库认得 __file__、不认部分文件里的 sys.argv
SCRIPT_HOOK_DIR = os.path.dirname(os.path.abspath(__file__))

try:   # 工作根与会话临时目录根的唯一定义：同目录的 lib_work_root.py（读不到时提示里那两处下的路径都认不出、不判，stderr 写一句）
    sys.dont_write_bytecode = True
    import lib_work_root as work_roots
    SCRATCH_MENTION_PREFIX, work_roots_error = work_roots.mention_prefix_pattern(), None
except Exception as error:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, SCRATCH_MENTION_PREFIX, work_roots_error = None, "(?!)", error
try:   # 写范围表的读法与判法（与 write-guard.sh、bash-command-detector.sh 共用一份）：同目录的 lib_write_scope.py（读不到时 ④ 不判，stderr 写一句）
    import lib_write_scope as write_scope
    write_scope_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    write_scope, write_scope_error = None, error
try:   # 草稿目录写者锁的判法与锁的格式：同目录的 lib_writer_lock.py（读不到时⑯不判，stderr 写一句）
    import lib_writer_lock as writer_lock
    writer_lock_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    writer_lock, writer_lock_error = None, error
try:   # 叫停轮登记表一行怎么判（与门禁 doc-process-records 的 abandoned-rounds 格共用）：同目录的 lib_abandoned_rounds.py（读不到时 ⑥ 不收任何叫停的轮，stderr 写一句）
    import lib_abandoned_rounds as abandoned_library
    abandoned_library_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    abandoned_library, abandoned_library_error = None, error
try:   # 三方轮开工快照的冲突判法（与 scripts/gate/apply-writer-patch.py 共用）：同目录的 lib_snapshot_conflicts.py（读不到时 ⑥ 不判，stderr 写一句）
    import lib_snapshot_conflicts as snapshot_library
    snapshot_library_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    snapshot_library, snapshot_library_error = None, error
try:   # 三方腿类型表的读法（登记处是定义 frontmatter 的 leg: three-way 一行；与交回闸、骨架、账表共用）：同目录的 lib_three_way_legs.py（读不到时 ⑰ 三方腿形态不判，stderr 写一句）
    import lib_three_way_legs as three_way_legs
    three_way_legs_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    three_way_legs, three_way_legs_error = None, error
try:   # 「本提示的草稿目录」那一行的认法（与完工清理闸、交回闸共用一份，只认行首「草稿目录：」）：同目录的 lib_prompt_scratch_line.py（读不到时 ⑯ 不建锁、⑰ 只判裸文件名，stderr 写一句）
    import lib_prompt_scratch_line as prompt_scratch_line
    prompt_scratch_line_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    prompt_scratch_line, prompt_scratch_line_error = None, error
try:   # 引号区间含不含某个位置（inside_spans）的判法（与 bash-command-detector.sh、project-code-guard.sh 共用一份）：同目录的 lib_shell_words.py，按文件路径导入（读不到时用到它的那几条抛异常，由 judge 外面的总闸记检出、放行；自检当场红）
    shell_words_specification = importlib.util.spec_from_file_location("lib_shell_words", os.path.join(SCRIPT_HOOK_DIR, "lib_shell_words.py"))
    shell_words = importlib.util.module_from_spec(shell_words_specification)
    shell_words_specification.loader.exec_module(shell_words)
    shell_words_error = None
except Exception as error:  # 文件不在、语法错、导入时抛的都算读不到
    shell_words, shell_words_error = None, error

NONE_WORDS = {"", "无", "没有", "无。", "-", "—", "空", "none"}

def labelled_value(prompt, label):
    match = re.search(label + r"[：:]\s*(.*)", prompt)
    return None if match is None else match.group(1).strip()


GENERAL_PURPOSE_TYPES = {"general-purpose", "claude"}
SKELETON_PLACEHOLDER = re.compile(r"<填[^>\n]*>")   # scripts/agents/dispatch-prompt.py 骨架里没填的格   # 内置通用类型：继承项目 CLAUDE.md 与全部规则，起步约 13 万 token
# 写回员（kb-scribe）给判决自己起草规格时，提示里点名的判决文件
VERDICT_MENTION = re.compile(r"research/prompts/[A-Za-z0-9][A-Za-z0-9_.\-]*-main-verification\.md")
# 提示正文里叫子 agent 跑项目代码的句子（①；判法在 project_code_run_sentences）：先去掉「」『』“”里的转述，按句（。；！？与换行）再按子句（逗号与冒号，反引号里的不算）切；
# 提到主 agent 的子句、同一句里前面提到主 agent 的子句不算（那是主 agent 跑的）。子 agent 一律不跑项目代码，也不往统一任务服务交任务，写成「要主 agent 跑的命令」交回
SENTENCE_QUOTED = re.compile(r"「[^「」\n]*」|『[^『』\n]*』|“[^“”\n]*”")
# 认领行（work-directory-ownership.sh claim <目录> --purpose <用途>）里 --purpose 后面的用途是给人看的一句话（记这个目录归谁、做什么用），不是叫子 agent 去做什么：
# ①、㉒、⑲ 三处逐行判句子之前先把它摘掉（加了引号的摘引号里的，没加引号的摘到行尾）。弄坏开关 claim-purpose-judged 下不摘
CLAIM_PURPOSE = re.compile(r"(work-directory-ownership\.sh\s+claim\b[^\n]*?--purpose(?:\s+|=))(?:'[^'\n]*'|\"[^\"\n]*\"|[^\n]*)")

def without_claim_purpose(line):
    """一行里认领行的 --purpose 用途文字换成空引号；不是认领行的原样交回。"""
    return line if break_switch_is("claim-purpose-judged") else CLAIM_PURPOSE.sub(r"\1''", line)

SENTENCE_SPLIT = re.compile(r"[。；;！？!?\n]")
CLAUSE_SPLIT = re.compile(r"[，,：:]")
MAIN_AGENT_MENTION = re.compile(r"主\s*agent")
NEGATION_BEFORE = re.compile(r"(?:不|没|别|勿|禁止|不许|不准|不得|严禁|无须|无需|不用|不要|切勿|莫|非)[\s*]*$")
# ① ㉒ ⑲ 只判叫子 agent 去跑、去交、去起的祈使与许可；下面三种叙述不是叫它做什么，句子判之前先放行（narrated_sentence、text_rewrite_before）：
# 「现象：」「症状：」起头的句子（讲已经发生的现象）、讲已经出了结局的任务（带任务号与「退 <码>」或 exit=<码>），这两种句里有对子 agent 说的请 / 你 / 自己 / 可以 / 允许 / 务必的不算、
# 那一处前面紧挨着在说文字改成什么（改成 / 改为 / 改写成 / 改记 / 改用 / 写成 / 换成……，到那一处不超过 TEXT_REWRITE_TAIL_LIMIT 个字、中间没有然后 / 之后 / 再 / 并 / 接着）。
# 弄坏开关 phenomenon-sentences-judged、finished-task-sentences-judged、text-rewrite-judged 各让一种不放行
NARRATION_LABEL = re.compile(r"^[ \t>*_\-]*(?:\d+[.)、][ \t]*)?[*_]*(?:现象|症状)[*_]*[ \t]*[：:]")
FINISHED_TASK_ID = re.compile(r"任务(?:号)?[ \t#：:]*\d{3,}")
FINISHED_TASK_OUTCOME = re.compile(r"退[ \t]*\d+|exit=\d+")
ADDRESSED_TO_AGENT = re.compile(r"请|你|自己|可以|允许|务必")
TEXT_REWRITE = re.compile(r"改成|改为|改写成|改写为|改记|改作|改用|写成|换成|替换成")
TEXT_REWRITE_BREAK = re.compile(r"然后|之后|再|并|接着")
TEXT_REWRITE_TAIL_LIMIT = 24
# 跑 / 执行 / 起 / 交：后面接过、完、出、的、了……或回（交回）、草、步、头的不是叫它去跑
RUN_VERB = re.compile(r"(?:复跑|重跑|跑|执行|运行|起|交)(?![过完出的了掉通绿红前员者回草步头点始]|不[了动起完过成])")
# 要跑的对象（本仓的代码）：会编的 cargo、--selftest、本仓里的脚本路径或入口脚本名、门禁阶段与格、自证、样本档、弄坏开关；
# 文本工具（直接跑的：scripts/kb/ 下的、copy-originals.py、cite-check.py、checklist-specs.py）不算
RUN_OBJECT = re.compile(r"cargo\s+(?:\+\S+\s+)?(?:test|t|run|r|bench|build|b|check|c|clippy|doc|nextest|miri|llvm-cov|hack|mutants)\b"
                        r"|--selftest(?![\w-])"
                        r"|(?:[\w.~/-]*/)?(?:scripts|task-server|prune-server)/(?!kb/|agents/(?:copy-originals|cite-check|checklist-specs)\.py)[\w./-]+\.(?:sh|py)"
                        r"|(?:[\w.~/-]*/)?\.claude/(?:gate\.d|hooks|scripts)/[\w./-]+\.(?:sh|py)"
                        r"|(?<![\w.-])(?:gate|gate-staged|check|lkmm|mutate|replay|crash-point-replay)\.sh"
                        r"|门禁阶段|门禁的?(?:阶段|格)|自证|样本档|弄坏开关")
RUN_OBJECT_DISTANCE = 40   # 动词后面这么多字以内找对象
# 重型入口（agent-common.md「重型测试就是 验证包那一批」点名的入口脚本）的名字：这些入口整条命令比距离长（`bash scripts/crash/crash-point-replay.sh --item x`），
# 名字起头在距离里就算要跑的对象，不要求整个落进去；别的对象仍要整个落在 RUN_OBJECT_DISTANCE 个字以内（放宽到所有对象，叙述里点名脚本的句子会大面积误拒）。
# 名字后面接 : 的是文件:行号的引用，不算。弄坏开关 heavy-entry-start-unjudged 下不用这一条
RUN_OBJECT_HEAVY = re.compile(r"(?<![\w.-])(?:crash-point-replay|crash-amplification(?:-item|-two-hosts)?|build-patched-qemu|qemu-device-streams|lkmm|crates-mutation-replay"
                              r"|formal-verification|broad-mutation|research-build-and-replay|model-differential-and-scenarios)\.sh(?![\w:-])"
                              r"|(?<![\w.-])(?:broad-mutation|verus-proof-check|e165-pin-red-examples)\.py(?![\w:-])")
# ㉒ 叫子 agent 自己交任务的说法：提交客户端的 submit / wait / cancel / clean，经 / 往 / 交给统一任务服务或 task-server，run_in_background（task_submission_sentences）
TASK_SUBMISSION_MENTION = re.compile(r"task-server/task\s+(?:submit|wait|cancel|clean)(?![\w-])"
                                     r"|(?:经|走|往|交给|交到|提交给|提交到|交)\s*(?:统一)?任务服务"
                                     r"|(?:经|走|往|交给|交到|提交给|提交到)\s*`?task-server(?!/task\s+(?:status|log))"
                                     r"|run_in_background")
TASK_SUBMISSION_NEGATION_BEFORE = re.compile(r"(?:不|没|别|勿|禁止|严禁|一律不)[^，,]{0,8}$")   # 那一处前面同一子句里八个字以内有否定
TASK_SUBMISSION_NEGATION_AFTER = re.compile(r"^[^，,]*?(?:不起|不交|不跑|不用|不许|不准|禁止|一律不|不要|不能|不得|严禁|都不|也不)")
# ㉒ 交回的说法：同一句里有它的是叫子 agent 把命令写进报告交回，不判；交回节的名字带引号写的先摘掉引号再认（别的引语照旧整段不判）
TASK_SUBMISSION_HAND_BACK = re.compile(r"写进(?:交回)?报告|写进交回|要主 agent 跑的命令")
TASK_SUBMISSION_HAND_BACK_SECTION_QUOTED = re.compile(r"[「『“]要主 agent 跑的命令[」』”]")
COMMON_CONSTRAINTS_READ = re.compile(r"开工先读[^\n]*\.claude/agent-common\.md")
SUBAGENT_CONCURRENCY_LIMIT = 8              # 一个主 agent 最多同时开 8 个 subagent
RUNNING_RECENT_SECONDS = 3 * 3600
UNLAUNCHED_GRACE_SECONDS = 600
DISPATCH_TIME_SLACK_SECONDS = 2   # 主会话记录里派发的 tool_use 与派发闸放行、建锁、登记之间的时钟误差（与 lib_writer_lock.py 的 LAUNCH_SLACK_SECONDS 同值）
DISPATCH_CALL_NEEDLES = ('"name": ?"(Agent|Task)"', '"is_error": ?true')   # 主会话记录里派发的 tool_use 与错误结果两种行（grep -E；记录是紧凑 JSON，冒号后的空格可有可无）
CRATES_FILES_LINE = re.compile(LINE_START + r"要动的\s*crates\s*文件[* \t]*[：:](.*)$", re.M)
CRATES_PATH = re.compile(r"crates/[^\s`'\"，。；：、（）()「」,;]+")
WRITER_GUARDED_AGENTS = frozenset({"implementation-writer", "tooling-writer"})   # ⑤ 撞文件登记、⑱ 不碰一行判的两种
TOOLING_FILES_LABEL = re.compile(r"要改的文件")   # ⑤ 工具员「要改的文件：」那一行的标签（整个标签就是这几个字）
TOOLING_FILES_HEADING = re.compile(r"^[ \t]{0,3}#{1,6}[ \t]*要改的文件[^\n]*$", re.M)   # ⑤ 工具员「## 要改的文件」那一节的标题
REPOSITORY_PATH_TOKEN = re.compile(r"(?<![\w./~$\-])\.?[A-Za-z0-9_][\w.\-]*/[^\s`'\"，。；：、（）()「」『』“”,;*<>]*")   # 从仓根起、带 / 的词
SCOPE_PATH_START = r"(?:" + SCRATCH_MENTION_PREFIX + r"|research/|\.claude/|crates/|litmus/|records/)"   # ④ 判的路径从这几处起头
SCOPE_PATH_HEAD = re.compile(SCOPE_PATH_START)
# ④ 自己读的「写进 / 写到 / 更新」；报告行（「报告」「报告路径」「报告写进」「报告写到」）由 report_line_mentions 解析，与 ⑰ 共用一份
SCOPE_MENTION = re.compile(r"(?:写进|写到|更新)[*\s]*[：:]?[*\s]*`?(?P<path>" + SCOPE_PATH_START + r"[^\s`'\"，。；：、（）()「」,;]*)")
# ④ 自己读后三种报告标签的写法：只有弄坏开关 report-line-labels-split 下才用
SCOPE_MENTION_BEFORE_MERGE = re.compile(r"(?:报告路径|报告写进|报告写到|写进|写到|更新)[*\s]*[：:]?[*\s]*`?(?P<path>" + SCOPE_PATH_START + r"[^\s`'\"，。；：、（）()「」,;]*)")
SCOPE_NEGATION = re.compile(r"不|别|禁止|勿")
# ④ 转述别人要写什么的认法（scope_mention_aimed_elsewhere；写法见文件头 ④）：动词所在子句的边界、别的会话 / agent 的说法、列表项起头、说已经做完什么的列表标题
SCOPE_CLAUSE_BREAK = re.compile(SENTENCE_SPLIT.pattern + "|" + CLAUSE_SPLIT.pattern)
SCOPE_MAIN_AGENT_ACTOR = re.compile(r"(?<![按照依据向对从])主\s*agent")   # 「按主 agent 给的格式写进 …」里的主 agent 是给格式的，不是写的人
SCOPE_OTHER_ACTOR_PHRASE = re.compile(r"(?:别的|另一个|另外的|下一个)\s*(?:子\s*)?(?:会话|agent)")
SCOPE_LIST_ITEM = re.compile(r"^[ \t]*(?:[-*+>]|\d+[.)、])[ \t]+")
SCOPE_NARRATED_HEADING = re.compile(r"(?:做成|做完|已做|已改|已完成|改动范围|改了什么|做了什么)[^\n：:]{0,8}[：:][*_ \t]*$")
MUTATION_TABLE_MENTION = re.compile(r"scripts/mutation/tables/[^\s`'\"]+\.tsv|crates/mutations\.tsv")
RERUN_REGISTRATION = re.compile(r"research/prompts/e(\d+)-r\d+-prereg\.md")
ABSENT_TASK = re.compile(r"Error: No task found with ID: (a[0-9a-f]{6,})\. Running background agents: (.*)", re.DOTALL)
HANDBACK_FROM = re.compile(r'agent-message from=\\?"(a[0-9a-f]{6,})\\?">(?:\\n|\s)*\[Subagent hand-back\]')
NOTIFICATION_BLOCK = re.compile(r"<task-notification>(.*?)</task-notification>", re.S)


def valve_reason(prompt, label):
    """行首「<label>：<理由>」的理由；没有、空、占位都返回 None。"""
    match = re.search(LINE_START + re.escape(label) + r"[* \t]*[：:][* \t]*(.*?)[ \t]*$", prompt, re.M)
    if match is None or break_switch_is("valve-reason-ignored"):
        return None
    reason = match.group(1).strip()
    return None if not reason or reason.startswith("<") or reason in NONE_WORDS else reason


def agent_definition_fields(project_root, subagent_type):
    """定义的 frontmatter → {键: 值}（值去掉行尾 # 注释）；定义不在返回 None。"""
    path = os.path.join(project_root, ".claude", "agents", f"{subagent_type}.md")
    try:
        text = open(path, encoding="utf-8").read()
    except OSError:
        return None
    fields = {}
    if text.startswith("---"):
        for line in text.split("\n---", 1)[0].splitlines()[1:]:
            if ":" in line:
                key, value = line.split(":", 1)
                fields[key.strip()] = value.split("#", 1)[0].strip()
    return fields


def missing_required_inputs(fields, prompt):
    if break_switch_is("inputs-ignored"):
        return []
    groups = [group.strip() for group in (fields.get("required-inputs") or "").split(",") if group.strip()]
    return [group for group in groups if not any(alternative.strip() and alternative.strip() in prompt for alternative in group.split("|"))]


def expanded_mention(text):
    """文字里写的 ~/…、$HOME/…、$SINGLEFS_WORK_ROOT/… 换成绝对路径（lib_work_root.py 的 expand_written_path）；读不到那份库原样交回。"""
    return work_roots.expand_written_path(text) if work_roots else text


def scope_patterns(project_root, subagent_type):
    """类型在写范围表（派发那一刻的仓根下 .claude/hooks/agent-write-scope.tsv）里的模式，$SINGLEFS_WORK_ROOT/ 起头的展开成工作根那一条。
    读表与展开都调同目录 lib_write_scope.py（write-guard.sh、bash-command-detector.sh 用的是同一份）；读不到那份库、读不了表时 ④ 不判，stderr 写一句。"""
    table_path = os.path.join(project_root, ".claude", "hooks", "agent-write-scope.tsv")
    if not os.path.isfile(table_path):
        return []
    if write_scope is None or break_switch_is("scope-library-unread"):
        note_unjudged("scope-library", f"! 派发闸读不到 {os.path.join(SCRIPT_HOOK_DIR, 'lib_write_scope.py')}（{write_scope_error!r}），写范围这一条没判")
        return []
    try:
        patterns = write_scope.load_scopes(table_path).get(subagent_type, [])
    except OSError as error:
        note_unjudged("scope-table", f"! 派发闸读不了写范围表，写范围这一条没判：{error}")
        return []
    if break_switch_is("scope-token-unexpanded"):
        return list(patterns)
    return write_scope.expanded_scope_patterns(patterns, work_roots)


def scope_other_role_words(project_root, subagent_type):
    """别的类型的称呼：各定义 frontmatter 的 description 冒号或句号之前那个名字（实验执行员、写回员），连同它以员 / 腿结尾、不少于三个字的各个尾巴（执行员），再加类型名，
    去掉这个类型自己的那几个。这个类型的定义读不到或没有 description 时交空集：不知道哪个称呼是它自己的，不认别的类型的称呼。
    弄坏开关 scope-own-role-as-other 下自己的称呼也留在里面。"""
    own, others, own_described = set(), set(), False
    agents_directory = os.path.join(project_root, ".claude", "agents")
    try:
        file_names = sorted(os.listdir(agents_directory))
    except OSError:
        return set()
    for file_name in file_names:
        if not file_name.endswith(".md"):
            continue
        try:
            with open(os.path.join(agents_directory, file_name), encoding="utf-8") as handle:
                frontmatter = handle.read().split("\n---", 1)[0]
        except (OSError, UnicodeDecodeError):
            continue
        type_name = file_name[:-3]
        found = re.search(r"^description:[ \t]*([^：。\n]+)", frontmatter, re.M)
        head = found.group(1).strip() if found else ""
        words = {type_name}
        if head:
            words.add(head)
            if head.endswith(("员", "腿")):
                words.update(head[index:] for index in range(len(head)) if len(head) - index >= 3)
        if type_name == subagent_type:
            own.update(words)
            own_described = bool(head)
        else:
            others.update(words)
    if not own_described:
        return set()
    return others | own if break_switch_is("scope-own-role-as-other") else others - own


def scope_mention_aimed_elsewhere(prompt, start, other_roles):
    """「写进 / 写到 / 更新」起头在 start 的这一处是不是在转述别人要写什么、不是要这个子 agent 写（认法在文件头 ④）：
    动词所在的子句（动词前面到最近一个逗号、冒号、句号或换行）里点名了主 agent（前面紧挨着按 / 照 / 依 / 据 / 向 / 对 / 从的不算）、别的会话 / agent、other_roles 里的称呼，而最后一个点名之后没有对子 agent 说的词（ADDRESSED_TO_AGENT）；
    或动词所在的行是列表项、列表上面最近一行是说已经做完什么的标题（SCOPE_NARRATED_HEADING）。
    弄坏开关 scope-other-actor-judged 下不认第一种，scope-addressed-clause-as-other 下别的执行者后面又对子 agent 说了话的也算转述，scope-narrated-list-judged 下不认第二种。"""
    line_start = prompt.rfind("\n", 0, start) + 1
    before_in_line = prompt[line_start:start]
    clause = SCOPE_CLAUSE_BREAK.split(before_in_line)[-1]
    actor_ends = [found.end() for pattern in (SCOPE_MAIN_AGENT_ACTOR, SCOPE_OTHER_ACTOR_PHRASE) for found in pattern.finditer(clause)]
    actor_ends += [position + len(role) for role in other_roles for position in [clause.rfind(role)] if position >= 0]
    if actor_ends and not break_switch_is("scope-other-actor-judged"):
        addressed_ends = [found.end() for found in ADDRESSED_TO_AGENT.finditer(clause)]
        if not addressed_ends or max(addressed_ends) < max(actor_ends) or break_switch_is("scope-addressed-clause-as-other"):
            return True
    if break_switch_is("scope-narrated-list-judged") or SCOPE_LIST_ITEM.match(before_in_line) is None:
        return False
    for earlier in reversed(prompt[:line_start].split("\n")):
        if earlier.strip() and SCOPE_LIST_ITEM.match(earlier) is None:
            return SCOPE_NARRATED_HEADING.search(earlier) is not None
    return False


def out_of_scope_mentions(project_root, subagent_type, prompt):
    patterns = scope_patterns(project_root, subagent_type)
    if not patterns or break_switch_is("scope-ignored"):
        return []
    labels_split = break_switch_is("report-line-labels-split")
    other_roles = scope_other_role_words(project_root, subagent_type)
    candidates = [(match.group("path"), prompt[max(0, match.start() - 6):match.start()], scope_mention_aimed_elsewhere(prompt, match.start(), other_roles))
                  for match in (SCOPE_MENTION_BEFORE_MERGE if labels_split else SCOPE_MENTION).finditer(prompt)]
    if not labels_split:   # 报告行点名的路径（与 ⑰ 同一份解析）；还有 <填 …> 占位的交 ⑬
        candidates += [(mention["named"], mention["before"], False) for mention in report_line_mentions(prompt)
                       if mention["named"] and not mention["placeholder"] and SCOPE_PATH_HEAD.match(mention["named"])]
    outside = []
    for written, before, aimed_elsewhere in candidates:
        if aimed_elsewhere:
            continue
        if SCOPE_NEGATION.search(before) and not break_switch_is("scope-negation-ignored"):
            continue
        path = expanded_mention(written.rstrip("/."))
        root_prefix = os.path.normpath(project_root) + "/"
        relative = path[len(root_prefix):] if path.startswith(root_prefix) else path
        if not any(write_scope.glob_to_regex(pattern).match(relative) for pattern in patterns) and relative not in outside:
            outside.append(relative)
    return outside


def parse_moment(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00")).timestamp()


def session_dispatch_events(transcript_path):
    """主会话记录 → (launches{agent 号: (派发时刻, 模型, 提示 sha256)}, ended{agent 号: 最晚结束时刻})。
    先用 grep 挑出带这几种记号的行，免得逐行解析上百兆的记录；读不了返回两个空的。"""
    launches, ended = {}, {}
    if not transcript_path or not os.path.isfile(transcript_path):
        return launches, ended
    needles = ['"async_launched"', "[Subagent hand-back]", "<task-notification>", "Successfully stopped task", "No task found with ID: "]
    try:
        grep = subprocess.run(["grep", "-F", *sum((["-e", needle] for needle in needles), []), transcript_path],
                              capture_output=True, text=True, timeout=50)
    except (OSError, subprocess.TimeoutExpired) as error:
        note_unjudged("transcript", f"! 派发闸读不了主会话记录 {transcript_path}，按会话判的几条没判：{error}")
        return launches, ended
    for line in grep.stdout.splitlines():
        try:
            record = json.loads(line)
            moment = parse_moment(record.get("timestamp") or "")
        except (ValueError, TypeError):
            continue
        result = record.get("toolUseResult")
        if isinstance(result, dict) and result.get("status") == "async_launched" and result.get("agentId"):
            digest = hashlib.sha256(str(result.get("prompt") or "").encode("utf-8")).hexdigest()
            launches[result["agentId"]] = (moment, result.get("resolvedModel") or "", digest)
        if isinstance(result, dict) and "Successfully stopped task" in str(result.get("message", "")) and result.get("task_id"):
            ended[result["task_id"]] = max(moment, ended.get(result["task_id"], 0))
        absent = ABSENT_TASK.match(result) if isinstance(result, str) else None
        if absent and absent.group(1) not in absent.group(2) and not break_switch_is("absent-task-counted"):
            ended[absent.group(1)] = max(moment, ended.get(absent.group(1), 0))
        for agent_id in HANDBACK_FROM.findall(line):
            ended[agent_id] = max(moment, ended.get(agent_id, 0))
        for block in NOTIFICATION_BLOCK.findall(line.replace("\\n", "\n").replace('\\"', '"')):
            status = re.search(r"<status>(\w+)</status>", block)
            for agent_id in re.findall(r"<task-id>(\w+)</task-id>", block):
                if status and status.group(1) in ("failed", "killed", "stopped"):
                    ended[agent_id] = max(moment, ended.get(agent_id, 0))
    return launches, ended


def is_running(agent_id, launches, ended, transcript_path):
    launched_at = launches[agent_id][0]
    if ended.get(agent_id, 0) >= launched_at:
        return False
    own = os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl") if transcript_path.endswith(".jsonl") else ""
    last_seen = max(launched_at, os.path.getmtime(own) if own and os.path.isfile(own) else 0)
    return time.time() - last_seen <= RUNNING_RECENT_SECONDS


def session_dispatch_calls(transcript_path):
    """主会话记录 → [(派发时刻, 提示 sha256, 被拒了吗)]，按时刻先后。一次派发是助手消息里一个 name 为 Agent / Task 的 tool_use；被拒是同 id 的 tool_result 为错误结果
    （is_error 为真：被挂在同一事件上的别的钩子或用户拒掉，没起来）。后台派发起来了的结果是 async_launched、前台派发交回的是正文，都不是错误；还没有结果的算没被拒。
    先用 grep 挑出这两种行；读不了返回空表。"""
    if not transcript_path or not os.path.isfile(transcript_path):
        return []
    try:
        grep = subprocess.run(["grep", "-E", *sum((["-e", needle] for needle in DISPATCH_CALL_NEEDLES), []), transcript_path],
                              capture_output=True, text=True, timeout=50)
    except (OSError, subprocess.TimeoutExpired, ValueError) as error:
        note_unjudged("transcript", f"! 派发闸读不了主会话记录 {transcript_path}，按会话判的几条没判：{error}")
        return []
    calls, failed = {}, set()
    for line in grep.stdout.splitlines():
        try:
            record = json.loads(line)
            moment = parse_moment(record.get("timestamp") or "")
            blocks = record["message"]["content"]
        except (ValueError, TypeError, KeyError, AttributeError):
            continue
        if not isinstance(blocks, list):
            continue
        for block in blocks:
            if not isinstance(block, dict):
                continue
            if record.get("type") == "assistant" and block.get("type") == "tool_use" and block.get("name") in ("Agent", "Task") and isinstance(block.get("input"), dict):
                calls[block.get("id")] = (moment, hashlib.sha256(str(block["input"].get("prompt") or "").encode("utf-8")).hexdigest())
            elif record.get("type") == "user" and block.get("type") == "tool_result" and block.get("is_error") is True:
                failed.add(block.get("tool_use_id"))
    return sorted((moment, digest, call_id in failed) for call_id, (moment, digest) in calls.items())


def dispatch_refusal_lookup(transcript_path):
    """→ refused(提示 sha256, 时刻)：主会话记录里这份提示在「时刻」之前（含 DISPATCH_TIME_SLACK_SECONDS 的误差）最后一次派发，是不是被拒掉了。⑯ 传锁的派发时刻，⑤ 传登记的时刻。
    派发闸放行之后才轮到挂在同一事件上的别的钩子（borrowed-convention-guard.sh 这类）：它拒了，这次派发没起来，派发闸留下的写者锁与撞文件登记没人用，
    提示改过的重派不该被它们挡 10 分钟。只放被拒掉的：回执已落进记录的（子 agent 在跑）、还没有结果的（同一条消息里并行派的上一件）、记录里找不到的都不放。
    记录到第一次要用时才读（登记里没有「派出去了没回执」的行、锁已有认领时不读）。
    弄坏开关：refused-dispatch-kept 一律不放；refused-dispatch-result-unchecked 记录里找得到这次派发就放，不看结果；
    refused-dispatch-any-earlier 这份提示在这一刻之前只要有过一次被拒就放，不只看最后一次。"""
    loaded = []

    def refused(digest, at):
        if break_switch_is("refused-dispatch-kept"):
            return False
        if not loaded:
            loaded.append(session_dispatch_calls(transcript_path))
        calls = [was_refused for moment, found, was_refused in loaded[0] if found == digest and moment <= at + DISPATCH_TIME_SLACK_SECONDS]
        if break_switch_is("refused-dispatch-any-earlier"):
            return any(calls)
        if break_switch_is("refused-dispatch-result-unchecked"):
            return bool(calls)
        return bool(calls) and calls[-1]
    return refused


def registry_path(hook_input):
    root = os.environ.get("DISPATCH_GUARD_STATE_ROOT") or (work_roots.state_directory("dispatch-guard-state") if work_roots
                                                           else os.path.join(tempfile.gettempdir(), "dispatch-guard-state"))
    session = re.sub(r"[^\w.-]", "_", str(hook_input.get("session_id") or "unknown-session"))
    return os.path.join(root, session, "implementation-writers.tsv")


def live_registry_rows(hook_input, launches, ended):
    """登记里还在跑的那几行（顺手把结束了的、派发之后没出现的删掉）。→ [(时刻, 提示 sha256, [文件], 类型)]。
    一行四列「时刻、提示 sha256、文件（空格分隔）、类型」。"""
    path = registry_path(hook_input)
    try:
        rows = [line.rstrip("\n").split("\t") for line in open(path, encoding="utf-8") if line.strip()]
    except FileNotFoundError:
        return []
    except OSError as error:   # 在而读不了：在跑的实现员、工具员认不出，⑤ 撞文件这一回判不全
        note_unjudged("registry-read", f"! 派发闸读不了实现员与工具员的登记 {path}：{error}；⑤ 撞文件这一次没判全")
        return []
    by_digest = {digest: agent_id for agent_id, (_, _, digest) in launches.items()}
    transcript_path = hook_input.get("transcript_path") or ""
    refused = dispatch_refusal_lookup(transcript_path)
    live = []
    for row in rows:
        if len(row) not in (3, 4):
            continue
        registered_at, digest, files = float(row[0]), row[1], row[2].split()
        kind = row[3] if len(row) == 4 and row[3] else "implementation-writer"
        agent_id = by_digest.get(digest)
        if not break_switch_is("registry-no-prune"):
            if agent_id is None and time.time() - registered_at > UNLAUNCHED_GRACE_SECONDS:
                continue
            if agent_id is None and refused(digest, registered_at):   # 那次派发被挂在同一事件上的别的钩子（或用户）拒掉，没起来：登记是本闸放行时留下的
                continue
            if agent_id is not None and not is_running(agent_id, launches, ended, transcript_path):
                continue
        live.append((registered_at, digest, files, kind))
    rewrite_registry(path, live)
    return live


def rewrite_registry(path, rows):
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        temporary = f"{path}.{os.getpid()}.tmp"
        with open(temporary, "w", encoding="utf-8") as handle:
            for registered_at, digest, files, kind in rows:
                handle.write(f"{registered_at}\t{digest}\t{' '.join(files)}\t{kind}\n")
        os.replace(temporary, path)
    except OSError as error:
        note_unjudged("registry-write", f"! 派发闸写不了实现员与工具员的登记 {path}：{error}；这一件没登记，之后的派发 ⑤ 撞文件认不出它")


def register_writer(hook_input, prompt, files, live_rows, kind):
    digest = hashlib.sha256(prompt.encode("utf-8")).hexdigest()
    rewrite_registry(registry_path(hook_input), live_rows + [(time.time(), digest, files, kind)])


def tooling_writer_files(prompt):
    """⑤ 工具员「要改的文件」：行首「要改的文件：」一行里的路径（值空着的，下面连着的列表项也算），与标题是「要改的文件」那一节里的路径（到下一个标题为止）；
    闭合了的围栏块里的不算。路径是从仓根起、带 / 的词（以 / 收尾的是目录，罩住它底下的文件）；~ 与绝对路径不算。→ 排好序的路径表。"""
    text = without_code_blocks(prompt)
    chunks = labelled_values_with_list(text, lambda found: TOOLING_FILES_LABEL.fullmatch(found.group("label").strip(" *")) is not None)
    for heading in TOOLING_FILES_HEADING.finditer(text):
        following = HEADING_LINE.search(text, heading.end())
        chunks.append(text[heading.end():following.start() if following else len(text)])
    files = set()
    for chunk in chunks:
        for token in REPOSITORY_PATH_TOKEN.findall(chunk):
            token = token.rstrip(".")
            if "://" not in token and not token.startswith(("~", "$")):
                files.add(token)
    return sorted(files)


def paths_overlap(one, other):
    """两条从仓根起的路径撞不撞：相同，或一条是以 / 收尾的目录、另一条落在它底下。"""
    return one == other or (one.endswith("/") and other.startswith(one)) or (other.endswith("/") and one.startswith(other))


def writer_collision_verdict(hook_input, prompt, subagent_type, launches, ended, registry=True):
    """⑤ 实现员与工具员撞文件 → (拒绝说明或 None, 要登记的文件, 还在跑的登记行)。registry 为假（--check-prompt）时只判那一行，不读不写登记。
    实现员要有「要动的 crates 文件：…」一行；工具员的文件从「要改的文件」取（tooling_writer_files，认不出路径的不拒，登记一份空的）。
    两种类型登记在同一份表里互比：实现员对实现员、工具员对工具员、实现员对工具员都算撞。弄坏开关 writer-collision-implementation-only 下：
    只判实现员、只比实现员，工具员不登记也不比。"""
    if subagent_type == "implementation-writer":
        line = CRATES_FILES_LINE.search(prompt)
        if line is None:
            return ("✗ 派 implementation-writer 的提示里没有「要动的 crates 文件：…」一行。\n"
                    "→ 怎么办：把这一件要动的 crates/ 文件逐个写进这一行（路径从仓根起）；说不全要动哪些文件，这件活还没切小，先切。"), [], []
        # crates/mutations.tsv 只追加、不算撞文件（主 agent 合补丁时按 mutations-append.tsv 并）；份数不设上限。
        files = sorted(set(CRATES_PATH.findall(line.group(1))) - {"crates/mutations.tsv"})
    else:
        files = sorted(set(tooling_writer_files(prompt)) - {"crates/mutations.tsv"})
    if not registry:
        return None, files, []
    live = live_registry_rows(hook_input, launches, ended)
    implementation_only = break_switch_is("writer-collision-implementation-only")
    if break_switch_is("registry-ignored") or (implementation_only and subagent_type != "implementation-writer"):
        return None, files, live
    overlapping = {}
    for _, _, registered, kind in live:
        if implementation_only and kind != "implementation-writer":
            continue
        for path in files:
            if any(paths_overlap(path, other) for other in registered):
                overlapping.setdefault(path, set()).add(kind)
    if overlapping:
        shown = "、".join(f"{path}（在跑的 {'、'.join(sorted(kinds))}）" for path, kinds in sorted(overlapping.items()))
        return (f"✗ 这一件要改的文件与还在跑的实现员或工具员撞了：{shown}。\n"
                "→ 规矩：同一个文件同一时刻只给一个子 agent 改（实现员与工具员登记在同一份表里互比，.claude/hooks/runner-dispatch-guard.sh 文件头 ⑤）。\n"
                "→ 怎么办：排在那个子 agent 交回之后再派，或把两件并给同一个子 agent（.claude/main-agent.md「一轮怎么开、怎么收」第 4 条）。"), files, live
    return None, files, live


def concurrency_verdict(hook_input, launches, ended):
    """⑦ 本会话在跑的子 agent（不分模型）满 SUBAGENT_CONCURRENCY_LIMIT 个就拒，交拒绝说明；没满交 None。"""
    transcript_path = hook_input.get("transcript_path") or ""
    if break_switch_is("concurrency-no-limit"):
        return None
    running = [agent_id for agent_id in launches if is_running(agent_id, launches, ended, transcript_path)]
    if len(running) >= SUBAGENT_CONCURRENCY_LIMIT:
        return (f"✗ 本会话已有 {len(running)} 个子 agent 在跑（{'、'.join(running[:10])}），一个主 agent 同时最多 {SUBAGENT_CONCURRENCY_LIMIT} 个（用户 2026-10-02 定）。\n"
                "→ 怎么办：等其中一个交回再派；派不下的排进下一波（.claude/main-agent.md「一轮怎么开、怎么收」第 1 条）。")
    return None


def kb_scribe_spec_verdict(prompt, project_root, hook_dir):
    if valve_reason(prompt, "规格检查已判") is not None or break_switch_is("spec-check-skipped"):
        return None
    checker = os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "scripts", "kb", "kb-spec-check.py")
    candidates = []
    for mention in sorted(set(SCRATCH_FILE_MENTION.findall(prompt))):
        path = expanded_mention(mention.rstrip("。，；：、."))
        if os.path.isfile(path) and path.endswith((".json", ".md", ".txt")):
            candidates.append(path)
    inline = None
    if not candidates and VERDICT_MENTION.search(prompt) and not break_switch_is("verdict-not-spec"):
        return None   # 写回员自己起草：给的是判决，规格由它写、它跑 kb-spec-check.py
    if not candidates:
        inline = tempfile.NamedTemporaryFile("w", suffix=".md", delete=False, encoding="utf-8")
        inline.write(prompt)
        inline.close()
        candidates = [inline.name]
    try:
        recognized, problems = 0, []
        for path in candidates:
            try:
                result = subprocess.run([sys.executable, checker, path, "--root", project_root], capture_output=True, text=True, timeout=50)
            except (OSError, subprocess.TimeoutExpired) as error:
                note_unjudged("spec-check", f"! 派发闸跑不了 kb-spec-check.py，规格这一条没判：{error}")
                return None
            if result.returncode in (0, 1):
                recognized += 1
            if result.returncode == 1:
                problems += [line.strip() for line in result.stdout.splitlines() if line.strip().startswith("✗")]
            elif result.returncode not in (0, 2):
                note_unjudged("spec-check", f"! kb-spec-check.py 退出码 {result.returncode}，规格这一条没判：{result.stdout[-200:]}{result.stderr[-200:]}")
                return None
        if problems:
            return ("✗ 给书记员的规格会让门禁红（scripts/kb/kb-spec-check.py）：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                    "\n→ 怎么办：改规格原文再派；判断不了的交用户；要写回员自己起草就给它判决与条目、不给规格。确要照这份派，提示里写一行「规格检查已判：<理由>」。")
        if recognized == 0:
            return ("✗ 派 kb-scribe 的提示里认不出一份规格（提示正文与点名的工作根或会话临时目录根下的文件都不是 kb-spec-check.py 认的写法）。\n"
                    "→ 怎么办：照 scripts/kb/kb-spec-check.py 文件头「规格的两种写法」写，或给判决让写回员自己起草；确要照原样派，提示里写一行「规格检查已判：<理由>」。")
        return None
    finally:
        if inline is not None:
            os.unlink(inline.name)


ADMISSION_PRECONDITION_LINES_SHOWN = 6   # ⑨ 退 3 的拒绝说明里抄 admission.py 报的前提至多几行
# ⑨ admission.py experiment 退 2 时自己打的两种句子（scripts/gate/admission.py 的 admit_experiment）；退 2 而 stderr 里没有这两句的，是判法本身没跑起来，同样拒，出路另写
ADMISSION_REGISTRATION_SENTENCE = re.compile(r"^✗ (准入登记里没有实验键 |\S+ 的准入登记有错：)")
# ⑨ admission.py 实验键拒算时退 0、在 stdout 打的产物头（sha256=refused）：它自己说判不了「没变」，照放行
ADMISSION_KEY_REFUSED_HEADER = re.compile(r"^E7INPUT .*\bsha256=refused\b", re.MULTILINE)
# ⑨ 删了的强制重跑变量（scripts/gate/admission.py 的 RETIRED_RERUN_REASON_VARIABLE）：环境里设着它 admission.py 照用法错退 2、打这一句；派发闸不清它，照退 2 拒，出路另写
ADMISSION_RETIRED_VARIABLE = "SINGLEFS_EXPERIMENT_RERUN_REASON"
ADMISSION_RETIRED_VARIABLE_SENTENCE = re.compile(rf"^✗ {ADMISSION_RETIRED_VARIABLE} 删了")

def admission_stderr_tail(result):
    return [line.strip() for line in result.stderr.splitlines() if line.strip()][-2:]

def designer_admission_verdict(prompt, project_root, hook_dir):
    match = RERUN_REGISTRATION.search(prompt)
    if match is None or break_switch_is("admission-ignored") or valve_reason(prompt, "准入已判") is not None:
        return None
    admission = (os.environ.get("RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT")
                 or os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "scripts", "gate", "admission.py"))
    environment = {name: value for name, value in os.environ.items()
                   if name != ADMISSION_RETIRED_VARIABLE or not break_switch_is("admission-rerun-reason-cleared")}
    try:
        result = subprocess.run([sys.executable, admission, "experiment", project_root, f"E{int(match.group(1))}"], capture_output=True, text=True, timeout=50, env=environment)
    except (OSError, subprocess.TimeoutExpired) as error:
        note_unjudged("admission", f"! 派发闸跑不了 admission.py，准入这一条没判：{error}")
        return None
    if result.returncode == 77:
        return (f"✗ E{int(match.group(1))} 的单元键与上次产物头记的相同（scripts/gate/admission.py experiment 退 77），不写它的重跑登记。\n"
                "→ 怎么办：沿用上次产物与结论；要为别的原因重跑（换臂、换判据），在提示里写一行「准入已判：<哪一样变了>」。")
    if result.returncode == 2 and not break_switch_is("admission-unregistered-allowed"):
        reasons = [line.strip() for line in result.stderr.splitlines() if line.strip().startswith("✗")][:2]
        # 环境里设着已删的强制重跑变量：admission.py 照用法错退 2（它自己打这一句），出路是从起会话的环境里去掉它，不是接准入，也不是让判法能跑
        if (any(ADMISSION_RETIRED_VARIABLE_SENTENCE.match(line.strip()) for line in result.stderr.splitlines())
                and not break_switch_is("admission-retired-variable-unrecognized")):
            return (f"✗ E{int(match.group(1))} 的重跑准入判不了：环境里设着已删的强制重跑变量 {ADMISSION_RETIRED_VARIABLE}，"
                    "scripts/gate/admission.py experiment 照用法错退 2，不写它的重跑登记。\n"
                    + "".join(f"  {reason}\n" for reason in reasons) +
                    f"→ 怎么办：从起本会话的环境里去掉 {ADMISSION_RETIRED_VARIABLE}（shell 的 export、.claude/settings*.json 的 env；派发闸不替它清）再派；"
                    "输入没变不重跑，要重跑就先改输入（装置入口闭包、复跑表那一行的参数）；为别的原因重跑，在提示里写一行「准入已判：<理由>」。")
        # 退 2 一律拒，有两种来历、出路不同：准入判法自己报的（登记里没有这个实验键、这个实验的登记有错，最后那段说明）；判法本身没跑起来的（脚本不在、python 只打用法、什么都没打）
        reported_by_admission = any(ADMISSION_REGISTRATION_SENTENCE.match(line.strip()) for line in result.stderr.splitlines())
        if not reported_by_admission and break_switch_is("admission-unrunnable-allowed"):
            note_unjudged("admission", "! 派发闸：scripts/gate/admission.py experiment 退 2，而输出不是准入判法报「没登记」「登记有错」的句子（python 自己退 2 或用法错），准入这一条没判，照常放行"
                          + (f"：{' / '.join(admission_stderr_tail(result))}" if admission_stderr_tail(result) else ""))
            return None
        if not reported_by_admission and not break_switch_is("admission-unrunnable-one-way-out"):
            return (f"✗ E{int(match.group(1))} 的重跑准入判不了：scripts/gate/admission.py experiment 退 2，而输出不是它自己报「没登记」「登记有错」的句子（判法本身没跑起来：脚本不在、python 只打用法、什么都没打），不写它的重跑登记。\n"
                    + "".join(f"  {line}\n" for line in admission_stderr_tail(result)) +
                    "→ 怎么办：先让 admission.py 能跑（核 scripts/gate/admission.py 在、python3 起得来，自己跑一遍 `python3 scripts/gate/admission.py experiment . E<号>` 看它打什么），能跑之后再派；"
                    "确要照现在的样子派，在提示里写一行「准入已判：<理由>」。")
        return (f"✗ E{int(match.group(1))} 的重跑准入判不了（scripts/gate/admission.py experiment 退 2：准入登记里没有这个实验键，或这个实验的登记有错），不写它的重跑登记。\n"
                + "".join(f"  {reason}\n" for reason in reasons) +
                "→ 怎么办：先在 .claude/gate.d/stage-inputs.tsv 接这个实验的准入（照 E142 那一行写），装置入口传同一把键；说登记有错的，照 admission.py 打的「→ 怎么办」改那一行；"
                "接好之后再派；为别的原因重跑，在提示里写一行「准入已判：<理由>」。")
    if result.returncode == 3 and not break_switch_is("admission-precondition-allowed"):
        # admission.py 退 3 时 stderr 是「✗ 前提没齐 …」一句加逐条前提，末了一句它自己的出路；出路换成派发这一侧的，别的照抄（至多 ADMISSION_PRECONDITION_LINES_SHOWN 行）
        reasons = [line.strip() for line in result.stderr.splitlines() if line.strip() and not line.strip().startswith(("✓", "→"))][:ADMISSION_PRECONDITION_LINES_SHOWN]
        return (f"✗ E{int(match.group(1))} 的重跑准入前提没齐（scripts/gate/admission.py experiment 退 3），不写它的重跑登记。\n"
                + "".join(f"  {reason}\n" for reason in reasons) +
                "→ 怎么办：先补齐 admission.py 报的前提（照它打的「→ 怎么办」做），补齐之后再派；确要照现在的样子派，在提示里写一行「准入已判：<理由>」。")
    if result.returncode == 0 and ADMISSION_KEY_REFUSED_HEADER.search(result.stdout) and not break_switch_is("admission-key-refused-silent"):
        # 退 0 而实验键拒算：admission.py 自己说判不了「没变」（照放行，下一趟也判不了），派发闸同样判不了，照「判不了时放行并记检出」
        note_unjudged("admission", "! 派发闸：scripts/gate/admission.py experiment 退 0，而它的产物头写着 sha256=refused（实验键拒算，判不了「单元键没变」），准入这一条没判，照常放行"
                      + (f"：{' / '.join(admission_stderr_tail(result))}" if admission_stderr_tail(result) else ""))
    if result.returncode not in (0, 2, 3, 77) and not break_switch_is("admission-other-exit-silent"):
        # 认得的结局只有 0、77、3、2（admission.py 文件头 experiment 那一行）；别的码（崩溃退 1 这类）判不了，照派发闸「判不了时放行并记检出」
        note_unjudged("admission", f"! 派发闸：scripts/gate/admission.py experiment 退 {result.returncode}（认得的只有 0、2、3、77），准入这一条没判，照常放行"
                      + (f"：{' / '.join(admission_stderr_tail(result))}" if admission_stderr_tail(result) else ""))
    return None

SNAPSHOT_GUARDED_AGENTS = {"kb-scribe", "implementation-writer"}
# 快照清单怎么读、哪几轮开着、放行行认不认、拒绝说明怎么写：同目录的 lib_snapshot_conflicts.py（与 scripts/gate/apply-writer-patch.py 共用，判法写在它的文件头）
# 一条路径在文字里「出现」：前后不连着路径字符（后面的 . 只在它后面还跟着路径字符时才算连着，句末的点不算）
PATH_LEFT_EDGE = r"(?<![A-Za-z0-9_\-./~])"
PATH_RIGHT_EDGE = r"(?![A-Za-z0-9_\-/]|\.[A-Za-z0-9_])"
DOT_SLASH_BEFORE_REPOSITORY_PATH = re.compile(PATH_LEFT_EDGE + r"\./(?=\.claude/|crates/)")
# 提示里点名的规格文件：工作根或会话临时目录根下的路径（写法见 lib_work_root.py 的 mention_prefix_pattern），到空白、引号、括号或中文标点为止
SCRATCH_FILE_MENTION = re.compile(SCRATCH_MENTION_PREFIX + r"[^\s`'\"，。；：、（）()「」『』“”‘’《》<>\[\]{}|,;*]+")
SPECIFICATION_READ_LIMIT_BYTES = 4 * 1024 * 1024

def break_switch_is(name):
    """自检用的弄坏开关：RUNNER_DISPATCH_GUARD_BREAK 等于 name 时，那一处判法故意走错，自检必须判红。"""
    return os.environ.get("RUNNER_DISPATCH_GUARD_BREAK") == name

# 这一次派发里判不了、照常放行的几处（各处 note_unjudged 交进来；decide 开头清空，放行时与 library_errors 一起经 record_unjudged 记一条检出）
UNJUDGED_NOTES = []
UNJUDGED_SITES = ("scope-library", "scope-table", "transcript", "spec-check", "admission", "snapshot-spec", "scratch-line", "leg-types", "leg-definitions",
                  "fence-compare", "lock-create", "handover-lock", "scratch-root", "experiment-page", "registry-read", "registry-write", "notations-library")

def note_unjudged(site, message, printed=True):
    """判不了而放行的一处：stderr 照原样写 message（printed 为假的那一处另有提醒写出来），记进 UNJUDGED_NOTES。
    弄坏开关 RUNNER_DISPATCH_GUARD_BREAK=<site>-unrecorded 下只打 stderr、不记。"""
    if printed:
        print(message, file=sys.stderr)
    if not break_switch_is(f"{site}-unrecorded"):
        UNJUDGED_NOTES.append(message.strip().removeprefix("! "))

ABANDONED_LEGACY_BREAK_MODES = ("source-any-path", "source-escapes", "line-unread", "name-substring", "round-name-unchecked", "quote-short-ok")   # 出处判法的这六项，派发闸里也认 RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-<项>

def abandoned_break_modes():
    """lib_abandoned_rounds.py 的弄坏开关（LIB_ABANDONED_ROUNDS_BREAK）加上派发闸自己的 snapshot-abandoned-<项>；LIB_ABANDONED_ROUNDS_BREAK 的值认不出时抛 ValueError。"""
    return abandoned_library.breaks() | {mode for mode in ABANDONED_LEGACY_BREAK_MODES if break_switch_is("snapshot-abandoned-" + mode)}

def specification_texts(prompt, notes):
    """提示里点名的工作根或会话临时目录根下的规格文件：[(路径, 内容)]。不在的、是目录的跳过；在而读不了的记进 notes、跳过。"""
    texts = []
    seen = set()
    for mention in SCRATCH_FILE_MENTION.finditer(prompt):
        candidate = expanded_mention(mention.group()).rstrip(".:")
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
            note_unjudged("snapshot-spec", notes[-1], printed=False)
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


def snapshot_conflict_verdict(subagent_type, prompt, project_root, unreadable=None, errors=None):
    """第三条。返回 (0, 给 stderr 的提醒或 None) 放行；(2, 说明) 拒绝。这一条自己出错一律放行，错误写进说明，并追加进 errors（给了的话；decide 放行时据此记检出，
    照「放行但必记检出」；读不到共用模块 lib_snapshot_conflicts.py 也算这一种）。
    读不了的快照清单逐份追加进 unreadable（给了的话；调用方放行时据此记检出），说明里逐份点名那一轮与那份清单（lib_snapshot_conflicts.py 的 unreadable_lines）。
    哪几轮开着、快照清单怎么读、放行行认不认、拒绝说明怎么写都在同目录的 lib_snapshot_conflicts.py（与 scripts/gate/apply-writer-patch.py 共用）；
    这里只找「要改的文件」：派发提示与它点名的规格文件里出现的、快照清单列着的路径。"""
    if break_switch_is("disable-snapshot"):
        return 0, None
    if subagent_type not in SNAPSHOT_GUARDED_AGENTS and not break_switch_is("snapshot-any-agent"):
        return 0, None
    errors = [] if errors is None else errors
    if snapshot_library is None:
        errors.append(f"⑥ 快照那一条读不到同目录的 lib_snapshot_conflicts.py（{snapshot_library_error!r}），这一次没判")
        return 0, f"! 派发闸读不到共用模块 lib_snapshot_conflicts.py（同目录，{snapshot_library_error!r}）：快照那一条这一次放行、不判"
    notes = []
    unreadable = [] if unreadable is None else unreadable
    try:
        rounds = snapshot_library.open_three_way_rounds(project_root, notes, break_switch_is, abandoned_break_modes)
        if not rounds:
            return 0, ("\n".join(notes) or None)
        sources = [("派发提示", prompt)]
        if not break_switch_is("snapshot-spec-unread"):
            sources += [(f"规格 {path}", text) for path, text in specification_texts(prompt, notes)]
        ruled = snapshot_library.ruled_snapshot_conflicts(prompt, break_switch_is, NONE_WORDS)
        conflicts = []  # [(轮名, 快照目录, [(路径, 出现在哪, 清单文件)])]
        for round_name, snapshot_directory in rounds:
            listed = snapshot_library.snapshot_listed_paths(snapshot_directory, notes, break_switch_is, round_name=round_name, unreadable=unreadable)
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
        errors.append(f"⑥ 快照那一条自己出错（{type(error).__name__}: {error}），这一次没判")
        return 0, "\n".join(notes + snapshot_library.unreadable_lines(project_root, unreadable))
    notes += snapshot_library.unreadable_lines(project_root, unreadable)
    if not conflicts:
        return 0, ("\n".join(notes) or None)
    return 2, snapshot_library.refusal_text(project_root, f"派 {subagent_type} 要改的文件", conflicts, notes, "再派", "在派发提示里单起一行写")

CODE_FENCE_OPEN = re.compile(r"^ {0,3}(`{3,})[^`]*$")   # 只有弄坏开关 code-block-line-rule 下才用：逐行认的开头围栏行（至多缩进 3 个空格，三个及以上反引号，语言名里不许有反引号）
CODE_FENCE_OPEN_LOOSE = re.compile(r"^[ \t]*(`{3,})")   # 只有弄坏开关 code-fence-loose 下还在用：更早的行首任意缩进的三个反引号就开块，后面接什么都不看
CODE_BLOCK_CLOSING_PREFIX = re.compile(r"^(?:[ \t]*>)*[ \t]*")   # 围栏块最后一行前面的引用块 > 与缩进（列表项的缩进也在里面），去掉之后才是关的那一行围栏字符

def without_code_blocks_by_lines(prompt, loose):
    """只有弄坏开关 code-block-line-rule、code-fence-loose 下才用：逐行认围栏，只认反引号；开头那一行至多缩进 3 个空格、语言名里不许有反引号
    （loose 时任意缩进的三个反引号就开块），往后找第一行同样多个反引号当关块（loose 时不限缩进），没找到的那一行照原样留着。不认波浪线、列表项与引用块的边界、HTML 块。"""
    fence_open = CODE_FENCE_OPEN_LOOSE if loose else CODE_FENCE_OPEN
    lines = prompt.split("\n")
    kept, index = [], 0
    while index < len(lines):
        opening = fence_open.match(lines[index].rstrip("\r"))
        if opening is not None:
            closing = re.compile((r"[ \t]*`{" if loose else r" {0,3}`{") + str(len(opening.group(1))) + r",}[ \t]*")
            end = next((later for later in range(index + 1, len(lines)) if closing.fullmatch(lines[later].rstrip("\r"))), None)
            if end is not None:
                kept.extend([""] * (end - index + 1))
                index = end + 1
                continue
        kept.append(lines[index])
        index += 1
    return "\n".join(kept)

code_block_library_note_printed = []   # 载不进 markdown-it-py 那一句 stderr 只写一次（几条判法各认一次代码块）
CODE_BLOCK_LIBRARY_VARIABLE = "RUNNER_DISPATCH_GUARD_CODE_BLOCK_LIBRARY"   # 认 CommonMark 用的模块名（不设是 markdown_it）；自检设成一个不在的模块名，走载不进的那一路
LINE_FENCE_OPEN = re.compile(r"(`{3,})[^`]*|(~{3,}).*")   # 宽的逐行认法的开块行（去掉行首的 > 与缩进之后）：反引号围栏后面不许再有反引号（那是行内代码），波浪线围栏后面接什么都行
BATCH_LABEL_LINE = re.compile(LINE_START + r"批次[* \t]*[：:]")   # ⑱ 批次一行的标签行：两种围栏认法下遮不遮要一样
# 只有弄坏开关 retired-lines-refused 下还在用：撤掉的 ⑭ ⑮ 判过的标签行，与正文里成对写死的本机、第二台线程数（撤掉的 ⑱ 写死台数那一判认的一种）
RETIRED_LABEL_LINE = re.compile(LINE_START + r"(?:线程上限(?:已判)?|预计挂钟|分片|续跑|多机(?:已判)?|分不了|复用)[* \t]*[：:]")
RETIRED_HOST_PAIR = re.compile(r"本机[^。；;\n]{0,8}?\d+\s*线程[^。；;\n]{0,16}?第二台[^。；;\n]{0,8}?\d+\s*线程")

def hidden_lines_by_commonmark(lines):
    """照 CommonMark 认（markdown-it-py 的 commonmark 预设）：闭合了的围栏块（块的最后一行去掉引用块的 > 与缩进之后，是同一种围栏字符、不短于开头那一行）
    从开块到关块的行号（从 0 起）；到文末没闭合的、在列表项或引用块结束处结束的、缩进代码块与 HTML 块都不算。载不进 markdown-it-py 交 None，stderr 写一句。"""
    try:
        if break_switch_is("code-block-library-missing"):
            raise ImportError("弄坏开关 code-block-library-missing")
        markdown_it = importlib.import_module(os.environ.get(CODE_BLOCK_LIBRARY_VARIABLE) or "markdown_it")
        parser = markdown_it.MarkdownIt("commonmark")
    except Exception as error:   # 没装、装坏了都按载不进办
        if not code_block_library_note_printed:
            code_block_library_note_printed.append(True)
            print(f"! 派发闸载不进 markdown-it-py（{error!r}）：代码块这一次"
                  + ("一行都不遮，整份照判" if break_switch_is("code-block-fallback-unmasked") else "照宽的逐行认法遮，不比两种认法"), file=sys.stderr)
        return None
    parsed = "\n".join(line.rstrip("\r").replace("\r", " ") for line in lines)   # 与 lines 一行对一行：markdown-it 把单独的 \r 也当换行
    hidden = set()
    for token in parser.parse(parsed):
        if token.type != "fence" or not token.map:
            continue
        first, end = token.map
        if end - first < 2 or end > len(lines):
            continue
        closing = CODE_BLOCK_CLOSING_PREFIX.sub("", lines[end - 1].rstrip("\r"), count=1)
        if re.fullmatch(re.escape(token.markup[0]) + "{" + str(len(token.markup)) + r",}[ \t]*", closing):
            hidden.update(range(first, end))
    return hidden

def hidden_lines_by_wide_line_rule(lines):
    """宽的逐行认法：反引号、波浪线都认，每一行先去掉行首的引用块 > 与任意缩进；开块那一行（LINE_FENCE_OPEN）往后第一行是同一种字符三个以上、后面只有空白的当关块，
    开块到关块的行号（从 0 起）都算；往后找不到关块的那一行不开块。不认列表项与引用块的边界、HTML 块，也不看关块的字符数够不够。"""
    hidden, index = set(), 0
    while index < len(lines):
        opening = LINE_FENCE_OPEN.fullmatch(CODE_BLOCK_CLOSING_PREFIX.sub("", lines[index].rstrip("\r"), count=1))
        if opening is not None:
            closing = re.compile(re.escape((opening.group(1) or opening.group(2))[0]) + r"{3,}[ \t]*")
            end = next((later for later in range(index + 1, len(lines))
                        if closing.fullmatch(CODE_BLOCK_CLOSING_PREFIX.sub("", lines[later].rstrip("\r"), count=1))), None)
            if end is not None:
                hidden.update(range(index, end + 1))
                index = end + 1
                continue
        index += 1
    return hidden

def without_code_blocks(prompt):
    """去掉提示里闭合了的围栏代码块（照 CommonMark 认，hidden_lines_by_commonmark）：围栏行和块里的行都换成空行（行数不变）。⑤ ⑪ ⑰ ⑱ ⑲ 都经这一个函数认代码块，
    ⑱ 批次一行遮之前先过 code_block_ambiguous_label_lines。载不进 markdown-it-py 时退回宽的逐行认法（hidden_lines_by_wide_line_rule）遮，stderr 写一句。"""
    if break_switch_is("code-fence-loose") or break_switch_is("code-block-line-rule"):
        return without_code_blocks_by_lines(prompt, loose=break_switch_is("code-fence-loose"))
    lines = prompt.split("\n")
    hidden = hidden_lines_by_commonmark(lines)
    if hidden is None:
        if break_switch_is("code-block-fallback-unmasked"):
            return prompt
        hidden = hidden_lines_by_wide_line_rule(lines)
    return "\n".join("" if index in hidden else line for index, line in enumerate(lines))

def code_block_ambiguous_label_lines(prompt, label_line):
    """标签行（label_line 认的那几种）在 CommonMark 认法与宽的逐行认法下遮不遮不一致的：[(行号从 1 起, 那一行, CommonMark 遮不遮, 逐行认法遮不遮)]。
    载不进 markdown-it-py（只剩一种认法）、弄坏开关 code-block-ambiguity-unchecked、code-fence-loose、code-block-line-rule 下交空表。"""
    if break_switch_is("code-block-ambiguity-unchecked") or break_switch_is("code-fence-loose") or break_switch_is("code-block-line-rule"):
        return []
    lines = prompt.split("\n")
    commonmark = hidden_lines_by_commonmark(lines)
    if commonmark is None:
        return []
    wide = hidden_lines_by_wide_line_rule(lines)
    return [(index + 1, line, index in commonmark, index in wide) for index, line in enumerate(lines)
            if label_line.match(line) and (index in commonmark) != (index in wide)]

def code_block_ambiguity_refusal(subagent_type, ambiguous, labels, rule=None, howto=None):
    """两种认法遮不遮不一致的拒绝说明：逐行点名、两种认法各怎么认，出路是把围栏写规整。rule、howto 给了的换掉批次一行那两行（⑯ 用）。"""
    shown = "\n".join(f"  第 {number} 行「{line.strip()[:50]}」：照 CommonMark 认{'在' if commonmark else '不在'}代码块里，照逐行配围栏认{'在' if wide else '不在'}代码块里"
                      for number, line, commonmark, wide in ambiguous[:6])
    return (f"✗ 派 {subagent_type} 的提示里{labels}这几行在不在代码块里，两种认法不一致（派发闸判的那一行与读提示的人读到的可能不是同一行）：\n{shown}\n"
            + (rule or "→ 规矩：⑱ 批次一行不判闭合了的围栏代码块（照 CommonMark 认）里的行，标签行在两种认法下遮不遮要一样（.claude/hooks/runner-dispatch-guard.sh 文件头 ⑱ 批次条）。\n")
            + (howto or "→ 怎么办：把围栏写规整：开和关用同一种字符（``` 或 ~~~）、同样多个，关的那一行与开的那一行同样缩进、在同一个列表项或引用块里（引用块里每一行都带 >），"
               "<details> 这类 HTML 块里不留空行；围栏里要示范围栏的，外层用 ~~~、里面用 ```；或者把示例行改写成不以这几个标签起头的一行。"))

# ④⑰ 共用的报告行（report_line_mentions）。④ 的标签只在行首（可带 - * > _ 与「1.」这类编号）或句首（。！？；之后）认，句中、「」『』“”与反引号里的不认；
# ⑰ 的文件名与三方腿形态两判在哪都认（前面不连着字），「」『』“”与反引号里的不认，再按点名的路径落不落在本提示草稿目录下定判不判（own_report_token）。
# 「报告路径 / 报告写进 / 报告写到」冒号可有可无；「报告」要接冒号，或接空白再接路径
REPORT_LABEL = r"(?:报告(?:路径|写进|写到)[* \t]*[：:]?|报告[* \t]*[：:]|报告(?=[ \t]+`?(?:~|/|\$|research/|\.claude/|crates/|litmus/|records/)))"
REPORT_LINE = re.compile(r"(?P<lead>^[ \t>*_\-]*(?:\d+[.)][ \t]+[ \t*_]*)?|(?<=[。！？；])[ \t*_]*)" + REPORT_LABEL + r"(?P<rest>[^\n]*)", re.M)
# 前面不连着字的标签在哪都认：⑰ 用它再去掉引号里的；弄坏开关 report-line-unanchored 下 ④⑰ 都用它、引号里的也认（正文「」里引着的「报告：」被当成报告行）
REPORT_LINE_UNANCHORED = re.compile(r"(?<![^\W_])" + REPORT_LABEL + r"(?P<rest>[^\n]*)")
# ⑰ 只判报告行里落在本提示「草稿目录：」一行点名的目录下的路径与不带目录的裸文件名；别的目录下的当转述别的子 agent 的报告，不判
# （按行首句首锚分不出自己的报告行与转述，所以按草稿目录判）。弄坏开关 report-line-position-anchored 下用 ④ 的锚法、不看目录。
# 那一行怎么认只在同目录 lib_prompt_scratch_line.py（与 ⑯、完工清理闸、交回闸 ⑥ 共用）
REPORT_SCRATCH_PLACEHOLDER = re.compile(r"<(?:填[ \t]*)?草稿目录[^>\n]*>(?:/|$)")   # 「<草稿目录>/report.md」这类写法：说的就是本提示的草稿目录
QUOTE_PAIRS = {"「": "」", "『": "』", "“": "”"}
# 报告行正文里紧跟在否定词后面的文件名（「（不要叫 report.md）」）不交文件名判；弄坏开关 report-name-negation-judged 下也判
REPORT_NAME_NEGATION_BEFORE = re.compile(r"(?:不|别|勿|禁止)[^，,；;。（）()]{0,4}$")
# ⑰ 只认的行首「报告：」：只有弄坏开关 report-line-labels-split 下才用
REPORT_LINE_BEFORE_MERGE = re.compile(LINE_START + r"报告[* \t]*[：:][* \t]*(?P<rest>.*?)[ \t]*$", re.M)
REPORT_NAME_TOKEN = re.compile(r"[^\s`'\"，。；：、（）()「」『』“”,;*]+")   # 报告行正文里的一段路径或词（尖括号不切开：「<轮>-sonnet-output.md」这类形态写法整段是一个词）
REPORT_NAME_EXTENSION = re.compile(r"\.[A-Za-z0-9]+$")   # 带扩展名才算文件名（光一个 report 字不算）
REPORT_WORD_LOWERCASE, REPORT_WORD_ANY_CASE = re.compile("report"), re.compile("report", re.I)
# 工具层另拒的名字：以 SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 .md（拆分提案第九十节 ㉑；实测 research/prompts/subagent-landing-audit-2026-10-22/b9-prefix-probe.md：
# 名字中间或末尾带的、不是 .md 的写得进去）。扩展名照工具层也不分大小写：.MD 工具层拒、这里照拒，.markdown 工具层放、这里照放
# （实测 research/prompts/subagent-landing-audit-2026-10-22/b12-scratch-lib.md「6.1 工具层真调」）。弄坏开关 report-name-prefixes-unchecked 下不判这三个前缀
REPORT_REFUSED_PREFIX_NAME = re.compile(r"(?i:summary|findings|analysis)[^/]*\.(?i:md)")
REPORT_REFUSED_PREFIX_NAME_LOWERCASE_EXTENSION = re.compile(r"(?i:summary|findings|analysis)[^/]*\.md")   # 只有弄坏开关 report-name-prefix-extension-lowercase 下才用：扩展名只认小写 .md

def report_name_refused_prefix(name):
    """名字（路径最后一段）以 SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）、扩展名是 .md（不分大小写）的：工具层的 Write 拒这种名字。
    弄坏开关 report-name-prefixes-unchecked 下一概不判，report-name-prefix-extension-lowercase 下扩展名只认小写。"""
    if break_switch_is("report-name-prefixes-unchecked"):
        return False
    pattern = REPORT_REFUSED_PREFIX_NAME_LOWERCASE_EXTENSION if break_switch_is("report-name-prefix-extension-lowercase") else REPORT_REFUSED_PREFIX_NAME
    return bool(pattern.fullmatch(name))

def report_token_is_file(token):
    """这个词的最后一段带扩展名：是一个文件名。"""
    return bool(REPORT_NAME_EXTENSION.search(token.rstrip("/.").rsplit("/", 1)[-1]))

def quoted_spans(text, keep_backtick_spans=True):
    """「」『』“”与反引号引着的区间 → [(起, 止)]。「」『』“”照整份配对、可以跨行（拆分提案第九十节 ㉟：一句引文折到下一行，后半截原来不当引号里的），
    可以套，没配上的开头不算；反引号按行配对（一行里没配上的反引号不开引文）。弄坏开关 quote-pairs-per-line 下「」这类也按行配对。
    keep_backtick_spans 为假时只交「」『』“”的区间（反引号里的引号字符照旧不算引号，只是反引号那一段本身不进结果）。"""
    spans, offset, stack = [], 0, []
    per_line = break_switch_is("quote-pairs-per-line")
    for line in text.split("\n"):
        index = 0
        if per_line:
            stack = []
        while index < len(line):
            character = line[index]
            if character == "`":
                run = len(line[index:]) - len(line[index:].lstrip("`"))
                close = line.find("`" * run, index + run)
                while close >= 0 and close + run < len(line) and line[close + run] == "`":
                    close = line.find("`" * run, close + run + 1)
                if close < 0:
                    index += run
                    continue
                if keep_backtick_spans:
                    spans.append((offset + index, offset + close + run))
                index = close + run
                continue
            if character in QUOTE_PAIRS:
                stack.append((character, offset + index))
            elif character in QUOTE_PAIRS.values():
                for depth in range(len(stack) - 1, -1, -1):
                    if QUOTE_PAIRS[stack[depth][0]] == character:
                        spans.append((stack[depth][1], offset + index + 1))
                        del stack[depth:]
                        break
            index += 1
        offset += len(line) + 1
    return spans

# ⑰ 三方腿：禁读清单那几段（拆分提案第九十节 ㉗、㉙）。行首（可带 - * > _ 与粗体）「禁读清单」或「禁读」接冒号的一行，冒号后面到行尾；
# 冒号后面是空的，往下连着的列表项（- * + 或「1.」「2)」起头）都算，到第一行不是列表项的为止；「#」起头的标题行里有「禁读」的，到下一行标题为止
FORBIDDEN_LABEL_LINE = re.compile(r"^[ \t>*_\-]*禁读(?:清单)?[* \t]*[：:][* \t]*(?P<rest>[^\n]*)$", re.M)
FORBIDDEN_HEADING_LINE = re.compile(r"^#{1,6}[ \t]+[^\n]*禁读[^\n]*$", re.M)
FORBIDDEN_LIST_ITEM = re.compile(r"[ \t]*(?:[-*+]|\d+[.)])[ \t]+\S")
HEADING_LINE = re.compile(r"^#{1,6}[ \t]", re.M)

def forbidden_list_spans(text):
    """禁读清单那几段在 text 里的区间 → [(起, 止)]（写法见上面的注释）。"""
    spans = []
    for match in FORBIDDEN_LABEL_LINE.finditer(text):
        end = match.end()
        if not match.group("rest").strip(" \t*"):
            while end < len(text):
                following_end = text.find("\n", end + 1)
                following_end = len(text) if following_end < 0 else following_end
                if not FORBIDDEN_LIST_ITEM.match(text[end + 1:following_end]):
                    break
                end = following_end
        spans.append((match.start(), end))
    for match in FORBIDDEN_HEADING_LINE.finditer(text):
        later = HEADING_LINE.search(text, match.end())
        spans.append((match.start(), later.start() if later else len(text)))
    return spans

def inside_any(position, spans):
    return any(start <= position < end for start, end in spans)

# ⑰ 三方腿：本腿自己的轮名（拆分提案第九十节 ㉗）。两处取：行首「背景材料：」一行里的 _<轮>-background.md 或 _<轮>-body.md（三种腿的定义都要「背景材料」这一样输入，
# 骨架那一行就是三方材料脚本拼好的那一份）；前面不连着字的「轮名」标签后面（冒号可有可无、可带粗体与一个反引号）的那个名字。引号与反引号里的「轮名」不算
ROUND_NAME = r"[A-Za-z0-9][A-Za-z0-9_.\-]*[A-Za-z0-9]"
BACKGROUND_LINE = re.compile(r"^[ \t>*_\-]*背景材料[* \t]*[：:](?P<rest>[^\n]*)$", re.M)
BACKGROUND_ROUND = re.compile(r"(?<![A-Za-z0-9_.\-])_(?P<name>" + ROUND_NAME + r")-(?:background|body)\.md(?![A-Za-z0-9_\-])")
ROUND_NAME_LABEL = re.compile(r"(?<![^\W_])轮名[* \t]*[：:]?[* \t]*`?(?P<name>" + ROUND_NAME + r")")

def leg_round_names(text, spans=None):
    """本腿自己的轮名 → 排好序的列表（取法见上面的注释）；spans 是引号区间（不给就现算），「轮名」标签落在里面的不算。"""
    spans = quoted_spans(text) if spans is None else spans
    names = set()
    for line in BACKGROUND_LINE.finditer(text):
        names.update(found.group("name") for found in BACKGROUND_ROUND.finditer(line.group("rest")))
    names.update(found.group("name") for found in ROUND_NAME_LABEL.finditer(text) if not shell_words.inside_spans(found.start(), spans))
    return sorted(names)

def prompt_scratch_directories(prompt, include_roots=False):
    """⑯ ⑰ 共用：提示里行首「草稿目录：」一行点名的、工作根底下的目录（展开成绝对路径，根本身不算；include_roots 为真时也算）。
    认法只在同目录 lib_prompt_scratch_line.py 的 scratch_directories；读不到那份库、它抛异常（弄坏开关写错这类）交空表，stderr 写一句。"""
    if prompt_scratch_line is None:
        note_unjudged("scratch-line", f"! 派发闸读不到共用模块 {os.path.join(SCRIPT_HOOK_DIR, 'lib_prompt_scratch_line.py')}（{prompt_scratch_line_error!r}）："
                                      "本提示的草稿目录认不出，⑯ 不建锁、⑰ 只判裸文件名")
        return []
    try:
        return prompt_scratch_line.scratch_directories(prompt, include_roots=include_roots)
    except Exception as error:   # 弄坏开关写错、读不到 lib_work_root.py
        note_unjudged("scratch-line", f"! 派发闸认本提示的草稿目录出错（{type(error).__name__}：{error}）：⑯ 不建锁、⑰ 只判裸文件名")
        return []

def report_scratch_directories(prompt):
    """⑰ 认的本提示草稿目录：行首「草稿目录：」一行点名的（prompt_scratch_directories）。"""
    return prompt_scratch_directories(prompt)

def own_report_token(token, scratches):
    """⑰ 判不判报告行里的这个词：不带目录的裸文件名、落在本提示草稿目录下的（「<草稿目录>/…」写法也算）判，别的目录下的不判。"""
    bare = token.rstrip("/")
    if "/" not in bare or REPORT_SCRATCH_PLACEHOLDER.match(bare):
        return True
    path = os.path.normpath(expanded_mention(bare))
    return any(path == scratch or path.startswith(scratch + "/") for scratch in scratches)

def report_line_mentions(prompt, anywhere=False):
    """④⑰ 共用的报告行解析（写法在文件头 ⑰ 条）→ [{"named": 这一行点名的路径（正文里第一个带 / 或带扩展名的词）或 None,
    "tokens": 正文里的每个词, "judged_tokens": 交文件名判的词（紧跟否定词的不算）, "placeholder": 正文里还有没有 <填 …> 占位,
    "before": 标签前面同一行里的记号（行首的 - * > 与编号；④ 看否定词，锚在行首句首之后这里不会有否定词）}]。
    anywhere 为假（④）标签只在行首句首认；为真（⑰）前面不连着字的在哪都认、引号与反引号里的不认（弄坏开关 report-line-position-anchored 下退回行首句首）。
    闭合了的围栏代码块里的行不认（弄坏开关 report-name-codeblock-judged 下也认）；弄坏开关 report-line-labels-split 下 ⑰ 只认行首「报告：」一种，
    report-line-unanchored 下 ④⑰ 都句中也认、引号里的也认，report-name-negation-judged 下紧跟否定词的文件名也交文件名判。"""
    judged = prompt if break_switch_is("report-name-codeblock-judged") else without_code_blocks(prompt)
    spans = []
    if break_switch_is("report-line-labels-split"):
        pattern = REPORT_LINE_BEFORE_MERGE
    elif break_switch_is("report-line-unanchored"):
        pattern = REPORT_LINE_UNANCHORED
    elif anywhere and not break_switch_is("report-line-position-anchored"):
        pattern, spans = REPORT_LINE_UNANCHORED, quoted_spans(judged)
    else:
        pattern = REPORT_LINE
    negation_judged = break_switch_is("report-name-negation-judged")
    mentions = []
    for match in pattern.finditer(judged):
        if spans and shell_words.inside_spans(match.start(), spans):
            continue
        rest = match.group("rest")
        token_matches = list(REPORT_NAME_TOKEN.finditer(rest))
        tokens = [token.group(0) for token in token_matches]
        judged_tokens = [token.group(0) for token in token_matches
                         if negation_judged or not REPORT_NAME_NEGATION_BEFORE.search(rest[:token.start()])]
        named = next((token.rstrip(".") for token in tokens if "/" in token or report_token_is_file(token)), None)
        lead = match.groupdict().get("lead")
        mentions.append({"named": named, "tokens": tokens, "judged_tokens": judged_tokens, "placeholder": bool(SKELETON_PLACEHOLDER.search(rest)),
                         "before": lead if lead is not None else judged[max(0, match.start() - 6):match.start()], "start": match.start()})
    return mentions

def report_file_name_verdict(subagent_type, prompt, project_root=None):
    """⑰ 报告行正文里的文件，名字（路径最后一段，带扩展名）里带 report（不分大小写）的、以 SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 .md 拒：
    Claude Code 自带的拦截不让子 agent 用 Write 建这样的文件，写 result.md 写得进去。
    报告行怎么认在 report_line_mentions（与 ④ 共用，anywhere 为真）；只判 own_report_token 认的词。给了 project_root、类型是三方腿的，本腿的报告路径
    （leg_report_reading 的 own，research/prompts/<名>-output.md，不在草稿目录下）照判名字（拆分提案第九十节 ㉗）。弄坏开关 report-name-unchecked / report-name-case-sensitive / report-name-whole-path /
    report-name-codeblock-judged / report-line-labels-split / report-line-unanchored / report-name-negation-judged / report-line-position-anchored / report-name-prefixes-unchecked / report-name-prefix-extension-lowercase /
    leg-report-name-unchecked（三方腿的报告路径不判名字）。"""
    if break_switch_is("report-name-unchecked"):
        return None
    word = REPORT_WORD_LOWERCASE if break_switch_is("report-name-case-sensitive") else REPORT_WORD_ANY_CASE
    anchored = break_switch_is("report-line-position-anchored")
    scratches = [] if anchored else report_scratch_directories(prompt)
    named = []
    for mention in report_line_mentions(prompt, anywhere=True):
        for token in mention["judged_tokens"]:
            if not anchored and not own_report_token(token, scratches):
                continue
            name = token if break_switch_is("report-name-whole-path") else token.rstrip("/.").rsplit("/", 1)[-1]
            prefixed = report_name_refused_prefix(name)
            if REPORT_NAME_EXTENSION.search(name) and (word.search(name) or prefixed) and token not in named:
                named.append(token)
    if (project_root is not None and not break_switch_is("leg-report-name-unchecked") and three_way_legs is not None
            and subagent_type in three_way_report_types(project_root)):
        for written in leg_report_reading(prompt, project_root)["own"]:
            name = written.rsplit("/", 1)[-1]
            prefixed = report_name_refused_prefix(name)
            if (word.search(name) or prefixed) and written not in named:
                named.append(written)
    if not named:
        return None
    shown = "、".join(f"「{token[:80]}」" for token in named[:4])
    return (f"✗ 派 {subagent_type} 的提示报告行点名的文件名里带 report，或是以 SUMMARY、FINDINGS、ANALYSIS 开头的 .md：{shown}。\n"
            "→ 规矩：Claude Code 自带的拦截不让子 agent 用 Write 建名字为 report.md 的文件（名字里带 report 的别的文件也有被拒的），工具结果原文是"
            "「Subagents should return findings as text, not write report files. Include this content in your final response instead.」；"
            "这条拦截不在仓里的任何钩子里，2026-10-14 起子 agent 写 report.md 209 次全被拒，写 result.md、progress.md 这类名字都写得进去"
            "（records/2026-10-22-子agent结果落地审计.md「第二批：用户定案之后」）；名字以 SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 .md 同样被拒，"
            "这三个词在名字中间或末尾的、不是 .md 的写得进去（research/prompts/subagent-landing-audit-2026-10-22/b9-prefix-probe.md）；报告行认「报告」「报告路径」「报告写进」「报告写到」四种标签（前面连着字的、"
            "「」『』“”与反引号里引着的不认），只判落在本提示草稿目录下的路径与不带目录的裸文件名（别的目录下的当转述别的子 agent 的报告，不判；提示里没有行首「草稿目录：」一行的只判裸文件名），"
            "三方腿另判本腿的报告路径 research/prompts/<轮>-<模型>-output.md 的名字（共用约束「报告」一节：报告文件名里不带 report），"
            "紧跟否定词的文件名（「不要叫 report.md」）不判。\n"
            "→ 怎么办：报告行改成草稿目录下的 result.md，例：「报告：~/.local/share/singlefs/work/<目录>/result.md」（三方腿照 research/prompts/<轮>-<模型>-output.md，<模型> 那一段写模型名、不带 report）；"
            "骨架（python3 scripts/agents/dispatch-prompt.py <类型>）默认就填这个名字；文件名里不带 report，目录名带不带都行。")

THREE_WAY_REPORT_TYPES_BEFORE_REGISTRY = ("three-way-forward", "three-way-attack", "three-way-defense")   # 写死的那一份：只有弄坏开关 leg-types-hardcoded 下才用

def three_way_report_types(project_root):
    """⑰ 要报告形态的三方腿类型：派发那一刻的仓根下定义 frontmatter 写了 leg: three-way 的（lib_three_way_legs.py 的 three_way_leg_types）。
    读不到那份库交空集、stderr 写一句；弄坏开关 leg-types-hardcoded 下用写死的三种。"""
    if break_switch_is("leg-types-hardcoded"):
        return frozenset(THREE_WAY_REPORT_TYPES_BEFORE_REGISTRY)
    if three_way_legs is None:
        note_unjudged("leg-types", f"! 派发闸读不到共用模块 {os.path.join(SCRIPT_HOOK_DIR, 'lib_three_way_legs.py')}（{three_way_legs_error!r}），三方腿的报告形态这一条没判")
        return frozenset()
    unreadable = three_way_legs.unreadable_definitions(project_root)
    if unreadable:
        note_unjudged("leg-definitions", f"! 派发闸：定义 {'、'.join(unreadable)} 读不了，三方腿类型表没读全，那几种的报告形态这一条没判")
    return three_way_legs.three_way_leg_types(project_root)

# 三方腿报告路径的形态只在同目录 lib_three_way_legs.py 认：报告行那一支（leg_report_written_tree）、整份提示那一支（leg_report_mentions）与交回闸 ②
# （leg_report_mentions_in_trees）都经那份库的 leg_report_mentions_in_trees 一个函数（拆分提案第九十节 ㉗）：没填的占位（<轮>）、
# 子目录下的、仓外副本里的都不认，仓根、./、工作树的绝对路径、~ 与 $HOME 起头的都认。报告行那一支认占位（下面这一份），只有弄坏开关 leg-report-placeholder-kept 下才用
THREE_WAY_REPORT_FORM_BEFORE_LIBRARY = re.compile(r"research/prompts/[^/\s`'\"]+-output\.md")

def leg_report_path_verdict(named, project_root):
    """⑰ 报告行点名的这一个词整个是不是三方腿报告形态（lib_three_way_legs.py 的 leg_report_written_tree）。
    弄坏开关 leg-report-placeholder-kept 下用这里自己的一份（认 <轮> 占位）；leg-report-line-root-only 下只去仓根前缀与 ./ 再调 is_leg_report_path（不解工作树、~、$HOME、../）。"""
    if break_switch_is("leg-report-placeholder-kept") or break_switch_is("leg-report-line-root-only"):
        root_prefix = os.path.normpath(project_root) + "/"
        relative = named[len(root_prefix):] if named.startswith(root_prefix) else named[2:] if named.startswith("./") else named
        if break_switch_is("leg-report-placeholder-kept"):
            return bool(THREE_WAY_REPORT_FORM_BEFORE_LIBRARY.fullmatch(relative))
        return three_way_legs.is_leg_report_path(relative)
    return three_way_legs.leg_report_written_tree(named, project_root) is not None

def leg_report_name(written):
    """三方腿报告路径的 research/prompts/<名>-output.md 那一段（禁读清单里比的就是它）；认不出（弄坏开关下放进来的占位）交原样。"""
    found = three_way_legs.LEG_REPORT_MENTION.search(written)
    return found.group(0) if found else written

def work_root_tokens_expanded(text):
    """⑰ 三方腿：文字里的 $SINGLEFS_WORK_ROOT/、${SINGLEFS_WORK_ROOT}/ 先换成展开后的工作根（同目录 lib_work_root.py 的 expand_written_path，与交回闸 ② 同一种解法），
    再交 lib_three_way_legs.py 认报告路径（那份库只展开 ~ 与 $HOME，带别的变量的前缀不认）：工作根下的本仓工作树里的报告这样才认得出。
    读不到 lib_work_root.py、工作根的弄坏开关写错时原样交回；弄坏开关 leg-report-work-root-token-kept 下不换。"""
    if work_roots is None or break_switch_is("leg-report-work-root-token-kept"):
        return text
    variable = re.escape(work_roots.WORK_ROOT_VARIABLE)
    try:
        return re.sub(r"\$" + variable + r"(?=/)|\$\{" + variable + r"\}(?=/)", lambda found: work_roots.expand_written_path(found.group(0)), text)
    except ValueError:   # SINGLEFS_WORK_ROOT_BREAK 写错
        return text

def leg_report_reading(prompt, project_root):
    """⑰ 三方腿本腿的报告路径（写法在文件头 ⑰ 条）→ dict：
    own 本腿的报告（research/prompts/<名>-output.md 那一段，按出现次序、去重）：报告行点名的合形态的（第一个像路径的词不合形态、又不是本提示自己的，
        这一行正文里后面合形态的那个词也算；报告标签落在禁读清单里的不算）；报告行里一处都没有时，
        整份提示里不在「」『』“”与反引号里、不在禁读清单里、名字以本腿轮名加「-」起头的；
    wrong 报告行点名的、落在本提示草稿目录下或是裸文件名、又不合形态的；elsewhere 报告行点名的别的目录下的（当转述）；placeholder 报告行里还有 <填 …> 占位；
    rounds 本腿轮名（leg_round_names）；forbidden_only 整份提示里只在禁读清单里的；other_round 不在本腿轮名下的；forbidden_own 禁读清单里点到本腿自己报告的。
    弄坏开关 leg-report-forbidden-counted（禁读清单当普通正文）、leg-report-any-round（不看轮名）、leg-report-forbidden-own-unchecked（不判禁读清单点到自己的报告）、
    report-line-position-anchored（报告行只认行首句首、点名的都判、不看整份提示）、leg-report-work-root-token-kept（$SINGLEFS_WORK_ROOT/ 不展开）。"""
    prompt = work_root_tokens_expanded(prompt)
    anchored = break_switch_is("report-line-position-anchored")
    judged = prompt if break_switch_is("report-name-codeblock-judged") else without_code_blocks(prompt)
    forbidden_all = forbidden_list_spans(judged)
    forbidden = [] if break_switch_is("leg-report-forbidden-counted") else forbidden_all
    scratches = [] if anchored else report_scratch_directories(prompt)
    reading = {"own": [], "good": [], "wrong": [], "elsewhere": [], "placeholder": False, "rounds": [], "forbidden_only": [], "other_round": [], "forbidden_own": []}
    outside = []   # own 里认在禁读清单之外的那几份：禁读清单点到的是不是本腿自己的报告，拿它们比（开关 leg-report-forbidden-counted 下禁读清单里认到的不算进来）
    for mention in report_line_mentions(prompt, anywhere=True):
        if mention["placeholder"]:
            reading["placeholder"] = True
            continue
        if not mention["named"] or not report_token_is_file(mention["named"]) or inside_any(mention["start"], forbidden):
            continue
        named = mention["named"]
        # 第一个像路径的词不合形态、也不是本提示自己的（形态写在括注里、真路径在后面）：这一行正文里后面合形态的那个词照认
        later = None if anchored else next((token.rstrip(".") for token in mention["tokens"]
                                            if token.rstrip(".") != named and leg_report_path_verdict(token.rstrip("."), project_root)), None)
        if leg_report_path_verdict(named, project_root) or (later and not own_report_token(named, scratches)):
            chosen = named if leg_report_path_verdict(named, project_root) else later
            reading["good"].append(chosen)
            if leg_report_name(chosen) not in reading["own"]:
                reading["own"].append(leg_report_name(chosen))
            if not inside_any(mention["start"], forbidden_all):
                outside.append(leg_report_name(chosen))
        elif anchored or own_report_token(named, scratches):
            reading["wrong"].append(named)
        else:
            reading["elsewhere"].append(named)
    spans = quoted_spans(judged)
    reading["rounds"] = leg_round_names(judged, spans)
    if not reading["good"] and not anchored:
        mentions_in_text = (list(three_way_legs.LEG_REPORT_MENTION.finditer(judged)) if break_switch_is("leg-report-outside-root-kept")
                            else three_way_legs.leg_report_mentions(judged, project_root))
        for match in mentions_in_text:
            written = match.group(0)
            if shell_words.inside_spans(match.start(), spans):
                continue
            if inside_any(match.start(), forbidden):
                reading["forbidden_only"].append(written)
            elif not break_switch_is("leg-report-any-round") and not any(written.rsplit("/", 1)[-1].startswith(name + "-") for name in reading["rounds"]):
                reading["other_round"].append(written)
            else:
                reading["good"].append(written)
                if written not in reading["own"]:
                    reading["own"].append(written)
                if not inside_any(match.start(), forbidden_all):
                    outside.append(written)
    if not break_switch_is("leg-report-forbidden-own-unchecked"):
        for match in three_way_legs.LEG_REPORT_MENTION.finditer(judged):
            if inside_any(match.start(), forbidden_all) and match.group(0) in outside and match.group(0) not in reading["forbidden_own"]:
                reading["forbidden_own"].append(match.group(0))
    return reading

def three_way_report_verdict(subagent_type, prompt, project_root):
    """⑰ 三方腿：要有一处本腿的报告路径 research/prompts/<名>-output.md（leg_report_reading 的 own：报告行点名的，或整份提示里本腿轮名下的）；
    报告行点名的文件落在本提示草稿目录下或是裸文件名（own_report_token）、又不是这一形态的拒；报告行点名的别的目录下的文件当转述，不判；
    禁读清单里点到本腿自己的报告路径的拒（拆分提案第九十节 ㉙）。报告行里还有 <填 …> 占位的交 ⑬，这里不看那一行。
    弄坏开关 three-way-report-unchecked 与 leg_report_reading 里那几条。"""
    if break_switch_is("three-way-report-unchecked") or subagent_type not in three_way_report_types(project_root):
        return None
    anchored = break_switch_is("report-line-position-anchored")
    reading = leg_report_reading(prompt, project_root)
    good, wrong, elsewhere = reading["good"], reading["wrong"], reading["elsewhere"]
    if reading["placeholder"] and not good and not wrong:
        return None
    if good and not wrong and not reading["forbidden_own"]:
        return None
    found = []
    if reading["forbidden_own"]:
        found.append("禁读清单里点到了本腿自己的报告：" + "、".join(f"「{written[:100]}」" for written in reading["forbidden_own"][:4])
                     + "（腿读不了自己要写的那一份，派出去也照禁读清单跳过它）")
    if wrong:
        found.append("报告行点名的文件不是这一形态：" + "、".join(f"「{named[:100]}」" for named in wrong[:4]))
    elif not good and anchored:
        found.append("一行报告行都没有（或报告行里没有点名文件）")
    elif not good:
        rounds = "、".join(reading["rounds"]) or "（认不出：没有「背景材料：」一行里的 _<轮>-background.md，也没有「轮名」标签）"
        found.append("没有一处本腿的报告路径 research/prompts/<轮>-<模型>-output.md（报告行点名的，或整份提示里不在「」『』“”与反引号里、不在禁读清单里、名字以本腿轮名起头的；"
                     f"本腿轮名 {rounds}）"
                     + ("；只在禁读清单里的" + "、".join(f"「{written[:100]}」" for written in reading["forbidden_only"][:4]) + "是别的腿的，不当本腿的" if reading["forbidden_only"] else "")
                     + ("；不在本腿轮名下的" + "、".join(f"「{written[:100]}」" for written in reading["other_round"][:4]) + "不当本腿的" if reading["other_round"] else "")
                     + ("；报告行点名的" + "、".join(f"「{named[:100]}」" for named in elsewhere[:4]) + "不在本提示草稿目录下，当转述不判" if elsewhere else ""))
    return (f"✗ 派 {subagent_type} 的提示{'；'.join(found)}；三方腿的报告要形如 research/prompts/<轮>-<模型>-output.md，写在「报告：」一行，不进禁读清单。\n"
            "→ 规矩：三方论证的云端腿，报告就是派发提示给的 research/prompts/<轮>-*-output.md（.claude/rules/three-way-inference.md「云端腿的报告要分段落盘」）；"
            "交回闸 .claude/hooks/handback-guard.sh 只认这种形态，派发时写草稿目录下的 result.md，腿照做必在交回时被拒；禁读清单列的是同一轮别的腿的产出，"
            "那里的 -output.md 不是这条腿的报告，点到这条腿自己的报告就是写错了腿（landing-audit-r4 攻方提示头一版）。\n"
            "→ 怎么办：写一行「报告：research/prompts/<轮>-<模型>-output.md」（轮与模型两处换成这一轮的轮名与腿的模型，例「报告：research/prompts/x-r1-opus-output.md」）；"
            "禁读清单只列别的腿的报告与模型目录；"
            f"骨架（python3 scripts/agents/dispatch-prompt.py {subagent_type}）打的就是这一行的占位。")

UNTOUCHED_LABEL_LINE = re.compile(LINE_START + r"(?P<label>[^：:\n]{0,40}?)[* \t]*[：:][* \t]*(?P<value>.*?)[ \t]*$")   # ⑱ 不碰那一行：行首标签（冒号前不到 40 个字）里有「不碰」
LIST_ITEM_LINE = re.compile(r"^[ \t>]*(?:[-*+]|\d+[.)])[ \t]+\S")

def labelled_values_with_list(prompt, label_matches):
    """行首标签行的值（闭合围栏块里的不算）：值空着的，接它下面连着的列表项行（「- …」「1. …」）当值。label_matches(行的 match) 判这一行算不算。→ [值]。"""
    lines = without_code_blocks(prompt).split("\n")
    values = []
    for index, line in enumerate(lines):
        found = UNTOUCHED_LABEL_LINE.match(line)
        if not found or not label_matches(found):
            continue
        value = found.group("value").strip(" *")
        if not value:
            items = []
            for following in lines[index + 1:]:
                if not LIST_ITEM_LINE.match(following):
                    break
                items.append(following.strip())
            value = " ".join(items)
        values.append(value)
    return values
