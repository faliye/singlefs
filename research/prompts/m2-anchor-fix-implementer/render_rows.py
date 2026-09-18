# 把 6 行的改前 / 改后、命中、变异与基线输出渲染成报告段落（每行一段），供分段追加进报告。
import os, re
draft = "/tmp/claude-1000/m2-anchor-fix"
root = "/home/fy5090/code/singlefs"
before = open(os.path.join(draft, "mutations.tsv.before"), encoding="utf-8").read().split("\n")
after = open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")
def unescape(text): return text.replace("\\n", "\n")
def clip(line):
    return line if len(line) <= 300 else line[:300] + f" …（这一行原长 {len(line)} 字符，报告里截到 300）"
def panic_block(output, test_name):
    block = re.search(r"^---- (\S+::)?" + re.escape(test_name) + r" stdout ----\n(.*?)(?=^---- |^failures:$|\Z)", output, re.M | re.S)
    return [clip(line) for line in block.group(2).strip().splitlines()] if block else ["（没找到这条测试的 stdout 段）"]
for line_number in (32, 37, 41, 42, 69, 107):
    old_fields = before[line_number - 1].split("\t")
    new_fields = after[line_number - 1].split("\t")
    source = open(os.path.join(root, new_fields[1]), encoding="utf-8").read()
    out = []
    out.append(f"### 第 {line_number} 行　{new_fields[0]}")
    out.append("")
    out.append(f"文件 `{new_fields[1]}`；第 5 段 `{new_fields[4]}`、第 6 段 `{new_fields[5]}` 都没动。")
    out.append("")
    out.append("改前原文（第 3 段，表里原样）：")
    out.append("```text"); out.append(old_fields[2]); out.append("```")
    out.append("改前替换文（第 4 段）：")
    out.append("```text"); out.append(old_fields[3]); out.append("```")
    out.append("改后原文（第 3 段）：")
    out.append("```text"); out.append(new_fields[2]); out.append("```")
    out.append("改后替换文（第 4 段）：")
    out.append("```text"); out.append(new_fields[3]); out.append("```")
    out.append(f"命中（\\n 当换行，数的是工作区今天的 `{new_fields[1]}`）：改前原文 {source.count(unescape(old_fields[2]))} 次，改后原文 {source.count(unescape(new_fields[2]))} 次。")
    out.append("")
    baseline = open(os.path.join(draft, "logs", f"baseline-row-{line_number}.log"), encoding="utf-8").read()
    mutated = open(os.path.join(draft, "logs", f"mutated-row-{line_number}-filtered.log"), encoding="utf-8").read()
    whole = open(os.path.join(draft, "logs", f"mutated-row-{line_number}-whole.log"), encoding="utf-8").read()
    out.append(f"不施加替换（基线），`cargo test --offline {new_fields[4]}` 原样输出行：")
    out.append("```text")
    out += re.findall(r"^test \S+ \.\.\. \S+$", baseline, re.M) + re.findall(r"^test result: .*$", baseline, re.M)
    out.append("```")
    out.append(f"施加替换，同一条命令，原样输出行（点名测试的结果行与它 stdout 段）：")
    out.append("```text")
    out += re.findall(r"^test \S+ \.\.\. \S+$", mutated, re.M) + re.findall(r"^test result: .*$", mutated, re.M)
    out += panic_block(mutated, new_fields[5])
    out.append("```")
    out.append("施加替换，跑整个测试二进制（去掉 `--` 之后的过滤串），红的测试与结果行：")
    out.append("```text")
    out += re.findall(r"^test \S+ \.\.\. FAILED$", whole, re.M) + re.findall(r"^test result: .*$", whole, re.M)
    out.append("```")
    open(os.path.join(draft, f"section-row-{line_number}.md"), "w", encoding="utf-8").write("\n".join(out) + "\n")
    print(line_number, len(out), "行")
