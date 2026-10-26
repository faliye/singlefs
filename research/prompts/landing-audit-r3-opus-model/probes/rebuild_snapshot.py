"""（diff 里删掉的文件也按改前 blob 放回去，git apply 才打得上）用开工快照的 judged-vs-index.diff 重建快照那一刻被判文件的树：每份文件取 diff 头「index 改前..改后」里的改前 blob（对象库里还在），
没进 diff 的取 judged-sha256.txt 列的路径在对象库里 sha256 对得上的那一版（先试暂存区、再试 HEAD），再打 diff。用法：rebuild_snapshot.py <仓根> <快照目录> <输出树>"""
import hashlib, os, re, subprocess, sys
repo, snap, out = sys.argv[1:4]
diff = open(os.path.join(snap, "judged-vs-index.diff"), encoding="utf-8", errors="surrogateescape").read()
want = {}
for line in open(os.path.join(snap, "judged-sha256.txt"), encoding="utf-8"):
    digest, path = line.rstrip("\n").split("  ", 1); want[path] = digest
pre = {}
for block in re.split(r"(?m)^(?=diff --git )", diff):
    m = re.match(r'diff --git "?a/(.+?)"? "?b/', block)
    if not m: continue
    path = m.group(1).encode("latin-1", "backslashreplace").decode("unicode_escape").encode("latin-1").decode("utf-8") if "\\" in m.group(1) else m.group(1)
    i = re.search(r"(?m)^index ([0-9a-f]+)\.\.", block)
    pre[path] = i.group(1) if i else None
def blob(spec):
    r = subprocess.run(["git", "-C", repo, "cat-file", "-p", spec], capture_output=True)
    return r.stdout if r.returncode == 0 else None
missing = []
for path in sorted(set(want) | set(pre)):
    digest = want.get(path)
    data = None
    if path in pre:
        data = b"" if (pre[path] is None or set(pre[path]) == {"0"}) else blob(pre[path])
    else:
        for spec in (f":{path}", f"HEAD:{path}"):
            d = blob(spec)
            if d is not None and digest and hashlib.sha256(d).hexdigest() == digest: data = d; break
    if data is None: missing.append(path); continue
    target = os.path.join(out, path); os.makedirs(os.path.dirname(target), exist_ok=True)
    if path in pre and pre[path] and set(pre[path]) == {"0"}: continue
    open(target, "wb").write(data)
print("paths", len(want), "in_diff", len(pre), "missing_pre_image", missing)
