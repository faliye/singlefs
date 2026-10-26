"""主工作区上起 cargo 之前认合入锁：锁被别的活着的进程占着，这条命令执行前拒绝。

heavy-test-guard.sh（PreToolUse[Bash]）在 decide() 里调这一份，按文件路径用 importlib 导入；判锁用 scripts/gate/lib_workspace_lock.py 的 status /
holder_alive / is_self_or_ancestor / describe（判法不在这里另写一份，锁文件的格式与死活判法只有那一份）。

为什么：两条线往同一个主工作区合入、在上面跑重活，互相撞，编译错、两趟都白跑
（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 20 行）；
合入这一半由 scripts/gate/merge-and-verify.sh 与 scripts/gate/apply-writer-patch.py 持锁，这里是「主工作区上的 cargo 起之前认锁」那一半。

判法（workspace_cargo_lock_refusal）：
  ① 命令位置上有 cargo：包装（bash -c、nice、timeout、env、capped.sh N、run-with-memory-cap.sh、/usr/bin/time 这一类启动器、gpu-quiesce-run.sh）
     剥掉之后的命令词是 cargo，子命令不是只查版本与清单的那几个（NON_BUILDING_SUBCOMMANDS）。目录取 cd / -C / --manifest-path 之后的。
  ② 那个目录在一棵 git 工作树里：取所在工作树的根。主工作树与 git worktree 加出来的树都算，各认各自那一把锁（合入锁按工作树分开，
     scripts/gate/lib_workspace_lock.py 文件头「锁文件」）；不在 git 仓里的仓副本不判；
     目录认不出（cd 到展开不了的变量路径；同一段命令里前面赋过的变量与 $HOME 展开之后照判，lib_shell_words.py 的 expand_known_variables）的也不判。
  ③ 锁文件（SINGLEFS_WORKSPACE_LOCK_FILE，或 <那棵工作树自己的 git 目录>/singlefs-main-workspace.lock，位置由 lib_workspace_lock.py 的 lock_path 定）
     里有持有者、而且它活着：拒。记录空着、持有者已死的放行。
  ④ 持有者之下的放行：SINGLEFS_WORKSPACE_LOCK_HOLDER（命令前写的，或钩子自己的环境里的）指着记录里的持有者、它是这个钩子进程自己或祖先
     （lib_workspace_lock.py 的重入判法）。
  读不了锁（建不了锁文件、读不到 lib_workspace_lock.py）不拒，交回一句 notice，由调用方记检出、照常执行。
  不读脚本正文：`bash x.sh` 里的 cargo 不在这一道的射程里（gate.sh、merge-and-verify.sh 这类自己持锁或不起 cargo 的另算）。
  不排队：钩子不等锁，被占着就拒，拒绝说明给出谁占着与下一步（等它放，或在副本里编）。

编译目录（workspace_cargo_target_refusal；heavy-test-guard.sh 在 decide() 里同合入锁一起调，主 agent 与子 agent 一样判）：
  用户定案「所有脚本自行编译，完成后自行删除 不再预留生成物」：主工作区里直接敲的 cargo 编进 <仓根>/target、删了又长，这一判在执行前拒。
  ① 命令位置上的每一条 cargo 照 cargo_invocations 取（与合入锁共用：包装剥掉之后剥出来的那条再切一遍，bash -c、cd 跟着走；run-build-directory.sh 那一层记下来；
     命令文本本身里只定义、调不到的 `function 名字 { …; }` 的函数体不判，与重型那一处同一个口径，拆开拼写调到的、认不准的照判：lib_shell_words.py 的 function_never_called）；
     子命令是会编的（BUILDING_SUBCOMMANDS：build、check、test、bench、run、doc、clippy、rustc、rustdoc、fix 与它们的内建别名）才判，fmt、metadata、tree、clean 这类放行。
  ② 起它的目录所在的 git 工作树根要正是主仓（调用方给的 main_root，钩子给 project_root）：仓副本、git worktree 加出来的树、目录认不出的不判。
  ③ 编译目录：显式给的（--target-dir、CARGO_TARGET_DIR、CARGO_BUILD_TARGET_DIR、--config build.target-dir=…；变量先看命令文本里设的，再看钩子自己的环境）在仓外放行、在仓里拒；
     没显式给而包在 scripts/hosts/run-build-directory.sh 里的放行（它给这一次一个自己的编译目录、跑完删）；再看配置文件（起它的目录往上每层的 .cargo/config.toml、
     $CARGO_HOME/config.toml）的 build.target-dir；都没有就是默认的 <工作区根>/target，在仓里，拒。写了而此刻算不出（带没赋过的变量）的不拒，交一句 notice。
  ④ 拒绝说明写出哪一条、为什么，与在第一个 cargo 前插上 `bash <主仓>/scripts/hosts/run-build-directory.sh ` 的那一条（照抄能过这一判）。
  不读脚本正文：入口脚本自己起的 cargo 不在射程里。

接口：
  workspace_cargo_lock_refusal(command, directory, shell_words, heavy_tests, lock_library_path, hook_environment=None) -> (拒绝说明 或 None, notices)
  workspace_cargo_target_refusal(command, directory, shell_words, heavy_tests, main_root, hook_environment=None) -> (拒绝说明 或 None, notices)
  cargo_invocations(command, directory, shell_words, heavy_tests) -> [CargoInvocation]   两判共用的取法
  build_held_workspace(work) -> (主工作区路径, 持锁的假进程, 锁文件路径)   自证与 heavy-test-guard.sh 的自检共用的现场（用完 holder.kill()）
  selftest() -> 0 / 1

弄坏开关 WORKSPACE_CARGO_LOCK_BREAK=<项>（几个用逗号连；只给自证判红用，不认识的项抛 ValueError）。项名、弄坏哪一处、打开后自证红在哪几格
（格名是 selftest 里 cell 的说明）：
  ignore-lock      不认锁，被占着也放行
      「锁被别的进程占着：拒」「拒绝说明给出谁占着与下一步」「cd 进主工作区再起：拒」「cd 进写在变量里的主工作区再起：拒」「经内存包装起：拒」「bash -c 里起：拒」「cargo -C 指到主工作区：拒」
      「cargo --manifest-path 指到主工作区：拒」「HOLDER 变量指着的不是祖先：拒」「持锁者自己没写 HOLDER：拒（重入要写明）」「锁文件在默认位置（主工作树的 git 目录下）：拒」「git worktree 加出来的树自己的锁被占着：拒」；
      heavy-test-guard.sh 自检「workspace-lock：主工作区里起 cargo、锁被别的进程占着：拒（退出码 2）」及它的两格说明（主 agent 与子 agent 各一）
  ignore-reentry   不认 HOLDER 变量，持锁者之下也拒
      「持锁者自己起 cargo（命令前写 HOLDER）：放行」「持锁者自己起 cargo（钩子环境里的 HOLDER）：放行」
  main-tree-only   照改前只认主工作树（.git 就是 git common-dir），git worktree 加出来的树里的 cargo 不认它自己那一把锁
      「git worktree 加出来的树自己的锁被占着：拒」
另有 scripts/gate/lib_workspace_lock.py 的 WORKSPACE_LOCK_BREAK=lock-in-common-dir（锁文件照改前放 git common-dir、所有工作树共用一把）：
      「锁文件在默认位置，主工作树的锁被占着，git worktree 加出来的树里起：放行」「git worktree 加出来的树自己的锁被占着：拒」
  refuse-free      持有者已死的记录也当锁住（记录空着的本来就没有持有者，这个开关管不到）
      「持有者已死：放行」
  not-build-too    只查版本、清单的子命令也拒
      「cargo --version 与 cargo locate-project 不拒」
编译目录那一判的四项：
  bare-cargo-allowed  不判编译目录，一律放行
      「编译目录：裸 cargo test：拒」「编译目录：cd 进 crates/ 之后的 cargo build：拒」「编译目录：经内存包装起：拒」「编译目录：capped.sh、nice、timeout、env 包着：拒」
      「编译目录：bash -c 里起：拒」「编译目录：CARGO_TARGET_DIR 指到仓里：拒」「编译目录：拒绝说明给出包一层的写法与出路」等这一判里该拒的 12 格与三方 shell-words-r2 判决第 8 条加的「编译目录：function 定义之后拆开拼写调用：拒」，
      与「编译目录：CARGO_TARGET_DIR 写在算不出的变量里：放行并记一条」（不记 notice）；heavy-test-guard.sh 自检「编译目录：」开头该拒的三格与「编译目录：function 定义之后拆开拼写调用：拒（退出码 2）」
  outside-target-not-recognized  不认显式给的编译目录（--target-dir、CARGO_TARGET_DIR、CARGO_BUILD_TARGET_DIR、--config）
      「编译目录：带仓外 CARGO_TARGET_DIR：放行」「编译目录：export 了仓外 CARGO_TARGET_DIR 再起：放行」「编译目录：会话环境里有仓外 CARGO_TARGET_DIR：放行」
      「编译目录：--target-dir 指到仓外：放行」「编译目录：--config build.target-dir 指到仓外：放行」「编译目录：包装脚本外面又给了仓里的 CARGO_TARGET_DIR：拒」等 9 格
  wrapper-not-recognized  不认 run-build-directory.sh 包着的
      「编译目录：经包装脚本起：放行」「编译目录：内存包装里套包装脚本起：放行」「编译目录：包装脚本里 bash -c 再 cd 进 crates/ 起：放行」「编译目录：照拒绝说明照抄的那一条：放行」
  non-building-refused  不编的子命令也判
      「编译目录：cargo fmt：放行」「编译目录：cargo metadata、cargo tree、cargo clean：放行」
  uncalled-functions-judged  命令文本本身里只定义、调不到的 function 的函数体也判（三方 shell-words-r2 判决第二节第 8 条之前的判法）
      「编译目录：只定义、调不到的 function 函数体：放行」；heavy-test-guard.sh 自检「编译目录：只定义、调不到的 function 函数体：放行（退出码 0）」与「lib_workspace_cargo_lock.py 的自检」
另有 lib_shell_words.py 的 LIB_SHELL_WORDS_BREAK=function-calls-literal-only（拆开拼写的调用认不出）与 function-keyword-as-command（function 关键字当命令词）：「编译目录：function 定义之后拆开拼写调用：拒」；
uncalled-functions-run（只定义的函数体照判）：「编译目录：只定义、调不到的 function 函数体：放行」
另有 lib_heavy_tests.py 的 LIB_HEAVY_TESTS_BREAK=run-build-directory-opaque（包装里那条看不见）：「编译目录：包装脚本外面又给了仓里的 CARGO_TARGET_DIR：拒」
"""
import os
import re
import shlex
import subprocess
import sys
import tempfile
import time
import tomllib
from typing import NamedTuple

