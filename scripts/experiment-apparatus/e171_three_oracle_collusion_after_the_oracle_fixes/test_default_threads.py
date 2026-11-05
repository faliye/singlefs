"""row_driver.py 的 --threads 缺省：给了照它；没给取登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它）÷ --lanes，向下取整，至少 1。
多机编排 rows_orchestrator.py 的每台线程与修订 8 单元编排 segment_one_units_revision_8.py 的本机线程额度：都等于登记值，与此刻空闲无关
（与 SINGLEFS_THREAD_CAP 之外的此刻状态无关）。

跑法：在这个目录里 `python3 -B -m unittest test_default_threads`。
"""
import os
import types
import unittest
from unittest import mock

import row_driver
import rows_orchestrator
import segment_one_units_revision_8 as units_revision_8

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


def environment_without_the_cap() -> dict:
    """此刻的环境去掉 SINGLEFS_THREAD_CAP：登记取法照登记文件读，不照调用方的上限。"""
    environment = dict(os.environ)
    environment.pop("SINGLEFS_THREAD_CAP", None)
    return environment


class RegisteredHostThreadsTests(unittest.TestCase):
    """每台线程等于登记值；槽数只看线程，不看内存。"""

    def test_every_host_of_the_rows_orchestrator_gets_the_registered_threads(self) -> None:
        registered = registered_threads()
        hosts = rows_orchestrator.two_hosts()
        with mock.patch.dict(os.environ, environment_without_the_cap(), clear=True):
            allocations = rows_orchestrator.allocate(hosts)
        self.assertEqual([allocation.threads for allocation in allocations], [registered, registered])
        self.assertTrue(all(allocation.slots >= 1 for allocation in allocations))
        self.assertEqual(rows_orchestrator.single_lane_host(allocations, 2).number, 2)
        self.assertEqual(rows_orchestrator.single_lane_host(allocations, 1).threads, registered)

    def test_the_local_budget_of_the_revision_8_units_takes_the_registered_threads(self) -> None:
        registered = registered_threads()
        settings = types.SimpleNamespace(repo="/nowhere")
        with mock.patch.dict(os.environ, environment_without_the_cap(), clear=True):
            budget = units_revision_8.local_budget(settings)
        self.assertEqual(budget.threads, registered)


if __name__ == "__main__":
    unittest.main()
