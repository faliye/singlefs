# R1：派发闸 ⑮ 线程上限。第一轮两腿的喂法（正推 D1-3 的表、攻方 A1/A3）在改前、改后两棵树上重喂；再加几种绕过写法。
# 用法：python3 -I r1_thread_limit.py <改前树> <改后树> <工作目录>
import os, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as c
trees, work = c.parse_trees(sys.argv)

def feed(typ, lines, env=None, wall=True):
    def fn(tree):
        rc, tg, out = c.check_prompt(tree, typ, c.prompt(typ, lines), env=env, workdir=work)
        return rc, ",".join(tg) or "-"
    return fn

# 正推 D1-3 的写法表（clerk）
for cid, value in [
    ("S7-31", "31"), ("S7-div", "31 ÷ 同时在跑的重活数"), ("S7-slash", "31/2"), ("S7-half", "31 的一半，即 15"), ("S7-031", "031"),
    ("S7-32", "32"), ("S7-30", "30"), ("S7-310", "310"), ("S7-31.5", "31.5"), ("S7-N=31", "N = 31"), ("S7-default", "default"),
    ("S7-cores-div", "整机核数 ÷ 同时在跑的重活数"),
    ("A3-formula", "31 ÷ 2"), ("A3-second-host", "31（本机），第二台 16"),
    # 加的绕过写法
    ("X-ok-paren", "31（本机）"), ("X-ok-paren-space", "31  （每台）"), ("X-fullwidth-digit", "３１"), ("X-superscript", "³¹"),
    ("X-zero-width-inside", "3​1"), ("X-zero-width-trail", "31​"), ("X-paren-digits", "31（每台 31 个）"),
    ("X-paren-not-number", "31（不看忙闲，机器忙也固定）"), ("X-paren-chinese-numeral", "31（本机，第二台十六）"), ("X-paren-fraction-words", "31（二分之一）"), ("X-halfwidth-paren", "31(本机)"), ("X-comma", "31，不看忙闲"),
    ("X-arabic-indic", "٣١"), ("X-plus", "+31"), ("X-leading-space-tab", "\t31"),
]:
    c.both("R1", cid, feed("clerk", f"线程上限：{value}\n"), trees)
# 行首记号
for cid, line in [("S7-halfwidth-colon", "线程上限: 31\n"), ("S7-list", "- 线程上限：31\n"), ("S7-bold", "**线程上限**：31\n"),
                  ("X-two-lines-31-and-31", "线程上限：31\n线程上限：31\n"), ("X-two-lines-31-and-16", "线程上限：31\n线程上限：16\n"),
                  ("X-indented", "  线程上限：16\n"), ("X-quote-prefix", "> 线程上限：16\n")]:
    c.both("R1", cid, feed("clerk", line), trees)
# 已判行
for cid, lines in [("S3-valve-reason", "线程上限：16\n线程上限已判：用户给了参数\n"), ("S3-valve-x", "线程上限已判：x\n"),
                   ("S3-valve-placeholder", "线程上限：16\n线程上限已判：<理由>\n"), ("S3-valve-empty", "线程上限：16\n线程上限已判：\n"),
                   ("X-valve-anything-formula", "线程上限：31 ÷ 2\n线程上限已判：随便\n"),
                   ("X-valve-whitespace-reason", "线程上限：16\n线程上限已判： \n")]:
    c.both("R1", cid, feed("clerk", lines), trees)
# 缺行：六种类型 + 不在表里的类型
for typ in ["implementation-writer", "investigator", "tooling-writer", "experiment-runner", "mutation-triage", "three-way-attack",
            "clerk", "sweep", "kb-scribe", "three-way-forward", "three-way-defense", "three-way-materials", "prior-art", "experiment-designer"]:
    c.both("R1", f"S8-missing-line-{typ}", feed(typ, ""), trees)
    c.both("R1", f"S8-missing-line-valve-{typ}", feed(typ, "线程上限已判：这件活只读不起重活\n"), trees)
    c.both("R1", f"S8-line-31-{typ}", feed(typ, "线程上限：31\n"), trees)
# 六种类型里写占位：<填 …> 交 ⑬，别的以 < 起头的值 ⑮ 当占位不判（写了一行 ⇒ 不算缺行）
for cid, value in [("X-placeholder-fill", "<填 登记值>"), ("X-placeholder-angle-anything", "<随便>"), ("X-placeholder-angle-31", "<31>")]:
    c.both("R1", cid + "-implementation-writer", feed("implementation-writer", f"线程上限：{value}\n"), trees)
# 攻方 A1：环境里 SINGLEFS_THREAD_CAP=8，提示写 8 / 31
for cid, lines in [("A1-env8-prompt31", "线程上限：31\n"), ("A1-env8-prompt8", "线程上限：8\n"), ("A1-env8-prompt8-valve", "线程上限：8\n线程上限已判：用户给了参数 8\n")]:
    c.both("R1", cid, feed("clerk", lines, env={"SINGLEFS_THREAD_CAP": "8"}), trees)

# 骨架：环境里 SINGLEFS_THREAD_CAP=8 时 dispatch-prompt.py 填什么
def skeleton(tree):
    e = dict(os.environ, SINGLEFS_THREAD_CAP="8")
    r = subprocess.run(["python3", "scripts/agents/dispatch-prompt.py", "clerk"], cwd=tree, capture_output=True, text=True, env=e)
    line = [l for l in r.stdout.splitlines() if l.startswith("线程上限")]
    return r.returncode, (line[0] if line else "-").replace(" ", "_")
c.both("R1", "A1-skeleton-env8", skeleton, trees)
