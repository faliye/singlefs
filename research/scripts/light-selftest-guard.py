#!/usr/bin/env python3
# admission: always 每一次调都盯这一趟起的那条自证跑的时候起了什么，上一次盯过不替这一次作保
# run-condition: command python3 bash
# gate-similar: agent-watch.py 它的「重型测试没带前缀」也扫进程树、照同一份登记表认登记成轻的自证，但那一类恰好在登记成轻的自证底下整棵不判，只报不拒、按复检间隔扫整个 Claude 实例；这里只盯一条自证，跑的那段时间里把 PATH 上的 cargo、rustc 换成会拒的替身、按短间隔扫它自己那棵进程树，判红交给调它的门禁格
# gate-similar: heavy-test-guard.sh 它在执行前按命令文本判轻重，登记成 light 的单独 --selftest 直接放行，看不进脚本里起的进程；这里在运行中看进程，两份判的不是同一个时刻
# gate-similar: code-tooling.sh 它的 research-script-selftests 格经这一份起 runner 表里的每一条：判法与替身在这里，阶段只收这一份写的判定文件
"""盯一条登记成 light 的自证：它跑的时候起了真 cargo / rustc、真清场或私有清场脚本就判红。

用法：
  light-selftest-guard.py [--only-if-light] [--registry-root <仓根>] [--verdict-file <文件>] [--interval <秒>] -- <命令…>
  light-selftest-guard.py --selftest

判据（.claude/hooks/selftest-registry.tsv 登记成 light 的自证，跑的时候它整棵进程树里出现下面任一样就判红）：
  ① 真的 Rust 工具链：可执行文件（/proc/<pid>/exe 解开符号链接）落在 rustup 的 toolchains 目录（${RUSTUP_HOME:-~/.rustup}/toolchains）、
     cargo 的 bin 目录（${CARGO_HOME:-~/.cargo}/bin）之下，或正是 PATH 上 cargo、rustc 解到的那一份，参数又不是版本查询
     （-V、--version、-vV、version 这几种，前面可以带 +<工具链>；rustc 的 --print <项> 也算查询）。
     另在 PATH 最前面放 cargo、rustc 两个替身：版本查询交给真的那一份，别的一律拒（退 97，真的那一份不起）、记一笔，记到的也判红。
  ② 真清场：起的脚本是某棵仓树里的 research/scripts/gpu-quiesce-run.sh（路径以这一段结尾；单独一个 --selftest 的除外），
     它环境里的 SINGLEFS_MULTI_HOST_CONFIG 没设、是空的，或解开之后正是主工作树根的 multi-host.env（它读的就是真多机配置里的清场命令）。
  ③ 私有清场脚本：参数里有一段落在 ~/.config/singlefs/quiesce 开头的路径上（私有的清场、回读、复原脚本放在那里）。
  假的不算：自证自己的样本工具目录里的假 cargo（解释器起的脚本、可执行文件不在上面那几处），别处路径上叫 gpu-quiesce-run.sh 的替身，
  SINGLEFS_MULTI_HOST_CONFIG 指到临时目录里一份假配置的真包装，都不判。
  另一份 light-selftest-guard.py 起的那一支整棵不扫：那一份自己判（它的自证故意起这几样）。

射程（管不到的，靠写自证的人）：
  进程树按 --interval 秒（默认 0.1）扫一次：活不到一个间隔、又不经 PATH 起的（绝对路径起、一闪就退的 cargo，一闪而过的私有脚本）可能漏掉；
  脱离这棵树的（setsid 之后被 init 收养、systemd 单元里起的、ssh 到第二台上起的）看不见；在同一个 shell 里 source 进来执行的清场命令
  （peer-host-lib.sh 那种）没有自己的进程，只认得出它另起的私有脚本；替身只拦经 PATH 找到的 cargo、rustc，把 PATH 整个换掉的照漏，
  只靠扫进程树。只判、不停任何进程：扫到真重活时它照样跑完，红在这一趟的退出码与判定文件上；经 PATH 的那几次被替身拒、根本没起。

本次未查（不是判过了）：
  --only-if-light 时命令不是登记成 light 的自证（登记成 heavy、heavy-if-set 而那个变量设了、表外的、带了 --selftest 之外参数的、
  起的不是一份脚本），不盯：判定文件写 not-watched 与理由，原样 exec 那条命令、退它的退出码。
  /proc/<pid>/task/*/children 读不到（内核没开 CONFIG_PROC_CHILDREN）时只有替身那一半，判定文件写 scan=no，成功句写明进程树没扫。

判定文件（--verdict-file，调用方逐份回读、数份数）：一行，制表符分隔。
  watched	findings=<N>	scan=yes|no	examined=<扫到的进程形态数>	exit=<命令的退出码>
  not-watched	<为什么不盯>
退出码：命令起了而没见真重活，交命令自己的退出码；见了真重活退 3（命令自己退几写进 ✗ 那一句与判定文件）；用法错退 2；
  --only-if-light 而不盯时就是那条命令的退出码（exec 过去了）。

弄坏开关 LIGHT_SELFTEST_GUARD_BREAK=<项>（逗号连几个；只在 --selftest 里认，平常跑设了也不认、打一句提醒）。
逐项写开关名、弄坏哪一处，下一行起写打开后 --selftest 哪几格判错（括号里是那一格「✗ 自证第 <号> 格」之后那一句的句首）：
  no-path-shims  PATH 上不放替身
      第 2 格（「登记成 light 的假自证经 PATH 起真 cargo build」）
  no-process-scan  不扫进程树
      第 4 格（「绝对路径起的真工具链」）、第 5 格（「真清场」）、第 6 格（「私有清场脚本」）
  version-query-flagged  版本查询也当真重活（替身拒它、扫到也记）
      第 3 格（「只用假 cargo、真 cargo 与 rustc 只问版本的绿样本」）
  quiesce-always-fake  gpu-quiesce-run.sh 不看 SINGLEFS_MULTI_HOST_CONFIG，一律当假配置
      第 5 格（「真清场」）、第 8 格（「退出途中读到的空环境应当不报」）
  private-script-ignored  不认私有清场脚本
      第 6 格（「私有清场脚本」）
  nested-guard-scanned  另一份守卫起的那一支也扫
      第 7 格（「另一份守卫起的那一支」）
  light-registry-ignored  --only-if-light 一律当不是登记成 light 的、不盯
      第 1 格（「照登记表分盯不盯」）、第 2 格（「登记成 light 的假自证经 PATH 起真 cargo build」）
  empty-environment-as-unset  读 /proc/<pid>/environ 读到空的、参数也读不到了（两次读之间它退出了）照「没设配置」报真清场
      第 8 格（「退出途中读到的空环境应当不报」）
"""
import importlib.util
import os
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
SCRIPT_PATH = os.path.realpath(__file__)
SCRIPT_DIRECTORY = os.path.dirname(SCRIPT_PATH)
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402

