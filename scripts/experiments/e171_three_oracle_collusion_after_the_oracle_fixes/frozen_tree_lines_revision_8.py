#!/usr/bin/env python3
"""E171 第 1 次重跑第一段·修订 8 那一趟：第 0 步判法与拍树前后几核（登记 research/prompts/e171-r1-prereg.md 修订 8 第 8.1、8.3、8.6 节，S16 ①–④、A12）。
修订 6 的判法 frozen_tree_lines_revision_6.py 与它那几份 fixture 不读；A12 十二项照搬第一段新树那份 frozen_tree_lines.py 的 GREPS（原样抄在下面）。
只读树、主仓、配置点名的文件，写一份新产物（已存在就拒绝）；配置是 revision_8_placeholders.json 那一形（占位 H1–H20，登记第 8.8 节）。

用法（都在仓根跑，--config 指占位配置；--output 是新产物路径，给 - 写到标准输出）：
  frozen_tree_lines_revision_8.py before-snapshot --config C --output P
      S16 ④：拍树前五项核对（第 8.1 节第 2 条 a–e），一项一行，末行 verdict。退 0 五项都过，退 1 有不过的（不拍）。
  frozen_tree_lines_revision_8.py at-snapshot --config C --output P
      拷完树、编之前：S16 ①（树对仓逐文件差、文件数）、H2（HEAD 与 git status 全文）、编前快照摘要、H4、H8、H9。退 0 S16 ① 不触发，退 1 触发。
  frozen_tree_lines_revision_8.py after-build --config C --output P
      树里 release 编完：编后快照摘要等于 at-snapshot 那份产物里的编前摘要、两个二进制的 sha256（H7）。退 0 相同，退 1 不同。
  frozen_tree_lines_revision_8.py step-0 --config C --output P
      第 0 步第一份产物（…-frozen-tree-r8.out）：抄 S16 ① 那几行、Z5、装置四处、S16 ②③、HEAD、A12、H7、H8、H9、H11、修订 6 树对修订 8 树的差表。
      退 0 S16 ①②③、A12、Z5、H7 都不触发（研究一侧有不同不算触发，另报 research_side_differs），退 1 有触发。
  frozen_tree_lines_revision_8.py prerequisites --config C
      每次起跑与续跑：快照摘要等于 H3、两个二进制等于 H7、仓里装置等于树里那份或只差认过的块（S16 ②）、树里 crates/mutations.tsv 等于 H8。
      结果打到标准输出；退 0 都过，退 1 有不过的。O3 冻结行（Z2）由单元编排另核。
  frozen_tree_lines_revision_8.py p1f-base-digest --directory D --output P
      第 2b 步之后：E170 第 2 次重跑坏状态目录里 P1-F 那一行的号与底态摘要（H12）。
  frozen_tree_lines_revision_8.py --selftest [--cells 格,…] [--rounds N]
退出码：0、1 见各子命令；2 用法错（含平常跑时设了弄坏开关）；78 配置读不到、或点名的树与文件不在。

弄坏开关 FROZEN_TREE_R8_BREAK=<项>（只在 --selftest 起的子进程里认，FROZEN_TREE_R8_SELFTEST=1）。项名、弄坏哪一处、打开后红在哪一格：
  s16-1-paths-only            S16 ① 只比路径、不比 sha256                        snapshot-table-content-differs
  s16-1-counts-target         逐文件表把 target/ 也收进来                        snapshot-table-ignores-target
  s16-2-no-known-differences  S16 ② 不撤认过的已知差额                          s16-2-known-difference-removed
  s16-3-any-source            块表的来源不核是不是那三种                        s16-3-source-kind
  s16-3-fuzz                  打块时容模糊（--fuzz=2）                           s16-3-blocks-apply
  fixture-timestamps-allowed  fixture 头两行带修改时刻也收                       fixture-header-timestamps
  class-a-missing-passes      甲类那一行在草稿产物里没有也算过                   class-a-checks
  announcement-count-ignored  宣布行不核恰一行                                   announcement-count
  after-sha-first-wins        同一文件几份 after.sha256 取先出现的那份           after-sha256-last-wins
  merge-process-ignored       不看在跑的合入验证                                 merge-process
  a12-zero-as-at-least-one    A12「恰 0」那几项当「≥ 1」判                        a12-greps
  anchors-count-substring     十五行的锚点按行名子串取                           named-row-anchors
  step-0-ignores-z5           第 0 步不核快照摘要等于 H3                         step-0-end-to-end-red
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import bad_implementation_table  # noqa: E402
import row_driver  # noqa: E402
from result_lines import ResultLines, field  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402

BREAK_VARIABLE = "FROZEN_TREE_R8_BREAK"
SELFTEST_VARIABLE = "FROZEN_TREE_R8_SELFTEST"
EXIT_RUN_CONDITION = 78
DEVICE = "crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs"
E171_BINARY = "e171_three_oracle_collusion_after_the_oracle_fixes"
E170_BINARY = "e170_o3_spec_executor_export"
MUTATIONS_TABLE = "crates/mutations.tsv"
FINAL_MAINLINE_TREE = "scripts/experiments/target/e171-r1-frozen-tree-final-mainline"
REVISION_6_TREE = "scripts/experiments/target/e171-r1-frozen-tree-revision-6"
# 第 8.3 节「后两个现在就定」：修订 6 那一趟第 0 步核过的仓里装置，与修订 6 树里那份
BASE_DEVICE_SHA256 = "ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df"
REVISION_6_TREE_DEVICE_SHA256 = "67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156"
SNAPSHOT_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "crates")
ANNOUNCEMENT_PREFIX = "- E171 最终主线拍树宣布"
RECORDS_FILE = "records/2026-10-01-里程碑三形式化接入.md"
PREREGISTRATION = "research/prompts/e171-r1-prereg.md"
REVISION_9_HEADING = "### 修订 9"
MERGE_VERIFY_SCRIPT = "merge-and-verify.sh"
BLOCK_SOURCE_KINDS = ("revision_9_constants", "core_refusal_arm", "main_agent_ruling")
# 第三节点名的十五行（两张坏实现表的整表编号）
NAMED_ROWS = (15, 23, 42, 49, 110, 125, 155, 161, 162, 174, 242, 252, 269, 270, 271)
ROW_TABLES = ("bad-implementations-117.tsv", "bad-implementations-expansion-164.tsv")
OTHER_FILES = ("crates/singlefs-core/src/allocator.rs", "crates/singlefs-harness/tests/common/mod.rs")
STREAMS = ("new_pool_file_creation", "fixed_script_through_e_then_unmount", "first_file_unlinked_then_inodes_across_one_leaf")
# 甲类（第 8.2 节「定义钉死，不许变」）：草稿副本 stream-constants 产物里这几条 name=s1 行都要 holds=true
CLASS_A_CHECKS = (("third_stream_split_publish", "Some(234)"), ("inode_leaf_records", "233"),
                  ("new_pool_file_creation_operations_equal_build_pool", None),
                  ("equivalence_pairing_in_flight_writes", "6"), ("equivalence_pairing_mirrored_pairs", "3")) \
    + tuple((f"{stream}_root_writes", None) for stream in STREAMS) \
    + tuple((f"{stream}_allocator_carries_the_root_ring_of_make_filesystem", None) for stream in STREAMS)
# 甲类里装置源码上的两项（装置第 256–257 行）：Q5 那一形的已确认调用与在途调用
CLASS_A_DEVICE_LINES = ("const CONFIRMED_LOSS_CONFIRMED_CALL: usize = 2;", "const CONFIRMED_LOSS_IN_FLIGHT_CALL: usize = 3;")
P1F_ROW_NAME = "P1-F"
P1F_STREAM = "new_pool_file_creation"
# A12 十二项：照搬第一段新树那份 frozen_tree_lines.py（草稿 e171-r1-segment1-rev6/，sha256 b153c801…）的 GREPS，下面一段由 sed 原样摘出，一字不改
# (名字, 文件, 子串, 期望命中数的判法)：at_least_one 是 ≥ 1，zero 是恰 0
GREPS = [
    ('answer_unmount_defined_in_the_model', 'crates/singlefs-harness/src/model.rs', 'pub fn answer_unmount(', 'at_least_one'),
    ('observed_unmount_defined_in_the_glue', 'crates/singlefs-harness/src/model_comparison.rs', 'pub fn observed_unmount(', 'at_least_one'),
    ('second_crop_entry_with_read_faults', 'crates/singlefs-harness/src/mount_transparency.rs', 'pub fn judge_writable_mount_refusal_of_a_crash_state(', 'at_least_one'),
    ('second_crop_read_faults_parameter', 'crates/singlefs-harness/src/mount_transparency.rs', 'read_faults: ReadFaultsInTheCrashState,', 'at_least_one'),
    ('o2_head_identifiers_of_the_tree_table_entries', 'crates/singlefs-checker/src/image.rs', 'HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES: &str = "I-9.27"', 'at_least_one'),
    ('o2_first_data_unit_of_the_mkfs_watermark_version', 'crates/singlefs-checker/src/image.rs', 'FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK: &str = "I-5.5"', 'at_least_one'),
    # 修订 2 第 2.2 节：25-1 之后胶水不再断言「只罩一片叶容器」（那件交回写的预期是 0）
    ('one_leaf_container_assertion_message', 'crates/singlefs-harness/src/model_comparison.rs', '模型今天只罩一片叶容器的那几种发布', 'zero'),
    ('glue_maps_every_inode_leaf_container', 'crates/singlefs-harness/src/model_comparison.rs', 'ModelUnitRole::inode_leaf_container_at(', 'at_least_one'),
    # 修订 2 第 2.3 节：RefFS 是 checker 档包的正式依赖，装置不再 #[path] 编它
    ('reference_filesystem_dependency_of_the_checker_tier', 'crates/singlefs-checker-tier/Cargo.toml', 'singlefs-reference-filesystem = { path = "../singlefs-reference-filesystem" }', 'at_least_one'),
    ('device_uses_the_reference_filesystem_crate', 'crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs', 'use singlefs_reference_filesystem as reference_filesystem;', 'at_least_one'),
    ('device_has_no_path_copy_of_the_reference_filesystem', 'crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs', 'singlefs-reference-filesystem/src/lib.rs"]', 'zero'),
    # 主 agent 认定第 6 条：「没有根写可判」记 q5_not_run=no_root_write
    ('device_records_no_root_write_as_q5_not_run', 'crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs', 'q5_not_run=no_root_write', 'at_least_one'),
]


class RunConditionError(Exception):
    """配置读不到，或点名的树、文件不在：退 78。"""


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def sha256_of_file(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def sha256_or_missing(path: str) -> str:
    return sha256_of_file(path) if os.path.isfile(path) else "missing"


def load_config(path: str) -> dict:
    try:
        with open(path, encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        raise RunConditionError(f"占位配置读不到：{path}（{error}）")


def resolve(repo: str, path: str) -> str:
    return path if os.path.isabs(path) else os.path.join(repo, path)


def tree_of(config: dict) -> str:
    return config.get("tree") or FINAL_MAINLINE_TREE


def snapshot_table(root: str) -> dict[str, str]:
    """S16 ① 的逐文件表：Cargo.toml、Cargo.lock、.cargo、crates 下（不进任何 target 目录）快照摘要罩的 .rs 与 Cargo.*，加 .cargo/ 下全部与 crates/mutations.tsv。"""
    table: dict[str, str] = {}
    for entry in SNAPSHOT_ENTRIES:
        top = os.path.join(root, entry)
        if os.path.isfile(top):
            table[entry] = sha256_of_file(top)
            continue
        for directory, subdirectories, files in os.walk(top):
            if not broken("s16-1-counts-target"):
                subdirectories[:] = [name for name in subdirectories if name != "target"]
            for name in files:
                path = os.path.join(directory, name)
                relative = os.path.relpath(path, root)
                if name.endswith(".rs") or name.startswith("Cargo.") or relative.startswith(".cargo/") or relative == MUTATIONS_TABLE:
                    table[relative] = sha256_of_file(path)
    return table


def table_differences(left: dict[str, str], right: dict[str, str]) -> list[tuple[str, str, str]]:
    """(路径, 左边 sha256 或 -, 右边 sha256 或 -)，只列不同的，按路径排。"""
    differing = []
    for path in sorted(set(left) | set(right)):
        same = (path in left and path in right) if broken("s16-1-paths-only") else left.get(path) == right.get(path)
        if not same:
            differing.append((path, left.get(path, "-"), right.get(path, "-")))
    return differing


def apply_patch(source: str, patch_file: str, output: str, reverse: bool = False) -> bool:
    fuzz = "--fuzz=2" if broken("s16-3-fuzz") else "--fuzz=0"
    command = ["patch", fuzz, "--no-backup-if-mismatch", "-s", "-o", output] + (["-R"] if reverse else []) + [source, patch_file]
    return subprocess.run(command, capture_output=True, text=True).returncode == 0


def apply_blocks_in_order(base: str, fixtures: list[str], work: str) -> tuple[str | None, int]:
    """base 依次无模糊打上各块；交回 (结果的 sha256，打不上的那一块的下标；都打上了是 -1)。"""
    current = os.path.join(work, "applied-0.rs")
    shutil.copyfile(base, current)
    for index, fixture in enumerate(fixtures):
        following = os.path.join(work, f"applied-{index + 1}.rs")
        if not apply_patch(current, fixture, following):
            return None, index
        current = following
    return sha256_of_file(current), -1


def remove_known_differences(repo_device: str, fixtures: list[str], work: str) -> tuple[str | None, int]:
    """仓里那份依次（后认的先撤）无模糊撤掉认过的各块；交回 (撤完的 sha256，撤不掉的那一块的下标；都撤掉了是 -1)。"""
    current = os.path.join(work, "removed-0.rs")
    shutil.copyfile(repo_device, current)
    if broken("s16-2-no-known-differences"):
        return sha256_of_file(current), -1
    for step, index in enumerate(reversed(range(len(fixtures)))):
        following = os.path.join(work, f"removed-{step + 1}.rs")
        if not apply_patch(current, fixtures[index], following, reverse=True):
            return None, index
        current = following
    return sha256_of_file(current), -1


def fixture_header_problems(path: str) -> list[str]:
    """块的 fixture 用 diff -u --label 出：头两行是「--- <名>」「+++ <名>」，不带制表符后的修改时刻（sha256 才可复现）。"""
    with open(path, encoding="utf-8") as handle:
        head = [handle.readline().rstrip("\n") for _ in range(2)]
    problems = []
    for line, mark in zip(head, ("--- ", "+++ ")):
        if not line.startswith(mark):
            problems.append(f"头一行不是「{mark.strip()} 」起头")
        elif "\t" in line and not broken("fixture-timestamps-allowed"):
            problems.append(f"「{mark.strip()}」那一行带修改时刻（要 diff -u --label）")
    return problems


def unified_diff_with_labels(left: str, right: str, left_label: str, right_label: str) -> str:
    """出一块 fixture 的正文（第 8.1 节：diff -u --label <旧名> --label <新名>）。"""
    return subprocess.run(["diff", "-u", "--label", left_label, "--label", right_label, left, right], capture_output=True, text=True).stdout


def fixed_device_sha256s() -> tuple[str, str]:
    """(ab35105f… 那份, 修订 6 树那份) 的期望 sha256。自证里换成假树的（FROZEN_TREE_R8_SELFTEST=1 时才认两个替换变量），平常一律取登记写死的。"""
    if os.environ.get(SELFTEST_VARIABLE) == "1":
        return (os.environ.get("FROZEN_TREE_R8_EXPECTED_BASE_SHA256", BASE_DEVICE_SHA256),
                os.environ.get("FROZEN_TREE_R8_EXPECTED_REVISION_6_DEVICE_SHA256", REVISION_6_TREE_DEVICE_SHA256))
    return BASE_DEVICE_SHA256, REVISION_6_TREE_DEVICE_SHA256


def section(config: dict, key: str) -> dict:
    value = config.get(key)
    return value if isinstance(value, dict) else {}


def key_values(line: str) -> dict[str, str]:
    return dict(token.split("=", 1) for token in line.split() if "=" in token)


def class_a_problems(product_lines: list[str], device_text: str) -> list[str]:
    """甲类没过的项名：草稿副本 stream-constants 产物里那一条 name=s1 行不在、不是 holds=true，或 local 不是登记写死的值；装置源码少了那两行常量。"""
    s1 = {}
    for line in product_lines:
        if " name=s1 " in f" {line} " or line.startswith("E7RESULT name=s1 "):
            fields = key_values(line)
            if "check" in fields:
                s1.setdefault(fields["check"], []).append(fields)
    problems = []
    for check, expected_local in CLASS_A_CHECKS:
        found = s1.get(check, [])
        if not found:
            if not broken("class-a-missing-passes"):
                problems.append(f"{check}:missing")
            continue
        if any(fields.get("holds") != "true" for fields in found):
            problems.append(f"{check}:holds_false")
        if expected_local is not None and any(fields.get("local") != expected_local for fields in found):
            problems.append(f"{check}:local_not_{expected_local}")
    problems += [f"device_line_missing:{line.split(':')[0].split()[-1]}" for line in CLASS_A_DEVICE_LINES if line not in device_text]
    return problems


def announcement_lines(records_text: str) -> list[tuple[int, str]]:
    found = [(number, line) for number, line in enumerate(records_text.splitlines(), start=1) if line.startswith(ANNOUNCEMENT_PREFIX)]
    return found[:1] if broken("announcement-count-ignored") else found


def last_sha256_by_path(after_files: list[str], device_swap_after: str | None) -> dict[str, str]:
    """照合入次序读各份 after.sha256（「sha256  路径」），同一个文件后合的算；装置文件取换常量那一行的换后值。"""
    chosen: dict[str, str] = {}
    for path in after_files:
        with open(path, encoding="utf-8") as handle:
            for line in handle:
                parts = line.rstrip("\n").split("  ", 1)
                if len(parts) == 2 and len(parts[0]) == 64:
                    if broken("after-sha-first-wins") and parts[1] in chosen:
                        continue
                    chosen[parts[1]] = parts[0]
    if device_swap_after:
        chosen[DEVICE] = device_swap_after
    return chosen


def process_arguments() -> list[str]:
    command = os.environ.get("FROZEN_TREE_R8_PS_COMMAND") if os.environ.get(SELFTEST_VARIABLE) == "1" else None
    completed = subprocess.run(["bash", "-c", command] if command else ["ps", "-o", "pid=,args=", "-u", str(os.getuid())],
                               capture_output=True, text=True)
    return completed.stdout.splitlines()


def merge_verification_processes(lines: list[str]) -> list[str]:
    if broken("merge-process-ignored"):
        return []
    return [line.strip() for line in lines if MERGE_VERIFY_SCRIPT in line and "frozen_tree_lines_revision_8" not in line]


def a12_results(tree: str) -> list[tuple[str, str, int, str, bool]]:
    """(名字, 文件, 命中数, 期望, 成立否)：照搬的十二项在这棵树上现判。"""
    results = []
    for name, relative, needle, expectation in GREPS:
        path = os.path.join(tree, relative)
        hits = open(path, encoding="utf-8").read().count(needle) if os.path.isfile(path) else 0
        at_least_one = expectation == "at_least_one" or broken("a12-zero-as-at-least-one")
        results.append((name, relative, hits, expectation, hits >= 1 if at_least_one else hits == 0))
    return results


def named_row_anchors(tree: str, research_directory: str = HERE) -> list[tuple[int, str, str, int]]:
    """第三节十五行：按名字在树里 crates/mutations.tsv 取得到否（bad_implementation_table.locate_by_name）、锚点在树里那份源文件命中几次。
    交回 (行号, 变异名, 状态, 命中数)；取不到的命中数记 -1。"""
    selected = {}
    for table in ROW_TABLES:
        for row in bad_implementation_table.read_selected_rows(os.path.join(research_directory, table)):
            selected[row.number] = row
    mutation_rows = bad_implementation_table.read_mutation_rows(os.path.join(tree, MUTATIONS_TABLE))
    results = []
    for number in NAMED_ROWS:
        row = selected[number]
        if broken("anchors-count-substring"):
            hits_rows = [candidate for candidate in mutation_rows if row.name in candidate.name]
            located = bad_implementation_table.Located("ok" if hits_rows else "not_found", hits_rows[0] if hits_rows else None, False)
        else:
            located = bad_implementation_table.locate_by_name(row, mutation_rows)
        hits = -1
        if located.row is not None:
            source = os.path.join(tree, located.row.file)
            hits = bad_implementation_table.anchor_hit_count(open(source, encoding="utf-8").read(), located.row) if os.path.isfile(source) else 0
        results.append((number, row.name, located.status, hits))
    return results


def research_side_rows(repo: str, reference: dict[str, str]) -> list[tuple[str, str, str]]:
    """H11：研究一侧目录全部文件（不含 __pycache__）与参照表点名的别处文件，(相对仓根的路径, 现在的 sha256, 修订 6 那一趟用的或 unrecorded)。"""
    research_relative = os.path.relpath(HERE, repo) if os.path.isabs(HERE) else HERE
    paths = set(reference)
    for directory, subdirectories, files in os.walk(os.path.join(repo, research_relative)):
        subdirectories[:] = [name for name in subdirectories if name != "__pycache__"]
        for name in files:
            paths.add(os.path.relpath(os.path.join(directory, name), repo))
    return [(path, sha256_or_missing(os.path.join(repo, path)), reference.get(path, "unrecorded")) for path in sorted(paths)]


def p1f_base_digest(directory: str) -> tuple[int, str]:
    """E170 坏状态目录（流一）里 P1-F：bad-expected.tsv 第 2 列是行名、第 1 列是行号；bad-states.tsv 那一行第 3 列是底态摘要。"""
    stream_directory = os.path.join(directory, P1F_STREAM)
    names = {}
    for line in open(os.path.join(stream_directory, "bad-expected.tsv"), encoding="utf-8"):
        columns = line.rstrip("\n").split("\t")
        if len(columns) >= 2 and not line.startswith("#"):
            names[columns[1]] = columns[0]
    row_id = names[P1F_ROW_NAME]
    for line in open(os.path.join(stream_directory, "bad-states.tsv"), encoding="utf-8"):
        columns = line.rstrip("\n").split("\t")
        if columns[0] == row_id and len(columns) >= 3:
            return int(row_id), columns[2].lower()
    raise ValueError(f"bad-states.tsv 里没有 {P1F_ROW_NAME} 那一行（号 {row_id}）")


def git_lines(repo: str) -> tuple[str, list[str]]:
    head = subprocess.run(["git", "-C", repo, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip() or "unreadable"
    status = subprocess.run(["git", "-C", repo, "-c", "core.quotepath=false", "status", "--porcelain", "--", "crates", "Cargo.toml", "Cargo.lock", ".cargo"],
                            capture_output=True, text=True).stdout.splitlines()
    return head, status


def config_line(lines: ResultLines, product: str, config_path: str, config: dict, repo: str) -> None:
    lines.result(f"name=config segment=s1 product={product} revision=8 tree={field(tree_of(config))} config={field(os.path.relpath(config_path, repo))} "
                 f"config_sha256={sha256_of_file(config_path)} dry_run={str(bool(config.get('dry_run'))).lower()}")


def before_snapshot(repo: str, config_path: str, output: str | None) -> int:
    """S16 ④：第 8.1 节第 2 条 a–e，一项一行。"""
    config = load_config(config_path)
    h1, h6 = section(config, "H1"), section(config, "H6")
    lines = ResultLines(output)
    config_line(lines, "before_snapshot", config_path, config, repo)
    checks: list[tuple[str, bool, str]] = []
    records_path = resolve(repo, h1.get("records_file") or RECORDS_FILE)
    records_text = open(records_path, encoding="utf-8").read() if os.path.isfile(records_path) else ""
    found = announcement_lines(records_text)
    line_matches = len(found) == 1 and found[0][0] == h1.get("line_number") and found[0][1] == h1.get("line_text")
    checks.append(("a", len(found) == 1 and line_matches, f"announcement_lines={len(found)} line_number={found[0][0] if found else '-'} "
                   f"equals_H1={str(line_matches).lower()}"))
    after_files = [resolve(repo, path) for path in h1.get("after_sha256_files_in_merge_order") or []]
    missing = [path for path in after_files if not os.path.isfile(path)]
    text_of_line = found[0][1] if found else ""
    unnamed = [path for path in h1.get("after_sha256_files_in_merge_order") or []
               if path not in text_of_line and path.replace(os.path.expanduser("~"), "~") not in text_of_line]
    swap = h1.get("device_swap") or {}
    mismatching: list[str] = []
    if after_files and not missing:
        for relative, expected in sorted(last_sha256_by_path(after_files, swap.get("after_sha256")).items()):
            if sha256_or_missing(os.path.join(repo, relative)) != expected:
                mismatching.append(relative)
    checks.append(("b", bool(after_files) and not missing and not unnamed and not mismatching,
                   f"after_files={len(after_files)} missing={len(missing)} not_named_in_the_announcement={len(unnamed)} "
                   f"files_differing_from_the_last_merge={field(','.join(mismatching) or 'none')}"))
    running = merge_verification_processes(process_arguments())
    checks.append(("c", not running, f"merge_verifications_running={len(running)}"))
    draft = resolve(repo, h6.get("draft_copy_root") or "") if h6.get("draft_copy_root") else ""
    binaries = [os.path.join(draft, "target/release", name) for name in (E171_BINARY, E170_BINARY)] if draft else []
    device_in_draft = os.path.join(draft, DEVICE) if draft else ""
    built = bool(binaries) and all(os.path.isfile(path) and os.access(path, os.X_OK) for path in binaries) and os.path.isfile(device_in_draft) \
        and all(os.path.getmtime(path) >= os.path.getmtime(device_in_draft) for path in binaries)
    checks.append(("d", built, f"draft_copy={field(h6.get('draft_copy_root') or '-')} both_binaries_built_after_the_device_source={str(built).lower()}"))
    product_path = resolve(repo, h6.get("draft_copy_stream_constants_product") or "") if h6.get("draft_copy_stream_constants_product") else ""
    product_lines = open(product_path, encoding="utf-8").read().splitlines() if product_path and os.path.isfile(product_path) else []
    device_text = open(device_in_draft, encoding="utf-8").read() if device_in_draft and os.path.isfile(device_in_draft) else ""
    problems = class_a_problems(product_lines, device_text) if product_lines else ["draft_product_missing"]
    revision_9 = REVISION_9_HEADING in open(os.path.join(repo, PREREGISTRATION), encoding="utf-8").read()
    repo_device = sha256_or_missing(os.path.join(repo, DEVICE))
    device_written = repo_device == h6.get("device_sha256_after_revision_9")
    checks.append(("e", not problems and revision_9 and device_written and bool(h6.get("constants_written_in_revision_9")),
                   f"class_a_problems={field(','.join(problems) or 'none')} revision_9_heading={str(revision_9).lower()} "
                   f"repo_device={repo_device} equals_revision_9={str(device_written).lower()}"))
    for name, holds, detail in checks:
        lines.result(f"name=before_snapshot check={name} holds={str(holds).lower()} {detail}")
    failed = [name for name, holds, _ in checks if not holds]
    lines.result(f"name=verdict s16_4_failed_checks={len(failed)} failed={','.join(failed) or 'none'} checks={len(checks)} "
                 f"triggered={'s16_4' if failed else 'none'}")
    lines.finish()
    return 1 if failed else 0


def at_snapshot(repo: str, config_path: str, output: str | None) -> int:
    """拷完树、编之前：S16 ①、H2、编前快照摘要、H4、H8、H9。"""
    config = load_config(config_path)
    tree = resolve(repo, tree_of(config))
    if not os.path.isdir(tree):
        raise RunConditionError(f"树不在：{tree}")
    lines = ResultLines(output)
    config_line(lines, "snapshot", config_path, config, repo)
    head, status = git_lines(repo)
    lines.result(f"name=snapshot key=git_head value={head}")
    lines.result(f"name=snapshot key=git_status_lines value={len(status)}")
    for line in status:
        lines.result(f"name=snapshot key=git_status line={field(line)}")
    digest, count = row_driver.snapshot_digest(tree)
    lines.result(f"name=snapshot key=snapshot_digest_before_build value={digest} files={count}")
    tree_table, repo_table = snapshot_table(tree), snapshot_table(repo)
    differing = table_differences(tree_table, repo_table)
    for path, tree_sha, repo_sha in differing:
        lines.result(f"name=frozen_tree key=s16_1 file={field(path)} tree_sha256={tree_sha} repo_sha256={repo_sha}")
    triggered = bool(differing) or len(tree_table) != len(repo_table)
    lines.result(f"name=frozen_tree key=s16_1_summary differing_files={len(differing)} tree_files={len(tree_table)} repo_files={len(repo_table)} "
                 f"triggered={str(triggered).lower()}")
    lines.result(f"name=snapshot key=device_sha256 tree={sha256_or_missing(os.path.join(tree, DEVICE))} repo={sha256_or_missing(os.path.join(repo, DEVICE))}")
    lines.result(f"name=snapshot key=mutations_table_sha256 tree={sha256_or_missing(os.path.join(tree, MUTATIONS_TABLE))} "
                 f"repo={sha256_or_missing(os.path.join(repo, MUTATIONS_TABLE))}")
    anchors = named_row_anchors(tree)
    for number, name, status_text, hits in anchors:
        lines.result(f"name=snapshot key=named_row row={number} status={status_text} anchor_hits={hits} name_of_the_row={field(name)}")
    lines.result(f"name=verdict s16_1_triggered={str(triggered).lower()} named_rows_not_hit_once={sum(1 for *_, hits in anchors if hits != 1)}")
    lines.finish()
    return 1 if triggered else 0


def product_line_value(path: str, prefix: str, key: str) -> str | None:
    for line in open(path, encoding="utf-8"):
        if line.startswith(prefix):
            return key_values(line).get(key)
    return None


def after_build(repo: str, config_path: str, output: str | None) -> int:
    config = load_config(config_path)
    tree = resolve(repo, tree_of(config))
    snapshot_product = resolve(repo, section(config, "H3").get("snapshot_product") or "")
    if not os.path.isfile(snapshot_product):
        raise RunConditionError(f"at-snapshot 那份产物不在（配置 H3.snapshot_product）：{snapshot_product}")
    before = product_line_value(snapshot_product, "E7RESULT name=snapshot key=snapshot_digest_before_build ", "value")
    digest, count = row_driver.snapshot_digest(tree)
    lines = ResultLines(output)
    config_line(lines, "snapshot_build", config_path, config, repo)
    lines.result(f"name=snapshot key=snapshot_digest_after_build value={digest} files={count} before={before}")
    for name in (E171_BINARY, E170_BINARY):
        lines.result(f"name=snapshot key=binary_sha256 binary={name} sha256={sha256_or_missing(os.path.join(tree, 'target/release', name))}")
    same = before == digest
    lines.result(f"name=verdict digest_before_equals_after={str(same).lower()}")
    lines.finish()
    return 0 if same else 1


def s16_2(repo: str, config: dict) -> tuple[bool, str]:
    """S16 ②：仓里装置等于树里那份；不等时，认过的已知差额（H5.known_repo_differences）从仓里那份依次无模糊撤得掉、撤完等于树里那份。"""
    tree = resolve(repo, tree_of(config))
    tree_device, repo_device = os.path.join(tree, DEVICE), os.path.join(repo, DEVICE)
    tree_sha, repo_sha = sha256_or_missing(tree_device), sha256_or_missing(repo_device)
    if tree_sha == repo_sha and tree_sha != "missing":
        return True, f"device_repo_equals_the_tree=true repo={repo_sha} tree={tree_sha}"
    known = section(config, "H5").get("known_repo_differences") or []
    fixtures = [resolve(repo, entry.get("fixture", "")) for entry in known]
    if not known or "missing" in (tree_sha, repo_sha) or not all(os.path.isfile(path) for path in fixtures):
        return False, f"device_repo_equals_the_tree=false repo={repo_sha} tree={tree_sha} known_differences={len(known)} removed=false"
    with tempfile.TemporaryDirectory(prefix="e171-r8-s16-2-") as work:
        removed, failed_index = remove_known_differences(repo_device, fixtures, work)
    holds = removed == tree_sha
    return holds, (f"device_repo_equals_the_tree=false repo={repo_sha} tree={tree_sha} known_differences={len(known)} "
                   f"removed={str(failed_index == -1).lower()} result_equals_the_tree={str(holds).lower()}")


def s16_3(repo: str, config: dict, target_sha: str) -> tuple[bool, list[str], str]:
    """S16 ③：ab35105f… 那份依次无模糊打上块表各块，结果等于 target_sha（树里装置）；每块的来源是那三种之一。交回 (成立否, 各块一行, 汇总)。"""
    h5 = section(config, "H5")
    base = resolve(repo, h5.get("base_device_copy") or "")
    blocks = h5.get("blocks") or []
    rows, no_source = [], 0
    for index, block in enumerate(blocks, start=1):
        fixture = resolve(repo, block.get("fixture", ""))
        kind = block.get("source_kind")
        valid = (kind in BLOCK_SOURCE_KINDS and bool(block.get("source"))) or broken("s16-3-any-source")
        no_source += not valid
        actual = sha256_or_missing(fixture)
        header = fixture_header_problems(fixture) if os.path.isfile(fixture) else ["missing"]
        rows.append(f"name=frozen_tree key=s16_3_block index={index} block={field(block.get('name', '-'))} source={kind if valid else 'none'} "
                    f"fixture_sha256={actual} expected={block.get('fixture_sha256', '-')} sha256_holds={str(actual == block.get('fixture_sha256')).lower()} "
                    f"header_problems={field(','.join(header) or 'none')}")
    base_sha = sha256_or_missing(base)
    fixtures = [resolve(repo, block.get("fixture", "")) for block in blocks]
    applied, failed_index = (None, -2)
    base_expected, _ = fixed_device_sha256s()
    if base_sha == base_expected and all(os.path.isfile(path) for path in fixtures):
        with tempfile.TemporaryDirectory(prefix="e171-r8-s16-3-") as work:
            applied, failed_index = apply_blocks_in_order(base, fixtures, work)
    fixture_problems = sum(1 for row in rows if "sha256_holds=false" in row or "header_problems=none" not in row)
    identical = applied == target_sha
    holds = identical and no_source == 0 and fixture_problems == 0
    summary = (f"name=frozen_tree key=s16_3 base_sha256={base_sha} base_expected={base_expected} blocks={len(blocks)} "
               f"first_block_not_applying={failed_index + 1 if failed_index >= 0 else 'none'} result_sha256={applied or 'none'} target_sha256={target_sha} "
               f"identical={str(identical).lower()} blocks_without_source={no_source} fixture_problems={fixture_problems}")
    return holds, rows, summary


def snapshot_lines_to_copy(path: str) -> list[str]:
    """at-snapshot 那份产物里 S16 ① 的几行（key=s16_1、key=s16_1_summary），原样抄进第 0 步产物（去掉 E7RESULT 前缀，由 ResultLines 补回）。"""
    return [line.rstrip("\n")[len("E7RESULT "):] for line in open(path, encoding="utf-8")
            if line.startswith("E7RESULT name=frozen_tree key=s16_1")]


def step_0(repo: str, config_path: str, output: str | None) -> int:
    config = load_config(config_path)
    tree = resolve(repo, tree_of(config))
    if not os.path.isdir(tree):
        raise RunConditionError(f"树不在：{tree}")
    h3, h4, h7, h8 = section(config, "H3"), section(config, "H4"), section(config, "H7"), section(config, "H8")
    lines = ResultLines(output)
    config_line(lines, "frozen_tree", config_path, config, repo)
    triggered: list[str] = []
    snapshot_product = resolve(repo, h3.get("snapshot_product") or "")
    snapshot_sha = sha256_or_missing(snapshot_product)
    lines.result(f"name=frozen_tree key=snapshot_product file={field(h3.get('snapshot_product') or '-')} sha256={snapshot_sha} "
                 f"expected={h3.get('snapshot_product_sha256', '-')} holds={str(snapshot_sha == h3.get('snapshot_product_sha256')).lower()}")
    copied = snapshot_lines_to_copy(snapshot_product) if snapshot_sha == h3.get("snapshot_product_sha256") else []
    for body in copied:
        lines.result(body)
    if not copied or any("triggered=true" in body for body in copied):
        triggered.append("s16_1")
    digest, count = row_driver.snapshot_digest(tree)
    z5 = digest == h3.get("snapshot_digest") or broken("step-0-ignores-z5")
    lines.result(f"name=frozen_tree key=snapshot_digest value={digest} files={count} expected={h3.get('snapshot_digest', '-')} holds={str(z5).lower()}")
    if not z5:
        triggered.append("z5")
    tree_device = sha256_or_missing(os.path.join(tree, DEVICE))
    base = resolve(repo, section(config, "H5").get("base_device_copy") or "")
    revision_6_device = sha256_or_missing(os.path.join(repo, REVISION_6_TREE, DEVICE))
    base_expected, revision_6_expected = fixed_device_sha256s()
    for place, actual, expected in (("tree", tree_device, h4.get("device_sha256_in_the_tree")), ("base_ab35105f", sha256_or_missing(base), base_expected),
                                    ("revision_6_tree", revision_6_device, revision_6_expected)):
        lines.result(f"name=frozen_tree key=device_sha256 place={place} sha256={actual} expected={expected or '-'} holds={str(actual == expected).lower()}")
        if actual != expected:
            triggered.append(f"device_{place}")
    holds_2, detail_2 = s16_2(repo, config)
    lines.result(f"name=frozen_tree key=s16_2 holds={str(holds_2).lower()} {detail_2}")
    if not holds_2:
        triggered.append("s16_2")
    holds_3, block_rows, summary_3 = s16_3(repo, config, tree_device)
    for body in block_rows + [summary_3]:
        lines.result(body)
    if not holds_3:
        triggered.append("s16_3")
    head, status = git_lines(repo)
    lines.result(f"name=frozen_tree key=git_head value={head} recorded_at_the_snapshot={section(config, 'H2').get('git_head', '-')}")
    lines.result(f"name=frozen_tree key=git_status_lines value={len(status)}")
    for line in status:
        lines.result(f"name=frozen_tree key=git_status line={field(line)}")
    for relative in OTHER_FILES:
        lines.result(f"name=frozen_tree key=file_sha256 file={relative} repo={sha256_or_missing(os.path.join(repo, relative))} "
                     f"tree={sha256_or_missing(os.path.join(tree, relative))}")
    for name, key in ((E171_BINARY, "e171_binary_sha256"), (E170_BINARY, "e170_binary_sha256")):
        actual = sha256_or_missing(os.path.join(tree, "target/release", name))
        lines.result(f"name=frozen_tree key=binary_sha256 binary={name} sha256={actual} expected={h7.get(key, '-')} holds={str(actual == h7.get(key)).lower()}")
        if actual != h7.get(key):
            triggered.append("h7")
    a12 = a12_results(tree)
    for name, relative, hits, expectation, holds in a12:
        lines.result(f"name=frozen_tree key=a12 check={name} file={relative} hits={hits} expected={expectation} holds={str(holds).lower()}")
    a12_failed = sum(1 for *_, holds in a12 if not holds)
    if a12_failed:
        triggered.append("a12")
    mutations = sha256_or_missing(os.path.join(tree, MUTATIONS_TABLE))
    lines.result(f"name=frozen_tree key=mutations_table_sha256 value={mutations} expected={h8.get('mutations_table_sha256', '-')} "
                 f"holds={str(mutations == h8.get('mutations_table_sha256')).lower()}")
    if mutations != h8.get("mutations_table_sha256"):
        triggered.append("z5_mutations_table")
    for number, name, status_text, hits in named_row_anchors(tree):
        lines.result(f"name=frozen_tree key=named_row row={number} status={status_text} anchor_hits={hits} name_of_the_row={field(name)}")
    reference_path = resolve(repo, section(config, "H11").get("revision_6_reference") or "")
    reference = json.load(open(reference_path, encoding="utf-8")) if os.path.isfile(reference_path) else {}
    research = research_side_rows(repo, reference.get("files", {}))
    for path, now, before in research:
        lines.result(f"name=frozen_tree key=research_side file={field(path)} sha256={now} revision_6_run={before} same={str(now == before).lower()}")
    differs = [path for path, now, before in research if now != before and path in reference.get("judged", [])]
    revision_6_tree = os.path.join(repo, REVISION_6_TREE)
    if os.path.isdir(revision_6_tree):
        for path, old_sha, new_sha in table_differences(snapshot_table(revision_6_tree), snapshot_table(tree)):
            lines.result(f"name=frozen_tree key=file_difference_against_the_revision_6_tree file={field(path)} revision_6_tree={old_sha} revision_8_tree={new_sha}")
    lines.result(f"name=verdict a12_failed_checks={a12_failed} a12_checks={len(a12)} s16_2_holds={str(holds_2).lower()} s16_3_holds={str(holds_3).lower()} "
                 f"research_side_differs={str(bool(differs) or not reference).lower()} research_side_files_differing={len(differs)} "
                 f"triggered={','.join(sorted(set(triggered))) or 'none'}")
    lines.finish()
    return 1 if triggered else 0


def prerequisite_results(repo: str, config: dict) -> list[tuple[str, bool, str]]:
    """每次起跑与续跑都核的四样（第 8.4 节「续跑」末条，O3 冻结行另核）：(项, 成立否, 细节)。"""
    tree = resolve(repo, tree_of(config))
    h3, h7, h8 = section(config, "H3"), section(config, "H7"), section(config, "H8")
    results = []
    digest, count = row_driver.snapshot_digest(tree) if os.path.isdir(tree) else ("missing", 0)
    results.append(("z5_snapshot_digest", digest == h3.get("snapshot_digest"), f"value={digest} files={count} expected={h3.get('snapshot_digest', '-')}"))
    for name, key in ((E171_BINARY, "e171_binary_sha256"), (E170_BINARY, "e170_binary_sha256")):
        actual = sha256_or_missing(os.path.join(tree, "target/release", name))
        results.append((f"h7_{name}", actual == h7.get(key), f"sha256={actual} expected={h7.get(key, '-')}"))
    holds, detail = s16_2(repo, config)
    results.append(("s16_2", holds, detail))
    mutations = sha256_or_missing(os.path.join(tree, MUTATIONS_TABLE))
    results.append(("h8_mutations_table", mutations == h8.get("mutations_table_sha256"), f"sha256={mutations} expected={h8.get('mutations_table_sha256', '-')}"))
    return results


def prerequisites(repo: str, config_path: str) -> int:
    results = prerequisite_results(repo, load_config(config_path))
    for name, holds, detail in results:
        print(f"name=prerequisite check={name} holds={str(holds).lower()} {detail}")
    failed = [name for name, holds, _ in results if not holds]
    print(f"name=prerequisites_verdict failed={','.join(failed) or 'none'} checks={len(results)}")
    return 1 if failed else 0


def p1f_command(directory: str, output: str | None) -> int:
    row_id, digest = p1f_base_digest(directory)
    lines = ResultLines(output)
    lines.result(f"name=config segment=s1 product=e170_second_run_p1_f directory={field(directory)}")
    lines.result(f"name=e170_second_run_p1_f row={P1F_ROW_NAME} row_id={row_id} base_digest={digest} stream={P1F_STREAM}")
    lines.finish()
    return 0


def main(argument_list: list[str]) -> int:
    refusal = harness.refuse_break_outside_selftest(BREAK_VARIABLE, SELFTEST_VARIABLE)
    if refusal:
        print(refusal, file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 修订 8 那一趟：第 0 步判法与拍树前后几核")
    parser.add_argument("mode", choices=("before-snapshot", "at-snapshot", "after-build", "step-0", "prerequisites", "p1f-base-digest"))
    parser.add_argument("--config")
    parser.add_argument("--output")
    parser.add_argument("--directory")
    parser.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    repo = os.path.abspath(arguments.repo)
    output = None if arguments.output == "-" else arguments.output
    if arguments.mode == "p1f-base-digest":
        if not arguments.directory or not arguments.output:
            parser.error("p1f-base-digest 要 --directory 与 --output")
        return p1f_command(arguments.directory, output)
    if not arguments.config or (arguments.mode != "prerequisites" and not arguments.output):
        parser.error(f"{arguments.mode} 要 --config" + ("" if arguments.mode == "prerequisites" else " 与 --output"))
    if output is not None and os.path.exists(output):
        print(f"产物已存在，不覆盖：{output}", file=sys.stderr)
        return 2
    commands = {"before-snapshot": before_snapshot, "at-snapshot": at_snapshot, "after-build": after_build, "step-0": step_0}
    try:
        if arguments.mode == "prerequisites":
            return prerequisites(repo, arguments.config)
        return commands[arguments.mode](repo, arguments.config, output)
    except RunConditionError as error:
        print(f"✗ {error}（退 78）", file=sys.stderr)
        return EXIT_RUN_CONDITION


# ── 自证：假仓、假树都建在自证的临时目录里，判法函数在进程里直接调 ──
def write_text(path: str, text: str) -> str:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    return path


def fake_repository(root: str) -> str:
    """一个最小的「仓」：快照四样、装置、坏实现表要的十五行（每行锚点在假源文件里恰 1 次）、A12 十二项该有的子串、登记与调度记录。"""
    write_text(os.path.join(root, "Cargo.toml"), "[workspace]\n")
    write_text(os.path.join(root, "Cargo.lock"), "# lock\n")
    write_text(os.path.join(root, ".cargo/config.toml"), "[env]\n")
    write_text(os.path.join(root, DEVICE), "".join(f"// device line {number}\n" for number in range(40)) + "\n".join(CLASS_A_DEVICE_LINES) + "\n")
    selected = {row.number: row for table in ROW_TABLES for row in bad_implementation_table.read_selected_rows(os.path.join(HERE, table))}
    mutation_lines = ["# 名\t文件\t原文\t替换\t参数\t测试"]
    for number in NAMED_ROWS:
        source = f"crates/fake/src/row_{number}.rs"
        write_text(os.path.join(root, source), f"fn anchor_{number}() {{}}\n")
        # 名字是十五行之一的前缀串的那一行排在前面：按子串取会取到它（锚点 0 次），按名字恰等取不到它
        mutation_lines.append(f"{selected[number].name}（另一行）\t{source}\tfn not_here_{number}() {{}}\tx\t-\t-")
        mutation_lines.append(f"{selected[number].name}\t{source}\tfn anchor_{number}() {{}}\tfn anchor_{number}() {{ panic!() }}\t-\t-")
    write_text(os.path.join(root, MUTATIONS_TABLE), "\n".join(mutation_lines) + "\n")
    for name, relative, needle, expectation in GREPS:
        path = os.path.join(root, relative)
        previous = open(path, encoding="utf-8").read() if os.path.exists(path) else ""
        write_text(path, previous + (f"{needle}\n" if expectation == "at_least_one" else ""))
    write_text(os.path.join(root, PREREGISTRATION), f"# 登记\n{REVISION_9_HEADING}（假）\n")
    return root


def copy_as_tree(repository: str, tree: str) -> str:
    for entry in SNAPSHOT_ENTRIES:
        source = os.path.join(repository, entry)
        if os.path.isdir(source):
            shutil.copytree(source, os.path.join(tree, entry))
        else:
            os.makedirs(tree, exist_ok=True)
            shutil.copy2(source, os.path.join(tree, entry))
    return tree


def cell_snapshot_table_ignores_target(scratch: str) -> tuple[bool, str]:
    repository = fake_repository(os.path.join(scratch, "repo"))
    tree = copy_as_tree(repository, os.path.join(scratch, "tree"))
    write_text(os.path.join(tree, "target/release/build/out.rs"), "// 编出来的\n")
    write_text(os.path.join(tree, "crates/fake/target/x.rs"), "// crate 下的 target\n")
    differing = table_differences(snapshot_table(tree), snapshot_table(repository))
    return not differing, f"拷过去、加了两个 target 下的 .rs：差 {len(differing)} 处（要 0）"


def cell_snapshot_table_content_differs(scratch: str) -> tuple[bool, str]:
    repository = fake_repository(os.path.join(scratch, "repo"))
    tree = copy_as_tree(repository, os.path.join(scratch, "tree"))
    write_text(os.path.join(tree, "crates/fake/src/row_15.rs"), "fn anchor_15() { changed }\n")
    write_text(os.path.join(tree, ".cargo/extra.txt"), "多一个\n")
    differing = [path for path, _, _ in table_differences(snapshot_table(tree), snapshot_table(repository))]
    return differing == [".cargo/extra.txt", "crates/fake/src/row_15.rs"], f"改一处内容、.cargo 下多一个文件：差 {differing}"


def three_versions(scratch: str) -> tuple[str, str, str]:
    lines = [f"line {number}\n" for number in range(30)]
    base = write_text(os.path.join(scratch, "base.rs"), "".join(lines))
    middle = write_text(os.path.join(scratch, "middle.rs"), "".join(lines[:5] + ["inserted A\n"] + lines[5:]))
    final = write_text(os.path.join(scratch, "final.rs"), "".join(lines[:5] + ["inserted A\n"] + lines[5:20] + ["inserted B\n"] + lines[20:]))
    return base, middle, final


def cell_s16_3_blocks_apply(scratch: str) -> tuple[bool, str]:
    base, middle, final = three_versions(scratch)
    first = write_text(os.path.join(scratch, "block-1.diff"), unified_diff_with_labels(base, middle, "a", "b"))
    second = write_text(os.path.join(scratch, "block-2.diff"), unified_diff_with_labels(middle, final, "b", "c"))
    in_order, _ = apply_blocks_in_order(base, [first, second], scratch)
    # 上下文差一行的块：无模糊打不上，--fuzz=2 打得上
    shifted = write_text(os.path.join(scratch, "shifted.rs"), open(middle, encoding="utf-8").read().replace("line 17\n", "line 17 changed\n"))
    shifted_after = write_text(os.path.join(scratch, "shifted-after.rs"), open(shifted, encoding="utf-8").read().replace("line 19\n", "line 19\ninserted C\n"))
    fuzzy = write_text(os.path.join(scratch, "fuzzy.diff"), unified_diff_with_labels(shifted, shifted_after, "s", "t"))
    fuzzy_result, fuzzy_index = apply_blocks_in_order(middle, [fuzzy], scratch)
    good = in_order == sha256_of_file(final) and fuzzy_result is None and fuzzy_index == 0
    return good, f"两块依次打上等于末一版：{in_order == sha256_of_file(final)}；上下文差一行的块打不上：{fuzzy_result is None}"


def cell_s16_3_source_kind(scratch: str) -> tuple[bool, str]:
    base, middle, _ = three_versions(scratch)
    fixture = write_text(os.path.join(scratch, "block.diff"), unified_diff_with_labels(base, middle, "a", "b"))
    config = {"H5": {"base_device_copy": base, "blocks": [{"name": "x", "source_kind": "someone_said_so", "source": "口头", "fixture": fixture,
                                                             "fixture_sha256": sha256_of_file(fixture)}]}}
    os.environ["FROZEN_TREE_R8_EXPECTED_BASE_SHA256"] = sha256_of_file(base)
    holds, rows, summary = s16_3(scratch, config, sha256_of_file(middle))
    good = not holds and "source=none" in rows[0] and "blocks_without_source=1" in summary and "identical=true" in summary
    return good, f"来源不在三种里：成立 {holds}；{rows[0][:90]}…"


def cell_s16_2_known_difference_removed(scratch: str) -> tuple[bool, str]:
    repository = fake_repository(os.path.join(scratch, "repo"))
    tree = copy_as_tree(repository, os.path.join(scratch, "tree"))
    repo_device = os.path.join(repository, DEVICE)
    write_text(repo_device, open(repo_device, encoding="utf-8").read().replace("// device line 20\n", "// device line 20\n        Arm => {}\n"))
    fixture = write_text(os.path.join(scratch, "known.diff"),
                         unified_diff_with_labels(os.path.join(tree, DEVICE), repo_device, "tree", "repo"))
    with_known = {"tree": tree, "H5": {"known_repo_differences": [{"name": "臂", "ruling": "认定", "fixture": fixture}]}}
    holds_known, detail = s16_2(repository, with_known)
    holds_without, _ = s16_2(repository, {"tree": tree, "H5": {}})
    return holds_known and not holds_without, f"认过的差额撤得掉：{holds_known}（{detail[:60]}…）；不认就不过：{not holds_without}"


def cell_fixture_header_timestamps(scratch: str) -> tuple[bool, str]:
    base, middle, _ = three_versions(scratch)
    plain = write_text(os.path.join(scratch, "plain.diff"), subprocess.run(["diff", "-u", base, middle], capture_output=True, text=True).stdout)
    labelled = write_text(os.path.join(scratch, "labelled.diff"), unified_diff_with_labels(base, middle, "old", "new"))
    plain_problems, labelled_problems = fixture_header_problems(plain), fixture_header_problems(labelled)
    return len(plain_problems) == 2 and not labelled_problems, f"不带 --label 的：{plain_problems}；带 --label 的：{labelled_problems or '无'}"


def fake_class_a_product(changed: dict[str, str] | None = None, dropped: str = "") -> list[str]:
    lines = []
    for check, local in CLASS_A_CHECKS:
        if check == dropped:
            continue
        holds = (changed or {}).get(check, "true")
        lines.append(f"E7RESULT name=s1 stream=x check={check} local={local or 'v'} crates={local or 'v'} holds={holds}")
    return lines


def cell_class_a_checks(scratch: str) -> tuple[bool, str]:
    device = "\n".join(CLASS_A_DEVICE_LINES)
    clean = class_a_problems(fake_class_a_product(), device)
    false_one = class_a_problems(fake_class_a_product({"first_file_unlinked_then_inodes_across_one_leaf_root_writes": "false"}), device)
    missing_one = class_a_problems(fake_class_a_product(dropped="third_stream_split_publish"), device)
    device_missing = class_a_problems(fake_class_a_product(), CLASS_A_DEVICE_LINES[0])
    good = (not clean and false_one == ["first_file_unlinked_then_inodes_across_one_leaf_root_writes:holds_false"]
            and missing_one == ["third_stream_split_publish:missing"] and device_missing == ["device_line_missing:CONFIRMED_LOSS_IN_FLIGHT_CALL"])
    return good, f"全过 {clean}；一项 false {false_one}；一项缺 {missing_one}；装置少一行 {device_missing}"


def cell_announcement_count(scratch: str) -> tuple[bool, str]:
    line = f"{ANNOUNCEMENT_PREFIX}（2026-10-08）：……"
    counts = [len(announcement_lines(text)) for text in ("# 记录\n", f"# 记录\n{line}\n", f"# 记录\n{line}\n{line}\n")]
    first_number = announcement_lines(f"a\nb\n{line}\n")[0][0]
    return counts == [0, 1, 2] and first_number == 3, f"零、一、两行宣布数出 {counts}；行号 {first_number}"


def cell_after_sha256_last_wins(scratch: str) -> tuple[bool, str]:
    first = write_text(os.path.join(scratch, "first.sha256"), f"{'a' * 64}  crates/x.rs\n{'b' * 64}  {DEVICE}\n")
    second = write_text(os.path.join(scratch, "second.sha256"), f"{'c' * 64}  crates/x.rs\n")
    chosen = last_sha256_by_path([first, second], "d" * 64)
    without_swap = last_sha256_by_path([first, second], None)
    good = chosen == {"crates/x.rs": "c" * 64, DEVICE: "d" * 64} and without_swap[DEVICE] == "b" * 64
    return good, f"后合的算：x.rs 取 {chosen.get('crates/x.rs', '-')[:1]}；装置取换后值 {chosen.get(DEVICE, '-')[:1]}"


def cell_merge_process(scratch: str) -> tuple[bool, str]:
    running = merge_verification_processes(["  12 bash scripts/merge-and-verify.sh --batch 40", "  13 python3 other.py"])
    idle = merge_verification_processes(["  13 python3 other.py"])
    return len(running) == 1 and not idle, f"有一条合入验证：数出 {len(running)}；没有：{len(idle)}"


def cell_a12_greps(scratch: str) -> tuple[bool, str]:
    tree = fake_repository(os.path.join(scratch, "tree"))
    clean_failed = [name for name, *_, holds in a12_results(tree) if not holds]
    zero_name, zero_relative, zero_needle, _ = next(item for item in GREPS if item[3] == "zero")
    write_text(os.path.join(tree, zero_relative), open(os.path.join(tree, zero_relative), encoding="utf-8").read() + zero_needle + "\n")
    dirty_failed = [name for name, *_, holds in a12_results(tree) if not holds]
    return not clean_failed and dirty_failed == [zero_name], f"齐的树不过的 {clean_failed}；「恰 0」那项出现一次后不过的 {dirty_failed}"


def cell_named_row_anchors(scratch: str) -> tuple[bool, str]:
    tree = fake_repository(os.path.join(scratch, "tree"))
    results = named_row_anchors(tree)
    good = len(results) == 15 and all(status == "ok" and hits == 1 for _, _, status, hits in results)
    return good, f"十五行按名字取、锚点各 1 次：{sum(1 for *_, hits in results if hits == 1)} 行"


def cell_p1f_digest(scratch: str) -> tuple[bool, str]:
    stream = os.path.join(scratch, P1F_STREAM)
    write_text(os.path.join(stream, "bad-expected.tsv"), f"0\tP0-F\tP\tF\n7\t{P1F_ROW_NAME}\tP\tF\n")
    write_text(os.path.join(stream, "bad-states.tsv"), f"0\t22\t{'a' * 64}\t{'b' * 64}\t-\n7\t22\t{'C' * 64}\t{'d' * 64}\t-\n")
    row_id, digest = p1f_base_digest(scratch)
    return (row_id, digest) == (7, "c" * 64), f"P1-F 行号 {row_id}、底态摘要 {digest[:8]}…"


def verdict_fields(path: str) -> dict[str, str]:
    lines = [line for line in open(path, encoding="utf-8") if line.startswith("E7RESULT name=verdict ")]
    return key_values(lines[-1]) if lines else {}


def fake_snapshot_world(scratch: str) -> tuple[str, str, str]:
    """假仓、从它拍的假树（带假的两个二进制）、一份填好的占位配置；交回 (仓, 树, 配置路径)。"""
    repository = fake_repository(os.path.join(scratch, "repo"))
    tree = copy_as_tree(repository, os.path.join(scratch, "tree"))
    for name in (E171_BINARY, E170_BINARY):
        write_text(os.path.join(tree, "target/release", name), f"binary {name}\n")
    device = os.path.join(tree, DEVICE)
    base = write_text(os.path.join(scratch, "base-device.rs"), open(device, encoding="utf-8").read().replace("// device line 30\n", ""))
    block = write_text(os.path.join(scratch, "block-constants.diff"), unified_diff_with_labels(base, device, "ab35105f", "tree"))
    reference = write_text(os.path.join(scratch, "reference.json"), json.dumps({"files": {}, "judged": []}))
    os.environ["FROZEN_TREE_R8_EXPECTED_BASE_SHA256"] = sha256_of_file(base)
    os.environ["FROZEN_TREE_R8_EXPECTED_REVISION_6_DEVICE_SHA256"] = "missing"
    config = {"tree": tree, "H4": {"device_sha256_in_the_tree": sha256_of_file(device)},
              "H5": {"base_device_copy": base, "blocks": [{"name": "常量", "source_kind": "revision_9_constants", "source": "修订 9", "fixture": block,
                                                           "fixture_sha256": sha256_of_file(block)}], "known_repo_differences": []},
              "H7": {"e171_binary_sha256": sha256_of_file(os.path.join(tree, "target/release", E171_BINARY)),
                     "e170_binary_sha256": sha256_of_file(os.path.join(tree, "target/release", E170_BINARY))},
              "H8": {"mutations_table_sha256": sha256_of_file(os.path.join(tree, MUTATIONS_TABLE))}, "H11": {"revision_6_reference": reference}}
    config_path = write_text(os.path.join(scratch, "config.json"), json.dumps(config))
    snapshot = os.path.join(scratch, "snapshot.out")
    at_snapshot(repository, config_path, snapshot)
    config["H3"] = {"snapshot_digest": row_driver.snapshot_digest(tree)[0], "snapshot_product": snapshot, "snapshot_product_sha256": sha256_of_file(snapshot)}
    write_text(config_path, json.dumps(config))
    return repository, tree, config_path


def cell_step_0_end_to_end_green(scratch: str) -> tuple[bool, str]:
    repository, _, config_path = fake_snapshot_world(scratch)
    output = os.path.join(scratch, "frozen-tree.out")
    code = step_0(repository, config_path, output)
    verdict = verdict_fields(output)
    copied = sum(1 for line in open(output, encoding="utf-8") if "key=s16_1" in line)
    return code == 0 and verdict.get("triggered") == "none" and copied == 1, f"退 {code}；triggered={verdict.get('triggered')}；抄进来的 S16 ① 行 {copied}"


def cell_step_0_end_to_end_red(scratch: str) -> tuple[bool, str]:
    repository, tree, config_path = fake_snapshot_world(scratch)
    write_text(os.path.join(tree, "crates/fake/src/row_23.rs"), "fn anchor_23() { 拍完又改 }\n")
    output = os.path.join(scratch, "frozen-tree.out")
    code = step_0(repository, config_path, output)
    triggered = verdict_fields(output).get("triggered", "")
    return code == 1 and "z5" in triggered.split(","), f"拍完改树里一个文件：退 {code}，triggered={triggered}"


def cell_before_snapshot_end_to_end(scratch: str) -> tuple[bool, str]:
    repository = fake_repository(os.path.join(scratch, "repo"))
    after = write_text(os.path.join(scratch, "batch-40", "after.sha256"),
                       "".join(f"{sha256_of_file(os.path.join(repository, path))}  {path}\n" for path in ("Cargo.toml", "crates/fake/src/row_15.rs")))
    line = f"{ANNOUNCEMENT_PREFIX}（2026-10-08）：三条线合完，合后清单 {after}"
    write_text(os.path.join(repository, RECORDS_FILE), f"# 记录\n\n{line}\n")
    draft = copy_as_tree(repository, os.path.join(scratch, "draft"))
    for name in (E171_BINARY, E170_BINARY):
        os.chmod(write_text(os.path.join(draft, "target/release", name), "binary\n"), 0o755)
    product = write_text(os.path.join(scratch, "stream-constants.out"), "\n".join(fake_class_a_product()) + "\n")
    config = {"H1": {"line_number": 3, "line_text": line, "after_sha256_files_in_merge_order": [after], "device_swap": None},
              "H6": {"draft_copy_root": draft, "draft_copy_stream_constants_product": product, "constants_written_in_revision_9": True,
                     "device_sha256_after_revision_9": sha256_of_file(os.path.join(repository, DEVICE))}}
    config_path = write_text(os.path.join(scratch, "config.json"), json.dumps(config))
    os.environ["FROZEN_TREE_R8_PS_COMMAND"] = "true"
    green = os.path.join(scratch, "green.out")
    green_code = before_snapshot(repository, config_path, green)
    write_text(os.path.join(repository, "crates/fake/src/row_15.rs"), "fn anchor_15() { 合后又改 }\n")
    red = os.path.join(scratch, "red.out")
    red_code = before_snapshot(repository, config_path, red)
    green_verdict, red_verdict = verdict_fields(green), verdict_fields(red)
    good = green_code == 0 and green_verdict.get("failed") == "none" and red_code == 1 and red_verdict.get("failed") == "b"
    return good, f"齐的：退 {green_code} failed={green_verdict.get('failed')}；合后又改一个文件：退 {red_code} failed={red_verdict.get('failed')}"


SELFTEST_CELLS = {
    "snapshot-table-ignores-target": cell_snapshot_table_ignores_target, "snapshot-table-content-differs": cell_snapshot_table_content_differs,
    "s16-3-blocks-apply": cell_s16_3_blocks_apply, "s16-3-source-kind": cell_s16_3_source_kind,
    "s16-2-known-difference-removed": cell_s16_2_known_difference_removed, "fixture-header-timestamps": cell_fixture_header_timestamps,
    "class-a-checks": cell_class_a_checks, "announcement-count": cell_announcement_count, "after-sha256-last-wins": cell_after_sha256_last_wins,
    "merge-process": cell_merge_process, "a12-greps": cell_a12_greps, "named-row-anchors": cell_named_row_anchors, "p1f-digest": cell_p1f_digest,
    "step-0-end-to-end-green": cell_step_0_end_to_end_green, "step-0-end-to-end-red": cell_step_0_end_to_end_red,
    "before-snapshot-end-to-end": cell_before_snapshot_end_to_end,
}
SELFTEST_SWITCHES = {
    "s16-1-paths-only": "snapshot-table-content-differs", "s16-1-counts-target": "snapshot-table-ignores-target",
    "s16-2-no-known-differences": "s16-2-known-difference-removed", "s16-3-any-source": "s16-3-source-kind", "s16-3-fuzz": "s16-3-blocks-apply",
    "fixture-timestamps-allowed": "fixture-header-timestamps", "class-a-missing-passes": "class-a-checks", "announcement-count-ignored": "announcement-count",
    "after-sha-first-wins": "after-sha256-last-wins", "merge-process-ignored": "merge-process", "a12-zero-as-at-least-one": "a12-greps",
    "anchors-count-substring": "named-row-anchors", "step-0-ignores-z5": "step-0-end-to-end-red",
}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
