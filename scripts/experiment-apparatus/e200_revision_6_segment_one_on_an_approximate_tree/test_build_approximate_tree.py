"""build_approximate_tree.py 与 approximate_tree_lines.py 的单测：登记第五节 5.2 阳性对照 P40（造树 T2–T5）、P41（Q1 判法三种结局）、P42（T4 删七行），
与第九节 M40–M47 各自会红的那一格。git 仓都是每条用例自己在临时目录里建的，不读真的仓；跑法：在本目录 `python3 -m unittest test_build_approximate_tree`。"""

from __future__ import annotations

import hashlib
import os
import subprocess
import tempfile
import unittest
from unittest import mock

import approximate_tree_lines as tree_lines
import build_approximate_tree as tree_builder
from result_lines import ResultLines

BASE_TIME = 1_700_000_000
PINNED_DANGLING = "crates/singlefs-core/src/pinned_dangling.rs"
PINNED_IN_HISTORY = "crates/singlefs-core/src/pinned_in_history.rs"
PINNED_NOWHERE = "crates/singlefs-core/src/pinned_nowhere.rs"
CHANGED_TWICE = "crates/singlefs-harness/src/changed_twice.rs"
NEVER_CHANGED = "crates/singlefs-harness/src/never_changed.rs"
OUTSIDE_STATUS = "crates/singlefs-harness/src/outside_status.rs"
ADDED_TWICE = "crates/singlefs-harness/tests/added_twice.rs"
NEVER_ADDED = "crates/singlefs-harness/tests/never_added.rs"
DANGLING_CONTENT = b"revision 6 dangling\n"
HISTORY_CONTENT = b"revision 6 history\n"


