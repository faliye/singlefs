# 检出钩子 bash-command-detector.sh 的 python 部分之一：自证函数 selftest_in_entries_and_summary：通删工作根、真实入口的样本、弄坏开关对账、汇总，与入口 main。
# 入口 bash-command-detector.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 DETECTOR_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
import importlib.util
import json
import os
import shlex
import shutil
import subprocess
import sys
from __main__ import DETECTOR_BREAK_VARIABLE, DETECTOR_PART_NAMES, detector_break_is_set
from bash_command_detector_hooks_and_overwrites import (
    APPENDING_REDIRECTS, FILE_DESCRIPTOR_WORD, GIT_HOOKS_INSTALL_COMMAND, HOOKS_BYPASS_WINDOW_VARIABLE, OVERWRITING_REDIRECTS, OverwriteCandidate,
    PythonCode, command_pieces, git_hook_refusal, git_hooks_library, record, subagent_background_refusal, substitute_assigned_variables,
    target_pattern, tee_files, unwaited_ampersand_refusal, wait_loop_refusal, whole_index_reset_refusal, worktree_discard_refusal,
)
from bash_command_detector_literals_and_signals import (
    LEDGER_HEADING_TEXT, LEDGER_REGISTRY_PATH, LEDGER_WRITING_FORMS, LITERAL_NOTATION_FORMS, LITERAL_NOTATION_LISTED, LITERAL_WHOLE_FILE_FORM,
    LiteralWrite, MANUAL_COPY_HOWTO_COMMAND, MANUAL_COPY_HOWTO_COPY_PLACEHOLDER, MINIMUM_ANNOTATION_REASON_CHARACTERS, OWN_SCOPE_EXEMPTABLE_KINDS,
    OWN_SCOPE_GUARD, OWN_SCOPE_GUARD_LOOKBACK_LINES, PENDING_LIST, PYTHON_OPENING_MODE, RECORD_HOWTO_COMMAND, RECORD_HOWTO_TEXT_PLACEHOLDER,
    SCANNED_SUFFIXES, WORK_ROOT_SWEEP_SCRIPT, bash_scope_refusal, compile_first_refusal, current_ledger_relative, follow_simple_command,
    howto_guard_environment, lands_on_ledger, literal_landing_text, literal_notation_remedies, literal_output, literal_write_landing,
    load_notations_library, manual_copy_howto, manual_repository_copy_refusal, overwrite_steps, process_signal_refusal,
    python_in_place_writes, record_append_landing, record_howto_tool, repository_in_place_edit_verdict, repository_root_of,
    results_overwrite_refusal, scan_script_file, script_in_place_refusal, service_source_refusal, whole_target_build_messages,
    work_root_removal_refusal,
)
from bash_command_detector_scope_and_index import (
    CLIPPY_LINT_TABLE_RELATIVE, COMPILE_THEN_SWAP_SCRIPT, DEFAULT_DETECTIONS, FIXTURE_NOTE_ENDINGS, HOOK_DIRECTORY, InPlaceEditVerdict, JUDGING_NOTICES,
    KNOWN_DETECTOR_BREAK_ITEMS, LAUNCHER_NAMES, SHELL_WORDS_PART_NAMES, ScopeVerdict, UNREADABLE_HOOK_INPUT, WHOLE_TARGET_HOWTO_COMMAND,
    WHOLE_TARGET_HOWTO_SAMPLES, WRITE_SCOPE_TABLE_NAME, append_detection, break_switch_set_mismatches, claude_launch_refusal, clippy_lint_tail,
    detaching_refusal, foreground_timeout_messages, foreground_timeout_refusal, load_shell_words, not_judged, parsed_hook_input,
    project_code_runs_library, refuse_break_outside_selftest, shell_words, shell_words_error, shell_words_library_path, watchdog_rejection,
    whole_target_build_refusal, work_roots, wrapped_commands,
)
from bash_command_detector_selftest_waits_and_scope import (
    selftest_in_waits_and_scope,
)
import bash_command_detector_hooks_and_overwrites

