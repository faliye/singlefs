#!/usr/bin/env python3
"""Z1：⑮ 线程上限的喂法，经 runner-dispatch-guard.sh --check-prompt。python3 z1_cases.py <树> [-v]
每例打 例名 / 类型 / 线程上限（⑮）这一项是否在拒绝项里；别的项（定义要的输入等）不看"""
import os, re, subprocess, sys, tempfile
tree = os.path.abspath(sys.argv[1])
HEAD = ("重型测试：不跑\n开工先读 `.claude/agent-common.md`\n草稿目录：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/\n"
        "报告：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/r.md\n预计挂钟：2 小时内\n这一段回答的岔路：岔路 1\n变异表 crates/mutations.tsv。\n要动的 crates 文件：crates/x.rs\n")
NINE = ["implementation-writer", "investigator", "tooling-writer", "experiment-runner", "mutation-triage", "three-way-attack", "clerk", "three-way-forward", "three-way-defense"]
INLINE = "```cargo test -p singlefs-harness --lib``` 是上一次的起法，照它起。\n"
cases = []
for t in NINE:
    cases += [(f"九类-缺行也没有已判行-{t}", t, ""), (f"九类-只有已判行-{t}", t, "线程上限已判：这件活只读不起重活\n"), (f"九类-已判行没写理由-{t}", t, "线程上限已判：\n"),
              (f"九类-N 等于登记值-{t}", t, "线程上限：31\n")]
cases += [
 ("别处写了别的线程数", "clerk", "线程上限：31\n请用 8 线程跑，KS_THREADS=4\n"),
 ("代码块里的 16 加块外的 31", "clerk", "```\n线程上限：16\n```\n线程上限：31\n"),
 ("代码块里的已判行顶不了缺行", "clerk", "```\n线程上限已判：这件活只读不起重活\n```\n"),
 ("波浪线围栏里的 16", "clerk", "~~~\n线程上限：16\n~~~\n线程上限：31\n"),
 ("列表项里的围栏里的 16", "clerk", "- ```\n  线程上限：16\n  ```\n线程上限：31\n"),
 ("引用块里的围栏里的 16", "clerk", "> ```\n> 线程上限：16\n> ```\n线程上限：31\n"),
 ("没闭合的围栏后面的 16", "clerk", "线程上限：31\n```\n线程上限：16\n"),
 ("缩进 4 格的代码块里的 16（前有空行）", "clerk", "线程上限：31\n\n    线程上限：16\n"),
 ("行首行内三反引号后面的 16", "clerk", INLINE + "线程上限：16\n线程上限：31\n"),
 ("行内三反引号后真代码块里的 16", "clerk", "线程上限：31\n" + INLINE + "```\n线程上限：16\n```\n"),
 ("开头围栏缩进 4 格不是围栏", "clerk", "线程上限：31\n    ```\n线程上限：16\n    ```\n"),
 ("开头围栏用制表符缩进不是围栏", "clerk", "线程上限：31\n\t```\n线程上限：16\n\t```\n"),
 ("围栏闭合行缩进 4 格", "clerk", "线程上限：31\n```\n线程上限：16\n    ```\n"),
 ("四个反引号围栏里夹三个反引号", "clerk", "线程上限：31\n````\n线程上限：16\n```\n线程上限：16\n````\n"),
 ("围栏信息串里有反引号", "clerk", "线程上限：31\n```a`b\n线程上限：16\n```\n"),
 ("普通正文行也以线程上限冒号起头", "clerk", "线程上限：31\n线程上限：见上，双机时两台各取这一个数\n"),
 ("说明行不以线程上限冒号起头", "clerk", "线程上限：31\n本机，双机时两台各取这一个数（31 与 31）\n"),
 ("旧写法线程上限说明冒号另起一行", "clerk", "线程上限：31\n线程上限说明：本机，双机时两台各取这一个数（31 与 31）\n"),
 ("N 不等于登记值而写了已判行", "clerk", "线程上限：8\n线程上限已判：用户明说让出 CPU\n"),
 ("N 不等于登记值而已判行在代码块里", "clerk", "线程上限：8\n```\n线程上限已判：用户明说让出 CPU\n```\n"),
]
for name, typ, body in cases:
    fd, path = tempfile.mkstemp(suffix=".md", dir=os.path.dirname(tree))
    os.write(fd, (HEAD + body).encode()); os.close(fd)
    r = subprocess.run(["bash", os.path.join(tree, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", typ, "--root", tree], capture_output=True, text=True)
    os.unlink(path)
    out = r.stdout + r.stderr
    m = re.search(r"会拒 \d+ 处（([^）]*)）", out)
    names = m.group(1) if m else ""
    print(f"{name}\t{typ}\t线程上限被拒={'是' if '线程上限' in names.split('、') else '否'}")
