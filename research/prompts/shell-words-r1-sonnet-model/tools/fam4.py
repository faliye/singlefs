"""fam4.py <families.json> <tree> <out.json>: 一棵树一个进程，逐条喂钩子（进程内），交回 [{...,rc,reason,note}]。工作根指到不存在的目录，免得仓副本落在放行范围里。"""
import sys, os, io, json, re, types, contextlib
os.environ["SINGLEFS_WORK_ROOT"]="/var/empty/swr"
root=os.environ["SW_ROOT"]
fams=json.load(open(sys.argv[1])); tree=sys.argv[2]; out=sys.argv[3]
hook_dir=f"{root}/{tree}/.claude/hooks"
sys.path.insert(0,hook_dir)
mods={}
def load(hook):
    fname={"guard":"heavy-test-guard.sh","det":"bash-command-detector.sh"}[hook]
    src=open(f"{hook_dir}/{fname}",encoding="utf-8").read()
    code=re.search(r"3<<'PY'\n(.*?)\nPY\n",src,re.S).group(1).replace("sys.exit(main())","")
    os.environ["AGENT_HOOK_DETECTIONS"]=f"{root}/scratch/detections-{tree}-{hook}.jsonl"
    os.environ.pop("SINGLEFS_HEAVY_TESTS",None)
    sys.argv=[fname,hook_dir]
    mod=types.ModuleType("hookmod_"+hook); mod.__dict__["__name__"]="hookmod_"+hook
    exec(compile(code,fname,"exec"),mod.__dict__)
    return mod
os.chdir(f"{root}/scratch")
res=[]
for f in fams:
    hook=f["hook"]
    if hook not in mods: mods[hook]=load(hook)
    mod=mods[hook]
    cwd=f.get("cwd",f"{root}/scratch")
    if cwd=="TREE": cwd=f"{root}/{tree}"
    ti={"command":f["cmd"]}
    if f.get("bg"): ti["run_in_background"]=True
    inp={"tool_name":"Bash","cwd":cwd,"tool_input":ti}
    if f["agent"]!="main": inp["agent_type"]=f["agent"]
    if f.get("env"): os.environ.update(f["env"])
    class FI: pass
    fi=FI(); fi.buffer=io.BytesIO(json.dumps(inp).encode())
    old=sys.stdin; sys.stdin=fi
    err=io.StringIO(); o=io.StringIO()
    try:
        with contextlib.redirect_stderr(err), contextlib.redirect_stdout(o): rc=mod.main()
    except SystemExit as e: rc=e.code
    except Exception as e: rc=f"EXC {e!r}"
    finally: sys.stdin=old
    text=err.getvalue()+o.getvalue()
    first=next((l for l in text.splitlines() if "✗" in l),"")[:160]
    note=next((l for l in text.splitlines() if l.strip().startswith("!")),"")[:160]
    res.append(dict(f,rc=rc,reason=first,note=note))
json.dump(res,open(out,"w"),ensure_ascii=False)
