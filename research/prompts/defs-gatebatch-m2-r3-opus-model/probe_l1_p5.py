"""P5（判不出按没标算）的代价「准入模块坏了时点名登记目标的 cargo test 全被拒——方向是多拒」量一遍：
在草稿目录搭一个镜像仓（.claude/hooks 拷真仓那三份；.claude/scripts 与 .claude/singlefs-ai-sop 链到真仓，preflight 要它们；
research/scripts/admission.py 是真仓那一份或弄坏的副本），仓里一个小工作区登记一条崩溃枚举用例，用例函数**没标** #[ignore]
（不带 --ignored 的 cargo test 真会跑到全量，闸该拒）。准入模块的坏法分两种：导入不了（语法错）与导入得了、判的那一刻抛异常（改了名 /
签名没跟上：别的会话改到一半、或这两份文件不同步时）。仓里被判的文件只读。
复跑：python3 probe_l1_p5.py"""
import json
import os
import shutil
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

cells = pc.Cells("L1 P5：准入模块坏了时闸往哪边偏")
REAL_ADMISSION = pc.ADMISSION
MC = f"bash {pc.REPOSITORY}/research/scripts/run-with-memory-cap.sh 8G"
# R3_HOOKS_SOURCE 指到改过的 hooks 副本时量攻方改法（副本上的数）
HOOKS_SOURCE = os.environ.get("R3_HOOKS_SOURCE") or os.path.join(pc.REPOSITORY, ".claude", "hooks")


def mirror(admission_text, marked):
    root = pc.scratch("p5-mirror-")
    os.makedirs(os.path.join(root, ".claude", "hooks"))
    for name in ("heavy-test-guard.sh", "lib_heavy_tests.py", "lib_shell_words.py"):
        shutil.copy2(os.path.join(HOOKS_SOURCE, name), os.path.join(root, ".claude", "hooks", name))
    for name in ("scripts", "singlefs-ai-sop"):
        os.symlink(os.path.join(pc.REPOSITORY, ".claude", name), os.path.join(root, ".claude", name))
    pc.write(os.path.join(root, "research", "scripts", "admission.py"), admission_text)
    pc.write(os.path.join(root, "Cargo.toml"), '[workspace]\nmembers = ["crates/pkg"]\n')
    pc.write(os.path.join(root, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\nedition = "2021"\n')
    pc.write(os.path.join(root, "crates/pkg/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
    pc.write(os.path.join(root, "crates/pkg/tests/own_case.rs"),
             "#[test]\n" + ("#[ignore]\n" if marked else "") + "fn the_case() { /* 全量 */ }\n")
    pc.write(os.path.join(root, ".claude/gate.d/stage-inputs.tsv"),
             "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0\t# 样本\n")
    pc.run(["git", "init", "-q", "-b", "master", root])
    return root


def judged(root, command):
    detections = os.path.join(root, "detections.jsonl")
    code, lines = pc.hook(command, root, detections=detections, hook_path=os.path.join(root, ".claude/hooks/heavy-test-guard.sh"), project=root)
    recorded = sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
    return code, lines, recorded


original = open(REAL_ADMISSION, encoding="utf-8").read()
assert original.count("def definitions_marked_ignored(") == 1 and original.count("    marks = definitions_marked_ignored(root, files, function)\n") == 1
assert original.count("def test_function_is_marked_ignored(") == 1
VARIANTS = [
    ("CONTROL", "A 准入模块原样", original),
    ("CONTROL", "B 导入不了（文件尾加一行语法错）", original + "\ndef broken(:\n"),
    ("ATTACK", "C 导入得了、判的那一刻抛 NameError（definitions_marked_ignored 改名，调它的一处没跟上）",
     original.replace("def definitions_marked_ignored(", "def definitions_marked_ignored_renamed(")),
    ("ATTACK", "D 导入得了、闸调的 test_function_is_marked_ignored 改了名（AttributeError）",
     original.replace("def test_function_is_marked_ignored(", "def crash_case_function_is_marked_ignored(")),
]
COMMAND = f"{MC} cargo test --release -p pkg --test own_case"
for kind, label, text in VARIANTS:
    root = mirror(text, marked=False)
    code, lines, recorded = judged(root, COMMAND)
    cells.expect(kind, f"{label}；用例函数没标 ignore、不带 --ignored ⇒ 应当拒", code == 2,
                 f"闸退 {code}，检出记了 {recorded} 条；stderr：{lines[:150]}")
# 对照：C 那种坏法下，带 --include-ignored 的照拒（ignored_reasons 先判，碰不到准入模块）
root = mirror(VARIANTS[2][2], marked=True)
code, lines, recorded = judged(root, f"{COMMAND} -- --include-ignored")
cells.expect("CONTROL", "C 那种坏法下带 --include-ignored ⇒ 应当拒（不经准入模块）", code == 2, f"闸退 {code}；stderr：{lines[:120]}")
sys.exit(cells.finish())
