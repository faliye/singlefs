# 检出钩子 bash-command-detector.sh 的 python 部分之一：python 这一层的读法、等待循环与放出追踪与看门狗、弄坏开关名单与对账、就地改仓内文件（⑧）、先编后换（⑨）、写范围（⑫）、服务源码（⑭）、起 claude（⑬）、清整个索引（⑮）、撤工作区改动（⑱）、前台 timeout（㉑）、cargo build 点全部目标（㉓）。
# 入口 bash-command-detector.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 DETECTOR_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
from __main__ import DETECTOR_BREAK_VARIABLE, detector_break_is_set, selftest_marked

import ast, glob, importlib.util, io, json, os, re, shlex, shutil, signal, subprocess, sys, tempfile, tokenize
from datetime import datetime, timezone
from typing import NamedTuple

# 钩子目录：部分文件与入口在同一个目录里（入口的引导把 argv[1] 放在 sys.path 最前、再 import 它们），按本文件自己的位置取。
# 同目录的 lib_*.py 按名字 import（入口已把它放进 sys.path）或按文件路径载入，路径都从这一个取；算键库认得 __file__、不认部分文件里的 sys.argv。
HOOK_DIRECTORY = os.path.dirname(os.path.abspath(__file__))

def shell_words_library_path(hook_dir):
    return os.path.join(hook_dir, "lib_shell_words.py")

SHELL_WORDS_PART_NAMES = ("lib_shell_words_core.py", "lib_shell_words_embedded.py")   # lib_shell_words.py 按自己的目录导入的同目录部分：自证拷库到临时钩子目录时一起拷

def load_shell_words(library_path):
    """切词与认命令位置的共用模块，与 project-code-guard.sh 同一份；读不到或导入出错就抛异常，由调用方处理。"""
    spec = importlib.util.spec_from_file_location("lib_shell_words", library_path)
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    return library

try:
    shell_words, shell_words_error = load_shell_words(shell_words_library_path(sys.argv[1])), None
except Exception as error:  # 文件不在、语法错、导入时抛的都算读不到
    shell_words, shell_words_error = None, error
try:   # 工作根与会话临时目录根的唯一定义：同目录的 lib_work_root.py（读不到时 ⑦ 只认仓里、⑧ 只放行 /tmp/ 下，检出记录不写）
    sys.dont_write_bytecode = True
    import lib_work_root as work_roots
    # ⑦ 认的根：草稿根（工作根）加会话临时目录根（各会话的 scratchpad 在里面，那里的脚本同样有 bash 在读）
    DEFAULT_DETECTIONS = work_roots.default_detections_path()
    SCRIPT_SCRATCH_ROOTS = tuple(dict.fromkeys((*work_roots.scratch_roots(), work_roots.claude_session_temporary_root())))
except Exception as error:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, DEFAULT_DETECTIONS, SCRIPT_SCRATCH_ROOTS = None, None, ()

WAIT_LOOP = re.compile(r"\b(until|while)\b[^\n]*?;\s*do\b[\s\S]*?\bsleep\b")
SELF_DETACH = re.compile(
    r"\bsetsid(?:\s+-[-\w]+)*?\s+(?:--fork\b|-[a-eg-z]*f[a-z]*\b)"
    r"|\bdisown\b|\bcoproc\b"
    r"|\bnohup\b[^\n]*(?<![&>|<])&(?![&>])"
    r"|\btmux\b[^\n]*\bnew(?:-session)?\b[^\n]*\s-d\b"
    r"|\bscreen\b[^\n]*\s-(?:[dD]m\w*|d\s+-m)"
    r"|\bsystemd-run\b(?![^\n]*--(?:wait|pipe|pty|scope)\b)"
    r"|\bstart-stop-daemon\b[^\n]*\s(?:-b|--background)\b")
LONE_AMPERSAND = re.compile(r"(?<![&>|<])&(?![&>])")
WAIT_COMMAND = re.compile(r"(?:^|[;&|\n({]|\b(?:then|do|else)\b)\s*wait\b(?![\w.(-])([^;&|\n)}]*)")
SUBSHELL_AMPERSAND = re.compile(r"(?<![&>|<])&\s*\)|\$\([^()]*(?<![&>|<])&(?![&>])[^()]*\)")
POLL_AFTER = re.compile(r"\btail\b[^\n]*--pid\b|\b(?:until|while)\b[^\n]*\bkill\s+-0\b")

def bare_setsid_forms(command):
    """命令位置上（每一层，前缀、capped.sh / run-with-memory-cap.sh 这类包装剥开）带 setsid 前缀、又没带 -w / --wait 的那几条（带 -f / --fork 的归 ③ 拒，这里不重复）。
    setsid 只在它自己是进程组组长时 fork、父进程马上退出；会不会是组长，执行前看不出（一百三十七节件 1b 第 5 条）。"""
    found = []
    for layer in shell_layers(command):
        for words in layer.commands:
            for prefix_words, _, name, arguments in wrapped_commands(words):
                for index, word in enumerate(prefix_words):
                    if os.path.basename(word) != "setsid":
                        continue
                    options = []
                    for option in prefix_words[index + 1:]:
                        if not option.startswith("-") or option == "-":
                            break
                        options.append(option)
                    short_letters = "".join(option[1:] for option in options if not option.startswith("--"))
                    if "--wait" in options or "w" in short_letters or "--fork" in options or "f" in short_letters:
                        continue
                    found.append(" ".join([word, *options, name or ""]).strip()[:120])
    return list(dict.fromkeys(found))

# 一次判定里判不了的几样（某一道抛了异常、某个库读不到）：not_judged 打 stderr 并收进这里，main 把它们记进检出（拒了的那一条也记）
JUDGING_NOTICES = []

def not_judged(message):
    """判不了的一样：stderr 打一行，收进 JUDGING_NOTICES 由 main 记检出；交回这一句。"""
    print(f"  ! {message}", file=sys.stderr)
    JUDGING_NOTICES.append(message)
    return message

HOOK_INPUT_HEAD_BYTES = 200   # hook 输入不是 JSON 对象时，诊断里贴它开头这么多字节
# hook 输入读不出来时记检出用的那一份：会话号、agent 都读不到，照实写读不到
UNREADABLE_HOOK_INPUT = {"session_id": "读不到", "agent_id": "读不到", "agent_type": "读不到", "transcript_path": "读不到", "tool_input": {"command": ""}}

def parsed_hook_input(raw):
    """标准输入读到的字节 → (hook 输入, None)；不是 JSON 对象（解析出错、解出来不是对象）→ (None, 一行诊断：钩子名、为什么、开头若干字节)。"""
    try:
        hook_input = json.loads(raw)
    except Exception as error:   # 不是合法 JSON、不是 UTF-8
        why = f"解析出错：{error!r}"
    else:
        if isinstance(hook_input, dict):
            return hook_input, None
        why = f"解出来是 {type(hook_input).__name__}，不是对象"
    head = raw[:HOOK_INPUT_HEAD_BYTES]
    return None, (f"bash-command-detector.sh 收到的 hook 输入不是 JSON 对象（{why}），这条命令没判、照常执行；"
                  f"开头 {len(head)} 字节（共 {len(raw)} 字节）：{head!r}")

def append_detection(hook_input, findings, detections_path, outcome="只记"):
    """检出记录追加一行；outcome 是这条命令的结局：只记不拦的写「只记」，判不了的几样跟着一条被拒的命令记时写「被拒」。"""
    tool_input = hook_input.get("tool_input") or {}
    command = tool_input.get("command") or ""
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": (command if isinstance(command, str) else repr(command))[:500],
        "findings": findings,
    }
    if not detector_break_is_set("detection-outcome-dropped"):
        entry["outcome"] = outcome   # 照常执行的写「只记」；拒了的命令只在带着判不了的几样时才记，写「被拒」。看门狗按这一栏说下一步
    if not detections_path:   # 读不到 lib_work_root.py、又没设 AGENT_HOOK_DETECTIONS：不知道往哪写
        return
    os.makedirs(os.path.dirname(detections_path), exist_ok=True)
    with open(detections_path, "a", encoding="utf-8") as handle:
        handle.write(json.dumps(entry, ensure_ascii=False) + "\n")

# 起看门狗的写法：只认命令位置，所以先按 shell 的规矩切词（共用模块），不拿正则在整条文本上找名字。
WATCH_SH_MODES_THAT_DO_NOT_WATCH = {"--selftest", "--report", "--dry-run"}

def is_watchdog_call(command):
    name, arguments = os.path.basename(command.words[0]), command.words[1:]
    if name == "watch.sh":
        return not WATCH_SH_MODES_THAT_DO_NOT_WATCH.intersection(arguments)
    return name == "agent-watch.py" and "watch" in arguments and arguments[:1] != ["--selftest"]

def null_redirects_reaching_watchdog(scan):
    """整条命令里输出重定向到 /dev/null 的处数，减去挂在别的简单命令上、碰不到看门狗输出的那几处：那一条不是看门狗、不在管道后段，或在管道后段而只改 2 号这类非标准输出。
    共用模块的 output_targets 收全部层的重定向；每条命令的 output_redirections 只收它自己那一条简单命令上的（同一条简单命令交出的几条共用一个元组，按元组认一次）。
    包着看门狗的那一层（bash -c 整条、花括号与圆括号整组、exec）上的重定向不挂在哪一条交出来的命令上，减不掉，照算。"""
    groups = {}
    for command in scan.commands:
        redirections, carries_watchdog, piped = groups.get(id(command.output_redirections), (command.output_redirections, False, False))
        groups[id(command.output_redirections)] = (redirections, carries_watchdog or is_watchdog_call(command), piped or command.piped_from is not None)
    elsewhere = sum(1 for redirections, carries_watchdog, piped in groups.values() if not carries_watchdog
                    for operator, target in redirections if target == "/dev/null" and not (piped and shell_words.redirects_standard_output(operator)))
    return scan.output_targets.count("/dev/null") - elsewhere

def watchdog_calls_and_detaching(text):
    """返回 (命令位置上起看门狗的调用, 整条命令里放后台或丢输出的写法)；bash -c、capped.sh 这类包装里的由共用模块递归进去一起交回。"""
    scan = shell_words.commands_at_command_position(text)
    calls, detaching = [], []
    if scan.has_lone_ampersand:
        detaching.append("命令里有单独的 &")
    if ("/dev/null" in scan.output_targets if detector_break_is_set("watchdog-null-redirect-whole-command")
            else null_redirects_reaching_watchdog(scan) > 0):
        detaching.append("输出重定向到了 /dev/null")
    detaching += [f"命令里有 {prefix}" for prefix in ("nohup", "setsid") if prefix in scan.prefix_names]
    for command in scan.commands:
        name, arguments = os.path.basename(command.words[0]), command.words[1:]
        if name == "disown":
            detaching.append("命令里有 disown")
        if (name == "tee" and "/dev/null" in [argument for argument in arguments if not argument.startswith("-")]
                and not detector_break_is_set("watchdog-tee-null-unseen")):
            detaching.append("输出经 tee 丢进了 /dev/null")   # `watch.sh | tee /dev/null`
        if is_watchdog_call(command):
            calls.append(" ".join([name, *arguments]))
    return calls, detaching

def watchdog_rejection(command, run_in_background):
    """起看门狗的写法不对就返回 (原因, 认出的调用)；写法对、或根本没起看门狗，原因为空。"""
    if detector_break_is_set("allow-watchdog-misuse"):
        return [], []
    calls, detaching = watchdog_calls_and_detaching(command)
    if not calls:
        return [], []
    reasons = [] if run_in_background is True else ["run_in_background 不是 true"]
    reasons += list(dict.fromkeys(detaching))
    return reasons, calls

# 前台没超时的等待循环：只认命令位置，按共用模块切出的简单命令与开头的关键字找循环的头尾，不拿正则在整条文本上找
WAIT_LOOP_KEYWORDS = {"while", "until"}
BLOCKS_CLOSED_BY_DONE = {"while", "until", "for", "select"}

def loop_keyword_changes(words):
    """一条简单命令开头的关键字：交回 (开了几层要 done 收的块, 收了几层, 开的是不是 while / until)。"""
    opened, closed, opens_wait_loop = 0, 0, False
    for word in words:
        if word == "done":
            closed += 1
        elif word in BLOCKS_CLOSED_BY_DONE:
            opened, opens_wait_loop = 1, word in WAIT_LOOP_KEYWORDS
            break
        elif word not in shell_words.COMMAND_KEYWORDS:
            break
    return opened, closed, opens_wait_loop

def is_sleep_command(words):
    """剥掉开头的关键字与 nice / timeout 这类前缀之后，命令词是不是 sleep。"""
    skipped = shell_words.skip_prefixes(words)
    return skipped.command_position is not None and os.path.basename(words[skipped.command_position]) == "sleep"

try:   # 写范围表的读法与判法：同目录的 lib_write_scope.py，与 write-guard.sh 同一份（读不到时 ⑫ 不判，stderr 报一句）
    import lib_write_scope as write_scope
    write_scope_error = None
except Exception as error:   # 文件不在、语法错都算读不到
    write_scope, write_scope_error = None, error

# scripts/hosts/two-logs.sh [--quiet] <名> -- <命令…>（日志出两份的包装）：wrapped_commands 把它剥掉，`--` 之后那条命令照原判法判（等待循环、放出追踪、终止进程、覆盖产物、先编后换、⑫ 写范围……）。
#   认法与剥法只有 lib_project_code_runs.py 的 command_under_two_logs 一份（项目代码闸与范围声明闸剥它用的也是这一个），按文件路径导入，第一次用到才装（它有几千行，没有 two-logs.sh 的命令不必付这个钱）；
#   装不上时 two-logs.sh 当不透明的命令（里面那条看不见），stderr 报一句
TWO_LOGS_SCRIPT_NAME = "two-logs.sh"
PROJECT_CODE_RUNS_LIBRARY = []   # 空表是还没装；装过的放一个模块，装不上放 None

