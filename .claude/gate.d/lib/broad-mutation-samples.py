"""门禁 checker-tier-broad-mutation 的样本现场：阶段在样本模式下（样本根放了 .broad-mutation-sample，里面一行 scenario=<情形>）开格之前起它，
在样本根里造好 gate-verdict 格要读的东西，阶段本身不知道各情形怎么造。

调用（阶段经参数把驱动与样本根传进来）：python3 <这份文件> <驱动 broad-mutation.py 的路径> <样本根> <情形>
情形分四族：
  summary:<名>     录好的驱动 --summary 输出：写 .broad-sample/summary.sh（打录好的那几行、退录好的码），格照它判，库目录里没有东西
  escalated:<名>   升档判的抓到与判定行的校验列：在 .broad-sample/home/ 造判定库（verdicts.tsv，每行带用 .broad-sample/result.key 算的 HMAC-SHA256 校验列）、
                   基线表（baselines.tsv）、两趟日志（escalation/ 下，末尾带驱动那一行、带包装那一句）、登记表（escalated-verdicts.tsv），
                   再写 .broad-sample/summary.sh 打录好的 --summary；sha256 与校验列在这里现算，不调驱动的函数
  gpu:<名>         GPU 那一趟的判定库（gpu-verdicts.tsv）的校验列：在 .broad-sample/home/ 造 verdicts.tsv 与 gpu-verdicts.tsv（每行带用 .broad-sample/result.key 算的 HMAC-SHA256 校验列），
                   再写 .broad-sample/summary.sh 打录好的 --summary（GPU 趟要 2 条、都有结论）；sha256 与校验列在这里现算，不调驱动的函数
  units:<名>       新键在真源码上的样子：样本根里写一棵小工作区（singlefs-core、singlefs-checker、singlefs-harness），载入驱动、用它的函数照这棵树
                   把清单按单元记进 .broad-sample/home/lists.tsv、每条记一行抓到（键是此刻算的单元键）；再照情形改源码。不写 summary.sh：
                   格起真的驱动 --summary（--toolchain-from / --tools-from 取样本里的两份文件），判法、单元键都是此刻驱动那一份
各情形改的那一处与期望写在 SCENARIOS 旁边的注释里；样本目录的 expect 写它该红还是该绿、该出现的句子。
"""
import argparse
import hashlib
import hmac
import importlib.util
import os
import shutil
import sys

RESULT_KEY_TEXT = "0f1e2d3c4b5a69788796a5b4c3d2e1f000112233445566778899aabbccddeeff"
KEY_ONE = "e5ca1a7e" + "0" * 54 + "f1"
KEY_TWO = "e5ca1a7e" + "0" * 54 + "f2"
OTHER_KEY = "01d00000" + "0" * 54 + "f0"
BASELINE_KEY = "ba5e1e00" + "0" * 54 + "aa"
NAME_ONE = "crates/singlefs-core/src/lib.rs:2:5: replace the value with MUTATED_1()"
NAME_TWO = "crates/singlefs-core/src/lib.rs:5:5: replace the value with MUTATED_2()"
VERDICT_HEADER = ("key", "sequence", "package", "name", "outcome", "stage", "caught_by", "test_key", "coverage_key", "build_seconds", "test_seconds",
                  "judged_on", "method", "locator", "refused", "note", "mac")
REGISTRY_HEADER = ("key", "sequence", "name", "label", "step", "baseline_key", "baseline", "baseline_row", "threads", "first_limit", "escalated_limit",
                   "required_multiple", "first_log", "first_log_sha256", "escalated_log", "escalated_log_sha256", "judged_on")
SUMMARY_COUNTS = ("total", "caught", "missed", "unviable", "timeout", "memory_cap", "stage_one_only", "driver_error", "not_run", "refused", "unlisted_units",
                  "gpu_needed", "gpu_reused", "gpu_not_run", "gpu_without_conclusion")
GPU_VERDICT_HEADER = ("key", "sequence", "package", "name", "outcome", "caught_by", "test_key", "coverage_key", "gpu_key", "test_seconds", "judged_on", "method", "note", "mac")
GPU_CASE = "singlefs-checker-tier::gpu_pair gpu_case"


def summary_line(**counts):
    values = dict.fromkeys(SUMMARY_COUNTS, 0)
    values.update(counts)
    return "BROAD_MUTATION_SUMMARY " + " ".join(f"{name}={values[name]}" for name in SUMMARY_COUNTS)


