#!/usr/bin/env python3
# admission: always 每一次换装置都要现编现判，判的是这一刻的草稿副本与主工作区，上一次编过不替这一次作保
# run-condition: command cargo nice
"""先编后换：草稿目录里改好的入库装置（crates/<crate>/src/bin/ 下的 .rs）先在仓副本里编过，再整份换进主工作区。

用法：
    compile-then-swap.py <草稿副本> <目标> --scratch <草稿目录> [--root 仓根] [--release]
    compile-then-swap.py --clean --scratch <草稿目录>     # 删掉这条脚本在草稿目录里建的仓副本与编译目录
    compile-then-swap.py --selftest                        # 假 cargo，不起真编译；COMPILE_THEN_SWAP_BREAK=<项> 时必须判红

<目标> 写相对仓根的路径（crates/singlefs-checker-tier/src/bin/e<号>_<英文名>.rs）或绝对路径；只收 crates/<crate>/src/bin/ 下的
<名>.rs 与 <名>/main.rs。<草稿副本> 与 <草稿目录> 都要在仓外。

做法：
  1. 把主工作区此刻的 Cargo.toml、Cargo.lock、.cargo/、crates/（不带 target/）与每台默认重活线程数的登记文件 .claude/heavy-thread-default
     （harness 包经 include_str! 在编译时读它，副本里没有就编不过）与装置启动时 preflight() 读的上游准入脚本
     .claude/singlefs-ai-sop/scripts/preflight.py、proc.py（装置把 env!("CARGO_MANIFEST_DIR") 烘进二进制，运行时按
     <树>/crates/<crate>/../../.claude/singlefs-ai-sop/scripts/preflight.py 找脚本、按 <树>/crates/<crate>/src/bin/<名>.rs 找自己的源文件）
     拷进 <草稿目录>/compile-then-swap/tree，把草稿副本放到目标在副本里的位置；主工作区一个字节不碰。
  2. 在副本里 `nice -n 19 cargo build --offline -p <crate> --bin <名>` 与 `cargo test --offline --no-run -p <crate> --bin <名>`
     （后一条编 bin 里的 #[cfg(test)]），CARGO_TARGET_DIR 放 <草稿目录>/compile-then-swap/target。线程数照环境里的
     CARGO_BUILD_JOBS（经 scripts/hosts/capped.sh N 起时就是 N）。编译输出落 <草稿目录>/compile-then-swap/build-*.log。
  3. 两条都编过，再核主工作区那份在编的这段时间里没被改过（开跑时记的字节与此刻相同，或开跑时与此刻都不存在）。
  4. 换上：目标已有的，同目录临时文件写完、fsync、照原权限位改名盖上（scripts/kb/lib_atomic_replace.py）；目标还没有的，
     同目录临时文件写完再硬链接到目标名（目标名这时被别人占了就失败）、删临时文件。换上之后回读，与草稿副本逐字节相同。
  5. 仓副本（树）与编译目录都留着，不删：烘进二进制的路径在树里，树不在了二进制起来就退 1；树里带着装置的源文件与上面那三份脚本。
     下一次编同一份草稿目录时树照原路径重建；装置产物跑完之后 --clean 一并删。
编不过、目标被改过、参数不对，都在换上之前停下：主工作区那份一个字节不动。

退出码：0 换上了；1 编不过（一个字节没写）；2 参数或路径不对（没编、一个字节没写）；3 编的时候主工作区那份被改了（一个字节没写）；
4 换上时失败或回读对不上（看提示行）。

自证（--selftest）：每一项各建一份自己的临时仓（workspace 加 crates/demo）与草稿目录，PATH 最前面放一份假 cargo
（FAKE_CARGO_SCRIPT），不起真编译。假 cargo 只认 build() 调的 `cargo build|test --offline [--no-run] -p <包> --bin <名>`，
按当前目录（应当是仓副本）里那份 bin 源码的标记演三种结局：没标记的编过；「FAKE_CARGO: build-fails」两条都编不过；
「FAKE_CARGO: test-build-fails」cargo build 编过、cargo test --no-run 编不过；找不到那个 bin 时照真 cargo 退 101。
每次被调往 COMPILE_THEN_SWAP_FAKE_CARGO_LOG 记一行「当前目录|CARGO_TARGET_DIR|参数」，每一项核它被调了哪几条、在不在仓副本里调。
PATH 上解到的 cargo 不是那一项自己的假 cargo，那一项不跑、判红。成功那句报查了几项、假 cargo 一共被调几次。
项（selftest() 里 SELFTEST_CASES 的标签）与它判的结局：
  build-fails          编不过的草稿（E0425 那一类）退 1、主工作区一个字节不写，cargo test 不再起
  test-build-fails     bin 编过而 #[cfg(test)] 编不过的也退 1、不写
  new-bin-build-fails  新 bin 编不过就不建
  changed-during-build 编的时候主工作区那份被改了，退 3、不盖
  swap-existing        编过的草稿整份换上、换的是新 inode
  create-new-bin       编过的新 bin 建出来
  swap-error           换上时目标目录写不进去，退 4、报「时失败」、目标一个字节没动
  readback-mismatch    换上之后别的进程又写了一份，回读对不上退 4
  non-bin-source       crate 里 bin 以外的源文件不收（退 2、不编）
  draft-inside-repo    草稿副本放在主工作区里不收
  outside-crates       crates/ 以外的 .rs 不收，那里有自己的 Cargo.toml 也不收
  tree-kept-until-clean  编完不 --clean 时仓副本还在草稿目录里，树里装置的源文件、.claude/singlefs-ai-sop/scripts/preflight.py、proc.py 都在
                       （装置的 preflight 读的路径）；--clean 之后树和这几个路径都不在
  clean-removes-work   --clean 删得掉编译目录
  thread-registry-copied  库源码里有 include_str 登记文件的标记（假 cargo 照它找 .claude/heavy-thread-default，找不到照真 cargo 退 101）：
                       仓副本里有那份登记文件，编过、换上
以 root 跑时目录的只读权限挡不住写，swap-error 会红。

弄坏开关（只给自证判红用，生效时往 stderr 打一行）：COMPILE_THEN_SWAP_BREAK=<一个值>。不认识的值直接拒绝，免得拼错的开关被当成「没开」。
逐项写开关名、弄坏哪一处，下一行起写打开后自证哪几项红（括号里是那条「✗ 自检 <项>：」后面的开头）：
  swap-on-failure     编不过也换上
      build-fails、test-build-fails（「退出码 0（应当 1）」）、new-bin-build-fails（「退出码 0（应当 1）」「目标 sha256」）
  skip-test-build     只跑 cargo build，不跑 cargo test --no-run（#[cfg(test)] 里编不过的放过去）
      test-build-fails（「退出码 0（应当 1）」）；changed-during-build、swap-existing、create-new-bin、swap-error、readback-mismatch、
      tree-kept-until-clean、clean-removes-work（「假 cargo 的调用 ['build']（应当 ['build', 'test']）」）
  build-in-root       cargo 在主工作区里编，不在仓副本里编（草稿副本没进编译）
      build-fails、test-build-fails（「退出码 0（应当 1）」）；凡调到假 cargo 的项（build-fails、test-build-fails、new-bin-build-fails、
      changed-during-build、swap-existing、create-new-bin、swap-error、readback-mismatch、tree-kept-until-clean、clean-removes-work）都报「假 cargo 在」
  skip-race-check     不核主工作区那份在编的时候改没改
      changed-during-build（「退出码 0（应当 3）」）
  in-place-write      不改名换上，直接在目标原来的 inode 上截断重写
      swap-existing（「换上之后 inode 没变」）、swap-error（「退出码 0（应当 4）」：只读目录挡不住在原 inode 上写）
  swap-error-ignored  换上时抛错不停下，接着往下走
      swap-error（「输出里没有「时失败」」：落到回读那一支才停）
  skip-readback       换上之后不回读
      readback-mismatch（「退出码 0（应当 4）」）
  remove-tree         编完就删仓副本（装置 preflight 要的路径跟着没了）
      tree-kept-until-clean（「编完之后仓副本不在草稿目录里」）
  preflight-not-copied  仓副本里不拷 .claude/singlefs-ai-sop/scripts/preflight.py、proc.py
      tree-kept-until-clean（「仓副本里缺 .claude/singlefs-ai-sop/scripts/preflight.py」）
  draft-in-repo       不核草稿副本在不在主工作区里
      draft-inside-repo（「退出码 0（应当 2）」）
  any-crates-source   不核目标是不是 src/bin/ 下的装置，crate 里别的 .rs 拿文件名当 bin 名
      non-bin-source（「退出码 1（应当 2）」：假 cargo 照真 cargo 找不到那个 bin、退 101）
  outside-crates      不核目标在不在 crates/ 里
      outside-crates（「退出码 1（应当 2）」：仓副本里没有 tools/，假 cargo 找不到那个 bin）
  registry-not-copied 仓副本里不拷 .claude/heavy-thread-default
      thread-registry-copied（「退出码 1（应当 0）」：假 cargo 照 include_str 标记找不到登记文件、退 101）
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
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "kb"))
from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402

BREAK_VARIABLE = "COMPILE_THEN_SWAP_BREAK"
KNOWN_BREAK_MODES = ("swap-on-failure", "skip-test-build", "build-in-root", "skip-race-check", "in-place-write", "swap-error-ignored",
                     "skip-readback", "remove-tree", "draft-in-repo", "any-crates-source", "outside-crates", "registry-not-copied",
                     "preflight-not-copied")
WORK_DIRECTORY_NAME = "compile-then-swap"
THREAD_REGISTRY_ENTRY = ".claude/heavy-thread-default"   # harness 经 include_str! 编进来的登记文件（crates/singlefs-harness/src/heavy_threads.rs）
# 装置启动时 preflight() 读的上游脚本（crates/singlefs-checker-tier/src/bin/e*.rs 的 preflight 按 <CARGO_MANIFEST_DIR>/../../ 找它；它再同目录加载 proc.py）
UPSTREAM_PREFLIGHT_ENTRIES = (".claude/singlefs-ai-sop/scripts/preflight.py", ".claude/singlefs-ai-sop/scripts/proc.py")
COPIED_ROOT_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain", "rust-toolchain.toml", "crates", THREAD_REGISTRY_ENTRY,
                       *UPSTREAM_PREFLIGHT_ENTRIES)
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


def locate_bin(target, root, mode=""):
    """目标 → (crate 目录, 包名, bin 名)；不是 crates/<crate>/src/bin/ 下的装置就抛 BadInput。"""
    crates_directory = os.path.join(root, "crates")
    check_crates = mode != "outside-crates"
    if not target.endswith(".rs") or (check_crates and not is_inside(os.path.dirname(target), crates_directory)):
        raise BadInput(f"目标 {target} 不是主工作区 crates/ 下的 .rs，这条脚本不换它\n"
                       "→ 怎么办：目标写 crates/<crate>/src/bin/<名>.rs（相对仓根）；crates/ 以外的文件照原来的办法改")
    crate_directory = os.path.dirname(target)
    while not os.path.isfile(os.path.join(crate_directory, "Cargo.toml")):
        parent = os.path.dirname(crate_directory)
        if parent == crate_directory or (check_crates and not is_inside(parent, crates_directory)):
            raise BadInput(f"目标 {target} 往上找不到 crate 的 Cargo.toml\n"
                           "→ 怎么办：目标要在 crates/<crate>/ 里面；先 ls crates/ 核 crate 名")
        crate_directory = parent
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
    elif mode == "any-crates-source":
        bin_name = os.path.splitext(parts[-1])[0]
    else:
        raise BadInput(f"目标 {target} 不是 {os.path.relpath(crate_directory, root)}/src/bin/ 下的装置（<名>.rs 或 <名>/main.rs）\n"
                       "→ 怎么办：这条脚本只换入库装置；crate 里别的源文件归 implementation-writer，写进报告交主 agent")
    if not package:
        raise BadInput(f"{crate_directory}/Cargo.toml 里没有 [package] name\n"
                       "→ 怎么办：这个目录不是一个包；先 ls crates/ 核目标路径")
    return crate_directory, package, bin_name


def copy_workspace(root, tree, mode=""):
    if os.path.lexists(tree):
        shutil.rmtree(tree)
    os.makedirs(tree)
    for entry in COPIED_ROOT_ENTRIES:
        if entry == THREAD_REGISTRY_ENTRY and mode == "registry-not-copied":
            continue
        if entry in UPSTREAM_PREFLIGHT_ENTRIES and mode == "preflight-not-copied":
            continue
        source = os.path.join(root, entry)
        if os.path.isdir(source):
            shutil.copytree(source, os.path.join(tree, entry), symlinks=True, ignore=shutil.ignore_patterns("target", ".git"))
        elif os.path.isfile(source):
            os.makedirs(os.path.dirname(os.path.join(tree, entry)), exist_ok=True)
            shutil.copy2(source, os.path.join(tree, entry))


def build(tree, target_directory, package, bin_name, release, log_path, mode=""):
    """两条 cargo 在 tree 里依次跑；交回 (都编过没有, 失败那条的错误行)。"""
    environment = dict(os.environ, CARGO_TARGET_DIR=target_directory)
    profile = ["--release"] if release else []
    commands = [["nice", "-n", "19", "cargo", "build", "--offline", "-p", package, "--bin", bin_name, *profile],
                ["nice", "-n", "19", "cargo", "test", "--offline", "--no-run", "-p", package, "--bin", bin_name, *profile]]
    if mode == "skip-test-build":
        commands = commands[:1]
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


def swap(draft, target, scratch, root, release=False, after_build=None, after_swap=None):
    """交回 (退出码, 提示行)。after_build、after_swap 只给自证用：编完、核改没改之前调一次；换上之后、回读之前调一次。"""
    mode = break_mode()
    root = os.path.realpath(root)
    draft = os.path.abspath(draft)
    target = os.path.normpath(target if os.path.isabs(target) else os.path.join(root, target))
    scratch = os.path.abspath(scratch)
    if not os.path.isfile(draft):
        raise BadInput(f"草稿副本 {draft} 不存在或不是文件\n"
                       "→ 怎么办：先把主工作区那份拷进草稿目录（cp crates/…/<名>.rs <草稿目录>/），在那一份上改，再把它的路径交给这条脚本")
    for label, path in (("草稿副本", draft), ("草稿目录", scratch)):
        if label == "草稿副本" and mode == "draft-in-repo":
            continue
        if os.path.realpath(path) == root or is_inside(path, root):
            raise BadInput(f"{label} {path} 在主工作区里\n"
                           "→ 怎么办：放进派发提示给的草稿目录（工作根 ~/.local/share/singlefs/work/<轮>/ 这类仓外位置）")
    crate_directory, package, bin_name = locate_bin(target, root, mode)
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
        copy_workspace(root, tree, mode)
        tree_target = os.path.join(tree, os.path.relpath(target, root))
        os.makedirs(os.path.dirname(tree_target), exist_ok=True)
        with open(tree_target, "wb") as handle:
            handle.write(draft_bytes)
        built, errors = build(root if mode == "build-in-root" else tree, target_directory, package, bin_name, release, log_path, mode)
    finally:
        if mode == "remove-tree":
            shutil.rmtree(tree, ignore_errors=True)
    lines.append(f"编译输出：{log_path}；编译目录 {target_directory} 与仓副本 {tree} 都留着（装置启动时的 preflight 按烘进二进制的树路径读"
                 f"里面的 preflight.py 与自己的源文件，树不在二进制退 1），装置产物跑完之后 --clean --scratch {scratch}")
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
        if mode != "swap-error-ignored":
            lines += [f"✗ 换上 {os.path.relpath(target, root)} 时失败：{error}",
                      "→ 怎么办：核主工作区那份此刻的字节（sha256sum）是不是还是开跑时那份；是就查目录权限与剩余空间后重跑，不是就交主 agent"]
            return EXIT_SWAP_FAILED, lines
    if after_swap is not None:
        after_swap()
    after = read_bytes_or_none(target)
    if after != draft_bytes and mode != "skip-readback":
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
    return [f"  ✓ 删了 {work}（仓副本、编译目录与编译输出；这之后编出来的二进制起不来，要先确认装置产物都跑完了）"]


SELFTEST_MANIFEST = '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n'
OUTSIDE_CRATES_MANIFEST = '[package]\nname = "demo2"\nversion = "0.1.0"\nedition = "2021"\n'
LIBRARY_SOURCE = "pub fn value() -> u32 {\n    1\n}\n"
GOOD_BIN = "fn main() {\n    println!(\"{}\", demo::value());\n}\n"
GOOD_BIN_CHANGED = "fn main() {\n    println!(\"{}\", demo::value() + 1);\n}\n"
BAD_BIN = "fn main() {\n    println!(\"{}\", unit_check_fields);\n}\n// FAKE_CARGO: build-fails\n"
BAD_TEST_MODULE = (GOOD_BIN + "#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        missing_helper();\n    }\n}\n"
                   "// FAKE_CARGO: test-build-fails\n")
OTHER_WRITER_TEXT = GOOD_BIN + "// 别的会话的改动\n"
REGISTRY_INCLUDE_MARKER = "// FAKE_CARGO: include_str ../../" + THREAD_REGISTRY_ENTRY + "\n"
REGISTRY_SAMPLE_TEXT = "7  # 自证样本的登记\n"
BOTH_CARGO_COMMANDS = ("build", "test")
FAKE_CARGO_LOG_VARIABLE = "COMPILE_THEN_SWAP_FAKE_CARGO_LOG"
FAKE_CARGO_SCRIPT = r"""#!/usr/bin/env bash
# 假 cargo：compile-then-swap.py --selftest 把它放在 PATH 最前面（文件头「自证」一段）。只认 build() 调的
# `cargo build|test --offline [--no-run] -p <包> --bin <名> [--release]`；按当前目录（应当是仓副本）里那份 bin 源码的
# FAKE_CARGO 标记演编过、编不过、测试编不过；每次被调往 $COMPILE_THEN_SWAP_FAKE_CARGO_LOG 记一行「当前目录|CARGO_TARGET_DIR|参数」
set -u
printf '%s|%s|%s\n' "$PWD" "${CARGO_TARGET_DIR:-}" "$*" >>"${COMPILE_THEN_SWAP_FAKE_CARGO_LOG:?假 cargo 没拿到调用记录文件}"
subcommand="${1:-}"
bin=""
previous=""
for argument in "$@"; do
  [[ "$previous" == --bin ]] && bin="$argument"
  previous="$argument"
