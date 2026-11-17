#!/usr/bin/env bash
# admission: always Claude Code 每一次 Bash 调用都要现判这一次的命令文本，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Bash）：find / bfs 的 -newermt、-newerat、-newerct、-newerBt 后面跟的时刻不是 ISO 8601 写法的，执行前拒绝（退出 2，stderr 写原因与改成 ISO 的那一条）。
# 本机 Claude Code 会话里的 find 是 bfs（.claude/kb/tooling.md「本机 Claude Code 会话里的 `grep` 与 `find` 不是 GNU 那两个」），不认带时区名、相对时间这类写法：
# 报 `bfs: error: … Invalid timestamp`、一行都不输出，接在管道或 `|| true` 后面读起来像零命中。ISO 8601 写法 GNU find 与 bfs 都认，所以两台、两种 find 一律照这一条判。
# 主 agent 与子 agent 一样拒。
# hook-events: PreToolUse:Bash
# gate-similar: bash-command-detector.sh 同挂 PreToolUse[Bash]、同用 lib_shell_words.py 切词；它的各道判的是进程、后台、写文件与 agent 角色的危险写法，这一道判的是 find 时刻参数的写法与本机 find 的实现对不对得上，判据没有一条相同；它已有七千多行、自证要跑几十个开关，并进去要动它的拒绝汇总与开关表
# gate-similar: project-code-guard.sh 同挂 PreToolUse[Bash]、同用 lib_shell_words.py；它判的是跑项目代码经没经 task-server，与 find 的参数无关
# gate-similar: run-scope-guard.sh 同挂 PreToolUse[Bash]；它判起验证的命令带没带范围声明，与 find 的参数无关
# gate-similar: timer-only-remind-guard.sh 同挂 PreToolUse[Bash]（另挂五个工具）、同用 lib_shell_words.py；它判的是定时的写法，与 find 的参数无关
# gate-similar: pattern-process-guard.sh（上游）同挂 PreToolUse[Bash]；它拒按模式找进程的写法，与 find 无关；上游副本不改
# gate-similar: root-entry-guard.sh（上游）同挂 PreToolUse[Bash]；它管项目根下新建文件与目录，与 find 无关；上游副本不改
#
# 拒的：命令位置上的 find、bfs（路径写法 /usr/bin/find 这类同样认；前缀、bash -c、管道、cd 之后、命令替换里的照 lib_shell_words.py 的 commands_at_command_position 认），
#   参数里有 -newer<X>t（X 是 a、B、c、m）而它后面那个词照 ISO_TIMESTAMP 不是 ISO 8601 写法：
#   日期时刻之后接时区名、相对时间（`-40 minutes`、`yesterday`）、月日不补零（`2026-11-9`）、带小数秒、时区偏移前有空格这一类。
#   同一段命令文本里前面赋过字面值的变量（`t='…'; find . -newermt "$t"`）照 expand_known_variables 展开之后判。
# 认的写法（ISO_TIMESTAMP；bfs 实测都认）：YYYY-MM-DD；后面接 T 或一个空格再接 HH、HH:MM、HH:MM:SS（冒号可省），可再接 Z 或 ±HH[:MM]；
#   八位的 YYYYMMDD 同样可接 T 与时刻；@<秒数>。
# 不拒的：-newer 文件、-newerXY 的 Y 不是 t（比的是另一个文件的时刻）；引号里当数据写的、grep 的参数、echo 的字、heredoc 正文；
#   时刻是命令替换（`"$(date -u -d '-40 minutes' +%Y-%m-%dT%H:%M:%SZ)"`）或没在这段文本里赋过值的变量（执行前算不出值，不判）。
# 判不到的：变量里拼出来的命令、eval 的变量代码、脚本文件里写的 find、xargs / find -exec 接进来的 find、时刻经命令替换或外面传进来的变量算出来的。
# 判不了的照「放行但必记检出」：输入不是 JSON 对象、读不到 lib_shell_words.py、command 不是字符串、命令切不开、判的时候抛异常，照常退 0，stderr 写一行卡在哪，
#   检出记录（AGENT_HOOK_DETECTIONS，没设取同目录 lib_work_root.py 的 default_detections_path，写法在它的 record_unjudged_pass）追加一行、结局「只记」，
#   findings 以「时刻写法闸没判、照常放行」起头；读不到 lib_work_root.py 时 stderr 另说一句没记成。
# 命令文本里没有 -newer 的直接放行，不载入切词模块。
# 自证样本里的时刻与时区名拆开拼（COLON、UTC_WORD）：写闸与门禁 doc-text 的 clock-times 格判源码里的钟点字面。
# 退役判据：本机与第二台的会话里 find 都不再是 bfs，或 bfs 认了这几种写法，就删掉这个钩子。
#
#   find-timestamp-guard.sh --selftest   在内存里判造出来的 hook JSON，再起进程核退出码与 stderr；末尾把每个弄坏开关各起一趟 --selftest-cells、按每台默认重活线程数并行，核它红在点名的格
# 弄坏开关 FIND_TIMESTAMP_GUARD_BREAK=<项>（只给证红用，逗号连几个；平常跑时不认）。项名、弄坏哪一处、打开后红在哪一格（格名是「✗ 自证 <格名>：」里的那一段）：
# 平常跑设了 FIND_TIMESTAMP_GUARD_BREAK 就在一切之前退 2、点名变量只在自证里认；自证起的子进程由 FIND_TIMESTAMP_GUARD_SELFTEST_MARK 认。
#   FIND_TIMESTAMP_GUARD_BREAK=zone-name-accepted  时刻后面带时区名的照 ISO 放行
#       「✗ 自证 zone-names：」
#   FIND_TIMESTAMP_GUARD_BREAK=relative-accepted  不以数字或 @ 起头的时刻（相对时间、yesterday）放行
#       「✗ 自证 relative-times：」
#   FIND_TIMESTAMP_GUARD_BREAK=date-shape-loose  四位数字起头的时刻一律放行（月日不补零、小数秒、偏移前有空格都不拒）
#       「✗ 自证 malformed-dates：」
#   FIND_TIMESTAMP_GUARD_BREAK=newer-variants-ignored  只判 -newermt，-newerat、-newerct、-newerBt 不判
#       「✗ 自证 newer-variants：」
#   FIND_TIMESTAMP_GUARD_BREAK=nested-unseen  只看整段文本切开后的第一条命令（bash -c 里、管道后段、cd 之后、bfs 与路径写法的 find 都看不到）
#       「✗ 自证 nested-commands：」
#   FIND_TIMESTAMP_GUARD_BREAK=refuse-iso-with-time  只认 YYYY-MM-DD，带时刻的 ISO 写法也拒
#       「✗ 自证 iso-forms：」
#   FIND_TIMESTAMP_GUARD_BREAK=refuse-in-data  不切词，整段文本里出现 -newermt 加非 ISO 的字就拒（grep 的参数、echo 的字、heredoc 正文也拒）
#       「✗ 自证 data-not-command：」
#   FIND_TIMESTAMP_GUARD_BREAK=refuse-unknown-values  时刻是命令替换或没赋过值的变量也拒
#       「✗ 自证 unknown-values：」
#   FIND_TIMESTAMP_GUARD_BREAK=suggestion-dropped  拒绝说明里不给换算出来的 ISO 时刻
#       「✗ 自证 suggestion：」
#   FIND_TIMESTAMP_GUARD_BREAK=no-next-step  拒绝说明里不写出路
#       「✗ 自证 next-step：」
#   FIND_TIMESTAMP_GUARD_BREAK=input-unrecorded  输入不是 JSON 对象时只打 stderr、不记检出
#       「✗ 自证 input-recorded：」
#   FIND_TIMESTAMP_GUARD_BREAK=library-unrecorded  读不到 lib_shell_words.py 时只打 stderr、不记检出
#       「✗ 自证 library-recorded：」
#   FIND_TIMESTAMP_GUARD_BREAK=command-shape-unrecorded  command 不是字符串时只打 stderr、不记检出
#       「✗ 自证 command-shape-recorded：」
#   FIND_TIMESTAMP_GUARD_BREAK=split-unrecorded  命令切不开时只打 stderr、不记检出
#       「✗ 自证 split-recorded：」
#   FIND_TIMESTAMP_GUARD_BREAK=exception-unrecorded  判的时候抛异常不接住、照原样抛出（进程退 1，照常放行、不记检出）
#       「✗ 自证 exception-recorded：」
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import concurrent.futures, importlib.util, json, os, re, shlex, shutil, subprocess, sys, tempfile

