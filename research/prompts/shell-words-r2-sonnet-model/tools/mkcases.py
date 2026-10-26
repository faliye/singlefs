"""mkcases.py <输出目录> : 按族造命令清单（cases-*.json），每条 {tag, cmd, hook, agent}，裁判由 judge.py 用真 bash 现跑。
模板里的占位：{P} 要起的重活（重型闸：cargo test --all；检出器：bash scripts/hosts/detached-run.sh start --label m -- sleep 1），
{SRC} 源脚本（s.sh / scripts/hosts/detached-run.sh），{UP} 上一层的源脚本（../s.sh / ../scripts/hosts/detached-run.sh），
{RUN:路径} 起这个路径的脚本（bash 路径 / bash 路径 start --label m -- sleep 1）。
每个模板对重型闸（主 agent）与检出器（clerk）各出一条。"""
import json, os, re, sys
out = sys.argv[1]
G = dict(P="cargo test --all", SRC="s.sh", UP="../s.sh", RUNTAIL="", hook="guard", agent="main")
D = dict(P="bash scripts/hosts/detached-run.sh start --label m -- sleep 1", SRC="scripts/hosts/detached-run.sh", UP="../scripts/hosts/detached-run.sh",
         RUNTAIL=" start --label m -- sleep 1", hook="det", agent="clerk")
def expand(t, k):
    s = t.replace("{P}", k["P"]).replace("{SRC}", k["SRC"]).replace("{UP}", k["UP"])
    return re.sub(r"\{RUN:([^}]*)\}", lambda m: "bash " + m.group(1) + k["RUNTAIL"], s)
def both(tag, templates, kinds=(G, D)):
    res = []
    for t in templates:
        for k in kinds:
            res.append(dict(tag=tag, cmd=expand(t, k), hook=k["hook"], agent=k["agent"], tpl=t))
    return res
def dump(name, cases):
    seen, uniq = set(), []
    for c in cases:
        key = (c["cmd"], c["hook"], c["agent"])
        if key in seen: continue
        seen.add(key); uniq.append(c)
    json.dump(uniq, open(os.path.join(out, f"cases-{name}.json"), "w"), ensure_ascii=False)
    print(name, len(uniq))

# ---- A1 花括号：空份、引号空串、补零序列 ----
brace = ["{P}", "{cargo,} test --all", "{,} cargo test --all", "{,}{,} cargo test --all", "{,cargo,} test --all", "{cargo,x} test --all", "cargo {,test} --all", "cargo test{,} --all",
         "cargo test --all{,}", "cargo test {--all,}", "cargo test {,--all}", "cargo {test,} --all", "c{,}argo test --all", "cargo{,} test --all", "{,}cargo test --all",
         "{\"\",cargo} test --all", "{'',cargo} test --all", "''{,} cargo test --all", "\"\"{,} cargo test --all", "{'',''} cargo test --all", "{a,} {cargo,} test --all",
         "X=1 {,} cargo test --all", "env {,} cargo test --all", "env X=1 {,} cargo test --all", "nice {,} cargo test --all", "nice -n 5 {,} cargo test --all", "time {,} cargo test --all",
         "time -p {,} cargo test --all", "timeout 9 {,} cargo test --all", "command {,} cargo test --all", "exec {,} cargo test --all", "sudo {,} cargo test --all",
         "bash -c {,} 'cargo test --all'", "bash {,} -c 'cargo test --all'", "bash -c '{,} cargo test --all'", "eval {,} 'cargo test --all'", "eval '{,}' 'cargo test --all'",
         "{bash,} -c 'cargo test --all'", "bash -c {'cargo test --all',}", "nice -n {,} cargo test --all", "nice {-n,} 5 cargo test --all", "env {-i,} cargo test --all",
         "env -u {,} cargo test --all", "env -u {A,} cargo test --all", "env {-u,} A cargo test --all", "env {-C,} . cargo test --all",
         "timeout {,} 5 cargo test --all", "timeout {5,} cargo test --all", "{timeout,} 5 cargo test --all", "{timeout,5} cargo test --all",
         "> {,}", "cat > {,}x", "cat s.sh > d{,}.sh && bash d.sh", "cat s.sh > {,}d.sh && bash d.sh", "cp s.sh {,}d.sh && bash d.sh", "cp {,}s.sh d.sh && bash d.sh",
         "cp s.sh d{,}.sh && bash d.sh", "cp s.sh {d,e}.sh && bash d.sh", "cp s.sh d.sh{,.bak} && bash d.sh", "cp s.sh d{1,2}.sh && bash d1.sh",
         "cp s.sh d{01..02}.sh && bash d01.sh", "cp s.sh d{1..02}.sh && bash d01.sh", "cp s.sh d{1..02}.sh && bash d1.sh", "cp s.sh d{01..2}.sh && bash d02.sh", "cp s.sh d{001..2}.sh && bash d001.sh",
         "cp s.sh d{-01..1}.sh && bash d-01.sh", "cp s.sh d{-1..01}.sh && bash d-1.sh", "cp s.sh d{0..00}.sh && bash d0.sh", "cp s.sh d{0..00}.sh && bash d00.sh",
         "cp s.sh d{1..05..2}.sh && bash d03.sh", "cp s.sh s{01..100..50}.sh && bash s001.sh", "cp s.sh s{01..100..50}.sh && bash s051.sh", "cp s.sh s{01..100..50}.sh && bash s01.sh", "cp s.sh s{01..100..50}.sh && bash s51.sh", "cp s.sh s{00..100..50}.sh && bash s000.sh", "cp s.sh s{1..100..50}.sh && bash s51.sh", "cp s.sh s{01..10..3}.sh && bash s04.sh", "cp s.sh s{09..100..30}.sh && bash s039.sh", "cp s.sh d{1..05..2}.sh && bash d3.sh", "cp s.sh d{a..c}.sh && bash db.sh", "cp s.sh d{c..a}.sh && bash db.sh",
         "cp s.sh d{1,2}{a,b}.sh && bash d2b.sh", "bash s{01..02}.sh", "bash s{1..2}.sh", "bash s{,1}.sh", "bash {s,l}.sh", "bash {l,s}.sh", "bash {light,s}.sh",
         "bash {,light}{,.sh}", "bash {s.sh,}", "bash {,s.sh}", "bash {,} s.sh", "bash {-x,} s.sh", "bash s.sh {,}", "bash -n {,s.sh}",
         "X={cargo,} ; $X test --all", "X=cargo ; {$X,} test --all", "echo {cargo,} test --all", "echo {cargo,}; {P}", "{P}; echo {,}", "echo {,} && {P}",
         "if {true,}; then {P}; fi", "for i in {,a}; do {P}; done", "for i in {,}; do {P}; done", "for i in {a,}; do echo $i; done; {P}",
         "case x in {x,y}) {P};; esac", "[[ -n {a,} ]] && {P}", "[ {,} ] || {P}", "{P} {,}", "{P} >{,}", "{P} 2>{,}", "{ {P}; }", "{,}; {P}", "{,} {P}"]