done
case "$subcommand" in
  build) failing_marker='FAKE_CARGO: build-fails' ;;
  test) failing_marker='FAKE_CARGO: (build-fails|test-build-fails)' ;;
  *) echo "假 cargo 认不出子命令「$subcommand」：只认 build、test" >&2; exit 2 ;;
esac
if [[ ! -f Cargo.toml ]]; then
  echo "error: could not find \`Cargo.toml\` in \`$PWD\` or any parent directory"
  exit 101
fi
# 照 include_str! 读编译时的文件：crates/<包>/src/ 下源码里的「FAKE_CARGO: include_str <相对包目录的路径>」标记点名的文件找不到，照真 cargo 退 101
while IFS= read -r marker_line; do
  marker_file="${marker_line%%:*}"
  included="${marker_line##*FAKE_CARGO: include_str }"
  crate_directory="${marker_file%%/src/*}"
  if [[ ! -f "$crate_directory/$included" ]]; then
    echo "error: couldn't read \`$crate_directory/$included\`: No such file or directory (os error 2)"
    exit 101
  fi
done < <(grep -rHo 'FAKE_CARGO: include_str [^[:space:]]*' crates 2>/dev/null)
source_file=""
for candidate in crates/*/src/bin/"$bin".rs crates/*/src/bin/"$bin"/main.rs; do
  [[ -f "$candidate" ]] && source_file="$candidate"