HEAVY_TESTS_LIBRARY_PATH = os.path.join(REPOSITORY_ROOT, ".claude", "hooks", "lib_heavy_tests.py")
REGISTRY_RELATIVE_PATH = os.path.join(".claude", "hooks", "selftest-registry.tsv")
SHIMMED_TOOLS = ("cargo", "rustc")
SHIM_MARKER = ".light-selftest-guard-shims"
SHIM_REFUSED_EXIT = 97
FINDINGS_EXIT = 3
USAGE_EXIT = 2
DEFAULT_INTERVAL_SECONDS = 0.1
QUIESCE_WRAPPER_SUFFIX = os.path.join("research", "scripts", "gpu-quiesce-run.sh")
PRIVATE_QUIESCE_PREFIX = "quiesce"
SHELL_NAMES = {"bash", "sh", "dash", "zsh", "ksh"}
VERSION_QUERIES = {("-V",), ("--version",), ("-vV",), ("-Vv",), ("-V", "-v"), ("-v", "-V"),
                   ("--version", "--verbose"), ("--verbose", "--version"), ("version",)}
BREAK_VARIABLE = "LIGHT_SELFTEST_GUARD_BREAK"
KNOWN_BREAKS = ("no-path-shims", "no-process-scan", "version-query-flagged", "quiesce-always-fake", "private-script-ignored",
                "nested-guard-scanned", "light-registry-ignored", "empty-environment-as-unset")
SELFTEST_CELL_COUNT = 8

SHIM_TEMPLATE = r'''#!/usr/bin/env bash
# light-selftest-guard.py 放在 PATH 最前面的替身（{tool}）：版本查询交给真的那一份，别的一律拒、记一笔、退 {refused}，真的那一份不起
real={real}
arguments=("$@")
[[ "${{arguments[0]:-}}" == +* ]] && arguments=("${{arguments[@]:1}}")
query=0
case "${{arguments[*]-}}" in
  -V|--version|-vV|-Vv|"-V -v"|"-v -V"|"--version --verbose"|"--verbose --version"|version|--print=*) query={query_allowed} ;;
esac
if [[ "${{#arguments[@]}}" == 2 && "${{arguments[0]}}" == --print ]]; then query={query_allowed}; fi
if ((query)); then
  printf '%s\t%s\n' "$$" "{tool} $*" >> "${{LIGHT_SELFTEST_GUARD_PASSED:-/dev/null}}"
  [[ -n "$real" ]] && exec "$real" "$@"
  echo "  ✗ light-selftest-guard 的替身：PATH 上找不到真的 {tool}，版本查询答不了" >&2
  echo "     → 怎么办：装好 Rust 工具链（bash .claude/scripts/env.sh 会报缺什么）再跑" >&2
  exit 127
fi
printf '%s\t%s\t%s\n' "$$" "$PWD" "{tool} $*" >> "${{LIGHT_SELFTEST_GUARD_RECORD:-/dev/null}}"
echo "  ✗ light-selftest-guard 的替身拒了：登记成 light 的自证起了真 {tool} $*（版本查询之外一律拒，真的那一份没起）" >&2
echo "     → 怎么办：自证里把这一步换成假的（假 {tool} 放进自证自己的样本工具目录、排在 PATH 最前面），或在 .claude/hooks/selftest-registry.tsv 把它改登记成 heavy" >&2
exit {refused}
'''


