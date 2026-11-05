"""mkx1_c.py <输出目录> : W5（time / builtin / function / bash 选项 / 双斜杠）与第二轮第 2–5 条（函数名拼写、数字紧贴 >、bash -c 之后的选项、env 赋值之后的选项）。"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from x1common import *
out = sys.argv[1]
cs = []
# ---- W5 ----
pre = ["time", "time -p", "time -p --", "time --", "builtin", "builtin --", "builtin command", "builtin builtin", "builtin exec", "builtin type", "builtin echo", "command", "command --", "command -p",
       "exec", "exec -c", "exec -a x", "nohup", "setsid", "nice -n 3", "function f { :; };", "function f { {P}; }; f #", "time time", "! time -p", "builtin :; time", "coproc", "FOO=1 time -p"]
for p in pre:
    for tpl in ["{PRE} {P}", "( {PRE} {P} )", "x=1; {PRE} {P}", "bash -c '{PRE} {P}'", "true && {PRE} {P}\n"]:
        cs += both("W5", "prefix", tpl.replace("{PRE}", p))
bopts = ["-c", "-c --", "-xc", "-c -x", "-ec", "-c -e", "-c -O extglob", "-O extglob -c", "-o pipefail -c", "-c -o pipefail", "-c +x", "-c -l", "-l -c", "--login -c", "--norc -c", "--noprofile -c", "--rcfile s.sh -c",
         "--init-file light.sh -c", "-c --posix", "--posix -c", "-c -n", "-nc", "-c -v", "-vc", "-c -u", "-uc", "-i -c", "-c -i", "-ic", "--restricted -c", "-r -c", "-c -r", "-c -p", "-p -c", "-c -s", "-sc", "--debugger -c"]
for sh in ["bash", "sh", "/bin/bash", "/usr/bin/env bash"]:
    for o in bopts:
        cs += both("W5", "bash-opts", f"{sh} {o} '{{P}}'")
        cs += both("W5", "bash-opts-argv", f"{sh} {o} '{{P}}' arg0 arg1")
for tpl in ["bash -s -- s.sh <<'EOF'\n{P}\nEOF", "bash - <<'EOF'\n{P}\nEOF", "bash -s <<'EOF'\n{P}\nEOF", "bash --rcfile s.sh <<'EOF'\n{P}\nEOF", "bash -s -- <<'EOF'\n{P}\nEOF", "bash -s x y <<'EOF'\n{P}\nEOF",
            "bash --rcfile light.sh s.sh", "bash --rcfile light.sh -c 'echo hi'", "bash --init-file s.sh -c 'echo hi'", "bash --rcfile s.sh", "bash --rcfile s.sh light.sh", "bash --rcfile s.sh -n light.sh", "bash -- s.sh", "bash - s.sh", "bash -x -- s.sh",
            "bash -O extglob s.sh", "bash -o posix s.sh", "bash --norc s.sh", "bash -l s.sh", "bash -c . s.sh", "bash --version s.sh", "bash --help s.sh", "bash -n s.sh", "bash -nx s.sh", "bash -e -n s.sh", "bash -vn s.sh"]:
    cs += both("W5", "bash-script", tpl)
for sl in ["//", "///", "/./", "/.//", "/../", "//./"]:
    cs.append(mk("W5", "slashes", f"cd /; cp {os.getcwd()}/scratch/s.sh /tmp/dd.sh; bash {sl}tmp/dd.sh"))
    cs.append(mk("W5", "slashes", f"bash {sl}{os.getcwd().lstrip('/')}/scratch/s.sh"))
    cs.append(mk("W5", "slashes-det", f"bash {sl}{os.getcwd().lstrip('/')}/scratch/scripts/hosts/detached-run.sh start --label m -- sleep 1", "det"))
    cs.append(mk("W5", "slashes-det", f"bash scripts{sl.replace('..','.')}hosts/detached-run.sh start --label m -- sleep 1", "det"))
dump(out + "/cases-x1-W5.json", cs)
# ---- 第二轮第 2 条：函数名拼写 ----
cs = []
defs = ["function foo { {P}; }", "foo() { {P}; }", "function foo() { {P}; }", "function foo { {P}; } >/dev/null", "foo () { {P}; }", "function foo\n{\n {P}\n}", "foo() ( {P} )"]
calls = ["foo", "\"fo\"o", "'fo'o", "f\"\"oo", "f''oo", "\\foo", "fo\\o", "fo{o,}", "{fo,}o", "{foo,}", "fo{,}o", "$'foo'", "$'f\\x6fo'", "\"$(echo foo)\"", "$(echo foo)", "`echo foo`", "F=foo; $F", "F=foo; \"$F\"", "F=fo; ${F}o",
         "F=foo; ${F:-x}", "eval foo", "eval 'fo''o'", "eval \"f\"\"oo\"", "command foo", "builtin foo", "exec foo", "env foo", "time foo", "time -p foo", "nice foo", "! foo", "( foo )", "{ foo; }", "foo &\nwait", "foo | cat", "echo | foo",
         "bash -c foo", "bash -c 'foo'", "(export -f foo; bash -c foo)", "export -f foo; bash -c foo", "foo=1 foo", "\"\"foo", "foo\"\"", "$empty foo", "fo?", "f*", "[f]oo", "fo[o]", "foo;", "if foo; then :; fi", "while foo; do break; done", "for i in 1; do foo; done", "trap foo EXIT", "coproc foo", "xargs -I{} foo", "foo foo", "echo foo", ": foo"]
for d in defs[:3] + defs[3:]:
    for c in calls:
        cs.append(mk("V2-2", "fncall", d.replace("{P}", GP) + ("\n" if c.startswith("foo &\n") else "; ") + c))
for d in defs[:3]:   # 只定义不调用
    cs.append(mk("V2-2", "fnonly", d.replace("{P}", GP)))
    cs.append(mk("V2-2", "fnonly", d.replace("{P}", GP) + "; echo foo"))
    cs.append(mk("V2-2", "fnonly", d.replace("{P}", GP) + "; echo \"fo\"o"))
    cs.append(mk("V2-2", "fnonly", d.replace("{P}", GP) + "; echo fo{o,}"))
    cs.append(mk("V2-2", "fnonly", d.replace("{P}", GP) + "; printf '%s\\n' 'foo' > t.txt"))
    cs.append(mk("V2-2", "fnonly", d.replace("{P}", GP) + "; printf '%s\\n' 'a{,}' > t.txt"))
    cs.append(mk("V2-2", "fnonly", d.replace("{P}", GP) + "; cat <<'X'\nfoo\nX"))
dump(out + "/cases-x1-V2-2.json", cs)
