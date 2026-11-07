#!/usr/bin/env python3
# admission: always 判的是此刻源树里要拷的那几处有多大、副本里已有什么，上一次的结论不替这一次作保
# run-condition: command python3 rsync
"""建最小仓副本：只拷编译要用的那几处，拷前报大小、超阈值拒绝，拷过去的文件修改时刻是此刻。

为什么：手敲 cp -a 建 GPU 证红的小副本时把 research/ 一起拷了，一下 31G（.claude/kb/milestone/03-third-txn.md「待优化」第 17 行）。
修改时刻：rsync -a 把源树里的旧修改时刻原样带过去，cargo 判副本里内容已换的文件「已编过」、静默跑旧二进制（同一张表第 1 行，
.claude/rules/test-script-discipline.md「编译与算法都要快」）；这里照 lib_repository_copy.py 的 copy_entries 拷（rsync -a --no-times --checksum）：按内容比，
内容变了的才拷、拷过去的修改时刻是此刻，没变的不动。

用法：
  minimal-copy.py <源树> <副本> [--with-gate-tables] [--with-test-inputs] [--with <条目>=<为什么>]… [--target-dir <编译目录>]
      名单不在这里写：取 scripts/hosts/lib_repository_copy.py 的 copy_closure（共用清单 crates/、Cargo.toml、Cargo.lock、.cargo/ 加它的闭包：
      path 依赖指到的目录（例：验证包依赖的仓根 prune-server/）、编译期 include_str! 读的仓根文件（例：.claude/heavy-thread-default）、
      副本里起的二进制与脚本运行期读的仓根文件连同它们 source、起的；那份文件头「闭包」）。--with-gate-tables 再声明 .claude/gate.d/（要门禁表的副本）；
      --with-test-inputs 连测试代码一起扫（副本里要跑测试的：scripts/mutation/prove-red.sh --copy 这类）；
      --with <条目>=<为什么> 逐条声明要多拷的仓根条目（副本里要起的脚本、要读的数据；为什么至少 4 个字，交 copy_closure 的 declared）：
      声明的脚本连同它 source、起、import 的仓内文件一起拷（例：--with scripts/prune-server/prune_server_client_selftest.py=副本里起它的自证，
      连 .claude/scripts/project_preflight.py 与规范副本里的 preflight.py 一起带上）。不写为什么、条目跳出仓根的退 2。
      --with-gate-tables 的 .claude/gate.d/ 与 --with 声明的目录同样：里面每一份脚本（fixtures/ 下的样本不算）连同它 source、起、import 的一起拷
      （例：门禁 source 的 .claude/scripts/preflight.sh；副本缺它，门禁 source 失败、参数被清空成全跑）；照脚本所在目录引到的文件源树里不在的拒、退 1、
      一个文件都不写，拒绝说明逐个点名缺哪几份（lib_repository_copy.py 文件头「闭包」声明的条目那一条）。
      闭包算不成（path 依赖指到仓根之外、编译期读的文件认不出或不在）拒绝、退 1、副本里一个文件都不写。
      拷之前先报每一处的大小与合计，合计超过阈值就拒绝、退 1，副本里一个文件都不写；名单里的每一处不进名叫 .git、target、target-*、__pycache__ 的目录。
      副本里起命令的，多机配置照 lib_repository_copy.py configuration-path <源树> 打的那一份显式交给 SINGLEFS_MULTI_HOST_CONFIG（副本不是 git 工作树，
      判法照仓位置找不到它）。
      不带 --delete：副本里已有的别的东西不动，顶层有名单之外的项（上一次手拷留下的 research/ 这类）报一行警告。
      给了 --target-dir：拷完对副本做一次 scripts/hosts/source-stamp.py 的 refresh（复用这个编译目录之前要做的那一步）；不给不动编译目录。
  minimal-copy.py --selftest
阈值 MAXIMUM_COPY_BYTES = 200 MiB，推的：量的名单（含 .claude/gate.d）合计约 27 MiB，取约 7 倍，容得下 crates/ 长大、挡得住整个 research/ 或 target。
  真长到超了阈值：先看拒绝信息列的最大几个文件是不是该进副本，确该进再抬 MAXIMUM_COPY_BYTES 并改上面这句的依据。
输出：一行 MINIMAL_COPY plan ...（大小）、一行 MINIMAL_COPY copied ...；退出码：0 拷好；1 超阈值或拷不成；2 用法错（源树不是仓根、副本在源树里面、参数错）。

弄坏开关 MINIMAL_COPY_BREAK=<项>（只给 --selftest 证红用，逗号连几个；在 --selftest 之外设了就拒、退 2）。项名、弄坏哪一处、打开后红在哪一格
  （格名是「✗ 自证 <格名>：」里的那一段；末尾那一轮照 SWITCH_EXPECTED_CELLS 逐个核）：
  MINIMAL_COPY_BREAK=copy-research  名单里多拷 research/
      「✗ 自证 whitelist-only：」
  MINIMAL_COPY_BREAK=no-size-limit  超阈值不拒绝，照拷
      「✗ 自证 over-threshold-refused：」
  MINIMAL_COPY_BREAK=keep-times  拷完把副本里的修改时刻改回源树那一份的
      「✗ 自证 mtime-now：」
  MINIMAL_COPY_BREAK=gate-tables-always  不带 --with-gate-tables 也拷 .claude/gate.d/
      「✗ 自证 gate-tables-opt-in：」
  MINIMAL_COPY_BREAK=no-size-report  拷前不报大小
      「✗ 自证 size-reported-before-copy：」
  MINIMAL_COPY_BREAK=no-refresh  给了 --target-dir 也不 refresh
      「✗ 自证 refresh-with-target-dir：」
  MINIMAL_COPY_BREAK=no-root-check  源树不是仓根也照拷
      「✗ 自证 refuses-non-root：」
  MINIMAL_COPY_BREAK=no-extra-warning  副本顶层有名单之外的项也不报
      「✗ 自证 warns-about-extra-top-level：」
  MINIMAL_COPY_BREAK=closure-ignored  不调 copy_closure，只拷共用清单里在的那几样
      「✗ 自证 thread-registry-copied：」、「✗ 自证 whitelist-only：」、「✗ 自证 path-dependencies-copied：」、「✗ 自证 path-dependency-outside-root-refused：」、
      「✗ 自证 test-inputs-opt-in：」、「✗ 自证 declared-entries-copied：」
  MINIMAL_COPY_BREAK=test-inputs-always  不带 --with-test-inputs 也扫测试代码
      「✗ 自证 test-inputs-opt-in：」
  MINIMAL_COPY_BREAK=declared-ignored  --with 声明的条目不交给 copy_closure，照没给
      「✗ 自证 declared-entries-copied：」
  MINIMAL_COPY_BREAK=gate-tables-closure-skipped  --with-gate-tables 与 --with 声明的条目里的脚本 source、起的不拷（副本缺 .claude/scripts/preflight.sh）
      「✗ 自证 gate-tables-closure-copied：」
"""
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import concurrent.futures  # noqa: E402
import contextlib  # noqa: E402
import importlib.util  # noqa: E402
import io  # noqa: E402
import os  # noqa: E402
import shutil  # noqa: E402
import subprocess  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402
import time  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
import lib_repository_copy as repository_copy  # noqa: E402

