#!/usr/bin/env python3
"""在草稿目录的假钩子上造 hook-swap.py 的几种落地：好换、回滚时别人已改、换上一半。
只在草稿目录下建假仓与假钩子；主工作区一个字节不碰。用法：swap-scenarios.py <草稿目录>"""
import importlib.util, os, shutil, sys
ROOT = "/home/fy5090/code/singlefs"
spec = importlib.util.spec_from_file_location("hook_swap", os.path.join(ROOT, "research/scripts/hook-swap.py"))
hs = importlib.util.module_from_spec(spec); spec.loader.exec_module(hs)

work = os.path.abspath(sys.argv[1]); shutil.rmtree(work, ignore_errors=True); os.makedirs(work)
GUARD = hs.SELFTEST_GUARD
LIB = 'MARKER = "HEAVY"\n\n\ndef refused(command):\n    return MARKER in command\n'
OLD = "    return MARKER in command\n"
NEW = '    return MARKER in command or "*" in command  # 新判法\n'
FEEDS = [{"label":"轻的放行","command":"ls -la","exit":0,"neutral":True},
         {"label":"重的照拒","command":"run HEAVY","exit":2,"neutral":True},
         {"label":"新判法：通配照拒","command":"run GLOB*","exit":2}]

def fresh(name, extra_file=None):
    root = os.path.join(work, name, "repo"); os.makedirs(os.path.join(root, ".claude/hooks")); os.makedirs(os.path.join(root, "research"))
    for fn, text, mode in (("fake-guard.sh", GUARD, 0o755), ("fake_lib.py", LIB, 0o644)):
        p = os.path.join(root, ".claude/hooks", fn)
        open(p, "w").write(text); os.chmod(p, mode)
    if extra_file:
        p = os.path.join(root, ".claude/hooks", extra_file); open(p, "w").write(LIB); os.chmod(p, 0o644)
    return root

def spec_for(root, files):
    return [{"file": os.path.join(root, ".claude/hooks", f), "old": OLD, "new": NEW} for f in files]

def run(label, root, spc, scratch, **kw):
    code, lines = hs.swap(spc, os.path.join(root, ".claude/hooks/fake-guard.sh"), scratch, root, feeds=FEEDS, **kw)
    print(f"\n### {label}  -> 退出码 {code}")
    for ln in lines[-8:]:
        print("   " + ln)
    return code

# S1 好换
r = fresh("s1"); run("S1 正常换上（单文件）", r, spec_for(r, ["fake_lib.py"]), os.path.join(work, "s1/scratch"))

# S2 回滚时别人已改：换上之后把新串那行改成别的（通配 feed 会失败 -> 回滚；反向替换找不到新串 -> 退 6）
r = fresh("s2")
def tamper(root=r):
    p = os.path.join(root, ".claude/hooks/fake_lib.py")
    open(p, "w").write(LIB.replace(OLD, '    return MARKER in command  # 别的会话改成了别的一行，新串没了\n'))
run("S2 换上之后别的会话改了新串那行（回滚找不回）", r, spec_for(r, ["fake_lib.py"]), os.path.join(work, "s2/scratch"), after_swap=tamper)

# S3 换上一半：两份文件，rehearse 前把第二份做成多硬链接（rehearse 阶段就拒，一个字节不写）
r = fresh("s3", extra_file="fake_lib2.py")
p2 = os.path.join(r, ".claude/hooks/fake_lib2.py"); os.link(p2, os.path.join(r, ".claude/hooks/fake_lib2.hardlink"))
code = run("S3 第二份有多个硬链接（rehearse 阶段拒）", r, spec_for(r, ["fake_lib.py", "fake_lib2.py"]), os.path.join(work, "s3/scratch"))
# 核：第一份有没有被写（应当没写）
first = open(os.path.join(r, ".claude/hooks/fake_lib.py")).read()
print("   第一份 fake_lib.py 还是原文：", OLD in first and NEW not in first)

# S4 换上一半的另一种：换上之后别的会话改了，但反向替换找得回（加了一行）-> 退 5 优雅回滚
r = fresh("s4")
def append_line(root=r):
    p = os.path.join(root, ".claude/hooks/fake_lib.py")
    existing = open(p).read()
    open(p, "w").write("# 别的会话在钩子里另加的一行\n" + existing)
run("S4 换上之后别的会话另加一行（反向替换找得回）", r, spec_for(r, ["fake_lib.py"]), os.path.join(work, "s4/scratch"),
    after_swap=append_line)
