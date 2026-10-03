import sys, re, os, subprocess
TREE = os.environ.get("K3_TREE", "/tmp/claude-1000/test-script-align-code-r1/tree")
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1], encoding="utf-8") if l.strip() and not l.startswith("#")]
table = open(f"{TREE}/crates/mutations.tsv", encoding="utf-8").read().split("\n")
for i, r in enumerate(rows, 1):
    name, path, orig, repl, args, test = r
    in_table = "\t".join(r) in table
    src = open(f"{TREE}/{path}", encoding="utf-8").read()
    o = orig.replace("\\n", "\n"); t = repl.replace("\\n", "\n")
    cnt = src.count(o)
    # test fn definitions in the frozen tree
    defs = subprocess.run(["grep", "-rn", f"fn {test}\\b", f"{TREE}/crates", "--include=*.rs"], capture_output=True, text=True).stdout.strip().splitlines()
    print(f"[{i}] {name}\n   in_table={in_table} anchor_hits={cnt} cols={len(r)} test_defs={len(defs)}")
    for d in defs: print("     def:", d.replace(TREE+'/', ''))
    print("   args:", args)