def selftest_in_entries_and_summary(shared):
    detections, handle, hook_dir, new_root, read_only_scope_not_run, removal_cases, removal_root, repository, repository_hooks, results, running, scope_hooks, scope_repository, through_entry, whole_target_root, work = shared["detections"], shared["handle"], shared["hook_dir"], shared["new_root"], shared["read_only_scope_not_run"], shared["removal_cases"], shared["removal_root"], shared["repository"], shared["repository_hooks"], shared["results"], shared["running"], shared["scope_hooks"], shared["scope_repository"], shared["through_entry"], shared["whole_target_root"], shared["work"]
    if "value" in shared:
        value = shared["value"]
    if "background" in shared:
        background = shared["background"]
    if "command" in shared:
        command = shared["command"]
    if "hook_payload" in shared:
        hook_payload = shared["hook_payload"]
    if "label" in shared:
        label = shared["label"]
    if "line" in shared:
        line = shared["line"]
    if "name" in shared:
        name = shared["name"]
    if "relative" in shared:
        relative = shared["relative"]
    if "want" in shared:
        want = shared["want"]
    if "want_notices" in shared:
        want_notices = shared["want_notices"]
    for label, command, want in removal_cases:
        results.append((f"通删工作根:{label}", want, 1 if work_root_removal_refusal(command, work) else 0))
    removal_scene = through_entry(f"rm -rf {removal_root}/*")[0]
    results.append(("stdin:rm -rf 工作根下通配拒绝（退出码 2）", 2, removal_scene.returncode))
    results.append(("stdin:通删工作根的拒绝写了出路（work-directory-ownership.sh 的 sweep 与 claim）", 1,
                    int("✗" in removal_scene.stderr and "→" in removal_scene.stderr
                        and f"bash {WORK_ROOT_SWEEP_SCRIPT} sweep --dry-run" in removal_scene.stderr
                        and f"bash {WORK_ROOT_SWEEP_SCRIPT} claim <目录> --purpose <一句>" in removal_scene.stderr)))
    results.append(("stdin:rm -rf 工作根下一个具体目录放行（退出码 0）", 0, through_entry(f"rm -rf {removal_root}/owed-x")[0].returncode))
    # 走真实入口：experiment-runner 在 Bash 里 cp 进主工作区 crates/ 下的 .rs 拒绝（退出码 2）、stderr 点名先编后换那条脚本；implementation-writer 放行
    def entry_as(agent_type, command):
        return subprocess.run(["bash", os.path.join(repository_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
                              env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections),
                              input=json.dumps({"tool_name": "Bash", "session_id": "s", "agent_type": agent_type,
                                                "tool_input": {"command": command, "run_in_background": False}}))
    def entry_in_background_as(agent_type, command, working_directory=None):
        hook_payload = {"tool_name": "Bash", "session_id": "s", "tool_input": {"command": command, "run_in_background": True}}
        if agent_type:
            hook_payload["agent_type"] = agent_type
        if working_directory:   # 真实输入带 cwd：客户端 task-server/task 写的是相对路径，按 cwd 解；不带 cwd 就解到临时仓，那里没有客户端
            hook_payload["cwd"] = working_directory
        return subprocess.run(["bash", os.path.join(repository_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
                              env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections), input=json.dumps(hook_payload))
    subagent_background = entry_in_background_as("implementation-writer", "cargo build -p singlefs-core")
    results.append(("stdin:子 agent 用 run_in_background 起命令拒绝（退出码 2）", 2, subagent_background.returncode))
    results.append(("stdin:子 agent 的 run_in_background 拒绝时 stderr 写了不起、写进报告「要主 agent 跑的命令」一节、结束本轮交回的出路，不教后台等", 1,
                    int("✗" in subagent_background.stderr and "→" in subagent_background.stderr
                        and "写进报告「要主 agent 跑的命令」一节交回" in subagent_background.stderr and "结束本轮交回主 agent" in subagent_background.stderr
                        and "task-server/task wait <号>" not in subagent_background.stderr)))
    # ⑩ 子 agent 的 run_in_background 一律拒，task-server/task wait 也拒（allow-subagent-background、disable-check 下这几格红）；
    # cwd 给本仓根（临时仓里没有 task-server/task），客户端在盘上也照拒
    task_repository = repository_root_of(hook_dir)
    for label, command, want in (
            ("stdin:子 agent 后台起 task-server/task wait 7 照拒（退出码 2）", "task-server/task wait 7", 2),
            ("stdin:子 agent 后台起 cd <仓根> && task-server/task wait 7 照拒（退出码 2）", f"cd {task_repository} && task-server/task wait 7", 2),
            ("stdin:子 agent 后台起 task-server/task wait 7 && echo x 照拒（退出码 2）", "task-server/task wait 7 && echo x", 2),
            ("stdin:子 agent 后台起 sleep 5 照拒（退出码 2）", "sleep 5", 2),
            ("stdin:子 agent 后台起 cargo build 照拒（退出码 2）", "cargo build", 2),
            ("stdin:子 agent 后台起 task-server/task wait 后面不是号照拒（退出码 2）", "task-server/task wait x", 2),
            ("stdin:子 agent 后台起 task-server/task status 照拒（退出码 2）", "task-server/task status", 2)):
        results.append((label, want, entry_in_background_as("implementation-writer", command, task_repository).returncode))
    results.append(("stdin:主 agent 用 run_in_background 起同一条命令放行（退出码 0）", 0, entry_in_background_as(None, "cargo build -p singlefs-core").returncode))
    results.append(("stdin:子 agent 前台起同一条命令放行（退出码 0）", 0, entry_as("implementation-writer", "cargo build -p singlefs-core").returncode))
    runner_copy = entry_as("experiment-runner", "cp ~/.local/share/singlefs/work/runner-x/e161_x.rs crates/singlefs-verification/src/bin/e161_x.rs")
    results.append(("stdin:experiment-runner cp 进主工作区的入库装置拒绝（退出码 2）", 2, runner_copy.returncode))
    results.append(("stdin:先编后换的拒绝点名那条脚本与出路", 1,
                    int("✗" in runner_copy.stderr and "→" in runner_copy.stderr and COMPILE_THEN_SWAP_SCRIPT in runner_copy.stderr)))
    # 出路经 task-server 提交、不再教 capped.sh 直接起（BASH_COMMAND_DETECTOR_BREAK=compile-first-capped-hint、disable-check、allow-compile-first-bypass 下这一格红）
    results.append(("stdin:先编后换的出路经 task-server 提交、不写 capped.sh 直接起", 1,
                    int(f"task-server/task submit -- python3 {COMPILE_THEN_SWAP_SCRIPT}" in runner_copy.stderr and "capped.sh" not in runner_copy.stderr)))
    writer_copy = entry_as("implementation-writer", "cp ~/.local/share/singlefs/work/runner-x/e161_x.rs crates/singlefs-verification/src/bin/e161_x.rs")
    results.append(("stdin:implementation-writer 同样的 cp 放行（退出码 0）", 0, writer_copy.returncode))
    # 走真实入口：子 agent 在 Bash 里起 claude 会话拒绝（弄坏开关 allow-subagent-claude-launch 与 disable-check 下前四格红）；主 agent 起同一条放行
    claude_refused = entry_as("implementation-writer", "claude -p '接着做'")
    results.append(("stdin:子 agent 起 claude 拒绝（退出码 2）", 2, claude_refused.returncode))
    results.append(("stdin:子 agent 经 \"$CLAUDE_CODE_EXECPATH\" 起 claude 拒绝（退出码 2）", 2, entry_as("clerk", '"$CLAUDE_CODE_EXECPATH" -p x').returncode))
    results.append(("stdin:子 agent 起以 /claude 结尾的可执行文件拒绝（退出码 2）", 2, entry_as("tooling-writer", "/home/u/.local/bin/claude --print x").returncode))
    results.append(("stdin:子 agent 起 claude 被拒时 stderr 点名「派发」一节、写交回主 agent 的出路", 1,
                    int("✗" in claude_refused.stderr and "→" in claude_refused.stderr and "「派发」" in claude_refused.stderr and "由主 agent 派" in claude_refused.stderr)))
    results.append(("stdin:主 agent 起同一条 claude 放行（退出码 0）", 0, entry_as(None, "claude -p '接着做'").returncode))
    launcher_library = project_code_runs_library()
    launcher_names = sorted(launcher_library.launcher_names()) if launcher_library is not None else None
    results.append(("启动器名单与 lib_project_code_runs 的一致", sorted(LAUNCHER_NAMES), launcher_names))
    # 照 bash 不跑的放行：case 里 ;; 之后的模式词（case-patterns-as-commands 下这两格红）
    for label, agent_type, command in (("case 里 ;; 之后的 reboot 模式不当命令", "implementation-writer", 'case "$1" in a) :;; reboot) :;; esac'),
                                       ("case 里 ;; 之后的 claude 模式不当命令", "implementation-writer", 'case "$1" in\n  start) echo s;;\n  claude) echo c;;\nesac')):
        results.append((f"stdin:子 agent {label}放行（退出码 0）", 0, entry_as(agent_type, command).returncode))
    results.append(("stdin:子 agent case 一项的命令里的 reboot 照拒（退出码 2）", 2, entry_as("implementation-writer", 'case "$1" in a) reboot;; esac').returncode))
    # 钩子认得出、判错了的几处，每处一对放与拒，
    # 拒的那一侧照报告里的同形写法；放的那一格后面的注释写弄坏它的开关（共用模块 LIB_SHELL_WORDS_BREAK=<项>，检出钩子 BASH_COMMAND_DETECTOR_BREAK=<项>）
    def d137_command(text, name):
        return next(command for command in shell_words.commands_at_command_position(text, work).commands if command.words and os.path.basename(command.words[0]) == name)
    def d137_python_scan(code):
        found_texts, found_unjudged = [], []
        shell_words.scan_python(code, found_texts, found_unjudged)
        return found_texts, found_unjudged
    def d137_sed_scan(script):
        found_texts, found_unjudged = [], []
        shell_words.sed_script_commands(script, found_texts, found_unjudged, shell_word=True)
        return found_texts, found_unjudged
    heredoc_code, heredoc_problem, _ = shell_words.interpreter_standard_input(d137_command('D=/tmp/d137-e1; python3 - <<EOF\nprint("$D")\nEOF', "python3"))
    results.append(('d137 误拒 E1：python3 读 heredoc，正文变量前面赋过字面值，照代入、不交判不了', (None, True), (heredoc_problem, 'print("/tmp/d137-e1")' in heredoc_code)))   # heredoc-variables-unexpanded
    for label, text in (("正文里有一个变量没赋过", 'python3 - <<EOF\nprint("$D137_NEVER_ASSIGNED")\nEOF'),
                        ("正文里有命令替换", 'D=/tmp/d137-e1; python3 - <<EOF\nprint("$D $(date)")\nEOF')):
        results.append((f"d137 误拒 E1：python3 读 heredoc，{label}，照交判不了", True, shell_words.interpreter_standard_input(d137_command(text, "python3"))[1] is not None))
    results.append(('d137 误拒 E6：subprocess.run([du, -sch] + ds)，ds 是赋过的字面列表，照算出的命令交出', (["du -sch f.rs"], []),
                    d137_python_scan('import subprocess; ds=["f.rs"]; subprocess.run(["du","-sch"]+ds)')))   # python-list-concatenation-unresolved
    results.append(('d137 误拒 E6：subprocess.run([du, -sch, *ds])，ds 算不出，照命令词 du 交出', ([f"du -sch {shell_words.PYTHON_UNRESOLVED_ARGUMENT}"], []),
                    d137_python_scan('import subprocess, sys\nds = sys.argv[1:]\nsubprocess.run(["du", "-sch", *ds])')))   # 同上
    for label, code in (("subprocess.run([git, log] + ds)", 'import subprocess, sys\nds = sys.argv[1:]\nsubprocess.run(["git", "log"] + ds)'),
                        ("subprocess.run([git, log, *ds])", 'import subprocess, sys\nds = sys.argv[1:]\nsubprocess.run(["git", "log", *ds])')):
        git_texts, git_unjudged = d137_python_scan(code)
        results.append((f"d137 误拒 E6：{label}，ds 算不出，命令词 git 照认不出", ([], True), (git_texts, bool(git_unjudged))))
    launcher_library = project_code_runs_library()
    known_launchers = ((set(launcher_library.LAUNCHER_OPTIONS_WITH_VALUE) | set(launcher_library.EXTRA_LAUNCHER_NAMES)) if launcher_library is not None
                       else {"读不到 lib_project_code_runs.py"})
    results.append(("d137 误拒 E6：lib_project_code_runs.py 认的启动器都在共用模块的 ARGUMENT_COMMAND_LAUNCHER_NAMES 里", set(),
                    known_launchers - shell_words.ARGUMENT_COMMAND_LAUNCHER_NAMES))
    results.append(('d137 误拒 E7：python 只调 compile 不执行，不交判不了', [], d137_python_scan("import sys; compile(open(sys.argv[1]).read(), sys.argv[1], 'exec')")[1]))   # python-compile-as-execution
    results.append(("d137 误拒 E7：python 调 exec 跑读进来的代码，照交判不了", True, bool(d137_python_scan("exec(open('gen.py').read())")[1])))
    results.append(("d137 误拒 E7：compile 的代码是字面的，照扫字面代码", (["cargo test --workspace"], []),
                    d137_python_scan("""compile("import subprocess; subprocess.run(['cargo', 'test', '--workspace'])", 'x', 'exec')""")))
    results.append(('d137 误拒 E8：sed 的 s 命令标志区有 GNU sed 不认的字（分隔符被转义的反斜杠提前收尾），编译就报错、什么都不起', ([], []), d137_sed_scan("s|ab\\\\| none zz|ab\\\\| none zz|")))   # sed-unknown-flags-as-execution
    for label, script in (("s|ab|cd|e 真带 e 标志", "s|ab|cd|e"), ("s/a/b/e#c 的 e 标志在 # 前面", "s/a/b/e#c")):
        results.append((f"d137 误拒 E8：sed 的 {label}，照交判不了", True, bool(d137_sed_scan(script)[1])))
    missing_directory = os.path.join(work, "d137-missing-directory")
    for label, text, name, want in (('d137 误拒 E9：cd 不在的目录 && ( bash x.sh )，子 shell 照 && 守着', f"cd {missing_directory} && (bash x.sh)", "x.sh", [True]),         # cd-subshell-guards-unseen
                                    ('d137 误拒 E9：cd 不在的目录 && time ( bash x.sh )，照 && 守着', f"cd {missing_directory} && time (bash x.sh)", "x.sh", [True]),    # 同上
                                    ("d137 误拒 E9：cd 不在的目录 && 子 shell 之后用分号接的 bash y.sh 照跑，不守", f"cd {missing_directory} && (bash x.sh); bash y.sh", "y.sh", [False]),
                                    ("d137 误拒 E9：cd 不在的目录; ( bash x.sh ) 照跑，不守", f"cd {missing_directory}; (bash x.sh)", "x.sh", [False])):
        results.append((label, want, [command.cd_failure_skips for command in shell_words.commands_at_command_position(text, work).commands if command.words[:1] == [name]]))
    results.append(("d137 误拒：名字() { 函数体 } 定义之后调用，函数体照判", True,
                    any(command.words[:1] == ["scripts/sample-called.sh"] for command in shell_words.commands_at_command_position(
                        "f() { bash scripts/sample-called.sh start --label m -- sleep 1; }; f", work, uncalled_functions_skipped=True).commands)))
    for label, command in zip(('stdin:d137 误拒：builtin coproc 里 coproc 不是内建、bash 报错不跑，放行（退出码 0）', 'stdin:d137 误拒：builtin -- coproc 里 coproc 不是内建、bash 报错不跑，放行（退出码 0）'), ("builtin coproc sleep 1", "builtin -- coproc sleep 1")):   # builtin-coproc-detached（拒的那一侧是后面「一百三十七：coproc 照拒」那一格）
        results.append((label, 0, entry_as(None, command).returncode))
    def scope_entry(agent_type, command, cwd=None):
        payload = {"tool_name": "Bash", "session_id": "s", "tool_input": {"command": command, "run_in_background": False}}
        if agent_type:
            payload["agent_type"] = agent_type
        if cwd:
            payload["cwd"] = cwd
        return subprocess.run(["bash", os.path.join(scope_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
                              env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections), input=json.dumps(payload))
    scope_refused = scope_entry("clerk", "python3 scripts/kb/replace-once.py .claude/kb/checks-owed.md a b")
    results.append(("stdin:clerk 在 Bash 里用 replace-once.py 写知识库拒绝（退出码 2）", 2, scope_refused.returncode))
    results.append(("stdin:写范围的拒绝点名写法、目标、写范围表与出路", 1,
                    int(all(fragment in scope_refused.stderr for fragment in ("✗ clerk", "replace-once.py .claude/kb/checks-owed.md", "agent-write-scope.tsv",
                                                                               "→ 怎么办：范围外的改动不自己做", "records/**")))))
    results.append(("stdin:clerk 在 Bash 里写自己的草稿目录放行（退出码 0）", 0,
                    scope_entry("clerk", f"python3 scripts/kb/replace-once.py {new_root}/clerk-x/notes.md a b").returncode))
    results.append(("stdin:主 agent 同一条写知识库的命令放行（退出码 0）", 0, scope_entry(None, "python3 scripts/kb/replace-once.py .claude/kb/checks-owed.md a b").returncode))
    # ⑭ 走真实入口：工具员（定义在、写范围表里没有它的行，⑫ 不判）往两个服务 src/ 下已存在的文件追加拒，新建与主 agent 放行
    with open(os.path.join(scope_repository, ".claude", "agents", "tooling-writer.md"), "w", encoding="utf-8") as handle:
        handle.write("---\ntools: Read, Edit, Write, Bash\n---\n")
    os.makedirs(os.path.join(scope_repository, "prune-server", "src"), exist_ok=True)
    with open(os.path.join(scope_repository, "prune-server", "src", "protocol.rs"), "w", encoding="utf-8") as handle:
        handle.write("pub enum Request {}\n")
    service_append = "cat >> prune-server/src/protocol.rs <<'EOF2'\n// probe\nEOF2"
    service_refused = scope_entry("tooling-writer", service_append)
    results.append(("stdin:工具员 cat >> 追加两个服务 src/ 下已存在的文件拒绝（退出码 2）", 2, service_refused.returncode))
    results.append(("stdin:工具员改两个服务源码的拒绝点名写法、目标与仓副本、补丁的出路", 1,
                    int(all(fragment in service_refused.stderr for fragment in ("✗ tooling-writer", "prune-server/src/protocol.rs", "→ 怎么办：在仓副本里改", "补丁", "要主 agent 跑的命令")))))
    service_removal_refused = scope_entry("tooling-writer", "rm prune-server/src/protocol.rs")
    results.append(("stdin:工具员 rm 已存在的两个服务 src/ 下的文件拒绝（退出码 2）、拒绝点名 rm 与目标", (2, True),
                    (service_removal_refused.returncode, "rm prune-server/src/protocol.rs" in service_removal_refused.stderr)))
    results.append(("stdin:工具员在 Bash 里新建两个服务 src/ 下的文件放行（退出码 0）", 0,
                    scope_entry("tooling-writer", "cat > prune-server/src/new_module.rs <<'EOF'\npub fn x() {}\nEOF").returncode))
    results.append(("stdin:主 agent 同一条追加两个服务源码的命令放行（退出码 0）", 0, scope_entry(None, service_append).returncode))
    scope_linked = scope_entry("clerk", "echo x > records/kb-link/a.md")
    results.append(("stdin:范围内的链接指到仓里范围外，经它写拒绝（退出码 2）、拒绝说明写出解开符号链接之后的落点", (2, True),
                    (scope_linked.returncode, "records/kb-link/a.md（解开符号链接之后落在 .claude/kb/a.md）" in scope_linked.stderr)))
    before_scope_noted = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    scope_variable = scope_entry("clerk", f'mkdir -p {new_root}/clerk-x/originals && for f in records/a.md .claude/kb/b.md; do cp -p "$f" "{new_root}/clerk-x/originals/$(echo $f | tr / _)"; done')
    scope_lines = open(detections, encoding="utf-8").read().splitlines()[before_scope_noted:] if os.path.exists(detections) else []
    results.append(("stdin:目标里有命令替换的拒绝（退出码 2）、不记检出", (2, 0), (scope_variable.returncode, len(scope_lines))))
    results.append(("stdin:算不出落点的拒绝说明点名算不出、copy-originals.py 与写死的词的 for 循环两条出路", 1,
                    int(all(fragment in scope_variable.stderr for fragment in ("✗ clerk", "算不出最终落点", "$(echo records/a.md | tr / _)",
                                                                                "python3 scripts/agents/copy-originals.py copy <草稿目录> <仓内文件>…",
                                                                                "for 名字 in 写死的词…", "逐条写字面路径")))))
    before_scope_settled = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    scope_settled = scope_entry("clerk", f"for b in one two; do echo x > {new_root}/clerk-x/breaks/$b.out; done; cd records && for k in a b; do echo x > gz-$k/.gate-cells; done")
    results.append(("stdin:写死的词的循环变量拼的目标放行（退出码 0）、不记检出", (0, 0),
                    (scope_settled.returncode, (sum(1 for _ in open(detections)) if os.path.exists(detections) else 0) - before_scope_settled)))
    scope_cat_body = scope_entry("clerk", f"cat > {new_root}/clerk-x/x.sh <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF")
    results.append(("stdin:cat 写草稿脚本的 heredoc 正文里有 sed -i 知识库放行（退出码 0）", 0, scope_cat_body.returncode))
    scope_bash_body = scope_entry("clerk", "bash <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF")
    results.append(("stdin:喂给 bash 的 heredoc 里同一句 sed -i 知识库拒绝（退出码 2）", 2, scope_bash_body.returncode))
    scope_pipe_python = scope_entry("clerk", "cat <<'EOF' | python3 -\nopen('.claude/kb/a.md', 'w').write('x')\nEOF")
    results.append(("stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 拒绝（退出码 2，写范围拒）", (2, True),
                    (scope_pipe_python.returncode, "在 Bash 里写了它写范围之外的仓内文件" in scope_pipe_python.stderr)))
    results.append(("stdin:管道喂给 python 的 heredoc 里 open(知识库, 'w') 同样被 ⑧ 拒（有 Edit 的子 agent 就地改仓内文件）", 1,
                    int("就地改仓内文件" in scope_pipe_python.stderr)))
    results.append(("stdin:管道喂给 grep 的 heredoc 正文放行（退出码 0）", 0,
                    scope_entry("clerk", "cat <<'EOF' | grep x\nopen('.claude/kb/a.md', 'w')\nsed -i 's/a/b/' .claude/kb/a.md\nEOF").returncode))
    results.append(("stdin:cd 到草稿目录之后 sed -i 相对路径放行（退出码 0）", 0,
                    scope_entry("clerk", f"cd {new_root}/clerk-x && sed -i 's/a/b/' x.py").returncode))
    scope_sed_in_records = scope_entry("clerk", "cd records && sed -i 's/a/b/' x.md")
    results.append(("stdin:cd 到仓内 records 之后 sed -i 相对路径被 ⑧ 拒绝（退出码 2）", (2, True),
                    (scope_sed_in_records.returncode, "就地改仓内文件" in scope_sed_in_records.stderr)))
    results.append(("stdin:hook 输入的 cwd 在 /tmp 下时 sed -i 相对路径放行（退出码 0）", 0,
                    scope_entry("clerk", "sed -i 's/a/b/' x.py", "/tmp").returncode))
    results.append(("stdin:hook 输入没有 cwd 时同一句 sed -i 相对路径拒绝（退出码 2）", 2, scope_entry("clerk", "sed -i 's/a/b/' x.py").returncode))
    before_pwd_noted = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    scope_pwd = scope_entry("clerk", f"cd {new_root}/clerk-x && echo x > $PWD/a.md; W=$(mktemp -d) && cp -a records/a.md $W/")
    results.append(("stdin:cd 之后的 $PWD/… 与 mktemp -d 赋的变量放行（退出码 0）、不记检出", (0, 0),
                    (scope_pwd.returncode, (sum(1 for _ in open(detections)) if os.path.exists(detections) else 0) - before_pwd_noted)))
    results.append(("stdin:拒了的写范围不记检出", 0, sum(1 for line in (open(detections, encoding="utf-8").read().splitlines() if os.path.exists(detections) else [])
                                                      if "replace-once.py .claude/kb/checks-owed.md" in line)))
    unknown_break_items = [item for item in os.environ.get(DETECTOR_BREAK_VARIABLE, "").split(",") if item and item not in KNOWN_DETECTOR_BREAK_ITEMS]
    results.append((f"{DETECTOR_BREAK_VARIABLE} 里的项都认得（认得的：{'、'.join(KNOWN_DETECTOR_BREAK_ITEMS)}）", [], unknown_break_items))
    # 弄坏开关的名字四处对账（名单、文件头开关表、代码里读的名字、各节「弄坏开关：」一串，break_switch_name_sets 从源文件现读，读法按开关表的文法）：这份钩子的源文件对得上；
    #   样本源文件逐个造一种写法或一处对不上：写法对的必须一条都不报，对不上的必须点名那个名字与缺在哪几处、读不成一条的点名那一行。
    #   样本里读开关的写法用字符串拼，免得这份源码里出现字面的读法被上面那一格当成真读。
    #   弄坏开关 break-switch-sets-one-way 下反方向的（文件头、代码、各节里有而名单没有）查不出，样本「文件头有、名单没有」「代码读了、名单没有」「文件头与代码有、名单没有」「各节写着已删的开关」四格红；
    #   break-switch-indirect-reads-unseen 下常量与 bash 层读的名字、单引号写法不认，这份源文件与「单引号」「常量」「bash 层 ==」「读的名字认不出」「常量、bash 层读的名字」几格红；
    #   break-switch-table-narrow 下开关表照一行一个名字读，「逗号隔开的几项」「同一行写第二条」「名字套反引号」「等号后换行」「缩进」「说明行」几格红
    read_call = "detector_break_is_set"
    marker_text = "# 批量登记的开关："
    entry_prefix = "BASH_COMMAND_DETECTOR_BREAK="
    detector_source = ""   # 入口加上 python 各部分文件拼成一份源文本：文件头开关表在入口里，名单与读开关的代码在部分文件里
    for source_name in ("bash-command-detector.sh",) + DETECTOR_PART_NAMES:
        with open(os.path.join(hook_dir, source_name), encoding="utf-8") as source_handle:
            detector_source += source_handle.read()

    def break_switch_sample(known, header_rows, code_rows):
        sample_lines = ["#!/usr/bin/env bash", "# 样本钩子"] + list(header_rows)
        sample_lines += ["set -uo pipefail", "KNOWN_DETECTOR_BREAK_ITEMS = (" + ", ".join(f"\"{name}\"" for name in known) + ")", ""]
        return "\n".join(sample_lines + list(code_rows)) + "\n"

    def table_rows(*names):
        return [f"#   {entry_prefix}{name}  说明" for name in names]

    def read_rows(*names):
        return [f"    if {read_call}(\"{name}\"):" for name in names]

    def layer_row(name, operator):
        return "if [[ \",${X:-},\" " + operator + " *," + name + ",* ]]; then"

    both = ("a-one", "b-two")
    results.append(("弄坏开关三处一致:这份钩子的名单、文件头开关表、代码里读的名字、各节弄坏开关一串对得上", [], break_switch_set_mismatches(detector_source)))
    for label, sample_source, keywords in (
            ("三处一致的样本没有对不上的名字", break_switch_sample(both, table_rows(*both), read_rows(*both)), None),
            ("名单有、文件头没有", break_switch_sample(both, table_rows("a-one"), read_rows(*both)), ["b-two：只在名单、读开关的代码里有，文件头开关表里没有"]),
            ("名单有、代码没读", break_switch_sample(both, table_rows(*both), read_rows("a-one")), ["b-two：只在名单、文件头开关表里有，读开关的代码里没有"]),
            ("名单有、另两处都没有", break_switch_sample(both, table_rows("a-one"), read_rows("a-one")), ["b-two：只在名单里有，文件头开关表、读开关的代码里没有"]),
            ("文件头有、名单没有", break_switch_sample(("a-one",), table_rows(*both), read_rows("a-one")), ["b-two：只在文件头开关表里有，名单、读开关的代码里没有"]),
            ("代码读了、名单没有", break_switch_sample(("a-one",), table_rows("a-one"), read_rows(*both)), ["b-two：只在读开关的代码里有，名单、文件头开关表里没有"]),
            ("文件头与代码有、名单没有", break_switch_sample(("a-one",), table_rows(*both), read_rows(*both)), ["b-two：只在文件头开关表、读开关的代码里有，名单里没有"]),
            ("逗号隔开的几项各算一条", break_switch_sample(both, [f"#   {entry_prefix}a-one,b-two  说明"], read_rows(*both)), None),
            ("同一行写第二条的点名那一行", break_switch_sample(both, [f"#   {entry_prefix}a-one 或 {entry_prefix}b-two  说明"], read_rows(*both)), ["同一行写了第二条开关"]),
            ("名字套反引号的点名那一行", break_switch_sample(both, table_rows("a-one") + [f"#   {entry_prefix}`b-two`  说明"], read_rows(*both)), ["文法读不到"]),
            ("等号后换行的点名那一行", break_switch_sample(both, table_rows("a-one") + [f"#   {entry_prefix}", "#     b-two  说明"], read_rows(*both)), ["文法读不到"]),
            ("缩进两个空格与不缩进的行都认", break_switch_sample(both, [f"#  {entry_prefix}a-one  说明", f"#{entry_prefix}b-two  说明"], read_rows(*both)), None),
            ("说明行写着带连字符的词不当开关", break_switch_sample(both, table_rows(*both) + ["#   git-commit-a  这一行是说明文字，不是开关"], read_rows(*both)), None),
            ("批量登记那一段的行首写法认", break_switch_sample(both, table_rows("a-one") + [marker_text, "#   b-two  说明"], read_rows(*both)), None),
            ("单引号写的读法", break_switch_sample(("a-one",), table_rows("a-one"), read_rows("a-one") + [f"    if {read_call}('b-two'):"]), ["b-two：只在读开关的代码里有"]),
            ("经常量名读", break_switch_sample(("a-one",), table_rows("a-one"), read_rows("a-one") + ["PROBE_ITEM = \"b-two\"", f"    if {read_call}(PROBE_ITEM):"]), ["b-two：只在读开关的代码里有"]),
            ("bash 层 == 读", break_switch_sample(("a-one",), table_rows("a-one"), read_rows("a-one") + [layer_row("b-two", "==")]), ["b-two：只在读开关的代码里有"]),
            ("读的名字认不出", break_switch_sample(both, table_rows(*both), read_rows(*both) + [f"    if {read_call}(probe_name):"]), ["读的名字认不出"]),
            ("常量、bash 层读的名字也算读", break_switch_sample(("a-one", "d-four", "e-five", "f-six"), table_rows("a-one", "d-four", "e-five", "f-six"),
                                                         read_rows("a-one") + ["GUARD_OFF_ITEM = \"d-four\"", layer_row("f-six", "!="), "E_FIVE = \"e-five\"", f"    if {read_call}(E_FIVE):"]), None),
            ("各节写着已经删掉的开关", break_switch_sample(both, table_rows(*both) + ["#   弄坏开关：a-one（x）、c-three（y）。"], read_rows(*both)), ["c-three：只在各节"]),
            ("各节写的开关都在", break_switch_sample(both, table_rows(*both) + ["#   弄坏开关：a-one（x、y）、", "#   b-two（z）。"], read_rows(*both)), None),
            ("各节写了缩写", break_switch_sample(both, table_rows(*both) + ["#   弄坏开关：a-one、-two。"], read_rows(*both)), ["缩写"])):
        found = break_switch_set_mismatches(sample_source)
        if keywords is None:
            results.append((f"弄坏开关三处一致:{label}", [], found))
        else:
            results.append((f"弄坏开关三处一致:{label}", [True] * len(keywords), [any(keyword in line for line in found) for keyword in keywords]))
    renamed, renamed_recorded = through_entry(f"cp /tmp/new.sh scripts/.running.sh.installing && mv scripts/.running.sh.installing {running}")
    results.append(("stdin:写到临时文件再 mv 换上放行（退出码 0）、不记检出", (0, 0), (renamed.returncode, renamed_recorded)))
    # 同走 run_in_background 的那一条：只记检出
    results.append(("等待循环:同一条走 run_in_background 记一条检出", 1,
                    record({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "until grep -q x f; do sleep 10; done", "run_in_background": True}},
                           detections)))
    # 走真实入口：从标准输入喂 JSON，起看门狗的写法不对退出码 2、stderr 给出路，写法对的退出码 0
    script_for_watchdog = os.path.join(hook_dir, "bash-command-detector.sh")
    watchdog_environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections)
    foreground_watchdog = subprocess.run(["bash", script_for_watchdog], capture_output=True, text=True, env=watchdog_environment,
                                         input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "bash scripts/agents/watch.sh a1 > /dev/null 2>&1 &"}}))
    results.append(("stdin:前台起看门狗拒绝（退出码 2）", 2, foreground_watchdog.returncode))
    results.append(("stdin:拒绝时 stderr 写了出路（run_in_background: true 的起法）", 1,
                    int("run_in_background: true" in foreground_watchdog.stderr and "→" in foreground_watchdog.stderr)))
    background_watchdog = subprocess.run(["bash", script_for_watchdog], capture_output=True, text=True, env=watchdog_environment,
                                         input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "bash scripts/agents/watch.sh a1", "run_in_background": True}}))
    results.append(("stdin:run_in_background 起看门狗放行（退出码 0）", 0, background_watchdog.returncode))
    # 走真实入口：从标准输入喂 JSON，run_in_background 里没超时的等待循环退出码必须是 0（不拦），检出要落进文件
    detector_script = os.path.join(hook_dir, "bash-command-detector.sh")
    before = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    waiting = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                 "command": "until grep -q done log; do sleep 10; done", "run_in_background": True}}),
                             capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    results.append(("stdin:run_in_background 里没超时的等待循环只记不拦（退出码 0）", 0, waiting.returncode))
    results.append(("stdin:检出落进检出记录", 1, after - before))
    # 走真实入口：前台没超时的等待循环拒绝（退出码 2）、不记检出，stderr 给出两种写法；套上 timeout 的放行（退出码 0）
    foreground_wait = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                         "command": "until grep -q x f; do sleep 10; done"}}),
                                     capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:前台没超时的等待循环拒绝（退出码 2）", 2, foreground_wait.returncode))
    results.append(("stdin:拒绝时 stderr 写了两种写法（run_in_background 与 proc.py wait --timeout）", 1,
                    int("✗" in foreground_wait.stderr and "→" in foreground_wait.stderr and "run_in_background: true" in foreground_wait.stderr
                        and "proc.py wait <pid> --timeout <秒>" in foreground_wait.stderr)))
    bounded_wait = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                      "command": "timeout 60 bash -c 'until grep -q x f; do sleep 10; done'"}}),
                                  capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:套上 timeout 的前台等待循环放行（退出码 0）", 0, bounded_wait.returncode))
    results.append(("stdin:前台拒了的与套上 timeout 的都不记检出", after, sum(1 for _ in open(detections)) if os.path.exists(detections) else 0))
    # 走真实入口：判不了的照常放行、stderr 报一句、记一条检出——hook 输入不是 JSON 对象（坏 JSON、解出来是数组；会话号写读不到，诊断带钩子名与开头的字节）、
    # 各道判的时候抛异常（tool_input.command 不是字符串）、here-string 喂给 bash 的是变量；拒了的命令里带着判不了的输入，照记一条（结局「被拒」）。
    # BASH_COMMAND_DETECTOR_BREAK=failure-unrecorded 下这五格红；LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下后两格与「here-string 里前台起看门狗」一格红
    def detection_lines():
        return sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
    def last_detection_entry():
        if not os.path.exists(detections):
            return {}
        with open(detections, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
        return json.loads(lines[-1]) if lines else {}
    for label, raw in (("stdin:坏 JSON 放行（退出码 0）、stderr 写诊断、记一条检出", b'{"tool_name": "Bash", "tool_input": '),
                       ("stdin:不是 JSON 对象放行（退出码 0）、stderr 写诊断、记一条检出", b'["Bash"]')):
        before = detection_lines()
        unreadable = subprocess.run(["bash", detector_script], input=raw, capture_output=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
        stderr_text = unreadable.stderr.decode("utf-8", "replace")
        results.append((label, (0, True, 1, "读不到"),
                        (unreadable.returncode, "bash-command-detector.sh 收到的 hook 输入不是 JSON 对象" in stderr_text and repr(raw[:10])[2:-1] in stderr_text,
                         detection_lines() - before, last_detection_entry().get("session_id"))))
    before = detection_lines()
    raising = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s-raise", "tool_input": {"command": ["sleep", "1"]}}),
                             capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:判的时候抛异常放行（退出码 0）、stderr 写诊断、记一条检出", (0, True, 1, "s-raise"),
                    (raising.returncode, "bash-command-detector.sh 没判成" in raising.stderr, detection_lines() - before, last_detection_entry().get("session_id"))))
    before = detection_lines()
    fed_variable = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s-fed", "tool_input": {"command": 'bash <<< "$CMD"'}}),
                                  capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:here-string 喂给 bash 的是变量放行（退出码 0）、记一条检出", (0, 1, True), (fed_variable.returncode, detection_lines() - before,
                    any("here-string `$CMD`" in finding for finding in last_detection_entry().get("findings", [])))))
    before = detection_lines()
    refused_fed = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s-fed-refused", "tool_input": {
                                     "command": 'until grep -q x f; do sleep 10; done; grep -h step steps.sh | bash'}}),
                                 capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:拒了的命令里有判不了的输入（grep 的输出经管道喂给 bash），照记一条、结局被拒", (2, 1, "被拒"),
                    (refused_fed.returncode, detection_lines() - before, last_detection_entry().get("outcome"))))
    fed_watchdog = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                      "command": "bash <<< 'bash scripts/agents/watch.sh a1 > /dev/null 2>&1 &'"}}),
                                  capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:here-string 里前台起看门狗照拒（退出码 2）", 2, fed_watchdog.returncode))

    # 一百三十七节那一批（records/2026-10-29-一百三十七节那一轮调度记录.md 件 1）：每格后面的注释写弄坏它的开关
    def batch137_entry(command, agent_type=None, run_in_background=False, environment=None):
        payload = {"tool_name": "Bash", "session_id": "s-137", "tool_input": {"command": command, "run_in_background": run_in_background}}
        if agent_type:
            payload["agent_type"] = agent_type
        return subprocess.run(["bash", detector_script], input=json.dumps(payload), capture_output=True, text=True,
                              env=environment or dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    for label, command, agent_type, background, want in (
            ("一百三十七：子 agent here-string 喂给 bash 的 claude 照拒（退出码 2）", "bash <<< 'claude -p hi'", "clerk", False, 2),          # claude-launch-layers-narrow
            ("一百三十七：子 agent env -S 拆串里的 claude 照拒（退出码 2）", "env -S 'claude -p hi'", "clerk", False, 2),                      # 同上
            ("一百三十七：子 agent xargs 起的 claude 照拒（退出码 2）", "echo hi | xargs claude -p", "clerk", False, 2),                        # 同上
            ("一百三十七：子 agent ssh 起的 claude 照拒（退出码 2）", "ssh localhost 'claude -p hi'", "clerk", False, 2),                       # 同上
            ("一百三十七：看门狗输出经 tee 丢进 /dev/null 照拒（退出码 2）", "bash scripts/agents/watch.sh a1 | tee /dev/null", None, True, 2),  # watchdog-tee-null-unseen
            ("一百三十七：两个 & 只等其中一个照拒（退出码 2）", "sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1", None, True, 2),                  # partial-wait-allowed
            ("一百三十七：两个 & 各等一个放行（退出码 0）", "sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1; wait $p2", None, True, 0),
            ("一百三十七：一个 & 之后 wait -n 放行（退出码 0）", "sleep 1 & wait -n", None, True, 0),                                         # wait-forms-unseen
            ("一百三十七：一个 & 之后 tail --pid 等它放行（退出码 0）", "sleep 2 & tail --pid=$! -f /dev/null", None, True, 0),                 # 同上
            ("一百三十七：一个 & 之后 kill -0 轮询等它放行（退出码 0）", "sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done", None, True, 0),  # 同上
            ("一百三十七：wait 写在函数里、函数被调用放行（退出码 0）", "w() { wait; }; sleep 1 & w", None, True, 0),                        # 同上
            ("一百三十七：coproc 照拒（退出码 2）", "coproc sleep 1", None, False, 2)):                                                       # coproc-prefix-unseen（连同 LIB_SHELL_WORDS_BREAK=coproc-as-command 时不红）
        results.append((label, want, batch137_entry(command, agent_type, background).returncode))
    # 暂存区非空时 git stash、git rm -r --cached .、git checkout HEAD -- . 与子命令写成变量的 git（⑮；whole-index-other-forms-unseen 下前三格红，
    # git-subcommand-unexpanded-allowed 下第四格红）：在一个暂存了改动的临时仓里判
    index_repository = os.path.join(work, "index-repository-137")
    os.makedirs(index_repository)
    subprocess.run(["git", "-C", index_repository, "init", "-q"], check=True)
    with open(os.path.join(index_repository, "a.txt"), "w", encoding="utf-8") as handle:
        handle.write("a\n")
    subprocess.run(["git", "-C", index_repository, "add", "a.txt"], check=True)
    for label, command in (("一百三十七：暂存区非空时 git stash 照拒（退出码 2）", "git stash"),
                           ("一百三十七：暂存区非空时 git rm -r --cached . 照拒（退出码 2）", "git rm -r --cached -q ."),
                           ("一百三十七：暂存区非空时 git checkout HEAD -- . 照拒（退出码 2）", "git checkout HEAD -- ."),
                           ("一百三十七：暂存区非空时子命令写成没赋过值的变量照拒（退出码 2）", "git $SUBCOMMAND_FROM_ELSEWHERE")):
        results.append((label, 2, batch137_entry(f"cd {index_repository} && {command}").returncode))
    results.append(("一百三十七：暂存区非空时 git stash push -- 具体路径放行（退出码 0）", 0, batch137_entry(f"cd {index_repository} && git stash list").returncode))
    # 拒绝说明：重定向目标上的花括号点名 ambiguous redirect（brace-target-reason-generic 下红）；等待循环点名真会结束的几种也要套 timeout（wait-loop-rewrite-unexplained 下红）
    brace_target = batch137_entry(f"echo hi > {work}/o.{{a,b}}", "implementation-writer")
    results.append(("一百三十七：重定向目标花括号照拒，拒绝说明点名 ambiguous redirect 与改法", (2, True),
                    (brace_target.returncode, "ambiguous redirect" in brace_target.stderr and "逐份写" in brace_target.stderr)))
    counted_loop = batch137_entry("i=0; while [ $i -lt 3 ]; do sleep 0.1; i=$((i+1)); done")
    results.append(("一百三十七：带计数上限的前台等待循环照拒，拒绝说明点名真会结束的几种也要套 timeout", (2, True),
                    (counted_loop.returncode, "真会结束的循环" in counted_loop.stderr and "tail --pid" in counted_loop.stderr)))
    # 钩子 bash 这一层起不来时也记一条检出（bash-layer-unrecorded 下红）：PATH 前面放一份只对 /dev/fd/3 退 127 的假 python3
    shim = os.path.join(work, "bash-layer-shim")
    os.makedirs(shim, exist_ok=True)
    with open(os.path.join(shim, "python3"), "w", encoding="utf-8") as handle:
        handle.write(f'#!/bin/bash\nif [ "$1" = /dev/fd/3 ]; then exit 127; fi\nexec {sys.executable} "$@"\n')
    os.chmod(os.path.join(shim, "python3"), 0o755)
    before = detection_lines()
    bash_layer = batch137_entry("ls", "clerk", environment=dict(os.environ, AGENT_HOOK_DETECTIONS=detections, PATH=shim + os.pathsep + os.environ.get("PATH", "")))
    results.append(("一百三十七：钩子 bash 这一层起不来（假 python3 退 127）：放行（退出码 0）、记一条检出", (0, 1, "s-137", "只记"),
                    (bash_layer.returncode, detection_lines() - before, last_detection_entry().get("session_id"), last_detection_entry().get("outcome"))))
    # PATH 上没有 python3（python-missing-unrecorded 下红）：PATH 只放记检出要用的 cat、date、mkdir、dirname
    no_python = os.path.join(work, "no-python-path")
    os.makedirs(no_python, exist_ok=True)
    for tool in ("cat", "date", "mkdir", "dirname"):
        if shutil.which(tool) and not os.path.lexists(os.path.join(no_python, tool)):
            os.symlink(shutil.which(tool), os.path.join(no_python, tool))
    before = detection_lines()
    no_python_payload = {"tool_name": "Bash", "session_id": "s-137", "agent_type": "clerk", "tool_input": {"command": "claude -p hi", "run_in_background": False}}
    no_python_run = subprocess.run([shutil.which("bash"), detector_script], input=json.dumps(no_python_payload), capture_output=True, text=True,
                                   env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections, PATH=no_python))
    results.append(("d137 H13：PATH 上没有 python3：放行（退出码 0）、记一条检出", (0, 1, "s-137", "只记"),
                    (no_python_run.returncode, detection_lines() - before, last_detection_entry().get("session_id"), last_detection_entry().get("outcome"))))
    # 引导读不到、编不过任何一份部分文件：拷一份钩子目录（入口加部分文件），删掉或写坏其中一份，照「判不了的（放行但必记检出）」处置：放行（退出码 0）、记一条点名部分文件的检出、
    #   stderr 点名那一份、没有 Traceback（part-unreadable-passes 下引导照原样抛错，Traceback 退 1，bash 那一层兜成放行与泛泛的检出，下面几格红）；
    #   截成空文件的也一样（编得过、什么都不定义）：第一份（scope_and_index）截空、带 main 的那一份（selftest_entries_and_main）截空各一格，不静默退 0 而不记检出
    bootstrap_hooks = os.path.join(work, "bootstrap-copy", ".claude", "hooks")
    os.makedirs(bootstrap_hooks, exist_ok=True)
    bootstrap_payload = {"tool_name": "Bash", "session_id": "s-bootstrap", "agent_type": "clerk", "tool_input": {"command": "ls", "run_in_background": False}}
    for part_name in DETECTOR_PART_NAMES:
        for damage, damage_label in (("missing", "读不到"), ("syntax-error", "编不过"), ("emptied", "截空")):
            for copied_name in ("bash-command-detector.sh",) + DETECTOR_PART_NAMES:
                shutil.copy(os.path.join(hook_dir, copied_name), bootstrap_hooks)
            if damage == "missing":
                os.remove(os.path.join(bootstrap_hooks, part_name))
            else:
                with open(os.path.join(bootstrap_hooks, part_name), "w", encoding="utf-8") as damaged_part:
                    damaged_part.write("" if damage == "emptied" else "def broken(:\n")
            before = detection_lines()
            bootstrap_run = subprocess.run(["bash", os.path.join(bootstrap_hooks, "bash-command-detector.sh")], input=json.dumps(bootstrap_payload), capture_output=True, text=True,
                                           env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
            bootstrap_findings = last_detection_entry().get("findings") or [""]
            results.append((f"引导:{damage_label}部分文件 {part_name} 照常放行（退出码 0）、记一条点名部分文件的检出", (0, 1, True, True, True),
                            (bootstrap_run.returncode, detection_lines() - before,
                             f"部分文件 {os.path.join(bootstrap_hooks, part_name)}" in bootstrap_run.stderr and "Traceback" not in bootstrap_run.stderr,
                             "部分文件读不到或编不过" in bootstrap_findings[0], last_detection_entry().get("session_id") == "s-bootstrap")))
    # 共用切词模块的部分模块按这一份所在的目录载入、按目录各存一份：同一个进程从两个目录各载入一次，后载入的用它自己目录里的部分模块，载入前后 sys.path 与 sys.modules 里的部分模块不变，
    #   同一个目录载入两次用的是同一份部分模块（本进程里别的库按文件路径载入它，类与哨兵对象要对得上）
    #   （LIB_SHELL_WORDS_BREAK=part-modules-cached 下前两格红：部分模块沿用 sys.modules 里先载入的那一份、sys.path 留着目录；=part-modules-per-load 下第三格红：每载入一次就各载入一份）
    plain_directory = os.path.join(work, "shell-words-plain")
    marked_directory = os.path.join(work, "shell-words-marked")
    for directory in (plain_directory, marked_directory):
        os.makedirs(directory, exist_ok=True)
        for library_name in ("lib_shell_words.py",) + SHELL_WORDS_PART_NAMES:
            shutil.copy(os.path.join(hook_dir, library_name), directory)
    marked_core_path = os.path.join(marked_directory, "lib_shell_words_core.py")
    with open(marked_core_path, encoding="utf-8") as core_handle:
        core_text = core_handle.read()
    marked_core_text = core_text.replace("def shell_tokens(text):\n", "def shell_tokens(text):\n    return [\"marked-directory\"]\n", 1)   # 这个目录里的 shell_tokens 换成交一个记号
    with open(marked_core_path, "w", encoding="utf-8") as core_handle:
        core_handle.write(marked_core_text)
    part_module_names = ("lib_shell_words_core", "lib_shell_words_embedded")
    path_before_loads, parts_before_loads = list(sys.path), [name in sys.modules for name in part_module_names]
    plain_library = load_shell_words(os.path.join(plain_directory, "lib_shell_words.py"))
    marked_library = load_shell_words(os.path.join(marked_directory, "lib_shell_words.py"))
    results.append(("共用切词模块:同一个进程从两个目录各载入一次，后载入的用它自己目录里的部分模块", (True, ["marked-directory"], 2),
                    (marked_core_text != core_text, marked_library.shell_tokens("a b"), len(plain_library.shell_tokens("a b")))))
    results.append(("共用切词模块:载入之后 sys.path 与 sys.modules 里的部分模块不变", (True, True),
                    (sys.path == path_before_loads, [name in sys.modules for name in part_module_names] == parts_before_loads)))
    plain_library_again = load_shell_words(os.path.join(plain_directory, "lib_shell_words.py"))
    results.append(("共用切词模块:同一个目录载入两次，用的是同一份部分模块", (True, True),
                    (plain_library_again.ShellWord is plain_library.ShellWord, plain_library_again.embedded_launches.__globals__ is plain_library.embedded_launches.__globals__)))
    # 检出记录的位置照 lib_work_root.py 取、不退回家目录：AGENT_HOOK_DETECTIONS、SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT 都不设，PATH 上没有 python3，HOME 换成临时目录
    unrooted_home = os.path.join(work, "unrooted-home")
    os.makedirs(unrooted_home, exist_ok=True)
    unrooted_environment = {name: value for name, value in os.environ.items() if name not in ("AGENT_HOOK_DETECTIONS", "SINGLEFS_WORK_ROOT", "SINGLEFS_DATA_ROOT")}
    unrooted_run = subprocess.run([shutil.which("bash"), detector_script], input=json.dumps(no_python_payload), capture_output=True, text=True,
                                  env=dict(unrooted_environment, HOME=unrooted_home, PATH=no_python))
    results.append(("d137 H13：PATH 上没有 python3、工作根取不到：放行（退出码 0）、不往家目录下写、stderr 说检出没记成", (0, False, True),
                    (unrooted_run.returncode, os.path.exists(os.path.join(unrooted_home, ".local")), "检出没记成" in unrooted_run.stderr)))
    # 走真实入口：前台 timeout 超过 240000 毫秒拒（退出码 2，出路按调用方写）；不超过的、run_in_background 的放行（弄坏开关 allow-long-foreground-timeout 下拒的几格红）
    def timeout_entry(agent_type, timeout, background=False):
        timeout_payload = {"tool_name": "Bash", "session_id": "s-timeout", "tool_input": {"command": "ls", "run_in_background": background, "timeout": timeout}}
        if agent_type:
            timeout_payload["agent_type"] = agent_type
        completed = subprocess.run(["bash", detector_script], input=json.dumps(timeout_payload), capture_output=True, text=True,
                                   env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
        return completed
    timeout_main = timeout_entry(None, 600000)
    results.append(("stdin:主 agent 前台 timeout 600000 毫秒拒绝（退出码 2）、出路写 run_in_background 与 task-server/task wait", (2, True, True),
                    (timeout_main.returncode, "run_in_background: true" in timeout_main.stderr, "task-server/task wait" in timeout_main.stderr)))
    timeout_sub = timeout_entry("implementation-writer", 600000)
    results.append(("stdin:子 agent 前台 timeout 600000 毫秒拒绝（退出码 2）、出路写进报告交回主 agent", (2, True, False),
                    (timeout_sub.returncode, "要主 agent 跑的命令" in timeout_sub.stderr, "task-server/task wait" in timeout_sub.stderr)))
    results.append(("stdin:主 agent 前台 timeout 240001 毫秒拒绝（退出码 2）", 2, timeout_entry(None, 240001).returncode))
    results.append(("stdin:主 agent 前台 timeout 240000 毫秒放行（退出码 0）", 0, timeout_entry(None, 240000).returncode))
    results.append(("stdin:主 agent run_in_background 带 timeout 600000 毫秒放行（退出码 0）", 0, timeout_entry(None, 600000, True).returncode))
    results.append(("stdin:主 agent timeout 写成不是数的字符串放行（退出码 0）", 0, timeout_entry(None, "soon").returncode))
    # 走真实入口：run_in_background 里 & disown 拒绝（退出码 2），stderr 给出 run_in_background 的起法
    detached = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                  "command": "nice -n 19 bash long.sh > long.log 2>&1 & disown 2>/dev/null || true & wait", "run_in_background": True}}),
                              capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:run_in_background 里 & disown 拒绝（退出码 2）", 2, detached.returncode))
    results.append(("stdin:放出追踪的拒绝 stderr 写了 run_in_background 的起法", 1,
                    int("✗" in detached.stderr and "→" in detached.stderr and "run_in_background: true" in detached.stderr)))
    # 走真实入口：进程替换 `<(…)` 里的命令另按一条完整的命令判，`)` 之后的词是外层命令的参数；现场误拒的原句放行、不记检出，括号里是前台没超时的等待循环的拒绝（退出码 2）
    before_substitution = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    substitution_original = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                               "command": "diff <(git show HEAD:.claude/scripts/check.sh) .claude/scripts/check.sh"}}),
                                           capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:进程替换的现场原句 diff <(git show …) check.sh 放行（退出码 0）", 0, substitution_original.returncode))
    results.append(("stdin:进程替换的现场原句不记检出", before_substitution, sum(1 for _ in open(detections)) if os.path.exists(detections) else 0))
    substitution_loop = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                           "command": "diff <(until grep -q x f; do sleep 10; done; cat f) x"}}),
                                       capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:进程替换里前台没超时的等待循环拒绝（退出码 2）", 2, substitution_loop.returncode))
    # 走真实入口：run_in_background 里后面没有 wait 的单独 & 拒绝（退出码 2），stderr 写两条出路；前台的同一条放行（退出码 0）；两条都不记检出
    before_ampersand = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    unwaited = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                  "command": "bash long.sh > long.log 2>&1 & echo $!", "run_in_background": True}}),
                              capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:run_in_background 里后面没有 wait 的单独 & 拒绝（退出码 2）", 2, unwaited.returncode))
    results.append(("stdin:拒绝时 stderr 写了两条出路（run_in_background 里不加 & 与 proc.py wait <pid> --timeout <秒>）", 1,
                    int("✗" in unwaited.stderr and "→" in unwaited.stderr and "run_in_background" in unwaited.stderr
                        and "不加 `&`" in unwaited.stderr and "proc.py wait <pid> --timeout <秒>" in unwaited.stderr)))
    foreground_ampersand = subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                              "command": "bash long.sh > long.log 2>&1 & echo $!"}}),
                                          capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    results.append(("stdin:前台的同一条放行（退出码 0）", 0, foreground_ampersand.returncode))
    results.append(("stdin:拒了的与前台的单独 & 都不记检出", before_ampersand, sum(1 for _ in open(detections)) if os.path.exists(detections) else 0))
    # 走真实入口：终止进程的写法拒绝（退出码 2）、stderr 给出按 pid 停一个的出路、不记检出；点名一个的放行。命令只交给 hook 判，一条都不执行
    before_signal = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    def signal_entry(command, run_in_background=False):
        return subprocess.run(["bash", detector_script], input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                  "command": command, "run_in_background": run_in_background}}),
                              capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections))
    group_signal = signal_entry("kill -TERM -1234")
    results.append(("stdin:kill 负进程号拒绝（退出码 2）", 2, group_signal.returncode))
    results.append(("stdin:拒绝时 stderr 写了按 pid 停一个的出路", 1,
                    int("✗" in group_signal.stderr and "→" in group_signal.stderr and "proc.py stop <pid>" in group_signal.stderr and "`$!`" in group_signal.stderr)))
    results.append(("stdin:kill 写死的 1 拒绝（退出码 2，读真的进程表）", 2, signal_entry("kill -TERM 1").returncode))
    results.append(("stdin:kill 写死的自检进程号（hook 的祖先）拒绝（退出码 2）", 2, signal_entry(f"kill -TERM {os.getpid()}").returncode))
    results.append(("stdin:run_in_background 里循环 kill 拒绝（退出码 2）", 2, signal_entry('for p in $(cat pids); do kill "$p"; done', True).returncode))
    results.append(("stdin:kill \"$!\" 放行（退出码 0）", 0, signal_entry('kill "$!"').returncode))
    results.append(("stdin:proc.py stop 一个不存在的进程号放行（退出码 0）", 0, signal_entry("python3 .claude/singlefs-ai-sop/scripts/proc.py stop 99999999").returncode))
    results.append(("stdin:终止进程拒了的与放行的都不记检出", before_signal, sum(1 for _ in open(detections)) if os.path.exists(detections) else 0))
    # 走真实入口：一条命令同时踩终止进程、放出追踪、前台等待循环三类，一次列全（BASH_COMMAND_DETECTOR_BREAK=stop-at-first-refusal 下第一格红）；
    # 只记的检出记录写结局「只记」（detection-outcome-dropped 下第二格红）
    three_kinds = signal_entry("kill -TERM -1234; nohup sleep 100 & disown; until false; do sleep 1; done")
    results.append(("stdin:一条命令同时踩几类时一次列全（终止进程、放出追踪、前台等待循环）", (2, True),
                    (three_kinds.returncode, all(fragment in three_kinds.stderr for fragment in
                                                 ("踩了 3 类", "✗ 终止进程的写法", "✗ 把活放出了追踪", "✗ 前台的等待循环没有超时")))))
    signal_entry("until false; do sleep 1; done", True)
    recorded_lines = open(detections, encoding="utf-8").read().splitlines() if os.path.exists(detections) else []
    last_outcome = json.loads(recorded_lines[-1]).get("outcome") if recorded_lines else None   # 没记下检出时这一格照样红，不抛异常
    # ⑮ 暂存区非空时不带具体路径清整个索引：临时仓两份（一份暂存了两个文件、一份暂存区空），进程里判与走入口各几格；(说明, 起点仓, 命令, 该不该拒)
    staged_repository, clean_repository = os.path.join(work, "index-staged"), os.path.join(work, "index-clean")
    for index_repository in (staged_repository, clean_repository):
        subprocess.run(["git", "init", "-q", index_repository], check=True)
        subprocess.run(["git", "-C", index_repository, "-c", "user.email=t@t", "-c", "user.name=t", "-c", "commit.gpgsign=false",
                        "commit", "-q", "--allow-empty", "-m", "base"], check=True)
        for name in ("a.txt", "b.txt"):
            with open(os.path.join(index_repository, name), "w", encoding="utf-8") as handle:
                handle.write(name + "\n")
    subprocess.run(["git", "-C", staged_repository, "add", "a.txt", "b.txt"], check=True)
    index_cases = [
        ("清整个索引:git reset 拒", staged_repository, "git reset", 1),
        ("清整个索引:git reset HEAD 拒", staged_repository, "git reset HEAD", 1),
        ("清整个索引:git reset --mixed 拒", staged_repository, "git reset --mixed", 1),
        ("清整个索引:git reset -q --hard 拒", staged_repository, "git reset -q --hard", 1),
        ("清整个索引:git restore --staged . 拒", staged_repository, "git restore --staged .", 1),
        ("清整个索引:git restore --staged :/ 拒", staged_repository, "git restore --staged :/", 1),
        ("清整个索引:git -C 点名暂存了的仓、起点在别处，拒", clean_repository, f"git -C {staged_repository} reset", 1),
        ("清整个索引:cd 进暂存了的仓之后 git reset HEAD 拒", clean_repository, f"cd {staged_repository} && git reset HEAD", 1),
        ("清整个索引:cd 到没赋过值的变量之后 git reset，暂存区判不了，拒", clean_repository, 'cd "$WHERE" && git reset', 1),
        ("清整个索引:git reset HEAD -- a.txt 带路径放行", staged_repository, "git reset HEAD -- a.txt", 0),
        ("清整个索引:git reset a.txt 带路径放行", staged_repository, "git reset a.txt", 0),
        ("清整个索引:git restore --staged -- a.txt b.txt 带路径放行", staged_repository, "git restore --staged -- a.txt b.txt", 0),
        ("清整个索引:git reset --soft HEAD 不碰索引，放行", staged_repository, "git reset --soft HEAD", 0),
        ("清整个索引:暂存区为空 git reset 放行", clean_repository, "git reset", 0),
        ("清整个索引:暂存区为空 git restore --staged . 放行", clean_repository, "git restore --staged .", 0),
        ("清整个索引:grep 参数里的 git reset 不判", staged_repository, "grep -rn 'git reset' a.txt", 0),
    ]
    for label, start, command, want in index_cases:
        results.append((label, want, 1 if whole_index_reset_refusal(command, start) else 0))
    for agent_type in (None, "clerk"):
        index_payload = {"tool_name": "Bash", "session_id": "s", "cwd": staged_repository, "tool_input": {"command": "git reset", "run_in_background": False}}
        if agent_type:
            index_payload["agent_type"] = agent_type
        index_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                     env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "index-detections.jsonl")), input=json.dumps(index_payload))
        results.append((f"stdin:{'子 agent' if agent_type else '主 agent'} 在暂存区非空时 git reset 拒、出路写只清自己的路径（退出码 2）", (2, True),
                        (index_entry.returncode, "git restore --staged -- <路径…>" in index_entry.stderr)))
    # ㉒ 绕开 git 钩子的 git commit / git push 与 git 钩子没装上：真 git 仓几份（进仓的钩子目录里放两个桩钩子，只看在不在、有没有执行位，不跑），进程里判与走入口各几格；
    # git 一律读不到用户与系统的配置（GIT_CONFIG_GLOBAL、GIT_CONFIG_NOSYSTEM），免得用户全局设的 core.hooksPath 漏进「没装上」的样本；(说明, 命令, 起点仓, 该不该拒)
    large_ceiling = os.environ.get("GIT_CEILING_DIRECTORIES")
    saved_git_environment = {name: os.environ.get(name) for name in ("GIT_CONFIG_GLOBAL", "GIT_CONFIG_NOSYSTEM")}
    os.environ["GIT_CEILING_DIRECTORIES"] = os.path.realpath(work)   # 样本目录往上有仓时 git 也不往上找
    os.environ["GIT_CONFIG_GLOBAL"], os.environ["GIT_CONFIG_NOSYSTEM"] = "/dev/null", "1"
    hook_git_environment = {name: value for name, value in os.environ.items() if name not in ("GIT_INDEX_FILE", "GIT_DIR", "GIT_WORK_TREE")}
    hook_git_environment.update(GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@t", GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@t")

    def hook_git(repository, *arguments):
        subprocess.run(["git", "-C", repository, "-c", "commit.gpgsign=false", *arguments], check=True, capture_output=True, env=hook_git_environment)

    hooks_library = git_hooks_library()   # 桩钩子照 lib_git_hooks 的 HOOK_NAMES 与 HOOKS_DIRECTORY 建，不写死名字与目录
    stub_hook_names, stub_hooks_directory = hooks_library.HOOK_NAMES, hooks_library.HOOKS_DIRECTORY

    def hook_repository(name, config="installed", executable=True, with_hooks=True):
        repository = os.path.join(work, name)
        subprocess.run(["git", "init", "-q", repository], check=True, env=hook_git_environment)
        hook_git(repository, "symbolic-ref", "HEAD", "refs/heads/main")
        if with_hooks:
            for hook_name in stub_hook_names:
                hook_path = os.path.join(repository, stub_hooks_directory, hook_name)
                os.makedirs(os.path.dirname(hook_path), exist_ok=True)
                with open(hook_path, "w", encoding="utf-8") as hook_handle:
                    hook_handle.write("#!/usr/bin/env bash\nexit 0\n")
                os.chmod(hook_path, 0o755 if executable else 0o644)
        with open(os.path.join(repository, "base.txt"), "w", encoding="utf-8") as base_handle:
            base_handle.write("base\n")
        hook_git(repository, "add", "-A")
        hook_git(repository, "commit", "-q", "-m", "base")
        if config:
            hook_git(repository, "config", "core.hooksPath", os.path.join(repository, stub_hooks_directory) if config == "installed" else config)   # "installed" 是装对的值：这个仓的钩子目录的绝对路径
        return repository

    hooked_repository = hook_repository("hook-hooked")                                  # 装好了：core.hooksPath 是这个仓的钩子目录的绝对路径，桩钩子有执行位
    unhooked_repository = hook_repository("hook-unhooked", config=None)                 # 有钩子目录、core.hooksPath 没设
    wrong_repository = hook_repository("hook-wrong", config=".git/hooks")               # core.hooksPath 指到别处
    no_exec_repository = hook_repository("hook-no-exec", executable=False)              # 指对了、钩子文件没有执行位
    plain_repository = hook_repository("hook-plain", config=None, with_hooks=False)     # 没有进仓钩子目录的仓（样本仓、临时仓）
    unhooked_alias_repository = hook_repository("hook-unhooked-alias", config=None)     # 没装钩子，仓里配了别名 ci
    hook_git(unhooked_alias_repository, "config", "alias.ci", "commit -q")
    linked_repository = os.path.join(work, "hook-linked")                               # 装好了的仓的链接工作树：core.hooksPath 写在共用的配置里，指到主工作树的钩子目录
    hook_git(hooked_repository, "worktree", "add", "-q", "-b", "linked", linked_repository)
    unhooked = unhooked_repository
    installed_value = os.path.join(hooked_repository, stub_hooks_directory)   # 安装脚本装的那个值（这个仓的钩子目录的绝对路径）：设成它的 git config、-c、GIT_CONFIG_KEY 放行
    installed_relative = stub_hooks_directory   # 同一个钩子目录写成相对路径：三种写法照拒
    git_hook_cases = [
        ('git commit --no-verify 拒', "git commit --no-verify -m x", hooked_repository, 1),
        ('git commit -n（--no-verify 的短写）拒', "git commit -n -m x", hooked_repository, 1),
        ('git commit -nm 合写的 -n 也认，拒', "git commit -nm x", hooked_repository, 1),
        ('git commit -m 的值是 -n，不当 -n 认，放行', "git commit -m -n", hooked_repository, 0),
        ('git commit --no-veri 是 --no-verify 的缩写，拒', "git commit -q --no-veri -m x", hooked_repository, 1),
        ('git commit --no-edit 不是绕开钩子，放行', "git commit --no-edit --amend", hooked_repository, 0),
        ('git push --no-verify 拒', "git push --no-verify origin main", hooked_repository, 1),
        ('git push -n 是 --dry-run，不是绕开钩子，放行', "git push -n origin main", hooked_repository, 0),
        ('命令行 -c core.hooksPath=… 的 git commit 拒', "git -c core.hooksPath=/dev/null commit -m x", hooked_repository, 1),
        ('命令行 -c core.hooksPath=… 的 git push 拒', "git -c core.hooksPath=/dev/null push", hooked_repository, 1),
        ('GIT_CONFIG_KEY_0=core.hooksPath 的 git commit 拒', "GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null git commit -m x", hooked_repository, 1),
        ('git config 改 core.hooksPath 再 git commit 拒', "git config core.hooksPath /dev/null && git commit -m x", hooked_repository, 1),
        ('git config --get core.hooksPath 只读，放行', "git config --get core.hooksPath", hooked_repository, 0),
        ('HUSKY=0 这类别的工具的开关不算，放行', "HUSKY=0 git commit -m x", hooked_repository, 0),
        ('命令行 -c alias.ci=commit --no-verify 写的别名认成 git commit，拒', "git -c 'alias.ci=commit --no-verify' ci -m x", hooked_repository, 1),
        ('bash -c 里的 git commit --no-verify 拒', "bash -c 'git commit --no-verify -m x'", hooked_repository, 1),
        ('echo | xargs git commit -n 拒', "echo x | xargs git commit -n -m", hooked_repository, 1),
        ('find -exec git commit -n 拒', "find . -maxdepth 0 -exec git commit -n -m x ';'", hooked_repository, 1),
        ('git log -n 5 只读，放行', "git log -n 5", hooked_repository, 0),
        ('钩子没装上（core.hooksPath 没设）的仓里 git commit 拒', "git commit -m x", unhooked_repository, 1),
        ('钩子没装上的仓里 git push 拒', "git push", unhooked_repository, 1),
        ('钩子装上了的仓里 git commit 放行', "git commit -m x", hooked_repository, 0),
        ('core.hooksPath 指到 .git/hooks 的仓里 git commit 拒', "git commit -m x", wrong_repository, 1),
        ('钩子文件没有执行位的仓里 git commit 拒', "git commit -m x", no_exec_repository, 1),
        ('没有进仓钩子目录的仓（样本仓、临时仓）里 git commit 不判，放行', "git commit -m x", plain_repository, 0),
        ('钩子没装上的仓里 git commit --dry-run 不起钩子，放行', "git commit --dry-run", unhooked_repository, 0),
        ('钩子没装上的仓里 git status 只读，放行', "git status -s", unhooked_repository, 0),
        ('git -C 点名没装钩子的仓、起点在别处，拒', f"git -C {unhooked} commit -m x", hooked_repository, 1),
        ('cd 进没装钩子的仓之后 git commit 拒', f"cd {unhooked} && git commit -m x", hooked_repository, 1),
        ('仓里配的别名 git ci（alias.ci=commit -q）在没装钩子的仓里认成 git commit，拒', "git ci -m x", unhooked_alias_repository, 1),
        ('链接工作树里 core.hooksPath 是主工作树钩子目录的绝对路径，装好的放行', "git commit -m x", linked_repository, 0),
        ('--git-dir 改了仓的位置的 git commit 不判，放行', f"git --git-dir={unhooked}/.git commit -m x", hooked_repository, 0),
        ('窗口外提交的环境变量 SINGLEFS_COMMIT_OUTSIDE_WINDOW=1 不算绕开钩子，放行', "SINGLEFS_COMMIT_OUTSIDE_WINDOW=1 git commit -m x", hooked_repository, 0),
        ('git merge --no-verify 拒', "git merge --no-verify -q --no-edit linked", hooked_repository, 1),
        ('git merge -n 是 --no-stat，不是绕开钩子，放行', "git merge -n --no-edit linked", hooked_repository, 0),
        ('git merge -m 的值是 --no-verify，不当绕开认，放行', "git merge -m --no-verify linked", hooked_repository, 0),
        ('git merge --no-edit 不是绕开钩子，放行', "git merge --no-edit linked", hooked_repository, 0),
        ('命令行 -c core.hooksPath=… 的 git merge 拒', "git -c core.hooksPath=/dev/null merge linked", hooked_repository, 1),
        ('git am --no-verify 拒', "git am --no-verify patch.mbox", hooked_repository, 1),
        ('git am -n 是 --no-verify 的短写，拒', "git am -n patch.mbox", hooked_repository, 1),
        ('git am -3n 合写的 -n 也认，拒', "git am -3n patch.mbox", hooked_repository, 1),
        ('git am --no-veri 是 --no-verify 的缩写，拒', "git am -q --no-veri patch.mbox", hooked_repository, 1),
        ('命令行 -c core.hooksPath=… 的 git am 拒', "git -c core.hooksPath=/dev/null am patch.mbox", hooked_repository, 1),
        ('git am -p 的值是 -n，不当绕开认，放行', "git am -p -n patch.mbox", hooked_repository, 0),
        ('git am --verify 不是绕开钩子，放行', "git am --verify --3way patch.mbox", hooked_repository, 0),
        ('git am 不带绕开写法，放行', "git am -q --signoff patch.mbox", hooked_repository, 0),
        ('git config --remove-section core 再 git commit 拒', "git config --remove-section core && git commit -m x", hooked_repository, 1),
        ('git config --rename-section core moved 再 git commit 拒', "git config --rename-section core moved && git commit -m x", hooked_repository, 1),
        ('git config --remove-sec core（--remove-section 的缩写）拒', "git config --remove-sec core", hooked_repository, 1),
        ('git config remove-section core（子命令写法）拒', "git config remove-section core", hooked_repository, 1),
        ('git config --remove-section user 不碰 core 段，放行', "git config --remove-section user", hooked_repository, 0),
        ('git config core.hooksPath 设成进仓钩子目录的绝对路径，放行', f"git config core.hooksPath {installed_value}", hooked_repository, 0),
        ('命令行 -c core.hooksPath=<进仓钩子目录的绝对路径> 的 git commit 放行', f"git -c core.hooksPath={installed_value} commit -m x", hooked_repository, 0),
        ('GIT_CONFIG_KEY_0=core.hooksPath 加 GIT_CONFIG_VALUE_0=<进仓钩子目录的绝对路径> 的 git commit 放行',
         f"GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0={installed_value} git commit -m x", hooked_repository, 0),
        ('git config --add core.hooksPath <进仓钩子目录的绝对路径> 照拒（再加一个值）', f"git config --add core.hooksPath {installed_value}", hooked_repository, 1),
        ('git config core.hooksPath 设成进仓钩子目录的相对路径，拒', f"git config core.hooksPath {installed_relative}", hooked_repository, 1),
        ('命令行 -c core.hooksPath=<进仓钩子目录的相对路径> 的 git commit 拒', f"git -c core.hooksPath={installed_relative} commit -m x", hooked_repository, 1),
        ('GIT_CONFIG_KEY_0=core.hooksPath 加 GIT_CONFIG_VALUE_0=<进仓钩子目录的相对路径> 的 git commit 拒',
         f"GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0={installed_relative} git commit -m x", hooked_repository, 1),
    ]
    for label, command, start, want in git_hook_cases:
        results.append((f"提交前钩子:{label}", want, 1 if git_hook_refusal(command, start, None) else 0))
    notices_before = len(JUDGING_NOTICES)
    real_hook_findings = bash_command_detector_hooks_and_overwrites.git_hook_findings
    bash_command_detector_hooks_and_overwrites.git_hook_findings = lambda command, directory: [("git commit -n", "bypass")]   # 项缺说明：拆元组出错
    try:
        guarded_lines = git_hook_refusal("git commit -n", hooked_repository, None)
    except Exception:   # 弄坏开关 git-hook-messages-unguarded 下异常到这里
        guarded_lines = None
    finally:
        bash_command_detector_hooks_and_overwrites.git_hook_findings = real_hook_findings
    results.append(("提交前钩子:写拒绝说明那一步出错（项缺说明）折成判不了：不拒、记一条检出", ([], 1), (guarded_lines, len(JUDGING_NOTICES) - notices_before)))
    del JUDGING_NOTICES[:]
    hook_entry_environment = dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "hook-detections.jsonl"))
    for label, agent_type, start, command, timeout, want in (
            ("stdin:主 agent 的 git commit --no-verify 拒绝（退出码 2），出路写去掉它与窗口外提交的环境变量", None, hooked_repository, "git commit --no-verify -m x", None, (2, True, True)),
            ("stdin:子 agent 在没装钩子的仓里 git commit 拒绝（退出码 2），出路写进报告交主 agent", "clerk", unhooked_repository, "git commit -m x", None, (2, True, True)),
            ("stdin:主 agent 在没装钩子的仓里 git push 拒绝（退出码 2），出路给任务服务装钩子的整条命令", None, unhooked_repository, "git push", None, (2, True)),
            ("stdin:装好钩子的仓里 git commit 放行（退出码 0）", None, hooked_repository, "git commit -m x", None, (0, False)),
            ("stdin:主 agent 的 git am --no-verify 拒绝（退出码 2），出路写去掉它", None, hooked_repository, "git am --no-verify patch.mbox", None, (2, True, True)),
            ("stdin:前台 timeout 600000 毫秒又 git commit --no-verify，两类一起列", None, hooked_repository, "git commit --no-verify -m x", 600000, (2, True, True))):
        hook_payload = {"tool_name": "Bash", "session_id": "s-hook", "cwd": start, "tool_input": {"command": command, "run_in_background": False}}
        if agent_type:
            hook_payload["agent_type"] = agent_type
        if timeout:
            hook_payload["tool_input"]["timeout"] = timeout
        hook_entry = subprocess.run(["bash", detector_script], input=json.dumps(hook_payload), capture_output=True, text=True, env=hook_entry_environment)
        if timeout:
            got = (hook_entry.returncode, "踩了 2 类" in hook_entry.stderr, "前台命令的 timeout" in hook_entry.stderr and "绕开了 git 的钩子" in hook_entry.stderr)
        elif agent_type:
            got = (hook_entry.returncode, "要主 agent 跑的命令" in hook_entry.stderr, GIT_HOOKS_INSTALL_COMMAND in hook_entry.stderr)
        elif command == "git push":
            got = (hook_entry.returncode, f"task-server/task submit -- {GIT_HOOKS_INSTALL_COMMAND}" in hook_entry.stderr)
        elif want[0] == 0:
            got = (hook_entry.returncode, "git 钩子" in hook_entry.stderr)
        else:
            got = (hook_entry.returncode, "--no-verify" in hook_entry.stderr, f"{HOOKS_BYPASS_WINDOW_VARIABLE}=1" in hook_entry.stderr)
        results.append((label, want, got))
    if large_ceiling is None:
        del os.environ["GIT_CEILING_DIRECTORIES"]
    else:
        os.environ["GIT_CEILING_DIRECTORIES"] = large_ceiling
    for name, value in saved_git_environment.items():
        if value is None:
            os.environ.pop(name, None)
        else:
            os.environ[name] = value
    # ⑰ 往仓里的文件写字面内容：临时目录里造一个假仓（判据库用 hook 所在仓的那一份），进程里判几格，走入口两格；
    # 样本串里的会话名与钟点拆开拼，这份源码里不出现字面写法（门禁 doc-text 也扫它）；(说明, 命令, 该命中的形态)
    literal_session = "".join(("singlefs-", "e1"))
    literal_clock = "".join(("用户 23", ":", "10 定"))
    literal_repository, literal_outside = os.path.join(work, "literal-repo"), os.path.join(work, "literal-outside")
    for literal_directory in (os.path.join(literal_repository, "records"), os.path.join(literal_repository, "scripts"),
                              os.path.join(literal_repository, "research", "prompts"), literal_outside):
        os.makedirs(literal_directory, exist_ok=True)
    # 任务号、会话短号与批号判写前写后两份整份目标：fence-open.md 末尾开着围栏，fence-closed.md 的已经收了，stale-append.md 给同一条命令两次追加（前一次收掉围栏），
    # old-number.md 带一行旧号，table.md 是带「批次」表头的一张表
    for fence_name, fence_text in (("fence-open.md", "前文\n```\n代码\n"), ("fence-closed.md", "前文\n```\n代码\n```\n"),
                                   ("stale-append.md", "前文\n```\n代码\n"), ("old-number.md", "旧行（任务 21036）\n"),
                                   ("table.md", "| 批次 | 依赖 |\n|---|---|\n| 批 5 | 无 |\n")):
        with open(os.path.join(literal_repository, "records", fence_name), "w", encoding="utf-8") as handle:
            handle.write(fence_text)
    closing_fence_heredoc = "<<'EOF'\n```\n\n正文：这件事 (5b) 知道\nEOF"
    literal_cases = [
        ("字面写入:heredoc 追加会话名拒", f"cat >> records/x.md <<'EOF'\n已发消息，{literal_session} 回话\nEOF", ("会话号",)),
        ("字面写入:heredoc 追加批次代号拒", "cat >> records/x.md <<'EOF'\n- 24c-1 交回\nEOF", ("批次代号",)),
        ("字面写入:printf 追加钟点拒", f"printf '%s\\n' '{literal_clock}' >> records/x.md", ("钟点",)),
        ("字面写入:echo 经管道交 tee -a 追加批次代号拒", "echo '22a 的接口改了' | tee -a records/x.md", ("批次代号",)),
        ("字面写入:cd 进仓里之后按相对路径写照判", "cd records && cat > notes.md <<'EOF'\n批 10 甲交回\nEOF", ("批次代号",)),
        ("字面写入:bash -c 里的 echo 追加会话名照判", f"bash -c \"echo '{literal_session} 回话' >> records/x.md\"", ("会话号",)),
        ("字面写入:here-string 喂给 cat 写批次代号照判", "cat <<< '24c-1 交回' >> records/x.md", ("批次代号",)),
        ("字面写入:heredoc 追加任务号拒", "cat >> records/x.md <<'EOF'\n- 调度那一条跑完（任务 21036）\nEOF", ("任务号、会话短号或批号",)),
        ("字面写入:printf 追加括号里的会话短号拒", "printf '%s\\n' '这件事 (5b) 知道' >> records/x.md", ("任务号、会话短号或批号",)),
        ("字面写入:echo 经管道交 tee -a 追加别的会话的批号拒", "echo '它的批 105 在修' | tee -a records/x.md", ("任务号、会话短号或批号",)),
        ("字面写入:记录里的 A/B 读数一行拒", "cat >> records/x.md <<'EOF'\nA/B 读数：任务 11964\nEOF", ("任务号、会话短号或批号",)),
        ("字面写入:反引号里的任务号拒", "echo '结局行 `任务 21036 退 0` 原样' >> records/x.md", ("任务号、会话短号或批号",)),
        ("字面写入:往开着围栏的文件追加以收尾围栏起头的内容拒", f"cat >> records/fence-open.md {closing_fence_heredoc}", ("任务号、会话短号或批号",)),
        ("字面写入:往围栏已收的文件追加以围栏起头的内容放行", f"cat >> records/fence-closed.md {closing_fence_heredoc}", ()),
        ("字面写入:整份写不看目标里原有的围栏，以围栏起头的内容放行", f"cat > records/fence-open.md {closing_fence_heredoc}", ()),
        ("字面写入:同一条命令两次追加让批号出了围栏拒",
         "printf '%s\\n' '```' >> records/stale-append.md; printf '%s\\n' '那一路第 7 批交回' >> records/stale-append.md", ("任务号、会话短号或批号",)),
        ("字面写入:记录里表格别的列的批号拒", "cat >> records/table.md <<'EOF'\n| 批 7 | 批 6 |\nEOF", ("任务号、会话短号或批号",)),
        ("字面写入:记录里表格行的批号放行", "cat >> records/table.md <<'EOF'\n| 批 6 | 交回 |\nEOF", ()),
        # 拆开写的号：printf 的格式串与参数分开、前面赋过字面值的变量、echo -e 的转义、echo -n 接下一处追加，照 shell 真写出的字判
        ("字面写入:printf 格式串与参数拆开写的任务号拒", "printf '（任务 %s）退 0\\n' 21036 >> records/x.md", ("任务号、会话短号或批号",)),
        ("字面写入:前面赋过字面值的变量写进的任务号拒", 'n=21036; echo "（任务 $n）退 0" >> records/x.md', ("任务号、会话短号或批号",)),
        ("字面写入:结束词不带引号的 heredoc 里前面赋过字面值的变量照代入拒", "n=21036; cat >> records/x.md <<EOF\n- 跑完（任务 $n）\nEOF", ("任务号、会话短号或批号",)),
        ("字面写入:echo -e 转义拼出的批号拒", "echo -e '批\\x20105 在修' >> records/x.md", ("任务号、会话短号或批号",)),
        ("字面写入:echo -n 接下一处 echo 拼出的任务号拒", "echo -n '（任务 ' >> records/x.md; echo '21036）退 0' >> records/x.md", ("任务号、会话短号或批号",)),
        ("字面写入:不带 -e 的 echo 不换转义放行", "echo '批\\x20105 在修' >> records/x.md", ()),
        ("字面写入:没赋过值的变量不代入放行", 'echo "（任务 $UNSET_NUMBER）退 0" >> records/x.md', ()),
        ("字面写入:往有旧号的文件追加干净内容放行", "echo '新的一行' >> records/old-number.md", ()),
        ("字面写入:判决里行首的 A/B 读数一行放行", "cat >> research/prompts/x-main-verification.md <<'EOF'\nA/B 读数：任务 11964\nEOF", ()),
        ("字面写入:干净内容放行", "cat >> records/x.md <<'EOF'\n配置判法那一份补丁交回\nEOF", ()),
        ("字面写入:两位会话短号放行", "echo '已告诉 87 与 7a' >> records/x.md", ()),
        ("字面写入:仓外路径不判", f"cat >> {literal_outside}/notes.md <<'EOF'\n24c-1 交回，{literal_session}\nEOF", ()),
        ("字面写入:排除前缀下的样本不判", f"echo '{literal_session}' > .claude/gate.d/fixtures/x/expect", ()),
        ("字面写入:写进 .py 的批次代号不判（批次代号只判 .md、.tsv、.txt）", "echo '# 24c-1 交回' >> scripts/x.py", ()),
        ("字面写入:目标算不出的不判", "cat >> \"$TARGET\" <<'EOF'\n24c-1 交回\nEOF", ()),
        ("字面写入:cat 拷文件不是字面内容，不判", "cat a.md >> records/x.md", ()),
    ]
    for label, command, want in literal_cases:
        literal_hits = literal_notation_hits(command, literal_repository, literal_repository)
        results.append((label, want, tuple(sorted({form_name for _, _, form_name, _, _ in literal_hits}))))
    for label, command, want in (
            ("stdin:Bash heredoc 往仓里追加会话名拒绝（退出码 2，点名会话号、出路指到 insert-row.py 与 replace-once.py）",
             f"cat >> research/bash-command-detector-selftest-literal.md <<'EOF'\n已发消息，{literal_session} 回话\nEOF", (2, True)),
            ("stdin:Bash heredoc 往仓里追加干净内容放行（退出码 0）",
             "cat >> research/bash-command-detector-selftest-literal.md <<'EOF'\n配置判法那一份补丁交回\nEOF", (0, False))):
        literal_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                       env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "literal-detections.jsonl")),
                                       input=json.dumps({"tool_name": "Bash", "session_id": "s", "cwd": repository_root_of(hook_dir),
                                                         "tool_input": {"command": command, "run_in_background": False}}))
        results.append((label, want, (literal_entry.returncode, all(fragment in literal_entry.stderr
                                                                   for fragment in (literal_session, "会话号", "insert-row.py", "replace-once.py")))))
    # ⑱ 带路径撤掉工作区里没暂存的改动：临时仓一份（a.txt 有没暂存的改动、b.txt 干净、c.txt 改了而且整份暂存、u.txt 没跟踪），进程里判与走入口各几格；
    # (说明, 起点, 命令, 该不该拒)；起点 work 不是 git 仓
    discard_repository = os.path.join(work, "discard-repository")
    subprocess.run(["git", "init", "-q", discard_repository], check=True)
    for name in ("a.txt", "b.txt", "c.txt"):
        with open(os.path.join(discard_repository, name), "w", encoding="utf-8") as handle:
            handle.write(name + "\n")
    subprocess.run(["git", "-C", discard_repository, "add", "a.txt", "b.txt", "c.txt"], check=True)
    subprocess.run(["git", "-C", discard_repository, "-c", "user.email=t@t", "-c", "user.name=t", "-c", "commit.gpgsign=false",
                    "commit", "-q", "-m", "base"], check=True)
    for name, text in (("a.txt", "a 没暂存的改动\n"), ("c.txt", "c 暂存了的改动\n"), ("u.txt", "没跟踪\n")):
        with open(os.path.join(discard_repository, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    subprocess.run(["git", "-C", discard_repository, "add", "c.txt"], check=True)
    discard_cases = [
        ("撤工作区改动:git checkout -- 有改动的路径拒", discard_repository, "git checkout -- a.txt", 1),
        ("撤工作区改动:git checkout HEAD -- 有改动的路径拒", discard_repository, "git checkout HEAD -- a.txt", 1),
        ("撤工作区改动:git checkout 不带 -- 的有改动路径拒", discard_repository, "git checkout a.txt", 1),
        ("撤工作区改动:git restore 有改动的路径拒", discard_repository, "git restore a.txt", 1),
        ("撤工作区改动:git restore --staged --worktree 有改动的路径拒", discard_repository, "git restore --staged --worktree -- a.txt", 1),
        ("撤工作区改动:git stash push -- 有改动的路径拒", discard_repository, "git stash push -- a.txt", 1),
        ("撤工作区改动:git stash push -m 带说明、有改动的路径拒", discard_repository, "git stash push -m keep -- a.txt", 1),
        ("撤工作区改动:git clean -f 点名没跟踪的文件拒", discard_repository, "git clean -f u.txt", 1),
        ("撤工作区改动:有改动的路径与没改动的路径同写拒", discard_repository, "git checkout -- b.txt a.txt", 1),
        ("撤工作区改动:git checkout -- . 整棵树里有改动拒", discard_repository, "git checkout -- .", 1),
        ("撤工作区改动:前面赋过值的变量照展开，有改动拒", discard_repository, 'f=a.txt; git checkout -- "$f"', 1),
        ("撤工作区改动:cd 进仓之后 git checkout -- 有改动的路径拒", work, f"cd {discard_repository} && git checkout -- a.txt", 1),
        ("撤工作区改动:git -C 点名仓、起点在别处，拒", work, f"git -C {discard_repository} restore a.txt", 1),
        ("撤工作区改动:路径写成没赋过值的变量，判不了，拒", discard_repository, 'git checkout -- "$SOMEWHERE_UNSET"', 1),
        ("撤工作区改动:cd 到没赋过值的变量之后，判不了，拒", work, 'cd "$WHERE_UNSET" && git checkout -- a.txt', 1),
        ("撤工作区改动:git checkout -- 没改动的路径放行", discard_repository, "git checkout -- b.txt", 0),
        ("撤工作区改动:git restore 没改动的路径放行", discard_repository, "git restore b.txt", 0),
        ("撤工作区改动:只暂存了、工作区与暂存区相同的路径放行", discard_repository, "git restore c.txt", 0),
        ("撤工作区改动:git stash push -- 没改动的路径放行", discard_repository, "git stash push -- b.txt", 0),
        ("撤工作区改动:git clean -f 点名的路径下没有没跟踪的文件放行", discard_repository, "git clean -f b.txt", 0),
        ("撤工作区改动:不在仓里的路径放行", discard_repository, "git checkout -- not-here.txt", 0),
        ("撤工作区改动:git restore --staged 有改动的路径放行（只动暂存区）", discard_repository, "git restore --staged a.txt", 0),
        ("撤工作区改动:git restore -S 有改动的路径放行（只动暂存区）", discard_repository, "git restore -S -- a.txt", 0),
        ("撤工作区改动:git clean -n 只列不删放行", discard_repository, "git clean -n u.txt", 0),
        ("撤工作区改动:git checkout -b 切新分支放行", discard_repository, "git checkout -b side", 0),
        ("撤工作区改动:grep 参数里的 git checkout 不判", discard_repository, "grep -n 'git checkout -- a.txt' a.txt", 0),
    ]
    for label, start, command, want in discard_cases:
        results.append((label, want, 1 if worktree_discard_refusal(command, start) else 0))
    for agent_type in (None, "clerk"):
        discard_payload = {"tool_name": "Bash", "session_id": "s", "cwd": discard_repository,
                           "tool_input": {"command": "git checkout -- a.txt", "run_in_background": False}}
        if agent_type:
            discard_payload["agent_type"] = agent_type
        discard_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                       env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "discard-detections.jsonl")), input=json.dumps(discard_payload))
        results.append((f"stdin:{'子 agent' if agent_type else '主 agent'} 带路径撤掉没暂存的改动拒、出路写先留底与 git apply -R（退出码 2）", (2, True),
                        (discard_entry.returncode, "git apply -R" in discard_entry.stderr)))
    # ⑯ 往现行账本直接追加节标题：假仓里放登记表与两份账本（进程里判），再走真实入口一趟（真仓的登记表，只判不写）；(说明, 命令, 该不该拒)
    ledger_repository = os.path.join(work, "ledger-repo")
    os.makedirs(os.path.join(ledger_repository, ".claude", "gate.d"))
    os.makedirs(os.path.join(ledger_repository, "records"))
    with open(os.path.join(ledger_repository, LEDGER_REGISTRY_PATH), "w", encoding="utf-8") as handle:
        handle.write("current\trecords/ledger.md\t-\t样本里的现行账本，自证用\nsealed\trecords/old.md\t" + "0" * 64 + "\t样本里的封存账本，自证用\n")
    for ledger_name in ("ledger.md", "old.md"):
        with open(os.path.join(ledger_repository, "records", ledger_name), "w", encoding="utf-8") as handle:
            handle.write("# 账本\n\n## 一、甲\n")
    new_heading = "## 一百七十九、新节"
    ledger_cases = [
        ("往现行账本追加:cat >> 加 heredoc 拒", f"cat >> records/ledger.md <<'EOF'\n{new_heading}\n\n正文\nEOF", 1),
        ("往现行账本追加:echo >> 拒", f"echo '{new_heading}' >> records/ledger.md", 1),
        ("往现行账本追加:管道接 tee -a 拒", f"printf '\\n{new_heading}\\n' | tee -a records/ledger.md", 1),
        ("往现行账本追加:cd 进 records 之后 >> 拒", f"cd records && echo '{new_heading}' >> ledger.md", 1),
        ("往现行账本追加:绝对路径 >> 拒", f"echo '{new_heading}' >> {ledger_repository}/records/ledger.md", 1),
        ("往现行账本追加:喂给 python 的 heredoc 里 open(…, 'a') 拒", f"python3 - <<'EOF'\nopen('records/ledger.md', 'a').write('{new_heading}')\nEOF", 1),
        ("往现行账本追加:喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个是现行账本拒",
         f"python3 - <<'EOF'\nfor f in ('records/notes.md', 'records/ledger.md'):\n    open(f, 'a').write('{new_heading}')\nEOF", 1),
        ("往现行账本追加:写别的文件放行", f"echo '{new_heading}' >> records/notes.md", 0),
        ("往现行账本追加:封存的账本放行", f"echo '{new_heading}' >> records/old.md", 0),
        ("往现行账本追加:往账本写不带节标题的一行放行", "echo '去向：做完' >> records/ledger.md", 0),
        ("往现行账本追加:只读账本里的节标题放行", f"grep -n '{new_heading}' records/ledger.md", 0),
        ("往现行账本追加:经 ledger-append-section.py 放行", f"python3 scripts/kb/ledger-append-section.py --title '{new_heading}' --body-file draft.txt", 0),
        ("往现行账本追加:sed -i 改节号放行", f"sed -i 's/^## 一、/{new_heading}/' records/ledger.md", 0),
    ]
    for label, command, want in ledger_cases:
        results.append((label, want, 1 if ledger_append_refusal(command, ledger_repository, ledger_repository)[0] else 0))
    real_ledger, _ = current_ledger_relative(repository_root_of(hook_dir))
    for agent_type in (None, "clerk"):
        ledger_payload = {"tool_name": "Bash", "session_id": "s", "cwd": repository_root_of(hook_dir),
                          "tool_input": {"command": f"echo '{new_heading}' >> {real_ledger}", "run_in_background": False}}
        if agent_type:
            ledger_payload["agent_type"] = agent_type
        ledger_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                      env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "ledger-detections.jsonl")), input=json.dumps(ledger_payload))
        results.append((f"stdin:{'子 agent' if agent_type else '主 agent'} 往现行账本追加节标题拒、出路写经 ledger-append-section.py（退出码 2）", (2, True),
                        (ledger_entry.returncode, "scripts/kb/ledger-append-section.py" in ledger_entry.stderr)))
    # ⑲ 往 records/ 下的 .md 追加：假仓里判几格（进程里），再走真实入口两格（真仓里一份不在的记录名，只判不写）；(说明, 命令, 该不该拒)
    record_repository = os.path.join(work, "record-repo")
    for record_directory in ("records", "scripts", "research"):
        os.makedirs(os.path.join(record_repository, record_directory))
    with open(os.path.join(record_repository, "records", "run.md"), "w", encoding="utf-8") as handle:
        handle.write("# 记录\n")
    os.symlink(os.path.join("..", "records", "run.md"), os.path.join(record_repository, "scripts", "link-in.md"))
    os.symlink(os.path.join(work, "record-outside.md"), os.path.join(record_repository, "records", "link-out.md"))
    record_cases = [
        ("往 records 追加:echo >> 拒", "echo '- 乙' >> records/run.md", 1),
        ("往 records 追加:cat >> 加 heredoc 拒", "cat >> records/run.md <<'EOF'\n- 乙\nEOF", 1),
        ("往 records 追加:&>> 拒", "make &>> records/run.md", 1),
        ("往 records 追加:管道接 tee -a 拒", "printf '%s\\n' '- 乙' | tee -a records/run.md", 1),
        ("往 records 追加:tee --append 拒", "echo x | tee --append records/run.md", 1),
        ("往 records 追加:cd 进 records 之后 >> 拒", "cd records && echo x >> run.md", 1),
        ("往 records 追加:绝对路径 >> 拒", f"echo x >> {record_repository}/records/run.md", 1),
        ("往 records 追加:前面赋过值的变量照代入拒", "R=records/run.md; echo x >> $R", 1),
        ("往 records 追加:bash -c 里的 >> 拒", "bash -c \"echo x >> records/run.md\"", 1),
        ("往 records 追加:python3 -c 里 open(…, 'a') 拒", "python3 -c \"open('records/run.md', 'a').write('x')\"", 1),
        ("往 records 追加:喂给 python 的 heredoc 里 open(…, mode='a') 拒", "python3 - <<'EOF'\nopen('records/run.md', mode='a').write('x')\nEOF", 1),
        ("往 records 追加:喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个落在 records 下拒",
         "python3 - <<'EOF'\nfor f in ('research/notes.md', 'records/run.md'):\n    open(f, 'a').write('x')\nEOF", 1),
        ("往 records 追加:python 的 Path.open('a') 拒", "python3 -c \"from pathlib import Path; Path('records/run.md').open('a').write('x')\"", 1),
        ("往 records 追加:子目录里的 .md 也拒", "echo x >> records/sub/notes.md", 1),
        ("往 records 追加:大写后缀 .MD 也拒", "echo x >> records/RUN.MD", 1),
        ("往 records 追加:指进 records 的符号链接照落点拒", "echo x >> scripts/link-in.md", 1),
        ("往 records 追加:经 append-record.py 放行", "python3 scripts/kb/append-record.py records/run.md draft.txt", 0),
        ("往 records 追加:标准输入喂给 append-record.py 放行", "printf '%s\\n' '- 乙' | python3 scripts/kb/append-record.py records/run.md -", 0),
        ("往 records 追加:records 下别的后缀不判", "echo x >> records/table.tsv", 0),
        ("往 records 追加:records 之外的 .md 不判", "echo x >> research/notes.md", 0),
        ("往 records 追加:records 同名前缀的兄弟目录不判", "echo x >> records-old/run.md", 0),
        ("往 records 追加:records 里指到仓外的符号链接照落点不判", "echo x >> records/link-out.md", 0),
        ("往 records 追加:仓外同名路径不判", f"echo x >> {work}/elsewhere/records/run.md", 0),
        ("往 records 追加:> 整份写不是追加，不判", "echo x > records/run.md", 0),
        ("往 records 追加:python open(…, 'w') 不是追加，不判", "python3 -c \"open('records/run.md', 'w').write('x')\"", 0),
        ("往 records 追加:只读不判", "grep -n x records/run.md", 0),
        ("往 records 追加:目标算不出的不判", "echo x >> \"$UNSET_RECORD\"", 0),
    ]
    for label, command, want in record_cases:
        results.append((label, want, 1 if record_append_refusal(command, record_repository, record_repository) else 0))
    howto_not_run = list(read_only_scope_not_run)   # 没判的格（出路过别的闸那几格：hook 旁边没有 project-code-guard.sh；只读定义那几格：仓里没有定义），成功句里报「本次未跑」
    for agent_type in (None, "clerk"):
        record_payload = {"tool_name": "Bash", "session_id": "s", "cwd": repository_root_of(hook_dir),
                          "tool_input": {"command": "echo '- 自证' >> records/bash-command-detector-selftest-record.md", "run_in_background": False}}
        if agent_type:
            record_payload["agent_type"] = agent_type
        record_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                      env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "record-detections.jsonl")), input=json.dumps(record_payload))
        results.append((f"stdin:{'子 agent' if agent_type else '主 agent'} 往 records 追加拒、出路给 append-record.py 的整条写法（退出码 2）", (2, True),
                        (record_entry.returncode, "python3 scripts/kb/append-record.py records/bash-command-detector-selftest-record.md" in record_entry.stderr)))
        # 出路本身要过得了别的闸：拒绝说明里那条整条写法照抄（文本文件换成草稿里的一份真文件），喂给同挂 PreToolUse[Bash] 的 project-code-guard.sh，要放行
        howto_label = f"出路过别的闸:{'子 agent' if agent_type else '主 agent'} 照抄 ⑲ 的出路喂 project-code-guard.sh 放行（退出码 0）"
        guard_script = os.path.join(hook_dir, "project-code-guard.sh")
        if not os.path.isfile(guard_script):
            howto_not_run.append(f"{howto_label}（hook 旁边没有 project-code-guard.sh）")
        else:
            howto = RECORD_HOWTO_COMMAND.search(record_entry.stderr)
            howto_exit = None   # 拒绝说明里取不出出路也判红
            if howto:
                howto_text = os.path.join(work, "record-howto-text.txt")
                with open(howto_text, "w", encoding="utf-8") as handle:
                    handle.write("- 自证\n")
                guard_payload = {"tool_name": "Bash", "session_id": "s", "cwd": repository_root_of(hook_dir),
                                 "tool_input": {"command": howto.group(1).replace(RECORD_HOWTO_TEXT_PLACEHOLDER, howto_text), "run_in_background": False}}
                if agent_type:
                    guard_payload["agent_type"] = agent_type
                howto_exit = subprocess.run(["bash", guard_script], capture_output=True, text=True, input=json.dumps(guard_payload),
                                            env=howto_guard_environment(os.environ, os.path.join(work, "record-howto-detections.jsonl"))).returncode
            results.append((howto_label, 0, howto_exit))
    # ⑳ 手拷仓副本：(说明, 命令, 该不该拒[, 起点目录])；假仓在草稿里（crates/demo、Cargo.toml），草稿目录另放一份副本的 crates/ 与笔记；起点不写是假仓根
    copy_repository = os.path.join(work, "manual-copy-repository")
    copy_scratch = os.path.join(work, "manual-copy-scratch")
    for base_directory, relative in ((copy_repository, "crates/demo/src/lib.rs"), (copy_repository, "Cargo.toml"),
                                     (copy_scratch, "notes/a.md"), (copy_scratch, "copy/crates/demo/src/lib.rs")):
        os.makedirs(os.path.dirname(os.path.join(base_directory, relative)), exist_ok=True)
        with open(os.path.join(base_directory, relative), "w", encoding="utf-8") as handle:
            handle.write("\n")
    manual_copy_cases = [
        ("rsync 拷 crates 进仓外目录", f"rsync -a --exclude target crates {copy_scratch}/copy/", 1),
        ("cp -r 拷 crates", f"cp -r crates {copy_scratch}/copy/", 1),
        ("rsync 拷仓根整个（./）", f"rsync -a --exclude target --exclude .git ./ {copy_scratch}/copy/", 1),
        ("rsync 拷仓根（绝对路径带结尾的 /）", f"rsync -a --exclude target --exclude .git {copy_repository}/ {copy_scratch}/copy/", 1),
        ("cp -a 拷仓根 .", f"cp -a . {copy_scratch}/copy", 1),
        ("cp -R 拷 crates/（带结尾的 /）", f"cp -R crates/ {copy_scratch}/copy/crates", 1),
        ("cp --archive 拷 crates", f"cp --archive crates {copy_scratch}/copy/", 1),
        ("cp -t 目录写在前面、-r 写在后面", f"cp -t {copy_scratch}/copy -r crates", 1),
        ("rsync 的 --exclude=值 合写", f"rsync -av --exclude=target crates {copy_scratch}/copy/", 1),
        ("rsync -e 带值的短选项之后的源照认", f"rsync -a -e ssh crates {copy_scratch}/copy/", 1),
        ("cd 进仓根之后 rsync crates", f"cd {copy_repository} && rsync -a crates {copy_scratch}/copy/", 1, copy_scratch),
        ("前面赋过值的变量代入", f"R={copy_repository}; rsync -a $R/crates {copy_scratch}/copy/", 1, copy_scratch),
        ("bash -c 里", f"bash -c 'cp -r crates {copy_scratch}/copy/'", 1),
        ("nice 前缀", f"nice -n 19 rsync -a crates {copy_scratch}/copy/", 1),
        ("仓根里的通配 * 盖住整层", f"cp -r * {copy_scratch}/copy/", 1),
        ("crates/* 盖住 crates 整层", f"cp -r crates/* {copy_scratch}/copy/crates/", 1),
        ("crates/.. 规整之后是仓根", f"rsync -a crates/.. {copy_scratch}/copy/", 1),
        ("喂给 bash 的 heredoc 里", f"bash <<'EOF'\nrsync -a crates {copy_scratch}/copy/\nEOF", 1),
        ("rsync 一次给几个源、其中有 crates", f"rsync -a Cargo.toml crates {copy_scratch}/copy/", 1),
        ("minimal-copy.py 本身放行", f"python3 scripts/hosts/minimal-copy.py {copy_repository} {copy_scratch}/copy --with-test-inputs", 0),
        ("拷草稿目录里不含本仓 crates 的东西放行", f"rsync -a {copy_scratch}/notes/ {copy_scratch}/notes-backup/", 0),
        ("拷草稿目录里别的副本的 crates 放行", f"cp -r {copy_scratch}/copy/crates {copy_scratch}/copy-two/", 0),
        ("cp 不带递归放行（目录拷不过去）", f"cp crates {copy_scratch}/x", 0),
        ("rsync -n 只演一遍放行", f"rsync -an crates {copy_scratch}/copy/", 0),
        ("rsync --dry-run 放行", f"rsync --dry-run -a crates {copy_scratch}/copy/", 0),
        ("rsync 只给一个操作数是只列，放行", "rsync -a crates", 0),
        ("拷 crates 下单个包的目录放行", f"cp -r crates/demo {copy_scratch}/copy/crates/", 0),
        ("拷 crates 下单个文件放行", f"cp crates/demo/src/lib.rs {copy_scratch}/lib.rs", 0),
        ("echo 引号里当数据写的放行", f"echo 'rsync -a crates {copy_scratch}/copy/'", 0),
        ("grep 参数里的放行", "grep -n 'cp -r crates' notes.md", 0),
        ("写进文件的 heredoc 正文放行", f"cat > {copy_scratch}/note.md <<'EOF'\nrsync -a crates /x/\nEOF", 0),
        ("源是没赋过值的变量放行（算不出）", 'rsync -a "$MANUAL_COPY_SOURCE_UNSET" /x/', 0),
        ("远端源（主机:路径）放行", "rsync -a host:crates /x/", 0),
        ("同名前缀的兄弟目录放行", f"cp -r crates-old {copy_scratch}/x", 0),
        ("-- 之后的 -r 是文件名、不算递归，放行", "cp -- -r crates", 0),
    ]
    for label, command, want, *start in manual_copy_cases:
        results.append((f"手拷仓副本:{label}", want, 1 if manual_repository_copy_refusal(command, copy_repository, start[0] if start else copy_repository) else 0))
    # 走真实入口（这一份钩子、它所在的仓根）：主 agent 与子 agent 手拷本仓的 crates 各拒一格、拒绝说明给 minimal-copy.py 的整条写法；
    # 出路本身要过得了别的闸：那条整条写法照抄（副本换成草稿里的一个路径）喂给同挂 PreToolUse[Bash] 的 project-code-guard.sh，要放行；起 minimal-copy.py 本身放行
    manual_copy_root = repository_root_of(hook_dir)
    for agent_type, command, label in ((None, f"rsync -a --exclude target crates {copy_scratch}/copy/", "主 agent rsync 拷 crates"),
                                       ("implementation-writer", f"cp -r crates {copy_scratch}/copy/", "子 agent cp -r 拷 crates")):
        manual_payload = {"tool_name": "Bash", "session_id": "s", "cwd": manual_copy_root, "tool_input": {"command": command, "run_in_background": False}}
        if agent_type:
            manual_payload["agent_type"] = agent_type
        manual_entry = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                      env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "manual-copy-detections.jsonl")), input=json.dumps(manual_payload))
        manual_howto = MANUAL_COPY_HOWTO_COMMAND.search(manual_entry.stderr)
        results.append((f"stdin:{label} 拒、出路给 minimal-copy.py 的整条写法（退出码 2）", (2, True), (manual_entry.returncode, manual_howto is not None)))
        manual_howto_label = f"出路过别的闸:{'子 agent' if agent_type else '主 agent'} 照抄 ⑳ 的出路喂 project-code-guard.sh 放行（退出码 0）"
        manual_guard_script = os.path.join(hook_dir, "project-code-guard.sh")
        if not os.path.isfile(manual_guard_script):
            howto_not_run.append(f"{manual_howto_label}（hook 旁边没有 project-code-guard.sh）")
            continue
        manual_howto_exit = None   # 拒绝说明里取不出出路也判红
        if manual_howto:
            manual_guard_payload = {"tool_name": "Bash", "session_id": "s", "cwd": manual_copy_root,
                                    "tool_input": {"command": manual_howto.group(1).replace(MANUAL_COPY_HOWTO_COPY_PLACEHOLDER, os.path.join(copy_scratch, "howto-copy")),
                                                   "run_in_background": False}}
            if agent_type:
                manual_guard_payload["agent_type"] = agent_type
            manual_howto_exit = subprocess.run(["bash", manual_guard_script], capture_output=True, text=True, input=json.dumps(manual_guard_payload),
                                               env=howto_guard_environment(os.environ, os.path.join(work, "manual-copy-howto-detections.jsonl"))).returncode
        results.append((manual_howto_label, 0, manual_howto_exit))
    manual_green = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                  env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "manual-copy-detections.jsonl")),
                                  input=json.dumps({"tool_name": "Bash", "session_id": "s", "cwd": manual_copy_root, "tool_input": {
                                      "command": f"python3 scripts/hosts/minimal-copy.py {manual_copy_root} {copy_scratch}/copy --with-test-inputs",
                                      "run_in_background": False}}))
    results.append(("stdin:主 agent 起 minimal-copy.py 建副本放行（退出码 0）", 0, manual_green.returncode))
    # ㉓ 走真实入口（这一份钩子、它所在的仓根）：主 agent 与子 agent 经 submit 交 cargo build 点了全部目标各拒一格，拒绝说明给三种 cargo 写法；带 --full-gate、点名包的放行。
    # 出路本身要过得了别的闸：三种写法照抄（占位照 WHOLE_TARGET_HOWTO_SAMPLES 换成样本，cargo test 那一条前面加 target 种类的范围声明）拼成 submit 的整条，
    # 主 agent 喂这一份钩子、同挂 PreToolUse[Bash] 的 project-code-guard.sh 与 run-scope-guard.sh，都要放行；hook 旁边没有那一份的格报本次未跑
    def whole_target_entry(command, agent_type=None):
        entry_payload = {"tool_name": "Bash", "session_id": "s", "cwd": whole_target_root, "tool_input": {"command": command, "run_in_background": False}}
        if agent_type:
            entry_payload["agent_type"] = agent_type
        return subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True, input=json.dumps(entry_payload),
                              env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "whole-target-detections.jsonl")))
    whole_target_main = whole_target_entry("task-server/task submit -- cargo build -p singlefs-verification --offline --all-targets")
    whole_target_howtos = WHOLE_TARGET_HOWTO_COMMAND.findall(whole_target_main.stderr)
    results.append(("stdin:主 agent 经 submit 交 cargo build --all-targets 拒（退出码 2）、出路给三种 cargo 写法", (2, ["build", "clippy", "test"]),
                    (whole_target_main.returncode, sorted(howto.split()[1] for howto in whole_target_howtos))))
    whole_target_sub = whole_target_entry("task-server/task submit -- cargo build -p singlefs-verification --tests", "implementation-writer")
    results.append(("stdin:子 agent 经 submit 交 cargo build --tests 拒（退出码 2）、出路写进报告交回", (2, True),
                    (whole_target_sub.returncode, "要主 agent 跑的命令" in whole_target_sub.stderr)))
    results.append(("stdin:主 agent 带 --full-gate 交 cargo build --all-targets 放行（退出码 0）", 0,
                    whole_target_entry("task-server/task submit --full-gate -- cargo build --workspace --all-targets").returncode))
    results.append(("stdin:主 agent 经 submit 交 cargo build -p 点名包放行（退出码 0）", 0,
                    whole_target_entry("task-server/task submit -- cargo build -p singlefs-verification --offline").returncode))
    whole_target_forms = {howto.split()[1]: howto for howto in whole_target_howtos}   # clippy、test、build → 出路里的写法
    whole_target_scope = ("SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-format 的测试目标 format_layout 里的 1 条测试; 范围=1 个测试; "
                          "为什么=自证照抄 ㉓ 出路里点名目标的写法喂范围声明闸，核它放行这一种写法' ")
    for form in ("clippy", "test", "build"):
        whole_target_howto = whole_target_forms.get(form)
        if whole_target_howto is not None:
            for placeholder, sample in WHOLE_TARGET_HOWTO_SAMPLES:
                whole_target_howto = whole_target_howto.replace(placeholder, sample)
            whole_target_howto = (whole_target_scope if form == "test" else "") + f"task-server/task submit -- {whole_target_howto}"
        for guard_name in ("bash-command-detector.sh", "project-code-guard.sh", "run-scope-guard.sh"):
            whole_target_label = f"出路过别的闸:主 agent 照抄 ㉓ 出路的 cargo {form} 写法喂 {guard_name} 放行（退出码 0）"
            whole_target_guard = os.path.join(hook_dir, guard_name)
            if not os.path.isfile(whole_target_guard):
                howto_not_run.append(f"{whole_target_label}（hook 旁边没有 {guard_name}）")
                continue
            whole_target_exit = None   # 拒绝说明里取不出这一种写法也判红
            if whole_target_howto is not None:
                whole_target_exit = subprocess.run(
                    ["bash", whole_target_guard], capture_output=True, text=True,
                    input=json.dumps({"tool_name": "Bash", "session_id": "s", "cwd": whole_target_root,
                                      "tool_input": {"command": whole_target_howto, "run_in_background": False}}),
                    env=howto_guard_environment(os.environ, os.path.join(work, "whole-target-howto-detections.jsonl"))).returncode
            results.append((whole_target_label, 0, whole_target_exit))
    # ㉓ 出路里 cargo clippy 写法的 lint 表：拒绝说明带 check.sh 现在的整段（弄坏开关 whole-target-howto-clippy-lints-dropped 下红）；
    # 表的读法逐格验在样本 check.sh 上（共用模块的 LIB_CODE_DISCIPLINE_LINTS_BREAK=entry-form-unchecked、other-assignments-unscanned 下读不成的那几格红）；(说明, check.sh 的正文或 None 当文件不在, 该交的整段, 该记几条判不了)
    clippy_table_path = os.path.join(repository_root_of(hook_dir), CLIPPY_LINT_TABLE_RELATIVE)
    notices_before_lints = len(JUDGING_NOTICES)
    clippy_table_expected = clippy_lint_tail(clippy_table_path)
    del JUDGING_NOTICES[notices_before_lints:]
    if clippy_table_expected:
        clippy_way = whole_target_build_messages([("cargo build --all-targets", ["--all-targets"])])[1]
        del JUDGING_NOTICES[notices_before_lints:]
        results.append(("㉓ 出路:cargo clippy 写法的 -- 之后带 check.sh 现在的 CODE_DISCIPLINE_LINTS 整段（-D warnings 加每一条 -D clippy::<名>）", True,
                        f"cargo clippy -p <包> --all-targets {clippy_table_expected}" in clippy_way))
    else:
        howto_not_run.append("㉓ 出路:cargo clippy 写法的 -- 之后带 check.sh 现在的 CODE_DISCIPLINE_LINTS 整段（hook 旁边的仓里读不到 check.sh 的那一段）")
    clippy_sample_text = ("#!/usr/bin/env bash\n  -D clippy::outside_before   # 表之前的，不算\nCODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint  # 第一条\n"
                          "  -D clippy::beta_lint\n)\n  -D clippy::outside_after   # 表之后的，不算\n")
    clippy_sample_path = os.path.join(work, "clippy-lint-table-sample.sh")
    clippy_empty_path = os.path.join(work, "clippy-lint-table-empty.sh")
    for sample_path, sample_text in ((clippy_sample_path, clippy_sample_text), (clippy_empty_path, "#!/usr/bin/env bash\necho no table\n")):
        with open(sample_path, "w", encoding="utf-8") as sample_handle:
            sample_handle.write(sample_text)
    for label, table_file, want_tail, want_notices in (
            ("㉓ 出路的 lint 表:只读 CODE_DISCIPLINE_LINTS=( … ) 那一段，表前表后的 -D clippy:: 行不算",
             clippy_sample_path, "-- -D warnings -D clippy::alpha_lint -D clippy::beta_lint", 0),
            ("㉓ 出路的 lint 表:文件里没有那一段，交空串并记一条判不了", clippy_empty_path, "", 1),
            ("㉓ 出路的 lint 表:文件不在，交空串并记一条判不了", os.path.join(work, "clippy-lint-table-absent.sh"), "", 1)):
        notices_before_lints = len(JUDGING_NOTICES)
        got_tail = clippy_lint_tail(table_file)
        results.append((label, (want_tail, want_notices), (got_tail, len(JUDGING_NOTICES) - notices_before_lints)))
        del JUDGING_NOTICES[notices_before_lints:]
    # 表只认一种写法（lib_code_discipline_lints.py 文件头）：读得成的，出路里的整段必须与整轮门禁真传给 clippy 的那张数组逐个词相同（「门禁真传的」那张数组是拿真 bash 对每一种写法
    # source 之后 printf 出 ${CODE_DISCIPLINE_LINTS[@]} 现算出来、照抄在下面的字面量；自证里不起 bash：source 一个参数给的路径，算键的库认不出读哪一份，整份钩子的键算不出）；
    # 读不成的（bash 读得懂、门禁照样传给 clippy，这里读到的会与门禁真传的对不上）交空串并记一条判不了
    clippy_form_truth = {
        "照今天的写法（每行 -D 加名字，带说明）": ["-D", "clippy::alpha_lint", "-D", "clippy::beta_lint"],
        "开头一行前面带 readonly": ["-D", "clippy::alpha_lint", "-D", "clippy::beta_lint"],
        "开头一行前面带 declare -a": ["-D", "clippy::alpha_lint"],
        "收尾的 ) 缩进": ["-D", "clippy::alpha_lint"],
        "表里有空行与整行注释": ["-D", "clippy::alpha_lint"],
        "别处用 ${CODE_DISCIPLINE_LINTS[@]} 与 ${#CODE_DISCIPLINE_LINTS[@]} 读它": ["-D", "clippy::alpha_lint"],
    }
    for label, table_text in (
            ("照今天的写法（每行 -D 加名字，带说明）", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint  # 第一条\n  -D clippy::beta_lint\n)\n"),
            ("开头一行前面带 readonly", "readonly CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n  -D clippy::beta_lint\n)\n"),
            ("开头一行前面带 declare -a", "declare -a CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n)\n"),
            ("收尾的 ) 缩进", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n  )\n"),
            ("表里有空行与整行注释", "CODE_DISCIPLINE_LINTS=(\n  # 说明\n\n  -D clippy::alpha_lint\n)\n"),
            ("别处用 ${CODE_DISCIPLINE_LINTS[@]} 与 ${#CODE_DISCIPLINE_LINTS[@]} 读它",
             "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n)\ncargo clippy -- -D warnings \"${CODE_DISCIPLINE_LINTS[@]}\"\necho \"${#CODE_DISCIPLINE_LINTS[@]}\"\n")):
        form_path = os.path.join(work, "clippy-lint-table-form.sh")
        with open(form_path, "w", encoding="utf-8") as form_handle:
            form_handle.write(table_text)
        truth = clippy_form_truth[label]
        notices_before_lints = len(JUDGING_NOTICES)
        got_words = shlex.split(clippy_lint_tail(form_path))
        del JUDGING_NOTICES[notices_before_lints:]
        results.append((f"㉓ 出路的 lint 表:{label}，出路与门禁真传给 clippy 的那张表逐个词相同", ["--", "-D", "warnings", *truth], got_words))
    # 读不成的写法（弄坏开关 LIB_CODE_DISCIPLINE_LINTS_BREAK=entry-form-unchecked 下前九格红，other-assignments-unscanned 下后三格红）
    for label, table_text in (
            ("第一条写在开头那一行", "CODE_DISCIPLINE_LINTS=( -D clippy::alpha_lint\n  -D clippy::beta_lint\n)\n"),
            ("一行写两条", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint -D clippy::beta_lint\n  -D clippy::gamma_lint\n)\n"),
            ("-D 与名字贴着写", "CODE_DISCIPLINE_LINTS=(\n  -Dclippy::alpha_lint\n  -D clippy::beta_lint\n)\n"),
            ("--deny 写法", "CODE_DISCIPLINE_LINTS=(\n  --deny clippy::alpha_lint\n  -D clippy::beta_lint\n)\n"),
            ("加了一条不是 clippy 的 rustc lint", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n  -D unsafe_code\n)\n"),
            ("引号里写的 -D 与名字", "CODE_DISCIPLINE_LINTS=(\n  \"-D\" \"clippy::alpha_lint\"\n  '-D' clippy::beta_lint\n)\n"),
            ("花括号", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::cast_{possible_truncation,sign_loss}\n)\n"),
            ("数组元素", "BASE_LINTS=( -D clippy::alpha_lint )\nCODE_DISCIPLINE_LINTS=(\n  \"${BASE_LINTS[@]}\"\n  -D clippy::beta_lint\n)\n"),
            ("-F 禁止", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n  -F clippy::beta_lint\n)\n"),
            ("收尾的 ) 贴在最后一个名字上", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint -D clippy::beta_lint)\n"),
            ("行末反斜杠接下一行", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint \\\n  -D clippy::beta_lint\n)\n"),
            ("没有收尾的 )", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n"),
            ("+= 追加", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n)\nCODE_DISCIPLINE_LINTS+=( -D clippy::beta_lint )\n"),
            ("重新赋值", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n)\nCODE_DISCIPLINE_LINTS=( -D clippy::beta_lint )\n"),
            ("元素赋值", "CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n)\nCODE_DISCIPLINE_LINTS[1]=-A\n")):
        form_path = os.path.join(work, "clippy-lint-table-unreadable.sh")
        with open(form_path, "w", encoding="utf-8") as form_handle:
            form_handle.write(table_text)
        notices_before_lints = len(JUDGING_NOTICES)
        got_tail = clippy_lint_tail(form_path)
        results.append((f"㉓ 出路的 lint 表:{label}，读不成：交空串并记一条判不了", ("", 1), (got_tail, len(JUDGING_NOTICES) - notices_before_lints)))
        del JUDGING_NOTICES[notices_before_lints:]
    sample_environment = howto_guard_environment({"LIB_SHELL_WORDS_BREAK": "x", "PROJECT_CODE_GUARD_BREAK": "y", "PATH": "/sample"}, "/sample/detections.jsonl")
    results.append(("出路过别的闸:喂 project-code-guard.sh 的环境里不带弄坏开关变量，别的变量照带", (False, False, True, True),
                    ("LIB_SHELL_WORDS_BREAK" in sample_environment, "PROJECT_CODE_GUARD_BREAK" in sample_environment, sample_environment.get("PATH") == "/sample",
                     sample_environment.get("AGENT_HOOK_DETECTIONS") == "/sample/detections.jsonl")))
    results.append(("stdin:只记的检出记录写了结局「只记」", "只记", last_outcome))
    after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    # 走真实入口：hook 旁边没有共用切词模块时照常放行（退出码 0，前台起看门狗也不拦），stderr 点名那个模块，并记一条检出
    lonely_hook_directory = os.path.join(work, "hook-without-shared-words")
    os.makedirs(lonely_hook_directory)
    for lonely_name in ("bash-command-detector.sh",) + DETECTOR_PART_NAMES:   # 没有共用切词模块的钩子目录：入口与 python 各部分在，lib_shell_words*.py 不在
        shutil.copy(os.path.join(hook_dir, lonely_name), lonely_hook_directory)
    without_library = subprocess.run(["bash", os.path.join(lonely_hook_directory, "bash-command-detector.sh")], capture_output=True, text=True,
                                     env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections),
                                     input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "bash scripts/agents/watch.sh a1 > /dev/null 2>&1 &"}}))
    recorded_after = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
    results.append(("stdin:读不到共用切词模块照常放行（退出码 0）", 0, without_library.returncode))
    results.append(("stdin:读不到共用切词模块时 stderr 点名它、记一条检出", 1,
                    int("lib_shell_words.py" in without_library.stderr and recorded_after == after + 1)))
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 findings_for()、record()、watchdog_rejection()、wait_loop_refusal() / unbounded_wait_loops()、detaching_refusal() / detaching_forms()、"
              "unwaited_ampersand_refusal() / unwaited_background_jobs() / jobs_with_endings()、"
              "results_overwrite_refusal() / results_overwrite_verdict() / tracked_verdicts() / overwrite_steps() / follow_simple_command() / destination_words() 与共用的 lib_shell_words.py；"
              "「覆盖产物:临时 git 仓建得起来」红的是本机的 git 起不来，先看 git init 能不能跑；"
              "BASH_COMMAND_DETECTOR_BREAK 设着的话（项与红的格见文件头）这里本来就该红；"
              "「终止进程:」「扫脚本:」红的看 process_signal_refusal() / ProcessTableOnDemand / termination_findings() / call_findings() / scan_scripts()；"
              "「就地改脚本:」红的看 script_in_place_refusal() / script_in_place_verdict() / python_in_place_writes() / is_existing_script()；"
              "「先编后换:」红的看 compile_first_refusal() / compile_first_verdict() / overwrite_steps(every_write=True)，BASH_COMMAND_DETECTOR_BREAK=allow-compile-first-bypass 设着的话本来就该红；"
              "「服务源码:」红的看 service_source_refusal() / ServiceSourceView / service_directory_copy_hits() / bash_scope_verdict() 的 service_view 一支 / follow_simple_command() 的 service_writes 几支，"
              "BASH_COMMAND_DETECTOR_BREAK=allow-service-source-write、service-source-overwrite-only、service-source-lexical-only、service-source-backup-kept、service-source-directory-copy-unseen、"
              "service-source-link-unseen、service-source-hardlink-inode-unseen、service-source-extract-unseen、service-source-removal-unseen 设着的话本来就该红；"
              "「通删工作根:」红的看 work_root_removal_refusal() / recursive_removal_operands() / removal_target_path() / removal_hits_work_root()，BASH_COMMAND_DETECTOR_BREAK=allow-work-root-removal 设着的话本来就该红；"
              "「两道包装:」红的看 wrapped_commands() / two_logs_inner_words() / project_code_runs_library() 与 lib_project_code_runs.py 的 command_under_two_logs()，BASH_COMMAND_DETECTOR_BREAK=opaque-two-logs 设着的话本来就该红；"
              "「Bash写范围:」红的看 bash_scope_refusal() / bash_scope_verdict() / bind_loop_variables() / loop_value_choices() / home_expanded() / known_write_script_steps() / agent_scope_patterns() / writer_lock_refusal() / other_session_owner() / leg_round_problem() / leg_report_base() 与 lib_write_scope.py，"
              "mktemp_directory_value() / mktemp_created_directory() / mktemp_option_directory() / mktemp_inside_pwd() / mktemp_template_directory() / own_scratch_directory() 与 lib_writer_lock.py / remember_working_directory() / piped_output_bodies() / command_pieces()，"
              "BASH_COMMAND_DETECTOR_BREAK 的 allow-bash-scope-write、scope-known-scripts-unseen、scope-main-agent-judged、scope-undeterminable-only-noted、scope-loop-values-unknown、scope-loop-landings-trusted、scope-home-unexpanded、scope-landing-lexical-only、heredoc-body-judged-as-commands、"
              "heredoc-pipe-fed-as-data、pwd-variable-unexpanded、mktemp-directory-unknown、mktemp-scratch-template-unknown、mktemp-template-owner-unchecked、mktemp-option-directory-unknown、mktemp-nested-substitution-unknown 设着的话本来就该红；"
              "「平常跑设了弄坏开关」红的看 refuse_break_outside_selftest()，「扫脚本:待改清单里的行所在的目录这一趟没扫」红的看 scan_scripts()，break-honored-outside-selftest、pending-outside-scanned-as-stale 设着的话本来就该红；"
              "「就地改仓内文件:」红的看 repository_in_place_edit_verdict() / sed_target_resolved_in_scratch() / embedded_python_codes() / python_repository_write_pattern() / "
              "perl_in_place_files() / python_script_bodies() / python_unliteral_write_targets()，"
              "sed-in-place-ignores-cd、python-move-into-repository-unseen 设着的话本来就该红；"
              "「起 claude:」与 stdin 里起 claude 那几格红的看 claude_launch_refusal() / claude_launches()，allow-subagent-claude-launch 设着的话本来就该红；"
              "「前台超时:」与 stdin 里 timeout 那几格红的看 foreground_timeout_refusal() / foreground_timeout_messages() 与 main() 里 ⑩ 之后的那一段，allow-long-foreground-timeout 设着的话本来就该红；"
              "「提交前钩子:」与 stdin 里 git commit、git push 那几格红的看 git_hook_refusal() / git_hook_findings() / git_hook_bypass_reasons() / git_config_hooks_path_write() / git_config_core_section_change() / hooks_path_is_installed_value() / "
              "git_hook_form_judged() / git_hooks_not_installed() / git_option_tokens() / resolved_git_subcommand() / expand_long_option() / git_command_steps() / git_hook_refusal_lines() "
              "与 lib_git_hooks.py 的 hooks_path_problems()，"
              "allow-git-hook-guard、bypass-flag-unseen、bypass-config-unseen、merge-bypass-unseen、am-bypass-unseen、config-section-unseen、hooks-path-right-value-refused、hooks-path-relative-accepted、"
              "hooks-not-installed-allowed、bypass-every-git-command、alias-unresolved、dynamic-launcher-unseen、"
              "long-option-prefix-unseen、git-hook-messages-unguarded 设着的话本来就该红；"
              "「全部目标构建:」、stdin 里 submit 交 cargo build 那几格与 ㉓ 出路过别的闸那几格红的看 whole_target_build_refusal() / submit_options_have_full_gate() / whole_target_build_messages() / "
              "workspace_cargo_lock_library() 与 lib_project_code_runs.py 的 task_server_submission、lib_workspace_cargo_lock.py 的 cargo_invocations，"
              "allow-whole-target-build、whole-target-full-gate-unseen、whole-target-howto-unnamed-test 设着的话本来就该红；"
              "「㉓ 出路」那几格（cargo clippy 写法的 -- 之后带不带 lint 表、lint 表的读法）红的看 clippy_lint_tail() / whole_target_build_messages() 的 clippy 写法，"
              "whole-target-howto-clippy-lints-dropped 与 LIB_CODE_DISCIPLINE_LINTS_BREAK=entry-form-unchecked、other-assignments-unscanned 设着的话本来就该红；"
              "「Bash写范围:」里脚本路径在变量里的那几格红的看 known_write_script_steps() / unresolved_script_word() 与 bash_scope_verdict() 的 UnresolvedScriptTarget 一支，"
              "scope-script-variable-unexpanded 设着的话本来就该红；"
              "「Bash写范围:」里没有项目定义的 agent 那几格红的看 agent_scope_patterns() / bash_scope_refusal() 的 builtin 一支，builtin-agents-unjudged 设着的话本来就该红；"
              "「Bash写范围:只读定义:」那几格红的看 agent_scope_patterns() 读到的写范围表行（.claude/hooks/agent-write-scope.tsv 里只带 Read、Bash 的八类定义的行）与 definition_can_write()，"
              "scope-read-only-definitions-unjudged 设着的话本来就该红；「Bash写范围:只读定义:写者锁:」那几格红的看 writer_lock_refusal()，scope-writer-lock-unchecked 设着的话本来就该红；"
              "「弄坏开关三处一致」那几格红的看 break_switch_name_sets() / break_switch_set_mismatches() 与点名的名字缺在哪一处（KNOWN_DETECTOR_BREAK_ITEMS、文件头开关表、detector_break_is_set 的读法），"
              "break-switch-sets-one-way、break-switch-indirect-reads-unseen、break-switch-table-narrow 设着的话本来就该红；"
              "「后台:」与 stdin 里子 agent run_in_background 那几格红的看 subagent_background_refusal()，"
              "allow-subagent-background 设着的话本来就该红；「清整个索引:」与 stdin 里 git reset 那两格红的看 whole_index_reset_refusal() / git_reset_clears_index() / "
              "git_restore_clears_index() / staged_index_state()，allow-whole-index-reset、whole-index-staged-unchecked、whole-index-pathspec-ignored 设着的话本来就该红；"
              "「关守卫项与别的项同写」红的看 refuse_break_outside_selftest()，break-honored-outside-selftest-with-others 设着的话本来就该红；"
              "「往现行账本追加:」与 stdin 里往现行账本追加节标题那两格红的看 ledger_append_refusal() / current_ledger_relative() / lands_on_ledger() / overwrite_steps(every_write=True)，"
              "allow-ledger-append 设着的话本来就该红；"
              "「字面写入:」与 stdin 里 Bash heredoc 往仓里追加那两格红的看 literal_notation_hits() / literal_writes() / literal_output() / printf_rendered() / literal_write_landing() "
              "与 ../gate.d/lib-forbidden-notations.py，allow-literal-notations、literal-notations-outside-judged 与库的 LIB_FORBIDDEN_NOTATIONS_BREAK 设着的话本来就该红；"
              "「撤工作区改动:」与 stdin 里撤工作区改动那两格红的看 worktree_discard_refusal() / worktree_discard_paths() / unstaged_hits()，"
              "allow-path-discard、path-discard-changes-unchecked、path-discard-staged-counted 设着的话本来就该红；"
              "「Bash写范围:」里 ledger-append-section.py 那几格红的看 known_write_script_steps() / last_option_value() / ledger_append_root() 与 bash_scope_verdict() 的 LedgerAppendTarget 一支，"
              "scope-ledger-append-unseen 设着的话本来就该红；"
              "「Bash写范围:」里 append-record.py 那几格红的看 known_write_script_steps() 的 RECORD_APPEND_SCRIPT 一支与 first_operand()，"
              "scope-record-append-unseen、scope-record-append-root-as-target 设着的话本来就该红；"
              "「Bash写范围:」里 insert-row.py --absent-literal 写在文件前面那一格红的看 known_write_script_steps() 的 insert_row_options 与 INSERT_ROW_OPTIONS_WITH_VALUE，"
              "scope-insert-row-absent-literal-as-target 设着的话本来就该红；"
              "「往 records 追加:」与 stdin 里往 records 追加那两格红的看 record_append_refusal() / record_append_landing() / overwrite_steps(every_write=True) 的 tee 一支与 python_in_place_writes()，"
              "allow-record-append、record-append-python-unseen、record-append-tee-unseen 设着的话本来就该红；"
              "「手拷仓副本:」与 stdin 里手拷 crates、⑳ 出路过别的闸那几格红的看 manual_repository_copy_refusal() / copied_sources() / rsync_copied_sources() / copy_arguments_recursive() / copied_source_hits() / manual_copy_howto()，allow-manual-repository-copy、manual-copy-howto-unlisted-tool 设着的话本来就该红")
        return 1
    summary = ("没超时的等待循环、run_in_background 里又自己放后台记进检出记录，普通命令、重定向里的 & 、前台的 & 、写进文件的 heredoc 正文与按模式找进程（归上游钩子）不记；"
               "起看门狗不用 run_in_background、或带 &、nohup、setsid、disown、丢进 /dev/null 的拒绝（退出码 2），run_in_background 只写 watch.sh 的、把名字当参数的、"
               "不盯的模式与 agent-watch.py report 放行；前台没超时的等待循环拒绝（退出码 2、不记检出），外层套了 timeout 的、run_in_background 里的（只记检出）、"
               "引号与注释里当数据写的、不带 sleep 的与 for 循环放行；命令位置上的 disown、coproc、setsid -f、nohup … &、tmux / screen 分离、"
               "不等结束的 systemd-run 前台后台都拒，不带 & 的 nohup、裸 setsid、systemd-run --wait、并行加 wait 与当数据写的放行；"
               "run_in_background 里以单独的 & 收尾、之后同一层同一对圆括号里没有 wait 的拒绝（结尾 &、& 后接 echo $!、capped.sh 包着的、"
               "子 shell、命令替换、bash -c 与喂给 shell 的 heredoc 里的，wait -n 与按 pid 轮询不算 wait），之后用 wait / wait \"$pid\" 等的、"
               "&&、2>&1、|&、&>、当数据写的与前台的放行；在临时 git 仓里，整份覆盖 research/results/ 下已存在又没进 git 的文件拒绝（>、>|、&>、2>、>& 文件、"
               "不带 -a 的 tee、cp / mv / install 的目标与 -t、写进已有目录、dd of=（带不带 conv=notrunc 都算）、truncate 的文件（与 ⑦ 共用同一段 overwrite_steps），"
               "bash -c、heredoc、命令替换、进程替换、整组重定向里的，cd 跟着算、cd 算不出来按仓库根、"
               "前面赋过值的变量代入），>>、&>>、tee -a、新文件名、已跟踪、仓外与 research/results/ 外、2>&1、cp -n / --backup、install -d、同一条命令里先 git add 或 mv 挪走的、"
               "当数据写的放行；目标算不出而对得上未跟踪产物的、git 判不了的只记检出；"
               "终止进程：kill 负进程号、0、$PPID、一次几个目标、命令替换与整批展开、循环里逐个 kill 或 proc.py stop、xargs / find -exec 喂给 kill、pkill、killall、fuser -k、"
               "先从 cgroup.procs 或 pgrep 挑再发、python 里 os.killpg 与循环里 os.kill、systemctl 停 SSH / 会话 / user@ / user.slice / 私有配置登记的本地模型服务单元或带通配、isolate 与 reboot、"
               "loginctl terminate / kill、写 cgroup.kill、写死的进程号是 sshd、VSCode、自己的 Claude、别的 Claude 会话或它起的、私有配置登记的本地模型服务、pid 1 的都拒（退出码 2、不记检出），"
               "私有配置不在或没有那两个键时本地模型服务那一项放行、别的照拒，"
               "kill \"$!\"、kill %1、kill -0、kill -l、这个会话起的与没人认领的写死进程号、proc.py stop 一个、自己 slice 的 systemctl stop、只读 cgroup.procs 与当数据写的放行；"
               "自己用 setsid 或 start_new_session=True 起的子进程的进程组放行，不是这样起的、改绑过的照拒；"
               "扫脚本：进程组、循环 kill、没核 scope 的 own-scope 标注、给进程组标的与不起作用的标注、python 循环 os.kill 与 os.killpg 按行号报红，"
               "自己用新会话起的子进程的组不报，参数、属性、改绑过的、没开新会话的、setsid --fork 的、os.getpgid(0)、cgroup.procs 读来的组按行号报红，"
               "核过 singlefs-memory-cap-*.scope 又标了的放行，待改清单里的逐个列名、清单里干净的与不存在的判红；"
               "就地改脚本：在临时仓里，已有的 .sh、.py、带执行位的与 scratch 根下的脚本被 >、>|、&>、2>、>& 文件、cat >、不带 -a 的 tee、cp（-f、-a、写进目录、-t）、"
               "dd of=、truncate、python 的 open(…, 'w' / 'r+' / mode='wb')、Path.open('w')、write_text、write_bytes、shutil.copyfile / copy 写的拒绝"
               "（bash -c、heredoc、命令替换、cd 与变量跟着算，python 的只赋过一次的名字、Path / 、os.path.join 算得出），"
               ">>、&>>、tee -a、python 'a' / 'x' / 读、新文件名、临时文件再 mv、mv、install、cp --remove-destination / -l / -n / -b、sed -i、先 rm 或挪走再写、"
               "os.replace、非脚本文件、仓外与 scratch 根外、当数据写的与算不出的放行；dd、truncate 现在 ⑤ 也认（同一段 overwrite_steps），⑤ 仍不认 python 的写法；"
               "先编后换：experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs（cp、cat >、>>、tee -a、mv 进目录、cd 之后 cp 到 .、sed -i、perl -pi、rsync、rm、mv 挪走、"
               "bash -c、变量里的目录、算不出而对得上已有装置的、python shutil.copy / open 'a' / os.replace）拒绝并点名 compile-then-swap.py，"
               "经那条脚本换进来、拷进草稿目录、改草稿目录、追加 crates/mutations.tsv、只读、当数据写的与 implementation-writer、主 agent 放行；"
               "通删工作根：rm -r 作用在工作根本身、它的上级或工作根下第一层带通配上的（~、$HOME、$SINGLEFS_WORK_ROOT、export 的变量、cd 之后的 *、bash -c、喂给 bash 的 heredoc、前缀、-fr / -r -f / --recursive / 选项写在后面、会话临时目录根 /tmp/claude-<uid>/*、/tmp/*）拒绝并点名 work-directory-ownership.sh 的 sweep 与 claim，工作根下具体目录与它里面的通配、同名前缀的兄弟目录、别处、不带 -r、当数据写的与算不出的变量放行；"
               "两道包装：scripts/hosts/two-logs.sh <名> -- 包着的前台没超时的等待循环、kill 负进程号、setsid -f、cp 盖入库装置照拒，--from-log、别处的同名拷贝与包着的普通命令放行；"
               "Bash 写范围（⑫）：有写范围的项目子 agent（clerk、kb-scribe）用 replace-once.py / insert-row.py / replace-batch.py（规格里的 file）、sed -i、>、>>、tee、cp / mv、python 的 open(…, 'w') "
               "（bash -c、heredoc、cd、变量、two-logs.sh 包着的都跟着算）写进本仓、又落在 agent-write-scope.tsv 里它那几行之外的拒绝（退出码 2，拒绝说明点名写范围表与交回主 agent 的出路），"
               "范围里的、工作根下的草稿目录、仓外的、--dry-run 与 --selftest、只读、当数据写的、主 agent、没有项目定义的 agent 与表里没有行的定义放行；落点算不出的拒（命令替换、没赋过值或来源不明的变量、while read 与 for $列表 的循环变量、通配、cd 算不出之后的相对路径、读不到规格的 replace-batch.py，静态前缀在仓外也拒；两条检出原文 $(echo $f | tr / _) 拒，拒绝说明点名 copy-originals.py），算得出的照算：for 写死的词逐个取值（取值里爬回仓里的 .. 照判）、前面赋的写死值、cd 之后的 $PWD、mktemp -d、$HOME，落点与写闸同一份判法逐段解开符号链接（范围内的链接指到仓里范围外的照拒，范围外的链接指回范围内的放行，解开之后落在仓外的不判）；"
               "heredoc 正文只在喂给 bash 或 python 时判（cat、tee 写草稿脚本的正文里的 sed -i、重定向、cp、open 是数据，它们的目标照判；⑧ 的 python 判法同样只看 python -c 与喂给 python 的正文；"
               "经管道喂给 python 或 bash 的也是代码：`cat <<'EOF' | python3 -`、`| bash`，echo / printf 的文字同理，喂给 grep 的是数据）；"
               "⑧ 的 sed -i 相对路径从 cd 之后的目录算（cd 到草稿目录放行，cd 到仓内或算不出的照拒）；⑫ 里 cd 到写死的目录之后 $PWD 按那个目录展开，mktemp -d 赋的变量落在 ${TMPDIR:-/tmp} 下（仓外），模板在自己认领的草稿目录里的按草稿目录内算（别的子 agent 的、别的会话的、/tmp 下的、没有 agent_id 的照记）；"
               "进程表只在判到 kill / proc.py stop 的写死的进程号时读、一次（没有终止词的带数字命令不读），一条命令里几个确定的产物目标一次 git ls-files 查完；"
               "⑧ 另认 python 的 os.replace / os.rename / shutil.move 挪进仓里（scripts/ 起头的同算仓内），目标在草稿目录的放行；"
               "⑫ 的脚本词先代入前面赋过的值（脚本路径放在变量里的 replace-once.py / insert-row.py / replace-batch.py 照认），python3 起的脚本词算不出而第一个参数落在仓里的照算不出拒；"
               "⑬ 子 agent 命令位置上起 claude、\"$CLAUDE_CODE_EXECPATH\"、以 /claude 结尾的可执行文件拒绝（bash -c、heredoc、命令替换、前缀里的都认），"
               "command -v claude、当参数与当数据写的、主 agent 放行；"
               "㉑ 前台 timeout 超过 240000 毫秒（主 agent 与子 agent 一样，整数、小数与只含数字的字符串）拒绝（退出码 2），出路主 agent 写 run_in_background 与 task-server/task wait、子 agent 写进报告交回，"
               "正好 240000、没写、写成布尔或不是数的字符串、run_in_background 带 timeout 的（子 agent 的由 ⑩ 拒）放行；"
               "㉒ 真 git 仓里（门限压到 1000 字节）暂存区有超门限文件的 git commit、-a / -am / --all 时工作区改大了的跟踪文件、点名的路径、同一条命令里排在前面的 git add（含 -A、.、-f 与没跟踪的）、"
               "GIT_INDEX_FILE 指到的私有索引里的、没推的提交里有超门限 blob 的 git push 拒绝（-C、cd、bash -c、前缀里的都认，退出码 2，出路逐个文件写 gzip -n -9 -k、git rm --cached、git add .gz，push 写推之前重做那几笔提交），"
               "工作区改大了而没暂存的、点名了小路径的、--help、--dry-run、git log、git status、当参数写的、没跟踪的、git add 排在 git commit 之后的、没推的提交里没有超门限 blob 的放行，"
               "没有上游、不在 git 仓里、判法起不来、目录算不出、--git-dir 的放行并记一条检出；"
               "㉓ 经 task-server/task submit 交的 cargo build（cargo b）带 --all-targets、--tests、--benches、--examples、--workspace、--all 之一的拒绝（范围声明在前、--cwd、-- 之后的 env、bash -c 与 cd、"
               "外层 bash -c、&& 后一段、只带 --whole-package-confirmed 的都认；主 agent 与子 agent 走入口都退 2，出路给 clippy、点名目标的 test、点名包的 build 三种写法，"
               "照抄喂这一份钩子、project-code-guard.sh、run-scope-guard.sh 都放行），带 --full-gate 的、cargo build -p 点名包、cargo clippy --all-targets、合入后验证脚本、cargo test、"
               "没经 submit 直接起的、当数据写的、盘上没有的同名客户端放行；"
               "⑫ 没有项目定义的 agent（内置类型、定义已删的旧实例）只许写工作根：写仓里、仓外别处拒，写工作根、/dev/null 与只读放行；"
               "⑧ 的 python 相对路径从 cd 之后算：cd 进草稿目录里的仓副本再写放行，仓根下与目录算不出的照拒；"
               "⑧ 另认 perl -i（开关簇里的 -i，合写与带备份后缀的都算，-I、-M 簇里的 i 与只读的 perl -ne 放行，目标在 /tmp/ 或工作根下的放行），"
               "与 python3 起的工作根下脚本的正文（按字面量写仓内路径的拒，目标取自 sys.argv 或变量的放行并记一条检出，仓里的脚本不读）；"
               "⑮ 暂存区非空时 git reset（不带路径，HEAD、--mixed、--hard）、git restore --staged . / :/、git -C 与 cd 进去的、目录算不出的拒（主 agent 与子 agent 一样），"
               "带具体路径的、--soft、暂存区为空的、当参数写的放行；关守卫项与别的项同写平常跑与带标记的子进程都拒并点名它；"
               "⑯ 命令里有「## <节号>、」又往拆分提案现行账本写（>、>>、tee -a、cd 之后的相对路径、绝对路径、喂给 python 的 heredoc 里 open(…, 'a')）拒（主 agent 与子 agent 一样），"
               "写别的文件、封存的账本、不带节标题的一行、只读、经 ledger-append-section.py、sed -i 改节号放行；"
               "⑰ heredoc、printf、echo 经管道交 tee -a、cd 之后的相对路径、bash -c、here-string 往假仓里写会话名、钟点、批次代号、任务号、括号里的会话短号、别的会话的批号拒，"
               "记录里的 A/B 读数一行、反引号里的任务号、往末尾开着围栏的文件追加以收尾围栏起头的内容拒，"
               "干净内容、不带括号的两位短号、记录里表格行的批号、判决里行首的 A/B 读数一行、往围栏已收的文件追加与整份写以围栏起头的内容、仓外、排除前缀下、.py 里的批次代号、"
               "目标算不出的、cat 拷文件放行，走入口拒一格放一格；"
               "⑱ 带路径的 git checkout [<提交>] [--]、git restore（不带 --staged 或带 --worktree）、git stash push、git clean 点名的路径里有没暂存的改动（clean 是没跟踪的文件）拒（主 agent 与子 agent 一样），cd、git -C、前面赋过值的变量跟着算，路径或目录算不出的拒；没改动的、只暂存了的、不在仓里的路径、--staged、git clean -n、切分支、当参数写的放行；"
               "⑫ 另认 ledger-append-section.py 写的登记表 current 那一份账本：写范围里没有 records/ 的子 agent 起它（python3 起的、直接执行的）拒，--root 算不出的拒，"
               "写范围有 records/ 的、主 agent、--dry-run、--root= 指到仓外副本的放行；"
               "⑫ 另认 append-record.py 的记录：写范围里没有 records/ 的子 agent 起它（python3 起的、直接执行的、cd 之后相对路径的、--root 写在前面的）拒，记录算不出的拒，"
               "写范围有 records/ 的、主 agent、--dry-run、--selftest、记录在仓外的放行；"
               "⑲ 往假仓 records/ 下的 .md 追加（>>、&>>、tee -a、tee --append、cd 之后的相对路径、绝对路径、前面赋过值的变量、bash -c、python 的 open(…, 'a') 与 Path.open('a')、"
               "子目录、大写后缀、指进 records 的符号链接）拒，经 append-record.py、records 下别的后缀、records 外与仓外、同名前缀的兄弟目录、指到仓外的链接、"
               "> 整份写、python 'w'、只读、目标算不出的放行，走入口主 agent 与子 agent 各拒一格并给出整条写法，"
               "那条写法照抄喂给 project-code-guard.sh 主 agent 与子 agent 都放行（hook 旁边没有它时报本次未跑）；"
               "⑳ rsync、cp -r / -R / -a / --archive 拷假仓的 crates/ 或仓根整个（./、.、绝对路径带结尾的 /、crates/、crates/..、仓根里的 *、crates/*、-t 写在前面、--exclude 带值、-e 带值、cd 之后、前面赋过值的变量、bash -c、nice 前缀、喂给 bash 的 heredoc、几个源里有 crates）拒，minimal-copy.py 本身、拷草稿目录里别的东西、cp 不带递归、rsync -n / --dry-run / 只给一个操作数、拷 crates/ 下单个包或单个文件、当数据写的、算不出的变量、远端源、同名前缀的兄弟目录、-- 之后的 -r 放行，走入口主 agent 与子 agent 各拒一格并给出整条写法、那条写法照抄喂给 project-code-guard.sh 两边都放行，起 minimal-copy.py 本身走入口放行")
    not_run_note = f"；本次未跑 {len(howto_not_run)} 格：{'、'.join(howto_not_run)}" if howto_not_run else ""
    print(f"  ✓ 自检通过（查了 {len(results)} 种）：{summary}{not_run_note}")
    return 0


