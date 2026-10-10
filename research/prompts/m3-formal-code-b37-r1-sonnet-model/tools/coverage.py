#!/usr/bin/env python3
"""B1 模型：只读源码文本，不编译。把 RunProfile 的判档谓词与 Criterion::is_judged_under 从源码里解析出来，
对测试里的三趟（r3 Direct、r5 Direct、r3 + Both）算每个 Criterion 行被哪几趟判、有没有一行三趟都不判。
用法：coverage.py <e165 装置源文件>"""
import re, sys
src = open(sys.argv[1], encoding="utf-8").read()

def body_of(fn_header_regex, start=0):
    m = re.compile(fn_header_regex).search(src, start)
    assert m, fn_header_regex
    i = src.index("{", m.end()); depth = 0
    for j in range(i, len(src)):
        if src[j] == "{": depth += 1
        elif src[j] == "}":
            depth -= 1
            if depth == 0: return src[i+1:j]
    raise SystemExit("unbalanced")

# CriteriaSet 变体
cs = re.search(r"enum CriteriaSet \{(.*?)\n\}", src, re.S).group(1)
criteria_sets = re.findall(r"^\s{4}([A-Za-z0-9]+),", cs, re.M)

def predicate(name):
    b = body_of(r"const fn %s\(self\) -> bool" % name)
    table = {}
    for arms, val in re.findall(r"((?:CriteriaSet::\w+\s*\|?\s*)+)=>\s*(true|false)", b):
        for v in re.findall(r"CriteriaSet::(\w+)", arms): table[v] = (val == "true")
    assert set(table) == set(criteria_sets), (name, table)
    return table
P = {n: predicate(n) for n in ["judges_j6qn", "judges_j7", "judges_j8", "splits_j6qn_by_i812"]}

# Criterion 变体
enum_block = re.search(r"enum Criterion \{(.*?)\n    \}\n\}", src, re.S).group(1)
criteria = re.findall(r"^\s+(\w+) =>", enum_block, re.M)

# is_judged_under 的臂
b = body_of(r"const fn is_judged_under\(self, run_profile: RunProfile\) -> bool")
arms = re.findall(r"((?:Self::\w+\s*\|?\s*)+)=>\s*(\{[^}]*\}|[^,]*?),?\s*(?=Self::|\}\s*\Z)", b, re.S)
judged = {}
for names, expr in arms:
    for n in re.findall(r"Self::(\w+)", names):
        judged[n] = expr.strip()
assert set(judged) == set(criteria), (set(criteria) ^ set(judged))

def holds(expr, prof):
    expr = expr.strip("{} \n")
    if expr == "true": return True
    m = re.fullmatch(r"run_profile\.(\w+)\(\)", expr)
    assert m, expr
    f = m.group(1)
    if f == "compares_the_two_record_tables": return prof["table"] == "Both"
    return P[f][prof["criteria"]]

profiles = {
    "r1":     dict(criteria="RegisteredBeforeR2", table="Direct"),
    "r2":     dict(criteria="WithJ6QnAndJ7", table="Direct"),
    "pass1 r3 Direct": dict(criteria="WithJ6QnAndJ7SplitByI812", table="Direct"),
    "pass2 r5 Direct": dict(criteria="WithJ6QnAndJ8SplitByI812", table="Direct"),
    "pass3 r3 Both":   dict(criteria="WithJ6QnAndJ7SplitByI812", table="Both"),
}
print("Criterion 行数:", len(criteria))
cover = {c: [] for c in criteria}
for name, prof in profiles.items():
    for c in criteria:
        if holds(judged[c], prof): cover[c].append(name)
uncovered = [c for c in criteria if not any(p.startswith("pass") for p in cover[c])]
for c in criteria:
    print(f"{c:48s} judged_by={cover[c]}")
print("三趟都不判的行:", uncovered)
p1 = {c for c in criteria if "pass1 r3 Direct" in cover[c]}
p3 = {c for c in criteria if "pass3 r3 Both" in cover[c]}
only3 = sorted(p3 - p1)
print("pass1 判的行数:", len(p1), " pass3 判的行数:", len(p3), " pass1 是 pass3 的子集:", p1 <= p3, " pass3 比 pass1 多判:", only3)
for k in ["pass1 r3 Direct", "pass2 r5 Direct", "pass3 r3 Both"]:
    only = [c for c in criteria if cover[c] and [p for p in cover[c] if p.startswith("pass")] == [k]]
    print(f"只有 {k} 判的行: {only}")

# ---- 变异推演：三趟换成别的档之后，第一条没有读数的行是哪条（断言信息「判据 X 没有对照的读数」按 Criterion::ALL 次序报第一条）
P3 = profiles["pass1 r3 Direct"]; P5 = profiles["pass2 r5 Direct"]; P3B = profiles["pass3 r3 Both"]; P2 = profiles["r2"]
scenarios = {
    "原样 [r3, r5, r3+Both]": [P3, P5, P3B],
    "旧测试 [r3]": [P3],
    "行 3383 [r3, r3, r3+Both]（r5 那趟改回 r3）": [P3, P3, P3B],
    "行 3407 [r3, r5, r3]（Both 改回 Direct）": [P3, P5, P3],
    "第一趟换成 r2 [r2, r5, r3+Both]": [P2, P5, P3B],
    "第一趟换成 r5 [r5, r5, r3+Both]": [P5, P5, P3B],
    "第一趟整个删掉 [r5, r3+Both]": [P5, P3B],
    "第三趟换成 r5+Both [r3, r5, r5+Both]": [P3, P5, dict(criteria=P5["criteria"], table="Both")],
}
print()
for name, passes in scenarios.items():
    missing = [c for c in criteria if not any(holds(judged[c], p) for p in passes)]
    print(f"{name:48s} 没有读数的行数={len(missing)} 第一条={missing[:1]} 全部={missing}")
