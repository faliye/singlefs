#!/usr/bin/env python3
"""攻方 J2-b：门禁 gate-verdict 升档复核那一段的判红分支逐个删掉（每种一份树拷贝，只改 .claude/gate.d/checker-tier-broad-mutation.sh 一处），
看 36 份样本翻不翻。用法：j2b-gate-mutants.py <原样树根> <放拷贝的目录>；每行打「名字<TAB>删的是哪一支」。旧串在文件里必须恰好命中一次。"""
import os, shutil, subprocess, sys
base, into = sys.argv[1:3]
STAGE = ".claude/gate.d/checker-tier-broad-mutation.sh"
MUTANTS = [
    ("g01-label", "标签认不出", 'if label not in multiples:\n        return', 'if False:\n        return'),
    ("g02-baseline-missing", "那一版没记基线", '    if baseline_text is None:\n        found.append', '    if False:\n        found.append'),
    ("g03-baseline-differs", "登记的基线与那一版记的不同", '    elif baseline_text != row["baseline"]:', '    elif False:'),
    ("g05-not-larger", "升档那趟没比原来的大", '        elif escalated <= first:', '        elif False:'),
    ("g06m-memory-multiple", "撞顶那一边的倍数门槛", 'memory_label: driver.ESCALATION_MEMORY_BASELINE_MULTIPLE}', 'memory_label: 0}'),
    ("g07-trailer", "日志末尾没有驱动那一行", '        if trailer not in text:', '        if False:'),
    ("g07b-trailer-prefix", "驱动那一行只核版本键（不核序号、步名、哪一趟、限时、退出码）",
     '        trailer = f"BROAD_MUTATION_EVIDENCE version={row[\'version\']} sequence={row[\'sequence\']} step={row[\'step\']} attempt={attempt} limit={limit} exit={hit_exit}"',
     '        trailer = f"BROAD_MUTATION_EVIDENCE version={row[\'version\']}"'),
    ("g08-wrapper", "日志里没有包装那一句", '        if wrapper not in text:', '        if False:'),
    ("g09-columns", "登记行段数不对", '            if len(fields) != len(columns):', '            if len(fields) < 10:'),
    ("g10-escalated-field", "有 ESCALATED 而判定行没有 escalated=", '    elif "escalated" not in gate and escalated_lines:', '    elif False:'),
    ("g11-sequence-only", "ESCALATED 对登记行只比序号（不比标签、变异名）",
     '    if len(fields) != 4 or (fields[1], fields[2], fields[3]) not in verified:', '    if len(fields) != 4 or fields[1] not in {entry[0] for entry in verified}:'),
    ("g14-anywhere", "日志不在 escalation/ 下也认（在别处、在别的版本目录里）",
     '        if not path.startswith(os.path.join(version_directory, evidence_directory) + os.sep) or not os.path.isfile(path):',
     '        if not os.path.isfile(path):'),
]
for name, what, old, new in MUTANTS:
    target = os.path.join(into, name)
    shutil.rmtree(target, ignore_errors=True)
    subprocess.run(["rsync", "-a", base.rstrip("/") + "/", target + "/"], check=True)
    path = os.path.join(target, STAGE)
    text = open(path, encoding="utf-8").read()
    if text.count(old) != 1:
        sys.exit(f"{name}：旧串命中 {text.count(old)} 次，不是 1")
    open(path, "w", encoding="utf-8").write(text.replace(old, new))
    print(f"{name}\t{what}")
