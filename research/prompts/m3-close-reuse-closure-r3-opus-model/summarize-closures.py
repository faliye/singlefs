"""每个入口、每棵树一行：单元数（不含 via、refused、key 行）、拒算（形态:个数）、去掉 NOT_JUDGING_FILES 两份之后的单元行摘要（前 12 位，当键的替身：
键由这些单元的指纹加工具链等定，工具链各树相同），与对照树比多了几个、少了几个单元（按种类、限定名、文件比）。
用法：summarize-closures.py <闭包输出目录> <入口清单> <对照树> <树…>"""
import collections
import hashlib
import sys

out, listing, reference, trees = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4:]
NOT_JUDGING = ("scripts/crash/lib_verdict_library.py", "scripts/gate/lib_function_units.py")
entries = [line.strip() for line in open(listing, encoding="utf-8") if line.strip()]


def read(tree, index):
    units, refusals = {}, collections.Counter()
    for line in open(f"{out}/c-{tree}-{index}.txt", encoding="utf-8"):
        fields = line.rstrip("\n").split("\t")
        if fields[0] == "refused":
            refusals[fields[1]] += 1
        elif fields[0] not in ("via", "key") and len(fields) >= 4:
            units[(fields[0], fields[1], fields[2].rsplit(":", 1)[0])] = fields[3]
    judging = sorted(f"{k[0]}\t{k[1]}\t{k[2]}\t{v}" for k, v in units.items() if k[2] not in NOT_JUDGING)
    digest = hashlib.sha256("\n".join(judging).encode()).hexdigest()[:12]
    return units, refusals, digest


for index, entry in enumerate(entries):
    base_units, base_refusals, base_digest = read(reference, index)
    for tree in trees:
        units, refusals, digest = read(tree, index)
        added = len(set(units) - set(base_units))
        dropped = len(set(base_units) - set(units))
        text = ";".join(f"{form}:{count}" for form, count in sorted(refusals.items())) or "none"
        print(f"name=closure_summary index={index} entry={entry} tree={tree} units={len(units)} refusals={text} "
              f"judging_digest={digest} vs_{reference}=+{added}/-{dropped} digest_same_as_{reference}={'yes' if digest == base_digest else 'no'}")
