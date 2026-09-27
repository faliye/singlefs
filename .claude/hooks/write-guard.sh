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
#    为什么：门禁 12 号只在收尾跑整轮门禁时判，2026-09-24 实验设计员照样把这类名字写进三份重跑登记、执行员又抄进源码。
#    字符集与 12 号同一份：../gate.d/lib-prime-marks.py，不在这里另抄；读不到就拒绝（不静默放行）。
# 四、先编后换（Write、Edit）：experiment-runner（名单是 COMPILE_FIRST_AGENTS）写主工作区 crates/ 下的 .rs（改已有的、新建的都算）一律拒绝，
#    出路是在草稿目录的副本里改、经 research/scripts/compile-then-swap.py 编过再整份换进来。排在写范围那一道之前判，拒绝信息给的是这条出路。
#    implementation-writer、主 agent、别的 agent 不判；草稿目录里的副本与 crates/ 下不是 .rs 的（crates/mutations.tsv）不判。
#    为什么：执行员直接在主工作区改入库装置，改到一半编不过，别的会话带 --all-targets 的编译一起卡住
#    （records/2026-09-16-subagent拆分提案.md 第四十节那张表第 51 行）。Bash 里的同一类写法归 bash-command-detector.sh 的 ⑨。
#
# 三道原本是两个 hook（refuse-overwrite-untracked.sh、agent-write-scope.sh），2026-09-17 用户定合并；第三道 2026-09-24 加进来。
# 它们拒的是一次写，不停任务和脚本；拒绝时退出码 2、stderr 交给做这次写的模型，同时往检出记录
# （默认 /tmp/claude-1000/agent-hook-detections.jsonl，环境变量 AGENT_HOOK_DETECTIONS 可改）追加一行，
# 主 agent 的看门狗（research/scripts/agent-watch.py watch）读到就叫醒主 agent，由主 agent 判断怎么处理。
# 这份源码与 lib-prime-marks.py 里撇号类字符一律写成转义（门禁 12 号也扫它们）；改带转义的那几行用 Bash 里的 python，反斜杠拿 chr(92) 拼：
# 2026-09-24 写这一道时实测，Write 的 content 里写的反斜杠 u 转义落盘成了字符本身，这道闸也会拒这样的写。
#
#   write-guard.sh             # 从 stdin 读 hook 的 JSON
#   write-guard.sh --selftest  # 走一遍四道判定的放行与拒绝；WRITE_GUARD_DISABLE_OVERWRITE=1、WRITE_GUARD_DISABLE_COMPILE_FIRST=1 或 WRITE_GUARD_DISABLE_SCOPE=1 时自检必须判红
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

def prime_marks_library_path(hook_dir):
    return os.path.join(os.path.dirname(hook_dir), "gate.d", "lib-prime-marks.py")

def load_prime_marks(library_path):
    """撇号类字符集，与门禁 12 号同一份；读不到或读出来是空的就抛异常，由调用方按拒绝处理。"""
    spec = importlib.util.spec_from_file_location("lib_prime_marks", library_path)
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
                   "→ 怎么办：那一份是门禁 12 号与这道 hook 共用的唯一定义，先 git status 看它是被删了还是被改坏了，"
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

def decide(hook_input, project_root, table_path, prime_marks_library):
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
    library = prime_marks_library_path(hook_dir)
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
                       library_path=os.path.join(work, "no-such-lib-prime-marks.py")),
        ]
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
        print("    → 看 decide_overwrite() / decide_compile_first() / decide_scope() / decide_prime_marks() 与入口；"
              "WRITE_GUARD_DISABLE_OVERWRITE / WRITE_GUARD_DISABLE_COMPILE_FIRST / WRITE_GUARD_DISABLE_SCOPE 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(cases)} 种情形）：未跟踪的已有文件整份覆盖拒绝，已跟踪 / 不存在 / 仓外 / Edit 放行；主 agent 与内置 agent 放行、范围内放行、"
          "范围外与 .. 绕路与未登记的项目 agent 拒绝；experiment-runner 写主工作区 crates/ 下的 .rs（改、新建、.. 绕路）按先编后换拒绝并点名 compile-then-swap.py，"
          "implementation-writer 与主 agent 写同一处、执行员写草稿目录与 crates/mutations.tsv 放行；Write 内容、Edit 与 MultiEdit 的 new_string 里有撇号类角标（五个字符各一例，主 agent 与子 agent 一样）拒绝，"
          "只在 old_string 里有、ASCII 单引号放行，共用字符集读不到拒绝；拒绝都记进检出记录")
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
code, message, finding = decide(hook_input, project_root, os.path.join(hook_dir, "agent-write-scope.tsv"), prime_marks_library_path(hook_dir))
if code:
    print(message, file=sys.stderr)
    record_detection(hook_input, finding, os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS)
sys.exit(code)
PY
