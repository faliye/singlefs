#!/usr/bin/env python3
"""S5：攻方第一轮写的弱点（行号提示那一行恰好是另一句带这一轮名字与「」原话的话）在真记录里造不造得出来。
一、数：<树>/records/*.md 里『恰有一个 <名>-r<数字> 形轮名、且有一句去掉空白后至少 6 个字的「」』的行（库对出处那一行的全部要求里除『别的轮名』外能机械满足的），
   以及同一份记录里同一个轮名有两行及以上这样的行的（文件, 轮名）对数。
二、造：取第一对，在临时树里复制那份真记录，登记一行叫停（原话写成记录里没有的一段话，行号写成这对里的第一行），分别用 b9、b10 的库判。
用法：s5_real_records.py <b9 树> <b10 树> <工作目录>"""
import collections, glob, os, re, shutil, subprocess, sys
t9, t10, work = (os.path.abspath(p) for p in sys.argv[1:4]); os.makedirs(work, exist_ok=True)
name = re.compile(r"(?<![A-Za-z0-9_\-])[a-z0-9][a-z0-9-]*-r[0-9]+(?![A-Za-z0-9_\-])")
quote = re.compile(r"「([^」]*)」")
eligible = collections.defaultdict(list)
total = 0
files = sorted(glob.glob(os.path.join(t10, "records", "*.md")))
for f in files:
    for number, line in enumerate(open(f, encoding="utf-8").read().split("\n"), 1):
        names = set(name.findall(line))
        if len(names) == 1 and any(len(re.sub(r"\s+", "", q)) >= 6 for q in quote.findall(line)):
            total += 1; eligible[(os.path.basename(f), next(iter(names)))].append(number)
pairs = {k: v for k, v in eligible.items() if len(v) >= 2}
print(f"records 下 {len(files)} 份；满足条件的行 {total} 行，分布在 {len({k[0] for k in eligible})} 份文件；同一文件里同一轮名有两行及以上的 (文件, 轮名) 对 {len(pairs)} 对")
(file_name, round_name), lines = sorted(pairs.items())[0]
print(f"取第一对：{file_name} 里的 {round_name}，满足条件的行号 {lines[:6]}")
for label, tree in (("b9", t9), ("b10", t10)):
    sandbox = os.path.join(work, label); shutil.rmtree(sandbox, ignore_errors=True)
    os.makedirs(os.path.join(sandbox, "records")); os.makedirs(os.path.join(sandbox, "research/prompts"))
    shutil.copy(os.path.join(t10, "records", file_name), os.path.join(sandbox, "records", file_name))
    open(os.path.join(sandbox, f"research/prompts/_{round_name}-body.md"), "w").close()
    for kind, excerpt, hint in (("原话瞎编（记录里没有），行号写成那一对的第一行", "完全不在记录里的一段话，写它没有人核对", lines[0]),
                                ("原话瞎编，行号写成那一对的第二行", "完全不在记录里的一段话，写它没有人核对", lines[1])):
        row = f"{round_name}\t2026-10-12\t用户叫停（出处 records/{file_name} 第 {hint} 行〔{excerpt}〕）\t不写判决\n"
        open(os.path.join(sandbox, "research/prompts/abandoned-rounds.tsv"), "w", encoding="utf-8").write(row)
        done = subprocess.run(["python3", "-B", os.path.join(tree, ".claude/hooks/lib_abandoned_rounds.py"), sandbox], capture_output=True, text=True,
                              env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        first = (done.stdout + done.stderr).strip().replace("\n", " ")
        print(f"{label}\t{kind}\t退出={done.returncode}\t{first[:120]}")