def gate_line(total, concluded, missed=0, without=0, refused=0, unlisted=0, escalated=0, verdict="green", escalated_field=True,
              gpu_needed=0, gpu_reused=0, gpu_not_run=0, gpu_without=0, gpu_fields=True):
    return (f"BROAD_MUTATION_GATE total={total} concluded={concluded} missed={missed} without_conclusion={without} refused={refused} unlisted_units={unlisted}"
            + (f" escalated={escalated}" if escalated_field else "")
            + (f" gpu_needed={gpu_needed} gpu_reused={gpu_reused} gpu_not_run={gpu_not_run} gpu_without_conclusion={gpu_without}" if gpu_fields else "")
            + f" verdict={verdict}")


MISSED_NAME = "crates/singlefs-core/src/sample.rs:12:9: replace + with - in add"
GPU_NAME_FOUR = "crates/singlefs-core/src/lib.rs:11:5: replace the value with MUTATED_4()"
GPU_NAME_FIVE = "crates/singlefs-core/src/lib.rs:14:5: replace the value with MUTATED_5()"
# summary 族：录好的 --summary 输出与退出码（行用 \t 分段，照驱动的写法）
RECORDED = {
    # 全有结论（键相同），没有没红：绿
    "green": ([summary_line(total=6, caught=5, unviable=1), gate_line(6, 6)], 0),
    # 一条没红：驱动判红退 8，格照抄 MISSED
    "missed": ([summary_line(total=6, caught=4, missed=1, unviable=1), f"MISSED\t3\tsinglefs-core\t{MISSED_NAME}", gate_line(6, 6, missed=1, verdict="red")], 8),
    # 三条沿用不了也没现判：驱动判红退 8，格照抄 NO_CONCLUSION（带作废时变了的单元）
    "no-conclusion": ([summary_line(total=6, caught=2, unviable=1, not_run=3), gate_line(6, 3, without=3, verdict="red"),
                       "NO_CONCLUSION\t4\tnone\tcrates/singlefs-core/src/sample.rs:20:5: replace add -> u32 with 0\tno-record；变了：改了 add（crates/singlefs-core/src/sample.rs）",
                       "NO_CONCLUSION\t5\tnone\tcrates/singlefs-core/src/sample.rs:20:5: replace add -> u32 with 1\tno-record",
                       "NO_CONCLUSION\t6\tnone\tcrates/singlefs-core/src/sample.rs:24:9: replace == with != in add\tcatching-test-changed"], 8),
    # 判定行写 green、退 0，而 missed=1：驱动的判红失灵
    "silent-driver": ([summary_line(total=6, caught=4, missed=1, unviable=1), f"MISSED\t3\tsinglefs-core\t{MISSED_NAME}", gate_line(6, 6, missed=1)], 0),
    # 只有汇总行、没有判定行、退 0（旧驱动）
    "line-missing": ([summary_line(total=6, caught=5, unviable=1)], 0),
    # 判定行写 red、计数与明细一致、却退 0
    "red-exit-zero": ([summary_line(total=6, caught=4, missed=1, unviable=1), f"MISSED\t3\tsinglefs-core\t{MISSED_NAME}", gate_line(6, 6, missed=1, verdict="red")], 0),
    # 判定行 green、计数全零，却列了 2 行 NO_CONCLUSION
    "detail-count": ([summary_line(total=6, caught=5, unviable=1), gate_line(6, 6),
                      "NO_CONCLUSION\t5\tnone\tcrates/singlefs-core/src/sample.rs:20:9: replace x\tno-record",
                      "NO_CONCLUSION\t6\tnone\tcrates/singlefs-core/src/sample.rs:24:9: replace y\tno-record"], 0),
    # 清单 0 条、判定行 green
    "empty-list": ([summary_line(total=0), gate_line(0, 0)], 0),
    # verdict=pending、退 8
    "unknown-verdict": ([summary_line(total=6, caught=5, unviable=1), gate_line(6, 6, verdict="pending")], 8),
    # 汇总行 total=5091、not_run=5085，判定行 total=6 写 green
    "totals-mismatch": ([summary_line(total=5091, caught=5, unviable=1, not_run=5085), gate_line(6, 6)], 0),
    # 判定行 refused=1、汇总行 refused=1，却一行 REFUSED 都没列
    "refused-count": ([summary_line(total=6, caught=5, unviable=1, refused=1), gate_line(6, 6, refused=1)], 0),
    # 判定行写 green、退 0，而 unlisted_units=2（有单元没记清单，清单不全）
    "unlisted-lie": ([summary_line(total=6, caught=5, unviable=1, unlisted_units=2), "UNLISTED\tcrates/singlefs-core/src/lib.rs:30\tfn\tnew_function",
                      "UNLISTED\tcrates/singlefs-core/src/lib.rs:40\tfn\tother_function", gate_line(6, 6, unlisted=2)], 0),
    # GPU 那一趟（驱动文件头「GPU 那一趟」）：要 GPU 趟的 3 条都有库里键相同的结论（其中 2 条是 GPU 趟抓到的，所以八栏里是抓到）：绿
    "gpu-green": ([summary_line(total=6, caught=5, unviable=1, gpu_needed=3, gpu_reused=3), gate_line(6, 6, gpu_needed=3, gpu_reused=3)], 0),
    # 要 GPU 趟的 3 条里 2 条本次未跑（没卡）：它们不算没红也不算通过，是没结论，驱动判红退 8，格照抄 NO_CONCLUSION（理由 gpu-not-run）
    "gpu-no-conclusion": ([summary_line(total=6, caught=3, unviable=1, not_run=2, gpu_needed=3, gpu_reused=1, gpu_not_run=2),
                           gate_line(6, 4, without=2, verdict="red", gpu_needed=3, gpu_reused=1, gpu_not_run=2),
                           f"NO_CONCLUSION\t4\tnone\t{GPU_NAME_FOUR}\tgpu-not-run", f"NO_CONCLUSION\t5\tnone\t{GPU_NAME_FIVE}\tgpu-not-run"], 8),
    # 要 GPU 趟的 3 条里 1 条 GPU 趟超时（没结论）：驱动判红退 8
    "gpu-timeout": ([summary_line(total=6, caught=4, unviable=1, timeout=1, gpu_needed=3, gpu_reused=2, gpu_without_conclusion=1),
                     gate_line(6, 5, without=1, verdict="red", gpu_needed=3, gpu_reused=2, gpu_without=1), f"NO_CONCLUSION\t4\ttimeout\t{GPU_NAME_FOUR}\tgpu-timeout"], 8),
    # 判定行写 green、退 0、without_conclusion=0，而 gpu_not_run=2：驱动把没有 GPU 结论的条当有结论数了
    "gpu-silent-driver": ([summary_line(total=6, caught=5, unviable=1, gpu_needed=3, gpu_reused=1, gpu_not_run=2), gate_line(6, 6, gpu_needed=3, gpu_reused=1, gpu_not_run=2)], 0),
    # 判定行 green，gpu_needed=3 却只有 1 条复用、另两项是 0：三项加不起来
    "gpu-counts": ([summary_line(total=6, caught=5, unviable=1, gpu_needed=3, gpu_reused=1), gate_line(6, 6, gpu_needed=3, gpu_reused=1)], 0),
    # 判定行没有 gpu_ 四个数（旧驱动）
    "gpu-fields-missing": ([summary_line(total=6, caught=5, unviable=1), gate_line(6, 6, gpu_fields=False)], 0),
    # 汇总行与判定行的 GPU 趟计数不同
    "gpu-summary-mismatch": ([summary_line(total=6, caught=5, unviable=1, gpu_needed=3, gpu_reused=2, gpu_not_run=1), gate_line(6, 6, gpu_needed=3, gpu_reused=3)], 0),
}
ESCALATED_VARIANTS = ("green", "multiple", "log-digest", "old-key", "handwritten", "count", "threads-rerun", "baseline-row-changed", "baseline-lie", "not-larger",
                      "memory-multiple", "no-trailer", "trailer-wrong", "no-wrapper", "columns", "no-escalated-field", "label-differs", "logs-elsewhere",
                      "baseline-key-differs", "result-mac-forged", "result-mac-key-missing")
