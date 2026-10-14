# R3 / R4 的静态证据：在改前、改后两棵树里 grep -nF 同一句，打行号（没有打 -）；再数 --test-threads 在各脚本里的次数、--lib 与挂特性测试文件里 #[test] 的个数。
# 用法：python3 -I r3_r4_static.py <改前树> <改后树> <工作目录>
import os, re, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as c
trees, W = c.parse_trees(sys.argv)

def first_line(path, needle):
    try:
        for number, line in enumerate(open(path, encoding="utf-8"), 1):
            if needle in line: return number
    except OSError: return "文件不在"
    return "-"

def count(path, needle):
    try: return sum(1 for line in open(path, encoding="utf-8") if needle in line)
    except OSError: return "文件不在"

facts = [
    ("R3-gate-gpu-test-arguments-set", ".claude/gate.d/checker-tier-layer0-replay.sh", "gpu_test_arguments=(--test-threads 1)"),
    ("R3-gate-feature-pass-command", ".claude/gate.d/checker-tier-layer0-replay.sh", "--lib \"${target_arguments[@]}\" -- --nocapture ${gpu_test_arguments[@]"),
    ("R3-gate-comment-cpu-side-claim", ".claude/gate.d/checker-tier-layer0-replay.sh", "CPU 一侧判器的线程照 SINGLEFS_THREAD_CAP 与登记值"),
    ("R3-quick-test-threads-set", "scripts/crash/crash-point-replay.sh", "test_threads_arguments=(--test-threads 1)"),
    ("R3-quick-header-every-trip", "scripts/crash/crash-point-replay.sh", "开了 GPU 的每一趟快档测试参数带"),
    ("R3-quick-header-cpu-side-claim", "scripts/crash/crash-point-replay.sh", "CPU 一侧判器的线程照 capped.sh default 交的登记值"),
    ("R3-quick-two-hosts-branch", "scripts/crash/crash-point-replay.sh", "run_fast_tier_target_on_two_hosts \"$target\" \"$owners\""),
    ("R4-verif-20wan", ".claude/rules/verification.md", "日常验证（harness 档与轻型三条）的崩溃状态数压在 20 万量级"),
    ("R4-verif-gpu-binary-one-thread", ".claude/rules/verification.md", "单个 GPU 测试二进制只用本机点名的卡，带 `--test-threads 1` 跑"),
    ("R4-verif-full-thread-line", ".claude/rules/verification.md", "GPU 对拍与内存对拍不压并发：开满默认线程"),
    ("R4-verif-two-hosts-all-cards", ".claude/rules/verification.md", "崩溃放量与对拍驱动照多机配置用两台与全部卡"),
    ("R4-verif-old-20wan", ".claude/rules/verification.md", "日常验证的规模压在 20 万态量级"),
    ("R4-verif-old-all-cards", ".claude/rules/verification.md", "配置里的机器与卡都用"),
    ("R4-main-agent-no-narrowing", ".claude/main-agent.md", "挂钟长不是收窄它的理由"),
]
for cid, path, needle in facts:
    cells = " ".join(f"{label}_line={first_line(f'{tree}/{path}', needle)}" for label, tree in trees)
    print(f"name=R3R4 fact={cid} file={path} {cells}")
for path in ["scripts/crash/crash-amplification-two-hosts.sh", "scripts/gate/admission.py", "scripts/crash/crash-amplification-item.sh", "scripts/crash/crash-amplification.sh",
             "scripts/crash/crash-amplification-lib.sh", "scripts/crash/gpu-quiesce-run.sh", "scripts/crash/crash-point-replay.sh", ".claude/gate.d/checker-tier-layer0-replay.sh"]:
    cells = " ".join(f"{label}_count={count(f'{tree}/{path}', '--test-threads')}" for label, tree in trees)
    print(f"name=R3R4 count=--test-threads file={path} {cells}")
tree = trees[1][1]
tier = f"{tree}/crates/singlefs-checker-tier"
def tests_in(paths):
    n = 0
    for p in paths:
        for line in open(p, encoding="utf-8", errors="replace"):
            if re.match(r"\s*#\[test\]", line): n += 1
    return n
lib_files = [os.path.join(d, f) for d, _, fs in os.walk(f"{tier}/src") if "/bin" not in d.replace(tier, "") for f in fs if f.endswith(".rs")]
gated = [f"{tier}/tests/{f}" for f in sorted(os.listdir(f"{tier}/tests")) if f.endswith(".rs") and re.search(r'feature = "gpu"', open(f"{tier}/tests/{f}", encoding="utf-8", errors="replace").read())]
print(f"name=R3R4 count=test-attributes lib_src={tests_in(lib_files)} gpu_gated_files={len(gated)} gpu_gated_tests={tests_in(gated)}")
cpu_var = "SINGLEFS_CRASH_JUDGE_CPU_THREADS"
print(f"name=R3R4 fact=cpu-side-variable-of-gpu-agreement-test var_in_capped_sh={count(f'{tree}/scripts/hosts/capped.sh', cpu_var)} "
      f"reads_thread_cap_in_test={count(f'{tier}/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs', 'SINGLEFS_THREAD_CAP')} "
      f"fallback_available_parallelism_line={first_line(f'{tier}/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs', 'std::thread::available_parallelism()')}")
