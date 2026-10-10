# 用法：apply.py <mutations.py> <变异名|noret> <仓副本根>：把那一处（或 noret 那一组）改进去，原文要恰好命中一次，不中退 3。
import sys, importlib.util
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("m", sys.argv[1]); m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
name, root = sys.argv[2], sys.argv[3]
edits = [tuple(e) for e in m.NORET_EDITS] if name == "noret" else [m.MUTATIONS[name]]
for f, old, new in edits:
    old = old.replace("\\n", "\n") if name == "noret" else old
    p = root + "/" + f
    s = open(p).read()
    n = s.count(old)
    if n != 1:
        print(f"anchor count {n} for {name}: {old[:60]!r}"); sys.exit(3)
    open(p, "w").write(s.replace(old, new))
print(f"applied {name} ({len(edits)} edits)")
