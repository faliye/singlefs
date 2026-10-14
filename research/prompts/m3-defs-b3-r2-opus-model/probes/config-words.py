# 只打计数，不打值与文件名：~/.config/singlefs/ 下每份文件里 8 个字以上的词，有几个不在 multi-host.env 的词里
import os, re, shlex, sys
env_path, cfg_dir = sys.argv[1], os.path.expanduser(sys.argv[2])
def words(text):
    out = set()
    for w in re.split(r"[\s'\"=,;@]+", text):
        w = w.strip()
        if len(w) >= 8 and not w.startswith('-') and not w.startswith('#'):
            out.add(w)
    return out
env_words = words(open(env_path, encoding='utf-8', errors='replace').read())
files = sorted(os.path.join(d, f) for d, _, fs in os.walk(cfg_dir) for f in fs)
total_new = set()
for i, p in enumerate(files, 1):
    try:
        t = open(p, encoding='utf-8', errors='replace').read()
    except OSError:
        print(f"file={i} unreadable"); continue
    ws = words(t)
    new = ws - env_words
    total_new |= new
    print(f"file={i} long_words={len(ws)} not_in_multi_host_env={len(new)}")
print(f"total_not_in_multi_host_env={len(total_new)}")
