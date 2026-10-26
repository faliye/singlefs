#!/usr/bin/env python3
"""攻方腿 landing-audit-r1 的 K4 复算：盲判（blind-rework.md）与 rework.md 在 E171、E188 两条链上的主因一致数、可避免与否一致数。
两份报告都只读第三节 / 「每条链的逐次表」里的 markdown 表；主因按开头的类名归一（见 CAUSES），可避免按「Y 或 P」对「N」归成两类。
调度记录没写这两个数是怎么比出来的（没有命令、没有对照表），这里的归一是攻方腿自己定的，写在 CAUSES 里。
用法：blind_agreement.py <rework.md> <blind-rework.md>
"""
import re, sys

CAUSES = [("首派", "首派"), ("计划内分段", "计划内分段"), ("前一次产物有错", "产物有错"), ("前一次的产物有错", "产物有错"),
          ("规则或命名纪律变了", "规则变了"), ("前一次被截断", "被截断"), ("主 agent 改了问题或方向", "改方向"),
          ("主agent改了问题或方向", "改方向"), ("主 agent 改问题或方向", "改方向"), ("派发提示缺输入或写错", "提示错"), ("别的", "别的"), ("拿不准", "拿不准")]

def cause_of(text):
    text = text.strip()
    for prefix, name in CAUSES:
        if text.startswith(prefix):
            return name
    inherit = re.match(r"承\s*#?(\d+)", text)
    if inherit:
        return "承"
    return "认不出：" + text[:20]

def tables(path, chains, cause_column, avoid_column):
    rows, chain = {}, None
    for line in open(path, encoding="utf-8"):
        heading = re.match(r"^###\s+(E\d+)", line)
        if heading:
            chain = heading.group(1) if heading.group(1) in chains else None
            continue
        if chain and line.startswith("| ") and not line.startswith("|---"):
            cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
            if not cells[0].isdigit():
                continue
            rows[(chain, int(cells[0]))] = (cells[cause_column], cells[avoid_column])
    return rows

def avoid_class(text):
    text = text.strip()
    if text in ("Y", "P"):
        return "YP"
    if text == "N":
        return "N"
    return text or "-"

def main():
    rework = tables(sys.argv[1], {"E171", "E188"}, 5, 6)
    blind = tables(sys.argv[2], {"E171", "E188"}, 3, 4)
    print(f"rework.md 行 {len(rework)}，blind-rework.md 行 {len(blind)}，共有键 {len(set(rework) & set(blind))}")
    same_cause = same_avoid = compared = 0
    disagreements = []
    for key in sorted(set(rework) & set(blind)):
        r_cause, r_avoid = rework[key]
        b_cause, b_avoid = blind[key]
        if cause_of(r_cause) == "首派" or cause_of(b_cause) == "首派":
            continue
        compared += 1
        cause_equal = cause_of(r_cause) == cause_of(b_cause)
        avoid_equal = avoid_class(r_avoid) == avoid_class(b_avoid)
        same_cause += cause_equal
        same_avoid += avoid_equal
        if not cause_equal or not avoid_equal:
            disagreements.append(f"  {key[0]} #{key[1]}\trework：{cause_of(r_cause)} / {r_avoid}\t盲判：{cause_of(b_cause)} / {b_avoid}")
    print(f"续派 {compared} 次；主因一致 {same_cause}；可避免（Y 或 P 对 N）一致 {same_avoid}")
    print("对不上的：")
    print("\n".join(disagreements))

main()