class Settings:
    """怎么认「真的」：工具链目录与那几份可执行文件、主工作树根的多机配置、私有清场脚本的目录。平常跑取 default_settings()，自证换成临时目录里的。"""

    def __init__(self, toolchain_roots, toolchain_binaries, real_configuration, private_directory):
        self.toolchain_roots = [os.path.realpath(root) for root in toolchain_roots]
        self.toolchain_binaries = {os.path.realpath(path) for path in toolchain_binaries}
        self.real_configuration = os.path.realpath(real_configuration) if real_configuration else ""
        self.private_directory = os.path.realpath(private_directory)


def path_without_shims(path_text):
    """PATH 里去掉守卫的替身目录（这一份与外层守卫放的，认目录里的标记文件）。"""
    return os.pathsep.join(entry for entry in path_text.split(os.pathsep) if entry and not os.path.exists(os.path.join(entry, SHIM_MARKER)))


def real_tool(tool, path_text):
    found = shutil.which(tool, path=path_without_shims(path_text))
    return os.path.realpath(found) if found else ""


def main_worktree_configuration():
    """主工作树根的 multi-host.env（主工作树的根按 git common-dir 取，同 gpu-quiesce-run.sh）；取不到就用这份脚本所在仓的根。"""
    completed = subprocess.run(["git", "-C", REPOSITORY_ROOT, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                               capture_output=True, text=True, stdin=subprocess.DEVNULL)
    common = completed.stdout.strip() if completed.returncode == 0 else ""
    return os.path.join(os.path.dirname(common) if common else REPOSITORY_ROOT, "multi-host.env")


def default_settings():
    home = os.path.expanduser("~")
    roots = [os.path.join(os.environ.get("RUSTUP_HOME") or os.path.join(home, ".rustup"), "toolchains"),
             os.path.join(os.environ.get("CARGO_HOME") or os.path.join(home, ".cargo"), "bin")]
    binaries = [path for path in (real_tool(tool, os.environ.get("PATH", "")) for tool in SHIMMED_TOOLS) if path]
    return Settings(roots, binaries, main_worktree_configuration(), os.path.join(home, ".config", "singlefs"))


# ── 进程树 ───────────────────────────────────────────────────────────────────

def child_pids(pid):
    """pid 的直接子进程（逐个线程的 /proc/<pid>/task/<tid>/children 合起来）；读不到交 None。"""
    try:
        tasks = os.listdir(f"/proc/{pid}/task")
    except OSError:
        return None
    children, readable = [], False
    for task in tasks:
        try:
            with open(f"/proc/{pid}/task/{task}/children", encoding="ascii") as handle:
                children.extend(int(word) for word in handle.read().split())
            readable = True
        except (OSError, ValueError):
            continue
    return children if readable else None


def process_start(pid):
    with open(f"/proc/{pid}/stat", "rb") as handle:
        data = handle.read()
    return int(data[data.rindex(b")") + 2:].split()[19])   # 第 22 栏 starttime（右括号之后从第 3 栏数起）


def process_arguments(pid):
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as handle:
            raw = handle.read()
    except OSError:
        return None
    return [os.fsdecode(part) for part in raw.split(b"\0")[:-1]] if raw else None


def process_executable(pid):
    try:
        return os.path.realpath(os.readlink(f"/proc/{pid}/exe").removesuffix(" (deleted)"))
    except OSError:
        return ""


def process_environment(pid):
    try:
        with open(f"/proc/{pid}/environ", "rb") as handle:
            raw = handle.read()
    except OSError:
        return None
    return dict(os.fsdecode(entry).partition("=")[::2] for entry in raw.split(b"\0") if b"=" in entry)


def script_of(arguments):
    """进程起的是哪个脚本：(脚本词, 它的参数)；bash / sh 这类取解释器后面第一个不是选项的词，-c 的一段代码交 (None, [])。"""
    if os.path.basename(arguments[0]) not in SHELL_NAMES:
        return arguments[0], arguments[1:]
    for position in range(1, len(arguments)):
        word = arguments[position]
        if word.startswith("-") and word not in ("-", "--"):
            if not word.startswith("--") and "c" in word[1:]:
                return None, []
            continue
        return word, arguments[position + 1:]
    return None, []


def is_nested_guard(arguments):
    """这个进程是不是另一份 light-selftest-guard.py（python 起它，或直接执行它；哪棵树里的那一份都算，按文件名认）。"""
    return any(os.path.basename(word) == os.path.basename(SCRIPT_PATH) for word in arguments[:3])


def is_version_query(arguments):
    rest = list(arguments)
    if rest and rest[0].startswith("+"):
        rest = rest[1:]
    return tuple(rest) in VERSION_QUERIES or (len(rest) == 2 and rest[0] == "--print") or (len(rest) == 1 and rest[0].startswith("--print="))


def is_real_toolchain(executable, settings):
    return bool(executable) and (executable in settings.toolchain_binaries
                                 or any(executable.startswith(root + os.sep) for root in settings.toolchain_roots))


def judge_process(pid, arguments, settings, broken):
    """一个进程此刻的样子里有哪几样真重活：[(类别, 说明)]。"""
    found = []
    executable = process_executable(pid)
    if is_real_toolchain(executable, settings) and ("version-query-flagged" in broken or not is_version_query(arguments[1:])):
        found.append(("真工具链", f"{os.path.basename(executable)}（{executable}）：{shlex.join(arguments)}"))
    script, rest = script_of(arguments)
    if (script and os.path.normpath(script).endswith(os.sep + QUIESCE_WRAPPER_SUFFIX) and rest[:1] != ["--selftest"]
            and "quiesce-always-fake" not in broken):
        environment = process_environment(pid)
        # 读到空的环境而参数也读不到了（或变了），是它恰好在两次读之间退出了（内存已经放掉），不当「没设配置」：下一次扫照判
        exited_meanwhile = not environment and process_arguments(pid) != arguments and "empty-environment-as-unset" not in broken
        if environment is not None and not exited_meanwhile:
            configured = environment.get("SINGLEFS_MULTI_HOST_CONFIG", "")
            if not configured or os.path.realpath(configured) == settings.real_configuration:
                found.append(("真清场", f"{shlex.join(arguments)}（SINGLEFS_MULTI_HOST_CONFIG={configured or '没设'}，读的是真多机配置 {settings.real_configuration}）"))
    if "private-script-ignored" not in broken:
        private_prefix = os.path.join(settings.private_directory, PRIVATE_QUIESCE_PREFIX)
        if any(private_prefix in argument for argument in arguments):
            found.append(("私有清场脚本", shlex.join(arguments)))
    return found


class Watch:
    """一条命令那棵进程树的扫描状态：见过的进程形态（进程号、起始时刻、参数）与扫到的真重活。"""

    def __init__(self, settings, broken):
        self.settings, self.broken = settings, broken
        self.seen, self.findings, self.scan_available = set(), [], True

    def scan(self, root_pid):
        """从这一份起的那个进程本身扫起（它 exec 成别的样子也算），逐层往下。"""
        if child_pids(root_pid) is None and os.path.exists(f"/proc/{root_pid}"):
            self.scan_available = False
            return
        stack = [root_pid]
        while stack:
            pid = stack.pop()
            arguments = process_arguments(pid)
            if not arguments:
                continue
            if is_nested_guard(arguments) and "nested-guard-scanned" not in self.broken:
                continue   # 另一份守卫：它那一支它自己判，整棵不扫
            stack.extend(child_pids(pid) or [])
            try:
                key = (pid, process_start(pid), tuple(arguments))
            except (OSError, ValueError, IndexError):
                continue
            if key in self.seen:
                continue
            self.seen.add(key)
            for kind, text in judge_process(pid, arguments, self.settings, self.broken):
                if (pid, kind) not in {(finding[0], finding[1]) for finding in self.findings}:
                    self.findings.append((pid, kind, text))


def read_lines(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            return [line.rstrip("\n") for line in handle if line.strip()]
    except OSError:
        return []


def write_shims(directory, path_text, broken):
    with open(os.path.join(directory, SHIM_MARKER), "w", encoding="utf-8") as handle:
        handle.write("light-selftest-guard.py 的替身目录\n")
    for tool in SHIMMED_TOOLS:
        path = os.path.join(directory, tool)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(SHIM_TEMPLATE.format(tool=tool, real=shlex.quote(real_tool(tool, path_text)), refused=SHIM_REFUSED_EXIT,
                                              query_allowed=0 if "version-query-flagged" in broken else 1))
        os.chmod(path, 0o755)


def guard(command, settings, broken=frozenset(), interval=DEFAULT_INTERVAL_SECONDS, verdict_file=None, environment=None, directory=None, quiet=False,
          output=None):
    """起 command、盯到它退出：交 (退出码, 扫到的 [(进程号, 类别, 说明)], 判定行)。stdout、stderr 原样交给命令（output 给了就都接到它上）；
    quiet 时不打判定句。"""
    environment = dict(os.environ if environment is None else environment)
    environment.pop(BREAK_VARIABLE, None)
    shim_directory = tempfile.mkdtemp(prefix="light-selftest-guard-")
    record, passed = os.path.join(shim_directory, "refused.tsv"), os.path.join(shim_directory, "passed.tsv")
    try:
        if "no-path-shims" not in broken:
            write_shims(shim_directory, environment.get("PATH", ""), broken)
            environment["PATH"] = shim_directory + os.pathsep + environment.get("PATH", "")
        environment.update(LIGHT_SELFTEST_GUARD_RECORD=record, LIGHT_SELFTEST_GUARD_PASSED=passed)
        watch = Watch(settings, broken)
        process = subprocess.Popen(command, env=environment, cwd=directory, stdout=output, stderr=output)

        def forward(signal_number, _frame):
            if process.poll() is None:
                process.send_signal(signal_number)   # 只发给自己起的那一个子进程

        previous = {number: signal.signal(number, forward) for number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)}
        try:
            while True:
                if "no-process-scan" not in broken:
                    watch.scan(process.pid)
                if process.poll() is not None:
                    break
                time.sleep(interval)
        finally:
            for number, handler in previous.items():
                signal.signal(number, handler)
        exit_code = process.returncode
        findings = list(watch.findings)
        for line in read_lines(record):
            pid, _tab, rest = line.partition("\t")
            where, _tab, invocation = rest.partition("\t")
            findings.append((pid, "替身拒了的真工具链", f"{invocation}（当前目录 {where}；经 PATH 找的那一次被替身拒了，真的那一份没起）"))
        passed_count = len(read_lines(passed))
    finally:
        shutil.rmtree(shim_directory, ignore_errors=True)
    scanned = "no-process-scan" not in broken and watch.scan_available
    verdict = (f"watched\tfindings={len(findings)}\tscan={'yes' if scanned else 'no'}\texamined={len(watch.seen)}\texit={exit_code}")
    if verdict_file:
        with open(verdict_file, "w", encoding="utf-8") as handle:
            handle.write(verdict + "\n")
    shown = shlex.join(command)
    if findings:
        if not quiet:
            print(f"  ✗ light-selftest-guard：{shown} 是登记成 light 的自证，跑的时候起了 {len(findings)} 处真重活（命令自己退 {exit_code}）：", file=sys.stderr)
            for pid, kind, text in findings:
                print(f"      进程 {pid}：{kind}：{text}", file=sys.stderr)   # gate-lint:detail
            print("  → 怎么办：自证里把那一步换成假的（假 cargo、rustc 放进自证自己的样本工具目录、排在 PATH 最前面；清场把 SINGLEFS_MULTI_HOST_CONFIG "
                  "指到临时目录里只写了假清场命令的一份），或在 .claude/hooks/selftest-registry.tsv 把它改登记成 heavy / heavy-if-set:<变量>、理由以「偏离上游：」起头",
                  file=sys.stderr)
        return FINDINGS_EXIT, findings, verdict
    if not quiet:
        scan_text = (f"进程树每 {interval} 秒扫一次、扫到 {len(watch.seen)} 个进程形态" if scanned
                     else "进程树没扫（/proc/<pid>/task/*/children 读不到，本次未查这一半，只有替身那一半）")
        print(f"  ✓ light-selftest-guard：{shown} 跑完（退 {exit_code}），{scan_text}，替身放过 {passed_count} 次版本查询，"
              "没见真 cargo / rustc、真清场与私有清场脚本", file=sys.stderr)
    return exit_code, findings, verdict


# ── 照登记表分盯不盯 ─────────────────────────────────────────────────────────

def heavy_tests_library():
    specification = importlib.util.spec_from_file_location("lib_heavy_tests_for_light_selftest_guard", HEAVY_TESTS_LIBRARY_PATH)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def light_judgement(command, directory, registry_root, environment, broken):
    """(盯不盯, 一句理由)：命令起的脚本照 lib_heavy_tests.py 的 classify_selftest 判，是登记成 light 的单独 --selftest 才盯。
    registry_root 给了就读那棵树的登记表、命令词要解到那棵树里的那一份（样本根有自己的登记表）。"""
    if "light-registry-ignored" in broken:
        return False, "弄坏开关 light-registry-ignored：一律当不是登记成 light 的"
    try:
        library = heavy_tests_library()
    except Exception as error:   # 文件不在、语法错、它导入 lib_shell_words 失败：认不出轻重，照登记成 light 的盯（宁可多盯）
        return True, f"导入 {HEAVY_TESTS_LIBRARY_PATH} 失败（{type(error).__name__}：{error}），认不出轻重，照登记成 light 的盯"
    shell_words, name = library.shell_words, os.path.basename(command[0])
    script, rest = command[0], command[1:]
    if name in shell_words.SHELL_NAMES:
        invocation = shell_words.shell_invocation(command[1:])
        launched = invocation.code_string is None and invocation.script_index is not None
    elif shell_words.PYTHON_NAME.match(name):
        invocation = shell_words.python_invocation(command[1:])
        launched = invocation.code_string is None and invocation.module is None and invocation.script_index is not None
    else:
        invocation, launched = None, True
    if invocation is not None and launched and 1 + invocation.script_index >= len(command):
        launched = False
    if invocation is not None:
        script, rest = (command[1 + invocation.script_index], command[2 + invocation.script_index:]) if launched else (None, [])
    if script is None:
        return False, "起的不是一份脚本（bash -c、python -c / -m 或从标准输入读）"
    saved = (library.SELFTEST_REGISTRY_PATH, library.SELFTEST_REGISTRY_ROOT)
    if registry_root:
        library.SELFTEST_REGISTRY_ROOT = os.path.realpath(registry_root)
        library.SELFTEST_REGISTRY_PATH = os.path.join(library.SELFTEST_REGISTRY_ROOT, REGISTRY_RELATIVE_PATH)
    try:
        is_selftest, test = library.classify_selftest(script, rest, directory, environment)
        registry = library.SELFTEST_REGISTRY_PATH
    finally:
        library.SELFTEST_REGISTRY_PATH, library.SELFTEST_REGISTRY_ROOT = saved
    if not is_selftest:
        return False, f"{script} 不是登记表管的自证（不在 research/scripts/、.claude/hooks/、.claude/gate.d/lib/ 下，或没带 --selftest）"
    if test is not None:
        return False, f"不是登记成 light 的：{test.detail}"
    return True, f"{script} 在 {registry} 里登记成 light（或 heavy-if-set 而那个变量没设）"


# ── 入口 ─────────────────────────────────────────────────────────────────────

def parse(arguments):
    """交 (选项, 命令) 或 (None, 出错的一句)。"""
    options = {"only_if_light": False, "registry_root": "", "verdict_file": "", "interval": DEFAULT_INTERVAL_SECONDS}
    position = 0
    while position < len(arguments):
        word = arguments[position]
        if word == "--":
            command = arguments[position + 1:]
            return (options, command) if command else (None, "-- 后面没有命令")
        if word == "--only-if-light":
            options["only_if_light"] = True
            position += 1
            continue
        if word in ("--registry-root", "--verdict-file", "--interval") and position + 1 < len(arguments):
            value = arguments[position + 1]
            if word == "--interval":
                try:
                    options["interval"] = float(value)
                except ValueError:
                    return None, f"--interval 要一个秒数，给的是「{value}」"
                if not 0 < options["interval"] <= 5:
                    return None, f"--interval 要在 0 到 5 秒之间，给的是「{value}」"
            else:
                options[word[2:].replace("-", "_")] = value
            position += 2
            continue
        return None, f"认不出的参数「{word}」"
    return None, "没给 -- <命令…>"


def own_arguments(arguments):
    """-- 之前那几个是这一份自己的（preflight 只看它们：命令里的 --force 是命令自己的，不摘）。"""
    return arguments[:arguments.index("--")] if "--" in arguments else list(arguments)


def main(arguments):
    if arguments == ["--selftest"]:
        return run_selftest()
    own = [word for word in own_arguments(arguments) if word != "--force"]   # preflight 已经认过它
    options, command = parse(own + arguments[len(own_arguments(arguments)):])
    if options is None:
        print(f"  ✗ light-selftest-guard.py 用法不对：{command}", file=sys.stderr)
        print("     → 怎么办：写成 light-selftest-guard.py [--only-if-light] [--registry-root <仓根>] [--verdict-file <文件>] [--interval <秒>] -- <命令…>", file=sys.stderr)
        return USAGE_EXIT
    if os.environ.get(BREAK_VARIABLE):
        print(f"  ! {BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 只在 --selftest 里认，这一趟不认（照常盯）", file=sys.stderr)
    if options["only_if_light"]:
        watched, reason = light_judgement(command, os.getcwd(), options["registry_root"], dict(os.environ), frozenset())
        if not watched:
            if options["verdict_file"]:
                with open(options["verdict_file"], "w", encoding="utf-8") as handle:
                    handle.write(f"not-watched\t{reason}\n")
            print(f"  · light-selftest-guard：{shlex.join(command)} 本次未盯（{reason}），原样起", file=sys.stderr)
            sys.stderr.flush()
            try:
                os.execvp(command[0], command)
            except OSError as error:
                print(f"  ✗ light-selftest-guard：起不了 {command[0]}（{error.strerror}）", file=sys.stderr)
                print("     → 怎么办：看命令的第一个词在不在 PATH 上、有没有执行权限", file=sys.stderr)
                return 127
    exit_code, _findings, _verdict = guard(command, default_settings(), interval=options["interval"], verdict_file=options["verdict_file"] or None)
    return exit_code


# ── 自证 ─────────────────────────────────────────────────────────────────────

def selftest_write(path, text, executable=False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    if executable:
        os.chmod(path, 0o755)


def run_selftest():
    broken = frozenset(item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item)
    unknown = sorted(broken - set(KNOWN_BREAKS))
    if unknown:
        print(f"  ✗ {BREAK_VARIABLE} 里有认不出的开关：{'、'.join(unknown)}", file=sys.stderr)
        print(f"     → 怎么办：只认 {'、'.join(KNOWN_BREAKS)}（文件头「弄坏开关」一段）", file=sys.stderr)
        return USAGE_EXIT
    work = tempfile.mkdtemp(prefix="light-selftest-guard-selftest-")
    failures, cells = [], 0
    try:
        cells, failures = selftest_cells(work, broken)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    if failures:
        for number, sentence in failures:
            print(f"  ✗ 自证第 {number} 格：{sentence}", file=sys.stderr)   # gate-lint:detail
        print(f"  ✗ light-selftest-guard.py 自证 {cells} 格里 {len(failures)} 格判错", file=sys.stderr)
        print("     → 怎么办：设了 LIGHT_SELFTEST_GUARD_BREAK 的，这就是要的结果（文件头「弄坏开关」点名的那几格）；没设的，照判错那一格看 "
              "light_judgement、write_shims、Watch.scan 与 judge_process 里对应的一段", file=sys.stderr)
        return 1
    print(f"  ✓ light-selftest-guard.py 自证 {cells} 格都对（照登记表分盯不盯、登记成 light 的假自证经 PATH 起真 cargo build 被替身拒并判红、"
          "只用假 cargo 与只问版本的绿、绝对路径起的真工具链扫得到、真清场与假配置分得开、私有清场脚本认得出、另一份守卫那一支不扫、退出途中读到的空环境不当没设配置）")
    return 0


def selftest_cells(work, broken):
    """八格；交 (格数, [(格号, 判错的一句)])。真家伙只起 cargo / rustc 的版本查询与一条马上报错退出的 cargo build（清单不在）。"""
    failures = []
    quiet_settings = Settings([os.path.join(work, "toolchain")], [], os.path.join(work, "main-worktree", "multi-host.env"), os.path.join(work, "private"))
    environment = {key: value for key, value in os.environ.items() if key not in ("SINGLEFS_MULTI_HOST_CONFIG", BREAK_VARIABLE)}
    repository = os.path.join(work, "repository")
    registry = os.path.join(repository, REGISTRY_RELATIVE_PATH)
    selftest_write(registry, "research/scripts/fake-light-real-cargo.sh\tlight\t# 自证造的：经 PATH 起真 cargo build\n"
                             "research/scripts/fake-light-fake-cargo.sh\tlight\t# 自证造的：只用自己的假 cargo，真的只问版本\n"
                             "research/scripts/fake-heavy.sh\theavy\t# 偏离上游：自证造的登记成 heavy 的一份\n")
    missing_manifest = os.path.join(work, "no-such-crate", "Cargo.toml")
    selftest_write(os.path.join(repository, "research", "scripts", "fake-light-real-cargo.sh"),
                   f"#!/usr/bin/env bash\ncargo build --manifest-path {shlex.quote(missing_manifest)} >/dev/null 2>&1\nexit 0\n", executable=True)
    selftest_write(os.path.join(repository, "sample-tools", "cargo"), "#!/usr/bin/env bash\necho \"假 cargo $*\"\n", executable=True)
    selftest_write(os.path.join(repository, "research", "scripts", "fake-light-fake-cargo.sh"),
                   "#!/usr/bin/env bash\nPATH=\"$(dirname \"$0\")/../../sample-tools:$PATH\" cargo build --release >/dev/null\n"
                   "if command -v cargo >/dev/null; then cargo --version >/dev/null 2>&1; fi\n"
                   "if command -v rustc >/dev/null; then rustc -V >/dev/null 2>&1; fi\nexit 5\n", executable=True)
    selftest_write(os.path.join(repository, "research", "scripts", "fake-heavy.sh"), "#!/usr/bin/env bash\nexit 0\n", executable=True)

    def judged(command):
        return light_judgement(command, repository, repository, environment, broken)[0]

    # 第 1 格：照登记表分盯不盯（样本仓自己的登记表），不是登记成 light 的原样 exec、判定文件写 not-watched
    verdict_file = os.path.join(work, "not-watched.verdict")
    exec_run = subprocess.run([sys.executable, SCRIPT_PATH, "--only-if-light", "--registry-root", repository, "--verdict-file", verdict_file,
                               "--", "bash", "-c", "exit 4"], cwd=repository, env=environment, capture_output=True, text=True)
    verdict_line = (read_lines(verdict_file) or [""])[0]
    if not (judged(["bash", "research/scripts/fake-light-real-cargo.sh", "--selftest"])
            and not judged(["bash", "research/scripts/fake-heavy.sh", "--selftest"])
            and not judged(["bash", "research/scripts/fake-light-real-cargo.sh", "--selftest", "--extra"])
            and not judged(["bash", "research/scripts/not-registered.sh", "--selftest"])
            and exec_run.returncode == 4 and verdict_line.startswith("not-watched\t")):
        failures.append((1, f"照登记表分盯不盯不对（不是登记成 light 的那一趟退 {exec_run.returncode}、判定「{verdict_line}」）"))

    # 第 2 格（红样本）：登记成 light 的假自证经 PATH 起真 cargo build：替身拒了、记一笔，整趟退 3、判定 findings=1
    verdict_file = os.path.join(work, "real-cargo.verdict")
    command = ["bash", "research/scripts/fake-light-real-cargo.sh", "--selftest"]
    exit_code, findings = FINDINGS_EXIT, []
    if judged(command):
        exit_code, findings, _verdict = guard(command, quiet_settings, broken, 0.02, verdict_file, environment, repository, quiet=True)
    refused = [finding for finding in findings if finding[1] == "替身拒了的真工具链" and "cargo build" in finding[2]]
    if not (judged(command) and exit_code == FINDINGS_EXIT and refused and (read_lines(verdict_file) or [""])[0].startswith("watched\tfindings=1\t")):
        failures.append((2, f"登记成 light 的假自证经 PATH 起真 cargo build，应当被替身拒、判红退 3（实际退 {exit_code}，扫到 {findings}）"))

    # 第 3 格（绿样本）：只用自己的假 cargo，真 rustc 只问版本：退命令自己的 5、一处都不报
    exit_code, findings, verdict = guard(["bash", "research/scripts/fake-light-fake-cargo.sh", "--selftest"], quiet_settings, broken, 0.02,
                                         None, environment, repository, quiet=True)
    if exit_code != 5 or findings or not verdict.startswith("watched\tfindings=0\t"):
        failures.append((3, f"只用假 cargo、真 cargo 与 rustc 只问版本的绿样本应当照命令退 5、一处不报（实际退 {exit_code}，扫到 {findings}）"))

    # 第 4 格：绝对路径起的真工具链（自证把工具链目录换成临时目录，里面一份拷来的 sleep 叫 rustc），不经 PATH，靠扫进程树
    sleeper = shutil.which("sleep") or "/bin/sleep"
    toolchain_rustc = os.path.join(work, "toolchain", "stable", "bin", "rustc")
    os.makedirs(os.path.dirname(toolchain_rustc), exist_ok=True)
    shutil.copy(sleeper, toolchain_rustc)
    exit_code, findings, _verdict = guard(["bash", "-c", f"{shlex.quote(toolchain_rustc)} 0.4"], quiet_settings, broken, 0.02,
                                          None, environment, work, quiet=True)
    if not (exit_code == FINDINGS_EXIT and any(kind == "真工具链" and toolchain_rustc in text for _pid, kind, text in findings)):
        failures.append((4, f"绝对路径起的真工具链（{toolchain_rustc} 0.4）应当扫到、判红退 3（实际退 {exit_code}，扫到 {findings}）"))

    # 第 5 格：真清场（某棵仓树的 research/scripts/gpu-quiesce-run.sh，配置没设或是主工作树根的那一份）判红；指到假配置的、别处叫这个名字的替身不判
    wrapper = os.path.join(work, "tree", QUIESCE_WRAPPER_SUFFIX)
    selftest_write(wrapper, "#!/usr/bin/env bash\nsleep 0.4\n", executable=True)
    lookalike = os.path.join(work, "tools", "gpu-quiesce-run.sh")
    selftest_write(lookalike, "#!/usr/bin/env bash\nsleep 0.4\n", executable=True)
    fake_configuration = os.path.join(work, "fake-multi-host.env")
    selftest_write(fake_configuration, "QUIESCE_STOP_COMMAND=true\nQUIESCE_STOPPED_CHECK_COMMAND=true\n")
    outcomes = {}
    for label, script, configured in (("没设配置", wrapper, None), ("主工作树根的配置", wrapper, quiet_settings.real_configuration),
                                      ("假配置", wrapper, fake_configuration), ("别处的同名替身", lookalike, None)):
        run_environment = dict(environment)
        if configured is not None:
            run_environment["SINGLEFS_MULTI_HOST_CONFIG"] = configured
        outcomes[label] = guard(["bash", script, "--", "true"], quiet_settings, broken, 0.02, None, run_environment, work, quiet=True)[0]
    if outcomes != {"没设配置": FINDINGS_EXIT, "主工作树根的配置": FINDINGS_EXIT, "假配置": 0, "别处的同名替身": 0}:
        failures.append((5, f"真清场与假配置没分开（各趟退出码 {outcomes}；要没设配置与主工作树根的配置退 3，另两样退 0）"))

    # 第 6 格：私有清场脚本（私有目录换成临时目录）判红；别处叫同一个名字的不判
    private_script = os.path.join(work, "private", "quiesce-stop.sh")
    selftest_write(private_script, "#!/usr/bin/env bash\nsleep 0.4\n", executable=True)
    elsewhere = os.path.join(work, "elsewhere", "quiesce-stop.sh")
    selftest_write(elsewhere, "#!/usr/bin/env bash\nsleep 0.4\n", executable=True)
    private_exit = guard(["bash", private_script], quiet_settings, broken, 0.02, None, environment, work, quiet=True)[0]
    elsewhere_exit = guard(["bash", elsewhere], quiet_settings, broken, 0.02, None, environment, work, quiet=True)[0]
    if private_exit != FINDINGS_EXIT or elsewhere_exit != 0:
        failures.append((6, f"私有清场脚本应当判红退 3、别处同名的退 0（实际 {private_exit}、{elsewhere_exit}）"))

    # 第 7 格：另一份守卫起的那一支整棵不扫——里层那一份自己判红退 3，外层只转它的退出码、自己一处不报
    inner = [sys.executable, SCRIPT_PATH, "--interval", "0.02", "--", "bash", wrapper, "--", "true"]
    exit_code, findings, _verdict = guard(inner, quiet_settings, broken, 0.02, None, environment, work, quiet=True, output=subprocess.DEVNULL)
    if exit_code != FINDINGS_EXIT or findings:
        failures.append((7, f"另一份守卫起的那一支应当整棵不扫：外层照转里层的退 3、自己一处不报（实际退 {exit_code}，外层扫到 {findings}）"))

    # 第 8 格：真清场那一判读到空的环境——参数也读不到了（两次读之间它退出了）不报；参数还在（环境本来就是空的）照报。拿替身换掉两个读 /proc 的函数演
    arguments = ["bash", wrapper, "--", "true"]
    saved = (globals()["process_environment"], globals()["process_arguments"])
    try:
        globals()["process_environment"] = lambda _pid: {}
        globals()["process_arguments"] = lambda _pid: None
        exited = [kind for kind, _text in judge_process(1, arguments, quiet_settings, broken) if kind == "真清场"]
        globals()["process_arguments"] = lambda _pid: list(arguments)
        empty = [kind for kind, _text in judge_process(1, arguments, quiet_settings, broken) if kind == "真清场"]
    finally:
        globals()["process_environment"], globals()["process_arguments"] = saved
    if exited or not empty:
        failures.append((8, f"退出途中读到的空环境应当不报、本来就空的环境照报（实际 {len(exited)} 处、{len(empty)} 处）"))
    return SELFTEST_CELL_COUNT, failures


if __name__ == "__main__":
    preflight(__file__, own_arguments(sys.argv[1:]))
    sys.exit(main(sys.argv[1:]))
