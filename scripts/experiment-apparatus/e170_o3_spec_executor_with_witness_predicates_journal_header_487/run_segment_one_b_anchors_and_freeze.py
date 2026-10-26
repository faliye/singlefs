"""E170（O3规约执行器第一版） 第 2 次重跑第一段乙（登记 research/prompts/e170-r2-prereg.md 12.8 分段表「第一段乙」）。

executor_main.py（v2-487）判完 Z1 批 1、批 2 之后，往同一份产物末尾追加（只追加，不改已写的行）：
  Z1 导出照 manifest.tsv 核 sha256（登记 5.3、V2；impl-verdicts.tsv 照 12.6 末条不打开，不核）、B8–B10、
  B4-487（每批态行数）与 V11（digest）、B11–B14、B16（= executor_main 打的 B1 那一行）、两条新臂单测、
  格式补丁以外的文件与原目录逐字节相同（12.5「其余文件不许动」）、两份格式补丁在共有文件上相同（V19 同形，附带）、
  v2-487 与 v1-487 各自的冻结行与种子（取法同 5.6：除 comparator*、mutation_driver*、witness_reference*、run_* 之外的文件，
  按文件名排序逐个喂进 sha256，前 16 个十六进制字符）。
文件名以 run_ 开头，不进冻结（5.6）。锚点不符走 F10（两边都查，不作废、不改锚点），这里照打 holds=false。

跑法（仓根，经 capped.sh 与 run-with-memory-cap.sh；PYTHONDONTWRITEBYTECODE=1，不在执行器目录里留 __pycache__）：
  python3 <本文件> --dumps scripts/experiment-apparatus/target/e170-r2-today-dumps/e170-dump --product <executor_main 写的那份产物>
"""

from __future__ import annotations

import argparse
import collections
import difflib
import hashlib
import os
import subprocess
import sys

THIS_DIRECTORY = os.path.dirname(os.path.abspath(__file__))
BIN_DIRECTORY = os.path.dirname(THIS_DIRECTORY)
ARM_DIRECTORY_BY_NAME = {
    "v2_487": THIS_DIRECTORY,
    "v1_487": os.path.join(BIN_DIRECTORY, "e170_second_run_o3_spec_executor_journal_header_487"),
}
ORIGIN_DIRECTORY_BY_ARM = {
    "v2_487": os.path.join(BIN_DIRECTORY, "e170_o3_spec_executor_with_witness_predicates"),
    "v1_487": os.path.join(BIN_DIRECTORY, "e170_second_run_o3_spec_executor"),
}
# 登记 5.6：冻结的文件 = 目录里除这几个前缀之外的全部
FILES_OUTSIDE_THE_FREEZE = ("comparator", "mutation_driver", "witness_reference", "run_")
# 登记 12.5：格式补丁只许动的文件（格一至格五）；其余文件与原目录逐字节相同
FILES_THE_FORMAT_PATCH_MAY_CHANGE = ("on_disk_layout.py", "replay_prefix.py", "journal_ring.py", "synthetic_pool_for_tests.py",
                                     "test_executor_on_synthetic_pool.py", "spec-gaps.tsv", "spec_gaps.py")
FIRST_STREAM = "new_pool_file_creation"
STREAM_NAMES = (FIRST_STREAM, "fixed_script_through_e_then_unmount")
JUDGED_BATCHES = (1, 2)

sys.path.insert(0, THIS_DIRECTORY)
import google_crc32c  # noqa: E402  B8–B10 与 B13 的第二条路子，不经执行器的 crc32c_castagnoli

import crc32c_castagnoli  # noqa: E402
import spec_gaps  # noqa: E402
from crash_dump import load_stream_dump  # noqa: E402
from crash_image import build_crash_image  # noqa: E402
from executor_core import prepare_stream_context, recover  # noqa: E402
from replay_prefix import UnitChecksumCache  # noqa: E402

