#!/usr/bin/env python3
# admission: always 每个实现员补丁交回都要打一次，判的是这一刻的补丁目录与主工作区
# run-condition: command git
"""把实现员交回的补丁目录打进主工作区：补丁、变异表按名字合并、跑 code-source-discipline，记一笔「自上次崩溃验证全绿以来打了几个补丁」。

用法：
    apply-writer-patch.py <补丁目录> [--root 仓根] [--dry-run]
    apply-writer-patch.py --selftest          # APPLY_WRITER_PATCH_BREAK=<项> 时必须判红

补丁目录里的文件（格式由 .claude/agents/implementation-writer.md「产出」一节定）：
    crates.patch                 `git diff -- crates litmus ':!crates/mutations.tsv'` 的输出；没有代码改动可以没有
    mutations-append.tsv         要追加的变异行，每行六段（与 crates/mutations.tsv 同），名字在表里不许已有
    mutations-replacements.tsv   要整行换掉的变异行，每行六段，按第一段的名字找表里恰好一行换掉
    mutations-delete.txt         要删的变异名，一行一个，表里要恰好一行
    report.md                    实现员的报告；补丁动了 crates/singlefs-checker/src/ 的，报告里要有「受影响的层 0 流与崩溃枚举用例」一节
次序：先全部核一遍（补丁 git apply --check、名字在不在、行是不是六段、checker 那一节在不在），有一处不对就一个字不改、退 2；
都对了才 git apply、改名换上新的变异表，跑 .claude/gate.d/code-source-discipline.sh 的 mutation-tables 那一格（--check mutation-tables），把这一次记进 <git common-dir>/singlefs-applied-writer-patches.tsv。
自上次崩溃验证全绿（git common-dir 里最新的 singlefs-crash-case-green.* 标记）以来打过的补丁到 CRASH_VERIFY_HINT_PATCHES 个（5，推的）就打一行提示：
只提示、不拦，跑不跑由用户定。
退出码：0 打上了（code-source-discipline 绿）；1 打上了但 code-source-discipline 红；2 核的时候就不对，一个字没改。
"""
import glob
import hashlib
import os
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402
BROKEN = os.environ.get("APPLY_WRITER_PATCH_BREAK", "")
CRASH_VERIFY_HINT_PATCHES = int(os.environ.get("CRASH_VERIFY_HINT_PATCHES", "5"))   # 推的，没量过
CHECKER_SECTION = "受影响的层 0 流与崩溃枚举用例"
LOG_NAME = "singlefs-applied-writer-patches.tsv"
MARKER_PREFIX = "singlefs-crash-case-green."


def git(root, *arguments, check=False):
    return subprocess.run(["git", "-C", root, *arguments], capture_output=True, text=True, check=check)


def table_rows(path):
    return [line.rstrip("\n").split("\t") for line in open(path, encoding="utf-8") if line.strip() and not line.startswith("#")] if os.path.isfile(path) else []


def plan(patch_directory, root):
    """→ (问题清单, 新的变异表全文或 None, 补丁路径或 None)。"""
    problems = []
    patch_path = os.path.join(patch_directory, "crates.patch")
    patch = patch_path if os.path.isfile(patch_path) and os.path.getsize(patch_path) > 0 else None
    if patch:
        touched = [line[len("+++ b/"):].strip() for line in open(patch, encoding="utf-8", errors="replace") if line.startswith("+++ b/")]
        if "crates/mutations.tsv" in touched:
            problems.append("crates.patch 里动了 crates/mutations.tsv：变异表的改动写进那三个文件，按名字合并")
        report_path = os.path.join(patch_directory, "report.md")
        report = open(report_path, encoding="utf-8", errors="replace").read() if os.path.isfile(report_path) else ""
        if any(path.startswith("crates/singlefs-checker/src/") for path in touched) and CHECKER_SECTION not in report and BROKEN != "checker-section-ignored":
            problems.append(f"补丁动了 crates/singlefs-checker/src/，report.md 里却没有「{CHECKER_SECTION}」一节")
        checked = git(root, "apply", "--check", patch)
        if checked.returncode != 0 and BROKEN != "apply-unchecked":
            problems.append(f"git apply --check 不过：{checked.stderr.strip()[:300]}")
    table_path = os.path.join(root, "crates", "mutations.tsv")
    if not os.path.isfile(table_path):
        return problems + ["主工作区没有 crates/mutations.tsv"], None, patch
    lines = open(table_path, encoding="utf-8").read().splitlines(keepends=True)
    names = [line.split("\t", 1)[0] for line in lines if line.strip() and not line.startswith("#")]
    appended = table_rows(os.path.join(patch_directory, "mutations-append.tsv"))
    replaced = table_rows(os.path.join(patch_directory, "mutations-replacements.tsv"))
    deletion_path = os.path.join(patch_directory, "mutations-delete.txt")
    deleted = [line.strip() for line in open(deletion_path, encoding="utf-8")] if os.path.isfile(deletion_path) else []
    deleted = [name for name in deleted if name and not name.startswith("#")]
    for kind, rows in (("追加", appended), ("替换", replaced)):
        for row in rows:
            if len(row) != 6:
                problems.append(f"{kind}的「{row[0]}」不是六段（{len(row)} 段）")
    for row in appended:
        if names.count(row[0]) != 0 and BROKEN != "append-duplicates":
            problems.append(f"追加的「{row[0]}」表里已经有了")
    for name in [row[0] for row in replaced] + deleted:
        if names.count(name) != 1:
            problems.append(f"「{name}」在表里有 {names.count(name)} 行，要恰好 1 行")
    if problems:
        return problems, None, patch
    replacement_by_name = {row[0]: "\t".join(row) + "\n" for row in replaced}
    merged = []
    for line in lines:
        name = line.split("\t", 1)[0] if line.strip() and not line.startswith("#") else None
        if name in deleted:
            continue
        merged.append(replacement_by_name.get(name, line))
    if merged and not merged[-1].endswith("\n"):
        merged[-1] += "\n"
    merged += ["\t".join(row) + "\n" for row in appended]
    return [], "".join(merged), patch


