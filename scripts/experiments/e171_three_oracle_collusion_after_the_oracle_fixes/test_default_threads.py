"""row_driver.py 的 --threads 缺省：给了照它；没给取登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它）÷ --lanes，向下取整，至少 1。

跑法：在这个目录里 `python3 -B -m unittest test_default_threads`。
"""
import os
import unittest

import row_driver

REGISTRY = os.path.join(row_driver.HERE, "..", "..", "..", ".claude", "heavy-thread-default")


def registered_threads() -> int:
    with open(REGISTRY, encoding="utf-8") as handle:
        for line in handle:
            if line.strip() and not line.lstrip().startswith("#"):
                return int(line.split()[0])
    raise AssertionError("登记文件里没有数")


class ThreadsPerLaneTests(unittest.TestCase):
    def test_an_explicit_value_is_kept(self) -> None:
        self.assertEqual(row_driver.threads_per_lane(5, 3, {}), 5)

    def test_without_a_value_the_registered_default_is_split_over_the_lanes(self) -> None:
        registered = registered_threads()
        self.assertEqual(row_driver.threads_per_lane(None, 1, {}), registered)
        self.assertEqual(row_driver.threads_per_lane(None, 2, {}), max(1, registered // 2))
        self.assertEqual(row_driver.threads_per_lane(None, registered + 100, {}), 1)

    def test_the_cap_from_capped_sh_wins_over_the_registered_default(self) -> None:
        self.assertEqual(row_driver.threads_per_lane(None, 1, {"SINGLEFS_THREAD_CAP": "7"}), 7)
        self.assertEqual(row_driver.threads_per_lane(None, 2, {"SINGLEFS_THREAD_CAP": "7"}), 3)


if __name__ == "__main__":
    unittest.main()
