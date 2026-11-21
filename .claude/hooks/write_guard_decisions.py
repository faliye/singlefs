# 写闸 write-guard.sh 的 python 部分之一：各道判定（覆盖、写范围、写者锁、先编后换、撇号类角标、行尾空白、钟点、会话号、批次代号、任务号、账本、整份换上、语法）与检出记录。
# 入口 write-guard.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 WRITE_GUARD_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
# 同目录的 lib_*.py 按名字 import：入口的引导已把钩子目录（argv[1]）放在 sys.path 最前，部分文件自己不插。
import importlib.util, json, os, re, shutil, subprocess, sys, tempfile, time
from datetime import datetime, timezone

try:   # 工作根与会话临时目录根的唯一定义：同目录的 lib_work_root.py
    sys.dont_write_bytecode = True
    import lib_work_root as work_roots
    DEFAULT_DETECTIONS, work_roots_error = work_roots.default_detections_path(), None
except Exception as error:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, DEFAULT_DETECTIONS, work_roots_error = None, None, error

try:   # 草稿目录写者锁的判法与锁的格式：同目录的 lib_writer_lock.py（读不到时第九道不判，stderr 写一句并记检出）
    import lib_writer_lock as writer_lock
    writer_lock_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    writer_lock, writer_lock_error = None, error

# 写范围表的读法与判法（读表、展开工作根、glob 转正则、路径落没落进范围）在同目录的 lib_write_scope.py，bash-command-detector.sh 判 Bash 里的写用同一份；
# 读不到它时子 agent 的写按拒判（decide_scope），拒绝信息点名那份库
try:
    import lib_write_scope as write_scope
    write_scope_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    write_scope, write_scope_error = None, error

def expanded_scope_patterns(patterns):
    """写范围表里的模式：$SINGLEFS_WORK_ROOT/ 起头的展开成工作根那一条（弄坏开关 WRITE_GUARD_BREAK=scope-root-unanchored 时根后面不带 /）。"""
    return write_scope.expanded_scope_patterns(patterns, work_roots, os.environ.get("WRITE_GUARD_BREAK") != "scope-root-unanchored")

def glob_to_regex(pattern):
    return write_scope.glob_to_regex(pattern)

def load_scopes(table_path):
    return write_scope.load_scopes(table_path)

def absolute_target(hook_input, project_root):
    file_path = (hook_input.get("tool_input") or {}).get("file_path") or ""
    if not file_path:
        return ""
    return os.path.normpath(file_path if os.path.isabs(file_path) else os.path.join(project_root, file_path))

UNJUDGED = []   # 这一次调用里判不了、照常放行的几处：入口逐条已打过 stderr，收尾记一条检出（结局只记）

def note_unjudged(text):
    """判不了、照常放行的一处：stderr 写一行，记进 UNJUDGED 等入口记检出。弄坏开关 WRITE_GUARD_BREAK=unjudged-not-recorded 下只打 stderr。"""
    print(f"! write-guard.sh：{text}", file=sys.stderr)
    if os.environ.get("WRITE_GUARD_BREAK") != "unjudged-not-recorded":
        UNJUDGED.append(text)

def landing_lexical_only():
    return os.environ.get("WRITE_GUARD_BREAK") == "landing-lexical-only"

def landing_target(hook_input, project_root):
    """这一次写最终落在哪：absolute_target 规整 .. 之后逐段解开符号链接（目标不存在时解到最深的已存在那一层，lib_write_scope.landing_forms）。
    取的是 landing_forms 里「先规整 .. 再解开」那一种（工具先规整路径再打开）；解不开时交字面那一份并记一处没判全；弄坏开关 WRITE_GUARD_BREAK=landing-lexical-only 下交字面那一份。"""
    absolute = absolute_target(hook_input, project_root)
    if not absolute or landing_lexical_only():
        return absolute
    try:
        return os.path.realpath(absolute)
    except (OSError, ValueError) as error:
        note_unjudged(f"{absolute} 的符号链接解不开（{type(error).__name__}: {error}），覆盖、先编后换、语法、钟点与写者锁几道按字面路径判")
        return absolute

def landing_root(project_root):
    """和 landing_target 配对的仓根：也解开符号链接（仓根自己在链接底下时，落点照样算得出相对仓根的路径）。"""
    return os.path.normpath(project_root) if landing_lexical_only() else os.path.realpath(project_root)

def decide_overwrite(hook_input, project_root):
    """一、Write 整份覆盖未跟踪的已有文件。"""
    if hook_input.get("tool_name") != "Write" or os.environ.get("WRITE_GUARD_DISABLE_OVERWRITE") == "1":
        return 0, None
    target = landing_target(hook_input, project_root)
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

# 没有项目定义的 agent（内置类型、插件带来的、定义已删的旧实例）的写范围：只有工作根（文件头第二道；bash-command-detector.sh 的 ⑫ 用同一句模式）
BUILTIN_AGENT_SCOPE = ("$SINGLEFS_WORK_ROOT/**",)

def decide_builtin_scope(agent_type, absolute, project_root):
    """二之一：没有项目定义的 agent 只许写工作根（BUILTIN_AGENT_SCOPE 展开成工作根那一条），仓里与别处一律拒；落点照 lib_write_scope.landings_out_of_scope 解开符号链接。"""
    if work_roots is None:
        return 2, (f"✗ 读不到工作根的定义 .claude/hooks/lib_work_root.py（{work_roots_error!r}），没有项目定义的 {agent_type} 只许写工作根，这一次按拒判：{absolute}\n"
                   "→ 怎么办：先 git status 看那份库是被删了还是改坏了，用 Bash 恢复它；这件活交回主 agent。")
    try:
        outside = write_scope.landings_out_of_scope(expanded_scope_patterns(list(BUILTIN_AGENT_SCOPE)), absolute, project_root, landing_lexical_only())
    except (OSError, ValueError) as error:
        return 2, (f"✗ {absolute} 的符号链接解不开（{type(error).__name__}: {error}），算不出这次写落在哪，没有项目定义的 {agent_type} 的写按拒判\n"
                   "→ 怎么办：直接写链接指向的那个路径（ls -l 看它指到哪）；落在工作根之外的写进报告交回主 agent。")
    if not outside:
        return 0, None
    landed = (f"（解开符号链接之后落在 {'、'.join(outside)}）" if outside != [absolute] else "")
    return 2, (f"✗ {agent_type} 没有项目定义（.claude/agents/{agent_type}.md 不在：内置类型、插件带来的或定义已删的旧实例），只许写工作根，不含 {absolute}{landed}\n"
               f"→ 它能写的：{work_roots.work_root()}/ 下；仓里与别处（会话临时目录根 {work_roots.claude_session_temporary_root()}/ 在内）一律不写（用户 2026-10-26 定「只许写草稿目录」）。\n"
               "→ 怎么办：草稿、报告写进派发提示给的草稿目录；要改仓里的文件，写进报告交回主 agent，由主 agent 改或派有写范围的项目 agent（一次性的活派 clerk）。")