def patches_since_last_crash_verification(root, log_path):
    common = git(root, "rev-parse", "--git-common-dir").stdout.strip()
    common = common if os.path.isabs(common) else os.path.join(root, common)
    markers = glob.glob(os.path.join(common, MARKER_PREFIX + "*"))
    since = max((os.path.getmtime(path) for path in markers), default=0.0)
    rows = [line.split("\t") for line in open(log_path, encoding="utf-8")] if os.path.isfile(log_path) else []
    return sum(1 for row in rows if row and float(row[0]) > since)


def apply(patch_directory, root, dry_run, gate33):
    problems, merged, patch = plan(patch_directory, root)
    if problems:
        return 2, [f"✗ {problem}" for problem in problems] + ["→ 怎么办：一个字都没改。照每处改补丁目录（或退给实现员），再跑一次"]
    if dry_run:
        return 0, [f"✓ 核过了（--dry-run，没改）：补丁 {'有' if patch else '没有'}，变异表合并之后 {merged.count(chr(10))} 行"]
    if patch:
        applied = git(root, "apply", patch)
        if applied.returncode != 0:
            return 2, [f"✗ git apply 失败：{applied.stderr.strip()[:300]}", "→ 怎么办：工作区在核与打之间被改了；git status 看一眼再跑"]
    table_path = os.path.join(root, "crates", "mutations.tsv")
    temporary = f"{table_path}.{os.getpid()}.tmp"
    with open(temporary, "x", encoding="utf-8") as handle:
        handle.write(merged)
    os.replace(temporary, table_path)
    lines = [f"✓ 打上了：{os.path.basename(os.path.normpath(patch_directory))}（补丁 {'有' if patch else '没有'}）"]
    gate = subprocess.run(gate33, cwd=root, capture_output=True, text=True)
    gate_line = ([line for line in gate.stdout.splitlines() if line.strip()] or ["（没有输出）"])[-1]
    lines.append(f"code-source-discipline：退出码 {gate.returncode}，{gate_line.strip()}")
    common = git(root, "rev-parse", "--git-common-dir").stdout.strip()
    log_path = os.path.join(common if os.path.isabs(common) else os.path.join(root, common), LOG_NAME)
    digest = hashlib.sha256(open(patch, "rb").read()).hexdigest() if patch else "-"
    with open(log_path, "a", encoding="utf-8") as handle:
        handle.write(f"{time.time()}\t{os.path.abspath(patch_directory)}\t{digest}\n")
    count = patches_since_last_crash_verification(root, log_path)
    if count >= CRASH_VERIFY_HINT_PATCHES and BROKEN != "no-hint":
        lines.append(f"! 自上次崩溃验证全绿以来已打 {count} 个实现员补丁（提示线 {CRASH_VERIFY_HINT_PATCHES}，推的）：弹窗问用户要不要用 "
                     "SINGLEFS_HEAVY_TESTS=user-request 派 crash-verifier 跑一次（54 号按用例复用，只跑输入变了的崩溃枚举用例）；只提示，不拦")
    lines.append(f"调度表可贴的一行：| {os.path.basename(os.path.normpath(patch_directory))} | 已打（{time.strftime('%m-%d %H:%M', time.localtime())}） | code-source-discipline {gate.returncode} |")
    return (0 if gate.returncode == 0 else 1), lines


