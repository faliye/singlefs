"""主工作区上起 cargo 之前认合入锁：锁被别的活着的进程占着，这条命令执行前拒绝。

heavy-test-guard.sh（PreToolUse[Bash]）在 decide() 里调这一份，按文件路径用 importlib 导入；判锁用 scripts/gate/lib_workspace_lock.py 的 status /
holder_alive / is_self_or_ancestor / describe（判法不在这里另写一份，锁文件的格式与死活判法只有那一份）。

为什么：两条线往同一个主工作区合入、在上面跑重活，互相撞，编译错、两趟都白跑
（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 20 行）；
合入这一半由 scripts/gate/merge-and-verify.sh 与 scripts/gate/apply-writer-patch.py 持锁，这里是「主工作区上的 cargo 起之前认锁」那一半。

判法（workspace_cargo_lock_refusal）：
  ① 命令位置上有 cargo：包装（bash -c、nice、timeout、env、capped.sh N、run-with-memory-cap.sh、/usr/bin/time 这一类启动器、gpu-quiesce-run.sh）
     剥掉之后的命令词是 cargo，子命令不是只查版本与清单的那几个（NON_BUILDING_SUBCOMMANDS）。目录取 cd / -C / --manifest-path 之后的。
  ② 那个目录在主工作区里：所在 git 仓的工作树根，它的 .git 就是 git common-dir（主工作树）。仓副本（不在 git 仓里、或是自己的仓）、
     git worktree 加出来的树都不是；目录认不出（cd 到变量路径）的也不判。
  ③ 锁文件（SINGLEFS_WORKSPACE_LOCK_FILE 或 <common-dir>/singlefs-main-workspace.lock）里有持有者、而且它活着：拒。记录空着、持有者已死的放行。
  ④ 持有者之下的放行：SINGLEFS_WORKSPACE_LOCK_HOLDER（命令前写的，或钩子自己的环境里的）指着记录里的持有者、它是这个钩子进程自己或祖先
     （lib_workspace_lock.py 的重入判法）。
  读不了锁（建不了锁文件、读不到 lib_workspace_lock.py）不拒，交回一句 notice，由调用方记检出、照常执行。
  不读脚本正文：`bash x.sh` 里的 cargo 不在这一道的射程里（gate.sh、merge-and-verify.sh 这类自己持锁或不起 cargo 的另算）。
  不排队：钩子不等锁，被占着就拒，拒绝说明给出谁占着与下一步（等它放，或在副本里编）。

接口：
  workspace_cargo_lock_refusal(command, directory, shell_words, heavy_tests, lock_library_path, hook_environment=None) -> (拒绝说明 或 None, notices)
  build_held_workspace(work) -> (主工作区路径, 持锁的假进程, 锁文件路径)   自证与 heavy-test-guard.sh 的自检共用的现场（用完 holder.kill()）
  selftest() -> 0 / 1

弄坏开关 WORKSPACE_CARGO_LOCK_BREAK=<项>（几个用逗号连；只给自证判红用，不认识的项抛 ValueError）。项名、弄坏哪一处、打开后自证红在哪几格
（格名是 selftest 里 cell 的说明）：
  ignore-lock      不认锁，被占着也放行
      「锁被别的进程占着：拒」「拒绝说明给出谁占着与下一步」「cd 进主工作区再起：拒」「经内存包装起：拒」「bash -c 里起：拒」「cargo -C 指到主工作区：拒」
      「cargo --manifest-path 指到主工作区：拒」「HOLDER 变量指着的不是祖先：拒」「持锁者自己没写 HOLDER：拒（重入要写明）」「锁文件在默认位置（common-dir 下）：拒」；
      heavy-test-guard.sh 自检「workspace-lock：主工作区里起 cargo、锁被别的进程占着：拒（退出码 2）」及它的两格说明（主 agent 与子 agent 各一）
  ignore-reentry   不认 HOLDER 变量，持锁者之下也拒
      「持锁者自己起 cargo（命令前写 HOLDER）：放行」「持锁者自己起 cargo（钩子环境里的 HOLDER）：放行」
  any-tree         不判是不是主工作树，git worktree 加出来的树里的 cargo 也按主工作区认（不在 git 仓里的目录本来就没有锁可认，这个开关管不到）
      「git worktree 加出来的树：放行」「锁文件在默认位置，git worktree 加出来的树里起：放行」
  refuse-free      持有者已死的记录也当锁住（记录空着的本来就没有持有者，这个开关管不到）
      「持有者已死：放行」
  not-build-too    只查版本、清单的子命令也拒
      「cargo --version 与 cargo locate-project 不拒」
"""
import os
import subprocess
import sys
import tempfile
import time

