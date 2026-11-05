#!/usr/bin/env python3
"""G1 其余钩子的喂法：写闸、检出钩子、派发闸（假的 admission.py）、内存包装拷贝边界。每行：组 / 期望 / 实际 / 标签。最后一行数符合几条。"""
import sys, os, shutil, warnings
warnings.filterwarnings("ignore")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import feed, sandbox_reset as R
LEG, TREE, S = feed.LEG, feed.TREE, feed.SANDBOX
U = os.getuid(); SES = "/tmp/claude-%d" % U
WR = LEG + "/workroot/s1"; HOME_DIRECTORY = os.path.expanduser("~"); OUT = HOME_DIRECTORY + "/outside_d137"
ROWS = []
def rec(group, want, got, label):
    ROWS.append((group, want, got, label)); print("%s\t%s\t%s\t%s\t%s" % (group, want, got, "符合" if want == got else "不符", label))
def det(group, label, cmd, want, agent="implementation-writer", bg=False, cwd=TREE):
    rc, err, notes = feed.feed("bash-command-detector.sh", cmd, agent=agent, cwd=cwd, background=bg)
    rec(group, want, rc, "%s | %s" % (label, cmd.replace("\n", "\\n")[:90]))
def write_tool(group, agent, path, want):
    rc, err, notes = feed.feed("write-guard.sh", None, agent=agent, tool="Write", tool_input={"file_path": path, "content": "x\n"})
    rec(group, want, rc, "Write %s -> %s" % (agent or "主agent", path.replace(LEG, "LEG")))
os.makedirs(WR, exist_ok=True); os.makedirs(SES + "/d137-probe", exist_ok=True)
open(SES + "/d137-probe/old.sh", "w").write("echo old\n"); os.chmod(SES + "/d137-probe/old.sh", 0o755)
open(WR + "/work.sh", "w").write("echo w\n")
# ---- 撤旧根：写闸
for agent in ("implementation-writer", "general-purpose"):
    for p in (SES + "/scratch-probe/x.md", SES + "/proj/sess/scratchpad/x.md"):
        write_tool("旧根:写闸", agent, p, 2)
    write_tool("旧根:写闸", agent, LEG + "/workroot/r1/x.md", 0)
for p in (SES + "/scratch-probe/x.md", LEG + "/workroot/r1/x.md"):
    write_tool("旧根:写闸", None, p, 0)
# ---- 撤旧根：检出钩子
for agent, want_new in (("general-purpose", 2), ("implementation-writer", 0), (None, 0)):
    det("旧根:检出", "新建文件 %s" % (agent or "主agent"), "echo x > %s/d137-probe/new.txt" % SES, want_new, agent=agent, cwd=WR)
    det("旧根:检出", "tee %s" % (agent or "主agent"), "echo x | tee %s/d137-probe/new2.txt" % SES, want_new, agent=agent, cwd=WR)
for agent in ("general-purpose", "implementation-writer", None):
    det("旧根:检出⑦", "已有脚本 > %s" % (agent or "主agent"), "echo x > %s/d137-probe/old.sh" % SES, 2, agent=agent, cwd=WR)
    det("旧根:检出⑦", "已有脚本 sed -i（不在 ⑦ 里）%s" % (agent or "主agent"), "sed -i s/a/b/ %s/d137-probe/old.sh" % SES, 0, agent=agent, cwd=WR)
# ---- ⑫ 写范围（cwd 在拷来的树，实现员）
for label, cmd, want in [
    ("仓内范围外 >", "echo x > kb.md", 2), ("范围内 >", "echo x > crates/ok.rs", 0), ("records >", "echo x > records/r.md", 2),
    ("tee 仓内范围外", "echo x | tee records/r.md", 2), ("tee 花括号", "echo x | tee records/{a,b}.md", 2), ("引号里花括号的重定向目标", "echo x > \"records/{a,b}.md\"", 2),
    ("cp", "cp a records/a.md", 2), ("for 写死词", "for f in a b; do echo x > records/$f; done", 2), ("for $(ls)", "for f in $(ls); do echo x > records/$f; done", 2),
    ("eval 字面", "eval 'echo x > records/a'", 2), ("sed -i", "sed -i s/a/b/ records/a.md", 2), ("find -exec sed -i", "find . -exec sed -i s/a/b/ {} \;", 2),
    ("xargs sed -i", "echo f | xargs sed -i s/a/b/", 2), ("xargs -a", "xargs -a list sed -i s/a/b/", 2), ("python 写", "python3 -c \"open('records/z','w')\"", 2),
    ("变量代入", "d=records; echo x > $d/z", 2), ("$PWD", "echo x > $PWD/records/z", 2), ("cd 之后", "cd records && echo x > z", 2), ("../ 回爬", "echo x > crates/../records/z", 2),
    ("bash -c", "bash -c 'echo x > records/z'", 2), ("parallel sed -i", "parallel sed -i s/a/b/ ::: records/a.md", 2), ("mktemp -d 赋的变量", "d=$(mktemp -d); echo x > $d/f", 0),
    ("仓外不判", "echo x > %s/f" % OUT, 0), ("仓外 tee 花括号不判", "echo x | tee %s/o{1,2}" % OUT, 0)]:
    det("⑫写范围", label, cmd, want)
