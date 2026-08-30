import re, sys
index_path, *bodies = sys.argv[1:]
idx = open(index_path, encoding="utf-8").read()
CN = {"一":1,"二":2,"两":2,"三":3,"四":4,"五":5,"六":6,"七":7,"八":8,"九":9,"十":10}
def open_count(s):
    m = re.search(r'([0-9]+|[一二两三四五六七八九十])\s*[项条]未定', s)
    if not m: return 0
    g = m.group(1)
    return int(g) if g.isdigit() else CN[g]
def kind(s):
    for k in ("已定","半定","待定"):
        if s.lstrip("*").startswith(k): return k
    return "?"
fail = 0
for p in bodies:
    body = open(p, encoding="utf-8").read()
    t = body.split("\n", 1)[0]
    m = re.match(r'## (D\d+) .+? —— (.+)$', t)
    if not m: continue
    num, bs = m.group(1), m.group(2)
    row = re.search(r'^\| %s（[^|]*\) *\| ([^|]+)\|' % num, idx, flags=re.M)
    if not row:
        row = re.search(r'^\| %s（[^|]*） *\| ([^|]+)\|' % num, idx, flags=re.M)
    if not row:
        print(f"  ✗ {num} 在 decisions.md 索引里找不到对应行"); fail = 1; continue
    isx = row.group(1).strip()
    if open_count(bs) != open_count(isx):
        print(f"  ✗ {num} 正文标题说「{open_count(bs)} 项未定」，索引行说「{open_count(isx)} 项未定」")
        fail = 1
    elif kind(bs) != kind(isx):
        print(f"  ✗ {num} 正文标题是「{kind(bs)}」，索引行是「{kind(isx)}」")
        fail = 1
if not fail:
    print("  ✓ 决策索引行与正文标题一致（状态与未定项条数）")
sys.exit(1 if fail else 0)
