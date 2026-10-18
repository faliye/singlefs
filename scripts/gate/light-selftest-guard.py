#!/usr/bin/env python3
# admission: always 每一次调都盯这一趟起的那条自证跑的时候起了什么，上一次盯过不替这一次作保
# run-condition: command python3 bash
# gate-similar: agent-watch.py 它的「重型测试没带前缀」也扫进程树、照同一份登记表认登记成轻的自证，但那一类恰好在登记成轻的自证底下整棵不判，只报不拒、按复检间隔扫整个 Claude 实例；这里只盯一条自证，跑的那段时间里把 PATH 上的 cargo、rustc 换成会拒的替身、按短间隔扫它自己那棵进程树、经 strace 记下它的每一次 execve，判红交给调它的门禁格
# gate-similar: heavy-test-guard.sh 它在执行前按命令文本判轻重，登记成 light 的单独 --selftest 直接放行，看不进脚本里起的进程；这里在运行中看进程，两份判的不是同一个时刻
# gate-similar: code-tooling.sh 它的 research-script-selftests 格经这一份起 runner 表里的每一条：判法与替身在这里，阶段只收这一份写的判定文件
"""盯一条登记成 light 的自证（或 stage-selftest.sh 喂的样本档）：它跑的时候起了真 cargo / rustc / qemu-system-* / herd7 / verus、真清场或私有清场脚本就判红。

用法：
  light-selftest-guard.py [--only-if-light] [--registry-root <仓根>] [--verdict-file <文件>] [--interval <秒>] -- <命令…>
  light-selftest-guard.py --selftest

判据（.claude/hooks/selftest-registry.tsv 登记成 light 的自证，跑的时候它整棵进程树里出现下面任一样就判红）：
  ① 真的 Rust 工具链：可执行文件（/proc/<pid>/exe 解开符号链接）落在 rustup 的 toolchains 目录（${RUSTUP_HOME:-~/.rustup}/toolchains）、
     cargo 的 bin 目录（${CARGO_HOME:-~/.cargo}/bin）之下，或正是 PATH 上 cargo、rustc 解到的那一份，参数又不是版本查询
     （-V、--version、-vV、version 这几种，前面可以带 +<工具链>；rustc 的 --print <项> 也算查询）。
     另在 PATH 最前面放 cargo、rustc 两个替身：版本查询交给真的那一份，别的一律拒（退 97，真的那一份不起）、记一笔，记到的也判红。
  ② 真清场：起的脚本是某棵仓树里的 scripts/crash/gpu-quiesce-run.sh（路径以这一段结尾；单独一个 --selftest 的除外），
     它环境里的 SINGLEFS_MULTI_HOST_CONFIG 没设、是空的，或解开之后正是主工作树根的 multi-host.env（它读的就是真多机配置里的清场命令）。
  ③ 私有清场脚本：参数里有一段落在 ~/.config/singlefs/quiesce 开头的路径上（私有的清场、回读、复原脚本放在那里）。
  ④ 起进程的系统调用（C613）：命令经 `strace -f --seccomp-bpf --kill-on-exit -ttt -e trace=execve -o <文件>` 起，每一次成功的 execve 取出可执行文件路径与参数。
     文件名（路径最后一段，不解开符号链接）是 cargo、rustc、herd7、verus、rust-verify（rust_verify）或 qemu-system-* 之一，
     参数又不是版本查询（-V、--version、-version、-vV 这几种，前面可以带 +<工具链>；rustc 的 --print <项> 也算查询），
     而可执行文件既不在这一趟自己建的临时目录里，也不在被判的那棵树里（--registry-root，不给取守卫所在的仓：样本根随样本带的 sample-tools/cargo 这类假工具）
     （路径的最后一段是符号链接的，再看它指到哪：指到工具链目录里的、或指到的文件名也是这张表里的工具的，不算在里面；指到 sleep、true 这类别的程序冒充 cargo 的算在里面），
     路径上也没有一段叫 .crash-point-replay-sample-tools（样本根的假工具目录）：判红，
     点名第几次 execve、进程号、路径与参数。这一趟自己建的临时目录是守卫给命令的 TMPDIR（命令里 mktemp、tempfile 建的东西都在它下面）与放替身的目录；
     命令把 TMPDIR 换掉或写死 /tmp/… 放的假 cargo 不在其内，会被判红，出路是放进 $TMPDIR。
     补采样采不到的那一半：一个 50 毫秒的进程采样采不到，execve 记录里一次不漏（PATH 搜索试错的失败调用不算）。
     --only-if-light 认的命令除了登记成 light 的单独 --selftest，还有 stage-selftest.sh 喂的样本档（lib_heavy_tests.py 的 stage_selftest_problems 放行的：
     喂的每一道都登记成 samples-light、样本与 HEAD 加暂存区相同）；样本的 setup.sh 里写 cargo test 同样经这一条判红。
  假的不算：自证自己的样本工具目录里的假 cargo（解释器起的脚本、可执行文件不在上面那几处），别处路径上叫 gpu-quiesce-run.sh 的替身，
  SINGLEFS_MULTI_HOST_CONFIG 指到临时目录里一份假配置的真包装，都不判。
  另一份 light-selftest-guard.py 起的那一支整棵不扫：那一份自己判（它的自证故意起这几样）。

射程（管不到的，靠写自证的人）：
  ④ 在命令自己的进程树里全程看得见每一次 execve，不靠采样；管不到的是它之外的：没被 execve 起的（同一个 shell 里 source 进来的、python 里 ctypes 直接调库的）、
  文件名不在上面那张表里的（把真 cargo 拷成别的名字再起、rustup run 之外的包装）、被跟踪的进程自己 ptrace 或改 seccomp 躲开的、
  脱离这棵树的（setsid 之后被 init 收养的仍在被 strace 跟着，systemd 单元里起的、ssh 到第二台上起的不在）。
  只判、不停任何进程：红在这一趟的退出码与判定文件上；被改成真跑重活的自证要等它跑完才报，要拦在起之前得另想办法。
  进程树按 --interval 秒（默认 0.1）扫一次（① ② ③ 仍靠它）：活不到一个间隔、又不经 PATH 起的一闪而过的私有脚本可能漏掉；
  在同一个 shell 里 source 进来执行的清场命令
  （peer-host-lib.sh 那种）没有自己的进程，只认得出它另起的私有脚本；替身只拦经 PATH 找到的 cargo、rustc，把 PATH 整个换掉的照漏，
  只靠扫进程树与 execve 记录。经 PATH 的那几次被替身拒、根本没起。

本次未查（不是判过了）：
  --only-if-light 时命令不是登记成 light 的自证或 samples-light 的样本档（登记成 heavy、heavy-if-set 而那个变量设了、表外的、带了 --selftest 之外参数的、
  起的不是一份脚本），不盯：判定文件写 not-watched 与理由，原样 exec 那条命令、退它的退出码。
  /proc/<pid>/task/*/children 读不到（内核没开 CONFIG_PROC_CHILDREN）时只有替身那一半，判定文件写 scan=no，成功句写明进程树没扫。
  strace 不在 PATH 上、ptrace 不让用、或这一份自己已经在 strace 之下（外层那一份跟着，判定文件写 trace=outer）时 execve 那一半没跑，
  判定文件写 trace=no（外层跟着的写 trace=outer；命令是另一份守卫的写 trace=nested，那一份自己判），成功句写「execve 跟踪本次未跑」与理由，不当绿；--selftest 里需要 strace 的几格同样报本次未跑、整轮退 77。

判定文件（--verdict-file，调用方逐份回读、数份数）：一行，制表符分隔。
  watched	findings=<N>	scan=yes|no	examined=<扫到的进程形态数>	exit=<命令的退出码>	trace=yes|no|outer|nested	execs=<跟踪到的成功 execve 次数>
    trace=nested 是命令本身就是另一份 light-selftest-guard.py（外层不跟它，它自己起 strace 判）；trace=outer 是这一份自己已经在 strace 之下。
  not-watched	<为什么不盯>
退出码：命令起了而没见真重活，交命令自己的退出码；见了真重活退 3（命令自己退几写进 ✗ 那一句与判定文件）；用法错退 2；
  --only-if-light 而不盯时就是那条命令的退出码（exec 过去了）；--selftest 需要 strace 的几格没跑时退 77。

弄坏开关 LIGHT_SELFTEST_GUARD_BREAK=<项>（逗号连几个；只在 --selftest 里认，平常跑设了也不认、打一句提醒）。
逐项写开关名、弄坏哪一处，下一行起写打开后 --selftest 哪几格判错（括号里是那一格「✗ 自证第 <号> 格」之后那一句的句首）：
  no-path-shims  PATH 上不放替身
      第 2 格（「登记成 light 的假自证经 PATH 起真 cargo build」）
  no-process-scan  不扫进程树
      第 4 格（「绝对路径起的真工具链」）、第 5 格（「真清场」）、第 6 格（「私有清场脚本」）
  version-query-flagged  版本查询也当真重活（替身拒它、扫到也记、execve 记录里也记）
      第 3 格（「只用假 cargo、真 cargo 与 rustc 只问版本的绿样本」）、第 10 格（「execve 跟踪下假 cargo 与只问版本的真工具不红」）
  quiesce-always-fake  gpu-quiesce-run.sh 不看 SINGLEFS_MULTI_HOST_CONFIG，一律当假配置
      第 5 格（「真清场」）、第 8 格（「退出途中读到的空环境应当不报」）
  private-script-ignored  不认私有清场脚本
      第 6 格（「私有清场脚本」）
  nested-guard-scanned  另一份守卫起的那一支也扫
      第 7 格（「另一份守卫起的那一支」）
  light-registry-ignored  --only-if-light 一律当不是登记成 light 的、不盯
      第 1 格（「照登记表分盯不盯」）、第 2 格（「登记成 light 的假自证经 PATH 起真 cargo build」）、
      第 9 格（「立刻失败的真 cargo build 要被 execve 跟踪抓到」，那一格先问登记表）、第 12 格（「样本档的 setup.sh 里起真 cargo」，那一格先问 stage-selftest.sh 喂的样本档盯不盯）
  empty-environment-as-unset  读 /proc/<pid>/environ 读到空的、参数也读不到了（两次读之间它退出了）照「没设配置」报真清场
      第 8 格（「退出途中读到的空环境应当不报」）
  exec-trace-skipped  不经 strace 跟踪 execve，退回只有采样与替身
      第 9 格（「立刻失败的真 cargo build 要被 execve 跟踪抓到」）、第 10 格（「execve 跟踪下假 cargo 与只问版本的真工具不红」）、
      第 11 格（「按文件名认 qemu、herd7、verus」）、第 12 格（「样本档的 setup.sh 里起真 cargo」）
  shim-argv0-dropped  替身把版本查询交给真的那一份时 argv[0] 不留成 cargo / rustc（真的那一份常是 rustup 代理的符号链接，解开是 rustup，答出来的是 rustup 的版本）
      第 3 格（「只用假 cargo、真 cargo 与 rustc 只问版本的绿样本」，那份假自证核版本行是 cargo、rustc 自己的）
  launcher-not-stripped  认替身那一行时不剥掉前面的 env（经 shebang 起的替身 cmdline 是 /usr/bin/env bash <替身> -V）
      第 14 格（「替身刚 exec 成真工具链那一瞬的版本查询」）
"""
import contextlib
import importlib.util
import io
import os
import re
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
QUIESCE_WRAPPER_SUFFIX = os.path.join("scripts", "crash", "gpu-quiesce-run.sh")
PRIVATE_QUIESCE_PREFIX = "quiesce"
SHELL_NAMES = {"bash", "sh", "dash", "zsh", "ksh"}
VERSION_QUERIES = {("-V",), ("--version",), ("-vV",), ("-Vv",), ("-V", "-v"), ("-v", "-V"),
                   ("--version", "--verbose"), ("--verbose", "--version"), ("version",)}