MEBIBYTE = 1024 * 1024
MAXIMUM_COPY_BYTES = 200 * MEBIBYTE          # 依据见文件头「阈值」，推的
THREAD_REGISTRY_ITEM = ".claude/heavy-thread-default"   # 自证样本里 include_str! 读的登记文件（名单由 copy_closure 现算，这里只给样本用）
REQUIRED_ITEMS = ("crates", "Cargo.toml")     # 没有这两样的目录不是仓根
GATE_TABLES_ITEM = ".claude/gate.d"
GATE_TABLES_REASON = "带 --with-gate-tables：副本里要读门禁表"
SKIPPED_DIRECTORY_NAMES = (".git", "__pycache__")
LISTED_LARGEST_FILES = 5
EXIT_REFUSED = 1
EXIT_USAGE = 2
BREAK_VARIABLE = "MINIMAL_COPY_BREAK"
STAMP_SCRIPT = os.path.join(os.path.dirname(os.path.realpath(__file__)), "source-stamp.py")


def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


def skipped_directory(name):
    return name in SKIPPED_DIRECTORY_NAMES or name == "target" or name.startswith("target-")


def human(byte_count):
    if byte_count >= 1024 * MEBIBYTE:
        return f"{byte_count / (1024 * MEBIBYTE):.1f} GiB"
    if byte_count >= MEBIBYTE:
        return f"{byte_count / MEBIBYTE:.1f} MiB"
    return f"{byte_count / 1024:.1f} KiB"


def items_to_copy(source, with_gate_tables, with_test_inputs, declarations=()):
    """要拷的仓根条目：lib_repository_copy.py 的 copy_closure（文件头「用法」）；declarations 是 --with 声明的 [(条目, 为什么)]（已过 checked_declarations）。
    闭包算不成抛 repository_copy.ClosureRefused。弄坏开关 closure-ignored：只交共用清单里在的那几样与声明的条目；test-inputs-always：不带 --with-test-inputs 也扫测试代码；
    declared-ignored：--with 声明的条目照没给。"""
    declared = [(GATE_TABLES_ITEM, GATE_TABLES_REASON)] if with_gate_tables or broken("gate-tables-always") else []
    if not broken("declared-ignored"):
        declared += list(declarations)
    if broken("closure-ignored"):
        items = [item for item in repository_copy.COMMON_ENTRIES if os.path.lexists(os.path.join(source, item))]
        items += [item for item, _ in declared if os.path.lexists(os.path.join(source, item))]
    else:
        closure = repository_copy.copy_closure(source, declared, test_inputs=with_test_inputs or broken("test-inputs-always"))
        items = list(closure.entries)
        if broken("gate-tables-closure-skipped"):   # 弄坏开关：不收声明的条目里的脚本 source、起的（只有那一支收进来的条目去掉）
            items = [item for item in items if not all(why.startswith(repository_copy.DECLARED_SCRIPTS_REASON) for why in closure.reasons.get(item, ("",)))]
    if broken("copy-research"):
        items.append("research")
    return items


