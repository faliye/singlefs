#!/usr/bin/env python3
# admission: always 判的是此刻这棵树的源文件与编译目录里的源码戳，上一次的结论不替这一次作保
# run-condition: command python3 rsync
"""换了源码却没重编的拦法：复用编译目录之前，把内容或 mtime 与上一次留下的样子不一样的源文件 touch 到此刻，cargo 下一次就重编它们。

为什么：cargo 按 mtime 判本地包新不新鲜（源文件比上一次编的 dep-info 新才重编）。往一份已经编过的副本树里 rsync -a 铺新源码，
新内容带着源树里的旧 mtime，比编译目录里的 dep-info 还旧，cargo 认它已编：2026-10-06 GPU 与 CPU 不一致那一次，
CPU 一侧用的是旧 walk.rs 编出来的库（records/2026-09-30-里程碑三开工第一轮.md「GPU 与 CPU 不一致调查员交回」那一行）。

用法：
  source-stamp.py refresh <树> [--target-dir <编译目录>] [--include <相对树根的路径>]…
      编之前调。源文件与编译目录里的源码戳（<编译目录>/.singlefs-source-stamp.tsv）逐个比：内容摘要或 mtime 与戳里记的不一样、
      或戳里没有这个文件的，touch 到此刻；然后把此刻每个源文件的 mtime 与摘要写回戳（先写临时文件再改名）。
      编译目录不在、或里面除了戳什么都没有（还没编过）：不 touch，只写戳。编译目录里有东西而没有戳（头一次用、或戳被删了）：
      判不了哪些编过，源文件全部 touch（头一次在一个大编译目录上用，会重编一遍工作区里的本地包，依赖不重编）。
      mtime 也比：touch 过、编到一半失败（产物还是旧的），之后又被 rsync -a 改回旧 mtime 的，照样再 touch。
  source-stamp.py sync <源树> <副本> [--target-dir <编译目录>] [--include …]…
      手工拷树的入口：rsync -a --no-times --checksum（不带 --delete；排除 .git、__pycache__，与任意一层名叫 target、target-* 的目录：
      仓根 .gitignore 的 target/ 管的就是任意一层，scripts/experiment-apparatus/target 这类编译目录动辄几十 G，与 refresh 不进的目录同一批）把源树拷进副本：
      按内容比，内容变了的才拷、拷过去的 mtime 是此刻，没变的不动；再对副本做一次 refresh（拷之前被别人 rsync -a 过的也一并补上）。
      手敲 rsync -a 之后跑 refresh 同样拦得住，只是 rsync -a 把 touch 过的文件改回旧 mtime，下一次 refresh 会再 touch 它们、多重编一次。
  source-stamp.py --selftest
  source-stamp.py --write-selftest-cargo <路径>   只给自证（这一份与 scripts/gate/merge-and-verify.sh 的）：写一份照 mtime 判新不新鲜的假 cargo
编译目录：--target-dir 给了用它；不给用 CARGO_TARGET_DIR（相对路径按树根算）；都没有用 <树>/target。
源文件：--include 给了就是点名的那几处（文件或目录），不给是树根的 Cargo.toml、Cargo.lock、.cargo/ 与 crates/ 里存在的那几处；
  目录里的每个普通文件都算，不进名叫 .git、target、target-*、__pycache__ 的目录。
输出：一行 SOURCE_STAMP touched=<touch 了几个> files=<源文件几个> reason=<first-use|changed|fresh-target|unchanged> stamp=<戳的路径>；touch 的逐个列在它前面。
退出码：0 刷新完；1 刷新不成（树不在、读写不了、rsync 退非 0），没 touch 完的不算数；2 用法错。

弄坏开关 SOURCE_STAMP_BREAK=<项>（只给 --selftest 证红用，逗号连几个；在 --selftest 之外设了就拒、退 2）。项名、弄坏哪一处、打开后红在哪几格
  （格名是「✗ 自证 <格名>：」里的那一段；末尾那一轮照 SWITCH_EXPECTED_CELLS 逐个核）：
  no-touch            refresh 只写戳、一个都不 touch          refresh-rebuilds、first-use-touches-all、failed-build-then-resync
                      （sync-rebuilds 不红：sync 拷过去的文件 mtime 本来就是此刻）
  content-only        只比内容摘要，不比 mtime                 failed-build-then-resync
  trust-missing-stamp 编译目录有东西而没有戳时一个都不 touch    first-use-touches-all
  sync-plain-rsync    sync 退回 rsync -a（带源树的旧 mtime）、拷完不做 refresh   sync-rebuilds
  touch-everything    每次都把源文件全部 touch                 unchanged-not-touched、fresh-target、sync-rebuilds
  sync-copies-build-directories  sync 照改前只排除树根那一层的 target、target-*，下面几层的编译目录照拷   sync-skips-build-directories
另有一格 stale-reproduced 不经这份脚本、只 rsync -a 再编，核的是假 cargo 真会被旧 mtime 骗（复现那一次的样子）：它红说明样本不成立。
"""
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import concurrent.futures  # noqa: E402
import hashlib  # noqa: E402
import os  # noqa: E402
import shutil  # noqa: E402
import subprocess  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402

