#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一条命令、现查各台此刻的资源，上一次查到的不替这一次作保
# run-condition: command python3
# PreToolUse hook（Bash）：认出要起重活或起 GPU 的命令时，先跑 scripts/hosts/host-resources.py 查各台此刻能用多少、带着这条命令判它放不放得下，
# 把那几行表与结论作为提醒交给发命令的一方（hookSpecificOutput.additionalContext，同一段另作 systemMessage 给人看）；命令照常放行，从不拒、不排队。
# 用户 2026-10-06 定：「你写个脚本 每次跑之前必须运行 测一下双机的显卡和cpu 然后回报，提醒对话可以使用的最大的cpu资源数和gpu资源数」
#   「先跑脚本获取资源数，然后根据资源数和自己的历史运行时间来确实能不能跑起来，如果可以就跑，如果不行就回报」；records/2026-09-16-subagent拆分提案.md 第五十五节。
# hook-events: PreToolUse:Bash
# gate-similar: heavy-test-guard.sh 同挂 PreToolUse[Bash]、认的也是重型测试（同一份判定 lib_heavy_tests.py 的 classify 与 runs_compiled_code，按文件路径导入）；它按 agent_type 与 SINGLEFS_HEAVY_TESTS 前缀拒，这里一条都不拒、只查资源出提醒，还另认 GPU 的跑法，并进它会让「拒」与「只提醒」挤在同一个退出码与同一份 stderr 里，它的自证与放行表也跟着要改
# gate-similar: bash-command-detector.sh 同挂 PreToolUse[Bash]、同用 lib_shell_words.py 切词；它判命令的形状（没超时的等待循环、放出追踪、起看门狗的写法）记检出或拒，不起子进程查外部状态；这里要起报告脚本、等它至多一个时限，放进它会拖慢它判的每一条命令
# gate-similar: pattern-process-guard.sh 同挂 PreToolUse[Bash]；它是上游 SOP 的钩子，只判按模式找进程的写法，与资源无关，上游副本也不归本仓改
# gate-similar: session-start.sh 挂 SessionStart，开会话时打一次本机状态；这里在每一条重活或 GPU 命令之前现查各台，触发点不同
# gate-similar: run-scope-guard.sh 同挂 PreToolUse[Bash]、也认 scripts/gate/merge-and-verify.sh（lib_run_scope.py 用 lib_heavy_tests.located_in 认位置，这里调同一个函数，不另写）；它要合入后验证带范围声明、不带就拒，这里只报各台资源、从不拒，并进它会让拒与提醒挤在同一个退出码与同一份 stderr 里
# gate-similar: timer-only-remind-guard.sh 同挂 PreToolUse、matcher 里有 Bash；它只拒 scripts/agents/remind.sh 之外的定时办法，与资源无关、不起子进程查外部状态，判的对象与触发的命令都不同
#
# 认法（只看命令文本里命令位置上的命令，剥前缀与包装照 lib_shell_words.py，/usr/bin/time、flock、systemd-run、detached-run.sh start 这一类照
#   lib_heavy_tests.command_under_launcher 剥开再判；执行的脚本文件不读进去）：
#   重活：lib_heavy_tests.classify 认出的重型测试（提交时才跑的检查那几类 COMMIT_CHECK_KINDS 不算：它们是 lint 与轻阶段），
#         或 lib_heavy_tests.runs_compiled_code 认出的跑编译出来的代码（cargo test / run / bench、直接执行测试二进制与编出来的装置）。
#   合入后验证：scripts/gate/merge-and-verify.sh（按文件名与位置认：lib_heavy_tests.located_in(<命令词>, <目录>, "scripts/gate")，同 lib_run_scope.py 认它的写法；
#         source 进来的、参数只有 --selftest 的不算）。它自己经 run-with-memory-cap.sh 起 cargo test、fmt、clippy、build，命令位置上看不到 cargo，上面两条认不出；
#         本机排不上时它的出路是带 --peer 交给第二台（它文件头「--peer」），起它之前先看各台放不放得下。
#   GPU：带 gpu 特性的 cargo（--features / -F 的值里有 gpu 或 <包>/gpu，或 --all-features；`--` 之后的不看）、
#         scripts/crash/gpu-quiesce-run.sh、scripts/crash-amplification*.sh、scripts/hosts/host-dispatch.py（按文件名认；source 进来的、
#         参数只有 --selftest 的不算）。认出 GPU 时报告脚本带 --gpu。
#   报告脚本带着认出的那一条（剥掉包装之后的词，命令里设的 RUN_WITH_MEMORY_CAP_KEY 照传）跑：host-resources.py [--gpu] -- <那一条>。
# 时限：报告脚本至多跑 REPORT_TIME_LIMIT_SECONDS 秒（HOST_RESOURCES_REMINDER_TIME_LIMIT 可换），到点只停自己起的那一个进程（按它的进程号），
#   提醒里照实写「各台此刻的资源没读到（报告脚本 N 秒没跑完）」与它到那时已经打出来的几行，命令照常放行。报告脚本退 0、1 以外的码同样写「没读到」。
#   .claude/settings.json 里这一条的 timeout 要大于这个时限。
# HOST_RESOURCES_REMINDER_SCRIPT 换掉报告脚本（只给自证）。读不到共用模块、JSON 读不出、认不出命令时什么都不打、退 0。
# 退役判据：派发器与各条跑法都改成开跑前自己查资源、提醒不再有人看时删掉。
#
#   host-resources-reminder.sh --selftest   在临时目录里喂造出来的 hook JSON，报告脚本换成假的；末尾逐个打开弄坏开关核它红在点名的格
# 弄坏开关 HOST_RESOURCES_REMINDER_BREAK=<项>（只给证红用，逗号连几个）。项名、弄坏哪一处、打开后红在哪几格（格名是「✗ 自证 <格名>：」里的那一段）：
#   ignore-heavy        重活不认                         heavy-test、compiled-code、report-timeout
#   ignore-merge-and-verify  合入后验证不认              merge-and-verify
#   ignore-gpu          GPU 的跑法不认                   gpu-script、gpu-feature
#   remind-everything   哪条命令都出提醒                 plain-command、merge-and-verify
#   no-time-limit       等报告脚本不设时限               report-timeout
#   refuse-command      出了提醒就拒（退 2）             heavy-test、compiled-code、gpu-script、gpu-feature、report-timeout、merge-and-verify
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（同 heavy-test-guard.sh）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import concurrent.futures, fnmatch, importlib.util, json, os, re, shlex, shutil, subprocess, sys, tempfile, time