# 锚点里出自登记的绝对值（12.8 B11–B14；D23 已定项 4、10、15；layout/01-first-txn.md 第 79、325 行；第 216 行位置条目）。
# 这几个数在这里写成字面量，不取执行器 on_disk_layout.py 的常量：锚点要钉的正是执行器读得对不对。
HEADER_BYTES_FROM_D23_ITEM_4 = 487
HEADER_BYTES_OF_THE_OLD_FORMAT = 311  # 只用来自证 B13 的比较分得出头宽
HEADER_CHECKSUM_FIELD_START = 46
HEADER_CHECKSUM_FIELD_END = 78
LOCATION_ENTRY_START_IN_A_POINTER = 50  # layout 第 216 行「头部 50」
SLOT_NUMBER_START_IN_A_LOCATION_ENTRY = 4  # D19 已定项 4：设备 4、槽号 6、密文校验和 4
SLOT_NUMBER_BYTES = 6
JSN_INSTANCE_SPAN = (16, 20)  # D23 已定项 4 十个字段按列举序：jsn 实例代号 4
JSN_COUNTER_SPAN = (20, 26)  # jsn 计数器 6
RECORD_BYTES = 4096
EXPECTED_RING_B11 = [(0, (1, 1), 1), (4096, (1, 2), 2), (8192, (1, 3), 3)]
EXPECTED_RECORD_THREE_FIELDS_B12 = {"named_item_count": 12, "transaction_number": 1, "commit_marker": 1,
                                    "ordinal_within_publish": 1, "last_of_publish": 1, "tree_table_slot": 50252,
                                    "central_mapping_slot": 50251, "tree_identifier_watermark": 65548, "rollback_floor": 0,
                                    "unmount_marker": 0}
EXPECTED_RECORDS_ONE_AND_TWO_FIELDS_B12 = {"tree_table_slot": 50178, "central_mapping_slot": 0,
                                           "tree_identifier_watermark": 65536, "rollback_floor": 0}


def sha256_of_file(path: str) -> str:
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def frozen_file_names(directory: str) -> list[str]:
    return [file_name for file_name in sorted(os.listdir(directory))
            if os.path.isfile(os.path.join(directory, file_name)) and not file_name.startswith(FILES_OUTSIDE_THE_FREEZE)
            and not file_name.endswith(".pyc")]


def freeze_lines_of_arm(arm_name: str) -> tuple[list[str], list[tuple[str, str]]]:
    """取法同登记 5.6（与 executor_main.freeze_lines 同一取法，这里另写一份，按臂名分开打）。"""
    directory = ARM_DIRECTORY_BY_NAME[arm_name]
    seed_hasher = hashlib.sha256()
    pairs = []
    for file_name in frozen_file_names(directory):
        with open(os.path.join(directory, file_name), "rb") as handle:
            content = handle.read()
        seed_hasher.update(content)
        pairs.append((file_name, hashlib.sha256(content).hexdigest()))
    lines = [f"name=freeze arm={arm_name} file={file_name} sha256={digest}" for file_name, digest in pairs]
    lines.append(f"name=freeze_seed arm={arm_name} sha256_prefix16={seed_hasher.hexdigest()[:16]}")
    return lines, pairs


def manifest_check_lines(dumps_directory: str) -> list[str]:
    lines = []
    for stream_name in STREAM_NAMES:
        stream_directory = os.path.join(dumps_directory, stream_name)
        manifest_path = os.path.join(stream_directory, "manifest.tsv")
        lines.append(f"name=input_manifest stream={stream_name} sha256={sha256_of_file(manifest_path)}")
        with open(manifest_path, encoding="utf-8") as handle:
            for row in handle:
                key, _, value = row.rstrip("\n").partition("\t")
                if not key.startswith("sha256:"):
                    continue
                file_name = key[len("sha256:"):]
                if file_name == "impl-verdicts.tsv":
                    lines.append(f"name=manifest_check stream={stream_name} file={file_name} ok=not_checked "
                                 f"reason=prereg_12.6_writer_of_the_executor_does_not_open_it")
                    continue
                actual = sha256_of_file(os.path.join(stream_directory, file_name))
                lines.append(f"name=manifest_check stream={stream_name} file={file_name} ok={str(actual == value).lower()} "
                             f"manifest={value} actual={actual}")
    return lines


def crc_anchor_lines() -> list[str]:
    """B8–B10（登记第七节）：两条路子（执行器的 crc32c_castagnoli、google_crc32c 直接调）各算一遍。"""
    cases = (("B8", b"123456789", 0xE3069283), ("B9", bytes(4096), 0x98F94189), ("B10", bytes(512), 0x30FCEDC0))
    lines = []
    for anchor, data, expected in cases:
        executor_value = crc32c_castagnoli.crc32c(data)
        library_value = google_crc32c.value(data)
        lines.append(f"name=crc_anchor id={anchor} holds={str(executor_value == library_value == expected).lower()} "
                     f"executor=0x{executor_value:08X} library=0x{library_value:08X} expected=0x{expected:08X}")
    return lines


