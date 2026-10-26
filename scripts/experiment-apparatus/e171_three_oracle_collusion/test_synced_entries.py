"""row_driver.py 的快照与线道的树带上每台默认重活线程数的登记文件 .claude/heavy-thread-default（harness 包经 include_str! 在编译时读它，
没有就编不过），快照摘要只算 .rs 与 Cargo.*、不因它而变。sync_tree 用真的 rsync，在临时目录里造源树。

跑法：在这个目录里 `python3 -B -m unittest test_synced_entries`。
弄坏开关 E171_ROW_DRIVER_BREAK=registry-not-synced（row_driver.py 文件头）：test_the_thread_registry_reaches_the_snapshot_and_the_lane_tree 红。
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


if __name__ == "__main__":
    unittest.main()
