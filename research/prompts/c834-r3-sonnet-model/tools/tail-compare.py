"""映射根变体在第二轮只跑了头尾两段（序号 0–19 与 16000 起）。把这一轮全量的逐状态输出里这两段挑出来，用第二轮的汇总脚本重汇总，
和第二轮入库的 aggregate-mapping-tail.txt 逐行比（例子序号去掉）：头尾两段的数对不对得上第二轮报的。
用法：python3 tail-compare.py <第二轮 aggregate.py> <第二轮 aggregate-mapping-tail.txt> <这一轮辩方腿逐状态输出…>
打出：name=r3_compare tail same|differs rows_mine=N rows_ref=N；不同的行逐行列出。"""
import re, subprocess, sys, tempfile

aggregate, reference, *paths = sys.argv[1:]
kept = []
for path in paths:
    for line in open(path, encoding="utf-8"):
        if line.startswith("name=c834_history"):
            kept.append(line)
        elif line.startswith("name=c834_state") and "candidate=mapping_root_only" in line:
            ordinal = int(re.search(r" ordinal=(\d+)", line).group(1))
            if ordinal < 20 or ordinal >= 16000:
                kept.append(line)
with tempfile.NamedTemporaryFile("w", suffix=".out", delete=False, encoding="utf-8") as handle:
    handle.writelines(kept)
    name = handle.name
mine = subprocess.run([sys.executable, aggregate, name], capture_output=True, text=True, check=True).stdout.splitlines()
strip = lambda line: re.sub(r" example_ordinal=\d+", "", line)
mine = [strip(line) for line in mine if "candidate=mapping_root_only" in line]
ref = [strip(line.rstrip("\n")) for line in open(reference, encoding="utf-8") if "candidate=mapping_root_only" in line]
only_mine, only_ref = sorted(set(mine) - set(ref)), sorted(set(ref) - set(mine))
same = not only_mine and not only_ref and len(mine) == len(ref)
print(f"name=r3_compare tail {'same' if same else 'differs'} rows_mine={len(mine)} rows_ref={len(ref)}")
for line in only_mine:
    print("只在这一轮：" + line)
for line in only_ref:
    print("只在第二轮：" + line)
sys.exit(0 if same else 1)