def sha256_hex(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


class ScenarioRepository:
    """P40 的临时 git 仓：一个基线提交、三个后代提交（按提交时刻先后）、一个悬空 blob。"""

    def __init__(self, root: str):
        self.root = root
        os.makedirs(root, exist_ok=True)
        self.environment = dict(os.environ, HOME=root, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
        self.git("init", "-q", "-b", "main")
        self.commits: dict[str, str] = {}

    def git(self, *arguments: str, time: int | None = None) -> str:
        environment = dict(self.environment)
        if time is not None:
            environment.update(GIT_AUTHOR_DATE=f"{time} +0000", GIT_COMMITTER_DATE=f"{time} +0000")
        return subprocess.run(["git", "-c", "user.name=e200", "-c", "user.email=e200@example.invalid", *arguments], cwd=self.root,
                              env=environment, check=True, capture_output=True, text=True).stdout.strip()

    def commit(self, name: str, time: int, files: dict[str, bytes | None]) -> str:
        for path, content in files.items():
            target = os.path.join(self.root, path)
            if content is None:
                os.remove(target)
                continue
            os.makedirs(os.path.dirname(target), exist_ok=True)
            with open(target, "wb") as handle:
                handle.write(content)
        self.git("add", "-A")
        self.git("commit", "-q", "-m", name, time=time)
        self.commits[name] = self.git("rev-parse", "HEAD")
        return self.commits[name]

    def dangling_blob(self, content: bytes) -> str:
        path = os.path.join(self.root, "dangling-input")
        with open(path, "wb") as handle:
            handle.write(content)
        blob = self.git("hash-object", "-w", path)
        os.remove(path)
        return blob


def scenario(root: str) -> ScenarioRepository:
    repository = ScenarioRepository(root)
    repository.commit("base", BASE_TIME, {
        "Cargo.toml": b"[workspace]\n", "Cargo.lock": b"lock\n", ".cargo/config.toml": b"[env]\n", "crates/mutations.tsv": b"base\n",
        PINNED_DANGLING: b"base dangling\n", PINNED_IN_HISTORY: b"base history\n", PINNED_NOWHERE: b"base nowhere\n",
        CHANGED_TWICE: b"base twice\n", NEVER_CHANGED: b"base never\n", OUTSIDE_STATUS: b"base outside\n"})
    repository.commit("first", BASE_TIME + 100, {CHANGED_TWICE: b"first change\n", PINNED_IN_HISTORY: HISTORY_CONTENT,
                                                 OUTSIDE_STATUS: b"later outside\n", ADDED_TWICE: b"added first\n"})
    repository.commit("second", BASE_TIME + 200, {CHANGED_TWICE: b"second change\n", PINNED_IN_HISTORY: b"after history\n",
                                                  PINNED_NOWHERE: b"after nowhere\n", ADDED_TWICE: None})
    repository.commit("third", BASE_TIME + 300, {ADDED_TWICE: b"added second\n"})
    repository.dangling_blob(DANGLING_CONTENT)
    return repository


def scenario_inputs(extra_added: tuple[str, ...] = (), expected: int = 9) -> tree_builder.BuildInputs:
    """94 行的替身：六份改过的（含 crates/mutations.tsv）、一份新加的；OUTSIDE_STATUS 不在里面。三份钉值。"""
    status = [("M ", path) for path in (PINNED_DANGLING, PINNED_IN_HISTORY, PINNED_NOWHERE, CHANGED_TWICE, NEVER_CHANGED, "crates/mutations.tsv")]
    status += [("A ", path) for path in (ADDED_TWICE, *extra_added)]
    pins = [tree_builder.Pin(PINNED_DANGLING, "new_tree_file", sha256_hex(DANGLING_CONTENT)),
            tree_builder.Pin(PINNED_IN_HISTORY, "new_tree_file", sha256_hex(HISTORY_CONTENT)),
            tree_builder.Pin(PINNED_NOWHERE, "revision_6_tree", sha256_hex(b"never stored\n"))]
    return tree_builder.BuildInputs(status, pins, [], expected)


class Built:
    """在临时仓上造一次树：manifest 的行按路径取、产物的行按 name 取、退出码。"""

    def __init__(self, root: str, variant: str = tree_builder.VARIANT_FIRST_COMMIT, inputs: tree_builder.BuildInputs | None = None,
                 threads: int = 2):
        self.repository = scenario(os.path.join(root, "repository"))
        self.output = os.path.join(root, "output")
        os.makedirs(self.output)
        product = os.path.join(root, "build.out")
        with mock.patch.object(tree_builder, "BASE_COMMIT", self.repository.commits["base"]):
            lines = ResultLines(product)
            self.exit_code = tree_builder.build(tree_builder.GitRepository(self.repository.root), inputs or scenario_inputs(), variant,
                                                self.output, lines, threads, self.repository.root)
            lines.finish()
        with open(product, encoding="utf-8") as handle:
            self.product = handle.read().splitlines()
        self.rows = {row["path"]: row for row in tree_lines.read_manifest(os.path.join(self.output, "manifest.tsv"))}

    def content(self, path: str) -> bytes:
        with open(os.path.join(self.output, "tree", path), "rb") as handle:
            return handle.read()

    def lines_named(self, name: str) -> list[dict[str, str]]:
        return [tree_builder.key_values(line) for line in self.product if f" name={name} " in line + " "]


class TreeVersionChoice(unittest.TestCase):
    """P40：T2–T5 各一格（近似树甲），每条用例自己建仓、造一次树。"""

    def test_pin_kept_only_as_an_unreachable_blob_is_exact_pinned(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root)
            row = built.rows[PINNED_DANGLING]
            self.assertEqual((row["marker"], row["source"]), (tree_builder.MARKER_EXACT_PINNED, tree_builder.UNREACHABLE))
            self.assertEqual(built.content(PINNED_DANGLING), DANGLING_CONTENT)

    def test_pin_found_in_history_is_exact_pinned_with_the_commit_that_wrote_it(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root)
            row = built.rows[PINNED_IN_HISTORY]
            self.assertEqual((row["marker"], row["source"]), (tree_builder.MARKER_EXACT_PINNED, built.repository.commits["first"]))

    def test_pin_found_nowhere_falls_to_the_first_change_after_the_base(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root)
            row = built.rows[PINNED_NOWHERE]
            self.assertEqual((row["marker"], row["source"]), (tree_builder.MARKER_FIRST_COMMIT, built.repository.commits["second"]))
            self.assertEqual(built.content(PINNED_NOWHERE), b"after nowhere\n")

    def test_unpinned_path_changed_twice_takes_the_earlier_commit(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root)
            row = built.rows[CHANGED_TWICE]
            self.assertEqual((row["marker"], row["source"]), (tree_builder.MARKER_FIRST_COMMIT, built.repository.commits["first"]))
            self.assertEqual(built.content(CHANGED_TWICE), b"first change\n")

    def test_new_file_added_twice_takes_the_earliest_addition(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root)
            row = built.rows[ADDED_TWICE]
            self.assertEqual((row["marker"], row["source"]), (tree_builder.MARKER_FIRST_COMMIT, built.repository.commits["first"]))
            self.assertEqual(built.content(ADDED_TWICE), b"added first\n")

    def test_path_never_changed_after_the_base_is_approximate_head(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root)
            row = built.rows[NEVER_CHANGED]
            self.assertEqual((row["marker"], row["source"]), (tree_builder.MARKER_APPROXIMATE_HEAD, built.repository.commits["base"]))
            self.assertEqual(built.content(NEVER_CHANGED), b"base never\n")

    def test_path_outside_the_status_lines_takes_the_base_version_although_changed_later(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root)
            self.assertEqual(built.rows[OUTSIDE_STATUS]["marker"], tree_builder.MARKER_EXACT_HEAD)
            self.assertEqual(built.content(OUTSIDE_STATUS), b"base outside\n")

    def test_extra_paths_are_written_but_not_counted_in_the_digest(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root)
            self.assertEqual(built.content(".cargo/config.toml"), b"[env]\n")
            self.assertEqual(built.content("crates/mutations.tsv"), b"base\n")
            summary = built.lines_named("summary")[0]
            self.assertEqual(summary["digest_files"], "9")
            self.assertEqual(built.exit_code, 0)


class HeadVariant(unittest.TestCase):
    """近似树乙（登记第八节）：T5 一律取 31e47115… 那一版，新文件照取最早加它的那一版。"""

    def test_changed_path_keeps_the_base_version(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root, variant=tree_builder.VARIANT_HEAD)
            self.assertEqual(built.rows[CHANGED_TWICE]["marker"], tree_builder.MARKER_APPROXIMATE_HEAD)
            self.assertEqual(built.content(CHANGED_TWICE), b"base twice\n")

    def test_new_file_still_takes_the_earliest_addition(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root, variant=tree_builder.VARIANT_HEAD)
            self.assertEqual(built.rows[ADDED_TWICE]["source"], built.repository.commits["first"])


class PathSetStops(unittest.TestCase):
    """停机 S40 的两种触发：新文件后代里从没加过；路径集份数不是登记的数。"""

    def test_new_file_never_added_is_missing_and_stops(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root, inputs=scenario_inputs(extra_added=(NEVER_ADDED,), expected=10))
            self.assertEqual(built.rows[NEVER_ADDED]["marker"], tree_builder.MARKER_MISSING)
            self.assertEqual([line["clause"] for line in built.lines_named("stop")], ["S40"])
            self.assertEqual(built.exit_code, 1)

    def test_path_count_other_than_the_registered_count_stops(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root, inputs=scenario_inputs(expected=8))
            self.assertEqual(built.lines_named("path_set")[0]["holds"], "false")
            self.assertEqual(built.exit_code, 1)


class ParallelHashing(unittest.TestCase):
    """算 sha256 按区间切片并行：一条线程与几条线程逐项相同（登记第十四节；test-script-discipline「编译与算法都要快」的对拍）。"""

    def test_one_thread_and_three_threads_agree(self):
        with tempfile.TemporaryDirectory() as root:
            repository = scenario(os.path.join(root, "repository"))
            reader = tree_builder.GitRepository(repository.root)
            blobs = sorted({entry.split()[0] for entry in repository.git("rev-list", "--objects", "--all").splitlines()
                            if len(entry.split()) == 2 and reader.git("cat-file", "-t", entry.split()[0]).strip() == b"blob"} |
                           set(reader.unreachable_blobs()))
            single = tree_builder.hash_blobs(reader, blobs, 1)
            self.assertEqual(single, tree_builder.hash_blobs(reader, blobs, 3))
            self.assertIn(sha256_hex(DANGLING_CONTENT), single.values())


def device_text(with_name: bool) -> bytes:
    """3500 行；第 3478–3484 行（从 1 起）是一块七行，第 3480 行带登记点名的枚举名（with_name 为假时不带）。"""
    lines = [f"line {number}\n" for number in range(1, 3501)]
    for number in range(3478, 3485):
        lines[number - 1] = f"block line {number}\n"
    if with_name:
        lines[3480 - 1] = f"    {tree_builder.REMOVED_BLOCK_NAME},\n"
    return "".join(lines).encode("utf-8")


def independently_removed(text: bytes) -> bytes:
    """用例自己删第 3478–3484 行（不经 remove_device_block 的常量）。"""
    lines = text.splitlines(keepends=True)
    return b"".join(lines[:3477] + lines[3484:])


class FakeDeviceRepository:
    """T4 那一格的替身：只有 f8a9ddaab 那一版装置。"""

    def __init__(self, text: bytes):
        self.text = text

    def blob_at(self, commit: str, path: str) -> str | None:
        return "device-blob" if commit == tree_builder.DEVICE_BASE_COMMIT and path == tree_builder.DEVICE_PATH else None

    def read_blob(self, blob: str) -> bytes:
        return self.text


class DeviceBlockRemoval(unittest.TestCase):
    """P42：T4 删七行。"""

    def test_removal_matches_the_independent_deletion(self):
        text = device_text(with_name=True)
        removed = tree_builder.remove_device_block(text)
        self.assertEqual(sha256_hex(removed), sha256_hex(independently_removed(text)))
        self.assertEqual(len(removed.splitlines()) + 7, 3500)

    def test_block_without_the_name_stops_on_s44(self):
        with self.assertRaises(tree_builder.DeviceBlockMismatch):
            tree_builder.remove_device_block(device_text(with_name=False))

    def test_device_rebuilt_to_the_revision_6_digest_is_exact_reconstructed(self):
        text = device_text(with_name=True)
        pin = tree_builder.Pin(tree_builder.DEVICE_PATH, "new_tree_file", sha256_hex(independently_removed(text)))
        with mock.patch.object(tree_builder, "REVISION_6_DEVICE_SHA256", pin.target_sha256):
            choice, notes = tree_builder.choose_tree_version(FakeDeviceRepository(text), tree_builder.DEVICE_PATH, "AM", pin,
                                                             tree_builder.VARIANT_FIRST_COMMIT, lambda path: {}, {}, [])
        self.assertEqual((choice.marker, choice.source, notes["t3"]),
                         (tree_builder.MARKER_EXACT_RECONSTRUCTED, tree_builder.DEVICE_BASE_COMMIT, "not_found"))

    def test_device_rebuilt_to_another_digest_is_approximate_device(self):
        text = device_text(with_name=True)
        pin = tree_builder.Pin(tree_builder.DEVICE_PATH, "new_tree_file", sha256_hex(b"something else\n"))
        with mock.patch.object(tree_builder, "REVISION_6_DEVICE_SHA256", pin.target_sha256):
            choice, _ = tree_builder.choose_tree_version(FakeDeviceRepository(text), tree_builder.DEVICE_PATH, "AM", pin,
                                                         tree_builder.VARIANT_FIRST_COMMIT, lambda path: {}, {}, [])
        self.assertEqual(choice.marker, tree_builder.MARKER_APPROXIMATE_DEVICE)
        self.assertEqual(choice.content, independently_removed(text))


class Q1Candidate(unittest.TestCase):
    """P41：Q1 判法三种结局（登记第一节读法 3），类按 classify 取。"""

    def test_only_the_device_unequal_is_candidate_one(self):
        self.assertEqual(tree_lines.q1_candidate([tree_builder.classify(tree_builder.DEVICE_PATH)]), tree_lines.CANDIDATE_DEVICE_AND_RESEARCH)

    def test_a_core_file_unequal_is_candidate_two(self):
        classes = [tree_builder.classify(tree_builder.DEVICE_PATH), tree_builder.classify("crates/singlefs-core/src/allocator.rs")]
        self.assertEqual(tree_lines.q1_candidate(classes), tree_lines.CANDIDATE_PRODUCT)

    def test_only_a_harness_file_unequal_is_the_third_kind(self):
        self.assertEqual(tree_lines.q1_candidate([tree_builder.classify("crates/singlefs-harness/src/model.rs")]), tree_lines.THIRD_KIND)

    def test_no_unequal_pin_is_candidate_one(self):
        self.assertEqual(tree_lines.q1_candidate([]), tree_lines.CANDIDATE_DEVICE_AND_RESEARCH)

    def test_lock_file_and_manifests_count_as_other_verification_packages(self):
        self.assertEqual({tree_builder.classify(path) for path in ("Cargo.lock", "crates/singlefs-core/Cargo.toml")}, {tree_builder.CLASS_OTHER})

    def test_same_length_different_digest_is_unequal(self):
        self.assertFalse(tree_lines.pin_holds(sha256_hex(b"revision 6\n"), sha256_hex(b"approximate\n")))
        self.assertTrue(tree_lines.pin_holds(sha256_hex(b"same\n"), sha256_hex(b"same\n")))


def q1_product(root: str, built: Built) -> tuple[int, list[dict[str, str]]]:
    product = os.path.join(root, "q1.out")
    lines = ResultLines(product)
    exit_code = tree_lines.q1_lines(built.output, scenario_inputs().pins, [], lines)
    lines.finish()
    with open(product, encoding="utf-8") as handle:
        return exit_code, [tree_builder.key_values(line) for line in handle.read().splitlines()]


class Q1Lines(unittest.TestCase):
    """Q1 那几行在造出来的树上：逐份比、判候选、树与清单逐份相同（作废 Z40 的造树那一刻）。"""

    def test_each_pin_is_compared_and_the_unequal_core_file_gives_candidate_two(self):
        with tempfile.TemporaryDirectory() as root:
            exit_code, rows = q1_product(root, Built(root))
            equals = {row["path"]: row["equals"] for row in rows if row.get("name") == "q1_file"}
            self.assertEqual(equals, {PINNED_DANGLING: "true", PINNED_IN_HISTORY: "true", PINNED_NOWHERE: "false"})
            verdict = next(row for row in rows if row.get("name") == "verdict")
            self.assertEqual((verdict["q1_candidate"], verdict["unequal_pins"], verdict["tree_mismatches"]), (tree_lines.CANDIDATE_PRODUCT, "1", "0"))
            self.assertEqual(exit_code, 0)

    def test_unpinned_changed_paths_are_counted_by_class_and_marker(self):
        with tempfile.TemporaryDirectory() as root:
            _, rows = q1_product(root, Built(root))
            counts = {(row["class"], row["marker"]): row["count"] for row in rows if row.get("name") == "q1_unpinned"}
            self.assertEqual(counts, {(tree_builder.CLASS_OTHER, tree_builder.MARKER_FIRST_COMMIT): "2",
                                      (tree_builder.CLASS_OTHER, tree_builder.MARKER_APPROXIMATE_HEAD): "2"})

    def test_a_tree_file_changed_after_building_is_reported(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root)
            with open(os.path.join(built.output, "tree", NEVER_CHANGED), "ab") as handle:
                handle.write(b"changed after building\n")
            exit_code, rows = q1_product(root, built)
            self.assertEqual(next(row for row in rows if row.get("name") == "tree_matches_manifest")["mismatched"], "1")
            self.assertEqual(exit_code, 1)


class CompileLines(unittest.TestCase):
    """Q2-编那几行：编过且两个二进制都在、树没变才算编过。"""

    def compile_product(self, root: str, exit_code: int, with_binaries: bool, change_tree: bool) -> tuple[int, dict[str, str]]:
        tree = os.path.join(root, "tree")
        os.makedirs(os.path.join(tree, "target", "release"))
        with open(os.path.join(tree, "Cargo.toml"), "w", encoding="utf-8") as handle:
            handle.write("[workspace]\n")
        digest_before = tree_builder.snapshot_digest(tree)[0]
        if with_binaries:
            for name in tree_lines.BINARIES:
                with open(os.path.join(tree, "target", "release", name), "wb") as handle:
                    handle.write(name.encode("utf-8"))
        if change_tree:
            with open(os.path.join(tree, "Cargo.lock"), "w", encoding="utf-8") as handle:
                handle.write("written by cargo\n")
        log = os.path.join(root, "compile.log")
        with open(log, "w", encoding="utf-8") as handle:
            handle.write("".join(f"diagnostic {number}\n" for number in range(30)))
        product = os.path.join(root, "compile.out")
        lines = ResultLines(product)
        result = tree_lines.compile_lines(tree, exit_code, log, digest_before, lines)
        lines.finish()
        with open(product, encoding="utf-8") as handle:
            rows = [tree_builder.key_values(line) for line in handle.read().splitlines()]
        self.assertEqual(sum(row.get("name") == "compile_diagnostic" for row in rows), tree_lines.DIAGNOSTIC_TAIL)
        return result, next(row for row in rows if row.get("name") == "verdict")

    def test_compiled_with_both_binaries_and_an_unchanged_tree(self):
        with tempfile.TemporaryDirectory() as root:
            result, verdict = self.compile_product(root, 0, with_binaries=True, change_tree=False)
            self.assertEqual((result, verdict["compiled"], verdict["tree_unchanged"]), (0, "true", "true"))

    def test_cargo_failure_is_not_compiled(self):
        with tempfile.TemporaryDirectory() as root:
            result, verdict = self.compile_product(root, 101, with_binaries=False, change_tree=False)
            self.assertEqual((result, verdict["compiled"]), (1, "false"))

    def test_tree_changed_by_the_build_is_reported(self):
        with tempfile.TemporaryDirectory() as root:
            result, verdict = self.compile_product(root, 0, with_binaries=True, change_tree=True)
            self.assertEqual((result, verdict["tree_unchanged"]), (1, "false"))


class WholeTree(unittest.TestCase):
    """Q1-全树：快照摘要与份数等于修订 6 树时另写「就是修订 6 树」；那时 Q1-逐份还有不等的，走停机 S40（判法自相矛盾）。"""

    def test_whole_tree_equal_with_an_unequal_pin_is_the_s40_contradiction(self):
        with tempfile.TemporaryDirectory() as root:
            built = Built(root)
            digest, count = tree_builder.snapshot_digest(os.path.join(built.output, "tree"))
            with mock.patch.object(tree_lines, "REVISION_6_SNAPSHOT_DIGEST", digest), mock.patch.object(tree_lines, "REVISION_6_DIGEST_FILES", count):
                exit_code, rows = q1_product(root, built)
            self.assertEqual(next(row for row in rows if row.get("name") == "q1_whole_tree")["equals"], "true")
            self.assertEqual(next(row for row in rows if row.get("name") == "verdict")["s40_whole_tree_equal_but_a_pin_unequal"], "true")
            self.assertEqual(exit_code, 1)

    def test_whole_tree_other_than_revision_6_is_reported_unequal(self):
        with tempfile.TemporaryDirectory() as root:
            _, rows = q1_product(root, Built(root))
            whole = next(row for row in rows if row.get("name") == "q1_whole_tree")
            self.assertEqual((whole["equals"], whole["files"]), ("false", "9"))


def archived_text(lines: list[str]) -> bytes:
    return "".join(line + "\n" for line in lines).encode("utf-8")


NEW_TREE_TEXT = archived_text([
    "E7RESULT name=frozen_tree key=git_status line=M__crates/singlefs-core/src/allocator.rs",
    f"E7RESULT name=frozen_tree key=file_sha256 changed_by=o2 file=crates/singlefs-checker/src/walk.rs sha256={'a' * 64} equals_the_main_tree_now=true",
    f"E7RESULT name=frozen_tree key=file_sha256 changed_by=device file={tree_builder.DEVICE_PATH} sha256={tree_builder.NEW_TREE_DEVICE_SHA256} "
    "equals_the_main_tree_now=true"])
REVISION_6_TEXT = archived_text([
    f"E7RESULT name=frozen_tree key=device_sha256 place=tree sha256={tree_builder.REVISION_6_DEVICE_SHA256} expected=x holds=true",
    f"E7RESULT name=frozen_tree key=file_sha256 file=crates/singlefs-core/src/allocator.rs repo={'b' * 64} tree={'c' * 64}"])


class FakeArchive:
    """存档产物的替身：git show <提交>:<路径> 交手造的两份。"""

    def git(self, *arguments: str) -> bytes:
        return NEW_TREE_TEXT if arguments[1].endswith("frozen-tree.out") else REVISION_6_TEXT


class ArchivedPins(unittest.TestCase):
    """读法 2 的钉值从存档产物现取：装置那一份的目标换成修订 6 树那一份；产物 sha256 或行数不是登记的，退 3（作废 Z42 那一类）。"""

    def patched(self):
        return (mock.patch.object(tree_builder, "NEW_TREE_PRODUCT", ("x/frozen-tree.out", sha256_hex(NEW_TREE_TEXT), 3)),
                mock.patch.object(tree_builder, "REVISION_6_PRODUCT", ("x/frozen-tree-r6-r3.out", sha256_hex(REVISION_6_TEXT), 2)))

    def test_device_target_is_the_revision_6_device_and_tree_pins_are_added(self):
        first, second = self.patched()
        with first, second:
            pins = tree_builder.load_pins(FakeArchive())
            status = tree_builder.load_status_entries(FakeArchive())
        self.assertEqual([(pin.path, pin.role, pin.target_sha256) for pin in pins], [
            ("crates/singlefs-checker/src/walk.rs", "new_tree_file", "a" * 64),
            (tree_builder.DEVICE_PATH, "new_tree_file", tree_builder.REVISION_6_DEVICE_SHA256),
            ("crates/singlefs-core/src/allocator.rs", "revision_6_tree", "c" * 64)])
        self.assertEqual(status, [("M ", "crates/singlefs-core/src/allocator.rs")])

    def test_research_side_entry_kept_outside_the_repository_maps_to_the_e171_directory(self):
        table = ('{"files": {"' + tree_builder.E171_RESEARCH_DIRECTORY + '/row_driver.py": "' + "1" * 64 + '", '
                 '"/work-root/draft/s2_compare.py": "' + "2" * 64 + '"}}').encode("utf-8")

        class FakeTable:
            def git(self, *arguments: str) -> bytes:
                return table

        with mock.patch.object(tree_builder, "RESEARCH_SIDE_TABLE", ("commit", "table.json", sha256_hex(table))):
            pins = tree_builder.research_side_pins(FakeTable())
        self.assertEqual(pins, [(tree_builder.E171_RESEARCH_DIRECTORY + "/row_driver.py", "1" * 64, False),
                                (tree_builder.E171_RESEARCH_DIRECTORY + "/s2_compare.py", "2" * 64, True)])

    def test_archived_product_with_another_digest_is_refused(self):
        with mock.patch.object(tree_builder, "NEW_TREE_PRODUCT", ("x/frozen-tree.out", "0" * 64, 3)):
            with self.assertRaises(tree_builder.ArchivedInputMismatch):
                tree_builder.load_pins(FakeArchive())


class ResearchSideChoice(unittest.TestCase):
    """读法 4 研究一侧：钉值找得到记 exact_pinned；找不到取今天那一份；两张坏实现表找不到记 missing；没钉值的记 unrecorded。"""

    class Blobs:
        def read_blob(self, blob: str) -> bytes:
            return b"pinned version\n"

    def choose(self, root: str, path: str, pinned: str | None, found: bool) -> tree_builder.Choice:
        today = os.path.join(root, path)
        os.makedirs(os.path.dirname(today), exist_ok=True)
        with open(today, "wb") as handle:
            handle.write(b"today\n")
        candidates = {"blob-1": "commit-1"}
        hashes = {"blob-1": pinned if found else "f" * 64}
        return tree_builder.choose_research_version(path, pinned, candidates, hashes, self.Blobs(), root)

    def test_pinned_version_found_is_exact_pinned(self):
        with tempfile.TemporaryDirectory() as root:
            choice = self.choose(root, tree_builder.E171_RESEARCH_DIRECTORY + "/row_driver.py", "d" * 64, found=True)
            self.assertEqual((choice.marker, choice.source, choice.content), (tree_builder.MARKER_EXACT_PINNED, "commit-1", b"pinned version\n"))

    def test_pinned_version_not_found_takes_today(self):
        with tempfile.TemporaryDirectory() as root:
            choice = self.choose(root, tree_builder.E171_RESEARCH_DIRECTORY + "/row_driver.py", "d" * 64, found=False)
            self.assertEqual((choice.marker, choice.content), (tree_builder.MARKER_TODAY, b"today\n"))

    def test_bad_implementation_table_not_found_is_missing(self):
        with tempfile.TemporaryDirectory() as root:
            choice = self.choose(root, tree_builder.BAD_IMPLEMENTATION_TABLES[0], "d" * 64, found=False)
            self.assertEqual(choice.marker, tree_builder.MARKER_MISSING)

    def test_unpinned_file_is_unrecorded(self):
        with tempfile.TemporaryDirectory() as root:
            choice = self.choose(root, tree_builder.UNRECORDED_RESEARCH_FILES[0], None, found=False)
            self.assertEqual((choice.marker, choice.content), (tree_builder.MARKER_UNRECORDED, b"today\n"))


if __name__ == "__main__":
    unittest.main()