def project_code_runs_library():
    if not PROJECT_CODE_RUNS_LIBRARY:
        try:
            specification = importlib.util.spec_from_file_location("lib_project_code_runs", os.path.join(HOOK_DIRECTORY, "lib_project_code_runs.py"))
            module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(module)
            PROJECT_CODE_RUNS_LIBRARY.append(module)
        except Exception as error:   # 文件不在、语法错、它导入 lib_shell_words 失败
            not_judged(f"bash-command-detector.sh 读不到 lib_project_code_runs.py（{error!r}），scripts/hosts/two-logs.sh 包着的命令这一次没剥开判")
            PROJECT_CODE_RUNS_LIBRARY.append(None)
    return PROJECT_CODE_RUNS_LIBRARY[0]

def two_logs_inner_words(words):
    """words[0] 是 scripts/hosts/two-logs.sh 而参数是 `[--quiet] <名> -- <命令…>` 时交回 <命令…> 的词，不是交 None。"""
    library = project_code_runs_library()
    inner = library.command_under_two_logs(words) if library is not None else None
    return inner[0] if inner else None

# 弄坏开关 BASH_COMMAND_DETECTOR_BREAK=<项>（几个用逗号连；只给自检证明点名的几格会红，项名与红的格写在文件头）
# 变量名 DETECTOR_BREAK_VARIABLE 与读法 detector_break_is_set 定义在入口的引导里：引导在这几份载入之前就要读它
KNOWN_DETECTOR_BREAK_ITEMS = ("disable-check", "disable-self-background", "keep-heredoc-bodies", "allow-watchdog-misuse", "allow-foreground-wait-loop", "allow-subagent-background", "allow-detaching",
                              "allow-unwaited-ampersand", "allow-background-task-output-wait", "allow-repository-in-place-edit", "allow-compile-first-bypass", "allow-results-overwrite",
                              "allow-process-signals", "ignore-private-config", "allow-script-in-place-write", "keep-selftest-scratch", "refuse-every-wait-loop", "stop-at-first-refusal",
                              "detection-outcome-dropped", "allow-work-root-removal", "always-read-process-table", "per-path-git-query", "opaque-two-logs", "allow-bash-scope-write",
                              "scope-known-scripts-unseen", "scope-main-agent-judged", "scope-undeterminable-only-noted", "scope-loop-values-unknown", "scope-loop-landings-trusted",
                              "scope-home-unexpanded", "scope-landing-lexical-only", "heredoc-body-judged-as-commands", "sed-in-place-ignores-cd", "heredoc-pipe-fed-as-data",
                              "pwd-variable-unexpanded", "mktemp-directory-unknown", "mktemp-scratch-template-unknown", "mktemp-template-owner-unchecked", "mktemp-option-directory-unknown",
                              "mktemp-nested-substitution-unknown", "break-honored-outside-selftest", "pending-outside-scanned-as-stale", "failure-unrecorded", "allow-subagent-claude-launch",
                              "allow-long-foreground-timeout", "python-move-into-repository-unseen", "alias-unresolved", "dynamic-launcher-unseen", "long-option-prefix-unseen",
                              "allow-whole-target-build", "whole-target-full-gate-unseen", "whole-target-howto-unnamed-test", "scope-script-variable-unexpanded", "builtin-agents-unjudged",
                              "allow-whole-index-reset", "whole-index-staged-unchecked", "whole-index-pathspec-ignored", "break-honored-outside-selftest-with-others", "allow-ledger-append",
                              "allow-path-discard", "path-discard-changes-unchecked", "path-discard-staged-counted", "scope-ledger-append-unseen", "scope-record-append-unseen",
                              "scope-record-append-root-as-target", "scope-insert-row-absent-literal-as-target", "python-relative-path-ignores-cd", "compile-first-ignores-cwd",
                              "claude-launch-layers-narrow", "watchdog-tee-null-unseen", "rsync-inplace-unseen", "partial-wait-allowed", "wait-forms-unseen", "whole-index-other-forms-unseen",
                              "git-subcommand-unexpanded-allowed", "brace-target-reason-generic", "wait-loop-rewrite-unexplained", "coproc-prefix-unseen", "bash-layer-unrecorded",
                              "startup-file-claude-unread", "python-missing-unrecorded", "script-body-claude-unread", "embedded-launch-claude-unread", "background-ampersand-by-regex",
                              "bare-setsid-unrecorded", "scope-launcher-writes-unseen", "sed-unknown-target-resolved-from-cwd", "watchdog-null-redirect-whole-command",
                              "in-place-script-session-temporary-root-unseen", "builtin-coproc-detached", "compile-first-capped-hint", "allow-literal-notations", "literal-notations-outside-judged",
                              "literal-notations-new-text-only", "literal-notations-before-from-disk", "literal-printf-format-unexpanded", "literal-echo-escapes-unexpanded",
                              "literal-variables-unsubstituted", "allow-record-append", "record-append-python-unseen", "record-append-tee-unseen",
                              "record-howto-unlisted-tool", "howto-guard-environment-keeps-breaks", "own-session-group-unrecognized", "own-session-group-session-unchecked",
                              "own-session-group-first-binding-only", "allow-manual-repository-copy", "manual-copy-howto-unlisted-tool", "perl-in-place-unseen", "perl-switch-cluster-by-regex",
                              "python-script-body-unread", "python-script-any-location-read", "python-script-unliteral-write-unnoted", "allow-service-source-write", "service-source-overwrite-only",
                              "service-source-lexical-only", "service-source-backup-kept", "service-source-directory-copy-unseen", "service-source-link-unseen",
                              "service-source-hardlink-inode-unseen", "service-source-extract-unseen", "service-source-removal-unseen", "service-source-unlink-find-unseen",
                              "service-source-rsync-removal-unseen", "service-source-compressor-unseen", "service-source-in-place-tools-unseen", "service-source-read-write-redirect-unseen",
                              "service-source-unzip-unseen", "service-source-python-delete-unseen", "service-source-python-in-place-unseen", "service-source-stdout-extract-judged",
                              "service-source-patch-operand-ignored", "gate-fixtures-unseen", "scope-swap-spec-out-unseen", "whole-target-howto-clippy-lints-dropped", "scope-read-only-definitions-unjudged",
                              "break-switch-sets-one-way", "break-switch-indirect-reads-unseen", "allow-git-hook-guard", "bypass-flag-unseen", "bypass-config-unseen", "hooks-not-installed-allowed",
                              "bypass-every-git-command", "git-hook-messages-unguarded", "scope-writer-lock-unchecked", "break-switch-table-narrow",
                              "merge-bypass-unseen", "am-bypass-unseen", "config-section-unseen", "hooks-path-right-value-refused", "hooks-path-relative-accepted",
                              "scope-leg-round-unchecked", "scope-owner-only-directory-unjudged", "scope-unclaimed-receipt-unrecorded", "part-unreadable-passes",
                              "python-loop-variable-unexpanded")

# 弄坏开关的名字在四处写：名单 KNOWN_DETECTOR_BREAK_ITEMS、文件头开关表、代码里读的名字、各节文字里「弄坏开关：」那一串；自证从这份钩子的源文件现读（break_switch_name_sets），
#   不是照一份写死的名字：新加开关写全，缺哪一处自证点名那个名字与缺在哪一处。读法按开关表的文法：
#   文件头开关表：开头那一段连续的注释行里，`BASH_COMMAND_DETECTOR_BREAK=<项>` 起头的行（与门禁 code-tooling.sh 的 SWITCH_ENTRY 同一种：缩进随意，值是小写起头的字母、数字与 _ . , : -，
#     值里逗号隔开的每一项各算一条，值后面紧接两个以上空格或（…）注；值后面只隔一个空格接命令的是用法示例，不算），与 BREAK_SWITCH_BATCH_MARKER 那一行之后的 `#   <项>  说明` 行（几件批量登记的那几段）；
#     写在表的位置上而读不成一条的（名字套反引号、等号后换行、同一行写第二条、缺值）点名那一行、不算进表里：门禁读不到它，它的复跑与三样写全都不判；说明文字里 `<项>  …` 起头的行不是开关；
#   代码里读的名字（注释行不算）：`detector_break_is_set(…)` 的参数是双引号或单引号的字面，或模块里 `<名字> = "<项>"` 登记过的常量名（读不出的写法点名），GUARD_OFF_ITEM 与
#     GUARD_OFF_WITH_OTHERS_ITEM 的值，bash 层 `[[ ",${BASH_COMMAND_DETECTOR_BREAK:-}," != *,<项>,* ]]` 与 `== *,<项>,*` 里的项；
#   各节文字里「弄坏开关：」到句号为止，（…）注之外按「、」隔开的每一项（写全名，不写 `-后缀` 的缩写）；这一处只查一个方向：写着的项都要在名单里（删了开关、改了名之后那一串还写着的判红）。
BREAK_SWITCH_KNOWN_TUPLE = re.compile(r"^KNOWN_DETECTOR_BREAK_ITEMS = \((.*?)\)[ \t]*$", re.M | re.S)
BREAK_SWITCH_TABLE_ENTRY = re.compile(r"^\s*#?\s*BASH_COMMAND_DETECTOR_BREAK=(?P<value>[a-z0-9][A-Za-z0-9_.,:-]*)(?P<rest>.*)$")
BREAK_SWITCH_TABLE_POSITION = re.compile(r"^#\s+BASH_COMMAND_DETECTOR_BREAK=(?!<)")
BREAK_SWITCH_SECOND_MENTION = "BASH_COMMAND_DETECTOR_BREAK="
BREAK_SWITCH_BATCH_MARKER = "# 批量登记的开关"
BREAK_SWITCH_BATCH_ENTRY = re.compile(r"^#   ([a-z][a-z0-9]*(?:-[a-z0-9]+)+)(?:  |$)")
BREAK_SWITCH_DIRECT_READ = re.compile(r"detector_break_is_set\(\s*(?:\"([a-z0-9-]+)\"|'([a-z0-9-]+)'|([A-Za-z_][A-Za-z0-9_]*))\s*\)")
BREAK_SWITCH_DIRECT_READ_LITERAL = re.compile(r"detector_break_is_set\(\s*\"([a-z0-9-]+)\"")
BREAK_SWITCH_CONSTANT = re.compile(r"^([A-Z][A-Z0-9_]*) = [\"']([a-z0-9-]+)[\"']", re.M)
BREAK_SWITCH_GUARD_OFF = re.compile(r"^(?:GUARD_OFF_ITEM|GUARD_OFF_WITH_OTHERS_ITEM) = \"([a-z0-9-]+)\"", re.M)
BREAK_SWITCH_BASH_LAYER = re.compile(r"[!=]= \*,([a-z0-9-]+),\*")
BREAK_SWITCH_SECTION_LIST = re.compile(r"弄坏开关：(.*?)。", re.S)
BREAK_SWITCH_PLACES = ("名单", "文件头开关表", "读开关的代码")
BREAK_SWITCH_SECTION_NAME = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)+(?:；.*)?", re.S)
BREAK_SWITCH_SECTION_ABBREVIATION = re.compile(r"-[a-z0-9]+(?:-[a-z0-9]+)*(?:；.*)?", re.S)

def break_switch_name_sets(source):
    """钩子源文件的正文 → (名单里的名字, 文件头开关表里的名字, 代码里读的名字, 各节「弄坏开关：」里的名字, 读不出的写法的说明列表)。
    弄坏开关 break-switch-indirect-reads-unseen 下代码里读的名字只认 detector_break_is_set("<项>") 的双引号字面；
    break-switch-table-narrow 下文件头开关表照一行一个名字、不认逗号隔开的几项与批量登记那一段的行首写法、也不点名读不成一条的行。"""
    lines = source.split("\n")
    header_end = next((index for index, line in enumerate(lines) if line.strip() and not line.startswith("#")), len(lines))
    header = lines[:header_end]
    code = "\n".join(line for line in lines[header_end:] if not line.lstrip().startswith("#"))
    problems = []
    known_tuple = BREAK_SWITCH_KNOWN_TUPLE.search(code)
    known = set(re.findall(r"\"([a-z0-9-]+)\"", known_tuple.group(1))) if known_tuple else set()
    table, in_batch = set(), False
    narrow = detector_break_is_set("break-switch-table-narrow")
    for number, row in enumerate(header, 1):
        if row.startswith(BREAK_SWITCH_BATCH_MARKER):
            in_batch = True
            continue
        entry = BREAK_SWITCH_TABLE_ENTRY.match(row)
        readable = entry is not None and (not entry.group("rest") or re.match(r"\s{2,}\S|（|\(", entry.group("rest")) is not None)
        if narrow:
            found = re.match(r"^#\s+BASH_COMMAND_DETECTOR_BREAK=([a-z0-9-]+)|^#   ([a-z][a-z0-9]*(?:-[a-z0-9]+)+)(?:  |$)", row)
            if found:
                table.add(found.group(1) or found.group(2))
        elif readable:
            table.update(item for item in entry.group("value").split(",") if item)
            if BREAK_SWITCH_SECOND_MENTION in entry.group("rest"):
                problems.append(f"文件头第 {number} 行同一行写了第二条开关，开关表一行一条、门禁读不到第二条：{row.strip()[:80]}")
        elif entry is not None and BREAK_SWITCH_SECOND_MENTION in entry.group("rest"):
            problems.append(f"文件头第 {number} 行同一行写了第二条开关，开关表一行一条、门禁读不到第二条：{row.strip()[:80]}")
            table.update(item for item in entry.group("value").split(",") if item)
        elif BREAK_SWITCH_TABLE_POSITION.match(row) and entry is None:
            problems.append(f"文件头第 {number} 行写了 BASH_COMMAND_DETECTOR_BREAK= 而开关表的文法读不到它（名字不能套反引号、等号后面不能换行）：{row.strip()[:80]}")
        elif in_batch:
            batch = BREAK_SWITCH_BATCH_ENTRY.match(row)
            if batch:
                table.add(batch.group(1))
    read = set(BREAK_SWITCH_DIRECT_READ_LITERAL.findall(code))
    if not detector_break_is_set("break-switch-indirect-reads-unseen"):
        constants = dict(BREAK_SWITCH_CONSTANT.findall(code))
        for double, single, identifier in BREAK_SWITCH_DIRECT_READ.findall("\n".join(line for line in code.split("\n") if not line.lstrip().startswith("def "))):
            if double or single:
                read.add(double or single)
            elif identifier in constants:
                read.add(constants[identifier])
            else:
                problems.append(f"代码里 detector_break_is_set({identifier}) 读的名字认不出（不是字面串，也不是 <名字> = \"<项>\" 登记过的常量）")
        read |= set(BREAK_SWITCH_GUARD_OFF.findall(code))
        read |= set(BREAK_SWITCH_BASH_LAYER.findall(code))
    header_text = "\n".join(re.sub(r"^#\s?", "", row) for row in header)
    sections = set()
    for listing in BREAK_SWITCH_SECTION_LIST.findall(header_text):
        flat = re.sub(r"\s+", " ", listing)
        while re.search(r"（[^（）]*）", flat):
            flat = re.sub(r"（[^（）]*）", "", flat)
        for piece in flat.split("、"):
            text = piece.strip()
            if BREAK_SWITCH_SECTION_NAME.fullmatch(text):
                sections.add(re.match(r"[a-z0-9]+(?:-[a-z0-9]+)+", text).group(0))
            elif BREAK_SWITCH_SECTION_ABBREVIATION.fullmatch(text):
                problems.append(f"各节「弄坏开关：」一串里写了缩写 {text[:30]}，写全名")
    return known, table, read, sections, problems