BREAK_VARIABLE = "WORKSPACE_CARGO_LOCK_BREAK"
BREAK_ITEMS = ("ignore-lock", "ignore-reentry", "main-tree-only", "refuse-free", "not-build-too",
               "bare-cargo-allowed", "outside-target-not-recognized", "wrapper-not-recognized", "non-building-refused", "uncalled-functions-judged")
# 编译目录那一判（文件头「编译目录」一段）：会编、会往编译目录里写的子命令（别名照 cargo 内建的认）；别的（fmt、metadata、tree、clean、update……）不判
BUILDING_SUBCOMMANDS = {"build", "b", "check", "c", "test", "t", "bench", "run", "r", "doc", "d", "clippy", "rustc", "rustdoc", "fix"}
TARGET_DIRECTORY_VARIABLES = ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR")
RUN_BUILD_DIRECTORY_SCRIPT = "scripts/hosts/run-build-directory.sh"
# 拒绝说明里给的「包一层」写法插在命令文本里第一个像命令词的 cargo 之前（前面是行首、空白、;&|( 或引号）
FIRST_CARGO_WORD = re.compile(r"""(?:^|(?<=[\s;&|('"]))cargo(?=\s|$)""", re.M)
HOLDER_VARIABLE = "SINGLEFS_WORKSPACE_LOCK_HOLDER"
LOCK_FILE_VARIABLE = "SINGLEFS_WORKSPACE_LOCK_FILE"
# 不编、不动 target 的子命令：只查版本、清单与帮助
NON_BUILDING_SUBCOMMANDS = {"version", "help", "locate-project", "pkgid", "read-manifest", "verify-project"}
LAUNCHER_LAYERS = 4


