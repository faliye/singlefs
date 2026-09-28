#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Write、Edit）：写文件之前的四道判定，合在一个 hook 里；拒绝的同时把它记进检出记录，交主 agent 看。
#
# 一、整份覆盖未跟踪文件（只管 Write）：仓里已存在、又没进 git 的文件，拒绝用 Write 整份覆盖。
#    为什么：几个会话同时在一个仓里干活。2026-09-11 一个会话用 Write 新建 `research/prompts/e137-preregistration.md`，
#    另一个会话几分钟前刚写了同名的未提交文件，Write 回报「updated」就把它盖了——没进过 git，事后只能从对话记录里重放复原。
#    放行：目标不存在；目标已被 git 跟踪（盖了还能从 git 找回）；目标在仓外（scratchpad 之类）。
# 二、项目 subagent 的写范围（Write、Edit）：项目定义的 subagent 写文件时，目标必须落在它登记的写范围里。
#    为什么：执行类 agent 越界写，不该等人翻 diff 才发现（records/2026-09-16-subagent拆分提案.md 第五节，用户 2026-09-17 定走项目 settings hook）。
#    输入里没有 agent_type（主 agent）放行；agent_type 没有项目定义（内置 agent）放行；有项目定义而表里没有它的写范围，拒绝；
#    目标路径规范化之后命中它的任一模式才放行。表在同目录的 agent-write-scope.tsv。拦不住 Bash 里的写，定义的「没做什么」照写。
# 三、撇号类角标（Write、Edit、MultiEdit）：Write 的 content、Edit 的 new_string、MultiEdit 每一处的 new_string 里
#    出现撇号类字符之一就拒绝；只在 old_string 里出现（删掉它的改动）放行，ASCII 单引号不判。主 agent 与所有子 agent 一样判，目标在不在仓里都判。
#    为什么：门禁 doc-text 只在收尾跑整轮门禁时判，2026-09-24 实验设计员照样把这类名字写进三份重跑登记、执行员又抄进源码。
#    字符集与 doc-text 同一份：../gate.d/lib-forbidden-notations.py 的 PRIME_MARKS，不在这里另抄；读不到就拒绝（不静默放行）。
# 四、先编后换（Write、Edit）：experiment-runner（名单是 COMPILE_FIRST_AGENTS）写主工作区 crates/ 下的 .rs（改已有的、新建的都算）一律拒绝，
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
#
# 三道原本是两个 hook（refuse-overwrite-untracked.sh、agent-write-scope.sh），2026-09-17 用户定合并；第三道 2026-09-24 加进来。
# 它们拒的是一次写，不停任务和脚本；拒绝时退出码 2、stderr 交给做这次写的模型，同时往检出记录
# （默认 /tmp/claude-1000/agent-hook-detections.jsonl，环境变量 AGENT_HOOK_DETECTIONS 可改）追加一行，
# 主 agent 的看门狗（research/scripts/agent-watch.py watch）读到就叫醒主 agent，由主 agent 判断怎么处理。
# 这份源码里撇号类字符一律写成转义、lib-forbidden-notations.py 里用码位拼（门禁 doc-text 也扫它们）；改带转义的那几行用 Bash 里的 python，反斜杠拿 chr(92) 拼：
# 2026-09-24 写这一道时实测，Write 的 content 里写的反斜杠 u 转义落盘成了字符本身，这道闸也会拒这样的写。
#
#   write-guard.sh             # 从 stdin 读 hook 的 JSON
#   write-guard.sh --selftest  # 走一遍五道判定的放行与拒绝；WRITE_GUARD_DISABLE_OVERWRITE=1、WRITE_GUARD_DISABLE_COMPILE_FIRST=1、
#                              # WRITE_GUARD_DISABLE_SCOPE=1 或 WRITE_GUARD_DISABLE_CLOCK_TIMES=1 时自检必须判红；
#                              # LIB_FORBIDDEN_NOTATIONS_BREAK=<判据行键>（钟点的 date-zone-hour、date-zone-period、period-hour、hour-suffix、
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

