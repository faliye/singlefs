"""fuzz.py <种子> <条数> <输出.json> : 随机组合（重型闸、主 agent）：带装饰的 cargo test --all、bash -c / eval 包装、命令替换、heredoc 喂 shell、
先写脚本再起、function 定义与调用、控制结构、重定向，造出命令清单，交 judge.py 用真 bash 当裁判、在修补前后两棵树上比。
只用无害的东西：假 cargo、echo、cp、ln -s、printf、bash -c；不含 rm、kill、sudo、网络。"""
import random, json, shlex, sys
random.seed(int(sys.argv[1])); N = int(sys.argv[2]); out = sys.argv[3]
P = "cargo test --all"
def pick(xs): return random.choice(xs)
def q(s): return shlex.quote(s)
def decorate_run(depth):
    base = pick(["cargo test --all", "cargo test --all", "cargo test --all", "cargo test --workspace", "cargo t --all", "cargo test -p singlefs-core"])
    forms = [
        lambda: base,
        lambda: pick(["{cargo,}", "{,}cargo", "{,} cargo", "c{,}argo", "cargo{,}", "{cargo,x}", "{x,cargo}", "''{,}cargo", "{'',cargo}"]) + " " + base.split(" ", 1)[1],
        lambda: "cargo " + pick(["{test,}", "{,test}", "{test,x}", "t{,}est", "test{,}"]) + " " + base.split(" ", 2)[2],
        lambda: base + " " + pick(["{,}", "{,x}", "{,--quiet}", "{--quiet,}"]),
        lambda: pick(["env", "env A=1", "env =1", "env = ", "env A=1 =2", "env -u X", "env -u X =1", "env -- =1", "env -C . =1", "env -C .", "env -i PATH=\"$PATH\" ORACLE_LOG=\"$ORACLE_LOG\"", "env -S 'A=1'"]) + " " + base,
        lambda: pick(["time", "time -p", "command", "exec", "nice", "nice -n 5", "timeout 20", "nohup", "setsid", "stdbuf -oL", "builtin command", "builtin exec", "A=1", "A=1 B=2", "A=\"$(date)\"", "A=`date`"]) + " " + base,
        lambda: pick(["builtin", "builtin --", "builtin builtin", "builtin echo", "builtin cd . ;", "builtin :;"]) + " " + base,
        lambda: "(" + base + ")", lambda: "{ " + base + "; }", lambda: "! " + base, lambda: base + " &" + " wait",
    ]
    return pick(forms)()
def benign():
    return pick(["echo cargo test --all", "echo 'cargo test --all'", ": cargo test --all", "printf '%s\\n' 'cargo test --all'", "true", "echo hi", "echo \"$(echo cargo test --all)\"", "cat <<'EOF'\ncargo test --all\nEOF",
                 "echo cargo{,} test --all", "echo {,} cargo", "grep -c cargo s.sh", "ls s.sh", "pwd", "bash -n s.sh", "bash -c ':'", "builtin cargo test --all", "builtin cargo", "builtin foo", "function g { cargo test --all; }", "function g { cargo test --all; }; echo hi",
                 "bash -s -- s.sh <<'EOF'\necho hi\nEOF", "echo hi | bash -s -- s.sh", "ln -s s.sh sub/dangling.sh", "ln -s s.sh sub/dangling.sh; ls sub", "X=cargo; echo $X", "cat s.sh > /dev/null", "cp s.sh d.sh", "cp s.sh d.sh 2>/dev/null; ls d.sh"])