def switch_is_set(item):
    chosen = [name for name in os.environ.get(BREAK_VARIABLE, "").split(",") if name]
    unknown = [name for name in chosen if name not in BREAK_ITEMS]
    if unknown:
        raise ValueError(f"{BREAK_VARIABLE} 里有不认识的项 {unknown}（认的：{', '.join(BREAK_ITEMS)}）")
    return item in chosen


class CargoInvocation(NamedTuple):
    subcommand: str
    directory: str | None          # 起它的目录（cd、env -C、-C、--manifest-path 之后的）；认不出是 None
    environment: dict              # 这条命令看得到的、命令文本里设过的变量（lib_shell_words 的 CommandAtPosition.environment，值照写的样子）
    rest: list                     # 子命令后面的参数
    configuration_values: list     # 全局选项 --config 的值（lib_heavy_tests 的 CargoCommand.configuration_values）
    variables: dict                # 跑这条命令时命令文本里记下的变量（展开参数用）
    own_build_directory: bool      # 包在 scripts/hosts/run-build-directory.sh 里（这一趟一个自己的编译目录）


def cargo_invocations(command, directory, shell_words, heavy_tests, environment=None, own_build_directory=False, depth=0):
    """命令文本里命令位置上起的每一条 cargo（合入锁与编译目录两判共用这一份）：包装（command_under_launcher 认的那几样、gpu-quiesce-run.sh）剥掉之后，
    剥出来的那条当一段命令文本再切一遍（里面的 bash -c、cd 照 lib_shell_words 跟着走），最多套 LAUNCHER_LAYERS 层；剥的那一层是 run-build-directory.sh 的，
    里面起的每一条记 own_build_directory（弄坏开关 wrapper-not-recognized 下不记）。→ [CargoInvocation]。
    最外一层（depth 0，Bash 调用的命令文本本身）里只定义、调不到的 `function 名字 { …; }` 的函数体不判，与 heavy-test-guard.sh 判重型那一处同一个口径
    （lib_shell_words.py 的 uncalled_functions_skipped / function_never_called：拆开拼写调到的、认不准的照判；剥出来的里一层照判函数体）；
    弄坏开关 uncalled-functions-judged 下最外一层也照判函数体（改前的判法）。"""
    invocations = []
    skip_uncalled = depth == 0 and not switch_is_set("uncalled-functions-judged")
    for found in shell_words.commands_at_command_position(command, directory, environment, uncalled_functions_skipped=skip_uncalled).commands:
        words, where = list(found.words), found.directory
        launched = heavy_tests.command_under_launcher(words, where) or heavy_tests.command_under_gpu_quiesce(words, where)
        if launched and launched[0]:
            if depth < LAUNCHER_LAYERS:
                wrapped = own_build_directory or (heavy_tests.command_under_run_build_directory(words, where) is not None
                                                  and not switch_is_set("wrapper-not-recognized"))
                inner_directory = shell_words.resolve_path(where, launched[1]) if launched[1] and where is not None else where
                invocations += cargo_invocations(shlex.join(launched[0]), inner_directory, shell_words, heavy_tests, found.environment, wrapped, depth + 1)
            continue
        if not words or os.path.basename(words[0]) != "cargo":
            continue
        cargo = heavy_tests.cargo_subcommand(words[1:], where)
        if cargo is None:
            continue
        where = cargo.directory
        for index, argument in enumerate(cargo.rest):
            manifest = argument[len("--manifest-path="):] if argument.startswith("--manifest-path=") else (
                cargo.rest[index + 1] if argument == "--manifest-path" and index + 1 < len(cargo.rest) else None)
            if manifest is not None:
                resolved = shell_words.resolve_path(where, manifest)
                where = os.path.dirname(resolved) if resolved else None
                break
        invocations.append(CargoInvocation(cargo.subcommand, where, dict(found.environment), list(cargo.rest), list(cargo.configuration_values),
                                           dict(found.variables or {}), own_build_directory))
    return invocations


def cargo_runs_in(command, directory, shell_words, heavy_tests):
    """命令文本里命令位置上起的、要认合入锁的每一条 cargo：→ [(cargo 的子命令, 起它的目录或 None, 命令前后写的 HOLDER 变量或 "")]。"""
    return [(run.subcommand, run.directory, run.environment.get(HOLDER_VARIABLE, ""))
            for run in cargo_invocations(command, directory, shell_words, heavy_tests)
            if run.subcommand not in NON_BUILDING_SUBCOMMANDS or switch_is_set("not-build-too")]


