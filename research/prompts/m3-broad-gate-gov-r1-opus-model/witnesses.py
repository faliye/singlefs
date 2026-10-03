#!/usr/bin/env python3
"""攻方腿模型：给 matrix.py 报「没有样本盯着」的每一支造一份输入，看原样的格判红、删掉那一支的变异判什么。
变异仍判红的那一支是被别的分支兜住的（记「冗余」）；变异判绿的是「这一支没了就漏」的输入，可直接做成红样本。
另造一份原样的格就判绿的输入（汇总行与判定行的 total 不一致），那是缺一支、不是缺样本。
用法：witnesses.py <tree> <run-fixed 目录> <草稿目录>
  run-fixed 里 library-version-is-staged-tree-green、gate-verdict-green 两份要已经照 fix-green.py 改成今天的指纹（gate-verdict-green 不用改）。"""
import os, shutil, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cells
tree, fixed, scratch = sys.argv[1:4]
LIB, VERDICT = "library-version-is-staged-tree", "gate-verdict"

def copy(sample, name):
    target = os.path.join(scratch, "witness", name)
    shutil.rmtree(target, ignore_errors=True)
    shutil.copytree(os.path.join(fixed, sample), target, symlinks=True)
    return target

def git(root, *arguments, data=None):
    environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    return subprocess.run(["git", "-C", root, *arguments], input=data, capture_output=True, text=True, env=environment, check=True).stdout

def summary(root, lines, exit_code):
    with open(os.path.join(root, ".broad-sample", "summary.sh"), "w", encoding="utf-8") as handle:
        handle.write("#!/usr/bin/env bash\ncat <<'EOF'\n" + "\n".join(lines) + "\nEOF\nexit " + str(exit_code) + "\n")

def first_line(text):
    return next((line.strip() for line in text.splitlines() if "✗" in line or "✓" in line), text.strip().splitlines()[-1] if text.strip() else "")

def judge(name, cell, root, mutant):
    code, text = cells.run(tree, cell, root)
    row = f"WITNESS {name}\t原样 exit={code}「{first_line(text)}」"
    if mutant:
        mutant_code, mutant_text = cells.run(tree, cell, root, mutant=mutant)
        kind = "冗余（别的分支兜住）" if mutant_code != 0 else "这一支没了就漏"
        row += f"\t删掉 {mutant} exit={mutant_code}「{first_line(mutant_text)}」\t{kind}"
    print(row)

FP = "970a802431217260"
HEAD_LINE = f"BROAD_MUTATION_SUMMARY fingerprint={FP} commit=0123456789ab total=6 caught=5 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 driver_error=0 not_run=0"

root = copy("library-version-is-staged-tree-green", "M1")
open(os.path.join(root, ".broad-sample", "home", "tree.source.pending"), "w").write("{}\n")
judge("M1 tree.source.pending 在", LIB, root, "M1-pending")

root = copy("library-version-is-staged-tree-green", "M2")
git(root, "rm", "-q", "--cached", "crates/sample/src/lib.rs")
judge("M2 暂存区删了 lib.rs", LIB, root, "M2-only-recorded")

root = copy("library-version-is-staged-tree-green", "M3")
shutil.rmtree(os.path.join(root, ".broad-sample"))
shutil.copytree(os.path.join(scratch, "run", "library-version-is-staged-tree-green", ".broad-sample"), os.path.join(root, ".broad-sample"))
judge("M3 库目录是判法版本进指纹之前的驱动换的那一版（今天样本 setup.sh 造的就是它）", LIB, root, "M3-version-key")

root = copy("library-version-is-staged-tree-green", "M4")
line = git(root, "ls-files", "-s", "--", "crates/sample/src/lib.rs").split()
mode, blob = line[0], line[1]
other = git(root, "hash-object", "-w", "--stdin", data="pub fn add(left: u32, right: u32) -> u32 { left - right }\n").strip()
git(root, "update-index", "--index-info", data=f"0 {'0' * 40}\tcrates/sample/src/lib.rs\n{mode} {other} 1\tcrates/sample/src/lib.rs\n{mode} {other} 2\tcrates/sample/src/lib.rs\n{mode} {blob} 3\tcrates/sample/src/lib.rs\n")
judge("M4 索引里 lib.rs 在合并冲突（ours 是另一版，theirs 与那一版相同）", LIB, root, "M4-merge-conflict")