UNITS_VARIANTS = ("green", "callee-changed", "comment-only", "unlisted", "gpu-green", "gpu-no-conclusion")
GPU_VARIANTS = ("mac-green", "mac-forged")


def write(path, text, mode=None):
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    if mode is not None:
        os.chmod(path, mode)


def write_summary(root, lines, exit_code):
    """录好的 --summary：引号括住的 here-doc 原样打那几行（行里的制表符照写进去），再退录好的码。"""
    body = "".join(line + "\n" for line in lines)
    write(os.path.join(root, ".broad-sample", "summary.sh"), f"#!/usr/bin/env bash\n# 样本：录好的驱动 --summary 输出\ncat <<'SUMMARY'\n{body}SUMMARY\nexit {exit_code}\n", 0o755)


def mac(fields):
    return hmac.new(bytes.fromhex(RESULT_KEY_TEXT), "\t".join(fields).encode("utf-8"), hashlib.sha256).hexdigest()


def build_escalated(root, variant):
    """升档族（情形名见 ESCALATED_VARIANTS）：第 1 条（编译那一步两趟都超时）、第 2 条（测试那一步两趟都撞顶）是升档判的抓到，另有四条普通的抓到。
    green 两行都齐、另有一行别的键的坏行（不看）；multiple 第 1 条升档那趟的限时 150 不到基线 10.0 秒的 20 倍；log-digest 第 1 条升档那趟的日志登记之后又改了；
    old-key 第 1 条的登记行是别的键的（证据本身齐），这把键没有；handwritten 第 1 条的登记行指的日志不在；count 判定行 escalated=3、ESCALATED 只有 2 行；
    threads-rerun 换了 --threads 16 之后 16 线程的基线追加在后、第 1 条在 16 线程下又升档登记了一行（指 16 线程那一行），旧登记行指的 8 线程那一行没动，两行都复核得过；
    baseline-row-changed 第 1 条登记行指的那一行基线的线程数被改成 16；baseline-lie 第 1 条登记的基线 1.0，它指的那一行记的是 10.0；not-larger 第 1 条两趟的限时都是 3600；
    memory-multiple 第 2 条的基线峰值 20 GiB，升档那趟 36864M 不到它的 2 倍；no-trailer 第 1 条两份日志末尾没有驱动那一行；trailer-wrong 驱动那一行写成别的；
    no-wrapper 第 1 条两份日志里没有包装那一句；columns 第 1 条的登记行少了末一段；no-escalated-field 判定行没有 escalated=；label-differs ESCALATED 第 1 行的标签与登记的不同；
    logs-elsewhere 第 1 条的两份日志挪到 logs/ 下、登记行跟着指过去；baseline-key-differs 第 1 条登记的基线键与它指的那一行的不同；
    result-mac-forged 判定库末尾手追加一行没有校验列的 caught 与一行校验列对不上的 caught；result-mac-key-missing 样本的密钥不在。"""
    home = os.path.join(root, ".broad-sample", "home")
    os.makedirs(os.path.join(home, "escalation"), exist_ok=True)
    peak_two = 21474836480 if variant == "memory-multiple" else 1073741824
    baseline_rows = [(BASELINE_KEY, "one-build", "ok", "10.0", "2026-10-03", "8", "536870912"), (BASELINE_KEY, "one-test", "ok", "40.0", "2026-10-03", "8", str(peak_two))]
    if variant == "threads-rerun":
        baseline_rows += [(BASELINE_KEY, "one-build", "ok", "1.2", "2026-10-04", "16", "536870912"), (BASELINE_KEY, "one-test", "ok", "30.0", "2026-10-04", "16", "1074790400")]
    if variant == "baseline-row-changed":
        baseline_rows[0] = baseline_rows[0][:5] + ("16",) + baseline_rows[0][6:]
    write(os.path.join(home, "baselines.tsv"), "# baseline_key\tstep\toutcome\tseconds\tfinished_at\tthreads\tpeak_bytes\n"
          + "".join("\t".join(row) + "\n" for row in baseline_rows))
    first_limit, escalated_limit = ("100", "150") if variant == "multiple" else ("900", "3600")
    if variant == "not-larger":
        first_limit = "3600"
    one_key = OTHER_KEY if variant == "old-key" else KEY_ONE

    def log(relative, wrapper, key, sequence, step, attempt, limit, exit_code, trailer=True):
        lines = ["   Compiling singlefs-core（样本）"]
        if wrapper:
            lines.append(f"run-with-memory-cap: {wrapper}（样本）")
        if trailer:
            lines.append(f"BROAD_MUTATION_EVIDENCE key={key} sequence={sequence} step={step} attempt={attempt} limit={limit} exit={exit_code}")
        write(os.path.join(home, relative), "\n".join(lines) + "\n")

    one_first, one_escalated = "escalation/1.one-build.100.first.log", "escalation/1.one-build.100.escalated.log"
    first_wrapper, escalated_wrapper = ("", "") if variant == "no-wrapper" else (f"超过限时 {first_limit} 秒", f"超过限时 {escalated_limit} 秒")
    if variant == "trailer-wrong":
        log(one_first, first_wrapper, one_key, 1, "one-build", "first", first_limit, 0)
        log(one_escalated, escalated_wrapper, one_key, 1, "one-build", "first", escalated_limit, 0)
    else:
        log(one_first, first_wrapper, one_key, 1, "one-build", "first", first_limit, 253, trailer=variant != "no-trailer")
        log(one_escalated, escalated_wrapper, one_key, 1, "one-build", "escalated", escalated_limit, 253, trailer=variant != "no-trailer")
    log("escalation/2.one-test.200.first.log", "撞了内存上限 24G", KEY_TWO, 2, "one-test", "first", "24G", 250)
    log("escalation/2.one-test.200.escalated.log", "撞了内存上限 36864M", KEY_TWO, 2, "one-test", "escalated", "36864M", 250)
    if variant == "logs-elsewhere":
        os.makedirs(os.path.join(home, "logs"), exist_ok=True)
        os.replace(os.path.join(home, one_first), os.path.join(home, "logs/1.first.log"))
        os.replace(os.path.join(home, one_escalated), os.path.join(home, "logs/1.escalated.log"))
        one_first, one_escalated = "logs/1.first.log", "logs/1.escalated.log"

    def digest(relative):
        with open(os.path.join(home, relative), "rb") as handle:
            return hashlib.sha256(handle.read()).hexdigest()
    one_first_digest, one_escalated_digest = digest(one_first), digest(one_escalated)
    if variant == "handwritten":
        one_first, one_escalated = "escalation/1.one-build.999.first.log", "escalation/1.one-build.999.escalated.log"
        one_first_digest = one_escalated_digest = "0" * 64
    if variant == "log-digest":
        with open(os.path.join(home, one_escalated), "a", encoding="utf-8") as handle:
            handle.write("登记之后又改了这一份\n")
    one_baseline = "1.0" if variant == "baseline-lie" else "10.0"
    one_baseline_key = OTHER_KEY if variant == "baseline-key-differs" else BASELINE_KEY
    one_row = [one_key, "1", NAME_ONE, "caught-by-timeout", "one-build", one_baseline_key, one_baseline, "2", "8", first_limit, escalated_limit, "20",
               one_first, one_first_digest, one_escalated, one_escalated_digest, "2026-10-03"]
    if variant == "columns":
        one_row = one_row[:-1]
    registry = [one_row, [KEY_TWO, "2", NAME_TWO, "caught-by-memory", "one-test", BASELINE_KEY, str(peak_two), "3", "8", "24G", "36864M", "2",
                          "escalation/2.one-test.200.first.log", digest("escalation/2.one-test.200.first.log"),
                          "escalation/2.one-test.200.escalated.log", digest("escalation/2.one-test.200.escalated.log"), "2026-10-03"]]
    if variant == "green":
        registry.append([OTHER_KEY, "9", "别的键的坏行", "caught-by-timeout", "one-build", BASELINE_KEY, "1.0", "2", "8", "1", "2", "20",
                         "escalation/no-such.log", "0", "escalation/no-such.log", "0", "2026-10-03"])
    if variant == "threads-rerun":
        log("escalation/1.one-build.300.first.log", "超过限时 900 秒", KEY_ONE, 1, "one-build", "first", "900", 253)
        log("escalation/1.one-build.300.escalated.log", "超过限时 3600 秒", KEY_ONE, 1, "one-build", "escalated", "3600", 253)
        registry.append([KEY_ONE, "1", NAME_ONE, "caught-by-timeout", "one-build", BASELINE_KEY, "1.2", "4", "16", "900", "3600", "20",
                         "escalation/1.one-build.300.first.log", digest("escalation/1.one-build.300.first.log"),
                         "escalation/1.one-build.300.escalated.log", digest("escalation/1.one-build.300.escalated.log"), "2026-10-04"])
    write(os.path.join(home, "escalated-verdicts.tsv"), "# " + "\t".join(REGISTRY_HEADER) + "\n" + "".join("\t".join(row) + "\n" for row in registry))
    rows = [[KEY_ONE, "1", "singlefs-core", NAME_ONE, "caught", "one", "caught-by-timeout", "", "", "0.0", "3600.2", "2026-10-03", "fresh", "", "",
             "one-build 超时之后升档重跑（900 → 3600），两趟都超时"],
            [KEY_TWO, "2", "singlefs-core", NAME_TWO, "caught", "one", "caught-by-memory", "", "", "0.0", "12.0", "2026-10-03", "fresh", "", "",
             "one-test 撞顶之后升档重跑（24G → 36864M），两趟都撞顶"]]
    for sequence, line in ((3, 8), (4, 11), (5, 14), (6, 17)):
        rows.append([f"{sequence:064x}", str(sequence), "singlefs-core", f"crates/singlefs-core/src/lib.rs:{line}:5: replace the value with MUTATED_{sequence}()",
                     "caught", "one", "singlefs-harness::catching some_case", "c" * 64, "", "0.0", "1.0", "2026-10-03", "fresh", "", "", ""])
    text = "# " + "\t".join(VERDICT_HEADER) + "\n" + "".join("\t".join(row + [mac(row)]) + "\n" for row in rows)
    if variant == "result-mac-forged":
        forged = [f"{4:064x}", "4", "singlefs-core", "crates/singlefs-core/src/lib.rs:11:5: replace the value with MUTATED_4()", "caught", "one", "hand::written", "", "",
                  "0.0", "0.0", "2026-10-03", "fresh", "", "", ""]
        text += "\t".join(forged) + "\n" + "\t".join(forged + ["0" * 64]) + "\n"
    write(os.path.join(home, "verdicts.tsv"), text)
    if variant != "result-mac-key-missing":
        write(os.path.join(root, ".broad-sample", "result.key"), RESULT_KEY_TEXT + "\n", 0o600)
    escalated_count = 3 if variant == "count" else 2
    label_one = "caught-by-memory" if variant == "label-differs" else "caught-by-timeout"
    write_summary(root, [summary_line(total=6, caught=6), gate_line(6, 6, escalated=escalated_count, escalated_field=variant != "no-escalated-field"),
                         f"ESCALATED\t1\t{label_one}\t{NAME_ONE}\t{KEY_ONE}", f"ESCALATED\t2\tcaught-by-memory\t{NAME_TWO}\t{KEY_TWO}"], 0)


