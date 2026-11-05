"""主工作区里直接起的、编译目录会落进主仓的 cargo：执行前拒绝（主 agent 与子 agent 一样判）。

project-code-guard.sh（PreToolUse[Bash]）在 scan_decision() 里调这一份，按文件路径用 importlib 导入。

编译目录（workspace_cargo_target_refusal）：
  用户定案「所有脚本自行编译，完成后自行删除 不再预留生成物」：主工作区里直接敲的 cargo 编进 <仓根>/target、删了又长，这一判在执行前拒。
  ① 命令位置上的每一条 cargo 照 cargo_invocations 取（包装剥掉之后剥出来的那条再切一遍，bash -c、cd 跟着走；run-build-directory.sh 那一层记下来；
     命令文本本身里只定义、调不到的 `function 名字 { …; }` 的函数体不判，与项目代码那一处同一个口径，拆开拼写调到的、认不准的照判：lib_shell_words.py 的 function_never_called）；
     子命令是会编的（BUILDING_SUBCOMMANDS：build、check、test、bench、run、doc、clippy、rustc、rustdoc、fix 与它们的内建别名）才判，fmt、metadata、tree、clean 这类放行。
  ② 起它的目录所在的 git 工作树根要正是主仓（调用方给的 main_root，钩子给 project_root）：仓副本、git worktree 加出来的树、目录认不出的不判。
  ③ 编译目录：显式给的（--target-dir、CARGO_TARGET_DIR、CARGO_BUILD_TARGET_DIR、--config build.target-dir=…；变量先看命令文本里设的，再看钩子自己的环境）在仓外放行、在仓里拒；
     没显式给而包在 scripts/hosts/run-build-directory.sh 里的放行（它给这一次一个自己的编译目录、跑完删）；再看配置文件（起它的目录往上每层的 .cargo/config.toml、
     $CARGO_HOME/config.toml）的 build.target-dir；都没有就是默认的 <工作区根>/target，在仓里，拒。写了而此刻算不出（带没赋过的变量）的不拒，交一句 notice。
  ④ 拒绝说明写出哪一条、为什么，与出路：经 task-server 提交（`task-server/task submit -- <原命令>`，服务给这一次一个仓外的编译目录、跑完留给提交它的 agent，
     用完 `task-server/task clean <号>` 删；records/2026-10-30-编译绕过内存总上限.md「收尾批」那一节「统一收起来、统一管理」）。不再给「包一层」「自己给仓外 CARGO_TARGET_DIR」的写法：
     不经服务直接起的会编的 cargo 由 project-code-guard.sh 文件头「经 task-server 起」① 照拒，那两种写法照抄也过不去。
  不读脚本正文：入口脚本自己起的 cargo 不在射程里。

接口：
  workspace_cargo_target_refusal(command, directory, shell_words, project_code_runs, main_root, hook_environment=None) -> (拒绝说明 或 None, notices)
  cargo_invocations(command, directory, shell_words, project_code_runs) -> [CargoInvocation]
  build_main_workspace(work) -> 主工作区路径   自证与 project-code-guard.sh 的自检共用的现场（work 下一个带空提交的临时 git 仓）
  selftest() -> 0 / 1

弄坏开关 WORKSPACE_CARGO_LOCK_BREAK=<项>（几个用逗号连；只给自证判红用，不认识的项抛 ValueError）。项名、弄坏哪一处、打开后自证红在哪几格；本份的命令行入口（--selftest 之外）设了就在一切之前退 2、点名变量只在自证里认
（格名是 selftest 里 cell 的说明）：
  WORKSPACE_CARGO_LOCK_BREAK=bare-cargo-allowed  不判编译目录，一律放行
      「编译目录：裸 cargo test：拒」「编译目录：cd 进 crates/ 之后的 cargo build：拒」「编译目录：经内存包装起：拒」「编译目录：capped.sh、nice、timeout、env 包着：拒」
      「编译目录：bash -c 里起：拒」「编译目录：CARGO_TARGET_DIR 指到仓里：拒」「编译目录：拒绝说明给出经 task-server 提交的出路」等这一判里该拒的 12 格、「cd 进写在变量里的主工作区再起：拒」与三方 shell-words-r2 判决第 8 条加的「编译目录：function 定义之后拆开拼写调用：拒」，
      与「编译目录：CARGO_TARGET_DIR 写在算不出的变量里：放行并记一条」（不记 notice）；project-code-guard.sh 自检「编译目录：」开头该拒的三格与「编译目录：function 定义之后拆开拼写调用：拒（退出码 2）」
  WORKSPACE_CARGO_LOCK_BREAK=outside-target-not-recognized  不认显式给的编译目录（--target-dir、CARGO_TARGET_DIR、CARGO_BUILD_TARGET_DIR、--config）
      「编译目录：带仓外 CARGO_TARGET_DIR：放行」「编译目录：export 了仓外 CARGO_TARGET_DIR 再起：放行」「编译目录：会话环境里有仓外 CARGO_TARGET_DIR：放行」
      「编译目录：--target-dir 指到仓外：放行」「编译目录：--config build.target-dir 指到仓外：放行」「编译目录：包装脚本外面又给了仓里的 CARGO_TARGET_DIR：拒」等 9 格
  WORKSPACE_CARGO_LOCK_BREAK=wrapper-not-recognized  不认 run-build-directory.sh 包着的
      「编译目录：经包装脚本起：放行」「编译目录：内存包装里套包装脚本起：放行」「编译目录：包装脚本里 bash -c 再 cd 进 crates/ 起：放行」
  WORKSPACE_CARGO_LOCK_BREAK=refusal-submit-dropped  拒绝说明不写经 task-server 提交与 task clean 的出路
      「编译目录：拒绝说明给出经 task-server 提交的出路」；project-code-guard.sh 自检「编译目录：拒绝说明给出经 task-server 提交的出路」「lib_workspace_cargo_lock.py 的自检」
  WORKSPACE_CARGO_LOCK_BREAK=non-building-refused  不编的子命令也判
      「编译目录：cargo fmt：放行」「编译目录：cargo metadata、cargo tree、cargo clean：放行」
  WORKSPACE_CARGO_LOCK_BREAK=uncalled-functions-judged  命令文本本身里只定义、调不到的 function 的函数体也判（三方 shell-words-r2 判决第二节第 8 条之前的判法）
      「编译目录：只定义、调不到的 function 函数体：放行」；project-code-guard.sh 自检「编译目录：只定义、调不到的 function 函数体：放行（退出码 0）」与「lib_workspace_cargo_lock.py 的自检」
另有 lib_shell_words.py 的 LIB_SHELL_WORDS_BREAK=function-calls-literal-only（拆开拼写的调用认不出）与 function-keyword-as-command（function 关键字当命令词）：「编译目录：function 定义之后拆开拼写调用：拒」；
uncalled-functions-run（只定义的函数体照判）：「编译目录：只定义、调不到的 function 函数体：放行」
另有 lib_project_code_runs.py 的 LIB_PROJECT_CODE_RUNS_BREAK=run-build-directory-opaque（包装里那条看不见）：「编译目录：包装脚本外面又给了仓里的 CARGO_TARGET_DIR：拒」
"""
import os
import shlex
import subprocess
import sys
import tempfile
import tomllib
from typing import NamedTuple