done
if [[ -z "$source_file" ]]; then
  echo "error: no bin target named \`$bin\`"
  exit 101
fi
if grep -qE "$failing_marker" "$source_file"; then
  echo "error[E0425]: cannot find value \`unit_check_fields\` in this scope"
  echo " --> $source_file:2:20"
  echo "error: could not compile \`demo\` due to 1 previous error"
  exit 101
fi
echo "    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.01s"
exit 0
"""


def write_text(path, text):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


class SelftestFixture:
    """一项一份：临时仓（workspace 加 crates/demo，src/bin/e1_demo.rs 是 GOOD_BIN）、草稿目录、假 cargo 与它的调用记录。"""

    def __init__(self, work):
        self.root = os.path.join(work, "repo")
        self.bin_directory = os.path.join(self.root, "crates", "demo", "src", "bin")
        self.target = os.path.join(self.bin_directory, "e1_demo.rs")
        self.new_target = os.path.join(self.bin_directory, "e2_demo.rs")
        self.library = os.path.join(self.root, "crates", "demo", "src", "lib.rs")
        self.scratch = os.path.join(work, "scratch")
        self.fake_bin = os.path.join(work, "fake-bin")
        self.fake_cargo = os.path.join(self.fake_bin, "cargo")
        self.cargo_log = os.path.join(work, "fake-cargo.log")
        for directory in (self.bin_directory, self.scratch, self.fake_bin):
            os.makedirs(directory)
        for entry in UPSTREAM_PREFLIGHT_ENTRIES:
            os.makedirs(os.path.dirname(os.path.join(self.root, entry)), exist_ok=True)
            write_text(os.path.join(self.root, entry), "# 自证样本：上游准入脚本\n")
        write_text(os.path.join(self.root, "Cargo.toml"), '[workspace]\nresolver = "2"\nmembers = ["crates/demo"]\n')
        write_text(os.path.join(self.root, "crates", "demo", "Cargo.toml"), SELFTEST_MANIFEST)
        write_text(self.library, LIBRARY_SOURCE)
        write_text(self.target, GOOD_BIN)
        write_text(self.fake_cargo, FAKE_CARGO_SCRIPT)
        os.chmod(self.fake_cargo, 0o755)
        write_text(self.cargo_log, "")

    def draft(self, text, name="draft.rs"):
        path = os.path.join(self.scratch, name)
        write_text(path, text)
        return path

    def cargo_calls(self):
        """假 cargo 的调用记录：每次一个 (当前目录, CARGO_TARGET_DIR, 参数)。"""
        with open(self.cargo_log, encoding="utf-8") as handle:
            return [tuple(line.rstrip("\n").split("|", 2)) for line in handle if line.strip()]


def check_swap(fixture, draft_path, target, want_code, want_bytes, want_calls, after_build=None, after_swap=None,
               want_new_inode=False, want_line=None):
    """跑一次 swap，交回问题列表（空就是判对了）。want_calls 是假 cargo 依次该被调的子命令。"""
    inode_before = os.stat(target).st_ino if os.path.exists(target) else None
    try:
        code, lines = swap(draft_path, target, fixture.scratch, fixture.root, after_build=after_build, after_swap=after_swap)
    except BadInput as error:
        code, lines = EXIT_BAD_INPUT, str(error).splitlines()
    got_bytes = read_bytes_or_none(target)
    target_directory = os.path.dirname(target)
    leftovers = [name for name in os.listdir(target_directory) if name.startswith(".")] if os.path.isdir(target_directory) else []
    problems = []
    if code != want_code:
        problems.append(f"退出码 {code}（应当 {want_code}）")
    if got_bytes != want_bytes:
        problems.append(f"目标 sha256 {sha256_of(got_bytes)}（应当 {sha256_of(want_bytes)}）")
    if leftovers:
        problems.append(f"目标目录里留下临时文件 {leftovers}")
    if want_new_inode and inode_before is not None and got_bytes is not None and os.stat(target).st_ino == inode_before:
        problems.append("换上之后 inode 没变：在原 inode 上写的，正在读它的进程会读到半份")
    if code not in (EXIT_SWAPPED, EXIT_BAD_INPUT) and not any(line.lstrip().startswith("→") for line in lines):
        problems.append("拒绝没带「→」出路")
    if want_line is not None and not any(want_line in line for line in lines):
        problems.append(f"输出里没有「{want_line}」")
    calls = fixture.cargo_calls()
    subcommands = [arguments.split(" ", 1)[0] for _, _, arguments in calls]
    if subcommands != list(want_calls):
        problems.append(f"假 cargo 的调用 {subcommands}（应当 {list(want_calls)}）")
    work = os.path.join(fixture.scratch, WORK_DIRECTORY_NAME)
    wanted_places = (os.path.realpath(os.path.join(work, "tree")), os.path.realpath(os.path.join(work, "target")))
    misplaced = [(directory, cargo_target) for directory, cargo_target, _ in calls
                 if (os.path.realpath(directory), os.path.realpath(cargo_target)) != wanted_places]
    if misplaced:
        problems.append(f"假 cargo 在 {misplaced[0][0]}（CARGO_TARGET_DIR {misplaced[0][1]}）里被调，"
                        f"应当在仓副本 {wanted_places[0]}（{wanted_places[1]}）")
    if problems:
        problems.append(f"输出末三行 {lines[-3:]}")
    return problems


def case_build_fails(fixture):
    return check_swap(fixture, fixture.draft(BAD_BIN), fixture.target, EXIT_BUILD_FAILED, GOOD_BIN.encode(), ["build"])


def case_test_build_fails(fixture):
    return check_swap(fixture, fixture.draft(BAD_TEST_MODULE), fixture.target, EXIT_BUILD_FAILED, GOOD_BIN.encode(),
                      BOTH_CARGO_COMMANDS)


def case_new_bin_build_fails(fixture):
    return check_swap(fixture, fixture.draft(BAD_BIN), fixture.new_target, EXIT_BUILD_FAILED, None, ["build"])


def case_changed_during_build(fixture):
    def someone_edits():
        write_text(fixture.target, OTHER_WRITER_TEXT)
    return check_swap(fixture, fixture.draft(GOOD_BIN_CHANGED), fixture.target, EXIT_CHANGED_DURING_BUILD,
                      OTHER_WRITER_TEXT.encode(), BOTH_CARGO_COMMANDS, after_build=someone_edits)


def case_swap_existing(fixture):
    return check_swap(fixture, fixture.draft(GOOD_BIN_CHANGED), fixture.target, EXIT_SWAPPED, GOOD_BIN_CHANGED.encode(),
                      BOTH_CARGO_COMMANDS, want_new_inode=True)


def case_create_new_bin(fixture):
    return check_swap(fixture, fixture.draft(GOOD_BIN), fixture.new_target, EXIT_SWAPPED, GOOD_BIN.encode(), BOTH_CARGO_COMMANDS)


def case_swap_error(fixture):
    def directory_turns_read_only():
        os.chmod(fixture.bin_directory, 0o555)
    try:
        return check_swap(fixture, fixture.draft(GOOD_BIN_CHANGED), fixture.target, EXIT_SWAP_FAILED, GOOD_BIN.encode(),
                          BOTH_CARGO_COMMANDS, after_build=directory_turns_read_only, want_line="时失败")
    finally:
        os.chmod(fixture.bin_directory, 0o755)


def case_readback_mismatch(fixture):
    def another_writer():
        write_text(fixture.target, OTHER_WRITER_TEXT)
    return check_swap(fixture, fixture.draft(GOOD_BIN_CHANGED), fixture.target, EXIT_SWAP_FAILED, OTHER_WRITER_TEXT.encode(),
                      BOTH_CARGO_COMMANDS, after_swap=another_writer, want_line="与草稿副本不同")


def case_non_bin_source(fixture):
    return check_swap(fixture, fixture.draft("pub fn value() -> u32 { 2 }\n", "lib.rs"), fixture.library, EXIT_BAD_INPUT,
                      LIBRARY_SOURCE.encode(), [])


def case_draft_inside_repo(fixture):
    inside = os.path.join(fixture.root, "draft-inside.rs")
    write_text(inside, GOOD_BIN_CHANGED)
    return check_swap(fixture, inside, fixture.target, EXIT_BAD_INPUT, GOOD_BIN.encode(), [])


def case_outside_crates(fixture):
    crate = os.path.join(fixture.root, "tools", "demo2")
    os.makedirs(os.path.join(crate, "src", "bin"))
    write_text(os.path.join(crate, "Cargo.toml"), OUTSIDE_CRATES_MANIFEST)
    return check_swap(fixture, fixture.draft(GOOD_BIN), os.path.join(crate, "src", "bin", "x.rs"), EXIT_BAD_INPUT, None, [])


def case_tree_kept_until_clean(fixture):
    """编完不 --clean：树还在，装置 preflight 读的路径（装置源文件、上游 preflight.py 与 proc.py）都在；--clean 之后一个不剩。"""
    problems = check_swap(fixture, fixture.draft(GOOD_BIN_CHANGED), fixture.target, EXIT_SWAPPED, GOOD_BIN_CHANGED.encode(),
                          BOTH_CARGO_COMMANDS)
    tree = os.path.join(fixture.scratch, WORK_DIRECTORY_NAME, "tree")
    wanted = [os.path.relpath(fixture.target, fixture.root), *UPSTREAM_PREFLIGHT_ENTRIES]
    problems += [f"编完之后仓副本里缺 {entry}：装置起来读不到它会退 1" for entry in wanted if not os.path.isfile(os.path.join(tree, entry))]
    if not os.path.isdir(tree):
        problems.append("编完之后仓副本不在草稿目录里：烘进二进制的树路径没了，装置起来退 1")
    elif read_bytes_or_none(os.path.join(tree, wanted[0])) != GOOD_BIN_CHANGED.encode():
        problems.append("仓副本里装置的源文件不是换上的那一份")
    clean(fixture.scratch)
    problems += [f"--clean 之后 {entry} 还在" for entry in wanted if os.path.lexists(os.path.join(tree, entry))]
    if os.path.lexists(tree):
        problems.append("--clean 之后仓副本还在")
    return problems


def case_thread_registry_copied(fixture):
    write_text(fixture.library, LIBRARY_SOURCE + REGISTRY_INCLUDE_MARKER)
    os.makedirs(os.path.dirname(os.path.join(fixture.root, THREAD_REGISTRY_ENTRY)), exist_ok=True)
    write_text(os.path.join(fixture.root, THREAD_REGISTRY_ENTRY), REGISTRY_SAMPLE_TEXT)
    return check_swap(fixture, fixture.draft(GOOD_BIN_CHANGED), fixture.target, EXIT_SWAPPED, GOOD_BIN_CHANGED.encode(),
                      BOTH_CARGO_COMMANDS)


def case_clean_removes_work(fixture):
    problems = check_swap(fixture, fixture.draft(GOOD_BIN_CHANGED), fixture.target, EXIT_SWAPPED, GOOD_BIN_CHANGED.encode(),
                          BOTH_CARGO_COMMANDS)
    clean(fixture.scratch)
    if os.path.exists(os.path.join(fixture.scratch, WORK_DIRECTORY_NAME)):
        problems.append("--clean 之后编译目录还在")
    return problems


# 标签与文件头「自证」一段逐项对应；每一项各用一份 SelftestFixture，不读别的项留下的东西
SELFTEST_CASES = (
    ("build-fails", case_build_fails),
    ("test-build-fails", case_test_build_fails),
    ("new-bin-build-fails", case_new_bin_build_fails),
    ("changed-during-build", case_changed_during_build),
    ("swap-existing", case_swap_existing),
    ("create-new-bin", case_create_new_bin),
    ("swap-error", case_swap_error),
    ("readback-mismatch", case_readback_mismatch),
    ("non-bin-source", case_non_bin_source),
    ("draft-inside-repo", case_draft_inside_repo),
    ("outside-crates", case_outside_crates),
    ("tree-kept-until-clean", case_tree_kept_until_clean),
    ("clean-removes-work", case_clean_removes_work),
    ("thread-registry-copied", case_thread_registry_copied),
)


def run_selftest_case(case):
    """一项一份临时目录，PATH 最前面放这一项的假 cargo；跑完还原环境、删临时目录。交回 (问题列表, 假 cargo 被调了几次)。"""
    work = tempfile.mkdtemp(prefix="compile-then-swap-selftest-")
    saved = {name: os.environ.get(name) for name in ("PATH", FAKE_CARGO_LOG_VARIABLE)}
    try:
        fixture = SelftestFixture(work)
        os.environ["PATH"] = fixture.fake_bin + os.pathsep + (saved["PATH"] or "")
        os.environ[FAKE_CARGO_LOG_VARIABLE] = fixture.cargo_log
        resolved = shutil.which("cargo")
        if resolved is None or os.path.realpath(resolved) != os.path.realpath(fixture.fake_cargo):
            return [f"PATH 上解到的 cargo 是 {resolved}，不是这一项的假 cargo {fixture.fake_cargo}：这一项不跑，免得起真编译"], 0
        try:
            problems = case(fixture)
        except Exception as error:  # 一项抛了异常只记这一项红，别的项照跑
            problems = [f"抛了 {type(error).__name__}：{error}"]
        return problems, len(fixture.cargo_calls())
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        shutil.rmtree(work, ignore_errors=True)


def selftest():
    failures, cargo_calls = [], 0
    for label, case in SELFTEST_CASES:
        problems, calls = run_selftest_case(case)
        cargo_calls += calls
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}")
    for failure in failures:
        print(f"  ✗ 自检 {failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 swap() / build() / locate_bin() 的判法；{BREAK_VARIABLE} 设着的话这里本来就该红"
              "（文件头「弄坏开关」逐个写了红在哪几项）")
        return 1
    print(f"  ✓ compile-then-swap 自检通过（查了 {len(SELFTEST_CASES)} 项，假 cargo 一共被调 {cargo_calls} 次，不起真编译）："
          "编不过的（bin 本身、#[cfg(test)]、新 bin）一个字节不写，编的时候主工作区那份被改了不盖，编过的整份换上且换的是新 inode、"
          "新 bin 建得出来，换上失败与回读对不上都退 4，bin 以外的源文件、crates/ 以外的 .rs、放在主工作区里的草稿都不收，"
          "cargo 只在仓副本里调，仓副本带着 include_str 读的登记文件 .claude/heavy-thread-default，"
          "编完不 --clean 时仓副本留着、带着装置 preflight 读的源文件与 .claude/singlefs-ai-sop/scripts/preflight.py、proc.py，--clean 之后树和编译目录都删")
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
