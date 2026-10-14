# 只打计数：~/.config/singlefs/ 第 N 份文件里不在 multi-host.env 的 8 字以上词，按种类分，各在冻结副本里命中几份文件；不打值、不打配置文件名
import os, re, sys
env_path, cfg_dir, repo, pick = sys.argv[1], os.path.expanduser(sys.argv[2]), sys.argv[3], int(sys.argv[4])
def words(text):
    return {w.strip() for w in re.split(r"[\s'\"=,;@()]+", text) if len(w.strip()) >= 8 and not w.strip().startswith(('-', '#'))}
env_words = words(open(env_path, encoding='utf-8', errors='replace').read())
files = sorted(os.path.join(d, f) for d, _, fs in os.walk(cfg_dir) for f in fs)
new = sorted(words(open(files[pick-1], encoding='utf-8', errors='replace').read()) - env_words)
corpus = []
for d, ds, fs in os.walk(repo):
    ds[:] = [x for x in ds if x not in ('.git', 'target', '__pycache__')]
    for f in fs:
        p = os.path.join(d, f)
        if f == 'multi-host.env': continue
        try:
            with open(p, 'rb') as h: corpus.append((p, h.read()))
        except OSError: pass
kinds = {}
for w in new:
    kind = 'path' if '/' in w else ('dotted' if '.' in w else 'word')
    hits = sum(1 for _, b in corpus if w.encode() in b)
    k = kinds.setdefault(kind, [0, 0, 0])
    k[0] += 1; k[1] += (hits == 0); k[2] += (hits > 0)
for kind, (n, zero, some) in sorted(kinds.items()):
    print(f"file={pick} kind={kind} words={n} in_repo_0_files={zero} in_repo_some_files={some}")