def parsed_fields(line: str) -> dict[str, str]:
    return dict(token.partition("=")[::2] for token in line.split(" ") if "=" in token)


def executor_state_lines(product_lines: list[str]) -> list[dict[str, str]]:
    return [parsed_fields(line) for line in product_lines if line.startswith("name=state ")]


def batch_count_lines(dumps_directory: str, product_lines: list[str]) -> list[str]:
    """B4-487：每一趟判出的态行数等于那份 Z 导出 states.tsv 的按批行数（不等 → V4）；V11：digest=mismatch 的态数。"""
    states = executor_state_lines(product_lines)
    verdict_states = {(fields["stream"], fields["batch"]): fields["states"] for fields in map(parsed_fields, product_lines)
                      if fields.get("name") == "verdict" and "stream" in fields}
    lines = []
    for stream_name in STREAM_NAMES:
        dump = load_stream_dump(os.path.join(dumps_directory, stream_name))
        for batch_number in JUDGED_BATCHES:
            in_dump = sum(state.batch_number == batch_number for state in dump.states)
            judged = [fields for fields in states if fields["stream"] == stream_name and fields["batch"] == str(batch_number)]
            distinct = len({fields["state"] for fields in judged})
            lines.append(f"name=batch_count id=B4-487 stream={stream_name} batch={batch_number} "
                         f"holds={str(in_dump == len(judged) == distinct == int(verdict_states.get((stream_name, str(batch_number)), -1))).lower()} "
                         f"states_tsv={in_dump} state_lines={len(judged)} distinct_states={distinct} "
                         f"verdict_line_states={verdict_states.get((stream_name, str(batch_number)), 'absent')} "
                         f"digest_mismatch={sum(fields.get('digest') != 'match' for fields in judged)} "
                         f"error_lines={sum(fields.get('strict') == 'error' for fields in judged)}")
    return lines


def pointer_first_slot(pointer: bytes) -> int:
    start = LOCATION_ENTRY_START_IN_A_POINTER + SLOT_NUMBER_START_IN_A_LOCATION_ENTRY
    return int.from_bytes(pointer[start:start + SLOT_NUMBER_BYTES], "little")


def record_fields(record) -> dict[str, int]:
    return {"named_item_count": record.unsigned("named_item_count"), "transaction_number": record.unsigned("transaction_number"),
            "commit_marker": record.unsigned("commit_marker"), "ordinal_within_publish": record.unsigned("ordinal_within_publish"),
            "last_of_publish": int(record.is_last_of_publish),
            "tree_table_slot": pointer_first_slot(record.field_bytes("new_root_tree_table_pointer")),
            "central_mapping_slot": pointer_first_slot(record.field_bytes("new_root_central_mapping_root_pointer")),
            "central_mapping_pointer_all_zero": int(not any(record.field_bytes("new_root_central_mapping_root_pointer"))),
            "tree_identifier_watermark": record.unsigned("new_root_tree_identifier_watermark"),
            "rollback_floor": record.unsigned("new_root_rollback_floor"),
            "unmount_marker": record.unsigned("new_root_unmount_marker")}


def back_chain_over(previous_raw: bytes, header_bytes: int) -> int:
    header = previous_raw[:header_bytes]
    zeroed = header[:HEADER_CHECKSUM_FIELD_START] + bytes(HEADER_CHECKSUM_FIELD_END - HEADER_CHECKSUM_FIELD_START) \
        + header[HEADER_CHECKSUM_FIELD_END:]
    return google_crc32c.value(zeroed)


