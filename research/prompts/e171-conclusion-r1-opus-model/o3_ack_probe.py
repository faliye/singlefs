"""e171-conclusion-r1 攻方腿 G3：造一个只让 O3 红的坏实现（写流层面模拟，推的等价于三处代码变异），喂冻结的 O3。
坏实现「暖机的第二次空发布不等持久就返回」：拿掉暖机第二轮 journal 记录与根槽之间那道屏障、根槽写不带 FUA、轮换之后那道屏障也拿掉。
写的内容与基线逐字节相同，只改屏障与写种类；历史截到下一次调用（新池新建文件的 publish）的第一组单元写（两块盘各一次）为止。
崩溃状态：前面各段与基线相同（不重判），合并出来的末段 7 次写的整写子集全枚举，可撕裂的写（torn.tsv 有字节的）另加撕裂一态：
2^5 × 3^2 = 288 态，一个不落。
输出：编辑过的导出目录（中性格式，给 Rust 探针读）与每态 O3 的严格 / 暂代判定、ack、着陆版本、盘上已持久的最新根。
用法：python3 o3_ack_probe.py <o3copy> <基线新池新建文件流目录> <输出目录>
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

base_dir, out_dir = sys.argv[2], sys.argv[3]
dump = load_stream_dump(base_dir)
DROP = {15, 16, 20, 21}  # 暖机第二轮：记录与根槽之间的两道屏障、轮换之后的两道屏障（流序下标，基线 ops.tsv）
TO_PLAIN = {17}  # 暖机第二轮的根槽 FUA 写 → 普通写
LAST_KEPT = 23  # 截到 publish 的第一组单元写（流序 22、23）
assert dump.operations[17].kind == OPERATION_KIND_WRITE_FORCED_UNIT_ACCESS
assert all(dump.operations[i].kind == OPERATION_KIND_BARRIER for i in DROP)
new_ops, new_index_of = [], {}
for op in dump.operations[:LAST_KEPT + 1]:
    if op.stream_index in DROP:
        continue
    kind = OPERATION_KIND_WRITE if op.stream_index in TO_PLAIN else op.kind
    new_index_of[op.stream_index] = len(new_ops)
    new_ops.append(RecordedOperation(len(new_ops), op.device_identity, kind, op.byte_offset, op.byte_count, op.payload))
calls = [DriverCall(0, "acquire", 0, new_index_of[4] if 4 in new_index_of else 4, True),
         DriverCall(1, "warm_up", new_index_of[4], new_index_of[22], True),
         DriverCall(2, "publish", new_index_of[22], len(new_ops) + 1, False)]  # 截断：publish 在这段历史里没返回
write_indices = [op.stream_index for op in new_ops if op.kind != OPERATION_KIND_BARRIER]
write_count = len(write_indices)
torn = {o: b for o, b in dump.torn_payload_by_write_ordinal.items() if o < write_count}
last_barrier = max(op.stream_index for op in new_ops if op.kind == OPERATION_KIND_BARRIER)
tail = [o for o, i in enumerate(write_indices) if i > last_barrier]
choices = [("0", "2", "1") if o in torn else ("0", "2") for o in tail]
edited = StreamDump("new_pool_file_creation", dump.manifest, dump.device_identities, dump.sector_bytes, dump.base_sectors,
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
lines, counts = [], {"states": 0, "o3_strict_not_in": 0, "o3_provisional_not_in": 0, "spec_gap": 0}
for state in states:
    j = judge_state(context, state, cache)
    persisted_roots = [root_writes[write_indices[o]].projection for o, c in enumerate(state.landing_vector)
                       if c == "2" and write_indices[o] in root_writes]
    newest = max(((p.checkpoint_txg, p.instance) for p in persisted_roots), default=(0, 0))
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
    f.write(f"stream\tnew_pool_file_creation_edited_warm_up_returns_before_durable\ndevices\t0,1\n"
            f"device_bytes\t{dump.manifest['device_bytes']}\nsector_bytes\t512\nops\t{len(new_ops)}\nwrites\t{write_count}\n"
            f"calls\t3\nsampled_states\t{len(states)}\n")
with open(os.path.join(out_dir, "o3-verdicts.tsv"), "w") as f:
    f.write("\n".join(lines) + "\n")
print(f"name=o3_ack_probe tail_writes={len(tail)} torn_in_tail={sum(o in torn for o in tail)} "
      + " ".join(f"{k}={v}" for k, v in counts.items()) + f" anomalies={len(context.version_table.anomalies)}")
