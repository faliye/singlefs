"""单测：执行器在单测小池（synthetic_pool_for_tests.py）上的各步与绝对值。

跑法（在本目录）：python3 -m unittest -v test_executor_on_synthetic_pool
写序号（单测小池的写流，21 次写）：0–1 取号系统配置；2–3 记录 1、4 根 1、5–6 系统配置；7–8 记录 2、9 根 2、10–11 系统配置；
12–15 数据单元与节点（盘 0 两次、盘 1 两次）、16–17 记录 3、18 根 3、19–20 系统配置。
"""

from __future__ import annotations

import os
import tempfile
import unittest

import on_disk_layout as layout
import spec_gaps
import synthetic_pool_for_tests as synthetic
from crash_dump import load_stream_dump
from crash_image import build_crash_image
from crc32c_castagnoli import crc32c, crc32c_bit_by_bit, crc32c_by_table
from executor_core import VERDICT_IN, VERDICT_NOT_IN, VERDICT_SPEC_GAP, judge_state, prepare_stream_context, recover
from replay_prefix import UnitChecksumCache
from version_table import allowed_versions

ALL_PERSISTED = "2" * 21
RECORD_THREE_PERSISTED_ROOT_THREE_NOT = "2" * 18 + "000"
DATA_UNIT_COPY_ON_DEVICE_ONE_MISSING = "2" * 14 + "0" + "222" + "000"
ACQUIRE_SLOT_ZERO_TORN_ON_DEVICE_ZERO = "1" + "0" * 20
LANDING_VECTORS = [ALL_PERSISTED, RECORD_THREE_PERSISTED_ROOT_THREE_NOT, DATA_UNIT_COPY_ON_DEVICE_ONE_MISSING,
                   ACQUIRE_SLOT_ZERO_TORN_ON_DEVICE_ZERO]
SCRATCH_ROOT = os.environ.get("E170_TEST_SCRATCH")