BREAK_VARIABLE = "FIND_TIMESTAMP_GUARD_BREAK"
HOOK_NAME = "find-timestamp-guard.sh"
UNJUDGED_PREFIX = "时刻写法闸没判、照常放行"
FIND_COMMANDS = ("find", "bfs")
NEWER_TIME_OPTION = re.compile(r"-newer[aBcm]t")
COLON = ":"
UTC_WORD = "U" + "TC"
# ISO 8601 写法里 bfs 实测认的那几种（文件头「认的写法」）
CLOCK_PART = r"\d{2}(?::?\d{2}(?::?\d{2})?)?"
ZONE_PART = r"(?:Z|[+-]\d{2}(?::?\d{2})?)?"
ISO_TIMESTAMP = re.compile(r"(?:\d{4}-\d{2}-\d{2}(?:[T ]" + CLOCK_PART + ZONE_PART + r")?|\d{8}(?:T" + CLOCK_PART + ZONE_PART + r")?|@\d+)")
ISO_DATE_ONLY = re.compile(r"\d{4}-\d{2}-\d{2}")
ZONE_NAME_SUFFIX = re.compile(r"\s+[A-Za-z]{2,5}")
LOOSE_DATE_SHAPE = re.compile(r"\d{4}.*", re.S)
UTC_NAMES = (UTC_WORD, "GMT", "Z")
STRING_ZONE_FORM = re.compile(r"(\d{4}-\d{2}-\d{2})[T ](\d{2}:\d{2}(?::\d{2})?)(?:\.\d+)?\s*(" + "|".join(UTC_NAMES) + r"|[+-]\d{2}:?\d{2})", re.I)
FRACTION_FORM = re.compile(r"(\d{4}-\d{2}-\d{2})[T ](\d{2}:\d{2}(?::\d{2})?)\.\d+")
UNPADDED_DATE = re.compile(r"(\d{4})-(\d{1,2})-(\d{1,2})")


