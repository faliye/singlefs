"""映射根变体全量里「中间」那一段（序号 20 到 15999）单独汇总：第二轮没跑的就是这一段。
用法：python3 middle.py <第二轮 aggregate.py> <这一轮辩方腿逐状态输出…>
打出第二轮汇总脚本对这一段的汇总（候选 mapping_root_only，例子序号去掉）。"""
import re, subprocess, sys, tempfile

aggregate, *paths = sys.argv[1:]
kept = []
for path in paths:
    for line in open(path, encoding="utf-8"):
        if line.startswith("name=c834_history"):
            kept.append(line)
        elif line.startswith("name=c834_state") and "candidate=mapping_root_only" in line:
            ordinal = int(re.search(r" ordinal=(\d+)", line).group(1))
            if 20 <= ordinal < 16000:
                kept.append(line)
with tempfile.NamedTemporaryFile("w", suffix=".out", delete=False, encoding="utf-8") as handle:
    handle.writelines(kept)
    name = handle.name
for line in subprocess.run([sys.executable, aggregate, name], capture_output=True, text=True, check=True).stdout.splitlines():
    if "candidate=mapping_root_only" in line:
        print(re.sub(r" example_ordinal=\d+", "", line))
