#!/usr/bin/env bash
# admission: always Claude Code 每一次 Bash 命令做完都要现判这一条是不是 git commit、此刻共用索引有没有停在旧 HEAD 上，上一次的结论不替这一次作保
# run-condition: command python3 git
# PostToolUse hook（Bash）：命令文本里命令位置上有 git commit（经 -C、cd、bash -c 起的也认）时，提交做完之后跑 scripts/gate/shared-index-stale-check.py --repo <那条命令所在的仓>，
# 查共用索引有没有停在旧 HEAD 上（C420：几个会话共写一个仓，有人用 GIT_INDEX_FILE 指私有索引暂存并提交，共用索引一个字没动，
# `git diff --cached` 于是把刚进仓的文件全列成已暂存的改动，下一个人提交就把它们反向提交回去）。检出了就把检查脚本的输出
# （逐个列文件，加出路）作为提醒交给发命令的一方（hookSpecificOutput.additionalContext，同一段另作 systemMessage 给人看）；
# 只检出、只报告，从不拒、不改任何索引。判法与自证格在检查脚本里；这里只管认命令、找仓、转述。
# hook-events: PostToolUse:Bash
# gate-similar: host-resources-reminder.sh 同样认命令位置上的命令（lib_shell_words.py）、只提醒不拒、把结论交 additionalContext；它挂 PreToolUse[Bash]，在命令之前查各台资源，这里挂 PostToolUse[Bash]，要等提交做完才有 HEAD 可比，触发点不同，并不进去会让「之前」与「之后」两个事件挤在一个入口里
# gate-similar: bash-command-detector.sh 同用 lib_shell_words.py 切词；它在执行前判命令的形状（没超时的等待循环、放出追踪）记检出或拒，不看命令做完之后仓里的状态
# gate-similar: heavy-test-guard.sh 在执行前按命令文本拒重型测试，与索引无关；这里命令做完才查
# gate-similar: timer-only-remind-guard.sh 同挂 Bash 这一个工具，但挂 PreToolUse，判的是定时办法，与提交无关
# gate-similar: pattern-process-guard.sh 上游 SOP 的钩子，只判按模式找进程的写法，上游副本也不归本仓改
# gate-similar: session-start.sh 开会话时打一次本机状态；这里在每次提交之后现查共用索引，触发点不同
# gate-overlap:copy-kept host-resources-reminder.sh 自证末尾「逐个弄坏开关、起 --selftest-cells、核红在点名的格」的轮次样板各份各有自己的开关表与格名；抽成共用库要改 host-resources-reminder.sh 的自证，不在这一批的范围
# gate-similar: scripts/gate/stage-mine.py 管提交之前「只暂存这一轮的块」，不看提交之后共用索引与 HEAD 的差；判法在 scripts/gate/shared-index-stale-check.py，这个钩子只是把它挂到提交之后自动跑
#
# 认法：lib_shell_words.commands_at_command_position 切出命令位置上的每一条命令（前缀、包装、cd 照它），命令词是 git（取文件名），
#   跳过 git 的全局选项（-C <目录> 另记下，让仓目录跟着它走；-c、--git-dir、--work-tree、--namespace、--super-prefix、--config-env 各带一个值），
#   第一个不以 - 开头的词是 commit 才算。查的是那条命令所在的目录（cd 跟着走），hook 输入的 cwd 当起点，认不出目录时取 cwd。
#   merge、cherry-pick、revert、pull、rebase 这些也动 HEAD，但私有索引的做法只出现在 commit 上，这里不认。
# 时限：检查脚本至多跑 CHECK_TIME_LIMIT_SECONDS 秒（SHARED_INDEX_STALE_REMINDER_TIME_LIMIT 可换），到点只停自己起的那一个进程，什么都不提醒。
# 提醒超过 MAXIMUM_CONTEXT_LINES 行时只留前面这些行，末尾写怎么看全的。读不到共用模块、JSON 读不出、检查脚本退 0、77 以外也不是 1 的，什么都不提醒、退 0
#   （检查脚本退别的码时往 stderr 写一句没判成）。命令失败时有没有这个事件，取决于 Claude Code；这个钩子不看命令的结局，只看仓此刻的状态。
# 退役判据：不再有人用 GIT_INDEX_FILE 指私有索引提交，或提交的做法改成每个会话各用一个工作树时删掉。
#
#   shared-index-stale-reminder.sh --selftest   在临时目录里造小仓、喂造出来的 hook JSON；末尾逐个打开弄坏开关核它红在点名的格
# 弄坏开关 SHARED_INDEX_STALE_REMINDER_BREAK=<项>（只给证红用，逗号连几个）。项名、弄坏哪一处、打开后红在哪几格（格名是「✗ 自证 <格名>：」里的那一段）：
#   ignore-commit        git commit 一律不认                          commit-stale、commit-with-dash-c、commit-after-cd、commit-in-bash-c
#   remind-everything    哪条命令都查                                 not-commit
#   ignore-dash-c        -C 的值当成子命令                             commit-with-dash-c
#   ignore-directory     不跟 -C 与 cd，一律取 hook 输入的 cwd           commit-with-dash-c、commit-after-cd
#   refuse-command       检出了就拒（退 2）                           commit-stale、commit-with-dash-c、commit-after-cd、commit-in-bash-c
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（同 host-resources-reminder.sh）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import concurrent.futures, importlib.util, json, os, shutil, subprocess, sys, tempfile