def build_gpu(root, variant):
    """gpu 族（情形名见 GPU_VARIANTS）：verdicts.tsv 里 6 条抓到、gpu-verdicts.tsv 里 2 行（一条 GPU 趟抓到、一条没红，都带校验列）；
    mac-green 全部校验得过；mac-forged GPU 判定库末尾手追加一行没有校验列（十三段）、一行校验列对不上。录好的 --summary：GPU 趟要 2 条、都有结论（判绿）。"""
    home = os.path.join(root, ".broad-sample", "home")
    os.makedirs(home, exist_ok=True)
    rows = []
    for sequence, line in ((1, 2), (2, 5), (3, 8), (4, 11), (5, 14), (6, 17)):
        rows.append([f"{sequence:064x}", str(sequence), "singlefs-core", f"crates/singlefs-core/src/lib.rs:{line}:5: replace the value with MUTATED_{sequence}()",
                     "caught", "one", "singlefs-harness::catching some_case", "c" * 64, "", "0.0", "1.0", "2026-10-22", "fresh", "", "", ""])
    write(os.path.join(home, "verdicts.tsv"), "# " + "\t".join(VERDICT_HEADER) + "\n" + "".join("\t".join(row + [mac(row)]) + "\n" for row in rows))
    gpu_rows = [[f"{4:064x}", "4", "singlefs-core", GPU_NAME_FOUR, "caught", GPU_CASE, "d" * 64, "", "9" * 64, "1.0", "2026-10-22", "fresh", "GPU 趟抓到"],
                [f"{5:064x}", "5", "singlefs-core", GPU_NAME_FIVE, "missed", "", "", "e" * 64, "9" * 64, "1.0", "2026-10-22", "fresh", "3 条测试都过了"]]
    text = "# " + "\t".join(GPU_VERDICT_HEADER) + "\n" + "".join("\t".join(row + [mac(row)]) + "\n" for row in gpu_rows)
    if variant == "mac-forged":
        forged = [f"{4:064x}", "4", "singlefs-core", GPU_NAME_FOUR, "caught", GPU_CASE, "d" * 64, "", "9" * 64, "1.0", "2026-10-22", "fresh", "手写的"]
        text += "\t".join(forged) + "\n" + "\t".join(forged + ["0" * 64]) + "\n"
    write(os.path.join(home, "gpu-verdicts.tsv"), text)
    write(os.path.join(root, ".broad-sample", "result.key"), RESULT_KEY_TEXT + "\n", 0o600)
    write_summary(root, [summary_line(total=6, caught=6, gpu_needed=2, gpu_reused=2), gate_line(6, 6, gpu_needed=2, gpu_reused=2)], 0)