REPORT_TIME_LIMIT_SECONDS = 25.0   # 报告脚本读别的台每台至多 12 秒、一秒采样，并行读；留余量，又不让一条命令等太久
MAXIMUM_LAUNCHER_DEPTH = 4
GPU_SCRIPT_NAMES = {"gpu-quiesce-run.sh", "host-dispatch.py"}
GPU_SCRIPT_PATTERNS = ("crash-amplification*.sh",)
MERGE_AND_VERIFY = ("merge-and-verify.sh", "scripts/gate")   # 合入后验证：文件名与所在目录（文件头「认法」）
BREAK_VARIABLE = "HOST_RESOURCES_REMINDER_BREAK"

def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")

def load_heavy_tests(hook_dir):
    spec = importlib.util.spec_from_file_location("lib_heavy_tests", os.path.join(hook_dir, "lib_heavy_tests.py"))
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    return library

def cargo_uses_gpu(words):
    if os.path.basename(words[0]) != "cargo":
        return False
    arguments = words[1:words.index("--")] if "--" in words else words[1:]
    values = []
    for index, word in enumerate(arguments):
        if word == "--all-features":
            return True
        if word in ("--features", "-F") and index + 1 < len(arguments):
            values.append(arguments[index + 1])
        elif word.startswith("--features="):
            values.append(word.split("=", 1)[1])
        elif word.startswith("-F") and len(word) > 2:
            values.append(word[2:])
    return any(feature == "gpu" or feature.endswith("/gpu") for value in values for feature in re.split(r"[\s,]+", value) if feature)

def script_uses_gpu(command):
    name = os.path.basename(command.words[0])
    if command.launcher == "source" or command.words[1:] == ["--selftest"]:
        return False
    return name in GPU_SCRIPT_NAMES or any(fnmatch.fnmatchcase(name, pattern) for pattern in GPU_SCRIPT_PATTERNS)

def runs_merge_and_verify(heavy_tests, command):
    """合入后验证（文件头「认法」）：命令位置上的 scripts/gate/merge-and-verify.sh；source 进来的、参数只有 --selftest 的不算。"""
    if broken("ignore-merge-and-verify") or command.launcher == "source" or command.words[1:] == ["--selftest"]:
        return False
    name, relative_directory = MERGE_AND_VERIFY
    return os.path.basename(command.words[0]) == name and heavy_tests.located_in(command.words[0], command.directory, relative_directory)

