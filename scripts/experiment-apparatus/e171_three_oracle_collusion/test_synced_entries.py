"""row_driver.py 的快照与线道的树带上每台默认重活线程数的登记文件 .claude/heavy-thread-default（验证包经 include_str! 在编译时读它，
没有就编不过），快照摘要只算 .rs 与 Cargo.*、不因它而变。sync_tree 用真的 rsync，在临时目录里造源树。

跑法：在这个目录里 `python3 -B -m unittest test_synced_entries`。
弄坏开关 E171_ROW_DRIVER_BREAK=registry-not-synced（row_driver.py 文件头）：test_the_thread_registry_reaches_the_snapshot_and_the_lane_tree 红。
弄坏开关 E171_ROW_DRIVER_BREAK=path-dependencies-not-synced：test_a_path_dependency_outside_crates_reaches_the_snapshot_and_the_lane_tree 红
（crates 经 path 依赖指到的仓根目录 prune-server/ 也要同步，否则验证包在线道的树里编不过）。
弄坏开关 E171_ROW_DRIVER_BREAK=configuration-check-not-synced：test_the_configuration_check_and_what_it_sources_reach_the_snapshot_and_the_lane_tree 红
（多机配置判法和它 source 的文件也要同步，否则装置在线道的树里起不了给定状态表的引擎）。
"""
import os
import tempfile
import unittest

import row_driver

REGISTRY_TEXT = "7  # 样本的登记\n"