BREAK_VARIABLE = "LIGHT_SELFTEST_GUARD_BREAK"
KNOWN_BREAKS = ("no-path-shims", "no-process-scan", "version-query-flagged", "quiesce-always-fake", "private-script-ignored",
                "nested-guard-scanned", "light-registry-ignored", "empty-environment-as-unset", "exec-trace-skipped",
                "shim-argv0-dropped", "launcher-not-stripped")
LAUNCHER_NAMES = {"env"}
SELFTEST_CELL_COUNT = 14
NOT_RUN_EXIT = 77
STAGE_SELFTEST_NAME = "stage-selftest.sh"
# execve 跟踪认的真工具：文件名（路径最后一段）；样本根的假工具目录不算；strace 的参数里 -e signal=none 去掉信号行，-qq 去掉退出行，-s 放宽字符串长度
REAL_TOOL_NAME = re.compile(r"^(?:cargo|rustc|herd7|verus|rust[-_]verify|qemu-system-[A-Za-z0-9_.+-]+)$")
SAMPLE_TOOLS_DIRECTORY = ".crash-point-replay-sample-tools"
EXEC_VERSION_QUERIES = VERSION_QUERIES | {("-version",)}
STRACE_FLAGS = ("-f", "--seccomp-bpf", "--kill-on-exit", "-ttt", "-s", "4096", "-qq", "-e", "signal=none", "-e", "trace=execve")
TRACE_LINE = re.compile(r"^(\d+)\s+(\d+(?:\.\d+)?)\s+(.*)$")
TRACE_EXEC_DONE = re.compile(r"^execve\((.*)\)\s+=\s+(-?\d+)(?:\s.*)?$")
TRACE_EXEC_RESUMED = re.compile(r"^<\.\.\. execve resumed>.*\)\s+=\s+(-?\d+)(?:\s.*)?$")
TRACE_EXEC_UNFINISHED = re.compile(r"^execve\((.*)\s<unfinished \.\.\.>$")
C_ESCAPES = {"n": "\n", "t": "\t", "r": "\r", "v": "\v", "f": "\f", "a": "\a", "b": "\b", "\\": "\\", '"': '"', "'": "'"}

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
  [[ -n "$real" ]] && exec {argv0}"$real" "$@"   # argv[0] 留成 {tool}：真的那一份常是 rustup 代理的符号链接，解开是 rustup，rustup 按 argv[0] 的名字认自己是 cargo 还是 rustc
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

    def __init__(self, toolchain_roots, toolchain_binaries, real_configuration, private_directory, exempt_roots=()):
        self.toolchain_roots = [os.path.realpath(root) for root in toolchain_roots]
        self.toolchain_binaries = {os.path.realpath(path) for path in toolchain_binaries}
        self.real_configuration = os.path.realpath(real_configuration) if real_configuration else ""
        self.private_directory = os.path.realpath(private_directory)
        self.exempt_roots = [os.path.realpath(root) for root in exempt_roots]   # execve 跟踪下不当真工具的目录：自证自己建的临时目录（自证里是它的工作目录）


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


