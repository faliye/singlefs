#!/usr/bin/env python3
# rerun.sh 调的主体：可续跑。见 rerun.sh 文件头。
import os, re, shutil, subprocess, sys, time, hashlib

REPO = os.environ["SINGLEFS_REPO"]
FROZEN = os.environ["FROZEN_TREE"]
WORK = os.environ["WORK"]
TREE = f"{WORK}/tree"
TARGET = f"{WORK}/target"
LOG = f"{WORK}/log"
DONE = f"{WORK}/done"
SUMMARY = f"{WORK}/summary.txt"
SNAPSHOT = f"{REPO}/research/prompts/m3-batch21-code-r1-snapshot/crates-sha256.txt"
CAP_SCRIPT = f"{REPO}/scripts/run-with-memory-cap.sh"
THREADS_SCRIPT = f"{REPO}/scripts/capped.sh"
CALL_BUDGET = 215.0  # 秒：一次调用至多干这么久（前台命令不超过 240 秒）
START = time.time()
for d in (LOG, DONE):
    os.makedirs(d, exist_ok=True)

# 本批点名要验的变异行（冻结副本 crates/mutations.tsv 的行号）：A 两行、W 三行、R 的 N3/N4/Q-F/C2/C479 各行、C2 替换行。G 的行在 checker 档，不跑。
ROWS = [2749, 2750, 2751, 2752, 2753, 2757, 2758, 2759, 2760, 2761, 2762, 2672, 2764, 2763, 302]
# 基线：本批动到的 harness / core 目标（整份跑）
BASELINES = [
    ("core-lib-watermark", ["-p", "singlefs-core", "--lib", "--", "inode_number_watermark"]),
    ("pool_checker_judges", ["-p", "singlefs-harness", "--test", "pool_checker_judges_the_deletion_state_of_every_walked_version"]),
    ("wm_without_previous", ["-p", "singlefs-harness", "--test", "inode_number_watermark_switch_on_a_publish_without_a_previous_version"]),
    ("wm_pressed_back", ["-p", "singlefs-harness", "--test", "inode_number_watermark_pressed_back_reissues_a_number"]),
    ("unlink_mount_continuation", ["-p", "singlefs-harness", "--test", "unlink_mount_continuation_and_the_emptied_version"]),
    ("unlink_writes_fast", ["-p", "singlefs-harness", "--test", "unlink_writes_per_transaction_do_not_grow_with_the_file"]),
    ("crashed_unmount", ["-p", "singlefs-harness", "--test", "the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads"]),
    ("write_accounting", ["-p", "singlefs-harness", "--test", "write_accounting_per_structure"]),
    ("writable_mount_second_instance", ["-p", "singlefs-harness", "--test", "writable_mount_takes_a_second_instance"]),
    ("many_inodes", ["-p", "singlefs-harness", "--test", "many_inodes_write_and_read"]),
    ("unlink_in_histories", ["-p", "singlefs-harness", "--test", "unlink_in_histories_judged_by_the_model"]),
    ("checker_known_bad_images", ["-p", "singlefs-harness", "--test", "checker_known_bad_images"]),
    ("harness-lib", ["-p", "singlefs-harness", "--lib"]),
    ("unlink_writes_slow", ["-p", "singlefs-harness", "--test", "unlink_writes_per_transaction_do_not_grow_with_the_file", "--", "--include-ignored"]),
]

failures = []

def remaining():
    return CALL_BUDGET - (time.time() - START)

def out_of_time():
    print(f"本次调用的时间用完（已 {time.time()-START:.0f} 秒），原样再调一次 rerun.sh 接着做", flush=True)
    sys.exit(99)

def done(step):
    return os.path.exists(f"{DONE}/{step}")

def mark(step, line):
    with open(f"{DONE}/{step}", "w", encoding="utf-8") as f:
        f.write(line + "\n")
    with open(SUMMARY, "a", encoding="utf-8") as f:
        f.write(f"{step}\t{line}\n")
    print(f"[{step}] {line}", flush=True)

