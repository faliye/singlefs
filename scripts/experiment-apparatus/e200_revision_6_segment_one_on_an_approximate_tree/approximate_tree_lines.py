"""E200（修订6近似树重跑） 第一段的读数：问题单 Q1 那几行（登记第六节 Q1-逐份、Q1-装置、Q1-研究一侧、Q1-未钉、Q1-全树）与 Q2-编那几行。

三个子命令（都只读，不改树）：
  q1 --repository <仓根> --built <造树的输出目录> --product <新产物>
      钉值另从 git 现取（不读造树产物里的那一份）：build_approximate_tree.load_pins / research_side_pins；先核树与逐文件清单逐份相同（作废 Z40 的造树那一刻），
      再逐份比十五份钉值、按登记第一节读法 3 判 Q1 的候选。退出码：0 判完；1 判完而树与清单对不上、或 Q1-全树相等而 Q1-逐份有不等（停机 S40，判法自相矛盾）；
      2 用法错或产物已存在；3 存档产物或钉值表与登记不符（同造树的退 3）。
  tree-digest <树根>
      打一行 `<快照摘要> <文件数>`（同 build_approximate_tree.snapshot_digest）；编译脚本编之前、编之后各调一次。
  compile-lines --tree <树根> --exit-code <cargo 的退出码> --copy-exit-code <拷二进制那两条 cp 的退出码> --build-output <这一趟编译目录的 release/>
                --log <cargo 的诊断> --digest-before <编之前的快照摘要> --product <新产物>
      Q2-编：cargo 退出码、拷的退出码、诊断末 20 行（家目录写成 ~）、两个二进制在树里 target/release/ 与在这一趟编译目录里的 sha256、两份逐字节同不同、
      编之前编之后快照摘要同不同。编过 = cargo 退 0、cp 退 0、两个二进制在树里都在且与这一趟的构建输出逐字节相同（树里那份在、却不是这一趟拷进来的旧二进制，不算编过）。
      退出码：0 编过且树没变；1 没编过（含没拷好）或树变了；2 用法错。
"""

from __future__ import annotations

import argparse
import filecmp
import os
import sys

import build_approximate_tree as tree_builder
from result_lines import ResultLines, field

REVISION_6_SNAPSHOT_DIGEST = "7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4"   # 登记 K40
REVISION_6_DIGEST_FILES = 284
CANDIDATE_DEVICE_AND_RESEARCH = "only_device_and_research_side"
CANDIDATE_PRODUCT = "reaches_product_code_or_pool_checker"
THIRD_KIND = "third_kind_reaches_other_verification_packages"
BINARIES = ("e171_three_oracle_collusion_after_the_oracle_fixes", "e170_o3_spec_executor_export")
DIAGNOSTIC_TAIL = 20
ABSENT_ROW = {"marker": tree_builder.MARKER_MISSING, "source": "-", "blob": "-", "sha256": "-"}   # 清单里没有这条路径时 Q1 那几行照这一行写


def pin_holds(near_sha256: str, target_sha256: str) -> bool:
    """Q1-逐份的「相等」：内容 sha256 逐字节相等（登记第五节 5.1 Q1 右栏：不做任何归一）。"""
    return near_sha256 == target_sha256


def q1_candidate(classes_of_unequal_pins: list[str]) -> str:
    """登记第一节读法 3：十五份钉值里不等的那几份落在哪一类。有一份在产品代码与池级 checker → 候选二；
    没有、而有在别的验证包 → 第三种；其余（只在装置与研究一侧，含一份不等都没有）→ 候选一。认不出类的单独报。"""
    if tree_builder.CLASS_PRODUCT in classes_of_unequal_pins:
        return CANDIDATE_PRODUCT
    if tree_builder.CLASS_OTHER in classes_of_unequal_pins:
        return THIRD_KIND
    if tree_builder.CLASS_UNCLASSIFIED in classes_of_unequal_pins:
        return tree_builder.CLASS_UNCLASSIFIED
    return CANDIDATE_DEVICE_AND_RESEARCH


def read_manifest(path: str) -> list[dict[str, str]]:
    with open(path, encoding="utf-8") as handle:
        header = handle.readline().rstrip("\n").split("\t")
        return [dict(zip(header, line.rstrip("\n").split("\t"))) for line in handle if line.strip()]