def break_switch_set_mismatches(source):
    """四处名字的对账：交排好序的说明列表，每个对不上的名字一条（`<名字>：只在<几处>里有，<几处>里没有`），再接读不出的写法的说明；全对交 []。
    名单、文件头开关表、读开关的代码三处两两包含；各节「弄坏开关：」里写着的名字都要在名单里。
    弄坏开关 break-switch-sets-one-way 下只查名单里的名字在另两处有没有（反方向的：文件头或代码里有而名单没有、各节里写着而名单没有，不查）。"""
    known, table, read, sections, problems = break_switch_name_sets(source)
    sets = (known, table, read)
    names = known if detector_break_is_set("break-switch-sets-one-way") else known | table | read
    mismatches = []
    for name in sorted(names):
        present = [place for place, found in zip(BREAK_SWITCH_PLACES, sets) if name in found]
        if len(present) < len(BREAK_SWITCH_PLACES):
            absent = [place for place in BREAK_SWITCH_PLACES if place not in present]
            mismatches.append(f"{name}：只在{'、'.join(present)}里有，{'、'.join(absent)}里没有")
    if not detector_break_is_set("break-switch-sets-one-way"):
        mismatches += [f"{name}：只在各节「弄坏开关：」一串里有，名单、文件头开关表、读开关的代码里没有" for name in sorted(sections - known - table - read)]
        mismatches += [f"{name}：各节「弄坏开关：」一串里写着，名单里没有" for name in sorted((sections & (table | read)) - known)]
    return mismatches + problems

# 弄坏开关只在自证里认：自证起的子进程靠变量 SELFTEST_MARK_VARIABLE 指到自证建的标记文件（内容是 SELFTEST_MARK_CONTENT）才认，门禁 code-tooling 的 break-switch-outside-selftest 格逐份探这一条；
# 两个名字与读标记的 selftest_marked 定义在入口的引导里：引导在这几份载入之前就要认它

# 它导入的共用库的弄坏开关变量（lib_code_discipline_lints.py 的）平常设了同样拒：㉓ 出路里的 lint 表读法认它，设了就改写出路里整段（关守卫项只在 BASH_COMMAND_DETECTOR_BREAK 里单独一项时放行它们）
LIBRARY_BREAK_VARIABLES = ("LIB_CODE_DISCIPLINE_LINTS_BREAK",)
# 关守卫项（records/2026-10-23-subagent拆分提案.md 第一百二十二节 ②）：只在它是唯一一项时认；与别的项同写照拒并点名它（带标记的自证子进程里也拒），
#   免得「关守卫项,别的项」平常跑借道把别的项也认了。GUARD_OFF_WITH_OTHERS_ITEM 是它的弄坏开关：带标记的自证子进程里同写着它时，同写的关守卫项照认
GUARD_OFF_ITEM = "break-honored-outside-selftest"
GUARD_OFF_WITH_OTHERS_ITEM = "break-honored-outside-selftest-with-others"

def refuse_break_outside_selftest():
    """弄坏开关只在自证里认：平常跑（不是 --selftest，也不是自证起的子进程）设了 BASH_COMMAND_DETECTOR_BREAK 就退 2，stderr 点名变量只在自证里认；没设、--selftest、
    标记文件在而内容对得上的放行（返回 0）。关守卫项 break-honored-outside-selftest 单独一项时不拒；与别的项同写一律拒、点名它
    （带标记的子进程里同写着 break-honored-outside-selftest-with-others 的除外）。"""
    value = os.environ.get(DETECTOR_BREAK_VARIABLE, "")
    library_set = [name for name in LIBRARY_BREAK_VARIABLES if os.environ.get(name)]
    if (not value and not library_set) or (len(sys.argv) > 2 and sys.argv[2] == "--selftest"):
        return 0
    items = [item for item in value.split(",") if item]
    marked = selftest_marked()
    others = [item for item in items if item != GUARD_OFF_ITEM]
    if GUARD_OFF_ITEM in items and others and not (marked and GUARD_OFF_WITH_OTHERS_ITEM in others):
        print(f"  ✗ {DETECTOR_BREAK_VARIABLE}={value}：关守卫项 {GUARD_OFF_ITEM} 只在它是唯一一项时认，与别的项（{'、'.join(others)}）同写照拒；这一次没判", file=sys.stderr)
        print(f"     → 怎么办：unset {DETECTOR_BREAK_VARIABLE} 再来；要看某一项红在哪一格，跑 {DETECTOR_BREAK_VARIABLE}=<项> bash .claude/hooks/bash-command-detector.sh --selftest，一次只设那一项", file=sys.stderr)
        return 2
    if marked or items == [GUARD_OFF_ITEM]:
        return 0
    set_variables = ([DETECTOR_BREAK_VARIABLE] if value else []) + library_set
    for name in set_variables:
        print(f"  ✗ {name}={os.environ[name]} 是弄坏开关，只在自证（bash-command-detector.sh --selftest）里认；平常跑设了就拒，这一次没判", file=sys.stderr)
    listed_variables = "、".join(set_variables)
    print(f"     → 怎么办：unset {listed_variables} 再来；要看开关红在哪一格，跑 {set_variables[0]}=<项> bash .claude/hooks/bash-command-detector.sh --selftest（开关写在文件头，一次只设一项）", file=sys.stderr)
    return 2
# 剥掉之后里面那条接着判的启动器：lib_project_code_runs.py 的 LAUNCHER_OPTIONS_WITH_VALUE 的键（/usr/bin/time、flock、systemd-run……，剥法也在那一份：command_under_launcher）。
#   命令词是这几个才去装那份几千行的库；两边的名字集合不一致时自检「启动器名单与 lib_project_code_runs 的一致」那一格红
LAUNCHER_NAMES = frozenset({"time", "flock", "chrt", "prlimit", "systemd-run", "strace", "perf", "rustup",
                            # 名单外启动器（lib_project_code_runs.py 的 EXTRA_LAUNCHER_NAMES；一百三十七节 ②）
                            "doas", "numactl", "unshare", "setpriv", "nsenter", "ltrace", "valgrind", "faketime", "unbuffer", "cpulimit",
                            "dbus-run-session", "firejail", "proot", "chroot", "cgexec", "xargs", "find", "ssh", "su", "runuser", "sg", "script",
                            "watch", "gdb", "scl", "parallel", "bwrap"})
def wrapped_commands(words):
    """一条简单命令一层层剥开前缀、capped.sh N、`bash 包装脚本`（bash 起的是脚本时接着往里剥）与 `two-logs.sh … --`：
    依次交回 (这一层命令词前面的前缀词, 路过的前缀名, 命令词的名字, 参数)；最后只剩关键字、赋值与前缀的，名字是 None。"""
    remaining = words
    for _ in range(shell_words.MAXIMUM_NESTING):
        skipped = shell_words.skip_prefixes(remaining)
        if skipped.command_position is None:
            yield remaining, skipped.prefix_names, None, []
            return
        prefix_words, remaining = remaining[:skipped.command_position], remaining[skipped.command_position:]
        name, arguments = os.path.basename(remaining[0]), remaining[1:]
        yield prefix_words, skipped.prefix_names, name, arguments
        if name in shell_words.WRAPPERS_TAKING_ONE_ARGUMENT and len(arguments) >= 2 and not arguments[0].startswith("-"):
            remaining = arguments[1:]
            continue
        if name == TWO_LOGS_SCRIPT_NAME and not detector_break_is_set("opaque-two-logs"):
            inner = two_logs_inner_words(remaining)
            if inner:
                remaining = inner
                continue
        if name in shell_words.SHELL_NAMES:
            invocation = shell_words.shell_invocation(arguments)
            if invocation.code_string is None and invocation.script_index is not None:
                remaining = arguments[invocation.script_index:]
                continue
        return

def shell_code_and_timeout(words):
    """一条简单命令剥开包装之后：交回 (bash -c 的那段代码或 None, 路过的前缀里有没有 timeout)。"""
    has_timeout = False
    for _, prefix_names, name, arguments in wrapped_commands(words):
        has_timeout = has_timeout or "timeout" in prefix_names
        if name in shell_words.SHELL_NAMES:
            code = shell_words.shell_invocation(arguments).code_string
            if code is not None:
                return code, has_timeout
    return None, has_timeout

class ShellLayer(NamedTuple):
    commands: list             # 这一层的简单命令，每条是词列表（重定向去掉、包装没剥）
    has_lone_ampersand: bool   # 这一层有单独的 &（放后台；不是 &&、>&、&>、|&）
    under_timeout: bool        # 这一层外面有 timeout（挂它的那条命令、或更外面一层带 timeout 前缀）
    tokens: list               # 这一层的记号（共用模块 shell_tokens 切的，喂给 shell 的 heredoc 正文已剥出去另成一层）

def shell_layers(text, under_timeout=False, honor_timeout=True, depth=0):
    """整条命令的每一层：顶层、bash -c 的那段代码、喂给 shell 的 heredoc 正文、命令替换，各交一个 ShellLayer。
    外面那条带 timeout 前缀的，里面各层都算有 timeout；honor_timeout 为假时 timeout 一律不算（自检的弄坏开关用）。"""
    if depth > shell_words.MAXIMUM_NESTING:
        return []
    shell_heredocs = []
    tokens = shell_words.shell_tokens(shell_words.strip_data_heredocs(text, shell_heredocs=shell_heredocs))
    commands, _, substitutions, _ = shell_words.simple_commands(tokens)
    layers = [ShellLayer(commands, (True, "&") in tokens, under_timeout, tokens)]
    for head, body in shell_heredocs:
        head_commands = shell_words.simple_commands(shell_words.shell_tokens(shell_words.open_substitution_tail(head)))[0]   # 挂在没收尾的 $( / <( 里的，在那一层里找挂它的命令
        feeds_under_timeout = bool(head_commands) and shell_code_and_timeout(head_commands[-1])[1] and honor_timeout
        layers += shell_layers(body, under_timeout or feeds_under_timeout, honor_timeout, depth + 1)
    for words, command_substitutions in zip(commands, substitutions):
        for substitution in command_substitutions:
            layers += shell_layers(substitution, under_timeout, honor_timeout, depth + 1)
        code, has_timeout = shell_code_and_timeout(words) if words else (None, False)
        if code is not None:
            layers += shell_layers(code, under_timeout or (has_timeout and honor_timeout), honor_timeout, depth + 1)
    return layers

def wait_loops_in(commands, loop_filter=None):
    """一层里的 until / while 循环，条件或循环体里有 sleep 的（给了 loop_filter 的还要它对整个循环的简单命令判真），交回每个循环开头那条简单命令。"""
    found, index = [], 0
    while index < len(commands):
        if not loop_keyword_changes(commands[index])[2]:
            index += 1
            continue
        end, level = index + 1, 1
        while end < len(commands) and level > 0:
            opened, closed, _ = loop_keyword_changes(commands[end])
            level += opened - closed
            end += 1
        if any(is_sleep_command(words) for words in commands[index:end] if words) and (loop_filter is None or loop_filter(commands[index:end])):
            found.append(" ".join(commands[index])[:120])
            index = end
        else:
            index += 1
    return found

def unbounded_wait_loops(text, honor_timeout=True):
    """命令位置上的 until / while 循环里（条件或循环体）有 sleep、外面又没有 timeout 的，交回每个循环开头那条简单命令。"""
    return [loop for layer in shell_layers(text, honor_timeout=honor_timeout) if not layer.under_timeout for loop in wait_loops_in(layer.commands)]

# 把活放出追踪的写法（agent-common.md「长活交回主 agent」那一条列的）：命令词是这几个的
DETACHING_COMMANDS = {"disown", "coproc"}

def detaching_command(name, arguments):
    """命令词与参数是不是 tmux / screen 的分离模式、不等结束的 systemd-run、disown、coproc；是就交回认出的写法，不是交 None。"""
    if name in DETACHING_COMMANDS:
        return name
    if name == "tmux" and any(argument in ("new", "new-session") for argument in arguments) and any(
            argument.startswith("-") and not argument.startswith("--") and "d" in argument[1:] for argument in arguments):
        return "tmux new -d"
    if name == "screen" and (any(re.match(r"^-[dD]m", argument) for argument in arguments)
                             or ({"-d", "-D"} & set(arguments) and "-m" in arguments)):
        return "screen -dm"
    if name == "systemd-run" and not any(argument in ("--wait", "--pipe", "--pty", "--scope", "-P", "-t") for argument in arguments):
        return "systemd-run（不带 --wait / --pipe / --pty / --scope）"
    return None

