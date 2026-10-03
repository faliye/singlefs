#!/usr/bin/env python3
"""攻方 J1-c 续：在每份 admission.py 写错版的拷贝上，载入那份拷贝里的驱动，按驱动的 build_environment_of 在一棵小树上算构建环境，
逐类设一个变量或放一份文件，看全局键里的构建环境变不变（变 = 驱动真跑会退 7、门禁会红；不变 = 旧结论照沿用）。
用法：j1c-env-gap.py <放拷贝的目录> <草稿目录> <名字…>"""
import importlib.util, os, shutil, sys
into, work = sys.argv[1:3]; names = sys.argv[3:]
root = os.path.join(work, "j1c-tree", "home", "broad-mutation", "tree")
shutil.rmtree(os.path.join(work, "j1c-tree"), ignore_errors=True)
os.makedirs(os.path.join(root, ".cargo")); os.makedirs(os.path.join(work, "j1c-tree", "cargo-home"))
open(os.path.join(root, ".cargo", "config.toml"), "w").write("[build]\njobs = 1\n")
wrapper = os.path.join(work, "j1c-tree", "wrap.sh"); open(wrapper, "w").write("#!/bin/sh\nexec \"$@\"\n"); os.chmod(wrapper, 0o755)
base_env = {"HOME": os.path.join(work, "j1c-tree"), "CARGO_HOME": os.path.join(work, "j1c-tree", "cargo-home"), "PATH": os.environ["PATH"], "RUSTUP_HOME": os.environ.get("RUSTUP_HOME") or os.path.expanduser("~/.rustup")}
CASES = [
    ("CARGO_PROFILE_DEV_OVERFLOW_CHECKS=false", lambda: dict(base_env, CARGO_PROFILE_DEV_OVERFLOW_CHECKS="false"), None),
    ("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=wrap.sh", lambda: dict(base_env, CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=wrapper), None),
    ("库目录上一层放 rust-toolchain.toml", lambda: base_env, ("rust-toolchain.toml", "[toolchain]\nchannel = \"nightly\"\n")),
    ("RUSTC_WRAPPER 值不变、程序内容改了", lambda: dict(base_env, RUSTC_WRAPPER=wrapper), "rewrite-wrapper"),
    ("库目录那一层放 .cargo/config.toml（[profile.dev] overflow-checks = false）", lambda: base_env, (".cargo/config.toml", "[profile.dev]\noverflow-checks = false\n")),
    ("RUSTC=rustc", lambda: dict(base_env, RUSTC="rustc"), None),
    ("CARGO_ENCODED_RUSTFLAGS=-Coverflow-checks=off", lambda: dict(base_env, CARGO_ENCODED_RUSTFLAGS="-Coverflow-checks=off"), None),
    ("树里 .cargo/config.toml 的 build.rustc-wrapper 指的程序内容改了", lambda: base_env, "rewrite-configured"),
]
def load(name):
    spec = importlib.util.spec_from_file_location(f"d_{name}", os.path.join(into, name, "research/scripts/broad-mutation.py"))
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module
for name in names:
    d = load(name)
    changed_any = []
    for label, env_of, extra in CASES:
        open(wrapper, "w").write("#!/bin/sh\nexec \"$@\"\n")
        open(os.path.join(root, ".cargo", "config.toml"), "w").write("[build]\njobs = 1\n" + (f"rustc-wrapper = \"{wrapper}\"\n" if extra == "rewrite-configured" else ""))
        before = d.build_environment_of(root, env_of() if extra in ("rewrite-wrapper", "rewrite-configured") else base_env)
        placed = None
        if isinstance(extra, tuple):
            placed = os.path.join(os.path.dirname(root), extra[0]); os.makedirs(os.path.dirname(placed), exist_ok=True); open(placed, "w").write(extra[1])
            after = d.build_environment_of(root, base_env)
        elif extra in ("rewrite-wrapper", "rewrite-configured"):
            open(wrapper, "w").write("#!/bin/sh\n# 换了内容\nexec \"$@\"\n")
            after = d.build_environment_of(root, env_of())
        else:
            after = d.build_environment_of(root, env_of())
        if placed:
            os.remove(placed)
        changed_any.append(f"{label}：{'变' if before != after else '不变'}")
    print(f"{name}\t" + "；".join(changed_any))