root = copy("library-version-is-staged-tree-green", "M5")
import json
path = os.path.join(root, ".broad-sample", "home", "tree.source")
source = json.load(open(path, encoding="utf-8"))
source["source_fingerprint"] = "0" * 64
json.dump(source, open(path, "w", encoding="utf-8"))
judge("M5 tree.source 记的源码指纹与 manifest.tsv 不是同一次写的", LIB, root, "M5-source-fingerprint")

root = copy("library-version-is-staged-tree-green", "M6")
version = json.load(open(os.path.join(root, ".broad-sample", "home", "tree.source"), encoding="utf-8"))["fingerprint"]
os.remove(os.path.join(root, ".broad-sample", "home", "versions", version, "toolchain.txt"))
judge("M6 那一版没有 toolchain.txt", LIB, root, "M6-toolchain-missing")

root = copy("library-version-is-staged-tree-green", "M7")
git(root, "rm", "-q", "-r", "--cached", "crates", "Cargo.toml", "Cargo.lock", "litmus", ".cargo")
judge("M7 暂存树在导出范围里一个文件都没有", LIB, root, "M7-empty-staged")

root = copy("gate-verdict-green", "N1")
summary(root, [HEAD_LINE, f"BROAD_MUTATION_GATE total=6 concluded=6 carried=4 fresh=2 missed=0 without_conclusion=0 flipped=0 verdict=green"], 1)
judge("N1 判定行写 green、--summary 退 1（结果表对不上）", VERDICT, root, "N1-exit-not-0-8")

root = copy("gate-verdict-green", "N2")
summary(root, [HEAD_LINE.replace("caught=5 missed=0", "caught=4 missed=1"), "MISSED\t3\tsinglefs-core\tcrates/singlefs-core/src/sample.rs:12:9: replace + with - in add",
               "BROAD_MUTATION_GATE total=6 concluded=6 carried=3 fresh=3 missed=1 without_conclusion=0 flipped=0 verdict=red"], 0)
judge("N2 判定行写 red、计数与明细一致、--summary 退 0", VERDICT, root, "N2-verdict-exit-mismatch")

root = copy("gate-verdict-green", "N4")
summary(root, [HEAD_LINE, "BROAD_MUTATION_GATE total=6 concluded=6 carried=4 fresh=2 missed=0 without_conclusion=1 flipped=0 verdict=red"], 0)
judge("N4 concluded 加 without_conclusion 是 7、total 是 6，判定行 red 而退 0", VERDICT, root, "N4-sum-not-total")

root = copy("gate-verdict-green", "N5")
summary(root, [HEAD_LINE, "BROAD_MUTATION_GATE total=6 concluded=6 carried=4 fresh=2 missed=0 without_conclusion=0 flipped=0 verdict=green",
               "NO_CONCLUSION\t5\tnone\tcrates/singlefs-core/src/sample.rs:20:9: replace x", "NO_CONCLUSION\t6\tnone\tcrates/singlefs-core/src/sample.rs:24:9: replace y"], 0)
judge("N5 判定行 green、计数全零，却列了 2 行 NO_CONCLUSION", VERDICT, root, "N5-detail-vs-counts")

root = copy("gate-verdict-green", "N6")
summary(root, [HEAD_LINE.replace("total=6 caught=5", "total=0 caught=0").replace("unviable=1", "unviable=0"),
               "BROAD_MUTATION_GATE total=0 concluded=0 carried=0 fresh=0 missed=0 without_conclusion=0 flipped=0 verdict=green"], 0)
judge("N6 这一版清单 0 条、判定行 green", VERDICT, root, "N6-total-zero")

root = copy("gate-verdict-green", "N7")
summary(root, [HEAD_LINE, "BROAD_MUTATION_GATE total=6 concluded=6 carried=4 fresh=2 missed=0 without_conclusion=0 flipped=0 verdict=pending"], 8)
judge("N7 判定行 verdict=pending、--summary 退 8", VERDICT, root, "N7-unknown-verdict")

root = copy("gate-verdict-green", "G1")
summary(root, [f"BROAD_MUTATION_SUMMARY fingerprint={FP} commit=0123456789ab total=5091 caught=5 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 driver_error=0 not_run=5085",
               "BROAD_MUTATION_GATE total=6 concluded=6 carried=4 fresh=2 missed=0 without_conclusion=0 flipped=0 verdict=green"], 0)
judge("G1 汇总行 total=5091、not_run=5085，判定行 total=6 green（两行的 total 不一致）", VERDICT, root, None)
