"""两台对称（第 2 次重跑登记 research/prompts/e171-r2-prereg.md 修订 5 第 5.8 条）：分片的台照 TASK_SERVER_SHARD_HOST 给的地址记，
编排那一片所在的台是第 1 台，登记地址与它不同的是第 2 台；不认 local、peer 这类角色名。
每条用例自己建、自己删临时目录；不起编排、不连别的台。地址取文档用的地址段（RFC 5737）。"""
from __future__ import annotations

import os
import shutil
import tempfile
import unittest

import rows_orchestrator as orchestrator
import shard_relay

THIS_ADDRESS, OTHER_ADDRESS = "192.0.2.1", "192.0.2.2"


class ShardRegistrationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.plan = tempfile.mkdtemp(prefix="e171-symmetric-hosts-test-")

    def tearDown(self) -> None:
        shutil.rmtree(self.plan, ignore_errors=True)

    def test_a_registration_records_the_host_address_it_was_given(self) -> None:
        shard_relay.register(self.plan, 0, THIS_ADDRESS)
        shard_relay.register(self.plan, 1, OTHER_ADDRESS)
        self.assertEqual(shard_relay.registrations(self.plan), {0: THIS_ADDRESS, 1: OTHER_ADDRESS})

    def test_a_registration_with_a_blank_host_is_not_read(self) -> None:
        shard_relay.register(self.plan, 1, " ")
        self.assertEqual(shard_relay.registrations(self.plan), {})


class ShardPhaseTests(unittest.TestCase):
    def test_a_shard_without_a_host_address_is_refused_as_unrecognized(self) -> None:
        plan = tempfile.mkdtemp(prefix="e171-symmetric-hosts-test-")
        try:
            said: list[str] = []
            code = orchestrator.shard_phase_before_layer("/unused", {"TASK_SERVER_SHARD_PHASE": "shard", "TASK_SERVER_SHARD_PLAN_DIRECTORY": plan,
                                                                     "TASK_SERVER_SHARD": "1/2", "TASK_SERVER_SHARD_HOST": ""}, said.append)
        finally:
            shutil.rmtree(plan, ignore_errors=True)
        self.assertEqual(code, 2)
        self.assertTrue(any("认不出" in line for line in said), said)


if __name__ == "__main__":
    unittest.main()
