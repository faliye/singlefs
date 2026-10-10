#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Write、Edit、MultiEdit：.claude/settings.json 里注册在 matcher "Write|Edit|MultiEdit" 上）：写文件之前的八道判定（一至五、七、八写在下面，六是语法检查、写在 --selftest 那一段），合在一个 hook 里；拒绝的同时把它记进检出记录，交主 agent 看。
#
# 一、整份覆盖未跟踪文件（只管 Write）：仓里已存在、又没进 git 的文件，拒绝用 Write 整份覆盖。
#    为什么：几个会话同时在一个仓里干活。2026-09-11 一个会话用 Write 新建 `research/prompts/e137-preregistration.md`，
#    另一个会话几分钟前刚写了同名的未提交文件，Write 回报「updated」就把它盖了——没进过 git，事后只能从对话记录里重放复原。
#    放行：目标不存在；目标已被 git 跟踪（盖了还能从 git 找回）；目标在仓外（scratchpad 之类）。
# 二、项目 subagent 的写范围（Write、Edit、MultiEdit）：项目定义的 subagent 写文件时，目标必须落在它登记的写范围里。
#    为什么：执行类 agent 越界写，不该等人翻 diff 才发现（records/2026-09-16-subagent拆分提案.md 第五节，用户 2026-09-17 定走项目 settings hook）。
#    输入里没有 agent_type（主 agent）放行；agent_type 没有项目定义（内置 agent）放行；有项目定义而表里没有它的写范围，拒绝；
#    目标路径规范化之后命中它的任一模式才放行。表在同目录的 agent-write-scope.tsv。拦不住 Bash 里的写，定义的「没做什么」照写。
#    表里以 $SINGLEFS_WORK_ROOT/ 起头的模式由同目录的 lib_work_root.py 展开成工作根（默认 ~/.local/share/singlefs/work）与过渡期旧根 /tmp/claude-<uid> 各一条，
#    根后面的 / 不省（同名前缀的兄弟目录不放行）；旧根那一条在跑的子 agent 与别的会话的子 agent 都交回之后由主 agent 撤（撤法在那份库的文件头）。
#    读不到那份库时，这样的模式按拒判，拒绝信息点名那份库。
# 三、撇号类角标（Write、Edit、MultiEdit）：Write 的 content、Edit 的 new_string、MultiEdit 每一处的 new_string 里
#    出现撇号类字符之一就拒绝；只在 old_string 里出现（删掉它的改动）放行，ASCII 单引号不判。主 agent 与所有子 agent 一样判，目标在不在仓里都判。
#    为什么：门禁 doc-text 只在收尾跑整轮门禁时判，2026-09-24 实验设计员照样把这类名字写进三份重跑登记、执行员又抄进源码。
#    字符集与 doc-text 同一份：../gate.d/lib-forbidden-notations.py 的 PRIME_MARKS，不在这里另抄；读不到就拒绝（不静默放行）。
# 四、先编后换（Write、Edit、MultiEdit）：experiment-runner（名单是 COMPILE_FIRST_AGENTS）写主工作区 crates/ 下的 .rs（改已有的、新建的都算）一律拒绝，
#    出路是在草稿目录的副本里改、经 research/scripts/compile-then-swap.py 编过再整份换进来。排在写范围那一道之前判，拒绝信息给的是这条出路。
#    implementation-writer、主 agent、别的 agent 不判；草稿目录里的副本与 crates/ 下不是 .rs 的（crates/mutations.tsv）不判。
#    为什么：执行员直接在主工作区改入库装置，改到一半编不过，别的会话带 --all-targets 的编译一起卡住
#    （records/2026-09-16-subagent拆分提案.md 第四十节那张表第 51 行）。Bash 里的同一类写法归 bash-command-detector.sh 的 ⑨。
# 五、描述里的钟点、时区词与时间戳（Write、Edit、MultiEdit）：目标在仓里、不在 lib-forbidden-notations.py 里钟点那一形态（clock-times）的排除前缀下，
#    写进去的内容命中那一形态的判据行之一就拒绝；仓外（草稿目录、报告目录）不判。主 agent 与所有子 agent 一样判。
#    判据与门禁 doc-text 同一份：../gate.d/lib-forbidden-notations.py，不在这里另抄；读不到就拒绝。Bash 里的写（cp、重定向）归 doc-text 在收尾时判。
#    形态有几族：时:分（日期后、时区词旁、x 通配、Z 后缀、不挨日期时区的时:分与区间）、ISO 时间戳、光秃的时区词，
#    与中文的「N 点 / N 时」和时段词（挨着日期或时区词的、时段词后面跟钟点的、不挨日期时区却带「前后 / 左右 / 许 / 半 / 钟」的）；
#    长得像钟点的冒号对靠库里的上下文规则放过，.sh、.py、.rs 里带「clock-times:allow <理由>」的那一行放过（目标路径交给库判后缀）；
#    逐个形态的判据与放行写在那份库的文档串里。
# 七、会话号（Write、Edit、MultiEdit）：ListAgents 列的会话名（目录名后面跟「-」与十六进制）写进仓里就拒绝，排除前缀、放行标记与仓外不判同五；
#    判据是同一份库里的 session-names 那一形态，与门禁 doc-text 的 session-names 格同一份。改过标题的会话认不出。
# 八、old_string 以空白收尾（Edit、MultiEdit）：目标不是 .md / .mdx（后缀不分大小写）时，Edit 的 old_string、MultiEdit 每一处的 old_string
#    以空格或制表符收尾就拒绝；主 agent 与所有子 agent 一样判，目标在不在仓里都判。.md / .mdx 目标与 Write 不判。
#    为什么：Edit 在 .md / .mdx 以外的目标上先去掉 new_string 每一行行尾的空格与制表符，钩子拿到的已是去过的 new_string，old_string 原样。
#    old_string 收尾的空白在文件里照样命中、被换掉，new_string 里对应的那一个却没了，紧跟着的字就粘上来。
#    2026-10-01 把钩子收到的输入落盘核过：.txt 目标 old_string、new_string 都以「declared = 」收尾，
#    钩子收到的 old_string 带着那个空格、new_string 没有，落盘成「declared =foo」；门禁注释精简那一批因此把
#    doc-decisions.sh 的 `python3 - "$DEC"` 改成 `python3 -"$DEC"`（records/2026-10-01-门禁注释精简.md「第 3 批」件一）。
#    看 old_string 而不看 new_string：new_string 收尾的空白到不了钩子，判它一次都拦不到；old_string 收尾的空白钩子看得到，粘连只在它那里发生。
#    .md / .mdx 不判：同一次核查里 .md、.mdx、.MD 目标的 new_string 行尾空白原样到了钩子、原样落盘，在那里以空白收尾的框选写得对。
#    后缀只认这两个：2026-10-03 拿 Edit 在草稿目录里各改一份 .md、.MD、.markdown、.mdown、.mkd、.txt（new_string 头一行以两个空格收尾），
#    落盘只有 .md、.MD 留着那两个空格，.markdown、.mdown、.mkd 与 .txt 一样去掉了，所以它们照判、不并进 TRAILING_BLANK_EXEMPT_SUFFIXES。
#    不判 old_string 中间行的行尾空白：new_string 那一行去掉的空白后面是换行，不会把两个词粘起来。
#    出路：把框选范围收到不以空白结尾（多框到下一个非空白字符，或止于行尾）；确实要删行尾空白的，用 Bash 跑 research/scripts/replace-once.py。
#
# 八道判定合在这一个 hook 里：一至五、七、八各占文件头一段，六写在 --selftest 那一行的说明里。
# 八道都判完再报：不成立的几道一次列全，每道带自己的出路；钟点与会话号逐处列出（行号与总数）。只有一处例外：
# 先编后换那一道拒了，写范围那一道不再报（目标是同一个，先编后换的出路已经写了范围外的交回主 agent）。
# 它们拒的是一次写，不停任务和脚本；拒绝时退出码 2、stderr 交给做这次写的模型，同时往检出记录
# （默认工作根下的 agent-hook-detections.jsonl，lib_work_root.py 的 default_detections_path；环境变量 AGENT_HOOK_DETECTIONS 可改）追加一行，
# 主 agent 的看门狗（research/scripts/agent-watch.py watch）读到就叫醒主 agent，由主 agent 判断怎么处理。
# 这份源码里撇号类字符一律写成转义、lib-forbidden-notations.py 里用码位拼（门禁 doc-text 也扫它们）；改带转义的那几行用 Bash 里的 python，反斜杠拿 chr(92) 拼：
# 2026-09-24 写这一道时实测，Write 的 content 里写的反斜杠 u 转义落盘成了字符本身，这道闸也会拒这样的写。
#
#   write-guard.sh             # 从 stdin 读 hook 的 JSON
#   write-guard.sh --selftest  # 走一遍各道判定的放行与拒绝；WRITE_GUARD_DISABLE_OVERWRITE=1、WRITE_GUARD_DISABLE_COMPILE_FIRST=1、
#                              # WRITE_GUARD_DISABLE_SCOPE=1、WRITE_GUARD_DISABLE_CLOCK_TIMES=1、WRITE_GUARD_DISABLE_SESSION_NAMES=1、WRITE_GUARD_DISABLE_SYNTAX=1 或
#                              # WRITE_GUARD_BREAK=trailing-blank-ignored（第八道放行以空白收尾的 old_string）时自检必须判红；
#                              # WRITE_GUARD_BREAK=stop-at-first（decide() 判到第一道不成立就交回、后面几道不判）时「八道判完再报:覆盖、越界与钟点一次列全」一格红；
#                              # LIB_FORBIDDEN_NOTATIONS_BREAK=first-hit-only（库的 all_hits 只交第一处）时「钟点:两处都报（行号与总数）」「会话号:两处都报（行号与总数）」两格红；
#                              # 第二道里工作根那一条模式的三样各一个弄坏开关，打开时点名的几项红：
#                              #   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（工作根退回旧根，lib_work_root.py）：「范围:工作根下的报告目录」「先编后换:experiment-runner 改草稿目录里的副本放行」
#                              #     「stdin:实现员写工作根下的报告」「stdin:七次拒绝都落进检出记录」（多出一次拒绝）
#                              #   SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown（不认旧根）：「范围:过渡期旧根下的报告目录」
#                              #   WRITE_GUARD_BREAK=scope-root-unanchored（展开时根后面不带 /）：「范围:工作根同名前缀的兄弟目录」
#                              # 第六道：写进 .claude/hooks/、.claude/gate.d/、.claude/scripts/、research/scripts/ 下的 .sh 先过 bash -n、.py 先过 ast 解析，
#                              # 语法错的拒（带语法错误的钩子一上线，每一次 Bash 调用都收到它的报错）；
#                              # LIB_FORBIDDEN_NOTATIONS_BREAK=<判据行键>（钟点的 date-zone-hour、date-zone-period、period-hour、hour-suffix、twelve-hour、
#                              # bare-clock、iso-timestamp、zone-word，键写在 ../gate.d/lib-forbidden-notations.py）关掉一条判据行时，点名那一行的用例必须判红；
#                              # LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=clock-times 时靠上下文放行的用例、LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=clock-times 时
#                              # 带放行标记的用例必须判红
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
# 写成 `python3 - <<PY` 时程序占了标准输入，JSON 读不到，判定一律放行——2026-09-17 写范围闸实测撞到的。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import importlib.util, json, os, re, shutil, subprocess, sys, tempfile
from datetime import datetime, timezone