dump("brace", both("brace", brace))

# ---- A2 heredoc 挂在哪：双引号 / 反引号 / 注释 / 进程替换 / 算术等前缀 ----
body = "\ncargo test --all\nEOF"
heads_top = ['echo "$(echo a)"; ', 'echo "$(echo "x")"; ', 'x="$(date)" ', 'x="$(date)"; ', "echo \\$\\(; ", "echo '$('; ", 'echo "\\$("; ', "echo \"a\" '$('; ", "echo $((1+2)); ",
             'echo ${x:-"$(date)"}; ', 'echo ${x:-$(date)}; ', 'echo "${x:-$(echo ")")}"; ', "echo `date`; ", 'echo "`date`"; ', 'echo "a`echo b`c"; ', 'echo "$(echo `date`)"; ',
             '[[ -n "$(date)" ]] && ', 'cat <(echo a) | ', 'diff <(echo a) <(echo b); ', 'echo "<(" ; ', 'echo ">(" ; ', 'f() { echo "$(date)"; }; ', 'case x in x) echo "$(date)" ;; esac; ',
             'if [ "$(echo a)" = a ]; then ', 'echo "$(echo a)" "$(echo b)"; ', 'echo "$(echo a)""$(echo b)"; ', 'echo "x" "$(echo a)"; ', "echo '\"' ; ", "echo \"'\" ; ", 'echo "a\\"b$(date)"; ',
             'echo "#"; ', 'echo a#b; ', 'echo $#; ', 'echo ${#x}; ', 'echo "$(echo a # b\n)"; ', "echo a # note\n", 'echo "a" "$(echo \'"\')"; ', "echo \"$(echo ')')\"; ", 'echo "$( (echo a) )"; ',
             'echo "$(echo a | tr a b)" | ', 'echo "$(echo a)" | ', 'echo "$(echo a)" && ', 'echo "$(echo a)" || ', 'echo "$(echo a)" ; ', 'echo "$(echo a)" & ', 'true "$(echo a)" ; ',
             'export X="$(date)"; ', 'declare Y="$(date)" ; ', 'X="$(date)" Y="$(date)" ', 'X="$(date)" Y=`date` ', 'X=`date` Y="$(date)" ', "X='$(' Y=\"$(date)\" ", 'X="a" Y="$(echo b)"; ']
feeds = ["bash <<'EOF'" + body, "sh <<'EOF'" + body, "bash -s <<'EOF'" + body, "cat <<'EOF'" + body, "bash -c 'echo hi' <<'EOF'" + body, "bash <<EOF" + body, "bash <<-'EOF'" + body]
cs = []
for h in heads_top:
    for f in feeds:
        cs.append(h + f)
# 挂在还没收尾的替换里
opens = [('x=$(', '\n)'), ('x="$(', '\n)"'), ('x=`', '\n`'), ('diff <(', '\n) /dev/null'), ('echo "$(echo "$(', '\n)")"'), ('echo "$(echo `', '\n`)"'), ('echo $(echo "$(', '\n)")'),
         ('x=$(echo a; ', '\n)'), ('x=$(echo "a"; ', '\n)'), ('x=$(echo "$(echo b)"; ', '\n)'), ('x="$(echo "b"; ', '\n)"'), ('cat >(', '\n) </dev/null'), ('x=$( (echo a); ', '\n)'),
         ('x=$(case x in x) echo a;; esac; ', '\n)'), ("x=$(echo '(' ; ", '\n)'), ('x=$(echo ")" ; ', '\n)')]
for h, c in opens:
    for f in feeds[:4]:
        cs.append(h + f + c)
dump("heredoc", [dict(tag="heredoc", cmd=c, hook="guard", agent="main") for c in cs])

