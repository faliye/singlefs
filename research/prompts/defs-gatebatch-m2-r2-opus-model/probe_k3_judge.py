"""K3 探针：拿 .claude/gate.d/stage-inputs.tsv 里 floor-raise、c561 两条用例今天的登记行，喂合成日志给被判的 admission.py 的判法
（judge_crash_case_log，即 54 号 --full 调的 crash-case-judge；再拿判绿时要记的行拼一格标记，过 crash_case_marker_problems，即快档核的那一步）。
复跑：python3 probe_k3_judge.py
格：CONTROL 今天应当判对；GAP 是门禁这一层判不出、只靠用例自己的断言挡（不计入打中数，报告里逐条交代）。"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

cells = pc.Cells("K3 两条登记行的判法")
admission = pc.load(pc.ADMISSION, "admission_k3")
rows = admission.read_registration_rows(pc.REPOSITORY)
FLOOR = admission.crash_case_of_key(rows, "crash-case:floor-raise-pushed-by-the-session")
C561 = admission.crash_case_of_key(rows, "crash-case:c561-sigma-full")
OK = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 700.00s"
print(f"# floor-raise 登记：count_lines={FLOOR.count_lines} exhaustive={FLOOR.exhaustive_lines} threads={FLOOR.thread_lines}")
print(f"# c561 登记：count_lines={C561.count_lines} exhaustive={C561.exhaustive_lines} threads={C561.thread_lines}")


def finished(states=20493, slices=512, worker_threads=32, resumed=0, fresh=512, source="available_parallelism"):
    return (f"LAYER0_PARALLEL_FINISHED states={states} slices={slices} worker_threads={worker_threads} configured_worker_threads={worker_threads} "
            f"worker_threads_source={source} resumed_slices={resumed} freshly_run_slices={fresh} progress_file_after_completion=none elapsed_seconds=700.0")


def judged(case, log, cores=32, explicit_one=False):
    problems, recorded, _notes = admission.judge_crash_case_log(case, log, cores, explicit_one)
    marker_problems = None
    if not problems:
        text = "".join(f"{line}\n" for line in recorded)
        marker = {"fields": {}, "raw_lines": [], "input_files": []}
        for line in text.split("\n"):
            if not line:
                continue
            if __import__("re").match(r"^[a-z_]+=", line):
                name, value = line.split("=", 1)
                marker["fields"].setdefault(name, value)
            else:
                marker["raw_lines"].append(line)
        marker["fields"].update({"input_hash": "f" * 64, "case": case.key})
        marker_problems = admission.crash_case_marker_problems(case, "f" * 64, marker)
    return problems, marker_problems


def cell(kind, label, case, log, want_green, cores=32, explicit_one=False):
    problems, marker_problems = judged(case, log, cores, explicit_one)
    green = not problems and not marker_problems
    cells.expect(kind, f"{label} ⇒ {'判绿（--full 写标记、快档认它）' if want_green else '判红'}", green == want_green,
                 f"--full 判：{'绿' if not problems else '红：' + problems[0][:110]}；快档核标记：{'—' if marker_problems is None else ('作数' if not marker_problems else '不作数：' + marker_problems[0][:80])}")


# ── floor-raise（count-line=LAYER0_PARALLEL_FINISHED threads=LAYER0_PARALLEL_FINISHED，没有 exhaustive=）──
cell("CONTROL", "F1 跑完全部片（512 片、32 线程）", FLOOR, f"{finished()}\n{OK}\n", True)
cell("CONTROL", "F2 只跑了一半的片（resumed 0 + fresh 256 ≠ 512），用例却报过了", FLOOR, f"{finished(fresh=256)}\n{OK}\n", False)
cell("CONTROL", "F3 跑了 512 片却只起 1 个线程（没显式设）", FLOOR, f"{finished(worker_threads=1)}\n{OK}\n", False)
cell("CONTROL", "F4 显式 SINGLEFS_LAYER0_THREADS=1", FLOOR, f"{finished(worker_threads=1, source='environment_variable')}\n{OK}\n", True, explicit_one=True)
cell("CONTROL", "F5 用例 panic 在枚举之后（test result FAILED）", FLOOR, f"{finished()}\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out\n", False)
cell("GAP", "F6 片数守恒、线程对，但 states=64（展开被改窄、用例断言跟着改）", FLOOR, f"{finished(states=64, slices=4, fresh=4)}\n{OK}\n", False)
cell("GAP", "F7 片数守恒，但 resumed=256（NoProgressFile 下不该有读回的片）", FLOOR, f"{finished(resumed=256, fresh=256)}\n{OK}\n", False)
# ── c561（只有 count-line=C561_SIGMA_FULL）──
cell("CONTROL", "S1 262144 个状态、0 个判缺席", C561, f"C561_SIGMA_FULL states=262144 record_claimed_state_missing_unit=0\n{OK}\n", True)
cell("GAP", "S2 计数行 states=4（只评了 4 个状态、用例断言跟着改）", C561, f"C561_SIGMA_FULL states=4 record_claimed_state_missing_unit=0\n{OK}\n", False)
cell("GAP", "S3 计数行报 5 个判缺席（用例断言被改成不判它）", C561, f"C561_SIGMA_FULL states=262144 record_claimed_state_missing_unit=5\n{OK}\n", False)
cell("GAP", "S4 单线程跑完（日志里没有线程的任何痕迹），本机 32 核、线程数没显式设", C561, f"C561_SIGMA_FULL states=262144 record_claimed_state_missing_unit=0\n{OK}\n", False)
cell("CONTROL", "S5 计数行打了两遍", C561, f"C561_SIGMA_FULL states=262144 record_claimed_state_missing_unit=0\nC561_SIGMA_FULL states=262144 record_claimed_state_missing_unit=0\n{OK}\n", False)
gaps = [row for row in cells.rows if row[0] == "GAP" and not row[2]]
print(f"GAPS {len(gaps)}")
sys.exit(cells.finish())
