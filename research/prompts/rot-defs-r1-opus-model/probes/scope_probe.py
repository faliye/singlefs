"""rot-defs-r1 攻方腿：照改后的定义写出的命令喂给 run-scope-guard.sh，看放不放（只喂钩子，不起 cargo、不跑门禁）。
用法：python3 scope_probe.py <仓根> <输出目录>；检出记录写进输出目录，不写默认的那一份。"""
import json, os, subprocess, sys
repo, work = sys.argv[1], sys.argv[2]
R = "为什么=这一批只改了这一处，只跑它看改动有没有弄坏东西"
cases = [
 ("kb-scribe 照第 3 步 doc-decisions --write，不带声明", "kb-scribe", "bash .claude/gate.d/doc-decisions.sh --write"),
 ("kb-scribe doc-decisions --write，带 whole 声明", "kb-scribe", f"SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=doc-decisions 重生成; 范围=整轮; {R}' bash .claude/gate.d/doc-decisions.sh --write"),
 ("kb-scribe doc-decisions --write --check decision-items-sync，带 items 声明", "kb-scribe", f"SINGLEFS_RUN_SCOPE='种类=items; 跑什么=doc-decisions 重生成; 范围=1 格; {R}' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync"),
 ("主 agent doc-decisions --write，不带声明", None, "bash .claude/gate.d/doc-decisions.sh --write"),
 ("experiment-runner 照 1b 在 scripts/experiments 下 cargo test，不带声明", "experiment-runner", "cd scripts/experiments && cargo test --release"),
 ("experiment-runner scripts/experiments 下 cargo test --bin，带 target 声明", "experiment-runner", f"cd scripts/experiments && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=e999 单测; 范围=1 个测试目标; {R}' cargo test --release --bin e999-sample"),
 ("experiment-runner 第 3 步 mutate.sh，不带声明", "experiment-runner", "cd scripts/experiments && nice -n 19 bash ../mutation/mutate.sh e999-sample src/bin/e999_sample.rs ../mutation/tables/e999_sample.tsv"),
 ("implementation-writer 跑动到的测试二进制，不带声明", "implementation-writer", "bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-harness --test model_differential"),
 ("investigator 原样复现整包 cargo test，带 whole 声明", "investigator", f"SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=core 整包; 范围=整包; {R}' bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-core"),
 ("three-way-attack 照写范围 cargo test -p --test，不带声明", "three-way-attack", "bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-harness --test my_prototype"),
 ("implementation-writer 第 4 步一条命令，开头 export 一份 target 声明", "implementation-writer",
  f"export SINGLEFS_RUN_SCOPE='种类=target; 跑什么=harness 一个目标; 范围=1 个测试目标; {R}'; bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-harness --test model_differential; bash scripts/hosts/run-with-memory-cap.sh 8G bash .claude/gate.d/checker-tier-formal-verification.sh --quick ."),
 ("implementation-writer 第 4 步一条命令，每段各带自己的声明", "implementation-writer",
  f"cargo fmt --all -- --check; SINGLEFS_RUN_SCOPE='种类=target; 跑什么=harness 一个目标; 范围=1 个测试目标; {R}' bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-harness --test model_differential; SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=形式化轻型; 范围=轻型核心部分; {R}' bash scripts/hosts/run-with-memory-cap.sh 8G bash .claude/gate.d/checker-tier-formal-verification.sh --quick ."),
 ("主 agent 照 main-agent 第 94 行起 --full，不带声明", None, "SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/checker-tier-crash-point-replay.sh --full ."),
]
env = dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "scope-detections.jsonl"), CLAUDE_PROJECT_DIR=repo)
for label, agent, command in cases:
    hook_input = {"tool_name": "Bash", "tool_input": {"command": command}, "cwd": repo, "session_id": "probe-rot-defs-r1-opus"}
    if agent:
        hook_input["agent_type"] = agent
    result = subprocess.run(["bash", os.path.join(repo, ".claude/hooks/run-scope-guard.sh")], input=json.dumps(hook_input), text=True, capture_output=True, env=env)
    lines = [line for line in result.stderr.splitlines() if line.strip()][:4]
    print(f"[{result.returncode}] {label}")
    for line in lines:
        print("      " + line.replace(repo, "<仓根>")[:260])
