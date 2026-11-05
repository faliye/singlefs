#!/usr/bin/env python3
"""G2：共用约束与脚本文件头里新写的句子，拿句子的意思逐条喂钩子。期望照句子写，不照钩子输出。"""
import sys, os, warnings, shutil
warnings.filterwarnings("ignore")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import feed
LEG, TREE, S = feed.LEG, feed.TREE, feed.SANDBOX
ROWS = []
def rec(group, want, got, label, sentence):
    ROWS.append((group, want, got)); print("%s\t%s\t%s\t%s\t%s\t%s" % (group, want, got, "符合" if want == got else "不符", sentence, label))
def run(hook, group, sentence, label, cmd, want, agent="implementation-writer", bg=False, cwd=TREE, env=None):
    rc, err, notes = feed.feed(hook, cmd, agent=agent, cwd=cwd, background=bg, env=env)
    rec(group, want, rc, "%s | %s" % (label, cmd.replace("\n", "\\n")[:100]), sentence)
D, H = "bash-command-detector.sh", "heavy-test-guard.sh"
os.makedirs(S + "/g2", exist_ok=True)
shutil.copy(TREE + "/scripts/hosts/detached-run.sh", S + "/g2/dr-copy.sh")
lines = open(TREE + "/scripts/hosts/detached-run.sh", encoding="utf-8").read().split("\n")
open(S + "/g2/dr-nosig.sh", "w", encoding="utf-8").write("\n".join(l for l in lines if "detached-run-script-signature" not in l))
shutil.copy(TREE + "/scripts/crash/crash-point-replay.sh", S + "/g2/cpr-copy.sh")
if not os.path.exists(S + "/g2/cpr-link.sh"): os.symlink(TREE + "/scripts/crash/crash-point-replay.sh", S + "/g2/cpr-link.sh")
for n in ():
    pass
# ---- detached-run.sh 文件头：「认得出」与「罩不住」
RUN = "start --wall-clock within-2h --label probe -- true"
dr = "scripts/hosts/detached-run.sh"
S1 = "detached-run.sh 文件头：bash/sh 起的、source/. 读进来的、直接执行的都认；改名拷贝按签名行认"
run(D, "dr", S1, "直接 bash 起", "bash %s %s" % (dr, RUN), 2)
run(D, "dr", S1, "source 读进来", "source %s %s" % (dr, RUN), 2)
run(D, "dr", S1, "直接执行", "./%s %s" % (dr, RUN), 2)
run(D, "dr", S1, "带签名行的改名拷贝（在 cwd 里）", "bash dr-copy.sh %s" % RUN, 2, cwd=S + "/g2")
S2 = "detached-run.sh 文件头：同一条命令里先 cp、install、ln、mv 再起的按目标认"
for verb in ("cp", "install -m755", "ln -s", "mv"):
    run(D, "dr", S2, verb + " 再起", "%s %s ./dr_x.sh && bash ./dr_x.sh %s" % (verb, dr if verb != "mv" else "scripts/hosts/detached-run.sh", RUN), 2)