def decide_scope(hook_input, project_root, table_path):
    """二、项目 subagent 的写范围；没有项目定义的 agent 只许写工作根（decide_builtin_scope）。"""
    agent_type = hook_input.get("agent_type")
    if not agent_type:
        return 0, None
    defined = os.path.isfile(os.path.join(project_root, ".claude", "agents", f"{agent_type}.md"))
    if not defined and os.environ.get("WRITE_GUARD_BREAK") == "builtin-agents-unscoped":   # 弄坏开关：没有项目定义的一概放行
        return 0, None
    if os.environ.get("WRITE_GUARD_DISABLE_SCOPE") == "1":
        return 0, None
    file_path = (hook_input.get("tool_input") or {}).get("file_path") or ""
    if write_scope is None:
        return 2, (f"✗ 读不到写范围的判法 .claude/hooks/lib_write_scope.py（{write_scope_error!r}），{agent_type} 的写按拒判：{file_path}\n"
                   "→ 怎么办：先 git status 看那份库是被删了还是改坏了，用 Bash 恢复它（Edit / Write 这一刻都被这道闸拦着）；这件活交回主 agent。")
    if not defined:
        return decide_builtin_scope(agent_type, absolute_target(hook_input, project_root), project_root)
    patterns = load_scopes(table_path).get(agent_type, [])
    if not patterns:
        return 2, (f"✗ {agent_type} 在写范围表里没有登记，按拒绝处理：{file_path}\n"
                   f"→ 怎么办：在 .claude/hooks/agent-write-scope.tsv 按它定义的「写范围」一节补上路径模式；补之前这件活交回主 agent。")
    absolute = absolute_target(hook_input, project_root)
    try:   # 落点：规整 .. 并逐段解开符号链接，每一种认法都要在范围里（lib_write_scope.py 的 landings_out_of_scope）
        outside = write_scope.landings_out_of_scope(expanded_scope_patterns(patterns), absolute, project_root, landing_lexical_only())
    except (OSError, ValueError) as error:
        return 2, (f"✗ {absolute} 的符号链接解不开（{type(error).__name__}: {error}），算不出这次写落在哪，{agent_type} 的写按拒判\n"
                   "→ 怎么办：直接写链接指向的那个路径（ls -l 看它指到哪）；落在写范围之外的写进报告交回主 agent。")
    if not outside:
        return 0, None
    landed = (f"（解开符号链接之后落在 {'、'.join(outside)}）" if outside != [absolute] else "")
    unexpanded = (f"\n→ 读不到工作根的定义 .claude/hooks/lib_work_root.py（{work_roots_error!r}），表里 $SINGLEFS_WORK_ROOT/ 起头的模式按拒判："
                  "先 git status 看那份库是被删了还是改坏了，用 Bash 恢复它") if work_roots is None and any(p.startswith("$") for p in patterns) else ""
    return 2, (f"✗ {agent_type} 的写范围不含 {absolute}{landed}\n"
               f"→ 它的写范围：{'、'.join(patterns)}（.claude/hooks/agent-write-scope.tsv；$SINGLEFS_WORK_ROOT 是工作根 ~/.local/share/singlefs/work；会话临时目录根 /tmp/claude-<uid> 不在里面）。\n"
               f"→ 怎么办：范围外的改动不自己做，写进报告交回主 agent，由主 agent 改或派该写的 agent。{unexpanded}")

def decide_writer_lock(hook_input, project_root):
    """九、子 agent 写草稿根底下的文件，那个目录（或往上的某一层）有写者锁、而它不是持锁的那一个。判法与锁的格式在 lib_writer_lock.py。"""
    agent_id = hook_input.get("agent_id")
    if not agent_id or os.environ.get("WRITE_GUARD_DISABLE_WRITER_LOCK") == "1":
        return 0, None
    if writer_lock is None or work_roots is None:
        note_unjudged(f"写者锁这一道读不到 lib_writer_lock.py 或 lib_work_root.py（{(writer_lock_error or work_roots_error)!r}），这一次没判、照常放行")
        return 0, None
    target = landing_target(hook_input, project_root)
    if not target:
        return 0, None
    transcript_path = hook_input.get("transcript_path") or ""
    try:
        scratch = writer_lock.find_lock_directory(target, work_roots.scratch_roots(), any_entry=True)   # 嵌套的草稿目录只认最近一层的锁（lib_writer_lock.py 文件头「嵌套的草稿目录」）
        if not scratch:
            return 0, None
        state, lock = writer_lock.lock_state(scratch)
        if state == "absent":
            return 0, None
        if state == "unreadable":
            lock_file = writer_lock.lock_path(scratch)
            return 2, (f"✗ {target} 在草稿目录 {scratch} 里，这个目录的写者锁 {lock_file} 在、却判不了谁持锁（{lock}）：锁判不了的目录，子 agent 一律不写。\n"
                       "→ 怎么办：停下对这个目录的写，已经写的与没写完的在报告里交回主 agent，交回里点名这把锁的路径。\n"
                       f"→ 主 agent：核过这个目录现在归谁（.owner、在跑的子 agent）之后，删掉这把锁（rm {lock_file}；锁的位置是目录的 rm -r）再重派，"
                       "派发闸重建（提示里「草稿目录：」点名它）；锁判不了时派发闸不认「写者锁已判」接手，不删就派会被它拒。")
        verdict, holder = writer_lock.writer_verdict(scratch, lock, agent_id, transcript_path)
        if verdict == "holder":
            return 0, None
        if verdict == "unclaimed":   # 回执还没落盘：照「放行但必记检出」放行、不认领（lib_writer_lock.py 文件头「谁是持锁者」2）
            note_unjudged(f"写者锁这一道：{agent_id} 写 {target}，派发回执还没落盘，没认领 {scratch} 的写者锁"
                          f"（主会话记录 {transcript_path or '（输入里没有）'} 里还找不到它的回执，或记录读不了），这一次照常放行；回执落盘之后照旧核回执再认领")
            return 0, None
        if verdict == "broken-claim":
            claim_file = writer_lock.claim_path(scratch, lock)
            return 2, (f"✗ {target} 在草稿目录 {scratch} 里，这个目录的写者锁的认领文件 {claim_file} 在、却判不了谁持锁（{holder}）：认领判不了的目录，子 agent 一律不写。\n"
                       "→ 怎么办：停下对这个目录的写，已经写的与没写完的在报告里交回主 agent，交回里点名这份认领文件的路径。\n"
                       f"→ 主 agent：核过这个目录现在归谁（.owner、在跑的子 agent）之后，删掉这份认领文件（rm {claim_file}；是目录的 rm -r），"
                       "持锁者下一次写照派发回执重新认领；归属核不清的，换一个草稿目录重派。")
        unknown = None if holder else writer_lock.describe_unknown_holder(lock, agent_id, transcript_path)
    except Exception as error:   # 挂在每一次写上的闸，坏了不能把所有写都挡住
        note_unjudged(f"写者锁这一道自己出错，这一次放行：{type(error).__name__}: {error}")
        return 0, None
    owner = f"子 agent {holder}" if holder else f"{unknown}（你的派发回执不是这把锁登记的那一份提示，或早于这把锁）"
    return 2, (f"✗ {target} 在草稿目录 {scratch} 里，这个目录的写者锁（派发 {lock.get('subagent_type') or '?'}，派发于 {str(lock.get('dispatched_date'))[:10]}）由{owner}持有，"
               f"你（{agent_id}）不是：同一个草稿目录同一时刻只许持锁的那一个子 agent 写。\n"
               "→ 怎么办：停下对这个目录的写，已经写的与没写完的在报告里交回主 agent；主 agent 给你换一个没人持锁的草稿目录重派，"
               "或在派发提示里单起一行「写者锁已判：<理由>」接手这个目录（接手之后原持锁者写这个目录会被拒）。\n"
               "→ 规矩：锁的格式与谁算持锁者见 .claude/hooks/lib_writer_lock.py 文件头；Bash 里的写这道闸看不见，同样不许写进别人持锁的目录。")

