"""mkx1_d.py <输出目录> : 第二轮第 3 条（数字紧贴 >）、第 6 条（ANSI-C、heredoc 写文件）、第 7 条（case 模式词与 [[ ]]）、W3（detached-run 与拷出再起）。"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from x1common import *
out = sys.argv[1]
cs = []
# ---- 第 3 条：fd 号 ----
ops = [">", "1>", "01>", "001>", "2>", "02>", "3>", "10>", "1>|", ">|", "&>", "&>>", ">>", "1>>", "1 >", "2 >", "\"1\">", "'1'>", "\\1>", "a1>", "x=1>", "1\">\"", "{v}>", "1>&-", ">&2", "2>&1 >", ">&2 2>", "2>&1 1>", "2> e.txt >",
       "2>/dev/null >", ">/dev/null 2>", "3> e.txt >&3", ">&3 3>", "1>&3 3>", "0>", "0<>", "1<>", "<> "]
for op in ops:
    for sep in ["&&", ";", "||", "\n"]:
        cs += both("V2-3", "fd", f"cat {{SRC}} {op} d.sh {sep} {{RUN}}".replace("{SRC}", "SRC") .replace("SRC", "S").replace("{RUN}", "R"))
cs = [dict(c, cmd=c["cmd"].replace("S", "s.sh", 1)) for c in cs]   # 占位见下面重造
cs = []
for op in ops:
    for sep in ["&&", ";", "||", "\n"]:
        cs.append(mk("V2-3", "fd", f"cat s.sh {op} d.sh {sep} bash d.sh"))
        cs.append(mk("V2-3", "fd-det", f"cat scripts/hosts/detached-run.sh {op} d.sh {sep} bash d.sh start --label m -- sleep 1", "det"))
for tpl in ["exec 3> d.sh; cat s.sh >&3; bash d.sh", "exec 3> d.sh; cat s.sh 1>&3; bash d.sh", "exec 3> d.sh; cat s.sh 2>&3; bash d.sh", "{ cat s.sh; } > d.sh; bash d.sh", "( cat s.sh ) > d.sh; bash d.sh",
            "cat s.sh | tee d.sh >/dev/null; bash d.sh", "cat < s.sh > d.sh; bash d.sh", "cat - < s.sh > d.sh; bash d.sh", "cat s.sh s.sh > d.sh; bash d.sh", "cat s.sh light.sh > d.sh; bash d.sh", "cat light.sh s.sh > d.sh; bash d.sh",
            "cat s.sh > d.sh 2>/dev/null; bash d.sh", "cat s.sh 2>/dev/null > d.sh; bash d.sh", "cat s.sh >d.sh;bash d.sh", "cat s.sh>d.sh;bash d.sh", "cat s.sh >> d.sh; bash d.sh", "cp s.sh d.sh 2>/dev/null; bash d.sh",
            "cat s.sh > \"d.sh\"; bash d.sh", "cat s.sh > 'd.sh'; bash d.sh", "cat s.sh > d\".sh\"; bash d.sh", "cat s.sh > ./d.sh; bash ./d.sh", "cat s.sh > sub/d.sh; bash sub/d.sh", "cd sub; cat ../s.sh > d.sh; bash d.sh",
            "cat s.sh > d.sh; . d.sh", "cat s.sh > d.sh; source d.sh", "cat s.sh > d.sh; sh d.sh", "cat s.sh > d.sh; chmod +x d.sh; ./d.sh", "cat s.sh > d.sh; exec bash d.sh", "cat s.sh > d.sh; (bash d.sh)", "cat s.sh > d.sh; bash -c 'bash d.sh'"]:
    cs.append(mk("V2-3", "fd-forms", tpl))
dump(out + "/cases-x1-V2-3.json", cs)
# ---- 第 6 条：ANSI-C 引号、heredoc 写文件、补零序列 ----
cs = []
ans = ["echo $'a\\'b'", "echo $'a\\\\'", "echo $'\\''", "echo $'a\\x27b'", "echo $'\\\\\\''", "echo $\"a\"", "echo $'a'$'b'", "echo '$'\"'\"", "echo a$'\\''", "echo $'a\\'b' $'c\\'d'"]
for a in ans:
    for t in ["{A}; bash <<'EOF'\n" + GP + "\nEOF", "{A}; x=$(bash <<'EOF'\n" + GP + "\nEOF\n)", "{A} && " + GP, "{A}; cat <<'EOF' > d.sh\n" + GP + "\nEOF\nbash d.sh", "{A}; echo hi > d.sh; bash d.sh", "{A} | " + GP]:
        cs.append(mk("V2-6", "ansic", t.replace("{A}", a)))
for fdop in ["", "2", "3", "1", "01", "2>&1 ", ">&2 2", "3>d.sh >&3 #", "2>e.txt", "&", ">&2 2>d.sh #"]:
    for cmdw in ["cat", "tee d.sh", "bash -c 'cat'"]:
        cs.append(mk("V2-6", "hwrite", f"cat <<'EOS' {fdop}>d.sh\n{GP}\nEOS\nbash d.sh"))
        cs.append(mk("V2-6", "hwrite", f"tee d.sh {fdop}>e.sh <<'EOS' >/dev/null\n{GP}\nEOS\nbash d.sh"))
        cs.append(mk("V2-6", "hwrite", f"tee d.sh {fdop}>e.sh <<'EOS' >/dev/null\n{GP}\nEOS\nbash e.sh"))
for seq in ["{01..100..100}", "{1..100..50}", "{01..100..50}", "{001..10..3}", "{09..100..30}", "{1..005..2}", "{000..2}", "{-01..01}", "{01..-1}", "{010..1..3}", "{1..3}{1..2}", "{a..e..2}", "{01..2..1}", "{0..10..5}"]:
    for n in ["s001", "s01", "s1", "s51", "s051", "s100", "s000", "s0", "s5", "s003", "s039", "s-01", "s02", "s2", "s10", "s010", "sa", "sc", "s11", "s12", "s21"]:
        cs.append(mk("V2-6", "seq-pad", f"cp s.sh s{seq}.sh && bash {n}.sh"))
dump(out + "/cases-x1-V2-6.json", cs)
# ---- 第 7 条：case 模式词与 [[ ]] ----
cs = []
pats = ["herd7", "killall", "cargo", "bash", "reboot", "qemu-system-x86_64", "rm", "kill", "pkill", "cargo test --all", "*", "a|herd7", "herd7|b", "(herd7", "herd7 | killall", "scripts/gate/gate.sh"]
for p in pats:
    for tpl in ["case \"$X\" in a) echo a;; {P}) echo h;; esac", "case x in\n a) :;;\n {P}) :;;\nesac", "case x in a) :;; {P}) :;; esac; echo done", "case x in {P}) :;; esac", "case x in a) :;& {P}) :;; esac", "case x in a) :;;& {P}) :;; esac",
                "x=1; case $x in a) case b in {P}) :;; esac;; esac", "case cargo in {P}) cargo test --all;; esac", "case x in {P}) cargo test --all;; x) :;; esac", "[[ -f README.md && {P} == x ]]", "[[ x == {P} ]]", "[[ {P} ]]",
                "[[ -n x || {P} ]] && echo y", "[[ $(echo {P}) ]]", "[[ -n \"$(cargo test --all)\" ]]", "case \"$(cargo test --all)\" in *) :;; esac", "case x in a) : ;; esac && cargo test --all", "if [[ -n cargo && x == cargo ]]; then cargo test --all; fi"]:
        cs.append(mk("V3-7", "case", tpl.replace("{P}", p)))
        cs.append(mk("V3-7", "case-det", tpl.replace("{P}", p).replace("cargo test --all", "bash scripts/hosts/detached-run.sh start --label m -- sleep 1"), "det"))
dump(out + "/cases-x1-V3-7.json", cs)