# ---- A3 重定向的 fd 号与拷贝 ----
ops = [">", "1>", "2>", "3>", "9>", "10>", "01>", "0>", ">>", "1>>", "2>>", "&>", "&>>", ">&", ">|", "1>|", "2>|", "\"2\">", "\\2>", "''2>", "2 >", "1 >", "a2>", "2>&1 >", ">&2 >", "2>/dev/null >", ">/dev/null 2>", "2>x.err >", "{fd}>"]
tp = []
for op in ops:
    tp.append("cat {SRC} " + op + " d.sh && {RUN:d.sh}")
    tp.append("cat {SRC} " + op + " d.sh 2>/dev/null && {RUN:d.sh}")
    tp.append("cat {SRC} 2>e.txt " + op + " d.sh && {RUN:d.sh}")
    tp.append("cat " + op + " d.sh < {SRC} && {RUN:d.sh}")
    tp.append("cat " + op + " d.sh {SRC} && {RUN:d.sh}")
    tp.append("cat {SRC} " + op + " d.sh ; {RUN:d.sh}")
tp += ["cat {SRC} > d.sh 2> e.sh && {RUN:e.sh}", "cat {SRC} > d.sh 2> e.sh && {RUN:d.sh}", "cat {SRC} 2> e.sh > d.sh && {RUN:d.sh}", "cat {SRC} 2> e.sh > d.sh && {RUN:e.sh}",
       "cat {SRC} 1> d.sh 2> e.sh && {RUN:d.sh}", "cat {SRC} 1> d.sh 2>&1 && {RUN:d.sh}", "cat {SRC} 2>&1 > d.sh && {RUN:d.sh}", "cat {SRC} > d.sh 2>&1 && {RUN:d.sh}",
       "cat {SRC} >d.sh 2>/dev/null; {RUN:d.sh}", "cat {SRC} >d.sh 2>/dev/null || {RUN:d.sh}", "( cat {SRC} > d.sh 2>/dev/null ) && {RUN:d.sh}",
       "{ cat {SRC} > d.sh 2>/dev/null; } && {RUN:d.sh}", "cat {SRC} > d.sh 2>/dev/null; cat d.sh > e.sh 2>/dev/null; {RUN:e.sh}", "cat {SRC} > d.sh 2>/dev/null | cat; {RUN:d.sh}",
       "cp {SRC} d.sh 2>/dev/null && {RUN:d.sh}", "cp {SRC} d.sh 2> e.sh && {RUN:e.sh}", "cp {SRC} d.sh 2>e.sh && {RUN:d.sh}", "install {SRC} d.sh 2>/dev/null && {RUN:d.sh}",
       "mv {SRC} d.sh 2>/dev/null && {RUN:d.sh}", "tee d.sh < {SRC} 2>/dev/null >/dev/null && {RUN:d.sh}", "cat {SRC} | tee d.sh 2>/dev/null >/dev/null && {RUN:d.sh}",
       "cat {SRC} | tee d.sh > /dev/null 2>&1 && {RUN:d.sh}", "sed 's/x/x/' {SRC} > d.sh 2>/dev/null && {RUN:d.sh}", "echo hi 2>d.sh; {RUN:d.sh}", "echo hi 2>d.sh; {P}",
       "{P} 2>/dev/null", "{P} 2>&1 | tail -n 1", "{P} 1>/dev/null 2>&1", "{P} &>/dev/null", "{P} >/dev/null 2>&1", "2>/dev/null {P}", "{P} 2>>x.log", "{P} 3>x.log", "exec 2>/dev/null; {P}",
       "exec 3>d.sh; cat {SRC} >&3; {RUN:d.sh}", "cat {SRC} >&3 3>d.sh; {RUN:d.sh}", "cat {SRC} 3>d.sh >&3; {RUN:d.sh}"]
dump("redirect", both("redirect", tp))

# ---- A4 ln / 链接 ----
lns = []
for opts in ["-s", "-sf", "-sn", "-sfn", "-sr", "-s -r", "-rs", "--symbolic", "--symbolic --relative", "-s --", "-sv", "-s -v", "-sT", "", "-f", "--force", "-sL", "-sP", "-sd", "-s -f --"]:
    for src, link, run in [("{SRC}", "l.sh", "l.sh"), ("{UP}", "sub/l.sh", "sub/l.sh"), ("{SRC}", "sub/l.sh", "sub/l.sh"), ("./{SRC}", "sub/l.sh", "sub/l.sh"), ("$PWD/{SRC}", "sub/l.sh", "sub/l.sh"),
                           ("{UP}", "sub/", "sub/s.sh"), ("{SRC}", "sub/", "sub/s.sh"), ("{UP}", "sub", "sub/s.sh"), ("{UP}", "./sub/l.sh", "sub/l.sh"), ("sub/../{SRC}", "sub/l.sh", "sub/l.sh"),
                           ("{UP}", "sub/x/../l.sh", "sub/l.sh"), ("{UP}", "../scratch/sub/l.sh", "sub/l.sh"), ("../{SRC}", "sub/l.sh", "sub/l.sh")]:
        lns.append(f"ln {opts} {src} {link} && {{RUN:{run}}}")