UNITS_CORE = "pub fn one() -> u32 {\n    1\n}\nfn helper() -> u32 {\n    7\n}\npub fn three() -> u32 {\n    helper()\n}\n"
UNITS_CHECKER = "pub fn seven() -> u32 {\n    7\n}\n"
UNITS_TREE = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/singlefs-core", "crates/singlefs-checker", "crates/singlefs-harness"]\nresolver = "2"\n',
    "crates/singlefs-core/Cargo.toml": '[package]\nname = "singlefs-core"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/singlefs-core/src/lib.rs": UNITS_CORE,
    "crates/singlefs-checker/Cargo.toml": '[package]\nname = "singlefs-checker"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/singlefs-checker/src/lib.rs": UNITS_CHECKER,
    "crates/singlefs-harness/Cargo.toml": '[package]\nname = "singlefs-harness"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/singlefs-harness/src/lib.rs": "",
    "crates/singlefs-harness/tests/catching.rs": "#[test]\nfn the_case_that_catches_it() {\n    assert_eq!(1 + 1, 2);\n}\n",
}
UNITS_MUTANTS = (("singlefs-core", "crates/singlefs-core/src/lib.rs", (2, 5), (2, 6), "0", "replace one -> u32 with 0"),
                 ("singlefs-core", "crates/singlefs-core/src/lib.rs", (5, 5), (5, 6), "0", "replace helper -> u32 with 0"),
                 ("singlefs-core", "crates/singlefs-core/src/lib.rs", (8, 5), (8, 13), "0", "replace three -> u32 with 0"),
                 ("singlefs-checker", "crates/singlefs-checker/src/lib.rs", (2, 5), (2, 6), "0", "replace seven -> u32 with 0"))