# 四、先编后换：这几个项目 subagent 不直接写主工作区 crates/ 下的 .rs，出路是 scripts/hosts/compile-then-swap.py。
# Bash 里的同一类写法由 bash-command-detector.sh 的 ⑨ 判，agent 名单与路径判据两边各写一份、改的时候一起改。
COMPILE_FIRST_AGENTS = ("experiment-runner",)
COMPILE_THEN_SWAP_SCRIPT = "scripts/hosts/compile-then-swap.py"

def decide_compile_first(hook_input, project_root):
    """四、登记在 COMPILE_FIRST_AGENTS 里的 agent 写主工作区 crates/ 下的 .rs。"""
    agent_type = hook_input.get("agent_type")
    if agent_type not in COMPILE_FIRST_AGENTS or os.environ.get("WRITE_GUARD_DISABLE_COMPILE_FIRST") == "1":
        return 0, None
    absolute = landing_target(hook_input, project_root)
    root = landing_root(project_root)
    if not absolute or not absolute.endswith(".rs") or not absolute.startswith(os.path.join(root, "crates") + os.sep):
        return 0, None
    relative = os.path.relpath(absolute, root)
    return 2, (f"✗ {agent_type} 不直接写主工作区的 {relative}：入库装置先在草稿目录的副本里改，编过再整份换进主工作区，"
               "主工作区里任何时候都只放编得过的版本。\n"
               f"→ 怎么办：cp {relative} <草稿目录>/ 拷一份（新建的装置直接在草稿目录里建），在草稿目录那一份上用 Edit / Write 改；"
               + (f"改完跑 bash scripts/hosts/capped.sh <线程上限> python3 {COMPILE_THEN_SWAP_SCRIPT} <草稿目录>/{os.path.basename(relative)} "
                  f"{relative} --scratch <草稿目录>，" if os.environ.get("WRITE_GUARD_BREAK") == "compile-first-capped-hint" else
                  f"改完把 `task-server/task submit -- python3 {COMPILE_THEN_SWAP_SCRIPT} <草稿目录>/{os.path.basename(relative)} "
                  f"{relative} --scratch <草稿目录>` 写进报告「要主 agent 跑的命令」一节交回、由主 agent 交服务（子 agent 不往统一任务服务交任务），")
               + "它编过才整份换进来，编不过一个字节不写。\n"
               "→ 目标不是 crates/<crate>/src/bin/ 下的装置（crate 的库、测试）的，不归你改，写进报告交回主 agent。")

def notations_library_path(hook_dir):
    return os.path.join(os.path.dirname(hook_dir), "gate.d", "lib-forbidden-notations.py")

NOTATIONS_LIBRARIES = {}      # 库路径 → 装好的模块：一个进程里同一份库只装一次（一次写入判撇号、钟点、会话号三道，都从这里取）
NOTATIONS_LIBRARY_LOADS = [0]  # 真装了几次，自证数它

def load_notations_library(library_path):
    """门禁 doc-text 与本钩子共用的判据库 lib-forbidden-notations.py：装过的直接交回，没装过的装一次并记下；读不到、语法错抛异常（失败不记，下一回再试）。
    弄坏开关 WRITE_GUARD_BREAK=library-reloaded-each-time 每回重装。"""
    library = NOTATIONS_LIBRARIES.get(library_path)
    if library is not None and os.environ.get("WRITE_GUARD_BREAK") != "library-reloaded-each-time":
        return library
    spec = importlib.util.spec_from_file_location("lib_forbidden_notations", library_path)
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    NOTATIONS_LIBRARY_LOADS[0] += 1
    NOTATIONS_LIBRARIES[library_path] = library
    return library

def load_prime_marks(library_path):
    """撇号类字符集，与门禁 doc-text 同一份；读不到或读出来是空的就抛异常，由调用方按拒绝处理。"""
    library = load_notations_library(library_path)
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
               "→ 确实要删行尾空白的：用 Bash 跑 python3 scripts/kb/replace-once.py <文件> <旧串> <新串>（旧串恰好命中一次才写）。")

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

def library_unreadable_refusal(form_name, library_path, error):
    """判据库读不到时的拒绝说明。"""
    return (f"{LIBRARY_UNREADABLE_PREFIX}{form_name}判据 {library_path}（{type(error).__name__}：{error}），这次写没法判，按拒绝处理。\n"
            "→ 怎么办：那一份是门禁 doc-text 与这道 hook 共用的唯一定义，先 git status 看它是被删了还是被改坏了，"
            "用 Bash 恢复它（不在 hook 里另抄一份）；恢复之前往仓里的 Write / Edit 都会被拒。")

def library_hits_in_written_texts(hook_input, project_root, library_path, form_key, form_name):
    """写进仓里的内容在共用库那一形态下的每一处：(拒绝信息, None, []) 读不到库；(None, None, []) 没命中或不判；
    (None, 相对路径, [(在哪, 判据行名字, 命中的串)]) 命中（库的 all_hits，按出现的次序）。"""
    places = [(text, where) for text, where in written_text_places(hook_input, project_root) if text]
    absolute = landing_target(hook_input, project_root)
    root = landing_root(project_root)
    if not places or not absolute or os.path.commonpath([absolute, root]) != root:
        return None, None, []
    try:
        library = load_notations_library(library_path)
        relative = os.path.relpath(absolute, root)
        if library.is_excluded(form_key, relative):
            return None, None, []
        found = [(where(number), name, matched) for text, where in places for number, _, name, matched in library.all_hits(form_key, text, relative)]
    except Exception as error:
        return library_unreadable_refusal(form_name, library_path, error), None, []
    return None, relative, found

