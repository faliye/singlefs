#!/usr/bin/env python3
"""S5：叫停登记出处的日常改动矩阵。用法：s5_drift_matrix.py <名>=<树根> ... --work <新目录>
      s5_drift_matrix.py --emit-gate-trees <新目录>   不判，把四个有代表性的例子各建成一棵小树（给主 agent 跑门禁 abandoned-rounds 格用）
每例造一棵小树（records/log.md 一份、abandoned-rounds.tsv 一行、_<轮>-body.md 空桩），按『改动』改记录，
再用各树自己的 .claude/hooks/lib_abandoned_rounds.py 的命令行判（派发闸 ⑥ 与门禁 abandoned-rounds 格共用这一份库）。
打印：例名 / 树名 / 退出码 / 第一条结论。"""
import argparse, os, shutil, subprocess, sys

parser = argparse.ArgumentParser()
parser.add_argument("roots", nargs="*")
parser.add_argument("--work")
parser.add_argument("--emit-gate-trees")
arguments = parser.parse_args()
ROUND = "zz-halt-r1"
LAUNCH = f"- 开轮 {ROUND}：用户说「先把这件事做成三条腿的零轮形态」，快照在工作根下。"
STOP = f"- 用户「这一轮先停掉，不写判决了」。去向：{ROUND} 两条腿当场停掉（没交回、不写判决）"
OTHER = f"- 补记：{ROUND} 的叫停登记已收，出处抄的是「去向：{ROUND} 两条腿当场停掉（没交回、不写判决）」。"
OTHER_QUOTE = f"- 另记：{ROUND} 的第二批腿由用户「换一批腿重跑，这一批别管了」接手。"
BASE = ["# 调度记录", "", "## 2026-10-12", "", LAUNCH, STOP, "- 别的活照常。", ""]
EXCERPT = f"去向：{ROUND} 两条腿当场停掉"

def row(line, excerpt=EXCERPT, form="第"):
    src = f"records/log.md 第 {line} 行〔{excerpt}〕" if excerpt is not None else f"records/log.md 第 {line} 行"
    return f"{ROUND}\t2026-10-12\t用户叫停（出处 {src}）\t不写判决\n"

def insert_before(lines, n=1):
    return lines[:2] + [f"### 新插的一节{k}" for k in range(n)] + lines[2:]

# (例名, 记录行 -> 记录行, 登记行写的行号, 原话)
cases = [
    ("基线", lambda l: l, 6, EXCERPT),
    ("漂移：在前面插一行（行号对不上，原话唯一）", lambda l: insert_before(l), 6, EXCERPT),
    ("漂移：在前面插三行", lambda l: insert_before(l, 3), 6, EXCERPT),
    ("重复：后面追加一行抄了原话的补记（提示行仍是原行）", lambda l: l[:-1] + [OTHER] + l[-1:], 6, EXCERPT),
    ("重复：补记插在原行前面（提示行落在补记上？命中两处，提示 6 在其中）", lambda l: l[:5] + [OTHER] + l[5:], 6, EXCERPT),
    ("重复：补记插在原行前面且行号提示落在原行新位置之外", lambda l: l[:5] + [OTHER] + l[5:], 5, EXCERPT),
    ("重复：整行逐字重复，原话抄成整行", lambda l: l[:6] + [STOP] + l[6:], 6, STOP.lstrip("- ")),
    ("改字：原话里改了字（叫停了->叫停过），提示行不动", lambda l: [x.replace("当场停掉", "当场停过") for x in l], 6, EXCERPT),
    ("改字：术语改名换了原话里的词，提示行不动", lambda l: [x.replace("两条腿", "两条评审腿") for x in l], 6, EXCERPT),
    ("改字+漂移：插一行再改字（提示行落在别的行）", lambda l: [x.replace("当场停掉", "当场停过") for x in insert_before(l)], 6, EXCERPT),
    ("重复+漂移：插一行再在后面追加补记（命中两处，提示不在其中）", lambda l: insert_before(l[:-1] + [OTHER] + l[-1:]), 6, EXCERPT),
    ("折行：把原行折成两行", lambda l: l[:5] + [STOP.replace("。去向", "。\n去向")] + l[6:], 6, EXCERPT),
    ("删行：原行被删", lambda l: l[:5] + l[6:], 6, EXCERPT),
    ("弱点：改字后提示行恰好是另一句带轮名与「」的话（开轮行）", lambda l: [x.replace("当场停掉", "当场停过") for x in l], 5, EXCERPT),
    ("弱点：原话照旧唯一，行号提示写成开轮行（原话唯一时不看提示）", lambda l: l, 5, EXCERPT),
    ("弱点：同一份记录里另有一句带轮名与「」的话，原话写成瞎编的", lambda l: l[:5] + [OTHER_QUOTE] + l[5:], 6, "完全不在记录里的一段话，写它没有人核对"),
    ("弱点：原话瞎编，提示行是开轮行", lambda l: l, 5, "完全不在记录里的一段话，写它没有人核对"),
    ("对照：原话抄成另一句带轮名与「」的话（它在记录里恰好一次），行号写叫停那一行", lambda l: l[:5] + [OTHER_QUOTE] + l[5:], 7, OTHER_QUOTE.lstrip("- ")),
    ("无原话：只写行号", lambda l: l, 6, None),
]
def build(tree, mutate, hint, excerpt):
    shutil.rmtree(tree, ignore_errors=True)
    os.makedirs(os.path.join(tree, "records")); os.makedirs(os.path.join(tree, "research/prompts"))
    open(os.path.join(tree, f"research/prompts/_{ROUND}-body.md"), "w").close()
    open(os.path.join(tree, "records/log.md"), "w", encoding="utf-8").write("\n".join(mutate(list(BASE))))
    open(os.path.join(tree, "research/prompts/abandoned-rounds.tsv"), "w", encoding="utf-8").write(row(hint, excerpt))

if arguments.emit_gate_trees:
    wanted = {"重复：后面追加一行抄了原话的补记（提示行仍是原行）": "append-note", "改字：术语改名换了原话里的词，提示行不动": "reworded",
              "改字+漂移：插一行再改字（提示行落在别的行）": "insert-then-reword", "重复+漂移：插一行再在后面追加补记（命中两处，提示不在其中）": "insert-then-append"}
    for name, mutate, hint, excerpt in cases:
        if name in wanted:
            build(os.path.join(arguments.emit_gate_trees, wanted[name]), mutate, hint, excerpt)
    print("建好：" + " ".join(sorted(wanted.values())))
    sys.exit(0)
os.makedirs(arguments.work, exist_ok=True)
for name, mutate, hint, excerpt in cases:
    for entry in arguments.roots:
        label, root = entry.split("=", 1)
        tree = os.path.join(arguments.work, f"{label}-{abs(hash(name)) % 100000}")
        build(tree, mutate, hint, excerpt)
        done = subprocess.run(["python3", "-B", os.path.join(root, ".claude/hooks/lib_abandoned_rounds.py"), tree], capture_output=True, text=True,
                              env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        first = (done.stdout + done.stderr).strip().replace("\n", " ")
        print(f"{name}\t{label}\t退出={done.returncode}\t{first[:170]}")
        shutil.rmtree(tree, ignore_errors=True)
