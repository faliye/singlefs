"""e171-conclusion-r2 攻方腿：两种「已返回的调用丢了」写流（e171-conclusion-r1 攻方 `o3_ack_probe.py` 改参数，其余逐行照抄），喂冻结的 O3，
并给 Rust 判官探针（opus_r2_judge_probe.rs）写中性格式导出与 judge-extra.tsv（态号 \t 盘上已持久的最新根 实例,txg）。
形 warmup（r1 那一形）：流一，暖机第二次空发布的记录与根槽之间、轮换之后的屏障拿掉，根槽 FUA 改普通写；截到 publish 的第一组单元写。
形 content（这一轮加的）：流二，第一次带文件的发布（新池新建文件，调用 2）的记录与根槽之间、轮换之后的屏障拿掉，根槽 FUA 改普通写；
  截到下一次调用（覆盖写，调用 3）的第一组单元写。写的字节与基线逐字节相同，只改屏障与写种类。
崩溃状态：末段合并出的写的整写子集全枚举，可撕裂的写另加撕裂态，一个不落；前面各段与基线相同，不重判。
用法：python3 o3_ack_shapes.py <o3copy> <warmup|content> <基线流目录> <输出目录>
"""
import hashlib
import itertools
import os
import sys

sys.path.insert(0, sys.argv[1])
from crash_dump import (OPERATION_KIND_BARRIER, OPERATION_KIND_WRITE, OPERATION_KIND_WRITE_FORCED_UNIT_ACCESS,  # noqa: E402
                        DriverCall, RecordedOperation, SampledState, StreamDump, load_stream_dump)
from crash_image import build_crash_image  # noqa: E402
from executor_core import judge_state, prepare_stream_context  # noqa: E402
from replay_prefix import UnitChecksumCache  # noqa: E402

shape, base_dir, out_dir = sys.argv[2], sys.argv[3], sys.argv[4]
dump = load_stream_dump(base_dir)
if shape == "warmup":
    DROP, TO_PLAIN, LAST_KEPT = {15, 16, 20, 21}, {17}, 23
    CALLS = [("acquire", 0, 4, True), ("warm_up", 4, 22, True), ("publish", 22, None, False)]
elif shape == "content":
    DROP, TO_PLAIN, LAST_KEPT = {50, 51, 55, 56}, {52}, 58
    CALLS = [("acquire", 0, 4, True), ("warm_up", 4, 22, True), ("publish", 22, 57, True), ("publish", 57, None, False)]
else:
    raise SystemExit(f"形 {shape}")
assert all(dump.operations[i].kind == OPERATION_KIND_WRITE_FORCED_UNIT_ACCESS for i in TO_PLAIN)
assert all(dump.operations[i].kind == OPERATION_KIND_BARRIER for i in DROP)
new_ops, new_index_of = [], {}
for op in dump.operations[:LAST_KEPT + 1]:
    if op.stream_index in DROP:
        continue
    kind = OPERATION_KIND_WRITE if op.stream_index in TO_PLAIN else op.kind
    new_index_of[op.stream_index] = len(new_ops)
    new_ops.append(RecordedOperation(len(new_ops), op.device_identity, kind, op.byte_offset, op.byte_count, op.payload))
calls = [DriverCall(n, label, new_index_of[first], new_index_of[ret] if ret is not None else len(new_ops) + 1, ok)
         for n, (label, first, ret, ok) in enumerate(CALLS)]
write_indices = [op.stream_index for op in new_ops if op.kind != OPERATION_KIND_BARRIER]
write_count = len(write_indices)
torn = {o: b for o, b in dump.torn_payload_by_write_ordinal.items() if o < write_count}
last_barrier = max(op.stream_index for op in new_ops if op.kind == OPERATION_KIND_BARRIER)
tail = [o for o, i in enumerate(write_indices) if i > last_barrier]
choices = [("0", "2", "1") if o in torn else ("0", "2") for o in tail]
edited = StreamDump(dump.stream_name, dump.manifest, dump.device_identities, dump.sector_bytes, dump.base_sectors,
                    new_ops, write_indices, calls, torn, [], [], [])
context = prepare_stream_context(edited)
states = []
for number, combo in enumerate(itertools.product(*choices)):
    vector = ["2"] * write_count
    for o, c in zip(tail, combo):
        vector[o] = c
    landing = "".join(vector)
    digest = build_crash_image(edited, context.prepared_writes, landing).digest()
    states.append(SampledState(number, number, 1, 1, landing, digest))