class SyntheticPoolFixture(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory(dir=SCRATCH_ROOT)
        cls.stream, cls.parts = synthetic.new_pool_file_creation_stream()
        synthetic.write_dump(cls.temporary.name, cls.stream, LANDING_VECTORS)
        cls.dump = load_stream_dump(cls.temporary.name)
        cls.context = prepare_stream_context(cls.dump)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def judge(self, landing_vector: str):
        state = next(state for state in self.dump.states if state.landing_vector == landing_vector)
        return judge_state(self.context, state, UnitChecksumCache())

    def recover_patched(self, landing_vector: str, patches: list[tuple[int, int, bytes]]):
        image = build_crash_image(self.dump, self.context.prepared_writes, landing_vector)
        for device, byte_offset, content in patches:
            image.write_bytes(device, byte_offset, content)
        recovery = recover(image, self.dump.device_identities, self.dump.sector_bytes, UnitChecksumCache(),
                           spec_gaps.GapsHitInOneState())
        allowed = allowed_versions(self.context.version_table, landing_vector)
        is_in = recovery.landed is not None and any(version.projection == recovery.landed for version in allowed.allowed)
        return recovery, (VERDICT_IN if is_in else VERDICT_NOT_IN)


class ChecksumAndLayoutPins(unittest.TestCase):
    def test_crc32c_matches_the_three_pinned_values_on_every_route(self):
        for data, expected in ((b"123456789", 0xE3069283), (bytes(4096), 0x98F94189), (bytes(512), 0x30FCEDC0)):
            self.assertEqual(crc32c_by_table(data), expected)
            self.assertEqual(crc32c_bit_by_bit(data), expected)
            self.assertEqual(crc32c(data), expected)

    def test_field_offsets_equal_every_offset_the_kb_writes_out(self):
        layout.assert_kb_anchors_hold()
        self.assertEqual(layout.SYSTEM_CONFIGURATION_FIELDS["slot_generation"].byte_offset, 147)
        self.assertEqual(layout.SYSTEM_CONFIGURATION_FIELDS["whole_slot_checksum"].byte_offset, 155)
        self.assertEqual(layout.JOURNAL_RECORD_FIELDS["jsn_counter"].byte_offset, 20)

    def test_gap_table_and_gap_module_list_the_same_identifiers(self):
        table_path = os.path.join(os.path.dirname(__file__), "spec-gaps.tsv")
        with open(table_path, encoding="utf-8") as handle:
            rows = [line.split("\t") for line in handle.read().splitlines()[1:] if line.strip()]
        self.assertEqual([row[0] for row in rows], list(spec_gaps.ALL_GAP_IDENTIFIERS))
        self.assertTrue(all(len(row) == 8 for row in rows), "每行八列")
        self.assertTrue(all(row[4] in ("两种读法", "缺一句", "只有读 crates 才写得下去") for row in rows))

    def test_recording_an_unregistered_gap_is_refused(self):
        with self.assertRaises(AssertionError):
            spec_gaps.GapsHitInOneState().record_hit("G99")


class DumpReadingAndImages(SyntheticPoolFixture):
    def test_dump_counts_and_checksums_agree_with_the_manifest(self):
        self.assertEqual(self.dump.count_mismatches, [])
        self.assertEqual(self.dump.checksum_mismatches, [])
        self.assertEqual(len(self.dump.write_operation_indices), 21)
        self.assertEqual(len(self.dump.operations), 37)
        self.assertEqual(sorted(self.dump.torn_payload_by_write_ordinal), [0, 1, 5, 6, 10, 11, 19, 20])

    def test_every_rebuilt_image_digest_equals_the_dumped_digest(self):
        for landing_vector in LANDING_VECTORS:
            self.assertEqual(self.judge(landing_vector).digest_matches, True, landing_vector)

    def test_a_changed_landing_vector_changes_the_digest(self):
        image = build_crash_image(self.dump, self.context.prepared_writes, ALL_PERSISTED)
        other = build_crash_image(self.dump, self.context.prepared_writes, RECORD_THREE_PERSISTED_ROOT_THREE_NOT)
        self.assertNotEqual(image.digest(), other.digest())

    def test_a_manifest_count_that_disagrees_is_reported(self):
        with tempfile.TemporaryDirectory(dir=SCRATCH_ROOT) as directory:
            synthetic.write_dump(directory, self.stream, LANDING_VECTORS)
            manifest_path = os.path.join(directory, "manifest.tsv")
            with open(manifest_path, encoding="utf-8") as handle:
                text = handle.read()
            with open(manifest_path, "w", encoding="utf-8") as handle:
                handle.write(text.replace("writes\t21\n", "writes\t22\n"))
            self.assertEqual(load_stream_dump(directory).count_mismatches, ["writes manifest=22 counted=21"])


class AllPersistedStateAbsoluteValues(SyntheticPoolFixture):
    """单测小池上的 A1–A7 同形：全持久态的每一步钉绝对值（不是真流一，数照单测小池的写流）。"""

    def setUp(self):
        image = build_crash_image(self.dump, self.context.prepared_writes, ALL_PERSISTED)
        self.recovery = recover(image, self.dump.device_identities, self.dump.sector_bytes, UnitChecksumCache(),
                                spec_gaps.GapsHitInOneState())

    def test_lands_on_instance_one_txg_three(self):
        self.assertEqual((self.recovery.landed.instance, self.recovery.landed.checkpoint_txg), (1, 3))
        self.assertEqual(self.recovery.landed.tree_identifier_watermark, 65548)

    def test_four_self_verified_roots_sit_where_the_region_formula_puts_them(self):
        found = sorted((root.location.region, root.location.slot_in_region, root.location.device_identity,
                        root.unsigned("instance"), root.unsigned("checkpoint_txg"), root.location.byte_offset)
                       for root in self.recovery.root_reading.self_verified_roots)
        self.assertEqual(found, [(0, 0, 0, 0, 0, 1048576), (0, 1, 0, 1, 3, 1048576 + 4096),
                                 (1, 0, 1, 1, 1, 4194304), (2, 0, 0, 1, 2, 7340032)])

    def test_each_device_ring_holds_three_records_at_offsets_zero_4096_8192(self):
        for device in (0, 1):
            records = self.recovery.ring_scan.valid_records_by_device[device]
            self.assertEqual([(record.ring_slot_index * 4096, record.jsn, record.unsigned("checkpoint_txg")) for record in records],
                             [(0, (1, 1), 1), (4096, (1, 2), 2), (8192, (1, 3), 3)])

    def test_each_device_chooses_generation_five_in_slot_one(self):
        for device in (0, 1):
            slot = self.recovery.pool.chosen_slot_by_device[device]
            self.assertEqual((slot.unsigned("slot_generation"), slot.slot_number, slot.unsigned("journal_tail"),
                              slot.unsigned("journal_instance")), (5, 1, 3, 1))

    def test_version_table_has_four_versions_in_order(self):
        self.assertEqual([(version.projection.instance, version.projection.checkpoint_txg)
                          for version in self.context.version_table.versions], [(0, 0), (1, 1), (1, 2), (1, 3)])
        self.assertEqual(self.context.version_table.anomalies, [])

    def test_record_three_carries_two_named_items_transaction_one_and_the_last_flag(self):
        record = self.recovery.ring_scan.records_by_counter[3]
        self.assertEqual((record.unsigned("named_item_count"), record.unsigned("transaction_number"),
                          record.unsigned("commit_marker"), record.unsigned("ordinal_within_publish"),
                          record.is_last_of_publish), (2, 1, 1, 1, True))

    def test_allowed_set_of_the_all_persisted_state_is_versions_two_and_three(self):
        # 读法 3 照字面：末次调用的返回点 37 > m = 34，末次调用不算确认 ⇒ ack = 版本 2（登记 A1 写「恰一个元素」，与这一读数不同，交主 agent）
        allowed = allowed_versions(self.context.version_table, ALL_PERSISTED)
        self.assertEqual((allowed.largest_landed_write_stream_index, allowed.acknowledged_version_index,
                          allowed.begun_version_index), (34, 2, 3))
        self.assertEqual([version.version_index for version in allowed.allowed], [2, 3])


class SampledStateVerdicts(SyntheticPoolFixture):
    def test_all_persisted_state_is_in_and_strict_run_reports_the_magic_and_offset_gaps(self):
        judgement = self.judge(ALL_PERSISTED)
        self.assertEqual(judgement.provisional_verdict, VERDICT_IN)
        self.assertEqual(judgement.strict_verdict, VERDICT_SPEC_GAP)
        self.assertEqual(judgement.gaps_hit, ["G01", "G02"])

    def test_record_persisted_before_its_root_is_replayed_onto_the_previous_root(self):
        judgement = self.judge(RECORD_THREE_PERSISTED_ROOT_THREE_NOT)
        self.assertEqual(judgement.recovery_summary["chosen_root"], "(1,2)@r2s0")
        self.assertEqual(judgement.recovery_summary["applied"], 1)
        self.assertEqual(judgement.recovery_summary["path"], "anchor_readable")
        self.assertTrue(judgement.landed_text.startswith("(1,3,"))
        self.assertEqual(judgement.provisional_verdict, VERDICT_IN)

    def test_a_named_unit_missing_one_copy_stops_the_prefix_and_lands_the_previous_version(self):
        judgement = self.judge(DATA_UNIT_COPY_ON_DEVICE_ONE_MISSING)
        self.assertEqual(judgement.recovery_summary["stop"], "named_unit_not_verified_on_both_copies")
        self.assertEqual(judgement.recovery_summary["applied"], 0)
        self.assertTrue(judgement.landed_text.startswith("(1,2,"))
        self.assertEqual(judgement.provisional_verdict, VERDICT_IN)

    def test_torn_slot_zero_sends_the_reader_probing_for_slot_one(self):
        judgement = self.judge(ACQUIRE_SLOT_ZERO_TORN_ON_DEVICE_ZERO)
        self.assertIn("G03", judgement.gaps_hit)
        self.assertTrue(judgement.landed_text.startswith("(0,0,"))
        self.assertEqual(judgement.provisional_verdict, VERDICT_IN)
        self.assertEqual(judgement.recovery_summary["torn"], 1)


def flipped(content: bytes, byte_offset: int) -> bytes:
    return content[:byte_offset] + bytes([content[byte_offset] ^ 0x01]) + content[byte_offset + 1:]


class PositiveControlsOnTheAllPersistedState(SyntheticPoolFixture):
    """阳性对照的单测小池版（登记 5.5 P 行、N 行的同形改坏）：P 型判「不在」，N 型判「在」。"""

    def record_patches(self, records: list[synthetic.RecordFields]) -> list[tuple[int, int, bytes]]:
        patches = []
        for fields in records:
            content = synthetic.journal_record(fields)
            patches += [(device, synthetic.journal_slot_offset(fields.counter), content) for device in synthetic.DEVICES]
        return patches

    def forged_continuation(self, counters: list[int], flags: list[bool], watermark_increase: int = 1000):
        new_root = synthetic.RootFields(1, 4, 3, 3, 65548 + watermark_increase)
        previous = self.parts["records"][3]
        forged = []
        for ordinal, (counter, flag) in enumerate(zip(counters, flags), start=1):
            fields = synthetic.RecordFields(1, counter, 4, ordinal, flag, 0, 1, (), new_root, synthetic.chain_value_of(previous))
            forged.append(fields)
            previous = synthetic.journal_record(fields)
        return self.record_patches(forged)

    def test_p1_root_ring_base_off_by_one_in_the_chosen_slots_refuses_the_pool(self):
        patches = [(device, synthetic.SPACING, synthetic.system_configuration_slot(device, 5, 3, 1, overrides={"root_ring_base_slot": 65}))
                   for device in synthetic.DEVICES]
        recovery, verdict = self.recover_patched(ALL_PERSISTED, patches)
        self.assertEqual(recovery.landed, None)
        self.assertTrue(recovery.refusal_reason.startswith("system_configuration_value_refused:root_ring_base_not_first_version_slot"))
        self.assertEqual(verdict, VERDICT_NOT_IN)

    def test_p2_forged_root_three_txgs_ahead_is_chosen_and_lies_outside_the_allowed_set(self):
        device, byte_offset = synthetic.root_slot_of_txg(6)
        recovery, verdict = self.recover_patched(ALL_PERSISTED, [(device, byte_offset, synthetic.root_record(synthetic.RootFields(1, 6, 3, 3, 65548)))])
        self.assertEqual((recovery.landed.instance, recovery.landed.checkpoint_txg), (1, 6))
        self.assertEqual(verdict, VERDICT_NOT_IN)

    def test_p3_forged_single_record_continuation_is_applied_and_lies_outside_the_allowed_set(self):
        recovery, verdict = self.recover_patched(ALL_PERSISTED, self.forged_continuation([4], [True]))
        self.assertEqual((recovery.landed.checkpoint_txg, recovery.landed.tree_identifier_watermark), (4, 66548))
        self.assertEqual(verdict, VERDICT_NOT_IN)

    def test_p4_forged_two_record_publish_with_the_flag_on_the_second_is_applied(self):
        recovery, verdict = self.recover_patched(ALL_PERSISTED, self.forged_continuation([4, 5], [False, True]))
        self.assertEqual(len(recovery.replay.applied_records), 2)
        self.assertEqual(verdict, VERDICT_NOT_IN)

    def test_n2_forged_record_after_a_counter_gap_is_not_applied(self):
        recovery, verdict = self.recover_patched(ALL_PERSISTED, self.forged_continuation([5], [True]))
        self.assertEqual(recovery.replay.stop_reason, "jsn_not_readable_or_gap")
        self.assertEqual(verdict, VERDICT_IN)

    def test_n3_forged_publish_without_any_last_flag_is_not_applied(self):
        recovery, verdict = self.recover_patched(ALL_PERSISTED, self.forged_continuation([4, 5], [False, False]))
        self.assertEqual(recovery.replay.applied_records, [])
        self.assertEqual(recovery.landed.checkpoint_txg, 3)
        self.assertEqual(verdict, VERDICT_IN)

    def test_p5_root_instance_lowered_stops_at_the_instance_boundary_and_lies_outside(self):
        device, byte_offset = synthetic.root_slot_of_txg(3)
        recovery, verdict = self.recover_patched(ALL_PERSISTED, [(device, byte_offset, synthetic.root_record(synthetic.RootFields(0, 3, 3, 3, 65548)))])
        self.assertEqual(recovery.replay.stop_reason, "head_belongs_to_another_instance")
        self.assertEqual((recovery.landed.instance, recovery.landed.checkpoint_txg), (0, 3))
        self.assertEqual(verdict, VERDICT_NOT_IN)

    def test_n1_root_txg_lowered_in_place_replays_record_three(self):
        device, byte_offset = synthetic.root_slot_of_txg(3)
        recovery, verdict = self.recover_patched(ALL_PERSISTED, [(device, byte_offset, synthetic.root_record(synthetic.RootFields(1, 1, 3, 3, 65548)))])
        self.assertEqual(recovery.chosen_root.choice_key, (2, 1))
        self.assertEqual(recovery.landed.checkpoint_txg, 3)
        self.assertEqual(verdict, VERDICT_IN)

    def test_n4_latest_root_unverifiable_replays_record_three(self):
        device, byte_offset = synthetic.root_slot_of_txg(3)
        root_three = synthetic.root_record(self.parts["roots"][3])
        recovery, verdict = self.recover_patched(ALL_PERSISTED, [(device, byte_offset, flipped(root_three, 122))])
        self.assertEqual(recovery.landed.checkpoint_txg, 3)
        self.assertEqual(verdict, VERDICT_IN)

    def test_p8_on_the_all_persisted_base_lands_version_two_which_reading_three_still_allows(self):
        """登记 5.5 P8 期望「不在」靠「v_n 已确认（ack = n）」；读法 3 照字面在全持久态上 ack = n − 1，于是判「在」。
        这一条钉的是今天的读数，交主 agent（不是执行器改得掉的：A(c) 是登记第一节定的）。"""
        device, byte_offset = synthetic.root_slot_of_txg(3)
        root_three = synthetic.root_record(self.parts["roots"][3])
        record_three = self.parts["records"][3]
        patches = [(device, byte_offset, flipped(root_three, 122))]
        patches += [(each_device, synthetic.journal_slot_offset(3), flipped(record_three, 311)) for each_device in synthetic.DEVICES]
        recovery, verdict = self.recover_patched(ALL_PERSISTED, patches)
        self.assertEqual(recovery.landed.checkpoint_txg, 2)
        self.assertEqual(verdict, VERDICT_IN)

    def test_no_system_configuration_on_any_device_refuses_the_pool(self):
        patches = [(device, slot_offset, bytes(4096)) for device in synthetic.DEVICES for slot_offset in (0, synthetic.SPACING)]
        recovery, verdict = self.recover_patched(ALL_PERSISTED, patches)
        self.assertEqual(recovery.refusal_reason, "no_choosable_system_configuration_on_any_device")
        self.assertEqual(verdict, VERDICT_NOT_IN)



class GapTriggersFireOnlyWhereTheReadingsPart(SyntheticPoolFixture):
    """严格跑法的「撞上」要分得出：两种读法给同一结果时不记，给不同结果（或走到缺句的那一支）时记。"""

    def gaps_after(self, patches: list[tuple[int, int, bytes]]):
        image = build_crash_image(self.dump, self.context.prepared_writes, ALL_PERSISTED)
        for device, byte_offset, content in patches:
            image.write_bytes(device, byte_offset, content)
        gaps = spec_gaps.GapsHitInOneState()
        recovery = recover(image, self.dump.device_identities, self.dump.sector_bytes, UnitChecksumCache(), gaps)
        return recovery, gaps.sorted_identifiers()

    def test_g05_root_in_a_slot_its_txg_does_not_give_changes_the_choice(self):
        device, byte_offset = synthetic.root_slot_of_txg(6)
        recovery, gaps = self.gaps_after([(device, byte_offset, synthetic.root_record(synthetic.RootFields(1, 9, 3, 3, 65548)))])
        self.assertEqual(recovery.chosen_root.choice_key, (9, 1))
        self.assertEqual(gaps, ["G01", "G02", "G05"])

    def test_g06_root_with_a_foreign_identity_changes_the_choice(self):
        device, byte_offset = synthetic.root_slot_of_txg(6)
        foreign = synthetic.root_record(synthetic.RootFields(1, 6, 3, 3, 65548), pool_identity=bytes(16))
        recovery, gaps = self.gaps_after([(device, byte_offset, foreign)])
        self.assertEqual(recovery.chosen_root.choice_key, (3, 1))
        self.assertEqual(gaps, ["G01", "G02", "G06"])

    def test_g07_two_valid_copies_that_differ_are_reported(self):
        record_three = self.parts["records"][3]
        other = synthetic.journal_record(synthetic.RecordFields(1, 3, 3, 1, True, 1, 1, (), synthetic.RootFields(1, 3, 3, 3, 1), 0))
        recovery, gaps = self.gaps_after([(1, synthetic.journal_slot_offset(3), other)])
        self.assertNotEqual(other, record_three)
        self.assertIn("G07", gaps)
        self.assertEqual(recovery.landed.checkpoint_txg, 3)

    def test_g08_record_outside_its_slot_is_reported(self):
        stray = synthetic.journal_record(synthetic.RecordFields(1, 10, 4, 1, True, 0, 1, (), synthetic.RootFields(1, 4, 3, 3, 1), 0))
        _recovery, gaps = self.gaps_after([(0, synthetic.journal_slot_offset(4), stray)])
        self.assertEqual(gaps, ["G01", "G02", "G08"])

    def test_g20_second_last_flag_in_the_root_publish_stops_recovery(self):
        second = synthetic.journal_record(synthetic.RecordFields(1, 4, 3, 2, True, 1, 1, (), self.parts["roots"][3],
                                                                 synthetic.chain_value_of(self.parts["records"][3])))
        recovery, gaps = self.gaps_after([(device, synthetic.journal_slot_offset(4), second) for device in synthetic.DEVICES])
        self.assertEqual(recovery.landed, None)
        self.assertIn("G20", gaps)


class DriverEndToEnd(unittest.TestCase):
    """驱动从导出目录读到写完产物：单进程与两个工作进程各跑一趟，判定行逐态相同、汇总行与包含自检照打。"""

    def run_driver(self, dumps_directory: str, output_path: str, worker_count: int) -> tuple[int, list[str]]:
        import executor_main
        executor_main.contexts_by_stream.clear()
        exit_code = executor_main.main(["--dumps", dumps_directory, "--streams", "new_pool_file_creation",
                                        "--batches", "1", "--workers", str(worker_count), "--output", output_path])
        if exit_code != 0:
            return exit_code, []
        with open(output_path, encoding="utf-8") as handle:
            return exit_code, handle.read().splitlines()

    def test_single_and_two_worker_runs_give_the_same_state_verdicts(self):
        stream, _parts = synthetic.new_pool_file_creation_stream()
        with tempfile.TemporaryDirectory(dir=SCRATCH_ROOT) as directory:
            dumps_directory = os.path.join(directory, "dumps")
            synthetic.write_dump(os.path.join(dumps_directory, "new_pool_file_creation"), stream, LANDING_VECTORS,
                                 stream_name="new_pool_file_creation")
            single_exit, single = self.run_driver(dumps_directory, os.path.join(directory, "single.out"), 1)
            parallel_exit, parallel = self.run_driver(dumps_directory, os.path.join(directory, "parallel.out"), 2)
            self.assertEqual((single_exit, parallel_exit), (0, 0))
            self.assertEqual(single[-1], "name=complete")

            def verdict_fields(lines):
                return sorted(" ".join(field for field in line.split() if not field.startswith(("time_ns=", "worker=")))
                              for line in lines if line.startswith("name=state "))

            self.assertEqual(len(verdict_fields(single)), 4)
            self.assertEqual(verdict_fields(single), verdict_fields(parallel))
            for lines in (single, parallel):
                timing = [line for line in lines if line.startswith("name=timing ")]
                self.assertEqual(len(timing), 1)
                self.assertIn("contains_self_check=pass", timing[0])
            verdict = [line for line in single if line.startswith("name=verdict ")][0]
            self.assertIn("strict_spec_gap_states=4", verdict)
            self.assertIn("provisional_in=4", verdict)
            self.assertIn("digest_mismatches=0", verdict)
            assertions = {line.split()[1]: line.split()[3] for line in single if line.startswith("name=anchor_assertion ")}
            self.assertEqual(assertions, {"id=A6": "holds=true", "id=A1": "holds=false", "id=A2": "holds=true",
                                          "id=A3": "holds=true", "id=A4": "holds=true", "id=A5": "holds=true",
                                          "id=A7": "holds=false"})
            torn = [line for line in single if line.startswith("name=torn_reading ")][0]
            self.assertIn("differing_entries=0", torn)
            freeze = [line for line in single if line.startswith("name=freeze ")]
            self.assertTrue(any("file=executor_main.py" in line for line in freeze))
            refused_exit, _lines = self.run_driver(dumps_directory, os.path.join(directory, "single.out"), 1)
            self.assertEqual(refused_exit, 2)


if __name__ == "__main__":
    unittest.main()