BREAK_VARIABLE = "WORKSPACE_CARGO_LOCK_BREAK"
BREAK_ITEMS = ("ignore-lock", "ignore-reentry", "any-tree", "refuse-free", "not-build-too")
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


def cargo_runs_in(command, directory, shell_words, heavy_tests):
    """命令文本里命令位置上起的每一条 cargo：→ [(cargo 的子命令, 起它的目录或 None, 命令前后写的 HOLDER 变量或 "")]。"""
    runs = []
    for found in shell_words.commands_at_command_position(command, directory).commands:
        words, where = list(found.words), found.directory
        for _ in range(LAUNCHER_LAYERS):
            launched = heavy_tests.command_under_launcher(words, where) or heavy_tests.command_under_gpu_quiesce(words, where)
            if not launched or not launched[0]:
                break
            words = list(launched[0])
            if launched[1] and where is not None:
                where = shell_words.resolve_path(where, launched[1])
        if not words or os.path.basename(words[0]) != "cargo":
            continue
        cargo = heavy_tests.cargo_subcommand(words[1:], where)
        if cargo is None or (cargo.subcommand in NON_BUILDING_SUBCOMMANDS and not switch_is_set("not-build-too")):
            continue
        where = cargo.directory
        for index, argument in enumerate(cargo.rest):
            manifest = argument[len("--manifest-path="):] if argument.startswith("--manifest-path=") else (
                cargo.rest[index + 1] if argument == "--manifest-path" and index + 1 < len(cargo.rest) else None)
            if manifest is not None:
                resolved = shell_words.resolve_path(where, manifest)
                where = os.path.dirname(resolved) if resolved else None
                break
        runs.append((cargo.subcommand, where, found.environment.get(HOLDER_VARIABLE, "")))
    return runs


def main_workspace_root(directory):
    """directory 所在的 git 工作树根，仅当它是主工作树（.git 就是 git common-dir）；仓副本、git worktree 加出来的树、不在仓里的交 None。"""
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
    if switch_is_set("any-tree") or os.path.realpath(common) == os.path.realpath(os.path.join(top, ".git")):
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
    in_main_workspace = [(subcommand, main_workspace_root(where), command_holder) for subcommand, where, command_holder in runs]
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
        cell("git worktree 加出来的树：放行", "cargo build", linked, False)
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
        cell("锁文件在默认位置（common-dir 下）：拒", "cargo build", main, True, extra=names_holder)
        cell("锁文件在默认位置，git worktree 加出来的树里起：放行", "cargo build", linked, False)
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
        print(f"  → 看 workspace_cargo_lock_refusal / cargo_runs_in / main_workspace_root；{BREAK_VARIABLE} 设着的话这里本来就该红（项与红的格见文件头）")
        return 1
    print(f"  ✓ lib_workspace_cargo_lock 自检通过（查了 {checked} 格）：主工作区里起 cargo 锁被别的活进程占着拒绝并写出谁占着与下一步（cd、-C、--manifest-path、内存包装、bash -c 都认），"
          "持锁者自己（HOLDER 变量）放行，锁空着、持有者已死、仓副本与 git worktree 的树、只查版本的子命令放行，默认位置的锁文件认得")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    print("用法：python3 .claude/hooks/lib_workspace_cargo_lock.py --selftest（判定入口按文件路径导入这份模块再调，见文件头）", file=sys.stderr)
    sys.exit(2)
