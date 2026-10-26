import sys, collections
# 用法：closure_stats.py <closure 输出> ；打出 fn 个数、fn 行数、核内（crates/singlefs-replay-prefix）fn 个数与行数
path = sys.argv[1]
fns = lines = kfns = klines = 0
files = collections.Counter()
for row in open(path, encoding="utf-8"):
    parts = row.rstrip("\n").split("\t")
    if len(parts) < 3 or ":" not in parts[2] or parts[0] != "fn":
        continue
    file, span = parts[2].rsplit(":", 1)
    a, b = span.split("-")
    n = int(b) - int(a) + 1
    fns += 1; lines += n
    files[file] += 1
    if file.startswith("crates/singlefs-replay-prefix/"):
        kfns += 1; klines += n
print(f"fn={fns} fn_lines={lines} kernel_fn={kfns} kernel_fn_lines={klines} kernel_share_of_lines={100.0*klines/lines:.2f}%")