def library_hits_before_and_after(hook_input, project_root, library_path, form_key, form_name):
    """这一次写在共用库那一形态下多出来的每一处：写前（现文件，不在是空串）、写后（Write 的 content，Edit / MultiEdit 施加到现文件之后，edited_content）两份整文
    交库的 new_violation_hits，按违例相抵：(判据行名字, 命中的那段字面) 在两份整文里各数一遍，写后某一种比写前多出几处才算几处，在哪一行不论
    （.claude/gate.d/lib-forbidden-notations.py 的 new_violation_hits 文档串）。返回同 library_hits_in_written_texts，「在哪」是写后文件的行号。
    施加不了（Edit 的旧串找不到、文件不在）的不判，那一次 Edit 自己会拒。
    弄坏开关 WRITE_GUARD_BREAK=numbers-new-text-only：只把每一段新写进的文字（Write 的 content、各处的 new_string）当写后、写前当空串，看不到写完之后整份的样子。"""
    absolute = landing_target(hook_input, project_root)
    root = landing_root(project_root)
    if not absolute or os.path.commonpath([absolute, root]) != root:
        return None, None, []
    if os.environ.get("WRITE_GUARD_BREAK") == "numbers-new-text-only":
        pairs = [("", text, where) for text, where in written_text_places(hook_input, project_root) if text]
    else:
        after = edited_content(hook_input, absolute)
        if not after:
            return None, None, []
        try:
            before = open(absolute, encoding="utf-8", errors="replace").read() if os.path.isfile(absolute) else ""
        except OSError as error:
            note_unjudged(f"{absolute} 读不了（{type(error).__name__}: {error.strerror or error}），写前的样子算不出，{form_name}这一次没判、照常放行")
            return None, None, []
        pairs = [(before, after, lambda number: f"第 {number} 行")]
    try:
        library = load_notations_library(library_path)
        relative = os.path.relpath(absolute, root)
        if library.is_excluded(form_key, relative):
            return None, None, []
        found = [(where(number), name, matched) for before, after, where in pairs
                 for number, _, name, matched in library.new_violation_hits(form_key, before, after, relative)]
    except Exception as error:
        return library_unreadable_refusal(form_name, library_path, error), None, []
    return None, relative, found

def listed_hits(found, with_shape):
    """逐处一行：「  第 n 行（判据行名字）：「命中的串」」；超过 LISTED_HITS_LIMIT 处的只报另有几处。"""
    shown = [f"  {where}（{shape}）：「{matched}」" if with_shape else f"  {where}：「{matched}」" for where, shape, matched in found[:LISTED_HITS_LIMIT]]
    if len(found) > LISTED_HITS_LIMIT:
        shown.append(f"  （另有 {len(found) - LISTED_HITS_LIMIT} 处没列，改完上面这些再写一次会列出来）")
    return "\n".join(shown) + "\n"

COMPACT_TIMESTAMP_SHAPE = "ISO 紧凑时间戳"   # 库里 iso-compact-timestamp 那一条判据行的名字；只给弄坏开关 compact-timestamp-skipped 认形态用，判法在库里

def decide_clock_times(hook_input, project_root, library_path):
    """五、写进仓里的内容带描述性的钟点。"""
    if os.environ.get("WRITE_GUARD_DISABLE_CLOCK_TIMES") == "1":
        return 0, None
    unreadable, relative, found = library_hits_in_written_texts(hook_input, project_root, library_path, "clock-times", "钟点")
    if unreadable:
        return 2, unreadable
    if os.environ.get("WRITE_GUARD_BREAK") == "compact-timestamp-skipped":   # 弄坏开关：把紧凑时间戳那一形态的命中丢掉
        found = [hit for hit in found if hit[1] != COMPACT_TIMESTAMP_SHAPE]
    if not found:
        return 0, None
    return 2, (f"✗ 写进 {relative} 的内容在描述里写了钟点，共 {len(found)} 处：\n" + listed_hits(found, True) +
               "→ 怎么办：时间只写到日期，把钟点（时:分、「N 点 / N 时」、上午下午这类时段词）连同挨着它的时区词、「前后」一起删掉；"
               "光秃的时区词、ISO 时间戳（含运行目录名、日志文件名里连写的紧凑写法，路径里的也判：目录名里的时间戳段删掉）与 date 的原样输出也删，只写日期。\n"
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
               "→ 怎么办：会话号全名与后面的两位短号都不写进仓，别的会话写成 mainagent(A)、mainagent(B)，子 agent 写成 subagent(A)、subagent(B)，"
               "同一份文字里按出场次序编字母，写它改了哪个文件、做了什么，也不写「<做什么>那一路的会话」这类描述；"
               "标签式的「（会话 <号>，<日期>）」只留日期，事情与日期照留；改过标题的会话（「会话「<标题>」」）同样不写标题。\n"
               "→ .sh、.py、.rs 里代码真要用这些字面（解析 ListAgents 输出的正则、造会话号的自检输入），在那一行写「# session-names:allow <理由>」（Rust 写「//」）")

def decide_batch_codes(hook_input, project_root, library_path):
    """七之另一判：写进仓里的内容带批次代号（库里 batch-codes 那一形态，库按后缀只判 .md、.tsv、.txt）。弄坏开关 WRITE_GUARD_BREAK=batch-codes-unjudged 下不判。"""
    if os.environ.get("WRITE_GUARD_BREAK") == "batch-codes-unjudged":
        return 0, None
    unreadable, relative, found = library_hits_in_written_texts(hook_input, project_root, library_path, "batch-codes", "批次代号")
    if unreadable:
        return 2, unreadable
    if not found:
        return 0, None
    remedy = load_notations_library(library_path).FORMS_BY_KEY["batch-codes"]["remedy"]
    return 2, (f"✗ 写进 {relative} 的内容写了批次代号，共 {len(found)} 处：\n" + listed_hits(found, True) +
               "\n".join(f"→ {'怎么办：' if index == 0 else ''}{line}" for index, line in enumerate(remedy)))

def decide_task_and_session_numbers(hook_input, project_root, library_path):
    """七之第三判：这一次写让文件多出带任务号、括号里的会话短号或批号的行（库里 task-and-session-numbers 那一形态，库按后缀只判 .md、.tsv、.txt）：
    写前、写后两份整文交库判（library_hits_before_and_after），文件里已有的旧号不挡，这一次写让后面已有的行出了围栏、离开了「批次」那一列的照拒。
    弄坏开关 WRITE_GUARD_BREAK=task-and-session-numbers-unjudged 下不判。"""
    if os.environ.get("WRITE_GUARD_BREAK") == "task-and-session-numbers-unjudged":
        return 0, None
    unreadable, relative, found = library_hits_before_and_after(hook_input, project_root, library_path, "task-and-session-numbers", "任务号与批号")
    if unreadable:
        return 2, unreadable
    if not found:
        return 0, None
    remedy = load_notations_library(library_path).FORMS_BY_KEY["task-and-session-numbers"]["remedy"]
    return 2, (f"✗ 写进 {relative} 的内容写了任务号、会话短号或批号，共 {len(found)} 处（写前、写后整份各判一遍，写后多出来的行；文件里已有的旧号不算）：\n"
               + listed_hits(found, True) +
               "\n".join(f"→ {'怎么办：' if index == 0 else ''}{line}" for index, line in enumerate(remedy)))

LEDGER_REGISTRY = os.path.join(".claude", "gate.d", "split-proposal-ledgers.tsv")   # 拆分提案账本登记表（门禁 doc-process-records 的 deferred-items-registered、ledger-section-numbers、sealed-ledgers-unchanged 三格读同一张）
LEDGER_HEADING = re.compile(r"^##[ \t]+[零〇一二两三四五六七八九十百千]+、", re.M)   # 节标题的写法，同门禁 ledger-section-numbers 格认节的 HEADING