def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


def load_shell_words(hook_dir):
    spec = importlib.util.spec_from_file_location("lib_shell_words", os.path.join(hook_dir, "lib_shell_words.py"))
    library = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(library)
    return library


def iso_timestamp_accepted(value):
    """value 是不是这一道认的 ISO 8601 写法（文件头「认的写法」）；弄坏开关逐个放宽或收紧。"""
    if broken("zone-name-accepted") and ZONE_NAME_SUFFIX.search(value):
        value = ZONE_NAME_SUFFIX.sub("", value)
    if broken("relative-accepted") and not value[:1].isdigit() and not value.startswith("@"):
        return True
    if broken("date-shape-loose") and LOOSE_DATE_SHAPE.fullmatch(value):
        return True
    if broken("refuse-iso-with-time"):
        return ISO_DATE_ONLY.fullmatch(value) is not None
    return ISO_TIMESTAMP.fullmatch(value) is not None


def iso_suggestion(value):
    """从原值换算得出的 ISO 写法；换算不出（相对时间、认不出的形状）交 None。"""
    if broken("suggestion-dropped"):
        return None
    zoned = STRING_ZONE_FORM.fullmatch(value.strip())
    if zoned:
        zone = zoned.group(3)
        return f"{zoned.group(1)}T{zoned.group(2)}{'Z' if zone.upper() in UTC_NAMES else zone}"
    fraction = FRACTION_FORM.fullmatch(value.strip())
    if fraction:
        return f"{fraction.group(1)}T{fraction.group(2)}"
    unpadded = UNPADDED_DATE.fullmatch(value.strip())
    if unpadded:
        return f"{unpadded.group(1)}-{int(unpadded.group(2)):02d}-{int(unpadded.group(3)):02d}"
    return None


def literal_value(shell_words, word, variables):
    """时刻那个词执行前的字面值：赋过字面值的变量展开；命令替换、反引号、没赋过值的变量交 None（不判）。"""
    if broken("refuse-unknown-values"):
        return word
    return shell_words.expand_known_variables(word, variables or {})


def find_word_lists(shell_words, text):
    """命令位置上每一条 find / bfs：(词表, 那一刻记下的变量)。弄坏开关 nested-unseen 下只看整段文本切开后的第一条。"""
    if broken("nested-unseen"):
        words = shlex.split(text, comments=True)
        return [(words, {})] if words and words[0] in FIND_COMMANDS else []
    scan = shell_words.commands_at_command_position(text, uncalled_functions_skipped=True)
    return [(command.words, command.variables) for command in scan.commands
            if command.words and os.path.basename(command.words[0]) in FIND_COMMANDS]


def timestamp_refusals(shell_words, text):
    """命令文本里每一处不是 ISO 写法的时刻：[(选项, 原值)]。"""
    if "-newer" not in text:
        return []
    if broken("refuse-in-data"):
        return [(found.group(1), found.group(2)) for found in re.finditer(r"(-newer[aBcm]t)\s+['\"]?([^'\"\s]+)", text)
                if not iso_timestamp_accepted(found.group(2))]
    found = []
    for words, variables in find_word_lists(shell_words, text):
        for position, word in enumerate(words[1:-1], start=1):
            if not NEWER_TIME_OPTION.fullmatch(word):
                continue
            if word != "-newermt" and broken("newer-variants-ignored"):
                continue
            value = literal_value(shell_words, words[position + 1], variables)
            if value is not None and not iso_timestamp_accepted(value):
                found.append((word, value))
    return found