for label, cmd, want in [("重定向花括号", "echo x > out{1,2}.txt", 2), ("引号里的重定向花括号（真 bash 写一个字面名文件）", "echo x > \"out{1,2}.txt\"", 0),
                          ("单项花括号", "echo x > out{1}.txt", 0), ("tee 花括号（工作根内）", "echo x | tee out{1,2}.txt", 0)]:
    det("⑫花括号", label, cmd, want, cwd=WR)
det("⑫花括号", "主agent 同一条（不判）", "echo x > out{1,2}.txt", 0, agent=None, cwd=WR)
# ---- ② ④ ① ⑬ ⑮（主 agent 为主）
for label, cmd, want, bg in [
    ("setsid & 后台", "setsid cargo build &", 2, True), ("setsid --wait", "setsid --wait cargo build", 0, True), ("setsid 不带 --wait 记检出放行", "setsid bash -c 'sleep 1'", 0, True),
    ("两个 & 各有 wait $p", "( sleep 1 ) & p1=$!; ( sleep 1 ) & p2=$!; wait $p1; wait $p2", 0, True), ("两个 & 只等一个", "sleep 1 & sleep 2 & wait $!", 2, True),
    ("coproc", "coproc sleep 1", 2, True), ("nohup &", "nohup sleep 1 &", 2, True), ("看门狗 > /dev/null", "bash scripts/agents/watch.sh > /dev/null", 2, True),
    ("看门狗 2>/dev/null（挂在看门狗自己身上）", "bash scripts/agents/watch.sh 2>/dev/null", 2, True), ("花括号整组丢进 /dev/null", "{ bash scripts/agents/watch.sh; } > /dev/null", 2, True),
    ("别的命令的 2>/dev/null", "grep x f 2>/dev/null; bash scripts/agents/watch.sh", 0, True),
    ("git reset", "git reset", 2, False), ("git restore --staged .", "git restore --staged .", 2, False), ("git stash", "git stash", 2, False),
    ("git stash push -- 路径", "git stash push -- crates/a.rs", 0, False), ("git 子命令放变量", "g=reset; git $g", 2, False)]:
    det("②④①⑮", label, cmd, want, agent=None, bg=bg)
open(S + "/claudeenv.sh", "w").write("claude -p hi\n")
det("⑬", "claude -p", "claude -p hi", 2)
det("⑬", "BASH_ENV 文件里起 claude", "BASH_ENV=%s/claudeenv.sh bash -c true" % S, 2)
det("⑬", "--rcfile 文件里起 claude", "bash --rcfile %s/claudeenv.sh -i -c true" % S, 2)
det("⑬", "BASH_ENV 文件无害", "BASH_ENV=%s/bashenv.sh bash -c true" % S, 0)
# ---- 内存包装与拷贝边界
os.makedirs(S + "/cp", exist_ok=True)
for name in ("run-with-memory-cap.sh", "capped.sh"):
    shutil.copy(TREE + "/scripts/hosts/" + name, S + "/cp/" + name)
shutil.copy(TREE + "/scripts/hosts/run-with-memory-cap.sh", S + "/cp/rwmc-mod.sh")
shutil.copy(TREE + "/scripts/hosts/capped.sh", S + "/cp/capped-mod.sh")
for n in ("rwmc-mod.sh", "capped-mod.sh"):
    with open(S + "/cp/" + n, "a") as f: f.write("cargo test --all\n")
def heavy(group, label, cmd, want, cwd=None, agent="implementation-writer"):
    rc, err, notes = feed.feed("heavy-test-guard.sh", cmd, agent=agent, cwd=cwd)
    rec(group, want, rc, "%s | %s" % (label, cmd[:90]))