def detaching_prefixes(prefix_words, has_lone_ampersand, prefix_names):
    """命令词前面那段前缀里的 `setsid -f`、（这一层有单独的 & 时的）`nohup` 与共用模块当关键字跳过的 `coproc`（prefix_names 是共用模块记下的路过的前缀名）。"""
    found = []
    for index, word in enumerate(prefix_words):
        name = os.path.basename(word)
        if name == "coproc" and not detector_break_is_set("coproc-prefix-unseen"):
            # 共用模块把 coproc 当关键字跳过时把它记进路过的前缀名，命令词前面那段里认它；`builtin coproc …`、`builtin -- coproc …`
            # 里的 coproc 不是内建，bash 报 not a shell builtin、什么都不跑，共用模块不记它，这里也不认（复跑分类报告 d137-replay-classify 6.5 节；
            # 弄坏开关 builtin-coproc-detached 下照词认）
            if "coproc" in prefix_names or detector_break_is_set("builtin-coproc-detached"):
                found.append("coproc")
            continue
        if name == "setsid":
            options = []
            for option in prefix_words[index + 1:]:
                if not option.startswith("-") or option == "-":
                    break
                options.append(option)
            if any(option == "--fork" or (not option.startswith("--") and "f" in option[1:]) for option in options):
                found.append("setsid -f")
        elif name == "nohup" and has_lone_ampersand:
            found.append("nohup … &")
    return found

def detaching_forms(text):
    """命令位置上把活放出追踪的写法：disown、coproc、setsid -f、nohup … &、tmux / screen 的分离模式、不等结束的 systemd-run。
    每一层（bash -c、喂给 shell 的 heredoc、命令替换）都判，前缀、capped.sh N 与 `bash 包装脚本` 剥开往里找。"""
    found = []
    for layer in shell_layers(text):
        for words in layer.commands:
            for prefix_words, prefix_names, name, arguments in wrapped_commands(words):
                found += detaching_prefixes(prefix_words, layer.has_lone_ampersand, prefix_names)
                form = detaching_command(name, arguments) if name else None
                if form:
                    found.append(form)
    return list(dict.fromkeys(found))

def detaching_refusal(command):
    """命令位置上有把活放出追踪的写法就交回认出的写法；否则交 []。前台、run_in_background 一样拒。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-detaching")):
        return []
    return detaching_forms(command)

# ⑧ 有 Edit 工具的项目子 agent 在 Bash 里就地改仓内文件：命令位置上的 `sed -i`、`perl -i`（目标不在 /tmp/ 下、也不在工作根下），或 python 代码里 open(<仓内路径字面量>, 'w' / 'a' / 'x')、
#   Path(<仓内路径字面量>).write_text( / .write_bytes(，os.replace / os.rename / shutil.move 的目标（第二个参数或 dst=）是仓内路径字面量的；
#   python 代码另含 `python3 <脚本文件>` 起的、落在工作根下的脚本正文（python_script_bodies）。
#   仓内路径字面量：crates/、scripts/、.claude/、research/、records/、litmus/、briefs/ 起头的，或仓根起的绝对路径。
#   认不出：路径放在变量里的（脚本正文里的只记一条检出）、工作根之外的脚本文件（replace-once.py、insert-row.py 这类定义点名的脚本本来就放行）。
#   弄坏开关 python-move-into-repository-unseen 下前缀不含 scripts/，三种挪法不认（PYTHON_REPOSITORY_WRITE_BEFORE_MOVES）。
REPOSITORY_PATH_PREFIX = r"(?:\./)?(?:crates|scripts|\.claude|research|records|litmus|briefs)/"
REPOSITORY_PATH_PREFIX_BEFORE_MOVES = r"(?:\./)?(?:crates|\.claude|research|records|litmus|briefs)/"
# 挪法的第一个参数：一个字符串字面量，或不带逗号、括号与引号的一个表达式（名字、属性）
PYTHON_MOVE_SOURCE = r"""(?:f?'[^']*'|f?"[^"]*"|[^,()'"]+)"""

def python_repository_write_pattern(prefix, with_moves):
    """⑧ python 一半的正则（{root} 处由调用方换成仓根）：open(…, 'w' / 'a' / 'x')、Path(…).write_text / write_bytes，with_moves 时另认 os.replace / os.rename / shutil.move 的目标。"""
    alternatives = [r"""open\(\s*f?['"](?P<open>(?:{root})?""" + prefix + r"""[^'"]*)['"]\s*,\s*['"][wax]b?\+?['"]""",
                    r"""Path\(\s*f?['"](?P<path>(?:{root})?""" + prefix + r"""[^'"]*)['"]\s*\)\s*\.write_(?:text|bytes)\("""]
    if with_moves:
        alternatives.append(r"""(?:\bos\.(?:replace|rename)|\bshutil\.move)\(\s*""" + PYTHON_MOVE_SOURCE + r"""\s*,\s*(?:dst\s*=\s*)?f?['"](?P<move>(?:{root})?"""
                            + prefix + r"""[^'"]*)['"]""")
    return re.compile("|".join(alternatives))

PYTHON_REPOSITORY_WRITE = python_repository_write_pattern(REPOSITORY_PATH_PREFIX, True)
PYTHON_REPOSITORY_WRITE_BEFORE_MOVES = python_repository_write_pattern(REPOSITORY_PATH_PREFIX_BEFORE_MOVES, False)

def agent_has_edit_tool(repository_root, agent_type):
    try:
        head = open(os.path.join(repository_root, ".claude", "agents", f"{agent_type}.md"), encoding="utf-8").read().split("\n---", 1)[0]
    except OSError:
        return False
    tools_line = next((line for line in head.splitlines() if line.startswith("tools:")), "")
    return "Edit" in {tool.strip() for tool in tools_line[len("tools:"):].split(",")}

def sed_in_place_targets(arguments):
    """sed 的参数里带 -i / --in-place 时交回它要改的文件（不在 /tmp/ 下、也不在工作根下的；工作根认绝对路径与 ~/…、$HOME/… 写法）；不带交 []。"""
    if not any(argument == "-i" or re.match(r"^-[a-zA-Z]*i", argument) or argument.startswith("--in-place") for argument in arguments):
        return []
    has_script_option = any(argument in ("-e", "-f") or argument.startswith(("--expression", "--file")) for argument in arguments)
    operands = [argument for argument in arguments if not argument.startswith("-")]
    files = operands if has_script_option else operands[1:]
    return [path for path in files if not in_place_target_in_scratch(path)]

def in_place_target_in_scratch(path):
    """sed -i / perl -i 的目标照文字就落在 /tmp/ 下或工作根下（绝对路径与 ~/…、$HOME/…、$SINGLEFS_WORK_ROOT/… 写法）：不是仓内文件，不交给 ⑧ ⑨ ⑫。"""
    return path.startswith("/tmp/") or bool(work_roots and path.startswith(("/", "~", "$")) and written_path_is_absolute(path)
                                            and work_roots.is_under_scratch_root(path))

# perl 的开关簇（perlrun）：不带值的字母可以合写在一簇里；-i 之后到簇尾是备份后缀（`-pie` 是后缀为 e 的 -i，不是 -e）；
#   -e / -E 之后到簇尾是代码，簇尾就是它时取下一个词；-0 接八进制数（或 x 接十六进制），-l 接八进制数；-I 之后到簇尾是目录，簇尾就是它时取下一个词；
#   -M、-m、-x、-C、-d、-D、-F、-V 吃掉簇里剩下的。`--` 之后、第一个不以 - 起头的词起是操作数：给了 -e / -E 的全是文件，没给的第一个是程序文件。
PERL_REST_OF_CLUSTER_SWITCHES = "MmxCdDFV"
PERL_OCTAL_DIGITS = "01234567"
PERL_HEXADECIMAL_DIGITS = "0123456789abcdefABCDEF"

def perl_in_place_files(arguments):
    """perl 的参数里有 -i（开关簇里认，`-Ilib`、`-MList::Util` 里的 i 不算）时交回它要就地改的文件；没有 -i 交 []。
    弄坏开关 perl-switch-cluster-by-regex 下照正则 ^-[a-zA-Z]*i 认 -i、-e 只认整个词（`-Ilib -ne` 也当就地改）。"""
    if detector_break_is_set("perl-switch-cluster-by-regex"):
        if not any(re.match(r"^-[a-zA-Z]*i", argument) for argument in arguments):
            return []
        files, skip_next, script_given = [], False, False
        for argument in arguments:
            if skip_next:
                skip_next = False
            elif re.match(r"^-[a-zA-Z]*[eE]$", argument):
                skip_next, script_given = True, True
            elif not argument.startswith("-"):
                files.append(argument)
        return files if script_given else files[1:]
    in_place, code_given, position = False, False, 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--":
            position += 1
            break
        if not argument.startswith("-") or argument == "-":
            break
        position += 1
        letters, index = argument[1:], 0
        while index < len(letters):
            letter = letters[index]
            index += 1
            if letter == "i":
                in_place = True
                break
            if letter in "eE":
                code_given = True
                if index == len(letters):
                    position += 1
                break
            if letter == "I":
                if index == len(letters):
                    position += 1
                break
            if letter in PERL_REST_OF_CLUSTER_SWITCHES:
                break
            if letter == "0":
                digits = PERL_HEXADECIMAL_DIGITS if letters[index:index + 1] == "x" else PERL_OCTAL_DIGITS
                index += 1 if digits is PERL_HEXADECIMAL_DIGITS else 0
                while index < len(letters) and letters[index] in digits:
                    index += 1
            elif letter == "l":
                while index < len(letters) and letters[index] in PERL_OCTAL_DIGITS:
                    index += 1
    if not in_place:
        return []
    operands = arguments[position:]
    return operands if code_given else operands[1:]

def written_path_is_absolute(path):
    """sed -i 的目标照文字展开（$SINGLEFS_WORK_ROOT、$HOME、~）之后是绝对路径：展开不了的（`$F`、xargs 的占位词）不拿钩子进程自己的当前目录去补成绝对路径
    （否则钩子进程的当前目录在工作根下时，`sed -i … $F` 被当成工作根下的草稿、从目标里丢掉）。弄坏开关 sed-unknown-target-resolved-from-cwd 下一律补成绝对路径。"""
    if detector_break_is_set("sed-unknown-target-resolved-from-cwd"):
        return True
    return os.path.isabs(work_roots.expand_written_path(path))

def sed_target_resolved_in_scratch(step, repository_root):
    """sed -i / perl -i 的目标是相对路径、前面 cd 到了算得出的目录时，从那个目录起算：落在 /tmp/ 或工作根下、而不在本仓里的，不是仓内文件。
    cd 的目录算不出（step.directory 是 None）、目标本身带变量或通配、本来就是绝对路径的，这里不管：照旧按仓内判（绝对路径的草稿目录在 sed_in_place_targets 里已经放行）。"""
    if step.directory is None or re.search(r"[$`*?\[{]", step.target) or os.path.isabs(os.path.expanduser(step.target)):
        return False
    absolute = shell_words.resolve_path(step.directory, step.target)
    if any(candidate == root or candidate.startswith(root + os.sep)
           for candidate in (absolute, os.path.realpath(absolute)) for root in (os.path.normpath(repository_root), os.path.realpath(repository_root))):
        return False
    return absolute.startswith("/tmp/") or bool(work_roots and work_roots.is_under_scratch_root(absolute))

def python_target_outside_repository(target, directories, repository_root):
    """⑧ python 一半的目标是相对路径、每一处跑它的当前目录都算得出、按那里解开都落在本仓之外时为真（cd 进草稿目录里的仓副本再写 crates/…、scripts/…）。
    仓根起的绝对路径、当前目录算不出或不知道（directories 是 None 或空）、有一处落在仓里的都为假，照旧按仓内判。"""
    if os.path.isabs(target) or not directories or None in directories:
        return False
    roots = (os.path.normpath(repository_root), os.path.realpath(repository_root))
    for directory in directories:
        absolute = shell_words.resolve_path(directory, target)
        if absolute is None or any(candidate == root or candidate.startswith(root + os.sep)
                                   for candidate in (absolute, os.path.realpath(absolute)) for root in roots):
            return False
    return True

class InPlaceEditVerdict(NamedTuple):
    refused: list   # 要拒的写法与目标（「sed -i 目标」「perl -i 目标」「python 写 目标」「python 脚本 <脚本> 写 目标」这类）
    noted: list     # 只记不拦的（python 脚本里写文件的目标不是字面量，判不出写不写仓内文件）

PYTHON_SCRIPT_READ_LIMIT_BYTES = 4 * 1024 * 1024   # ⑧ 读 python 脚本正文的上限；更大的不读，照判不到办

def python_repository_write_regex(repository_root):
    """⑧ python 一半的正则，{root} 换成这一次的仓根（弄坏开关 python-move-into-repository-unseen 下前缀不含 scripts/、三种挪法不认）。"""
    template = PYTHON_REPOSITORY_WRITE_BEFORE_MOVES if detector_break_is_set("python-move-into-repository-unseen") else PYTHON_REPOSITORY_WRITE
    return re.compile(template.pattern.replace("{root}", re.escape(repository_root.rstrip("/") + "/")))

def python_repository_writes(code, pattern, directories, repository_root, label):
    """一段 python 代码里按字面量写仓内路径的每一处（pattern 是 python_repository_write_regex）；相对路径在 directories 里每一处跑它的目录解开都落在本仓之外的不算。"""
    found = []
    for match in pattern.finditer(code):
        moved = match.groupdict().get("move")
        target = moved or match.group('open') or match.group('path')
        if python_target_outside_repository(target, directories, repository_root):
            continue
        found.append(f"{label} 挪进 {moved}" if moved else f"{label} 写 {target}")
    return found