lns += ["ln -s -t sub {UP} && {RUN:sub/s.sh}", "ln -st sub {UP} && {RUN:sub/s.sh}", "ln -s -t sub {SRC} && {RUN:sub/s.sh}", "ln -s --target-directory=sub {UP} && {RUN:sub/s.sh}",
        "ln -s {UP} light.sh sub/ ; {RUN:sub/s.sh}", "ln -s {SRC} light.sh sub/ ; {RUN:sub/s.sh}", "cd sub && ln -s {UP} l.sh && {RUN:l.sh}", "cd sub && ln -s {SRC} l.sh && {RUN:l.sh}",
        "mkdir -p sub2 && ln -s ../{SRC} sub2/l.sh && {RUN:sub2/l.sh}", "mkdir -p sub2 && ln -s {SRC} sub2/l.sh && {RUN:sub2/l.sh}", "mkdir -p a/b && ln -s ../../{SRC} a/b/l.sh && {RUN:a/b/l.sh}",
        "mkdir -p a/b && ln -s ../{SRC} a/b/l.sh && {RUN:a/b/l.sh}", "mkdir -p a/b && ln -s ../../{SRC} a/b/l.sh ; ln -s b/l.sh a/m.sh ; {RUN:a/m.sh}",
        "ln -s {SRC} l1.sh && ln -s l1.sh l2.sh && {RUN:l2.sh}", "ln -s {SRC} l1.sh && ln -s ../l1.sh sub/l2.sh && {RUN:sub/l2.sh}", "ln -s l1.sh sub/l2.sh && ln -s {SRC} l1.sh && {RUN:sub/l2.sh}",
        "cp {SRC} c.sh && ln -s ../c.sh sub/l.sh && {RUN:sub/l.sh}", "cp {SRC} c.sh && ln -s c.sh sub/l.sh && {RUN:sub/l.sh}", "cp {SRC} sub/c.sh && ln -s c.sh sub/l.sh && {RUN:sub/l.sh}",
        "cp {SRC} sub/c.sh && ln -s ../sub/c.sh sub/l.sh && {RUN:sub/l.sh}", "ln {SRC} h.sh && {RUN:h.sh}", "ln {UP} sub/h.sh && {RUN:sub/h.sh}", "ln {SRC} sub/h.sh && {RUN:sub/h.sh}",
        "ln -s ../{SRC} sub/l.sh 2>/dev/null && {RUN:sub/l.sh}", "ln -s -- ../{SRC} sub/l.sh && {RUN:sub/l.sh}", "ln -sf -- ../{SRC} sub/l.sh && {RUN:sub/l.sh}"]
dump("ln", both("ln", lns))

# ---- A5 env 的 = ----
envs = []
for pre in ["env", "env -i PATH=\"$PATH\"", "env -u X", "env -- ", "env -C .", "env -C . --", "env --unset=X", "env -S ''", "env A=1", "env A=1 B=2", "env -u X A=1", "env -i A=1 PATH=\"$PATH\""]:
    for eq in ["=1", "=", "==", "=a=b", "\"=1\"", "'=1'", "\\=1", "=\"\"", "A=", "a-b=1", "1a=1", "=-i", "=--", "x=y=z", "=1 =2", "A=1 =2", "=1 A=2"]:
        envs.append(f"{pre} {eq} {{P}}")
envs += ["env -S '=1 {P}'", "env -S '=1' {P}", "env -S \"=1 {P}\"", "env --split-string='=1 {P}'", "env -S '=1 =2 {P}'", "env -S ' =1  {P}'", "env -S=1 {P}", "env -iS '=1 {P}'",
         "env =1 -u X {P}", "env =1 -i {P}", "env =1 -- {P}", "env -u =1 {P}", "env -u=1 {P}", "env -C =1 {P}", "env -C=1 {P}", "env --chdir==1 {P}", "env --unset==1 {P}",
         "command env =1 {P}", "sudo env =1 {P}", "nice env =1 {P}", "time env =1 {P}", "time -p env =1 {P}", "nohup env =1 {P}", "timeout 9 env =1 {P}", "exec env =1 {P}", "FOO=1 env =1 {P}",
         "env =1", "env =1 echo hi", "env =1; {P}", "env =1 echo hi; {P}", "env =1 true && {P}", "env =1 env =2 {P}", "env env =1 {P}", "env =1 bash -c '{P}'", "bash -c 'env =1 {P}'", "env =1 bash s.sh",
         "env =1 sh -c '{P}'", "eval 'env =1 {P}'", "env =1 /usr/bin/env =2 {P}", "xargs env =1 {P}", "env =1 nice {P}", "env = nice -n 1 {P}", "env =1 timeout 5 {P}"]
dump("env", both("env", envs))

# ---- A6 bash / sh 的选项：-c 之后的选项、-s 位置参数 ----
shs = []
for sh in ["bash", "sh", "env bash"]:
    for opts in ["-c", "-c --", "-c -", "-c -x", "-c -e", "-c -- -x", "-c -x --", "-c -o pipefail", "-c -O extglob", "-c +O extglob", "-c +o pipefail", "-o pipefail -c", "-O extglob -c", "-ec", "-xc", "-c -ex",
                 "-c -u -x", "-c --norc", "-c --noprofile", "-c --rcfile light.sh", "--rcfile light.sh -c", "--norc -c", "-lc", "-ic", "-c -i", "-c -l", "-c --posix", "--posix -c", "-c -s", "-s -c", "-cs", "-sc",
                 "-c +x", "-c -- --", "-c - -x", "-c -o", "-c -O", "-c --init-file light.sh", "-c --debugger", "-c --version", "-c --help", "-c -n", "-n -c", "-cn", "-c -v", "--login -c", "-c --login"]:
        shs.append(f"{sh} {opts} '{{P}}'")
        shs.append(f"{sh} {opts} '{{P}}' arg0 arg1")
        shs.append(f"{sh} {opts} '{{P}}' -x")