def current_ledger(project_root):
    """登记表里 current 那一行点名的账本 → (相对仓根的路径, None)；表读不到或 current 不是恰好一行 → (None, 原因)。"""
    registry = os.path.join(project_root, LEDGER_REGISTRY)
    try:
        with open(registry, encoding="utf-8") as handle:
            rows = [line.rstrip("\n").split("\t") for line in handle if line.strip() and not line.startswith("#")]
    except OSError as error:
        return None, f"读不到 {registry}（{type(error).__name__}: {error}）"
    current = [parts[1] for parts in rows if len(parts) == 4 and parts[0] == "current"]
    if len(current) != 1:
        return None, f"{registry} 里 current 的行有 {len(current)} 行，要恰好一行"
    return current[0], None

def ledger_heading_gain(hook_input, target):
    """这一次写让账本的「## <节号>、」标题净增几个：Write 是新内容比现有文件多出的；Edit 与 MultiEdit 是每一处新串比旧串多出的（改节号、改标题文字，个数不变，不算）。"""
    tool_name = hook_input.get("tool_name")
    tool_input = hook_input.get("tool_input") or {}
    if tool_name == "Write":
        try:
            with open(target, encoding="utf-8", errors="replace") as handle:
                existing = handle.read()
        except OSError:
            existing = ""
        pairs = [(existing, str(tool_input.get("content") or ""))]
    elif tool_name == "Edit":
        pairs = [(str(tool_input.get("old_string") or ""), str(tool_input.get("new_string") or ""))]
    elif tool_name == "MultiEdit":
        pairs = [(str((edit or {}).get("old_string") or ""), str((edit or {}).get("new_string") or "")) for edit in tool_input.get("edits") or []]
    else:
        return 0
    return sum(max(0, len(LEDGER_HEADING.findall(new)) - len(LEDGER_HEADING.findall(old))) for old, new in pairs)

def decide_ledger_append(hook_input, project_root):
    """十、往现行账本追加节标题。写进去的文字里没有节标题的不读登记表；登记表读不到照常放行并记一处没判全。弄坏开关 WRITE_GUARD_BREAK=ledger-append-ignored 下该拒的也放行。"""
    if not any(LEDGER_HEADING.search(text) for text in written_texts(hook_input)):
        return 0, None
    target = landing_target(hook_input, project_root)
    if not target:
        return 0, None
    relative, problem = current_ledger(project_root)
    if relative is None:
        note_unjudged(f"账本追加这一道没判：{problem}，这一次照常放行")
        return 0, None
    if os.path.realpath(os.path.join(project_root, relative)) != os.path.realpath(target):
        return 0, None
    gained = ledger_heading_gain(hook_input, target)
    if not gained or os.environ.get("WRITE_GUARD_BREAK") == "ledger-append-ignored":
        return 0, None
    return 2, (f"✗ 往现行账本 {relative} 追加「## <节号>、」标题（这一次写比原来多 {gained} 个节标题）：几个会话同时追加，各自读文件、各自取号，节号会撞，"
               "到提交时门禁 doc-process-records 的 ledger-section-numbers 格才判出重号。\n"
               "→ 怎么办：新节经 `python3 scripts/kb/ledger-append-section.py --title '<标题（含日期）>' --body-file <草稿目录里写好的正文文件>`：现取最大节号加一，"
               "一次写入索引表末行与末尾新节，写前复核 sha256，撞了拒（重跑取下一个号）；索引行的判定、历史行、当日还开着的几行用 --judgement、--history-rows、--open-rows 给。\n"
               "→ 改已有节的节号或标题文字（撞了号改成下一个空号）：新旧串里节标题个数相同的 Edit 放行，索引表里同步改。")

# 十一、这几个项目 subagent 不在主工作区里就地改已存在的脚本，出路是草稿目录里写完整份、经 scripts/gate/hook-swap.py 一次换上；
# 两个服务的 Rust 源码目录下已存在的文件同样不就地改，出路是仓副本与补丁。
# 目录与 hook-swap.py 的 SCRIPT_SPEC_DIRECTORIES 相同（它不收的 target/、src/ 在这里分在 SWAP_ONLY_EXCLUDED 与 SWAP_ONLY_SOURCE_DIRECTORIES），两边各写一份、改的时候一起改。
SWAP_ONLY_AGENTS = ("tooling-writer",)
SWAP_ONLY_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/scripts", "task-server", "prune-server", "gpu-verification-server")
SWAP_ONLY_EXCLUDED = (".claude/gate.d/fixtures", "prune-server/target", "gpu-verification-server/target")
SWAP_ONLY_FIXTURE_DIRECTORY = ".claude/gate.d/fixtures"   # 门禁样本档的根：第一层目录名是门禁阶段的文件名；已有门禁的那几个目录另走一支（existing_gate_of_fixture）
SWAP_ONLY_SOURCE_DIRECTORIES = ("prune-server/src", "gpu-verification-server/src")
SWAP_ONLY_SOURCE_FILES = ("prune-server/Cargo.toml", "prune-server/Cargo.lock", "gpu-verification-server/Cargo.toml", "gpu-verification-server/Cargo.lock")   # 两个服务的构建清单：与 src/ 一起改，改到一半同样编不过
TEMPORARY_SERVICE_TABLE = ".claude/hooks/selftest-temporary-service.tsv"   # 要在临时 prune-server 底下起的自证登记表（相对项目根）：一行「<相对仓根的路径><空白><required 或 own-server><空白># <理由>」
TEMPORARY_SERVICE_REQUIRED = "required"
TEMPORARY_SERVICE_WRAPPER = "scripts/prune-server/prune_server_client_selftest.py"   # required 的自证写成 task-server/task submit -- python3 <它> --with-temporary-service -- <解释器> <脚本> --selftest
SWAP_ONLY_SCRIPT_SUFFIXES = (".sh", ".py")
HOOK_SWAP_SCRIPT = "scripts/gate/hook-swap.py"
MAKE_SWAP_SPEC_SCRIPT = "scripts/gate/make-swap-spec.py"   # 第十一道出路里生成脚本规格的那一份（谁都能直接跑：lib_project_code_runs.py 的 TEXT_TOOLS_RUN_DIRECTLY）

def swap_only_directory(relative):
    """相对仓根的路径落在 SWAP_ONLY_SOURCE_DIRECTORIES 或 SWAP_ONLY_DIRECTORIES 里哪一处（SWAP_ONLY_EXCLUDED 下的不算）：交那一处（源码目录先认），不在交 None。"""
    def under(directory):
        return relative.startswith(directory + "/")
    if any(under(directory) for directory in SWAP_ONLY_EXCLUDED):
        return None
    return next((directory for directory in SWAP_ONLY_SOURCE_DIRECTORIES + SWAP_ONLY_DIRECTORIES if under(directory)), None)

def temporary_service_required(project_root, relative):
    """relative 在项目根下 TEMPORARY_SERVICE_TABLE 里的判定是不是 required：是交 True，不在表里或判定是 own-server 交 False；表读不到交 None 并记一处没判全。"""
    table = os.path.join(project_root, TEMPORARY_SERVICE_TABLE)
    try:
        with open(table, encoding="utf-8") as handle:
            for row in handle:
                parts = row.split()
                if len(parts) >= 2 and not parts[0].startswith("#") and parts[0] == relative:
                    return parts[1] == TEMPORARY_SERVICE_REQUIRED
    except (OSError, UnicodeDecodeError) as error:
        note_unjudged(f"第十一道（就地改已存在的脚本）：读不到 {table}（{type(error).__name__}: {error}），{relative} 要不要临时服务认不出，拒绝说明两种出路都写")
        return None
    return False