S3 = "detached-run.sh 文件头：路径写在前面赋过字面值的变量里的照代入再认；eval 的字面代码切开再认；/usr/bin/time、flock、systemd-run --wait 与 xargs 后面写成字面路径的剥开再认"
run(D, "dr", S3, "变量代入", 'S=%s; bash "$S" %s' % (dr, RUN), 2)
run(D, "dr", S3, "eval 字面", "eval 'bash %s %s'" % (dr, RUN), 2)
run(D, "dr", S3, "/usr/bin/time", "/usr/bin/time bash %s %s" % (dr, RUN), 2)
run(D, "dr", S3, "flock", "flock lockfile bash %s %s" % (dr, RUN), 2)
run(D, "dr", S3, "systemd-run --wait", "systemd-run --user --wait bash %s %s" % (dr, RUN), 2)
run(D, "dr", S3, "xargs 字面路径", "echo x | xargs bash %s %s" % (dr, RUN), 2)
S4 = "detached-run.sh 文件头：认不出路径、由重型闸照「认不出起什么的写法」拒：eval 的参数是变量、路径在没赋过值的变量里、xargs -I 与 find -exec 把读进来的词接成路径"
run(H, "dr", S4, "eval \"$c\"", 'eval "$c"', 2)
run(H, "dr", S4, "没赋过值的变量作路径", 'bash "$U" %s' % RUN, 2)
run(H, "dr", S4, "xargs -I 接成路径", "echo %s | xargs -I{} bash {} %s" % (dr, RUN), 2)
run(H, "dr", S4, "find -exec 接成路径", "find scripts/hosts -name detached-run.sh -exec bash {} %s \;" % RUN, 2)
S5 = "detached-run.sh 文件头：罩不住的写法：删掉签名行的拷贝；python 的 subprocess 起的"
run(D, "dr", S5, "删掉签名行的拷贝（罩不住，放）", "bash dr-nosig.sh %s" % RUN, 0, cwd=S + "/g2")
run(D, "dr", S5, "python subprocess（罩不住，放）", "python3 -c \"import subprocess; subprocess.run(['bash','%s','start','--wall-clock','within-2h','--label','p','--','true'])\"" % dr, 0)
# ---- agent-common.md ②：前台没超时的等待循环
S6 = "agent-common.md 检出 ②：带计数上限、break、while read、kill -0 探活这类真会结束的循环同样照拒，外面套 timeout 就放行"
run(D, "②", S6, "until sleep 无上限", "until [ -f done ]; do sleep 2; done", 2, agent=None)
run(D, "②", S6, "带 break 的 while", "while true; do sleep 1; [ -f done ] && break; done", 2, agent=None)
run(D, "②", S6, "带计数的 for+sleep 不是 until/while（不判）", "for i in 1 2 3; do sleep 1; done", 0, agent=None)
run(D, "②", S6, "while read + sleep", "while read -r l; do sleep 1; done < list", 2, agent=None)
run(D, "②", S6, "kill -0 探活", "while kill -0 123 2>/dev/null; do sleep 1; done", 2, agent=None)
run(D, "②", S6, "外面套 timeout", "timeout 60 bash -c 'until [ -f done ]; do sleep 2; done'", 0, agent=None)