def main():
    hook_dir = sys.argv[1]
    refused = refuse_break_outside_selftest()
    if refused:
        return refused
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    if len(sys.argv) > 2 and sys.argv[2] == "--scan-scripts":
        if shell_words is None:
            print(f"  ✗ 读不到共用切词模块 {shell_words_library_path(hook_dir)}（{shell_words_error!r}），脚本没扫")
            print("     → 怎么办：恢复 .claude/hooks/lib_shell_words.py（切词与认命令位置只有那一份），再跑门禁 code-tooling 的 research-gate-lint 格")
            return 1
        if len(sys.argv) < 5:
            print("  ✗ --scan-scripts 要仓根与至少一个目录")
            print("     → 怎么办：写成 bash .claude/hooks/bash-command-detector.sh --scan-scripts <仓根> <目录>…（门禁 code-tooling 的 research-gate-lint 格这样调）")
            return 2
        return scan_scripts(sys.argv[3], sys.argv[4:])
    hook_input, unreadable = parsed_hook_input(sys.stdin.buffer.read())
    if unreadable:   # 不是 JSON 对象：照常执行，stderr 打一行可定位的诊断、记一条检出（会话号读不到，照实写读不到）
        print(f"  ! {unreadable}", file=sys.stderr)
        if not detector_break_is_set("failure-unrecorded"):
            try:
                append_detection(UNREADABLE_HOOK_INPUT, [unreadable], os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS)
            except Exception as error:
                print(f"  ! bash-command-detector.sh 没把这一条写进检出记录（{error!r}）", file=sys.stderr)
        return 0
    if shell_words is None:
        warning = f"bash-command-detector.sh 读不到共用切词模块 {shell_words_library_path(hook_dir)}（{shell_words_error!r}），这条命令没判、照常执行"
        print(f"  ! {warning}", file=sys.stderr)
        try:
            append_detection(hook_input, [warning], os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS)
        except Exception:
            pass
        return 0
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    # 拒绝的那几类都判完再报：每类两行（✗ 原因、→ 怎么办），按下面的次序收进来，一类都没有才记检出、放行
    refusal_lines = []
    if subagent_background_refusal(hook_input.get("agent_type"), tool_input.get("run_in_background")):
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 用 run_in_background 起命令：子 agent 一律不起后台任务，task wait 也不起"
              "（用户 2026-11-01 定子 agent 一律不往统一任务服务交任务，records/2026-11-17-subagent账本自上而下清欠.md 第一百七十四节）")
        refusal_lines.append("     → 怎么办：这一条不起。跑项目代码的命令照共用约束 .claude/agent-common.md「不做」一节「子 agent 不交任务」那一条拼成 submit 的整条"
              "（范围声明写在 submit 前面），写进报告「要主 agent 跑的命令」一节交回，标它在 .claude/hooks/selftest-registry.tsv 登记的档；"
              "停在能交接的点，结束本轮交回主 agent，由主 agent 交服务、等结局、新派同类 agent 接着做。只读的短命令照常前台跑")
    try:
        long_timeout = foreground_timeout_refusal(hook_input)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成前台命令的 timeout（{error!r}），这条命令照常执行")
        long_timeout = None
    if long_timeout is not None:
        refusal_lines.extend(foreground_timeout_messages(long_timeout, hook_input.get("agent_type")))
    noted_overwrites = []
    try:
        reasons, calls = watchdog_rejection(tool_input.get("command") or "", tool_input.get("run_in_background"))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成看门狗的起法（{error!r}），这条命令照常执行")
        reasons, calls = [], []
    try:
        signal_problems = process_signal_refusal(tool_input.get("command") or "")
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成终止进程的写法（{error!r}），这条命令照常执行")
        signal_problems = []
    if signal_problems:
        refusal_lines.append(f"  ✗ 终止进程的写法打得到不是自己点名的那一个进程（{'；'.join(signal_problems)}）："
              "后面的命令不许停掉前面的任务，不许动 SSH、VSCode 与 Claude 会话，一次只停点名的一个")
        refusal_lines.append("     → 怎么办：只停自己起的进程：记下它的 pid（`$!`）或任务号，一次停一个，单独 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`"
              "（或 `kill \"$pid\"`、`kill %1`）；不带负号、不给 0、不一次给几个、不在循环里、不按名字或 cgroup 挑。"
              "要停的不是自己起的（别的会话的、SSH、VSCode、系统服务），别停，交主 agent 问用户。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        forms = detaching_refusal(tool_input.get("command") or "")
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成放出追踪的写法（{error!r}），这条命令照常执行")
        forms = []
    if forms:
        refusal_lines.append(f"  ✗ 把活放出了追踪（认出的写法：{'、'.join(forms)}）：进程移出 shell 的作业表或另起会话，随后的 wait 立刻返回、"
              "完成通知当场发出，它跑完不会叫醒任何人，成了没人追踪的孤儿")
        refusal_lines.append("     → 怎么办：要等的活用 Bash 的 run_in_background: true 起，命令里只写那条活本身（不加 `disown`、`coproc`、`setsid -f`、"
              "`nohup … &`、`tmux` / `screen` 的分离模式、`systemd-run`），然后结束本轮，等它的完成通知再接着做；"
              "几条活要并行就在同一条命令里每件把退出码写进自己的文件（`{ <命令>; echo \"$?\" > <草稿目录>/b<批号>-<件号>.rc; } &`，"
              "每批开跑前先删掉这一批的 .rc），最后单独一个不带参数的 `wait` 等齐，数一遍 .rc 与派出去的件数对不上整批作废，"
              "对得上再按派活的次序逐个读（共用约束 .claude/agent-common.md「执行前拒绝的写法」那一条的 ④）；"
              "跑得过后台时限的长活由主 agent 经 `task-server/task submit -- <命令…>` 交给服务（起成脱离会话的单元，会话被停也不停），"
              "会话里只用 run_in_background 起 `task-server/task wait <号>` 等它；子 agent 不交，写进报告「要主 agent 跑的命令」一节交回")
    given_directory = hook_input.get("cwd")
    try:
        claude_calls = claude_launch_refusal(tool_input.get("command") or "", hook_input.get("agent_type"),
                                             given_directory if isinstance(given_directory, str) and os.path.isdir(given_directory) else repository_root_of(hook_dir))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成子 agent 在 Bash 里起 claude 会话的写法（{error!r}），这条命令照常执行")
        claude_calls = []
    if claude_calls:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里起 claude 会话（认出的调用：{'；'.join(claude_calls)}）："
              "子 agent 不派 subagent，也不从 Bash 里起 claude 会话（共用约束 .claude/agent-common.md「派发」一节），这样起的会话不受派发闸与写范围表管")
        refusal_lines.append("     → 怎么办：不起这条命令；要另一个 agent 做的活（连同要它读什么、产出写到哪）写进交回报告，由主 agent 派。"
              "只查 claude 在不在的 `command -v claude` 不拦。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        jobs = unwaited_ampersand_refusal(tool_input.get("command") or "", tool_input.get("run_in_background"))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成 run_in_background 里的单独 &（{error!r}），这条命令照常执行")
        jobs = []
    if jobs:
        refusal_lines.append(f"  ✗ run_in_background 里又把活放到了后台（以单独的 & 收尾、之后同一条命令里没有 wait 的作业：{'；'.join(jobs)}）："
              "外层 shell 起完它就退出，完成通知当场发出，真跑完的那个进程不会叫醒任何人")
        refusal_lines.append("     → 怎么办：长活直接放 run_in_background，命令里只写那条活本身，不加 `&`"
              "；几条活要并行，就每件把退出码写进自己的文件（`{ <命令>; echo \"$?\" > <草稿目录>/b<批号>-<件号>.rc; } &`，"
              "每批开跑前先删掉这一批的 .rc），最后单独一个不带参数的 `wait` 等齐，数一遍 .rc 与派出去的件数对不上整批作废，"
              "对得上再按派活的次序逐个读；不用 `wait \"$pid\"` 收，也不写 `a & b & wait`：它吞掉每件的退出码"
              "（共用约束 .claude/agent-common.md「执行前拒绝的写法」那一条的 ④）；"
              "已经在跑、pid 已知的，另起一条 run_in_background 等它：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    try:
        loops = wait_loop_refusal(tool_input.get("command") or "", tool_input.get("run_in_background"))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成前台的等待循环（{error!r}），这条命令照常执行")
        loops = []
    if loops and tool_input.get("run_in_background") is True:
        refusal_lines.append(f"  ✗ run_in_background 里轮询后台任务的输出文件（认出的循环：{'；'.join(loops)}）：等自己起的后台任务不写轮询")
        refusal_lines.append("     → 怎么办：结束本轮，等那个后台任务的完成通知再接着做（共用约束 .claude/agent-common.md「长活交回主 agent」）；"
              "等的是别人起的进程，按进程号等：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    elif loops:
        refusal_lines.append(f"  ✗ 前台的等待循环没有超时（认出的循环：{'；'.join(loops)}）：等的条件不成立就一直不返回，"
              "这一次调用卡在这里期间，发给你的消息也送不到")
        refusal_lines.append("     → 怎么办，二选一：① 要等的活用 Bash 的 run_in_background: true 起，然后结束本轮，等它的完成通知再接着做；"
              "② 按进程号等：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`（到点还没退出就列出还活着的、退出码 3）。"
              "确实要在前台轮询一个文件，外面套一层 timeout：`timeout <秒> bash -c 'until …; do sleep …; done'`。"
              + ("" if detector_break_is_set("wait-loop-rewrite-unexplained") else
                 "带计数上限、break、`while read`、`kill -0` 探活这类真会结束的循环，这道闸照字面分不出会不会停（正推腿乙报告第五节 Z10），同样照拒：外面套 timeout 就放行，"
                 "等进程退出的改成 `timeout <秒> tail --pid=<pid> -f /dev/null` 或上面的 proc.py wait。")
              + "这一道只拒这种写法，不停你在跑的任何东西")
    try:
        overwrites, noted_overwrites = results_overwrite_refusal(tool_input.get("command") or "", repository_root_of(hook_dir))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成整份覆盖 research/results/ 产物的写法（{error!r}），这条命令照常执行")
        overwrites, noted_overwrites = [], []
    if overwrites:
        refusal_lines.append(f"  ✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：{'；'.join(overwrites)}）："
              "旧字节没有任何一份副本，盖掉就找不回来")
        refusal_lines.append("     → 怎么办：按日期另存一个新文件名，旧的留着；确实要换掉旧产物，先 `git add` 它或者把它挪到带日期的旧名"
              "（`git add` 或 `mv 旧名 带日期的旧名` 可以写在同一条命令里、排在写之前）。"
              "追加用 `>>` 或 `tee -a`，不拦。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        in_place_writes = script_in_place_refusal(tool_input.get("command") or "", repository_root_of(hook_dir))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成在原 inode 上改脚本的写法（{error!r}），这条命令照常执行")
        in_place_writes = []
    if in_place_writes:
        refusal_lines.append(f"  ✗ 要在同一个 inode 上改一个已经存在的脚本（认出的写法与目标：{'；'.join(in_place_writes)}）："
              "正在跑它的 bash 按文件偏移往下读，改完会从新内容的同一偏移接着读，读到的是别的东西")
        refusal_lines.append("     → 怎么办：写到同目录临时文件再 `mv` 换上（`mv` 换 inode，正在跑它的进程读的还是旧内容），"
              "或者用 `scripts/kb/replace-once.py` / `insert-row.py` 定点改。追加（`>>`、`tee -a`）与新建不拦；"
              "Edit / Write 工具本来就换 inode。python 的 `for` 逐个取字面元素时不看循环体里的条件（`if`、`continue`），候选里有一个是已有脚本就拒："
              "要改已有脚本的用 Edit 工具或写临时文件再 `mv`，只写新文件的把已有脚本从候选里拿掉。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        crate_directory = hook_input.get("cwd")
        crate_writes = compile_first_refusal(tool_input.get("command") or "", hook_input.get("agent_type"), repository_root_of(hook_dir),
                                             crate_directory if isinstance(crate_directory, str) and os.path.isdir(crate_directory) else None)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成绕过先编后换的写法（{error!r}），这条命令照常执行")
        crate_writes = []
    if crate_writes:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里写主工作区 crates/ 下的 .rs（认出的写法与目标：{'；'.join(crate_writes)}）："
              "入库装置先在草稿目录的副本里改，编过再整份换进主工作区，主工作区里任何时候都只放编得过的版本")
        swap_launch = (f"跑 `bash scripts/hosts/capped.sh <线程上限> python3 {COMPILE_THEN_SWAP_SCRIPT} " if detector_break_is_set("compile-first-capped-hint")
                       else f"由主 agent 交 `task-server/task submit -- python3 {COMPILE_THEN_SWAP_SCRIPT} ")
        refusal_lines.append(f"     → 怎么办：在草稿目录那一份上改完，{swap_launch}"
              "<草稿副本> crates/<crate>/src/bin/<名>.rs --scratch <草稿目录>`，它在仓副本里编过才整份换进来，编不过一个字节不写；子 agent 不往统一任务服务交任务，"
              "这一条写进报告「要主 agent 跑的命令」一节交回、由主 agent 交；"
              "把主工作区那份拷进草稿目录（`cp crates/… <草稿目录>/`）不拦。目标不是 crates/<crate>/src/bin/ 下的装置的，不归你改，"
              "写进报告交回主 agent。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        edit_directory = hook_input.get("cwd")
        edit_verdict = repository_in_place_edit_verdict(tool_input.get("command") or "", hook_input.get("agent_type"), repository_root_of(hook_dir),
                                                        edit_directory if isinstance(edit_directory, str) and os.path.isdir(edit_directory) else None)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成 Bash 里就地改仓内文件的写法（{error!r}），这条命令照常执行")
        edit_verdict = InPlaceEditVerdict([], [])
    repository_edits = edit_verdict.refused
    if repository_edits:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 有 Edit 工具，却在 Bash 里就地改仓内文件（认出的写法与目标：{'；'.join(repository_edits)}）："
              "写范围闸只看 Write / Edit，sed -i、perl -i、python 代码与草稿目录里的 python 脚本这样写它看不见")
        refusal_lines.append("     → 怎么办：仓内文件用 Edit 改（旧串在文件里恰好命中一次），或用定义点名的脚本（replace-once.py、insert-row.py 这类）；"
              "草稿目录（工作根 ~/.local/share/singlefs/work/ 下）里随便写，草稿里的脚本只改草稿目录里的仓副本（cd 进副本再写相对路径）。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        scope_directory = hook_input.get("cwd")
        scope_verdict = bash_scope_refusal(
            tool_input.get("command") or "", hook_input.get("agent_type"), repository_root_of(hook_dir),
            scope_directory if isinstance(scope_directory, str) and os.path.isdir(scope_directory) else repository_root_of(hook_dir),
            os.path.join(hook_dir, WRITE_SCOPE_TABLE_NAME),
            {key: hook_input.get(key) for key in ("agent_id", "session_id", "transcript_path")})
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成 Bash 里写写范围之外的仓内文件（{error!r}），这条命令照常执行")
        scope_verdict = ScopeVerdict([], [], [], [])
    if scope_verdict.out_of_scope and scope_verdict.builtin:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 没有项目定义（.claude/agents/{hook_input.get('agent_type')}.md 不在：内置类型、插件带来的或定义已删的旧实例），"
              f"只许写工作根，在 Bash 里写到了工作根之外（认出的写法与目标：{'；'.join(scope_verdict.out_of_scope)}）：与写范围闸 write-guard.sh 第二道同一口径")
        refusal_lines.append(f"     → 怎么办：草稿、报告写进派发提示给的草稿目录（工作根 {work_roots.work_root() if work_roots else '~/.local/share/singlefs/work'}/ 下；会话临时目录根 /tmp/claude-<uid>/ 不算）；"
              "仓里与别处的改动不自己做，写进报告交回主 agent，由主 agent 改或派有写范围的项目 agent（一次性的活派 clerk）。"
              "2>/dev/null 这类设备不拦。这一道只拒这种写法，不停你在跑的任何东西")
    elif scope_verdict.out_of_scope:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里写了它写范围之外的仓内文件（认出的写法与目标：{'；'.join(scope_verdict.out_of_scope)}）："
              "写范围闸 write-guard.sh 只看 Write / Edit，Bash 里的写由这一道照同一张表（.claude/hooks/agent-write-scope.tsv）判")
        refusal_lines.append(f"     → 怎么办：范围外的改动不自己做，写进报告交回主 agent，由主 agent 改或派该写的 agent；它的写范围：{'、'.join(scope_verdict.patterns)}"
              "（$SINGLEFS_WORK_ROOT 是工作根 ~/.local/share/singlefs/work；会话临时目录根 /tmp/claude-<uid> 不在里面）；草稿目录里随便写。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    if scope_verdict.undeterminable:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里写的目标这一刻算不出最终落点（{'；'.join(scope_verdict.undeterminable)}）："
              "落点算不出就判不了在不在写范围里（变量的值可以带 `/..` 爬回仓里），这一道照 .claude/hooks/agent-write-scope.tsv 判之前先要算得出落点")
        refusal_lines.append("     → 怎么办：改仓内文件之前留底原文件，用 `python3 scripts/agents/copy-originals.py copy <草稿目录> <仓内文件>…`"
              "（拷到 <草稿目录>/originals/<相对仓根的路径>、记 sha256 清单；之后 `python3 scripts/agents/copy-originals.py check <草稿目录>` 列出改过的），"
              "不手拼 `$(echo $f | tr / _)` 这类目标；按文件写多份输出的，循环写成 `for 名字 in 写死的词…; do … > <草稿目录>/<写死的段>/$名字.out; done`，"
              "目标只用这些循环变量、同一条命令里前面赋过写死值的变量与写死的段，或逐条写字面路径；别的算不出的写法（命令替换、没赋过值或来源不明的变量、"
              "`while read` 读进来的、`for f in $列表`、通配、cd 到算不出的目录之后的相对路径、读不到规格的 replace-batch.py）改成字面路径。"
              "算得出的照算：cd 之后的 $PWD、mktemp -d 赋的变量、$HOME。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        service_directory = hook_input.get("cwd")
        service_writes = service_source_refusal(tool_input.get("command") or "", hook_input.get("agent_type"), repository_root_of(hook_dir),
                                                service_directory if isinstance(service_directory, str) and os.path.isdir(service_directory) else None)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成 Bash 里改两个服务源码与构建清单的写法（{error!r}），这条命令照常执行")
        service_writes = []
    fixture_writes = [item for item in service_writes if item.endswith(FIXTURE_NOTE_ENDINGS)]
    service_writes = [item for item in service_writes if not item.endswith(FIXTURE_NOTE_ENDINGS)]
    if fixture_writes:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里往主工作区已有门禁的样本档目录写（认出的写法与目标：{'；'.join(fixture_writes)}）："
              "改已有的样本、在那个目录里新建样本、删与挪走、整目录拷、解包与打补丁都算；样本先于门禁新版进主工作区、门禁新版还在草稿里，别的会话这段时间跑这一道的样本档会判错；"
              "写闸 write-guard.sh 第十一道只看 Write / Edit，Bash 里的写由这一道照同一份判法（lib_write_scope.py 的 existing_gate_of_fixture）判")
        refusal_lines.append("     → 怎么办：样本的新版写在草稿目录：照仓里的相对路径，改前的样本拷进 <草稿目录>/fixtures-base/（只新建样本的建成空目录），改后的与新建的放进 <草稿目录>/fixtures-new/；"
              "在草稿目录里 `git diff --no-index --binary fixtures-base fixtures-new > fixtures.patch` 生成补丁；把 `git apply -p2 --check <草稿目录>/fixtures.patch` 与 "
              "`git apply -p2 <草稿目录>/fixtures.patch`（在仓根起）写进报告「要主 agent 跑的命令」一节，排在门禁新版换上之后、样本档之前，由主 agent 打上。"
              "不拦：还没有门禁阶段的样本目录（新门禁的样本与新门禁一起写），草稿目录与仓副本里的文件，主 agent。这一道只拒这种写法，不停你在跑的任何东西")
    if service_writes:
        refusal_lines.append(f"  ✗ {hook_input.get('agent_type')} 在 Bash 里改了 prune-server、gpu-verification-server 里已存在的源码或构建清单（认出的写法与目标：{'；'.join(service_writes)}）："
              "写、带备份的拷与挪、整目录拷、删（rm）、挪走、换成链接、建硬链接、往硬链接追加、解包（tar -x）、打补丁（patch、git apply）都算这一道判的改；"
              "源码与 Cargo.toml、Cargo.lock 分几次改到一半（加了枚举成员而各处 match 还没补全、依赖与 src/ 对不上），主工作区里的服务编不过，别的会话带临时判定库服务的自证都取不到二进制；"
              "写闸 write-guard.sh 第十一道只看 Write / Edit，Bash 里的写由这一道判同一组落点")
        refusal_lines.append("     → 怎么办：在仓副本里改（建法见 .claude/agent-common.md「建仓副本」；删、挪走、解包、打补丁也在副本里做），把改动存成补丁写进草稿目录，打补丁的命令写进报告「要主 agent 跑的命令」一节，由主 agent 一次打上。"
              "不拦：新建的文件，草稿目录与仓副本里的文件，两个服务的 target/ 下的文件，主 agent。这一道只拒这种写法，不停你在跑的任何东西")
    given_cwd = hook_input.get("cwd")
    command_directory = given_cwd if isinstance(given_cwd, str) and os.path.isdir(given_cwd) else repository_root_of(hook_dir)
    try:
        notation_hits = literal_notation_hits(tool_input.get("command") or "", repository_root_of(hook_dir), command_directory)
        notation_remedies = literal_notation_remedies(notation_hits) if notation_hits else []
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成往仓里写的字面内容里的会话号、钟点、批次代号与任务号、会话短号、批号（{error!r}），这条命令照常执行")
        notation_hits, notation_remedies = [], []
    if notation_hits:
        shown = [f"{form} {relative} 第 {number} 行{form_name}「{matched}」" for form, relative, form_name, number, matched in notation_hits[:LITERAL_NOTATION_LISTED]]
        if len(notation_hits) > LITERAL_NOTATION_LISTED:
            shown.append(f"另有 {len(notation_hits) - LITERAL_NOTATION_LISTED} 处没列，改完上面这些再起会列出来")
        refusal_lines.append(f"  ✗ 往仓里的文件写的字面内容里有会话号、钟点、批次代号或任务号、会话短号、批号（认出的写法与目标：{'；'.join(shown)}）："
              "写闸 write-guard.sh 只看 Write / Edit，Bash 里 heredoc、echo、printf、tee 往仓里写的字面内容由这一道照同一份判据库（.claude/gate.d/lib-forbidden-notations.py）判")
        refusal_lines.append("     → 怎么办：" + "；".join(notation_remedies)
              + "。改完用 Edit 写，或用 `python3 scripts/kb/insert-row.py` / `python3 scripts/kb/replace-once.py` 定点写（它们写之前也判会话号、钟点、批次代号与任务号、会话短号、批号）；"
              "命中的其实不是这几样的，改那份判据库的上下文规则或排除前缀。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        index_resets = whole_index_reset_refusal(tool_input.get("command") or "", command_directory)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成清整个索引的 git 写法（{error!r}），这条命令照常执行")
        index_resets = []
    if index_resets:
        refusal_lines.append(f"  ✗ 暂存区里有东西，这条命令不带具体路径清整个索引（认出的写法：{'；'.join(index_resets)}）："
              "暂存区里可能有别的会话暂存的改动，整份清掉就是替它们撤了暂存")
        refusal_lines.append("     → 怎么办：只清自己的路径：`git restore --staged -- <路径…>`；先 `git diff --cached --name-only` 看暂存区里有哪些、哪些是自己的。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    try:
        discards = worktree_discard_refusal(tool_input.get("command") or "", command_directory)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成带路径撤掉工作区改动的 git 写法（{error!r}），这条命令照常执行")
        discards = []
    if discards:
        refusal_lines.append(f"  ✗ 这条命令要撤掉工作区里还没暂存的改动（认出的写法：{'；'.join(discards)}）："
              "git checkout / git restore 拿暂存区或提交盖掉点名的路径，git stash push 把它们收走，git clean 删掉没跟踪的文件；"
              "主工作区几个会话共用，这几处可能是别的会话还没提交的改动，撤了就找不回来")
        refusal_lines.append("     → 怎么办：先 cp 一份留底（`python3 scripts/agents/copy-originals.py copy <草稿目录> <路径…>`）；"
              "只撤自己那几块：把自己的改动存成补丁（`git diff -- <路径> > <草稿目录>/own.diff`，删掉不是自己的块），再 `git apply -R <草稿目录>/own.diff`，别人的块照留；"
              "只动暂存区的 `git restore --staged -- <路径…>` 不拦。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        ledger_writes, noted_ledger = ledger_append_refusal(tool_input.get("command") or "", repository_root_of(hook_dir), command_directory)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成往现行账本直接追加节标题的写法（{error!r}），这条命令照常执行")
        ledger_writes, noted_ledger = [], []
    if ledger_writes:
        refusal_lines.append(f"  ✗ 要往现行账本直接追加「## <节号>、」标题（认出的写法与目标：{'；'.join(ledger_writes)}）："
              "几个会话同时追加，各自读文件、各自取号，节号会撞，到提交时门禁 doc-process-records 的 ledger-section-numbers 格才判出重号")
        refusal_lines.append("     → 怎么办：新节经 `python3 scripts/kb/ledger-append-section.py --title '<标题（含日期）>' --body-file <草稿目录里写好的正文文件>`：现取最大节号加一，"
              "一次写入索引表末行与末尾新节，写前复核 sha256，撞了拒（重跑取下一个号）；索引行的判定、历史行、当日还开着的几行用 --judgement、--history-rows、--open-rows 给。"
              "改已有节的节号或正文用 `scripts/kb/replace-once.py` 定点改。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        record_appends = record_append_refusal(tool_input.get("command") or "", repository_root_of(hook_dir), command_directory)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成往 records/ 下的 .md 追加的写法（{error!r}），这条命令照常执行")
        record_appends = []
    if record_appends:
        shown_appends = "；".join(f"{form} {relative}" for form, relative in record_appends)
        refusal_lines.append(f"  ✗ 要在 Bash 里往 records/ 下的 .md 追加（认出的写法与目标：{shown_appends}）："
              "这样追加的文本不过写闸 write-guard.sh 的钟点、会话号与批次代号判定，要等提交时门禁 doc-text 才判出来")
        refusal_lines.append(f"     → 怎么办：追加的文本先写进草稿目录里的文件，再 `python3 {record_howto_tool()} {record_appends[0][1]} {RECORD_HOWTO_TEXT_PLACEHOLDER}`"
              "（文本从标准输入读写 `-`）：写之前照门禁 doc-text 的 clock-times、session-names 与批次代号同一份判据判这段文本，命中就拒、点名第几行；"
              "读时记 sha256、换上之前复核、改名换上。它不收一、二级标题：新开一节用 Edit 接在文件末尾（写闸照判同几样），拆分提案现行账本加节用 ledger-append-section.py。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    try:
        removal_directory = hook_input.get("cwd")
        removals = work_root_removal_refusal(tool_input.get("command") or "",
                                             removal_directory if isinstance(removal_directory, str) and os.path.isdir(removal_directory)
                                             else repository_root_of(hook_dir))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成 rm -r 通删工作根的写法（{error!r}），这条命令照常执行")
        removals = []
    if removals:
        refusal_lines.append(f"  ✗ rm -r 要删工作根本身、它的上级或工作根下第一层的通配（认出的写法：{'；'.join(removals)}）："
              "工作根下的目录分属不同会话，按名字通删会连别的会话正在用的副本、报告与只存在仓外的跑前登记一起删掉，删了救不回来")
        refusal_lines.append(f"     → 怎么办：清本会话自己的目录，先 `bash {WORK_ROOT_SWEEP_SCRIPT} sweep --dry-run` 看会删哪些，再 `bash {WORK_ROOT_SWEEP_SCRIPT} sweep`"
              "（只删归属是本会话、没有进程开着的，删前逐项连大小写进 <工作根>/sweep-<日期>.log；别的会话的、没有归属文件的列出来交用户定）；"
              f"在工作根下新建目录时 `bash {WORK_ROOT_SWEEP_SCRIPT} claim <目录> --purpose <一句>` 写归属文件。"
              "删一个具体目录直接写它的路径（`rm -rf <工作根>/<目录>`，不带通配）。这一道只拒这种写法，不停你在跑的任何东西")
    try:
        copy_directory = hook_input.get("cwd")
        manual_copies = manual_repository_copy_refusal(tool_input.get("command") or "", repository_root_of(hook_dir),
                                                       copy_directory if isinstance(copy_directory, str) and os.path.isdir(copy_directory)
                                                       else repository_root_of(hook_dir))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成手拷仓副本的写法（{error!r}），这条命令照常执行")
        manual_copies = []
    if manual_copies:
        refusal_lines.append(f"  ✗ 要手拷仓副本（认出的写法：{'；'.join(manual_copies)}）：手列的拷贝范围跟不上编译与测试要读的仓根条目"
              "（path 依赖指到的目录、include_str! 与测试读的仓根文件），拷出来的副本缺东西，到 cargo 编不过、测试读不到才知道")
        refusal_lines.append(f"     → 怎么办：仓副本只用 `{manual_copy_howto(repository_root_of(hook_dir))}` 建（拷哪些由 scripts/hosts/lib_repository_copy.py 的闭包现算："
              "path 依赖、编译期与运行期读的仓根文件、测试输入；直接跑、不经服务，副本写进自己的草稿目录），要门禁表的加 `--with-gate-tables`，"
              "要在副本里起仓里脚本的逐条加 `--with scripts=<为什么>`；它拒了照它的出路办。拷草稿目录里自己的东西、拷 crates/ 下单个包或单个文件不拦。"
              "这一道只拒这种写法，不停你在跑的任何东西")
    try:
        git_hook_lines = git_hook_refusal(tool_input.get("command") or "", command_directory, hook_input.get("agent_type"))
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成 git commit / git push 有没有绕开 git 钩子、钩子装没装上（{error!r}），这条命令照常执行")
        git_hook_lines = []
    refusal_lines.extend(git_hook_lines)
    try:
        whole_target_builds = whole_target_build_refusal(tool_input.get("command") or "", command_directory)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成交给任务服务的 cargo build 点没点全部目标（{error!r}），这条命令照常执行")
        whole_target_builds = []
    if whole_target_builds:
        refusal_lines.extend(whole_target_build_messages(whole_target_builds))
    if reasons:
        refusal_lines.append(f"  ✗ 看门狗起法不对（{'；'.join(reasons)}；认出的调用：{'、'.join(calls)}）：这样起的看门狗叫不醒主 agent，等于没盯")
        refusal_lines.append("     → 怎么办：看门狗用 Bash 的 run_in_background: true 起，命令只写 `bash scripts/agents/watch.sh [--ack …] <agent 号，逗号分隔>`，"
              "不加 `&`、`nohup`、`disown`，不重定向输出（只盯自己起的长进程写 `bash scripts/agents/watch.sh --processes`）")
    # 喂给从标准输入读命令的 shell、判不了的输入（here-string 或 printf / echo 的参数里有变量、命令替换，管道前一段是 cat 文件这类；lib_shell_words.py 的 shell_standard_input）：
    # 照常执行、记一条检出（字面的那一部分已经当命令切过，上面各道照判）
    try:
        unjudged_inputs = list(shell_words.commands_at_command_position(tool_input.get("command") or "").unjudged_shell_inputs)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没判成喂给 shell 的标准输入（{error!r}），这条命令照常执行")
        unjudged_inputs = []
    # 判不了的几样（各道抛异常、库读不到、喂给 shell 的不是字面）都记检出；弄坏开关 failure-unrecorded 下不记（只打 stderr）
    unjudged_notes = [] if detector_break_is_set("failure-unrecorded") else list(JUDGING_NOTICES) + unjudged_inputs
    detections_path = os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS
    if refusal_lines:
        blocks = [refusal_lines[index:index + 2] for index in range(0, len(refusal_lines), 2)]
        if detector_break_is_set("stop-at-first-refusal"):
            blocks = blocks[:1]   # 弄坏开关：退回只报第一类的旧样子（只给自检证明「一次列全」那一格会红）
        if len(blocks) > 1:
            print(f"  ✗ 这条命令踩了 {len(blocks)} 类执行前拒绝的写法，下面一次列全；逐类照各自的「→ 怎么办」改完再起", file=sys.stderr)
        for block in blocks:
            print("\n".join(block), file=sys.stderr)
        if unjudged_notes:   # 拒了的命令不记别的检出，判不了的几样照记（结局「被拒」）
            try:
                append_detection(hook_input, unjudged_notes, detections_path, "被拒")
            except Exception as error:
                print(f"  ! bash-command-detector.sh 没把判不了的几样写进检出记录（{error!r}）", file=sys.stderr)
        return 2
    try:
        record(hook_input, detections_path, list(noted_overwrites) + list(scope_verdict.noted) + list(noted_ledger) + list(edit_verdict.noted)
               + unjudged_notes)
    except Exception as error:
        print(f"  ! bash-command-detector.sh 没把检出写进检出记录（{error!r}），这条命令照常执行", file=sys.stderr)
    return 0

def ledger_append_refusal(command, repository_root, start_directory):
    """⑯：交回 (要拒的写法与目标, 要记的检出)。命令里没有节标题的不读登记表；弄坏开关 allow-ledger-append 下一概放行。"""
    if detector_break_is_set("allow-ledger-append") or not LEDGER_HEADING_TEXT.search(command):
        return [], []
    relative, problem = current_ledger_relative(repository_root)
    if relative is None:
        return [], [f"判不了往现行账本追加节标题：{problem}，没拦"]
    ledger = os.path.realpath(os.path.join(repository_root, relative))
    refused = []
    for step in overwrite_steps(command, start_directory, every_write=True):
        base = step.directory if getattr(step, "directory", None) is not None else repository_root
        if isinstance(step, PythonCode):
            if LEDGER_HEADING_TEXT.search(step.code):
                refused += [f"{write.form} {relative}" for write in python_in_place_writes(step.code, every_write=True) if lands_on_ledger(write.path, base, ledger)]
        elif isinstance(step, OverwriteCandidate) and step.form in LEDGER_WRITING_FORMS:
            pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(step.target))
            if not has_unknown_piece and not has_glob and lands_on_ledger(pattern, base, ledger):
                refused.append(f"{step.form} {relative}")
    return list(dict.fromkeys(refused)), []

def record_append_refusal(command, repository_root, start_directory):
    """⑲：交回要拒的 [(写法, 相对仓根的目标)]，去重、按出现次序。目标这一刻算不出的不判（文件头 ⑲「判不了的」）。
    弄坏开关 allow-record-append 下一概放行；record-append-python-unseen 下不认 python 的 open(…, 'a')；record-append-tee-unseen 下不认 tee -a。"""
    if detector_break_is_set("allow-record-append"):
        return []
    refused = []
    for step in overwrite_steps(command, start_directory, every_write=True):
        if isinstance(step, PythonCode):
            if detector_break_is_set("record-append-python-unseen"):
                continue
            for write in python_in_place_writes(step.code, every_write=True):
                mode = PYTHON_OPENING_MODE.match(write.form)
                landing = record_append_landing(write.path, step.directory, repository_root) if mode and "a" in mode.group(1) else None
                if landing:
                    refused.append((write.form, landing))
        elif isinstance(step, OverwriteCandidate):
            if step.form in APPENDING_REDIRECTS:
                form = step.form
            elif step.form == "tee" and step.appends and not detector_break_is_set("record-append-tee-unseen"):
                form = "tee -a"
            else:
                continue
            pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(step.target))
            landing = None if has_unknown_piece or has_glob else record_append_landing(pattern, step.directory, repository_root)
            if landing:
                refused.append((form, landing))
    return list(dict.fromkeys(refused))

def literal_writes(text, directory, variables=None, depth=0):
    """整条命令每一层（顶层、bash -c 的代码、喂给 shell 的 heredoc 正文、命令替换）命令位置上往文件里写字面内容的几处（LiteralWrite，按执行的先后）：
    cat / echo / printf 的输出经 >、>>、>|、&>、&>> 写进的，与 tee（带不带 -a）把 heredoc、here-string、管道前一段的字面写进的。cd 与赋值跟着算（follow_simple_command）。"""
    if depth > shell_words.MAXIMUM_NESTING:
        return []
    variables = dict(variables or {})
    writes, data_heredocs, carried = [], [], []
    pieces, piped = command_pieces(shell_words.shell_tokens(shell_words.strip_data_heredocs(text, stripped_bodies=data_heredocs)))
    for piece, after_pipe in zip(pieces, piped):
        for is_operator, substitution in piece:
            if is_operator is None:
                writes += literal_writes(substitution, directory, variables, depth + 1)
        commands, _, _, redirections = shell_words.simple_commands(piece)
        words = commands[0] if commands and commands[0] else []
        if not words:
            carried = []
            continue
        fed = [data_heredocs[int(marker.group(1))] for operator, target in redirections[0] if operator in ("<<", "<<-")
               for marker in [shell_words.HEREDOC_BODY_MARKER.match(target)] if marker and int(marker.group(1)) < len(data_heredocs)]
        fed += [shell_words.HeredocBody("<<<", False, target + "\n") for operator, target in redirections[0] if operator == "<<<"]
        if not detector_break_is_set("literal-variables-unsubstituted"):
            # 结束词不带引号的 heredoc 与 here-string，shell 照样展开正文里的变量：前面赋过字面值的先代入
            fed = [body if body.is_delimiter_quoted else body._replace(text=substitute_assigned_variables(body.text, variables)) for body in fed]
        stdin_bodies = fed or (carried if after_pipe else [])
        output = literal_output(words, stdin_bodies, variables)
        name, arguments = None, []
        for _, _, name, arguments in wrapped_commands(words):
            pass
        if output is not None:
            for (is_operator, operator), (next_is_operator, target) in zip(piece, piece[1:]):
                if (is_operator and operator in OVERWRITING_REDIRECTS | APPENDING_REDIRECTS and next_is_operator is False
                        and not (operator == ">&" and FILE_DESCRIPTOR_WORD.match(target))):
                    writes.append(LiteralWrite(f"{name} {operator}", substitute_assigned_variables(target, variables), directory, output))
        if name == "tee" and stdin_bodies:
            form = "tee" if tee_files(arguments) else "tee -a"   # tee_files 对 -a / --append 交空表
            writes += [LiteralWrite(form, substitute_assigned_variables(argument, variables), directory, stdin_bodies[-1].text)
                       for argument in arguments if argument != "-" and not argument.startswith("-")]
        if name in shell_words.SHELL_NAMES:
            code = shell_words.shell_invocation(arguments).code_string
            if code is not None:
                writes += literal_writes(code, directory, variables, depth + 1)
            elif stdin_bodies and shell_words.heredoc_feeds_shell(shlex.join([name, *arguments])):
                writes += literal_writes(stdin_bodies[-1].text, directory, variables, depth + 1)
        directory = follow_simple_command(words, directory, variables, [], depth)
        carried = [shell_words.HeredocBody("|", True, output)] if output is not None else []
    return writes

def literal_notation_hits(command, repository_root, start_directory):
    """⑰：这条命令往仓里的文件写的字面内容里，判据库四种形态（会话号、钟点、批次代号、任务号与会话短号与批号）的每一处：[(写法, 相对仓根的目标, 形态名, 行号, 命中的串)]。
    落在仓外、落点算不出、在那一形态的排除前缀下的不判；后缀、放行标记与按路径的上下文规则由库判（批次代号与任务号、会话短号、批号只判 .md、.tsv、.txt）。
    会话号、钟点与批次代号判写进去的字面内容（行号是内容里的第几行）；任务号、会话短号与批号判这一处写之前、之后两份整份目标（库的 new_violation_hits，行号是写后文件的行号）：
    追加的写法（>>、&>>、tee -a）写后是写前接上内容，整份写的写后就是内容；同一条命令里几处写同一个落点的，按执行的先后在内存里依次施加（literal_landing_text），
    前一处写进的收尾围栏让后一处追加的行出了围栏、照判。读不到库抛异常。
    弄坏开关 allow-literal-notations 下交空表；literal-notations-outside-judged 下仓外的目标也判（按文件名当相对路径）；
    literal-notations-new-text-only 下任务号那一形态也只判写进去的内容（写前当空串、写后就是内容），看不到写之后整份的样子。"""
    if detector_break_is_set("allow-literal-notations"):
        return []
    root, hits, written = os.path.realpath(repository_root), [], {}
    for write in literal_writes(command, start_directory):
        landing = literal_write_landing(write)
        if landing is None:
            continue
        inside = os.path.commonpath([landing, root]) == root
        if not inside and not detector_break_is_set("literal-notations-outside-judged"):
            continue
        relative = os.path.relpath(landing, root) if inside else os.path.basename(landing)
        library = load_notations_library()
        before = literal_landing_text(landing, written)
        appending = write.form.endswith((">>", "&>>")) or write.form == "tee -a"
        after = before + write.text if appending else write.text
        written[landing] = after
        for form_key, form_name in LITERAL_NOTATION_FORMS:
            if library.is_excluded(form_key, relative):
                continue
            if form_key == LITERAL_WHOLE_FILE_FORM and not detector_break_is_set("literal-notations-new-text-only"):
                found = library.new_violation_hits(form_key, before, after, relative)
            elif form_key == LITERAL_WHOLE_FILE_FORM:
                found = library.new_violation_hits(form_key, "", write.text, relative)
            else:
                found = library.all_hits(form_key, write.text, relative)
            hits += [(write.form, relative, form_name, number, matched) for number, _, _, matched in found]
    return hits

def read_pending_list(root):
    """待改清单：一行一个相对仓根的路径，# 后写为什么。交回 ({路径: (行号, 理由)}, 清单本身的毛病)。"""
    path = os.path.join(root, PENDING_LIST)
    entries, problems = {}, []
    if not os.path.exists(path):
        return entries, problems
    with open(path, encoding="utf-8") as handle:
        for number, line in enumerate(handle, 1):
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            relative, _, reason = stripped.partition("#")
            relative, reason = relative.strip(), reason.strip()
            if len(reason) < MINIMUM_ANNOTATION_REASON_CHARACTERS:
                problems.append(f"{PENDING_LIST}:{number} {relative} 没写为什么（# 后至少 {MINIMUM_ANNOTATION_REASON_CHARACTERS} 个字）")
            if not os.path.isfile(os.path.join(root, relative)):
                problems.append(f"{PENDING_LIST}:{number} {relative} 指向不存在的文件（不起作用的排除项让人以为那一份被绕开了）")
            entries[relative] = (number, reason)
    return entries, problems

def scan_scripts(root, directories):
    """--scan-scripts 的本体：退出码 0 全绿、1 有红。"""
    pending, list_problems = read_pending_list(root)
    red, exempted, pending_hits, scanned = [], [], {}, {".sh": 0, ".py": 0}
    scanned_folders = {os.path.realpath(directory) for directory in directories if os.path.isdir(directory)}   # 这一趟真扫的目录（只扫这一层，不进子目录）
    for directory in directories:
        if not os.path.isdir(directory):
            continue
        for file_name in sorted(os.listdir(directory)):
            path = os.path.join(directory, file_name)
            if not (os.path.isfile(path) and file_name.endswith(SCANNED_SUFFIXES)):
                continue
            relative = os.path.relpath(path, root)
            scanned[os.path.splitext(file_name)[1]] += 1
            text = open(path, encoding="utf-8", errors="replace").read()
            findings, annotations, lines = scan_script_file(path, text)
            used_annotations = set()
            for line, finding in findings:
                annotation = annotations.get(line)
                where = f"{relative}:{line or '?'}"
                if annotation is not None:
                    guarded = any(OWN_SCOPE_GUARD.search(earlier) for earlier in lines[max(0, line - 1 - OWN_SCOPE_GUARD_LOOKBACK_LINES):line - 1])
                    if finding.kind not in OWN_SCOPE_EXEMPTABLE_KINDS:
                        red.append((relative, f"{where}  {finding.problem}；own-scope 标注只放行循环与按 cgroup 挑进程两类，这一类没有例外"))
                    elif len(annotation) < MINIMUM_ANNOTATION_REASON_CHARACTERS:
                        red.append((relative, f"{where}  {finding.problem}；own-scope 标注没写怎么核的（至少 {MINIMUM_ANNOTATION_REASON_CHARACTERS} 个字）"))
                    elif not guarded:
                        red.append((relative, f"{where}  {finding.problem}；标了 own-scope，往上 {OWN_SCOPE_GUARD_LOOKBACK_LINES} 行里却没有核 singlefs-memory-cap-*.scope 的那一句"))
                    else:
                        exempted.append(where)
                    used_annotations.add(line)
                    continue
                if relative in pending:
                    pending_hits.setdefault(relative, []).append(where)
                    continue
                red.append((relative, f"{where}  {finding.problem}"))
            for line in sorted(set(annotations) - used_annotations):
                red.append((relative, f"{relative}:{line}  标了 # process-safety:own-scope，这一行却没有要放行的发信号写法（不起作用的标注让人以为那一处核过）"))
    not_judged_here = []   # 待改清单里所在的目录这一趟没扫的行：没扫就不知道它还有没有要放行的，判「本次未判」，不判成「一处要放行的都没有」
    for relative in sorted(pending):
        if relative not in pending_hits and os.path.isfile(os.path.join(root, relative)):
            if (os.path.realpath(os.path.dirname(os.path.join(root, relative))) not in scanned_folders
                    and not detector_break_is_set("pending-outside-scanned-as-stale")):
                not_judged_here.append(relative)
            else:
                list_problems.append(f"{PENDING_LIST}:{pending[relative][0]} {relative} 里一处要放行的都没有（改好了就从清单里删掉这一行）")
    not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
    for relative in not_judged_here:
        print(f"  ! 本次未判（这不是通过）：{PENDING_LIST}:{pending[relative][0]} {relative} 所在的目录 {os.path.dirname(relative) or '.'} 这一趟没扫，"
              "它还有没有要放行的没判；要判就把那个目录也交给 --scan-scripts")
        if not_run_file and os.path.exists(not_run_file):
            with open(not_run_file, "a", encoding="utf-8") as handle:
                handle.write(f"进程安全：{PENDING_LIST} 里 {relative} 所在的目录这一趟没扫，本次未判（要判就把 {os.path.dirname(relative) or '.'} 也交给 bash-command-detector.sh --scan-scripts）\n")
    for problem in list_problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    for _, message in red:
        print(f"  ✗ {message}")  # gate-lint:detail
    if red or list_problems:
        print(f"  ✗ 进程安全：{len(red)} 处发信号的写法打得到不是自己点名的那一个进程，待改清单 {len(list_problems)} 处不对（上面逐处列出）")  # gate-lint:summary
        print("     → 怎么办：只停自己起的、点名的那一个：记下它的 pid（`$!`）或任务号，单独 `kill \"$pid\"`、`kill %1` 或 "
              "`python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`，不在循环里、不一次给几个、不带负号或 0、不按名字挑；"
              "确实要按 cgroup 批量发信号的，先核 cgroup 路径是自己开的 singlefs-memory-cap-*.scope，再在那一行写 `# process-safety:own-scope <怎么核的>`；"
              f"别人正在改、这一轮动不了的文件登记进 {PENDING_LIST}（一行一个路径、# 后写为什么），改好了删掉那一行")
        return 1
    pending_note = "；".join(f"{relative}（{'、'.join(hits)}）" for relative, hits in sorted(pending_hits.items())) or "无"
    print(f"  ✓ 进程安全：查了 {scanned['.sh'] + scanned['.py']} 个脚本（.sh {scanned['.sh']} 个、.py {scanned['.py']} 个），"
          f"发信号的写法都只打得到点名的一个进程；own-scope 标注放行 {len(exempted)} 处（{'、'.join(exempted) or '无'}）；"
          f"没判的：{PENDING_LIST} 里的 {len(pending_hits)} 个文件 {pending_note}"
          + (f"；本次未判：{PENDING_LIST} 里 {len(not_judged_here)} 个文件所在的目录这一趟没扫（{'、'.join(not_judged_here)}）" if not_judged_here else ""))
    return 0

def selftest(hook_dir):
    if shell_words is None:
        print(f"  ✗ 自检：读不到共用切词模块 {shell_words_library_path(hook_dir)}（{shell_words_error!r}）")
        print("    → 怎么办：恢复 .claude/hooks/lib_shell_words.py（切词与认命令位置只有那一份，别在 hook 里再抄一份），再跑 --selftest")
        return 1
    # 临时目录由共用库建（通过、判红、抛异常都删），那三格也由它判；主体在 selftest_in
    scratch_library_path = os.path.join(HOOK_DIRECTORY, "lib_selftest_scratch.py")
    try:
        scratch_specification = importlib.util.spec_from_file_location("lib_selftest_scratch", scratch_library_path)
        scratch = importlib.util.module_from_spec(scratch_specification)
        scratch_specification.loader.exec_module(scratch)
    except Exception as error:   # 文件不在、语法错、导入时抛的都算读不到
        print(f"  ✗ 自检：bash-command-detector 读不到共用的临时目录库 {scratch_library_path}（{error!r}）")
        print("    → 怎么办：恢复 .claude/hooks/lib_selftest_scratch.py（自证临时目录的建法与三格探查只有那一份），再跑 --selftest")
        return 1
    exit_code, problems = scratch.run_with_scratch(
        lambda work: selftest_in(hook_dir, work), "bash-command-detector-",
        detector_break_is_set("keep-selftest-scratch"), "BASH_COMMAND_DETECTOR_SELFTEST_ROLE",
        ["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--selftest"])
    return scratch.merged_exit_code(exit_code, problems, ".claude/hooks/bash-command-detector.sh 的 selftest()",
                                    "BASH_COMMAND_DETECTOR_BREAK=keep-selftest-scratch")
# 自证现场写成两个函数（每份部分文件不超过 2000 行）：selftest_in_waits_and_scope 末尾把 selftest_in_entries_and_summary 要用的局部变量放进 shared，后者开头照名取回
def selftest_in(hook_dir, work):
    shared = {"hook_dir": hook_dir, "work": work}
    selftest_in_waits_and_scope(shared)
    return selftest_in_entries_and_summary(shared)