def measure(source, items):
    """名单里每一处的 (相对路径, 文件数, 字节数)，与全部文件的 (字节数, 相对路径) 表；源树里没有的那一处不出现在第一个表里。
    字节数是文件自己的大小（lstat 的 st_size，符号链接按链接本身），不进名叫 .git、target、target-*、__pycache__ 的目录。"""
    per_item, every_file = [], []
    for item in items:
        path = os.path.join(source, item)
        item_bytes, item_files = 0, 0
        if os.path.isdir(path) and not os.path.islink(path):
            for directory, subdirectories, names in os.walk(path):
                subdirectories[:] = sorted(name for name in subdirectories if not skipped_directory(name))
                for name in names:
                    full = os.path.join(directory, name)
                    size = os.lstat(full).st_size
                    item_bytes += size
                    item_files += 1
                    every_file.append((size, os.path.relpath(full, source)))
        elif os.path.lexists(path):
            size = os.lstat(path).st_size
            item_bytes, item_files = size, 1
            every_file.append((size, item))
        else:
            continue
        per_item.append((item, item_files, item_bytes))
    return per_item, every_file


def top_level_extras(copy, items):
    """副本顶层有、名单里没有的项（.git、编译目录、名单里 .claude/… 的上一层 .claude 不算）。"""
    allowed = {item.split("/", 1)[0] for item in items} | {".git"}
    try:
        names = os.listdir(copy)
    except FileNotFoundError:
        return []
    return sorted(name for name in names if name not in allowed and not skipped_directory(name))