def full_state_anchor_lines(context, dump) -> list[str]:
    """B11–B13：Z1 流一 F 态（层 0 序号 T − 1，落地向量全 2）上，v2-487 的环扫描读到的记录。"""
    all_persisted = "2" * len(context.prepared_writes)
    sampled = [state.state_number for state in dump.states if state.landing_vector == all_persisted]
    if not sampled:
        return [f"name=format_anchor_assertion id={anchor} stream={FIRST_STREAM} holds=not_run detail=no_sampled_all_persisted_state"
                for anchor in ("B11", "B12", "B13")]
    image = build_crash_image(dump, context.prepared_writes, all_persisted)
    recovery = recover(image, dump.device_identities, dump.sector_bytes, UnitChecksumCache(), spec_gaps.GapsHitInOneState())
    by_device = recovery.ring_scan.valid_records_by_device
    ring = {device: [(record.ring_slot_index * RECORD_BYTES, record.jsn, record.unsigned("checkpoint_txg")) for record in records]
            for device, records in sorted(by_device.items())}
    b11 = len(ring) == 2 and all(records == EXPECTED_RING_B11 for records in ring.values())
    lines = [f"name=format_anchor_assertion id=B11 stream={FIRST_STREAM} holds={str(b11).lower()} sampled_states={sampled} "
             f"ring={str(ring).replace(' ', '')}"]
    differing_b12 = []
    observed_b12 = {}
    for device, records in sorted(by_device.items()):
        for record in records:
            counter = record.jsn[1]
            fields = record_fields(record)
            observed_b12[(device, counter)] = fields
            expected = EXPECTED_RECORD_THREE_FIELDS_B12 if counter == 3 else EXPECTED_RECORDS_ONE_AND_TWO_FIELDS_B12
            differing_b12 += [f"d{device}c{counter}:{key}" for key in expected if fields[key] != expected[key]]
    b12 = b11 and not differing_b12
    lines.append(f"name=format_anchor_assertion id=B12 stream={FIRST_STREAM} holds={str(b12).lower()} "
                 f"differing={','.join(differing_b12) or '-'} observed={str(observed_b12).replace(' ', '')}")
    chain_rows = []
    differing_b13 = []
    for device, records in sorted(by_device.items()):
        ordered = sorted(records, key=lambda record: record.jsn[1])
        for position, record in enumerate(ordered):
            stored = record.unsigned("back_chain")
            if position == 0:
                expected_487 = expected_311 = 0
            else:
                expected_487 = back_chain_over(ordered[position - 1].raw, HEADER_BYTES_FROM_D23_ITEM_4)
                expected_311 = back_chain_over(ordered[position - 1].raw, HEADER_BYTES_OF_THE_OLD_FORMAT)
            if stored != expected_487:
                differing_b13.append(f"d{device}c{record.jsn[1]}")
            chain_rows.append(f"d{device}c{record.jsn[1]}:stored=0x{stored:08X}:over487=0x{expected_487:08X}:"
                              f"over311=0x{expected_311:08X}:311_differs={str(position == 0 or stored != expected_311).lower()}")
    b13 = b11 and not differing_b13
    lines.append(f"name=format_anchor_assertion id=B13 stream={FIRST_STREAM} holds={str(b13).lower()} "
                 f"differing={','.join(differing_b13) or '-'} chain={','.join(chain_rows)}")
    return lines


def record_three_write_ordinals_by_device(context) -> dict[int, list[int]]:
    """写流里把 jsn (1, 3) 那条记录写进环内偏移 8192 的那几次写（D23 已定项 18：计数器 3 落在环内槽 2），逐盘。"""
    ring_start = context.base_pool.unsigned("journal_ring_start_slot") * 16384
    slot_start = ring_start + 2 * RECORD_BYTES
    found: dict[int, list[int]] = {}
    for prepared in context.prepared_writes:
        if not (prepared.byte_offset <= slot_start and prepared.byte_offset + prepared.byte_count >= slot_start + RECORD_BYTES):
            continue
        piece = prepared.new_content[slot_start - prepared.byte_offset:slot_start - prepared.byte_offset + RECORD_BYTES]
        jsn = (int.from_bytes(piece[JSN_INSTANCE_SPAN[0]:JSN_INSTANCE_SPAN[1]], "little"),
               int.from_bytes(piece[JSN_COUNTER_SPAN[0]:JSN_COUNTER_SPAN[1]], "little"))
        if jsn == (1, 3):
            found.setdefault(prepared.device_identity, []).append(prepared.write_ordinal)
    return found


