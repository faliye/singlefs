#!/usr/bin/env python3
# admission: always 每一次换装置都要现编现判，判的是这一刻的草稿副本与主工作区，上一次编过不替这一次作保
# run-condition: command cargo nice
"""先编后换：草稿目录里改好的入库装置（crates/<crate>/src/bin/ 下的 .rs）先在仓副本里编过，再整份换进主工作区。

用法：
    compile-then-swap.py <草稿副本> <目标> --scratch <草稿目录> [--root 仓根] [--release]
    compile-then-swap.py --clean --scratch <草稿目录>     # 删掉这条脚本在草稿目录里建的仓副本与编译目录
    compile-then-swap.py --selftest                        # COMPILE_THEN_SWAP_BREAK=<项> 时必须判红

<目标> 写相对仓根的路径（crates/singlefs-checker-tier/src/bin/e<号>_<英文名>.rs）或绝对路径；只收 crates/<crate>/src/bin/ 下的
<名>.rs 与 <名>/main.rs。<草稿副本> 与 <草稿目录> 都要在仓外。

做法：
  1. 把主工作区此刻的 Cargo.toml、Cargo.lock、.cargo/ 与 crates/（不带 target/）拷进 <草稿目录>/compile-then-swap/tree，
     把草稿副本放到目标在副本里的位置；主工作区一个字节不碰。
  2. 在副本里 `nice -n 19 cargo build --offline -p <crate> --bin <名>` 与 `cargo test --offline --no-run -p <crate> --bin <名>`
     （后一条编 bin 里的 #[cfg(test)]），CARGO_TARGET_DIR 放 <草稿目录>/compile-then-swap/target。线程数照环境里的
     CARGO_BUILD_JOBS（经 research/scripts/capped.sh N 起时就是 N）。编译输出落 <草稿目录>/compile-then-swap/build-*.log。
  3. 两条都编过，再核主工作区那份在编的这段时间里没被改过（开跑时记的字节与此刻相同，或开跑时与此刻都不存在）。
  4. 换上：目标已有的，同目录临时文件写完、fsync、照原权限位改名盖上（research/scripts/lib_atomic_replace.py）；目标还没有的，
     同目录临时文件写完再硬链接到目标名（目标名这时被别人占了就失败）、删临时文件。换上之后回读，与草稿副本逐字节相同。
  5. 删掉仓副本，编译目录留着给下一次增量编；收工前 --clean。
编不过、目标被改过、参数不对，都在换上之前停下：主工作区那份一个字节不动。

退出码：0 换上了；1 编不过（一个字节没写）；2 参数或路径不对（没编、一个字节没写）；3 编的时候主工作区那份被改了（一个字节没写）；
4 换上时失败或回读对不上（看提示行）。

弄坏开关（只给自证判红用，生效时往 stderr 打一行）：COMPILE_THEN_SWAP_BREAK=
  swap-on-failure   编不过也换上
  skip-race-check   不核主工作区那份在编的时候改没改
  in-place-write    不改名换上，直接在目标原来的 inode 上截断重写
不认识的值直接拒绝，免得拼错的开关被当成「没开」。
"""
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402

BREAK_VARIABLE = "COMPILE_THEN_SWAP_BREAK"
KNOWN_BREAK_MODES = ("swap-on-failure", "skip-race-check", "in-place-write")
WORK_DIRECTORY_NAME = "compile-then-swap"
COPIED_ROOT_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain", "rust-toolchain.toml", "crates")
ERROR_LINES_SHOWN = 40
EXIT_SWAPPED, EXIT_BUILD_FAILED, EXIT_BAD_INPUT, EXIT_CHANGED_DURING_BUILD, EXIT_SWAP_FAILED = 0, 1, 2, 3, 4


class BadInput(Exception):
    """参数或路径不对：str 是两行，第二行以「→」开头写下一步。"""


def break_mode():
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in KNOWN_BREAK_MODES:
        raise BadInput(f"{BREAK_VARIABLE}={mode} 不认识，没编、一个字节没写\n"
                       f"→ 怎么办：弄坏开关只认 {'、'.join(KNOWN_BREAK_MODES)}；平时不设这个变量")
    if mode:
        print(f"  ! {BREAK_VARIABLE}={mode} 生效：故意走坏的写法，只给自证判红用", file=sys.stderr)
    return mode


def is_inside(path, directory):
    return os.path.realpath(path).startswith(os.path.realpath(directory) + os.sep)


def read_bytes_or_none(path):
    try:
        with open(path, "rb") as handle:
            return handle.read()
    except FileNotFoundError:
        return None


