import sys, re, collections, glob
c = collections.Counter(); ranges = []
for i in range(64):
    for line in open(f"{sys.argv[1]}-{i}.txt", encoding="utf-8"):
        m = re.search(r"ordinals=(\d+)\.\.(\d+)", line)
        if m: ranges.append((int(m.group(1)), int(m.group(2))))
        m = re.match(r"stream=\S+ count=(\d+) (.*)$", line)
        if m: c[m.group(2)] += int(m.group(1))
ranges.sort()
contiguous = all(ranges[k][1] == ranges[k+1][0] for k in range(len(ranges)-1))
print(f"shares={len(ranges)} first={ranges[0]} last={ranges[-1]} contiguous={contiguous} states_counted={sum(c.values())}")
for line, n in sorted(c.items(), key=lambda kv: -kv[1]):
    print(f"count={n} {line}")