STAMP_NAME = ".singlefs-source-stamp.tsv"
DEFAULT_INCLUDES = ("Cargo.toml", "Cargo.lock", ".cargo", "crates")
SKIPPED_DIRECTORY_NAMES = (".git", "__pycache__")
STAMP_HEADER = [
    "# 源码戳：scripts/hosts/source-stamp.py refresh 上一次编之前看到的每个源文件（touch 之后的样子），别手改。",
    "# 列（制表符分隔）：mtime（纳秒）、sha256、相对树根的路径。",
]
EXIT_REFUSED = 1
EXIT_USAGE = 2
BREAK_VARIABLE = "SOURCE_STAMP_BREAK"


def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


def skipped_directory(name):
    return name in SKIPPED_DIRECTORY_NAMES or name == "target" or name.startswith("target-")


def source_files(tree, includes):
    """树里的源文件（相对路径，排好序）：includes 里每一处，目录就往下找普通文件。"""
    found = []
    for include in includes:
        path = os.path.join(tree, include)
        if os.path.isfile(path) and not os.path.islink(path):
            found.append(os.path.normpath(include))
        elif os.path.isdir(path):
            for directory, subdirectories, files in os.walk(path):
                subdirectories[:] = sorted(name for name in subdirectories if not skipped_directory(name))
                for name in files:
                    full = os.path.join(directory, name)
                    if os.path.isfile(full) and not os.path.islink(full):
                        found.append(os.path.relpath(full, tree))
    return sorted(set(found))


def digest(path):
    hasher = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            hasher.update(block)
    return hasher.hexdigest()


