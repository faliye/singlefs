"""full_table_run.py 的单测：读法 10 的键（V13：键要随实际改坏用的变异行变）、5.6 的哨兵（V14：库里的结论被改坏、哨兵要判出不同）、复用三问（自证那一格原样调）、
复用账、分片、存进库那一份的打包、T3 的取舍；读法 1 的拷树与记录、T3 的写法（write_t3）、Z1、Z2 与 Z3、Z4、Z6、S1–S7、K2 与 K3、F3、A7 各条判法
（登记落地清单里上一段写「无用例」的那几行）。不起 cargo、编译、装置、服务：判定库是内存里的假库，装置、逐行驱动、E171 比对器的函数用 mock 换成假的，
导出目录是每条用例自己在临时目录里造的小文件。跑法：在本目录 `python3 -B -m unittest test_full_table_run`。"""

from __future__ import annotations

import gzip
import json
import os
import re
import tempfile
import unittest
from unittest import mock

import full_table_run as run

TREE = run.TreeInputs("a" * 64, "b" * 64, run.DEVICE_ARGUMENTS["s1"], run.DEVICE_FEATURES, "rustc 1", "gpu", (("x.py", "c" * 64),), "d" * 64)


def silent(_line: str) -> None:
    return None


class KeyTests(unittest.TestCase):
    def test_the_key_changes_with_the_mutation_line_actually_used(self):
        """V13：只差改坏用的变异行，键就不同（不同就不复用）。"""
        self.assertNotEqual(run.row_key(TREE, run.RowInput(15, "line", "1" * 64)), run.row_key(TREE, run.RowInput(15, "line", "2" * 64)))

    def test_the_key_changes_with_the_list_line_the_snapshot_and_the_route(self):
        row = run.RowInput(15, "line", "1" * 64)
        base = run.row_key(TREE, row)
        self.assertNotEqual(base, run.row_key(TREE, run.RowInput(15, "line-changed", "1" * 64)))
        for changed in (dict(snapshot_digest="e" * 64), dict(judging_route="cpu"), dict(o3_freeze_digest="f" * 64), dict(mutations_table_sha256="9" * 64)):
            self.assertNotEqual(base, run.row_key(run.dataclasses.replace(TREE, **changed), row), changed)

    def test_a_row_that_needs_a_substitution_is_never_reused(self):
        store = run.MemoryStore()
        row = run.RowInput(15, "line", "needs_substitution")
        store.write(run.library_relative(run.row_key(TREE, row)), b"old")
        plan = run.plan_reuse(TREE, [row], store)
        self.assertEqual((plan.fresh, plan.reused, plan.sentinels), ((15,), (), ()))


class SentinelTests(unittest.TestCase):
    def test_the_sentinel_count_follows_five_point_six(self):
        self.assertEqual([run.sentinel_count(count) for count in (0, 1, 2, 50, 100, 101, 278)], [0, 1, 2, 2, 2, 3, 6])

    def test_sentinels_are_drawn_from_the_reused_rows_and_rerun(self):
        store = run.MemoryStore()
        rows = run.selftest_rows()
        for row in rows:
            store.write(run.library_relative(run.row_key(TREE, row)), b"verdict")
        plan = run.plan_reuse(TREE, rows, store)
        self.assertEqual(len(plan.sentinels), 2)
        self.assertTrue(set(plan.sentinels) <= set(plan.fresh))
        self.assertFalse(set(plan.sentinels) & set(plan.reused))
        self.assertEqual(len(plan.reused) + len(plan.sentinels), len(rows))

    def test_a_corrupted_library_entry_is_caught_by_a_sentinel(self):
        """V14 与作废 Z7：自证那一格原样调。"""
        self.assertTrue(run.cell_sentinel_detects_a_corrupted_library_entry(printer=silent))


class ReuseTests(unittest.TestCase):
    def test_reuse_three_questions(self):
        """复用三问：头一趟全现判、第二趟全复用（哨兵算强制现判）、闭包外改动全复用、改一行的名单行文只现判那一行。"""
        self.assertTrue(run.cell_reuse_three_questions(printer=silent))

    def test_the_reuse_account(self):
        self.assertTrue(run.cell_reuse_account(printer=silent))


class PieceTests(unittest.TestCase):
    def test_rows_alternate_between_two_shards(self):
        self.assertEqual(run.split_into_shards((1, 2, 3, 4, 5)), [[1, 3, 5], [2, 4]])
        self.assertEqual(run.split_into_shards(()), [[], []])

    def test_a_packed_row_leaves_out_the_three_bulky_files_and_unpacks_the_same(self):
        with tempfile.TemporaryDirectory() as root:
            row = os.path.join(root, "row-15")
            os.makedirs(os.path.join(row, "dump", "new_pool_file_creation"))
            for relative, content in (("row-result.tsv", "row_number\t15\n"), ("o3.out", "E7RESULT name=done emitted=1\n"),
                                      ("dump/new_pool_file_creation/states.tsv", "1\t2\n"), ("dump/new_pool_file_creation/base-sectors.bin", "big")):
                with open(os.path.join(row, relative), "w", encoding="utf-8") as handle:
                    handle.write(content)
            packed = run.pack_row_directory(row)
            self.assertEqual(packed, run.pack_row_directory(row), "同一份内容同一串字节")
            run.unpack_row_directory(packed, os.path.join(root, "back"))
            found = sorted(os.path.relpath(os.path.join(base, name), os.path.join(root, "back")) for base, _, names in os.walk(os.path.join(root, "back")) for name in names)
        self.assertEqual(found, ["dump/new_pool_file_creation/states.tsv", "o3.out", "row-result.tsv"])

    def test_t3_keeps_the_streams_of_its_segment(self):
        first, second = run.t3_keeps("s1"), run.t3_keeps("s2")
        self.assertTrue(first("name=q1 reading=strict streams=two_streams batch=1 rows=7"))
        self.assertFalse(first("name=q1 reading=strict streams=three_streams batch=1 rows=7"))
        self.assertTrue(second("name=q1 reading=strict streams=three_streams batch=1 rows=7"))
        self.assertTrue(first("name=q2_named_row row=49 ran=false status=missing"))
        self.assertFalse(first("name=q2_named_row row=49 streams=stream_3 touched=true"))
        self.assertTrue(second(f"name=q2_named_row_call row=49 stream={run.STREAM_DIRECTORIES[2]} call=3"))
        self.assertFalse(second(f"name=q2_named_row_call row=49 stream={run.STREAM_DIRECTORIES[0]} call=3"))

    def test_outside_the_task_server_the_entry_refuses(self):
        environment = dict(os.environ)
        environment.pop("TASK_SERVER_SHARD_PHASE", None)
        with mock.patch.dict(os.environ, environment, clear=True):
            self.assertEqual(run.main(["run"]), run.EXIT_USAGE)