shs = [s for s in shs if not s.startswith(("sh -c -o", "sh -c -O", "sh -c +O"))] + ["sh -c -o pipefail '{P}'"]
shs += ["bash -s -- {SRC}", "bash -s {SRC}", "bash -s -- {SRC} {SRC}", "bash -s", "bash -s -x -- {SRC}", "bash -xs -- {SRC}", "bash -sx {SRC}", "bash -s -- -c", "bash -s -- -n", "bash -s -- -",
        "bash -s - {SRC}", "bash - -s {SRC}", "bash -- -s {SRC}", "bash -s -O extglob -- {SRC}", "bash -O extglob -s {SRC}", "bash -s --rcfile light.sh {SRC}", "bash --rcfile light.sh -s {SRC}",
        "bash -i -s -- {SRC}", "bash -s -i -- {SRC}", "bash -s -n -- {SRC}", "bash -n -s -- {SRC}", "bash -ns {SRC}", "bash -sn {SRC}", "sh -s -- {SRC}", "sh -s {SRC}", "bash -s +x {SRC}", "bash +s {SRC}", "bash +s -- {SRC}",
        "echo '{P}' | bash -s -- foo", "echo '{P}' | bash -s foo", "echo '{P}' | bash -s", "echo '{P}' | bash - foo", "echo '{P}' | bash -- foo", "echo '{P}' | bash foo", "echo '{P}' | sh -s -- foo",
        "bash -s -- foo <<'EOF'\n{P}\nEOF", "bash -s foo <<'EOF'\n{P}\nEOF", "bash -s <<'EOF'\n{P}\nEOF", "bash foo <<'EOF'\n{P}\nEOF", "bash -s -- {SRC} <<'EOF'\necho hi\nEOF",
        "bash -s -- foo <<< '{P}'", "bash -s foo <<< '{P}'", "bash -s <<< '{P}'", "bash foo <<< '{P}'", "bash -s -- foo < {SRC}", "bash -s foo < {SRC}", "bash -s < {SRC}", "bash < {SRC}", "bash foo < {SRC}",
        "bash -s -- \"$({P})\"", "bash -s -- '{P}'", "bash -s -- foo; {P}", "bash -s -- $(echo {SRC})", "cat {SRC} | bash -s -- foo", "cat {SRC} | bash -s", "cat {SRC} | bash foo", "bash <(cat {SRC})", "bash -s -- <(cat {SRC})",
        "source <(cat {SRC})", "bash -s -- foo < <(cat {SRC})", "bash -c -- '{P}' ; {P}", "bash -c -- \"$(echo '{P}')\"", "bash -c '{P}' -- foo", "bash -c -- '{P}' -- foo"]
dump("shell", both("shell", shs))

# ---- A7 builtin ----
bi = []
for b in ["cargo", "echo", "cd", "command", "exec", "eval", "source", ".", "type", "env", "time", "nice", "bash", "sh", "[", "[[", "test", "true", "false", ":", "trap", "builtin", "enable", "hash", "read", "mapfile", "wait", "kill", "alias", "set",
          "shopt", "declare", "local", "export", "help", "printf", "pwd", "unset", "caller", "compgen", "complete", "bg", "fg", "jobs", "fc", "getopts", "history", "dirs", "pushd", "popd", "times", "ulimit", "umask", "disown", "suspend", "logout", "let", "return", "shift", "break", "continue", "exit", "typeset", "readonly", "readarray", "unalias", "bind", "compopt", "xargs", "function", "if", "while", "!", "{", "coproc", "select", "until", "case", "for", "in", "do", "then", "esac", "fi", "done"]:
    bi.append(f"builtin {b} {{P}}")
    bi.append(f"builtin -- {b} {{P}}")
bi += ["builtin command {P}", "builtin command -p {P}", "builtin exec {P}", "builtin eval '{P}'", "builtin eval \"{P}\"", "builtin source {SRC}", "builtin . {SRC}", "builtin builtin command {P}", "builtin builtin cargo test --all",
       "builtin cd . && {P}", "builtin cd . ; builtin {P}", "builtin {P}; {P}", "builtin trap '{P}' EXIT", "builtin echo '{P}'", "builtin echo \"$({P})\"", "builtin cargo test \"$(cargo test --all)\"", "builtin cargo `cargo test --all`",
       "builtin cargo test --all <(cargo test --all)", "builtin cargo test --all > >(cargo test --all)", "builtin $(echo cargo) test --all", "builtin ${X:-cargo} test --all", "X=cargo; builtin $X test --all", "builtin car* test --all",
       "builtin ca?go test --all", "builtin 'cargo' test --all", "builtin \"cargo\" test --all", "builtin car\\go test --all", "builtin {cargo,echo} test --all", "builtin {echo,cargo} test --all", "builtin {,echo} cargo test --all",
       "builtin {cd,cargo} test --all", "builtin -p cargo test --all", "builtin -x cargo test --all", "builtin -- -- cargo test --all", "builtin --  cargo test --all", "builtin", "builtin; {P}", "builtin ''; {P}", "builtin '' {P}",
       "FOO=1 builtin cargo test --all", "time builtin cargo test --all", "command builtin cargo test --all", "env builtin cargo test --all", "nice builtin cargo test --all", "bash -c 'builtin cargo test --all'", "sh -c 'builtin cargo test --all'",
       "eval 'builtin cargo test --all'", "builtin cargo test --all && {P}", "builtin cargo test --all || {P}", "builtin cargo test --all | {P}", "builtin cargo test --all & {P}", "( builtin cargo test --all )", "{ builtin cargo test --all; }",
       "if builtin cargo test --all; then :; fi", "while builtin cargo test --all; do :; done", "! builtin cargo test --all", "builtin enable -n cargo", "enable -f x.so cargo; builtin cargo test --all"]
dump("builtin", both("builtin", bi))