# units 族改源码的那一处：callee-changed 被调的 helper 改了函数体（three 里那一条的闭包里有它，要现判；helper 自己的单元变了，那个单元没记清单）；
# comment-only 只加注释（零条要现判）；unlisted 新加一个函数（那个单元没记清单）；green 不改
UNITS_CHANGES = {
    "green": {},
    "callee-changed": {"crates/singlefs-core/src/lib.rs": UNITS_CORE.replace("    7\n", "    8\n")},
    "comment-only": {"crates/singlefs-core/src/lib.rs": "// 只加注释：函数单元的指纹不变\npub fn one() -> u32 {\n    // 体里也加一行\n    1\n}\n/* 块注释 */ fn helper() -> u32 {\n    7 // 行尾注释\n}\n"
                                                         "pub fn three() -> u32 {\n    helper()\n}\n"},
    "unlisted": {"crates/singlefs-core/src/lib.rs": UNITS_CORE + "pub fn nine() -> u32 {\n    9\n}\n"},
}


# units 族的 GPU 情形：UNITS_TREE 加一个 checker 档包（特性 gpu、依赖 singlefs-core），tests/gpu_pair.rs 的测试体里有挂 gpu 特性的语句、经 judge() 够得着 one；
# 第 1 条（one）CPU 那一趟没红，别的条抓到。gpu-green：第 1 条的 GPU 判定行是抓到；gpu-no-conclusion：GPU 趟还没有结论
UNITS_GPU_FILES = {
    "Cargo.toml": UNITS_TREE["Cargo.toml"].replace('"crates/singlefs-harness"]', '"crates/singlefs-harness", "crates/singlefs-checker-tier"]'),
    "crates/singlefs-checker-tier/Cargo.toml": '[package]\nname = "singlefs-checker-tier"\nversion = "0.1.0"\nedition = "2021"\n\n[features]\ngpu = []\n\n'
                                              '[dependencies]\nsinglefs-core = { path = "../singlefs-core" }\n',
    "crates/singlefs-checker-tier/src/lib.rs": "pub fn judge() -> u32 {\n    singlefs_core::one()\n}\n",
    "crates/singlefs-checker-tier/tests/gpu_pair.rs": '#[test]\nfn gpu_case() {\n    #[cfg(feature = "gpu")]\n    let extra = 1;\n    #[cfg(not(feature = "gpu"))]\n    let extra = 0;\n'
                                                      '    assert!(singlefs_checker_tier::judge() + extra > 0);\n}\n',
}


