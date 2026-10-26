"""row_driver.py 的 --threads 缺省：给了照它；没给取登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它）÷ --lanes，向下取整，至少 1。
多机编排 rows_orchestrator.py 的每台线程与修订 8 单元编排 segment_one_units_revision_8.py 的本机线程额度：都等于登记值，与此刻空闲无关
（喂的假资源此刻空闲只报 1 线程）。

跑法：在这个目录里 `python3 -B -m unittest test_default_threads`。
"""
import json
import os
import shlex
import tempfile
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
    """每台线程等于登记值、与此刻空闲无关：假资源里此刻空闲只有 1 线程，内存放得下几槽。"""

    def test_every_host_of_the_rows_orchestrator_gets_the_registered_threads(self) -> None:
        registered = registered_threads()
        hosts = rows_orchestrator.two_hosts()
        entries = [rows_orchestrator.resources_entry(1, 1, 200, 200), rows_orchestrator.resources_entry(2, 1, 200, 200)]
        found, unread = rows_orchestrator.resources_from_json({"hosts": entries}, hosts)
        self.assertEqual(unread, [])
        with mock.patch.dict(os.environ, environment_without_the_cap(), clear=True):
            allocations = rows_orchestrator.allocate(hosts, found)
        self.assertEqual([allocation.threads for allocation in allocations], [registered, registered])
        self.assertTrue(all(allocation.slots >= 1 for allocation in allocations))
        self.assertEqual(rows_orchestrator.single_lane_host(allocations).threads, registered)

    def test_the_local_budget_of_the_revision_8_units_takes_the_registered_threads(self) -> None:
        registered = registered_threads()
        with tempfile.TemporaryDirectory() as scratch:
            resources = os.path.join(scratch, "resources.json")
            with open(resources, "w", encoding="utf-8") as handle:
                json.dump({"hosts": [{"number": 1, "read": True, "cpu": {"available_threads": 1, "cores": 2},
                                      "memory": {"available": 40 << 30, "slice_room": 20 << 30, "slice_total": 60 << 30, "mem_total": 80 << 30}}]}, handle)
            environment = environment_without_the_cap()
            environment.update(UNITS_R8_SELFTEST="1", UNITS_R8_RESOURCES_COMMAND=f"cat {shlex.quote(resources)}")
            settings = types.SimpleNamespace(repo=scratch)
            with mock.patch.dict(os.environ, environment, clear=True):
                budget = units_revision_8.local_budget(settings)
                idle = units_revision_8.idle_budget(settings)
        self.assertEqual(budget.threads, registered)
        self.assertEqual(budget.memory_bytes, 20 << 30)
        self.assertEqual(idle.threads, registered)
        self.assertEqual(idle.memory_bytes, 60 << 30)


if __name__ == "__main__":
    unittest.main()
