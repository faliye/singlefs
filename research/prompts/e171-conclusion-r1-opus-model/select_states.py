"""e171-conclusion-r1 攻方腿：从 E171 导出的一条流里挑出要喂探针的态，写一份只换了 states.tsv 的导出目录（其余文件链到原目录，只读）。
挑法（批 1、2 里）：
  differs-from-baseline <基线流目录>：同号态的镜像摘要与基线不同的态（落地向量要与基线同号态相同，不同就报错停）；
  same-as-baseline-numbers <别的挑选目录>：取那份挑选目录 states.tsv 里的态号，在本流里取同号的态；
  rotation-not-durable-after-return：第 270 行那一判（已返回的调用有轮换写没落、而落了的写里有一条在它返回之后，借 O3 的版本表认轮换写）；
  rotation-landed-root-not：第 242 行那一判（同一次发布的系统配置写落了（持久或撕裂）、它后面那条根槽写没持久）。
用法：python3 select_states.py <o3copy> <流目录> <输出目录> <挑法> [参数]
"""
import os
import sys

sys.path.insert(0, sys.argv[1])
import on_disk_layout as layout  # noqa: E402
from crash_dump import load_stream_dump  # noqa: E402
from executor_core import prepare_stream_context  # noqa: E402

stream_dir, out_dir, how = sys.argv[2], sys.argv[3], sys.argv[4]
dump = load_stream_dump(stream_dir)
states = [s for s in dump.states if s.batch_number in (1, 2)]
chosen = []
if how == "differs-from-baseline":
    base = {s.state_number: s for s in load_stream_dump(sys.argv[5]).states}
    for s in states:
        b = base[s.state_number]
        assert b.landing_vector == s.landing_vector, f"态 {s.state_number} 落地向量与基线不同"
        if b.image_digest != s.image_digest:
            chosen.append(s)
elif how == "same-as-baseline-numbers":
    wanted = {int(line.split("\t")[0]) for line in open(os.path.join(sys.argv[5], "states.tsv")) if line[0].isdigit()}
    chosen = [s for s in states if s.state_number in wanted]
else:
    context = prepare_stream_context(dump)
    table = context.version_table
    spacing = context.base_pool.unsigned("fixed_structure_slot_spacing")
    syscfg = [p.write_ordinal for p in context.prepared_writes
              if p.byte_offset in (0, spacing) and p.byte_count == layout.SYSTEM_CONFIGURATION_SLOT_BYTES]
    root_ordinals = [o for o, i in enumerate(table.write_stream_indices)
                     if i in {v.root_write_stream_index for v in table.versions[1:]}]
    if how == "rotation-not-durable-after-return":
        rotation = {}
        for o in syscfg:
            owner = table.owner_version_by_write_ordinal[o]
            if owner is not None:
                rotation.setdefault(owner, []).append(o)
        for s in states:
            landed = [table.write_stream_indices[o] for o, c in enumerate(s.landing_vector) if c != "0"]
            largest = max(landed, default=-1)
            if any(v.return_point is not None and v.return_point <= largest
                   and any(s.landing_vector[o] == "0" for o in rotation.get(v.version_index, []))
                   for v in table.versions[1:]):
                chosen.append(s)
    elif how == "rotation-landed-root-not":
        # 每条根槽写之前、上一条根槽写之后的系统配置写，算这一次发布挪到根之前的轮换
        previous_root = -1
        pairs = []
        for r in root_ordinals:
            pairs.append((r, [o for o in syscfg if previous_root < o < r]))
            previous_root = r
        for s in states:
            if any(s.landing_vector[r] != "2" and any(s.landing_vector[o] != "0" for o in before) for r, before in pairs if before):
                chosen.append(s)
    else:
        raise SystemExit(f"不认得的挑法 {how}")
os.makedirs(out_dir, exist_ok=False)
for name in os.listdir(stream_dir):
    if name != "states.tsv":
        os.symlink(os.path.join(stream_dir, name), os.path.join(out_dir, name))
with open(os.path.join(out_dir, "states.tsv"), "w") as f:
    for s in chosen:
        f.write(f"{s.state_number}\t{s.layer_zero_sequence}\t{s.segment_number}\t{s.batch_number}\t{s.landing_vector}\t{s.image_digest}\n")
print(f"name=select how={how} stream={stream_dir.rstrip('/').split('/')[-3:]} batch12_states={len(states)} chosen={len(chosen)}")