try:   # 工作根与过渡期旧根的唯一定义：同目录的 lib_work_root.py
    sys.dont_write_bytecode = True
    sys.path.insert(0, sys.argv[1]); import lib_work_root as work_roots
    DEFAULT_DETECTIONS, work_roots_error = work_roots.default_detections_path(), None
except Exception as error:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, DEFAULT_DETECTIONS, work_roots_error = None, None, error

def expanded_scope_patterns(patterns):
    """写范围表里的模式：$SINGLEFS_WORK_ROOT/ 起头的展开成工作根与过渡期旧根各一条；读不到 lib_work_root.py 时这样的模式丢掉（按拒判）。"""
    anchored = os.environ.get("WRITE_GUARD_BREAK") != "scope-root-unanchored"
    expanded = []
    for pattern in patterns:
        if pattern.startswith("$"):
            expanded += work_roots.expand_scope_pattern(pattern, anchored) if work_roots else []
        else:
            expanded.append(pattern)
    return expanded

def glob_to_regex(pattern):
    parts, index = [], 0
    while index < len(pattern):
        if pattern.startswith("**", index):
            parts.append(".*"); index += 2
        elif pattern[index] == "*":
            parts.append("[^/]*"); index += 1
        else:
            parts.append(re.escape(pattern[index])); index += 1
    return re.compile("^" + "".join(parts) + "$")

def load_scopes(table_path):
    scopes = {}
    for line in open(table_path, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) >= 2 and fields[0].strip() and fields[1].strip():
            scopes.setdefault(fields[0].strip(), []).append(fields[1].strip())
    return scopes

def absolute_target(hook_input, project_root):
    file_path = (hook_input.get("tool_input") or {}).get("file_path") or ""
    if not file_path:
        return ""
    return os.path.normpath(file_path if os.path.isabs(file_path) else os.path.join(project_root, file_path))

def decide_overwrite(hook_input, project_root):
    """一、Write 整份覆盖未跟踪的已有文件。"""
    if hook_input.get("tool_name") != "Write" or os.environ.get("WRITE_GUARD_DISABLE_OVERWRITE") == "1":
        return 0, None
    target = absolute_target(hook_input, project_root)
    if not target or not os.path.exists(target):
        return 0, None
    top = subprocess.run(["git", "-C", os.path.dirname(target), "rev-parse", "--show-toplevel"], capture_output=True, text=True)
    if top.returncode != 0:
        return 0, None
    tracked = subprocess.run(["git", "-C", top.stdout.strip(), "ls-files", "--error-unmatch", "--", target], capture_output=True, text=True)
    if tracked.returncode == 0:
        return 0, None
    return 2, (f"✗ 拒绝用 Write 整份覆盖 {target}：它已存在而且没进 git，可能是别的会话还没提交的文件，盖掉就找不回来。\n"
               "→ 新建文件：换一个没被占的名字，用排他方式写（set -o noclobber; cat > 路径 <<'EOF'，或 python open(路径, 'x')）。\n"
               "→ 确认是这一轮自己的文件：先 Read 看一眼，再用 Edit 改；整份重写就先 git add 它。")

def decide_scope(hook_input, project_root, table_path):
    """二、项目 subagent 的写范围。"""
    agent_type = hook_input.get("agent_type")
    if not agent_type:
        return 0, None
    if not os.path.isfile(os.path.join(project_root, ".claude", "agents", f"{agent_type}.md")):
        return 0, None
    if os.environ.get("WRITE_GUARD_DISABLE_SCOPE") == "1":
        return 0, None
    patterns = load_scopes(table_path).get(agent_type, [])
    file_path = (hook_input.get("tool_input") or {}).get("file_path") or ""
    if not patterns:
        return 2, (f"✗ {agent_type} 在写范围表里没有登记，按拒绝处理：{file_path}\n"
                   f"→ 怎么办：在 .claude/hooks/agent-write-scope.tsv 按它定义的「写范围」一节补上路径模式；补之前这件活交回主 agent。")
    absolute = absolute_target(hook_input, project_root)
    root = os.path.normpath(project_root)
    relative = os.path.relpath(absolute, root) if absolute == root or absolute.startswith(root + os.sep) else None
    for pattern in expanded_scope_patterns(patterns):
        target = absolute if pattern.startswith("/") else relative
        if target is not None and glob_to_regex(pattern).match(target):
            return 0, None
    unexpanded = (f"\n→ 读不到工作根的定义 .claude/hooks/lib_work_root.py（{work_roots_error!r}），表里 $SINGLEFS_WORK_ROOT/ 起头的模式按拒判："
                  "先 git status 看那份库是被删了还是改坏了，用 Bash 恢复它") if work_roots is None and any(p.startswith("$") for p in patterns) else ""
    return 2, (f"✗ {agent_type} 的写范围不含 {absolute}\n"
               f"→ 它的写范围：{'、'.join(patterns)}（.claude/hooks/agent-write-scope.tsv；$SINGLEFS_WORK_ROOT 是工作根 ~/.local/share/singlefs/work，过渡期另认旧根 /tmp/claude-<uid>）。\n"
               f"→ 怎么办：范围外的改动不自己做，写进报告交回主 agent，由主 agent 改或派该写的 agent。{unexpanded}")

# 四、先编后换：这几个项目 subagent 不直接写主工作区 crates/ 下的 .rs，出路是 research/scripts/compile-then-swap.py。
# Bash 里的同一类写法由 bash-command-detector.sh 的 ⑨ 判，agent 名单与路径判据两边各写一份、改的时候一起改。
COMPILE_FIRST_AGENTS = ("experiment-runner",)
COMPILE_THEN_SWAP_SCRIPT = "research/scripts/compile-then-swap.py"

def decide_compile_first(hook_input, project_root):
    """四、登记在 COMPILE_FIRST_AGENTS 里的 agent 写主工作区 crates/ 下的 .rs。"""
    agent_type = hook_input.get("agent_type")
    if agent_type not in COMPILE_FIRST_AGENTS or os.environ.get("WRITE_GUARD_DISABLE_COMPILE_FIRST") == "1":
        return 0, None
    absolute = absolute_target(hook_input, project_root)
    root = os.path.normpath(project_root)
    if not absolute or not absolute.endswith(".rs") or not absolute.startswith(os.path.join(root, "crates") + os.sep):
        return 0, None
    relative = os.path.relpath(absolute, root)
    return 2, (f"✗ {agent_type} 不直接写主工作区的 {relative}：入库装置先在草稿目录的副本里改，编过再整份换进主工作区，"
               "主工作区里任何时候都只放编得过的版本。\n"
               f"→ 怎么办：cp {relative} <草稿目录>/ 拷一份（新建的装置直接在草稿目录里建），在草稿目录那一份上用 Edit / Write 改；"
               f"改完跑 bash research/scripts/capped.sh <线程上限> python3 {COMPILE_THEN_SWAP_SCRIPT} <草稿目录>/{os.path.basename(relative)} "
               f"{relative} --scratch <草稿目录>，它编过才整份换进来，编不过一个字节不写。\n"
               "→ 目标不是 crates/<crate>/src/bin/ 下的装置（crate 的库、测试）的，不归你改，写进报告交回主 agent。")

def notations_library_path(hook_dir):
    return os.path.join(os.path.dirname(hook_dir), "gate.d", "lib-forbidden-notations.py")

