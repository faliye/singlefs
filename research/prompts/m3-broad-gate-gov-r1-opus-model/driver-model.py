#!/usr/bin/env python3
"""攻方腿模型（m3-broad-gate-gov-r1，G1）：把驱动 research/scripts/broad-mutation.py 当模块载入，只调纯函数（version_key、global_key、
mutation-prune 的 carry_decision、select_items、summarise），库目录是草稿目录里手造的；不起 cargo、不起驱动的真跑、不碰真库目录。
global_key 会起 rustc -Vv、cargo nextest --version、cargo mutants --version、llvm-profdata --version（只问版本）。
用法：driver-model.py <tree> <草稿目录>
情形一（全局键变了，版本指纹不变）：这一版里已有现判的结论，换排除表 / 换 CPU 限时倍数之后：版本指纹相同、全局键不同；
  --changed-since 的计划对每条都是「全局键变了、要现判」，select_items 照样一条现判过的都不选；--summary 判绿退 0。
情形二（超时与内存撞顶）：这一版里一条是 timeout、一条是 memory-cap：--summary 判红退 8 列成 NO_CONCLUSION；
  它给的出路是 --changed-since 补齐，可 --changed-since 的计划要现判它们时 select_items 也不选，--item 点名不带 --rerun 同样跳过。"""
import argparse, contextlib, importlib.util, io, json, os, shutil, sys
tree, scratch = sys.argv[1:3]
specification = importlib.util.spec_from_file_location("driver_for_model", os.path.join(tree, "research/scripts/broad-mutation.py"))
driver = importlib.util.module_from_spec(specification)
specification.loader.exec_module(driver)
driver.TEST_MODE = False
rules = driver.prune_rules()

ITEMS = [{"sequence": number, "package": "singlefs-core", "name": f"crates/singlefs-core/src/sample.rs:{number}:9: replace x{number}",
          "file": "crates/singlefs-core/src/sample.rs", "start": [number, 9], "end": [number, 10], "replacement": "y"} for number in (1, 2, 3)]

def library_with(name, rows):
    home = os.path.join(scratch, "driver-model", name)
    shutil.rmtree(home, ignore_errors=True)
    os.makedirs(os.path.join(home, "tree"))
    manifest = {"Cargo.toml": "a" * 64, "Cargo.lock": "b" * 64, "crates/singlefs-core/src/sample.rs": "c" * 64}
    toolchain = driver.toolchain_text(os.path.join(home, "tree"))
    source_fingerprint = driver.fingerprint_of_manifest(manifest)
    fingerprint = driver.version_key(home, source_fingerprint, toolchain)
    version = os.path.join(home, "versions", fingerprint)
    os.makedirs(version)
    with open(os.path.join(version, "manifest.tsv"), "w", encoding="utf-8") as handle:
        handle.write("".join(f"{manifest[path]}\t{path}\n" for path in sorted(manifest)))
    with open(os.path.join(version, "list.json"), "w", encoding="utf-8") as handle:
        json.dump(ITEMS, handle)
    with open(os.path.join(version, "results.tsv"), "w", encoding="utf-8") as handle:
        handle.write("# " + "\t".join(driver.RESULT_COLUMNS) + "\n")
        for sequence, outcome, stage in rows:
            item = ITEMS[sequence - 1]
            handle.write("\t".join([str(sequence), item["package"], item["name"], outcome, stage, "", "0.0", "0.0", "2026-10-01T00:00:00Z", fingerprint, "模型"]) + "\n")
    with open(os.path.join(home, "tree.source"), "w", encoding="utf-8") as handle:
        json.dump({"commit": "c" * 40, "fingerprint": fingerprint, "source_fingerprint": source_fingerprint, "export_paths": ["crates", "Cargo.toml", "Cargo.lock"]}, handle)
    library = driver.Library(home)
    library.load_source(for_run=False)
    library.load_manifest()
    library.load_items()
    return library, source_fingerprint, toolchain

