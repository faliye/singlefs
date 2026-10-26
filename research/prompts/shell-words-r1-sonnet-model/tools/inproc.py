"""inproc.py <tree> <hook:guard|det> <agent:main|名字> <corpus.json> <out.json> : 环境 SW_ROOT；在进程内批量喂钩子，交回 {命令: [rc, 首行原因]}"""
import sys, os, io, json, re, types, contextlib
root=os.environ["SW_ROOT"]
tree,hook,agent,corpus,out=sys.argv[1:6]
hook_dir=f"{root}/{tree}/.claude/hooks"
fname={"guard":"heavy-test-guard.sh","det":"bash-command-detector.sh"}[hook]
src=open(f"{hook_dir}/{fname}",encoding="utf-8").read()
m=re.search(r"3<<'PY'\n(.*?)\nPY\n",src,re.S)
code=m.group(1).replace("sys.exit(main())","")
os.environ["AGENT_HOOK_DETECTIONS"]=f"{root}/scratch/detections-{tree}-{hook}.jsonl"
os.environ.pop("SINGLEFS_HEAVY_TESTS",None)
sys.argv=[fname,hook_dir]
sys.path.insert(0,hook_dir)
mod=types.ModuleType("hookmod"); mod.__dict__["__name__"]="hookmod"
exec(compile(code,fname,"exec"),mod.__dict__)
cmds=json.load(open(corpus))
res={}
os.chdir(f"{root}/scratch")
for c in cmds:
    inp={"tool_name":"Bash","cwd":f"{root}/scratch","tool_input":{"command":c}}
    if agent!="main": inp["agent_type"]=agent
    data=json.dumps(inp).encode()
    class FakeIn: pass
    fi=FakeIn(); fi.buffer=io.BytesIO(data)
    old=sys.stdin; sys.stdin=fi
    err=io.StringIO(); o=io.StringIO()
    try:
        with contextlib.redirect_stderr(err), contextlib.redirect_stdout(o):
            rc=mod.main()
    except SystemExit as e: rc=e.code
    except Exception as e: rc=f"EXC {e!r}"; 
    finally: sys.stdin=old
    text=err.getvalue()+o.getvalue()
    first=next((l for l in text.splitlines() if "✗" in l),"")[:160]
    res[c]=[rc,first]
json.dump(res,open(out,"w"),ensure_ascii=False)
print(tree,hook,agent,len(res),sum(1 for v in res.values() if v[0]==2))
