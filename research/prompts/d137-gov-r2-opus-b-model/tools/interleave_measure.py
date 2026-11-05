import glob, os, collections
root = os.path.expanduser("~/.local/state/singlefs/detached-runs")
paired = collections.defaultdict(list); lower = collections.defaultdict(list); files = 0; shared = 0
for f in sorted(glob.glob(root + "/*/serial-declarations")):
    files += 1
    rows = []
    for line in open(f, errors="replace"):
        if "\t" not in line: continue
        t, b = line.rstrip("\n").split("\t", 1)
        try: rows.append((float(t), b))
        except ValueError: pass
    keys = set()
    for i, (t, b) in enumerate(rows):
        if not b.startswith("此刻串行") or i + 1 >= len(rows): continue
        key = b.split("：", 1)[1][:14]
        keys.add(key)
        nt, nb = rows[i + 1]
        (paired if nb.startswith("此刻并行") else lower)[key].append(nt - t)
    if len(keys) > 1: shared += 1
print(f"name=declaration_files files={files} files_with_more_than_one_reason={shared}")
for key in sorted(set(paired) | set(lower)):
    p, l = paired.get(key, []), lower.get(key, [])
    print(f"name=segments key={key} paired={len(p)} paired_max={max(p) if p else 0:.0f} interrupted={len(l)} interrupted_gap_max={max(l) if l else 0:.0f}")