def load_stamp_module():
    spec = importlib.util.spec_from_file_location("source_stamp_for_minimal_copy", STAMP_SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def copy_tree(source, copy, items):
    """照 lib_repository_copy.py 的 copy_entries 拷（那份文件头「拷法」）；rsync 退非 0 抛 OSError。交拷法的说明（进 copied 那一行）。
    弄坏开关 keep-times：拷完把副本里每份文件的修改时刻改回源树那一份的。"""
    repository_copy.copy_entries(source, copy, items)
    if not broken("keep-times"):
        return "-a --no-times --checksum"
    for item in items:
        for directory, subdirectories, names in os.walk(os.path.join(copy, item)):
            subdirectories[:] = [name for name in subdirectories if not skipped_directory(name)]
            for name in names:
                copied = os.path.join(directory, name)
                original = os.path.join(source, os.path.relpath(copied, copy))
                if os.path.isfile(original):
                    os.utime(copied, (os.stat(original).st_mtime, os.stat(original).st_mtime))
        single = os.path.join(copy, item)
        if os.path.isfile(single) and os.path.isfile(os.path.join(source, item)):
            os.utime(single, (os.stat(os.path.join(source, item)).st_mtime, os.stat(os.path.join(source, item)).st_mtime))
    return "-a"


def execute(arguments):
    """整个流程：交退出码。--selftest 的各格直接调它（不经 main 的弄坏开关拒绝）。"""
    positional, with_gate_tables, with_test_inputs, target, declared = [], False, False, None, []
    index = 0
    try:
        while index < len(arguments):
            word = arguments[index]
            if word == "--with-gate-tables":
                with_gate_tables = True
            elif word == "--with-test-inputs":
                with_test_inputs = True
            elif word == "--with":
                if index + 1 >= len(arguments):
                    raise ValueError("--with 后面没给 <条目>=<为什么>")
                entry, separator, reason = arguments[index + 1].partition("=")
                if not separator:
                    raise ValueError(f"--with 要写成 <条目>=<为什么>，读到「{arguments[index + 1]}」")
                declared.append((entry, reason))
                index += 1
            elif word == "--target-dir":
                if index + 1 >= len(arguments):
                    raise ValueError("--target-dir 后面没给值")
                target = arguments[index + 1]
                index += 1
            elif word.startswith("-"):
                raise ValueError(f"认不出的参数「{word}」")
            else:
                positional.append(word)
            index += 1
        if len(positional) != 2:
            raise ValueError(f"要 <源树> <副本> 两个路径，读到 {len(positional)} 个")
        source, copy = os.path.realpath(positional[0]), os.path.realpath(positional[1])
        if not os.path.isdir(source):
            raise ValueError(f"源树 {source} 不在")
        missing = [item for item in REQUIRED_ITEMS if not os.path.lexists(os.path.join(source, item))]
        if missing and not broken("no-root-check"):
            raise ValueError(f"源树 {source} 不是仓根：缺 {'、'.join(missing)}")
        if os.path.commonpath((source, copy)) in (source, copy):
            raise ValueError(f"副本 {copy} 与源树 {source} 相同或互相包含")
        declarations = repository_copy.checked_declarations(declared)
    except ValueError as error:
        print(f"  ✗ minimal-copy.py 用法错：{error}", file=sys.stderr)
        print("  → 怎么办：minimal-copy.py <仓根> <副本目录> [--with-gate-tables] [--with-test-inputs] [--with <条目>=<为什么>]… [--target-dir <编译目录>]；"
              "副本放在仓外（草稿目录），--with 的条目写仓根下的相对路径、为什么至少 4 个字", file=sys.stderr)
        return EXIT_USAGE
    try:
        items = items_to_copy(source, with_gate_tables, with_test_inputs, declarations)
    except (repository_copy.ClosureRefused, OSError) as error:
        print(f"  ✗ 最小副本拒绝建：拷贝范围的闭包算不成：{error}；副本里一个文件都没写", file=sys.stderr)
        print("  → 怎么办：path 依赖要指到仓根之下（包挪进仓里，或改它的 Cargo.toml）；Cargo.toml 写坏了的先修好它；编译期读的文件写成"
              " scripts/hosts/lib_repository_copy.py 文件头「闭包」认得的写法并让它进仓；声明的条目里的脚本引到的文件不在的，补上上面点名的那几份、"
              "或改掉引到它们的那一行", file=sys.stderr)
        return EXIT_REFUSED
    per_item, every_file = measure(source, items)
    total = sum(item_bytes for _, _, item_bytes in per_item)
    total_files = sum(item_files for _, item_files, _ in per_item)
    limit_text = f"阈值 {human(MAXIMUM_COPY_BYTES)}（推的，文件头「阈值」）"
    if not broken("no-size-report"):
        print("MINIMAL_COPY plan: " + "、".join(f"{item} {human(item_bytes)}（{item_files} 个文件）" for item, item_files, item_bytes in per_item)
              + f"；合计 {human(total)}（{total_files} 个文件），{limit_text}", flush=True)
    if total > MAXIMUM_COPY_BYTES and not broken("no-size-limit"):
        print(f"  ✗ 最小副本拒绝建：要拷的合计 {human(total)} 超过{limit_text}，副本里一个文件都没写", file=sys.stderr)
        for size, relative in sorted(every_file, reverse=True)[:LISTED_LARGEST_FILES]:
            print(f"      {human(size):>10}  {relative}", file=sys.stderr)   # gate-lint:detail
        print("  → 怎么办：上面是最大的几个文件，不该进副本的从源树里挪走；确该进副本、是 crates/ 长大了，再抬 minimal-copy.py 的 MAXIMUM_COPY_BYTES 并改文件头写的依据", file=sys.stderr)
        return EXIT_REFUSED
    try:
        os.makedirs(copy, exist_ok=True)
        times = copy_tree(source, copy, items)
        print(f"MINIMAL_COPY copied: {source} → {copy}（rsync {times}，不带 --delete）", flush=True)
        if target is not None and not broken("no-refresh"):
            stamp = load_stamp_module()
            stamp.refresh(copy, stamp.target_directory(copy, target, os.environ), stamp.DEFAULT_INCLUDES, lambda line: print(line, flush=True))
    except OSError as error:
        print(f"  ✗ minimal-copy.py 没拷成：{error}", file=sys.stderr)
        print("  → 怎么办：看副本目录可不可写、盘满没有、rsync 在不在；没拷成的副本别拿去编", file=sys.stderr)
        return EXIT_REFUSED
    extras = [] if broken("no-extra-warning") else top_level_extras(copy, items)
    if extras:
        print(f"  ! 副本顶层有名单之外的项：{'、'.join(extras)}（不带 --delete，没动它们；上一次手拷留下的多半不该留，确认没用就删掉再编）", file=sys.stderr)
    return 0


# ── 自证 ──
OLD_MTIME_SECONDS = 1577836800     # 2020 年 1 月 1 日：源树里的文件带着这个旧修改时刻
RECENT_SECONDS = 3600              # 「此刻」：自证里文件的修改时刻离现在不超过这么多秒
REGISTRY_SAMPLE_TEXT = "7  # 自证样本的登记\n"
LIBRARY_SAMPLE_TEXT = 'pub const THREADS: &str = include_str!("../../../' + THREAD_REGISTRY_ITEM + '");\npub fn f() -> u32 { 1 }\n'


def write(tree, relative, text, mtime_seconds=OLD_MTIME_SECONDS):
    path = os.path.join(tree, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.utime(path, (mtime_seconds, mtime_seconds))


def make_source(base, name):
    """一棵带着名单里的东西、也带着不该拷的东西（research/、scripts/、各层编译目录、.git）的源树，文件都是旧修改时刻；
    crates/demo 经 include_str! 在编译时读每台默认重活线程数的登记文件（闭包认它）。"""
    source = os.path.join(base, name, "source")
    write(source, "Cargo.toml", "[workspace]\n")
    write(source, "Cargo.lock", "# lock\n")
    write(source, ".cargo/config.toml", "[build]\n")
    write(source, "crates/demo/Cargo.toml", '[package]\nname = "demo"\n')
    write(source, "crates/demo/src/lib.rs", LIBRARY_SAMPLE_TEXT)
    write(source, "crates/demo/target/debug/artifact.bin", "x" * 1000)
    write(source, ".claude/gate.d/stage.sh", "echo gate\n")
    write(source, THREAD_REGISTRY_ITEM, REGISTRY_SAMPLE_TEXT)
    write(source, ".claude/agents/a.md", "agent\n")
    write(source, "research/results/big.out", "r" * 5000)
    write(source, "scripts/tool.py", "print(1)\n")
    write(source, "target/release/binary", "t" * 5000)
    write(source, ".git/HEAD", "ref\n")
    return source


def run_captured(arguments):
    """execute 的 (退出码, stdout, stderr)。"""
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        code = execute(arguments)
    return code, out.getvalue(), err.getvalue()


def selftest_cells(root):
    """各格：交 (失败的格名列表, 查了几格)。每格自己一份源树与副本。"""
    failures, checked = [], 0

    def check(cell, condition, detail):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append(cell)
            print(f"  ✗ 自证 {cell}：{detail}")   # gate-lint:detail

    def tree_files(tree):
        return sorted(os.path.relpath(os.path.join(directory, name), tree) for directory, _, names in os.walk(tree) for name in names)

    # whitelist-only：源树里有 research/、scripts/、.claude/agents/、各层编译目录、.git ⇒ 副本里只有名单里的东西
    source = make_source(root, "whitelist-only")
    copy = os.path.join(root, "whitelist-only", "copy")
    code, _, error = run_captured([source, copy])
    wanted = [".cargo/config.toml", THREAD_REGISTRY_ITEM, "Cargo.lock", "Cargo.toml", "crates/demo/Cargo.toml", "crates/demo/src/lib.rs"]
    check("whitelist-only", code == 0 and tree_files(copy) == wanted,
          f"副本里应当只有 {wanted}，实际退 {code}、有 {tree_files(copy)}：{error.strip()[-200:]}")

    # over-threshold-refused：crates/ 里有个比阈值还大的文件（稀疏文件，不占盘）⇒ 退 1、副本里一个文件都没有、拒绝信息点名那个文件
    source = make_source(root, "over-threshold-refused")
    copy = os.path.join(root, "over-threshold-refused", "copy")
    with open(os.path.join(source, "crates/demo/data.bin"), "wb") as handle:
        handle.truncate(MAXIMUM_COPY_BYTES + MEBIBYTE)
    code, _, error = run_captured([source, copy])
    check("over-threshold-refused", code == EXIT_REFUSED and not tree_files(copy) and "crates/demo/data.bin" in error and "MAXIMUM_COPY_BYTES" in error,
          f"合计超过阈值应当退 {EXIT_REFUSED}、副本里没有文件、拒绝信息点名 crates/demo/data.bin 并写出路，实际退 {code}、副本有 {tree_files(copy)}：{error.strip()[-300:]}")

    # size-reported-before-copy：成功的一趟 ⇒ 先有「plan」行（带合计与阈值），再有「copied」行
    source = make_source(root, "size-reported-before-copy")
    copy = os.path.join(root, "size-reported-before-copy", "copy")
    code, output, _ = run_captured([source, copy])
    plan_at, copied_at = output.find("MINIMAL_COPY plan:"), output.find("MINIMAL_COPY copied:")
    check("size-reported-before-copy", code == 0 and 0 <= plan_at < copied_at and "合计" in output and "阈值" in output,
          f"拷之前应当先报每一处的大小、合计与阈值，实际退 {code}：{output.strip()[-300:]}")

    # mtime-now：源树文件是 2020 年的修改时刻 ⇒ 头一次拷过去的是此刻；副本里已有的那份内容变了的也改到此刻，没变的不动
    source = make_source(root, "mtime-now")
    copy = os.path.join(root, "mtime-now", "copy")
    run_captured([source, copy])
    write(source, "crates/demo/src/lib.rs", "pub fn f() -> u32 { 22 }\n")           # 内容变了、修改时刻还是旧的
    stale_copy_path = os.path.join(copy, "crates/demo/src/lib.rs")
    os.utime(stale_copy_path, (OLD_MTIME_SECONDS, OLD_MTIME_SECONDS))             # 副本里那份拷后又回到旧时刻（cargo 会认它已编）
    unchanged_path = os.path.join(copy, "Cargo.toml")
    os.utime(unchanged_path, (OLD_MTIME_SECONDS, OLD_MTIME_SECONDS))              # 没变的：不动
    code, _, _ = run_captured([source, copy])
    now = time.time()
    first = os.path.join(copy, "Cargo.lock")
    check("mtime-now", code == 0 and now - os.stat(first).st_mtime < RECENT_SECONDS and now - os.stat(stale_copy_path).st_mtime < RECENT_SECONDS
          and os.stat(unchanged_path).st_mtime == OLD_MTIME_SECONDS and open(stale_copy_path, encoding="utf-8").read().endswith("22 }\n"),
          f"头一次拷的、内容变了的应当是此刻的修改时刻，没变的不动，实际 Cargo.lock 离现在 {now - os.stat(first).st_mtime:.0f} 秒、lib.rs {now - os.stat(stale_copy_path).st_mtime:.0f} 秒、"
          f"Cargo.toml 的修改时刻{'没动' if os.stat(unchanged_path).st_mtime == OLD_MTIME_SECONDS else '被改了'}")

    # gate-tables-opt-in：不带 --with-gate-tables 副本里没有 .claude/gate.d/；带了有 .claude/gate.d/ 而没有 .claude/agents/
    source = make_source(root, "gate-tables-opt-in")
    plain, with_tables = os.path.join(root, "gate-tables-opt-in", "plain"), os.path.join(root, "gate-tables-opt-in", "with-tables")
    code_plain, _, _ = run_captured([source, plain])
    code_with, _, _ = run_captured([source, with_tables, "--with-gate-tables"])
    check("gate-tables-opt-in", code_plain == 0 and code_with == 0 and not os.path.exists(os.path.join(plain, ".claude/gate.d"))
          and os.path.isfile(os.path.join(with_tables, ".claude/gate.d/stage.sh")) and not os.path.exists(os.path.join(with_tables, ".claude/agents")),
          f"不带参数副本里不该有 .claude/gate.d/，带 --with-gate-tables 该有 .claude/gate.d/stage.sh 而没有 .claude/agents/，实际退 {code_plain} 与 {code_with}、"
          f"不带的有 {tree_files(plain)}、带的有 {tree_files(with_tables)}")

    # thread-registry-copied：源树里有 .claude/heavy-thread-default ⇒ 副本里有它、内容相同，.claude/ 底下不多拷别的（harness 经 include_str! 读它）
    source = make_source(root, "thread-registry-copied")
    copy = os.path.join(root, "thread-registry-copied", "copy")
    code, _, error = run_captured([source, copy])
    registry_copy = os.path.join(copy, THREAD_REGISTRY_ITEM)
    registry_text = open(registry_copy, encoding="utf-8").read() if os.path.isfile(registry_copy) else None
    claude_files = [path for path in tree_files(copy) if path.startswith(".claude/")]
    check("thread-registry-copied", code == 0 and registry_text == REGISTRY_SAMPLE_TEXT and claude_files == [THREAD_REGISTRY_ITEM],
          f"副本里应当有 {THREAD_REGISTRY_ITEM}、内容与源树相同、.claude/ 底下只有它，实际退 {code}、登记文件内容 {registry_text!r}、"
          f".claude/ 底下有 {claude_files}：{error.strip()[-200:]}")

    # refresh-with-target-dir：编译目录里有东西而没有戳（头一次用）、给了 --target-dir ⇒ 输出源码戳那一行、戳写进编译目录
    source = make_source(root, "refresh-with-target-dir")
    copy = os.path.join(root, "refresh-with-target-dir", "copy")
    target = os.path.join(root, "refresh-with-target-dir", "target")
    write(target, "debug/old-artifact", "built")
    code, output, error = run_captured([source, copy, "--target-dir", target])
    check("refresh-with-target-dir", code == 0 and "SOURCE_STAMP" in output and os.path.isfile(os.path.join(target, ".singlefs-source-stamp.tsv")),
          f"给了 --target-dir 应当拷完 refresh、输出 SOURCE_STAMP 行、编译目录里有戳，实际退 {code}：{output.strip()[-200:]} {error.strip()[-200:]}")

    # refuses-non-root：源树不是仓根（没有 Cargo.toml 与 crates/）⇒ 退 2、副本里没有东西
    base = os.path.join(root, "refuses-non-root")
    write(base, "source/research/results/big.out", "r" * 5000)
    copy = os.path.join(base, "copy")
    code, _, error = run_captured([os.path.join(base, "source"), copy])
    check("refuses-non-root", code == EXIT_USAGE and not tree_files(copy) and "不是仓根" in error,
          f"没有 Cargo.toml 与 crates/ 的目录应当退 {EXIT_USAGE}、报「不是仓根」、副本里没有文件，实际退 {code}、副本有 {tree_files(copy)}：{error.strip()[-200:]}")

    # warns-about-extra-top-level：副本里已有上一次手拷留下的 research/ ⇒ 报警告点名它，不删它
    source = make_source(root, "warns-about-extra-top-level")
    copy = os.path.join(root, "warns-about-extra-top-level", "copy")
    write(copy, "research/old.out", "leftover")
    code, _, error = run_captured([source, copy])
    check("warns-about-extra-top-level", code == 0 and "名单之外的项：research" in error and os.path.isfile(os.path.join(copy, "research/old.out")),
          f"副本顶层有 research/ 应当报警告点名它、不删它，实际退 {code}：{error.strip()[-200:]}")

    # path-dependencies-copied：crates/demo 经 path 依赖仓根下的 outside/（它再依赖 nested/）⇒ 两处都拷、它们的编译目录不拷、不报名单之外
    source = make_source(root, "path-dependencies-copied")
    write(source, "crates/demo/Cargo.toml", '[package]\nname = "demo"\n[dependencies]\noutside = { path = "../../outside", optional = true }\n')
    write(source, "outside/Cargo.toml", '[package]\nname = "outside"\n[target.\'cfg(unix)\'.dependencies]\nnested = { path = "../nested" }\n')
    write(source, "outside/src/lib.rs", "pub fn g() {}\n")
    write(source, "outside/target/debug/artifact.bin", "x" * 1000)
    write(source, "nested/Cargo.toml", '[package]\nname = "nested"\n')
    write(source, "unrelated/Cargo.toml", '[package]\nname = "unrelated"\n')
    copy = os.path.join(root, "path-dependencies-copied", "copy")
    code, _, error = run_captured([source, copy])
    copied = [name for name in tree_files(copy) if name.split("/", 1)[0] in ("outside", "nested", "unrelated")]
    wanted = ["nested/Cargo.toml", "outside/Cargo.toml", "outside/src/lib.rs"]
    check("path-dependencies-copied", code == 0 and copied == wanted and "名单之外" not in error,
          f"path 依赖指到的 outside/ 与它再指到的 nested/ 应当拷进副本（编译目录不拷、unrelated/ 不拷、不报名单之外），实际退 {code}、拷了 {copied}：{error.strip()[-200:]}")

    # path-dependency-outside-root-refused：path 依赖指到仓根之外 ⇒ 退 1、副本里一个文件都没有、拒绝信息点名它
    source = make_source(root, "path-dependency-outside-root-refused")
    write(source, "crates/demo/Cargo.toml", '[package]\nname = "demo"\n[dev-dependencies]\naway = { path = "../../../away" }\n')
    copy = os.path.join(root, "path-dependency-outside-root-refused", "copy")
    code, _, error = run_captured([source, copy])
    check("path-dependency-outside-root-refused", code == EXIT_REFUSED and not tree_files(copy) and "仓根" in error and "away" in error,
          f"path 依赖指到仓根之外应当退 {EXIT_REFUSED}、副本里没有文件、拒绝信息点名它，实际退 {code}、副本有 {tree_files(copy)}：{error.strip()[-200:]}")

    # test-inputs-opt-in：crates/demo/tests/ 下的测试读仓根的 litmus/a.litmus ⇒ 不带 --with-test-inputs 不拷它，带了拷
    source = make_source(root, "test-inputs-opt-in")
    write(source, "crates/demo/tests/reads.rs", 'const LITMUS: &str = "../../litmus/a.litmus";\n')
    write(source, "litmus/a.litmus", "C a\n")
    plain, with_tests = os.path.join(root, "test-inputs-opt-in", "plain"), os.path.join(root, "test-inputs-opt-in", "with-tests")
    code_plain, _, _ = run_captured([source, plain])
    code_with, _, error = run_captured([source, with_tests, "--with-test-inputs"])
    check("test-inputs-opt-in", code_plain == 0 and code_with == 0 and not os.path.exists(os.path.join(plain, "litmus"))
          and os.path.isfile(os.path.join(with_tests, "litmus/a.litmus")),
          f"不带参数副本里不该有 litmus/、带 --with-test-inputs 该有 litmus/a.litmus，实际退 {code_plain} 与 {code_with}、"
          f"不带的有 {tree_files(plain)}、带的有 {tree_files(with_tests)}：{error.strip()[-200:]}")

    # declared-entries-copied：--with 声明副本里要起的 tools/run.py ⇒ 它与它 import 的同目录 helper.py 一起拷进副本，同目录没被点名的 unused.py 不拷；
    # 不写为什么（--with tools/run.py=）⇒ 退 2、报「为什么」、副本里一个文件都不写
    source = make_source(root, "declared-entries-copied")
    write(source, "tools/run.py", "import helper\nprint(helper.value)\n")
    write(source, "tools/helper.py", "value = 1\n")
    write(source, "tools/unused.py", "value = 2\n")
    copy = os.path.join(root, "declared-entries-copied", "copy")
    unreasoned = os.path.join(root, "declared-entries-copied", "unreasoned")
    code, _, error = run_captured([source, copy, "--with", "tools/run.py=副本里要起这份脚本"])
    code_unreasoned, _, error_unreasoned = run_captured([source, unreasoned, "--with", "tools/run.py="])
    copied_tools = [path for path in tree_files(copy) if path.startswith("tools/")]
    check("declared-entries-copied", code == 0 and copied_tools == ["tools/helper.py", "tools/run.py"] and code_unreasoned == EXIT_USAGE
          and not tree_files(unreasoned) and "为什么" in error_unreasoned,
          f"--with 声明的 tools/run.py 应当连它 import 的 tools/helper.py 一起拷、不拷 tools/unused.py，不写为什么应当退 {EXIT_USAGE}、副本里没有文件，"
          f"实际退 {code}、拷了 {copied_tools}，不写为什么退 {code_unreasoned}、副本有 {tree_files(unreasoned)}：{error.strip()[-200:]} {error_unreasoned.strip()[-200:]}")

    # gate-tables-closure-copied（副本缺 .claude/scripts/preflight.sh，门禁 source 失败、参数被清空成全跑）：
    # 带 --with-gate-tables，门禁表里的 stage.sh 照自己所在目录 source ../scripts/preflight.sh ⇒ 副本里有 .claude/scripts/preflight.sh；
    # 源树里没有那一份（闭包算不全）⇒ 退 1、副本里一个文件都不写、拒绝说明点名缺的那一份
    source = make_source(root, "gate-tables-closure-copied")
    write(source, ".claude/gate.d/stage.sh", '#!/usr/bin/env bash\nsource "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"\necho gate\n')
    write(source, ".claude/scripts/preflight.sh", "preflight() { :; }\n")
    copy = os.path.join(root, "gate-tables-closure-copied", "copy")
    code, _, error = run_captured([source, copy, "--with-gate-tables"])
    gap_source = make_source(root, "gate-tables-closure-gap")
    write(gap_source, ".claude/gate.d/stage.sh", '#!/usr/bin/env bash\nsource "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"\necho gate\n')
    gap_copy = os.path.join(root, "gate-tables-closure-gap", "copy")
    gap_code, _, gap_error = run_captured([gap_source, gap_copy, "--with-gate-tables"])
    check("gate-tables-closure-copied", code == 0 and os.path.isfile(os.path.join(copy, ".claude/scripts/preflight.sh"))
          and gap_code == EXIT_REFUSED and not tree_files(gap_copy) and ".claude/scripts/preflight.sh" in gap_error,
          f"带 --with-gate-tables 时门禁表 source 的 .claude/scripts/preflight.sh 应当一起拷，源树里没有它时应当退 {EXIT_REFUSED}、副本里没有文件、点名它，"
          f"实际退 {code}、副本里 .claude/ 下有 {[path for path in tree_files(copy) if path.startswith('.claude/')]}，缺的那一棵退 {gap_code}、副本有 {tree_files(gap_copy)}："
          f"{error.strip()[-200:]} {gap_error.strip()[-300:]}")
    return failures, checked


SWITCH_EXPECTED_CELLS = {   # 文件头「弄坏开关」那一段，两处要一致
    "copy-research": ["whitelist-only"],
    "no-size-limit": ["over-threshold-refused"],
    "keep-times": ["mtime-now"],
    "gate-tables-always": ["gate-tables-opt-in"],
    "no-size-report": ["size-reported-before-copy"],
    "no-refresh": ["refresh-with-target-dir"],
    "no-root-check": ["refuses-non-root"],
    "no-extra-warning": ["warns-about-extra-top-level"],
    "closure-ignored": ["thread-registry-copied", "whitelist-only", "path-dependencies-copied", "path-dependency-outside-root-refused", "test-inputs-opt-in",
                        "declared-entries-copied"],
    "test-inputs-always": ["test-inputs-opt-in"],
    "declared-ignored": ["declared-entries-copied"],
    "gate-tables-closure-skipped": ["gate-tables-closure-copied"],
}


def run_cells_in_scratch():
    root = tempfile.mkdtemp(prefix="minimal-copy-selftest-")
    try:
        return selftest_cells(root)
    finally:
        shutil.rmtree(root, ignore_errors=True)


def selftest():
    failures, checked = run_cells_in_scratch()
    if failures:
        print(f"  ✗ minimal-copy.py 自证 {len(failures)} 格不对（查了 {checked} 格）：{'、'.join(failures)}")
        print(f"  → 怎么办：照上面逐格改 measure / copy_tree / execute；{BREAK_VARIABLE} 设着的话本来就该红")
        return 1
    # 开关轮的并发：每台默认重活线程数（scripts/hosts/heavy_threads.py，登记位 .claude/heavy-thread-default），再按开关条数夹；取不了就拒，不退回按核数算
    sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
    import heavy_threads
    try:
        switch_round_threads = heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 弄坏开关轮取不了每台默认重活线程数：{error}")
        print(f"  → 怎么办：{error.howto}；不退回按核数算")
        return 2
    jobs = max(1, min(len(SWITCH_EXPECTED_CELLS), switch_round_threads))

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
    print(f"  ✓ minimal-copy.py 自证通过（查了 {checked} 格：只拷名单里的东西不拷 research/ 与各层编译目录、每台默认重活线程数的登记文件拷进副本、"
          "合计超阈值拒绝且副本里一个文件都不写并点名最大的文件、"
          "拷前先报大小、拷过去的与内容变了的是此刻的修改时刻而没变的不动、门禁表要 --with-gate-tables 才拷、给了 --target-dir 就刷源码戳、"
          f"源树不是仓根退 {EXIT_USAGE}、副本顶层有名单之外的项报警告、path 依赖逐层拷进副本而指到仓根之外的拒绝、测试读的仓根文件要 --with-test-inputs 才拷、"
          f"--with 声明的脚本连它 import 的一起拷而不写为什么退 {EXIT_USAGE}、门禁表里的脚本 source 的一起拷而缺的那一份点名拒绝；"
          f"弄坏开关 {len(results)} 个逐个打开都红在点名的格）")
    return 0


def main(arguments):
    if os.environ.get(BREAK_VARIABLE):
        print(f"  ✗ 弄坏开关 {BREAK_VARIABLE} 设在 --selftest 之外，什么都没做", file=sys.stderr)
        print(f"  → 怎么办：去掉 {BREAK_VARIABLE} 再跑；它只给 minimal-copy.py --selftest 证红用", file=sys.stderr)
        return EXIT_USAGE
    return execute(arguments)


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if sys.argv[1:] == ["--selftest-cells"]:
        cell_failures, _ = run_cells_in_scratch()
        sys.exit(1 if cell_failures else 0)
    sys.exit(main(sys.argv[1:]))