CHECK_TIME_LIMIT_SECONDS = 30.0
MAXIMUM_CONTEXT_LINES = 80
BREAK_VARIABLE = "SHARED_INDEX_STALE_REMINDER_BREAK"
GIT_OPTIONS_WITH_VALUE = {"-c", "--git-dir", "--work-tree", "--namespace", "--super-prefix", "--config-env"}

def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")

def load_library(hook_dir, name):
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), name_path(hook_dir, name))
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    return library

def name_path(hook_dir, name):
    if name == "lib_shell_words":
        return os.path.join(hook_dir, "lib_shell_words.py")
    return os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "scripts", "gate", name + ".py")

def commit_directories(shell_words, text, start):
    """命令文本里每条 git commit 所在仓的目录（去重，保持出现次序）。"""
    found = []
    for command in shell_words.commands_at_command_position(text, start).commands:
        words = command.words
        if os.path.basename(words[0]) != "git":
            continue
        directory = start if broken("ignore-directory") else (command.directory or start)
        index, subcommand = 1, None
        while index < len(words):
            word = words[index]
            if word == "-C" and not broken("ignore-dash-c") and index + 1 < len(words):
                if not broken("ignore-directory"):
                    directory = shell_words.resolve_path(directory, words[index + 1]) or directory
                index += 2
            elif word in GIT_OPTIONS_WITH_VALUE:
                index += 2
            elif word.startswith("-") and word != "-C":
                index += 1
            else:
                subcommand = word
                break
        if (subcommand == "commit" and not broken("ignore-commit")) or broken("remind-everything"):
            if directory not in found:
                found.append(directory)
    return found

def check_repository(hook_dir, directory):
    """→ 检查脚本的输出行（检出了）；没检出或没判成交 None。"""
    script = name_path(hook_dir, "shared-index-stale-check")
    limit = float(os.environ.get("SHARED_INDEX_STALE_REMINDER_TIME_LIMIT") or CHECK_TIME_LIMIT_SECONDS)
    environment = {name: value for name, value in os.environ.items() if name not in ("SHARED_INDEX_STALE_CHECK_BREAK",)}
    try:
        completed = subprocess.run([sys.executable, "-B", script, "--repo", directory], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                                   env=environment, timeout=limit, check=False)
    except subprocess.TimeoutExpired:
        return None   # run 到点只停自己起的这一个进程
    if completed.returncode == 1:
        return completed.stdout.strip().splitlines()
    if completed.returncode not in (0, 77):
        tail = completed.stderr.strip().splitlines()[-1][:300] if completed.stderr.strip() else ""
        print(f"  ! shared-index-stale-reminder.sh 没判成（检查脚本退 {completed.returncode}{'：' + tail if tail else ''}），这条命令照常算数", file=sys.stderr)
    return None