def refusal_reason(shell_words, hook_input, unjudged=None):
    """要拒就交 [(选项, 原值)]，放行交 []。command 不是字符串、命令切不开（判不了）的放行，给了 unjudged（列表）的往里记 (弄坏开关名, 一句)。"""
    if hook_input.get("tool_name") != "Bash":
        return []
    command = (hook_input.get("tool_input") or {}).get("command")
    if not isinstance(command, str):
        if unjudged is not None:
            unjudged.append(("command-shape-unrecorded", f"Bash 的 tool_input.command 不是字符串（{type(command).__name__}），find 的时刻写法没判"))
        return []
    try:
        return timestamp_refusals(shell_words, command)
    except Exception as error:
        if unjudged is not None:
            unjudged.append(("split-unrecorded", f"Bash 的命令切不开（{type(error).__name__}: {error}），find 的时刻写法没判"))
        return []


def refusal_text(refusals):
    lines = [f"✗ find 的时刻参数不是 ISO 8601 写法：{'；'.join(f'{option} {value!r}' for option, value in refusals)}。"
             "本机会话里的 find 是 bfs，不认这种写法，报 Invalid timestamp、一行都不输出，接在管道后面像零命中（.claude/kb/tooling.md「本机 Claude Code 会话里的 `grep` 与 `find` 不是 GNU 那两个」）"]
    if not broken("no-next-step"):
        for option, value in refusals:
            suggestion = iso_suggestion(value)
            if suggestion is not None:
                lines.append(f"→ 怎么办：{option} {value!r} 改成 {option} '{suggestion}'（ISO 8601，GNU find 与 bfs 都认）")
            else:
                lines.append(f"→ 怎么办：{option} {value!r} 先用 date 换算成 ISO 8601 再交 find："
                             f"{option} \"$(date -u -d {shlex.quote(value)} +%Y-%m-%dT%H:%M:%SZ)\"")
        lines.append("→ 认的写法：YYYY-MM-DD、YYYY-MM-DDTHH:MM:SS（可接 Z 或 ±HH:MM）、@<秒数>")
    return "\n".join(lines)


def record_unjudged(hook_dir, hook_input, problem, switch):
    """判不了、照常放行的这一次：stderr 写一行，照「放行但必记检出」往检出记录追加一行（lib_work_root.py 的 record_unjudged_pass，结局只记）。
    弄坏开关 switch 下只打 stderr、不记；读不到 lib_work_root.py、写不进去时 stderr 说一句没记成。"""
    print(f"! {HOOK_NAME}：{problem}，这一次没判、照常放行", file=sys.stderr)
    if broken(switch):
        return
    tool_input = hook_input.get("tool_input") if isinstance(hook_input, dict) and isinstance(hook_input.get("tool_input"), dict) else {}
    try:
        sys.dont_write_bytecode = True
        if hook_dir not in sys.path:
            sys.path.insert(0, hook_dir)
        import lib_work_root
        _, said = lib_work_root.record_unjudged_pass(HOOK_NAME, hook_input, f"Bash：{tool_input.get('command') or ''}", [f"{UNJUDGED_PREFIX}：{problem}"])
    except Exception as error:   # 文件不在、语法错都算读不到
        said = f"检出没记成：读不到同目录的 lib_work_root.py（{error!r}）"
    print(f"  {said}", file=sys.stderr)


def judge_raw(raw, hook_dir):
    """hook 的原始输入（字节）→ 退出码；拒的往 stderr 写原因与出路，判不了的照「放行但必记检出」（文件头「判不了的」一段）。"""
    try:
        hook_input = json.loads(raw.decode("utf-8"))
        problem = None if isinstance(hook_input, dict) else f"输入顶层是 {type(hook_input).__name__}，不是 JSON 对象"
    except ValueError as error:
        hook_input, problem = None, f"输入不是合法的 JSON（{error}；前 200 字节：{raw[:200].decode('utf-8', 'replace')!r}）"
    if problem:
        record_unjudged(hook_dir, None, problem, "input-unrecorded")
        return 0
    command = hook_input["tool_input"].get("command") if isinstance(hook_input.get("tool_input"), dict) else None
    if isinstance(command, str) and "-newer" not in command:
        return 0
    try:
        shell_words = load_shell_words(hook_dir)
    except Exception as error:
        record_unjudged(hook_dir, hook_input, f"读不到同目录的 lib_shell_words.py（{error!r}）", "library-unrecorded")
        return 0
    unjudged = []
    try:
        refusals = refusal_reason(shell_words, hook_input, unjudged)
    except Exception as error:
        if broken("exception-unrecorded"):   # 弄坏开关：不接住，照原样抛出（进程退 1，照常放行、不记检出）
            raise
        record_unjudged(hook_dir, hook_input, f"判的时候抛异常（{type(error).__name__}: {error}）", "exception-unrecorded")
        return 0
    for switch, problem in unjudged:
        record_unjudged(hook_dir, hook_input, problem, switch)
    if not refusals:
        return 0
    print(refusal_text(refusals), file=sys.stderr)
    return 2


# ── 自证：每一格造答案已知的输入。红样本必须被拒，绿样本必须放行
def bash_input(command, **extra):
    return dict({"tool_name": "Bash", "tool_input": {"command": command}}, **extra)


