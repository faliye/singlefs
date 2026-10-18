"""rot-defs-r1 攻方腿：照 three-way-attack.md 3b 与「写范围」写一个调 enumerate_crash_states 的原型包（只建 Cargo.toml 与空 main，不编），
把起它的几种命令以 three-way-attack 的身份喂给 heavy-test-guard.sh，看放不放。
用法：python3 heavy_probe.py <仓根> <输出目录>；原型包建在输出目录里，检出记录写进输出目录。"""
import json, os, subprocess, sys
repo, work = sys.argv[1], sys.argv[2]
proto = os.path.join(work, "proto-crate")
os.makedirs(os.path.join(proto, "src"), exist_ok=True)
with open(os.path.join(proto, "Cargo.toml"), "x", encoding="utf-8") as handle:
    handle.write(f"""[package]
name = "attack-prototype-probe"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
singlefs-checker-tier = {{ path = "{repo}/crates/singlefs-checker-tier" }}
singlefs-harness = {{ path = "{repo}/crates/singlefs-harness" }}
""")
with open(os.path.join(proto, "src", "main.rs"), "x", encoding="utf-8") as handle:
    handle.write("fn main() {}\n")
R = "为什么=原型只造一段小流，跑它看这一段历史的崩溃状态判没判对"
cases = [
 ("three-way-attack 在调 enumerate_crash_states 的原型包里 cargo run（照 3b 与写范围）",
  f"SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=原型; 范围=整包; {R}' bash {repo}/scripts/hosts/capped.sh 31 bash {repo}/scripts/hosts/run-with-memory-cap.sh 8G cargo run --release"),
 ("three-way-attack 同一原型包 cargo test --test，带 target 声明",
  f"SINGLEFS_RUN_SCOPE='种类=target; 跑什么=原型; 范围=1 个测试目标; {R}' bash {repo}/scripts/hosts/capped.sh 31 bash {repo}/scripts/hosts/run-with-memory-cap.sh 8G cargo test --release --test attack_prototype"),
 ("three-way-attack 同一原型包只 cargo build", f"bash {repo}/scripts/hosts/capped.sh 31 cargo build --release"),
 ("three-way-attack 直接执行原型包编出来的二进制", f"bash {repo}/scripts/hosts/run-with-memory-cap.sh 8G ./target/release/attack-prototype-probe"),
]
env = dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "heavy-detections.jsonl"), CLAUDE_PROJECT_DIR=repo)
for label, command in cases:
    hook_input = {"tool_name": "Bash", "tool_input": {"command": command}, "cwd": proto, "session_id": "probe-rot-defs-r1-opus", "agent_type": "three-way-attack"}
    result = subprocess.run(["bash", os.path.join(repo, ".claude/hooks/heavy-test-guard.sh")], input=json.dumps(hook_input), text=True, capture_output=True, env=env, cwd=proto)
    lines = [line for line in result.stderr.splitlines() if line.strip()][:2]
    print(f"[{result.returncode}] {label}")
    for line in lines:
        print("      " + line.replace(work, "<输出目录>").replace(repo, "<仓根>")[:300])