def remind(hook_input, hook_dir):
    """交提醒正文；不用提醒交 None。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    text = tool_input.get("command") or ""
    start = hook_input.get("cwd") or os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()
    shell_words = load_library(hook_dir, "lib_shell_words")
    reports = []
    for directory in commit_directories(shell_words, text, start):
        lines = check_repository(hook_dir, directory)
        if lines:
            reports.append((directory, lines))
    if not reports:
        return None
    pieces = ["提交后检查：这条命令做了 git commit，共用索引没有跟上新 HEAD（scripts/gate/shared-index-stale-check.py 只报告，不拦命令、没动任何索引）："]
    for directory, lines in reports:
        pieces.append(f"仓 {directory}")
        pieces.extend(lines)
    if len(pieces) > MAXIMUM_CONTEXT_LINES:
        omitted = len(pieces) - MAXIMUM_CONTEXT_LINES
        pieces = pieces[:MAXIMUM_CONTEXT_LINES] + [f"（还有 {omitted} 行没列；看全：python3 scripts/gate/shared-index-stale-check.py --repo <仓>）"]
    return "\n".join(pieces)

def main_hook(hook_dir):
    try:
        hook_input = json.load(sys.stdin)
    except Exception:
        return 0
    if not isinstance(hook_input, dict):
        return 0
    try:
        message = remind(hook_input, hook_dir)
    except Exception as error:   # 共用模块读不到、切词抛的：什么都不提醒
        print(f"  ! shared-index-stale-reminder.sh 没判成（{error!r}）", file=sys.stderr)
        return 0
    if message is None:
        return 0
    print(json.dumps({"systemMessage": message, "hookSpecificOutput": {"hookEventName": "PostToolUse", "additionalContext": message}}, ensure_ascii=False))
    return 2 if broken("refuse-command") else 0

# ── 自证 ──
def selftest_cells(hook_dir, root):
    failures = []
    checked = 0
    hook = os.path.join(hook_dir, "shared-index-stale-reminder.sh")
    scenarios = load_library(hook_dir, "shared-index-stale-check")
    clean = {name: value for name, value in os.environ.items() if name not in (BREAK_VARIABLE, "GIT_INDEX_FILE")}

    def check(cell, condition, detail):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append(cell)
            print(f"  ✗ 自证 {cell}：{detail}")   # gate-lint:detail

    def feed(kind, command, directory=None, raw=None):
        fixture = scenarios.make_scenario(root, kind)
        text = raw if raw is not None else command.format(repository=fixture.repository, private_index=fixture.private_index)
        start = fixture.root if directory == "outside" else fixture.repository
        completed = subprocess.run(["bash", hook], input=text if raw is not None else json.dumps({"tool_name": "Bash", "tool_input": {"command": text}, "cwd": start}),
                                   capture_output=True, text=True, env=dict(clean, **({BREAK_VARIABLE: os.environ[BREAK_VARIABLE]} if BREAK_VARIABLE in os.environ else {})),
                                   timeout=120, check=False)
        try:
            context = json.loads(completed.stdout)["hookSpecificOutput"]["additionalContext"] if completed.stdout.strip() else ""
            event = json.loads(completed.stdout)["hookSpecificOutput"]["hookEventName"] if completed.stdout.strip() else ""
        except (ValueError, KeyError, TypeError):
            context, event = f"（stdout 不是那份 JSON：{completed.stdout[:200]}）", ""
        return completed.returncode, context, event

    def reported(context):
        return all(name in context for name in ("H.txt", "M.txt", "D.txt")) and "git read-tree HEAD" in context and "拿不准就先问" in context

    code, context, event = feed("stale", "GIT_INDEX_FILE={private_index} git commit -q -m x")
    check("commit-stale", code == 0 and event == "PostToolUse" and reported(context),
          f"私有索引提交之后：应当退 0、事件名 PostToolUse、提醒点名 D.txt、H.txt、M.txt 与出路；实际退 {code}，事件「{event}」，提醒「{context[:200]}」")
    code, context, _ = feed("stale", "git -C {repository} commit -q -m x", directory="outside")
    check("commit-with-dash-c", code == 0 and reported(context),
          f"命令在仓外、用 git -C 指仓：应当查那个仓、点名 H.txt 并退 0；实际退 {code}，提醒「{context[:200]}」")
    code, context, _ = feed("stale", "cd {repository} && git commit -q -m x", directory="outside")
    check("commit-after-cd", code == 0 and reported(context),
          f"命令先 cd 进仓再提交：应当查 cd 到的仓；实际退 {code}，提醒「{context[:200]}」")
    code, context, _ = feed("stale", "bash -c 'git commit -q -m x'")
    check("commit-in-bash-c", code == 0 and reported(context),
          f"bash -c 里的提交也认：应当报；实际退 {code}，提醒「{context[:200]}」")
    code, context, _ = feed("normal", "git commit -q -m x")
    check("commit-clean", code == 0 and context == "", f"正常提交之后：应当退 0、不出提醒；实际退 {code}，提醒「{context[:200]}」")
    code, context, _ = feed("stale", "git status --short && git log -1 --oneline && git diff --cached --name-only")
    check("not-commit", code == 0 and context == "", f"不是提交的命令（仓此刻确实是过期的）：应当不查、不出提醒；实际退 {code}，提醒「{context[:200]}」")
    code, context, _ = feed("stale", "", raw="这不是 JSON")
    check("bad-json", code == 0 and context == "", f"读不出的输入：应当退 0、什么都不说；实际退 {code}，提醒「{context[:200]}」")
    return failures, checked

SWITCH_EXPECTED_CELLS = {
    "ignore-commit": ["commit-stale", "commit-with-dash-c", "commit-after-cd", "commit-in-bash-c"],
    "remind-everything": ["not-commit"],
    "ignore-dash-c": ["commit-with-dash-c"],
    "ignore-directory": ["commit-with-dash-c", "commit-after-cd"],
    "refuse-command": ["commit-stale", "commit-with-dash-c", "commit-after-cd", "commit-in-bash-c"],
}

def cells_in_scratch(hook_dir):
    root = tempfile.mkdtemp(prefix="shared-index-stale-reminder-selftest-")
    try:
        return selftest_cells(hook_dir, root)
    finally:
        shutil.rmtree(root, ignore_errors=True)

def selftest(hook_dir):
    failures, checked = cells_in_scratch(hook_dir)
    if failures:
        print(f"  ✗ shared-index-stale-reminder.sh 自证 {len(failures)} 格不对（查了 {checked} 格）：{'、'.join(failures)}")
        print(f"  → 怎么办：照上面逐格改 commit_directories / check_repository / remind；{BREAK_VARIABLE} 设着的话本来就该红")
        return 1
    hook = os.path.join(hook_dir, "shared-index-stale-reminder.sh")
    clean = {name: value for name, value in os.environ.items() if name != BREAK_VARIABLE}

    def one(switch):
        completed = subprocess.run(["bash", hook, "--selftest-cells"], capture_output=True, text=True, env=dict(clean, **{BREAK_VARIABLE: switch}),
                                   timeout=600, stdin=subprocess.DEVNULL, check=False)
        return switch, set(line.split("：", 1)[0].removeprefix("  ✗ 自证 ") for line in completed.stdout.splitlines() if line.startswith("  ✗ 自证 "))

    with concurrent.futures.ThreadPoolExecutor(max_workers=len(SWITCH_EXPECTED_CELLS)) as pool:
        results = list(pool.map(one, SWITCH_EXPECTED_CELLS))
    switch_failures = [] if len(results) == len(SWITCH_EXPECTED_CELLS) else [f"开关轮派出 {len(SWITCH_EXPECTED_CELLS)} 个、收回 {len(results)} 个"]
    for switch, red in results:
        print(f"  · 弄坏开关 {switch}：红在 {'、'.join(sorted(red)) or '一格都没有'}")
        missing = [cell for cell in SWITCH_EXPECTED_CELLS[switch] if cell not in red]
        if missing:
            switch_failures.append(f"{switch} 没红在 {'、'.join(missing)}（红的是 {'、'.join(sorted(red)) or '一格都没有'}）")
    for failure in switch_failures:
        print(f"  ✗ 弄坏开关 {failure}")   # gate-lint:detail
    if switch_failures:
        print("  → 怎么办：照上面那几个开关看它在代码里弄坏的那一处还在不在判法上；改了判法就改 SWITCH_EXPECTED_CELLS 与文件头的表")
        return 1
    print(f"  ✓ shared-index-stale-reminder.sh 自证通过（查了 {checked} 格：私有索引提交之后（含 git -C、cd 之后、bash -c 里）出提醒且照常放行，正常提交、不是提交的命令、读不出的输入不出提醒；"
          f"弄坏开关 {len(results)} 个逐个打开都红在点名的格）")
    return 0

def main():
    hook_dir = sys.argv[1]
    if sys.argv[2:] == ["--selftest"]:
        return selftest(hook_dir)
    if sys.argv[2:] == ["--selftest-cells"]:
        failures, _ = cells_in_scratch(hook_dir)
        return 1 if failures else 0
    return main_hook(hook_dir)

sys.exit(main())
PY