# ---- A8 function ----
fn = []
bodies = ["{P}", "echo a; {P}", "{ {P}; }", "env =1 {P}"]
for b in bodies:
    b2 = b if b.endswith(";") else b + ";"
    for call in ["", "f", "f;", "$(f)", "\"$(f)\"", "`f`", "echo f", "echo 'f'", "export -f f; bash -c f", "export -f f; bash -c 'f'", "f &", "(f)", "{ f; }", "command f", "eval f", "eval 'f'",
                 "builtin f", "xargs -n1 f", "bash -c f", "trap f EXIT", "type f", "declare -f f", "declare -F f", "alias g=f; g", "g=f; $g", "g=f; ${g}", "\"f\"", "'f'", "\\f", "fo''o", "'fo'o", "f''", "\"\"f", "$'f'", "f{,}", "{f,}", "{f,g}", "{,f}", "[f]", "f*", "?", "*", "fu[n]", "echo $(echo f)", "time f", "nice f", "env f",
                 "set -a; f", "source <(echo f)", "FOO=1 f", "! f", "f || true", "echo hi | f", "f | cat", "for x in 1; do f; done", "while f; do break; done", "case x in x) f;; esac", "[[ -z $(f) ]]", "enable -n f", "unset -f f", "f() { :; }; f"]:
        fn.append(("function f { " + b2 + " }" + ("; " + call if call else "")))
        fn.append(("function f() { " + b2 + " }" + ("; " + call if call else "")))
        fn.append(("function f\n{ " + b2 + " }" + ("\n" + call if call else "")))
        fn.append(("f() { " + b2 + " }" + ("; " + call if call else "")))
fn_named = []
for name in ["foo", "g", "cargo", "my-run", "run.it", "f1", "_f", "F", "bash", "cd", "ls", "echo", "command_not_found_handle", "exit", "if", "time", "}", "{", "f:", "f=1", "f=", "a/b", "fo\\o", "f$x", "f*", "-f", "--"]:
    fn_named.append("function " + name + " { {P}; }")
    fn_named.append("function " + name + " { {P}; }; " + name)
    fn_named.append("function " + name + " { {P}; }; echo hi")
    fn_named.append("function " + name + " { {P}; }; ls")
    fn_named.append("function " + name + " { {P}; }; cd .")
fn += fn_named
fn += ["function f { {P}; } >log.txt", "function f { {P}; } 2>&1", "function f { {P}; } &", "function f { {P}; }; function g { f; }", "function f { {P}; }; function g { f; }; g", "function f { {P}; }; function g { :; }; g",
       "function f { {P}; } function g { :; }", "function f { {P}; } ; cargo test --all", "function f { echo a; } ; {P}", "function f { echo } ; {P}; }", "function f { echo } ; {P} ; }", "function f { {P} }", "function f { {P}\n}", "function f {\n{P}\n}",
       "function f { {P}; }\nf", "function f { {P}; }\n\nf", "function f { {P}; }  #f", "function f { {P}; } # f\n", "function f { {P}; }; # f", "function f { {P}; }; echo '# f'", "function f { {P}; }; echo \"f\"", "function f ( {P} )", "function f ( {P} ); f",
       "function f { {P}; }; function f { :; }", "function f { :; }; function f { {P}; }", "function f { {P}; }; unset -f f", "function f { { {P}; }; }", "function f { {P}; }; }", "function f { {P}; } && echo ok", "function f { {P}; } || echo ok", "function f { {P}; } | cat",
       "if true; then function f { {P}; }; fi", "{ function f { {P}; }; }", "( function f { {P}; } )", "echo | function f { {P}; }", "time function f { {P}; }", "! function f { {P}; }", "FOO=1 function f { {P}; }", "env function f { {P}; }", "nice function f { {P}; }",
       "bash -c 'function f { {P}; }'", "eval 'function f { {P}; }'", "bash <<'EOF'\nfunction f { {P}; }\nEOF", "cat <<'EOF'\nfunction f { {P}; }\nEOF", "echo $(function f { {P}; })", "x=$(function f { {P}; }); echo", "function f { {P}; }; x=$(echo)", "function f { {P}; }; echo $HOME",
       "function f { {P}; }; echo `date`", "function f { {P}; }; echo \"$(date)\"", "function f { {P}; }; echo *", "function f { {P}; }; echo ?", "function f { {P}; }; echo [a]", "function f { {P}; }; echo {a,b}", "function f { {P}; }; X=1", "function f { {P}; }; export X=1", "function f { {P}; }; set -x", "function f { {P}; }; set -a",
       "function f { {P}; }; eval :", "function f { {P}; }; source /dev/null", "function f { {P}; }; . /dev/null", "function f { {P}; }; trap : EXIT", "function f { {P}; }; compgen -A function", "function f { {P}; }; declare -f", "function f { {P}; }; typeset -F", "function f { {P}; }; alias", "function f { {P}; }; enable -n true",
       "function f { {P}; }; shopt -s expand_aliases", "function f { {P}; }; cat < /dev/null", "function f { {P}; }; nonexistent-command-xyz", "function f { {P}; }; command_not_found_handle() { f; }; nonexistent-command-xyz", "function command_not_found_handle { {P}; }; nonexistent-command-xyz", "function command_not_found_handle { {P}; }",
       "function f { {P}; }; PROMPT_COMMAND=f", "function f { {P}; }; export -f f", "function f { {P}; }; export -f f; bash -c f", "function f { {P}; }; export -f f; bash -c 'f'", "function f { {P}; }; export -f f; sh -c f", "function f { {P}; }; declare -fx f; bash -c f", "function f { {P}; }; typeset -fx f",
       "function f { {P}; }; set -a; bash -c :", "function f { {P}; }; set -o allexport", "function f { {P}; }; set -a; function g { :; }", "function f { {P}; }; ENV=x"]
dump("function", both("function", fn))

