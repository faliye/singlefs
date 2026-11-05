"""mkx1_b.py <输出目录> : W2（heredoc 进没收尾的替换）、W4（env 赋值）、W5（time / builtin / function / bash 选项 / 双斜杠）。"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from x1common import *
out = sys.argv[1]
# ---- W2 ----
body = "cargo test --all"
opens = [("x=$(", "\n)"), ("x=\"$(", "\n)\""), (": \"$(", "\n)\""), ("echo ${z:-$(", "\n)}"), ("y=`", "\n`"), ("echo \"a$(echo 'b)'; ", "\n)\""), ("echo \"$(echo \\\")\"\"$(", "\n)\""),
         ("[[ -n \"$(", "\n)\" ]]"), ("a=(\"$(", "\n)\")"), ("echo \"x\"\"$(", "\n)y\""), ("echo '#' \"$(", "\n)\""), ("x=$(echo \"q\" ; echo '\"' ; ", "\n)"),
         ("echo \"$(case a in a) echo \\\"1\\\" ;; esac; ", "\n)\""), ("echo $(( 1 + 2 )) \"$(", "\n)\""), ("echo \"$( # c)\n", "\n)\""), ("cat <(", "\n)"), ("diff <(echo a) <(", "\n)")]
feeds = ["bash <<'EOF'\n{B}\nEOF", "sh <<\"EOF\"\n{B}\nEOF", "bash -s <<-EOF\n\t{B}\n\tEOF", "source /dev/stdin <<'EOF'\n{B}\nEOF", ". /dev/stdin <<'EOF'\n{B}\nEOF", "bash /dev/stdin <<'EOF'\n{B}\nEOF",
         "bash -c 'cat | bash' <<'EOF'\n{B}\nEOF", "cat <<'EOF'\n{B}\nEOF", "bash <<'E O F'\n{B}\nE O F", "bash <<'END'\n{B}\nEND"]
cs = []
for a, b in opens:
    for f in feeds:
        cs.append(mk("W2", "heredoc-open", a + f.replace("{B}", body) + b))
        cs.append(mk("W2", "heredoc-top", a.replace("$(", "$(true)").replace("`", "`true`") + "x; " + f.replace("{B}", body)))
for f in feeds:
    for pre in ["cd \"$(git rev-parse --show-toplevel)\" && ", "cd \"$(pwd)\" && ", "cd \"$(dirname \"$(pwd)\")\" && ", "x=\"$(echo \"a\")\"; ", "if [ \"$(echo a)\" = a ]; then ", "FOO=\"$(echo a)\" "]:
        suf = "\nfi" if pre.startswith("if") else ""
        cs.append(mk("W2", "heredoc-after-quoted-subst", pre + f.replace("{B}", body) + suf))
dump(out + "/cases-x1-W2.json", cs)
# ---- W4 env ----
envs = ["env =1", "env A=1", "env A=1 =2", "env -u X", "env -u X A=1", "env -uX", "env --unset=X", "env --unset X", "env -i", "env -i A=1", "env -C .", "env -C. ", "env --chdir=.", "env -0", "env -v", "env --",
        "env -- A=1", "env A=1 --", "env =1 --", "env =1 -u X", "env A=1 -u X", "env A=1 -i", "env -S 'A=1'", "env -S 'A=1 -u X'", "env -- =1", "env -i -- A=1", "env A='x y'", "env 'A=x y'", "env \"A=1\"",
        "env a-b=1", "env 1a=2", "env A=1 B=2 C=3", "env -u X -u Y", "env -u X -- A=1", "env A=$(echo 1)", "env -u '' A=1", "env ==", "env A", "env -u", "env -x", "env --foo", "env A= ", "env -P /usr/bin", "env --ignore-environment"]
cs = []
for e in envs:
    for tpl in ["{E} {P}", "{E} {P}; echo done", "true && {E} {P}", "bash -c '{E} {P}'", "nice {E} {P}", "{E} {E} {P}", "time {E} {P}", "( {E} {P} )"]:
        t = tpl.replace("{E}", e)
        cs += both("W4", "env", t)
dump(out + "/cases-x1-W4.json", cs)
