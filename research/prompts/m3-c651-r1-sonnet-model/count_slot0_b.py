"""只读：E170 两条流的取样态里，系统配置槽 0 与 4096 处槽 1 每盘自证与否的分布（正推腿草稿）。"""
import os, sys
sys.path.insert(0, os.getcwd())
import spec_gaps
from crash_dump import load_stream_dump
from crash_image import build_crash_image
from executor_core import prepare_stream_context
import system_configuration as sc

root = "/home/fy5090/code/singlefs/research/target/e170-dumps"
for stream in ("new_pool_file_creation", "fixed_script_through_e_then_unmount"):
    dump = load_stream_dump(os.path.join(root, stream))
    ctx = prepare_stream_context(dump)
    tally = {}
    for state in dump.states:
        image = build_crash_image(dump, ctx.prepared_writes, state.landing_vector)
        gaps = spec_gaps.GapsHitInOneState()
        zero = [(lambda r: r is not None and sc.slot_is_self_verified(r, gaps))(image.read_bytes(d, 0, sc.SLOT_BYTES)) for d in dump.device_identities]
        one = [(lambda r: r is not None and sc.slot_is_self_verified(r, gaps))(image.read_bytes(d, 4096, sc.SLOT_BYTES)) for d in dump.device_identities]
        key = (state.batch_number, sum(zero), sum(one))
        tally[key] = tally.get(key, 0) + 1
    print(stream, "devices", dump.device_identities, "states", len(dump.states))
    for k in sorted(tally):
        print("  batch=%s 槽0自证过的盘数=%s/2 槽1(4096)自证过的盘数=%s/2 态数=%s" % (k[0], k[1], k[2], tally[k]))