def work_tree_root(directory):
    """directory 所在的 git 工作树根：主工作树与 git worktree 加出来的树都交（各认各自的那一把锁）；不在仓里的交 None。
    弄坏开关 main-tree-only 下照改前只交主工作树（.git 就是 git common-dir）。"""
    if not directory or not os.path.isdir(directory):
        return None
    environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    completed = subprocess.run(["git", "-C", directory, "rev-parse", "--show-toplevel", "--git-common-dir"],
                               capture_output=True, text=True, env=environment)
    lines = completed.stdout.splitlines()
    if completed.returncode != 0 or len(lines) < 2:
        return None
    top = os.path.realpath(lines[0])
    common = lines[1] if os.path.isabs(lines[1]) else os.path.join(directory, lines[1])
    if not switch_is_set("main-tree-only") or os.path.realpath(common) == os.path.realpath(os.path.join(top, ".git")):
        return top
    return None


def load_lock_library(lock_library_path):
    import importlib.util
    specification = importlib.util.spec_from_file_location("lib_workspace_lock_for_hook", lock_library_path)
    library = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(library)
    return library


def refusal_text(command_detail, tree, path, record, lock):
    """拒绝说明：谁占着照 lib_workspace_lock.py status 的输出，再写下一步。"""
    return "\n".join([
        f"✗ 主工作区合入锁被别的进程占着，这条命令要在主工作区 {tree} 里起 cargo：{command_detail}",
        f"  谁占着（照 `python3 scripts/gate/lib_workspace_lock.py status --root {tree}` 的输出）：主工作区合入锁 {path}：{lock.describe(record)}",
        "→ 怎么办：",
        f"  ① 等它放：隔一会儿跑 `python3 scripts/gate/lib_workspace_lock.py status --root {tree}`，打出「没人拿」再重跑这一条；",
        "  ② 不想等就在仓副本里编：只拷要编的（crates/、Cargo.toml、Cargo.lock、.cargo）到自己的草稿目录，cd 进副本再起 cargo，副本里起的不认这把锁；",
        "  持锁的该停就按进程号停它（停了之后锁自动判失效），不删锁文件，也不换写法绕过去（锁的来历与持锁方见 scripts/gate/lib_workspace_lock.py 文件头）。",
    ])


def workspace_cargo_lock_refusal(command, directory, shell_words, heavy_tests, lock_library_path, hook_environment=None):
    """→ (拒绝说明 或 None, notices)。notices 是没判成的原因（读不到锁库、读不了锁），不拒、由调用方记检出。"""
    hook_environment = os.environ if hook_environment is None else hook_environment
    if switch_is_set("ignore-lock"):
        return None, []
    runs = cargo_runs_in(command, directory, shell_words, heavy_tests)
    if not runs:
        return None, []
    in_main_workspace = [(subcommand, work_tree_root(where), command_holder) for subcommand, where, command_holder in runs]
    in_main_workspace = [entry for entry in in_main_workspace if entry[1] is not None]
    if not in_main_workspace:
        return None, []
    try:
        lock = load_lock_library(lock_library_path)
    except Exception as error:   # 文件不在、语法错、导入时抛的都算读不到
        return None, [f"读不到主工作区合入锁的库 {lock_library_path}：{error!r}，这条 cargo 没认锁"]
    for subcommand, tree, command_holder in in_main_workspace:
        try:
            path, record = lock.status(tree)
        except Exception as error:   # LockError（建不了锁文件）与别的
            return None, [f"读不了 {tree} 的主工作区合入锁：{getattr(error, 'problem', repr(error))}，这条 cargo 没认锁"]
        alive = bool(record) and lock.holder_alive(record)
        if not record or (not alive and not switch_is_set("refuse-free")):
            continue
        holder = command_holder or hook_environment.get(HOLDER_VARIABLE, "")
        if alive and not switch_is_set("ignore-reentry") and holder == record.get("pid") and lock.is_self_or_ancestor(int(record["pid"])):
            continue
        return refusal_text(f"cargo {subcommand}", tree, path, record, lock), []
    return None, []


# ── 编译目录（文件头「编译目录」一段） ──
def inside(path, root):
    path, root = os.path.realpath(path), os.path.realpath(root)
    return path == root or path.startswith(root + os.sep)


def written_directory(text, invocation, shell_words, from_command):
    """命令里写的一个目录（命令前的赋值、--target-dir 的值）→ 绝对路径；带算不出的变量、命令替换的交 None。相对路径照 cargo 按它起的目录解。
    from_command：值照写的样子（要展开 ~ 与变量）；钩子环境里读到的是已展开的值。"""
    value = shell_words.assigned_value(text, invocation.variables) if from_command else text
    if value is None or invocation.directory is None:
        return None
    return shell_words.resolve_path(invocation.directory, value)


def explicit_target_directory(invocation, shell_words, hook_environment):
    """这条 cargo 显式给的编译目录：→ (从哪来, 绝对路径或 None)；路径 None 是写了而此刻算不出。哪儿都没给交 (None, None)。
    次序照 cargo：--target-dir，CARGO_TARGET_DIR，CARGO_BUILD_TARGET_DIR，--config build.target-dir=…；变量先看命令文本里设的，再看钩子自己的环境。
    弄坏开关 outside-target-not-recognized 下一律交 (None, None)。"""
    if switch_is_set("outside-target-not-recognized"):
        return None, None
    rest = invocation.rest[: invocation.rest.index("--")] if "--" in invocation.rest else invocation.rest
    for index, argument in enumerate(rest):
        value = argument[len("--target-dir="):] if argument.startswith("--target-dir=") else (
            rest[index + 1] if argument == "--target-dir" and index + 1 < len(rest) else None)
        if value is not None:
            return "--target-dir", written_directory(value, invocation, shell_words, True)
    for name in TARGET_DIRECTORY_VARIABLES:
        if invocation.environment.get(name):
            return f"命令里的 {name}", written_directory(invocation.environment[name], invocation, shell_words, True)
        if hook_environment.get(name):
            return f"会话环境里的 {name}", written_directory(hook_environment[name], invocation, shell_words, False)
    for value in invocation.configuration_values:
        key, separator, assigned = value.partition("=")
        if separator and key.strip() == "build.target-dir":
            return "--config build.target-dir", written_directory(assigned.strip().strip("\"'"), invocation, shell_words, True)
    return None, None


