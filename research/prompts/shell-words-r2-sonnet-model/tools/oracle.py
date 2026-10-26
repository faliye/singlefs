"""oracle.py <模板目录> <假 cargo 所在 bin> <命令清单.json> <输出.json>
命令清单：[{tag, cmd, hook, agent, ...}]。每条命令在模板目录的一份新拷贝里用真 bash（bash -c）跑一遍（5 秒超时），
PATH 前面放假 cargo（只记参数）；假 detached-run.sh 在模板的 scripts/hosts/ 里（只记参数）。
交回 expect：guard 看日志里有没有 cargo test|t 带 --all 或 --workspace，det 看有没有 `detached-run start`；有算 2（该拒），没有算 0。
另记 bash_rc、bash_stderr 首行、log。"""
import sys, os, json, subprocess, tempfile, shutil, shlex, resource, re, time
tmpl, binp, inp, outp = sys.argv[1:5]
items = json.load(open(inp))
res = []
base = tempfile.mkdtemp(prefix="oracle-", dir=os.path.dirname(os.path.abspath(outp)))
for it in items:
    if it.get("noexec"):
        res.append(dict(it, bash_rc="未跑", bash_err="", log=[])); continue
    d = tempfile.mkdtemp(dir=base)
    sb = os.path.join(d, "scratch")
    shutil.copytree(tmpl, sb, symlinks=True)
    os.makedirs(os.path.join(d, "tmp"))
    log = os.path.join(d, ".oracle-log")
    open(log, "w").close()
    env = {"PATH": binp + ":/usr/bin:/bin", "ORACLE_LOG": log, "HOME": d, "LC_ALL": "C.UTF-8", "TIMEFORMAT": ""}   # TIMEFORMAT 空：time 关键字不往 stderr 打耗时（随机器负载变）
    if it.get("env"): env.update(it["env"])
    try:
        p = subprocess.run(["bash", "-c", it["cmd"].replace("/tmp/", d + "/tmp/")], cwd=sb, env=env, capture_output=True, text=True, timeout=30, stdin=subprocess.DEVNULL, preexec_fn=lambda: resource.setrlimit(resource.RLIMIT_AS, (1 << 30, 1 << 30)))   # 跑飞的命令（花括号爆炸之类）最多吃 1G 虚存
        rc, err = p.returncode, p.stderr.strip().splitlines()[:1]
        if err:   # 沙盒、假 cargo、草稿根的路径换成占位，免得报告里的错误行随草稿目录变
            work_root = os.path.dirname(os.path.dirname(os.path.abspath(binp)))
            for real, placeholder in ((d, "<sbx>"), (os.path.abspath(binp), "<bin>"), (os.path.abspath(tmpl), "<template>"), (work_root, "<work>")):
                err[0] = err[0].replace(real, placeholder)
    except subprocess.TimeoutExpired:
        rc, err = "timeout", []
    if re.search(r"(?<![&>|])&(?![&>])", it["cmd"]):
        time.sleep(0.6)   # 命令里有放后台的 &：等后台的假 cargo 把日志写完再读
    lines = open(log).read().splitlines()
    heavy = any((lambda t: t[:1] == ["cargo"] and t[1:2] and t[1] in ("test", "t") and ("--all" in t or "--workspace" in t))(shlex.split(l)) for l in lines if l.startswith("cargo"))
    detached = any(l.startswith("detached-run start") for l in lines)
    exp = 2 if (heavy if it["hook"] == "guard" else detached) else 0
    res.append(dict(it, expect=exp, bash_rc=rc, bash_err=(err[0][:100] if err else ""), log=lines))
json.dump(res, open(outp, "w"), ensure_ascii=False)
shutil.rmtree(base)
print(len(res), sum(1 for r in res if r["expect"] == 2))