edited.states = states
context = prepare_stream_context(edited)
root_writes = {v.root_write_stream_index: v for v in context.version_table.versions[1:]}
cache = UnitChecksumCache()
os.makedirs(out_dir, exist_ok=False)
extra, lines, counts = [], [], {"states": 0, "o3_strict_not_in": 0, "o3_provisional_not_in": 0, "spec_gap": 0}
for state in states:
    j = judge_state(context, state, cache)
    persisted_roots = [root_writes[write_indices[o]].projection for o, c in enumerate(state.landing_vector)
                       if c == "2" and write_indices[o] in root_writes]
    newest = max(((p.checkpoint_txg, p.instance) for p in persisted_roots), default=(0, 0))
    extra.append(f"{state.state_number}\t{newest[1]},{newest[0]}")
    counts["states"] += 1
    counts["o3_strict_not_in"] += j.strict_verdict == "not_in"
    counts["o3_provisional_not_in"] += j.provisional_verdict == "not_in"
    counts["spec_gap"] += j.strict_verdict == "spec_gap"
    lines.append(f"{state.state_number}\t{state.landing_vector}\tstrict={j.strict_verdict}\tprovisional={j.provisional_verdict}"
                 f"\tack={j.allowed.acknowledged_version_index}\tbegun={j.allowed.begun_version_index}\tlanded={j.landed_text}"
                 f"\tnewest_persisted_root_txg_instance={newest[0]},{newest[1]}\tdigest_ok={j.digest_matches}")
# 写中性格式，给 Rust 探针（opus_e171_crash_then_mount_probe.rs）读
payload = bytearray()
with open(os.path.join(out_dir, "ops.tsv"), "w") as f:
    for op in new_ops:
        if op.payload is None:
            f.write(f"{op.stream_index}\t{op.device_identity}\t{op.kind}\t{op.byte_offset}\t{op.byte_count}\t-\t-\n")
        else:
            f.write(f"{op.stream_index}\t{op.device_identity}\t{op.kind}\t{op.byte_offset}\t{op.byte_count}\t{len(payload)}\t"
                    f"{hashlib.sha256(op.payload).hexdigest()}\n")
            payload += op.payload
open(os.path.join(out_dir, "ops-payload.bin"), "wb").write(payload)
torn_bytes = bytearray()
with open(os.path.join(out_dir, "torn.tsv"), "w") as f:
    for o, b in sorted(torn.items()):
        f.write(f"{o}\t{len(torn_bytes)}\t{len(b)}\t{hashlib.sha256(b).hexdigest()}\n")
        torn_bytes += b
open(os.path.join(out_dir, "torn-payload.bin"), "wb").write(torn_bytes)
with open(os.path.join(base_dir, "base-sectors.bin"), "rb") as src:
    open(os.path.join(out_dir, "base-sectors.bin"), "wb").write(src.read())
with open(os.path.join(out_dir, "calls.tsv"), "w") as f:
    for c in calls:
        f.write(f"{c.call_number}\t{c.label}\t{c.first_operation_index}\t{c.return_point}\t{'ok' if c.returned_ok else 'not_returned'}\n")
with open(os.path.join(out_dir, "states.tsv"), "w") as f:
    for s in states:
        f.write(f"{s.state_number}\t{s.layer_zero_sequence}\t{s.segment_number}\t{s.batch_number}\t{s.landing_vector}\t{s.image_digest}\n")
with open(os.path.join(out_dir, "manifest.tsv"), "w") as f:
    f.write(f"stream\t{shape}_returns_before_durable\ndevices\t0,1\n"
            f"device_bytes\t{dump.manifest['device_bytes']}\nsector_bytes\t512\nops\t{len(new_ops)}\nwrites\t{write_count}\n"
            f"calls\t{len(calls)}\nsampled_states\t{len(states)}\n")
with open(os.path.join(out_dir, "judge-extra.tsv"), "w") as f:
    f.write("\n".join(extra) + "\n")
with open(os.path.join(out_dir, "o3-verdicts.tsv"), "w") as f:
    f.write("\n".join(lines) + "\n")
print(f"name=o3_ack_shape shape={shape} tail_writes={len(tail)} torn_in_tail={sum(o in torn for o in tail)} "
      + " ".join(f"{k}={v}" for k, v in counts.items()) + f" anomalies={len(context.version_table.anomalies)}")
