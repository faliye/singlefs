"""门禁 broad-mutation 的样本现场：阶段在样本模式下（样本根放了 .broad-mutation-sample，里面一行 scenario=<情形>）开格之前起它，
在样本根里造好 gate-verdict 格要读的东西，阶段本身不知道各情形怎么造。

调用（阶段经参数把驱动与样本根传进来）：python3 <这份文件> <驱动 broad-mutation.py 的路径> <样本根> <情形>
情形分两族：
  summary:<名>     录好的驱动 --summary 输出：写 .broad-sample/summary.sh（打录好的那几行、退录好的码），格照它判，判定库里没有东西
  units:<名>       新键在真源码上的样子：样本根里写一棵小工作区（singlefs-core、singlefs-checker、singlefs-format），载入驱动、用它的函数照这棵树
                   把清单按单元记进判定库、每条记一行抓到（键是此刻算的单元键）；再照情形改源码。不写 summary.sh：
                   格起真的驱动 --summary（--toolchain-from / --tools-from 取样本里的两份文件），判法、单元键都是此刻驱动那一份
                   判定库是 prune-server 里那一份（驱动文件头「判定库」）：库目录 .broad-sample/home 只当名字、本地不建；样本根在临时目录底下
                   （共享门禁的 stage-selftest.sh 拷进 mktemp -d），客户端照它另占一份命名空间；要连得上服务，连不上客户端退 75、这份现场造不成
各情形改的那一处与期望写在 SCENARIOS 旁边的注释里；样本目录的 expect 写它该红还是该绿、该出现的句子。
"""
import argparse
import importlib.util
import os
import sys

SUMMARY_COUNTS = ("total", "caught", "missed", "unviable", "stage_one_only", "driver_error", "not_run", "refused", "unlisted_units",
                  "gpu_needed", "gpu_reused", "gpu_not_run", "gpu_without_conclusion")


def summary_line(**counts):
    values = dict.fromkeys(SUMMARY_COUNTS, 0)
    values.update(counts)
    return "BROAD_MUTATION_SUMMARY " + " ".join(f"{name}={values[name]}" for name in SUMMARY_COUNTS)


def gate_line(total, concluded, missed=0, without=0, refused=0, unlisted=0, verdict="green",
              gpu_needed=0, gpu_reused=0, gpu_not_run=0, gpu_without=0, gpu_fields=True):
    return (f"BROAD_MUTATION_GATE total={total} concluded={concluded} missed={missed} without_conclusion={without} refused={refused} unlisted_units={unlisted}"
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
    # GPU 那一趟（驱动文件头「GPU 那一趟」）：要 GPU 趟的 3 条都有库里键相同的结论（其中 2 条是 GPU 趟抓到的，所以六栏里是抓到）：绿
    "gpu-green": ([summary_line(total=6, caught=5, unviable=1, gpu_needed=3, gpu_reused=3), gate_line(6, 6, gpu_needed=3, gpu_reused=3)], 0),
    # 要 GPU 趟的 3 条里 2 条本次未跑（没卡）：它们不算没红也不算通过，是没结论，驱动判红退 8，格照抄 NO_CONCLUSION（理由 gpu-not-run）
    "gpu-no-conclusion": ([summary_line(total=6, caught=3, unviable=1, not_run=2, gpu_needed=3, gpu_reused=1, gpu_not_run=2),
                           gate_line(6, 4, without=2, verdict="red", gpu_needed=3, gpu_reused=1, gpu_not_run=2),
                           f"NO_CONCLUSION\t4\tnone\t{GPU_NAME_FOUR}\tgpu-not-run", f"NO_CONCLUSION\t5\tnone\t{GPU_NAME_FIVE}\tgpu-not-run"], 8),
    # 要 GPU 趟的 3 条里 1 条 GPU 趟驱动错（没结论）：驱动判红退 8
    "gpu-driver-error": ([summary_line(total=6, caught=4, unviable=1, driver_error=1, gpu_needed=3, gpu_reused=2, gpu_without_conclusion=1),
                          gate_line(6, 5, without=1, verdict="red", gpu_needed=3, gpu_reused=2, gpu_without=1), f"NO_CONCLUSION\t4\tdriver-error\t{GPU_NAME_FOUR}\tgpu-driver-error"], 8),
    # 判定行写 green、退 0、without_conclusion=0，而 gpu_not_run=2：驱动把没有 GPU 结论的条当有结论数了
    "gpu-silent-driver": ([summary_line(total=6, caught=5, unviable=1, gpu_needed=3, gpu_reused=1, gpu_not_run=2), gate_line(6, 6, gpu_needed=3, gpu_reused=1, gpu_not_run=2)], 0),
    # 判定行 green，gpu_needed=3 却只有 1 条复用、另两项是 0：三项加不起来
    "gpu-counts": ([summary_line(total=6, caught=5, unviable=1, gpu_needed=3, gpu_reused=1), gate_line(6, 6, gpu_needed=3, gpu_reused=1)], 0),
    # 判定行没有 gpu_ 四个数（旧驱动）
    "gpu-fields-missing": ([summary_line(total=6, caught=5, unviable=1), gate_line(6, 6, gpu_fields=False)], 0),
    # 汇总行与判定行的 GPU 趟计数不同
    "gpu-summary-mismatch": ([summary_line(total=6, caught=5, unviable=1, gpu_needed=3, gpu_reused=2, gpu_not_run=1), gate_line(6, 6, gpu_needed=3, gpu_reused=3)], 0),
    # 判定库服务不可用：驱动读判定库时客户端打它那两行、退 75（scripts/prune-server/prune_server_client.py 文件头「退出码」），一行汇总都没打
    "service-unavailable": (["  ✗ prune-server 客户端：连不上 prune-server：试了 30 次、120.0 秒（样本录的）（退 75）",
                             "     → 怎么办：看服务在不在跑（prune-server/control check）、第二台连的地址与端口通不通；服务回来之后重交这个任务；不要改成走本地库"], 75),
}
UNITS_VARIANTS = ("green", "callee-changed", "comment-only", "unlisted", "gpu-green", "gpu-no-conclusion", "driver-error")


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




