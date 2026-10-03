#!/usr/bin/env python3
"""攻方腿模型（m3-broad-gate-gov-r1）：把门禁 checker-tier-broad-mutation.sh 两格的内嵌 python 原样抽出来，照阶段里 bash 的调法在样本根上起。
只读样本根与它造的库目录，不起驱动的真跑、不起 cargo。被判文件取冻结副本的拷贝（--tree 给根）。

  cells.py extract <tree>                         打两段内嵌 python 的 sha256 与行数
  cells.py run <tree> <cell> <样本根> [弄坏项] [--mutant <名>]   照阶段的调法跑一格，打输出、末行 exit=<码>
  cells.py mutants                                列出这里定义的变异（每条删一个判红分支）
"""
import hashlib, os, subprocess, sys, tempfile

def extract(tree):
    path = os.path.join(tree, ".claude/gate.d/checker-tier-broad-mutation.sh")
    lines = open(path, encoding="utf-8").read().split("\n")
    bodies, inside, current = [], False, []
    for line in lines:
        if not inside and line.endswith("<<'PY'"):
            inside, current = True, []
            continue
        if inside and line == "PY":
            bodies.append("\n".join(current) + "\n")
            inside = False
            continue
        if inside:
            current.append(line)
    assert len(bodies) == 2, f"内嵌 python 段数 {len(bodies)}，应当是 2"
    return {"library-version-is-staged-tree": bodies[0], "gate-verdict": bodies[1]}

# 每条变异：(格, 原文里恰好出现一次的片段, 换成什么, 删掉的是哪个判红分支)
MUTANTS = {
    "M1-pending": ("library-version-is-staged-tree", 'if os.path.exists(os.path.join(home, "tree.source.pending")):', "if False:", "tree.source.pending 在判红"),
    "M2-only-recorded": ("library-version-is-staged-tree", "if only_staged or only_recorded or differing:", "if only_staged or differing:", "那一版有、暂存树没有（暂存区删了文件）判红"),
    "M3-version-key": ("library-version-is-staged-tree", "and driver.version_key(home, staged_source_fingerprint, current_toolchain) != fingerprint:", "and False:", "按驱动 version_key 复算的版本指纹不等判红"),
    "M4-merge-conflict": ("library-version-is-staged-tree", 'if stage_number != "0":', "if False:", "索引里有合并冲突判红"),
    "M5-source-fingerprint": ("library-version-is-staged-tree", 'if not problems and staged_source_fingerprint != source.get("source_fingerprint"):', "if False:", "逐文件相同而源码指纹不同判红"),
    "M6-toolchain-missing": ("library-version-is-staged-tree", "if not os.path.isfile(toolchain_path):", "if False:", "那一版没记工具链判红"),
    "M7-empty-staged": ("library-version-is-staged-tree", "if not staged:\n", "if False:\n", "导出范围里一个文件都没有判红"),
    "N1-exit-not-0-8": ("gate-verdict", "if summary_exit not in (0, 8):", "if False:", "--summary 退的不是 0 也不是 8 判红"),
    "N2-verdict-exit-mismatch": ("gate-verdict", 'elif summary_exit != expected_exit and "summary-red-passes" not in broken:', "elif False:", "判定行与退出码对不上判红"),
    "N3-green-with-counts": ("gate-verdict", 'if verdict == "green" and (missed or without or concluded != total):', "if False:", "判定行写 green 而计数不为零判红"),
    "N4-sum-not-total": ("gate-verdict", "if concluded + without != total:", "if False:", "concluded 加 without_conclusion 不等于 total 判红"),
    "N5-detail-vs-counts": ("gate-verdict", 'if len(no_conclusion) != without or len(missed_lines) != missed or number("missed", summary) != missed:', "if False:", "明细行数与计数不同判红"),
    "N6-total-zero": ("gate-verdict", "if total <= 0:", "if False:", "清单 0 条判红"),
    "N7-unknown-verdict": ("gate-verdict", 'if verdict not in ("green", "red"):', "if False:", "verdict 不是 green / red 判红"),
}

def mutated(bodies, mutant):
    cell, old, new, _what = MUTANTS[mutant]
    body = bodies[cell]
    assert body.count(old) == 1, f"{mutant}：原文片段出现 {body.count(old)} 次"
    changed = dict(bodies)
    changed[cell] = body.replace(old, new)
    return changed

def run(tree, cell, root, broken="", mutant=None, home=None, summary=None, rustc_override=None):
    """照阶段的调法：cd 进根；样本档下库目录取 <根>/.broad-sample/home、--summary 换成 bash <根>/.broad-sample/summary.sh、rustc -Vv 换成 rustc-vv.txt。"""
    bodies = extract(tree)
    if mutant:
        bodies = mutated(bodies, mutant)
    driver = os.path.join(tree, "research/scripts/broad-mutation.py")
    home = home or os.path.join(root, ".broad-sample", "home")
    if rustc_override is None:
        candidate = os.path.join(root, ".broad-sample", "rustc-vv.txt")
        rustc_override = candidate if os.path.isfile(candidate) else ""
    environment = {k: v for k, v in os.environ.items() if k != "BROAD_MUTATION_BREAK"}
    if cell == "library-version-is-staged-tree":
        done = subprocess.run(["python3", "-", root, home, driver, rustc_override, broken], input=bodies[cell], text=True,
                              capture_output=True, cwd=root, env=environment)
        return done.returncode, done.stdout + done.stderr
    summary = summary or ["bash", os.path.join(root, ".broad-sample", "summary.sh")]
    with tempfile.NamedTemporaryFile("w+", delete=False) as handle:
        produced = subprocess.run(summary, stdout=handle, stderr=subprocess.STDOUT, cwd=root, env=environment)
        output_path = handle.name
    try:
        done = subprocess.run(["python3", "-", output_path, str(produced.returncode), home, broken], input=bodies[cell], text=True,
                              capture_output=True, cwd=root, env=environment)
    finally:
        os.unlink(output_path)
    return done.returncode, done.stdout + done.stderr

def main(argv):
    if argv[:1] == ["extract"]:
        for cell, body in extract(argv[1]).items():
            print(f"{cell} lines={body.count(chr(10))} sha256={hashlib.sha256(body.encode()).hexdigest()}")
        return 0
    if argv[:1] == ["mutants"]:
        for name, (cell, _old, _new, what) in MUTANTS.items():
            print(f"{name}\t{cell}\t{what}")
        return 0
    if argv[:1] == ["run"]:
        rest = argv[1:]
        mutant = None
        if "--mutant" in rest:
            index = rest.index("--mutant")
            mutant = rest[index + 1]
            rest = rest[:index] + rest[index + 2:]
        tree, cell, root = rest[:3]
        broken = rest[3] if len(rest) > 3 else ""
        code, text = run(tree, cell, root, broken, mutant)
        sys.stdout.write(text)
        print(f"exit={code}")
        return 0
    print(__doc__)
    return 2

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
