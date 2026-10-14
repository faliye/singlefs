# 重型测试闸（D5-1 的 -p 探针表、bench、直接执行二进制；D5-2 的合入锁）：第一轮正推腿的喂法在改前、改后两棵树上重喂。
# 用法：python3 -I r5_heavy_guard.py <改前树> <改后树> <工作目录>
import importlib.util, json, os, shutil, subprocess, sys, tempfile
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as c
trees, W = c.parse_trees(sys.argv)

def feed(tree, agent, cwd, command, env=None):
    payload = {"tool_name": "Bash", "cwd": cwd, "tool_input": {"command": command}, "session_id": "s-d5"}
    if agent != "-": payload["agent_type"] = agent
    e = dict(os.environ, CLAUDE_PROJECT_DIR=tree, AGENT_HOOK_DETECTIONS=f"{W}/detections.jsonl")
    e.pop("SINGLEFS_HEAVY_TESTS", None)
    if env: e.update(env)
    r = subprocess.run(["bash", f"{tree}/.claude/hooks/heavy-test-guard.sh"], input=json.dumps(payload), capture_output=True, text=True, env=e)
    return r.returncode

MEM = "bash scripts/hosts/run-with-memory-cap.sh 4G "
for label, tree in trees:
    P = f"{W}/probes-{label}"; shutil.rmtree(P, ignore_errors=True)
    for name, dep in (("linked", f'singlefs-checker-tier={{path="{tree}/crates/singlefs-checker-tier"}}'),
                      ("gitdep", 'singlefs-checker-tier={git="https://example.invalid/x.git"}'), ("plain", "")):
        os.makedirs(f"{P}/{name}/src", exist_ok=True)
        open(f"{P}/{name}/src/main.rs", "w").write("fn main(){}\n")
        open(f"{P}/{name}/Cargo.toml", "w").write(f'[package]\nname="attack-probe"\nversion="0.1.0"\nedition="2021"\n[dependencies]\n{dep}\n' if name != "plain" else
                                                   '[package]\nname="plain-probe"\nversion="0.1.0"\nedition="2021"\n')
    os.makedirs(f"{P}/ws/src", exist_ok=True)
    open(f"{P}/ws/Cargo.toml", "w").write(f'[workspace]\nmembers=["m"]\nresolver="2"\n')
    os.makedirs(f"{P}/ws/m/src", exist_ok=True)
    open(f"{P}/ws/m/src/main.rs", "w").write("fn main(){}\n")
    open(f"{P}/ws/m/Cargo.toml", "w").write(f'[package]\nname="ws-member"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nsinglefs-checker-tier={{path="{tree}/crates/singlefs-checker-tier"}}\n')
    os.makedirs(f"{P}/linked/target/release", exist_ok=True); open(f"{P}/linked/target/release/attack-probe", "w").write("")
    cases = [
        ("test --manifest-path linked", "implementation-writer", W, f"cargo test --manifest-path {P}/linked/Cargo.toml"),
        ("run --manifest-path linked", "implementation-writer", W, f"cargo run --manifest-path {P}/linked/Cargo.toml"),
        ("test --manifest-path gitdep", "implementation-writer", W, f"cargo test --manifest-path {P}/gitdep/Cargo.toml"),
        ("nextest --manifest-path linked", "implementation-writer", W, f"cargo nextest run --manifest-path {P}/linked/Cargo.toml"),
        ("cwd linked: cargo test", "implementation-writer", f"{P}/linked", "cargo test"),
        ("main agent no prefix: cd linked && cargo run", "-", W, f"cd {P}/linked && cargo run --release"),
        ("main agent with commit prefix: cargo run linked", "-", f"{P}/linked", "SINGLEFS_HEAVY_TESTS=commit cargo run"),
        ("test --manifest-path plain", "implementation-writer", W, f"cargo test --manifest-path {P}/plain/Cargo.toml"),
        ("D5-1 cwd linked: cargo test -p attack-probe", "implementation-writer", f"{P}/linked", "cargo test -p attack-probe"),
        ("D5-1 cwd W: -p attack-probe --manifest-path linked", "implementation-writer", W, f"cargo test -p attack-probe --manifest-path {P}/linked/Cargo.toml"),
        ("D5-1 cwd linked: cargo run -p attack-probe", "implementation-writer", f"{P}/linked", "cargo run -p attack-probe"),
        ("D5-1 cwd linked: cargo test --package attack-probe", "implementation-writer", f"{P}/linked", "cargo test --package attack-probe"),
        ("D5-1 cwd linked: cargo run --package=attack-probe", "implementation-writer", f"{P}/linked", "cargo run --package=attack-probe"),
        ("D5-1 bench --manifest-path linked", "implementation-writer", W, f"cargo bench --manifest-path {P}/linked/Cargo.toml"),
        ("D5-1 build --manifest-path linked (compile only)", "implementation-writer", W, f"cargo build --manifest-path {P}/linked/Cargo.toml"),
        ("D5-1 exec compiled probe binary", "implementation-writer", W, f"{P}/linked/target/release/attack-probe"),
        ("D5-1 virtual workspace root: cargo run", "implementation-writer", f"{P}/ws", "cargo run"),
        ("X plain probe cwd: cargo test -p plain-probe", "implementation-writer", f"{P}/plain", "cargo test -p plain-probe"),
        ("X -p singlefs-checker-tier from tree root", "implementation-writer", tree, "cargo test -p singlefs-checker-tier --lib"),
        ("X -p singlefs-harness from tree root (harness tier)", "implementation-writer", tree, "cargo test -p singlefs-harness --lib"),
        ("X cwd checker-tier crate: cargo test -p singlefs-harness --lib", "implementation-writer", f"{tree}/crates/singlefs-checker-tier", "cargo test -p singlefs-harness --lib"),
        ("X cwd harness crate: cargo test --lib", "implementation-writer", f"{tree}/crates/singlefs-harness", "cargo test --lib"),
        ("X cwd checker-tier crate: cargo test --lib", "implementation-writer", f"{tree}/crates/singlefs-checker-tier", "cargo test --lib"),
        ("X tree root: cargo bench", "implementation-writer", tree, "cargo bench"),
        ("X tree root: cargo bench -p singlefs-harness", "implementation-writer", tree, "cargo bench -p singlefs-harness"),
        ("X tree root: cargo bench --no-run", "implementation-writer", tree, "cargo bench --no-run"),
    ]
    for cid, agent, cwd, cmd in cases:
        full = cmd if cmd.startswith(("cd ", "SINGLEFS")) else MEM + cmd
        print(f"name=R5 tree={label} case={cid!r} rc={feed(tree, agent, cwd, full)}")
    # 同名探针：探针包叫 singlefs-harness（工作区成员同名），当前目录在探针里 cargo test -p singlefs-harness
    os.makedirs(f"{P}/shadow/src", exist_ok=True); open(f"{P}/shadow/src/main.rs", "w").write("fn main(){}\n")
    open(f"{P}/shadow/Cargo.toml", "w").write(f'[package]\nname="singlefs-harness"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nsinglefs-checker-tier={{path="{tree}/crates/singlefs-checker-tier"}}\n')
    print(f"name=R5 tree={label} case='X shadow probe named like a workspace member: cwd probe, -p singlefs-harness' rc={feed(tree, 'implementation-writer', f'{P}/shadow', MEM + 'cargo test -p singlefs-harness')}")

