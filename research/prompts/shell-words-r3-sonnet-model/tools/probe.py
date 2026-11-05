"""probe.py [--det] [--agent 名] [--bg] [--cwd 路径] <命令>… : 每条命令：真 bash（进程组）裁判结果与 pre、r1、r2、now 四棵树上钩子的退出码。默认重型闸 + 主 agent；--det 检出钩子 + clerk。"""
import sys, json, subprocess, os, tempfile
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
a = sys.argv[1:]; hook = "guard"; agent = None; bg = False; cwd = None
while a and a[0].startswith("--"):
    o = a.pop(0)
    if o == "--det": hook = "det"
    elif o == "--agent": agent = a.pop(0)
    elif o == "--bg": bg = True
    elif o == "--cwd": cwd = a.pop(0)
agent = agent or ("main" if hook == "guard" else "clerk")
cases = [dict(cmd=c, hook=hook, agent=agent, bg=bg, **({"cwd": cwd} if cwd else {})) for c in a]
tmp = tempfile.mkdtemp(dir=ROOT + "/tmp"); json.dump(cases, open(tmp + "/c.json", "w"))
subprocess.run(["python3", "-I", ROOT + "/tools/myoracle.py", tmp + "/c.json", tmp + "/o.json", "4"], stdout=subprocess.DEVNULL, check=True)
o = json.load(open(tmp + "/o.json")); res = {}
for t in ("pre", "r1", "r2", "now"):
    subprocess.run(["python3", "-I", ROOT + "/tools/feedhooks.py", tmp + "/c.json", t, tmp + f"/{t}.json", "4"], stdout=subprocess.DEVNULL, check=True)
    res[t] = json.load(open(tmp + f"/{t}.json"))
for i, c in enumerate(cases):
    x = o[i]; truth = 2 if (x["heavy"] if hook == "guard" else x["detached"]) else 0
    print(f"{c['cmd']!r}\n    bash rc={x['rc']} err={x['err1'][:70]!r} 跑出重活={'是' if truth == 2 else '否'}(应{truth}) | pre={res['pre'][i]['rc']} r1={res['r1'][i]['rc']} r2={res['r2'][i]['rc']} now={res['now'][i]['rc']}" + (f" | now理由:{res['now'][i]['reason'][:90]}" if res['now'][i]['rc'] == 2 else ""))
import shutil; shutil.rmtree(tmp, ignore_errors=True)
