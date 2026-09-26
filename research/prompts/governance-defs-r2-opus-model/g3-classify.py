#!/usr/bin/env python3
"""用法：python3 g3-classify.py <仓根> <合成输出目录>
从仓里今天的 59 号与 mutate.sh 原样抽出「一条变异的输出判成哪一档」那一段（按锚点行抽，不抄），
把同一份合成的 cargo test 输出分别喂给两边，打印各自判成哪一档、判不判整轮失败。"""
import os, re, subprocess, sys

root, outputs = sys.argv[1], sys.argv[2]
stage = open(os.path.join(root, ".claude/gate.d/59-crates-mutation-replay.sh"), encoding="utf-8").read().splitlines()
start = next(i for i, l in enumerate(stage) if l.startswith("    # 包装报的几种结局先判"))
end = next(i for i, l in enumerate(stage) if i > start and "点名的测试 {expected} 没跑到" in l and l.lstrip().startswith("return"))
constants = [l for l in stage if re.match(r"^(MEMORY_CAP_HIT_EXIT|MEMORY_CAP_UNAVAILABLE_EXIT|MEMORY_ADMISSION_REFUSED_EXIT|TIME_LIMIT_HIT_EXIT|SLICE_TOTAL_HIT_EXIT) = ", l)]
body = "\n".join(constants) + "\n\ndef judge59(run, output, tail, table, line_number, name, expected, timeout_seconds, memory_max):\n" + "\n".join(stage[start:end + 1]) + "\n"
namespace = {"re": re}
exec(body, namespace)
FAILING_59 = {"invalid", "failure", "memory", "timeout", "squeezed", "refused", "unavailable"}  # 59 号 `if invalid or failures or …: sys.exit(1)` 那一行

mutate = open(os.path.join(root, "research/scripts/mutate.sh"), encoding="utf-8").read().splitlines()
m_start = next(i for i, l in enumerate(mutate) if l.startswith("  if ! grep -q '^running [0-9]* test' <<<\"$out\"; then"))
m_end = next(i for i, l in enumerate(mutate) if i > m_start and l.startswith('    record_verdict "$row_index" caught 0 "✅ [$name] 红：$red" ""'))
mutate_snippet = "\n".join(mutate[m_start:m_end + 2])  # 连同收尾的 fi

class Run:
    def __init__(self, returncode): self.returncode = returncode

expected = "inode_and_extent_lookups_from_the_root_read_the_first_file_back"
for file_name in sorted(os.listdir(outputs)):
    output = open(os.path.join(outputs, file_name), encoding="utf-8").read()
    returncode = 101
    _, verdict59, text59 = namespace["judge59"](Run(returncode), output, "", "crates/mutations.tsv", 7, "合成变异", expected, 120, "4G")
    script = ("record_verdict() { printf '%s %s\\n' \"$2\" \"$3\"; }\n" "judge() {\n  local row_index=0 name=合成变异 out sig red\n  out=\"$(cat \"$1\")\"\n"
              + mutate_snippet + "\n}\njudge \"$1\"\n")
    verdict_mutate = subprocess.run(["bash", "-c", script, "_", os.path.join(outputs, file_name)], capture_output=True, text=True).stdout.strip()
    category, fails = verdict_mutate.split()
    print(f"{file_name}\t59 号：{verdict59}（整道判红：{'是' if verdict59 in FAILING_59 else '否'}）\t{text59.splitlines()[0][:60]}\tmutate.sh：{category}（判整轮失败：{'是' if fails == '1' else '否'}）")