PYTHON_UNLITERAL_LISTED = 3   # 只记不拦的那一条里列几处目标不是字面量的写

# ⑨ 先编后换：experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs。写法沿用 ⑤ ⑦ 的 overwrite_steps（every_write=True），
#   mv 挪走、rm 删掉的源也算改。agent 名单与路径判据与 write-guard.sh「四、先编后换」各写一份、改的时候一起改。
COMPILE_FIRST_AGENTS = ("experiment-runner",)
COMPILE_THEN_SWAP_SCRIPT = "scripts/hosts/compile-then-swap.py"

# ⑫ 有写范围的项目子 agent 在 Bash 里写自己写范围之外的仓内文件。写范围闸 write-guard.sh 只看 Write / Edit（它文件头「拦不住 Bash 里的写」），
#   子 agent 在 Bash 里跑 scripts/kb/replace-once.py 就写进了定义不让它写的知识库目录（records/2026-09-16-subagent拆分提案.md 第八十一节 ④）。这一道照
#   同目录 agent-write-scope.tsv 那张表判（读法与判法在 lib_write_scope.py，write-guard.sh 用的是同一份）：该 agent 的行里没有一条模式落到这个文件，就拒。
#   认的写法：⑨ 认的那一批（overwrite_steps 的 every_write：>、>>、&>、tee（带不带 -a）、cp / mv / install 的目标、dd of=、truncate、sed -i、perl -i、rsync 的目标，
#   python 代码里 open(…, 带 w / a / x / + 的模式)、Path.write_text / write_bytes、shutil.copy*、os.replace / rename），加上 KNOWN_WRITE_SCRIPTS 六个定点写脚本的目标文件：
#   replace-once.py 的 文件、insert-row.py 的 文件（带 --dry-run 的不写，不算）、replace-batch.py 规格 JSON 里每一项的 file（规格读不到或不是数组的拒；带 --dry-run 的不算）、
#   ledger-append-section.py 写的登记表 current 那一份账本（LedgerAppendTarget，仓根的认法见文件头 ⑫）、append-record.py 的 <记录>（第一个不是选项的参数，--root 的值不算）、
#   make-swap-spec.py 的 --out 那一份规格。
#   落点算得出而在仓外的（/tmp、/dev/null、工作根下的草稿目录）不判。没有写范围的不判：主 agent（输入里没有 agent_type）、没有项目定义的内置 agent、
#   表里没有它那几行的定义（表给每个定义都登记，包括只带 Read、Bash 的；新建而还没登记的定义走到这里，门禁 code-tooling 的 agent-write-scope 格会红）。
#   落点这一刻算不出的（没赋过值或来源不明的变量、命令替换、通配、cd 算不出之后的相对路径）一律拒，静态前缀在仓外也拒；拒绝说明指到 scripts/agents/copy-originals.py 与写死的词的 for 循环。
#   算得出的照算（前面赋的写死值、for 写死的词逐个取值、cd 之后的 $PWD、mktemp -d、$HOME；落点规整 .. 并逐段解开符号链接），判法见文件头 ⑫。
#   heredoc 正文：喂给 shell 的照命令判，喂给 python 的照 python 写文件判，喂给 cat、tee 这类的是数据不判（它们的目标照判）。mv 挪走的源与 rm 删掉的不判（只判写进去的）。cd 跟着算，相对路径从 hook 输入的 cwd 起算。
#   前台、run_in_background 一样拒。判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、patch 与 git apply、经 subprocess 起的写法、xargs / find -exec。
KNOWN_WRITE_SCRIPTS = ("replace-once.py", "replace-batch.py", "insert-row.py", "ledger-append-section.py", "append-record.py", "make-swap-spec.py")   # scripts/kb/ 下定点写文件的五个脚本，加 scripts/gate/ 下写 --out 的 make-swap-spec.py
SWAP_SPEC_SCRIPT = "make-swap-spec.py"            # 写的是 --out 给的那一份规格 JSON
LEDGER_APPEND_SCRIPT = "ledger-append-section.py"   # 不带目标文件参数：写的是仓根下登记表 current 那一份账本（LedgerAppendTarget）
WRITE_SCOPE_TABLE_NAME = "agent-write-scope.tsv"                                  # 与 write-guard.sh 同一张表，在这个 hook 的同目录
INSERT_ROW_OPTIONS_WITH_VALUE = {"--after", "--before", "--after-literal", "--before-literal", "--line", "--absent", "--absent-literal"}
NON_WRITING_FLAGS = {"--selftest", "--dry-run", "-h", "--help"}
RECORD_APPEND_SCRIPT = "append-record.py"         # 写的是 <记录>：第一个不是选项的参数（往 records/ 下的 .md 追加，⑲ 的出路）
RECORD_APPEND_OPTIONS_WITH_VALUE = {"--root"}     # append-record.py 带值的选项：它的值是仓根，不是记录

class UnknownScriptTarget(NamedTuple):
    """定点写脚本要改哪个文件这一刻读不出来（⑫ 只记检出）。"""
    form: str
    reason: str

def first_operand(arguments, options_with_value=()):
    """参数里第一个不是选项的词（options_with_value 里的选项连它的值一起跳过）；没有交 None。"""
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in options_with_value:
            position += 2
        elif argument.startswith("-") and argument != "-":
            position += 1
        else:
            return argument
    return None

class UnresolvedScriptTarget(NamedTuple):
    """python3 起的脚本词代入前面赋过的值之后还有算不出的段（认不出起的是不是定点写脚本），第一个不是选项的参数是 target：
    bash_scope_verdict 算它的落点，落在本仓里的照「落点算不出」拒。"""
    form: str
    target: str
    directory: str | None

def unresolved_script_word(word):
    """脚本词代入之后还有算不出的段：没赋过值或来源不明的变量、命令替换、for 循环变量（代入时包着 LOOP_VALUE_MARK）。"""
    return "$" in word or "`" in word or LOOP_VALUE_MARK in word

class LedgerAppendTarget(NamedTuple):
    """ledger-append-section.py 要写的账本（⑫）：它不带目标文件参数，写的是仓根下登记表 current 那一份；仓根照它自己的认法取（ledger_append_root）。"""
    form: str
    root: str | None        # --root 给的（前面赋过的变量已代入）；没给是 None
    script: str | None      # 没给 --root 时 python3 起的脚本路径（变量已代入）；直接执行只剩脚本名，是 None
    directory: str | None   # 那一刻的当前目录；cd 算不出来是 None

def last_option_value(arguments, option):
    """参数里 option 的值：`--root X` 与 `--root=X` 两种写法都认，写了几次取最后一次（同 argparse）；没写交 None。"""
    value = None
    for position, argument in enumerate(arguments):
        if argument == option and position + 1 < len(arguments):
            value = arguments[position + 1]
        elif argument.startswith(option + "="):
            value = argument[len(option) + 1:]
    return value

# 没有项目定义的 agent（内置类型、插件带来的、定义已删的旧实例）的写范围：只有工作根（文件头 ⑫；write-guard.sh 第二道用同一句模式）
BUILTIN_AGENT_SCOPE = ("$SINGLEFS_WORK_ROOT/**",)
DEVICE_PREFIXES = ("/dev/", "/proc/")   # 没有项目定义的 agent 写仓外时不判的：设备与文件描述符（2>/dev/null、> /dev/stderr）

def agent_has_definition(repository_root, agent_type):
    return os.path.isfile(os.path.join(repository_root, ".claude", "agents", f"{agent_type}.md"))

AGENT_DEFINITION_WRITE_TOOLS = ("Write", "Edit", "MultiEdit", "*")

def definition_can_write(repository_root, agent_type):
    """项目定义的 tools 里有 Write、Edit、MultiEdit 或 *（没写 tools 的继承全部工具，也算）；只给弄坏开关 scope-read-only-definitions-unjudged 用。"""
    with open(os.path.join(repository_root, ".claude", "agents", f"{agent_type}.md"), encoding="utf-8") as handle:
        head = handle.read().split("\n---", 1)[0]
    tools_line = next((line for line in head.split("\n") if line.startswith("tools:")), None)
    if tools_line is None:
        return True
    return any(tool.strip().strip("'\"[]") in AGENT_DEFINITION_WRITE_TOOLS for tool in tools_line[len("tools:"):].split(","))

def agent_scope_patterns(repository_root, agent_type, table_path):
    """项目定义的 agent 在写范围表里的模式（表里没有它的行交 []；每个定义都登记，只带 Read、Bash 的也有行）；没有项目定义的交 BUILTIN_AGENT_SCOPE（弄坏开关 builtin-agents-unjudged 下交 []）。
    读表读不了抛 OSError。弄坏开关 scope-read-only-definitions-unjudged 下 tools 里没有 Write、Edit 的定义的行读成没有。"""
    if not agent_has_definition(repository_root, agent_type):
        return [] if detector_break_is_set("builtin-agents-unjudged") else list(BUILTIN_AGENT_SCOPE)
    if detector_break_is_set("scope-read-only-definitions-unjudged") and not definition_can_write(repository_root, agent_type):
        return []
    return write_scope.load_scopes(table_path).get(agent_type, [])

LOOP_VALUE_MARK = "\x01"                          # ⑫ 里「for 名字 in 写死的词…」的循环变量代入时写成 \x01词\x02词…\x01，判的时候逐个取值；路径里不会有这两个字符
LOOP_VALUE_SEPARATOR = "\x02"
LOOP_VALUE_PIECE = re.compile("\x01([^\x01]*)\x01")
LITERAL_LOOP_WORD = re.compile(r"[^$`*?\[\]{}~\x01\x02]+")   # 循环列表里写死的词：没有变量、命令替换、通配、花括号与 ~
LOOP_BINDING_COMMANDS = {"for", "select"}
READING_COMMANDS = {"read", "mapfile", "readarray"}  # 读进来的值在命令文本里算不出
MAXIMUM_LOOP_LANDINGS = 4096                      # 一个目标里几个循环变量的取值组合多过这么多，照算不出判
VARIABLE_NAME = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
HOME_REFERENCE = re.compile(r"\$(?:HOME\b|\{HOME\})")

class ScopeVerdict(NamedTuple):
    """⑫ 判一条命令的结局：写到写范围之外的仓内文件（要拒）、落点算不出的（要拒；弄坏开关 scope-undeterminable-only-noted 下改记进 noted）、只记检出的、该 agent 的写范围模式。"""
    out_of_scope: list
    undeterminable: list
    noted: list
    patterns: list
    builtin: bool = False   # 没有项目定义的 agent：只许写工作根，仓外别处的落点也拒

def bind_loop_variables(name, arguments, variables):
    """⑫ 跟 for / select 与 read / mapfile 这类改写变量的命令（arguments 是还没代入变量的原词）：`for 名字 in 词…` 列出的词都写死时，名字记成这几个取值
    （LOOP_VALUE_MARK 包着，之后的目标逐个取值算落点）；列表算不出的（来自变量、命令替换、通配、花括号、没写 in）与 read 这类读进来的名字从 variables 里去掉，
    之后照算不出判（前面赋过的值也作废）。弄坏开关 scope-loop-values-unknown 下写死的词也当算不出。"""
    if name in LOOP_BINDING_COMMANDS:
        if not arguments or not VARIABLE_NAME.fullmatch(arguments[0]):
            return
        listed = arguments[2:] if arguments[1:2] == ["in"] else []
        if listed and all(LITERAL_LOOP_WORD.fullmatch(word) for word in listed) and not detector_break_is_set("scope-loop-values-unknown"):
            variables[arguments[0]] = LOOP_VALUE_MARK + LOOP_VALUE_SEPARATOR.join(listed) + LOOP_VALUE_MARK
        else:
            variables.pop(arguments[0], None)
        return
    for argument in arguments:
        if VARIABLE_NAME.fullmatch(argument):
            variables.pop(argument, None)

def loop_value_choices(texts):
    """目标与源里代入的循环取值逐个组合：交回 [{包着的那一段: 取值}]；没有循环取值交 [{}]，组合多过 MAXIMUM_LOOP_LANDINGS 交 None（照算不出判）。
    弄坏开关 scope-loop-landings-trusted 下每一段只取一个写死的普通名字（取值里的 `..` 与 / 看不见）。"""
    pieces = list(dict.fromkeys(match.group(0) for text in texts for match in LOOP_VALUE_PIECE.finditer(text)))
    if detector_break_is_set("scope-loop-landings-trusted"):
        return [{piece: "plain-value" for piece in pieces}]
    choices = [{}]
    for piece in pieces:
        values = piece[1:-1].split(LOOP_VALUE_SEPARATOR)
        if len(choices) * len(values) > MAXIMUM_LOOP_LANDINGS:
            return None
        choices = [{**choice, piece: value} for choice in choices for value in values]
    return choices

def with_loop_values(text, choice):
    return LOOP_VALUE_PIECE.sub(lambda match: choice.get(match.group(0), match.group(0)), text)

def readable_loop_values(text):
    """拒绝说明里把循环取值写回 `{词,词}` 的样子。"""
    return LOOP_VALUE_PIECE.sub(lambda match: "{" + ",".join(match.group(1).split(LOOP_VALUE_SEPARATOR)) + "}", text)

def home_expanded(word):
    """目标词里没在这条命令里赋过值的 $HOME、${HOME}（赋过的已经代入）换成 hook 自己环境里的值：钩子与它判的命令是同一个用户起的。弄坏开关 scope-home-unexpanded 下不换。"""
    home = os.environ.get("HOME")
    if not home or detector_break_is_set("scope-home-unexpanded"):
        return word
    return HOME_REFERENCE.sub(lambda _: home, word)