# ---- A9 函数调用的各种拼法（名字拆开写） ----
calls = ["foo", "fo{o,}", "{fo,}o", "f{o,}o", "{foo,}", "{,foo}", "fo{o,o}", '"fo"o', "fo''o", 'f""oo', "fo\\o", "f'o'o", "'f'oo", '"foo"', "'foo'", "\\foo", "$'foo'", "$'f\\x6fo'", "fo?", "fo*", "[f]oo", "f[o]o", "foo{,}", "foo{,.x}",
         "${X:-foo}", "$(echo foo)", "`echo foo`", "f\\\no\\\no", "eval foo", "eval fo\\o", "command foo", "builtin foo", "exec foo", "nice foo", "env foo", "xargs foo", "time foo", "! foo", "(foo)", "{ foo; }", "foo &", "foo | cat", "echo | foo",
         "foo > /dev/null", "foo 2>&1", "FOO=1 foo", "foo a b", "foo;", 'FOO="" foo']
cs = []
for k in (G, D):
    for c in calls:
        for form in ["function foo { {P}; }; {C}", "function foo() { {P}; }; {C}", "foo() { {P}; }; {C}", "function foo { {P}; }\n{C}", "function foo { {P}; } && {C}", "function foo { {P}; } ; echo hi; {C}"]:
            cs.append(dict(tag="fn-call-forms", cmd=form.replace("{P}", k["P"]).replace("{C}", c), hook=k["hook"], agent=k["agent"]))
dump("fncall", cs)

# ---- A10 引号写出的空串放在各个位置 ----
E = ["''", '""', "''''", '""' + "''"]
toks = ["cargo", "test", "--all"]
cs = []
for e in E:
    for pos in range(4):
        w = toks[:]; w.insert(pos, e); cs.append(" ".join(w))
    cs += [f"{e}cargo test --all", f"car{e}go test --all", f"cargo te{e}st --all", f"cargo test --{e}all", f"cargo test --all{e}", f"cargo test {e}--all", f"cargo{e} test --all"]
    for pre in ["env", "env A=1", "env -u X", "env --", "nice", "nice -n 5", "time", "time -p", "command", "exec", "timeout 5", "nohup", "builtin command", "env =1", "env -C ."]:
        cs.append(f"{pre} {e} cargo test --all"); cs.append(f"{pre} cargo {e} test --all")
    cs += [f"bash -c {e}", f"bash -c {e} 'cargo test --all'", f"bash {e} -c 'cargo test --all'", "bash -c '' 'cargo test --all'", f"eval {e} 'cargo test --all'", f"X={e} cargo test --all", f"X={e} {e} cargo test --all", f"{e} X=1 cargo test --all",
           f"{e}; cargo test --all", f"cargo test --all {e}", f"cargo test --all > {e}", f"cargo test --all {e}{{,}}", f"{e}{{,}} cargo test --all", f"{{{e},}} cargo test --all", f"{{{e},x}} cargo test --all", f"cargo test {{{e},--all}}", f"cargo test {{{e},}} --all", f"cargo {{{e},test}} --all",
           f"cargo {{test,{e}}} --all", f"cargo test --all{{{e},}}", f"cargo test --all{{,{e}}}", f"{e}{{{e},}}", f"for i in {e}; do cargo test --all; done", f"case {e} in {e}) cargo test --all;; esac", f"[[ -z {e} ]] && cargo test --all", f"test -z {e} && cargo test --all"]
dump("empties", [dict(tag="empties", cmd=c, hook="guard", agent="main") for c in cs])

# ---- A11 heredoc 写出文件时的 fd 号 ----
H = "\ncargo test --all\nEOS\n"
cs = []
for t in ["cat <<'EOS' > d.sh 2>e.sh" + H + "bash e.sh", "cat <<'EOS' > d.sh 2>e.sh" + H + "bash d.sh", "cat <<'EOS' 2>e.sh > d.sh" + H + "bash e.sh", "cat <<'EOS' 2>e.sh > d.sh" + H + "bash d.sh", "cat <<'EOS' 2>d.sh" + H + "bash d.sh", "cat <<'EOS' 1>d.sh" + H + "bash d.sh",
          "cat <<'EOS' 3>d.sh" + H + "bash d.sh", "cat <<'EOS' >d.sh" + H + "bash d.sh", "cat <<'EOS' >>d.sh" + H + "bash d.sh", "cat <<'EOS' &>d.sh" + H + "bash d.sh", "cat <<'EOS' >&d.sh" + H + "bash d.sh", "cat 2>d.sh <<'EOS'" + H + "bash d.sh",
          "cat 0<<'EOS' >d.sh" + H + "bash d.sh", "cat >d.sh <<'EOS'" + H + "bash d.sh", "tee d.sh <<'EOS' >/dev/null" + H + "bash d.sh", "tee d.sh 2>e.sh <<'EOS' >/dev/null" + H + "bash e.sh", "sh -c 'cat >d.sh' <<'EOS'" + H + "bash d.sh",
          "cat <<'EOS' | tee d.sh >/dev/null" + H + "bash d.sh", "cat <<'EOS' | tee e.sh >/dev/null 2>d.sh" + H + "bash d.sh", "cat <<'EOS' 4>d.sh >e.sh" + H + "bash d.sh", "cat <<'EOS' 4>d.sh >e.sh" + H + "bash e.sh", "cat <<'EOS' >&2 2>d.sh" + H + "bash d.sh"]:
    cs.append(dict(tag="heredoc-write", cmd=t, hook="guard", agent="main"))
dump("hwrite", cs)