def cargo(args, cap, label):
    """经 run-with-memory-cap.sh（cap）与 capped.sh（线程 10）跑一条 cargo；交回 (退出码, stdout, stderr)。"""
    limit = int(remaining())
    if limit < 40:
        out_of_time()
    env = dict(os.environ, CARGO_TARGET_DIR=TARGET,
               RUN_WITH_MEMORY_CAP_TIME_LIMIT=str(limit),
               RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方腿正推腿各跑各的副本与编译目录",
               RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示显式给编译 6G、跑 4G，不用 auto")
    cmd = ["nice", "-n", "19", "bash", CAP_SCRIPT, cap, "bash", THREADS_SCRIPT, "10", "cargo"] + args
    r = subprocess.run(cmd, cwd=TREE, env=env, capture_output=True, text=True)
    with open(f"{LOG}/{label}.log", "w", encoding="utf-8") as f:
        f.write("CMD: " + " ".join(cmd) + f"\nEXIT: {r.returncode}\n--stdout--\n{r.stdout}\n--stderr--\n{r.stderr}\n")
    if r.returncode == 253:
        out_of_time()
    if r.returncode in (250, 251, 252, 254):
        print(f"包装自己的结局 {r.returncode}（不算结果）：{label}", flush=True)
        sys.exit(2)
    return r.returncode, r.stdout, r.stderr

def prepare():
    if done("prepare"):
        return
    rows = [l.split() for l in open(SNAPSHOT, encoding="utf-8").read().splitlines() if l.strip()]
    bad = 0
    for sha, path in rows:
        got = hashlib.sha256(open(f"{FROZEN}/{path}", "rb").read()).hexdigest()
        if got != sha:
            print(f"sha256 对不上：{path}", flush=True); bad += 1
    if bad or len(rows) != 22:
        print(f"冻结副本与快照清单对不上（{len(rows)} 份，{bad} 份不同）", flush=True)
        sys.exit(2)
    if os.path.exists(TREE):
        shutil.rmtree(TREE)
    os.makedirs(TREE)
    shutil.copytree(f"{FROZEN}/crates", f"{TREE}/crates")
    for name in ("Cargo.toml", "Cargo.lock"):
        shutil.copy(f"{FROZEN}/{name}", f"{TREE}/{name}")
    mark("prepare", f"22 份 sha256 全对，副本在 {TREE}")

def build():
    for step, args in (("build-core-lib", ["test", "--offline", "-p", "singlefs-core", "--lib", "--no-run"]),
                       ("build-harness-tests", ["test", "--offline", "-p", "singlefs-harness", "--no-run",
                                                 "--test", "pool_checker_judges_the_deletion_state_of_every_walked_version",
                                                 "--test", "inode_number_watermark_switch_on_a_publish_without_a_previous_version",
                                                 "--test", "inode_number_watermark_pressed_back_reissues_a_number",
                                                 "--test", "unlink_mount_continuation_and_the_emptied_version",
                                                 "--test", "unlink_writes_per_transaction_do_not_grow_with_the_file",
                                                 "--test", "the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads",
                                                 "--test", "write_accounting_per_structure",
                                                 "--test", "writable_mount_takes_a_second_instance",
                                                 "--test", "many_inodes_write_and_read"])):
        if done(step):
            continue
        rc, _out, err = cargo(args, "6G", step)
        if rc != 0:
            print(f"{step} 编译失败，见 {LOG}/{step}.log", flush=True)
            failures.append(step)
            mark(step, f"编译失败 exit={rc}")
            sys.exit(1)
        mark(step, "编译通过")

def summarize(out):
    res = [l for l in out.splitlines() if l.startswith("test result")]
    return res[-1] if res else "(无 test result 行)"

def baselines():
    for label, args in BASELINES:
        step = f"base-{label}"
        if done(step):
            continue
        rc, out, _err = cargo(["test", "--offline"] + args, "4G", step)
        ok = rc == 0
        if not ok:
            failures.append(step)
        mark(step, f"exit={rc} {summarize(out)}")

def first_panic(out, test):
    m = re.search(r"---- " + re.escape(test) + r" stdout ----\n(.*?)(?:\n\n|\Z)", out, re.S)
    if not m:
        return "(无 panic 段)"
    return " ".join(m.group(1).split())[:400]

def mutate(row_no):
    step = f"mut-{row_no}"
    if done(step):
        return
    rows = open(f"{FROZEN}/crates/mutations.tsv", encoding="utf-8").read().split("\n")
    name, path, old, new, args, test = rows[row_no - 1].split("\t")
    old = old.replace("\\n", "\n"); new = new.replace("\\n", "\n")
    pristine = open(f"{FROZEN}/{path}", encoding="utf-8").read()
    if pristine.count(old) != 1:
        failures.append(step)
        mark(step, f"锚点命中 {pristine.count(old)} 次，没跑")
        return
    target_file = f"{TREE}/{path}"
    argv = args.split()
    base_args = argv[:argv.index("--")] if "--" in argv else argv
    extra = ["--", "--include-ignored"] if "--include-ignored" in argv else []
    full = ["test", "--offline"] + base_args + extra  # 整份目标跑，列出红的全部用例名
    try:
        with open(target_file, "w", encoding="utf-8") as f:
            f.write(pristine.replace(old, new))
        rc_b, _o, err_b = cargo(["test", "--offline"] + base_args + ["--no-run"], "6G", f"{step}-build")
        if rc_b != 0:
            failures.append(step)
            mark(step, f"变异后编译不过 exit={rc_b}（见 {LOG}/{step}-build.log）")
            return
        rc, out, _e = cargo(full, "4G", step)
    finally:
        with open(target_file, "w", encoding="utf-8") as f:
            f.write(pristine)
    failed_names = re.findall(r"^test (\S+)(?: - should panic)? \.\.\. FAILED", out, re.M)
    named_failed = any(n.endswith(test) for n in failed_names)
    if not (rc != 0 and named_failed):
        failures.append(step)
    mark(step, f"exit={rc} 点名用例红={named_failed} 红的用例共 {len(failed_names)} 条：{failed_names[:8]} {summarize(out)} | 点名用例红在：{first_panic(out, test)}")

PROBE_FILE = "crates/singlefs-harness/tests/probe_n8_unraised_and_two_inodes.rs"
PROBE_TEST = "probe_n8_unraised_and_two_inodes"

def make_probe_n8():
    """N8 文件头与变体 c 文档里两句话的探针：前半从入库的用例文件机械剪出来（把抬 F 那一段拿掉），后半是 probe_n8_tail.rs.txt。
    ① 不抬 F 时池级 checker 还走着意图那条根，I-9.6 先红在树内最大 key 那一半；
    ② 压回 2 建两个（p + n = 4 大于两处最大号）重发 2、3 号，I-9.6 成立。"""
    path = f"{TREE}/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs"
    src = open(path, encoding="utf-8").read()
    stop = src.index("/// C479 变体 c")
    start = src.index('    let publish_parameters = parameters();\n    let devices = pool.devices.as_mut().expect("镜像还开着");\n    let raised = raise_rollback_floor_to_the_admission_ceiling(')
    end = src.index("    (pool, txg_of_the_last_deletion_step)\n}")
    cut = src[:start] + src[end:stop]
    cut = cut.replace("mod common;\n", "#![allow(unused)]\nmod common;\n", 1)
    tail = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "probe_n8_tail.rs.txt"), encoding="utf-8").read()
    with open(f"{TREE}/{PROBE_FILE}", "w", encoding="utf-8") as f:
        f.write(cut + tail)