# ⑭ 工具员在 Bash 里改两个服务 src/ 下已存在的文件与各自已存在的 Cargo.toml、Cargo.lock（文件头 ⑭）：写法的认法沿用 ⑫ 的 bash_scope_verdict（同一份切词、cd 与变量、定点写脚本、python 写法），
#   只把「落点在不在写范围里」换成「落点是不是这一组已存在的文件」；落点与 write-guard.sh 第十一道的源码一类、构建清单是同一组，两边各写一份、改的时候一起改。
SERVICE_SOURCE_AGENTS = ("tooling-writer",)
FIXTURE_NOTE_ENDINGS = ("的样本档目录）",)   # service_source_refusal 交的那几条里，落在已有门禁样本档目录的带这个收尾（ServiceSourceView.gate_fixture_note），拒绝说明另给补丁那一句出路
SERVICE_SOURCE_DIRECTORIES = ("prune-server/src", "gpu-verification-server/src")
SERVICE_BUILD_MANIFESTS = ("prune-server/Cargo.toml", "prune-server/Cargo.lock", "gpu-verification-server/Cargo.toml", "gpu-verification-server/Cargo.lock")

COMPRESSOR_FORM = "压缩或解压（原文件被删掉）"
SERVICE_FORM_MODES = {"rm": "holding", "mv 挪走": "holding", "unlink": "holding", "find -delete": "holding", "find -exec": "holding",
                      "rsync --delete": "holding", "rsync --remove-source-files": "holding", COMPRESSOR_FORM: "holding",
                      "tar -x": "tree", "patch": "tree", "git apply": "tree", "unzip": "tree"}   # bash_scope_verdict 的 judge 按写法认的判法：holding 是落点或包着已有服务源码的目录，tree 是落点目录碰得到已有服务源码

class ServiceSourceView:
    """主工作区里两个服务此刻已存在的源码与构建清单（SERVICE_SOURCE_DIRECTORIES 之下、SERVICE_BUILD_MANIFESTS 里的普通文件）。按这一条命令判的那一刻现读，每一次判新建一份，不跟到下一次。
    路径都按解开符号链接的写法（os.path.realpath），与 lib_write_scope.landing_forms 交的落点同一种。"""
    def __init__(self, repository_root):
        self.base = os.path.realpath(repository_root)
        self.cached_files = None
        self.cached_inodes = None

    def files(self):
        """已存在的服务源码与构建清单。"""
        if self.cached_files is None:
            found = []
            for directory in SERVICE_SOURCE_DIRECTORIES:
                for walked, _, names in os.walk(os.path.join(self.base, directory)):
                    found += [os.path.realpath(os.path.join(walked, name)) for name in names if os.path.isfile(os.path.join(walked, name))]
            found += [os.path.realpath(os.path.join(self.base, manifest)) for manifest in SERVICE_BUILD_MANIFESTS if os.path.isfile(os.path.join(self.base, manifest))]
            self.cached_files = found
        return self.cached_files

    def inodes(self):
        """files() 每一份的 (设备号, inode 号)。"""
        if self.cached_inodes is None:
            self.cached_inodes = set()
            for path in self.files():
                try:
                    status = os.stat(path)
                except OSError:
                    continue
                self.cached_inodes.add((status.st_dev, status.st_ino))
        return self.cached_inodes

    def shares_inode(self, absolute):
        """absolute 是已存在的普通文件、有不止一个名字、而且与某份服务源码是同一个文件（设备号与 inode 号相同，硬链接）：往这个名字写就是往服务源码写，落在仓外的也认。
        弄坏开关 service-source-hardlink-inode-unseen 下不认。"""
        if detector_break_is_set("service-source-hardlink-inode-unseen") or not os.path.isfile(absolute):
            return False
        try:
            status = os.stat(absolute)
        except OSError:
            return False
        return status.st_nlink > 1 and (status.st_dev, status.st_ino) in self.inodes()

    def is_source(self, relative, absolute):
        """relative（相对仓库根，落在仓外是 None）在 SERVICE_SOURCE_DIRECTORIES 之下或就是 SERVICE_BUILD_MANIFESTS 里的一份，而且 absolute 是已经存在的普通文件（新建的不算）；
        或 absolute 与服务源码是同一个文件的硬链接（shares_inode）。"""
        listed = relative is not None and (relative in SERVICE_BUILD_MANIFESTS or any(relative.startswith(directory + "/") for directory in SERVICE_SOURCE_DIRECTORIES))
        return (listed and os.path.isfile(absolute)) or self.shares_inode(absolute)

    def holds_source(self, directory):
        """directory 本身是已有的服务源码，或它里面（往下各层）有已有的服务源码。"""
        directory = os.path.realpath(directory)
        prefix = directory.rstrip(os.sep) + os.sep
        return any(path == directory or path.startswith(prefix) for path in self.files())

    def gate_fixture_note(self, relative, absolute, mode=None):
        """⑭ 另一半，已有门禁的样本档目录（判法同 write-guard.sh 第十一道那一半，都调 lib_write_scope.existing_gate_of_fixture）：
        落点（relative 相对仓根，仓根本身照 "." 认；落在仓外的不判）在已有门禁的样本档目录里，新建也算，落点是目录时 .claude/gate.d/fixtures/<门禁> 本身也算，
        交「已有门禁 <门禁> 的样本档目录」；mode 是 holding（rm、mv 挪走）或 tree（解包、打补丁）而落点是样本档根或它的上级、底下有已有门禁的样本目录的，
        交「包着已有门禁的样本档目录」；别的交 None。弄坏开关 gate-fixtures-unseen 下一律交 None。"""
        if detector_break_is_set("gate-fixtures-unseen"):
            return None
        if relative is None:
            relative = "." if os.path.realpath(absolute) == self.base else None
        if relative is None:
            return None
        gate = write_scope.existing_gate_of_fixture(relative, self.base, directory_itself=os.path.isdir(absolute))
        if gate:
            return f"已有门禁 {gate} 的样本档目录"
        if mode in ("holding", "tree") and write_scope.holds_existing_gate_fixtures(relative, self.base):
            return "包着已有门禁的样本档目录"
        return None

    def touches(self, directory):
        """往 directory 里解包、打补丁会不会碰到已有的服务源码：directory 在服务源码目录里面（含它自己），或它里面有已有的服务源码（仓根、prune-server/ 这类上级）。"""
        directory = os.path.realpath(directory)
        for name in SERVICE_SOURCE_DIRECTORIES:
            source_directory = os.path.realpath(os.path.join(self.base, name))
            if directory == source_directory or directory.startswith(source_directory + os.sep):
                return True
        return self.holds_source(directory)

    def overwritten_by_directory_copy(self, source, destination, contents_only):
        """把目录 source 拷进已有的目录 destination 会盖掉的已有服务源码：contents_only（源写成 `目录/.`、rsync 的 `目录/`、cp -T）时 source 里的文件直接落在 destination 下，
        否则落在 destination/<source 的名字>/ 下；落点上已有服务源码、source 里同一个相对位置有文件（或链接）的，交回那份服务源码。"""
        destination = os.path.realpath(destination)
        top = destination if contents_only else os.path.join(destination, os.path.basename(os.path.normpath(source)))
        hits = []
        for path in self.files():
            if path.startswith(top + os.sep):
                entry = os.path.join(source, os.path.relpath(path, top))
                if os.path.lexists(entry) and not os.path.isdir(entry):
                    hits.append(path)
        return hits

# ⑬ 项目子 agent 在 Bash 里起 claude 会话（共用约束「派发」一节：不从 Bash 里起 claude 会话）：命令位置上的命令词名叫 claude（光名字或以 /claude 结尾的路径），
#   或是会话自己那份可执行文件的变量 $CLAUDE_CODE_EXECPATH。只判输入里有 agent_type 的，前台、run_in_background 一样拒。
CLAUDE_EXECUTABLE_NAME = "claude"
CLAUDE_EXECUTABLE_PATH_WORDS = {"$CLAUDE_CODE_EXECPATH", "${CLAUDE_CODE_EXECPATH}"}

STARTUP_FILE_READ_LIMIT = 1 << 20   # 读 BASH_ENV / ENV 指的文件最多读这么多字节，同 project-code-guard.sh 读脚本的上限
STARTUP_FILE_HEAD_BYTES = 4096      # 直接执行的脚本读开头这么多字节认 #! 行，同 project-code-guard.sh 的 SCRIPT_HEAD_BYTES

def claude_launches(command, directory=None):
    """命令里起 claude 的简单命令（claude_launches_in_text 认的），加上读得到正文的文件里起的（file_claude_launches：shell 启动时先读的文件、bash / sh / source 起的与直接执行的 shell 脚本、
    别的程序读进去当代码的 awk -f / sed -f / Makefile）；directory 是命令开头的当前目录，解相对路径用。"""
    found = claude_launches_in_text(command)
    found += file_claude_launches(command, directory)
    return list(dict.fromkeys(found))

def startup_files_of(command):
    """这一条命令起来时它的 shell 先读的文件，每个一条共用模块交的 launcher 为 "startup" 的命令：命令本身就是（BASH_ENV / ENV / --rcfile / .bashrc / zsh 的启动文件，
    shell_words.startup_file_commands 交的）；或是直接执行的脚本（路径带斜杠、字面的、读得到的）的 #! 行指到的 shell 先读的（shell_words.executed_script_startup）。"""
    if command.launcher == "startup":
        return [command]
    word = command.words[0]
    if command.launcher is not None or "/" not in word or "$" in word or "`" in word:
        return []
    resolved = shell_words.resolve_path(command.directory, os.path.expanduser(word))
    head = None
    if resolved and os.path.isfile(resolved):
        try:
            with open(resolved, "rb") as handle:
                head = handle.read(STARTUP_FILE_HEAD_BYTES)
        except OSError:
            pass
    return shell_words.executed_script_startup(command, head)[0]

def heredoc_written_files(commands):
    """同一段命令文本里 cat / tee 读 heredoc 写出的文件：真实路径 → 写出的正文（`cat > x.sh <<'EOF' … EOF` 之后紧接着 `bash x.sh`，判的那一刻文件还不在，拿写出的内容判；
    同 project-code-guard.sh 认 heredoc 写出的那一种，cp 拷来的、别的写法写出的不认）。"""
    written = {}
    for command in commands:
        if not command.standard_input_heredocs or command.launcher is not None:
            continue
        name = os.path.basename(command.words[0])
        if name == "cat":
            targets = [target for operator, target in command.output_redirections if operator in (">", ">|", ">>", "&>")][-1:]
        elif name == "tee":
            targets = [word for word in command.words[1:] if not word.startswith("-")]
        else:
            continue
        for target in targets:
            resolved = shell_words.resolve_path(command.directory, os.path.expanduser(target)) if "$" not in target and "`" not in target else None
            if resolved:
                written[os.path.realpath(resolved)] = command.standard_input_heredocs[-1].text
    return written

def readable_body(word, directory, written, executed):
    """字面的路径词（`~`、`~+` 照展开，按 directory 解）读成正文：交 (真实路径, 正文)；同一段命令文本里前面 heredoc 写出的拿写出的内容。
    executed 为真（直接执行的）只读 #! 指到 shell、或没有 #! 而扩展名是 .sh 的；带变量、命令替换的，不在的，超过 STARTUP_FILE_READ_LIMIT、有 NUL 字节、不是 UTF-8 的交 None。"""
    if "$" in word or "`" in word:
        return None
    expanded = shell_words.expand_directory_tilde(word, directory)
    resolved = shell_words.resolve_path(directory, os.path.expanduser(expanded)) if expanded is not None else None
    if resolved is None:
        return None
    real = os.path.realpath(resolved)
    if real in written:
        return real, written[real]
    if not os.path.isfile(real):
        return None
    try:
        with open(real, "rb") as handle:
            data = handle.read(STARTUP_FILE_READ_LIMIT + 1)
    except OSError:
        return None
    if len(data) > STARTUP_FILE_READ_LIMIT or b"\0" in data:
        return None
    if executed:
        shebang = shell_words.shebang_command(data[:STARTUP_FILE_HEAD_BYTES])
        if (shebang is None and not resolved.endswith(".sh")) or (shebang is not None and not shell_words.is_shell_name(shebang[0])):
            return None
    try:
        return real, data.decode("utf-8")
    except UnicodeDecodeError:
        return None

def file_claude_launches(text, directory=None, environment=None, depth=0, seen=None):
    """text 里读得到正文的文件，读进来认里面起的 claude，文件里再有这样的写法（环境里的变量照样带进去）接着读，同一份只读一遍，MAXIMUM_NESTING 层为限；路径解不出的、读不到的不判
    （子 agent 由 project-code-guard.sh 照认不出的写法拒）。读哪几种：shell 启动时先读的文件（startup_files_of；弄坏开关 startup-file-claude-unread 下不读）；
    bash / sh / source 起的脚本与直接执行的 shell 脚本（路径带斜杠的；同一段命令文本里前面 cat / tee 读 heredoc 写出的拿写出的内容；
    `bash x.sh` 起的脚本正文里起 claude，与 project-code-guard.sh 读脚本正文同一个口子；弄坏开关 script-body-claude-unread 下不读）；
    别的程序读进去当代码的文件（共用模块 embedded_launches 交的 awk -f、sed -f、Makefile，正文交 program_text_launches 切出命令；弄坏开关 embedded-launch-claude-unread 下不读）。"""
    seen = set() if seen is None else seen
    found = []
    if depth >= shell_words.MAXIMUM_NESTING:
        return found
    commands = shell_words.commands_at_command_position(text, directory, environment).commands
    written = heredoc_written_files(commands)
    library = project_code_runs_library()
    for command in commands:
        targets = []   # (路径词, 当前目录, 环境, 是不是直接执行, 程序文件或 None)
        if not detector_break_is_set("startup-file-claude-unread"):
            targets += [(startup.words[0], startup.directory, startup.environment, False, None) for startup in startup_files_of(command)]
        if not detector_break_is_set("script-body-claude-unread") and command.words and (
                command.launcher in ("shell", "source") or (command.launcher is None and "/" in command.words[0])):
            targets.append((command.words[0], command.directory, command.environment, command.launcher is None, None))
        if not detector_break_is_set("embedded-launch-claude-unread"):
            targets += [(word, program_file.directory, command.environment, False, program_file)
                        for program_file in shell_words.embedded_launches(command).program_files for word in program_file.words]
        for word, target_directory, target_environment, executed, program_file in targets:
            read = readable_body(word, target_directory, written, executed)
            if read is None or read[0] in seen:
                continue
            real, body = read
            seen.add(real)
            bodies = shell_words.program_text_launches(program_file.language, body, program_file.variables)[0] if program_file is not None else [
                library.script_directory_expanded(body, real) if library is not None else body]
            for program_text in bodies:
                found += claude_launches_in_text(program_text)
                found += file_claude_launches(program_text, target_directory, target_environment, depth + 1, seen)
    return found

