# D5-5：看门狗「草稿根里超阈值的编译目录与仓副本」报告（scratch_hog_report）与「被停的同样算」那句文字。
# 用法：python3 -I s14_watchdog.py <改前树> <改后树> <工作目录>
import datetime, hashlib, importlib.util, os, shutil, sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as c
trees, W = c.parse_trees(sys.argv)
def mk(p, files, age_h):
    os.makedirs(p, exist_ok=True)
    for f in files:
        fp = os.path.join(p, f); os.makedirs(os.path.dirname(fp), exist_ok=True); open(fp, "w").write("x")
    t = time.time() - age_h * 3600
    for dp, dn, fn in os.walk(p):
        for n in dn + fn: os.utime(os.path.join(dp, n), (t, t))
        os.utime(dp, (t, t))
for label, tree in trees:
    spec = importlib.util.spec_from_file_location("aw_" + label, f"{tree}/scripts/agents/agent-watch.py")
    m = importlib.util.module_from_spec(spec); sys.path.insert(0, f"{tree}/scripts/agents"); spec.loader.exec_module(m)
    root = f"{W}/hogroot-{label}"; shutil.rmtree(root, ignore_errors=True)
    mk(f"{root}/old-copy-git", [".git/HEAD", "crates/x.rs"], 10); mk(f"{root}/old-copy-nogit", ["Cargo.toml", "crates/a/lib.rs"], 10)
    mk(f"{root}/old-copy-cargo-only", ["Cargo.toml", "src/lib.rs"], 10); mk(f"{root}/old-notes", ["notes.md"], 10)
    mk(f"{root}/old-target/target", ["debug/x"], 10); open(f"{root}/old-target/target/CACHEDIR.TAG", "w").write("Signature: 8a477f597d28d172789f06886806bc55\n")
    for stale in (f"{root}/old-target/target/CACHEDIR.TAG", f"{root}/old-target/target", f"{root}/old-target"):
        os.utime(stale, (time.time() - 10 * 3600,) * 2)   # 写完签名文件之后目录的修改时刻变成现在，要一并拨回去
    mk(f"{root}/fresh-copy", [".git/HEAD"], 1)
    lines = m.scratch_hog_report([root], datetime.datetime.now(datetime.timezone.utc), 6)
    print(f"name=S14 tree={label} case=scratch-hog-report reported={len(lines)} names={sorted(os.path.basename(l.split('：')[0].split()[-1]) if '：' in l else l[:20] for l in lines)}")
    shutil.rmtree(root, ignore_errors=True)
    sha = hashlib.sha256(open(f"{tree}/scripts/agents/agent-watch.py", "rb").read()).hexdigest()[:12]
    text = open(f"{tree}/.claude/main-agent.md", encoding="utf-8").read()
    print(f"name=S14 tree={label} case=agent-watch-py-sha256-prefix sha={sha} main_agent_says_stopped_counts={'被停的同样算' in text}")