def load_prime_marks(library_path):
    """撇号类字符集，与门禁 doc-text 同一份；读不到或读出来是空的就抛异常，由调用方按拒绝处理。"""
    spec = importlib.util.spec_from_file_location("lib_forbidden_notations", library_path)
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    prime_marks = library.PRIME_MARKS
    if not isinstance(prime_marks, str) or not prime_marks:
        raise ValueError(f"PRIME_MARKS 是 {prime_marks!r}，不是非空字符串")
    return prime_marks

def written_texts(hook_input):
    """这一次写要落进文件的文字：Write 的 content、Edit 的 new_string、MultiEdit 每一处的 new_string。old_string 不算。"""
    tool_name = hook_input.get("tool_name")
    tool_input = hook_input.get("tool_input") or {}
    if tool_name == "Write":
        return [str(tool_input.get("content") or "")]
    if tool_name == "Edit":
        return [str(tool_input.get("new_string") or "")]
    if tool_name == "MultiEdit":
        return [str((edit or {}).get("new_string") or "") for edit in tool_input.get("edits") or []]
    return []

def decide_prime_marks(hook_input, library_path):
    """三、写进去的内容里有撇号类角标。"""
    texts = [text for text in written_texts(hook_input) if text]
    if not texts:
        return 0, None
    try:
        prime_marks = load_prime_marks(library_path)
    except Exception as error:
        return 2, (f"✗ 读不到撇号类字符集 {library_path}（{type(error).__name__}：{error}），这次写没法判，按拒绝处理。\n"
                   "→ 怎么办：那一份是门禁 doc-text 与这道 hook 共用的唯一定义，先 git status 看它是被删了还是被改坏了，"
                   "用 Bash 恢复它（不在 hook 里另抄一份）；恢复之前 Write / Edit 都会被拒。")
    occurrences = sum(text.count(mark) for text in texts for mark in prime_marks)
    if not occurrences:
        return 0, None
    offending_text = next(text for text in texts if any(mark in text for mark in prime_marks))
    position, mark = min((offending_text.find(mark), mark) for mark in prime_marks if mark in offending_text)
    line_start = offending_text.rfind("\n", 0, position) + 1
    line_end = offending_text.find("\n", position)
    line = offending_text[line_start:line_end if line_end != -1 else len(offending_text)]
    column = position - line_start
    snippet = line[max(0, column - 20):column + 10].strip()
    file_path = (hook_input.get("tool_input") or {}).get("file_path") or ""
    return 2, (f"✗ 写进去的内容里有撇号类角标 {mark}（U+{ord(mark):04X}）在「…{snippet}…」"
               f"（{file_path}，这次写的内容里共 {occurrences} 处）\n"
               "→ 怎么办：给这个变体起一个新名字——在它那一族里取下一个没用过的号，或起一个短的描述性名字；"
               "做法见 .claude/rules/path-moves.md「变体起新名字，不用角标」\n"
               "→ 引用户原话或原文时也不例外：把那个字符改写成文字（写成「B2（加撇号）」这类）")

TRAILING_BLANKS = (" ", "\t")
TRAILING_BLANK_EXEMPT_SUFFIXES = (".md", ".mdx")

def decide_trailing_blank(hook_input):
    """八、目标不是 .md / .mdx 时，Edit 的 old_string、MultiEdit 每一处的 old_string 以空格或制表符收尾。"""
    if os.environ.get("WRITE_GUARD_BREAK") == "trailing-blank-ignored":
        return 0, None
    tool_name = hook_input.get("tool_name")
    if tool_name not in ("Edit", "MultiEdit"):
        return 0, None
    tool_input = hook_input.get("tool_input") or {}
    if str(tool_input.get("file_path") or "").lower().endswith(TRAILING_BLANK_EXEMPT_SUFFIXES):
        return 0, None
    edits = (tool_input.get("edits") or []) if tool_name == "MultiEdit" else [tool_input]
    texts = [str((edit or {}).get("old_string") or "") for edit in edits]
    offending = [index for index, text in enumerate(texts) if text.endswith(TRAILING_BLANKS)]
    if not offending:
        return 0, None
    text = texts[offending[0]]
    last_line = text[text.rfind("\n") + 1:]
    blank = last_line[len(last_line.rstrip(" \t")):]
    shown = last_line.rstrip(" \t")[-30:] + blank.replace(" ", "␠").replace("\t", "⇥")
    where = f"第 {offending[0] + 1} 处（共 {len(offending)} 处以空白收尾）的 " if tool_name == "MultiEdit" else ""
    file_path = tool_input.get("file_path") or ""
    return 2, (f"✗ {tool_name} {where}old_string 以空白收尾：「…{shown}」（{file_path}；␠ 是空格，⇥ 是制表符）\n"
               "  .md / .mdx 以外的目标，Edit 先去掉 new_string 每一行行尾的空格与制表符再施加：old_string 收尾的空白照样被换掉，"
               "new_string 里对应的那一个却没了，文件里紧跟着的字就粘上来（`python3 - \"$DEC\"` 落成 `python3 -\"$DEC\"`）。\n"
               "→ 怎么办：把 old_string 与 new_string 的框选范围收到不以空白结尾：两边一起往后多框到下一个非空白字符，或止于行尾（框到换行之前）。\n"
               "→ 确实要删行尾空白的：用 Bash 跑 python3 research/scripts/replace-once.py <文件> <旧串> <新串>（旧串恰好命中一次才写）。")

LIBRARY_UNREADABLE_PREFIX = "✗ 读不到"
LISTED_HITS_LIMIT = 20

def written_text_places(hook_input, project_root):
    """written_texts 的每一段，连同「它的第 n 行在哪」的说法：[(文字, 行号 → 说法)]。Write 的行号就是文件的行号；
    Edit 的 old_string 在现文件里找得到时折算成施加之后文件里的行号，找不到写 new_string 的第几行；MultiEdit 写第几处的第几行。"""
    tool_name = hook_input.get("tool_name")
    tool_input = hook_input.get("tool_input") or {}
    texts = written_texts(hook_input)
    if tool_name == "Edit":
        base, old_string = None, str(tool_input.get("old_string") or "")
        try:
            current = open(absolute_target(hook_input, project_root), encoding="utf-8", errors="replace").read()
            index = current.find(old_string) if old_string and not tool_input.get("replace_all") else -1
            base = current[:index].count("\n") if index >= 0 else None
        except (OSError, ValueError):
            pass
        return [(texts[0], (lambda number: f"第 {base + number} 行") if base is not None else (lambda number: f"new_string 第 {number} 行"))]
    if tool_name == "MultiEdit":
        return [(text, lambda number, place=place: f"第 {place} 处 new_string 第 {number} 行") for place, text in enumerate(texts, 1)]
    return [(text, lambda number: f"第 {number} 行") for text in texts]

def library_hits_in_written_texts(hook_input, project_root, library_path, form_key, form_name):
    """写进仓里的内容在共用库那一形态下的每一处：(拒绝信息, None, []) 读不到库；(None, None, []) 没命中或不判；
    (None, 相对路径, [(在哪, 判据行名字, 命中的串)]) 命中（库的 all_hits，按出现的次序）。"""
    places = [(text, where) for text, where in written_text_places(hook_input, project_root) if text]
    absolute = absolute_target(hook_input, project_root)
    root = os.path.normpath(project_root)
    if not places or not absolute or os.path.commonpath([absolute, root]) != root:
        return None, None, []
    try:
        spec = importlib.util.spec_from_file_location("lib_forbidden_notations", library_path)
        library = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(library)
        relative = os.path.relpath(absolute, root)
        if library.is_excluded(form_key, relative):
            return None, None, []
        found = [(where(number), name, matched) for text, where in places for number, _, name, matched in library.all_hits(form_key, text, relative)]
    except Exception as error:
        return (f"{LIBRARY_UNREADABLE_PREFIX}{form_name}判据 {library_path}（{type(error).__name__}：{error}），这次写没法判，按拒绝处理。\n"
                "→ 怎么办：那一份是门禁 doc-text 与这道 hook 共用的唯一定义，先 git status 看它是被删了还是被改坏了，"
                "用 Bash 恢复它（不在 hook 里另抄一份）；恢复之前往仓里的 Write / Edit 都会被拒。"), None, []
    return None, relative, found

def listed_hits(found, with_shape):
    """逐处一行：「  第 n 行（判据行名字）：「命中的串」」；超过 LISTED_HITS_LIMIT 处的只报另有几处。"""
    shown = [f"  {where}（{shape}）：「{matched}」" if with_shape else f"  {where}：「{matched}」" for where, shape, matched in found[:LISTED_HITS_LIMIT]]
    if len(found) > LISTED_HITS_LIMIT:
        shown.append(f"  （另有 {len(found) - LISTED_HITS_LIMIT} 处没列，改完上面这些再写一次会列出来）")
    return "\n".join(shown) + "\n"

def decide_clock_times(hook_input, project_root, library_path):
    """五、写进仓里的内容带描述性的钟点。"""
    if os.environ.get("WRITE_GUARD_DISABLE_CLOCK_TIMES") == "1":
        return 0, None
    unreadable, relative, found = library_hits_in_written_texts(hook_input, project_root, library_path, "clock-times", "钟点")
    if unreadable:
        return 2, unreadable
    if not found:
        return 0, None
    return 2, (f"✗ 写进 {relative} 的内容在描述里写了钟点，共 {len(found)} 处：\n" + listed_hits(found, True) +
               "→ 怎么办：时间只写到日期，把钟点（时:分、「N 点 / N 时」、上午下午这类时段词）连同挨着它的时区词、「前后」一起删掉；"
               "光秃的时区词、ISO 时间戳与 date 的原样输出也删，只写日期。\n"
               "→ .sh、.py、.rs 里代码真要用这些字面（TZ 环境变量、解析外部文本的正则、造时间戳的自检输入），在那一行写「# clock-times:allow <理由>」（Rust 写「//」）；"
               "命中的其实不是钟点（产物字段、切片、编号对），改 .claude/gate.d/lib-forbidden-notations.py 的上下文规则\n"
               "→ 钟点确实是被测输入的整份文件（hook 自检、门禁样本），登记进 .claude/gate.d/lib-forbidden-notations.py 里钟点那一形态的 excluded_prefixes 并写明理由")