def tree_against_manifest(built: str, rows: list[dict[str, str]]) -> tuple[int, int, int]:
    """逐份重算树里与研究一侧每一份的 sha256 与清单比：交回 (清单里有内容的份数, 对不上或不在的份数, 树里多出来的份数)。
    树里 target/ 下的不算（编出来的二进制拷进 tree/target/release/）。"""
    places = {"tree": os.path.join(built, "tree"), "research_side": os.path.join(built, "research-side")}
    listed = {(row["place"], row["path"]) for row in rows if row["sha256"] != "-"}
    mismatched = 0
    for place, path in sorted(listed):
        target = os.path.join(places[place], path)
        if not os.path.isfile(target) or tree_builder.sha256_of_file(target) != next(row["sha256"] for row in rows if (row["place"], row["path"]) == (place, path)):
            mismatched += 1
    extra = 0
    for place, root in places.items():
        for directory, subdirectories, files in os.walk(root):
            if place == "tree" and os.path.relpath(directory, root) == ".":
                subdirectories[:] = [name for name in subdirectories if name != "target"]
            for name in files:
                extra += (place, os.path.relpath(os.path.join(directory, name), root)) not in listed
    return len(listed), mismatched, extra


def q1_lines(built: str, pins: list, research_pins: list, lines: ResultLines) -> int:
    rows = read_manifest(os.path.join(built, "manifest.tsv"))
    listed, mismatched, extra = tree_against_manifest(built, rows)
    lines.result(f"name=tree_matches_manifest listed={listed} mismatched={mismatched} extra={extra} holds={str(mismatched == 0 and extra == 0).lower()}")
    tree_rows = {row["path"]: row for row in rows if row["place"] == "tree"}
    research_rows = {row["path"]: row for row in rows if row["place"] == "research_side"}
    unequal_classes = []
    for pin in pins:
        row = tree_rows.get(pin.path, ABSENT_ROW)
        equals = pin_holds(row["sha256"], pin.target_sha256)
        path_class = tree_builder.classify(pin.path)
        if not equals:
            unequal_classes.append(path_class)
        device = (f" new_tree_sha256={tree_builder.NEW_TREE_DEVICE_SHA256} equals_new_tree={str(row['sha256'] == tree_builder.NEW_TREE_DEVICE_SHA256).lower()}"
                  if pin.path == tree_builder.DEVICE_PATH else "")
        lines.result(f"name=q1_file path={pin.path} class={path_class} role={pin.role} target_sha256={pin.target_sha256} marker={row['marker']} "
                     f"source={field(row['source'])} blob={row['blob']} near_sha256={row['sha256']} equals={str(equals).lower()}{device}")
    for path, pinned, outside in research_pins:
        row = research_rows.get(path, ABSENT_ROW)
        lines.result(f"name=q1_research_side path={path} pinned_outside_the_repository={str(outside).lower()} target_sha256={pinned} "
                     f"marker={row['marker']} source={field(row['source'])} blob={row['blob']} near_sha256={row['sha256']} "
                     f"equals={str(pin_holds(row['sha256'], pinned)).lower()}")
    for path in tree_builder.UNRECORDED_RESEARCH_FILES:
        row = research_rows.get(path, ABSENT_ROW)
        lines.result(f"name=q1_research_side path={path} target_sha256=unrecorded marker={row['marker']} near_sha256={row['sha256']}")
    pinned_paths = {pin.path for pin in pins}
    unpinned = [row for row in tree_rows.values() if row["status"] != "none" and row["path"] not in pinned_paths]
    tally: dict[tuple[str, str], int] = {}
    for row in sorted(unpinned, key=lambda item: item["path"]):
        tally[(row["class"], row["marker"])] = tally.get((row["class"], row["marker"]), 0) + 1
        lines.result(f"name=q1_unpinned_path path={row['path']} class={row['class']} status={row['status']} marker={row['marker']} source={field(row['source'])}")
    for (path_class, marker), count in sorted(tally.items()):
        lines.result(f"name=q1_unpinned class={path_class} marker={marker} count={count}")
    digest, count = tree_builder.snapshot_digest(os.path.join(built, "tree"))
    whole_equal = digest == REVISION_6_SNAPSHOT_DIGEST and count == REVISION_6_DIGEST_FILES
    lines.result(f"name=q1_whole_tree snapshot_digest={digest} files={count} target={REVISION_6_SNAPSHOT_DIGEST} target_files={REVISION_6_DIGEST_FILES} "
                 f"equals={str(whole_equal).lower()}")
    contradiction = whole_equal and bool(unequal_classes)
    by_class = ",".join(f"{name}:{unequal_classes.count(name)}" for name in sorted(set(unequal_classes))) or "none"
    lines.result(f"name=verdict q1_candidate={q1_candidate(unequal_classes)} pins={len(pins)} unequal_pins={len(unequal_classes)} unequal_by_class={by_class} "
                 f"unpinned_paths={len(unpinned)} tree_mismatches={mismatched + extra} s40_whole_tree_equal_but_a_pin_unequal={str(contradiction).lower()}")
    return 1 if contradiction or mismatched or extra else 0