def recognize(heavy_tests, text, directory, environment=None, depth=0):
    """命令文本里第一条要提醒的：(那一条的 CommandAtPosition, 重活的说明或 None, 是不是 GPU)；没有交 None。"""
    shell_words = heavy_tests.shell_words
    for command in shell_words.commands_at_command_position(text, directory, environment).commands:
        launched = heavy_tests.command_under_launcher(command.words, command.directory)
        if launched is not None:
            if depth < MAXIMUM_LAUNCHER_DEPTH:
                inner_words, inner_directory = launched
                inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
                found = recognize(heavy_tests, shlex.join(inner_words), inner_start, command.environment, depth + 1)
                if found:
                    return found
            continue
        heavy = None
        if not broken("ignore-heavy"):
            test = heavy_tests.classify(command.words, command.directory, command.environment)
            if test is not None and test.kind not in heavy_tests.COMMIT_CHECK_KINDS:
                heavy = f"{test.category}：{test.detail}"
            else:
                compiled = heavy_tests.runs_compiled_code(command.words, command.directory)
                heavy = f"跑编译出来的代码：{compiled}" if compiled else None
        if not heavy and runs_merge_and_verify(heavy_tests, command):
            heavy = "合入后验证：scripts/gate/merge-and-verify.sh 自己经内存包装起 cargo test、fmt、clippy、build"
        gpu = not broken("ignore-gpu") and (cargo_uses_gpu(command.words) or script_uses_gpu(command))
        if heavy or gpu or broken("remind-everything"):
            return command, heavy, gpu
    return None

def run_report(hook_dir, command, gpu):
    """(提醒正文的第一行之后那几行, 说明这一次报告怎么样的一句)。报告脚本的输出落进临时文件：到点停它时不被它留下的子进程握着的管道卡住。"""
    script = os.environ.get("HOST_RESOURCES_REMINDER_SCRIPT") or os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "scripts", "hosts", "host-resources.py")
    limit = float(os.environ.get("HOST_RESOURCES_REMINDER_TIME_LIMIT") or REPORT_TIME_LIMIT_SECONDS)
    arguments = [sys.executable, "-B", script] + (["--gpu"] if gpu else []) + ["--", *command.words]
    environment = dict(os.environ)
    if command.environment.get("RUN_WITH_MEMORY_CAP_KEY"):
        environment["RUN_WITH_MEMORY_CAP_KEY"] = command.environment["RUN_WITH_MEMORY_CAP_KEY"]
    with tempfile.TemporaryFile(mode="w+", encoding="utf-8") as output, tempfile.TemporaryFile(mode="w+", encoding="utf-8") as errors:
        child = subprocess.Popen(arguments, stdout=output, stderr=errors, stdin=subprocess.DEVNULL, env=environment, cwd=command.directory or None)
        try:
            code = child.wait(timeout=None if broken("no-time-limit") else limit)
        except subprocess.TimeoutExpired:
            child.kill()   # 只停自己起的这一个进程（按它的进程号）；它起的 ssh 由 ssh 自己收尾
            child.wait()
            code = None
        output.seek(0)
        errors.seek(0)
        stdout, stderr = output.read().strip(), errors.read().strip()
    if code is None:
        return stdout, f"各台此刻的资源没读到（报告脚本 {limit:g} 秒没跑完，停了它；下面是它到那时打出来的）" if stdout else f"各台此刻的资源没读到（报告脚本 {limit:g} 秒没跑完，停了它）"
    if code not in (0, 1):
        tail = stderr.splitlines()[-1][:300] if stderr else ""
        return stdout, f"各台此刻的资源没读到（报告脚本退 {code}{'：' + tail if tail else ''}）"
    return stdout, ""

