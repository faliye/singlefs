"""参照谓词 W_ref、T_ref 的手造真假（`research/prompts/e170-r2-prereg.md` 5.1「比对器自己的单测里各有一个手造的真、一个手造的假」）。
期望照登记读法 2、3 与 layout 的字段表推，不照任何一版执行器的输出。跑法：python3 -m unittest -v witness_reference_hand_made_cases
文件名以 witness_reference 起头，不进 v2 冻结的 sha256。"""

from __future__ import annotations

import sys
import unittest

sys.dont_write_bytecode = True

import crc32c_castagnoli  # noqa: E402
import witness_reference as reference  # noqa: E402
from crash_image import SparseCrashImage  # noqa: E402


def record(instance: int, counter: int, txg: int, last: bool) -> reference.SelfVerifiedJournalRecord:
    return reference.SelfVerifiedJournalRecord(0, 0, 1, instance, counter, txg, last)


def rotation(version: int, tail: int | None) -> reference.PublishRotation:
    return reference.PublishRotation(version, 10 * version, (), () if tail is None else (tail,), tail, tail, 0)


def hand_built_system_configuration_slot(tail: int) -> bytes:
    """layout 第 158 行的偏移：magic 0、格式版本 4、incompat 位图 6、槽距 429、tail 469；整槽校验和 155 按零参与之后填。"""
    content = bytearray(4096)
    content[0:4] = b"SFSB"
    content[4:6] = (1).to_bytes(2, "little")
    content[6] = 0b10  # incompat 位 1
    content[429:433] = (4096).to_bytes(4, "little")
    content[469:477] = tail.to_bytes(8, "little")
    content[155:159] = crc32c_castagnoli.crc32c(bytes(content)).to_bytes(4, "little")
    return bytes(content)


def hand_built_journal_record(flags: int) -> bytes:
    """layout 第 359 行的偏移（311 格式）：magic 0、类型 4、标志 7、长度 8、点名项数 12、实例 16、计数器 20、txg 26、头校验和 46、载荷校验和 95。"""
    content = bytearray(4096)
    content[0:4] = b"SFSJ"
    content[4:6] = (1).to_bytes(2, "little")
    content[7] = flags
    content[8:12] = (4096).to_bytes(4, "little")
    content[16:20] = (1).to_bytes(4, "little")
    content[20:26] = (3).to_bytes(6, "little")
    content[26:34] = (3).to_bytes(8, "little")
    content[95:99] = crc32c_castagnoli.crc32c(b"").to_bytes(4, "little")
    content[46:50] = crc32c_castagnoli.crc32c(bytes(content)).to_bytes(4, "little")
    return bytes(content)


class WitnessReferenceHandMadeCases(unittest.TestCase):
    def test_witness_counter_above_the_last_record_counter_makes_w_true(self):
        result = reference.witness_reference((1, 3), 4, [record(1, 3, 3, True)])
        self.assertEqual((result.branch, result.witness_true), ("last_record", True))

    def test_witness_counter_equal_to_the_last_record_counter_makes_w_false(self):
        result = reference.witness_reference((1, 3), 3, [record(1, 3, 3, True)])
        self.assertEqual((result.branch, result.witness_true), ("last_record", False))

    def test_record_at_the_witness_counter_with_a_larger_key_makes_w_true_and_a_smaller_key_false(self):
        self.assertTrue(reference.witness_reference((1, 3), 4, [record(1, 4, 4, False)]).witness_true)
        self.assertFalse(reference.witness_reference((1, 3), 2, [record(1, 2, 2, True)]).witness_true)

    def test_both_unreadable_makes_w_true_and_a_zero_witness_counter_makes_w_false(self):
        self.assertEqual(reference.witness_reference((1, 3), 5, []).branch, "both_unreadable")
        self.assertTrue(reference.witness_reference((1, 3), 5, []).witness_true)
        self.assertEqual((reference.witness_reference((1, 3), 0, []).branch, reference.witness_reference((1, 3), 0, []).witness_true), ("nothing", False))
        self.assertFalse(reference.witness_reference(None, 5, []).evaluated)

    def test_tail_check_true_at_the_boundary_and_false_one_below(self):
        rotations = {2: rotation(2, 2)}
        self.assertEqual(reference.tail_check_reference((1, 3), 2, 2, rotations).value, "true")
        self.assertEqual(reference.tail_check_reference((1, 3), 2, 1, rotations).value, "false")

    def test_tail_check_not_evaluated_for_version_zero_and_for_a_rotation_without_a_self_verified_slot(self):
        self.assertEqual(reference.tail_check_reference((1, 3), 0, 9, {}).value, "not_evaluated")
        self.assertEqual(reference.tail_check_reference((1, 3), 2, 9, {2: rotation(2, None)}).value, "not_evaluated")
        self.assertEqual(reference.tail_check_reference(None, 2, 9, {2: rotation(2, 2)}).value, "not_evaluated")

    def test_a_hand_built_system_configuration_slot_self_verifies_and_one_flipped_byte_does_not(self):
        image = SparseCrashImage({0: {}})
        image.write_bytes(0, 0, hand_built_system_configuration_slot(7))
        slot = reference.system_configuration_slot_if_self_verified(image, 0, 0)
        self.assertIsNotNone(slot)
        self.assertEqual(slot.journal_tail, 7)
        image.write_bytes(0, 1000, b"\x01")
        self.assertIsNone(reference.system_configuration_slot_if_self_verified(image, 0, 0))

    def test_a_hand_built_journal_record_self_verifies_and_an_unknown_flag_bit_is_damage(self):
        verified = reference.journal_record_if_self_verified(hand_built_journal_record(0x01), 0, 0)
        self.assertEqual((verified.instance, verified.counter, verified.checkpoint_txg, verified.is_last_of_publish), (1, 3, 3, True))
        self.assertIsNone(reference.journal_record_if_self_verified(hand_built_journal_record(0x03), 0, 0))


if __name__ == "__main__":
    unittest.main()