def build_units(driver_path, root, variant):
    """units 族（情形名见 UNITS_VARIANTS）：照驱动此刻的函数在样本根这棵小工作区上记清单与判定，再照 UNITS_CHANGES 改源码。
    gpu-green、gpu-no-conclusion 另加 UNITS_GPU_FILES 那个 checker 档包：第 1 条（one）CPU 那一趟记没红，GPU 判定行照驱动的函数记（gpu-green 抓到、gpu-no-conclusion 不记）。"""
    gpu_case = variant.startswith("gpu-")
    for path, text in {**UNITS_TREE, **(UNITS_GPU_FILES if gpu_case else {})}.items():
        write(os.path.join(root, path), text)
    sample = os.path.join(root, ".broad-sample")
    write(os.path.join(sample, "rustc-vv.txt"), "rustc 1.0.0-sample (样本)\nhost: x86_64-unknown-linux-gnu\n")
    write(os.path.join(sample, "tool-versions.txt"), "cargo-nextest 0.0.0-sample\ncargo-mutants 0.0.0-sample\nllvm-profdata 0.0.0-sample\n")
    write(os.path.join(sample, "result.key"), RESULT_KEY_TEXT + "\n", 0o600)
    specification = importlib.util.spec_from_file_location("broad_mutation_driver_for_samples", driver_path)
    driver = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(driver)
    library = driver.Library(os.path.join(sample, "home"))
    os.makedirs(library.home, exist_ok=True)
    library.result_key_override = os.path.join(sample, "result.key")
    arguments = argparse.Namespace(timeout_floor=900, memory_cap="24G", cpu_multiplier=3.0)
    with open(os.path.join(sample, "rustc-vv.txt"), encoding="utf-8") as handle:
        toolchain = handle.read().strip() + "\n"
    with open(os.path.join(sample, "tool-versions.txt"), encoding="utf-8") as handle:
        tools = [line.strip() for line in handle if line.strip()]
    judgement = driver.Judgement(library, root, arguments, for_run=False, exclusions=[], toolchain=toolchain, tools=tools)
    items = [{"package": package, "file": path, "start": list(start), "end": list(end), "replacement": replacement,
              "name": f"{path}:{start[0]}:{start[1]}: {description}"} for package, path, start, end, replacement, description in UNITS_MUTANTS]
    items = driver.canonical_items(items)
    driver.record_list_cache(library, judgement, items)
    caught_by = "singlefs-harness::catching the_case_that_catches_it"
    test_key, why = judgement.test_key_for(caught_by)
    if not test_key:
        raise SystemExit(f"样本里抓到它的那条测试算不出测试键：{why}")
    for item in items:
        current = judgement.key_of(item)
        library.write_composition(current["key"], current["composition"])
        if gpu_case and item["sequence"] == 1:
            # CPU 那一趟没红：覆盖者键取此刻够得着它的测试各自测试键的多重集（驱动的 coverers_for）；GPU 那一趟照驱动的函数记
            library.record_verdict([current["key"], item["sequence"], item["package"], item["name"], "missed", "two", "", "", judgement.coverers_for(current["unit"])[0] or "",
                                    "0.0", "1.0", driver.today_text(), "fresh", current["locator"], ",".join(current["refused"]), "样本"])
            if variant == "gpu-green":
                judging = driver.GpuJudging(library, judgement, arguments)
                case = "singlefs-checker-tier::gpu_pair gpu_case"
                gpu_test_key, gpu_why = judging.catching_key(case)
                if not gpu_test_key:
                    raise SystemExit(f"样本里 GPU 趟抓到它的那条测试算不出测试键：{gpu_why}")
                driver.record_gpu_verdict(library, [current["key"], item["sequence"], item["package"], item["name"], "caught", case, gpu_test_key, "", judging.pass_key(),
                                                   "1.0", driver.today_text(), "fresh", "样本"])
            continue
        library.record_verdict([current["key"], item["sequence"], item["package"], item["name"], "caught", "one", caught_by, test_key, "", "0.0", "1.0",
                                driver.today_text(), "fresh", current["locator"], ",".join(current["refused"]), "样本"])
    for path, text in UNITS_CHANGES.get(variant, {}).items():
        write(os.path.join(root, path), text)