DEFAULT_DETECTIONS = "/tmp/claude-1000/agent-hook-detections.jsonl"

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
    for pattern in patterns:
        target = absolute if pattern.startswith("/") else relative
        if target is not None and glob_to_regex(pattern).match(target):
            return 0, None
    return 2, (f"✗ {agent_type} 的写范围不含 {absolute}\n"
               f"→ 它的写范围：{'、'.join(patterns)}（.claude/hooks/agent-write-scope.tsv）。\n"
               f"→ 怎么办：范围外的改动不自己做，写进报告交回主 agent，由主 agent 改或派该写的 agent。")

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

def decide_clock_times(hook_input, project_root, library_path):
    """五、写进仓里的内容带描述性的钟点。"""
    if os.environ.get("WRITE_GUARD_DISABLE_CLOCK_TIMES") == "1":
        return 0, None
    texts = [text for text in written_texts(hook_input) if text]
    absolute = absolute_target(hook_input, project_root)
    root = os.path.normpath(project_root)
    if not texts or not absolute or os.path.commonpath([absolute, root]) != root:
        return 0, None
    try:
        spec = importlib.util.spec_from_file_location("lib_forbidden_notations", library_path)
        library = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(library)
        relative = os.path.relpath(absolute, root)
        if library.is_excluded("clock-times", relative):
            return 0, None
        found = next((result for result in (library.first_hit("clock-times", text, relative) for text in texts) if result), None)
    except Exception as error:
        return 2, (f"✗ 读不到钟点判据 {library_path}（{type(error).__name__}：{error}），这次写没法判，按拒绝处理。\n"
                   "→ 怎么办：那一份是门禁 doc-text 与这道 hook 共用的唯一定义，先 git status 看它是被删了还是被改坏了，"
                   "用 Bash 恢复它（不在 hook 里另抄一份）；恢复之前往仓里的 Write / Edit 都会被拒。")
    if not found:
        return 0, None
    position, shape, matched = found
    return 2, (f"✗ 写进 {relative} 的内容在描述里写了钟点（{shape}：「{matched}」）\n"
               "→ 怎么办：时间只写到日期，把钟点（时:分、「N 点 / N 时」、上午下午这类时段词）连同挨着它的时区词、「前后」一起删掉；"
               "光秃的时区词、ISO 时间戳与 date 的原样输出也删，只写日期。\n"
               "→ .sh、.py、.rs 里代码真要用这些字面（TZ 环境变量、解析外部文本的正则、造时间戳的自检输入），在那一行写「# clock-times:allow <理由>」（Rust 写「//」）；"
               "命中的其实不是钟点（产物字段、切片、编号对），改 .claude/gate.d/lib-forbidden-notations.py 的上下文规则\n"
               "→ 钟点确实是被测输入的整份文件（hook 自检、门禁样本），登记进 .claude/gate.d/lib-forbidden-notations.py 里钟点那一形态的 excluded_prefixes 并写明理由")