def decide_session_names(hook_input, project_root, library_path):
    """七、写进仓里的内容带会话号（ListAgents 列的会话名）。"""
    if os.environ.get("WRITE_GUARD_DISABLE_SESSION_NAMES") == "1":
        return 0, None
    unreadable, relative, found = library_hits_in_written_texts(hook_input, project_root, library_path, "session-names", "会话号")
    if unreadable:
        return 2, unreadable
    if not found:
        return 0, None
    return 2, (f"✗ 写进 {relative} 的内容写了会话号，共 {len(found)} 处：\n" + listed_hits(found, False) +
               "→ 怎么办：会话号不写进仓：标签式的「（会话 <号>，<日期>）」只留日期，当主语用的写成「别的会话」「SOP 会话」这类不带号的说法，事情与日期照留；"
               "改过标题的会话（「会话「<标题>」」）同样不写标题。\n"
               "→ .sh、.py、.rs 里代码真要用这些字面（解析 ListAgents 输出的正则、造会话号的自检输入），在那一行写「# session-names:allow <理由>」（Rust 写「//」）")

SYNTAX_CHECKED_DIRECTORIES = (".claude/hooks", ".claude/gate.d", ".claude/scripts", "research/scripts")
PYTHON_SYNTAX_CHECK = "import ast, sys; ast.parse(open(sys.argv[1], encoding='utf-8').read(), sys.argv[1])"

def edited_content(hook_input, target):
    """Write 给的整份内容，或 Edit / MultiEdit 施加到现文件之后的内容；施加不了（文件不在、旧串找不到）返回 None，那一步 Edit 自己会拒。"""
    tool_name = hook_input.get("tool_name")
    tool_input = hook_input.get("tool_input") or {}
    if tool_name == "Write":
        return tool_input.get("content")
    if not os.path.isfile(target):
        return None
    try:
        text = open(target, encoding="utf-8", errors="replace").read()
    except OSError:
        return None
    edits = tool_input.get("edits") if tool_name == "MultiEdit" else [tool_input]
    for edit in edits or []:
        old_string = edit.get("old_string") or ""
        if not old_string or old_string not in text:
            return None
        new_string = edit.get("new_string") or ""
        text = text.replace(old_string, new_string) if edit.get("replace_all") else text.replace(old_string, new_string, 1)
    return text