def same_bytes(left: str, right: str) -> bool:
    """两份文件都在、内容逐字节相同。"""
    return os.path.isfile(left) and os.path.isfile(right) and filecmp.cmp(left, right, shallow=False)


def compile_lines(tree: str, exit_code: int, copy_exit_code: int, build_output: str, log: str, digest_before: str, lines: ResultLines) -> int:
    """Q2-编：cargo 退出码、拷的退出码、诊断末 DIAGNOSTIC_TAIL 行、两个二进制在树里与在构建输出里的 sha256 与逐字节同不同、编之前编之后树同不同。"""
    with open(log, encoding="utf-8", errors="replace") as handle:
        tail = handle.read().splitlines()[-DIAGNOSTIC_TAIL:]
    home = os.path.expanduser("~")
    for index, text in enumerate(tail, start=1):
        lines.result(f"name=compile_diagnostic index={index} text={field(text.replace(home, '~'))}")
    copied = {}
    for name in BINARIES:
        in_tree, built = os.path.join(tree, "target", "release", name), os.path.join(build_output, name)
        copied[name] = same_bytes(in_tree, built)
        lines.result(f"name=binary_sha256 binary={name} sha256={tree_builder.sha256_of_file(in_tree) if os.path.isfile(in_tree) else 'missing'} "
                     f"build_output_sha256={tree_builder.sha256_of_file(built) if os.path.isfile(built) else 'missing'} "
                     f"same_as_build_output={str(copied[name]).lower()}")
    digest_after, count = tree_builder.snapshot_digest(tree)
    unchanged = digest_after == digest_before
    compiled = exit_code == 0 and copy_exit_code == 0 and all(copied.values())
    lines.result(f"name=verdict cargo_exit={exit_code} copy_exit={copy_exit_code} compiled={str(compiled).lower()} digest_before={digest_before} "
                 f"digest_after={digest_after} digest_files={count} tree_unchanged={str(unchanged).lower()}")
    return 0 if compiled and unchanged else 1


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E200 第一段的读数：Q1 与 Q2-编")
    commands = parser.add_subparsers(dest="command", required=True)
    q1 = commands.add_parser("q1")
    q1.add_argument("--repository", required=True)
    q1.add_argument("--built", required=True)
    q1.add_argument("--product", required=True)
    digest = commands.add_parser("tree-digest")
    digest.add_argument("tree")
    compiled = commands.add_parser("compile-lines")
    compiled.add_argument("--tree", required=True)
    compiled.add_argument("--exit-code", required=True, type=int)
    compiled.add_argument("--copy-exit-code", required=True, type=int)
    compiled.add_argument("--build-output", required=True)
    compiled.add_argument("--log", required=True)
    compiled.add_argument("--digest-before", required=True)
    compiled.add_argument("--product", required=True)
    arguments = parser.parse_args(argument_list)
    if arguments.command == "tree-digest":
        print(*tree_builder.snapshot_digest(arguments.tree))
        return 0
    if os.path.exists(arguments.product):
        print(f"产物 {arguments.product} 已存在：不覆盖", file=sys.stderr)
        return 2
    if arguments.command == "compile-lines":
        lines = ResultLines(arguments.product)
        lines.result(f"name=config tree={field(tree_builder.home_relative(arguments.tree))} binaries={','.join(BINARIES)} diagnostic_tail={DIAGNOSTIC_TAIL}")
        exit_code = compile_lines(arguments.tree, arguments.exit_code, arguments.copy_exit_code, arguments.build_output, arguments.log,
                                  arguments.digest_before, lines)
        lines.finish()
        return exit_code
    repository = tree_builder.GitRepository(os.path.abspath(arguments.repository))
    try:
        pins, research_pins = tree_builder.load_pins(repository), tree_builder.research_side_pins(repository)
    except tree_builder.ArchivedInputMismatch as problem:
        print(f"作废 Z42 那一类：{problem}", file=sys.stderr)
        return 3
    lines = ResultLines(arguments.product)
    lines.result(f"name=config built={field(tree_builder.home_relative(arguments.built))} manifest_sha256="
                 f"{tree_builder.sha256_of_file(os.path.join(arguments.built, 'manifest.tsv'))} pins={len(pins)} research_side_pins={len(research_pins)}")
    exit_code = q1_lines(arguments.built, pins, research_pins, lines)
    lines.finish()
    return exit_code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