def decide(hook_input, project_root, table_path, prime_marks_library, clock_times_library=None):
    """返回 (0, None, None) 放行；(2, 说明, 哪一道) 拒绝。"""
    code, message = decide_overwrite(hook_input, project_root)
    if code:
        return code, message, "整份覆盖未跟踪文件"
    code, message = decide_compile_first(hook_input, project_root)
    if code:
        return code, message, "绕过先编后换"
    code, message = decide_scope(hook_input, project_root, table_path)
    if code:
        return code, message, "越出写范围"
    code, message = decide_prime_marks(hook_input, prime_marks_library)
    if code:
        return code, message, "写进撇号类角标"
    if clock_times_library is None:
        clock_times_library = prime_marks_library
    code, message = decide_clock_times(hook_input, project_root, clock_times_library)
    if code:
        return code, message, "描述里写了钟点"
    return 0, None, None

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
                         for pattern in patterns if pattern.startswith("/")]
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
        def prime_case(label, tool_name, agent, tool_input, want, library_path=library):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=f"{work}/crates/a.rs")}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library_path)[0])
        cases += [
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
            case("范围:仓外报告目录", "Write", "implementation-writer", "/tmp/claude-1000/agents/implementation-writer/report.md", 0),
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
                         "/tmp/claude-1000/runner-x/e161_x.rs", None),
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
            clock_case("钟点:仓外草稿目录放行", "/tmp/claude-1000/x/progress.md", {"content": f"2026-09-27 22{colon}41 {jst} 编过\n"}, None),
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
            shape = re.search(r"描述里写了钟点（(.+?)：「", message) if finding == "描述里写了钟点" else None
            return (label, want_shape, shape.group(1) if shape else finding)
        hour_shape, period_shape, period_hour_shape, suffix_shape = "日期或时区后面跟「N 点 / N 时」", "日期或时区后面跟时段词", "时段词后面跟钟点", "「N 点前后」这类钟点"
        date_shape, bare_shape, iso_shape, zone_shape = "日期后面跟钟点", "不挨日期时区的钟点", "ISO 时间戳", "时区词"
        cases += [
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
            clock_shape_case("钟点上下文:门禁文件名放行", f"门禁 20{colon}54-layer0-replay.sh 那一行", None),
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
            stdin_case("stdin:大内容越界", {"tool_name": "Write", "agent_type": "implementation-writer", "tool_input": {"file_path": f"{work}/research/big.md", "content": "y" * 300000}}, 2),
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
        recorded = sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
        cases.append(("stdin:五次拒绝都落进检出记录", 5, recorded))
        os.unlink(outside.name)
    finally:
        shutil.rmtree(work)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide_overwrite() / decide_compile_first() / decide_scope() / decide_prime_marks() / decide_clock_times() 与入口；"
              "WRITE_GUARD_DISABLE_OVERWRITE / WRITE_GUARD_DISABLE_COMPILE_FIRST / WRITE_GUARD_DISABLE_SCOPE / WRITE_GUARD_DISABLE_CLOCK_TIMES 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(cases)} 种情形）：未跟踪的已有文件整份覆盖拒绝，已跟踪 / 不存在 / 仓外 / Edit 放行；主 agent 与内置 agent 放行、范围内放行、"
          "范围外与 .. 绕路与未登记的项目 agent 拒绝；experiment-runner 写主工作区 crates/ 下的 .rs（改、新建、.. 绕路）按先编后换拒绝并点名 compile-then-swap.py，"
          "implementation-writer 与主 agent 写同一处、执行员写草稿目录与 crates/mutations.tsv 放行；Write 内容、Edit 与 MultiEdit 的 new_string 里有撇号类角标（五个字符各一例，主 agent 与子 agent 一样）拒绝，"
          "只在 old_string 里有、ASCII 单引号放行，共用字符集读不到拒绝；写进仓里的内容带钟点（日期后、时区词旁、x 通配、日期或时区词后的「N 点 / N 时」与时段词、时段词后的钟点、光秃的「N 点前后」、"
          "不挨日期时区的时分与区间、ISO 与 diff 头 stat 的机器时间、光秃的时区词，md 里与没写理由的放行标记不认，主 agent 与子 agent 一样）拒绝并点名形态，"
          f"只写日期、序号与个数后的「点」、不挨日期时区的时段词、标识符里的时区词、{len(context_cases)} 类长得像钟点的冒号对、"
          f".sh / .py / .rs 里带理由的放行标记 {len(allow_cases)} 例、仓外、排除的样本、hook 自检与 c510 提示、只在 old_string 里有放行，判据读不到拒绝；拒绝都记进检出记录")
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