def default_settings(exempt_roots=()):
    home = os.path.expanduser("~")
    roots = [os.path.join(os.environ.get("RUSTUP_HOME") or os.path.join(home, ".rustup"), "toolchains"),
             os.path.join(os.environ.get("CARGO_HOME") or os.path.join(home, ".cargo"), "bin")]
    binaries = [path for path in (real_tool(tool, os.environ.get("PATH", "")) for tool in SHIMMED_TOOLS) if path]
    return Settings(roots, binaries, main_worktree_configuration(), os.path.join(home, ".config", "singlefs"), exempt_roots)


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


def shim_launch_arguments(arguments, broken=()):
    """命令行是不是「解释器起一个脚本」那一形（`bash <替身> -V`，或经 shebang 的 `/usr/bin/env bash <替身> -V`）：是就交脚本词起的那一段，不是交 None。
    替身刚 exec 成真工具链那一瞬 exe 已换、cmdline 还是这一行；替身只是原样转交，exec 之后 argv 就是脚本词后面的参数。"""
    rest = list(arguments)
    if rest and os.path.basename(rest[0]) in LAUNCHER_NAMES and "launcher-not-stripped" not in broken:
        rest = rest[1:]
        while rest and (rest[0].startswith("-") or "=" in rest[0]):   # env 的选项与赋值；-u 带一个名字
            rest = rest[2:] if rest[0] in ("-u", "--unset") else rest[1:]
    if rest and os.path.basename(rest[0]) in SHELL_NAMES and len(rest) >= 2:
        return rest[1:]
    return None


def judge_process(pid, arguments, settings, broken):
    """一个进程此刻的样子里有哪几样真重活：[(类别, 说明)]。"""
    found = []
    executable = process_executable(pid)
    toolchain_arguments = arguments
    shim_rest = shim_launch_arguments(arguments, broken)
    if is_real_toolchain(executable, settings) and shim_rest is not None:
        # 替身脚本刚 exec 成真工具链：exe 已换、cmdline 还是替身那一行。再读一次 cmdline；
        # 还是替身那一行就按替身脚本后面的参数算（替身只是原样转交，exec 之后 argv 就是它们）
        reread = process_arguments(pid)
        toolchain_arguments = reread if reread and shim_launch_arguments(reread, broken) is None else shim_rest
    if is_real_toolchain(executable, settings) and ("version-query-flagged" in broken or not is_version_query(toolchain_arguments[1:])):
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


# ── 起进程的系统调用（strace 的 execve 记录）────────────────────────────────

EXEC_KIND = "execve 起真工具"


def is_exec_version_query(arguments):
    """execve 记录里 argv[1:] 是不是版本查询：-V、--version、-vV 这几种（前面可带 +<工具链>）、rustc 的 --print，加上 herd7 与 qemu 的 -version。"""
    rest = list(arguments)
    if rest and rest[0].startswith("+"):
        rest = rest[1:]
    return tuple(rest) == ("-version",) or is_version_query(arguments)


def under_directory(path, directory):
    return path == directory or path.startswith(directory + os.sep)


def judge_execution(path, arguments, settings, exempt_roots, broken, working_directory):
    """一次成功的 execve（可执行文件路径与 argv）算不算起了真工具：算交一句说明，不算交 None。
    名字取路径最后一段（不解开符号链接：~/.cargo/bin/cargo 解开是 rustup）；放过的：版本查询、路径上有一段叫 .crash-point-replay-sample-tools、
    路径（目录部分解开、最后一段不解开）在自证自己建的临时目录里。临时目录里的符号链接再看一眼它指到哪：指到工具链目录里的（rustup 代理、toolchains 下的 cargo、
    PATH 上的 cargo、rustc）、或指到的文件名本身也是上面那张表里的工具的，不放过；指到 sleep、true 这类别的程序的（自证拿它们冒充 cargo）放过。"""
    name = os.path.basename(path)
    if not REAL_TOOL_NAME.match(name):
        return None
    if "version-query-flagged" not in broken and is_exec_version_query(arguments[1:]):
        return None
    absolute = path if os.path.isabs(path) else os.path.join(working_directory, path)
    resolved = os.path.realpath(absolute)
    if SAMPLE_TOOLS_DIRECTORY in path.split(os.sep) or SAMPLE_TOOLS_DIRECTORY in resolved.split(os.sep):
        return None
    literal = os.path.join(os.path.realpath(os.path.dirname(absolute)), name)   # 目录部分的符号链接解开（/tmp 本身可能是链接），最后一段留着
    if any(under_directory(literal, root) for root in [*settings.exempt_roots, *exempt_roots]):
        if resolved == literal or not (is_real_toolchain(resolved, settings) or REAL_TOOL_NAME.match(os.path.basename(resolved))):
            return None
    return f"{name}（{resolved}）：{shlex.join(arguments)}"


def read_c_string(text, position):
    """strace 打印的一段 C 字符串：text[position] 是开头的引号；交 (字符串, 收尾引号后面的位置)；被 -s 截断的字符串后面跟的 ... 一并跳过。"""
    if text[position] != '"':
        raise ValueError("不是引号开头")
    position += 1
    raw = bytearray()
    while True:
        character = text[position]
        if character == '"':
            position += 1
            break
        if character != "\\":
            raw += character.encode("utf-8")
            position += 1
            continue
        escape = text[position + 1]
        octal = re.match(r"[0-7]{1,3}", text[position + 1:position + 4])
        if octal:
            raw.append(int(octal.group(), 8) & 0xFF)
            position += 1 + len(octal.group())
        elif escape == "x" and re.match(r"[0-9a-fA-F]{2}", text[position + 2:position + 4]):
            raw.append(int(text[position + 2:position + 4], 16))
            position += 4
        else:
            raw += C_ESCAPES.get(escape, escape).encode("utf-8")
            position += 2
    if text.startswith("...", position):
        position += 3
    return raw.decode("utf-8", errors="replace"), position


def parse_execve(body):
    """strace 一行 execve( 与 ) 之间的部分：'"路径", ["argv0", "argv1"], 0x… /* 58 vars */' → (路径, [argv…])；读不懂交 None。"""
    try:
        path, position = read_c_string(body, 0)
        if not body.startswith(", [", position):
            return None
        position += 3
        arguments = []
        while position < len(body) and body[position] != "]":
            if body.startswith(", ", position):
                position += 2
            elif body.startswith("...", position):
                position += 3
            else:
                value, position = read_c_string(body, position)
                arguments.append(value)
        return path, arguments
    except (IndexError, ValueError):
        return None


