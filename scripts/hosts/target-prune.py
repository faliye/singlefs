#!/usr/bin/env python3
# admission: always 判的是此刻编译目录里每一项的时刻与此刻在跑的 cargo / rustc，上一次的结论不替这一次作保
# run-condition: none 在不在用由脚本开跑前自己现判（判的是 --root 给的那一份编译目录，--selftest 在别的会话有 cargo 在跑时也要跑得了），不交给 preflight 预判
"""主工作区编译目录清旧产物：<仓根>/target/{debug,release}/{deps,build,incremental,.fingerprint} 下面一层里，N 天没被改过也没被读过的条目删掉。

用法：
  target-prune.py [--root <仓根>] [--config <配置>] [--dry-run]
  target-prune.py --selftest            # 临时目录里造编译目录与假 cargo 进程走一遍；TARGET_PRUNE_BREAK=<项> 时必须判红

删什么：四处（deps、build、incremental、.fingerprint，debug 与 release 各一套）下面一层的每一项。
  文件看修改时刻与访问时刻里晚的那个；目录看它自己的修改时刻，连同它底下每个文件的修改与访问时刻、每层子目录的修改时刻，有一个不早于 N 天前就留着。
  访问时刻也算：cargo 对没变的依赖不重写产物、只在编的时候读它（挂载是 relatime 时一天至少记一次），只看修改时刻会把天天在用的依赖删掉、下一次整个重编。
  目录不看访问时刻：这里自己列目录就会把它刷新，看了它就永远删不掉。
  四处之外（target/debug 下的可执行文件、examples、build 之外的 tmp、CACHEDIR.TAG、.rustc_info.json、doc……）一律不碰。
  N 在配置（默认 scripts/hosts/target-prune.conf）的 stale-days 一行，整数，至少 1。
编译目录：<仓根>/target（--root 不给是这份脚本所在的仓）；只清这一份，不认 CARGO_TARGET_DIR（副本与别处的编译目录跟着副本删）。
开跑前核在用：/proc 里进程名（comm）是 cargo、rustc、rustdoc、clippy-driver、cargo-clippy 的每个进程，
  环境里设了 CARGO_TARGET_DIR 或 CARGO_BUILD_TARGET_DIR（相对的按它的工作目录算）而指到这份编译目录或它底下；
  或两样都没设、工作目录在仓根底下、往上最近一个写了 [workspace] 的 Cargo.toml 就是仓根那一份（research/ 下自成工作区的不算）；
  或命令行里有这份编译目录底下的路径——有一个就退 78、一项都不删，逐个列出进程号、命令与哪一样指到它。读不了的进程（刚退出、别的用户的）跳过。
  78 在这里与准入的拒绝同一个意思：运行条件（没有 cargo 在用这份编译目录）不满足，一步都没做；照列出的进程等它们跑完再调。
输出：一行「删了 N 项、共 X（deps a、build b、incremental c、.fingerprint d）；没到 N 天留着 M 项；删不成 K 项（编译目录 <路径>）」，
  --dry-run 时「会删 N 项……」且一项不删。删不成的逐项列在它后面。
退出码：0 做完（含编译目录不在、没有可删的）；1 有删不成的；2 用法错、配置读不出；78 有 cargo / rustc 在用这份编译目录，一项没删。
由谁、什么时候调：主 agent 定（提交收尾或合入后验证收尾里挑一处），定了写进调用那一处；这里不自己挂到哪里。

弄坏开关 TARGET_PRUNE_BREAK=<项>（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（格名是「✗ 自证 <格名>：」那一段）：
  ignores-running    不核在用，照删
      在用-工作目录是仓根、在用-CARGO_TARGET_DIR、在用-命令行
  prunes-everything  不看时刻，四处底下全删
      旧的删新的留
  mtime-only         文件只看修改时刻，不看访问时刻
      最近读过的留着
  top-level-only     目录只看它自己的修改时刻，不看底下
      底下有新文件的目录留着
"""
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import os  # noqa: E402
import re  # noqa: E402
import shutil  # noqa: E402
import subprocess  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402
import time  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(SCRIPT_DIRECTORY)
DEFAULT_CONFIG = os.path.join(SCRIPT_DIRECTORY, "target-prune.conf")
PROFILES = ("debug", "release")
PRUNED_KINDS = ("deps", "build", "incremental", ".fingerprint")
CARGO_PROCESS_NAMES = ("cargo", "rustc", "rustdoc", "clippy-driver", "cargo-clippy")
TARGET_VARIABLES = (b"CARGO_TARGET_DIR=", b"CARGO_BUILD_TARGET_DIR=")
EXIT_IN_USE = 78
BREAK_VARIABLE = "TARGET_PRUNE_BREAK"
BREAK_SWITCHES = ("ignores-running", "prunes-everything", "mtime-only", "top-level-only")
SECONDS_PER_DAY = 86400