def read_stamp(path):
    """相对路径 → (mtime 纳秒, sha256)；戳不在交 None。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except FileNotFoundError:
        return None
    rows = {}
    for line in lines:
        fields = line.split("\t")
        if line.startswith("#") or len(fields) != 3 or not fields[0].isdigit():
            continue
        rows[fields[2]] = (int(fields[0]), fields[1])
    return rows


def write_stamp(path, rows):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    temporary = f"{path}.{os.getpid()}.partial"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write("\n".join(STAMP_HEADER + [f"{mtime}\t{sha}\t{relative}" for relative, (mtime, sha) in sorted(rows.items())]) + "\n")
    os.replace(temporary, path)


def target_directory(tree, explicit, environment):
    if explicit:
        return os.path.abspath(explicit)
    configured = environment.get("CARGO_TARGET_DIR")
    if configured:
        return configured if os.path.isabs(configured) else os.path.join(tree, configured)
    return os.path.join(tree, "target")


def target_has_artifacts(target):
    try:
        return any(name != STAMP_NAME and not name.startswith(STAMP_NAME + ".") for name in os.listdir(target))
    except FileNotFoundError:
        return False


def refresh(tree, target, includes, say):
    """文件头 refresh：交 touch 了几个。"""
    if not os.path.isdir(tree):
        raise OSError(f"树 {tree} 不在")
    stamp_path = os.path.join(target, STAMP_NAME)
    stamp = read_stamp(stamp_path)
    files = source_files(tree, includes)
    has_artifacts = target_has_artifacts(target)
    if stamp is None and has_artifacts:
        reason = "first-use"
    elif not has_artifacts:
        reason = "fresh-target"
    else:
        reason = "changed"
    touched = 0
    rows = {}
    for relative in files:
        path = os.path.join(tree, relative)
        sha = digest(path)
        mtime = os.stat(path).st_mtime_ns
        if reason == "fresh-target":
            touch = False
        elif reason == "first-use":
            touch = not broken("trust-missing-stamp")
        else:
            recorded = stamp.get(relative)
            touch = recorded is None or recorded[1] != sha or (recorded[0] != mtime and not broken("content-only"))
        if broken("touch-everything"):
            touch = True
        if touch and not broken("no-touch"):
            os.utime(path)
            mtime = os.stat(path).st_mtime_ns
            touched += 1
            say(f"  · touch {relative}")
        rows[relative] = (mtime, sha)
    write_stamp(stamp_path, rows)
    if reason == "changed" and touched == 0:
        reason = "unchanged"
    say(f"SOURCE_STAMP touched={touched} files={len(files)} reason={reason} stamp={stamp_path}")
    return touched


def sync(source, copy, target, includes, say):
    """文件头 sync：rsync -a 拷进副本，再 refresh。"""
    source, copy = os.path.realpath(source), os.path.realpath(copy)
    if os.path.commonpath((source, copy)) in (source, copy):
        raise OSError("源树与副本不能相同或互相包含；请选择源树之外的独立副本目录")
    if not os.path.isdir(source):
        raise OSError(f"源树 {source} 不在")
    os.makedirs(copy, exist_ok=True)
    # --no-times --checksum：按内容比，只拷内容变了的，拷过去的 mtime 是此刻（不带源树的旧 mtime）；没变的不动、mtime 照旧，cargo 不白重编
    times = ["-a"] if broken("sync-plain-rsync") else ["-a", "--no-times", "--checksum"]
    # 任意一层名叫 target、target-* 的目录都不拷（文件头 sync）；弄坏开关 sync-copies-build-directories 下照改前只排除树根那一层
    build_directories = [] if broken("sync-copies-build-directories") else ["--exclude", "target/", "--exclude", "target-*/"]
    command = ["rsync", *times, "--exclude", ".git", "--exclude", "/target", "--exclude", "/target-*", *build_directories, "--exclude", "__pycache__",
               source.rstrip("/") + "/", copy.rstrip("/") + "/"]
    completed = subprocess.run(command, capture_output=True, text=True, stdin=subprocess.DEVNULL)
    if completed.returncode != 0:
        raise OSError(f"rsync 退 {completed.returncode}：{completed.stderr.strip()[-300:]}")
    say(f"  · 拷好：{source} → {copy}（{' '.join(times)}，不带 --delete）")
    if broken("sync-plain-rsync"):
        return 0
    return refresh(copy, target, includes, say)


# ── 自证 ──
FAKE_CARGO = r'''#!/usr/bin/env python3
# 自证用的假 cargo：照 cargo 认本地包新不新鲜的办法编 crates/ 下的 .rs——源文件的 mtime 比上一次编的 dep-info 新、或产物里没有它，才重编；
# 「产物」记每个文件编进去时的内容摘要。test 另核产物与此刻的源码是不是同一份：不是就报那一条 FAILED、退 101。fmt、clippy 退 0。
# FAKE_CARGO_FAIL_BUILD 设了就编不过（产物与 dep-info 都不动）、退 101。
import hashlib, json, os, sys
subcommand = sys.argv[1] if len(sys.argv) > 1 else ""
if subcommand in ("fmt", "clippy"):
    sys.exit(0)
target = os.environ.get("CARGO_TARGET_DIR") or "target"
depinfo = os.path.join(target, "debug", "fake.d")
artifact_path = os.path.join(target, "debug", "fake-artifact.json")
if os.environ.get("FAKE_CARGO_FAIL_BUILD"):
    print("error: 假 cargo 编不过")
    sys.exit(101)
os.makedirs(os.path.dirname(depinfo), exist_ok=True)
built_at = os.stat(depinfo).st_mtime_ns if os.path.exists(depinfo) else -1
artifact = json.load(open(artifact_path)) if os.path.exists(artifact_path) else {}
sources = sorted(os.path.join(directory, name) for directory, _, names in os.walk("crates") for name in names if name.endswith(".rs"))
def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()
for path in sources:
    if os.stat(path).st_mtime_ns > built_at or path not in artifact:
        artifact[path] = sha(path)
        print(f"   Compiling {path}")
    else:
        print(f"       Fresh {path}")
with open(artifact_path, "w") as handle:
    json.dump(artifact, handle)
with open(depinfo, "a"):
    pass
os.utime(depinfo)
if subcommand == "test":
    stale = [path for path in sources if artifact.get(path) != sha(path)]
    print(f"running {len(sources)} tests")
    for path in sources:
        print(f"test current::{path.replace('/', '_')} ... {'FAILED' if path in stale else 'ok'}")
    print(f"test result: {'FAILED' if stale else 'ok'}. {len(sources) - len(stale)} passed; {len(stale)} failed")
    sys.exit(101 if stale else 0)
'''
OLD_MTIME_SECONDS = 1577836800     # 2020-01-01：源树里的文件都比编译目录里的 dep-info 旧（rsync -a 原样带过去的那种旧 mtime）


def write_fake_cargo(path):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(FAKE_CARGO)
    os.chmod(path, 0o755)


def selftest_cells(root):
    """各格：交 (失败的格名列表, 查了几格)。每格自己一份源树、副本与编译目录。"""
    failures = []
    checked = 0
    fake_bin = os.path.join(root, "bin")
    os.makedirs(fake_bin)
    write_fake_cargo(os.path.join(fake_bin, "cargo"))

    def check(cell, condition, detail):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append(cell)
            print(f"  ✗ 自证 {cell}：{detail}")   # gate-lint:detail

    def write(tree, relative, text, mtime_seconds):
        path = os.path.join(tree, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.utime(path, (mtime_seconds, mtime_seconds))

    def cargo(tree, *extra, failing=False):
        environment = {name: value for name, value in os.environ.items() if name not in ("CARGO_TARGET_DIR", "FAKE_CARGO_FAIL_BUILD")}
        environment["PATH"] = fake_bin + os.pathsep + environment.get("PATH", "")
        if failing:
            environment["FAKE_CARGO_FAIL_BUILD"] = "1"
        completed = subprocess.run(["cargo", "test", *extra], cwd=tree, capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL)
        return completed.returncode, completed.stdout

    def scene(name):
        """源树 source（walk.rs 是新内容、mtime 旧）与副本 copy（walk.rs 是旧内容、已经编过一次）。"""
        base = os.path.join(root, name)
        source, copy = os.path.join(base, "source"), os.path.join(base, "copy")
        write(copy, "Cargo.toml", "[workspace]\n", OLD_MTIME_SECONDS)
        write(copy, "crates/demo/src/lib.rs", "pub mod walk;\n", OLD_MTIME_SECONDS)
        write(copy, "crates/demo/src/walk.rs", "pub fn walk() -> u32 { 1 }\n", OLD_MTIME_SECONDS)
        code, _ = cargo(copy)
        if code != 0:
            raise RuntimeError(f"假 cargo 头一次编副本就退 {code}")
        write(source, "Cargo.toml", "[workspace]\n", OLD_MTIME_SECONDS)
        write(source, "crates/demo/src/lib.rs", "pub mod walk;\n", OLD_MTIME_SECONDS)
        write(source, "crates/demo/src/walk.rs", "pub fn walk() -> u32 { 22 }\n", OLD_MTIME_SECONDS + 100)
        return source, copy

    def rsync(source, copy):
        subprocess.run(["rsync", "-a", "--exclude", "/target", source + "/", copy + "/"], check=True, stdin=subprocess.DEVNULL)

    quiet = lambda line: None   # noqa: E731
    walk = "crates_demo_src_walk.rs"

    # stale-reproduced：只 rsync -a 再编 ⇒ 假 cargo 认 walk.rs 已编（Fresh），测试报产物不是此刻的源码（复现那一次）
    source, copy = scene("stale-reproduced")
    rsync(source, copy)
    code, output = cargo(copy)
    check("stale-reproduced", code == 101 and "Fresh crates/demo/src/walk.rs" in output and f"test current::{walk} ... FAILED" in output,
          f"旧 mtime 新内容应当骗过假 cargo（Fresh、FAILED），实际退 {code}：{output.strip()[-300:]}；样本不成立，别的格判不了")

    # refresh-rebuilds：rsync -a 之后 refresh 一次再编 ⇒ walk.rs 被 touch、重编、测试过
    source, copy = scene("refresh-rebuilds")
    refresh(copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, quiet)   # 编过之后头一次：没有戳、编译目录里有东西，全 touch
    cargo(copy)
    rsync(source, copy)
    lines = []
    touched = refresh(copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, lines.append)
    code, output = cargo(copy)
    check("refresh-rebuilds", code == 0 and "Compiling crates/demo/src/walk.rs" in output and "  · touch crates/demo/src/walk.rs" in lines,
          f"rsync 之后 refresh 应当 touch walk.rs、假 cargo 重编它、测试过，实际 touch {touched} 个（{lines}），假 cargo 退 {code}：{output.strip()[-300:]}")

    # sync-rebuilds：用 sync 入口拷树 ⇒ 同上
    source, copy = scene("sync-rebuilds")
    refresh(copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, quiet)
    cargo(copy)
    sync(source, copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, quiet)
    code, output = cargo(copy)
    check("sync-rebuilds", code == 0 and "Compiling crates/demo/src/walk.rs" in output and "Fresh crates/demo/src/lib.rs" in output,
          f"sync 之后假 cargo 应当重编 walk.rs、不重编 lib.rs、测试过，实际退 {code}：{output.strip()[-300:]}")

    # sync-skips-build-directories：源树里树根下面几层有编译目录（scripts/experiment-apparatus/target 这类，主工作区那一份几十 G）⇒ sync 之后副本里没有它们，别的源文件照拷
    base = os.path.join(root, "sync-skips-build-directories")
    source, copy = os.path.join(base, "source"), os.path.join(base, "copy")
    write(source, "Cargo.toml", "[workspace]\n", OLD_MTIME_SECONDS)
    write(source, "crates/demo/src/lib.rs", "pub fn f() {}\n", OLD_MTIME_SECONDS)
    write(source, "scripts/tool.py", "print(1)\n", OLD_MTIME_SECONDS)
    build_directories = ["scripts/experiment-apparatus/target", "scripts/experiment-apparatus/target-gpu", "crates/demo/target"]
    for directory in build_directories:
        write(source, f"{directory}/release/artifact.bin", "x" * (1 << 20), OLD_MTIME_SECONDS)
    sync(source, copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, quiet)
    copied_build_directories = [directory for directory in build_directories if os.path.exists(os.path.join(copy, directory))]
    missing_sources = [relative for relative in ("Cargo.toml", "crates/demo/src/lib.rs", "scripts/tool.py") if not os.path.isfile(os.path.join(copy, relative))]
    check("sync-skips-build-directories", not copied_build_directories and not missing_sources,
          f"sync 之后副本里不该有编译目录、源文件都该在，实际拷过去的编译目录 {copied_build_directories or '无'}，没拷过去的源文件 {missing_sources or '无'}")

    # first-use-touches-all：副本编过、编译目录里没有戳（头一次用）⇒ 判不了哪些编过，全 touch，编出来的是此刻的源码
    source, copy = scene("first-use-touches-all")
    rsync(source, copy)
    lines = []
    touched = refresh(copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, lines.append)
    code, output = cargo(copy)
    check("first-use-touches-all", code == 0 and touched == 3 and "reason=first-use" in lines[-1],
          f"没有戳而编译目录里有东西应当 3 个源文件全 touch、测试过，实际 touch {touched} 个（{lines[-1:]}），假 cargo 退 {code}：{output.strip()[-200:]}")

    # failed-build-then-resync：refresh touch 了 walk.rs、那一趟编不过（产物还是旧的），又被 rsync -a 改回旧 mtime ⇒ 戳里内容对得上、mtime 对不上，照样再 touch
    source, copy = scene("failed-build-then-resync")
    refresh(copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, quiet)
    cargo(copy)
    rsync(source, copy)
    refresh(copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, quiet)
    cargo(copy, failing=True)
    rsync(source, copy)
    touched = refresh(copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, quiet)
    code, output = cargo(copy)
    check("failed-build-then-resync", code == 0 and touched >= 1 and "Compiling crates/demo/src/walk.rs" in output,
          f"编不过之后又被改回旧 mtime 的 walk.rs 应当再 touch、重编，实际 touch {touched} 个，假 cargo 退 {code}：{output.strip()[-300:]}")

    # unchanged-not-touched：编过、戳是新的、什么都没改 ⇒ 一个都不 touch，假 cargo 全 Fresh
    source, copy = scene("unchanged-not-touched")
    refresh(copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, quiet)
    cargo(copy)
    lines = []
    touched = refresh(copy, os.path.join(copy, "target"), DEFAULT_INCLUDES, lines.append)
    code, output = cargo(copy)
    check("unchanged-not-touched", code == 0 and touched == 0 and "reason=unchanged" in lines[-1] and "Compiling" not in output,
          f"什么都没改应当一个都不 touch、假 cargo 全 Fresh，实际 touch {touched} 个（{lines[-1:]}），假 cargo：{output.strip()[-200:]}")

    # fresh-target：编译目录还不在 ⇒ 不 touch，只写戳
    base = os.path.join(root, "fresh-target")
    write(base, "crates/demo/src/lib.rs", "pub fn f() {}\n", OLD_MTIME_SECONDS)
    lines = []
    touched = refresh(base, os.path.join(base, "target"), DEFAULT_INCLUDES, lines.append)
    check("fresh-target", touched == 0 and "reason=fresh-target" in lines[-1] and os.path.isfile(os.path.join(base, "target", STAMP_NAME))
          and os.stat(os.path.join(base, "crates/demo/src/lib.rs")).st_mtime == OLD_MTIME_SECONDS,
          f"编译目录不在应当不 touch、只写戳，实际 touch {touched} 个（{lines[-1:]}）")
    return failures, checked


SWITCH_EXPECTED_CELLS = {   # 文件头「弄坏开关」那一段，两处要一致
    "no-touch": ["refresh-rebuilds", "first-use-touches-all", "failed-build-then-resync"],
    "content-only": ["failed-build-then-resync"],
    "trust-missing-stamp": ["first-use-touches-all"],
    "sync-plain-rsync": ["sync-rebuilds"],
    "touch-everything": ["unchanged-not-touched", "fresh-target", "sync-rebuilds"],
    "sync-copies-build-directories": ["sync-skips-build-directories"],
}


def run_cells_in_scratch():
    root = tempfile.mkdtemp(prefix="source-stamp-selftest-")
    try:
        return selftest_cells(root)
    finally:
        shutil.rmtree(root, ignore_errors=True)


def selftest():
    failures, checked = run_cells_in_scratch()
    if failures:
        print(f"  ✗ source-stamp.py 自证 {len(failures)} 格不对（查了 {checked} 格）：{'、'.join(failures)}")
        print("  → 怎么办：照上面逐格改 refresh / sync；stale-reproduced 红是假 cargo 没被旧 mtime 骗到，先修 FAKE_CARGO；SOURCE_STAMP_BREAK 设着的话本来就该红")
        return 1
    jobs = max(1, min(len(SWITCH_EXPECTED_CELLS), int(os.environ.get("SINGLEFS_THREAD_CAP") or 0) or os.cpu_count() or 1))

    def one(switch):
        completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--selftest-cells"], capture_output=True, text=True,
                                   env=dict(os.environ, **{BREAK_VARIABLE: switch}), timeout=600, stdin=subprocess.DEVNULL)
        return switch, set(line.split("：", 1)[0].removeprefix("  ✗ 自证 ") for line in completed.stdout.splitlines() if line.startswith("  ✗ 自证 "))

    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        results = list(pool.map(one, SWITCH_EXPECTED_CELLS))
    switch_failures = [] if len(results) == len(SWITCH_EXPECTED_CELLS) else [f"开关轮派出 {len(SWITCH_EXPECTED_CELLS)} 个、收回 {len(results)} 个"]
    for switch, red in results:
        print(f"  · 弄坏开关 {BREAK_VARIABLE}={switch}：红在 {'、'.join(sorted(red)) or '一格都没有'}")
        missing = [cell for cell in SWITCH_EXPECTED_CELLS[switch] if cell not in red]
        if missing:
            switch_failures.append(f"{switch} 没红在 {'、'.join(missing)}（红的是 {'、'.join(sorted(red)) or '一格都没有'}）")
    for failure in switch_failures:
        print(f"  ✗ 弄坏开关 {failure}")   # gate-lint:detail
    if switch_failures:
        print("  → 怎么办：照上面那几个开关看它在代码里弄坏的那一处还在不在判法上；改了判法就改 SWITCH_EXPECTED_CELLS 与文件头的表")
        return 1
    print(f"  ✓ source-stamp.py 自证通过（查了 {checked} 格：只 rsync -a 会被旧 mtime 骗、rsync -a 之后 refresh 一次就重编换了内容的、sync 入口只重编内容变了的、sync 不拷下面几层的编译目录、"
          f"没有戳时全 touch、编不过又被改回旧 mtime 的再 touch、没改的不 touch、编译目录不在只写戳；弄坏开关 {len(results)} 个逐个打开都红在点名的格）")
    return 0


def parse_options(arguments, positional_count):
    """(位置参数, target_dir, includes)；用法错抛 ValueError。"""
    positional, target, includes = [], None, []
    index = 0
    while index < len(arguments):
        word = arguments[index]
        if word in ("--target-dir", "--include"):
            if index + 1 >= len(arguments):
                raise ValueError(f"{word} 后面没给值")
            if word == "--target-dir":
                target = arguments[index + 1]
            else:
                includes.append(arguments[index + 1])
            index += 2
            continue
        if word.startswith("-"):
            raise ValueError(f"认不出的参数「{word}」")
        positional.append(word)
        index += 1
    if len(positional) != positional_count:
        raise ValueError(f"要 {positional_count} 个路径，读到 {len(positional)} 个")
    return positional, target, tuple(includes) or DEFAULT_INCLUDES


def main(arguments):
    if os.environ.get(BREAK_VARIABLE):
        print(f"  ✗ 弄坏开关 {BREAK_VARIABLE} 设在 --selftest 之外，什么都没做", file=sys.stderr)
        print(f"  → 怎么办：去掉 {BREAK_VARIABLE} 再跑；它只给 source-stamp.py --selftest 证红用", file=sys.stderr)
        return EXIT_USAGE
    action = arguments[0] if arguments else ""
    try:
        if action == "--write-selftest-cargo" and len(arguments) == 2:
            write_fake_cargo(arguments[1])
            return 0
        if action not in ("refresh", "sync"):
            raise ValueError("第一个参数要是 refresh、sync、--selftest 或 --write-selftest-cargo <路径>")
        positional, target, includes = parse_options(arguments[1:], 1 if action == "refresh" else 2)
    except ValueError as error:
        print(f"  ✗ source-stamp.py 用法错：{error}", file=sys.stderr)
        print("  → 怎么办：source-stamp.py refresh <树> [--target-dir <编译目录>] [--include <路径>]…，或 sync <源树> <副本> [同上]、--selftest", file=sys.stderr)
        return EXIT_USAGE
    tree = os.path.abspath(positional[-1])
    say = lambda line: print(line, flush=True)   # noqa: E731
    try:
        if action == "refresh":
            refresh(tree, target_directory(tree, target, os.environ), includes, say)
        else:
            sync(os.path.abspath(positional[0]), tree, target_directory(tree, target, os.environ), includes, say)
    except OSError as error:
        print(f"  ✗ source-stamp.py {action} 没做成：{error}", file=sys.stderr)
        print("  → 怎么办：看树与编译目录在不在、可不可写；没做成之前别复用这个编译目录去编（换一个新编译目录，或修好再跑这一条）", file=sys.stderr)
        return EXIT_REFUSED
    return 0


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if sys.argv[1:] == ["--selftest-cells"]:
        cell_failures, _ = run_cells_in_scratch()
        sys.exit(1 if cell_failures else 0)
    sys.exit(main(sys.argv[1:]))
