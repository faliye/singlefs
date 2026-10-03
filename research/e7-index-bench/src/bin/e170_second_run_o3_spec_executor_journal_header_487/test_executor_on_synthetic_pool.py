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
from crash_image import SparseCrashImage, build_crash_image
from system_configuration import SLOT_ONE_ROUTE_EVERY_SPACING, SLOT_ONE_ROUTE_FIRST_USABLE_SLOT_ZERO_OF_THE_POOL, \
    SLOT_ONE_ROUTE_OWN_SLOT_ZERO, PoolRefusal, choose_system_configuration
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
        layout.assert_every_offset_the_layout_lines_list_holds()
        self.assertEqual(layout.SYSTEM_CONFIGURATION_FIELDS["slot_generation"].byte_offset, 147)
        self.assertEqual(layout.SYSTEM_CONFIGURATION_FIELDS["whole_slot_checksum"].byte_offset, 155)
        self.assertEqual(layout.JOURNAL_RECORD_FIELDS["jsn_counter"].byte_offset, 20)
        self.assertEqual(layout.LOCATION_ENTRY_FIELDS["ciphertext_checksum"].byte_offset, 10)
        self.assertEqual(layout.SYSTEM_CONFIGURATION_FIELDS["fixed_structure_slot_spacing"].byte_offset, 429)

    def test_the_three_self_verifying_structures_carry_the_magic_bytes_the_layout_lines_give(self):
        # layout/01-first-txn.md 第 109 行 SFSB = 53 46 53 42；第 369 行 SFSR = 53 46 53 52；第 338 行 SFSJ = 53 46 53 4A
        self.assertEqual(layout.SYSTEM_CONFIGURATION_MAGIC, bytes([0x53, 0x46, 0x53, 0x42]))
        self.assertEqual(layout.ROOT_RECORD_MAGIC, bytes([0x53, 0x46, 0x53, 0x52]))
        self.assertEqual(layout.JOURNAL_RECORD_MAGIC, bytes([0x53, 0x46, 0x53, 0x4A]))
        self.assertEqual(layout.SYSTEM_CONFIGURATION_MAGIC + layout.ROOT_RECORD_MAGIC + layout.JOURNAL_RECORD_MAGIC, b"SFSBSFSRSFSJ")

    def test_gap_table_and_gap_module_list_the_same_identifiers(self):
        table_path = os.path.join(os.path.dirname(__file__), "spec-gaps.tsv")
        with open(table_path, encoding="utf-8") as handle:
            rows = [line.split("\t") for line in handle.read().splitlines()[1:] if line.strip()]
        self.assertEqual([row[0] for row in rows], list(spec_gaps.ALL_GAP_IDENTIFIERS))
        self.assertTrue(all(len(row) == 9 for row in rows), "每行九列（第九列是已由 kb 补的依据）")
        self.assertTrue(all(row[4] in ("两种读法", "缺一句", "只有读 crates 才写得下去", "已由 kb 补（2026-10-02）") for row in rows))
        filled_by_kb = [row[0] for row in rows if row[4] == "已由 kb 补（2026-10-02）"]
        self.assertEqual(filled_by_kb, ["G01", "G02", "G03"])
        self.assertTrue(all(row[8] != "-" for row in rows if row[0] in filled_by_kb), "已由 kb 补的三行第九列写 kb 行")
        self.assertTrue(all(row[8] == "-" for row in rows if row[0] not in filled_by_kb))
        # 格式补丁（登记 12.5 格五）：读法 11 最后一条的新欠一句接在已记的那几行之后，记 G23
        self.assertEqual([row[0] for row in rows][-3:], ["G21", "G22", "G23"], "写执行器时新发现的欠一句从 G21 起编")
        self.assertEqual(len(rows), 23)

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
    def test_all_persisted_state_is_in_and_strict_run_gives_the_verdict_with_no_gap_since_the_kb_fills_the_magic_and_offset_gaps(self):
        judgement = self.judge(ALL_PERSISTED)
        self.assertEqual(judgement.provisional_verdict, VERDICT_IN)
        self.assertEqual(judgement.strict_verdict, VERDICT_IN)
        self.assertEqual(judgement.gaps_hit, [])
        self.assertEqual(judgement.recovery_summary["slot_one"], "d0:r1:s1@4096g5,d1:r1:s1@4096g5")

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

    def test_torn_slot_zero_on_one_device_sends_that_device_to_the_second_step_for_slot_one(self):
        judgement = self.judge(ACQUIRE_SLOT_ZERO_TORN_ON_DEVICE_ZERO)
        self.assertEqual(judgement.gaps_hit, [])
        self.assertEqual(judgement.recovery_summary["slot_one"], "d0:r2:s1@4096g1,d1:r1:s0@0g1")
        self.assertEqual((judgement.recovery_summary["slot_one_route_one_disks"], judgement.recovery_summary["slot_one_route_two_disks"],
                          judgement.recovery_summary["slot_one_route_three_disks"]), (1, 1, 0))
        self.assertTrue(judgement.landed_text.startswith("(0,0,"))
        self.assertEqual(judgement.provisional_verdict, VERDICT_IN)
        self.assertEqual(judgement.recovery_summary["torn"], 1)


