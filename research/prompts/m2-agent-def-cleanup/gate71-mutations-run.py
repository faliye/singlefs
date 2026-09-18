import os, subprocess, sys, tempfile, shutil
G, FX, D = sys.argv[1], sys.argv[2], sys.argv[3]
src = open(G, encoding="utf-8").read()
mutations = [
 ("①关：标题不查历史", 'if heading and ("历史" in heading.group(1) or "变更史" in heading.group(1)):', 'if False:', "red"),
 ("②关：日期行一律算指路", '            elif POINTER.search(line):\n                dated_with_pointer += 1', '            elif True:\n                dated_with_pointer += 1', "red"),
 ("②放宽：只提目录也算指路", 'POINTER = re.compile(r"(?:records|\\.claude/kb)/[^\\s`\'\\"）)]+")', 'POINTER = re.compile(r"(?:records|\\.claude/kb)/")', "red"),
 ("②放宽：rules 与 sop 路径也算指路", 'POINTER = re.compile(r"(?:records|\\.claude/kb)/[^\\s`\'\\"）)]+")', 'POINTER = re.compile(r"(?:records/|\\.claude/kb/|\\.claude/rules/|\\.claude/singlefs-ai-sop/)")', "red"),
 ("②收紧：引号里的日期也算", '            if not DATE.search(without_quoted(line)):', '            if not DATE.search(line):', "green"),
 ("②收紧：只剥一层引号", '    while previous != text:\n        previous, text = text, QUOTED.sub("", text)', '    previous, text = text, QUOTED.sub("", text)', "green"),
 ("③关：段落头不判", '        opener = paragraph_opener(lines[index])\n        if opener:\n            explanatory_paragraphs', '        opener = None\n        if opener:\n            explanatory_paragraphs', "red"),
 ("④关：半句正则不判", '            for match in HALF_SENTENCE.finditer(line):', '            for match in []:', "red"),
 ("④关：段落中间行不判", '        if not breaks_paragraph[index] and not starts_paragraph[index]:\n            opener = paragraph_opener(line)', '        if False:\n            opener = paragraph_opener(line)', "red"),
 ("③放宽：标签词不要分隔符", '            if rest == "" or rest[0] in PARAGRAPH_LABEL_DELIMITERS:', '            if True:', "green"),
 ("④放宽：顿号也算分句", 'r"(?:[（(，,；;。]|——)', 'r"(?:[（(，,；;。、]|——)', "green"),
 ("④放宽：表格行也判", '        if not breaks_paragraph[index]:\n            for match in HALF_SENTENCE', '        if not fenced[index]:\n            for match in HALF_SENTENCE', "green"),
 ("放宽：围栏也判", '        if fenced[index]:\n            continue\n        scanned_lines', '        if False:\n            continue\n        scanned_lines', "green"),
]
def run(kind, gate_text):
    work = tempfile.mkdtemp(dir=D)
    shutil.copytree(os.path.join(FX, kind), work, dirs_exist_ok=True)
    subprocess.run(["bash", "setup.sh"], cwd=work, check=True, capture_output=True)
    gate = os.path.join(work, "_gate.sh"); open(gate, "w", encoding="utf-8").write(gate_text)
    out = subprocess.run(["bash", gate, work], cwd=work, capture_output=True, text=True)
    expect = open(os.path.join(FX, kind, "expect"), encoding="utf-8").read().splitlines()
    want_exit = int([l for l in expect if l.startswith("exit=")][0][5:])
    misses = [l[5:] for l in expect if l.startswith("want=") and l[5:] not in out.stdout + out.stderr]
    return out.returncode == want_exit and not misses, out.returncode, misses
ok, rc, miss = run("red", src); print(f"原脚本 red   判得对={ok} rc={rc}")
ok, rc, miss = run("green", src); print(f"原脚本 green 判得对={ok} rc={rc}")
for name, old, new, kind in mutations:
    assert src.count(old) == 1, f"锚点不唯一：{name} 命中 {src.count(old)} 次"
    ok, rc, miss = run(kind, src.replace(old, new, 1))
    print(f"{'✓ 被抓' if not ok else '✗ 没红'}  {name:<26} 样本={kind:<5} rc={rc} 缺的 want：{miss[:2]}")
