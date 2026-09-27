"""J1 探针：门禁 54 号（拷进临时小仓，不在 .claude/gate.d/ 下，拿假 cargo 跑）的快档与 --full 在「崩溃枚举用例的输入变了」时判不判得到。
复跑：python3 probe_j1_stage.py [--admission <admission.py>] [--stage <54 号>] [--scripts <research/scripts 目录>]
格：
  A1 只改登记表（新登记一条用例，测试目标早已提交）⇒ 快档应当核那一条的标记（红），不该退 77
  A2 只改一条用例登记行的第三列 ⇒ 同上
  A3 只改 54 号自己 ⇒ 同上
  A4 只改 admission.py 的判法（旧判法写的标记、新判法判那份日志为红）⇒ 指纹应当变、--full 应当重跑那一条
对照：同样的改动带 SINGLEFS_GATE_FULL=1 时快档判红（说明红是有的，是范围那一问把它摘掉了）；改 crates/ 下的文件快档不退 77。"""
import os
import shutil
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common  # noqa: E402

ADMISSION = probe_common.option("--admission", probe_common.DEFAULT_ADMISSION)
STAGE = probe_common.option("--stage", probe_common.DEFAULT_STAGE)
SCRIPTS = probe_common.option("--scripts", probe_common.DEFAULT_SCRIPTS)
module = probe_common.load_admission(ADMISSION)
cells = probe_common.Cells("J1 54 号")

CASES = [
    ("crash-case:stream-a", "first_transaction_step_seven_layer0", "stream_a_full", "count-line=LAYER0 exhaustive=LAYER0 threads=LAYER0"),
    ("crash-case:stream-b", "second_transaction_step_zero_layer0", "stream_b_full", "count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B"),
    ("crash-case:case-c", "case_c", "case_c_full", ""),
]
CASE_D_ROW = "crash-case:case-d\tcrates/ Cargo.toml\ttest=singlefs-harness:case_d:case_d_full\t# 新登记"


def row_of(key, target, function, conditions):
    return f"{key}\tcrates/ Cargo.toml\ttest=singlefs-harness:{target}:{function}{' ' + conditions if conditions else ''}\t# 样本"


