"""Count how many times each mutation row's original text occurs in its file (frozen copy, read-only).
Usage: python3 -I anchor_counts.py <repo root> <name substring>..."""
import sys, os
root = sys.argv[1]
wanted = sys.argv[2:]
rows = [line.rstrip("\n").split("\t") for line in open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8") if line.strip() and not line.startswith("#")]
for row in rows:
    name = row[0]
    if not any(w in name for w in wanted):
        continue
    path, original, replacement = row[1], row[2].replace("\\n", "\n"), row[3].replace("\\n", "\n")
    text = open(os.path.join(root, path), encoding="utf-8").read()
    print(f"hits={text.count(original)} replacement_differs={original != replacement} name={name}")