def configured_target_directory(directory, hook_environment):
    """cargo 配置文件里的 build.target-dir（从起它的目录往上每一层的 .cargo/config.toml、.cargo/config，再 $CARGO_HOME/config.toml）：
    → (从哪来, 绝对路径) 或 (None, None)。相对路径照 cargo 按那个 .cargo 目录的上一层解。读不出的文件跳过。"""
    candidates, current = [], os.path.realpath(directory)
    while True:
        candidates += [(current, os.path.join(current, ".cargo", name)) for name in ("config.toml", "config")]
        parent = os.path.dirname(current)
        if parent == current:
            break
        current = parent
    cargo_home = hook_environment.get("CARGO_HOME") or os.path.join(os.path.expanduser("~"), ".cargo")
    candidates += [(os.path.dirname(cargo_home), os.path.join(cargo_home, name)) for name in ("config.toml", "config")]
    for base, path in candidates:
        try:
            with open(path, "rb") as handle:
                configured = tomllib.load(handle).get("build", {}).get("target-dir")
        except (OSError, ValueError, AttributeError):
            continue
        if isinstance(configured, str) and configured:
            return f"配置文件 {path} 的 build.target-dir", os.path.normpath(os.path.join(base, configured))
    return None, None


def wrapped_suggestion(command, main):
    """拒绝说明里能照抄的那一条：第一个像命令词的 cargo 之前插 `bash <主仓>/scripts/hosts/run-build-directory.sh `；找不到交 None。"""
    found = FIRST_CARGO_WORD.search(command)
    if not found:
        return None
    return command[: found.start()] + f"bash {os.path.join(main, RUN_BUILD_DIRECTORY_SCRIPT)} " + command[found.start():]


def workspace_cargo_target_refusal(command, directory, shell_words, heavy_tests, main_root, hook_environment=None):
    """主仓（main_root 所在那棵主工作树）里起、编译目录会落进主仓的 cargo（会编的子命令）：→ (拒绝说明 或 None, notices)。
    放行：不编的子命令；起它的目录不在主仓这棵工作树里（仓副本、git worktree 加出来的树、目录认不出）；显式给的编译目录在仓外；
    包在 run-build-directory.sh 里而没有显式给编译目录；配置文件给的 build.target-dir 在仓外。编译目录写了而此刻算不出的不拒，交一句 notice。
    弄坏开关 bare-cargo-allowed 下一律放行；non-building-refused 下不编的子命令也判。"""
    hook_environment = os.environ if hook_environment is None else hook_environment
    if switch_is_set("bare-cargo-allowed"):
        return None, []
    main = os.path.realpath(main_root)
    refused, notices = [], []
    for run in cargo_invocations(command, directory, shell_words, heavy_tests):
        if run.subcommand not in BUILDING_SUBCOMMANDS and not switch_is_set("non-building-refused"):
            continue
        if run.directory is None or work_tree_root(run.directory) != main:
            continue
        source, target = explicit_target_directory(run, shell_words, hook_environment)
        if source is None and run.own_build_directory:
            continue
        if source is None:
            source, target = configured_target_directory(run.directory, hook_environment)
        if source is not None and target is None:
            notices.append(f"cargo {run.subcommand} 的编译目录写在{source}里、此刻算不出落在哪，没判它会不会编进 {main} 里")
            continue
        if target is not None and not inside(target, main):
            continue
        reason = f"{source}指到仓里的 {target}" if source else f"没给编译目录（--target-dir、CARGO_TARGET_DIR 都没有），默认编进 {main}/target"
        refused.append(f"cargo {run.subcommand}（{reason}）")
    if not refused:
        return None, notices
    suggestion = wrapped_suggestion(command, main)
    lines = [f"✗ 主工作区里直接起的 cargo 会编进主仓 {main} 里：{refused[0]}"] + [f"  另有：{entry}" for entry in refused[1:]]
    lines += ["  （用户定案「所有脚本自行编译，完成后自行删除 不再预留生成物」：编译目录用完就删，主工作区的 target/ 删了又长的那条路由这一道堵上）",
              f"→ 怎么办：经 bash {RUN_BUILD_DIRECTORY_SCRIPT} 包一层起，它给这一次一个自己的编译目录、跑完删（正常退、出错、被 TERM 都删）："]
    if suggestion:
        lines.append(f"    {suggestion}")
    lines += ["  范围声明（SINGLEFS_RUN_SCOPE）照旧写在最前面，run-with-memory-cap.sh、capped.sh、nice、timeout、env 照旧写在它外面；",
              "  或给 CARGO_TARGET_DIR 一个仓外、自己用完删的目录（例 CARGO_TARGET_DIR=<草稿目录>/target），交回或收工之前删掉；",
              "  不编的子命令（fmt、metadata、tree、clean）不拦，入口脚本自己起的 cargo 不经这一道；判法在 .claude/hooks/lib_workspace_cargo_lock.py 文件头「编译目录」一段。"]
    return "\n".join(lines), notices