def claude_launches_in_text(command):
    """整条命令每一层（顶层、bash -c、喂给 shell 的 heredoc、命令替换）命令位置上起 claude 的简单命令，前缀与包装剥开之后认；交回认出的那几条（截短）。
    另照共用模块切出的每一条命令认（here-string 与管道喂给 shell 的字面、env -S 拆出来的、名单外启动器 xargs / ssh / find -exec 这类剥开之后的，
    例：`bash <<< 'claude -p hi'`、`env -S 'claude -p hi'`、`xargs claude`、`ssh host claude`）；
    弄坏开关 claude-launch-layers-narrow 下只认前一种。"""
    found = []
    for layer in shell_layers(command):
        for words in layer.commands:
            for _, _, name, arguments in wrapped_commands(words):
                if name == CLAUDE_EXECUTABLE_NAME or name in CLAUDE_EXECUTABLE_PATH_WORDS:
                    found.append(" ".join([name, *arguments])[:120])
    if not detector_break_is_set("claude-launch-layers-narrow"):
        for words in launched_command_words(command):
            name = os.path.basename(words[0])
            if name == CLAUDE_EXECUTABLE_NAME or words[0] in CLAUDE_EXECUTABLE_PATH_WORDS:
                found.append(" ".join([name, *words[1:]])[:120])
    return list(dict.fromkeys(found))


def launched_command_words(text, depth=0):
    """共用模块切出的每一条命令的词（here-string、管道喂给 shell 的字面、env -S、bash -c 都已切开）；命令词是启动器（LAUNCHER_NAMES）的，
    照 lib_project_code_runs.py 的 command_under_launcher 剥掉、里面那条再切一遍接着交（MAXIMUM_NESTING 层为限）。lib_project_code_runs.py 读不到时只交切出来的那一层。"""
    library = project_code_runs_library()
    for command in shell_words.commands_at_command_position(text).commands:
        yield command.words
        # 别的程序（python -c、perl -e、awk、sed e、tclsh、git 别名、make --eval 这类）里字面写着起的命令照一段命令文本接着切（共用模块 embedded_launches；三方 d137-gov-r1 判决 H9；
        # 弄坏开关 embedded-launch-claude-unread 下不认）
        if depth < shell_words.MAXIMUM_NESTING and not detector_break_is_set("embedded-launch-claude-unread"):
            for embedded_text in shell_words.embedded_launches(command).texts:
                yield from launched_command_words(embedded_text, depth + 1)
        if library is not None and depth < shell_words.MAXIMUM_NESTING and os.path.basename(command.words[0]) in LAUNCHER_NAMES:
            inner = library.command_under_launcher(command.words, command.directory)
            if inner is not None:
                yield from launched_command_words(shlex.join(inner[0]), depth + 1)

def claude_launch_refusal(command, agent_type, directory=None):
    """交回要拒的起法；主 agent（没有 agent_type）不判。directory 是命令开头的当前目录（解 BASH_ENV / ENV 指的相对路径用）。弄坏开关 allow-subagent-claude-launch 与 disable-check 下放行。"""
    if not agent_type or detector_break_is_set("disable-check") or detector_break_is_set("allow-subagent-claude-launch"):
        return []
    return claude_launches(command, directory)

# ⑮ 暂存区非空时不带具体路径清整个索引（文件头 ⑮）：git reset 不带路径（--soft 除外），git restore --staged 的路径只有整棵树的写法
GIT_GLOBAL_OPTIONS_WITH_VALUE = {"-C", "-c", "--git-dir", "--work-tree", "--namespace", "--super-prefix", "--config-env", "--exec-path"}
GIT_LOCATION_OPTIONS = {"-C", "--git-dir", "--work-tree"}   # 现查暂存区时照带的（-c 这类配置不带进钩子自己起的 git）
GIT_RESET_INDEX_MODES = {"--mixed", "--hard", "--keep", "--merge"}
GIT_RESET_OPTIONS_WITHOUT_VALUE = {"-q", "--quiet", "--no-quiet", "-N", "--intent-to-add", "--no-refresh", "--refresh", "--recurse-submodules", "--no-recurse-submodules"}
GIT_RESTORE_OPTIONS_WITH_VALUE = {"-s", "--source", "--conflict", "--pathspec-from-file"}
WHOLE_TREE_PATHSPECS = {".", "./", ":/", ":/:", ":(top)", ":(top).", "*", ":/*", ":(glob)**"}
GIT_REVISION_FORMS = re.compile(r"^(?:HEAD|@|ORIG_HEAD|FETCH_HEAD|MERGE_HEAD|[0-9a-f]{7,40})(?:[~^][0-9]*|@\{[^}]*\})*$|^refs/")

def git_location(global_options, directory):
    """全局选项里管「在哪个仓」的那几个：交回 (-C 一层层换到的目录（directory 是 None 或 -C 带变量时交 None）, --git-dir / --work-tree 原样的几个词)；-c 这类配置不交。"""
    kept, index = [], 0
    while index < len(global_options):
        word = global_options[index]
        if word == "-C" and index + 1 < len(global_options):
            value = global_options[index + 1]
            directory = None if directory is None or set("$`*?") & set(value) else os.path.normpath(os.path.join(directory, os.path.expanduser(value)))
            index += 2
            continue
        if word in GIT_LOCATION_OPTIONS and index + 1 < len(global_options):
            kept += global_options[index:index + 2]
            index += 2
            continue
        if word.split("=", 1)[0] in GIT_LOCATION_OPTIONS and "=" in word:
            kept.append(word)
        index += 1
    return directory, kept

def whole_index_pathspec(pathspec):
    """路径都是整棵树的写法（或根本没给）就算清整个索引；弄坏开关 whole-index-pathspec-ignored 下带路径的也算。"""
    if detector_break_is_set("whole-index-pathspec-ignored"):
        return True
    return all(path in WHOLE_TREE_PATHSPECS for path in pathspec)

def git_reset_clears_index(arguments, directory, location=()):
    """git reset 的参数（子命令之后）：清整个索引交 True。--soft 不碰索引；-p 交互不判；--pathspec-from-file 当带了路径。
    没有 -- 时，第一个位置参数是提交（HEAD、HEAD~1、十六进制、refs/…；或不是现成路径而 git rev-parse 认得）还是路径照 git 的认法猜。"""
    options, positional, pathspec, after_separator = [], [], [], False
    for word in arguments:
        if after_separator:
            pathspec.append(word)
        elif word == "--":
            after_separator = True
        elif word.startswith("-"):
            options.append(word)
        else:
            positional.append(word)
    if "--soft" in options or any(option in ("-p", "--patch") or option.startswith("--pathspec-from-file") for option in options):
        return False
    if positional:
        leading_word = positional[0]
        looks_revision = bool(GIT_REVISION_FORMS.match(leading_word)) or (
            directory is not None and not os.path.exists(os.path.join(directory, leading_word)) and
            subprocess.run(["git", "-C", directory, *location, "rev-parse", "--verify", "--quiet", f"{leading_word}^{{commit}}"], capture_output=True, timeout=30).returncode == 0)
        pathspec = (positional[1:] if looks_revision else positional) + pathspec
    return not pathspec or whole_index_pathspec(pathspec)

def git_restore_clears_index(arguments):
    """git restore 的参数：带 --staged / -S（短选项合写的也算）、路径只有整棵树的写法交 True；没给路径（git 自己报错）、--pathspec-from-file 交 False。"""
    staged, pathspec, after_separator, skip = False, [], False, False
    for word in arguments:
        if skip:
            skip = False
            continue
        if after_separator:
            pathspec.append(word)
        elif word == "--":
            after_separator = True
        elif word in GIT_RESTORE_OPTIONS_WITH_VALUE:
            skip = True
        elif word == "--staged" or (word.startswith("-") and not word.startswith("--") and "S" in word[1:] and word[1:].isalpha()):
            staged = True
        elif word.startswith("--pathspec-from-file"):
            return False
        elif not word.startswith("-"):
            pathspec.append(word)
    return staged and bool(pathspec) and whole_index_pathspec(pathspec)

def git_other_form_clears_index(subcommand, arguments):
    """git stash / git rm --cached / git checkout 清不清整个索引：
    `git stash`、`git stash push|save`（不带 -k / --keep-index、不带路径或路径是整棵树）把暂存的改动收走；`git rm --cached` 的路径是整棵树（`-r --cached .`）把索引里的文件全删；
    `git checkout [<提交>] -- <整棵树>` 拿提交或索引盖掉整棵树的暂存改动。别的子命令与带具体路径的交 False。"""
    if subcommand == "stash":
        action = arguments[0] if arguments and not arguments[0].startswith("-") else "push"
        rest = arguments[1:] if arguments and not arguments[0].startswith("-") else arguments
        if action not in ("push", "save"):
            return False
        if any(option in ("-k", "--keep-index") for option in rest):
            return False
        pathspec = [word for word in (rest[rest.index("--") + 1:] if "--" in rest else rest) if not word.startswith("-")] if action == "push" else []
        return not pathspec or whole_index_pathspec(pathspec)
    if subcommand == "rm":
        if "--cached" not in arguments:
            return False
        pathspec = [word for word in (arguments[arguments.index("--") + 1:] if "--" in arguments else arguments) if not word.startswith("-")]
        return bool(pathspec) and whole_index_pathspec(pathspec)
    if subcommand == "checkout":
        if "--" in arguments:
            pathspec = arguments[arguments.index("--") + 1:]
        else:
            positional = [word for word in arguments if not word.startswith("-")]
            pathspec = positional[1:] if positional and GIT_REVISION_FORMS.match(positional[0]) else []
        return bool(pathspec) and whole_index_pathspec(pathspec)
    return False

def staged_index_state(location, directory):
    """暂存区里有没有东西：'有'、'空' 或 '判不了'（目录算不出、不是 git 仓、git 起不来）。弄坏开关 whole-index-staged-unchecked 下一律交 '有'。"""
    if detector_break_is_set("whole-index-staged-unchecked"):
        return "有"
    if directory is None:
        return "判不了"
    try:
        completed = subprocess.run(["git", "-C", directory, *location, "diff", "--cached", "--quiet"], capture_output=True, timeout=30)
    except (OSError, subprocess.SubprocessError):
        return "判不了"
    return {0: "空", 1: "有"}.get(completed.returncode, "判不了")

# ⑱ 带路径撤掉工作区里没暂存的改动（文件头 ⑱）：git checkout [<提交>] [--] <路径…>、git restore <路径…>（不带 --staged，或带 --worktree）、
#   git stash [push] [--] <路径…>、git clean <路径…>；点名的路径里有没暂存的改动（git clean 是会被删的没跟踪文件）就拒
GIT_CHECKOUT_BRANCH_OPTIONS = {"-b", "-B", "--orphan", "--detach"}
GIT_CHECKOUT_OPTIONS_WITH_VALUE = {"--conflict", "--pathspec-from-file"}
GIT_STASH_OPTIONS_WITH_VALUE = {"-m", "--message", "--pathspec-from-file"}
GIT_CLEAN_OPTIONS_WITH_VALUE = {"-e", "--exclude"}
PATHSPEC_FROM_FILE_OPTION = "--pathspec-from-file"

class WorktreeDiscard(NamedTuple):
    paths: list           # 命令点名的路径（照写的样子，变量还没展开）
    untracked: bool       # True：git clean，删的是没跟踪的文件；False：撤的是工作区比暂存区多出来的改动
    ignored: bool         # git clean 带 -x / -X：被忽略的文件也删
    unresolved: str       # 判不了的原因（路径从 --pathspec-from-file 读）；判得了是空串

def short_option_cluster_has(word, letter):
    """短选项合写（-fdx 这类）里有没有 letter；长选项与不是选项的词交 False。"""
    return word.startswith("-") and not word.startswith("--") and len(word) > 1 and word[1:].isalpha() and letter in word[1:]

def git_arguments_split(arguments, options_with_value):
    """子命令的参数切成 (-- 之前的位置参数, -- 之后的词, 选项, 有没有 --)；options_with_value 里分开写的选项连同它的值一起跳过（值不进位置参数）。"""
    positional, after, options, skip, separated = [], [], [], False, False
    for word in arguments:
        if skip:
            skip = False
        elif separated:
            after.append(word)
        elif word == "--":
            separated = True
        elif word in options_with_value:
            options.append(word)
            skip = True
        elif word.startswith("-") and word != "-":
            options.append(word)
        else:
            positional.append(word)
    return positional, after, options, separated

def git_names_commit(word, directory, location):
    """不带 -- 的 git checkout 第一个位置参数是不是提交：像提交的写法（HEAD、十六进制、refs/…），或盘上没有这个路径而 git rev-parse 认得（同 git_reset_clears_index）。"""
    if GIT_REVISION_FORMS.match(word):
        return True
    if directory is None or os.path.exists(os.path.join(directory, word)):
        return False
    return subprocess.run(["git", "-C", directory, *location, "rev-parse", "--verify", "--quiet", f"{word}^{{commit}}"],
                          capture_output=True, timeout=30).returncode == 0