def remind(hook_input, hook_dir, heavy_tests):
    """交提醒正文；不用提醒交 None。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    text = tool_input.get("command") or ""
    directory = hook_input.get("cwd") or os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()
    found = recognize(heavy_tests, text, directory)
    if found is None:
        return None
    command, heavy, gpu = found
    what = "、".join(piece for piece in [f"重活（{heavy}）" if heavy else "", "GPU" if gpu else ""] if piece) or "（弄坏开关 remind-everything）"
    stdout, problem = run_report(hook_dir, command, gpu)
    head = (f"资源提醒：这条命令认出要起{what}，先跑了 scripts/hosts/host-resources.py{' --gpu' if gpu else ''} -- {shlex.join(command.words)[:200]}；"
            "命令照常放行（这道钩子不拒、不排队），放不放得下看下面的结论，跑不起来就回报")
    return "\n".join(piece for piece in [head, problem, stdout] if piece)

def main_hook(hook_dir):
    try:
        hook_input = json.load(sys.stdin)
    except Exception:
        return 0
    if not isinstance(hook_input, dict):
        return 0
    try:
        heavy_tests = load_heavy_tests(hook_dir)
        message = remind(hook_input, hook_dir, heavy_tests)
    except Exception as error:   # 共用模块读不到、切词抛的：不拦命令，什么都不提醒
        print(f"  ! host-resources-reminder.sh 没判成（{error!r}），这条命令照常执行", file=sys.stderr)
        return 0
    if message is None:
        return 0
    print(json.dumps({"systemMessage": message, "hookSpecificOutput": {"hookEventName": "PreToolUse", "additionalContext": message}}, ensure_ascii=False))
    return 2 if broken("refuse-command") else 0

# ── 自证 ──
FAKE_REPORT = """import os, sys, time
with open(os.environ["FAKE_REPORT_LOG"], "a", encoding="utf-8") as handle:
    handle.write(" ".join(sys.argv[1:]) + "\\n")