def main(arguments):
    if len(arguments) != 3:
        print("  ✗ 用法：broad-mutation-samples.py <驱动> <样本根> <情形>", file=sys.stderr)
        print("  → 怎么办：它只由门禁 checker-tier-broad-mutation 在样本模式下起，情形写在样本根 .broad-mutation-sample 的 scenario= 那一行", file=sys.stderr)
        return 2
    driver_path, root, scenario = arguments
    family, _colon, variant = scenario.partition(":")
    if family == "summary" and variant in RECORDED:
        lines, exit_code = RECORDED[variant]
        write_summary(root, lines, exit_code)
    elif family == "escalated" and variant in ESCALATED_VARIANTS:
        build_escalated(root, variant)
    elif family == "gpu" and variant in GPU_VARIANTS:
        build_gpu(root, variant)
    elif family == "units" and variant in UNITS_VARIANTS:
        shutil.rmtree(os.path.join(root, ".broad-sample", "home"), ignore_errors=True)
        build_units(driver_path, root, variant)
    else:
        print(f"  ✗ 认不出的样本情形「{scenario}」", file=sys.stderr)
        print(f"  → 怎么办：写 summary:<{'/'.join(RECORDED)}>、escalated:<{'/'.join(ESCALATED_VARIANTS)}>、gpu:<{'/'.join(GPU_VARIANTS)}> 或 units:<{'/'.join(UNITS_VARIANTS)}>", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