def write_file(path: str, text: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def files_under(root: str) -> list[str]:
    return sorted(os.path.relpath(os.path.join(base, name), root) for base, _, names in os.walk(root) for name in names)


def read_bodies(path: str) -> list[str]:
    with open(path, encoding="utf-8") as handle:
        return [line.rstrip("\n").removeprefix("E7RESULT ") for line in handle if line.startswith("E7RESULT ") and " name=done " not in f" {line[9:]}"]


class PlanSnapshotTests(unittest.TestCase):
    """读法 1：拷进计划目录的是 SYNCED_ENTRIES 那几样（不带 target，含没提交的文件；假仓里只造了其中四样），记那一刻的 HEAD 与改动清单的每一行。git 是假的。"""

    def test_the_plan_copies_the_four_entries_without_target_and_records_head_and_every_status_line(self):
        with tempfile.TemporaryDirectory() as root:
            source = os.path.join(root, "repository")
            for relative, text in (("Cargo.toml", "[workspace]\n"), ("Cargo.lock", "lock\n"), (".cargo/config.toml", "[build]\n"),
                                   ("crates/a/src/lib.rs", "pub fn alpha() {}\n"), ("crates/a/src/uncommitted.rs", "pub fn beta() {}\n"),
                                   ("crates/mutations.tsv", "# 表头\n"), ("crates/a/target/release/stale.rs", "old\n"), ("README.md", "说明\n")):
                write_file(os.path.join(source, relative), text)
            asked = []

            def fake_git(*arguments: str) -> str:
                asked.append(arguments)
                return "abc123\n" if arguments[0] == "rev-parse" else " M crates/a/src/lib.rs\n?? crates/a/src/uncommitted.rs\n"

            tree = os.path.join(root, "plan", "tree")
            snapshot = run.take_tree_snapshot(tree, source, fake_git)
            copied = files_under(tree)
        self.assertEqual(copied, [".cargo/config.toml", "Cargo.lock", "Cargo.toml", "crates/a/src/lib.rs", "crates/a/src/uncommitted.rs", "crates/mutations.tsv"])
        self.assertEqual(snapshot.head, "abc123")
        self.assertEqual(snapshot.status_lines, (" M crates/a/src/lib.rs", "?? crates/a/src/uncommitted.rs"))
        self.assertEqual(snapshot.file_count, 4, "快照摘要只收 .rs 与 Cargo.*：两份 Cargo 加两份 .rs")
        # 改动清单罩住拷进来的每一样（登记修订 5、6）：问 git 的路径就是 SYNCED_ENTRIES，绝对值另钉在 RepositoryRootReferenceTests
        self.assertIn(("status", "--porcelain", "--", *run.SYNCED_ENTRIES), asked)
        self.assertIn("prune-server", run.SYNCED_ENTRIES)
        collected = run.CollectedLines()
        run.write_snapshot_status_lines(collected, snapshot)
        self.assertEqual(collected.bodies, ["name=snapshot_status line=_M_crates/a/src/lib.rs", "name=snapshot_status line=??_crates/a/src/uncommitted.rs"])


class PruneServerCopyTests(unittest.TestCase):
    """登记修订 5：crates 经 path 依赖到的仓根目录 prune-server 也拷进计划目录（不带 target），否则拷出的树编不过（停机 S6）。"""

    def test_the_copied_tree_has_the_prune_server_manifest_and_sources_without_its_target(self):
        with tempfile.TemporaryDirectory() as root:
            source = os.path.join(root, "repository")
            for relative, text in (("Cargo.toml", "[workspace]\n"), ("crates/a/src/lib.rs", "pub fn alpha() {}\n"),
                                   ("prune-server/Cargo.toml", "[package]\n"), ("prune-server/src/lib.rs", "pub fn serve() {}\n"),
                                   ("prune-server/target/release/stale.rs", "old\n")):
                write_file(os.path.join(source, relative), text)
            tree = os.path.join(root, "plan", "tree")
            run.copy_tree(tree, source)
            copied = files_under(tree)
        self.assertIn("prune-server/Cargo.toml", copied)
        self.assertIn("prune-server/src/lib.rs", copied)
        self.assertNotIn("prune-server/target/release/stale.rs", copied)

    def test_every_repository_root_directory_a_crate_reaches_by_path_is_copied(self):
        """读真仓：crates/*/Cargo.toml 里每条 `path = "../../<目录>"` 的目录都在 SYNCED_ENTRIES 里（拷树漏一个，计划拷出的树就编不过）。"""
        import re
        reached = set()
        crates_root = os.path.join(run.REPOSITORY_ROOT, "crates")
        for name in sorted(os.listdir(crates_root)):
            manifest = os.path.join(crates_root, name, "Cargo.toml")
            if os.path.isfile(manifest):
                with open(manifest, encoding="utf-8") as handle:
                    reached.update(re.findall(r'path\s*=\s*"\.\./\.\./([^/"]+)', handle.read()))
        self.assertIn("prune-server", reached, "判例：验证包依赖 prune-server 的客户端")
        self.assertEqual(sorted(reached - set(run.SYNCED_ENTRIES)), [])


# ── 登记修订 6：crates 编、跑时读的仓根文件（编译期 include_str! 与运行期从编二进制的那棵树起的脚本，连同它们 source、起的）都要拷 ──
RUST_STRING_LITERAL = re.compile(r'"((?:[^"\\]|\\.)*)"')
PATH_ATTRIBUTE_BEFORE_THE_LITERAL = re.compile(r'#\[path\s*=\s*$')
INCLUDE_BEFORE_THE_LITERAL = re.compile(r'\binclude(?:_str|_bytes)?!\(\s*$')
TEST_MODULE_HEADER = re.compile(r'(?:pub(?:\(crate\))? )?mod \w+ \{\s*$')
# 包目录下编装置二进制时不编的几个目录（集成测试、基准、示例）与编译目录
CRATE_DIRECTORIES_NOT_BUILT_FOR_A_BINARY = ("tests", "benches", "examples", "target")
# shell 脚本里「目录表达式/相对路径」：$(dirname "${BASH_SOURCE[0]}")/…、$…_directory/…、$(cd … && pwd -P)/…、${…%/*}/…
SHELL_REFERENCE_FROM_THE_SCRIPT_DIRECTORY = re.compile(r'(?:\}|\)|_directory|_DIRECTORY)/((?:\.\./)*[\w.\-]+(?:/[\w.\-]+)*\.(?:sh|py))\b')
PYTHON_IMPORT = re.compile(r'^(?:import|from)\s+([\w.]+)')


def lines_inside_test_modules(lines: list[str]) -> set[int]:
    """顶格 `#[cfg(test)]`、下一行（跳过别的属性行）是 `mod <名> {` 的模块：从属性那一行到它顶格的 `}` 那一行（行号从 0 数）。"""
    inside: set[int] = set()
    line_index = 0
    while line_index < len(lines):
        if lines[line_index].rstrip() == "#[cfg(test)]":
            following = line_index + 1
            while following < len(lines) and lines[following].startswith("#["):
                following += 1
            if following < len(lines) and TEST_MODULE_HEADER.match(lines[following]):
                closing = following + 1
                while closing < len(lines) and lines[closing].rstrip() != "}":
                    closing += 1
                inside.update(range(line_index, closing + 1))
                line_index = closing + 1
                continue
        line_index += 1
    return inside


def repository_root_paths_named_outside_test_code(repository_root: str) -> dict[str, list[str]]:
    """crates/<包>/ 下非测试代码（不在 tests、benches、examples 里，不在 #[cfg(test)] 模块里，注释行不算；build.rs 算）的字符串字面量里，
    以 `../` 起头（或 `/../` 起头，concat!(env!("CARGO_MANIFEST_DIR"), "/../…") 那种）的相对路径解到 crates/ 之外的：
    直接接在 include!、include_str!、include_bytes! 后面的按这份文件所在目录解，别的按包目录解（CARGO_MANIFEST_DIR 起算）；#[path = …] 不算。
    交回 {相对仓根的路径: [出处 文件:行]}。只认字面的 `../`：`.parent()` 往上走再拼的路径（build.rs 读仓根 Cargo.toml、Cargo.lock 那种）认不出。"""
    found: dict[str, list[str]] = {}
    crates_directory = os.path.join(repository_root, "crates")
    for crate_name in sorted(os.listdir(crates_directory)):
        crate_root = os.path.join(crates_directory, crate_name)
        if not os.path.isfile(os.path.join(crate_root, "Cargo.toml")):
            continue
        for directory, subdirectories, file_names in os.walk(crate_root):
            subdirectories[:] = sorted(name for name in subdirectories
                                       if name != "target" and not (directory == crate_root and name in CRATE_DIRECTORIES_NOT_BUILT_FOR_A_BINARY))
            for file_name in sorted(name for name in file_names if name.endswith(".rs")):
                path = os.path.join(directory, file_name)
                with open(path, encoding="utf-8", errors="replace") as handle:
                    lines = handle.read().splitlines()
                test_lines = lines_inside_test_modules(lines)
                for line_index, line in enumerate(lines):
                    if line_index in test_lines or line.lstrip().startswith("//"):
                        continue
                    for literal in RUST_STRING_LITERAL.finditer(line):
                        value = literal.group(1)
                        relative = value[1:] if value.startswith("/../") else value
                        before = line[:literal.start()]
                        if not relative.startswith("../") or PATH_ATTRIBUTE_BEFORE_THE_LITERAL.search(before):
                            continue
                        base = directory if INCLUDE_BEFORE_THE_LITERAL.search(before) else crate_root
                        resolved = os.path.relpath(os.path.normpath(os.path.join(base, relative)), repository_root)
                        if resolved == "crates" or resolved.startswith("crates" + os.sep):
                            continue
                        found.setdefault(resolved, []).append(f"{os.path.relpath(path, repository_root)}:{line_index + 1}")
    return found


def files_a_shell_script_sources_or_runs(repository_root: str, script: str) -> dict[str, str]:
    """从仓内一份 shell 脚本起，传递地收它 source 或起的仓内文件：shell 脚本非注释行里的「目录表达式/相对路径.sh 或 .py」按这份脚本所在目录解，
    python 文件顶格的 import 按同目录的同名 .py 解；仓里有这份文件的才收（解不到的是别的写法拼的退路，例如 git common-dir 那一处）。
    交回 {相对仓根的路径: 出处 文件:行}（起点自己的出处是空串）。"""
    found = {script: ""}
    pending = [script]
    while pending:
        current = pending.pop()
        path = os.path.join(repository_root, current)
        with open(path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().splitlines()
        if current.endswith(".py"):
            named = [(line_index, os.path.join(os.path.dirname(current), match.group(1).split(".")[0] + ".py"))
                     for line_index, line in enumerate(lines) if (match := PYTHON_IMPORT.match(line))]
        else:
            named = [(line_index, os.path.relpath(os.path.normpath(os.path.join(os.path.dirname(path), reference.group(1))), repository_root))
                     for line_index, line in enumerate(lines) if not line.lstrip().startswith("#")
                     for reference in SHELL_REFERENCE_FROM_THE_SCRIPT_DIRECTORY.finditer(line)]
        for line_index, target in named:
            if target.startswith("..") or target in found or not os.path.isfile(os.path.join(repository_root, target)):
                continue
            found[target] = f"{current}:{line_index + 1}"
            pending.append(target)
    return found


def paths_not_covered_by(paths, entries) -> list[str]:
    """不是某一项、也不在某一项（目录）底下的那几条路径（相对仓根）。"""
    return sorted(path for path in paths if not any(path == entry or path.startswith(entry + "/") for entry in entries))


class RepositoryRootReferenceTests(unittest.TestCase):
    """登记修订 6：计划拷树与线道树同步罩住 crates 编、跑时读的仓根文件（任务 5606 第 0 片停在 S6：拷出的树里没有 .claude/heavy-thread-default）。"""

    def test_every_repository_root_path_the_crates_name_outside_test_code_is_copied_and_synchronized(self):
        """读真仓：crates 非测试代码里 `../` 起头的字面路径解到仓根下的，都在两份 SYNCED_ENTRIES 罩住的范围里。"""
        import current_tree_row_driver as driver
        named = repository_root_paths_named_outside_test_code(run.REPOSITORY_ROOT)
        # 判例（免得扫法失效时空过）：编译期 include_str! 那一份与运行期起的多机配置判法都扫得到
        self.assertIn(".claude/heavy-thread-default", named)
        self.assertIn("scripts/crash/crash-point-replay-shard-configuration-check.sh", named)
        self.assertEqual(paths_not_covered_by(named, run.SYNCED_ENTRIES), [], named)
        self.assertEqual(paths_not_covered_by(named, driver.SYNCED_ENTRIES), [], named)

    def test_every_file_the_named_shell_scripts_source_or_run_is_copied_and_synchronized(self):
        """读真仓：上一条扫到的 shell 脚本传递地 source、起的仓内文件，都在两份 SYNCED_ENTRIES 罩住的范围里。"""
        import current_tree_row_driver as driver
        named = repository_root_paths_named_outside_test_code(run.REPOSITORY_ROOT)
        reached: dict[str, str] = {}
        for script in sorted(path for path in named if path.endswith(".sh")):
            reached.update(files_a_shell_script_sources_or_runs(run.REPOSITORY_ROOT, script))
        # 判例：多机配置判法 source 的两份、preflight 那一层 source 的规范副本与它起的 preflight.py 都收得到
        for expected in ("scripts/hosts/peer-host-lib.sh", ".claude/scripts/preflight.sh", ".claude/singlefs-ai-sop/scripts/preflight.sh",
                         ".claude/singlefs-ai-sop/scripts/preflight.py"):
            self.assertIn(expected, reached)
        self.assertEqual(paths_not_covered_by(reached, run.SYNCED_ENTRIES), [], reached)
        self.assertEqual(paths_not_covered_by(reached, driver.SYNCED_ENTRIES), [], reached)

    def test_every_synced_entry_is_in_the_repository_and_the_plan_and_the_lanes_copy_the_shared_closure(self):
        """两份 SYNCED_ENTRIES 都是 scripts/hosts/lib_repository_copy.py 在本仓上现算的那一份闭包，每一条在仓里都在。"""
        import current_tree_row_driver as driver
        self.assertEqual([entry for entry in run.SYNCED_ENTRIES if not os.path.exists(os.path.join(run.REPOSITORY_ROOT, entry))], [])
        self.assertEqual(sorted(run.SYNCED_ENTRIES), sorted(driver.SYNCED_ENTRIES))
        self.assertEqual(tuple(run.SYNCED_ENTRIES), run.lib_repository_copy.copy_closure(run.REPOSITORY_ROOT).entries)
        self.assertNotIn("multi-host.env", run.SYNCED_ENTRIES, "多机配置不进树，经 SINGLEFS_MULTI_HOST_CONFIG 显式交")

    def test_the_entry_hands_the_main_configuration_to_what_it_starts(self):
        """计划树、线道树都不是 git 工作树：入口一开头就把主工作区那一份多机配置的绝对路径交给 SINGLEFS_MULTI_HOST_CONFIG，之后起的子进程都继承。"""
        environment = {name: value for name, value in os.environ.items()
                       if name not in ("SINGLEFS_MULTI_HOST_CONFIG", "TASK_SERVER_SHARD_PHASE", run.BREAK_VARIABLE)}
        with mock.patch.dict(os.environ, environment, clear=True), mock.patch("sys.stderr"):
            code = run.main(["run"])
            handed = os.environ.get("SINGLEFS_MULTI_HOST_CONFIG")
        self.assertEqual(code, run.EXIT_USAGE)
        self.assertEqual(handed, run.lib_repository_copy.configuration_path(run.REPOSITORY_ROOT, {}))

    def test_the_crates_scan_leaves_out_test_code_comments_and_module_paths_and_resolves_includes_by_their_file(self):
        """扫法本身分得出：include_str! 接字面量按文件目录解（解回 crates 里不收），concat!(env!("CARGO_MANIFEST_DIR"), …) 与 .join 按包目录解，
        #[path]、注释、#[cfg(test)] 模块、tests/ 下的不收，模块之后的照收，build.rs 照收。"""
        with tempfile.TemporaryDirectory() as root:
            write_file(os.path.join(root, "crates", "alpha", "Cargo.toml"), "[package]\n")
            write_file(os.path.join(root, "crates", "alpha", "src", "lib.rs"), "\n".join((
                'pub const REGISTERED: &str = include_str!(concat!(',
                '    env!("CARGO_MANIFEST_DIR"),',
                '    "/",',
                '    "../../.claude/registered-number"',
                '));',
                'pub fn script() -> std::path::PathBuf { std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/check.sh") }',
                'pub const BESIDE: &str = include_str!("../../sibling/data.txt");',
                '// 注释里的 "../../scripts/in-a-comment.sh" 不算',
                '#[path = "../../beta/src/shared.rs"]',
                'mod shared;',
                '#[cfg(test)]',
                'mod tests {',
                '    const ONLY_FOR_TESTS: &str = "../../litmus/only-for-tests.litmus";',
                '}',
                'pub const AFTER_THE_TESTS: &str = "/../../.cargo/after-the-tests.toml";',
            )) + "\n")
            write_file(os.path.join(root, "crates", "alpha", "tests", "integration.rs"), 'const IN_AN_INTEGRATION_TEST: &str = "../../research/x";\n')
            write_file(os.path.join(root, "crates", "alpha", "build.rs"), 'fn main() { let _ = std::fs::read("../../Cargo.lock"); }\n')
            named = repository_root_paths_named_outside_test_code(root)
        self.assertEqual(sorted(named), [".cargo/after-the-tests.toml", ".claude/registered-number", "Cargo.lock", "scripts/check.sh"])
        self.assertEqual(named[".claude/registered-number"], ["crates/alpha/src/lib.rs:4"])

    def test_the_shell_follow_collects_sourced_and_run_files_transitively_and_skips_comments_and_missing_paths(self):
        with tempfile.TemporaryDirectory() as root:
            write_file(os.path.join(root, "scripts", "crash", "entry.sh"), "\n".join((
                '# source "$(dirname "${BASH_SOURCE[0]}")/../in-a-comment.sh"',
                'source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/shim.sh"',
                'entry_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"',
                'source "$entry_directory/../hosts/library.sh"',
                'source "${common%/*}/not-there.sh"',
            )) + "\n")
            write_file(os.path.join(root, "scripts", "in-a-comment.sh"), "\n")
            write_file(os.path.join(root, "scripts", "hosts", "library.sh"), "\n")
            write_file(os.path.join(root, ".claude", "scripts", "shim.sh"), 'library="$shim_directory/../tool/scripts/program.sh"\nsource "$library"\n')
            write_file(os.path.join(root, ".claude", "tool", "scripts", "program.sh"), 'PROGRAM="$(cd "$here" && pwd -P)/program.py"\n')
            write_file(os.path.join(root, ".claude", "tool", "scripts", "program.py"), "import os\nimport helper\n")
            write_file(os.path.join(root, ".claude", "tool", "scripts", "helper.py"), "\n")
            reached = files_a_shell_script_sources_or_runs(root, "scripts/crash/entry.sh")
        self.assertEqual(sorted(reached), [".claude/scripts/shim.sh", ".claude/tool/scripts/helper.py", ".claude/tool/scripts/program.py",
                                           ".claude/tool/scripts/program.sh", "scripts/crash/entry.sh", "scripts/hosts/library.sh"])
        self.assertEqual(reached[".claude/tool/scripts/helper.py"], ".claude/tool/scripts/program.py:2")

    def test_the_copied_tree_has_the_repository_root_files_under_their_directories_and_nothing_beside_them(self):
        """单个文件的条目拷进它在树里的上级目录（rsync 不建缺的上级目录）；同目录里没点名的文件不拷。"""
        with tempfile.TemporaryDirectory() as root:
            source = os.path.join(root, "repository")
            for relative in ("Cargo.toml", ".claude/heavy-thread-default", "scripts/crash/crash-point-replay-shard-configuration-check.sh",
                             ".claude/singlefs-ai-sop/scripts/preflight.py", ".claude/singlefs-ai-sop/scripts/not-named.py", "scripts/hosts/other.sh"):
                write_file(os.path.join(source, relative), "x\n")
            tree = os.path.join(root, "plan", "tree")
            run.copy_tree(tree, source)
            copied = files_under(tree)
        self.assertEqual(copied, [".claude/heavy-thread-default", ".claude/singlefs-ai-sop/scripts/preflight.py", "Cargo.toml",
                                  "scripts/crash/crash-point-replay-shard-configuration-check.sh"])


class FakeStream:
    def present(self) -> bool:
        return True


class FakeRun:
    def __init__(self, path: str):
        self.path = path
        self.streams = {name: FakeStream() for name in run.STREAM_DIRECTORIES}


class FakeTouch:
    def __init__(self, touched: bool):
        self.value = touched

    def touched(self) -> bool:
        return self.value


# 七行各自的（status、device_exit、触达否）：只有跑成（status=ok、装置退 0 或 3）且触达的行出 q1_row
T3_ROWS = {15: ("ok", "0", True), 23: ("ok", "3", True), 42: ("compile_failed", "-", True), 242: ("ok", "101", True), 269: ("ok", "0", False),
           270: ("ok", "0", True), 271: ("ok", "0", True)}


def row_number_of(path: str) -> int:
    return next(int(part.removeprefix("row-")) for part in path.split(os.sep) if part.startswith("row-"))


class T3Tests(unittest.TestCase):
    """读法 7 的 T3：E171 第一段比对器那几个函数换成假的（各写几行、记下被调的行与批），只看 write_t3 怎么取、怎么调。"""

    def write_t3_with_fakes(self, segment: str) -> tuple[list[str], list[tuple[int, int]]]:
        import collusion_matrix
        import run_directory
        import segment_one_comparator as segment_one
        called: list[tuple[int, int]] = []

        def q1_lines(lines, _baseline, _rows_root):
            for body in ("name=q1 reading=strict streams=two_streams batch=1 rows=7", "name=q1 reading=strict streams=three_streams batch=1 rows=7",
                         f"name=q1_row row=15 stream={run.STREAM_DIRECTORIES[2]} batch=1"):
                lines.result(body)
            return {"q1": "jia"}

        def row_column_lines(lines, number, _run, _o3, batch, streams):
            called.append((number, batch))
            lines.result(f"name=q1_row row={number} stream={streams[0]} batch={batch}")

        def q2_named_row_lines(lines, _baseline, _rows_root, _stability):
            lines.result("name=q2_named_row row=49 streams=streams_1_2 touched=true")
            lines.result("name=q2_named_row row=49 streams=stream_3 touched=true")

        def read_row_result(row_directory):
            status, device_exit, _ = T3_ROWS[row_number_of(row_directory)]
            return {"status": status, "device_exit": device_exit}

        def touch_report(_baseline, row_run):
            return FakeTouch(T3_ROWS[row_number_of(row_run.path)][2])

        with tempfile.TemporaryDirectory() as root, \
                mock.patch.object(segment_one, "q1_lines", q1_lines), mock.patch.object(segment_one, "row_column_lines", row_column_lines), \
                mock.patch.object(segment_one, "q2_named_row_lines", q2_named_row_lines), \
                mock.patch.object(segment_one, "q3_lines", lambda _lines, _baseline, _rows_root: "yi"), \
                mock.patch.object(segment_one, "read_row_result", read_row_result), mock.patch.object(segment_one, "read_o3", lambda _path: None), \
                mock.patch.object(run_directory, "load_run_directory", FakeRun), mock.patch.object(collusion_matrix, "touch_report", touch_report):
            path = os.path.join(root, "t3.out")
            run.write_t3(path, segment, os.path.join(root, "baseline"), os.path.join(root, "rows"), frozenset())
            return read_bodies(path), called

    def test_the_first_segment_takes_the_two_stream_lines_and_the_row_columns_of_the_touched_rows_that_ran(self):
        bodies, called = self.write_t3_with_fakes("s1")
        self.assertEqual(called, [(number, batch) for number in (15, 23, 270, 271) for batch in (1, 2)],
                         "编不过（第 42 行）、装置退 101（第 242 行）、未触达（第 269 行）的不出 q1_row")
        self.assertIn("name=q1 reading=strict streams=two_streams batch=1 rows=7", bodies)
        self.assertNotIn("name=q1 reading=strict streams=three_streams batch=1 rows=7", bodies)
        self.assertFalse(any(f"stream={run.STREAM_DIRECTORIES[2]}" in body for body in bodies), "流三那几行第一段不比")
        self.assertIn("name=q2_named_row row=49 streams=streams_1_2 touched=true", bodies)
        self.assertNotIn("name=q2_named_row row=49 streams=stream_3 touched=true", bodies)
        self.assertEqual(bodies[-1], "name=verdict q3=yi")

    def test_the_second_segment_takes_only_the_stream_three_lines_and_no_row_columns(self):
        bodies, called = self.write_t3_with_fakes("s2")
        self.assertEqual(called, [])
        self.assertIn("name=q1 reading=strict streams=three_streams batch=1 rows=7", bodies)
        self.assertNotIn("name=q1 reading=strict streams=two_streams batch=1 rows=7", bodies)
        self.assertIn(f"name=q1_row row=15 stream={run.STREAM_DIRECTORIES[2]} batch=1", bodies)
        self.assertEqual(bodies[-1], "name=verdict q1=jia")


STREAM = run.STREAM_DIRECTORIES[0]


def make_export(root: str, impl_timing: str = "12", ops: str = "ops\n", impl_verdict: str = "green") -> str:
    """一份假导出：一条流，Z1_FILES 每份一行字，impl-verdicts.tsv 末列是耗时。"""
    for name in run.Z1_FILES:
        write_file(os.path.join(root, STREAM, name), ops if name == "ops.tsv" else f"{name}\n")
    write_file(os.path.join(root, STREAM, "impl-verdicts.tsv"), f"state\t{impl_verdict}\t{impl_timing}\n")
    return root


def no_return_value_differences(_first: str, _second: str, _streams: list[str]) -> list[str]:
    return []


class SecondBaselineAndExportTests(unittest.TestCase):
    """作废 Z1 与停机 S2 的逐文件比（导出是临时目录里的小文件；call-return-values.tsv 那一比换成假的）。"""

    def test_two_baselines_that_differ_only_in_the_timing_column_are_the_same(self):
        with tempfile.TemporaryDirectory() as root:
            first, second = make_export(os.path.join(root, "first")), make_export(os.path.join(root, "second"), impl_timing="99")
            self.assertEqual(run.second_baseline_differences(first, second, [STREAM], no_return_value_differences), [])

    def test_a_second_baseline_with_another_write_stream_or_verdict_differs(self):
        with tempfile.TemporaryDirectory() as root:
            first = make_export(os.path.join(root, "first"))
            other_ops = make_export(os.path.join(root, "ops"), ops="other\n")
            other_verdict = make_export(os.path.join(root, "verdict"), impl_verdict="red")
            self.assertEqual(run.second_baseline_differences(first, other_ops, [STREAM], no_return_value_differences), [f"{STREAM}/ops.tsv"])
            self.assertEqual(run.second_baseline_differences(first, other_verdict, [STREAM], no_return_value_differences), [f"{STREAM}/impl-verdicts.tsv"])

    def test_the_return_values_without_the_unstable_objects_are_compared_too(self):
        with tempfile.TemporaryDirectory() as root:
            first, second = make_export(os.path.join(root, "first")), make_export(os.path.join(root, "second"))
            found = run.second_baseline_differences(first, second, [STREAM], lambda _one, _other, _streams: [f"{STREAM}/call-return-values.tsv:2"])
        self.assertEqual(found, [f"{STREAM}/call-return-values.tsv:2"])

    def test_the_e170_export_is_compared_on_the_eight_files_and_a_failed_dump_is_not_the_same(self):
        with tempfile.TemporaryDirectory() as root:
            baseline = make_export(os.path.join(root, "baseline"))
            same = make_export(os.path.join(root, "same"), impl_timing="77")
            other = make_export(os.path.join(root, "other"), ops="other\n")
            self.assertEqual(run.export_against_the_baseline(0, baseline, same, [STREAM]), [])
            self.assertEqual(run.export_against_the_baseline(0, baseline, other, [STREAM]), [f"{STREAM}/ops.tsv"])
            self.assertEqual(run.export_against_the_baseline(101, baseline, same, [STREAM]), ["dump_failed"])

    def test_the_cross_host_check_runs_only_when_the_shards_landed_on_two_hosts(self):
        """停机 S7：两片在两台、基线与空变异导出逐文件比；在同一台记本次未跑。"""
        with tempfile.TemporaryDirectory() as root:
            for shard in ("zero", "one", "other"):
                make_export(os.path.join(root, shard, "baseline"))
                make_export(os.path.join(root, shard, "empty", "row-0", "dump"), ops="other\n" if shard == "other" else "ops\n")
            zero, one, other = (os.path.join(root, shard) for shard in ("zero", "one", "other"))
            self.assertEqual(run.cross_host_comparison([(0, "本机", zero), (1, "第二台", one)], [STREAM]), ("same", []))
            self.assertEqual(run.cross_host_comparison([(0, "本机", zero), (1, "第二台", other)], [STREAM]), ("differs", [f"empty:{STREAM}/ops.tsv"]))
            self.assertEqual(run.cross_host_comparison([(0, "本机", zero), (1, "本机", other)], [STREAM]), ("not_run_same_host", []))


def make_planned_tree(root: str) -> tuple[dict, str, str]:
    """计划目录里的假树与仓里的一份计划记过的文件；交回 (计划记录, 树, 仓根)。"""
    import row_driver
    tree, repository = os.path.join(root, "tree"), os.path.join(root, "repository")
    write_file(os.path.join(tree, "crates", "a", "src", "lib.rs"), "pub fn alpha() {}\n")
    write_file(os.path.join(tree, "crates", "mutations.tsv"), "# 表头\n")
    write_file(os.path.join(repository, "scripts", "recorded.py"), "VALUE = 1\n")
    record = {"tree_inputs": {"snapshot_digest": row_driver.snapshot_digest(tree)[0],
                              "mutations_table_sha256": run.sha256_file(os.path.join(tree, "crates", "mutations.tsv"))},
              "recorded": {"scripts/recorded.py": run.sha256_file(os.path.join(repository, "scripts", "recorded.py"))}}
    return record, tree, repository


class ShardCheckTests(unittest.TestCase):
    """作废 Z2、Z3：每片开头与收尾核计划记的树、变异表、各文件与冻结的 O3（O3 那一核换成假的）。"""

    def test_an_unchanged_plan_has_no_problem(self):
        with tempfile.TemporaryDirectory() as root:
            record, tree, repository = make_planned_tree(root)
            self.assertEqual(run.shard_check_problems(record, tree, repository, lambda _root: []), [])

    def test_each_kind_of_drift_is_named(self):
        cases = (("tree", "crates/a/src/lib.rs", "snapshot_digest"), ("tree", "crates/mutations.tsv", "mutations_table"),
                 ("repository", "scripts/recorded.py", "scripts/recorded.py"))
        for where, relative, expected in cases:
            with self.subTest(relative=relative), tempfile.TemporaryDirectory() as root:
                record, tree, repository = make_planned_tree(root)
                write_file(os.path.join(tree if where == "tree" else repository, relative), "changed\n")
                self.assertEqual(run.shard_check_problems(record, tree, repository, lambda _root: []), [expected])

    def test_a_frozen_o3_difference_is_named(self):
        with tempfile.TemporaryDirectory() as root:
            record, tree, repository = make_planned_tree(root)
            self.assertEqual(run.shard_check_problems(record, tree, repository, lambda _root: ["executor_core.py"]), ["freeze:executor_core.py"])


class ShardStepTests(unittest.TestCase):
    """各片那一步：核对、编、装置、逐行驱动、A7 都换成假的，只看开头收尾的核对（Z2、Z3）与树编不过（S6）怎么落进 shard.out、退几。"""

    def run_shard(self, checks: list[list[str]], compile_exit: int = 0) -> tuple[int, list[str], mock.MagicMock, mock.MagicMock]:
        def fake_driver(command, _log, _on_poll=None):
            if "--empty-mutation" in command:
                output_root = command[command.index("--output-root") + 1]
                write_file(os.path.join(output_root, "row-0", "row-result.tsv"), "row_number\t0\n")
            return 0

        with tempfile.TemporaryDirectory() as root, mock.patch.dict(os.environ):
            os.environ.pop("CARGO_TARGET_DIR", None)
            plan_directory, output_directory = os.path.join(root, "plan"), os.path.join(root, "shard-1")
            os.makedirs(output_directory)
            write_file(os.path.join(plan_directory, "plan-record.json"),
                       json.dumps({"segment": "s1", "shards": [[], []], "keys": {}, "fresh": [], "reused": [], "sentinels": []}))
            with mock.patch.object(run, "shard_check_problems", side_effect=checks), mock.patch.object(run, "heavy_threads", return_value=12), \
                    mock.patch.object(run, "compile_clean_binaries", return_value=(compile_exit, "/fake/device", "/fake/export")) as compiled, \
                    mock.patch.object(run, "run_device", return_value=0) as device, \
                    mock.patch.object(run, "verdict_fields", return_value={"s1_failed_checks": "0"}), \
                    mock.patch.object(run, "run_driver", side_effect=fake_driver), mock.patch.object(run, "stored_row_differences", return_value=[]):
                exit_code = run.shard_step(plan_directory, output_directory, 1, run.MemoryStore())
            return exit_code, read_bodies(os.path.join(output_directory, "shard.out")), compiled, device

    def test_a_drift_at_the_start_stops_the_shard_before_anything_is_compiled(self):
        exit_code, bodies, compiled, _ = self.run_shard([["snapshot_digest"]])
        self.assertEqual(exit_code, run.EXIT_INVALID)
        self.assertIn("name=shard_check moment=start problems=snapshot_digest", bodies)
        compiled.assert_not_called()

    def test_a_tree_that_does_not_compile_is_stop_clause_s6(self):
        exit_code, bodies, _, device = self.run_shard([[]], compile_exit=101)
        self.assertEqual(exit_code, run.EXIT_INVALID)
        self.assertIn("name=stop clause=S6", bodies)
        device.assert_not_called()

    def test_a_drift_at_the_end_is_checked_after_the_rows_and_fails_the_shard(self):
        exit_code, bodies, _, _ = self.run_shard([[], ["freeze:executor_core.py"]])
        self.assertEqual(exit_code, run.EXIT_INVALID)
        self.assertEqual(bodies[-1], "name=shard_check moment=end problems=freeze:executor_core.py")
        self.assertIn("name=a7 row=0 differing=-", bodies)

    def test_a_clean_shard_checks_both_moments_and_exits_zero(self):
        exit_code, bodies, _, _ = self.run_shard([[], []])
        self.assertEqual(exit_code, 0)
        self.assertEqual([body for body in bodies if body.startswith("name=shard_check ")],
                         ["name=shard_check moment=start problems=-", "name=shard_check moment=end problems=-"])


class NotRunTests(unittest.TestCase):
    """作废 Z6：NOT_RUN 行数合计（文件、.gz、目录逐份数，NOT_RUN_<原因> 与 CRASH_AMPLIFICATION_NOT_RUN 都算），与要数的那几样。"""

    def test_not_run_lines_are_counted_in_files_gzip_files_and_directories(self):
        with tempfile.TemporaryDirectory() as root:
            write_file(os.path.join(root, "device.out"), "E7RESULT name=x\nNOT_RUN_NO_GPU_ADAPTER test=a\n")
            with gzip.open(os.path.join(root, "states.tsv.gz"), "wt", encoding="utf-8") as handle:
                handle.write("CRASH_AMPLIFICATION_NOT_RUN reason=b\nok\n")
            write_file(os.path.join(root, "dump", STREAM, "oracle-verdicts.tsv"), "NOT_RUN\nNOT_RUN\nok\n")
            count = run.not_run_count([os.path.join(root, "device.out"), os.path.join(root, "states.tsv.gz"), os.path.join(root, "dump"),
                                       os.path.join(root, "absent.out")])
        self.assertEqual(count, 4)

    def test_both_baselines_the_empty_mutation_the_controls_and_the_merged_rows_are_counted(self):
        paths = run.not_run_paths([(0, "本机", "/shard-0"), (1, "第二台", "/shard-1")], "/merge/rows")
        for shard in ("/shard-0", "/shard-1"):
            for name in ("baseline.device.out", "baseline", "baseline-second.device.out", "baseline-second", "empty", "controls"):
                self.assertIn(os.path.join(shard, name), paths)
        self.assertEqual(paths[-1], "/merge/rows")


def green_shard_records() -> dict[int, list[dict[str, str]]]:
    def baseline() -> dict[str, str]:
        return {"name": "baseline", "exit": "0", "s1_failed_checks": "0", "s3_o2_judgement_mismatches": "0", "s9_failed_checks": "0"}

    def moments() -> list[dict[str, str]]:
        return [{"name": "shard_check", "moment": "start", "problems": "-"}, {"name": "shard_check", "moment": "end", "problems": "-"}]

    return {0: [baseline(), {"name": "z1_second_baseline", "differing": "-"}, {"name": "s2_export_against_the_baseline", "differing": "-"}, *moments()],
            1: [baseline(), *moments()]}


def green_readings(**changes) -> run.MergeReadings:
    readings = run.MergeReadings(shard_records=green_shard_records(), cross="same", cross_differing=(), r1_problems=(), not_run=0, sentinels=(),
                                 control_failing=0, rows_not_run=(), t1_exit=0, self_check=(), q2_agrees=True, account=(), rebuild_problems=(),
                                 stored_row_problems=())
    return run.dataclasses.replace(readings, **changes)


def verdicts_of(readings: run.MergeReadings) -> dict[str, tuple[bool, str]]:
    return {clause: (holds, detail) for clause, holds, detail in run.clause_checks(readings)}


class ClauseCheckTests(unittest.TestCase):
    """合并那一步的作废、停机、失败条款（clause_checks 纯函数，读数手造）。"""

    def test_green_readings_hold_every_clause(self):
        verdicts = verdicts_of(green_readings())
        self.assertEqual(sorted(verdicts), sorted(["S1", "S3", "S5", "S4", "S2", "S6", "S7", "Z1", "Z2_Z3", "Z4", "Z5", "Z6", "Z7", "F1_F2", "F3", "A5", "A6",
                                                   "A7", "reuse_account"]))
        self.assertEqual([clause for clause, (holds, _) in verdicts.items() if not holds], [])

    def test_a_baseline_verdict_field_that_is_not_zero_or_missing_fails_its_own_clause(self):
        """S1、S3、S5 各读基线判决行自己的字段；S4 读基线退出码（退 3 而 S1、S3、S5 都过）。"""
        for key, clause in (("s1_failed_checks", "S1"), ("s3_o2_judgement_mismatches", "S3"), ("s9_failed_checks", "S5")):
            with self.subTest(clause=clause):
                records = green_shard_records()
                records[1][0][key] = "2"
                verdicts = verdicts_of(green_readings(shard_records=records))
                self.assertEqual(verdicts[clause], (False, "1:2"))
                self.assertEqual([name for name, (holds, _) in verdicts.items() if not holds], [clause])
                del records[1][0][key]
                self.assertEqual(verdicts_of(green_readings(shard_records=records))[clause], (False, "1:missing"))
        records = green_shard_records()
        records[0][0]["exit"] = "3"
        self.assertEqual(verdicts_of(green_readings(shard_records=records))["S4"], (False, "0:3,1:0"))

    def test_the_export_comparison_and_the_second_baseline_fail_when_a_file_differs_or_the_line_is_missing(self):
        for name, clause in (("s2_export_against_the_baseline", "S2"), ("z1_second_baseline", "Z1")):
            with self.subTest(clause=clause):
                records = green_shard_records()
                next(entry for entry in records[0] if entry["name"] == name)["differing"] = f"{STREAM}/ops.tsv"
                self.assertEqual(verdicts_of(green_readings(shard_records=records))[clause], (False, f"{STREAM}/ops.tsv"))
                records[0] = [entry for entry in records[0] if entry["name"] != name]
                self.assertEqual(verdicts_of(green_readings(shard_records=records))[clause], (False, "missing"))

    def test_a_stop_line_in_any_shard_fails_s6(self):
        records = green_shard_records()
        records[1].append({"name": "stop", "clause": "S6"})
        self.assertEqual(verdicts_of(green_readings(shard_records=records))["S6"], (False, "1:S6"))

    def test_two_hosts_that_judge_differently_fail_s7_and_one_host_is_not_a_failure(self):
        self.assertEqual(verdicts_of(green_readings(cross="differs", cross_differing=("empty:x",)))["S7"], (False, "empty:x"))
        self.assertTrue(verdicts_of(green_readings(cross="not_run_same_host"))["S7"][0])

    def test_a_drift_or_a_missing_moment_fails_z2_z3(self):
        records = green_shard_records()
        records[1][-1]["problems"] = "freeze:executor_core.py"
        self.assertEqual(verdicts_of(green_readings(shard_records=records))["Z2_Z3"], (False, "1:end:freeze:executor_core.py"))
        records = green_shard_records()
        records[0] = records[0][:-1]
        self.assertEqual(verdicts_of(green_readings(shard_records=records))["Z2_Z3"], (False, "0:end:not_checked"))

    def test_rows_not_run_today_and_a_failed_comparator_fail_f3(self):
        self.assertEqual(verdicts_of(green_readings(rows_not_run=(42, 99)))["F3"], (False, "rows=42,99 t1_exit=0"))
        self.assertFalse(verdicts_of(green_readings(t1_exit=1))["F3"][0])

    def test_the_rebuild_and_stored_row_problems_fail_z4_and_a7(self):
        self.assertEqual(verdicts_of(green_readings(rebuild_problems=("1:rows_driver:exit_5",)))["Z4"], (False, "1:rows_driver:exit_5"))
        self.assertEqual(verdicts_of(green_readings(stored_row_problems=("0:12:stored_row_differs",)))["A7"], (False, "0:12:stored_row_differs"))


class MergeReadingTests(unittest.TestCase):
    """合并那一步从产物里取读数的几样：F3 的行、Z4 的编前编后核对、A7 的行数与结局。"""

    def test_rows_that_did_not_run_for_a_reason_outside_reading_five_are_f3(self):
        t1 = "\n".join(f"E7RESULT name=row row={number} status=ok ran={ran} category_registration_batch_1={category}"
                       for number, ran, category in ((5, "false", "device_exit_101"), (6, "false", "compile_failed"), (7, "true", "counted"),
                                                     (8, "false", "timeout"), (9, "false", "device_stop"), (10, "false", "self_crash")))
        self.assertEqual(run.rows_not_run_today(t1, [12]), [5, 9, 12])

    def test_a_driver_exit_five_or_a_compiled_row_without_the_rebuild_check_is_z4(self):
        records = {0: [{"name": "empty_mutation", "exit": "0"}, {"name": "rows_driver", "exit": "5"}], 1: [{"name": "rows_driver", "exit": "0"}]}
        t2 = ("name=row_time_limit seconds=1800\nrow row_number=3 status=ok packages_checked_rebuilt=crates/singlefs-core\n"
              "row row_number=4 status=ok mutated_row_name=m4\nrow row_number=5 status=compile_failed\ntable rows=3\n")
        self.assertEqual(run.rebuild_check_problems(records, t2), ["0:rows_driver:exit_5", "row_4:packages_checked_rebuilt_missing"])

    def test_every_shard_reports_a7_for_the_empty_mutation_and_each_of_its_sentinels(self):
        record = {"shards": [[11, 12], [13]]}
        plan = run.ReusePlan({11: "a", 12: "b", 13: "c"}, (11, 12, 13), (), (12,))
        records = {0: [{"name": "a7", "row": "0", "differing": "-"}, {"name": "a7", "row": "12", "differing": "12:stored_row_differs"}],
                   1: []}
        self.assertEqual(run.stored_row_check_problems(records, record, plan), ["0:12:stored_row_differs", "1:a7_lines_0_expected_1"])


class StoredRowTests(unittest.TestCase):
    """第七节 A7：存进库那一份与完整导出在比对器上逐字相同（比对器换成假的，读行目录里的文件）。"""

    def test_the_selftest_cell_is_green_and_tells_a_comparator_reading_the_bulky_files_apart(self):
        self.assertTrue(run.cell_stored_row_matches_the_full_export(printer=silent))

    def test_a_comparator_that_reads_a_bulky_file_sees_the_stored_row_differ(self):
        with tempfile.TemporaryDirectory() as root:
            row_directory = os.path.join(root, "rows", "row-7")
            write_file(os.path.join(row_directory, "row-result.tsv"), "row_number\t7\n")
            write_file(os.path.join(row_directory, "dump", STREAM, "torn-payload.bin"), "payload\n")
            scratch = os.path.join(root, "scratch")
            reads_all = run.stored_row_differences("-", row_directory, 7, scratch, lambda _b, rows_root, number: run.files_of_a_row_directory(rows_root, number, ()))
            reads_stored = run.stored_row_differences("-", row_directory, 7, scratch,
                                                      lambda _b, rows_root, number: run.files_of_a_row_directory(rows_root, number, run.BULKY_DUMP_FILES))
            left_behind = os.path.exists(os.path.join(scratch, "stored-row-7"))
        self.assertEqual((reads_all, reads_stored, left_behind), (["7:stored_row_differs"], [], False))


def call_row(index: int, outcome: str = "ok", model_word: str = "not_compared"):
    from run_directory import CallRow
    return CallRow(index, f"call{index}", 0, 0, outcome, model_word, "-", "-", "-", "-", "-")


def panic_run(calls_of_stream_two: list, exported: bool = True):
    """流一三次调用不 panic、三态；流二照给的调用、两态（态只数个数）。"""
    from run_directory import RunDirectory, StreamRun
    stream_one = StreamRun(run.STREAM_DIRECTORIES[0], True, states=["s", "s", "s"], calls=[call_row(0), call_row(1), call_row(2, model_word="agrees")],
                           status={"ending": "completed"})
    stream_two = StreamRun(run.PANIC_CONTROL_STREAM, exported, states=["s", "s"], calls=calls_of_stream_two, status={"ending": "stopped_by_a_panic"})
    return RunDirectory("/fake", {run.STREAM_DIRECTORIES[0]: stream_one, run.PANIC_CONTROL_STREAM: stream_two,
                                  run.STREAM_DIRECTORIES[2]: StreamRun(run.STREAM_DIRECTORIES[2], False)})


# 阳性对照 panic 的正形：流二取号、暖机不比，第 2 次调用 agrees，第 3 次调用 panic，流截在这里
PANIC_AT_THE_THIRD_CALL = [call_row(0), call_row(1), call_row(2, model_word="agrees"), call_row(3, outcome="panic", model_word="self_crash_not_compared")]


class ControlTests(unittest.TestCase):
    """K2、K3（登记 5.2 第一张表与第七节）：装置报的计数、导出现算的三样、阴性对照的数法。E171 比对器的 A1 那两个函数换成假的。"""

    def test_a_device_counted_control_holds_only_at_the_registered_exact_number(self):
        with tempfile.TemporaryDirectory() as root:
            device_out = os.path.join(root, "device.out")
            for lines, expected in ((("o1_crash_judge_red_states=1", "o1_crash_judge_red_states=1"), ("2", True)),
                                    (("o1_crash_judge_red_states=1",), ("1", False)), (("o1_crash_judge_red_states=3",), ("3", False))):
                with self.subTest(lines=lines):
                    write_file(device_out, "".join(f"E7RESULT name=verdict {line}\n" for line in lines))
                    self.assertEqual(run.device_control_judgement("O1-崩溃判官", device_out), expected)
            self.assertEqual(run.device_control_judgement("O2", os.path.join(root, "absent.out")), ("missing", False))

    def test_the_panic_control_holds_when_stream_two_is_cut_at_the_third_call(self):
        observed, holds = run.dump_control_judgement("panic", None, panic_run(PANIC_AT_THE_THIRD_CALL))
        self.assertEqual((observed, holds), ("exported=true:panic_at=3:last_call=3:judged=1:states=5", True))

    def test_the_panic_control_fails_off_the_third_call_or_when_the_stream_goes_on_or_is_not_exported(self):
        cases = {"panic at the second call": (PANIC_AT_THE_THIRD_CALL[:2] + [call_row(2, outcome="panic")], True),
                 "calls after the panic": (PANIC_AT_THE_THIRD_CALL + [call_row(4)], True),
                 "stream two not exported": (PANIC_AT_THE_THIRD_CALL, False)}
        for label, (calls, exported) in cases.items():
            with self.subTest(label):
                self.assertFalse(run.dump_control_judgement("panic", None, panic_run(calls, exported))[1])

    def run_negative_controls(self, empty_dump: bool, touched: bool) -> tuple[int, list[str]]:
        import run_directory
        import segment_one_comparator as segment_one
        collected = run.CollectedLines()
        with tempfile.TemporaryDirectory() as root, mock.patch.object(segment_one, "a1_lines", return_value=0), \
                mock.patch.object(segment_one, "read_o3", return_value=None), mock.patch.object(segment_one, "empty_mutation_a1", return_value=(0, touched)), \
                mock.patch.object(run_directory, "load_run_directory", FakeRun):
            if empty_dump:
                os.makedirs(os.path.join(root, "empty", "row-0", "dump"))
            failing = run.control_lines(collected, root, os.path.join(root, "baseline"), "s2")
        return failing, collected.bodies

    def test_the_empty_mutation_must_have_run_and_stay_untouched(self):
        self.assertEqual(self.run_negative_controls(empty_dump=True, touched=False), (0, []))
        self.assertEqual(self.run_negative_controls(empty_dump=True, touched=True)[0], 1)
        self.assertEqual(self.run_negative_controls(empty_dump=False, touched=False), (1, ["name=k2_empty_mutation ran=false"]))


if __name__ == "__main__":
    unittest.main()
