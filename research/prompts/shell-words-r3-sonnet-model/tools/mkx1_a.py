"""mkx1_a.py <输出目录> : 造 W1（花括号）、W2（heredoc 进替换）、W4（env 赋值）、W5（time / builtin / function / bash 选项 / 双斜杠）的 X1 样本。"""
import sys, itertools, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from x1common import *
out = sys.argv[1]
# ---- W1 花括号：负载词的各种写法 × 外面的包法 ----
words = ["{cargo,} test --all", "cargo{,} test --all", "car{go,} test --all", "{c,}argo test --all", "cargo te{st,} --all", "cargo test --a{ll,}", "cargo test {--all,}", "cargo test {,--all}",
         "cargo test --{all,work{space,}}", "cargo test {,}--all", "cargo test -{-,}all", "cargo test --{all}", "cargo test '--all'{,}", "cargo test \"--{all,x}\"", "cargo test --all\\{,\\}",
         "cargo test --all{,,}", "cargo test {--all,--workspace}", "cargo {test,t} --all", "{,cargo} {,test} --all", "cargo test ${nope:-{--all,}}", "cargo test {-,}-all", "ca\"\"rgo{,} test --all",
         "{ca,}rgo test --all", "cargo test --all{}", "cargo test {--all}", "cargo test {1..2} --all", "cargo test --all {a..c}", "{cargo,cargo} test --all", "cargo test {--all,x}{,}", "cargo{,x} test --all"]
wraps = ["{W}", "env {W}", "env A=1 {W}", "nice {W}", "time {W}", "command {W}", "exec {W}", "bash -c '{W}'", "sh -c \"{W}\"", "eval '{W}'", "( {W} )", "{ {W}; }", "if true; then {W}; fi",
         "for i in 1; do {W}; done", "true && {W}", "false || {W}", "echo x | {W}", "{W} &", "echo $( {W} )", "X=1 {W}", "timeout 5 {W}", "bash <<'EOF'\n{W}\nEOF", "cd sub && {W}", "{W}\n:"]
cs = []
for w in words:
    for wr in wraps: cs += [mk("W1", "brace", wr.replace("{W}", w))]
for w in words[:12]:   # 检出钩子：detached-run 的词里放花括号
    pass
dets = ["bash scripts/hosts/detached-run{,}.sh start --label m -- sleep 1", "bash scripts/{hosts,x}/detached-run.sh start --label m -- sleep 1", "bash {,}scripts/hosts/detached-run.sh start --label m -- sleep 1",
        "{bash,} scripts/hosts/detached-run.sh start --label m -- sleep 1", "bash scripts/hosts/detached-run.sh {start,} --label m -- sleep 1", "bash scripts/hosts/detached-run.sh start --label m -- sleep {1,2}",
        "bash scripts/hosts/detached-run.sh sta{rt,} --label m -- sleep 1", "bash scripts/hosts/detached-run.sh \"{start,x}\" --label m -- sleep 1", "bash scripts/hosts/d{e,}tached-run.sh start --label m -- sleep 1",
        "bash scripts/hosts/detached-run.sh start --label {m,n} -- sleep 1"]
for d in dets:
    for wr in ["{W}", "env {W}", "bash -c '{W}'", "( {W} )", "true && {W}", "eval '{W}'"]:
        cs.append(mk("W1", "brace-det", wr.replace("{W}", d), "det"))
# 序列：补零、步长、字母、负数、反向
for seq in ["{01..03}", "{1..03}", "{001..2}", "{-1..1}", "{1..10..3}", "{01..10..3}", "{a..e}", "{e..a}", "{3..1}", "{03..01}", "{1..3..-1}", "{0..0}", "{5..5}"]:
    for tpl in ["cp s.sh x{S}.sh && bash x1.sh", "cp s.sh x{S}.sh && bash x01.sh", "cp s.sh x{S}.sh && bash x001.sh", "cp s.sh x{S}.sh && bash x4.sh", "cp s.sh x{S}.sh && bash x-1.sh", "cp s.sh x{S}.sh && bash xc.sh",
                "cp s.sh x{S}.sh && bash x0.sh", "cp s.sh x{S}.sh && bash x03.sh", "cp s.sh x{S}.sh && bash x5.sh", "cp s.sh x{S}.sh && bash x10.sh", "cp s.sh x{S}.sh && bash x7.sh", "cp s.sh x{S}.sh && bash xa.sh"]:
        cs.append(mk("W1", "brace-seq", tpl.replace("{S}", seq)))
dump(out + "/cases-x1-W1.json", cs)
