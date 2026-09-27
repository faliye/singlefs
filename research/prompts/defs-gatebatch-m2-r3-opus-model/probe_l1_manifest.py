"""P6 / P7（算出来的 include 按宽处理）与 P9（runner 参数里的脚本按内容进指纹）：在小 git 仓上跑仓里被判的 admission.py crash-case-manifest，
改一份用例其实读得到的文件，看这条用例的指纹变不变。
  P6/P7：include_str! 用 `use core::include_str as grab;` 改了名再调，参数照旧 concat!(env!(…), …) 拼（名字拆开写，整词点名那一条碰不到）。
  P9：runner 的参数写成相对路径。cargo 起 runner 时进程的当前目录是包目录（verify_semantics.sh 的 SEM-P9 量过：crates/pkg），
      能跑的写法是从包目录起的 `sh ../../tools/r.sh`；准入模块从 .cargo 那一层（环境变量是仓根）解，指不到它。
复跑：python3 probe_l1_manifest.py"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

cells = pc.Cells("L1 P6 P7 P9：清单与指纹")
RUNNER = "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER"


def repository(lib_source, cargo_configuration=None):
    work = pc.scratch("l1-manifest-")
    pc.write(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/pkg"]\n')
    pc.write(os.path.join(work, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\nedition = "2021"\n')
    pc.write(os.path.join(work, "crates/pkg/src/lib.rs"), lib_source)
    pc.write(os.path.join(work, "crates/pkg/src/bin/tool.rs"), "fn main() {}\n")
    pc.write(os.path.join(work, "crates/pkg/tests/own_case.rs"), "#[test]\n#[ignore]\nfn the_case() { assert_eq!(pkg::one(), 1); }\n")
    pc.write(os.path.join(work, "crates/pkg/tests/other.rs"), "#[test]\nfn other() {}\n")
    pc.write(os.path.join(work, "crates/mutations.tsv"), "row-one\n")
    pc.write(os.path.join(work, "tools/r.sh"), 'exec "$@"\n')
    if cargo_configuration:
        pc.write(os.path.join(work, ".cargo/config.toml"), cargo_configuration)
    pc.write(os.path.join(work, ".claude/gate.d/stage-inputs.tsv"),
             "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0\t# 样本\n")
    pc.run(["git", "init", "-q", "-b", "master", work])
    return work


def fingerprint(work, environment=None):
    code, out, err = pc.run([sys.executable, pc.ADMISSION, "crash-case-manifest", work, "crash-case:own", os.path.join(work, ".manifest"),
                             "--build-environment"], extra_environment=environment, drop=[RUNNER])
    parts = out.split()
    return (parts[0][:16], parts[1], parts[2]) if code == 0 and len(parts) == 3 else (f"退 {code}：{(out + err).strip()[:80]}", "-", "-")


def changes(kind, label, lib_source, edit_path, edit_text, want_change, cargo_configuration=None, environment=None):
    work = repository(lib_source, cargo_configuration)
    before = fingerprint(work, environment)
    pc.write(os.path.join(work, edit_path), edit_text)
    after = fingerprint(work, environment)
    changed = before[0] != after[0]
    cells.expect(kind, f"{label} ⇒ 改 {edit_path} 指纹应当{'变' if want_change else '不变'}", changed == want_change,
                 f"改前 {before[0]}… 改后 {after[0]}…（文件数 / 减去数 改前 {before[1]}/{before[2]} 改后 {after[1]}/{after[2]}）")


LIB = "pub fn one() -> u32 { 1 }\n"
SPLIT_TABLE = 'concat!(env!("CARGO_MANIFEST_DIR"), "/../mutations", ".tsv")'
SPLIT_BIN = 'concat!(env!("CARGO_MANIFEST_DIR"), "/src/b", "in/tool.rs")'
SPLIT_OTHER = 'concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oth", "er.rs")'
RENAME = "use core::include_str as grab;\n"
# ── P6 / P7 ──
changes("CONTROL", "M4r include_str!(concat!(…拆开写的 mutations.tsv…))（P7 修的那一格）", LIB + f"pub const T: &str = include_str!({SPLIT_TABLE});\n",
        "crates/mutations.tsv", "row-two\n", True)
changes("ATTACK", "N1 use core::include_str as grab; grab!(concat!(…拆开写的 mutations.tsv…))", LIB + RENAME + f"pub const T: &str = grab!({SPLIT_TABLE});\n",
        "crates/mutations.tsv", "row-two\n", True)
changes("CONTROL", "M5r include_str!(concat!(…拆开写的 src/bin/tool.rs…))", LIB + f"pub const T: &str = include_str!({SPLIT_BIN});\n",
        "crates/pkg/src/bin/tool.rs", "fn main() { let _ = 2; }\n", True)
changes("ATTACK", "N2 改名的 grab!(concat!(…拆开写的 src/bin/tool.rs…))", LIB + RENAME + f"pub const T: &str = grab!({SPLIT_BIN});\n",
        "crates/pkg/src/bin/tool.rs", "fn main() { let _ = 2; }\n", True)
changes("CONTROL", "M1r include_str!(concat!(…拆开写的 tests/other.rs…))（这个包的测试文件一份都不减）", LIB + f"pub const T: &str = include_str!({SPLIT_OTHER});\n",
        "crates/pkg/tests/other.rs", "#[test]\nfn other() { let _ = 2; }\n", True)
changes("ATTACK", "N3 改名的 grab!(concat!(…拆开写的 tests/other.rs…))", LIB + RENAME + f"pub const T: &str = grab!({SPLIT_OTHER});\n",
        "crates/pkg/tests/other.rs", "#[test]\nfn other() { let _ = 2; }\n", True)
# ── P9 ──
ABSOLUTE_CONFIGURATION = None  # 绝对路径那一格要知道仓的位置，下面单独搭
work = repository(LIB)
pc.write(os.path.join(work, ".cargo/config.toml"), f'[target.x86_64-unknown-linux-gnu]\nrunner = "sh {work}/tools/r.sh"\n')
before = fingerprint(work)
pc.write(os.path.join(work, "tools/r.sh"), 'exec "$@" --include-ignored\n')
after = fingerprint(work)
cells.expect("CONTROL", "G1a runner = \"sh <仓的绝对路径>/tools/r.sh\" ⇒ 改 tools/r.sh 指纹应当变", before[0] != after[0], f"改前 {before[0]}… 改后 {after[0]}…")
changes("CONTROL", "G1b runner = \"sh tools/r.sh\"（从 .cargo 那一层解；照 SEM-P9，cargo 在 crates/pkg 起它，这种写法跑不起来）", LIB, "tools/r.sh",
        'exec "$@" --include-ignored\n', True, cargo_configuration='[target.x86_64-unknown-linux-gnu]\nrunner = "sh tools/r.sh"\n')
changes("ATTACK", "P9a runner = \"sh ../../tools/r.sh\"（从包目录起，cargo 真能跑的写法）", LIB, "tools/r.sh",
        'exec "$@" --include-ignored\n', True, cargo_configuration='[target.x86_64-unknown-linux-gnu]\nrunner = "sh ../../tools/r.sh"\n')
changes("ATTACK", f"P9b 环境变量 {RUNNER}=\"sh ../../tools/r.sh\"", LIB, "tools/r.sh", 'exec "$@" --include-ignored\n', True,
        environment={RUNNER: "sh ../../tools/r.sh"})
sys.exit(cells.finish())
