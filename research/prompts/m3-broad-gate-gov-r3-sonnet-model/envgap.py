import importlib.util, hashlib, os, sys
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("drv", "/home/fy5090/code/singlefs/research/scripts/broad-mutation.py")
d = importlib.util.module_from_spec(spec); spec.loader.exec_module(d)
root = "/tmp/claude-1000/m3-gov-r3-defense/envroot/tree"; home = "/tmp/claude-1000/m3-gov-r3-defense/envhome"
base = {"PATH": os.environ["PATH"], "HOME": home}
def key(env):
    be = d.build_environment_of(root, env)
    t = d.global_key_lines(d.JUDGE_RULES_VERSION, "rustc x\n", {"Cargo.toml": "aa"}, be, ["tool a"], "none", "limits cpu=3x+30 wall=10x+300")
    return hashlib.sha256(t.encode()).hexdigest()[:16]
b = key(base); print("不设", b)
cases = [("RUSTFLAGS", "-C overflow-checks=off"), ("CARGO_PROFILE_DEV_OVERFLOW_CHECKS", "false"),
 ("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS", "-C overflow-checks=off"), ("RUSTC_WRAPPER", "/usr/bin/env"),
 ("CARGO_BUILD_TARGET", "x86_64-unknown-linux-musl"), ("RUSTDOCFLAGS", "--cfg x")]
changed = 0
for n, v in cases:
    k = key({**base, n: v}); changed += (k != b); print(n, k, "变了" if k != b else "没变")
os.makedirs(home + "/.cargo", exist_ok=True)
open(home + "/.cargo/config.toml", "w").write("[profile.dev]\noverflow-checks = false\n")
k = key(base); changed += (k != b); print("HOME/.cargo/config.toml", k, "变了" if k != b else "没变")
os.remove(home + "/.cargo/config.toml")
print("回到不设", key(base) == b)
print("对照（应不变）：")
for n, v in [("CARGO_BUILD_JOBS", "4"), ("FOO_UNRELATED", "1")]:
    print(n, key({**base, n: v}) == b)
print("变了", changed, "种（共 7 种）")