BREAK_VARIABLE = "WORKSPACE_CARGO_LOCK_BREAK"
BREAK_ITEMS = ("bare-cargo-allowed", "outside-target-not-recognized", "wrapper-not-recognized", "non-building-refused", "uncalled-functions-judged",
               "refusal-submit-dropped")
# 编译目录那一判（文件头「编译目录」一段）：会编、会往编译目录里写的子命令（别名照 cargo 内建的认）；别的（fmt、metadata、tree、clean、update……）不判
BUILDING_SUBCOMMANDS = {"build", "b", "check", "c", "test", "t", "bench", "run", "r", "doc", "d", "clippy", "rustc", "rustdoc", "fix"}
TARGET_DIRECTORY_VARIABLES = ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR")
RUN_BUILD_DIRECTORY_SCRIPT = "scripts/hosts/run-build-directory.sh"
TASK_SERVER_CLIENT = "task-server/task"
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
    configuration_values: list     # 全局选项 --config 的值（lib_project_code_runs 的 CargoCommand.configuration_values）
    variables: dict                # 跑这条命令时命令文本里记下的变量（展开参数用）
    own_build_directory: bool      # 包在 scripts/hosts/run-build-directory.sh 里（这一趟一个自己的编译目录）


def cargo_invocations(command, directory, shell_words, project_code_runs, environment=None, own_build_directory=False, depth=0):
    """命令文本里命令位置上起的每一条 cargo（编译目录那一判用这一份）：包装（command_under_launcher 认的那几样、gpu-quiesce-run.sh）剥掉之后，
    剥出来的那条当一段命令文本再切一遍（里面的 bash -c、cd 照 lib_shell_words 跟着走），最多套 LAUNCHER_LAYERS 层；剥的那一层是 run-build-directory.sh 的，
    里面起的每一条记 own_build_directory（弄坏开关 wrapper-not-recognized 下不记）。→ [CargoInvocation]。
    最外一层（depth 0，Bash 调用的命令文本本身）里只定义、调不到的 `function 名字 { …; }` 的函数体不判，与 project-code-guard.sh 判项目代码那一处同一个口径
    （lib_shell_words.py 的 uncalled_functions_skipped / function_never_called：拆开拼写调到的、认不准的照判；剥出来的里一层照判函数体）；
    弄坏开关 uncalled-functions-judged 下最外一层也照判函数体（改前的判法）。"""
    invocations = []
    skip_uncalled = depth == 0 and not switch_is_set("uncalled-functions-judged")
    for found in shell_words.commands_at_command_position(command, directory, environment, uncalled_functions_skipped=skip_uncalled).commands:
        words, where = list(found.words), found.directory
        launched = project_code_runs.command_under_launcher(words, where) or project_code_runs.command_under_gpu_quiesce(words, where)
        if launched and launched[0]:
            if depth < LAUNCHER_LAYERS:
                wrapped = own_build_directory or (project_code_runs.command_under_run_build_directory(words, where) is not None
                                                  and not switch_is_set("wrapper-not-recognized"))
                inner_directory = shell_words.resolve_path(where, launched[1]) if launched[1] and where is not None else where
                invocations += cargo_invocations(shlex.join(launched[0]), inner_directory, shell_words, project_code_runs, found.environment, wrapped, depth + 1)
            continue
        if not words or os.path.basename(words[0]) != "cargo":
            continue
        cargo = project_code_runs.cargo_subcommand(words[1:], where)
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