def clock_text(*parts):
    """自证样本里的时刻：拆开的时、分、秒用冒号连上（文件头「自证样本里的时刻」）。"""
    return COLON.join(parts)


SAMPLE_DAY = "2026-11-09"
SAMPLE_SECONDS = clock_text("20", "12", "29")
SAMPLE_MINUTES = clock_text("20", "12")
SAMPLE_OFFSET = "+" + clock_text("09", "00")
JST_WORD = "J" + "ST"
CELLS = [
    # (格名, 该拒的样本, 该放行的样本)；样本是 (标签, 命令文本)
    ("zone-names", [("带协调世界时", f"find . -newermt '{SAMPLE_DAY} {SAMPLE_SECONDS} {UTC_WORD}'"),
                    ("带 GMT", f"find . -newermt '{SAMPLE_DAY} {SAMPLE_MINUTES} GMT' -type f"),
                    ("T 写法带协调世界时", f"find . -newermt '{SAMPLE_DAY}T{SAMPLE_SECONDS} {UTC_WORD}'"),
                    ("带日本时区名", f"find . -newermt '{SAMPLE_DAY} {SAMPLE_SECONDS} {JST_WORD}'")], []),
    ("relative-times", [("相对分钟", "find . -newermt '-40 minutes'"), ("yesterday", "find /tmp -newermt yesterday -print"),
                        ("last week", "find . -newermt 'last week'")], []),
    ("malformed-dates", [("月日不补零", "find . -newermt 2026-11-9"), ("小数秒", f"find . -newermt {SAMPLE_DAY}T{SAMPLE_SECONDS}.5Z"),
                         ("偏移前有空格", f"find . -newermt '{SAMPLE_DAY}T{SAMPLE_SECONDS} {SAMPLE_OFFSET}'"),
                         ("小写 t", f"find . -newermt {SAMPLE_DAY}t{SAMPLE_MINUTES}")], []),
    ("newer-variants", [("-newerat", "find . -newerat 'yesterday'"), ("-newerct", f"find . -newerct '{SAMPLE_DAY} {SAMPLE_SECONDS} {UTC_WORD}'"),
                        ("-newerBt", "find . -newerBt '-1 day'")], []),
    ("nested-commands", [("bash -c 里", "bash -c \"find . -newermt yesterday\""), ("管道后段", "ls | find . -newermt yesterday | wc -l"),
                         ("cd 之后", "cd /tmp && find . -newermt yesterday"), ("bfs", "bfs . -newermt yesterday"),
                         ("路径写法", "/usr/bin/find . -newermt yesterday"),
                         ("赋过字面值的变量", f"t='{SAMPLE_DAY} {SAMPLE_SECONDS} {UTC_WORD}'; find . -newermt \"$t\""),
                         ("命令替换里", "echo \"$(find . -newermt yesterday | wc -l)\"")], []),
    ("iso-forms", [], [("日期", f"find . -newermt {SAMPLE_DAY}"), ("T 写法带 Z", f"find . -newermt '{SAMPLE_DAY}T{SAMPLE_SECONDS}Z'"),
                       ("空格写法", f"find . -newermt '{SAMPLE_DAY} {SAMPLE_SECONDS}'"), ("带偏移", f"find . -newermt {SAMPLE_DAY}T{SAMPLE_SECONDS}{SAMPLE_OFFSET}"),
                       ("只到分", f"find . -newermt {SAMPLE_DAY}T{SAMPLE_MINUTES}"), ("八位日期", "find . -newermt " + SAMPLE_DAY.replace("-", "")),
                       ("秒数", "find . -newermt @1700000000"), ("-newerct 日期", f"find . -newerct {SAMPLE_DAY} -type f"),
                       ("赋过 ISO 的变量", f"t={SAMPLE_DAY}T{SAMPLE_SECONDS}Z; find . -newermt \"$t\"")]),
    ("data-not-command", [], [("grep 的参数", "grep -rn -- \"-newermt yesterday\" .claude/kb"), ("echo 的字", "echo 'find . -newermt yesterday'"),
                              ("heredoc 正文", "cat > /tmp/x.txt <<'EOF'\nfind . -newermt yesterday\nEOF"),
                              ("-newer 文件", "find . -newer /tmp/stamp"), ("-newermm 比文件", "find . -newermm /tmp/stamp"),
                              ("普通命令", "ls -l && git status --short")]),
    ("unknown-values", [], [("命令替换", "find . -newermt \"$(date -u -d '-40 minutes' +%Y-%m-%dT%H:%M:%SZ)\""),
                            ("没赋过的变量", "find . -newermt \"$SINCE\"")]),
]
SUGGESTIONS = [(f"{SAMPLE_DAY} {SAMPLE_SECONDS} {UTC_WORD}", f"{SAMPLE_DAY}T{SAMPLE_SECONDS}Z"),
               (f"{SAMPLE_DAY}T{SAMPLE_SECONDS} {SAMPLE_OFFSET}", f"{SAMPLE_DAY}T{SAMPLE_SECONDS}{SAMPLE_OFFSET}"),
               (f"{SAMPLE_DAY}T{SAMPLE_SECONDS}.5Z", f"{SAMPLE_DAY}T{SAMPLE_SECONDS}Z"), ("2026-11-9", SAMPLE_DAY),
               (f"{SAMPLE_DAY} {SAMPLE_MINUTES} GMT", f"{SAMPLE_DAY}T{SAMPLE_MINUTES}Z")]