def worktree_discard_paths(subcommand, arguments, directory, location):
    """⑱ 认写法：带路径撤工作区改动的 git 命令交 WorktreeDiscard；没点名路径的、只动暂存区的 restore --staged、切分支、git clean -n、别的子命令交 None。
    弄坏开关 path-discard-staged-counted 下 restore --staged（不带 --worktree）也当动工作区。"""
    untracked = ignored = False
    if subcommand == "checkout":
        positional, after, options, separated = git_arguments_split(arguments, GIT_CHECKOUT_OPTIONS_WITH_VALUE)
        if any(option.split("=", 1)[0] in GIT_CHECKOUT_BRANCH_OPTIONS for option in options):
            return None
        if separated:
            paths = after
        elif positional and git_names_commit(positional[0], directory, location):
            paths = positional[1:]
        else:
            paths = positional
    elif subcommand == "restore":
        positional, after, options, _ = git_arguments_split(arguments, GIT_RESTORE_OPTIONS_WITH_VALUE)
        staged = any(option == "--staged" or short_option_cluster_has(option, "S") for option in options)
        worktree = any(option == "--worktree" or short_option_cluster_has(option, "W") for option in options)
        if staged and not worktree and not detector_break_is_set("path-discard-staged-counted"):
            return None
        paths = positional + after
    elif subcommand == "stash":
        explicit = bool(arguments) and not arguments[0].startswith("-")
        if explicit and arguments[0] != "push":
            return None
        positional, after, options, _ = git_arguments_split(arguments[1:] if explicit else arguments, GIT_STASH_OPTIONS_WITH_VALUE)
        paths = positional + after
    elif subcommand == "clean":
        positional, after, options, _ = git_arguments_split(arguments, GIT_CLEAN_OPTIONS_WITH_VALUE)
        if any(option == "--dry-run" or short_option_cluster_has(option, "n") for option in options):
            return None
        paths = positional + after
        untracked = True
        ignored = any(short_option_cluster_has(option, "x") or short_option_cluster_has(option, "X") for option in options)
    else:
        return None
    if any(option.split("=", 1)[0] == PATHSPEC_FROM_FILE_OPTION for option in options):
        return WorktreeDiscard([], untracked, ignored, f"路径从 {PATHSPEC_FROM_FILE_OPTION} 读")
    if not paths:
        return None
    return WorktreeDiscard(paths, untracked, ignored, "")

def unstaged_hits(discard, paths, location, directory):
    """点名的路径里有没暂存改动的（git clean 是会被删的没跟踪文件）：交回它们的路径（git 打的、相对仓根）；git 报错（不是 git 仓、路径在仓外：命令自己也跑不成）交 []；
    git 起不来交 None（判不了）。弄坏开关 path-discard-changes-unchecked 下不查，点名的路径都算有改动。"""
    if detector_break_is_set("path-discard-changes-unchecked"):
        return list(paths)
    if discard.untracked:
        query = ["status", "--porcelain=v1", "-z", "--untracked-files=all"] + (["--ignored"] if discard.ignored else []) + ["--", *paths]
    else:
        query = ["diff", "--name-only", "-z", "--", *paths]
    try:
        completed = subprocess.run(["git", "-C", directory, *location, *query], capture_output=True, timeout=30,
                                   env=dict(os.environ, GIT_OPTIONAL_LOCKS="0", LC_ALL="C"))
    except (OSError, subprocess.SubprocessError):
        return None
    if completed.returncode != 0:
        return []
    entries = [entry for entry in completed.stdout.decode("utf-8", "replace").split("\0") if entry]
    if discard.untracked:
        return [entry[3:] for entry in entries if entry[:3] in ("?? ", "!! ")]
    return entries

# ㉑ 前台 Bash 调用的 tool_input.timeout 超过 FOREGROUND_TIMEOUT_MILLISECONDS（文件头 ㉑）：主 agent 与子 agent 一样拒
FOREGROUND_TIMEOUT_MILLISECONDS = 240000

def foreground_timeout_refusal(hook_input):
    """前台（run_in_background 不是 true）的 tool_input.timeout 超过 FOREGROUND_TIMEOUT_MILLISECONDS：交回写的毫秒数（整数照整数，小数照小数）；
    run_in_background 为 true、没写、写的不是数（布尔、认不出的字符串）、不超过的交 None。弄坏开关 allow-long-foreground-timeout 下一概交 None。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    if tool_input.get("run_in_background") is True or detector_break_is_set("allow-long-foreground-timeout"):
        return None
    value = tool_input.get("timeout")
    if isinstance(value, str) and re.fullmatch(r"\s*\d+(?:\.\d+)?\s*", value):
        value = float(value)
    if isinstance(value, bool) or not isinstance(value, (int, float)) or value <= FOREGROUND_TIMEOUT_MILLISECONDS:
        return None
    return int(value) if float(value).is_integer() else value

def foreground_timeout_messages(milliseconds, agent_type):
    """㉑ 拒绝说明的两行（原因，出路）；出路按调用方写：子 agent 交回主 agent，主 agent 改 run_in_background 与 task wait。"""
    reason = (f"  ✗ 前台命令的 timeout 是 {milliseconds} 毫秒，超过 {FOREGROUND_TIMEOUT_MILLISECONDS} 毫秒：前台命令卡在这一次调用里，"
              "发给你的消息要等它返回才送得到，最长的 600000 毫秒就是十分钟")
    if agent_type:
        way = ("     → 怎么办：这一条不前台起。跑项目代码的命令照共用约束 .claude/agent-common.md「不做」一节「子 agent 不交任务」那一条拼成 submit 的整条，"
               "写进报告「要主 agent 跑的命令」一节，结束本轮交回主 agent，由主 agent 经任务服务交；只读的短命令前台跑、timeout 不超过 "
               f"{FOREGROUND_TIMEOUT_MILLISECONDS}。这一道只拒这种写法，不停你在跑的任何东西")
    else:
        way = ("     → 怎么办：命令改用 run_in_background: true 起、不设 timeout，结束本轮、等完成通知再接着做；跑项目代码的经 `task-server/task submit -- <命令>` 交，"
               "再用 run_in_background 起 `task-server/task wait <号>`（任务完成时退出，末行 TASK_SERVER_RESULT），起完结束本轮；"
               f"确要前台跑的，timeout 不超过 {FOREGROUND_TIMEOUT_MILLISECONDS}。这一道只拒这种写法，不停你在跑的任何东西")
    return reason, way

# ㉓ 交给任务服务的 cargo build 点了全部目标（文件头 ㉓）：主 agent 与子 agent 一样拒。认 submit 照 lib_project_code_runs.py 的 task_server_submission，
#   剥包装、认 cargo 子命令照 lib_workspace_cargo_lock.py 的 cargo_invocations（两份都按文件路径导入，这里不另写切词与剥法）
WHOLE_TARGET_BUILD_SUBCOMMANDS = ("build", "b")   # cargo build 与它的内建别名
WHOLE_TARGET_BUILD_FLAGS = ("--all-targets", "--tests", "--benches", "--examples", "--workspace", "--all")
WHOLE_TARGET_FULL_GATE_OPTION = "--full-gate"     # 提交时的整轮门禁带它交，这一道不拒
WHOLE_TARGET_SHOWN_CHARACTERS = 160               # 拒绝说明里每条交给服务的命令最多列这么多字
WORKSPACE_CARGO_LOCK_LIBRARY = []   # 空表是还没装；装过的放一个模块，装不上放 None

def workspace_cargo_lock_library():
    """同目录的 lib_workspace_cargo_lock.py（cargo_invocations 剥包装、认 cargo 子命令）：第一次用到才按文件路径装；装不上记一条判不了、交 None。"""
    if not WORKSPACE_CARGO_LOCK_LIBRARY:
        try:
            specification = importlib.util.spec_from_file_location("lib_workspace_cargo_lock", os.path.join(HOOK_DIRECTORY, "lib_workspace_cargo_lock.py"))
            module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(module)
            WORKSPACE_CARGO_LOCK_LIBRARY.append(module)
        except Exception as error:   # 文件不在、语法错、导入时抛的都算读不到
            not_judged(f"bash-command-detector.sh 读不到 lib_workspace_cargo_lock.py（{error!r}），交给任务服务的 cargo build 点没点全部目标这一次没判")
            WORKSPACE_CARGO_LOCK_LIBRARY.append(None)
    return WORKSPACE_CARGO_LOCK_LIBRARY[0]

def submit_options_have_full_gate(submit_words, library):
    """submit 自己的词（`submit` 起到 `--` 为止，task_server_submission 认过的那几个选项）里有没有 --full-gate；--cwd 与 --whole-package-confirmed 的值跳过
    （值恰好写成 --full-gate 的不算）。弄坏开关 whole-target-full-gate-unseen 下一律交 False。"""
    if detector_break_is_set("whole-target-full-gate-unseen"):
        return False
    valued = (library.TASK_SERVER_SUBMIT_DIRECTORY_OPTION, library.TASK_SERVER_SUBMIT_VALUE_OPTION)
    position = 0
    while position < len(submit_words):
        if submit_words[position] == WHOLE_TARGET_FULL_GATE_OPTION:
            return True
        position += 2 if submit_words[position] in valued else 1
    return False

def whole_target_build_refusal(command, start_directory):
    """㉓：命令位置上的 task-server/task submit（lib_project_code_runs.py 的 task_server_submission 认的那一种；bash -c、前缀、命令替换里的照认）`--` 之后那一条里，
    命令位置上会编的 cargo build（lib_workspace_cargo_lock.py 的 cargo_invocations：env、bash -c、cd、内存包装这类剥开，任务的当前目录取 --cwd）
    带 WHOLE_TARGET_BUILD_FLAGS 之一的：交 [(交给服务的那一条, [点到的选项…])]。submit 带 --full-gate 的放行；只判写在命令里的那一条，它起的脚本正文不读；
    别的 cargo 子命令（clippy、test……）不判。两份库读不到的这一次不判（各记一条判不了）。主 agent 与子 agent 一样判。弄坏开关 allow-whole-target-build 下一概交 []。"""
    if detector_break_is_set("allow-whole-target-build") or "submit" not in command or "cargo" not in command:
        return []
    library = project_code_runs_library()
    cargo_lock = workspace_cargo_lock_library() if library is not None else None
    if cargo_lock is None:
        return []
    found = []
    for command_at in shell_words.commands_at_command_position(command, start_directory).commands:
        words = list(command_at.words)
        submitted = library.task_server_submission(words, command_at.directory, command_at.launcher)
        if submitted is None:
            continue
        inner, task_directory = list(submitted[0]), submitted[1]
        if submit_options_have_full_gate(words[1:len(words) - len(inner)], library):
            continue
        flags = []
        for run in cargo_lock.cargo_invocations(shlex.join(inner), task_directory or command_at.directory, shell_words, library):
            if run.subcommand not in WHOLE_TARGET_BUILD_SUBCOMMANDS:
                continue
            options = run.rest[: run.rest.index("--")] if "--" in run.rest else run.rest
            flags += [flag for flag in WHOLE_TARGET_BUILD_FLAGS if flag in options and flag not in flags]
        if flags:
            found.append((shlex.join(inner)[:WHOLE_TARGET_SHOWN_CHARACTERS], flags))
    return found

# 出路里的三种 cargo 写法，自证从拒绝说明里取出来照抄喂别的闸；占位照 WHOLE_TARGET_HOWTO_SAMPLES 换成样本
WHOLE_TARGET_HOWTO_COMMAND = re.compile(r"`(cargo (?:clippy|test|build) [^`]*)`")
WHOLE_TARGET_HOWTO_SAMPLES = (("<包>", "singlefs-format"), ("<目标>", "format_layout"), ("<测试全名…>", "layout::sample_round_trip"))

# 出路里 cargo clippy 的 `--` 之后整段：项目代码闸 project-code-guard.sh（文件头「经 task-server 起」⑤）要 clippy 带 -D warnings 与 check.sh 里 CODE_DISCIPLINE_LINTS 那张表的每一条，
#   表每次现读（这一份钩子所在的仓里的那一份）；读法在同目录的 lib_code_discipline_lints.py，project-code-guard.sh 读的是同一份
CLIPPY_LINT_TABLE_RELATIVE = os.path.join(".claude", "singlefs-ai-sop", "scripts", "check.sh")
CLIPPY_LINT_LIBRARY = []   # 同目录的 lib_code_discipline_lints.py，第一次用到才导入；导入不了是 [None]

def clippy_lint_library():
    if not CLIPPY_LINT_LIBRARY:
        try:
            specification = importlib.util.spec_from_file_location("lib_code_discipline_lints", os.path.join(HOOK_DIRECTORY, "lib_code_discipline_lints.py"))
            module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(module)
            CLIPPY_LINT_LIBRARY.append(module)
        except Exception as error:   # 文件不在、语法错
            not_judged(f"bash-command-detector.sh 读不到 lib_code_discipline_lints.py（{error!r}），㉓ 出路里的 cargo clippy 写法这一次没带 lint 表")
            CLIPPY_LINT_LIBRARY.append(None)
    return CLIPPY_LINT_LIBRARY[0]

def clippy_lint_tail(check_script):
    """check_script（check.sh）里 CODE_DISCIPLINE_LINTS=( … ) 那张表现读的 `-- -D warnings <表里的词…>` 整段（lib_code_discipline_lints.table_tail，表里的词原样，与整轮门禁真传给 clippy 的相同）；
    读不到文件、没有那一段、有那一段而写法读不成（只认 `-D clippy::<名>` 一行一条，见 lib_code_discipline_lints.py 文件头）交空串并记一条判不了。"""
    library = clippy_lint_library()
    tail = library.table_tail(check_script) if library is not None else ""
    if not tail and library is not None:
        problem = library.table_problem(check_script)
        unreadable = f"（表有而写法读不成：{problem}；project-code-guard.sh 的 ⑤ 对读不成的表一概拒 clippy，出路交上游改成 `-D clippy::<名>` 一行一条）" if problem else ""
        not_judged(f"bash-command-detector.sh 在 {check_script} 里没读到 CODE_DISCIPLINE_LINTS 那张表{unreadable}，㉓ 出路里的 cargo clippy 写法这一次没带 lint 表")
    return tail