def broken(name):
    return os.environ.get(BREAK_VARIABLE, "") == name


def format_size(byte_count):
    for unit, size in (("GiB", 1 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)):
        if byte_count >= size:
            return f"{byte_count / size:.1f} {unit}"
    return f"{byte_count} B"


class ConfigProblem(Exception):
    pass


def read_stale_days(path):
    """配置里 stale-days=<整数> 那一行（# 起头与空行不算；别的 key 拒绝）。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except OSError as error:
        raise ConfigProblem(f"读不了配置 {path}：{error}")
    found = None
    for line in lines:
        text = line.strip()
        if not text or text.startswith("#"):
            continue
        key, separator, value = (part.strip() for part in text.partition("="))
        if not separator or key != "stale-days":
            raise ConfigProblem(f"配置 {path} 里认不出这一行：{line}（只认 stale-days=<整数>）")
        if not re.fullmatch(r"[0-9]+", value) or int(value) < 1:
            raise ConfigProblem(f"配置 {path} 里 stale-days 的值「{value}」不是不小于 1 的整数")
        found = int(value)
    if found is None:
        raise ConfigProblem(f"配置 {path} 里没有 stale-days=<整数> 那一行")
    return found


def inside(path, directory):
    return path == directory or path.startswith(directory.rstrip(os.sep) + os.sep)


def path_mentioned_in(text, path):
    return re.search(re.escape(path) + r"(?![A-Za-z0-9_.-])", text) is not None


def workspace_root_of(directory, root):
    """directory（在 root 底下）往上最近一个写了 [workspace] 的 Cargo.toml 所在的目录；到 root 为止都没有就是 root。"""
    current = directory
    while inside(current, root) and current != root:
        manifest = os.path.join(current, "Cargo.toml")
        try:
            with open(manifest, encoding="utf-8", errors="replace") as handle:
                if any(line.strip().startswith("[workspace]") for line in handle):
                    return current
        except OSError:
            pass
        current = os.path.dirname(current)
    return root


def processes_using(target, root):
    """在用这份编译目录的 cargo / rustc 进程：[(pid, 命令, 哪一样指到它)]。"""
    target_forms = {os.path.normpath(target), os.path.realpath(target)}
    root_forms = {os.path.normpath(root), os.path.realpath(root)}
    users = []
    for entry in os.listdir("/proc"):
        if not entry.isdigit() or int(entry) == os.getpid():
            continue
        base = os.path.join("/proc", entry)
        try:
            with open(os.path.join(base, "comm"), encoding="utf-8", errors="replace") as handle:
                name = handle.read().strip()
        except OSError:
            continue
        if name not in CARGO_PROCESS_NAMES:
            continue
        try:
            with open(os.path.join(base, "cmdline"), "rb") as handle:
                arguments = [word.decode(errors="replace") for word in handle.read().split(b"\0") if word]
            working_directory = os.readlink(os.path.join(base, "cwd"))
            with open(os.path.join(base, "environ"), "rb") as handle:
                environment_items = handle.read().split(b"\0")
        except OSError:
            continue
        target_variable = None
        for item in environment_items:
            for prefix in TARGET_VARIABLES:
                if item.startswith(prefix) and item[len(prefix):]:
                    target_variable = item[len(prefix):].decode(errors="replace")
        reasons = []
        if target_variable is not None:
            resolved = os.path.normpath(os.path.join(working_directory, target_variable))
            if any(inside(resolved, form) for form in target_forms):
                reasons.append(f"CARGO_TARGET_DIR={target_variable}")
        else:
            for form in root_forms:
                if inside(working_directory, form) and workspace_root_of(working_directory, form) == form:
                    reasons.append(f"工作目录 {working_directory} 在仓根底下、没设 CARGO_TARGET_DIR")
                    break
        if any(path_mentioned_in(argument, form) for argument in arguments for form in target_forms):
            reasons.append("命令行里有这份编译目录底下的路径")
        if reasons:
            users.append((int(entry), " ".join(arguments)[:160], "；".join(reasons)))
    return users


def recent(path, cutoff):
    """path（文件或目录）有没有不早于 cutoff 的时刻（文件头「删什么」）：有回 True。"""
    try:
        status = os.lstat(path)
    except FileNotFoundError:
        return False
    is_directory = os.path.isdir(path) and not os.path.islink(path)
    if is_directory:
        if status.st_mtime >= cutoff:
            return True
        if broken("top-level-only"):
            return False
        for current_directory, subdirectory_names, file_names in os.walk(path):
            for name in subdirectory_names:
                try:
                    if os.lstat(os.path.join(current_directory, name)).st_mtime >= cutoff:
                        return True
                except OSError:
                    continue
            for name in file_names:
                try:
                    child = os.lstat(os.path.join(current_directory, name))
                except OSError:
                    continue
                if child.st_mtime >= cutoff or (child.st_atime >= cutoff and not broken("mtime-only")):
                    return True
        return False
    return status.st_mtime >= cutoff or (status.st_atime >= cutoff and not broken("mtime-only"))


def allocated_bytes(path):
    try:
        status = os.lstat(path)
    except OSError:
        return 0
    total = status.st_blocks * 512
    if os.path.isdir(path) and not os.path.islink(path):
        for current_directory, subdirectory_names, file_names in os.walk(path):
            for name in [*subdirectory_names, *file_names]:
                try:
                    total += os.lstat(os.path.join(current_directory, name)).st_blocks * 512
                except OSError:
                    continue
    return total


def prune(root, stale_days, dry_run, say):
    """回退出码。"""
    target = os.path.join(root, "target")
    if not os.path.isdir(target):
        say(f"  ✓ 编译目录 {target} 不在，没有可删的（查了 0 项）")
        return 0
    users = [] if broken("ignores-running") else processes_using(target, root)
    if users:
        say(f"  ✗ 有 {len(users)} 个 cargo / rustc 进程在用编译目录 {target}，一项都没删：")
        for pid, command, reason in users:
            say(f"      进程 {pid}（{command}）：{reason}")   # gate-lint:detail
        say("  → 怎么办：等它们跑完再调（按进程号等：python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>）；"
            "不是自己起的别去停它")
        return EXIT_IN_USE
    cutoff = time.time() - stale_days * SECONDS_PER_DAY
    removed = {kind: [0, 0] for kind in PRUNED_KINDS}
    kept, failed = 0, []
    for profile in PROFILES:
        for kind in PRUNED_KINDS:
            directory = os.path.join(target, profile, kind)
            try:
                names = sorted(os.listdir(directory))
            except FileNotFoundError:
                continue
            for name in names:
                path = os.path.join(directory, name)
                if not broken("prunes-everything") and recent(path, cutoff):
                    kept += 1
                    continue
                size = allocated_bytes(path)
                if not dry_run:
                    try:
                        if os.path.isdir(path) and not os.path.islink(path):
                            shutil.rmtree(path)
                        else:
                            os.unlink(path)
                    except OSError as error:
                        failed.append((path, str(error)))
                        continue
                removed[kind][0] += 1
                removed[kind][1] += size
    count = sum(entry[0] for entry in removed.values())
    total = sum(entry[1] for entry in removed.values())
    verb = "会删" if dry_run else "删了"
    say(f"  ✓ {verb} {count} 项、共 {format_size(total)}（{'、'.join(f'{kind} {removed[kind][0]}' for kind in PRUNED_KINDS)}）；"
        f"没到 {stale_days} 天留着 {kept} 项；删不成 {len(failed)} 项（编译目录 {target}{'；--dry-run，一项没删' if dry_run else ''}）")
    for path, reason in failed:
        say(f"      删不成 {path}：{reason}")   # gate-lint:detail
    if failed:
        say("  → 怎么办：看上面那几项为什么删不成（权限、被占着），修好再调一次；删了的不会重删")
        return 1
    return 0


# ── 自证 ──

class Selftest:
    def __init__(self):
        self.cells = 0
        self.failures = []

    def check(self, label, condition, detail=""):
        self.cells += 1
        if not condition:
            self.failures.append(f"{label}：{detail}")


def write_file(path, text="x" * 4096):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def build_fake_tree(root, now):
    """一棵假仓：仓根 [workspace]、research/side 自成工作区；target 里旧的、新的、旧而最近读过的、目录旧而底下有新文件的，四处之外的旧条目。
    回 (应当删掉的相对路径, 应当留着的相对路径)。"""
    old = now - 10 * SECONDS_PER_DAY
    write_file(os.path.join(root, "Cargo.toml"), '[workspace]\nmembers = ["crates/demo"]\n')
    write_file(os.path.join(root, "research", "side", "Cargo.toml"), '[workspace]\nmembers = []\n')
    target = os.path.join(root, "target")
    gone, kept = [], []

    def file_at(relative, modified, accessed=None):
        path = os.path.join(target, relative)
        write_file(path)
        os.utime(path, (accessed if accessed is not None else modified, modified))

    def directory_at(relative, modified):
        os.utime(os.path.join(target, relative), (modified, modified))

    file_at("debug/deps/libold-aaaa.rlib", old); gone.append("debug/deps/libold-aaaa.rlib")
    file_at("debug/deps/libnew-bbbb.rlib", now); kept.append("debug/deps/libnew-bbbb.rlib")
    file_at("debug/deps/libread-cccc.rlib", old, accessed=now); kept.append("debug/deps/libread-cccc.rlib")
    file_at("debug/build/oldpkg-1111/out/gen.rs", old); directory_at("debug/build/oldpkg-1111/out", old); directory_at("debug/build/oldpkg-1111", old)
    gone.append("debug/build/oldpkg-1111")
    file_at("debug/build/nested-2222/out/gen.rs", now); directory_at("debug/build/nested-2222/out", old); directory_at("debug/build/nested-2222", old)
    kept.append("debug/build/nested-2222")
    file_at("debug/.fingerprint/oldpkg-3333/lib-oldpkg", old); directory_at("debug/.fingerprint/oldpkg-3333", old)
    gone.append("debug/.fingerprint/oldpkg-3333")
    file_at("debug/incremental/oldpkg-4444/s-1/query-cache.bin", old); directory_at("debug/incremental/oldpkg-4444/s-1", old)
    directory_at("debug/incremental/oldpkg-4444", old); gone.append("debug/incremental/oldpkg-4444")
    file_at("release/deps/libold-5555.rlib", old); gone.append("release/deps/libold-5555.rlib")
    file_at("debug/singlefs-old-binary", old); kept.append("debug/singlefs-old-binary")
    file_at("debug/examples/old-example", old); kept.append("debug/examples/old-example")
    return gone, kept


def tree_state(root, relatives):
    return {relative: os.path.lexists(os.path.join(root, "target", relative)) for relative in relatives}


def run_prune(root, stale_days=2, dry_run=False):
    lines = []
    exit_code = prune(root, stale_days, dry_run, lines.append)
    return exit_code, "\n".join(lines)


def start_fake_cargo(bin_directory, working_directory, environment):
    """名字（comm）是 cargo 的假进程：指向 sleep 的符号链接，只睡、不编。"""
    fake = os.path.join(bin_directory, "cargo")
    if not os.path.lexists(fake):
        os.makedirs(bin_directory, exist_ok=True)
        os.symlink(shutil.which("sleep") or "/bin/sleep", fake)
    return subprocess.Popen([fake, "120"], cwd=working_directory, env=environment, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def wait_until_named(process, name, timeout_seconds=10):
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        try:
            with open(f"/proc/{process.pid}/comm", encoding="utf-8") as handle:
                if handle.read().strip() == name:
                    return True
        except OSError:
            pass
        time.sleep(0.05)
    return False


def run_selftest():
    selftest = Selftest()
    work = tempfile.mkdtemp(prefix="target-prune-selftest-")
    plain_environment = {key: value for key, value in os.environ.items() if key not in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR")}
    started = []
    try:
        now = time.time()
        # 一：旧的删、新的留（文件看修改与访问里晚的，目录连同底下看），四处之外不碰
        root = os.path.join(work, "plain")
        gone, kept = build_fake_tree(root, now)
        exit_code, output = run_prune(root)
        state = tree_state(root, gone + kept)
        selftest.check("旧的删新的留", exit_code == 0 and not any(state[relative] for relative in gone)
                       and all(state[relative] for relative in kept if relative not in ("debug/deps/libread-cccc.rlib", "debug/build/nested-2222"))
                       and "删了 5 项" in output and "deps 2、build 1、incremental 1、.fingerprint 1" in output,
                       f"退 {exit_code}；还在的 {sorted(relative for relative, present in state.items() if present)}；{output}")
        selftest.check("最近读过的留着", state["debug/deps/libread-cccc.rlib"],
                       f"修改时刻 10 天前、访问时刻此刻的 debug/deps/libread-cccc.rlib 被删了；{output}")
        selftest.check("底下有新文件的目录留着", state["debug/build/nested-2222"],
                       f"自己的修改时刻 10 天前、底下 out/gen.rs 此刻改过的 debug/build/nested-2222 被删了；{output}")
        # --dry-run：一项不删
        root = os.path.join(work, "dry")
        gone, kept = build_fake_tree(root, now)
        exit_code, output = run_prune(root, dry_run=True)
        selftest.check("--dry-run 一项不删", exit_code == 0 and all(tree_state(root, gone + kept).values()) and "会删 5 项" in output,
                       f"退 {exit_code}；{output}")
        bin_directory = os.path.join(work, "bin")
        # 二：有 cargo 在用（工作目录是仓根、没设 CARGO_TARGET_DIR）：退 78、一项不删
        for label, working_directory_of, environment_of in (
                ("在用-工作目录是仓根", lambda root: root, lambda root: plain_environment),
                ("在用-CARGO_TARGET_DIR", lambda root: work, lambda root: dict(plain_environment, CARGO_TARGET_DIR=os.path.join(root, "target")))):
            root = os.path.join(work, label)
            gone, kept = build_fake_tree(root, now)
            fake = start_fake_cargo(bin_directory, working_directory_of(root), environment_of(root))
            started.append(fake)
            named = wait_until_named(fake, "cargo")
            exit_code, output = run_prune(root)
            fake.kill()
            fake.wait()
            selftest.check(label, named and exit_code == EXIT_IN_USE and all(tree_state(root, gone + kept).values()) and f"进程 {fake.pid}" in output,
                           f"假 cargo {fake.pid} 起{'来了' if named else '不来'}；退 {exit_code}（要 {EXIT_IN_USE}）；{output}")
        # 命令行里有这份编译目录底下的路径（工作目录与变量都不指到它）
        root = os.path.join(work, "在用-命令行")
        gone, kept = build_fake_tree(root, now)
        # 假 rustc 要收得下 --out-dir 这类参数（sleep 不收），写成直接以解释器为 #! 的 python 脚本：进程名（comm）是脚本的文件名 rustc，只睡、不编
        fake_rustc = os.path.join(bin_directory, "rustc")
        write_file(fake_rustc, f"#!{sys.executable}\nimport time\ntime.sleep(120)\n")
        os.chmod(fake_rustc, 0o755)
        fake = subprocess.Popen([fake_rustc, "--out-dir", os.path.join(root, "target", "debug", "deps")], cwd=work,
                                env=dict(plain_environment, CARGO_TARGET_DIR=os.path.join(work, "elsewhere")), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        started.append(fake)
        named = wait_until_named(fake, "rustc")
        exit_code, output = run_prune(root)
        fake.kill()
        fake.wait()
        selftest.check("在用-命令行", named and exit_code == EXIT_IN_USE and all(tree_state(root, gone + kept).values()),
                       f"假 rustc {fake.pid} 起{'来了' if named else '不来'}；退 {exit_code}（要 {EXIT_IN_USE}）；{output}")
        # 三：cargo 在跑而不用这一份（CARGO_TARGET_DIR 指到别处、工作目录在自成工作区的 research/side 底下）：照删
        root = os.path.join(work, "not-in-use")
        gone, kept = build_fake_tree(root, now)
        others = [start_fake_cargo(bin_directory, root, dict(plain_environment, CARGO_TARGET_DIR=os.path.join(work, "elsewhere"))),
                  start_fake_cargo(bin_directory, os.path.join(root, "research", "side"), plain_environment)]
        started.extend(others)
        named = all(wait_until_named(process, "cargo") for process in others)
        exit_code, output = run_prune(root)
        for process in others:
            process.kill()
            process.wait()
        selftest.check("别处在用的不算", named and exit_code == 0 and not any(tree_state(root, gone).values()),
                       f"退 {exit_code}；{output}")
        # 配置：认 stale-days、拒认不出的 key 与不是整数的值
        config = os.path.join(work, "prune.conf")
        write_file(config, "# 注释\n\nstale-days=3\n")
        good = read_stale_days(config) == 3
        bad = 0
        for text in ("stale-day=3\n", "stale-days=0\n", "stale-days=two\n", "# 只有注释\n"):
            write_file(config, text)
            try:
                read_stale_days(config)
            except ConfigProblem:
                bad += 1
        selftest.check("配置", good and bad == 4, f"认对 stale-days=3：{good}；四种写坏的拒了 {bad} 种")
        repository_days = None
        try:
            repository_days = read_stale_days(DEFAULT_CONFIG)
        except ConfigProblem as problem:
            repository_days = str(problem)
        selftest.check("仓里的配置读得出", isinstance(repository_days, int), f"{DEFAULT_CONFIG}：{repository_days}")
    finally:
        for process in started:
            if process.poll() is None:
                process.kill()
                process.wait()
        shutil.rmtree(work, ignore_errors=True)
    if selftest.failures:
        print(f"  ✗ target-prune.py 自证有 {len(selftest.failures)} 格判错（共 {selftest.cells} 格）：")
        for failure in selftest.failures:
            print(f"      ✗ 自证 {failure}")   # gate-lint:detail
        print("  → 怎么办：设了 TARGET_PRUNE_BREAK 的，这就是要的结果；没设的，照上面那几格看 prune、recent、processes_using 与 read_stale_days")
        return 1
    print(f"  ✓ target-prune.py 自证 {selftest.cells} 格都对（旧的删新的留、最近读过的留着、底下有新文件的目录留着、四处之外不碰、--dry-run 不删、"
          "有 cargo 在用时退 78 一项不删（工作目录是仓根、CARGO_TARGET_DIR、命令行三种）、别处在用的不算、配置写坏就拒）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        switch = os.environ.get(BREAK_VARIABLE, "")
        if switch and switch not in BREAK_SWITCHES:
            print(f"  ✗ {BREAK_VARIABLE}={switch} 认不出\n  → 怎么办：只认 {'、'.join(BREAK_SWITCHES)}")
            return 2
        return run_selftest()
    if os.environ.get(BREAK_VARIABLE):
        print(f"  ✗ {BREAK_VARIABLE} 只给 --selftest 用，这一次一项没删\n  → 怎么办：去掉它再跑（env -u {BREAK_VARIABLE} …）")
        return 2
    root, config, dry_run = REPOSITORY_ROOT, DEFAULT_CONFIG, False
    rest = list(arguments)
    while rest:
        argument = rest.pop(0)
        if argument in ("--root", "--config") and rest:
            value = rest.pop(0)
            if argument == "--root":
                root = os.path.abspath(value)
            else:
                config = value
        elif argument == "--dry-run":
            dry_run = True
        else:
            print(f"  ✗ 认不出参数 {argument}\n  → 怎么办：target-prune.py [--root <仓根>] [--config <配置>] [--dry-run]，或 --selftest")
            return 2
    try:
        stale_days = read_stale_days(config)
    except ConfigProblem as problem:
        print(f"  ✗ {problem}\n  → 怎么办：照 scripts/hosts/target-prune.conf 的写法改成 stale-days=<不小于 1 的整数>")
        return 2
    return prune(root, stale_days, dry_run, print)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
