#!/usr/bin/env python3
"""H1 第 5 条旁支：驱动 global_key_lines（门禁与驱动共用）对哪些编译配置不变，而 admission.py build_environment_lines（仓里另一份「构建环境」定义）会变。
用法：env_gap.py <树根> <草稿目录>（假 HOME 建在草稿目录里）"""
import hashlib, importlib.util, os, sys, tempfile
root = sys.argv[1]
def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path); module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module
bm = load("bm", os.path.join(root, "research/scripts/broad-mutation.py"))
ad = load("ad", os.path.join(root, "research/scripts/admission.py"))
manifest = {"Cargo.toml": "a" * 64, "Cargo.lock": "b" * 64, "crates/x/src/lib.rs": "c" * 64}
base_env = {"HOME": "/nonexistent-home", "PATH": os.environ.get("PATH", "")}
def gk(env):
    text = bm.global_key_lines("r", "rustc x\n", manifest, env, ["tool-a"], "none", "limits cpu=3.0x+30.0 wall=10.0x+300.0")
    return hashlib.sha256(text.encode()).hexdigest()[:16]
def ab(env):
    lines = ad.build_environment_lines(tempfile.gettempdir(), env)
    return hashlib.sha256(repr(lines).encode()).hexdigest()[:16]
cases = [("（不设）", {}),
         ("RUSTFLAGS=-C overflow-checks=off", {"RUSTFLAGS": "-C overflow-checks=off"}),
         ("CARGO_PROFILE_DEV_OVERFLOW_CHECKS=false", {"CARGO_PROFILE_DEV_OVERFLOW_CHECKS": "false"}),
         ("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS=-C overflow-checks=off", {"CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS": "-C overflow-checks=off"}),
         ("RUSTC_WRAPPER=/usr/bin/env", {"RUSTC_WRAPPER": "/usr/bin/env"}),
         ("CARGO_BUILD_TARGET=x86_64-unknown-linux-musl", {"CARGO_BUILD_TARGET": "x86_64-unknown-linux-musl"})]
home = tempfile.mkdtemp(prefix="env-gap-home.", dir=sys.argv[2])
os.makedirs(os.path.join(home, ".cargo"))
for label, extra in cases:
    env = dict(base_env, **extra)
    print(f"{label}\tglobal_key={gk(env)}\tadmission_build_env={ab(env)}")
with open(os.path.join(home, ".cargo", "config.toml"), "w") as handle:
    handle.write("[profile.dev]\noverflow-checks = false\n")
env = dict(base_env, HOME=home)
print(f"~/.cargo/config.toml 写 [profile.dev] overflow-checks = false（HOME={os.path.basename(home)}）\tglobal_key={gk(env)}\tadmission_build_env={ab(env)}")
env = dict(base_env, HOME=home); os.remove(os.path.join(home, ".cargo", "config.toml"))
print(f"同一个 HOME、删掉那份 config.toml\tglobal_key={gk(env)}\tadmission_build_env={ab(env)}")