# 合入锁（D5-2）：lib_workspace_cargo_lock.py 两树同一份（sha256 相同则 same_lib=True）
shas = {label: subprocess.run(["sha256sum", f"{tree}/.claude/hooks/lib_workspace_cargo_lock.py"], capture_output=True, text=True).stdout.split()[0] for label, tree in trees}
print(f"name=R5 case=lock-lib-same-file same_lib={shas['before'] == shas['after']}")
for label, tree in trees:
    spec = importlib.util.spec_from_file_location("lcl_" + label, f"{tree}/.claude/hooks/lib_workspace_cargo_lock.py")
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    work = tempfile.mkdtemp(prefix="d5-lock-", dir=W)
    main, holder, lockfile = m.build_held_workspace(work)
    os.makedirs(f"{main}/scripts", exist_ok=True)
    open(f"{main}/build.sh", "w").write("#!/usr/bin/env bash\ncargo build\n")
    copy = os.path.join(work, "copy"); shutil.copytree(main, copy)
    env = {"SINGLEFS_WORKSPACE_LOCK_FILE": lockfile}
    lock_cases = [("sub-agent cargo build in main workspace", "implementation-writer", main, "cargo build"), ("main agent cargo build", "-", main, "cargo build"),
                  ("sub-agent cargo --version", "implementation-writer", main, "cargo --version"), ("sub-agent bash build.sh", "implementation-writer", main, "bash build.sh"),
                  ("sub-agent cargo fmt", "implementation-writer", main, "cargo fmt"), ("sub-agent cargo metadata", "implementation-writer", main, "cargo metadata"),
                  ("sub-agent cargo tree", "implementation-writer", main, "cargo tree"), ("sub-agent cp -r copy with .git: cargo build", "implementation-writer", copy, "cargo build")]
    try:
        for cid, agent, cwd, cmd in lock_cases:
            print(f"name=R5 tree={label} case='lock: {cid}' rc={feed(tree, agent, cwd, cmd, env=env)}")
    finally:
        holder.kill(); holder.wait(); shutil.rmtree(work, ignore_errors=True)
