#!/usr/bin/env python3
"""攻方腿 m3-close-gov-b17-r2 H4：拿驱动自己的 plan_gpu_import 判一张「用过一阵的」第二台 gpu-verdicts.tsv（只追加的库里留着旧行）。
只调导入前的核（plan_gpu_import 一行都不写）；本机清单与 GPU 趟自己的键用替身（read_only_judgement、GpuJudging 换成桩），不起 cargo、不碰任何库目录。
用法：python3 gpu_import_model.py --root <仓副本根> --work <新的草稿目录>"""
import argparse, argparse as _a, importlib.util, os, types
parser = argparse.ArgumentParser(); parser.add_argument("--root", required=True); parser.add_argument("--work", required=True)
arguments = parser.parse_args()
root, work = os.path.realpath(arguments.root), os.path.realpath(arguments.work)
os.makedirs(work, exist_ok=False)
specification = importlib.util.spec_from_file_location("broad_mutation_model", os.path.join(root, "scripts/mutation/broad-mutation.py"))
driver = importlib.util.module_from_spec(specification); specification.loader.exec_module(driver)

CURRENT_PASS, OLD_PASS = "1" * 64, "0" * 64      # 本机此刻 GPU 趟自己的键；第二台上一版驱动时的键
K1, K2, K_GONE = "a" * 64, "b" * 64, "c" * 64     # K1、K2 在本机此刻清单里；K_GONE 那条变异后来改了源码、键已不在清单里
def row(key, sequence, outcome, gpu_key, method="fresh", note=""):
    fields = {"key": key, "sequence": str(sequence), "package": "singlefs-core", "name": f"m{sequence}", "outcome": outcome,
              "caught_by": "lib gpu_case" if outcome == "caught" else "", "test_key": "d" * 64 if outcome == "caught" else "",
              "coverage_key": "" if outcome == "caught" else "e" * 64, "gpu_key": gpu_key, "test_seconds": "1.0",
              "judged_on": "2026-10-20", "method": method, "note": note, "mac": "f" * 64}
    return "\t".join(fields[column] for column in driver.GPU_VERDICT_COLUMNS)
peer = os.path.join(work, "peer-library")
os.makedirs(peer)
SCENARIOS = {
    "fresh-peer": [row(K1, 1, "missed", CURRENT_PASS), row(K2, 5, "caught", CURRENT_PASS)],
    "peer-with-history": [row(K1, 1, "missed", OLD_PASS), row(K_GONE, 3, "missed", CURRENT_PASS),          # 上一版驱动判过的 K1、已不在清单里的 K_GONE
                          row(K1, 1, "missed", CURRENT_PASS), row(K2, 5, "caught", CURRENT_PASS)],      # 这一趟两台各跑一片时第二台新判的
}
class Judgement:
    def key_of(self, item):
        return {"key": item["key"]}
class GpuJudgingStub:
    def __init__(self, library, judgement, arguments):
        self.table = types.SimpleNamespace(rows=[], forged=False, malformed_lines=[])
    def pass_key(self):
        return CURRENT_PASS
driver.read_only_judgement = lambda library, arguments, source: (Judgement(), [{"key": K1}, {"key": K2}], [], [])
driver.GpuJudging = GpuJudgingStub
for name, rows in SCENARIOS.items():
    directory = os.path.join(work, name)
    os.makedirs(directory)
    with open(os.path.join(directory, driver.GPU_VERDICTS), "w", encoding="utf-8") as handle:
        handle.write("# " + "\t".join(driver.GPU_VERDICT_COLUMNS) + "\n" + "".join(line + "\n" for line in rows))
    for attempt in ("first-import", "same-file-again"):
        plan = driver.plan_gpu_import(None, _a.Namespace(source_root=root), directory)
        would_exit = driver.EXIT_INCONSISTENT if (plan.rejected or plan.conflicts) else 0
        print(f"scenario={name} attempt={attempt} rows_read={plan.rows_read} fresh={len(plan.fresh)} rejected={len(plan.rejected)} "
              f"reasons={','.join(sorted(reason for _row, reason in plan.rejected)) or '-'} conflicts={len(plan.conflicts)} import_exit_would_be={would_exit}")
