import os, subprocess, sys, tempfile, shutil
G, FX, D = sys.argv[1], sys.argv[2], sys.argv[3]
src = open(G, encoding="utf-8").read()
env = {k: v for k, v in os.environ.items() if k not in ("GATE_BASE", "GATE_STAGED_FROM")}
mutations = [
 ("点名一律算过", 'if grep -qF -- "$definition" "$verdict"; then named=1; break; fi', 'named=1; break', "red"),
 ("不认共用约束", r"'^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$'", r"'^\.claude/(agents/[^/]+\.md|main-agent\.md)$'", "red"),
 ("不认未跟踪文件", "git -c core.quotepath=false ls-files --others --exclude-standard -- ; ", "", "green"),
 ("不认主 agent 说明", r"'^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$'", r"'^\.claude/(agents/[^/]+\.md|agent-common\.md)$'", "green"),
 ("子目录也算定义", r"'^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$'", r"'^\.claude/(agents/.+\.md|agent-common\.md|main-agent\.md|README\.md)$|^README\.md$'", "green"),
]
def run(kind, gate_text):
    work = tempfile.mkdtemp(dir=D)
    shutil.copytree(os.path.join(FX, kind), work, dirs_exist_ok=True)
    subprocess.run(["bash", "setup.sh"], cwd=work, check=True, capture_output=True, env=env)
    gate = os.path.join(D, f"_gate_{os.path.basename(work)}.sh"); open(gate, "w", encoding="utf-8").write(gate_text)
    out = subprocess.run(["bash", gate, work], cwd=work, capture_output=True, text=True, env=env)
    expect = open(os.path.join(FX, kind, "expect"), encoding="utf-8").read().splitlines()
    want_exit = int([l for l in expect if l.startswith("exit=")][0][5:])
    misses = [l[5:] for l in expect if l.startswith("want=") and l[5:] not in out.stdout + out.stderr]
    return out.returncode == want_exit and not misses, out.returncode, misses, out.stdout.strip().splitlines()[-1:]
for kind in ("red", "green"):
    ok, rc, miss, last = run(kind, src); print(f"原脚本 {kind:<5} 判得对={ok} rc={rc}")
for name, old, new, kind in mutations:
    assert src.count(old) == 1, f"锚点不唯一：{name} 命中 {src.count(old)} 次"
    ok, rc, miss, last = run(kind, src.replace(old, new, 1))
    print(f"{'✓ 被抓' if not ok else '✗ 没红'}  {name:<12} 样本={kind:<5} rc={rc} 末行：{last}")
