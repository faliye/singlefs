import sys, re, pathlib
root = pathlib.Path("/home/fy5090/code/singlefs")
specs = [l.rstrip("\n") for l in open(sys.argv[1], encoding="utf-8") if l.strip() and not l.startswith("#")]
for spec in specs:
    label, path, pattern = spec.split("|", 2)
    lines = (root / path).read_text(encoding="utf-8").split("\n")
    norm = lambda t: re.sub(r"\s+", " ", t.replace("**", "").replace("`", "")).strip()
    hits = [i + 1 for i, l in enumerate(lines) if norm(pattern) in norm(l)]
    print(f"{label}\t{len(hits)}\t{hits[:6]}\t{path}\t{pattern}")