BREAK_TABLE = [("zone-name-accepted", "zone-names"), ("relative-accepted", "relative-times"), ("date-shape-loose", "malformed-dates"),
               ("newer-variants-ignored", "newer-variants"), ("nested-unseen", "nested-commands"), ("refuse-iso-with-time", "iso-forms"),
               ("refuse-in-data", "data-not-command"), ("refuse-unknown-values", "unknown-values"), ("suggestion-dropped", "suggestion"),
               ("no-next-step", "next-step"), ("input-unrecorded", "input-recorded"), ("library-unrecorded", "library-recorded"),
               ("command-shape-unrecorded", "command-shape-recorded"), ("split-unrecorded", "split-recorded"), ("exception-unrecorded", "exception-recorded")]


def selftest_cells(hook_dir):
    shell_words = load_shell_words(hook_dir)
    failures, checked = [], 0

    def fail(cell, detail):
        failures.append(cell)
        print(f"✗ 自证 {cell}：{detail}")
        print(f"→ 怎么办：看 find-timestamp-guard.sh 里 {cell} 对应的判定（timestamp_refusals、iso_timestamp_accepted、iso_suggestion）；{BREAK_VARIABLE} 设着的话这里本来就该红")

    for cell, red_samples, green_samples in CELLS:
        for label, command in red_samples:
            checked += 1
            if not refusal_reason(shell_words, bash_input(command)):
                fail(cell, f"「{label}」（{command!r}）应当被拒，实际放行了")
        for label, command in green_samples:
            checked += 1
            refusals = refusal_reason(shell_words, bash_input(command))
            if refusals:
                fail(cell, f"「{label}」（{command!r}）应当放行，实际被拒：{refusals}")
    # 出路：能换算的给出 ISO 时刻；照出路改过的命令（换算出的、date 换算的）再喂这一道要放行
    for value, wanted in SUGGESTIONS:
        checked += 1
        got = iso_suggestion(value)
        if got != wanted:
            fail("suggestion", f"{value!r} 应当换算成 {wanted!r}，实际 {got!r}")
            continue
        checked += 1
        fixed = f"find . -newermt '{got}'"
        if refusal_reason(shell_words, bash_input(fixed)):
            fail("suggestion", f"照出路改过的命令 {fixed!r} 被这一道自己拒了")
    for value in ("-40 minutes", "yesterday"):
        checked += 1
        fixed = f"find . -newermt \"$(date -u -d {shlex.quote(value)} +%Y-%m-%dT%H:%M:%SZ)\""
        if refusal_reason(shell_words, bash_input(fixed)):
            fail("suggestion", f"照出路改过的命令 {fixed!r} 被这一道自己拒了")
    # 进程层：红样本退 2、stderr 带 ISO 写法的出路，绿样本退 0 且 stderr 空
    hook = os.path.join(hook_dir, HOOK_NAME)
    scratch = tempfile.mkdtemp(prefix="find-timestamp-guard-selftest-")
    detections = os.path.join(scratch, "detections.jsonl")   # 检出记在自证自己的临时目录，不碰真的检出记录
    environment = dict({key: value for key, value in os.environ.items() if key != "SINGLEFS_WORK_ROOT_BREAK"}, AGENT_HOOK_DETECTIONS=detections)
    zoned_wanted = f"'{SAMPLE_DAY}T{SAMPLE_SECONDS}Z'"
    for label, command, want_code in (("带协调世界时", f"find . -newermt '{SAMPLE_DAY} {SAMPLE_SECONDS} {UTC_WORD}'", 2),
                                      ("相对时间", "find . -newermt yesterday", 2),
                                      ("ISO", f"find . -newermt {SAMPLE_DAY}T{SAMPLE_SECONDS}Z", 0), ("没有 -newer", "ls -l", 0)):
        checked += 1
        completed = subprocess.run(["bash", hook], input=json.dumps(bash_input(command)), capture_output=True, text=True, env=environment)
        if completed.returncode != want_code:
            fail("process", f"「{label}」应当退 {want_code}，实际退 {completed.returncode}，stderr：{completed.stderr!r}")
        elif want_code == 2:
            checked += 1
            if "→ 怎么办：" not in completed.stderr or "YYYY-MM-DDTHH:MM:SS" not in completed.stderr:
                fail("next-step", f"「{label}」的拒绝信息里应当写出路与认的 ISO 写法，实际 stderr：{completed.stderr!r}")
            elif label == "带协调世界时" and zoned_wanted not in completed.stderr:
                fail("suggestion", f"「{label}」的拒绝信息里应当给出换算好的 {zoned_wanted}，实际 stderr：{completed.stderr!r}")
        elif completed.stderr:
            fail("process", f"「{label}」放行时 stderr 应当是空的，实际：{completed.stderr!r}")

    def recorded():
        try:
            with open(detections, encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except OSError:
            return []

    def expect_recorded(cell, run, want, session):
        """run() 起一次（钩子入口或进程里的 judge_raw）交 (退出码, stderr)；要退 0、stderr 写一行、检出记录多 want 行（结局只记、会话号 session）。"""
        nonlocal checked
        checked += 1
        before = len(recorded())
        try:
            code, stderr = run()
        except Exception as error:   # 弄坏开关 exception-unrecorded：进程里照原样抛出
            code, stderr = f"抛出 {type(error).__name__}", ""
        added = recorded()[before:]
        if not (code == 0 and HOOK_NAME in stderr and len(added) == want and all(
                entry.get("outcome") == "只记" and entry.get("session_id") == session and entry.get("hook") == HOOK_NAME
                and all(str(finding).startswith(UNJUDGED_PREFIX) for finding in entry.get("findings") or []) for entry in added)):
            fail(cell, f"应当退 0、stderr 写一行、检出记录多 {want} 行（结局只记、会话号 {session}）；实际退 {code}，stderr {stderr.strip()[:200]!r}，检出多了 {len(added)} 行：{added!r:.300}")

    def through_entry(script, raw):
        def run():
            completed = subprocess.run(["bash", script], input=raw, capture_output=True, text=True, env=environment)
            return completed.returncode, completed.stderr
        return run

    def in_process(raw, replaced=None):
        """在进程里照钩子入口调 judge_raw；replaced 给了的把那几个函数临时换掉（造切不开、抛异常）。stderr 收进字符串。"""
        def run():
            import io
            saved = {name: globals()[name] for name in (replaced or {})}
            saved_stderr, saved_detections = sys.stderr, os.environ.get("AGENT_HOOK_DETECTIONS")
            globals().update(replaced or {})
            sys.stderr, os.environ["AGENT_HOOK_DETECTIONS"] = io.StringIO(), detections
            try:
                code = judge_raw(raw.encode("utf-8"), hook_dir)
                return code, sys.stderr.getvalue()
            finally:
                sys.stderr = saved_stderr
                globals().update(saved)
                if saved_detections is None:
                    os.environ.pop("AGENT_HOOK_DETECTIONS", None)
                else:
                    os.environ["AGENT_HOOK_DETECTIONS"] = saved_detections
        return run

    def raising(*arguments, **keywords):
        raise RuntimeError("自证造的异常")

    # 判不了的照「放行但必记检出」
    expect_recorded("input-recorded", through_entry(hook, "不是 JSON"), 1, "读不到")
    expect_recorded("input-recorded", through_entry(hook, "[]"), 1, "读不到")
    bare = os.path.join(scratch, "bare")   # 只拷入口与 lib_work_root.py：读不到 lib_shell_words.py
    os.makedirs(bare)
    for name in (HOOK_NAME, "lib_work_root.py"):
        shutil.copy(os.path.join(hook_dir, name), bare)
    expect_recorded("library-recorded", through_entry(os.path.join(bare, HOOK_NAME),
                                                      json.dumps(dict(bash_input("find . -newermt yesterday"), session_id="s-bare"))), 1, "s-bare")
    expect_recorded("command-shape-recorded", through_entry(hook, json.dumps({"tool_name": "Bash", "session_id": "s-shape", "tool_input": {"command": ["find", "-newermt"]}})),
                    1, "s-shape")
    expect_recorded("split-recorded", in_process(json.dumps(dict(bash_input("find . -newermt yesterday"), session_id="s-split")), {"timestamp_refusals": raising}), 1, "s-split")
    expect_recorded("exception-recorded", in_process(json.dumps(dict(bash_input("find . -newermt yesterday"), session_id="s-raise")), {"refusal_reason": raising}), 1, "s-raise")
    shutil.rmtree(scratch, ignore_errors=True)
    return failures, checked


# 弄坏开关只在自证里认：自证自带开关轮（--selftest 自己逐个设 FIND_TIMESTAMP_GUARD_BREAK 起 --selftest-cells），起这些轮的 --selftest 先设标记；
# 没有标记而变量非空（含 --selftest 本身：开关轮由它自己逐个设，外面设了没有用），读 hook 的 JSON 与一切参数之前就退 2、点名变量只在自证里认，不管值是什么。
SELFTEST_MARK_VARIABLE = "FIND_TIMESTAMP_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "find-timestamp-guard.sh --selftest 起的各轮"


def refuse_break_outside_selftest():
    value = os.environ.get(BREAK_VARIABLE, "")
    if not value or os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT:
        return None
    print(f"✗ {HOOK_NAME}：{BREAK_VARIABLE}={value} 是弄坏开关，只在自证（{HOOK_NAME} --selftest 起的各轮）里认，平常跑设了就拒，这一次没判", file=sys.stderr)
    print(f"→ 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红直接跑 bash .claude/hooks/{HOOK_NAME} --selftest（自证自己逐个打开弄坏开关，外面不用设）", file=sys.stderr)
    return 2


def switch_round_workers(hook_dir):
    """开关轮开几路：每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads，取法只在那一处）。取不到交 (None, 原因, 出路)。"""
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "scripts", "hosts"))
    try:
        import heavy_threads
    except ImportError as error:
        return None, f"载不进 scripts/hosts/heavy_threads.py：{error}", "它随仓走，被删了就从 git 找回来"
    try:
        return heavy_threads.default_heavy_threads(), None, None
    except heavy_threads.HeavyThreadDefaultError as error:
        return None, str(error), error.howto