def flipped(content: bytes, byte_offset: int) -> bytes:
    return content[:byte_offset] + bytes([content[byte_offset] ^ 0x01]) + content[byte_offset + 1:]


class PositiveControlsOnTheAllPersistedState(SyntheticPoolFixture):
    """阳性对照的单测小池版（登记 5.5 P 行、N 行的同形改坏）：P 型判「不在」，N 型判「在」。"""

    def record_patches(self, records: list[synthetic.RecordFields], magic: bytes = b"SFSJ") -> list[tuple[int, int, bytes]]:
        patches = []
        for fields in records:
            content = synthetic.journal_record(fields, magic=magic)
            patches += [(device, synthetic.journal_slot_offset(fields.counter), content) for device in synthetic.DEVICES]
        return patches

    def forged_continuation(self, counters: list[int], flags: list[bool], watermark_increase: int = 1000, magic: bytes = b"SFSJ"):
        new_root = synthetic.RootFields(1, 4, 3, 3, 65548 + watermark_increase)
        previous = self.parts["records"][3]
        forged = []
        for ordinal, (counter, flag) in enumerate(zip(counters, flags), start=1):
            fields = synthetic.RecordFields(1, counter, 4, ordinal, flag, 0, 1, (), new_root, synthetic.chain_value_of(previous))
            forged.append(fields)
            previous = synthetic.journal_record(fields, magic=magic)
        return self.record_patches(forged, magic)

    def test_u1a_a_slot_with_a_wrong_magic_and_a_resealed_checksum_is_not_usable_and_the_other_slot_of_the_disk_is_chosen(self):
        # C1：layout/01-first-txn.md 第 109 行、第 160 行「自证过（magic、整槽校验和、…都过）」
        resealed_wrong_magic = synthetic.system_configuration_slot(0, 5, 3, 1, overrides={"magic": b"SFSX"})
        recovery, _verdict = self.recover_patched(ALL_PERSISTED, [(0, synthetic.SPACING, resealed_wrong_magic)])
        chosen = recovery.pool.chosen_slot_by_device[0]
        self.assertEqual((chosen.slot_number, chosen.unsigned("slot_generation")), (0, 4))
        self.assertEqual(recovery.pool.chosen_slot_by_device[1].unsigned("slot_generation"), 5)

    def test_u1b_a_forged_root_with_a_wrong_magic_and_a_resealed_checksum_does_not_enter_the_candidates(self):
        # C1：layout/01-first-txn.md 第 369 行；伪根 txg 比最新根大，magic 对时择到它（见 P2）
        device, byte_offset = synthetic.root_slot_of_txg(6)
        forged = synthetic.root_record(synthetic.RootFields(1, 6, 3, 3, 65548), magic=b"SFSX")
        recovery, verdict = self.recover_patched(ALL_PERSISTED, [(device, byte_offset, forged)])
        self.assertEqual(recovery.chosen_root.unsigned("checkpoint_txg"), 3)
        self.assertEqual((recovery.landed.instance, recovery.landed.checkpoint_txg), (1, 3))
        self.assertEqual(verdict, VERDICT_IN)

    def test_u1c_a_forged_record_with_a_wrong_magic_and_a_resealed_checksum_does_not_extend_the_chain(self):
        # C1：layout/01-first-txn.md 第 338 行；magic 对时这条伪记录接上链、落点变（见 P3）
        recovery, verdict = self.recover_patched(ALL_PERSISTED, self.forged_continuation([4], [True], magic=b"SFSX"))
        self.assertEqual(recovery.replay.stop_reason, "jsn_not_readable_or_gap")
        self.assertEqual(recovery.replay.applied_records, [])
        self.assertEqual(recovery.landed.checkpoint_txg, 3)
        self.assertEqual(verdict, VERDICT_IN)

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

    def test_p8_on_the_all_persisted_base_lands_version_two_which_reading_three_allows_so_the_verdict_is_in(self):
        """第 1 次重跑登记 5.5 的修订：P8 期望「在，L = v_{n−1}」。读法 3：全持久态上末次调用的返回点大于 m(c)，ack = n − 1，
        v_{n−1} 在 A(c) 里，判「在」，落点 v_{n−1}。"""
        device, byte_offset = synthetic.root_slot_of_txg(3)
        root_three = synthetic.root_record(self.parts["roots"][3])
        record_three = self.parts["records"][3]
        patches = [(device, byte_offset, flipped(root_three, 122))]
        patches += [(each_device, synthetic.journal_slot_offset(3), flipped(record_three, 487)) for each_device in synthetic.DEVICES]
        recovery, verdict = self.recover_patched(ALL_PERSISTED, patches)
        self.assertEqual(recovery.landed.checkpoint_txg, 2)
        self.assertEqual(verdict, VERDICT_IN)

    def test_p9_a_landing_that_shares_instance_and_txg_with_an_allowed_version_but_not_its_projection_is_not_in(self):
        """P9 的单测小池同形：版本 3 的根没落盘，写序号 0–12 持久，A(c) = 版本 2、3；盘上伪造一条 (实例 1, txg 3) 的根，
        水位比版本 3 的大 1000，择到它，落点的 (实例, txg) 与版本 3 相同而投影不同 ⇒ 不在（D13 已定项 1 M2 按投影比）。"""
        landing_vector = "2" * 13 + "0" * 8
        device, byte_offset = synthetic.root_slot_of_txg(3)
        forged = synthetic.root_record(synthetic.RootFields(1, 3, 3, 3, 65548 + 1000))
        recovery, verdict = self.recover_patched(landing_vector, [(device, byte_offset, forged)])
        self.assertEqual((recovery.landed.instance, recovery.landed.checkpoint_txg, recovery.landed.tree_identifier_watermark), (1, 3, 66548))
        self.assertEqual(verdict, VERDICT_NOT_IN)

    def test_a_record_of_the_abandoned_instance_with_a_smaller_txg_still_counts_as_above_the_root_watermark_by_instance_first(self):
        """D23 已定项 14 注 2：记录水位 (实例代号, checkpoint_txg) 按实例代号为主比。根 3 的实例代号改成 0（P5 型），记录 3 两份改坏：
        水位之上剩记录 1、2（实例 1），链首是记录 1，它的实例代号与所选根不同 ⇒ 停在实例边界；改成 txg 为主比，水位之上一条都没有。"""
        device, byte_offset = synthetic.root_slot_of_txg(3)
        lowered_root = synthetic.root_record(synthetic.RootFields(0, 3, 3, 3, 65548))
        record_three = self.parts["records"][3]
        patches = [(device, byte_offset, lowered_root)]
        patches += [(each_device, synthetic.journal_slot_offset(3), flipped(record_three, 487)) for each_device in synthetic.DEVICES]
        recovery, _verdict = self.recover_patched(ALL_PERSISTED, patches)
        self.assertEqual(recovery.replay.stop_reason, "head_belongs_to_another_instance")

    def test_a_landing_below_the_acknowledged_version_is_not_in_because_a_c_has_a_lower_bound(self):
        """P10 的单测小池同形（登记 5.5）：写序号 0–12 持久（暖机那次调用的写全持久、发布那次调用的第一次单元写也持久），
        所以 ack = 版本 2（暖机那次调用返回了）；把根 2 与记录 2 两份都改坏，落点是版本 1，在下界之下 ⇒ 不在。"""
        landing_vector = "2" * 13 + "0" * 8
        allowed = allowed_versions(self.context.version_table, landing_vector)
        self.assertEqual((allowed.acknowledged_version_index, allowed.begun_version_index), (2, 3))
        device, byte_offset = synthetic.root_slot_of_txg(2)
        root_two = synthetic.root_record(self.parts["roots"][2])
        record_two = self.parts["records"][2]
        patches = [(device, byte_offset, flipped(root_two, 122))]
        patches += [(each_device, synthetic.journal_slot_offset(2), flipped(record_two, 487)) for each_device in synthetic.DEVICES]
        recovery, verdict = self.recover_patched(landing_vector, patches)
        self.assertEqual((recovery.landed.instance, recovery.landed.checkpoint_txg), (1, 1))
        self.assertEqual(verdict, VERDICT_NOT_IN)

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
        self.assertEqual(gaps, ["G05"])

    def test_g06_root_with_a_foreign_identity_changes_the_choice(self):
        device, byte_offset = synthetic.root_slot_of_txg(6)
        foreign = synthetic.root_record(synthetic.RootFields(1, 6, 3, 3, 65548), pool_identity=bytes(16))
        recovery, gaps = self.gaps_after([(device, byte_offset, foreign)])
        self.assertEqual(recovery.chosen_root.choice_key, (3, 1))
        self.assertEqual(gaps, ["G06"])

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
        self.assertEqual(gaps, ["G08"])

    def test_g20_second_last_flag_in_the_root_publish_stops_recovery(self):
        second = synthetic.journal_record(synthetic.RecordFields(1, 4, 3, 2, True, 1, 1, (), self.parts["roots"][3],
                                                                 synthetic.chain_value_of(self.parts["records"][3])))
        recovery, gaps = self.gaps_after([(device, synthetic.journal_slot_offset(4), second) for device in synthetic.DEVICES])
        self.assertEqual(recovery.landed, None)
        self.assertIn("G20", gaps)