time.sleep(float(os.environ.get("FAKE_REPORT_SLEEP") or 0))
print("假报告：本机 可用 8 线程")
print("结论：可以跑：在本机")
"""

def selftest_cells(hook_dir, root):
    failures = []
    checked = 0
    hook = os.path.join(hook_dir, "host-resources-reminder.sh")
    repository = os.path.dirname(os.path.dirname(hook_dir))
    fake = os.path.join(root, "fake-report.py")
    with open(fake, "w", encoding="utf-8") as handle:
        handle.write(FAKE_REPORT)

    def check(cell, condition, detail):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append(cell)
            print(f"  ✗ 自证 {cell}：{detail}")   # gate-lint:detail

    def feed(cell, command, **environment):
        log = os.path.join(root, f"{cell}.log")
        hook_environment = {name: value for name, value in os.environ.items() if not name.startswith(("HOST_RESOURCES_REMINDER_SCRIPT", "HOST_RESOURCES_REMINDER_TIME", "FAKE_REPORT_"))}
        hook_environment.update({"HOST_RESOURCES_REMINDER_SCRIPT": fake, "FAKE_REPORT_LOG": log, **environment})
        started = time.monotonic()
        try:
            completed = subprocess.run(["bash", hook], input=json.dumps({"tool_name": "Bash", "tool_input": {"command": command}, "cwd": repository}),
                                       capture_output=True, text=True, env=hook_environment, timeout=60)
            code, stdout = completed.returncode, completed.stdout
        except subprocess.TimeoutExpired:
            code, stdout = None, ""
        elapsed = time.monotonic() - started
        calls = open(log, encoding="utf-8").read().splitlines() if os.path.exists(log) else []
        try:
            context = json.loads(stdout)["hookSpecificOutput"]["additionalContext"] if stdout.strip() else ""
        except (ValueError, KeyError, TypeError):
            context = f"（stdout 不是那份 JSON：{stdout[:200]}）"
        return code, context, calls, elapsed

    code, context, calls, _ = feed("heavy-test", "cargo test --release -p singlefs-checker-tier --test crash_enumeration_selftest_stream")
    check("heavy-test", code == 0 and "资源提醒" in context and "结论：可以跑：在本机" in context and calls and "--gpu" not in calls[0]
          and calls[0].endswith("-- cargo test --release -p singlefs-checker-tier --test crash_enumeration_selftest_stream"),
          f"checker 档的 cargo test 应当出提醒、报告脚本不带 --gpu 带着这条命令跑、退 0；实际退 {code}，提醒「{context[:120]}」，报告脚本收到「{calls}」")
    code, context, calls, _ = feed("compiled-code", "bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-harness --test selftest_target")
    check("compiled-code", code == 0 and "资源提醒" in context and calls and calls[0].endswith("-- cargo test -p singlefs-harness --test selftest_target"),
          f"经内存包装的 harness 档 cargo test 应当出提醒、报告脚本拿剥掉包装的那一条；实际退 {code}，提醒「{context[:120]}」，报告脚本收到「{calls}」")
    code, context, calls, _ = feed("gpu-script", "cd scripts/experiments && bash ../crash/gpu-quiesce-run.sh -- true")
    check("gpu-script", code == 0 and "GPU" in context and calls and calls[0].startswith("--gpu --"),
          f"清场包装应当认成 GPU、报告脚本带 --gpu、退 0；实际退 {code}，提醒「{context[:120]}」，报告脚本收到「{calls}」")
    code, context, calls, _ = feed("gpu-feature", "cargo build --release -p singlefs-checker-tier --features verdict-store,gpu")
    check("gpu-feature", code == 0 and "GPU" in context and calls and calls[0].startswith("--gpu --"),
          f"带 gpu 特性的 cargo 应当认成 GPU、报告脚本带 --gpu、退 0；实际退 {code}，提醒「{context[:120]}」，报告脚本收到「{calls}」")
    code, context, calls, _ = feed("plain-command", "ls -l scripts && git status --short && bash scripts/crash/gpu-quiesce-run.sh --selftest")
    check("plain-command", code == 0 and context == "" and not calls,
          f"普通命令（与清场包装的 --selftest）应当不出提醒、不跑报告脚本；实际退 {code}，提醒「{context[:120]}」，报告脚本收到「{calls}」")
    code, context, calls, _ = feed("merge-and-verify", "bash scripts/gate/merge-and-verify.sh /tmp/selftest-patch-directory")
    selftest_code, selftest_context, selftest_calls, _ = feed("merge-and-verify-selftest", "bash scripts/gate/merge-and-verify.sh --selftest")
    check("merge-and-verify", code == 0 and "资源提醒" in context and "合入后验证" in context and calls
          and calls[0].endswith("-- scripts/gate/merge-and-verify.sh /tmp/selftest-patch-directory")
          and selftest_code == 0 and selftest_context == "" and not selftest_calls,
          f"合入后验证 merge-and-verify.sh <目录> 应当出提醒、报告脚本带着这一条跑、退 0，它的 --selftest 不出；"
          f"实际 <目录> 那一条退 {code}，提醒「{context[:120]}」，报告脚本收到「{calls}」；--selftest 那一条退 {selftest_code}，提醒「{selftest_context[:120]}」，报告脚本收到「{selftest_calls}」")
    code, context, calls, elapsed = feed("report-timeout", "cargo test -p singlefs-harness --test selftest_target", FAKE_REPORT_SLEEP="8",
                                         HOST_RESOURCES_REMINDER_TIME_LIMIT="1")
    check("report-timeout", code == 0 and elapsed < 6 and "没读到" in context and "1 秒没跑完" in context,
          f"报告脚本 8 秒才完、时限 1 秒：应当 6 秒内放行（退 0）、提醒写「没读到」；实际退 {code}、用了 {elapsed:.1f} 秒，提醒「{context[:160]}」")
    return failures, checked

SWITCH_EXPECTED_CELLS = {
    "ignore-heavy": ["heavy-test", "compiled-code", "report-timeout"],
    "ignore-merge-and-verify": ["merge-and-verify"],
    "ignore-gpu": ["gpu-script", "gpu-feature"],
    "remind-everything": ["plain-command", "merge-and-verify"],
    "no-time-limit": ["report-timeout"],
    "refuse-command": ["heavy-test", "compiled-code", "gpu-script", "gpu-feature", "report-timeout", "merge-and-verify"],
}

def cells_in_scratch(hook_dir):
    root = tempfile.mkdtemp(prefix="host-resources-reminder-selftest-")
    try:
        return selftest_cells(hook_dir, root)
    finally:
        shutil.rmtree(root, ignore_errors=True)

def selftest(hook_dir):
    failures, checked = cells_in_scratch(hook_dir)
    if failures:
        print(f"  ✗ host-resources-reminder.sh 自证 {len(failures)} 格不对（查了 {checked} 格）：{'、'.join(failures)}")
        print("  → 怎么办：照上面逐格改 recognize / cargo_uses_gpu / script_uses_gpu / run_report；HOST_RESOURCES_REMINDER_BREAK 设着的话本来就该红")
        return 1
    hook = os.path.join(hook_dir, "host-resources-reminder.sh")
    clean = {name: value for name, value in os.environ.items() if name != BREAK_VARIABLE}

    def one(switch):
        completed = subprocess.run(["bash", hook, "--selftest-cells"], capture_output=True, text=True, env=dict(clean, **{BREAK_VARIABLE: switch}),
                                   timeout=600, stdin=subprocess.DEVNULL)
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
    print(f"  ✓ host-resources-reminder.sh 自证通过（查了 {checked} 格：checker 档的 cargo test、经内存包装的 harness 档 cargo test、清场包装、"
          f"带 gpu 特性的 cargo、合入后验证 merge-and-verify.sh 都出提醒且照常放行，它的 --selftest 与普通命令不出，报告脚本超时照样放行并写没读到；弄坏开关 {len(results)} 个逐个打开都红在点名的格）")
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
