# 草稿：照 sweep-term.py 的 walk + 读 + mask + detect 逐文件跑，超过 0.5 秒的逐个立即打印；只读
import importlib.util, os, sys, time
root = sys.argv[1]
sys.path.insert(0, os.path.join(root, "research/scripts"))
spec = importlib.util.spec_from_file_location("sweep_term", os.path.join(root, "research/scripts/sweep-term.py"))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
entries = module.load_exempt(root); pairs = module.load_renames(root)
replacements, detect = module.compile_rules(pairs); archived = module.archived_product_file_names(root)
start = time.monotonic(); n = 0; total_read = total_mask = total_detect = 0.0
for relative in module.walk(root, entries):
    n += 1
    a = time.monotonic()
    try:
        text = open(os.path.join(root, relative), encoding="utf-8").read()
    except (UnicodeDecodeError, OSError):
        continue
    b = time.monotonic(); masked_text, masked = module.mask_archived_product_names(text, archived)
    c = time.monotonic(); count = len(detect.findall(masked_text)); d = time.monotonic()
    total_read += b - a; total_mask += c - b; total_detect += d - c
    if d - a > 0.5:
        print(f"SLOW\t{relative}\tchars={len(text)}\tread={b-a:.2f}\tmask={c-b:.2f}\tdetect={d-c:.2f}", flush=True)
    if n % 500 == 0:
        print(f"PROGRESS\t{n} files\t{time.monotonic()-start:.1f}s", flush=True)
print(f"DONE\t{n} files\t{time.monotonic()-start:.1f}s\tread={total_read:.1f}\tmask={total_mask:.1f}\tdetect={total_detect:.1f}", flush=True)