SWAP_WITH_VERIFY_REMEDY = "加 --verify '<被换脚本的 --selftest 命令>' 换上"
SWAP_WITHOUT_VERIFY_REMEDY = (
    f"不加 --verify 换上（这份脚本在 {TEMPORARY_SERVICE_TABLE} 里登记成 required：--verify 的命令在换上工具里起，拿不到任务服务给的编译目录与临时服务，退 2、回滚）；"
    f"自证换上之后另交一条：`task-server/task submit -- python3 {TEMPORARY_SERVICE_WRAPPER} --with-temporary-service -- <解释器> <脚本> --selftest`（.py 用 python3，别的用 bash），写进同一节。"
    "这样换没有自动回滚：自证验不过，要再出一份退回的规格（新旧两头对调生成，换上工具每次真换也在 <草稿目录>/hook-swap/ 留一份 reverse-<时刻>.json），由主 agent 再换一次；"
    "同一份规格里有一份登记成 required，整份规格都不加 --verify")

GATE_STAGE_DIRECTORY = ".claude/gate.d"   # 门禁阶段：这一层（不含子目录）下的 .sh；它们的验证是这一道的样本档，不在 TEMPORARY_SERVICE_TABLE 里
STAGE_SELFTEST_SCRIPT = ".claude/singlefs-ai-sop/scripts/stage-selftest.sh"   # 喂门禁阶段样本档的那一份（--item <阶段文件名> 只喂点名那一道）
SELFTEST_REGISTRY_TABLE = ".claude/hooks/selftest-registry.tsv"   # 自证与门禁阶段样本档的登记表：门禁阶段的一行写样本档要不要包在临时 prune-server 底下起
SWAP_GATE_STAGE_REMEDY = (
    f"不带 --verify 换上（门禁阶段不在 {TEMPORARY_SERVICE_TABLE} 里，它的验证是这一道的样本档：--verify 只起一条命令、在换上工具里起，"
    "拿不到任务服务给的编译目录与临时服务，而样本档有的要临时服务，起不了就退 2、回滚）；"
    f"样本档由主 agent 在换上之后另交一条：`task-server/task submit -- bash {STAGE_SELFTEST_SCRIPT} {GATE_STAGE_DIRECTORY} --item <阶段文件名>`，"
    f"{SELFTEST_REGISTRY_TABLE} 里这一道那一行写了样本档要包在临时 prune-server 底下起的，写成 "
    f"`task-server/task submit -- python3 {TEMPORARY_SERVICE_WRAPPER} --with-temporary-service -- bash {STAGE_SELFTEST_SCRIPT} {GATE_STAGE_DIRECTORY} --item <阶段文件名>`，写进同一节。"
    "这样换没有自动回滚：样本档验不过，要再出一份退回的规格（新旧两头对调生成，换上工具每次真换也在 <草稿目录>/hook-swap/ 留一份 reverse-<时刻>.json），由主 agent 再换一次；"
    "门禁阶段与别的脚本写在同一份规格里的，整份都不带 --verify，所以门禁阶段单放一份规格，别的脚本另放一份、照它们各自的出路")

def existing_gate_of_fixture(relative, root):
    """相对仓根的路径落在 SWAP_ONLY_FIXTURE_DIRECTORY/<门禁>/ 下、且 <root>/GATE_STAGE_DIRECTORY/<门禁> 是已存在的普通文件：交 <门禁>；别的交 None。
    判法在 lib_write_scope.existing_gate_of_fixture，bash-command-detector.sh ⑭ 判 Bash 里往样本档目录写用同一份。
    读不到那份库时交 None：子 agent 的写这时已由第二道（decide_scope）按拒判。弄坏开关 WRITE_GUARD_BREAK=gate-fixtures-in-place-allowed 下一律交 None。"""
    if os.environ.get("WRITE_GUARD_BREAK") == "gate-fixtures-in-place-allowed" or write_scope is None:
        return None
    return write_scope.existing_gate_of_fixture(relative, root)

def gate_fixture_refusal(agent_type, relative, gate):
    """第十一道已有门禁的样本档目录那一半的拒绝说明：出路是草稿目录里的补丁，主 agent 在门禁新版换上之后打上。"""
    return (f"✗ {agent_type} 不在主工作区里写已有门禁 {gate} 的样本档目录（{relative}；改已有的样本、在这个目录里新建样本都算）："
            f"样本与门禁新版要同一步落地，样本先进主工作区、门禁新版还在草稿里，别的会话这段时间跑 {gate} 的样本档会判错。\n"
            "→ 怎么办：样本的新版写在草稿目录：照仓里的相对路径，改前的样本拷进 <草稿目录>/fixtures-base/（只新建样本的建成空目录），改后的与新建的放进 <草稿目录>/fixtures-new/；"
            "在草稿目录里 `git diff --no-index --binary fixtures-base fixtures-new > fixtures.patch` 生成补丁；"
            "把 `git apply -p2 --check <草稿目录>/fixtures.patch` 与 `git apply -p2 <草稿目录>/fixtures.patch`（在仓根起）写进报告「要主 agent 跑的命令」一节，"
            f"排在门禁新版经 {HOOK_SWAP_SCRIPT} 换上那一条之后、样本档之前，由主 agent 打上（{HOOK_SWAP_SCRIPT} 的脚本规格只收已存在的文件，不收 {SWAP_ONLY_FIXTURE_DIRECTORY}/）。\n"
            f"→ 不拦的：{SWAP_ONLY_FIXTURE_DIRECTORY}/ 下还没有门禁阶段的样本目录（新门禁的样本与新门禁一起写），草稿目录与仓副本里的文件，主 agent 与别的 agent 类型。")

def is_gate_stage(relative):
    """相对仓根的路径是 GATE_STAGE_DIRECTORY 这一层的 .sh（门禁阶段）：交 True；这一层 lib- 起头的 .py 是门禁共用库（自证带 --selftest），子目录（lib/、fixtures/）下的与别的路径都交 False。"""
    return os.path.dirname(relative) == GATE_STAGE_DIRECTORY and relative.endswith(".sh")

def swap_verify_remedy(project_root, relative):
    """脚本规格换上那一步的写法：门禁阶段写不带 --verify、样本档另交（不读登记表）；登记成 required 的写不加 --verify、自证另交；不是的写加 --verify；表读不到两种都写。
    弄坏开关 WRITE_GUARD_BREAK=gate-stage-remedy-as-script 下门禁阶段与别的脚本走同一支；swap-remedy-always-verify 下一律写加 --verify（门禁阶段那一支不受它影响）。"""
    if is_gate_stage(relative) and os.environ.get("WRITE_GUARD_BREAK") != "gate-stage-remedy-as-script":
        return f"再去掉 --dry-run、{SWAP_GATE_STAGE_REMEDY}。"
    if os.environ.get("WRITE_GUARD_BREAK") == "swap-remedy-always-verify":
        return f"再去掉 --dry-run、{SWAP_WITH_VERIFY_REMEDY}。"
    required = temporary_service_required(project_root, relative)
    if required is None:
        return f"再去掉 --dry-run，按这份脚本在不在 {TEMPORARY_SERVICE_TABLE} 里登记成 required 二选一（表这一次读不到）：不在表里的{SWAP_WITH_VERIFY_REMEDY}；在表里的{SWAP_WITHOUT_VERIFY_REMEDY}。"
    if required:
        return f"再去掉 --dry-run、{SWAP_WITHOUT_VERIFY_REMEDY}。"
    return f"再去掉 --dry-run、{SWAP_WITH_VERIFY_REMEDY}。"