def work_tree_root(directory):
    """directory 所在的 git 工作树根（主工作树与 git worktree 加出来的树各交各的根）；目录不在、不在 git 仓里的交 None。"""
    if not directory or not os.path.isdir(directory):
        return None
    environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    completed = subprocess.run(["git", "-C", directory, "rev-parse", "--show-toplevel"], capture_output=True, text=True, env=environment)
    top = completed.stdout.strip()
    if completed.returncode != 0 or not top:
        return None
    return os.path.realpath(top)


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


def workspace_cargo_target_refusal(command, directory, shell_words, project_code_runs, main_root, hook_environment=None):
    """主仓（main_root 所在那棵主工作树）里起、编译目录会落进主仓的 cargo（会编的子命令）：→ (拒绝说明 或 None, notices)。
    放行：不编的子命令；起它的目录不在主仓这棵工作树里（仓副本、git worktree 加出来的树、目录认不出）；显式给的编译目录在仓外；
    包在 run-build-directory.sh 里而没有显式给编译目录；配置文件给的 build.target-dir 在仓外。编译目录写了而此刻算不出的不拒，交一句 notice。
    弄坏开关 bare-cargo-allowed 下一律放行；non-building-refused 下不编的子命令也判。"""
    hook_environment = os.environ if hook_environment is None else hook_environment
    if switch_is_set("bare-cargo-allowed"):
        return None, []
    main = os.path.realpath(main_root)
    refused, notices = [], []
    for run in cargo_invocations(command, directory, shell_words, project_code_runs):
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
    lines = [f"✗ 主工作区里直接起的 cargo 会编进主仓 {main} 里：{refused[0]}"] + [f"  另有：{entry}" for entry in refused[1:]]
    lines.append("  （用户定案「所有脚本自行编译，完成后自行删除 不再预留生成物」：编译目录用完就删，主工作区的 target/ 删了又长的那条路由这一道堵上）")
    if not switch_is_set("refusal-submit-dropped"):
        lines += [f"→ 怎么办：经 task-server 提交：`{TASK_SERVER_CLIENT} submit -- <原命令>`（范围声明写在 submit 前面，内存包装去掉；有 &&、管道、cd 的写成 "
                  f"`{TASK_SERVER_CLIENT} submit -- bash -c '<命令>'`），再用 run_in_background 起 `{TASK_SERVER_CLIENT} wait <号>`；",
                  f"  服务给这一次一个仓外的编译目录（wait 末行 build=），跑完留给你，用完 `{TASK_SERVER_CLIENT} clean <号>` 删，不直接 rm；",
                  "  子 agent 不往统一任务服务交任务（submit、wait、clean 都不起）：把那一条拼成 submit 的整条写进报告「要主 agent 跑的命令」一节交回，由主 agent 交；",
                  "  不编的子命令（fmt、metadata、tree、clean）不拦；判法在 .claude/hooks/lib_workspace_cargo_lock.py 文件头「编译目录」一段，提交写法在 task-server/README.md「提交写法」。"]
    return "\n".join(lines), notices