def main():
    hook_dir = sys.argv[1]
    refused = refuse_break_outside_selftest()
    if refused is not None:
        return refused
    if sys.argv[2:] == ["--selftest-cells"]:
        failures, checked = selftest_cells(hook_dir)
        if failures:
            print(f"✗ 自证：{len(failures)} 处红（共判了 {checked} 个样本）")
            print("→ 怎么办：照上面每一格的下一步改，再单独跑 find-timestamp-guard.sh --selftest")
            return 1
        print(f"✓ 各格都对（判了 {checked} 个样本）")
        return 0
    if sys.argv[2:] == ["--selftest"]:
        os.environ[SELFTEST_MARK_VARIABLE] = SELFTEST_MARK_CONTENT   # 起的各轮与 selftest_cells 起的钩子子进程（环境原样继承）靠它认
        hook = os.path.join(hook_dir, HOOK_NAME)
        clean = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
        completed = subprocess.run(["bash", hook, "--selftest-cells"], capture_output=True, text=True, env=clean)
        if completed.returncode != 0:
            print("✗ 自证：不带弄坏开关时各格应当全绿，实际：")
            print(completed.stdout + completed.stderr)
            print("→ 怎么办：先让 --selftest-cells 不带弄坏开关跑绿，再回来跑本自证")
            return 1
        sample_line = completed.stdout.strip().splitlines()[-1]
        workers, problem, howto = switch_round_workers(hook_dir)
        if workers is None:
            print(f"✗ 自证：取不到每台默认重活线程数，开关轮没起：{problem}")
            print(f"→ 怎么办：{howto}")
            return 1
        jobs = max(1, min(workers, len(BREAK_TABLE)))

        def run_switch(pair):
            switch, _cell = pair
            return pair, subprocess.run(["bash", hook, "--selftest-cells"], capture_output=True, text=True, env=dict(clean, **{BREAK_VARIABLE: switch}))

        # 各开关轮互不依赖、各起一个子进程、各用自己的临时目录：照每台默认重活线程数并行，收回之后按开关表的次序判
        with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
            results = list(pool.map(run_switch, BREAK_TABLE))
        if len(results) != len(BREAK_TABLE):
            print(f"✗ 自证：派出去 {len(BREAK_TABLE)} 个开关轮、收回 {len(results)} 个")
            print("→ 怎么办：整轮作废，重跑 --selftest")
            return 1
        failed = 0
        for (switch, cell), broken_run in results:
            if broken_run.returncode == 0 or f"✗ 自证 {cell}：" not in broken_run.stdout:
                failed += 1
                print(f"✗ 自证：打开弄坏开关 {switch} 之后应当红在「{cell}」格，实际退 {broken_run.returncode}，输出：{broken_run.stdout!r}")
                print(f"→ 怎么办：{switch} 弄坏的那一处要真影响「{cell}」格的判定；两边对不上就改开关或改格")
        if failed:
            return 1
        print(f"✓ {HOOK_NAME} 自证：{sample_line.removeprefix('✓ ')}；{len(BREAK_TABLE)} 个弄坏开关各红在点名的格（开关轮并行 {jobs} 路）；"
              "出路喂这一道自己放行，喂别的 Bash 闸本次未跑（bash-command-detector.sh、project-code-guard.sh 只判命令位置与写法，不判 find 的参数）")
        return 0
    return judge_raw(sys.stdin.buffer.read(), hook_dir)


sys.exit(main())
PY
