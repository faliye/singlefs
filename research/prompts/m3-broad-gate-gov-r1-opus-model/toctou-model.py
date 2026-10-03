#!/usr/bin/env python3
"""攻方腿模型（m3-broad-gate-gov-r1，G2）：两格先后各读一次 tree.source，中间别的会话 --sync-tree 换了版本指针，门禁两格都绿而判绿的不是这一次的暂存树那一版。
用法：toctou-model.py <tree> <run-fixed 目录> <草稿目录>
库目录里放两版：A（与样本暂存树相同，cells.py 起 library-version-is-staged-tree 判 A）、B（别的会话那棵树，每条都有结论）；
换指针照驱动 sync_tree 的最后一步（os.replace 临时件到 tree.source）。--summary 照阶段样本档的接法用录好的输出，内容是「此刻 tree.source 那一版」B 的汇总。
另跑一个对照：不换指针，第二格看到汇总行是 B 而 tree.source 是 A，判红。"""
import hashlib, importlib.util, json, os, shutil, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cells
tree, fixed, scratch = sys.argv[1:4]
specification = importlib.util.spec_from_file_location("driver_for_model", os.path.join(tree, "research/scripts/broad-mutation.py"))
driver = importlib.util.module_from_spec(specification)
specification.loader.exec_module(driver)

def build(name):
    root = os.path.join(scratch, "toctou", name)
    shutil.rmtree(root, ignore_errors=True)
    shutil.copytree(os.path.join(fixed, "library-version-is-staged-tree-green"), root, symlinks=True)
    home = os.path.join(root, ".broad-sample", "home")
    a = json.load(open(os.path.join(home, "tree.source"), encoding="utf-8"))
    toolchain = open(os.path.join(home, "versions", a["fingerprint"], "toolchain.txt"), encoding="utf-8").read()
    manifest = driver.read_manifest(os.path.join(home, "versions", a["fingerprint"], "manifest.tsv"))
    manifest["crates/sample/src/lib.rs"] = hashlib.sha256(b"// \xe5\x88\xab\xe7\x9a\x84\xe4\xbc\x9a\xe8\xaf\x9d\xe7\x9a\x84\xe9\x82\xa3\xe4\xb8\x80\xe7\x89\x88\n").hexdigest()
    source_b = driver.fingerprint_of_manifest(manifest)
    b = dict(a, fingerprint=driver.version_key(home, source_b, toolchain), source_fingerprint=source_b, commit="b" * 40)
    os.makedirs(os.path.join(home, "versions", b["fingerprint"]))
    with open(os.path.join(home, "versions", b["fingerprint"], "manifest.tsv"), "w", encoding="utf-8") as handle:
        handle.write("".join(f"{manifest[path]}\t{path}\n" for path in sorted(manifest)))
    shutil.copy(os.path.join(home, "versions", a["fingerprint"], "toolchain.txt"), os.path.join(home, "versions", b["fingerprint"], "toolchain.txt"))
    with open(os.path.join(home, "tree.source.b"), "w", encoding="utf-8") as handle:
        json.dump(b, handle)
    summary = os.path.join(root, ".broad-sample", "summary.sh")
    with open(summary, "w", encoding="utf-8") as handle:
        handle.write("#!/usr/bin/env bash\ncat <<'EOF'\n"
                     f"BROAD_MUTATION_SUMMARY fingerprint={b['fingerprint'][:16]} commit=bbbbbbbbbbbb total=6 caught=5 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 driver_error=0 not_run=0\n"
                     "BROAD_MUTATION_GATE total=6 concluded=6 carried=4 fresh=2 missed=0 without_conclusion=0 flipped=0 verdict=green\nEOF\nexit 0\n")
    return root, home, a, b

def line(text):
    return next((row.strip() for row in text.splitlines() if "✗" in row or "✓" in row), "")

for name, swap in (("swap-between-cells", True), ("no-swap-control", False)):
    root, home, a, b = build(name)
    code_one, text_one = cells.run(tree, "library-version-is-staged-tree", root)
    print(f"[{name}] 第一格 library-version-is-staged-tree 退 {code_one}：{line(text_one)}")
    if swap:
        os.replace(os.path.join(home, "tree.source.b"), os.path.join(home, "tree.source"))
        print(f"[{name}] 两格之间别的会话 --sync-tree 换指针：tree.source {a['fingerprint'][:16]} → {b['fingerprint'][:16]}（导出自 {b['commit'][:12]}）")
    code_two, text_two = cells.run(tree, "gate-verdict", root)
    print(f"[{name}] 第二格 gate-verdict 退 {code_two}：{line(text_two)}")
    print(f"[{name}] 整道：{'绿' if code_one == 0 and code_two == 0 else '红'}")