class ExecTrace:
    """一条命令的 strace -f -ttt -e trace=execve 记录（-o 写的文件）：跑的时候逐次读新追加的行，每一次成功的 execve 判一遍。
    失败的 execve（PATH 搜索试错）不算；多线程进程里被拆成 <unfinished ...> 与 <... execve resumed> 两行的按进程号接上。"""

    def __init__(self, path, settings, exempt_roots, broken, working_directory, started):
        self.path, self.settings, self.exempt_roots, self.broken = path, settings, exempt_roots, broken
        self.working_directory, self.started = working_directory, started
        self.offset, self.partial, self.pending = 0, b"", {}
        self.count, self.unparsed, self.findings = 0, 0, []

    def poll(self):
        try:
            with open(self.path, "rb") as handle:
                handle.seek(self.offset)
                data = handle.read()
        except OSError:
            return
        self.offset += len(data)
        *lines, self.partial = (self.partial + data).split(b"\n")
        for raw in lines:
            self.take(os.fsdecode(raw))

    def take(self, line):
        matched = TRACE_LINE.match(line)
        if not matched:
            return
        pid, stamp, rest = int(matched.group(1)), float(matched.group(2)), matched.group(3)
        unfinished = TRACE_EXEC_UNFINISHED.match(rest)
        if unfinished:
            self.pending[pid] = unfinished.group(1)
            return
        resumed = TRACE_EXEC_RESUMED.match(rest)
        if resumed:
            body = self.pending.pop(pid, None)
            if body is not None and resumed.group(1) == "0":
                self.record(pid, stamp, body)
            return
        done = TRACE_EXEC_DONE.match(rest)
        if done and done.group(2) == "0":
            self.record(pid, stamp, done.group(1))

    def record(self, pid, stamp, body):
        self.count += 1
        parsed = parse_execve(body)
        if parsed is None:
            self.unparsed += 1
            return
        path, arguments = parsed
        try:
            working_directory = os.readlink(f"/proc/{pid}/cwd")
        except OSError:
            working_directory = self.working_directory
        text = judge_execution(path, arguments, self.settings, self.exempt_roots, self.broken, working_directory)
        if text and pid not in {finding[0] for finding in self.findings}:   # 同一个进程再 exec（rustup 的 cargo 换成工具链里的）只记头一次
            self.findings.append((pid, EXEC_KIND, f"第 {self.count} 次 execve（进程 {pid}，起后 {max(stamp - self.started, 0):.2f} 秒）：{text}"))


def tracer_pid():
    try:
        with open("/proc/self/status", encoding="ascii", errors="replace") as handle:
            for line in handle:
                if line.startswith("TracerPid:"):
                    return int(line.split()[1])
    except (OSError, ValueError, IndexError):
        pass
    return 0


STRACE_PROBED = {}


def find_strace(broken, override=None):
    """(strace 路径或 None, 状态 yes|no|outer, 一句理由)。no：弄坏开关、找不到、ptrace 不让用；outer：这一份自己已经在别人的 strace 之下（被跟踪的进程再 ptrace 别的进程会被拒，外层那一份跟着）。"""
    if "exec-trace-skipped" in broken:
        return None, "no", "弄坏开关 exec-trace-skipped"
    if tracer_pid():
        return None, "outer", f"这一份已经在 {tracer_pid()} 号进程的 ptrace 之下，再起 strace 会被拒；外层那一份跟着它的每一次 execve"
    path = override if override is not None else shutil.which("strace")
    if not path or not os.access(path, os.X_OK):
        return None, "no", f"{override or 'PATH 上'}没有可执行的 strace"
    if path not in STRACE_PROBED:
        probe = subprocess.run([path, *STRACE_FLAGS, "-o", os.devnull, "--", shutil.which("true") or "/bin/true"],
                               capture_output=True, text=True, stdin=subprocess.DEVNULL)
        STRACE_PROBED[path] = "" if probe.returncode == 0 else ((probe.stderr.strip().splitlines() or ["退出码 " + str(probe.returncode)])[0])
    if STRACE_PROBED[path]:
        return None, "no", f"{path} 起不了跟踪（{STRACE_PROBED[path]}）"
    return path, "yes", ""


class Watch:
    """一条命令那棵进程树的扫描状态：见过的进程形态（进程号、起始时刻、参数）与扫到的真重活。"""

    def __init__(self, settings, broken):
        self.settings, self.broken = settings, broken
        self.seen, self.findings, self.scan_available = set(), [], True

    def scan(self, root_pid, skip_root=False):
        """从这一份起的那个进程本身扫起（它 exec 成别的样子也算），逐层往下；skip_root 时根进程（strace）本身不判、只往下。"""
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
            if skip_root and pid == root_pid:
                continue
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
                                              query_allowed=0 if "version-query-flagged" in broken else 1,
                                              argv0="" if "shim-argv0-dropped" in broken else f'-a "{tool}" '))
        os.chmod(path, 0o755)


def guard(command, settings, broken=frozenset(), interval=DEFAULT_INTERVAL_SECONDS, verdict_file=None, environment=None, directory=None, quiet=False,
          output=None, strace_binary=None):
    """起 command、盯到它退出：交 (退出码, 扫到的 [(进程号, 类别, 说明)], 判定行)。stdout、stderr 原样交给命令（output 给了就都接到它上）；
    quiet 时不打判定句。strace_binary 给了就用它跟踪 execve（自证拿不存在的路径演 strace 不在）。
    命令的 TMPDIR 换成这一趟自己建的目录（execve 跟踪里它与替身目录下的东西不当真工具）。"""
    environment = dict(os.environ if environment is None else environment)
    environment.pop(BREAK_VARIABLE, None)
    run_directory = tempfile.mkdtemp(prefix="light-selftest-guard-")
    shim_directory, private_temporary = os.path.join(run_directory, "shims"), os.path.join(run_directory, "tmp")
    os.mkdir(shim_directory)
    os.mkdir(private_temporary)
    record, passed = os.path.join(shim_directory, "refused.tsv"), os.path.join(shim_directory, "passed.tsv")
    nested = is_nested_guard(command) and "nested-guard-scanned" not in broken   # 另一份守卫要自己起 strace，外层不跟它（被跟踪的进程再 ptrace 会被拒）
    trace_path, trace_state, trace_reason = ((None, "nested", "命令是另一份 light-selftest-guard.py，它自己判、自己跟踪") if nested
                                             else find_strace(broken, strace_binary))
    started = time.time()
    try:
        if "no-path-shims" not in broken:
            write_shims(shim_directory, environment.get("PATH", ""), broken)
            environment["PATH"] = shim_directory + os.pathsep + environment.get("PATH", "")
        environment.update(LIGHT_SELFTEST_GUARD_RECORD=record, LIGHT_SELFTEST_GUARD_PASSED=passed, TMPDIR=private_temporary)
        watch = Watch(settings, broken)
        trace = None
        argv = list(command)
        if trace_path:
            trace = ExecTrace(os.path.join(run_directory, "execve.log"), settings, [os.path.realpath(run_directory)], broken, directory or os.getcwd(), started)
            argv = [trace_path, *STRACE_FLAGS, "-o", trace.path, "--", *command]
        process = subprocess.Popen(argv, env=environment, cwd=directory, stdout=output, stderr=output)

        def forward(signal_number, _frame):
            if process.poll() is None:
                process.send_signal(signal_number)   # 只发给自己起的那一个子进程（经 strace 起的，发给 strace，--kill-on-exit 让它跟着的进程一起收）

        previous = {number: signal.signal(number, forward) for number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)}
        try:
            while True:
                if trace:
                    trace.poll()
                if "no-process-scan" not in broken:
                    watch.scan(process.pid, skip_root=trace is not None)
                if process.poll() is not None:
                    break
                time.sleep(interval)
        finally:
            for number, handler in previous.items():
                signal.signal(number, handler)
        if trace:
            trace.poll()
        exit_code = process.returncode
        traced_pids = {finding[0] for finding in trace.findings} if trace else set()
        findings = list(trace.findings) if trace else []
        findings.extend(finding for finding in watch.findings if finding[0] not in traced_pids)   # 同一个进程 execve 记录已经点名的，采样不再重复记
        for line in read_lines(record):
            pid, _tab, rest = line.partition("\t")
            where, _tab, invocation = rest.partition("\t")
            findings.append((pid, "替身拒了的真工具链", f"{invocation}（当前目录 {where}；经 PATH 找的那一次被替身拒了，真的那一份没起）"))
        passed_count = len(read_lines(passed))
        executions = trace.count if trace else 0
        unparsed = trace.unparsed if trace else 0
    finally:
        shutil.rmtree(run_directory, ignore_errors=True)
    scanned = "no-process-scan" not in broken and watch.scan_available
    verdict = (f"watched\tfindings={len(findings)}\tscan={'yes' if scanned else 'no'}\texamined={len(watch.seen)}\texit={exit_code}"
               f"\ttrace={trace_state}\texecs={executions}")
    if verdict_file:
        with open(verdict_file, "w", encoding="utf-8") as handle:
            handle.write(verdict + "\n")
    shown = shlex.join(command)
    trace_text = (f"execve 跟踪了 {executions} 次起进程{f'（{unparsed} 次读不懂）' if unparsed else ''}" if trace_path
                  else f"execve 跟踪本次未跑（{trace_reason}）")
    if findings:
        if not quiet:
            print(f"  ✗ light-selftest-guard：{shown} 是登记成 light 的自证，跑的时候起了 {len(findings)} 处真重活（命令自己退 {exit_code}）：", file=sys.stderr)
            for pid, kind, text in findings:
                print(f"      进程 {pid}：{kind}：{text}", file=sys.stderr)   # gate-lint:detail
            if not trace_path:
                print(f"      ! {trace_text}：快进快出的真工具可能没扫到", file=sys.stderr)   # gate-lint:detail
            print("  → 怎么办：自证里把那一步换成假的（假 cargo、rustc、qemu、herd7、verus 放进自证自己建的临时目录（$TMPDIR 下）、排在 PATH 最前面；清场把 "
                  "SINGLEFS_MULTI_HOST_CONFIG 指到临时目录里只写了假清场命令的一份），或在 .claude/hooks/selftest-registry.tsv 把它改登记成 heavy / "
                  "heavy-if-set:<变量>、理由以「偏离上游：」起头", file=sys.stderr)
        return FINDINGS_EXIT, findings, verdict
    if not quiet:
        scan_text = (f"进程树每 {interval} 秒扫一次、扫到 {len(watch.seen)} 个进程形态" if scanned
                     else "进程树没扫（/proc/<pid>/task/*/children 读不到，本次未查这一半，只有替身那一半）")
        print(f"  ✓ light-selftest-guard：{shown} 跑完（退 {exit_code}），{scan_text}，替身放过 {passed_count} 次版本查询，{trace_text}，"
              "没见真 cargo / rustc / qemu-system-* / herd7 / verus（版本查询除外）、真清场与私有清场脚本" if trace_path else
              f"  ✓ light-selftest-guard：{shown} 跑完（退 {exit_code}），{scan_text}，替身放过 {passed_count} 次版本查询，{trace_text}；"
              "只有采样与替身，没见真 cargo / rustc、真清场与私有清场脚本", file=sys.stderr)
    return exit_code, findings, verdict


