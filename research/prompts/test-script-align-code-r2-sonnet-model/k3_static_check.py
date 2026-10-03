#!/usr/bin/env python3
"""J3 静态核：冻结副本里变异表第 FIRST..LAST 行（默认 r1→r2 有变动的行与新增的行）。
每行查：六段齐；文件在；原文在文件里命中次数（要恰好 1）；替换文与原文不同；点名的测试名在 crates 里 fn 定义个数（要 1）；
测试函数前一行起往上的属性里有没有 #[ignore]；参数里的 --features / --test / --lib / --bin 与测试所在文件对不对得上；
测试所在文件头部有没有 #![cfg(feature …)]，参数里带不带这个特性。只读，不跑任何测试。"""
import os, re, sys, subprocess
TREE = os.environ.get("K3_TREE", "/tmp/claude-1000/test-script-align-code-r2/tree")
OLD = os.environ.get("K3_OLD_TABLE", "/tmp/claude-1000/test-script-align-code-r1/tree/crates/mutations.tsv")
table_path = f"{TREE}/crates/mutations.tsv"
lines = open(table_path, encoding="utf-8").read().split("\n")
old = set(open(OLD, encoding="utf-8").read().split("\n"))
chosen = [i for i, l in enumerate(lines, 1) if l.strip() and not l.startswith("#") and l not in old]
if len(sys.argv) > 1:
    chosen = [int(x) for x in sys.argv[1:]]
seen_names = {}
for i, l in enumerate(lines, 1):
    if l.strip() and not l.startswith("#"):
        seen_names.setdefault(l.split("\t")[0], []).append(i)
def fn_defs(test):
    out = subprocess.run(["grep", "-rnE", rf"fn {re.escape(test)}\b", f"{TREE}/crates", "--include=*.rs"], capture_output=True, text=True).stdout
    return [d.replace(TREE + "/", "") for d in out.strip().splitlines() if d]
bad = 0
for i in chosen:
    cols = lines[i - 1].split("\t")
    if len(cols) != 6:
        print(f"row {i}: cols={len(cols)} BAD"); bad += 1; continue
    name, path, orig, repl, args, test = cols
    problems = []
    f = f"{TREE}/{path}"
    if not os.path.exists(f):
        print(f"row {i} {name[:40]}: file missing {path}"); bad += 1; continue
    src = open(f, encoding="utf-8").read()
    o = orig.replace("\\n", "\n"); t = repl.replace("\\n", "\n")
    hits = src.count(o)
    if hits != 1: problems.append(f"anchor_hits={hits}")
    if o == t: problems.append("replacement==original")
    if len(seen_names[name]) != 1: problems.append(f"name_dup_rows={seen_names[name]}")
    defs = fn_defs(test)
    if len(defs) != 1: problems.append(f"test_defs={len(defs)}")
    tfile, ignored, cfg_feats = None, False, []
    if len(defs) == 1:
        tfile, lineno = defs[0].split(":")[0], int(defs[0].split(":")[1])
        tl = open(f"{TREE}/{tfile}", encoding="utf-8").read().split("\n")
        k = lineno - 2
        while k >= 0 and (tl[k].lstrip().startswith("#[") or tl[k].lstrip().startswith("///") or tl[k].strip() == ""):
            if "#[ignore" in tl[k]: ignored = True
            if tl[k].strip() == "" and k < lineno - 3: break
            k -= 1
        if ignored: problems.append("test_is_ignored")
        for tline in tl[:30]:
            m = re.match(r'#!\[cfg\(feature = "([a-z\-]+)"\)\]', tline.strip())
            if m: cfg_feats.append(m.group(1))
        # 参数
        m_feat = re.search(r'--features\s+"?([a-z,\-]+)"?', args)
        feats = set(m_feat.group(1).split(",")) if m_feat else set()
        for cf in cfg_feats:
            if cf not in feats: problems.append(f"file_cfg_feature_{cf}_not_in_args")
        m_test = re.search(r'--test\s+(\S+)', args)
        if m_test and tfile.startswith("crates/") and "/tests/" in tfile:
            if os.path.basename(tfile)[:-3] != m_test.group(1): problems.append(f"args_--test_{m_test.group(1)}_but_test_in_{os.path.basename(tfile)}")
        if "--lib" in args and "/src/" not in tfile: problems.append("args_--lib_but_test_not_in_src")
        m_bin = re.search(r'--bin\s+(\S+)', args)
        if m_bin and not tfile.endswith(f"/bin/{m_bin.group(1)}.rs"): problems.append("args_--bin_mismatch")
    status = "OK " if not problems else "BAD"
    if problems: bad += 1
    print(f"{status} row {i} test_file={tfile} :: {name[:70]} :: {','.join(problems)}")
print(f"rows_checked={len(chosen)} rows_with_problems={bad}")