def sha256_of(data):
    return "不存在" if data is None else hashlib.sha256(data).hexdigest()


def locate_bin(target, root):
    """目标 → (crate 目录, 包名, bin 名)；不是 crates/<crate>/src/bin/ 下的装置就抛 BadInput。"""
    crates_directory = os.path.join(root, "crates")
    if not target.endswith(".rs") or not is_inside(os.path.dirname(target), crates_directory):
        raise BadInput(f"目标 {target} 不是主工作区 crates/ 下的 .rs，这条脚本不换它\n"
                       "→ 怎么办：目标写 crates/<crate>/src/bin/<名>.rs（相对仓根）；crates/ 以外的文件照原来的办法改")
    crate_directory = os.path.dirname(target)
    while not os.path.isfile(os.path.join(crate_directory, "Cargo.toml")):
        crate_directory = os.path.dirname(crate_directory)
        if not is_inside(crate_directory, crates_directory):
            raise BadInput(f"目标 {target} 往上找不到 crate 的 Cargo.toml\n"
                           "→ 怎么办：目标要在 crates/<crate>/ 里面；先 ls crates/ 核 crate 名")
    try:
        with open(os.path.join(crate_directory, "Cargo.toml"), "rb") as handle:
            manifest = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise BadInput(f"读不了 {crate_directory}/Cargo.toml（{error}）\n"
                       "→ 怎么办：主工作区的 Cargo.toml 此刻坏着，交主 agent；修好之前不换") from error
    package = (manifest.get("package") or {}).get("name")
    relative = os.path.relpath(target, crate_directory)
    declared = [entry.get("name") for entry in manifest.get("bin") or [] if os.path.normpath(entry.get("path", "")) == relative]
    parts = relative.split(os.sep)
    if declared and declared[0]:
        bin_name = declared[0]
    elif len(parts) == 3 and parts[:2] == ["src", "bin"]:
        bin_name = parts[2][:-len(".rs")]
    elif len(parts) == 4 and parts[:2] == ["src", "bin"] and parts[3] == "main.rs":
        bin_name = parts[2]
    else:
        raise BadInput(f"目标 {target} 不是 {os.path.relpath(crate_directory, root)}/src/bin/ 下的装置（<名>.rs 或 <名>/main.rs）\n"
                       "→ 怎么办：这条脚本只换入库装置；crate 里别的源文件归 implementation-writer，写进报告交主 agent")
    if not package:
        raise BadInput(f"{crate_directory}/Cargo.toml 里没有 [package] name\n"
                       "→ 怎么办：这个目录不是一个包；先 ls crates/ 核目标路径")
    return crate_directory, package, bin_name


def copy_workspace(root, tree):
    if os.path.lexists(tree):
        shutil.rmtree(tree)
    os.makedirs(tree)
    for entry in COPIED_ROOT_ENTRIES:
        source = os.path.join(root, entry)
        if os.path.isdir(source):
            shutil.copytree(source, os.path.join(tree, entry), symlinks=True, ignore=shutil.ignore_patterns("target", ".git"))
        elif os.path.isfile(source):
            shutil.copy2(source, os.path.join(tree, entry))


def build(tree, target_directory, package, bin_name, release, log_path):
    """两条 cargo 依次跑；交回 (都编过没有, 失败那条的错误行)。"""
    environment = dict(os.environ, CARGO_TARGET_DIR=target_directory)
    profile = ["--release"] if release else []
    commands = [["nice", "-n", "19", "cargo", "build", "--offline", "-p", package, "--bin", bin_name, *profile],
                ["nice", "-n", "19", "cargo", "test", "--offline", "--no-run", "-p", package, "--bin", bin_name, *profile]]
    with open(log_path, "a", encoding="utf-8") as log:
        for command in commands:
            log.write(f"$ {' '.join(command)}\n")
            log.flush()
            completed = subprocess.run(command, cwd=tree, env=environment, stdin=subprocess.DEVNULL,
                                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, errors="replace")
            log.write(completed.stdout)
            log.write(f"退出码 {completed.returncode}\n")
            if completed.returncode != 0:
                lines = completed.stdout.splitlines()
                shown = [line for index, line in enumerate(lines)
                         if line.startswith("error") or (line.lstrip().startswith("-->") and index > 0)]
                return False, [f"$ {' '.join(command[3:])}（退出码 {completed.returncode}）", *shown[:ERROR_LINES_SHOWN]]
    return True, []


