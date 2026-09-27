"""K1（Y5 / Y6 排除规则、Y7 runner 进指纹）探针：在小仓上跑被判的 admission.py crash-case-manifest，改一处用例其实读得到的文件，看指纹变不变。
复跑：python3 probe_k1_manifest.py
ATTACK 格：用例编译期或运行期读得到那份文件（或 runner 跑起来读得到），改它指纹应当变；打中 = 没变（下一趟 54 号 --full 复用旧的全绿标记）。
CONTROL 格：对照。编译期读得到这一句另在 includecrate / runnercrate 两个小 crate 上真编过（报告里贴原样）。"""
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

cells = pc.Cells("K1 排除规则与 runner 指纹")
ROW = "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0\t# 样本\n"
OWN_HEAD = "#[test]\n#[ignore]\nfn the_case() { let _ = PIECE; }\n"
DROPPED = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTC")


def clean_environment(extra=None):
    environment = {name: value for name, value in os.environ.items()
                   if name not in DROPPED and not name.startswith(("CARGO_PROFILE_", "CARGO_BUILD_", "CARGO_TARGET_"))}
    environment["CARGO_HOME"] = os.path.join(pc.SCRATCH_ROOT, "empty-cargo-home")
    environment.update(extra or {})
    return environment


def repository(own_piece, extra_files=None):
    work = pc.scratch("k1-manifest-")
    pc.write(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/*"]\n')
    pc.write(os.path.join(work, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\nedition = "2021"\n')
    pc.write(os.path.join(work, "crates/pkg/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
    pc.write(os.path.join(work, "crates/pkg/tests/own_case.rs"), own_piece + OWN_HEAD)
    pc.write(os.path.join(work, "crates/pkg/tests/other_target.rs"), "#[test]\nfn other() {}\n")
    pc.write(os.path.join(work, "crates/pkg/src/bin/tool.rs"), "fn main() {}\n")
    pc.write(os.path.join(work, "crates/mutations.tsv"), "row\tone\n")
    pc.write(os.path.join(work, ".claude/gate.d/stage-inputs.tsv"), ROW)
    for relative, text in (extra_files or {}).items():
        pc.write(os.path.join(work, relative), text)
    pc.run(["git", "init", "-q", "-b", "master", work])
    return work


def fingerprint(work, extra_environment=None):
    manifest = os.path.join(work, ".git", "k1-manifest.out")
    completed = subprocess.run([sys.executable, pc.ADMISSION, "crash-case-manifest", work, "crash-case:own", manifest,
                                "--toolchain", "--build-environment"], capture_output=True, text=True,
                               env=clean_environment(extra_environment))
    fields = completed.stdout.split()
    return (fields[0] if completed.returncode == 0 and fields else f"错（退 {completed.returncode}：{completed.stdout.strip()[:120]}）"), fields[1:]


def cell(kind, label, own_piece, changed_file, new_text, extra_files=None, extra_environment=None, want_change=True):
    work = repository(own_piece, extra_files)
    environment = {name: value.replace("<仓>", work) for name, value in (extra_environment or {}).items()}
    before, counts_before = fingerprint(work, environment)
    pc.write(os.path.join(work, changed_file), new_text)
    after, counts_after = fingerprint(work, environment)
    changed = before != after
    cells.expect(kind, f"{label} ⇒ 改 {changed_file} 指纹应当{'变' if want_change else '不变'}", changed == want_change,
                 f"改前 {before[:16]}… 改后 {after[:16]}…（文件数 / 减去数 改前 {'/'.join(counts_before)} 改后 {'/'.join(counts_after)}）")


OTHER = "crates/pkg/tests/other_target.rs"
OTHER_CHANGED = "#[test]\nfn other() { let changed = 2; }\n"
# ── 对照：别的测试目标独占的文件，改它指纹不变（排除法按设计减掉）──
cell("CONTROL", "C1 用例不读 other_target", "const PIECE: u32 = 0;\n", OTHER, OTHER_CHANGED, want_change=False)
cell("CONTROL", "C2 include_str!(concat!(…)) 拼出 other_target 的路径（Y5 认得的写法）",
     'const PIECE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oth", "er_target.rs"));\n', OTHER, OTHER_CHANGED)
# ── Y5 攻：写法换一种 ──
cell("ATTACK", "M1 ::core::include_str!(::core::concat!(…))（带路径的宏名，编得过、读同一份）",
     'const PIECE: &str = ::core::include_str!(::core::concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oth", "er_target.rs"));\n', OTHER, OTHER_CHANGED)
cell("ATTACK", "M2 include_str!(env!(\"K1_FIXTURE\"))，路径在仓根 .cargo/config.toml 的 [env] 里（relative = true）",
     'const PIECE: &str = include_str!(env!("K1_FIXTURE"));\n', OTHER, OTHER_CHANGED,
     extra_files={".cargo/config.toml": '[env]\nK1_FIXTURE = { value = "crates/pkg/tests/other_target.rs", relative = true }\n'})
GEN = {"crates/gen/Cargo.toml": '[package]\nname = "gen"\nversion = "0.0.0"\nedition = "2021"\n',
       "crates/gen/src/lib.rs": 'pub const LISTING: &str = include_str!(concat!(env!("OUT_DIR"), "/listing.txt"));\n'}
cell("CONTROL", "C3 别的包 gen 的 build.rs 自己按目录读 pkg 的 tests/（Y5 认得的写法）", "const PIECE: u32 = 0;\n", OTHER, OTHER_CHANGED,
     extra_files=dict(GEN, **{"crates/gen/build.rs": 'fn main() { let _ = std::fs::read_dir("../pkg/tests"); }\n'}))
cell("ATTACK", "M3 gen 的 build.rs 只写 mod scan;，按目录读 pkg 的 tests/ 的那几行在 scan.rs 里",
     "const PIECE: u32 = 0;\n", OTHER, OTHER_CHANGED,
     extra_files=dict(GEN, **{"crates/gen/build.rs": "mod scan;\nfn main() { scan::run(); }\n",
                              "crates/gen/scan.rs": 'pub fn run() { let _ = std::fs::read_dir("../pkg/tests"); }\n'}))
# ── Y6 攻：Y5 的「拼出来的名字整包不减」管不到 mutations.tsv 与 src/bin ──
cell("ATTACK", "M4 include_str!(concat!(…)) 拼出 crates/mutations.tsv",
     'const PIECE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../mutations", ".tsv"));\n',
     "crates/mutations.tsv", "row\ttwo\n")
cell("ATTACK", "M5 include!(concat!(…)) 拼出 src/bin/tool.rs 里的代码",
     'const PIECE: u32 = 0;\nmod tool_code { include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bin/", "tool.rs")); }\n',
     "crates/pkg/src/bin/tool.rs", "fn main() { let changed = 2; }\n")
# ── Y7：runner 指的程序 ──
RUNNER = "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER"
RUNNER_FILES = {"tools/r.sh": "#!/usr/bin/env bash\nsource \"$(dirname \"$0\")/r-lib.sh\"\nexec \"$@\" $EXTRA\n",
                "tools/r-lib.sh": "EXTRA=--include-ignored\n"}
cell("CONTROL", "C4 runner = <仓>/tools/r.sh（直接指脚本），改 r.sh", "const PIECE: u32 = 0;\n", "tools/r.sh", "#!/usr/bin/env bash\nexec \"$@\"\n",
     extra_files=RUNNER_FILES, extra_environment={RUNNER: "<仓>/tools/r.sh"})
cell("ATTACK", "G1 runner = bash <仓>/tools/r.sh（前面带解释器，cargo 按空白切），改 r.sh", "const PIECE: u32 = 0;\n", "tools/r.sh",
     "#!/usr/bin/env bash\nexec \"$@\"\n", extra_files=RUNNER_FILES, extra_environment={RUNNER: "bash <仓>/tools/r.sh"})
cell("ATTACK", "G2 .cargo/config.toml 里 runner = [\"bash\", \"tools/r.sh\"]，改 r.sh", "const PIECE: u32 = 0;\n", "tools/r.sh",
     "#!/usr/bin/env bash\nexec \"$@\"\n",
     extra_files=dict(RUNNER_FILES, **{".cargo/config.toml": '[target.x86_64-unknown-linux-gnu]\nrunner = ["bash", "tools/r.sh"]\n'}))
cell("ATTACK", "G3 runner = <仓>/tools/r.sh，它 source 的 r-lib.sh 改了", "const PIECE: u32 = 0;\n", "tools/r-lib.sh", "EXTRA=\n",
     extra_files=RUNNER_FILES, extra_environment={RUNNER: "<仓>/tools/r.sh"})

sys.exit(cells.finish())