def gen(depth=0):
    if depth >= 3:
        return decorate_run(depth) if random.random() < 0.6 else benign()
    k = random.random()
    if k < 0.25: return decorate_run(depth)
    if k < 0.33: return benign()
    if k < 0.45:
        inner = gen(depth + 1)
        w = pick(["bash -c", "sh -c", "bash -c --", "bash -xc", "bash -c -x", "bash -o pipefail -c", "bash -c -o pipefail", "bash -c -e --", "eval", "bash -O extglob -c", "bash -c -O extglob"])
        if w == "eval": return "eval " + q(inner)
        if w.startswith("sh") and ("pipefail" in w): w = "bash -c"
        return w + " " + q(inner) + pick(["", "", " arg0", " arg0 arg1"])
    if k < 0.53:
        inner = gen(depth + 1)
        return pick(["x=$({I})", "x=\"$({I})\"", "echo \"$({I})\"", "x=`{I}`", "echo `{I}`", "diff <({I}) /dev/null", "cat >({I}) </dev/null", "echo \"a$(echo \"{I}\")b\"", "echo $({I})"]).replace("{I}", inner.replace("\n", "; ") if False else inner)
    if k < 0.62:
        inner = gen(depth + 1)
        pre = pick(["", "", "cd \"$(pwd)\" ; ", "X=\"$(date)\" ", "echo \"$(date)\"; ", "echo `date`; ", "echo \"a\" ; ", "cd . ; ", "export Z=\"$(pwd)\"; ", "# note\n", "echo '$(' ; "])
        sh = pick(["bash", "sh", "bash -s", "bash -s --", "cat", "bash -s -- s.sh", "bash -x", "bash --noprofile", "bash -s -x --"])
        return pre + sh + " <<" + pick(["'EOF'", "EOF", "-'EOF'", "\"EOF\""]) + "\n" + inner + "\nEOF"
    if k < 0.72:
        inner = gen(depth + 1)
        name = pick(["t1.sh", "sub/t2.sh", "./t3.sh"])
        write = pick(["printf '%s\\n' {Q} > {N}", "echo {Q} > {N}", "cat > {N} <<'EOS'\n{I}\nEOS", "cat <<'EOS' > {N} 2>/dev/null\n{I}\nEOS", "printf '%s\\n' {Q} | tee {N} >/dev/null"]).replace("{Q}", q(inner)).replace("{I}", inner).replace("{N}", name)
        run = pick(["bash {N}", "sh {N}", ". {N}", "source {N}", "bash ./{N}", "chmod +x {N}; ./{N}", "cp {N} e.sh; bash e.sh", "cp {N} e.sh 2>/dev/null; bash e.sh", "cat {N} > e.sh 2>/dev/null; bash e.sh", "cat {N} > e.sh; bash e.sh", "ln -s {N} ln1.sh; bash ln1.sh", "ln -s ../{N} sub/ln2.sh; bash sub/ln2.sh", "ln -s {N} sub/ln3.sh; bash sub/ln3.sh", "ln -sf $PWD/{N} ln4.sh; bash ln4.sh", "bash -s -- {N}", "bash -s < {N}"]).replace("{N}", name)
        return write + "\n" + run
    if k < 0.80:
        inner = gen(depth + 1)
        body = pick(["{I}", "{I};"]).replace("{I}", inner.replace("\n", "; ") if "<<" not in inner else inner)
        call = pick(["", "", "f", "f", "$(f)", "echo f", "bash -c f", "export -f f; bash -c f", "fo{o,}", "\"f\"", "f &"])
        d = pick(["function f { %s }", "function f() { %s }", "f() { %s }", "function foo { %s }"]) % body
        if "foo" in d: call = call.replace("f", "foo") if call else call
        return d + ("\n" + call if call else "")
    if k < 0.90:
        a, b = gen(depth + 1), gen(depth + 1)
        return a + pick(["\n", "; ", "\n"]) + b
    inner = gen(depth + 1)
    return pick(["if true; then {I}\nfi", "for i in 1 2; do {I}\ndone", "case x in x) {I}\n;; esac", "{ {I}\n}", "( {I}\n)", "true; {I}", "for i in {a,}; do {I}\ndone", "for i in {a,b}; do {I}\ndone"]).replace("{I}", inner)
seen, cs = set(), []
while len(cs) < N:
    c = gen()
    if c in seen or len(c) > 900: continue
    seen.add(c); cs.append(dict(tag="fuzz", cmd=c, hook="guard", agent="main"))
json.dump(cs, open(out, "w"), ensure_ascii=False)
print(len(cs))