# ---- A12 ANSI-C 引号与单引号尾部反斜杠，后面接没收尾的替换里的 heredoc ----
cs = []
for h in ["echo $'a\\'b'; ", "echo $'a\\'b' ", "echo $'\\''; ", "echo 'a\\'; ", "echo 'a\\\\'; ", "x=$'a\\'b' ", "echo \"$'a\\'b'\"; "]:
    for o, c in [("x=$(", "\n)"), ('x="$(', '\n)"'), ("diff <(", "\n) /dev/null"), ("x=`", "\n`"), ("", "")]:
        cs.append(dict(tag="ansi-c", cmd=h + o + "bash <<'EOF'\ncargo test --all\nEOF" + c, hook="guard", agent="main"))
dump("ansic", cs)

# ---- A14 cp / mv / install / ln 的 -t、-T、多源、-- 各种写法（copy_pairs 改写后各条路径都走一遍） ----
cs = []
tpl = ["cp -t sub {SRC} && bash sub/{B}{RT}", "cp --target-directory=sub {SRC} && bash sub/{B}{RT}", "cp -T {SRC} sub/x.sh && bash sub/x.sh{RT}", "cp {SRC} sub/ && bash sub/{B}{RT}", "cp {SRC} sub && bash sub/{B}{RT}", "cp -t sub {SRC} && bash sub/x.sh{RT}",
       "mv -t sub {SRC} && bash sub/{B}{RT}", "install -t sub {SRC} && bash sub/{B}{RT}", "install {SRC} sub/x.sh && bash sub/x.sh{RT}", "install -m 755 {SRC} sub/ && bash sub/{B}{RT}", "install -D {SRC} sub/y/x.sh && bash sub/y/x.sh{RT}",
       "ln -t sub {SRC} && bash sub/{B}{RT}", "ln -T {SRC} sub/x.sh && bash sub/x.sh{RT}", "ln {SRC} sub/ && bash sub/{B}{RT}", "cp -a {SRC} sub/x.sh && bash sub/x.sh{RT}", "cp -r {SRC} sub/x.sh && bash sub/x.sh{RT}", "cp -p {SRC} sub/x.sh && bash sub/x.sh{RT}",
       "cp {SRC} {SRC} sub/ && bash sub/{B}{RT}", "cp {SRC} light.sh sub/ && bash sub/{B}{RT}", "cp light.sh {SRC} sub/ && bash sub/{B}{RT}", "cp -t sub light.sh {SRC} && bash sub/{B}{RT}", "cp -- {SRC} sub/x.sh && bash sub/x.sh{RT}", "cp -f -- {SRC} sub/x.sh && bash sub/x.sh{RT}",
       "cp -t sub -- {SRC} && bash sub/{B}{RT}", "cp -t -- sub {SRC} && bash sub/{B}{RT}"]
for k, base in ((G, "s.sh"), (D, "detached-run.sh")):
    for t in tpl:
        cs.append(dict(tag="cp-forms", cmd=t.replace("{SRC}", k["SRC"]).replace("{B}", base).replace("{RT}", k["RUNTAIL"]), hook=k["hook"], agent=k["agent"]))
dump("cpforms", cs)

# ---- A13 第一轮五格的关键反例（keyfeeds.sh 里那些），交真 bash 裁判 ----
key = []
for c in ["{cargo,} test --all", "{,} cargo test --all", "cargo {,test} --all", "FOO=\"$(pwd)\" bash <<'EOF'\ncargo test --all\nEOF", "cd \"$(pwd)\" && bash <<'EOF'\ncargo test --all\nEOF", "x=$(bash <<'EOF'\ncargo test --all\nEOF\n)",
          "cat s.sh > d1.sh 2>/dev/null && bash d1.sh", "cat s.sh > d2.sh && bash d2.sh", "env =1 cargo test --all", "bash -c -- 'cargo test --all'", "ln -s ../s.sh sub/l.sh && bash sub/l.sh", "ln -s s.sh sub/l2.sh && bash sub/l2.sh",
          "bash -s -- s.sh", "builtin cargo test --all", "function f { cargo test --all; }", "function f { cargo test --all; }; f", "time -p cargo test --all", "bash --rcfile light.sh s.sh", "bash -c 'cargo test --all' arg0"]:
    key.append(dict(tag="key", cmd=c, hook="guard", agent="main"))
for c in ["cat scripts/hosts/detached-run.sh > /tmp/shw-dr-b.sh 2>/dev/null && bash /tmp/shw-dr-b.sh start --label m -- sleep 1", "cat scripts/hosts/detached-run.sh > /tmp/shw-dr-a.sh && bash /tmp/shw-dr-a.sh start --label m -- sleep 1"]:
    key.append(dict(tag="key", cmd=c, hook="det", agent="clerk"))
dump("key", key)

# ---- A0 第一轮 575 条（命令照搬第一轮腿的家族文件）：裁判用真 bash 的限定在重型闸主 agent 与检出器 clerk 起 detached-run 的；别的沿用第一轮的 expect ----
if len(sys.argv) > 2:
    import glob
    rows = []
    for f in sorted(glob.glob(os.path.join(sys.argv[2], "*.json"))):
        for x in json.load(open(f)):
            y = dict(x); y["r1expect"] = x["expect"]
            elig = (x["hook"] == "guard" and x["agent"] == "main") or (x["hook"] == "det" and x["agent"] == "clerk" and "detached-run" in x["cmd"])
            if "kill" in x["cmd"] or "until" in x["cmd"] or "while" in x["cmd"] or x["tag"].startswith("W1-deep"): elig = False   # W1-deep 是 2^100 份花括号，真 bash 会吃光内存
            if not elig: y["noexec"] = True
            rows.append(y)
    json.dump(rows, open(os.path.join(out, "cases-r1-575.json"), "w"), ensure_ascii=False)
    print("r1-575", len(rows))