def probe_n1():
    if done("probe-n1"):
        return
    here = os.path.dirname(os.path.abspath(__file__))
    name = "probe_n1_legit_states"
    target = f"{TREE}/crates/singlefs-harness/tests/{name}.rs"
    shutil.copy(f"{here}/probe_n1_legit.rs.txt", target)
    try:
        rc_b, _o, _e = cargo(["test", "--offline", "-p", "singlefs-harness", "--test", name, "--no-run"], "6G", "probe-n1-build")
        if rc_b != 0:
            failures.append("probe-n1")
            mark("probe-n1", f"探针编译不过 exit={rc_b}（见 {LOG}/probe-n1-build.log）")
            return
        rc, out, _e = cargo(["test", "--offline", "-p", "singlefs-harness", "--test", name, "--", "--nocapture", "--test-threads=1"], "4G", "probe-n1")
        lines = re.findall(r"PROBE-N1-LEGIT[-A-Z]* .*", out)
        mark("probe-n1", f"exit={rc} 共 {len(lines)} 行")
        with open(f"{LOG}/probe-n1-lines.txt", "w", encoding="utf-8") as f:
            f.write("\n".join(lines) + "\n")
    finally:
        os.remove(target)

def probes():
    probe_n1()
    if done("probe-n8"):
        return
    make_probe_n8()
    rc_b, _o, _e = cargo(["test", "--offline", "-p", "singlefs-harness", "--test", PROBE_TEST, "--no-run"], "6G", "probe-n8-build")
    if rc_b != 0:
        failures.append("probe-n8")
        mark("probe-n8", f"探针编译不过 exit={rc_b}（见 {LOG}/probe-n8-build.log）")
        return
    rc, out, _e = cargo(["test", "--offline", "-p", "singlefs-harness", "--test", PROBE_TEST, "--", "--nocapture", "--test-threads=1"], "4G", "probe-n8")
    lines = re.findall(r"PROBE-N8-\w+ .*", out)
    mark("probe-n8", f"exit={rc} {lines}")
    os.remove(f"{TREE}/{PROBE_FILE}")