def create_new_file(target, data):
    """目标还不存在：同目录临时文件写完、fsync，再硬链接到目标名（被占了就失败），删临时文件。"""
    directory = os.path.dirname(target)
    descriptor, temporary = tempfile.mkstemp(prefix="." + os.path.basename(target) + ".", suffix=".swapping", dir=directory)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fchmod(handle.fileno(), 0o644)
            os.fsync(handle.fileno())
        os.link(temporary, target)
    finally:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def swap(draft, target, scratch, root, release=False, after_build=None):
    """交回 (退出码, 提示行)。after_build 只给自证用：编完、核改没改之前调一次。"""
    mode = break_mode()
    root = os.path.realpath(root)
    draft = os.path.abspath(draft)
    target = os.path.normpath(target if os.path.isabs(target) else os.path.join(root, target))
    scratch = os.path.abspath(scratch)
    if not os.path.isfile(draft):
        raise BadInput(f"草稿副本 {draft} 不存在或不是文件\n"
                       "→ 怎么办：先把主工作区那份拷进草稿目录（cp crates/…/<名>.rs <草稿目录>/），在那一份上改，再把它的路径交给这条脚本")
    for label, path in (("草稿副本", draft), ("草稿目录", scratch)):
        if os.path.realpath(path) == root or is_inside(path, root):
            raise BadInput(f"{label} {path} 在主工作区里\n"
                           "→ 怎么办：放进派发提示给的草稿目录（/tmp/claude-1000/<轮>/ 这类仓外位置）")
    crate_directory, package, bin_name = locate_bin(target, root)
    try:
        draft_bytes = open(draft, "rb").read()
        draft_bytes.decode("utf-8")
    except UnicodeDecodeError as error:
        raise BadInput(f"草稿副本 {draft} 不是 UTF-8 文本（{error}）\n"
                       "→ 怎么办：Rust 源文件必须是 UTF-8；核一下给的是不是那份 .rs") from error
    before = read_bytes_or_none(target)
    work = os.path.join(scratch, WORK_DIRECTORY_NAME)
    tree = os.path.join(work, "tree")
    target_directory = os.path.join(work, "target")
    os.makedirs(work, exist_ok=True)
    log_path = os.path.join(work, f"build-{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-{os.getpid()}.log")
    lines = [f"目标 {os.path.relpath(target, root)}（包 {package}，bin {bin_name}）；草稿副本 {draft}",
             f"主工作区那份开跑时 sha256 {sha256_of(before)}；草稿副本 sha256 {sha256_of(draft_bytes)}"]
    try:
        copy_workspace(root, tree)
        tree_target = os.path.join(tree, os.path.relpath(target, root))
        os.makedirs(os.path.dirname(tree_target), exist_ok=True)
        with open(tree_target, "wb") as handle:
            handle.write(draft_bytes)
        built, errors = build(tree, target_directory, package, bin_name, release, log_path)
    finally:
        shutil.rmtree(tree, ignore_errors=True)
    lines.append(f"编译输出：{log_path}；编译目录 {target_directory} 留着给下一次增量编，收工前 --clean --scratch {scratch}")
    if not built:
        lines += [f"✗ 草稿副本在仓副本里编不过，主工作区 {os.path.relpath(target, root)} 一个字节没写：", *errors,  # gate-lint:summary
                  f"→ 怎么办：照上面的错误改草稿副本 {draft}，再跑这条脚本；错误落在草稿副本以外的文件（别的会话在改的），"
                  "把错误行写进报告交主 agent，不自己去改那些文件"]
        if mode != "swap-on-failure":
            return EXIT_BUILD_FAILED, lines
    if after_build is not None:
        after_build()
    now = read_bytes_or_none(target)
    if now != before and mode != "skip-race-check":
        lines += [f"✗ 编的这段时间里主工作区 {os.path.relpath(target, root)} 被改了（开跑时 sha256 {sha256_of(before)}，此刻 {sha256_of(now)}），"
                  "一个字节没写",
                  "→ 怎么办：先看是谁改的（git diff 那一份、问主 agent）；把那一版的改动并进草稿副本之后再跑这条脚本，不盖掉别人的改动"]
        return EXIT_CHANGED_DURING_BUILD, lines
    try:
        if mode == "in-place-write":
            with open(target, "wb") as handle:
                handle.write(draft_bytes)
        elif now is None:
            create_new_file(target, draft_bytes)
        else:
            def unchanged_since_check():
                if read_bytes_or_none(target) != now:
                    raise ReplaceRefused(f"{target} 在换上前一刻又被改了\n→ 怎么办：重跑这条脚本")
            lines += replace_file_contents_by_rename(target, draft_bytes.decode("utf-8"), check_before_rename=unchanged_since_check)
    except (OSError, ReplaceRefused) as error:
        lines += [f"✗ 换上 {os.path.relpath(target, root)} 时失败：{error}",
                  "→ 怎么办：核主工作区那份此刻的字节（sha256sum）是不是还是开跑时那份；是就查目录权限与剩余空间后重跑，不是就交主 agent"]
        return EXIT_SWAP_FAILED, lines
    after = read_bytes_or_none(target)
    if after != draft_bytes:
        lines += [f"✗ 换上之后回读 {os.path.relpath(target, root)}，sha256 {sha256_of(after)} 与草稿副本不同",
                  "→ 怎么办：有别的进程同时在写这一份；把现状写进报告交主 agent，不再重跑"]
        return EXIT_SWAP_FAILED, lines
    lines.append(f"✓ 编过（cargo build 与 cargo test --no-run，-p {package} --bin {bin_name}），已整份换进主工作区 "
                 f"{os.path.relpath(target, root)}，回读 sha256 {sha256_of(after)}")
    return EXIT_SWAPPED, lines


