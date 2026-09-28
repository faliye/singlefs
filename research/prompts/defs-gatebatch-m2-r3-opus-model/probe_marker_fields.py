"""标记字段（configured_worker_threads= 与 started_worker_threads= 分记）在不读 SINGLEFS_LAYER0_THREADS 的用例上：
  ① 真仓登记表第 42 行 crash-case:crash-injection-fast-tier：crash-case-command 交的命令设 SINGLEFS_LAYER0_THREADS、不碰 SINGLEFS_CRASH_INJECTION_THREADS
     （这条用例读的是后一个，crates/singlefs-harness/src/crash_injection.rs 第 71 行）。把命令里 cargo 往后换成打那两个变量的 sh 起一趟（不跑用例），
     看调用方环境里的 SINGLEFS_CRASH_INJECTION_THREADS=1 漏不漏进去。只读真仓（crash-case-command 不写文件；指纹给 64 个 0，只用来拼进度目录的名字）。
  ② 小仓里照第 42 行的登记（只有 count-line=CRASH_INJECTION_FINISHED）判一份「1 个线程」的合成日志、写标记：标记里线程那两格写的是什么。
  ③ 本机核数取 nproc（admission.py crash_case_worker_threads），nproc 认调用方环境里的 OMP_NUM_THREADS：漏进来的 OMP_NUM_THREADS=1 时交的核数、线程数与判法。
复跑：python3 probe_marker_fields.py"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

cells = pc.Cells("标记字段与线程来源")
KEY = "crash-case:crash-injection-fast-tier"
code, out, err = pc.run([sys.executable, pc.ADMISSION, "crash-case-command", pc.REPOSITORY, KEY, "0" * 64],
                        extra_environment={"SINGLEFS_CRASH_INJECTION_THREADS": "1"}, drop=["SINGLEFS_LAYER0_THREADS"])
words = out.split("\0")[:-1]
command = words[4:]
print(f"INFO\tcrash-case-command 退 {code}；前四个 {words[:3]}；命令：{' '.join(command).replace(pc.REPOSITORY, '<仓>')}")
prefix = command[:command.index("cargo")]
probe = prefix + ["sh", "-c", 'echo "SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS-unset} SINGLEFS_CRASH_INJECTION_THREADS=${SINGLEFS_CRASH_INJECTION_THREADS-unset}"']
seen_code, seen, _ = pc.run(probe, extra_environment={"SINGLEFS_CRASH_INJECTION_THREADS": "1"})
cells.expect("ATTACK", "① 调用方设着 SINGLEFS_CRASH_INJECTION_THREADS=1，照 crash-case-command 的命令起的这条用例 ⇒ 不应当看到 1（该清掉或设成配的线程数）",
             "SINGLEFS_CRASH_INJECTION_THREADS=1" not in seen, f"起的进程看到「{seen.strip()}」（退 {seen_code}）")

work = pc.scratch("marker-fields-")
pc.write(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/pkg"]\n')
pc.write(os.path.join(work, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\nedition = "2021"\n')
pc.write(os.path.join(work, "crates/pkg/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
pc.write(os.path.join(work, "crates/pkg/tests/own_case.rs"), "#[test]\n#[ignore]\nfn the_case() {}\n")
pc.write(os.path.join(work, ".claude/gate.d/stage-inputs.tsv"),
         "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=CRASH_INJECTION_FINISHED\t# 照真仓第 42 行\n")
pc.run(["git", "init", "-q", "-b", "master", work])
manifest = os.path.join(work, ".manifest")
_c, summary, _e = pc.run([sys.executable, pc.ADMISSION, "crash-case-manifest", work, "crash-case:own", manifest])
fingerprint = summary.split()[0]
pc.write(os.path.join(work, "log"), "CRASH_INJECTION_FINISHED seeds=[7,31) slices=24 worker_threads=1 elapsed_seconds=900.0\n"
         "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 900.00s\n")
THREADS = ["--machine-cores", "32", "--threads", "32", "--threads-origin", "default"]
judge_code, judge_out, _ = pc.run([sys.executable, pc.ADMISSION, "crash-case-judge", work, "crash-case:own", os.path.join(work, "log"),
                                   os.path.join(work, "judged"), *THREADS])
record_code, marker_path, _ = pc.run([sys.executable, pc.ADMISSION, "crash-case-record", work, "crash-case:own", fingerprint, manifest,
                                      os.path.join(work, "judged"), "--files", "4", "--excluded", "0", "--started", "2026-09-27T00:00:00Z",  # clock-times:allow 传给被测脚本的开跑时间戳参数
                                      "--judged-root", work, *THREADS])
marker_lines = open(marker_path.strip(), encoding="utf-8").read().split("\n") if record_code == 0 else []
thread_lines = [line for line in marker_lines if line.startswith(("configured_worker_threads=", "started_worker_threads=", "CRASH_INJECTION_FINISHED"))]
for line in thread_lines:
    print(f"MARKER\t{line}")
cells.expect("ATTACK", "② 判 1 个线程跑完的日志、写标记 ⇒ configured_worker_threads= 不应当写一个这条用例不读的变量",
             not any(line.startswith("configured_worker_threads=SINGLEFS_LAYER0_THREADS=") for line in thread_lines),
             f"judge 退 {judge_code}、record 退 {record_code}；线程那几行见上面 MARKER")
# ③ 本机核数取 nproc，nproc 认调用方环境里的 OMP_NUM_THREADS：漏进来的 OMP_NUM_THREADS=1 让「多于 1 核却只起 1 个线程」那一格判不出
code3, out3, _ = pc.run([sys.executable, pc.ADMISSION, "crash-case-command", pc.REPOSITORY, KEY, "0" * 64],
                        extra_environment={"OMP_NUM_THREADS": "1"}, drop=["SINGLEFS_LAYER0_THREADS"])
words3 = out3.split("\0")[:4]
_n, real_cores, _e = pc.run(["nproc", "--all"])
pc.write(os.path.join(work, ".claude/gate.d/stage-inputs.tsv"),
         "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0 exhaustive=LAYER0 threads=LAYER0\t# 样本\n")
admission = pc.load(pc.ADMISSION, "admission_marker_probe")
pc.write(os.path.join(work, "one-thread.log"), admission.synthetic_layer0_log(worker_threads=1))
code_j, out_j, _ = pc.run([sys.executable, pc.ADMISSION, "crash-case-judge", work, "crash-case:own", os.path.join(work, "one-thread.log"),
                           os.path.join(work, "judged-omp"), "--machine-cores", words3[0], "--threads", words3[1], "--threads-origin", words3[2]])
cells.expect("ATTACK", "③ 调用方环境里 OMP_NUM_THREADS=1（没设 SINGLEFS_LAYER0_THREADS）⇒ crash-case-command 交的核数应当是本机的核数、64 片只起 1 个线程的日志应当判红",
             not (words3[0] == "1" and code_j == 0),
             f"nproc --all 报 {real_cores.strip()}；crash-case-command 交「核数 {words3[0]}、线程 {words3[1]}、{words3[2]}」；照它判 64 片 1 线程的日志退 {code_j}（{out_j.strip().splitlines()[0][:80] if out_j.strip() else ''}）")
sys.exit(cells.finish())