def decide_script_swap_only(hook_input, project_root):
    """十一、SWAP_ONLY_AGENTS 里的 agent 用 Write / Edit / MultiEdit 改主工作区 SWAP_ONLY_DIRECTORIES 下已存在的脚本（.sh、.py 或带执行位的普通文件），
    或 SWAP_ONLY_SOURCE_DIRECTORIES 下已存在的任一普通文件、SWAP_ONLY_SOURCE_FILES 里已存在的那几份构建清单（出路是仓副本与补丁），
    或已有门禁的样本档目录里的任一文件（新建也算；existing_gate_of_fixture，出路是草稿目录里的补丁）。
    认不出 agent 类型的子 agent 调用、目标 stat 失败的照常放行并记一处没判全。弄坏开关 WRITE_GUARD_BREAK=script-in-place-allowed 下该拒的也放行，
    swap-manifests-unseen 下构建清单那一半不拦。"""
    import stat
    if hook_input.get("tool_name") not in ("Write", "Edit", "MultiEdit") or os.environ.get("WRITE_GUARD_BREAK") == "script-in-place-allowed":
        return 0, None
    agent_type = hook_input.get("agent_type")
    recognized = isinstance(agent_type, str) and bool(agent_type)
    if recognized and agent_type not in SWAP_ONLY_AGENTS:
        return 0, None
    if agent_type is None and not hook_input.get("agent_id"):   # 主 agent：输入里没有 agent_type 也没有 agent_id
        return 0, None
    target = landing_target(hook_input, project_root)
    if not target:
        return 0, None
    relative = os.path.relpath(target, landing_root(project_root))
    if relative == ".." or relative.startswith("../") or os.path.isabs(relative):   # 落在主工作区之外：草稿目录、仓副本
        return 0, None
    gate = existing_gate_of_fixture(relative, landing_root(project_root))
    if gate is not None:
        if not recognized:
            note_unjudged(f"第十一道（已有门禁的样本档目录）：子 agent 的调用认不出 agent 类型（agent_type={agent_type!r}），{relative} 在已有门禁 {gate} 的样本档目录里，这一次没判、照常放行")
            return 0, None
        return 2, gate_fixture_refusal(agent_type, relative, gate)
    directory = swap_only_directory(relative)
    if directory is None:
        return 0, None
    try:
        status = os.stat(target)
    except (FileNotFoundError, NotADirectoryError):   # 新建文件
        return 0, None
    except OSError as error:
        note_unjudged(f"第十一道（就地改已存在的脚本）：{relative} stat 失败（{type(error).__name__}: {error.strerror or error}），认不出它在不在、是不是脚本，这一次没判、照常放行")
        return 0, None
    if not stat.S_ISREG(status.st_mode):
        return 0, None
    executable = bool(status.st_mode & 0o111)
    manifest = relative in SWAP_ONLY_SOURCE_FILES and os.environ.get("WRITE_GUARD_BREAK") != "swap-manifests-unseen"
    source = directory in SWAP_ONLY_SOURCE_DIRECTORIES or manifest
    if not source and not relative.endswith(SWAP_ONLY_SCRIPT_SUFFIXES) and not executable:   # 不带执行位的数据与文档
        return 0, None
    if not recognized:
        note_unjudged(f"第十一道（就地改已存在的脚本）：子 agent 的调用认不出 agent 类型（agent_type={agent_type!r}），{relative} 是主工作区里已存在的"
                      f"{'服务的构建清单' if manifest else '服务源码' if source else '脚本'}，这一次没判、照常放行")
        return 0, None
    if source:
        half_done = "构建清单改到一半（依赖或 feature 加了、src/ 里还没用上）" if manifest else "源码改到一半（加了枚举成员、各处 match 还没补全）"
        return 2, (f"✗ {agent_type} 不在主工作区里就地改 {directory}/ 下已存在的 {relative}：{half_done}，"
                   f"主工作区里的 {directory.split('/')[0]} 编不过，别的会话带临时判定库服务的自证都取不到二进制。\n"
                   "→ 怎么办：在仓副本里改（建法见 .claude/agent-common.md「建仓副本」），把改动存成补丁写进草稿目录，打补丁的命令写进报告「要主 agent 跑的命令」一节，由主 agent 一次打上。\n"
                   f"→ 不拦的：{directory}/ 下新建的文件，两个服务的 target/ 下的文件，草稿目录与仓副本里的文件。")
    suffix = os.path.splitext(relative)[1]
    kind = f"{suffix} 脚本" if relative.endswith(SWAP_ONLY_SCRIPT_SUFFIXES) else f"带执行位的{suffix or '无后缀'}文件"
    return 2, (f"✗ {agent_type} 不在主工作区里就地改已存在的脚本 {relative}（{kind}）：分几次改的中间态对别的会话可见，被拷进快照的任务、直接起它的服务会跑到改到一半的版本。\n"
               f"→ 怎么办：新版整份写在草稿目录（照主工作区此刻那一份拷过去再改），用 `python3 {MAKE_SWAP_SPEC_SCRIPT} --base <仓根> --new <草稿目录里的新版根> --out <草稿目录>/<规格.json> <相对路径>…`（子 agent 直接跑）生成 {HOOK_SWAP_SCRIPT} 的脚本规格（JSON 数组，每项 file / old / new，"
               "旧串在主工作区那一份里恰好命中 count 次；一件活改几份文件的写进同一份规格），把规格与那条换上命令写进报告「要主 agent 跑的命令」一节，由主 agent 一次换上："
               f"`task-server/task submit -- python3 {HOOK_SWAP_SCRIPT} <草稿目录>/<规格.json> --scratch <草稿目录> --root <仓根> --dry-run` 先验，"
               f"{swap_verify_remedy(project_root, relative)}\n"
               "→ 不拦的：新建文件，.tsv / .md / .json / .txt 这类不带执行位的数据与文档，.claude/gate.d/fixtures/ 下还没有门禁阶段的样本目录，草稿目录与仓副本里的文件；"
               "钩子（.claude/hooks/）照旧经 hook-swap.py 的钩子规格换上。")

SYNTAX_CHECKED_DIRECTORIES = (".claude/hooks", ".claude/gate.d", ".claude/scripts", "scripts")
PYTHON_SYNTAX_CHECK = "import ast, sys; ast.parse(open(sys.argv[1], encoding='utf-8').read(), sys.argv[1])"   # 只给弄坏开关 python-syntax-subprocess 用

def python_syntax_error(content, relative):
    """.py 的内容在本进程里 ast 解析（不写临时文件、不起子进程）：过得去交 None，过不去交错误说明（文件名写相对仓根的路径）。"""
    import ast, traceback
    try:
        ast.parse(content, filename=relative)
    except (SyntaxError, ValueError, RecursionError) as error:
        return "".join(traceback.format_exception_only(type(error), error)).strip()[-600:]
    return None