def arguments(**overrides):
    values = dict(skip_stage_two=False, stage_two_only=False, item=[], rerun=False, start_from=None, package=None, shard=None, milestone_full=False,
                  cpu_multiplier=rules.DEFAULT_CPU_MULTIPLIER, cpu_floor=rules.DEFAULT_CPU_FLOOR_SECONDS, wall_multiplier=rules.DEFAULT_WALL_MULTIPLIER,
                  wall_floor=rules.DEFAULT_WALL_FLOOR_SECONDS, llvm_bin=None)
    values.update(overrides)
    return argparse.Namespace(**values)

def summary(library):
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer), contextlib.redirect_stderr(buffer):
        code = driver.summarise(library)
    lines = [line for line in buffer.getvalue().splitlines() if line.startswith(("BROAD_MUTATION_GATE", "NO_CONCLUSION", "  → 怎么办"))]
    return code, lines

# 情形一
library, source_fingerprint, toolchain = library_with("global-key", [(1, "caught", "one"), (2, "unviable", "one"), (3, "caught", "carried")])
exclusions_old = os.path.join(scratch, "driver-model", "exclusions-old.tsv")
exclusions_new = os.path.join(scratch, "driver-model", "exclusions-new.tsv")
shutil.copy(driver.EXCLUSIONS_FILE, exclusions_old)
shutil.copy(driver.EXCLUSIONS_FILE, exclusions_new)
with open(exclusions_new, "a", encoding="utf-8") as handle:
    handle.write("singlefs-harness::model_only some_new_slow_case\t模型：排除表加一行\n")
keys = {}
for label, exclusions, multiplier in (("排除表旧、CPU 倍数 3", exclusions_old, 3.0), ("排除表加一行、CPU 倍数 3", exclusions_new, 3.0), ("排除表旧、CPU 倍数 4", exclusions_old, 4.0)):
    driver.EXCLUSIONS_FILE = exclusions
    _text, keys[label] = driver.global_key(library, arguments(cpu_multiplier=multiplier))
    print(f"[一] {label}：版本指纹 {driver.version_key(library.home, source_fingerprint, toolchain)[:16]}，全局键 {keys[label][:16]}")
old_key = keys["排除表旧、CPU 倍数 3"]
record = {"outcome": "caught", "global_key": old_key, "fingerprint": "f", "dependency": "", "test_fingerprint": "t"}
current = {"fingerprint": "f", "dependency": ""}
print(f"[一] 结论表里那一行记的是旧全局键，carry_decision 在新全局键下交 {rules.carry_decision(record, current, keys['排除表加一行、CPU 倍数 3'], 't')}")
plans = {item["sequence"]: (True, "global-key-changed", record) for item in ITEMS}
chosen = [item["sequence"] for item, _one, _two in driver.select_items(library, arguments(), plans)]
print(f"[一] --changed-since 的计划三条都要现判（全局键变了），select_items 选中的序号：{chosen}（这一版里已有现判结论的 1、2 不重判）")
code, lines = summary(library)
print(f"[一] 不跑 ③ 直接 --summary 退 {code}：{' | '.join(lines)}")

# 情形二
library, _source, _toolchain = library_with("environmental", [(1, "caught", "one"), (2, "timeout", "two"), (3, "memory-cap", "one")])
code, lines = summary(library)
print(f"[二] --summary 退 {code}：{' | '.join(lines)}")
plans = {item["sequence"]: (True, "outcome-timeout" if item["sequence"] == 2 else "no-record", None) for item in ITEMS}
chosen = [item["sequence"] for item, _one, _two in driver.select_items(library, arguments(), plans)]
print(f"[二] 照出路 --changed-since：计划三条都要现判，select_items 选中的序号：{chosen}")
buffer = io.StringIO()
with contextlib.redirect_stdout(buffer):
    chosen = [item["sequence"] for item, _one, _two in driver.select_items(library, arguments(item=["2", "3"]))]
print(f"[二] --item 2 --item 3 不带 --rerun：选中 {chosen}；驱动说：{' | '.join(line.strip() for line in buffer.getvalue().splitlines())}")
print(f"[二] 结论表只收 {driver.DECISIVE_OUTCOMES}（process_item 里 outcome in DECISIVE_OUTCOMES 才 record_conclusion），carry_decision 只沿用 {rules.CARRIED_OUTCOMES}：下一版里这两条也是现判，再超时、再撞顶就再红")
