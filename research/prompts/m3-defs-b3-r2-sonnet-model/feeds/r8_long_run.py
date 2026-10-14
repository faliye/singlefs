# R8：⑭ 长活三行。第一轮正推 D7-1：d7-ok（多机行写 host-resources 现算）、d7-quota 各写法、只判设计员与执行员（clerk / investigator / tooling-writer 喂同样的「超过 2 小时」）。
# 度量：--check-prompt 的退出码和 ⑭ 标签（14）。
# 用法：python3 -I r8_long_run.py <改前树> <改后树> <工作目录>
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as c
trees, W = c.parse_trees(sys.argv)

OVER = "预计挂钟：超过 2 小时，推的依据是上一轮同量跑了 5 小时\n"
SHARD = "分片：按变异表行切，片数按多机配置定\n"
RESUME = "续跑：同一条命令重起只补没做完的，用产物里已有的行认做完\n"
LIVE = "多机：台数从 layer0-shard-configuration-check.sh --emit-hosts 读、每台的线程与内存从 host-resources.py --json 现算，不写常量\n"
REGISTERED = "多机：台数取 layer0-shard-configuration-check.sh --emit-hosts，每台线程取 .claude/heavy-thread-default 的登记值\n"

def feed(typ, lines):
    def fn(tree):
        rc, tg, out = c.check_prompt(tree, typ, c.prompt(typ, "线程上限：31\n" + lines), workdir=W)
        return rc, ",".join(tg) or "-"
    return fn

hosts_cases = [
    ("S18-d7-ok-live-compute", LIVE),
    ("R8-registered-wording", REGISTERED),
    ("S18-quota-31threads-64G", "多机：两台，每台 31 线程、64G 内存，用 host-resources.py --json 与 --emit-hosts 核\n"),
    ("S18-quota-chinese-numerals", "多机：两台，每台三十一线程、一百二十八吉内存，用 host-resources.py --json 与 --emit-hosts 核\n"),
    ("S18-quota-chinese-numerals-registered", "多机：两台，每台三十一线程、一百二十八吉内存，取 --emit-hosts 与 .claude/heavy-thread-default\n"),
    ("S18-quota-cards-0-1-2", "多机：用 3 张卡（0、1、2 号），用 host-resources.py --json 与 --emit-hosts 核\n"),
    ("S18-quota-cards-registered", "多机：用 3 张卡（0、1、2 号），台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default\n"),
    ("X-registered-plus-live-word", "多机：台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，余量用 host-resources.py --json 现算\n"),
    ("X-registered-negated-live-word", "多机：台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default，不看 available_threads\n"),
    ("X-registered-no-emit-hosts", "多机：每台线程取 .claude/heavy-thread-default 的登记值\n"),
    ("X-registered-name-only", "多机：--emit-hosts heavy-thread-default\n"),
    ("X-registered-fullwidth-quota", "多机：台数取 --emit-hosts，每台线程取 .claude/heavy-thread-default（本机 ３１ 线程）\n"),
]
for cid, hosts in hosts_cases:
    c.both("R8", cid, feed("experiment-runner", OVER + SHARD + RESUME + hosts + "这一段回答的岔路：岔路单第 1 行\n"), trees)
c.both("R8", "S18-shard-line-with-numbers", feed("experiment-runner", OVER + "分片：按 2 台各 31 线程、每片 16G 切\n" + RESUME + REGISTERED), trees)
# 类型：超过 2 小时而三行都没写
for typ in ["experiment-designer", "experiment-runner", "implementation-writer", "investigator", "tooling-writer", "mutation-triage", "three-way-attack",
            "clerk", "sweep", "kb-scribe", "three-way-forward", "three-way-defense", "prior-art", "three-way-materials"]:
    c.both("R8", f"S18-over2h-nothing-else-{typ}", feed(typ, OVER), trees)
    c.both("R8", f"S18-no-wall-clock-{typ}", lambda tree, t=typ: (lambda r: (r[0], ",".join(r[1]) or "-"))(c.check_prompt(tree, t, c.prompt(t, "线程上限：31\n", wall=""), workdir=W)), trees)