def selftest():
    work = tempfile.mkdtemp(prefix="apply-writer-patch-selftest-")
    try:
        root = os.path.join(work, "repo")
        os.makedirs(os.path.join(root, "crates", "demo", "src"))
        os.makedirs(os.path.join(root, "crates", "singlefs-checker", "src"))
        open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w").write("fn a() -> u32 { 1 }\n")
        open(os.path.join(root, "crates", "singlefs-checker", "src", "lib.rs"), "w").write("fn check() {}\n")
        open(os.path.join(root, "crates", "mutations.tsv"), "w").write("# 表头\nkeep\tf\to\tr\t-p demo --lib\tt\nold\tf\to\tr\t-p demo --lib\tt\ngone\tf\to\tr\t-p demo --lib\tt\n")
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
            git(root, *command, check=True)
        def patch_directory(name, source_change=None, append="", replace="", delete="", report=""):
            directory = os.path.join(work, name)
            os.makedirs(directory)
            if source_change:
                relative, new_text = source_change
                path = os.path.join(root, relative)
                original = open(path).read()
                open(path, "w").write(new_text)
                open(os.path.join(directory, "crates.patch"), "w").write(git(root, "diff", "--", relative).stdout)
                open(path, "w").write(original)
            for file_name, text in (("mutations-append.tsv", append), ("mutations-replacements.tsv", replace), ("mutations-delete.txt", delete), ("report.md", report)):
                if text:
                    open(os.path.join(directory, file_name), "w").write(text)
            return directory
        gate33 = ["true"]
        failures = []
        good = patch_directory("good", ("crates/demo/src/lib.rs", "fn a() -> u32 { 2 }\n"), append="new\tf\to\tr\t-p demo --lib\tt\n",
                               replace="old\tf\to2\tr2\t-p demo --lib\tt\n", delete="gone\n")
        code, lines = apply(good, root, False, gate33)
        table = open(os.path.join(root, "crates", "mutations.tsv")).read()
        if code != 0 or "fn a() -> u32 { 2 }" not in open(os.path.join(root, "crates", "demo", "src", "lib.rs")).read() \
                or table != "# 表头\nkeep\tf\to\tr\t-p demo --lib\tt\nold\tf\to2\tr2\t-p demo --lib\tt\nnew\tf\to\tr\t-p demo --lib\tt\n":
            failures.append(f"好的补丁应当打上、按名字合表，实际 {code} {lines} {table!r}")
        checker = patch_directory("checker", ("crates/singlefs-checker/src/lib.rs", "fn check() { assert!(true); }\n"), report="只改了判定\n")
        code, lines = apply(checker, root, False, gate33)
        if code != 2 or not any("受影响的层 0 流" in line for line in lines):
            failures.append(f"动 checker 而报告没那一节应当拒绝，实际 {code} {lines}")
        duplicate = patch_directory("duplicate", append="keep\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(duplicate, root, False, gate33)
        if code != 2:
            failures.append(f"追加已有的名字应当拒绝，实际 {code} {lines}")
        stale = patch_directory("stale", ("crates/demo/src/lib.rs", "fn a() -> u32 { 9 }\n"))
        open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w").write("fn a() -> u32 { 3 }\n")
        code, lines = apply(stale, root, False, gate33)
        if code != 2 or not any("apply --check" in line for line in lines):
            failures.append(f"打不上的补丁应当在核的时候拒绝，实际 {code} {lines}")
        os.environ["CRASH_VERIFY_HINT_PATCHES"] = "1"
        global CRASH_VERIFY_HINT_PATCHES
        CRASH_VERIFY_HINT_PATCHES = 1
        only_table = patch_directory("onlytable", append="another\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(only_table, root, False, gate33)
        if code != 0 or not any("崩溃验证全绿以来已打" in line for line in lines):
            failures.append(f"打过的补丁数到提示线应当打提示，实际 {code} {lines}")
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 plan() / apply() 的判法；APPLY_WRITER_PATCH_BREAK 设着的话这里本来就该红")
            return 1
        print("  ✓ apply-writer-patch 自检通过：好补丁打上、变异表按名字追加替换删除，动 checker 没那一节、追加重名、打不上的都在核的时候拒绝，到提示线打提示（查了 5 种）")
        return 0
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    preflight(__file__)
    arguments = sys.argv[1:]
    if arguments == ["--selftest"]:
        sys.exit(selftest())
    root = REPOSITORY_ROOT
    if "--root" in arguments:
        position = arguments.index("--root")
        root = os.path.abspath(arguments[position + 1])
        arguments = arguments[:position] + arguments[position + 2:]
    dry_run = "--dry-run" in arguments
    arguments = [argument for argument in arguments if argument != "--dry-run"]
    if len(arguments) != 1 or not os.path.isdir(arguments[0]):
        print("✗ 要给一个补丁目录\n→ 怎么办：apply-writer-patch.py <补丁目录> [--root 仓根] [--dry-run]")
        sys.exit(2)
    code, lines = apply(arguments[0], root, dry_run, ["bash", os.path.join(root, ".claude", "gate.d", "code-source-discipline.sh"), "--check", "mutation-tables"])
    print("\n".join(lines))
    sys.exit(code)