UNITS_CORE = "pub fn one() -> u32 {\n    1\n}\nfn helper() -> u32 {\n    7\n}\npub fn three() -> u32 {\n    helper()\n}\n"
UNITS_CHECKER = "pub fn seven() -> u32 {\n    7\n}\n"
UNITS_TREE = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/singlefs-core", "crates/singlefs-checker", "crates/singlefs-format"]\nresolver = "2"\n',
    "crates/singlefs-core/Cargo.toml": '[package]\nname = "singlefs-core"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/singlefs-core/src/lib.rs": UNITS_CORE,
    "crates/singlefs-checker/Cargo.toml": '[package]\nname = "singlefs-checker"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/singlefs-checker/src/lib.rs": UNITS_CHECKER,
    "crates/singlefs-format/Cargo.toml": '[package]\nname = "singlefs-format"\nversion = "0.1.0"\nedition = "2021"\n',
    "crates/singlefs-format/src/lib.rs": "",
    "crates/singlefs-format/tests/catching.rs": "#[test]\nfn the_case_that_catches_it() {\n    assert_eq!(1 + 1, 2);\n}\n",
}
UNITS_MUTANTS = (("singlefs-core", "crates/singlefs-core/src/lib.rs", (2, 5), (2, 6), "0", "replace one -> u32 with 0"),
                 ("singlefs-core", "crates/singlefs-core/src/lib.rs", (5, 5), (5, 6), "0", "replace helper -> u32 with 0"),
                 ("singlefs-core", "crates/singlefs-core/src/lib.rs", (8, 5), (8, 13), "0", "replace three -> u32 with 0"),
                 ("singlefs-checker", "crates/singlefs-checker/src/lib.rs", (2, 5), (2, 6), "0", "replace seven -> u32 with 0"))
# units 族改源码的那一处：callee-changed 被调的 helper 改了函数体（three 里那一条的闭包里有它，要现判；helper 自己的单元变了，那个单元没记清单）；
# comment-only 只加注释（零条要现判）；unlisted 新加一个函数（那个单元没记清单）；green 不改；
# driver-error 第 2 条（helper）只有一行 driver-error（没结论）
UNITS_CHANGES = {
    "green": {},
    "callee-changed": {"crates/singlefs-core/src/lib.rs": UNITS_CORE.replace("    7\n", "    8\n")},
    "comment-only": {"crates/singlefs-core/src/lib.rs": "// 只加注释：函数单元的指纹不变\npub fn one() -> u32 {\n    // 体里也加一行\n    1\n}\n/* 块注释 */ fn helper() -> u32 {\n    7 // 行尾注释\n}\n"
                                                         "pub fn three() -> u32 {\n    helper()\n}\n"},
    "unlisted": {"crates/singlefs-core/src/lib.rs": UNITS_CORE + "pub fn nine() -> u32 {\n    9\n}\n"},
}


# units 族的 GPU 情形：UNITS_TREE 加一个 验证包（特性 gpu、依赖 singlefs-core），tests/gpu_pair.rs 的测试体里有挂 gpu 特性的语句、经 judge() 够得着 one；
# 第 1 条（one）CPU 那一趟没红，别的条抓到。gpu-green：第 1 条的 GPU 判定行是抓到；gpu-no-conclusion：GPU 趟还没有结论。
UNITS_GPU_FILES = {
    "Cargo.toml": UNITS_TREE["Cargo.toml"].replace('"crates/singlefs-format"]', '"crates/singlefs-format", "crates/singlefs-verification"]'),
    "crates/singlefs-verification/Cargo.toml": '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n\n[features]\ngpu = []\n\n'
                                              '[dependencies]\nsinglefs-core = { path = "../singlefs-core" }\n',
    "crates/singlefs-verification/src/lib.rs": "pub fn judge() -> u32 {\n    singlefs_core::one()\n}\n",
    "crates/singlefs-verification/tests/gpu_pair.rs": '#[test]\nfn gpu_case() {\n    #[cfg(feature = "gpu")]\n    let extra = 1;\n    #[cfg(not(feature = "gpu"))]\n    let extra = 0;\n'
                                                      '    assert!(singlefs_verification::judge() + extra > 0);\n}\n',
}