def syntax_error_by_checker(checker, content, suffix, relative):
    """内容写进临时文件、起 checker（bash -n）判：过得去交 None，过不去交错误说明；起不了、超时按放行交 None。"""
    handle = tempfile.NamedTemporaryFile("w", suffix=suffix, delete=False, encoding="utf-8")
    try:
        handle.write(content)
        handle.close()
        result = subprocess.run(checker + [handle.name], capture_output=True, text=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired) as error:
        note_unjudged(f"{' '.join(checker[:2])} 起不了或超时（{type(error).__name__}: {error}），{relative} 的语法这一次没判、照常放行")
        return None
    finally:
        handle.close()
        os.unlink(handle.name)
    if result.returncode == 0:
        return None
    return (result.stderr or result.stdout).strip().replace(handle.name, relative)[-600:]

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
    except OSError as error:   # 在而读不了：改完之后的内容算不出，语法这一道判不了，记一处没判全（弄坏开关 syntax-unreadable-unrecorded 下静默放行）
        if os.environ.get("WRITE_GUARD_BREAK") != "syntax-unreadable-unrecorded":
            note_unjudged(f"{target} 读不了（{type(error).__name__}: {error.strerror or error}），改完之后的内容算不出，语法这一次没判、照常放行")
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
    target = landing_target(hook_input, project_root)
    if not target:
        return 0, None
    relative = os.path.relpath(target, landing_root(project_root))
    if relative.startswith("..") or not any(relative.startswith(directory + "/") for directory in SYNTAX_CHECKED_DIRECTORIES):
        return 0, None
    if target.endswith(".sh"):
        checker, checker_name = ["bash", "-n"], "bash -n"
    elif target.endswith(".py"):
        checker, checker_name = None, "python3 的 ast 解析"
    else:
        return 0, None
    content = edited_content(hook_input, target)
    if content is None:
        return 0, None
    if checker is None and os.environ.get("WRITE_GUARD_BREAK") == "python-syntax-subprocess":   # 弄坏开关：.py 也另起 python3
        checker = [sys.executable, "-c", PYTHON_SYNTAX_CHECK]
    detail = python_syntax_error(content, relative) if checker is None else syntax_error_by_checker(checker, content, os.path.splitext(target)[1], relative)
    if detail is None:
        return 0, None
    return 2, (f"✗ 写进 {relative} 的内容过不了 {checker_name}：\n  {detail}\n"
               f"→ 怎么办：先在草稿目录里改到 `{checker_name}` 过，再写进来；钩子、门禁与脚本带着语法错误上线，之后每一次调用都会收到它的报错。")

def decide(hook_input, project_root, table_path, prime_marks_library, clock_times_library=None):
    """返回 (0, None, None) 放行；(2, 说明, 哪几道（「、」连）) 拒绝。十一道都判完，不成立的按下面的次序一次列全；
    先编后换拒了不再判写范围，钟点那一道读不到判据时会话号与批次代号不再报同一句。弄坏开关 WRITE_GUARD_BREAK=stop-at-first 判到第一道不成立就交回。"""
    if clock_times_library is None:
        clock_times_library = prime_marks_library
    judges = (("整份覆盖未跟踪文件", lambda: decide_overwrite(hook_input, project_root)),
              ("绕过先编后换", lambda: decide_compile_first(hook_input, project_root)),
              ("old_string 以空白收尾", lambda: decide_trailing_blank(hook_input)),
              ("写进语法错误", lambda: decide_shell_syntax(hook_input, project_root)),
              ("越出写范围", lambda: decide_scope(hook_input, project_root, table_path)),
              ("写进撇号类角标", lambda: decide_prime_marks(hook_input, prime_marks_library)),
              ("描述里写了钟点", lambda: decide_clock_times(hook_input, project_root, clock_times_library)),
              ("写了会话号", lambda: decide_session_names(hook_input, project_root, clock_times_library)),
              ("写了批次代号", lambda: decide_batch_codes(hook_input, project_root, clock_times_library)),
              ("写了任务号、会话短号或批号", lambda: decide_task_and_session_numbers(hook_input, project_root, clock_times_library)),
              ("写别的子 agent 持锁的草稿目录", lambda: decide_writer_lock(hook_input, project_root)),
              ("往现行账本追加节标题", lambda: decide_ledger_append(hook_input, project_root)),
              ("就地改已存在的脚本", lambda: decide_script_swap_only(hook_input, project_root)))
    refused = []   # [(哪一道, 说明)]
    for label, judge in judges:
        already = {name for name, _ in refused}
        if label == "越出写范围" and "绕过先编后换" in already:
            continue
        if (label in ("写了会话号", "写了批次代号", "写了任务号、会话短号或批号") and "描述里写了钟点" in already
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
    return 2, (f"✗ 这一次写有 {len(refused)} 道判定不成立（{labels}）：十一道都判完了，下面一次列全，逐道照各自的「→」改完再写。\n"
               + "\n".join(blocks)), labels

def record_detection(hook_input, finding, detections_path, outcome="被拒"):
    """检出记录追加一行：outcome「被拒」时 findings 是「写被拒：<哪几道>」，「只记」（判不了、照常放行）时 finding 是一串说明、各以「写闸没判全：」起头。
    写不进（没有路径、OSError）时 stderr 写一句。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": f"{hook_input.get('tool_name')} {tool_input.get('file_path', '')}"[:500],
        "findings": [f"写被拒：{finding}"] if outcome == "被拒" else [f"写闸没判全：{text}" for text in finding],
        "outcome": outcome,
    }
    if not detections_path:
        print("! write-guard.sh：读不到工作根的定义、又没设 AGENT_HOOK_DETECTIONS，这一条检出没地方记", file=sys.stderr)
        return
    try:
        os.makedirs(os.path.dirname(detections_path), exist_ok=True)
        with open(detections_path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError as error:
        print(f"! write-guard.sh：检出记录 {detections_path} 写不进（{error}）", file=sys.stderr)

INPUT_PREFIX_BYTES = 200
INPUT_FIELD = r'"{}"\s*:\s*"((?:[^"\\]|\\.)*)"'

def unparsable_input_entry(raw, problem):
    """输入不是合法的 JSON 对象：从原文里按字面找得到的会话号、agent 号与类型、记录路径照抄，找不到的会话号与类型写「读不到」。"""
    text = raw.decode("utf-8", "replace")
    def field(name):
        found = re.search(INPUT_FIELD.format(name), text)
        return found.group(1) if found else None
    prefix = raw[:INPUT_PREFIX_BYTES].decode("utf-8", "replace")
    return ({"session_id": field("session_id") or "读不到", "agent_id": field("agent_id"), "agent_type": field("agent_type") or "读不到",
             "transcript_path": field("transcript_path"), "tool_name": field("tool_name"), "tool_input": {"file_path": field("file_path") or ""}},
            f"输入不是合法的 JSON 对象（{problem}），这一次没判、照常放行；读到 {len(raw)} 字节，前 {INPUT_PREFIX_BYTES} 字节：{prefix!r}")

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
