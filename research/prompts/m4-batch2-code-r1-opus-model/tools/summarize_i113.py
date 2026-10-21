"""汇总 attack_i113_reasons 各批日志：每批按判定类（成立 / 违例 / 不适用的理由前缀）合计，另报上一步是 CreateInode（只在删完之后才调）的状态里成立几个。"""
import re
import sys

REASON_CLASSES = [
    ("谱系树里有记录", "不适用·C615 谱系"),
    ("退役树里有记录", "不适用·C768 退役"),
    ("落在它那条 inode 记录的 size 之外", "不适用·C827 变短"),
    ("墓碑压着的对象还有 inode 记录", "不适用·C827 删除中"),
]
for path in sys.argv[1:]:
    summary = ""
    totals = {}
    create_inode = {}
    with open(path, encoding="utf-8") as log:
        for line in log:
            if line.startswith("ATTACK_SUMMARY"):
                summary = line.strip()
            match = re.match(r"ATTACK_I113 (\d+) (.*)｜上一步 (\S+)", line.strip())
            if not match:
                continue
            count, verdict, previous = int(match.group(1)), match.group(2), match.group(3)
            label = verdict
            if verdict.startswith("不适用"):
                label = "不适用·其他：" + verdict
                for needle, name in REASON_CLASSES:
                    if needle in verdict:
                        label = name
            totals[label] = totals.get(label, 0) + count
            if previous == "CreateInode":
                create_inode[label] = create_inode.get(label, 0) + count
    states = sum(totals.values())
    print(f"{path.rsplit('/', 1)[-1]}  {summary}")
    print(f"  状态 {states}：" + "、".join(f"{label} {count}" for label, count in sorted(totals.items())))
    print("  上一步 CreateInode 的状态：" + ("、".join(f"{label} {count}" for label, count in sorted(create_inode.items())) or "无"))