def clean(scratch):
    work = os.path.join(os.path.abspath(scratch), WORK_DIRECTORY_NAME)
    if not os.path.isdir(work):
        return [f"  ! {work} 不存在，没有要删的"]
    shutil.rmtree(work)
    return [f"  ✓ 删了 {work}（仓副本、编译目录与编译输出）"]


SELFTEST_MANIFEST = '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n'
GOOD_BIN = "fn main() {\n    println!(\"{}\", demo::value());\n}\n"
GOOD_BIN_CHANGED = "fn main() {\n    println!(\"{}\", demo::value() + 1);\n}\n"
BAD_BIN = "fn main() {\n    println!(\"{}\", unit_check_fields);\n}\n"
BAD_TEST_MODULE = GOOD_BIN + "#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        missing_helper();\n    }\n}\n"


def selftest():
    work = tempfile.mkdtemp(prefix="compile-then-swap-selftest-")
    failures, checked = [], 0
    try:
        root = os.path.join(work, "repo")
        bin_directory = os.path.join(root, "crates", "demo", "src", "bin")
        os.makedirs(bin_directory)
        with open(os.path.join(root, "Cargo.toml"), "w") as handle:
            handle.write('[workspace]\nresolver = "2"\nmembers = ["crates/demo"]\n')
        with open(os.path.join(root, "crates", "demo", "Cargo.toml"), "w") as handle:
            handle.write(SELFTEST_MANIFEST)
        with open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w") as handle:
            handle.write("pub fn value() -> u32 {\n    1\n}\n")
        target = os.path.join(bin_directory, "e1_demo.rs")
        with open(target, "w") as handle:
            handle.write(GOOD_BIN)
        scratch = os.path.join(work, "scratch")
        os.makedirs(scratch)

        def draft(name, text):
            path = os.path.join(scratch, name)
            with open(path, "w") as handle:
                handle.write(text)
            return path

        def run(label, draft_path, target_path, want_code, want_bytes, after_build=None, want_new_inode=False):
            nonlocal checked
            checked += 1
            inode_before = os.stat(target_path).st_ino if os.path.exists(target_path) else None
            try:
                code, lines = swap(draft_path, target_path, scratch, root, after_build=after_build)
            except BadInput as error:
                code, lines = EXIT_BAD_INPUT, str(error).splitlines()
            got_bytes = read_bytes_or_none(target_path)
            leftovers = [name for name in os.listdir(os.path.dirname(target_path)) if name.startswith(".")] \
                if os.path.isdir(os.path.dirname(target_path)) else []
            problems = []
            if code != want_code:
                problems.append(f"退出码 {code}（应当 {want_code}）")
            if got_bytes != want_bytes:
                problems.append(f"目标 sha256 {sha256_of(got_bytes)}（应当 {sha256_of(want_bytes)}）")
            if leftovers:
                problems.append(f"目标目录里留下临时文件 {leftovers}")
            if want_new_inode and inode_before is not None and got_bytes is not None and os.stat(target_path).st_ino == inode_before:
                problems.append("换上之后 inode 没变：在原 inode 上写的，正在读它的进程会读到半份")
            if code not in (EXIT_SWAPPED, EXIT_BAD_INPUT) and not any(line.lstrip().startswith("→") for line in lines):
                problems.append("拒绝没带「→」出路")
            if problems:
                failures.append(f"{label}：{'；'.join(problems)}；输出 {lines[-3:]}")

        good = GOOD_BIN.encode()
        run("编不过的草稿（E0425 那一类）不写主工作区", draft("bad.rs", BAD_BIN), target, EXIT_BUILD_FAILED, good)
        run("bin 编过而 #[cfg(test)] 编不过的也不写", draft("bad-test.rs", BAD_TEST_MODULE), target, EXIT_BUILD_FAILED, good)
        new_target = os.path.join(bin_directory, "e2_demo.rs")
        run("新 bin 编不过就不建", draft("bad-new.rs", BAD_BIN), new_target, EXIT_BUILD_FAILED, None)

        def someone_edits():
            with open(target, "w") as handle:
                handle.write(GOOD_BIN + "// 别的会话的改动\n")
        run("编的时候主工作区那份被改了就不盖", draft("good-race.rs", GOOD_BIN_CHANGED), target, EXIT_CHANGED_DURING_BUILD,
            (GOOD_BIN + "// 别的会话的改动\n").encode(), after_build=someone_edits)
        with open(target, "w") as handle:
            handle.write(GOOD_BIN)
        run("编过的草稿整份换上、换的是新 inode", draft("good.rs", GOOD_BIN_CHANGED), target, EXIT_SWAPPED, GOOD_BIN_CHANGED.encode(),
            want_new_inode=True)
        run("编过的新 bin 建出来", draft("good-new.rs", GOOD_BIN), new_target, EXIT_SWAPPED, good)
        library = os.path.join(root, "crates", "demo", "src", "lib.rs")
        run("crate 里 bin 以外的源文件不收", draft("lib.rs", "pub fn value() -> u32 { 2 }\n"), library, EXIT_BAD_INPUT,
            b"pub fn value() -> u32 {\n    1\n}\n")
        inside = os.path.join(root, "draft-inside.rs")
        with open(inside, "w") as handle:
            handle.write(GOOD_BIN)
        run("草稿副本放在主工作区里不收", inside, target, EXIT_BAD_INPUT, GOOD_BIN_CHANGED.encode())
        outside_crates = os.path.join(root, "tools", "x.rs")
        run("crates/ 以外的 .rs 不收", draft("x.rs", GOOD_BIN), outside_crates, EXIT_BAD_INPUT, None)
        checked += 1
        if os.path.exists(os.path.join(scratch, WORK_DIRECTORY_NAME, "tree")):
            failures.append("跑完之后仓副本还在草稿目录里：交回闸会拦")
        checked += 1
        clean(scratch)
        if os.path.exists(os.path.join(scratch, WORK_DIRECTORY_NAME)):
            failures.append("--clean 之后编译目录还在")
    except BadInput as error:
        failures.append(f"自检现场本身被拒：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 swap() / build() / locate_bin() 的判法；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ compile-then-swap 自检通过（查了 {checked} 种）：编不过的（bin 本身、#[cfg(test)]、新 bin）一个字节不写，"
          "编的时候主工作区那份被改了不盖，编过的整份换上且换的是新 inode、新 bin 建得出来，bin 以外的源文件、crates/ 以外的 .rs、"
          "放在主工作区里的草稿都不收，仓副本跑完就删、--clean 删得掉编译目录")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    usage = ("compile-then-swap.py <草稿副本> <目标> --scratch <草稿目录> [--root 仓根] [--release]，"
             "或 compile-then-swap.py --clean --scratch <草稿目录>")
    options = {"--scratch": None, "--root": REPOSITORY_ROOT}
    flags = {"--release": False, "--clean": False}
    positional = []
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in options and position + 1 < len(arguments):
            options[argument] = arguments[position + 1]
            position += 2
            continue
        if argument in flags:
            flags[argument] = True
        else:
            positional.append(argument)
        position += 1
    if options["--scratch"] is None:
        print("✗ 没给 --scratch <草稿目录>")
        print(f"→ 怎么办：{usage}")
        return EXIT_BAD_INPUT
    if flags["--clean"]:
        print("\n".join(clean(options["--scratch"])))
        return 0
    if len(positional) != 2:
        print(f"✗ 要给草稿副本与目标两个路径，给了 {len(positional)} 个")
        print(f"→ 怎么办：{usage}")
        return EXIT_BAD_INPUT
    try:
        code, lines = swap(positional[0], positional[1], options["--scratch"], options["--root"], release=flags["--release"])
    except BadInput as error:
        problem, _, next_step = str(error).partition("\n")
        print(f"✗ {problem}")
        print(f"→ {next_step.lstrip('→ ') or usage}")
        return EXIT_BAD_INPUT
    print("\n".join(lines))
    return code


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
