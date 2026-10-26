import sys, re, collections
c = collections.Counter()
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8"):
        if "|" not in line: continue
        cell, image = line.split()[0], line.split()[1]
        cell = re.sub(r"-N\d+$", "", cell)
        verd = line.split("|")[1].strip()
        rec = re.search(r"rec=(\S+)", line).group(1)
        c[(cell, image, rec, verd)] += 1
for (cell, image, rec, verd), n in sorted(c.items()):
    print(f"{cell:<50} {image:<36} rec={rec} n={n:<3} | {verd}")
