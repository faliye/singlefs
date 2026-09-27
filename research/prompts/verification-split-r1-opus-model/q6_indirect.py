"""每个 crates/*/tests/*.rs：找出直接调 enumerate_layer0* 或自写枚举的非测试函数（helper），再找调 helper 的 #[test]（同文件内传递闭包），
逐条列出：文件、测试函数、经哪个 helper、helper 里枚举调用传的 expansion 实参（quick_tier_expansion / full_expansion / 别的）、#[ignore]、豁免注明。"""
import glob, os, re, sys, importlib.util
sys.dont_write_bytecode = True
S = sys.argv[1] if len(sys.argv) > 1 else "/tmp/claude-1000/three-way-attack/verification-split-r1/snap"
spec = importlib.util.spec_from_file_location("ccc", S + "/research/scripts/crash-case-check.py"); ccc = importlib.util.module_from_spec(spec); spec.loader.exec_module(ccc)
ANY_FN = re.compile(r"((?:[ \t]*#\[[^\]]*\][ \t]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*(?:<[^>{]*>)?\s*\(")
ENUM = re.compile(r"\b(enumerate_layer0\w*|enumerate_every_subset\w*)\s*(?:::<[^>]*>)?\s*\(")
EXPANSION = re.compile(r"&\s*(quick_tier_expansion|full_expansion|\w*expansion\w*)")
rows = []
for path in sorted(glob.glob(S + "/crates/*/tests/*.rs")):
    text = open(path, encoding="utf-8").read()
    fns = {}
    for m in ANY_FN.finditer(text):
        body = ccc.function_body(text, m.end())
        fns.setdefault(m.group(2), []).append((m.group(1), body, text.count("\n", 0, m.start(2)) + 1))
    direct = {name for name, defs in fns.items() for attrs, body, _ in defs if ENUM.search(body)}
    # helpers: non-test functions that reach an enumeration (transitively)
    reach = {name for name in direct if all("#[test]" not in a for a, _, _ in fns[name])}
    changed = True
    while changed:
        changed = False
        for name, defs in fns.items():
            if name in reach or any("#[test]" in a for a, _, _ in defs):
                continue
            if any(re.search(r"\b" + re.escape(h) + r"\s*\(", body) for _, body, _ in defs for h in reach):
                reach.add(name); changed = True
    for name, defs in fns.items():
        for attrs, body, line in defs:
            if "#[test]" not in attrs:
                continue
            via = sorted(h for h in reach if re.search(r"\b" + re.escape(h) + r"\s*\(", body))
            directly = bool(ENUM.search(body))
            if not via and not directly:
                continue
            exps = set(EXPANSION.findall(body))
            for h in via:
                for _, hb, _ in fns[h]:
                    exps |= set(EXPANSION.findall(hb))
            rows.append((os.path.relpath(path, S), line, name, "direct" if directly else "", ",".join(via), ",".join(sorted(exps)) or "-",
                         "ignore" if re.search(r"#\[ignore\b", attrs) else "", "waiver" if "not-a-crash-case" in attrs else ""))
for r in rows: print("\t".join(map(str, r)))
