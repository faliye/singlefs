#!/usr/bin/env python3
"""攻方 J1-c：驱动的 build_environment_of 与门禁现算全局键都调 research/scripts/admission.py 的 build_environment_lines（一份定义）。
这里把那一份逐类写错（每种一份树拷贝，只改 admission.py 一处），看谁翻面：门禁 36 份样本（本脚本之后由 run-samples.sh 跑）、
admission.py --selftest 与驱动 --selftest（拷贝上的自证被重型闸拒子 agent，写成 main-agent-heavy.sh 交主 agent）。
用法：j1c-admission-mutants.py <原样树根> <放拷贝的目录>；每行打「名字<TAB>写错的是哪一类」。旧串在文件里必须恰好命中一次。"""
import os, shutil, subprocess, sys
base, into = sys.argv[1:3]
TARGET = "research/scripts/admission.py"
MUTANTS = [
    ("a1-no-profile", "环境变量 CARGO_PROFILE_* 不进（改 overflow-checks、opt-level 的那一类）",
     'BUILD_ENVIRONMENT_VARIABLE_FORMS = (re.compile(r"^CARGO_PROFILE_"), re.compile(r"^CARGO_BUILD_"),',
     'BUILD_ENVIRONMENT_VARIABLE_FORMS = (re.compile(r"^CARGO_BUILD_"),'),
    ("a2-no-runner", "CARGO_TARGET_*_RUNNER 的值与它指的程序都不进",
     '                                    re.compile(r"^CARGO_TARGET_.+_RUSTFLAGS$"), re.compile(r"^CARGO_TARGET_.+_RUNNER$"))',
     '                                    re.compile(r"^CARGO_TARGET_.+_RUSTFLAGS$"),)'),
    ("a3-no-toolchain-file", "目录层级里的 rust-toolchain(.toml) 不进", 'TOOLCHAIN_FILE_NAMES = ("rust-toolchain", "rust-toolchain.toml")', 'TOOLCHAIN_FILE_NAMES = ()'),
    ("a4-program-name-only", "RUSTC_WRAPPER 这类指的程序只记值、不按内容进",
     '    program = words[0] if words else ""\n    path = resolved_program(program, base_directory, environment)',
     '    return b"program"\n    program = words[0] if words else ""\n    path = resolved_program(program, base_directory, environment)'),
    ("a5-no-ancestors", "只读树根那一层的 .cargo/config，不往上走（库目录、~/.local/share 这几层的 cargo 配置不进）",
     '        parent = os.path.dirname(directory)\n        if parent == directory:', '        parent = os.path.dirname(directory)\n        if True:'),
    ("a6-no-rustc", "RUSTC / CARGO_BUILD_RUSTC 的值与 -V 不进", 'BUILD_ENVIRONMENT_COMPILER_VARIABLES = ("RUSTC", "CARGO_BUILD_RUSTC")', 'BUILD_ENVIRONMENT_COMPILER_VARIABLES = ()'),
    ("a7-no-encoded", "CARGO_ENCODED_RUSTFLAGS 不进",
     'BUILD_ENVIRONMENT_EXACT_VARIABLES = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")',
     'BUILD_ENVIRONMENT_EXACT_VARIABLES = ("RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")'),
    ("a8-no-configured-programs", "cargo 配置里 build.rustc / rustc-wrapper / target.<…>.runner 指的程序不进",
     '    try:\n        settings = tomllib.loads(content.decode("utf-8"))\n    except (UnicodeDecodeError, ValueError):\n        return []',
     '    try:\n        settings = tomllib.loads(content.decode("utf-8"))\n    except (UnicodeDecodeError, ValueError):\n        return []\n    return []'),
    ("a9-control-no-rustflags", "（对照）RUSTFLAGS 不进：字面值样本的环境里设了它，应当翻",
     'BUILD_ENVIRONMENT_EXACT_VARIABLES = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")',
     'BUILD_ENVIRONMENT_EXACT_VARIABLES = ("CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")'),
]
for name, what, old, new in MUTANTS:
    target = os.path.join(into, name)
    shutil.rmtree(target, ignore_errors=True)
    subprocess.run(["rsync", "-a", base.rstrip("/") + "/", target + "/"], check=True)
    path = os.path.join(target, TARGET)
    text = open(path, encoding="utf-8").read()
    if text.count(old) != 1:
        sys.exit(f"{name}：旧串命中 {text.count(old)} 次，不是 1")
    open(path, "w", encoding="utf-8").write(text.replace(old, new))
    print(f"{name}\t{what}")