# ── 照登记表分盯不盯 ─────────────────────────────────────────────────────────

def heavy_tests_library():
    specification = importlib.util.spec_from_file_location("lib_heavy_tests_for_light_selftest_guard", HEAVY_TESTS_LIBRARY_PATH)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def light_judgement(command, directory, registry_root, environment, broken):
    """(盯不盯, 一句理由)：命令起的脚本照 lib_heavy_tests.py 的 classify_selftest 判，是登记成 light 的单独 --selftest 才盯；
    起的是 stage-selftest.sh 的，照 stage_selftest_problems 判它喂的样本档是不是全登记成轻的（samples-light、样本与 HEAD 加暂存区相同），是就盯。
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
    if os.path.basename(script) == STAGE_SELFTEST_NAME:
        problem = library.stage_selftest_problems(script, rest, directory, environment)
        if problem:
            return False, f"stage-selftest.sh 喂的样本档不是全登记成轻的：{problem}"
        return True, f"{script} 喂的样本档全登记成 samples-light（或点名的几道是），样本与 HEAD 加暂存区相同"
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
        return False, f"{script} 不是登记表管的自证（不在 scripts/、.claude/hooks/、.claude/gate.d/lib/ 下，或没带 --selftest）"
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
    # 被判的那棵树（--registry-root，不给取这份脚本所在的仓）里随样本带的假 cargo 之类（样本根的 sample-tools/cargo）不算真工具
    settings = default_settings([options["registry_root"] or REPOSITORY_ROOT])
    exit_code, _findings, _verdict = guard(command, settings, interval=options["interval"], verdict_file=options["verdict_file"] or None)
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
    outside = tempfile.mkdtemp(prefix="light-selftest-guard-outside-")   # 不在 work 之下：execve 跟踪里这里的假 qemu、herd7、verus 不被当「自证自己建的临时目录」放过
    failures, cells, not_run = [], 0, []
    try:
        cells, failures, not_run = selftest_cells(work, outside, broken)
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(outside, ignore_errors=True)
    if failures:
        for number, sentence in failures:
            print(f"  ✗ 自证第 {number} 格：{sentence}", file=sys.stderr)   # gate-lint:detail
        print(f"  ✗ light-selftest-guard.py 自证 {cells} 格里 {len(failures)} 格判错", file=sys.stderr)
        print("     → 怎么办：设了 LIGHT_SELFTEST_GUARD_BREAK 的，这就是要的结果（文件头「弄坏开关」点名的那几格）；没设的，照判错那一格看 "
              "light_judgement、write_shims、Watch.scan、judge_process、ExecTrace 与 judge_execution 里对应的一段", file=sys.stderr)
        return 1
    if not_run:
        for number, sentence in not_run:
            print(f"  ! 自证第 {number} 格本次未跑：{sentence}", file=sys.stderr)   # gate-lint:detail
        print(f"  ✗ light-selftest-guard.py 自证 {cells} 格里 {len(not_run)} 格本次未跑（其余都对）；这不是通过", file=sys.stderr)
        print("     → 怎么办：装好 strace（apt install strace）、确认 /proc/sys/kernel/yama/ptrace_scope 允许跟踪自己的子进程、不要在别的 ptrace 之下跑，再跑一遍；"
              "cargo 缺的先装 Rust 工具链（bash .claude/scripts/env.sh 会报缺什么）", file=sys.stderr)
        return NOT_RUN_EXIT
    print(f"  ✓ light-selftest-guard.py 自证 {cells} 格都对（照登记表分盯不盯、登记成 light 的假自证经 PATH 起真 cargo build 被替身拒并判红、"
          "只用假 cargo 与只问版本的绿、绝对路径起的真工具链扫得到、真清场与假配置分得开、私有清场脚本认得出、另一份守卫那一支不扫、退出途中读到的空环境不当没设配置、"
          "立刻失败的真 cargo build 被 execve 跟踪抓到、假 cargo 与只问版本的真工具不红、qemu / herd7 / verus 按文件名认、样本档 setup.sh 里起真 cargo 判红、"
          "strace 不在时报本次未跑、替身刚 exec 成真工具链那一瞬的版本查询不报）")
    return 0


def selftest_cells(work, outside, broken):
    """十四格；交 (格数, [(格号, 判错的一句)], [(格号, 本次未跑的一句)])。真家伙只起 cargo / rustc 的版本查询与立刻报错退出的 cargo build（清单不在）。
    第 9 到 12 格要 strace 与一份真 cargo 的路径，没有就记本次未跑（弄坏开关 exec-trace-skipped 下照跑、照判错）。"""
    global process_environment, process_arguments, process_executable  # 第 8、14 格拿替身换掉读 /proc 的这几个函数、判完换回
    failures, not_run = [], []
    quiet_settings = Settings([os.path.join(work, "toolchain")], [], os.path.join(work, "main-worktree", "multi-host.env"), os.path.join(work, "private"),
                              exempt_roots=[work])
    environment = {key: value for key, value in os.environ.items() if key not in ("SINGLEFS_MULTI_HOST_CONFIG", BREAK_VARIABLE)}
    repository = os.path.join(work, "repository")
    registry = os.path.join(repository, REGISTRY_RELATIVE_PATH)
    selftest_write(registry, "scripts/gate/fake-light-real-cargo.sh\tlight\t# 自证造的：经 PATH 起真 cargo build\n"
                             "scripts/gate/fake-light-fake-cargo.sh\tlight\t# 自证造的：只用自己的假 cargo，真的只问版本\n"
                             "scripts/gate/fake-heavy.sh\theavy\t# 偏离上游：自证造的登记成 heavy 的一份\n"
                             "scripts/gate/fake-light-absolute-cargo.sh\tlight\t# 自证造的：绝对路径起真 cargo build（清单不在，立刻失败）\n"
                             "scripts/gate/fake-light-trace-green.sh\tlight\t# 自证造的：假 cargo 加只问版本的真工具\n"
                             "scripts/gate/fake-light-names-red.sh\tlight\t# 自证造的：按文件名认的几样工具各起一次\n"
                             "scripts/gate/fake-light-names-green.sh\tlight\t# 自证造的：名字不在表里的与样本假工具目录里的\n")
    missing_manifest = os.path.join(work, "no-such-crate", "Cargo.toml")
    selftest_write(os.path.join(repository, "scripts", "gate", "fake-light-real-cargo.sh"),
                   f"#!/usr/bin/env bash\ncargo build --manifest-path {shlex.quote(missing_manifest)} >/dev/null 2>&1\nexit 0\n", executable=True)
    selftest_write(os.path.join(repository, "sample-tools", "cargo"), "#!/usr/bin/env bash\necho \"假 cargo $*\"\n", executable=True)
    # 版本查询经替身交给真的那一份，答出来的要是 cargo、rustc 自己的版本行（不是 rustup 的：替身 exec 时 argv[0] 掉了就会这样），不然退 6；真的没装就不核
    path_text = environment.get("PATH", "")
    selftest_write(os.path.join(repository, "scripts", "gate", "fake-light-fake-cargo.sh"),
                   "#!/usr/bin/env bash\nPATH=\"$(dirname \"$0\")/../../sample-tools:$PATH\" cargo build --release >/dev/null\n"
                   + ("[[ \"$(cargo --version 2>&1)\" == cargo\\ * ]] || exit 6\n" if real_tool("cargo", path_text) else "")
                   + ("[[ \"$(rustc -V 2>&1)\" == rustc\\ * ]] || exit 6\n" if real_tool("rustc", path_text) else "") + "exit 5\n", executable=True)
    selftest_write(os.path.join(repository, "scripts", "gate", "fake-heavy.sh"), "#!/usr/bin/env bash\nexit 0\n", executable=True)

    def judged(command):
        return light_judgement(command, repository, repository, environment, broken)[0]

    # 第 1 格：照登记表分盯不盯（样本仓自己的登记表），不是登记成 light 的原样 exec、判定文件写 not-watched
    verdict_file = os.path.join(work, "not-watched.verdict")
    exec_run = subprocess.run([sys.executable, SCRIPT_PATH, "--only-if-light", "--registry-root", repository, "--verdict-file", verdict_file,
                               "--", "bash", "-c", "exit 4"], cwd=repository, env=environment, capture_output=True, text=True)
    verdict_line = (read_lines(verdict_file) or [""])[0]
    if not (judged(["bash", "scripts/gate/fake-light-real-cargo.sh", "--selftest"])
            and not judged(["bash", "scripts/gate/fake-heavy.sh", "--selftest"])
            and not judged(["bash", "scripts/gate/fake-light-real-cargo.sh", "--selftest", "--extra"])
            and not judged(["bash", "scripts/not-registered.sh", "--selftest"])
            and exec_run.returncode == 4 and verdict_line.startswith("not-watched\t")):
        failures.append((1, f"照登记表分盯不盯不对（不是登记成 light 的那一趟退 {exec_run.returncode}、判定「{verdict_line}」）"))

    # 第 2 格（红样本）：登记成 light 的假自证经 PATH 起真 cargo build：替身拒了、记一笔，整趟退 3、判定 findings=1
    verdict_file = os.path.join(work, "real-cargo.verdict")
    command = ["bash", "scripts/gate/fake-light-real-cargo.sh", "--selftest"]
    exit_code, findings = FINDINGS_EXIT, []
    if judged(command):
        exit_code, findings, _verdict = guard(command, quiet_settings, broken, 0.02, verdict_file, environment, repository, quiet=True)
    refused = [finding for finding in findings if finding[1] == "替身拒了的真工具链" and "cargo build" in finding[2]]
    if not (judged(command) and exit_code == FINDINGS_EXIT and refused and (read_lines(verdict_file) or [""])[0].startswith("watched\tfindings=1\t")):
        failures.append((2, f"登记成 light 的假自证经 PATH 起真 cargo build，应当被替身拒、判红退 3（实际退 {exit_code}，扫到 {findings}）"))

    # 第 3 格（绿样本）：只用自己的假 cargo，真 rustc 只问版本：退命令自己的 5、一处都不报
    exit_code, findings, verdict = guard(["bash", "scripts/gate/fake-light-fake-cargo.sh", "--selftest"], quiet_settings, broken, 0.02,
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

    # 第 5 格：真清场（某棵仓树的 scripts/crash/gpu-quiesce-run.sh，配置没设或是主工作树根的那一份）判红；指到假配置的、别处叫这个名字的替身不判
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
    saved = (process_environment, process_arguments)
    try:
        process_environment = lambda _pid: {}
        process_arguments = lambda _pid: None
        exited = [kind for kind, _text in judge_process(1, arguments, quiet_settings, broken) if kind == "真清场"]
        process_arguments = lambda _pid: list(arguments)
        empty = [kind for kind, _text in judge_process(1, arguments, quiet_settings, broken) if kind == "真清场"]
    finally:
        process_environment, process_arguments = saved
    if exited or not empty:
        failures.append((8, f"退出途中读到的空环境应当不报、本来就空的环境照报（实际 {len(exited)} 处、{len(empty)} 处）"))

    # 第 14 格：替身刚 exec 成真工具链那一瞬（exe 已是工具链目录里的 cargo，cmdline 还是替身那一行）：`bash <替身> -V` 与经 shebang 的
    # `/usr/bin/env bash <替身> -V` 都是版本查询、不报；同一形带 build 的照报；不经替身的真 cargo -V 不报、cargo build 照报。拿替身换掉读 /proc 的两个函数演
    shim_path = os.path.join(work, "shims-of-cell-14", "cargo")
    toolchain_cargo = os.path.join(work, "toolchain", "cargo")
    shim_cases = [(["/usr/bin/env", "bash", shim_path, "-V"], 0), (["bash", shim_path, "-vV"], 0), (["/usr/bin/env", "-i", "bash", shim_path, "--version"], 0),
                  (["/usr/bin/env", "bash", shim_path, "build"], 1), (["bash", shim_path, "test"], 1), ([toolchain_cargo, "-V"], 0), ([toolchain_cargo, "build"], 1)]
    saved = (process_executable, process_arguments)
    judged = []
    try:
        process_executable = lambda _pid: toolchain_cargo
        for case_arguments, _expected in shim_cases:
            process_arguments = lambda _pid, held=case_arguments: list(held)
            judged.append(len([kind for kind, _text in judge_process(1, case_arguments, quiet_settings, broken) if kind == "真工具链"]))
    finally:
        process_executable, process_arguments = saved
    wrong_shim_cases = [f"{shlex.join(case_arguments)} 报了 {reported} 处（该 {expected}）"
                        for (case_arguments, expected), reported in zip(shim_cases, judged) if reported != expected]
    if wrong_shim_cases:
        failures.append((14, "替身刚 exec 成真工具链那一瞬的版本查询（bash 起、env bash 起）应当不报、别的参数照报：" + "；".join(wrong_shim_cases)))
    not_run.extend(execve_cells(work, outside, broken, quiet_settings, environment, repository, failures))
    return SELFTEST_CELL_COUNT, failures, not_run


def execve_cells(work, outside, broken, quiet_settings, environment, repository, failures):
    """第 9 到 13 格：起进程的系统调用。交本次未跑的 [(格号, 一句)]，判错的追加进 failures。
    真家伙只起 cargo / rustc 的版本查询与一条立刻失败的 cargo build（清单不在）；假 qemu、herd7、verus 是 outside 里只回显的脚本。"""
    path_text = path_without_shims(environment.get("PATH", ""))
    real_cargo, real_rustc = shutil.which("cargo", path=path_text), shutil.which("rustc", path=path_text)   # 不解开符号链接：名字取路径最后一段
    _strace, trace_state, trace_reason = find_strace(frozenset())
    not_run = []
    missing_manifest = os.path.join(work, "no-such-crate", "Cargo.toml")
    tools = os.path.join(outside, "tools")
    for name in ("cargo", "rustc", "herd7", "verus", "rust-verify", "rust_verify", "qemu-system-x86_64", "qemu-system-aarch64", "qemu-img", "cargo-fmt", "xcargo"):
        selftest_write(os.path.join(tools, name), '#!/usr/bin/env bash\necho "假 $0 $*"\n', executable=True)
    sample_tool = os.path.join(outside, "sample-root", SAMPLE_TOOLS_DIRECTORY, "cargo")
    selftest_write(sample_tool, '#!/usr/bin/env bash\necho "样本的假 cargo $*"\n', executable=True)
    os.makedirs(os.path.join(work, "links"), exist_ok=True)
    link = os.path.join(work, "links", "cargo")
    os.symlink(os.path.join(tools, "herd7"), link)   # 放过的目录里一个叫 cargo 的符号链接，解开是放过的目录之外的 herd7
    os.makedirs(os.path.join(work, "links-to-sleep"), exist_ok=True)
    link_to_sleep = os.path.join(work, "links-to-sleep", "cargo")
    os.symlink(shutil.which("true") or "/bin/true", link_to_sleep)   # 放过的目录里一个叫 cargo 的符号链接，指到 true（target-prune.py --selftest 那样拿 sleep 冒充 cargo）
    gate = os.path.join(repository, "scripts", "gate")
    cargo_build = f"{shlex.quote(real_cargo or 'cargo')} build --manifest-path {shlex.quote(missing_manifest)} >/dev/null 2>&1"
    selftest_write(os.path.join(gate, "fake-light-absolute-cargo.sh"), f"#!/usr/bin/env bash\n{cargo_build}\nexit 0\n", executable=True)
    selftest_write(os.path.join(gate, "fake-light-trace-green.sh"),
                   "#!/usr/bin/env bash\nhere=\"$(cd \"$(dirname \"$0\")/../..\" && pwd)\"\n\"$here/sample-tools/cargo\" build --release >/dev/null\n"
                   f"{shlex.quote(real_cargo or 'true')} -V >/dev/null 2>&1\n{shlex.quote(real_rustc or 'true')} -vV >/dev/null 2>&1\n"
                   f"{shlex.quote(os.path.join(tools, 'herd7'))} -version >/dev/null\n{shlex.quote(os.path.join(tools, 'qemu-system-x86_64'))} --version >/dev/null\n"
                   f"{shlex.quote(os.path.join(tools, 'verus'))} --version >/dev/null\nexit 5\n", executable=True)
    names = ("qemu-system-aarch64", "herd7", "verus", "rust-verify", "rust_verify", "cargo", "rustc")
    selftest_write(os.path.join(gate, "fake-light-names-red.sh"),
                   "#!/usr/bin/env bash\n" + "".join(f"{shlex.quote(os.path.join(tools, name))} -M virt >/dev/null\n" for name in names)
                   + f"{shlex.quote(link)} -x >/dev/null\nexit 0\n", executable=True)
    selftest_write(os.path.join(gate, "fake-light-names-green.sh"),
                   "#!/usr/bin/env bash\n" + "".join(f"{shlex.quote(os.path.join(tools, name))} -M virt >/dev/null\n" for name in ("qemu-img", "cargo-fmt", "xcargo"))
                   + f"{shlex.quote(sample_tool)} test >/dev/null\n{shlex.quote(link_to_sleep)} 0 >/dev/null\nexit 0\n", executable=True)
    feeder = os.path.join(gate, "fake-stage-feeder.sh")
    selftest_write(feeder, "#!/usr/bin/env bash\n# 学 stage-selftest.sh：样本先拷进临时目录，setup.sh 在那里跑\nscratch=\"$(mktemp -d)\"\ncp -r \"$1/.\" \"$scratch/\"\n"
                           "(cd \"$scratch\" && bash setup.sh)\nstatus=$?\nrm -rf \"$scratch\"\nexit $status\n", executable=True)
    red_sample, green_sample = os.path.join(work, "samples", "setup-cargo-test-red"), os.path.join(work, "samples", "setup-fake-cargo-green")
    selftest_write(os.path.join(red_sample, "setup.sh"), f"{shlex.quote(real_cargo or 'cargo')} test --manifest-path {shlex.quote(missing_manifest)} >/dev/null 2>&1\nexit 0\n")
    selftest_write(os.path.join(green_sample, SAMPLE_TOOLS_DIRECTORY, "cargo"), '#!/usr/bin/env bash\necho "样本的假 cargo $*"\n', executable=True)
    selftest_write(os.path.join(green_sample, "setup.sh"), f"PATH=\"$PWD/{SAMPLE_TOOLS_DIRECTORY}:$PATH\" cargo test >/dev/null\nexit 0\n")

    def judged(command):
        return light_judgement(command, repository, repository, environment, broken)[0]

    def run(command, interval=0.3, settings=quiet_settings, **keywords):
        return guard(command, settings, broken, interval, None, environment, repository, quiet=True, **keywords)

    unavailable = (f"strace 起不了（{trace_reason}）" if trace_state != "yes" else "" if real_cargo else "PATH 上没有真 cargo，造不出立刻失败的 cargo build")
    # 第 9 格（红样本）：登记成 light 的假自证绝对路径起真 cargo build（清单不在，立刻失败）：采样（间隔 0.3 秒）看不见、替身不经 PATH 也拦不到，只有 execve 记录抓得到
    if unavailable:
        not_run.append((9, unavailable))
    else:
        command = ["bash", "scripts/gate/fake-light-absolute-cargo.sh", "--selftest"]
        exit_code, findings, verdict = run(command)
        named = [text for _pid, kind, text in findings if kind == EXEC_KIND and "build --manifest-path" in text and text.startswith("第 ")]
        if not (judged(command) and exit_code == FINDINGS_EXIT and named and "\ttrace=yes\t" in verdict):
            failures.append((9, f"立刻失败的真 cargo build（绝对路径起、清单不在）要被 execve 跟踪抓到、点名第几次 execve，整趟退 3（实际退 {exit_code}，判定「{verdict}」，扫到 {findings}）"))

    # 第 10 格（绿样本）：自己写的假 cargo、真 cargo -V 与 rustc -vV、假 herd7 -version、假 qemu --version、假 verus --version：一处不报，命令自己的 5 照退，execve 跟踪了不止几次
    if unavailable:
        not_run.append((10, unavailable))
    else:
        exit_code, findings, verdict = run(["bash", "scripts/gate/fake-light-trace-green.sh", "--selftest"])
        executions = re.search(r"\texecs=(\d+)$", verdict)
        if not (exit_code == 5 and not findings and "\ttrace=yes\t" in verdict and executions and int(executions.group(1)) >= 6):
            failures.append((10, f"假 cargo 与只问版本的真工具应当一处不报、照命令退 5、execve 跟踪到至少 6 次（实际退 {exit_code}，判定「{verdict}」，扫到 {findings}）"))

    # 第 11 格：按文件名认——放过的目录之外的 qemu-system-*、herd7、verus、rust-verify、rust_verify、cargo、rustc 各起一次，加一个放过的目录里解开是 herd7 的 cargo 符号链接，全红；
    # 名字不在表里的 qemu-img、cargo-fmt、xcargo、样本根假工具目录里的 cargo、放过的目录里指到 true 的 cargo 符号链接不红
    if unavailable:
        not_run.append((11, unavailable))
    else:
        exit_code, findings, _verdict = run(["bash", "scripts/gate/fake-light-names-red.sh", "--selftest"])
        texts = [text for _pid, kind, text in findings if kind == EXEC_KIND]
        missing = [name for name in names if not any(f"{name}（{os.path.realpath(os.path.join(tools, name))}）" in text for text in texts)]
        link_named = any(f"cargo（{os.path.realpath(os.path.join(tools, 'herd7'))}）" in text for text in texts)   # 名字取符号链接的 cargo，路径是解开之后的 herd7
        green_exit, green_findings, _verdict = run(["bash", "scripts/gate/fake-light-names-green.sh", "--selftest"])
        # 被判的那棵树（--registry-root）里随样本带的假 cargo：树不在放过的目录里时红，把这棵树交给 Settings 当放过的目录就绿
        tree_tool = os.path.join(outside, "judged-tree", "sample-tools", "cargo")
        selftest_write(tree_tool, '#!/usr/bin/env bash\necho "样本根里的假 cargo $*"\n', executable=True)
        tree_script = os.path.join(gate, "fake-light-tree-tool.sh")
        selftest_write(tree_script, f"#!/usr/bin/env bash\n{shlex.quote(tree_tool)} test >/dev/null\nexit 0\n", executable=True)
        tree_settings = Settings([], [], "", os.path.join(work, "private"), exempt_roots=[work, os.path.join(outside, "judged-tree")])
        tree_red = run(["bash", "scripts/gate/fake-light-tree-tool.sh", "--selftest"])[0]
        tree_green = run(["bash", "scripts/gate/fake-light-tree-tool.sh", "--selftest"], settings=tree_settings)[0]
        if tree_red != FINDINGS_EXIT or tree_green != 0:
            failures.append((11, f"被判的那棵树里的假 cargo：树不在放过的目录里时应当红退 3（实际 {tree_red}），树在放过的目录里时应当绿退 0（实际 {tree_green}）"))
        if exit_code != FINDINGS_EXIT or missing or len(texts) != len(names) + 1 or not link_named or green_exit != 0 or green_findings:
            failures.append((11, f"qemu-system-*、herd7、verus、rust-verify、cargo、rustc 与解开是别的工具的符号链接要按文件名红（缺 {missing}，共 {len(texts)} 处，要 {len(names) + 1}），"
                                 f"qemu-img、cargo-fmt、xcargo、样本根假工具目录里的 cargo 与指到 true 的 cargo 符号链接不红（实际退 {green_exit}，扫到 {green_findings}）"))

    # 第 12 格：样本档——stage-selftest.sh 喂样本时 setup.sh 在拷出来的临时目录里跑。样本 setup.sh 里写 cargo test 红、只用样本自己的假 cargo 绿；
    # 本仓那一份 stage-selftest.sh 喂登记成轻的样本档算该盯的，别处路径的拷贝不算
    own_stage_selftest = os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts", STAGE_SELFTEST_NAME)
    watch_own = light_judgement(["bash", own_stage_selftest, "--list-items"], REPOSITORY_ROOT, None, environment, broken)[0]
    watch_copy = light_judgement(["bash", os.path.join(outside, STAGE_SELFTEST_NAME), "--list-items"], REPOSITORY_ROOT, None, environment, broken)[0]
    if unavailable:
        not_run.append((12, unavailable))
    else:
        red_exit, red_findings, _verdict = run(["bash", feeder, red_sample])
        green_exit, green_findings, _verdict = run(["bash", feeder, green_sample])
        red_named = [text for _pid, kind, text in red_findings if kind == EXEC_KIND and " test --manifest-path" in text]
        if not (watch_own and not watch_copy and red_exit == FINDINGS_EXIT and red_named and green_exit == 0 and not green_findings):
            failures.append((12, f"样本档：setup.sh 里写 cargo test 应当红退 3（实际退 {red_exit}，扫到 {red_findings}），只用样本假 cargo 的应当绿（实际退 {green_exit}，扫到 {green_findings}）；"
                                 f"本仓的 stage-selftest.sh --list-items 应当盯（{watch_own}）、别处路径的拷贝不盯（{watch_copy}）"))

    # 第 13 格：strace 不在（给一个不存在的路径）：不假装绿——判定文件写 trace=no，成功句或 ✗ 说明里写「execve 跟踪本次未跑」，也不写「没见真 … qemu-system-*」
    captured = io.StringIO()
    with contextlib.redirect_stderr(captured):
        exit_code, _findings, verdict = guard(["bash", "scripts/gate/fake-light-absolute-cargo.sh", "--selftest"], quiet_settings, broken, 0.3, None, environment, repository,
                                              strace_binary=os.path.join(work, "no-such-strace"))
    said = captured.getvalue()
    if not ("\ttrace=no\t" in verdict and "\texecs=0" in verdict and "execve 跟踪本次未跑" in said and "qemu-system-*" not in said):
        failures.append((13, f"strace 不在时判定文件应写 trace=no、execs=0，说明里应写「execve 跟踪本次未跑」而不是照常说没见真工具（判定「{verdict}」，说明「{said.strip()}」）"))
    return not_run


if __name__ == "__main__":
    preflight(__file__, own_arguments(sys.argv[1:]))
    sys.exit(main(sys.argv[1:]))