# ── 自证 ──
def build_main_workspace(work):
    """work 下摆一个主工作区（带一个空提交的 git 仓）→ 它的路径。"""
    main = os.path.join(work, "main")
    os.makedirs(main)
    environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    subprocess.run(["git", "init", "-q", main], check=True, env=environment, capture_output=True)
    subprocess.run(["git", "-C", main, "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "x"],
                   check=True, env=environment, capture_output=True)
    return main


def selftest():
    import importlib.util
    import shutil
    hooks_directory = os.path.dirname(os.path.realpath(__file__))
    heavy_path = os.path.join(hooks_directory, "lib_project_code_runs.py")
    specification = importlib.util.spec_from_file_location("lib_project_code_runs_for_cargo_target_selftest", heavy_path)
    project_code_runs = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(project_code_runs)
    shell_words = project_code_runs.shell_words
    failures, checked = [], 0
    work = tempfile.mkdtemp(prefix="workspace-cargo-target-selftest-")
    try:
        main = build_main_workspace(work)
        plain_copy = os.path.join(work, "plain-copy")
        os.makedirs(plain_copy)
        linked = os.path.join(work, "linked")
        environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
        subprocess.run(["git", "-C", main, "worktree", "add", "-q", linked], check=True, env=environment, capture_output=True)

        # 编译目录（文件头「编译目录」一段）：主仓是 main（临时 git 仓），钩子环境只给一个空的 CARGO_HOME（不看本会话与本机用户真有的配置）
        crates = os.path.join(main, "crates")
        os.makedirs(os.path.join(crates, "singlefs-verification"), exist_ok=True)
        outside = os.path.join(work, "outside-target")
        isolated = {"CARGO_HOME": os.path.join(work, "cargo-home")}
        configured_outside = os.path.join(main, "configured-outside")
        configured_inside = os.path.join(main, "configured-inside")
        for configured, written in ((configured_outside, os.path.join(work, "configured-target")), (configured_inside, "own-target")):
            os.makedirs(os.path.join(configured, ".cargo"))
            with open(os.path.join(configured, ".cargo", "config.toml"), "w", encoding="utf-8") as handle:
                handle.write(f'[build]\ntarget-dir = "{written}"\n')

        def target_cell(name, command, directory, want_refused, hook_environment=None, extra=None, want_notice=False):
            nonlocal checked
            checked += 1
            try:
                message, notices = workspace_cargo_target_refusal(command, directory, shell_words, project_code_runs, main, dict(isolated, **(hook_environment or {})))
            except Exception as error:
                failures.append(f"{name}：判的那一刻抛 {error!r}")
                return
            if bool(message) != want_refused or bool(notices) != want_notice:
                failures.append(f"{name}：命令 {command!r}（目录 {directory}）应当{'拒' if want_refused else '放行'}{'、记一条 notice' if want_notice else ''}，"
                                f"实际 {'拒' if message else '放行'}，notices {notices}，说明 {(message or '')[:300]!r}")
            elif message and extra and not extra(message):
                failures.append(f"{name}：拒了，但说明不合要求：{message!r}")

        target_cell("编译目录：裸 cargo test：拒", "cargo test -p singlefs-verification --lib", main, True)
        target_cell("编译目录：cd 进 crates/ 之后的 cargo build：拒", f"cd {crates} && cargo build", work, True)
        # lib_shell_words.py 文件头写着：目录变量那两个弄坏开关打开时这一格红（cd 进写在变量里的目录之后认不出目录）
        target_cell("cd 进写在变量里的主工作区再起：拒", f"D={main}; cd $D && cargo check", work, True)
        target_cell("编译目录：cd 进 crates/ 的子目录（相对路径）之后的 cargo check：拒", "cd crates/singlefs-verification && cargo check", main, True)
        target_cell("编译目录：经内存包装起：拒", "bash scripts/hosts/run-with-memory-cap.sh 12G cargo test -p singlefs-verification --lib", main, True)
        target_cell("编译目录：capped.sh、nice、timeout、env 包着：拒", "timeout 200 nice -n 19 env A=1 bash scripts/hosts/capped.sh 31 cargo clippy --all-targets", main, True)
        target_cell("编译目录：bash -c 里起：拒", "bash -c 'cd crates && cargo build --release'", main, True)
        target_cell("编译目录：/usr/bin/time 包着 bash -c 起：拒", "/usr/bin/time -v bash -c 'cargo doc --no-deps'", main, True)
        target_cell("编译目录：CARGO_TARGET_DIR 指到仓里：拒", f"CARGO_TARGET_DIR={main}/target-mine cargo build", main, True)
        target_cell("编译目录：--target-dir 指到仓里的相对路径：拒", "cargo build --target-dir target/mine", main, True)
        target_cell("编译目录：仓里的配置文件给的 build.target-dir 在仓里：拒", "cargo build", configured_inside, True)
        target_cell("编译目录：包装脚本外面又给了仓里的 CARGO_TARGET_DIR：拒", f"CARGO_TARGET_DIR={main}/target bash scripts/hosts/run-build-directory.sh cargo build", main, True)
        target_cell("编译目录：拒绝说明给出经 task-server 提交的出路", "SINGLEFS_RUN_SCOPE='x' bash scripts/hosts/run-with-memory-cap.sh 12G cargo test -p singlefs-verification --lib", main, True,
                    extra=lambda message: "→ 怎么办" in message and f"{TASK_SERVER_CLIENT} submit -- " in message and f"{TASK_SERVER_CLIENT} clean <号>" in message
                    and "默认编进" in message and RUN_BUILD_DIRECTORY_SCRIPT not in message)
        target_cell("编译目录：带仓外 CARGO_TARGET_DIR：放行", f"CARGO_TARGET_DIR={outside} cargo build", main, False)
        target_cell("编译目录：先赋变量再拼仓外 CARGO_TARGET_DIR：放行", f"D={work}; CARGO_TARGET_DIR=$D/outside-target cargo test -p singlefs-verification --lib", main, False)
        target_cell("编译目录：export 了仓外 CARGO_TARGET_DIR 再起：放行", f"export CARGO_TARGET_DIR={outside}; cargo test", main, False)
        target_cell("编译目录：经内存包装、env 给仓外 CARGO_TARGET_DIR：放行", f"bash scripts/hosts/run-with-memory-cap.sh 12G env CARGO_TARGET_DIR={outside} cargo test", main, False)
        target_cell("编译目录：会话环境里有仓外 CARGO_TARGET_DIR：放行", "cargo build", main, False, hook_environment={"CARGO_TARGET_DIR": outside})
        target_cell("编译目录：--target-dir 指到仓外：放行", f"cargo build --target-dir={outside}", main, False)
        target_cell("编译目录：--config build.target-dir 指到仓外：放行", f"cargo build --config 'build.target-dir=\"{outside}\"'", main, False)
        target_cell("编译目录：仓里的配置文件给的 build.target-dir 在仓外：放行", "cargo build", configured_outside, False)
        target_cell("编译目录：cargo fmt：放行", "cargo fmt --all", main, False)
        target_cell("编译目录：cargo metadata、cargo tree、cargo clean：放行", "cargo metadata --format-version 1; cargo tree; cargo clean", main, False)
        target_cell("编译目录：经包装脚本起：放行", "bash scripts/hosts/run-build-directory.sh cargo test -p singlefs-verification --lib", main, False)
        target_cell("编译目录：内存包装里套包装脚本起：放行", "bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/run-build-directory.sh cargo test", main, False)
        target_cell("编译目录：包装脚本里 bash -c 再 cd 进 crates/ 起：放行", "bash scripts/hosts/run-build-directory.sh bash -c 'cd crates && cargo build'", main, False)
        target_cell("编译目录：仓副本（不在 git 仓里）里起：放行", "cargo build", plain_copy, False)
        target_cell("编译目录：git worktree 加出来的树里起：放行", "cargo build", linked, False)
        target_cell("编译目录：CARGO_TARGET_DIR 写在算不出的变量里：放行并记一条", "CARGO_TARGET_DIR=$SOMEWHERE/target cargo build", main, False, want_notice=True)
        target_cell("编译目录：命令里只是提到 cargo 的字：放行", "echo cargo build; grep -n cargo Cargo.toml", main, False)
        # 只定义、调不到的 function 与判项目代码那一处同一个口径（三方 shell-words-r2 判决第二节第 2、8 条；uncalled-functions-judged 下第一格红，
        # lib_shell_words.py 的 function-calls-literal-only 下第二格红）
        target_cell("编译目录：只定义、调不到的 function 函数体：放行", "function noop { cargo build; }", main, False)
        target_cell("编译目录：function 定义之后拆开拼写调用：拒", 'function noop { cargo build; }; "no"op', main, True)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 workspace_cargo_target_refusal / cargo_invocations / explicit_target_directory / configured_target_directory / work_tree_root"
              f"（剥包装在 lib_project_code_runs.py 的 command_under_launcher）；{BREAK_VARIABLE} 设着的话这里本来就该红（项与红的格见文件头）")
        return 1
    print(f"  ✓ lib_workspace_cargo_lock 自检通过（查了 {checked} 格）："
          "编译目录：主仓里直接起的会编的 cargo（裸起、cd 进 crates/、经内存包装、capped.sh、nice、timeout、env、bash -c、/usr/bin/time 包着，CARGO_TARGET_DIR、--target-dir、配置文件指到仓里）拒绝并给出经 task-server 提交、用完 task clean 的出路，"
          "仓外的 CARGO_TARGET_DIR（命令前、export、env、会话环境）、--target-dir、--config、配置文件，run-build-directory.sh 包着的（连同内存包装里套它、它里面 bash -c），不编的子命令，仓副本与 git worktree 加出来的树放行，算不出的放行并记一条")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if os.environ.get(BREAK_VARIABLE):   # 弄坏开关只在自证里认：命令行不是单独一个 --selftest 而变量非空，一切之前就拒，不管值是什么
        print(f"  ✗ {BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 是弄坏开关，只在自证（lib_workspace_cargo_lock.py --selftest）里认，平常跑设了就拒，这一趟什么都没做", file=sys.stderr)
        print(f"  → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 .claude/hooks/lib_workspace_cargo_lock.py --selftest", file=sys.stderr)
        sys.exit(2)
    print("用法：python3 .claude/hooks/lib_workspace_cargo_lock.py --selftest（判定入口按文件路径导入这份模块再调，见文件头）", file=sys.stderr)
    sys.exit(2)
