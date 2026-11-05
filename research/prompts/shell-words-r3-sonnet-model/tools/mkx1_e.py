"""mkx1_e.py <输出目录> : W3（detached-run 新认法：拷出再起、命令词代入变量、eval 字面代码、包装器；重型闸认 ln -s / install / mv 再起）与主 agent 照 main-agent.md 的起法。"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from x1common import *
out = sys.argv[1]
SRC = "scripts/hosts/detached-run.sh"; TAIL = " start --label m -- sleep 1"
cs = []
copy_tpls = ["cp {S} d.sh && bash d.sh{T}", "cat {S} > d.sh && bash d.sh{T}", "cat {S} >d.sh; sh d.sh{T}", "install -m755 {S} d.sh && ./d.sh{T}", "install {S} d.sh; bash d.sh{T}", "ln -s {S} d.sh && bash d.sh{T}",
             "ln {S} d.sh && bash d.sh{T}", "mv {S} d.sh && bash d.sh{T}", "cp -a {S} d.sh; bash d.sh{T}", "cp {S} sub/d.sh; bash sub/d.sh{T}", "cp {S} ./d.sh; bash ./d.sh{T}", "cp {S} \"d.sh\"; bash 'd.sh'{T}",
             "cp {S} d.sh; (bash d.sh{T})", "cp {S} d.sh; bash -c 'bash d.sh{T}'", "cp {S} d.sh; nice bash d.sh{T}", "cp {S} d.sh; env A=1 bash d.sh{T}", "cp {S} d.sh; eval 'bash d.sh{T}'", "cp {S} d.sh; X=d.sh; bash $X{T}",
             "cp {S} d.sh; . d.sh{T}", "cp {S} d.sh; source d.sh{T}", "cp {S} d.sh; exec bash d.sh{T}", "cat {S} | tee d.sh >/dev/null; bash d.sh{T}", "tee d.sh < {S} >/dev/null; bash d.sh{T}", "cat {S} >> d.sh; bash d.sh{T}",
             "cp {S} d.sh; cp d.sh e.sh; bash e.sh{T}", "cat {S} > d.sh 2>/dev/null; bash d.sh{T}", "cat {S} > d.sh 2>&1; bash d.sh{T}", "ln -sf {S} d.sh; bash d.sh{T}", "ln -s $PWD/{S} d.sh; bash d.sh{T}", "ln -s ../{S} sub/d.sh; bash sub/d.sh{T}",
             "cp {S} d.sh; chmod +x d.sh; ./d.sh{T}", "cp {S} d.sh; sed -i s/x/y/ d.sh; bash d.sh{T}", "cd sub; cp ../{S} d.sh; bash d.sh{T}", "cd sub && ln -s ../{S} d.sh && bash d.sh{T}", "cp {S} d.sh; bash d.sh{T} &\nwait"]
for t in copy_tpls:
    cs.append(mk("W3", "copy-det", t.replace("{S}", SRC).replace("{T}", TAIL), "det"))
    cs.append(mk("W3", "copy-det-subcwd", t.replace("{S}", "../" + SRC).replace("{T}", TAIL), "det", cwd=os.getcwd() + "/scratch/sub"))
    cs.append(mk("W3", "copy-guard", t.replace("{S}", "s.sh").replace("{T}", "")))
    cs.append(mk("W3", "copy-guard-subcwd", t.replace("{S}", "../s.sh").replace("{T}", ""), cwd=os.getcwd() + "/scratch/sub"))
word_tpls = ["X=bash; $X {S}{T}", "X={S}; bash $X{T}", "X={S}; bash \"$X\"{T}", "X={S}; bash ${X}{T}", "X=detached-run.sh; bash scripts/hosts/$X{T}", "X=scripts/hosts; bash $X/detached-run.sh{T}", "X=\"bash {S}\"; $X{T}",
             "X='bash'; Y={S}; $X $Y{T}", "X={S}; $X{T}", "X={S}; sh -c \"bash $X{T}\"", "eval 'bash {S}{T}'", "eval \"bash {S}{T}\"", "eval bash {S}{T}", "eval 'eval \"bash {S}{T}\"'", "eval 'X=1; bash {S}{T}'", "eval \"$(echo bash {S}{T})\"",
             "/usr/bin/time bash {S}{T}", "/usr/bin/time -v bash {S}{T}", "flock /tmp/l bash {S}{T}", "flock -n /tmp/l -c 'bash {S}{T}'", "systemd-run --wait bash {S}{T}", "systemd-run --user --wait --pipe bash {S}{T}", "chrt -i 0 bash {S}{T}",
             "taskset -c 0 bash {S}{T}", "ionice -c3 bash {S}{T}", "timeout 5 bash {S}{T}", "stdbuf -oL bash {S}{T}", "setsid -w bash {S}{T}", "nohup bash {S}{T}", "sudo -n bash {S}{T}", "xargs -n1 bash <<< '{S}'", "bash -lc 'bash {S}{T}'",
             "script -qc 'bash {S}{T}' /dev/null", "nice -n 3 ionice -c 3 timeout 9 bash {S}{T}", "env -S 'bash {S}{T}'", "command bash {S}{T}", "exec bash {S}{T}", "time bash {S}{T}", "{ bash {S}{T}; }", "( bash {S}{T} )", "bash {S}{T} &\nwait", "bash {S}{T} | cat"]
for t in word_tpls:
    cs.append(mk("W3", "word-det", t.replace("{S}", SRC).replace("{T}", TAIL), "det"))
    cs.append(mk("W3", "word-det-main", t.replace("{S}", SRC).replace("{T}", " start --wall-clock within-2h --label m -- sleep 1"), "det", agent="main", noexec=True, expect=0))
for a in ["main", "clerk"]:    # 主 agent 照 main-agent.md 的起法
    pass
main_forms = ["bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label m -- sleep 1", "bash scripts/hosts/detached-run.sh start --wall-clock within-2h --label m -- sleep 1",
              "bash scripts/hosts/detached-run.sh start --label m --wall-clock over-2h -- sleep 1", "cd /home && bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label m -- sleep 1",
              "bash ./scripts/hosts/detached-run.sh start --wall-clock over-2h --label m -- sleep 1", "bash \"scripts/hosts/detached-run.sh\" start --wall-clock over-2h --label m -- sleep 1",
              "S=scripts/hosts/detached-run.sh; bash $S start --wall-clock over-2h --label m -- sleep 1", "scripts/hosts/detached-run.sh start --wall-clock over-2h --label m -- sleep 1",
              "bash scripts/hosts/detached-run.sh list", "bash scripts/hosts/detached-run.sh watch --label m", "cp scripts/hosts/detached-run.sh d.sh; bash d.sh start --wall-clock over-2h --label m -- sleep 1",
              "cat scripts/hosts/detached-run.sh > d.sh && bash d.sh start --wall-clock over-2h --label m -- sleep 1"]
for m in main_forms:
    for pre in ["", "nice ", "env A=1 "]:
        cs.append(mk("W3", "main-form", pre + m, "det", agent="main", noexec=True, expect=0))
dump(out + "/cases-x1-W3.json", cs)
