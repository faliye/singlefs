"""e171-conclusion-r2 攻方腿：在导出态上算一条单镜像谓词「无故障镜像上，D23（journal 的角色与格式） 已定项 14 可写挂载那一条
『更新状态读不出』判据为假」（这一轮提的，零轮），并与 r1 攻方那一条「见证值 < 已确认那一版轮换写带的 tail」并排数。
谓词 W（照 D23 已定项 14 第 392 行那一段的取法写，读盘、择根、扫环、重放都借冻结 O3 的代码，拷在 o3copy/）：
  c_见证 = 每块盘两槽里自证过的系统配置槽 journal_tail 的最大值；0 ⇒ 假；
  所选那一版（O3 重放之后落到的那一版）那次发布带末条标志的记录读得出 ⇒ c_见证 > 它的计数器即为真；
  读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) 大于所选那一版即为真；两条都读不出按真。
  W 为真的态，正确实现上可写挂载会在取号之前拒（重读一次仍为真时），盘上没有任何读故障。
用法：python3 witness_consistency.py <o3copy> <标签=流目录>...      只数批 1、批 2 的态
      设环境变量 WITNESS_PER_STATE=<文件> 时另把每一态「标签 \t 态号 \t W」追加进那个文件（给 join_witness_and_mount.py）。
"""
import os
import sys

sys.path.insert(0, sys.argv[1])
import on_disk_layout as layout  # noqa: E402
import spec_gaps  # noqa: E402
from crash_dump import load_stream_dump  # noqa: E402
from crash_image import build_crash_image  # noqa: E402
from executor_core import prepare_stream_context, recover  # noqa: E402
from replay_prefix import UnitChecksumCache  # noqa: E402
from system_configuration import slot_is_self_verified  # noqa: E402

TAIL = 469


def tail_of(raw: bytes) -> int:
    return int.from_bytes(raw[TAIL:TAIL + 8], "little")


for item in sys.argv[2:]:
    label, directory = item.split("=", 1)
    dump = load_stream_dump(directory)
    context = prepare_stream_context(dump)
    spacing = context.base_pool.unsigned("fixed_structure_slot_spacing")
    cache = UnitChecksumCache()
    counts = {"states": 0, "w_true": 0, "no_landing": 0}
    examples = []
    for state in dump.states:
        if state.batch_number not in (1, 2):
            continue
        counts["states"] += 1
        image = build_crash_image(dump, context.prepared_writes, state.landing_vector)
        witness = 0
        for device in dump.device_identities:
            for offset in (0, spacing):
                raw = image.read_bytes(device, offset, layout.SYSTEM_CONFIGURATION_SLOT_BYTES)
                if raw is not None and slot_is_self_verified(raw):
                    witness = max(witness, tail_of(raw))
        recovery = recover(image, dump.device_identities, dump.sector_bytes, cache, spec_gaps.GapsHitInOneState())
        if recovery.landed is None or recovery.ring_scan is None:
            counts["no_landing"] += 1
            continue
        selected = (recovery.landed.instance, recovery.landed.checkpoint_txg)
        records = recovery.ring_scan.records_by_counter
        if witness == 0:
            w = False
        else:
            last = [r for r in records.values() if (r.jsn[0], r.unsigned("checkpoint_txg")) == selected and r.is_last_of_publish]
            if last:
                w = witness > last[0].jsn[1]
            elif witness in records:
                r = records[witness]
                w = (r.jsn[0], r.unsigned("checkpoint_txg")) > selected
            else:
                w = True
        if os.environ.get("WITNESS_PER_STATE"):
            with open(os.environ["WITNESS_PER_STATE"], "a") as per_state:
                per_state.write(f"{label}\t{state.state_number}\t{w}\n")
        if w:
            counts["w_true"] += 1
            if len(examples) < 12:
                examples.append(f"{state.state_number}")
    print(f"name=witness_consistency label={label} " + " ".join(f"{k}={v}" for k, v in counts.items())
          + f" first_states={','.join(examples) or '-'}")
