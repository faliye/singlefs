#!/usr/bin/env python3
"""把探针产物里「一次发布一行 + 第九项一行」配成 Markdown 表（报告里的逐次发布表由它生成，不手抄）。

用法：python3 extract-tables.py 产物.txt
每一行状态行（含 acct_alloc=）与紧随其后的 ninth 行配对；没有 ninth 行的状态行（抬 F、回退那几行之前的）单独出一行，第九项那几列写「—」。
"""
import re
import sys

path = sys.argv[1]
lines = open(path, encoding="utf-8").read().splitlines()


def field(text, name):
    match = re.search(rf"(?:^| ){re.escape(name)}=(\S+)", text)
    return match.group(1) if match else "—"


def slot_count(compact):
    """「[50176-50177,50182]」这种写法里的槽数；「—」照原样。"""
    if compact == "—":
        return compact
    total = 0
    for part in compact.strip("[]").split(","):
        if not part:
            continue
        low, _, high = part.partition("-")
        total += (int(high) if high else int(low)) - int(low) + 1
    return str(total)


def verdict(text):
    tail = text.rsplit(" | ", 1)[-1]
    if tail == "GREEN":
        return "绿"
    names = sorted(set(re.findall(r"(G8p?|I-[0-9.]+)\[", tail)))
    return "红 " + ",".join(names)


rows = []
index = 0
while index < len(lines):
    text = lines[index]
    if "acct_alloc=" in text and " | " in text:
        ninth = lines[index + 1] if index + 1 < len(lines) and " ninth " in lines[index + 1] else None
        tag = text.split(" txg=")[0]
        state = {
            "event": tag.split(" ", 1)[1] if " " in tag else tag,
            "txg": field(text, "txg"),
            "inst": field(text, "inst"),
            "F": f"{field(text, 'F_root')}/{field(text, 'F_eff')}",
            "hit": field(text, "hit_abandoned"),
            "verdict": verdict(text),
        }
        if ninth:
            original = re.search(r"orig(\S*)", ninth)
            state.update(
                candidates=field(ninth, "candidates"),
                abandoned=field(ninth, "abandoned"),
                below=field(ninth, "below_F"),
                g5=field(ninth, "G5_isolated").split("[")[0],
                set_reading=field(ninth, "set_reading"),
                count_reading=field(ninth, "count_reading"),
                narrow=field(ninth, "narrow_by_valid"),
                original=original.group(1).split("=", 1)[1] if original and "=" in original.group(1) else "—",
                stale=slot_count(field(ninth, "stale_isolated")),
                missing=field(ninth, "missing_isolation"),
                lowest=field(ninth, "lowest_user_slot"),
            )
            index += 1
        rows.append(state)
    index += 1

columns = [
    ("event", "事件"), ("txg", "txg"), ("inst", "实例"), ("F", "根带的 F / F_生效"), ("candidates", "候选集 txg"), ("abandoned", "环里被抛弃根 txg"),
    ("below", "F 之下的有效根"), ("g5", "G5 隔离数"), ("set_reading", "改式·集合差"), ("count_reading", "改式·数值差"), ("original", "原式（被抛弃最新根 − R_old）"),
    ("narrow", "窄读（有效只按实例表）"), ("stale", "陈旧隔离槽数"), ("missing", "该隔离而没隔离"), ("lowest", "最低可发用户槽"), ("hit", "这次写撞被抛弃根的槽"), ("verdict", "checker"),
]
print("| " + " | ".join(title for _, title in columns) + " |")
print("|" + "---|" * len(columns))
for row in rows:
    print("| " + " | ".join(str(row.get(key, "—")) for key, _ in columns) + " |")