# ---- 格式补丁（E170 第 2 次重跑登记 12.5）：单测 U-F1–U-F5 与 G23 ----
# 期望照 D23 已定项 4、10、15、17 的正文推（读法 11）；登记 12.8 X32–X36 的会红点在这里。
import journal_ring  # noqa: E402
from root_ring import VersionProjection  # noqa: E402


def resealed_record(record: bytes, field_name: str, value: int) -> bytes:
    buffer = bytearray(record)
    synthetic.put_field(buffer, layout.JOURNAL_RECORD_FIELDS[field_name], value)
    synthetic.seal_self_checksum(buffer, layout.JOURNAL_RECORD_FIELDS["header_checksum"])
    return bytes(buffer)


class JournalHeaderOf487Bytes(SyntheticPoolFixture):
    def record_three_fields(self, new_root: synthetic.RootFields, back_chain: int, named_items=None) -> synthetic.RecordFields:
        items = named_items if named_items is not None else (
            synthetic.named_item(synthetic.DATA_UNIT_SLOT, 1, self.parts["data_unit"], 3),
            synthetic.named_item(synthetic.NODE_UNIT_SLOT, 2, self.parts["node_unit"], 3))
        return synthetic.RecordFields(1, 3, 3, 1, True, 1, 1, items, new_root, back_chain)

    def verdict_of_record(self, raw: bytes) -> str:
        return journal_ring.verify_record(raw, synthetic.RECORD_BYTES, synthetic.POOL_IDENTITY[:8], spec_gaps.GapsHitInOneState())

    def as_record_three_on_both_devices(self, record: bytes) -> list[tuple[int, int, bytes]]:
        return [(device, synthetic.journal_slot_offset(3), record) for device in synthetic.DEVICES]

    def test_u_f1_the_header_offsets_are_those_of_the_sentence_in_d23_item_4(self):
        # D23 已定项 4「事务号 78、提交标记 86、本次发布内序号 87、反向链 91、载荷校验和 95、新根段 99、fsid 463、MAC 471、头末 487」
        fields = layout.JOURNAL_RECORD_FIELDS
        for field_name, written_offset in {"transaction_number": 78, "commit_marker": 86, "ordinal_within_publish": 87, "back_chain": 91,
                                           "payload_checksum": 95, "new_root_tree_table_pointer": 99,
                                           "file_system_identity_low_eight_bytes": 463, "message_authentication_code": 471}.items():
            self.assertEqual(fields[field_name].byte_offset, written_offset, field_name)
        self.assertEqual((fields["message_authentication_code"].end_offset, layout.JOURNAL_HEADER_BYTES), (487, 487))
        # D23 已定项 15：新根段 364 = 86 + 86 + 8 + 8 + 86 + 86 + 4，按列举序累加
        self.assertEqual(fields["file_system_identity_low_eight_bytes"].byte_offset, fields["new_root_tree_table_pointer"].byte_offset + 364)
        self.assertEqual([fields[name].byte_offset for name in ("new_root_tree_table_pointer", "new_root_central_mapping_root_pointer",
                                                                 "new_root_tree_identifier_watermark", "new_root_rollback_floor",
                                                                 "new_root_instance_table_pointer", "new_root_allocation_records_root_pointer",
                                                                 "new_root_unmount_marker")], [99, 185, 271, 279, 287, 373, 459])
        # D23 已定项 4「⌊(4096 − 487) ÷ 56⌋」与已定项 17「4096 记录装 64 个点名项」；512 扇区上余 25
        self.assertEqual((4096 - layout.JOURNAL_HEADER_BYTES) // layout.NAMED_ITEM_BYTES, 64)
        self.assertEqual(layout.JOURNAL_HEADER_BYTES + 25, 512)

    def test_u_f2_a_replayed_record_replaces_all_seven_fields_of_the_chosen_root_and_the_projection_keeps_six(self):
        # D23 已定项 15「把所选根的这七个字段换成记录里的（实例表指针、卸载记号、分配记录树根从新根段来，不再照抄所选根）」
        new_root = synthetic.RootFields(1, 3, 3, 3, 65548, rollback_floor=5, flags=1, instance_table_seed=7, allocation_records_seed=8)
        record = synthetic.journal_record(self.record_three_fields(new_root, synthetic.chain_value_of(self.parts["records"][2])))
        recovery, _verdict = self.recover_patched(RECORD_THREE_PERSISTED_ROOT_THREE_NOT, self.as_record_three_on_both_devices(record))
        chosen = recovery.chosen_root
        self.assertEqual(chosen.choice_key, (2, 1))
        self.assertEqual(len(recovery.replay.applied_records), 1)
        rebuilt = recovery.replay.rebuilt_root
        pointer = synthetic.pointer_bytes
        self.assertEqual((rebuilt.instance, rebuilt.checkpoint_txg, rebuilt.tree_table_pointer, rebuilt.central_mapping_root_pointer,
                          rebuilt.tree_identifier_watermark, rebuilt.rollback_floor, rebuilt.instance_table_pointer,
                          rebuilt.allocation_records_root_pointer, rebuilt.unmount_marker),
                         (1, 3, pointer(3), pointer(3), 65548, 5, pointer(7), pointer(8), 1))
        chosen_seven = (chosen.field_bytes("tree_table_pointer"), chosen.field_bytes("central_mapping_root_pointer"),
                        chosen.unsigned("tree_identifier_watermark"), chosen.unsigned("rollback_floor"), chosen.field_bytes("instance_table_pointer"),
                        chosen.field_bytes("allocation_records_root_pointer"), chosen.unsigned("flags") & 1)
        rebuilt_seven = (rebuilt.tree_table_pointer, rebuilt.central_mapping_root_pointer, rebuilt.tree_identifier_watermark,
                         rebuilt.rollback_floor, rebuilt.instance_table_pointer, rebuilt.allocation_records_root_pointer, rebuilt.unmount_marker)
        self.assertTrue(all(before != after for before, after in zip(chosen_seven, rebuilt_seven)), "七个字段都与所选根不同")
        self.assertEqual(recovery.landed, VersionProjection(1, 3, pointer(3), pointer(3), 65548, 5))

    def test_u_f3_a_record_written_in_the_311_byte_header_format_does_not_self_verify_and_the_chosen_root_stays(self):
        fields = self.record_three_fields(self.parts["roots"][3], synthetic.chain_value_of(self.parts["records"][2]))
        record = synthetic.journal_record_in_the_311_byte_header_format(fields)
        self.assertNotEqual(self.verdict_of_record(record), journal_ring.RECORD_VALID)
        recovery, _verdict = self.recover_patched(RECORD_THREE_PERSISTED_ROOT_THREE_NOT, self.as_record_three_on_both_devices(record))
        self.assertEqual(recovery.replay.applied_records, [])
        self.assertEqual((recovery.landed.instance, recovery.landed.checkpoint_txg), (1, 2))

    def test_u_f4_sixty_four_named_items_fit_a_record_and_a_record_claiming_sixty_five_is_corrupt(self):
        items = tuple(synthetic.named_item(100 + index, 2, self.parts["node_unit"], 3) for index in range(64))
        fields = self.record_three_fields(self.parts["roots"][3], synthetic.chain_value_of(self.parts["records"][2]), named_items=items)
        record_with_sixty_four = synthetic.journal_record(fields)
        self.assertEqual(self.verdict_of_record(record_with_sixty_four), journal_ring.RECORD_VALID)
        record_claiming_sixty_five = resealed_record(record_with_sixty_four, "named_item_count", 65)
        self.assertEqual(self.verdict_of_record(record_claiming_sixty_five), journal_ring.RECORD_CORRUPT)
        recovery, _verdict = self.recover_patched(RECORD_THREE_PERSISTED_ROOT_THREE_NOT,
                                                  self.as_record_three_on_both_devices(record_claiming_sixty_five))
        self.assertEqual(recovery.replay.stop_reason, "jsn_not_readable_or_gap")
        self.assertEqual((recovery.landed.instance, recovery.landed.checkpoint_txg), (1, 2))

    def test_u_f5_a_back_chain_computed_over_the_first_311_bytes_of_the_previous_header_stops_the_chain(self):
        # D23 已定项 10「覆盖前一条记录的整个记录头」：整个 = 487 字节（头校验和 [46, 78) 按零，已定项 19 ②）
        record_two = self.parts["records"][2]
        chain_over_311_bytes = crc32c(record_two[:46] + bytes(32) + record_two[78:311])
        record = synthetic.journal_record(self.record_three_fields(self.parts["roots"][3], chain_over_311_bytes))
        recovery, _verdict = self.recover_patched(RECORD_THREE_PERSISTED_ROOT_THREE_NOT, self.as_record_three_on_both_devices(record))
        self.assertEqual((recovery.replay.stop_reason, recovery.replay.applied_records), ("back_chain_mismatch", []))
        self.assertEqual((recovery.landed.instance, recovery.landed.checkpoint_txg), (1, 2))
        self.assertEqual(synthetic.chain_value_of(record_two), crc32c(record_two[:46] + bytes(32) + record_two[78:487]))

    def test_g23_a_record_whose_unmount_marker_has_a_bit_other_than_bit_zero_is_corrupt_and_hits_the_gap_row(self):
        for marker, expected_gaps, record_three_readable in ((1, [], True), (2, ["G23"], False)):
            record = resealed_record(self.parts["records"][3], "new_root_unmount_marker", marker)
            image = build_crash_image(self.dump, self.context.prepared_writes, ALL_PERSISTED)
            for device, byte_offset, content in self.as_record_three_on_both_devices(record):
                image.write_bytes(device, byte_offset, content)
            gaps = spec_gaps.GapsHitInOneState()
            recovery = recover(image, self.dump.device_identities, self.dump.sector_bytes, UnitChecksumCache(), gaps)
            self.assertEqual(gaps.sorted_identifiers(), expected_gaps, marker)
            self.assertEqual(3 in recovery.ring_scan.records_by_counter, record_three_readable, marker)
            self.assertEqual((recovery.landed.instance, recovery.landed.checkpoint_txg), (1, 3), marker)


SPACING_BIG = 8192  # 槽距 8192：自述槽距要与它自述的根槽距、根环起点一致（D22 已定项 16 第 2 句、已定项 2），单测小池的几何在收的范围里


def slot_image(placements: dict[int, list[tuple[int, bytes]]], device_count: int = 2) -> SparseCrashImage:
    image = SparseCrashImage({device: {} for device in range(device_count)})
    for device, items in placements.items():
        for byte_offset, content in items:
            image.write_bytes(device, byte_offset, content)
    return image


def slot_with(device: int, generation: int, spacing: int = 4096, **overrides) -> bytes:
    return synthetic.system_configuration_slot(device, generation, 0, 1, overrides={"fixed_structure_slot_spacing": spacing} | overrides)


class SystemConfigurationSlotOneRoutes(unittest.TestCase):
    """C3、C4：槽 1 的偏移按 ① ② ③ 取（layout/01-first-txn.md 第 160 行、D22 已定项 16 读槽 1 的偏移那一句）。期望照 kb 句子推。"""

    def choose(self, image: SparseCrashImage, device_identities: list[int] | None = None):
        gaps = spec_gaps.GapsHitInOneState()
        pool = choose_system_configuration(image, [0, 1] if device_identities is None else device_identities, gaps)
        return pool, gaps.sorted_identifiers()

    def test_u2_a_device_without_a_usable_slot_zero_reads_slot_one_at_the_spacing_the_first_usable_slot_zero_of_the_pool_gives(self):
        # 盘 0 槽 0 自证不过；盘 1 槽 0 可用、自述槽距 8192；盘 0 的槽 1 在 8192；盘 0 偏移 4096 另放一个自证过的诱饵槽（槽距 4096、根环起点 65、世代号更大）
        image = slot_image({0: [(0, flipped(slot_with(0, 3, SPACING_BIG), 300)), (SPACING_BIG, slot_with(0, 4, SPACING_BIG)),
                                (4096, slot_with(0, 9, 4096, root_ring_base_slot=65))],
                            1: [(0, slot_with(1, 3, SPACING_BIG)), (SPACING_BIG, slot_with(1, 4, SPACING_BIG))]})
        pool, gaps = self.choose(image)
        self.assertNotIsInstance(pool, PoolRefusal, "不读诱饵：诱饵槽的根环起点越界，读到就整池拒")
        reading = pool.slot_one_reading_by_device
        self.assertEqual(reading[0].route, SLOT_ONE_ROUTE_FIRST_USABLE_SLOT_ZERO_OF_THE_POOL)
        self.assertEqual((reading[0].spacing_read, reading[0].chosen_offset), (SPACING_BIG, SPACING_BIG))
        self.assertEqual(reading[1].route, SLOT_ONE_ROUTE_OWN_SLOT_ZERO)
        self.assertEqual((pool.chosen_slot_by_device[0].byte_offset, pool.chosen_slot_by_device[0].unsigned("slot_generation")), (SPACING_BIG, 4))
        self.assertEqual(gaps, [])

    def test_u3_when_no_slot_zero_is_usable_the_largest_generation_among_the_slots_collected_at_every_spacing_is_chosen(self):
        placements = {device: [(0, flipped(slot_with(device, 3, SPACING_BIG), 300)), (4096, slot_with(device, 2, 4096)),
                               (SPACING_BIG, slot_with(device, 4, SPACING_BIG))] for device in (0, 1)}
        pool, gaps = self.choose(slot_image(placements))
        for device in (0, 1):
            reading = pool.slot_one_reading_by_device[device]
            self.assertEqual((reading.route, reading.collected_offsets), (SLOT_ONE_ROUTE_EVERY_SPACING, (4096, SPACING_BIG)))
            chosen = pool.chosen_slot_by_device[device]
            self.assertEqual((chosen.byte_offset, chosen.unsigned("slot_generation")), (SPACING_BIG, 4))
        self.assertEqual(gaps, [])

    def test_u4_two_collected_slots_with_the_same_generation_choose_the_smaller_offset(self):
        placements = {device: [(0, flipped(slot_with(device, 3, SPACING_BIG), 300)), (4096, slot_with(device, 3, 4096)),
                               (SPACING_BIG, slot_with(device, 3, SPACING_BIG))] for device in (0, 1)}
        pool, gaps = self.choose(slot_image(placements))
        for device in (0, 1):
            self.assertEqual(pool.chosen_slot_by_device[device].byte_offset, 4096)
        self.assertEqual(gaps, [])

    def test_u5_a_slot_collected_at_a_spacing_that_is_self_verified_but_out_of_range_refuses_the_whole_pool(self):
        # 越界的槽世代号比合规的小：不整池拒的话择到的是 4096 那一槽，落点正常（这一格对「去掉出口一」才敏感）
        placements = {device: [(0, flipped(slot_with(device, 3, SPACING_BIG), 300)), (4096, slot_with(device, 5, 4096)),
                               (SPACING_BIG, slot_with(device, 3, SPACING_BIG, root_ring_base_slot=65))] for device in (0, 1)}
        pool, _gaps = self.choose(slot_image(placements))
        self.assertIsInstance(pool, PoolRefusal)
        self.assertTrue(pool.reason.startswith("system_configuration_value_refused:root_ring_base_not_first_version_slot"))

    def test_u6_a_slot_with_an_unrecognised_incompat_bit_is_not_choosable_and_does_not_refuse_the_pool(self):
        unknown_incompat_bit_eight = bytes([0x02, 0x01]) + bytes(94)
        placements = {device: [(0, flipped(slot_with(device, 3, SPACING_BIG), 300)), (4096, slot_with(device, 3, 4096)),
                               (SPACING_BIG, slot_with(device, 5, SPACING_BIG, feature_bits=unknown_incompat_bit_eight))] for device in (0, 1)}
        pool, gaps = self.choose(slot_image(placements))
        self.assertNotIsInstance(pool, PoolRefusal)
        for device in (0, 1):
            self.assertEqual(pool.chosen_slot_by_device[device].byte_offset, 4096)
            self.assertEqual(pool.slot_one_reading_by_device[device].collected_offsets, (4096,))
        self.assertEqual(gaps, ["G14"], "G14（incompat 不认得的槽不可择还是整池拒）照旧记撞上，今天的 kb 只对逐档那一步写了「不可择」")

    def test_all_three_routes_report_themselves_in_the_trace(self):
        usable_everywhere = slot_image({device: [(0, slot_with(device, 3)), (4096, slot_with(device, 4))] for device in (0, 1)})
        pool, _gaps = self.choose(usable_everywhere)
        self.assertEqual({reading.route for reading in pool.slot_one_reading_by_device.values()}, {SLOT_ONE_ROUTE_OWN_SLOT_ZERO})
        nothing_usable = slot_image({device: [(4096, slot_with(device, 4))] for device in (0, 1)})
        pool, _gaps = self.choose(nothing_usable)
        self.assertEqual({reading.route for reading in pool.slot_one_reading_by_device.values()}, {SLOT_ONE_ROUTE_EVERY_SPACING})

    def test_g21_the_device_order_matters_only_when_usable_slot_zeros_disagree_on_the_spacing(self):
        # 三块盘：盘 0 与盘 1 的槽 0 可用但槽距不同（4096 与 8192），盘 2 的槽 0 不可用（走 ②）
        placements = {0: [(0, slot_with(0, 3, 4096)), (4096, slot_with(0, 4, 4096))],
                      1: [(0, slot_with(1, 3, SPACING_BIG)), (SPACING_BIG, slot_with(1, 4, SPACING_BIG))],
                      2: [(0, flipped(slot_with(2, 3, 4096), 300)), (4096, slot_with(2, 4, 4096))]}
        image = slot_image(placements, device_count=3)
        _pool, gaps_when_listed_in_identity_order = self.choose(image, [0, 1, 2])
        self.assertNotIn("G21", gaps_when_listed_in_identity_order)
        _pool, gaps_when_listed_otherwise = self.choose(image, [1, 0, 2])
        self.assertIn("G21", gaps_when_listed_otherwise)

    def test_g22_out_of_range_slot_whose_spacing_is_not_its_offset_is_the_case_the_two_orders_of_the_two_sentences_part_on(self):
        out_of_range_at_the_wrong_offset = slot_with(0, 5, 4096, root_ring_base_slot=65)
        placements = {device: [(0, flipped(slot_with(device, 3, 4096), 300)), (4096, slot_with(device, 3, 4096)),
                               (SPACING_BIG, out_of_range_at_the_wrong_offset)] for device in (0, 1)}
        pool, gaps = self.choose(slot_image(placements))
        self.assertIsInstance(pool, PoolRefusal, "暂代读法：先判越界、整池拒")
        self.assertIn("G22", gaps)
        in_range_at_the_wrong_offset = slot_with(0, 5, 4096)
        placements = {device: [(0, flipped(slot_with(device, 3, 4096), 300)), (4096, slot_with(device, 3, 4096)),
                               (SPACING_BIG, in_range_at_the_wrong_offset)] for device in (0, 1)}
        pool, gaps = self.choose(slot_image(placements))
        self.assertNotIsInstance(pool, PoolRefusal)
        self.assertEqual(gaps, [], "槽距不等于所在偏移、字段又都在范围里：不收，也不记撞上")
        self.assertEqual(pool.slot_one_reading_by_device[0].collected_offsets, (4096,))


class VersionComparisonInTheJudgement(unittest.TestCase):
    """judge_state 里「落点在不在允许集合」按投影比（D13 已定项 1 M2：返回的根的 (实例代号, checkpoint_txg) 与新根段四个字段）。
    P9 的同形：伪造的根与版本 3 同 (实例, txg)、水位不同，放在基镜像里，让落点走 judge_state 的比对。"""

    def test_a_landing_that_shares_instance_and_txg_with_an_allowed_version_but_not_its_projection_is_judged_not_in(self):
        stream, _parts = synthetic.new_pool_file_creation_stream()
        device, byte_offset = synthetic.root_slot_of_txg(3)
        stream.seed_base(device, byte_offset, synthetic.root_record(synthetic.RootFields(1, 3, 3, 3, 65548 + 1000)))
        landing_vector = "2" * 13 + "0" * 8
        with tempfile.TemporaryDirectory(dir=SCRATCH_ROOT) as directory:
            synthetic.write_dump(directory, stream, [landing_vector])
            dump = load_stream_dump(directory)
            context = prepare_stream_context(dump)
            judgement = judge_state(context, dump.states[0], UnitChecksumCache())
        self.assertEqual([(version.projection.instance, version.projection.checkpoint_txg) for version in judgement.allowed.allowed],
                         [(1, 2), (1, 3)])
        self.assertTrue(judgement.landed_text.startswith("(1,3,wm=66548"))
        self.assertEqual(judgement.provisional_verdict, VERDICT_NOT_IN)
        self.assertEqual(judgement.strict_verdict, VERDICT_NOT_IN)


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
            self.assertIn("strict_spec_gap_states=0", verdict)
            self.assertIn("strict_given_states=4", verdict)
            self.assertIn("provisional_in=4", verdict)
            self.assertIn("digest_mismatches=0", verdict)
            path_counts = [line for line in single if line.startswith("name=path_counts ")][0]
            self.assertIn("slot_one_route_one_disk_states=7", path_counts)  # 四态 × 两盘 = 8 盘次，其中撕裂那一态的盘 0 走 ②
            self.assertIn("slot_one_route_two_disk_states=1", path_counts)
            self.assertIn("slot_one_route_three_disk_states=0", path_counts)
            assertions = {line.split()[1]: line.split()[3] for line in single if line.startswith("name=anchor_assertion ")}
            # A7 在单测小池上是 false：小池的记录 3 点名 2 项，不是流一的 12 项（A7 钉的是流一的字节表）
            self.assertEqual(assertions, {"id=A6": "holds=true", "id=A1": "holds=true", "id=A2": "holds=true",
                                          "id=A3": "holds=true", "id=A4": "holds=true", "id=A5": "holds=true",
                                          "id=A13": "holds=true", "id=A12": "holds=true", "id=A7": "holds=false"})
            torn = [line for line in single if line.startswith("name=torn_reading ")][0]
            self.assertIn("differing_entries=0", torn)
            freeze = [line for line in single if line.startswith("name=freeze ")]
            self.assertTrue(any("file=executor_main.py" in line for line in freeze))
            refused_exit, _lines = self.run_driver(dumps_directory, os.path.join(directory, "single.out"), 1)
            self.assertEqual(refused_exit, 2)

    def test_u8_a_state_that_triggers_a_gap_row_is_judged_spec_gap_and_the_batch_summary_counts_it(self):
        # G19（D22 已定项 16「择槽取校验和过且世代号最大的」只写了取大）：盘 0 两槽世代号相同而内容不同（槽 1 的回退下界 F 与槽 0 不同）
        stream, _parts = synthetic.new_pool_file_creation_stream()
        stream.seed_base(0, synthetic.SPACING, synthetic.system_configuration_slot(0, 1, 0, 0, rollback_floor=7))
        nothing_persisted = "0" * 21
        with tempfile.TemporaryDirectory(dir=SCRATCH_ROOT) as directory:
            dumps_directory = os.path.join(directory, "dumps")
            synthetic.write_dump(os.path.join(dumps_directory, "new_pool_file_creation"), stream, [nothing_persisted, ALL_PERSISTED],
                                 stream_name="new_pool_file_creation")
            exit_code, lines = self.run_driver(dumps_directory, os.path.join(directory, "single.out"), 1)
            self.assertEqual(exit_code, 0)
            state_lines = [line for line in lines if line.startswith("name=state ")]
            self.assertEqual(sorted((line.split()[2], [field for field in line.split() if field.startswith("strict=")][0],
                                     [field for field in line.split() if field.startswith("gaps=")][0]) for line in state_lines),
                             [("state=0", "strict=spec_gap", "gaps=G19"), ("state=1", "strict=in", "gaps=-")])
            verdict = [line for line in lines if line.startswith("name=verdict ")][0]
            self.assertIn("strict_spec_gap_states=1", verdict)
            self.assertIn("strict_spec_gap_identifiers=G19", verdict)
            self.assertIn("strict_given_states=1", verdict)
            self.assertIn("provisional_in=2", verdict)
            trajectory = [line for line in lines if line.startswith("name=gap_trajectory ")][0]
            self.assertIn("segments=1 peak=1 positive_segments=1 last_segment_value=1", trajectory)


if __name__ == "__main__":
    unittest.main()