def replay_class_anchor_line(context, dump, product_lines: list[str]) -> str:
    """B14：Z1 流一批 1、批 2 里，(1, 3) 那条记录两份都持久、txg 3 的根槽写没持久（落地不是 2）、暖机两次的根槽写都持久的每一态：
    落点 (1, 3)、经重放（applied ≥ 1、最后施加的计数器 3）。个数为 0 照 Q5-e 记。"""
    versions = [(version.projection.instance, version.projection.checkpoint_txg) for version in context.version_table.versions]
    if versions != [(0, 0), (1, 1), (1, 2), (1, 3)]:
        return f"name=format_anchor_assertion id=B14 stream={FIRST_STREAM} holds=not_run detail=version_table_is_{versions}".replace(" ", "")
    ordinal_by_stream_index = {prepared.stream_index: prepared.write_ordinal for prepared in context.prepared_writes}
    root_ordinal = {index: ordinal_by_stream_index[context.version_table.versions[index].root_write_stream_index] for index in (1, 2, 3)}
    record_ordinals = record_three_write_ordinals_by_device(context)
    landing_by_state = {str(state.state_number): state.landing_vector for state in dump.states}
    in_class = []
    for fields in executor_state_lines(product_lines):
        if fields["stream"] != FIRST_STREAM or int(fields["batch"]) not in JUDGED_BATCHES:
            continue
        landing = landing_by_state[fields["state"]]
        record_persisted_on_both = len(record_ordinals) == 2 and all(
            any(landing[ordinal] == "2" for ordinal in ordinals) for ordinals in record_ordinals.values())
        if record_persisted_on_both and landing[root_ordinal[3]] != "2" and landing[root_ordinal[1]] == "2" and landing[root_ordinal[2]] == "2":
            in_class.append((fields, landing[root_ordinal[3]]))
    failing = [fields["state"] for fields, _ in in_class
               if not (fields["landed"].startswith("(1,3,") and int(fields["applied"]) >= 1 and fields["last_counter"] == "3")]
    chosen_roots = collections.Counter(fields["chosen_root"] for fields, _ in in_class)
    return (f"name=format_anchor_assertion id=B14 stream={FIRST_STREAM} holds={str(not failing).lower()} states={len(in_class)} "
            f"failing_states={','.join(failing) or '-'} root_write_txg3_landing_0={sum(landing == '0' for _, landing in in_class)} "
            f"root_write_txg3_landing_1={sum(landing == '1' for _, landing in in_class)} "
            f"record_three_write_ordinals={str(record_ordinals).replace(' ', '')} root_write_ordinals={str(root_ordinal).replace(' ', '')} "
            f"chosen_roots={str(dict(chosen_roots)).replace(' ', '')}")


def b16_line(product_lines: list[str]) -> str:
    """B16：Z1 流一 F 态 v2-487 的 W、T 字段与第七节 B1 写的逐个相同——executor_main 打的 B1 那一行就是这一比（同一组期望值）。"""
    source = [line for line in product_lines if line.startswith(f"name=witness_anchor_assertion id=B1 stream={FIRST_STREAM} ")]
    if len(source) != 1:
        return f"name=format_anchor_assertion id=B16 stream={FIRST_STREAM} holds=not_run detail=b1_lines_{len(source)}"
    fields = parsed_fields(source[0])
    return (f"name=format_anchor_assertion id=B16 stream={FIRST_STREAM} holds={fields['holds']} differing={fields['differing']} "
            f"source=witness_anchor_assertion_id_B1_of_this_product")


def unit_test_line(arm_name: str) -> str:
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    completed = subprocess.run([sys.executable, "-m", "unittest", "test_executor_on_synthetic_pool"], cwd=ARM_DIRECTORY_BY_NAME[arm_name],
                               env=environment, capture_output=True, text=True, check=False)
    ran = [line for line in completed.stderr.splitlines() if line.startswith("Ran ")]
    last = completed.stderr.strip().splitlines()[-1] if completed.stderr.strip() else "no_output"
    return (f"name=unit_tests arm={arm_name} exit={completed.returncode} ran={ran[-1].split()[1] if ran else 'absent'} "
            f"result={last.replace(' ', '_')}")