# ---- ④ 数法
S7 = "agent-common.md 检出 ④：不带参数的 wait、调用里有不带参数 wait 的函数算等齐；wait <进程号> 写了几个号算几个；wait -n、tail --pid、kill -0 各算一个"
run(D, "④", S7, "两个 & 一个不带参数 wait", "sleep 1 & sleep 2 & wait", 0, agent=None, bg=True)
run(D, "④", S7, "函数里不带参数 wait", "f() { wait; }; sleep 1 & sleep 2 & f", 0, agent=None, bg=True)
run(D, "④", S7, "两个 & 只 wait -n 一次", "sleep 1 & sleep 2 & wait -n", 2, agent=None, bg=True)
run(D, "④", S7, "一个 & 加 wait -n", "sleep 1 & wait -n", 0, agent=None, bg=True)
run(D, "④", S7, "一个 & 加 tail --pid", "sleep 1 & tail --pid=$! -f /dev/null", 0, agent=None, bg=True)
run(D, "④", S7, "两个 & 加一个 tail --pid", "sleep 1 & sleep 2 & tail --pid=$! -f /dev/null", 2, agent=None, bg=True)
run(D, "④", S7, "wait 两个号等两个", "sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1 $p2", 0, agent=None, bg=True)
run(D, "④", S7, "两个 & 只 wait 一个号", "sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1", 2, agent=None, bg=True)
run(D, "④", S7, "kill -0 轮询算一个", "sleep 1 & while kill -0 $!; do sleep 1; done", 0, agent=None, bg=True)
# ---- ⑦ ⑬ ⑮ 的新增写法
S8 = "agent-common.md ⑦：rsync --inplace 的目标；ⓕ⑬：名单外启动器、here-string、env -S 拆出来的、BASH_ENV/--rcfile/bash -i 读 ~/.bashrc 里起 claude"
run(D, "⑦⑬", S8, "rsync --inplace 改已有脚本", "rsync --inplace /dev/null scripts/hosts/capped.sh", 2, agent=None)
run(D, "⑦⑬", S8, "rsync 不带 --inplace 不拦", "rsync /dev/null scripts/hosts/capped.sh", 0, agent=None)
run(D, "⑦⑬", S8, "xargs 起 claude", "echo x | xargs claude", 2)
run(D, "⑦⑬", S8, "ssh 起 claude", "ssh localhost claude -p hi", 2)
run(D, "⑦⑬", S8, "find -exec claude", "find . -maxdepth 0 -exec claude {} \;", 2)
run(D, "⑦⑬", S8, "here-string 喂 bash", "bash <<< 'claude -p hi'", 2)
run(D, "⑦⑬", S8, "env -S 拆出来", "env -S 'claude -p hi'", 2)
open(S + "/claudeenv.sh", "w").write("claude -p hi\n")
open(S + "/.bashrc", "w").write("claude -p hi\n")
run(D, "⑦⑬", S8, "bash -i 读 HOME 下 .bashrc 里起 claude", "HOME=%s bash -i -c true" % S, 2)
run(D, "⑦⑬", S8, "bash -i --norc 不读", "HOME=%s bash --norc -i -c true" % S, 0)
run(D, "⑦⑬", S8, "ENV 指到的文件里起 claude（sh）", "ENV=%s/claudeenv.sh sh -c true" % S, 2)
run(D, "⑦⑬", S8, "ZDOTDIR 下 .zshrc 里起 claude", "ZDOTDIR=%s zsh -c true" % S, 0)
open(S + "/.bashrc", "w").write("cargo test --all\n")
# ---- ⑮：agent-common.md 句子里的几种
S9 = "agent-common.md ⑮：git rm --cached 整棵树、git checkout [<提交>] -- <整棵树>、git stash 带 -k 不拦"
run(D, "⑮", S9, "git rm --cached -r .", "git rm --cached -r .", 2, agent=None)
run(D, "⑮", S9, "git checkout HEAD -- .", "git checkout HEAD -- .", 2, agent=None)
run(D, "⑮", S9, "git checkout -- .", "git checkout -- .", 2, agent=None)
run(D, "⑮", S9, "git stash -k", "git stash push --keep-index", 0, agent=None)
run(D, "⑮", S9, "git reset --soft", "git reset --soft HEAD~1", 0, agent=None)
run(D, "⑮", S9, "git checkout -- 具体路径", "git checkout -- crates/a.rs", 0, agent=None)
# ---- 重型闸：放行靠参数的三份入口只认本仓那一份
S10 = "agent-common.md 重型闸：放行靠参数的三份入口（replay.sh、crash-point-replay.sh、crash-amplification.sh）只认本仓那一份，别处的拷贝与经符号链接起的照读正文"
q = "bash scripts/hosts/run-with-memory-cap.sh 4G bash "
run(H, "入口", S10, "本仓 crash-point-replay.sh --quick", q + "scripts/crash/crash-point-replay.sh --quick", 0)
run(H, "入口", S10, "别处拷贝 --quick", q + "../../sandbox/g2/cpr-copy.sh --quick", 2, cwd=S + "/g2")
run(H, "入口", S10, "符号链接 --quick", q + "cpr-link.sh --quick", 2, cwd=S + "/g2")
run(H, "入口", S10, "本仓 crash-point-replay.sh 不带参数", q + "scripts/crash/crash-point-replay.sh", 2)
# ---- 钩子 bash 那一层：python3 起不来（PATH 里没有 python3）时，文件头与 agent-common.md 说照常执行并记一条检出
nopy = LEG + "/nopy"
os.makedirs(nopy, exist_ok=True)
for name in os.listdir("/usr/bin"):
    if name.startswith(("python", "pypy")) or os.path.lexists(os.path.join(nopy, name)): continue
    try: os.symlink("/usr/bin/" + name, os.path.join(nopy, name))
    except OSError: pass
S11 = "agent-common.md 检出条目首句：判不了的（……钩子 bash 那一层起不来）照常执行并记一条检出（只记）"
for hook, tool, tool_input, agent in ((H, "Bash", {"command": "cargo test --all"}, "implementation-writer"), (D, "Bash", {"command": "claude -p hi"}, "implementation-writer")):
    before = sum(1 for _ in open(feed.DETECT)) if os.path.exists(feed.DETECT) else 0
    rc, err, notes = feed.feed(hook, None, agent=agent, cwd=TREE, tool=tool, tool_input=tool_input, env={"PATH": nopy})
    rec("bash层", 0, rc, "%s 没有 python3：退出码" % hook, S11)
    rec("bash层", 1, len(notes), "%s 没有 python3：记了几条检出" % hook, S11)
shutil.rmtree(S + "/g2", ignore_errors=True)
ok = sum(1 for r in ROWS if r[1] == r[2])
print("合计\t%d\t符合\t%d\t不符\t%d" % (len(ROWS), ok, len(ROWS) - ok))