work = tempfile.mkdtemp(prefix="opus-j1-stage-")
try:
    module.run_quietly(["git", "init", "-q", "-b", "master", work], module.GIT_IDENTITY)
    w = lambda relative, text: module.write_text(os.path.join(work, relative), text)  # noqa: E731
    w("Cargo.toml", '[workspace]\nmembers = ["crates/singlefs-harness"]\n')
    w("crates/singlefs-harness/Cargo.toml", '[package]\nname = "singlefs-harness"\nversion = "0.0.0"\n')
    w("crates/singlefs-harness/src/lib.rs", "pub fn one() -> u32 { 1 }\n")
    for _key, target, function, _conditions in CASES + [("", "case_d", "case_d_full", "")]:
        w(f"crates/singlefs-harness/tests/{target}.rs", f"#[test]\n#[ignore]\nfn {function}() {{}}\n")
    base_rows = ["54-layer0-replay.sh\tcrates/ Cargo.toml\tcommand=cargo command=rustc\t# 样本"] + [row_of(*case) for case in CASES]
    table = os.path.join(work, module.REGISTRATION_TABLE)
    w(module.REGISTRATION_TABLE, "".join(row + "\n" for row in base_rows))
    os.makedirs(os.path.join(work, "research/scripts"))
    admission_copy = os.path.join(work, "research/scripts/admission.py")
    shutil.copy(ADMISSION, admission_copy)
    for helper in ("stage-must-run.sh", "change-touches-crates.sh"):
        shutil.copy(os.path.join(SCRIPTS, helper), os.path.join(work, "research/scripts", helper))
    stage_copy = os.path.join(work, ".claude/stage-under-test/54-layer0-replay.sh")
    os.makedirs(os.path.dirname(stage_copy))
    shutil.copy(STAGE, stage_copy)
    w(".gitignore", ".control/\n.tools/\n")
    tools, control = os.path.join(work, ".tools"), os.path.join(work, ".control")
    os.makedirs(control)
    module.write_executable(os.path.join(tools, "cargo"), module.FAKE_CARGO_FOR_STAGE)
    module.write_executable(os.path.join(tools, "rustc"), module.FAKE_TOOLCHAIN_SCRIPTS["rustc"])
    module.write_executable(os.path.join(tools, "nproc"), "#!/usr/bin/env bash\necho 32\n")
    environment = module.environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""),
                                                             "CARGO_HOME": os.path.join(tools, "cargo-home"), "FAKE_CARGO_CONTROL": control})
    for name in ("SINGLEFS_LAYER0_THREADS", "SINGLEFS_LAYER0_START_OVER", "SINGLEFS_GATE_FULL", "SINGLEFS_STAGED_TREE"):
        environment.pop(name, None)
    common_directory = os.path.join(work, ".git")

    def git(*arguments):
        return module.run_in_environment(["git", "-C", work, *arguments], dict(environment, **module.GIT_IDENTITY))

    def set_logs(stream_b=None):
        w(".control/log.first_transaction_step_seven_layer0", module.stream_log("LAYER0") + "CHECKER x=0\n")
        w(".control/log.second_transaction_step_zero_layer0", stream_b or module.stream_log("LAYER0B"))
        for target in ("case_c", "case_d"):
            w(f".control/log.{target}", module.PASSED_ONE_LINE + "\n")

    def full_runs():
        path = os.path.join(control, "invocations")
        if not os.path.exists(path):
            return []
        with open(path, encoding="utf-8") as handle:
            lines = [line.split("\t") for line in handle.read().split("\n") if line]
        os.remove(path)
        return [line[0] for line in lines if line[2] == "1"]

    def stage(*arguments, gate_full=False):
        env = dict(environment, **({"SINGLEFS_GATE_FULL": "1"} if gate_full else {}))
        exit_code, output, messages = module.run_in_environment(["bash", stage_copy, *arguments, work], env)
        return exit_code, output + messages, full_runs()

    # 被判的 54 号若把准入模块也当「判它的」进指纹（改法 F1），这里照它的参数算，算出与它同一个数
    judge_module_arguments = (["--extra-file", "<判它的准入模块：admission.py>", admission_copy]
                              if "<判它的准入模块：admission.py>" in open(stage_copy, encoding="utf-8").read() else [])

    def fingerprint(key):
        _exit_code, output, _messages = module.run_in_environment(
            [sys.executable, admission_copy, "crash-case-manifest", work, key, os.path.join(control, "m"),
             "--extra-file", "<判它的 54 号：54-layer0-replay.sh>", stage_copy, *judge_module_arguments, "--toolchain", "--build-environment"], environment)
        return output.split()[0] if output.split() else f"算不出：{output.strip()}"

    def last_line(output):
        lines = [line for line in output.strip().split("\n") if line.strip()]
        return lines[-1].strip()[:160] if lines else ""

    set_logs()
    git("add", "-A")
    git("commit", "-q", "-m", "baseline")
    exit_code, output, runs = stage("--full")
    cells.expect("CONTROL", "基线：--full 头一趟三条逐条跑、各写一格，退 0", exit_code == 0 and runs == [c[1] for c in CASES],
                 f"退 {exit_code}，跑了 {runs}；末行「{last_line(output)}」")
    exit_code, output, _runs = stage()
    cells.expect("CONTROL", "基线：干净工作树上快档退 77（文件头写明的「没碰」）", exit_code == 77, f"退 {exit_code}；末行「{last_line(output)}」")

    def restore():
        git("reset", "-q", "--hard", "HEAD")

    # A0 对照：改 crates/ 下的共用文件，快档不退 77、判红
    w("crates/singlefs-harness/src/lib.rs", "pub fn one() -> u32 { 2 }\n")
    git("add", "-A")
    exit_code, output, _runs = stage()
    cells.expect("CONTROL", "A0 改 crates/ 下的 src/lib.rs ⇒ 快档不退 77、判红（缺这批输入的标记）", exit_code == 1,
                 f"退 {exit_code}；末行「{last_line(output)}」")
    restore()

    scenarios = [
        ("A1 只改登记表：新登记一条用例 crash-case:case-d（它的测试目标 case_d.rs 早已提交）", "crash-case:case-d",
         lambda: w(module.REGISTRATION_TABLE, "".join(row + "\n" for row in base_rows + [CASE_D_ROW]))),
        ("A2 只改 stream-b 那一行的第三列（去掉 exhaustive=LAYER0B）", "crash-case:stream-b",
         lambda: w(module.REGISTRATION_TABLE, "".join(row + "\n" for row in base_rows).replace(" exhaustive=LAYER0B", ""))),
        ("A3 只改 54 号自己（加一行注释；它按「判它的 54 号」进每条用例的指纹）", "crash-case:stream-a",
         lambda: open(stage_copy, "a", encoding="utf-8").write("# probe edit\n")),
    ]
    for label, key, change in scenarios:
        change()
        git("add", "-A")
        exit_code, output, _runs = stage()
        cells.expect("ATTACK", f"{label} ⇒ 快档应当核这条用例的标记，不该退 77", exit_code != 77,
                     f"退 {exit_code}；末行「{last_line(output)}」")
        exit_code, output, _runs = stage(gate_full=True)
        cells.expect("CONTROL", f"{label}，带 SINGLEFS_GATE_FULL=1 ⇒ 快档判红并点名 {key}", exit_code == 1 and key in output,
                     f"退 {exit_code}；末行「{last_line(output)}」")
        restore()

    # A4：admission.py 的判法变了。旧判法（不判工作线程）写下 stream-b 那一格；把修好的判法提交之后，那一格照样作数。
    with open(ADMISSION, encoding="utf-8") as handle:
        current_text = handle.read()
    needle = "problem, note = judge_worker_threads(prefix, line, log_lines, machine_cores, threads_explicitly_one)"
    cells.expect("CONTROL", "A4 能造出旧判法（被判的 admission.py 里判工作线程那一句恰好一处）", current_text.count(needle) == 1,
                 f"命中 {current_text.count(needle)} 处")
    if current_text.count(needle) == 1:
        module.write_text(admission_copy, current_text.replace(needle, 'problem, note = None, "旧判法：不判工作线程"'))
        git("add", "-A")
        git("commit", "-q", "-m", "old judge")
        for name in os.listdir(common_directory):
            if name.startswith(module.CRASH_CASE_MARKER_PREFIX + "stream-b."):
                os.remove(os.path.join(common_directory, name))
        one_thread_log = module.stream_log("LAYER0B", worker_threads=1)
        set_logs(stream_b=one_thread_log)
        exit_code, output, runs = stage("--full")
        fingerprint_old = fingerprint("crash-case:stream-b")
        cells.expect("CONTROL", "A4 旧判法下 --full：stream-b 的日志 64 片只起 1 个线程，旧判法判绿、写那一格", exit_code == 0 and CASES[1][1] in runs,
                     f"退 {exit_code}，跑了 {runs}；末行「{last_line(output)}」")
        module.write_text(admission_copy, current_text)
        git("add", "-A")
        git("commit", "-q", "-m", "fixed judge")
        fingerprint_new = fingerprint("crash-case:stream-b")
        cells.expect("ATTACK", "A4 提交了修好的判法 ⇒ stream-b 的输入指纹应当变（判它的 admission.py 进指纹）", fingerprint_new != fingerprint_old,
                     f"旧 {fingerprint_old[:16]}…，新 {fingerprint_new[:16]}…")
        exit_code, output, runs = stage("--full")
        cells.expect("ATTACK", "A4 修好判法之后 --full ⇒ 旧判法写的那一格不该复用，stream-b 应当重跑", CASES[1][1] in runs,
                     f"退 {exit_code}，跑了 {runs}；末行「{last_line(output)}」")
        exit_code, output, _runs = stage(gate_full=True)
        cells.expect("ATTACK", "A4 修好判法之后快档（SINGLEFS_GATE_FULL=1）⇒ 不该判绿", exit_code != 0,
                     f"退 {exit_code}；末行「{last_line(output)}」")
        log_path = os.path.join(control, "stream-b-log")
        module.write_text(log_path, one_thread_log)
        exit_code, output, _messages = module.run_in_environment(
            [sys.executable, admission_copy, "crash-case-judge", work, "crash-case:stream-b", log_path, os.path.join(control, "judged"),
             "--machine-cores", "32", "--threads", "32", "--threads-origin", "default"], environment)
        cells.expect("CONTROL", "A4 修好的判法拿同一份日志判 ⇒ 红（只起了 1 个工作线程）", exit_code == 1 and "只起了 1 个工作线程" in output,
                     f"退 {exit_code}；「{output.strip()[:160]}」")
finally:
    shutil.rmtree(work, ignore_errors=True)
sys.exit(cells.finish())