def write(path: str, text: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def sample_repository(root: str, with_registry: bool) -> None:
    write(os.path.join(root, "Cargo.toml"), "[workspace]\n")
    write(os.path.join(root, "Cargo.lock"), "# lock\n")
    write(os.path.join(root, "crates", "demo", "src", "lib.rs"), "pub fn f() -> u32 { 1 }\n")
    write(os.path.join(root, "research", "results", "big.out"), "not synced\n")
    if with_registry:
        write(os.path.join(root, row_driver.THREAD_REGISTRY_ENTRY), REGISTRY_TEXT)


class SyncedEntriesTests(unittest.TestCase):
    def test_the_thread_registry_reaches_the_snapshot_and_the_lane_tree(self) -> None:
        with tempfile.TemporaryDirectory() as work:
            repository, snapshot, lane = (os.path.join(work, name) for name in ("repository", "snapshot", "lane"))
            sample_repository(repository, with_registry=True)
            row_driver.sync_tree(repository, snapshot)
            row_driver.sync_tree(snapshot, lane)
            for tree in (snapshot, lane):
                registry = os.path.join(tree, row_driver.THREAD_REGISTRY_ENTRY)
                self.assertTrue(os.path.isfile(registry), f"{tree} 里没有 {row_driver.THREAD_REGISTRY_ENTRY}：harness 编不过")
                with open(registry, encoding="utf-8") as handle:
                    self.assertEqual(handle.read(), REGISTRY_TEXT)
            self.assertFalse(os.path.exists(os.path.join(lane, "research")), "快照只拷 SYNCED_ENTRIES，research/ 不该进线道的树")

    def test_the_snapshot_digest_does_not_count_the_thread_registry(self) -> None:
        with tempfile.TemporaryDirectory() as work:
            with_registry, without_registry = os.path.join(work, "with"), os.path.join(work, "without")
            sample_repository(with_registry, with_registry=True)
            sample_repository(without_registry, with_registry=False)
            snapshot_with, snapshot_without = os.path.join(work, "snapshot-with"), os.path.join(work, "snapshot-without")
            row_driver.sync_tree(with_registry, snapshot_with)
            row_driver.sync_tree(without_registry, snapshot_without)
            self.assertEqual(row_driver.snapshot_digest(snapshot_with), row_driver.snapshot_digest(snapshot_without))


class ConfigurationCheckTests(unittest.TestCase):
    def test_the_configuration_check_and_what_it_sources_reach_the_snapshot_and_the_lane_tree(self) -> None:
        """装置起给定状态表的引擎要先起 <树>/scripts/crash/crash-point-replay-shard-configuration-check.sh（gpu_card_configuration.rs 按编它的那棵树拼路径）；
        线道的树里没有它和它 source 的文件，引擎起不来（第 3 次重跑探路片：退 127，每行 streams_not_exported=2）。"""
        with tempfile.TemporaryDirectory() as work:
            repository, snapshot, lane = (os.path.join(work, name) for name in ("repository", "snapshot", "lane"))
            sample_repository(repository, with_registry=True)
            for entry in row_driver.CONFIGURATION_CHECK_ENTRIES:
                write(os.path.join(repository, entry), f"# {entry}\n")
            row_driver.sync_tree(repository, snapshot)
            row_driver.sync_tree(snapshot, lane)
            for tree in (snapshot, lane):
                for entry in row_driver.CONFIGURATION_CHECK_ENTRIES:
                    path = os.path.join(tree, entry)
                    self.assertTrue(os.path.isfile(path), f"{tree} 里没有 {entry}：装置起不了给定状态表的引擎")
                    with open(path, encoding="utf-8") as handle:
                        self.assertEqual(handle.read(), f"# {entry}\n")

    def test_the_entries_are_the_ones_the_check_script_and_the_rust_side_name(self) -> None:
        """读真仓：入口表里的每一份在真仓里是文件，判法脚本真的 source 它们，Rust 一侧拼的就是判法这一条相对路径。"""
        root = os.environ.get("E171_REPOSITORY_ROOT") or os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../.."))
        for entry in row_driver.CONFIGURATION_CHECK_ENTRIES:
            self.assertTrue(os.path.isfile(os.path.join(root, entry)), entry)
        check = row_driver.CONFIGURATION_CHECK_ENTRIES[0]
        with open(os.path.join(root, "crates/singlefs-verification/src/gpu_card_configuration.rs"), encoding="utf-8") as handle:
            self.assertIn("../../" + check, handle.read())
        with open(os.path.join(root, check), encoding="utf-8") as handle:
            script = handle.read()
        self.assertIn("../../.claude/scripts/preflight.sh", script)
        self.assertIn("../hosts/peer-host-lib.sh", script)
        self.assertIn("../hosts/heavy-threads.sh", script)
        self.assertIn("scripts/hosts/heavy-threads.sh", row_driver.CONFIGURATION_CHECK_ENTRIES)


class PathDependencyTests(unittest.TestCase):
    def test_a_path_dependency_outside_crates_reaches_the_snapshot_and_the_lane_tree(self) -> None:
        """验证包依赖仓根的 prune-server/（第 3 次重跑任务 11511：线道的树里没有它，116 行编译全败）；带 target 的目录里 target 不拷。"""
        with tempfile.TemporaryDirectory() as work:
            repository, snapshot, lane = (os.path.join(work, name) for name in ("repository", "snapshot", "lane"))
            sample_repository(repository, with_registry=True)
            write(os.path.join(repository, "crates", "demo", "Cargo.toml"),
                  '[dependencies]\nsibling = { path = "../sibling" }\nprune-server = { path = "../../prune-server", features = ["x"] }\n'
                  'outside = { path = "../../../elsewhere" }\n')
            write(os.path.join(repository, "prune-server", "Cargo.toml"), "[package]\n")
            write(os.path.join(repository, "prune-server", "target", "big.o"), "not synced\n")
            write(os.path.join(repository, "unrelated", "Cargo.toml"), "[package]\n")
            self.assertEqual(row_driver.path_dependency_entries(repository), ["prune-server"])
            row_driver.sync_tree(repository, snapshot)
            row_driver.sync_tree(snapshot, lane)
            for tree in (snapshot, lane):
                self.assertTrue(os.path.isfile(os.path.join(tree, "prune-server", "Cargo.toml")), f"{tree} 里没有 prune-server/Cargo.toml：验证包编不过")
                self.assertFalse(os.path.exists(os.path.join(tree, "prune-server", "target")))
                self.assertFalse(os.path.exists(os.path.join(tree, "unrelated")))

    def test_the_real_repository_syncs_the_path_dependency_of_the_verification_package(self) -> None:
        """读真仓：验证包的 Cargo.toml 里 path 指到仓根的 prune-server，驱动认得出（真仓根取 E171_REPOSITORY_ROOT，没有就按本文件位置推）。"""
        root = os.environ.get("E171_REPOSITORY_ROOT") or os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../.."))
        self.assertIn("prune-server", row_driver.path_dependency_entries(root))


if __name__ == "__main__":
    unittest.main()