def decide_shell_syntax(hook_input, project_root):
    """六、钩子、门禁、项目脚本与研究脚本目录下的 .sh 与 .py：写进去的内容先过语法，过不了就拒。"""
    if os.environ.get("WRITE_GUARD_DISABLE_SYNTAX") == "1" or hook_input.get("tool_name") not in ("Write", "Edit", "MultiEdit"):
        return 0, None
    target = absolute_target(hook_input, project_root)
    if not target:
        return 0, None
    relative = os.path.relpath(target, project_root)
    if relative.startswith("..") or not any(relative.startswith(directory + "/") for directory in SYNTAX_CHECKED_DIRECTORIES):
        return 0, None
    if target.endswith(".sh"):
        checker, checker_name = ["bash", "-n"], "bash -n"
    elif target.endswith(".py"):
        checker, checker_name = [sys.executable, "-c", PYTHON_SYNTAX_CHECK], "python3 的 ast 解析"
    else:
        return 0, None
    content = edited_content(hook_input, target)
    if content is None:
        return 0, None
    handle = tempfile.NamedTemporaryFile("w", suffix=os.path.splitext(target)[1], delete=False, encoding="utf-8")
    handle.write(content)
    handle.close()
    try:
        result = subprocess.run(checker + [handle.name], capture_output=True, text=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return 0, None
    finally:
        os.unlink(handle.name)
    if result.returncode == 0:
        return 0, None
    detail = (result.stderr or result.stdout).strip().replace(handle.name, relative)[-600:]
    return 2, (f"✗ 写进 {relative} 的内容过不了 {checker_name}：\n  {detail}\n"
               f"→ 怎么办：先在草稿目录里改到 `{checker_name}` 过，再写进来；钩子、门禁与脚本带着语法错误上线，之后每一次调用都会收到它的报错。")

def decide(hook_input, project_root, table_path, prime_marks_library, clock_times_library=None):
    """返回 (0, None, None) 放行；(2, 说明, 哪几道（「、」连）) 拒绝。八道都判完，不成立的按下面的次序一次列全；
    先编后换拒了不再判写范围，钟点那一道读不到判据时会话号那一道不再报同一句。弄坏开关 WRITE_GUARD_BREAK=stop-at-first 判到第一道不成立就交回。"""
    if clock_times_library is None:
        clock_times_library = prime_marks_library
    judges = (("整份覆盖未跟踪文件", lambda: decide_overwrite(hook_input, project_root)),
              ("绕过先编后换", lambda: decide_compile_first(hook_input, project_root)),
              ("old_string 以空白收尾", lambda: decide_trailing_blank(hook_input)),
              ("写进语法错误", lambda: decide_shell_syntax(hook_input, project_root)),
              ("越出写范围", lambda: decide_scope(hook_input, project_root, table_path)),
              ("写进撇号类角标", lambda: decide_prime_marks(hook_input, prime_marks_library)),
              ("描述里写了钟点", lambda: decide_clock_times(hook_input, project_root, clock_times_library)),
              ("写了会话号", lambda: decide_session_names(hook_input, project_root, clock_times_library)))
    refused = []   # [(哪一道, 说明)]
    for label, judge in judges:
        already = {name for name, _ in refused}
        if label == "越出写范围" and "绕过先编后换" in already:
            continue
        if (label == "写了会话号" and "描述里写了钟点" in already
                and dict(refused)["描述里写了钟点"].startswith(LIBRARY_UNREADABLE_PREFIX)):
            continue
        code, message = judge()
        if code:
            refused.append((label, message))
            if os.environ.get("WRITE_GUARD_BREAK") == "stop-at-first":
                break
    if not refused:
        return 0, None, None
    labels = "、".join(label for label, _ in refused)
    if len(refused) == 1:
        return 2, refused[0][1], labels
    blocks = [f"── 第 {number} 道：{label} ──\n{message}" for number, (label, message) in enumerate(refused, 1)]
    return 2, (f"✗ 这一次写有 {len(refused)} 道判定不成立（{labels}）：八道都判完了，下面一次列全，逐道照各自的「→」改完再写。\n"
               + "\n".join(blocks)), labels

def record_detection(hook_input, finding, detections_path):
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": f"{hook_input.get('tool_name')} {(hook_input.get('tool_input') or {}).get('file_path', '')}"[:500],
        "findings": [f"写被拒：{finding}"],
    }
    if not detections_path:
        return
    try:
        os.makedirs(os.path.dirname(detections_path), exist_ok=True)
        with open(detections_path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass

def selftest_base_directory(table_path):
    """自检的现场放在哪。不能落进写范围表里任何一条仓外绝对路径模式（落进去，「越界」那几例会被那条模式放行），
    也不能落在某个 git 仓里（「仓外文件」那一例要真在仓外）。TMPDIR 指到哪都不影响判定：依次试几处，取第一处两条都满足的。"""
    absolute_patterns = [glob_to_regex(pattern) for patterns in load_scopes(table_path).values()
                         for pattern in expanded_scope_patterns(patterns) if pattern.startswith("/")]
    for candidate in (tempfile.gettempdir(), "/tmp", "/var/tmp", "/dev/shm"):
        if not os.path.isdir(candidate) or not os.access(candidate, os.W_OK):
            continue
        probe = os.path.join(candidate, "write-guard-selftest-probe", "crates", "a.rs")
        if any(pattern.match(os.path.normpath(probe)) for pattern in absolute_patterns):
            continue
        if subprocess.run(["git", "-C", candidate, "rev-parse", "--show-toplevel"], capture_output=True, text=True).returncode == 0:
            continue
        return candidate
    return None

def selftest(hook_dir):
    if work_roots is None:
        print(f"  ✗ 自检读不到工作根的定义 {os.path.join(hook_dir, 'lib_work_root.py')}：{work_roots_error!r}")
        print("    → 怎么办：恢复那份库（工作根与过渡期旧根只在那里定义），SINGLEFS_WORK_ROOT_BREAK 只写它认的值，再跑 --selftest")
        return 1
    # 工作根下的样本路径不看弄坏开关（configured_work_root）；旧根照库里的写法
    new_root, old_root = work_roots.configured_work_root(), work_roots.legacy_root()
    base = selftest_base_directory(os.path.join(hook_dir, "agent-write-scope.tsv"))
    if base is None:
        print("  ✗ 自检找不到一处能放现场的临时目录：TMPDIR、/tmp、/var/tmp、/dev/shm 要么落进写范围表的仓外模式、要么在 git 仓里、要么写不进去")
        print("    → 怎么办：给一个不在 git 仓里、也不在 agent-write-scope.tsv 任何一条绝对路径模式底下的可写目录当 TMPDIR 再跑")
        return 1
    work = tempfile.mkdtemp(dir=base)
    cases = []
    library = notations_library_path(hook_dir)
    try:
        subprocess.run(["git", "init", "-q", work], check=True)
        subprocess.run(["git", "-C", work, "config", "user.email", "t@t"], check=True)
        subprocess.run(["git", "-C", work, "config", "user.name", "t"], check=True)
        os.makedirs(os.path.join(work, ".claude", "agents"))
        os.makedirs(os.path.join(work, ".claude", "kb"))
        for name in ("kb-scribe", "implementation-writer", "experiment-runner", "unscoped-writer"):
            open(os.path.join(work, ".claude", "agents", f"{name}.md"), "w").write("---\n")
        open(os.path.join(work, "tracked.md"), "w").write("a\n")
        subprocess.run(["git", "-C", work, "add", "tracked.md"], check=True)
        subprocess.run(["git", "-C", work, "commit", "-qm", "t"], check=True)
        open(os.path.join(work, "untracked.md"), "w").write("b\n")
        open(os.path.join(work, ".claude", "kb", "untracked-kb.md"), "w").write("c\n")
        outside = tempfile.NamedTemporaryFile(delete=False, dir=base)
        outside.close()
        table = os.path.join(work, "scope.tsv")
        shutil.copyfile(os.path.join(hook_dir, "agent-write-scope.tsv"), table)
        def case(label, tool_name, agent, path, want):
            hook_input = {"tool_name": tool_name, "tool_input": {"file_path": path}}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library)[0])
        def finding_case(label, tool_name, agent, path, want):
            hook_input = {"tool_name": tool_name, "tool_input": {"file_path": path}}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library)[2])
        os.makedirs(os.path.join(work, "research", "scripts"))
        open(os.path.join(work, "research", "scripts", "y.sh"), "w").write("#!/usr/bin/env bash\necho ok\n")
        def content_case(label, tool_name, path, tool_input, want):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=path)}
            return (label, want, decide(hook_input, work, table, library)[0])
        def prime_case(label, tool_name, agent, tool_input, want, library_path=library):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=f"{work}/crates/a.rs")}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library_path)[0])
        cases += [
            # 六、语法：钩子、门禁、项目脚本、研究脚本目录下的 .sh / .py
            content_case("语法:hooks 下写进语法错误的 .sh", "Write", f"{work}/.claude/hooks/x.sh", {"content": "if [ ; then\n  echo\n"}, 2),
            content_case("语法:hooks 下语法对的 .sh", "Write", f"{work}/.claude/hooks/x.sh", {"content": "#!/usr/bin/env bash\necho ok\n"}, 0),
            content_case("语法:Edit 把研究脚本改出语法错误", "Edit", f"{work}/research/scripts/y.sh", {"old_string": "echo ok", "new_string": "if [ ; then"}, 2),
            content_case("语法:Edit 研究脚本合法改动", "Edit", f"{work}/research/scripts/y.sh", {"old_string": "echo ok", "new_string": "echo fine"}, 0),
            content_case("语法:门禁目录下语法错误的 .py", "Write", f"{work}/.claude/gate.d/z.py", {"content": "def (:\n"}, 2),
            content_case("语法:门禁目录下语法对的 .py", "Write", f"{work}/.claude/gate.d/z.py", {"content": "print(1)\n"}, 0),
            content_case("语法:别处的 .sh 不判", "Write", f"{work}/crates/run.sh", {"content": "if [ ; then\n"}, 0),
            # 一、整份覆盖未跟踪文件
            case("覆盖:未跟踪的已有文件用 Write", "Write", None, f"{work}/untracked.md", 2),
            case("覆盖:未跟踪的已有文件用 Edit", "Edit", None, f"{work}/untracked.md", 0),
            case("覆盖:已跟踪文件", "Write", None, f"{work}/tracked.md", 0),
            case("覆盖:不存在的文件", "Write", None, f"{work}/missing.md", 0),
            case("覆盖:仓外文件", "Write", None, outside.name, 0),
            # 二、写范围
            case("范围:主 agent", "Edit", None, f"{work}/crates/a.rs", 0),
            case("范围:内置 agent", "Edit", "general-purpose", f"{work}/crates/a.rs", 0),
            case("范围:kb-scribe 范围内", "Edit", "kb-scribe", f"{work}/.claude/kb/decisions/01-x.md", 0),
            case("范围:kb-scribe 越界 crates", "Edit", "kb-scribe", f"{work}/crates/a.rs", 2),
            case("范围:kb-scribe 越界 records", "Edit", "kb-scribe", f"{work}/records/x.md", 2),
            case("范围:implementation-writer 范围内", "Edit", "implementation-writer", f"{work}/crates/singlefs-core/src/a.rs", 0),
            case("范围:.. 绕路", "Edit", "implementation-writer", f"{work}/crates/../.claude/kb/x.md", 2),
            case("范围:implementation-writer 越界 research", "Edit", "implementation-writer", f"{work}/research/x.md", 2),
            case("范围:工作根下的报告目录", "Write", "implementation-writer", f"{new_root}/agents/implementation-writer/report.md", 0),
            case("范围:过渡期旧根下的报告目录", "Write", "implementation-writer", f"{old_root}/agents/implementation-writer/report.md", 0),
            case("范围:工作根同名前缀的兄弟目录", "Write", "implementation-writer", f"{new_root}-other/report.md", 2),
            case("范围:仓外别处", "Edit", "implementation-writer", "/etc/passwd", 2),
            case("范围:experiment-runner 登记", "Edit", "experiment-runner", f"{work}/research/prompts/e12-preregistration.md", 0),
            case("范围:experiment-runner 三方正文", "Edit", "experiment-runner", f"{work}/research/prompts/_e12-body.md", 2),
            case("范围:experiment-runner replay.sh", "Edit", "experiment-runner", f"{work}/research/scripts/replay.sh", 0),
            case("范围:experiment-runner mutate.sh", "Edit", "experiment-runner", f"{work}/research/scripts/mutate.sh", 2),
            case("范围:未登记的项目 agent", "Edit", "unscoped-writer", f"{work}/anything.md", 2),
            case("两道都中时先报覆盖", "Write", "kb-scribe", f"{work}/untracked.md", 2),
            # 四、实验执行员对主工作区 crates/ 下 .rs 的写：拒的那一道要是「绕过先编后换」，不是写范围那一道（出路不同）
            finding_case("先编后换:experiment-runner Edit 主工作区的入库装置", "Edit", "experiment-runner",
                         f"{work}/crates/singlefs-checker-tier/src/bin/e161_x.rs", "绕过先编后换"),
            finding_case("先编后换:experiment-runner Write 新建入库装置", "Write", "experiment-runner",
                         f"{work}/crates/singlefs-checker-tier/src/bin/e999_new.rs", "绕过先编后换"),
            finding_case("先编后换:experiment-runner 改 crates 里 bin 以外的 .rs", "Edit", "experiment-runner",
                         f"{work}/crates/singlefs-core/src/lib.rs", "绕过先编后换"),
            finding_case("先编后换:.. 绕路进 crates 也算", "Edit", "experiment-runner",
                         f"{work}/research/../crates/singlefs-checker-tier/src/bin/e161_x.rs", "绕过先编后换"),
            finding_case("先编后换:implementation-writer 同一处放行", "Edit", "implementation-writer",
                         f"{work}/crates/singlefs-checker-tier/src/bin/e161_x.rs", None),
            finding_case("先编后换:主 agent 同一处放行", "Edit", None, f"{work}/crates/singlefs-checker-tier/src/bin/e161_x.rs", None),
            finding_case("先编后换:experiment-runner 改草稿目录里的副本放行", "Edit", "experiment-runner",
                         f"{new_root}/runner-x/e161_x.rs", None),
            finding_case("先编后换:experiment-runner 追加 crates/mutations.tsv 放行", "Edit", "experiment-runner",
                         f"{work}/crates/mutations.tsv", None),
            # 三、撇号类角标：五个字符各写死一例（共用字符集少了哪一个，这里就红）
            prime_case("角标:Write 内容含 B2 加一撇（U+2032）", "Write", None, {"content": "取 B2\u2032 的读法\n"}, 2),
            prime_case("角标:Edit new_string 含 P1 加两撇（U+2033）", "Edit", None, {"old_string": "P1", "new_string": "P1\u2033"}, 2),
            prime_case("角标:Write 内容含三撇（U+2034）", "Write", None, {"content": "臂 A\u2034\n"}, 2),
            prime_case("角标:Edit new_string 含 U+02B9", "Edit", None, {"old_string": "丙", "new_string": "丙\u02b9"}, 2),
            prime_case("角标:Write 内容含 U+02BA", "Write", None, {"content": "候选 K9\u02ba\n"}, 2),
            prime_case("角标:MultiEdit 第二处 new_string 含", "MultiEdit", None,
                       {"edits": [{"old_string": "a", "new_string": "b"}, {"old_string": "c", "new_string": "K9\u2032"}]}, 2),
            prime_case("角标:子 agent 在写范围内也判", "Edit", "implementation-writer", {"old_string": "x", "new_string": "G5\u2032"}, 2),
            prime_case("角标:子 agent 同一处不带角标放行（上一例的 2 出自角标这一道）", "Edit", "implementation-writer", {"old_string": "x", "new_string": "G6"}, 0),
            prime_case("角标:ASCII 单引号放行", "Write", None, {"content": "S1' 与 it's\n"}, 0),
            prime_case("角标:只在 old_string 里有（删掉它的改动）放行", "Edit", None, {"old_string": "B2\u2032", "new_string": "B3"}, 0),
            prime_case("角标:MultiEdit 只在 old_string 里有放行", "MultiEdit", None,
                       {"edits": [{"old_string": "B2\u2032", "new_string": "B3"}]}, 0),
            prime_case("角标:读不到共用字符集就拒绝", "Write", None, {"content": "普通内容\n"}, 2,
                       library_path=os.path.join(work, "no-such-lib-forbidden-notations.py")),
        ]
        # 八、old_string 以空白收尾：判的是拒绝那一道的名字；WRITE_GUARD_BREAK=trailing-blank-ignored 时拒绝的几例就红。
        # 不是 .md / .mdx 的目标，钩子收到的 new_string 已去过行尾空白，例子里照那样写
        trailing_finding = "old_string 以空白收尾"
        def trailing_case(label, tool_name, tool_input, want, agent=None, target=None):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=target or f"{work}/records/x.sh")}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library)[2])
        cases += [
            trailing_case("行尾空白:.sh 目标 old_string 以空格收尾", "Edit", {"old_string": "x = 1\ndeclared = ", "new_string": "x = 2\ndeclared ="}, trailing_finding),
            trailing_case("行尾空白:.sh 目标 old_string 以制表符收尾", "Edit", {"old_string": "key\t", "new_string": "key"}, trailing_finding),
            trailing_case("行尾空白:MultiEdit 第二处 old_string 以空格收尾", "MultiEdit",
                          {"edits": [{"old_string": "a", "new_string": "b"}, {"old_string": "python3 - ", "new_string": "python3 -"}]}, trailing_finding),
            trailing_case("行尾空白:.txt 目标也判", "Edit", {"old_string": "declared = ", "new_string": "declared ="}, trailing_finding, target=f"{work}/records/x.txt"),
            trailing_case("行尾空白:.markdown 目标也判（Edit 只在 .md / .mdx 上留行尾空白）", "Edit", {"old_string": "declared = ", "new_string": "declared ="},
                          trailing_finding, target=f"{work}/records/x.markdown"),
            trailing_case("行尾空白:子 agent 写仓外文件也判", "Edit", {"old_string": "a ", "new_string": "b"}, trailing_finding,
                          agent="implementation-writer", target=f"{new_root}/agents/implementation-writer/notes.txt"),
            trailing_case("行尾空白:.md 目标 old_string 以空格收尾放行", "Edit", {"old_string": "x = 1\ndeclared = ", "new_string": "x = 2\ndeclared = "}, None,
                          target=f"{work}/records/x.md"),
            trailing_case("行尾空白:.mdx 目标放行", "Edit", {"old_string": "a ", "new_string": "b "}, None, target=f"{work}/records/x.mdx"),
            trailing_case("行尾空白:大写 .MD 目标放行", "Edit", {"old_string": "a\t", "new_string": "b\t"}, None, target=f"{work}/records/X.MD"),
            trailing_case("行尾空白:.sh 目标 old_string 不以空白收尾放行", "Edit", {"old_string": "x = 1\ndeclared = foo", "new_string": "x = 2\ndeclared = foo"}, None),
            trailing_case("行尾空白:.sh 目标只在中间行有行尾空白放行", "Edit", {"old_string": "x = 1  \ny", "new_string": "x = 3\ny"}, None),
            trailing_case("行尾空白:old_string 以换行收尾、换行前有空白放行", "Edit", {"old_string": "a \n", "new_string": "b\n"}, None),
            trailing_case("行尾空白:只有 new_string 以空白收尾放行（旧判据已撤）", "Edit", {"old_string": "a", "new_string": "b "}, None),
            trailing_case("行尾空白:Write 不判", "Write", {"content": "x = 1 "}, None, target=f"{work}/records/new.sh"),
        ]
        # 五、钟点：样本串用 colon 拼、时区词用 jst / utc 两个变量拼，这份源码里不出现字面钟点与时区词（门禁 doc-text 也扫它）
        colon = ":"
        jst, utc = "J" + "ST", "U" + "TC"
        clock_library = notations_library_path(hook_dir)
        def clock_case(label, target, tool_input, want, agent=None, library_path=clock_library):
            hook_input = {"tool_name": "Edit" if "new_string" in tool_input else "Write",
                          "tool_input": dict(tool_input, file_path=target if os.path.isabs(target) else f"{work}/{target}")}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library, library_path)[2])
        cases += [
            clock_case("钟点:日期 时区词 时分x", "records/x.md", {"content": f"用户 2026-09-26 {jst} 15{colon}3x 定\n"}, "描述里写了钟点"),
            clock_case("钟点:日期 时分 时区词 前后", "records/x.md", {"content": f"实十九（2026-09-24 16{colon}50 {utc} 前后落地）\n"}, "描述里写了钟点"),
            clock_case("钟点:时区词 时分，不带日期", ".claude/kb/x.md", {"old_string": "a", "new_string": f"同日 {utc} 14{colon}00 前后：\n"}, "描述里写了钟点"),
            clock_case("钟点:时分 时区词，不带日期", "crates/a.rs", {"content": f"// 跑于 03{colon}44 {jst}\n"}, "描述里写了钟点"),
            clock_case("钟点:带 x 通配的裸时分", "records/x.md", {"content": f"主 agent 22{colon}5x 跑前修订\n"}, "描述里写了钟点"),
            clock_case("钟点:子 agent 也判", "research/prompts/e12-preregistration.md", {"content": f"交回 {jst} 22{colon}0x\n"}, "描述里写了钟点", agent="experiment-runner"),
            clock_case("钟点:只写日期放行", "records/x.md", {"content": "用户 2026-09-26 定\n"}, None),
            clock_case("钟点:仓外草稿目录放行", f"{new_root}/x/progress.md", {"content": f"2026-09-27 22{colon}41 {jst} 编过\n"}, None),
            clock_case("钟点:排除的门禁样本放行", ".claude/gate.d/fixtures/x/red/expect", {"content": f"改于 2026-09-05 00{colon}00\n"}, None),
            clock_case("钟点:排除的 hook 自检放行", ".claude/hooks/session-start.sh", {"content": f"2026-09-25 07{colon}58{colon}02 {utc}\n"}, None),
            clock_case("钟点:排除的 c510 提示放行", "research/prompts/_c510-date-gate-r1-body.md", {"content": f"本机时钟是 {utc}\n"}, None),
            clock_case("钟点:只在 old_string 里有放行", "records/x.md", {"old_string": f"{jst} 15{colon}3x", "new_string": ""}, None),
            clock_case("钟点:读不到判据就拒绝", "records/x.md", {"content": "普通内容\n"}, "描述里写了钟点",
                       library_path=os.path.join(work, "no-such-lib-forbidden-notations.py")),
        ]
        # 五、钟点里的各个判据行：样本串用 glued 把数字与「点」、日期与时段词拆开拼，这份源码里不出现字面写法；
        # 判的是拒绝信息点名的判据行名字，LIB_FORBIDDEN_NOTATIONS_BREAK 关掉哪一条判据行，点名它的那几例就红；
        # 放行的几组分别靠上下文规则、放行标记，LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT / _ALLOW 点名钟点那一形态时它们就红
        def glued(*parts):
            return "".join(parts)
        def clock_shape_case(label, content, want_shape, target="records/x.md"):
            code, message, finding = decide({"tool_name": "Write", "tool_input": {"file_path": f"{work}/{target}", "content": content + "\n"}},
                                            work, table, library, clock_library)
            shape = re.search(r"描述里写了钟点，共 \d+ 处：\n  [^（\n]*（(.+?)）：「", message) if finding == "描述里写了钟点" else None
            return (label, want_shape, shape.group(1) if shape else finding)
        hour_shape, period_shape, period_hour_shape, suffix_shape = "日期或时区后面跟「N 点 / N 时」", "日期或时区后面跟时段词", "时段词后面跟钟点", "「N 点前后」这类钟点"
        date_shape, bare_shape, iso_shape, zone_shape = "日期后面跟钟点", "不挨日期时区的钟点", "ISO 时间戳", "时区词"
        twelve_hour_shape = "十二小时制钟点"
        cases += [
            clock_shape_case("钟点:限额通知原样的英文月日十二小时制", glued("通知写 resets Oct 4 4", f"pm {utc}"), twelve_hour_shape),
            clock_shape_case("钟点:十二小时制隔空格大写", glued("11 ", "AM 截止"), twelve_hour_shape),
            clock_shape_case("钟点:十二小时制带点", glued("交回于 3", "p.m."), twelve_hour_shape),
            clock_shape_case("钟点:以 am / pm 起头的英文词放行", glued("2 AMD 显卡、3 ", "amps、5 ppm、pm2 进程、v1.2", "am"), None),
        ]
        # 七、会话号：样本串用 glued 把目录名与十六进制拆开拼，这份源码里不出现会话号；
        # WRITE_GUARD_DISABLE_SESSION_NAMES=1 或 LIB_FORBIDDEN_NOTATIONS_BREAK=session-names 时拒绝的几例就红，
        # LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=session-names 时带放行标记的那一例就红
        session_shape = "写了会话号"
        cases += [
            clock_shape_case("会话号:标签式（会话 号，日期）", glued("### 第 4 批开工（会话 singlefs-", "21，2026-10-01）"), session_shape),
            clock_shape_case("会话号:当主语用的", glued("已发消息，singlefs-", "e1 回话"), session_shape),
            clock_shape_case("会话号:上游三语仓的", glued("SOP 会话 singlefs-ai-sop-zh-", "86 同步来"), session_shape),
            clock_shape_case("会话号:四位十六进制", glued("会话 singlefs-", "e152 在跑"), session_shape),
            clock_shape_case("会话号:crate 名、上游仓目录名、临时目录名放行",
                             glued("改 singlefs-core 与 singlefs-ai-sop-zh；mktemp -d /tmp/singlefs-", "e152.XXXXXX；/tmp/singlefs-", "e152-packages"), None),
            clock_shape_case("会话号放行标记:sh 里的自检输入放行",
                             glued("echo singlefs-", "6a  # session-names:allow 解析 ListAgents 输出的自检输入"), None, target=".claude/scripts/parse.sh"),
            clock_shape_case("钟点:日期 时区词 N 点前后", glued(f"用户 2026-09-28 {jst} 08 ", "点前后要开"), hour_shape),
            clock_shape_case("钟点:时区词 日期 N 点后（时区词那一处先中）", glued(f"本机 {utc} 2026-09-26 23 ", "点后"), zone_shape),
            clock_shape_case("钟点:日期 时区词 N 时许", glued(f"2026-09-16 {utc} 17 ", "时许的快照"), hour_shape),
            clock_shape_case("钟点:不带年份的日期 时区词 N 点前后", glued(f"09-24 {jst} 19 ", "点前后问了四件"), hour_shape),
            clock_shape_case("钟点:日期 时区词 N 点后", glued(f"2026-09-25 {jst} 01 ", "点后弹窗"), hour_shape),
            clock_shape_case("钟点:日期 时区词 时段词", glued(f"交回于 2026-09-27 {jst} ", "上午"), period_shape),
            clock_shape_case("钟点:括号里日期后跟时段词", glued("降为待证（2026-09-13 ", "上午）"), period_shape),
            clock_shape_case("钟点:日期时段词再括号时区词", glued("2026-09-12 ", f"凌晨（{jst}）"), period_shape),
            clock_shape_case("钟点:括号里时区词日期时段词（时区词那一处先中）", glued(f"（{jst} 2026-09-27 ", "早上）"), zone_shape),
            clock_shape_case("钟点:日期时段词前后", glued("满载到 2026-09-28 ", "中午前后"), period_shape),
            clock_shape_case("钟点:标题里日期后括号时段词", glued("### 2026-09-27（", "下午）：E162"), period_shape),
            clock_shape_case("钟点:日期时段词再时分（日期那一处先中）", glued("2026-09-27 ", f"上午 08{colon}32 现跑"), period_shape),
            clock_shape_case("钟点:时段词后跟时分", f"上午 08{colon}32 现跑", period_hour_shape),
            clock_shape_case("钟点:时段词后跟 N 点", glued("晚上 ", "8 点"), period_hour_shape),
            clock_shape_case("钟点:光秃的 N 点前后", glued("19 ", "点前后问了"), suffix_shape),
            clock_shape_case("钟点:裸时分", f"用户 23{colon}10 定", bare_shape),
            clock_shape_case("钟点:裸时分秒", f"交回 14{colon}00{colon}10、续做", bare_shape),
            clock_shape_case("钟点:裸时分秒带毫秒", f"07{colon}58{colon}02.123 起跑", bare_shape),
            clock_shape_case("钟点:时分区间", f"14{colon}00–15{colon}30 之间跑完", bare_shape),
            clock_shape_case("钟点:月-日 时分", f"核查员缺快照（09-24 23{colon}44 那一次）", bare_shape),
            clock_shape_case("钟点:反引号里两位小时的时分", f"跑于 `09{colon}55` 的那一轮", bare_shape),
            clock_shape_case("钟点:日期后面跟带秒带偏移的 diff 头", f"--- a/x.rs\t2026-09-24 16{colon}27{colon}05.398761624 +0000", date_shape),
            clock_shape_case("钟点:stat 的修改时间", f"Modify: 2026-09-24 18{colon}48{colon}58.254848952 +0000", date_shape),
            clock_shape_case("钟点:ISO 时间戳", f"派腿 2026-09-27T09{colon}55{colon}07+00{colon}00", iso_shape),
            clock_shape_case("钟点:不带日期的 T 钟点", f"交回于 …T23{colon}15Z", iso_shape),
            clock_shape_case("钟点:光秃的时区词", f"本机时钟是 {utc}、直接 date 会差一天", zone_shape),
            clock_shape_case("钟点:日期后括号时区词", f"跑于 2026-09-27（{jst}），本机", zone_shape),
            clock_shape_case("钟点:两个时区的名字", f"{jst} 与 {utc} 两个时区", zone_shape),
            clock_shape_case("钟点:时区词 时间", f"{utc} 时间", zone_shape),
            clock_shape_case("钟点:放行标记写在 md 里不认", f"用户 23{colon}10 定  # clock-times:allow 写在 md 里不认", bare_shape),
            clock_shape_case("钟点:放行标记没写理由不认", f"// 跑于 23{colon}10  // clock-times:allow", bare_shape, target="crates/a.rs"),
            clock_shape_case("钟点:第 N 点前后是序号放行", "第 3 点前后两句", None),
            clock_shape_case("钟点:N 点意见放行", "3 点意见", None),
            clock_shape_case("钟点:个数后的点放行", "1000 个崩溃点", None),
            clock_shape_case("钟点:夜间放行", "全量由用户要求或夜间跑", None),
            clock_shape_case("钟点:只写日期放行（Write 整份）", "用户 2026-09-28 定", None),
            clock_shape_case("钟点:不挨日期时区的光秃时段词放行", "同一天下午续跑", None),
            clock_shape_case("钟点:标识符里的时区词放行", f"`{jst}_OFFSET`、`started_{utc}`", None),
        ]
        context_cases = [
            clock_shape_case("钟点上下文:产物行的名字=值放行", f"remount_chosen=2{colon}12 device_chosen=1{colon}16 values=1{colon}16.000,3{colon}12.000", None),
            clock_shape_case("钟点上下文:差分与计数的冒号对放行", f"adjacent_differences=0->1{colon}10,1->2{colon}8 steps_by_changed_leaf_count=1{colon}108,2{colon}36", None),
            clock_shape_case("钟点上下文:字典放行", f"分布 {{0{colon}14, 1{colon}14, 2{colon}16}}", None),
            clock_shape_case("钟点上下文:Python 切片放行", f"r[8{colon}12]=i.to_bytes(4); fields[11{colon}15]; arguments[5{colon}13]", None),
            clock_shape_case("钟点上下文:代码里的时区偏移放行", f'stamp.replace("Z", "+00{colon}00")', None),
            clock_shape_case("钟点上下文:双引号里的代码字面放行", f'            Some("2{colon}11"),', None),
            clock_shape_case("钟点上下文:单引号里的代码字面放行", f"                      printf '2{colon}11' ;;", None),
            clock_shape_case("钟点上下文:百分比放行", f"B Lost Writes **13{colon}54%**", None),
            clock_shape_case("钟点上下文:time -v 的格式说明放行", f"Elapsed (wall clock) time (h:mm:ss or m:ss): 0{colon}03.21", None),
            clock_shape_case("钟点上下文:门禁号行号表格放行", f"| 共用脚本不在 | 21{colon}38 | 31{colon}68 |", None),
            clock_shape_case("钟点上下文:门禁号行号列表放行", f"| kb 目录不在 | —— | 21{colon}39、24{colon}27、27{colon}39 |", None),
            clock_shape_case("钟点上下文:门禁文件名放行", f"门禁 20{colon}09-sample-stage.sh 那一行", None),
            clock_shape_case("钟点上下文:同一行带文件行号的编号对放行", f"触发的那一处（decisions/06-快照实现模型.md{colon}42），06{colon}42 正是没抽到的那一行", None),
            clock_shape_case("钟点上下文:反引号里一位小时的键放行", f"撞键 `3{colon}10`：冷重开是 `2{colon}11`", None),
            clock_shape_case("钟点上下文:产物字段里的 ISO 时间戳放行", f"taken_jst=2026-09-27T01{colon}20{colon}12+09{colon}00", None),
        ]
        allow_cases = [
            clock_shape_case("钟点放行标记:sh 里的 TZ 环境变量放行", f"TZ={utc}-14 date +%F  # clock-times:allow 环境变量写法，符号与东京时间相反", None, target=".claude/scripts/tz.sh"),
            clock_shape_case("钟点放行标记:py 里解析外部通知的正则放行", f'RESETS = re.compile(r"resets (\\d+)(am|pm) \\({utc}\\)")  # clock-times:allow 解析外部通知文本', None,
                             target=".claude/hooks/parse.py"),
            clock_shape_case("钟点放行标记:rs 里造时间戳的自检输入放行", f'let stamp = "2026-09-27T09{colon}55{colon}07Z"; // clock-times:allow 造时间戳格式的自检输入', None,
                             target="crates/a.rs"),
        ]
        cases += context_cases + allow_cases
        # 钟点、会话号逐处全报（行号与总数；LIB_FORBIDDEN_NOTATIONS_BREAK=first-hit-only 下前两格红）；Edit 的行号折算成文件里的行号；
        # 八道判完再报：覆盖、越界与钟点三道一次列全（WRITE_GUARD_BREAK=stop-at-first 下末格红）
        def refusal_of(tool_name, target, tool_input, agent=None):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=f"{work}/{target}")}
            if agent:
                hook_input["agent_type"] = agent
            code, message, finding = decide(hook_input, work, table, library, clock_library)
            return message or "", finding
        two_clocks, _ = refusal_of("Write", "records/x.md", {"content": f"用户 23{colon}10 定\n中间一行\n交回 14{colon}00{colon}10、续做\n"})
        cases.append(("钟点:两处都报（行号与总数）", True, all(fragment in two_clocks for fragment in ("共 2 处", "  第 1 行（", "  第 3 行（"))))
        two_sessions, _ = refusal_of("Write", "records/x.md", {"content": glued("会话 singlefs-", "e152 在跑\n已发消息，singlefs-", "e1 回话\n")})
        cases.append(("会话号:两处都报（行号与总数）", True, all(fragment in two_sessions for fragment in ("共 2 处", "  第 1 行：", "  第 2 行："))))
        edit_lines, _ = refusal_of("Edit", "tracked.md", {"old_string": "a", "new_string": f"x\n用户 23{colon}10 定"})
        cases.append(("钟点:Edit 的行号折算成文件里的行号", True, "  第 2 行（" in edit_lines))
        _, three_labels = refusal_of("Write", "untracked.md", {"content": f"用户 23{colon}10 定\n"}, agent="kb-scribe")
        cases.append(("八道判完再报:覆盖、越界与钟点一次列全", "整份覆盖未跟踪文件、越出写范围、描述里写了钟点", three_labels))
        # 走真实入口：把脚本当子进程、从标准输入喂 JSON，防「判定函数对、入口读不到输入」；拒绝要落进检出记录
        script = os.path.join(hook_dir, "write-guard.sh")
        detections = os.path.join(work, "detections.jsonl")
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=work, AGENT_HOOK_DETECTIONS=detections)
        def run_entry(hook_input):
            return subprocess.run(["bash", script], input=json.dumps(hook_input), capture_output=True, text=True, env=environment)
        def stdin_case(label, hook_input, want):
            return (label, want, run_entry(hook_input).returncode)
        cases += [
            stdin_case("stdin:主 agent 写新文件", {"tool_name": "Write", "tool_input": {"file_path": f"{work}/crates/a.rs", "content": "x"}}, 0),
            stdin_case("stdin:覆盖未跟踪文件", {"tool_name": "Write", "tool_input": {"file_path": f"{work}/untracked.md", "content": "x"}}, 2),
            stdin_case("stdin:kb-scribe 越界", {"tool_name": "Edit", "agent_type": "kb-scribe", "tool_input": {"file_path": f"{work}/crates/a.rs"}}, 2),
            stdin_case("stdin:kb-scribe 范围内", {"tool_name": "Edit", "agent_type": "kb-scribe", "tool_input": {"file_path": f"{work}/.claude/kb/x.md"}}, 0),
            stdin_case("stdin:kb-scribe 用 MultiEdit 越界", {"tool_name": "MultiEdit", "agent_type": "kb-scribe",
                                                            "tool_input": {"file_path": f"{work}/crates/a.rs", "edits": [{"old_string": "a", "new_string": "b"}]}}, 2),
            stdin_case("stdin:kb-scribe 用 MultiEdit 范围内", {"tool_name": "MultiEdit", "agent_type": "kb-scribe",
                                                              "tool_input": {"file_path": f"{work}/.claude/kb/x.md", "edits": [{"old_string": "a", "new_string": "b"}]}}, 0),
            stdin_case("stdin:大内容越界", {"tool_name": "Write", "agent_type": "implementation-writer", "tool_input": {"file_path": f"{work}/research/big.md", "content": "y" * 300000}}, 2),
            stdin_case("stdin:实现员写工作根下的报告", {"tool_name": "Write", "agent_type": "implementation-writer",
                                                       "tool_input": {"file_path": f"{new_root}/x/report.md", "content": "x"}}, 0),
        ]
        runner_entry = run_entry({"tool_name": "Edit", "agent_type": "experiment-runner",
                                  "tool_input": {"file_path": f"{work}/crates/singlefs-checker-tier/src/bin/e161_x.rs", "old_string": "a", "new_string": "b"}})
        cases.append(("stdin:experiment-runner 改主工作区的入库装置拒绝", 2, runner_entry.returncode))
        cases.append(("stdin:先编后换的拒绝点名那条脚本与草稿目录", True,
                      all(fragment in runner_entry.stderr for fragment in ("research/scripts/compile-then-swap.py", "草稿目录", "→ 怎么办"))))
        prime_entry = run_entry({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/crates/a.rs", "old_string": "B2", "new_string": "取 B2\u2032 的读法"}})
        cases.append(("stdin:Edit 写进角标", 2, prime_entry.returncode))
        cases.append(("stdin:角标的拒绝带字符、前后几个字与出路", True,
                      all(fragment in prime_entry.stderr for fragment in ("撇号类角标 \u2032", "取 B2\u2032 的读法", "→ 怎么办", "path-moves.md"))))
        trailing_entry = run_entry({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/records/x.sh", "old_string": "python3 - ", "new_string": "python3 -"}})
        cases.append(("stdin:.sh 目标 Edit old_string 以空白收尾", 2, trailing_entry.returncode))
        cases.append(("stdin:行尾空白的拒绝带收尾那几个字、两条出路", True,
                      all(fragment in trailing_entry.stderr for fragment in ("「…python3 -␠」", "→ 怎么办", "多框到下一个非空白字符", "replace-once.py"))))
        cases.append(("stdin:.md 目标 Edit old_string 以空白收尾放行", 0,
                      run_entry({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/records/x.md", "old_string": "python3 - ", "new_string": "python3 - "}}).returncode))
        recorded = sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
        cases.append(("stdin:七次拒绝都落进检出记录", 7, recorded))
        os.unlink(outside.name)
    finally:
        shutil.rmtree(work)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide_overwrite() / decide_compile_first() / decide_trailing_blank() / decide_shell_syntax() / decide_scope() / decide_prime_marks() / decide_clock_times() / decide_session_names() 与入口；"
              "WRITE_GUARD_DISABLE_OVERWRITE / WRITE_GUARD_DISABLE_COMPILE_FIRST / WRITE_GUARD_DISABLE_SYNTAX / WRITE_GUARD_DISABLE_SCOPE / WRITE_GUARD_DISABLE_CLOCK_TIMES / WRITE_GUARD_DISABLE_SESSION_NAMES、"
              "WRITE_GUARD_BREAK=trailing-blank-ignored / scope-root-unanchored 或 SINGLEFS_WORK_ROOT_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(cases)} 种情形）：钩子、门禁、项目脚本、研究脚本目录下写进语法错误的 .sh（Write 与 Edit）与 .py 拒绝、语法对的与别处的放行；"
          "未跟踪的已有文件整份覆盖拒绝，已跟踪 / 不存在 / 仓外 / Edit 放行；主 agent 与内置 agent 放行、范围内放行、"
          "工作根与过渡期旧根下的报告目录放行、工作根同名前缀的兄弟目录拒绝，范围外（Edit、Write、MultiEdit）与 .. 绕路与未登记的项目 agent 拒绝；experiment-runner 写主工作区 crates/ 下的 .rs（改、新建、.. 绕路）按先编后换拒绝并点名 compile-then-swap.py，"
          "implementation-writer 与主 agent 写同一处、执行员写草稿目录与 crates/mutations.tsv 放行；Write 内容、Edit 与 MultiEdit 的 new_string 里有撇号类角标（五个字符各一例，主 agent 与子 agent 一样）拒绝，"
          "只在 old_string 里有、ASCII 单引号放行，共用字符集读不到拒绝；写进仓里的内容带钟点（日期后、时区词旁、x 通配、日期或时区词后的「N 点 / N 时」与时段词、时段词后的钟点、光秃的「N 点前后」、"
          "不挨日期时区的时分与区间、ISO 与 diff 头 stat 的机器时间、光秃的时区词、英文十二小时制，md 里与没写理由的放行标记不认，主 agent 与子 agent 一样）拒绝并点名形态，"
          f"只写日期、序号与个数后的「点」、不挨日期时区的时段词、标识符里的时区词、以 am / pm 起头的英文词、{len(context_cases)} 类长得像钟点的冒号对、"
          f".sh / .py / .rs 里带理由的放行标记 {len(allow_cases)} 例、仓外、排除的样本、hook 自检与 c510 提示、只在 old_string 里有放行，判据读不到拒绝；写进仓里的会话号（标签式、当主语、上游三语仓、长十六进制）拒绝，crate 名、上游仓目录名、临时目录名与 .sh 里带理由的放行标记放行；"
          "目标不是 .md / .mdx 时 Edit 与 MultiEdit 的 old_string 以空格或制表符收尾（.sh、.txt、.markdown，仓里仓外、主 agent 与子 agent 一样）拒绝并给出收窄框选与 replace-once.py 两条出路，"
          ".md / .mdx / .MD 目标、不以空白收尾、只在中间行或换行前有行尾空白、只有 new_string 以空白收尾与 Write 放行；"
          "钟点与会话号逐处列出（行号与总数，Edit 折算成文件里的行号），几道都不成立的一次列全；拒绝都记进检出记录")
    return 0

hook_dir = sys.argv[1]
if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
    sys.exit(selftest(hook_dir))
raw = sys.stdin.read()
try:
    hook_input = json.loads(raw)
except ValueError:
    sys.exit(0)
project_root = os.environ.get("CLAUDE_PROJECT_DIR") or hook_input.get("cwd") or os.getcwd()
code, message, finding = decide(hook_input, project_root, os.path.join(hook_dir, "agent-write-scope.tsv"), notations_library_path(hook_dir))
if code:
    print(message, file=sys.stderr)
    record_detection(hook_input, finding, os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS)
sys.exit(code)
PY