def main():
    prepare()
    build()
    baselines()
    for r in ROWS:
        mutate(r)
    if not done("probe-n7-bound"):
        rc, out, _e = cargo(["test", "--offline", "-p", "singlefs-harness", "--test",
                              "unlink_writes_per_transaction_do_not_grow_with_the_file", "--", "--nocapture"], "4G", "probe-n7-bound")
        lines = [l for l in out.splitlines() if "上界" in l]
        mark("probe-n7-bound", f"exit={rc} {lines}")
    probes()
    # 还原之后再跑一遍一条基线，确认副本回到了绿
    if not done("restored-check"):
        rc, out, _e = cargo(["test", "--offline", "-p", "singlefs-harness", "--test",
                              "inode_number_watermark_switch_on_a_publish_without_a_previous_version"], "4G", "restored-check")
        if rc != 0:
            failures.append("restored-check")
        mark("restored-check", f"exit={rc} {summarize(out)}")
    here = os.path.dirname(os.path.abspath(__file__))
    os.makedirs(f"{here}/out", exist_ok=True)
    shutil.copy(SUMMARY, f"{here}/out/summary.txt")
    for extra in ("probe-n1-lines.txt",):
        if os.path.exists(f"{LOG}/{extra}"):
            shutil.copy(f"{LOG}/{extra}", f"{here}/out/{extra}")
    # 变异 302（known_bad_images 那条注释里 131 072 的出处）：把 I-3.1 那一句抽出来
    if os.path.exists(f"{LOG}/mut-302.log"):
        hits = re.findall(r'\("I-3\.1", Violated\("[^"]*"\)\)', open(f"{LOG}/mut-302.log", encoding="utf-8").read())
        with open(f"{here}/out/mut-302-I-3.1.txt", "w", encoding="utf-8") as f:
            f.write("\n".join(sorted(set(hits))) + "\n")
    print("---- 汇总 ----")
    print(open(SUMMARY, encoding="utf-8").read())
    bad = [l for l in open(SUMMARY, encoding="utf-8").read().splitlines()
           if ("exit=0" not in l and l.startswith("base-")) or ("点名用例红=False" in l) or ("编译失败" in l) or ("编译不过" in l) or ("没跑" in l)]
    if bad:
        print("有不符合预期的步骤：")
        print("\n".join(bad))
        sys.exit(1)
    print("全部步骤符合预期（基线全绿、每行变异点名用例红）")
    sys.exit(0)

main()