def untouched_and_patch_lines() -> list[str]:
    lines = []
    for arm_name, directory in ARM_DIRECTORY_BY_NAME.items():
        origin = ORIGIN_DIRECTORY_BY_ARM[arm_name]
        for file_name in frozen_file_names(directory):
            if file_name in FILES_THE_FORMAT_PATCH_MAY_CHANGE:
                continue
            origin_path = os.path.join(origin, file_name)
            identical = os.path.isfile(origin_path) and sha256_of_file(origin_path) == sha256_of_file(os.path.join(directory, file_name))
            lines.append(f"name=untouched arm={arm_name} file={file_name} identical_to_origin={str(identical).lower()}")
    common = sorted(set(frozen_file_names(ARM_DIRECTORY_BY_NAME["v2_487"])) & set(frozen_file_names(ARM_DIRECTORY_BY_NAME["v1_487"]))
                    & set(FILES_THE_FORMAT_PATCH_MAY_CHANGE))
    for file_name in common:
        texts = {}
        for arm_name in ARM_DIRECTORY_BY_NAME:
            with open(os.path.join(ORIGIN_DIRECTORY_BY_ARM[arm_name], file_name), encoding="utf-8") as handle:
                before = handle.read().splitlines()
            with open(os.path.join(ARM_DIRECTORY_BY_NAME[arm_name], file_name), encoding="utf-8") as handle:
                after = handle.read().splitlines()
            texts[arm_name] = (before, after)
        if texts["v2_487"][0] == texts["v1_487"][0]:
            method, same = "origins_identical_compare_patched_files", texts["v2_487"][1] == texts["v1_487"][1]
        else:
            def changed_lines(pair):
                return collections.Counter(line for line in difflib.unified_diff(pair[0], pair[1], n=0, lineterm="")
                                           if line[:1] in "+-" and not line.startswith(("+++", "---")))
            method, same = "origins_differ_compare_added_and_removed_line_multisets", changed_lines(texts["v2_487"]) == changed_lines(texts["v1_487"])
        lines.append(f"name=format_patch_same file={file_name} method={method} same={str(same).lower()}")
    return lines


def executor_freeze_cross_check_line(product_lines: list[str], wrapper_pairs: list[tuple[str, str]], wrapper_seed_line: str) -> str:
    """executor_main.freeze_lines 打的不带臂名的冻结行就是 v2-487 的；与这里按臂名另算的逐行比（两份代码同一取法）。"""
    executor_pairs = [(fields["file"], fields["sha256"]) for fields in map(parsed_fields, product_lines)
                      if fields.get("name") == "freeze" and "arm" not in fields]
    executor_seed = [fields["sha256_prefix16"] for fields in map(parsed_fields, product_lines)
                     if fields.get("name") == "freeze_seed" and "arm" not in fields]
    wrapper_seed = parsed_fields(wrapper_seed_line)["sha256_prefix16"]
    return (f"name=freeze_cross_check arm=v2_487 executor_freeze_lines={len(executor_pairs)} wrapper_freeze_lines={len(wrapper_pairs)} "
            f"files_equal={str(executor_pairs == wrapper_pairs).lower()} seed_equal={str(executor_seed == [wrapper_seed]).lower()}")


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑第一段乙：Z1 上的格式锚点与两条新臂的冻结，追加进 executor_main 的产物")
    parser.add_argument("--dumps", required=True, help="Z1 导出目录（scripts/experiment-apparatus/target/e170-r2-today-dumps/e170-dump）")
    parser.add_argument("--product", required=True, help="executor_main.py 刚写完的产物；只追加")
    arguments = parser.parse_args(argument_list)
    with open(arguments.product, encoding="utf-8") as handle:
        product_lines = handle.read().splitlines()
    if not product_lines or product_lines[-1] != "name=complete":
        print(f"{arguments.product} 末行不是 name=complete，executor_main 没跑完，不追加", file=sys.stderr)
        return 2
    if any(line.startswith("name=segment_one_b") for line in product_lines):
        print(f"{arguments.product} 已经追加过，不再追加", file=sys.stderr)
        return 2
    dump = load_stream_dump(os.path.join(arguments.dumps, FIRST_STREAM))
    context = prepare_stream_context(dump)
    appended = [f"name=segment_one_b_begin dumps={arguments.dumps} appended_after_line={len(product_lines)}"]
    appended += manifest_check_lines(arguments.dumps)
    appended += crc_anchor_lines()
    appended += batch_count_lines(arguments.dumps, product_lines)
    appended += full_state_anchor_lines(context, dump)
    appended.append(replay_class_anchor_line(context, dump, product_lines))
    appended.append(b16_line(product_lines))
    appended += [unit_test_line(arm_name) for arm_name in ARM_DIRECTORY_BY_NAME]
    appended += untouched_and_patch_lines()
    freeze_by_arm = {arm_name: freeze_lines_of_arm(arm_name) for arm_name in ARM_DIRECTORY_BY_NAME}
    for arm_name, (lines, _) in freeze_by_arm.items():
        appended += lines
    v2_lines, v2_pairs = freeze_by_arm["v2_487"]
    appended.append(executor_freeze_cross_check_line(product_lines, v2_pairs, v2_lines[-1]))
    appended.append("name=segment_one_b_complete")
    with open(arguments.product, "a", encoding="utf-8") as output:
        output.write("\n".join(appended) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