def build_units(driver_path, root, variant):
    """units 族（情形名见 UNITS_VARIANTS）：照驱动此刻的函数在样本根这棵小工作区上记清单与判定，再照 UNITS_CHANGES 改源码。
    gpu-green、gpu-no-conclusion 另加 UNITS_GPU_FILES 那个 验证包：第 1 条（one）CPU 那一趟记没红，GPU 判定行照驱动的函数记（gpu-green 抓到、gpu-no-conclusion 不记）；
    driver-error 第 2 条记一行 driver-error。"""
    gpu_case = variant.startswith("gpu-")
    for path, text in {**UNITS_TREE, **(UNITS_GPU_FILES if gpu_case else {})}.items():
        write(os.path.join(root, path), text)
    sample = os.path.join(root, ".broad-sample")
    write(os.path.join(sample, "rustc-vv.txt"), "rustc 1.0.0-sample (样本)\nhost: x86_64-unknown-linux-gnu\n")
    write(os.path.join(sample, "tool-versions.txt"), "cargo-nextest 0.0.0-sample\ncargo-mutants 0.0.0-sample\nllvm-profdata 0.0.0-sample\n")
    specification = importlib.util.spec_from_file_location("broad_mutation_driver_for_samples", driver_path)
    driver = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(driver)
    library = driver.Library(os.path.join(sample, "home"))     # 判定库在 prune-server 里，库目录只当命名空间的名字、不建
    arguments = argparse.Namespace()     # 只读的判法（for_run=False）不读跑法参数
    with open(os.path.join(sample, "rustc-vv.txt"), encoding="utf-8") as handle:
        toolchain = handle.read().strip() + "\n"
    with open(os.path.join(sample, "tool-versions.txt"), encoding="utf-8") as handle:
        tools = [line.strip() for line in handle if line.strip()]
    judgement = driver.Judgement(library, root, arguments, for_run=False, exclusions=[], toolchain=toolchain, tools=tools)
    items = [{"package": package, "file": path, "start": list(start), "end": list(end), "replacement": replacement,
              "name": f"{path}:{start[0]}:{start[1]}: {description}"} for package, path, start, end, replacement, description in UNITS_MUTANTS]
    items = driver.canonical_items(items)
    driver.record_list_cache(library, judgement, items)
    caught_by = "singlefs-format::catching the_case_that_catches_it"
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
                case = "singlefs-verification::gpu_pair gpu_case"
                gpu_test_key, gpu_why = judging.catching_key(case)
                if not gpu_test_key:
                    raise SystemExit(f"样本里 GPU 趟抓到它的那条测试算不出测试键：{gpu_why}")
                driver.record_gpu_verdict(library, [current["key"], item["sequence"], item["package"], item["name"], "caught", case, gpu_test_key, "", judging.pass_key(),
                                                   "1.0", driver.today_text(), "fresh", "样本"])
            continue
        if variant == "driver-error" and item["sequence"] == 2:
            library.record_verdict([current["key"], item["sequence"], item["package"], item["name"], driver.DRIVER_ERROR, "one", "", "", "", "0.0", "1.0",
                                    driver.today_text(), "fresh", current["locator"], ",".join(current["refused"]), "one-test 退 104（认不出的退出码，样本）"])
            continue
        library.record_verdict([current["key"], item["sequence"], item["package"], item["name"], "caught", "one", caught_by, test_key, "", "0.0", "1.0",
                                driver.today_text(), "fresh", current["locator"], ",".join(current["refused"]), "样本"])
    for path, text in UNITS_CHANGES.get(variant, {}).items():
        write(os.path.join(root, path), text)


def main(arguments):
    if len(arguments) != 3:
        print("  ✗ 用法：broad-mutation-samples.py <驱动> <样本根> <情形>", file=sys.stderr)
        print("  → 怎么办：它只由门禁 broad-mutation 在样本模式下起，情形写在样本根 .broad-mutation-sample 的 scenario= 那一行", file=sys.stderr)
        return 2
    driver_path, root, scenario = arguments
    family, _colon, variant = scenario.partition(":")
    if family == "summary" and variant in RECORDED:
        lines, exit_code = RECORDED[variant]
        write_summary(root, lines, exit_code)
    elif family == "units" and variant in UNITS_VARIANTS:
        build_units(driver_path, root, variant)
    else:
        print(f"  ✗ 认不出的样本情形「{scenario}」", file=sys.stderr)
        print(f"  → 怎么办：写 summary:<{'/'.join(RECORDED)}> 或 units:<{'/'.join(UNITS_VARIANTS)}>", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