heavy("拷贝边界", "本仓内存包装", "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib", 0, cwd=TREE)
heavy("拷贝边界", "逐字节相同的拷贝", "bash cp/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib", 0)
heavy("拷贝边界", "尾部多一行的拷贝", "bash cp/rwmc-mod.sh 4G cargo test -p singlefs-harness --lib", 2)
heavy("拷贝边界", "逐字节相同的 capped.sh 拷贝", "bash cp/capped.sh 4 bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib", 0, cwd=TREE)
heavy("拷贝边界", "尾部多一行的 capped.sh 拷贝", "bash cp/capped-mod.sh 4 echo hi", 2)
shutil.rmtree(S + "/cp", ignore_errors=True)
# ---- 派发闸
import dispatch_probe as d
stub = lambda n: os.path.join(LEG, "stubs", n)
count = [0]
def dispatch(group, label, code, want, extra="", draft=None, body="写重跑登记 research/prompts/e142-r19-prereg.md。"):
    count[0] += 1
    d.draft = draft or os.path.join(LEG, "workroot", "dg%d_%d" % (os.getpid(), count[0])); os.makedirs(d.draft, exist_ok=True) if not draft or draft.startswith(LEG) else None
    env = {"RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT": stub(code)} if code else {}
    rc, err, notes = feed.feed("runner-dispatch-guard.sh", None, env=env, tool="Agent", tool_input={"subagent_type": "experiment-designer", "prompt": d.prompt(extra, body=body)}, timeout=100)
    rec(group, want, rc, "%s（%d 条检出）" % (label, len(notes)))
for code, want in (("adm0.py", 0), ("adm2.py", 2), ("adm3.py", 2), ("adm77.py", 2), ("adm1.py", 0), ("adm4.py", 0), ("adm5.py", 0), ("adm124.py", 0), ("adm250.py", 0), ("admhang.py", 0)):
    dispatch("准入", "重跑登记 admission 退 %s" % code[3:-3], code, want)
dispatch("准入", "退 2 + 准入已判", "adm2.py", 0, extra="准入已判：换了一条臂\n")
dispatch("准入", "退 3 + 准入已判", "adm3.py", 0, extra="准入已判：换了一条臂\n")
dispatch("准入", "退 77 + 准入已判", "adm77.py", 0, extra="准入已判：换了一条臂\n")
dispatch("准入", "非重跑登记（退 2 的假脚本也不判）", "adm2.py", 0, body="写跑前登记 research/prompts/e170-preregistration.md。")
dispatch("准入", "admission.py 不在（python 自己退 2）", "nonexistent_d137.py", 0)
for label, draft, want in (("草稿目录在会话临时目录根", SES + "/draft-d137", 2), ("草稿目录是工作根本身", os.path.join(LEG, "workroot"), 2), ("草稿目录相对路径", "work/x", 2),
                           ("草稿目录在仓外别处", HOME_DIRECTORY + "/elsewhere_d137", 2), ("草稿目录 /tmp/x", "/tmp/d137x", 2), ("草稿目录 $HOME 写法的工作根下", ("$HOME" + LEG[len(HOME_DIRECTORY):] if LEG.startswith(HOME_DIRECTORY + "/") else LEG) + "/workroot/h%d" % os.getpid(), 0)):
    dispatch("草稿目录", label, "adm0.py", want, draft=draft, body="写跑前登记 research/prompts/e170-preregistration.md。")
dispatch("草稿目录", "报告行点名会话临时目录根", "adm0.py", 2, extra="报告：%s/out.md\n" % SES, body="写跑前登记 research/prompts/e170-preregistration.md。")
dispatch("草稿目录", "自证在哪跑点名会话临时目录根", "adm0.py", 2, extra="自证在哪跑：%s/x\n" % SES, body="写跑前登记 research/prompts/e170-preregistration.md。")
dispatch("草稿目录", "只读引用会话临时目录根下的路径", "adm0.py", 0, extra="读 %s/proj/sess/scratchpad/notes.md 里的材料。\n" % SES, body="写跑前登记 research/prompts/e170-preregistration.md。")
shutil.rmtree(SES + "/d137-probe", ignore_errors=True)
ok = sum(1 for r in ROWS if r[1] == r[2])
print("合计\t%d\t符合\t%d\t不符\t%d" % (len(ROWS), ok, len(ROWS) - ok))