# ── 自证 ──
def build_held_workspace(work):
    """work 下摆一个主工作区（git 仓）、一个持锁的假进程（sleep），锁文件设成 work 下的 held.lock（SINGLEFS_WORKSPACE_LOCK_FILE 由调用方设）。
    → (主工作区路径, 持锁的假进程, 锁文件路径)；用完 holder.kill()。"""
    main = os.path.join(work, "main")
    os.makedirs(main)
    environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    subprocess.run(["git", "init", "-q", main], check=True, env=environment, capture_output=True)
    subprocess.run(["git", "-C", main, "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "x"],
                   check=True, env=environment, capture_output=True)
    holder = subprocess.Popen(["sleep", "120"])
    lock_file = os.path.join(work, "held.lock")
    with open(lock_file, "w", encoding="utf-8") as handle:
        start = open(f"/proc/{holder.pid}/stat", encoding="utf-8").read().rsplit(")", 1)[1].split()[19]
        handle.write(f"pid={holder.pid}\nstart={start}\nsession=别的会话\ncommand=假合入 merge-and-verify\nsince={time.time()}\n")
    return main, holder, lock_file


def selftest():
    import importlib.util
    hooks_directory = os.path.dirname(os.path.realpath(__file__))
    root = os.path.dirname(os.path.dirname(hooks_directory))
    lock_library_path = os.path.join(root, "scripts", "gate", "lib_workspace_lock.py")
    heavy_path = os.path.join(hooks_directory, "lib_heavy_tests.py")
    specification = importlib.util.spec_from_file_location("lib_heavy_tests_for_cargo_lock_selftest", heavy_path)
    heavy_tests = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(heavy_tests)
    shell_words = heavy_tests.shell_words
    failures, checked = [], 0
    work = tempfile.mkdtemp(prefix="workspace-cargo-lock-selftest-")
    saved = {name: os.environ.pop(name, None) for name in (LOCK_FILE_VARIABLE, HOLDER_VARIABLE)}
    holder = None
    try:
        main, holder, held_lock = build_held_workspace(work)
        os.environ[LOCK_FILE_VARIABLE] = held_lock
        plain_copy = os.path.join(work, "plain-copy")
        os.makedirs(plain_copy)
        linked = os.path.join(work, "linked")
        environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
        subprocess.run(["git", "-C", main, "worktree", "add", "-q", linked], check=True, env=environment, capture_output=True)
        lock = load_lock_library(lock_library_path)

        def judge(command, directory, hook_environment=None):
            return workspace_cargo_lock_refusal(command, directory, shell_words, heavy_tests, lock_library_path, hook_environment or {})

        def cell(name, command, directory, want_refused, hook_environment=None, extra=None):
            nonlocal checked
            checked += 1
            try:
                message, notices = judge(command, directory, hook_environment)
            except Exception as error:
                failures.append(f"{name}：判的那一刻抛 {error!r}")
                return
            if bool(message) != want_refused or notices:
                failures.append(f"{name}：命令 {command!r}（目录 {directory}）应当{'拒' if want_refused else '放行'}，"
                                f"实际 {'拒' if message else '放行'}，notices {notices}，说明 {(message or '')[:200]!r}")
            elif message and extra and not extra(message):
                failures.append(f"{name}：拒了，但说明不合要求：{message!r}")

        names_holder = lambda message: f"进程 {holder.pid}" in message and "别的会话" in message and "假合入 merge-and-verify" in message
        names_next_step = lambda message: names_holder(message) and "→ 怎么办" in message and "等它放" in message and "仓副本" in message \
            and f"lib_workspace_lock.py status --root {os.path.realpath(main)}" in message
        cell("锁被别的进程占着：拒", "cargo build", main, True, extra=names_holder)
        cell("拒绝说明给出谁占着与下一步", "cargo test --lib", main, True, extra=names_next_step)
        cell("cd 进主工作区再起：拒", f"cd {main} && cargo check", work, True)
        cell("cd 进写在变量里的主工作区再起：拒", f"D={main}; cd $D && cargo check", work, True)
        cell("经内存包装起：拒", "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p x --lib", main, True)
        cell("bash -c 里起：拒", "bash -c 'cargo fmt --all'", main, True)
        cell("cargo -C 指到主工作区：拒", f"cargo -C {main} build", work, True)
        cell("cargo --manifest-path 指到主工作区：拒", f"cargo build --manifest-path {main}/Cargo.toml", work, True)
        cell("HOLDER 变量指着的不是祖先：拒", f"{HOLDER_VARIABLE}={holder.pid} cargo build", main, True)
        # 持锁者自己（这个自证进程：把记录换成自己）再起 cargo：命令前写 HOLDER、或钩子环境里有 HOLDER
        own_lock = os.path.join(work, "own.lock")
        start = lock.process_start(os.getpid())
        with open(own_lock, "w", encoding="utf-8") as handle:
            handle.write(f"pid={os.getpid()}\nstart={start}\nsession=自证\ncommand=持锁者自己\nsince={time.time()}\n")
        os.environ[LOCK_FILE_VARIABLE] = own_lock
        cell("持锁者自己没写 HOLDER：拒（重入要写明）", "cargo build", main, True)
        cell("持锁者自己起 cargo（命令前写 HOLDER）：放行", f"{HOLDER_VARIABLE}={os.getpid()} cargo build", main, False)
        cell("持锁者自己起 cargo（钩子环境里的 HOLDER）：放行", "cargo build", main, False, hook_environment={HOLDER_VARIABLE: str(os.getpid())})
        # 锁空着、持有者已死
        free_lock = os.path.join(work, "free.lock")
        open(free_lock, "w").close()
        os.environ[LOCK_FILE_VARIABLE] = free_lock
        cell("锁空着：放行", "cargo build", main, False)
        dead = subprocess.Popen(["true"])
        dead.wait()
        with open(free_lock, "w", encoding="utf-8") as handle:
            handle.write(f"pid={dead.pid}\nstart=1\nsession=死掉的会话\ncommand=假合入 dead\nsince={time.time()}\n")
        cell("持有者已死：放行", "cargo build", main, False)
        # 副本与别的树：锁文件仍指着被占着的那份，这些地方起的 cargo 不认这把锁
        os.environ[LOCK_FILE_VARIABLE] = held_lock
        cell("仓副本（不在 git 仓里）：放行", "cargo build", plain_copy, False)
        cell("目录认不出（cd 到变量路径之后）：放行", "cd $SOMEWHERE && cargo build", None, False)
        # 不是起 cargo
        cell("cargo --version 与 cargo locate-project 不拒", "cargo --version; cargo locate-project", main, False)
        cell("命令里只是提到 cargo 的字：放行", "echo cargo build; ls main", main, False)
        cell("grep 里写 cargo：放行", "grep -n cargo Cargo.toml", main, False)
        # 锁文件在默认位置（不设 SINGLEFS_WORKSPACE_LOCK_FILE）：主工作区的 .git 下
        os.environ.pop(LOCK_FILE_VARIABLE, None)
        default_lock = os.path.join(main, ".git", "singlefs-main-workspace.lock")
        start = lock.process_start(holder.pid)
        with open(default_lock, "w", encoding="utf-8") as handle:
            handle.write(f"pid={holder.pid}\nstart={start}\nsession=别的会话\ncommand=假合入 merge-and-verify\nsince={time.time()}\n")
        cell("锁文件在默认位置（主工作树的 git 目录下）：拒", "cargo build", main, True, extra=names_holder)
        cell("锁文件在默认位置，主工作树的锁被占着，git worktree 加出来的树里起：放行", "cargo build", linked, False)
        # git worktree 加出来的树自己那一把（<主工作树>/.git/worktrees/<名>/ 下）被占着、主工作树那一把空着：那棵树里起 cargo 拒
        with open(default_lock, "w", encoding="utf-8") as handle:
            handle.write("")
        # 位置照 git 自己报的 worktree 的 git 目录取，不经 lock_path（弄坏开关 lock-in-common-dir 下 lock_path 指回主工作树那一把，这一格要红）
        linked_git_directory = subprocess.run(["git", "-C", linked, "rev-parse", "--absolute-git-dir"], check=True, capture_output=True, text=True,
                                              env=environment).stdout.strip()
        linked_lock = os.path.join(linked_git_directory, "singlefs-main-workspace.lock")
        with open(linked_lock, "w", encoding="utf-8") as handle:
            handle.write(f"pid={holder.pid}\nstart={start}\nsession=别的会话\ncommand=假合入 merge-and-verify\nsince={time.time()}\n")
        cell("git worktree 加出来的树自己的锁被占着：拒", "cargo build", linked, True, extra=names_holder)

        # 编译目录（文件头「编译目录」一段）：主仓是 main（临时 git 仓），钩子环境只给一个空的 CARGO_HOME（不看本会话与本机用户真有的配置）
        crates = os.path.join(main, "crates")
        os.makedirs(os.path.join(crates, "singlefs-harness"), exist_ok=True)
        outside = os.path.join(work, "outside-target")
        isolated = {"CARGO_HOME": os.path.join(work, "cargo-home")}
        configured_outside = os.path.join(main, "configured-outside")
        configured_inside = os.path.join(main, "configured-inside")
        for configured, written in ((configured_outside, os.path.join(work, "configured-target")), (configured_inside, "own-target")):
            os.makedirs(os.path.join(configured, ".cargo"))
            with open(os.path.join(configured, ".cargo", "config.toml"), "w", encoding="utf-8") as handle:
                handle.write(f'[build]\ntarget-dir = "{written}"\n')
        wrapper = os.path.join(os.path.realpath(main), RUN_BUILD_DIRECTORY_SCRIPT)

        def target_cell(name, command, directory, want_refused, hook_environment=None, extra=None, want_notice=False):
            nonlocal checked
            checked += 1
            try:
                message, notices = workspace_cargo_target_refusal(command, directory, shell_words, heavy_tests, main, dict(isolated, **(hook_environment or {})))
            except Exception as error:
                failures.append(f"{name}：判的那一刻抛 {error!r}")
                return
            if bool(message) != want_refused or bool(notices) != want_notice:
                failures.append(f"{name}：命令 {command!r}（目录 {directory}）应当{'拒' if want_refused else '放行'}{'、记一条 notice' if want_notice else ''}，"
                                f"实际 {'拒' if message else '放行'}，notices {notices}，说明 {(message or '')[:300]!r}")
            elif message and extra and not extra(message):
                failures.append(f"{name}：拒了，但说明不合要求：{message!r}")

        target_cell("编译目录：裸 cargo test：拒", "cargo test -p singlefs-harness --lib", main, True)
        target_cell("编译目录：cd 进 crates/ 之后的 cargo build：拒", f"cd {crates} && cargo build", work, True)
        target_cell("编译目录：cd 进 crates/ 的子目录（相对路径）之后的 cargo check：拒", "cd crates/singlefs-harness && cargo check", main, True)
        target_cell("编译目录：经内存包装起：拒", "bash scripts/hosts/run-with-memory-cap.sh 12G cargo test -p singlefs-harness --lib", main, True)
        target_cell("编译目录：capped.sh、nice、timeout、env 包着：拒", "timeout 200 nice -n 19 env A=1 bash scripts/hosts/capped.sh 31 cargo clippy --all-targets", main, True)
        target_cell("编译目录：bash -c 里起：拒", "bash -c 'cd crates && cargo build --release'", main, True)
        target_cell("编译目录：/usr/bin/time 包着 bash -c 起：拒", "/usr/bin/time -v bash -c 'cargo doc --no-deps'", main, True)
        target_cell("编译目录：CARGO_TARGET_DIR 指到仓里：拒", f"CARGO_TARGET_DIR={main}/target-mine cargo build", main, True)
        target_cell("编译目录：--target-dir 指到仓里的相对路径：拒", "cargo build --target-dir target/mine", main, True)
        target_cell("编译目录：仓里的配置文件给的 build.target-dir 在仓里：拒", "cargo build", configured_inside, True)
        target_cell("编译目录：包装脚本外面又给了仓里的 CARGO_TARGET_DIR：拒", f"CARGO_TARGET_DIR={main}/target bash scripts/hosts/run-build-directory.sh cargo build", main, True)
        target_cell("编译目录：拒绝说明给出包一层的写法与出路", "SINGLEFS_RUN_SCOPE='x' bash scripts/hosts/run-with-memory-cap.sh 12G cargo test -p singlefs-harness --lib", main, True,
                    extra=lambda message: f"bash scripts/hosts/run-with-memory-cap.sh 12G bash {wrapper} cargo test -p singlefs-harness --lib" in message
                    and "→ 怎么办" in message and "CARGO_TARGET_DIR" in message and "默认编进" in message)
        target_cell("编译目录：带仓外 CARGO_TARGET_DIR：放行", f"CARGO_TARGET_DIR={outside} cargo build", main, False)
        target_cell("编译目录：先赋变量再拼仓外 CARGO_TARGET_DIR：放行", f"D={work}; CARGO_TARGET_DIR=$D/outside-target cargo test -p singlefs-harness --lib", main, False)
        target_cell("编译目录：export 了仓外 CARGO_TARGET_DIR 再起：放行", f"export CARGO_TARGET_DIR={outside}; cargo test", main, False)
        target_cell("编译目录：经内存包装、env 给仓外 CARGO_TARGET_DIR：放行", f"bash scripts/hosts/run-with-memory-cap.sh 12G env CARGO_TARGET_DIR={outside} cargo test", main, False)
        target_cell("编译目录：会话环境里有仓外 CARGO_TARGET_DIR：放行", "cargo build", main, False, hook_environment={"CARGO_TARGET_DIR": outside})
        target_cell("编译目录：--target-dir 指到仓外：放行", f"cargo build --target-dir={outside}", main, False)
        target_cell("编译目录：--config build.target-dir 指到仓外：放行", f"cargo build --config 'build.target-dir=\"{outside}\"'", main, False)
        target_cell("编译目录：仓里的配置文件给的 build.target-dir 在仓外：放行", "cargo build", configured_outside, False)
        target_cell("编译目录：cargo fmt：放行", "cargo fmt --all", main, False)
        target_cell("编译目录：cargo metadata、cargo tree、cargo clean：放行", "cargo metadata --format-version 1; cargo tree; cargo clean", main, False)
        target_cell("编译目录：经包装脚本起：放行", "bash scripts/hosts/run-build-directory.sh cargo test -p singlefs-harness --lib", main, False)
        target_cell("编译目录：内存包装里套包装脚本起：放行", "bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/run-build-directory.sh cargo test", main, False)
        target_cell("编译目录：包装脚本里 bash -c 再 cd 进 crates/ 起：放行", "bash scripts/hosts/run-build-directory.sh bash -c 'cd crates && cargo build'", main, False)
        target_cell("编译目录：照拒绝说明照抄的那一条：放行", f"bash scripts/hosts/run-with-memory-cap.sh 12G bash {wrapper} cargo test -p singlefs-harness --lib", main, False)
        target_cell("编译目录：仓副本（不在 git 仓里）里起：放行", "cargo build", plain_copy, False)
        target_cell("编译目录：git worktree 加出来的树里起：放行", "cargo build", linked, False)
        target_cell("编译目录：CARGO_TARGET_DIR 写在算不出的变量里：放行并记一条", "CARGO_TARGET_DIR=$SOMEWHERE/target cargo build", main, False, want_notice=True)
        target_cell("编译目录：命令里只是提到 cargo 的字：放行", "echo cargo build; grep -n cargo Cargo.toml", main, False)
        # 只定义、调不到的 function 与判重型那一处同一个口径（三方 shell-words-r2 判决第二节第 2、8 条；uncalled-functions-judged 下第一格红，
        # lib_shell_words.py 的 function-calls-literal-only 下第二格红）
        target_cell("编译目录：只定义、调不到的 function 函数体：放行", "function noop { cargo build; }", main, False)
        target_cell("编译目录：function 定义之后拆开拼写调用：拒", 'function noop { cargo build; }; "no"op', main, True)
    finally:
        if holder is not None:
            holder.kill()
            holder.wait()
        for name, value in saved.items():
            os.environ.pop(name, None)
            if value is not None:
                os.environ[name] = value
        import shutil
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 workspace_cargo_lock_refusal / workspace_cargo_target_refusal / cargo_invocations / explicit_target_directory / configured_target_directory / work_tree_root"
              f"（剥包装在 lib_heavy_tests.py 的 command_under_launcher）；{BREAK_VARIABLE} 设着的话这里本来就该红（项与红的格见文件头）")
        return 1
    print(f"  ✓ lib_workspace_cargo_lock 自检通过（查了 {checked} 格）：主工作区里起 cargo 锁被别的活进程占着拒绝并写出谁占着与下一步（cd、-C、--manifest-path、内存包装、bash -c 都认），"
          "持锁者自己（HOLDER 变量）放行，锁空着、持有者已死、不在 git 仓里的仓副本、只查版本的子命令放行，默认位置的锁文件认得，"
          "git worktree 加出来的树只认它自己那一把（主工作树的被占着放行、它自己的被占着拒）；"
          "编译目录：主仓里直接起的会编的 cargo（裸起、cd 进 crates/、经内存包装、capped.sh、nice、timeout、env、bash -c、/usr/bin/time 包着，CARGO_TARGET_DIR、--target-dir、配置文件指到仓里）拒绝并给出包一层的写法，"
          "仓外的 CARGO_TARGET_DIR（命令前、export、env、会话环境）、--target-dir、--config、配置文件，run-build-directory.sh 包着的（连同内存包装里套它、它里面 bash -c），不编的子命令，仓副本与 git worktree 加出来的树放行，算不出的放行并记一条")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    print("用法：python3 .claude/hooks/lib_workspace_cargo_lock.py --selftest（判定入口按文件路径导入这份模块再调，见文件头）", file=sys.stderr)
    sys.exit(2)
