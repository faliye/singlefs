"""代码轮第二轮攻方：变异行静态核。逐行：原文在文件里恰好命中几次、替换文是否不同、点名测试的 fn 定义几处与落在哪、
参数的目标（--test/--lib/--bin）罩不罩得住那份定义、筛选词是不是测试路径的子串、特性罩不罩得住文件与用例的 cfg、用例是否 #[ignore]。"""
import os, re, sys, glob
TREE = os.environ.get("K3_TREE", "/tmp/claude-1000/test-script-align-code-r2/tree")
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1], encoding="utf-8")]
def fn_defs(name):
    out = []
    for path in glob.glob(f"{TREE}/crates/**/*.rs", recursive=True):
        text = open(path, encoding="utf-8").read()
        for m in re.finditer(r"\bfn\s+" + re.escape(name) + r"\s*[<(]", text):
            out.append((path, text, m.start()))
    return out
def attrs_before(text, pos):
    # 这个 fn 之前连续的属性行（# 开头）与文档
    lines = text[:pos].split("\n")[:-1]
    attrs = []
    for line in reversed(lines):
        s = line.strip()
        if s.startswith("#[") or s.startswith("///") or s.startswith("//") or s == "" and False:
            attrs.append(s)
        elif s.endswith(")]") or s.endswith("]") and attrs and not s.startswith("fn"):
            attrs.append(s)
        else:
            break
    return attrs
def module_path(path, text, pos):
    # 库内用例：src 下的文件名当模块，加上 mod tests 一层（粗：数 pos 之前未闭合的 mod 块名字）
    mods = []
    for m in re.finditer(r"\bmod\s+(\w+)\s*\{", text[:pos]):
        depth = text[m.end():pos].count("{") - text[m.end():pos].count("}")
        if depth >= 0:
            mods.append(m.group(1))
    return mods
for row in rows:
    number, name, path, orig, repl, args, test = row
    src = open(f"{TREE}/{path}", encoding="utf-8").read()
    o = orig.replace("\\n", "\n"); t = repl.replace("\\n", "\n")
    hits = src.count(o)
    defs = fn_defs(test)
    features = set()
    m = re.search(r"--features\s+(\S+)", args)
    if m: features = set(m.group(1).split(","))
    target = re.search(r"--(test|lib|bin)\b\s*(\S*)", args)
    filt = args.split(" -- ", 1)[1].strip() if " -- " in args else ""
    verdicts = []
    for dpath, dtext, dpos in defs:
        rel = dpath.replace(TREE + "/", "")
        file_cfg = re.findall(r'#!\[cfg\(feature = "([\w-]+)"\)\]', dtext)
        attrs = attrs_before(dtext, dpos)
        fn_cfg = [f for a in attrs for f in re.findall(r'feature = "([\w-]+)"', a) if a.startswith("#[cfg")]
        ignored = any(a.startswith("#[ignore") for a in attrs)
        is_test = any(a.startswith("#[test]") for a in attrs)
        should_panic = [a for a in attrs if a.startswith("#[should_panic")]
        if target and target.group(1) == "test":
            in_target = re.search(r"/tests/" + re.escape(target.group(2)) + r"\.rs$", rel) is not None
            pathname = test
        elif target and target.group(1) == "lib":
            in_target = "/src/" in rel and "/src/bin/" not in rel
            modname = os.path.splitext(os.path.basename(rel))[0]
            pathname = "::".join([modname] + module_path(dpath, dtext, dpos) + [test])
        elif target and target.group(1) == "bin":
            in_target = rel.endswith(f"/src/bin/{target.group(2)}.rs")
            pathname = test
        else:
            in_target = None; pathname = test
        filter_ok = (filt == "" or filt.replace("--exact", "").strip() in pathname)
        missing_features = sorted(set(file_cfg + fn_cfg) - features)
        verdicts.append(f"def={rel} in_target={in_target} is_test={is_test} ignored={ignored} filter_ok={filter_ok} path={pathname} missing_features={missing_features} should_panic={len(should_panic)}")
    print(f"ROW {number} hits={hits} replacement_differs={o != t} test_defs={len(defs)} name={name}")
    for v in verdicts: print("   ", v)
