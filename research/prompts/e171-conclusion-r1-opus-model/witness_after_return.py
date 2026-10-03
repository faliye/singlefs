"""e171-conclusion-r1 攻方腿：在 E171 导出的态上数两样（只读导出目录，借冻结 O3 的读写流与建镜像代码，拷在 o3copy/）。
① 执行员式：有一次已返回的调用，它的系统配置轮换写里有没落的，而落了的写里有一条在它返回之后（第 270 行那一判的同形）。
② C577 式：见证值（全部自证过的系统配置槽 journal_tail 的最大值）小于已确认那一版（ack(c)）轮换写带的 tail。
   这是 `a_publish_returns_only_after_the_system_configuration_rotation_is_durable` 钉的性质「发布返回后每个崩溃点见证值 ≥ 末条记录计数器」
   在导出态上的读法（推的对应：轮换写带的 tail 就是那次发布末条记录的计数器，没逐条核）。
用法：python3 witness_after_return.py <o3copy 目录> <标签=流目录>...
"""
import sys

sys.path.insert(0, sys.argv[1])
import on_disk_layout as layout  # noqa: E402
from crash_dump import load_stream_dump  # noqa: E402
from crash_image import build_crash_image, LANDED_NOT_PERSISTED  # noqa: E402
from executor_core import prepare_stream_context  # noqa: E402
from system_configuration import slot_is_self_verified  # noqa: E402
from version_table import allowed_versions  # noqa: E402

TAIL = 469


def tail_of(raw: bytes) -> int:
    return int.from_bytes(raw[TAIL:TAIL + 8], "little")


for item in sys.argv[2:]:
    label, directory = item.split("=", 1)
    dump = load_stream_dump(directory)
    context = prepare_stream_context(dump)
    table = context.version_table
    spacing = context.base_pool.unsigned("fixed_structure_slot_spacing")
    syscfg = {p.write_ordinal for p in context.prepared_writes
              if p.byte_offset in (0, spacing) and p.byte_count == layout.SYSTEM_CONFIGURATION_SLOT_BYTES}
    rotation_by_version = {}
    for p in context.prepared_writes:
        owner = table.owner_version_by_write_ordinal[p.write_ordinal]
        if p.write_ordinal in syscfg and owner is not None:
            rotation_by_version.setdefault(owner, []).append(p)
    tail_by_version = {v: max(tail_of(p.new_content) for p in ws) for v, ws in rotation_by_version.items()}
    counts = {"states": 0, "executor_style": 0, "witness_below_ack_rotation": 0}
    examples = []
    for state in dump.states:
        if state.batch_number not in (1, 2):
            continue
        counts["states"] += 1
        landing = state.landing_vector
        landed_indices = [table.write_stream_indices[o] for o, c in enumerate(landing) if c != LANDED_NOT_PERSISTED]
        largest = max(landed_indices, default=-1)
        executor_style = False
        for version in table.versions[1:]:
            if version.return_point is None or version.return_point > largest:
                continue
            writes = rotation_by_version.get(version.version_index, [])
            if any(landing[p.write_ordinal] == LANDED_NOT_PERSISTED for p in writes):
                executor_style = True
        if executor_style:
            counts["executor_style"] += 1
        image = build_crash_image(dump, context.prepared_writes, landing)
        witness = 0
        for device in dump.device_identities:
            for offset in (0, spacing):
                raw = image.read_bytes(device, offset, layout.SYSTEM_CONFIGURATION_SLOT_BYTES)
                if raw is not None and slot_is_self_verified(raw):
                    witness = max(witness, tail_of(raw))
        ack = allowed_versions(table, landing).acknowledged_version_index
        needed = max((tail_by_version[v] for v in tail_by_version if v <= ack), default=0)
        if witness < needed:
            counts["witness_below_ack_rotation"] += 1
            if len(examples) < 3:
                examples.append(f"state={state.state_number} batch={state.batch_number} seg={state.segment_number} witness={witness} needed={needed} ack={ack}")
    print(f"name=witness_after_return label={label} " + " ".join(f"{k}={v}" for k, v in counts.items())
          + f" versions_with_rotation={len(tail_by_version)} examples={';'.join(examples) or '-'}")
