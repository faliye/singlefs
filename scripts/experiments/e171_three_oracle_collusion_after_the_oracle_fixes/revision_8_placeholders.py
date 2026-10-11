#!/usr/bin/env python3
"""E171 第 1 次重跑第一段·修订 8 那一趟：占位 H1–H20（登记 research/prompts/e171-r1-prereg.md 修订 8 第 8.8 节表）集中在一份 JSON 配置里，这里核它。
配置的形照同目录 revision_8_placeholders.json（模板：每个占位一节，值先全是 null；拍树那一刻由执行员与主 agent 照表填）。
H10、H12、H13、H19 是跑的时候现取的（第 0 步判法、第 2b 步、第 7 步、编排器），配置里只写一句它落在哪，核对时报 runtime、不算没填。

用法：
  revision_8_placeholders.py check --config C --phase <before-snapshot|start|contrast> [--repo R]
      这一阶段要的每个占位一行：name=placeholder key=H<n> filled=… holds=… 谁填、什么时候填、核了什么；末行 name=placeholders_verdict。
      before-snapshot：拍树之前（H1、H5、H6、H14、H18、H20）；start：起跑与每次续跑（除现取的四个与 H15 之外全部）；contrast：出两趟对照之前（H3、H15、H16）。
  revision_8_placeholders.py --selftest [--cells 格,…] [--rounds N]
退出码：0 这一阶段要的都填了、都核过；78 有没填或核不过的（verdict 行点名哪几个）；2 用法错。

弄坏开关 PLACEHOLDERS_R8_BREAK=<项>（只在 --selftest 起的子进程里认，PLACEHOLDERS_R8_SELFTEST=1）。项名、弄坏哪一处、打开后红在哪一格：
  unfilled-passes          没填（null）的占位当已填                          unfilled-named
  h14-formula-skipped      H14 不核「主取样点态数 = K 取法数 × 2^C」          h14-formula
  h16-collision-skipped    H16 不核头一次起跑时产物名撞没撞                    h16-collision
  h17-sha-skipped          H17 不核各文件的 sha256                             h17-files
  h18-revision-6-names     H18 与修订 6 那一趟同名也收                          h18-names
  h20-true-accepted        H20 写「分到别的台」也收（编排器没补那一段）        h20-not-supported
  dry-run-accepted         干跑配置在 start 阶段也算过                          dry-run-refused
"""
from __future__ import annotations

import argparse
import dataclasses
import json
import os
import re
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402
import row_driver  # noqa: E402

BREAK_VARIABLE = "PLACEHOLDERS_R8_BREAK"
SELFTEST_VARIABLE = "PLACEHOLDERS_R8_SELFTEST"
EXIT_RUN_CONDITION = 78
PHASES = ("before-snapshot", "start", "contrast")
PRODUCT_STEM = "research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes"
REVISION_8_SUFFIX = "-r8"
DUMPS_PARENT = "scripts/experiments/target/e171-r1-dumps/"
WORK_ROOT = os.path.expanduser("~/.local/share/singlefs/work/")
# 拍树前后那几份（第 8.2 节草稿副本的 stream-constants、拍树前五项核对、拍树那一刻、树里编后）也带同一前缀与后缀，它们不是跑法写的，H16 撞名不算它们
PRODUCTS_BEFORE_THE_RUN = ("stream-constants-on-the-draft-copy", "before-the-snapshot", "tree-snapshot", "tree-snapshot-build")
REVISION_6_PEER_NAMES = ("research/e171-s1-peer-kit", "research/e171-s1-frozen-snapshot", "scripts/experiments/target/e171-s1-peer-out")
RESEARCH_RELATIVE = "scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes"
S2_COMPARE = os.path.expanduser("~/.local/share/singlefs/work/e171-r1-segment1-rev6/s2_compare.py")
# H17：跑法脚本、第 0 步判法、编排器副本、续跑核对、对照脚本，连同它们按 sha256 载入的那几份
H17_REQUIRED_FILES = tuple(f"{RESEARCH_RELATIVE}/{name}" for name in (
    "run_segment_one_on_the_revision_8_tree_multi_host.sh", "segment_one_units_revision_8.py", "segment_one_unit_revision_8.sh",
    "frozen_tree_lines_revision_8.py", "revision_8_placeholders.py", "revision_8_selftest_harness.py", "rows_orchestrator_revision_8.py",
    "contrast_revision_6_against_revision_8.py", "rows_orchestrator.py", "cross_host_o3_check.py", "check_product.py", "device_timing.py",
    "row_driver.py", "peer_side.py")) + (S2_COMPARE,)
HEX64 = re.compile(r"[0-9a-f]{64}")
HEX40 = re.compile(r"[0-9a-f]{40}")
DATE = re.compile(r"\d{4}-\d{2}-\d{2}")
RUNTIME_PLACEHOLDERS = {
    "H10": "第 0 步判法每次现判（frozen-tree 产物 key=a12 那十二行）",
    "H12": "第 2b 步跑完现取（e170-bad-states-second-run-p1-f-base-digest 产物；第 3 步 O3 那一项 base_digest_matches=true 才过）",
    "H13": "第 7 步绿时现取（baseline-run1 的 -timing 产物 process_wall；逐行限时由驱动照它现算）",
    "H19": "编排器每次起跑与续跑现查现算（日志里「本机/第二台 … 槽」那几行）",
}


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def product_prefix(run_date: str) -> str:
    return f"{PRODUCT_STEM}-{run_date}-s1"


@dataclasses.dataclass(frozen=True)
class Outcome:
    filled: bool
    holds: bool
    detail: str


def unfilled(detail: str = "") -> Outcome:
    return Outcome(False, False, detail or "没填")


def is_hex64(value) -> bool:
    return isinstance(value, str) and bool(HEX64.fullmatch(value))


def missing_keys(values: dict, keys: tuple[str, ...]) -> list[str]:
    if broken("unfilled-passes"):
        return []
    return [key for key in keys if values.get(key) is None]


def file_sha_holds(repo: str, path, expected) -> bool:
    return isinstance(path, str) and is_hex64(expected) and judge.sha256_or_missing(judge.resolve(repo, path)) == expected


def verdict_of(repo: str, path: str) -> dict[str, str]:
    full = judge.resolve(repo, path)
    return judge.verdict_fields(full) if os.path.isfile(full) else {}


def check_h1(repo: str, config: dict, phase: str) -> Outcome:
    h1 = judge.section(config, "H1")
    absent = missing_keys(h1, ("line_number", "line_text", "after_sha256_files_in_merge_order"))
    if absent:
        return unfilled("没填：" + ",".join(absent))
    records = judge.resolve(repo, h1.get("records_file") or judge.RECORDS_FILE)
    text = open(records, encoding="utf-8").read() if os.path.isfile(records) else ""
    found = judge.announcement_lines(text)
    matches = len(found) == 1 and found[0] == (h1["line_number"], h1["line_text"])
    after = h1["after_sha256_files_in_merge_order"]
    after_ok = isinstance(after, list) and bool(after) and all(os.path.isfile(judge.resolve(repo, path)) for path in after)
    swap = h1.get("device_swap")
    swap_ok = True
    if swap:
        lines = text.splitlines()
        number = swap.get("line_number")
        swap_line = lines[number - 1] if isinstance(number, int) and 0 < number <= len(lines) else ""
        swap_ok = all(is_hex64(swap.get(key)) and swap[key][:8] in swap_line for key in ("before_sha256", "after_sha256"))
    return Outcome(True, matches and after_ok and swap_ok,
                   f"宣布行 {len(found)} 行、与配置的行号原文相同 {str(matches).lower()}；after.sha256 {len(after) if isinstance(after, list) else '-'} 份都在 "
                   f"{str(after_ok).lower()}；换装置那一行 {'没写' if not swap else str(swap_ok).lower()}（第 8.1 节第 2 条 a）")


def check_h2(repo: str, config: dict, phase: str) -> Outcome:
    h2 = judge.section(config, "H2")
    absent = missing_keys(h2, ("git_head", "git_status_porcelain", "before_snapshot_product", "before_snapshot_product_sha256"))
    if absent:
        return unfilled("没填：" + ",".join(absent))
    head_ok = isinstance(h2["git_head"], str) and bool(HEX40.fullmatch(h2["git_head"]))
    product_ok = file_sha_holds(repo, h2["before_snapshot_product"], h2["before_snapshot_product_sha256"])
    verdict = verdict_of(repo, h2["before_snapshot_product"]) if product_ok else {}
    return Outcome(True, head_ok and product_ok and verdict.get("triggered") == "none",
                   f"HEAD 形 {str(head_ok).lower()}；拍树前核对产物 sha256 对得上 {str(product_ok).lower()}、triggered={verdict.get('triggered', '-')}（第 8.1 节第 2 条 b、c）")


def check_h3(repo: str, config: dict, phase: str) -> Outcome:
    h3 = judge.section(config, "H3")
    keys = ("snapshot_digest", "files") if phase == "contrast" else ("snapshot_digest", "files", "snapshot_product", "snapshot_product_sha256",
                                                                     "build_product", "build_product_sha256")
    absent = missing_keys(h3, keys)
    if absent:
        return unfilled("没填：" + ",".join(absent))
    tree = judge.resolve(repo, judge.tree_of(config))
    digest = row_driver.snapshot_digest(tree)[0] if os.path.isdir(tree) else "missing"
    if phase == "contrast":
        return Outcome(True, digest == h3["snapshot_digest"], f"树的快照摘要现算 {digest[:12]}，等于 H3 {str(digest == h3['snapshot_digest']).lower()}")
    snapshot_ok = file_sha_holds(repo, h3["snapshot_product"], h3["snapshot_product_sha256"])
    build_ok = file_sha_holds(repo, h3["build_product"], h3["build_product_sha256"])
    s16_1 = verdict_of(repo, h3["snapshot_product"]).get("s16_1_triggered") if snapshot_ok else None
    same = verdict_of(repo, h3["build_product"]).get("digest_before_equals_after") if build_ok else None
    holds = snapshot_ok and build_ok and s16_1 == "false" and same == "true" and digest == h3["snapshot_digest"]
    return Outcome(True, holds, f"拍树产物 {str(snapshot_ok).lower()}（s16_1_triggered={s16_1}）；编后产物 {str(build_ok).lower()}（编前编后摘要相同 {same}）；"
                                f"树现算 {digest[:12]}，等于 H3 {str(digest == h3['snapshot_digest']).lower()}")


def check_h4(repo: str, config: dict, phase: str) -> Outcome:
    value = judge.section(config, "H4").get("device_sha256_in_the_tree")
    if value is None and not broken("unfilled-passes"):
        return unfilled()
    tree = judge.resolve(repo, judge.tree_of(config))
    actual = judge.sha256_or_missing(os.path.join(tree, judge.DEVICE))
    snapshot = judge.section(config, "H3").get("snapshot_product")
    recorded = None
    if isinstance(snapshot, str) and os.path.isfile(judge.resolve(repo, snapshot)):
        recorded = [judge.key_values(line) for line in open(judge.resolve(repo, snapshot), encoding="utf-8") if "key=device_sha256 " in line]
    at_moment = bool(recorded) and recorded[0].get("tree") == recorded[0].get("repo") == value
    return Outcome(True, actual == value and at_moment, f"树里装置现算 {actual[:12]}，等于 H4 {str(actual == value).lower()}；拍树那一刻树与仓相同且等于 H4 {str(at_moment).lower()}")


def check_h5(repo: str, config: dict, phase: str) -> Outcome:
    h5 = judge.section(config, "H5")
    absent = missing_keys(h5, ("base_device_copy", "blocks", "known_repo_differences"))
    if absent:
        return unfilled("没填：" + ",".join(absent))
    target = judge.sha256_or_missing(os.path.join(repo, judge.DEVICE)) if phase == "before-snapshot" else judge.section(config, "H4").get("device_sha256_in_the_tree")
    holds, rows, summary = judge.s16_3(repo, config, target or "missing")
    known = h5["known_repo_differences"]
    known_ok = isinstance(known, list) and all(file_sha_holds(repo, entry.get("fixture"), entry.get("fixture_sha256")) and entry.get("ruling") for entry in known)
    fields = judge.key_values(summary)
    return Outcome(True, holds and known_ok, f"块 {fields.get('blocks')} 块依次打上等于{'仓里' if phase == 'before-snapshot' else '树里'}装置 {fields.get('identical')}，"
                                             f"没来源 {fields.get('blocks_without_source')}、fixture 不对 {fields.get('fixture_problems')}；认过的已知差额 {len(known) if isinstance(known, list) else '-'} 块都有认定与 sha256 {str(known_ok).lower()}")


def draft_product_lines(repo: str, config: dict) -> list[str]:
    path = judge.section(config, "H6").get("draft_copy_stream_constants_product")
    full = judge.resolve(repo, path) if isinstance(path, str) else ""
    return open(full, encoding="utf-8").read().splitlines() if full and os.path.isfile(full) else []


def check_h6(repo: str, config: dict, phase: str) -> Outcome:
    h6 = judge.section(config, "H6")
    keys = ("draft_copy_root", "draft_copy_stream_constants_product", "device_sha256_after_revision_9", "constants_written_in_revision_9")
    absent = missing_keys(h6, keys + (("line_numbers_written_in_revision_9",) if phase == "start" else ()))
    if absent:
        return unfilled("没填：" + ",".join(absent))
    lines = draft_product_lines(repo, config)
    device = os.path.join(judge.resolve(repo, h6["draft_copy_root"]), judge.DEVICE)
    problems = judge.class_a_problems(lines, open(device, encoding="utf-8").read() if os.path.isfile(device) else "") if lines else ["draft_product_missing"]
    heading = judge.REVISION_9_HEADING in open(os.path.join(repo, judge.PREREGISTRATION), encoding="utf-8").read()
    target = judge.sha256_or_missing(os.path.join(repo, judge.DEVICE)) if phase == "before-snapshot" else judge.section(config, "H4").get("device_sha256_in_the_tree")
    device_ok = h6["device_sha256_after_revision_9"] == target
    written = h6["constants_written_in_revision_9"] is True and (phase != "start" or h6.get("line_numbers_written_in_revision_9") is True)
    return Outcome(True, not problems and heading and device_ok and written,
                   f"甲类没过的 {','.join(problems) or '无'}；登记里有「{judge.REVISION_9_HEADING}」{str(heading).lower()}；修订 9 写的装置 sha256 等于"
                   f"{'仓里' if phase == 'before-snapshot' else '树里'}那份 {str(device_ok).lower()}；修订 9 常量{'与行号' if phase == 'start' else ''}已写 {str(written).lower()}")


def check_h7(repo: str, config: dict, phase: str) -> Outcome:
    h7 = judge.section(config, "H7")
    absent = missing_keys(h7, ("e171_binary_sha256", "e170_binary_sha256"))
    if absent:
        return unfilled("没填：" + ",".join(absent))
    tree = judge.resolve(repo, judge.tree_of(config))
    actual = {key: judge.sha256_or_missing(os.path.join(tree, "target/release", name))
              for key, name in (("e171_binary_sha256", judge.E171_BINARY), ("e170_binary_sha256", judge.E170_BINARY))}
    build = judge.section(config, "H3").get("build_product")
    recorded = set()
    if isinstance(build, str) and os.path.isfile(judge.resolve(repo, build)):
        recorded = {judge.key_values(line).get("sha256") for line in open(judge.resolve(repo, build), encoding="utf-8") if "key=binary_sha256 " in line}
    holds = all(actual[key] == h7[key] for key in actual) and {h7["e171_binary_sha256"], h7["e170_binary_sha256"]} <= recorded
    return Outcome(True, holds, f"树里两个二进制现算等于 H7 {str(all(actual[key] == h7[key] for key in actual)).lower()}；编后产物记的与 H7 相同 "
                                f"{str({h7['e171_binary_sha256'], h7['e170_binary_sha256']} <= recorded).lower()}")


def check_h8(repo: str, config: dict, phase: str) -> Outcome:
    value = judge.section(config, "H8").get("mutations_table_sha256")
    if value is None and not broken("unfilled-passes"):
        return unfilled()
    tree = judge.resolve(repo, judge.tree_of(config))
    actual = judge.sha256_or_missing(os.path.join(tree, judge.MUTATIONS_TABLE))
    snapshot = judge.section(config, "H3").get("snapshot_product")
    lines = [judge.key_values(line) for line in open(judge.resolve(repo, snapshot), encoding="utf-8") if "key=mutations_table_sha256 " in line] \
        if isinstance(snapshot, str) and os.path.isfile(judge.resolve(repo, snapshot)) else []
    at_moment = bool(lines) and lines[0].get("tree") == lines[0].get("repo") == value
    return Outcome(True, actual == value and at_moment, f"树里 crates/mutations.tsv 现算等于 H8 {str(actual == value).lower()}；拍树那一刻等于仓里那份 {str(at_moment).lower()}")


def check_h9(repo: str, config: dict, phase: str) -> Outcome:
    h9 = judge.section(config, "H9")
    hits = h9.get("anchor_hits")
    if hits is None and not broken("unfilled-passes"):
        return unfilled()
    tree = judge.resolve(repo, judge.tree_of(config))
    now = {str(number): count for number, _, _, count in judge.named_row_anchors(tree)} if os.path.isdir(tree) else {}
    same = isinstance(hits, dict) and now == {key: value for key, value in hits.items()}
    not_once = sorted((key for key, value in now.items() if value != 1), key=int)
    decided = not not_once or bool(h9.get("main_agent_decision_for_rows_not_hit_once"))
    return Outcome(True, same and decided, f"十五行现取与 H9 相同 {str(same).lower()}；不是恰 1 次的行 {','.join(not_once) or '无'}"
                                          f"{'（主 agent 定：' + str(h9.get('main_agent_decision_for_rows_not_hit_once')) + '）' if not_once else ''}")


def check_h11(repo: str, config: dict, phase: str) -> Outcome:
    path = judge.section(config, "H11").get("revision_6_reference")
    if path is None and not broken("unfilled-passes"):
        return unfilled()
    full = judge.resolve(repo, path or "")
    try:
        reference = json.load(open(full, encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return Outcome(True, False, f"参照表读不到：{path}")
    good = isinstance(reference.get("files"), dict) and isinstance(reference.get("judged"), list) and bool(reference.get("judged"))
    return Outcome(True, good, f"修订 6 那一趟的研究一侧参照表：记了 {len(reference.get('files', {}))} 个文件，判「有不同就补变异表」的 {len(reference.get('judged', []))} 个")


def check_h14(repo: str, config: dict, phase: str) -> Outcome:
    h14 = judge.section(config, "H14")
    absent = missing_keys(h14, ("key_choices", "in_flight_writes", "reverse_states", "main_states", "piece_panic_states"))
    if absent:
        return unfilled("没填：" + ",".join(absent))
    key_choices, writes = h14["key_choices"], h14["in_flight_writes"]
    formula = (h14["main_states"] == key_choices * 2 ** writes and h14["reverse_states"] == key_choices
               and h14["piece_panic_states"] == [key_choices * 16, key_choices * 16, key_choices * 8]) or broken("h14-formula-skipped")
    crates = {}
    for line in draft_product_lines(repo, config):
        fields = judge.key_values(line)
        if fields.get("check") in ("confirmed_loss_key_choices", "confirmed_loss_in_flight_writes", "confirmed_loss_states"):
            crates[fields["check"]] = fields.get("crates")
    from_draft = crates == {"confirmed_loss_key_choices": str(key_choices), "confirmed_loss_in_flight_writes": str(writes),
                            "confirmed_loss_states": str(h14["main_states"])}
    return Outcome(True, formula and from_draft, f"主取样点 {h14['main_states']} = {key_choices} × 2^{writes}、反向 = K、三件 panic 对照 = K×16、K×16、K×8："
                                                 f"{str(formula).lower()}；与草稿副本 stream-constants 的 crates= 那一栏相同 {str(from_draft).lower()}")


def check_h15(repo: str, config: dict, phase: str) -> Outcome:
    h15 = judge.section(config, "H15")
    absent = missing_keys(h15, ("comparator_product", "dumps_root", "product_prefix", "product_suffix", "rows_tag"))
    if absent:
        return unfilled("没填：" + ",".join(absent))
    comparator = os.path.isfile(judge.resolve(repo, h15["comparator_product"]))
    dumps = os.path.isdir(judge.resolve(repo, h15["dumps_root"]))
    finished = h15.get("rows_finished") is True or bool(h15.get("stopped_at"))
    return Outcome(True, comparator and dumps and finished, f"修订 6 那一趟比对器产物在 {str(comparator).lower()}、导出根在 {str(dumps).lower()}、"
                                                          f"第 11–13 步跑完或写明停在哪 {str(finished).lower()}")


def check_h16(repo: str, config: dict, phase: str) -> Outcome:
    h16 = judge.section(config, "H16")
    absent = missing_keys(h16, ("run_date", "run_suffix", "dumps_root", "logs_directory"))
    if absent:
        return unfilled("没填：" + ",".join(absent))
    dry = bool(config.get("dry_run"))
    shapes = (bool(DATE.fullmatch(str(h16["run_date"]))) and (dry or h16["run_suffix"] == REVISION_8_SUFFIX)
              and str(h16["dumps_root"]).startswith(DUMPS_PARENT) and os.path.isabs(str(h16["logs_directory"]))
              and os.path.abspath(str(h16["logs_directory"])).startswith(WORK_ROOT))
    first_start = not os.path.exists(os.path.join(str(h16["logs_directory"]), "sequence.log"))
    collisions = []
    if first_start and phase == "start" and not broken("h16-collision-skipped"):
        prefix = os.path.basename(product_prefix(h16["run_date"]))
        results = os.path.join(repo, "research/results")
        before_the_run = {f"{prefix}-{stem}{h16['run_suffix']}{ending}" for stem in PRODUCTS_BEFORE_THE_RUN for ending in (".out", "-timing.out")}
        collisions = [name for name in os.listdir(results) if name.startswith(prefix + "-") and name not in before_the_run and
                      (name.endswith(f"{h16['run_suffix']}.out") or name.endswith(f"{h16['run_suffix']}-timing.out"))]
        if os.path.exists(judge.resolve(repo, h16["dumps_root"])):
            collisions.append(h16["dumps_root"])
    return Outcome(True, shapes and not collisions, f"日期、后缀 {h16['run_suffix']}、导出根在 {DUMPS_PARENT} 下、日志目录在工作根下：{str(shapes).lower()}；"
                                                   f"{'头一次起跑，撞名的 ' + str(len(collisions)) + ' 个' if first_start else '续跑（日志目录里已有 sequence.log），不核撞名'}")


def check_h17(repo: str, config: dict, phase: str) -> Outcome:
    files = judge.section(config, "H17").get("files")
    if files is None and not broken("unfilled-passes"):
        return unfilled()
    files = files or {}
    missing = [path for path in H17_REQUIRED_FILES if path not in files]
    differing = [] if broken("h17-sha-skipped") else [path for path, expected in files.items() if judge.sha256_or_missing(judge.resolve(repo, path)) != expected]
    return Outcome(True, not missing and not differing, f"登了 {len(files)} 份；该登没登的 {len(missing)} 份；sha256 与现在不同的 {','.join(os.path.basename(p) for p in differing) or '无'}")


def check_h18(repo: str, config: dict, phase: str) -> Outcome:
    h18 = judge.section(config, "H18")
    absent = missing_keys(h18, ("peer_kit", "peer_snapshot", "peer_out"))
    if absent:
        return unfilled("没填：" + ",".join(absent))
    names = (h18["peer_kit"], h18["peer_snapshot"], h18["peer_out"])
    clash = [] if broken("h18-revision-6-names") else [name for name in names if name in REVISION_6_PEER_NAMES]
    shapes = all(isinstance(name, str) and name.startswith("research/") for name in names) and len(set(names)) == 3 \
        and str(h18["peer_out"]).startswith("scripts/experiments/target/")
    return Outcome(True, shapes and not clash, f"三个落点都在 research/ 下、互不相同、取回的那个在 scripts/experiments/target/ 下 {str(shapes).lower()}；与修订 6 那一趟同名的 {clash or '无'}")


def check_h20(repo: str, config: dict, phase: str) -> Outcome:
    h20 = judge.section(config, "H20")
    absent = missing_keys(h20, ("steps_3_and_9_on_other_hosts", "why"))
    if absent:
        return unfilled("没填：" + ",".join(absent))
    supported = h20["steps_3_and_9_on_other_hosts"] is False or broken("h20-true-accepted")
    return Outcome(True, supported and len(str(h20["why"])) >= 8,
                   "第 3、9 步留本机（逐行编排外层没补「别的台从快照编、跑一趟装置、取回」那一段）" if h20["steps_3_and_9_on_other_hosts"] is False
                   else "写了分到别的台，而 rows_orchestrator_revision_8.py 没有那一段：不收")


def check_tree(repo: str, config: dict, phase: str) -> Outcome:
    tree = judge.tree_of(config)
    dry = bool(config.get("dry_run"))
    if dry and phase == "start" and not broken("dry-run-accepted"):
        return Outcome(True, False, f"这是干跑配置（{config.get('dry_run_note') or '没写说明'}）：只给 PLAN_ONLY 用，不许真起跑")
    holds = dry or tree == judge.FINAL_MAINLINE_TREE
    return Outcome(True, holds and os.path.isdir(judge.resolve(repo, tree)), f"树 {tree}{'（干跑）' if dry else ''} 在 {str(os.path.isdir(judge.resolve(repo, tree))).lower()}")


@dataclasses.dataclass(frozen=True)
class Placeholder:
    key: str
    what: str
    who: str
    when: str
    phases: tuple[str, ...]
    check: object


PLACEHOLDERS = (
    Placeholder("tree", "修订 8 树的路径（不是干跑配置时必须是 scripts/experiments/target/e171-r1-frozen-tree-final-mainline）", "模板写死", "—", ("start",), check_tree),
    Placeholder("H1", "宣布行的行号与原文", "主 agent 写宣布行，执行员抄", "全部合完时", ("before-snapshot", "start"), check_h1),
    Placeholder("H2", "HEAD 与 git status 全文", "执行员", "拍树那一刻", ("start",), check_h2),
    Placeholder("H3", "修订 8 树的快照摘要与文件数", "执行员", "拍完、树里编过", ("start", "contrast"), check_h3),
    Placeholder("H4", "树里装置的 sha256", "执行员", "拍树那一刻", ("start",), check_h4),
    Placeholder("H5", "装置对 ab35105f… 的块表", "改装置的那一方（常量那几块是执行员），主 agent 认", "拍树之前", ("before-snapshot", "start"), check_h5),
    Placeholder("H6", "修订 9：常量与第三节行号", "执行员", "常量在拍树之前；行号在拍树之后、第 0 步之前", ("before-snapshot", "start"), check_h6),
    Placeholder("H7", "两个 release 二进制的 sha256", "执行员", "树里编完", ("start",), check_h7),
    Placeholder("H8", "树里 crates/mutations.tsv 的 sha256", "执行员", "拍树那一刻", ("start",), check_h8),
    Placeholder("H9", "十五行按名字取得到否、锚点命中次数", "执行员", "拍树之后、第 0 步之前", ("start",), check_h9),
    Placeholder("H11", "研究一侧各文件 sha256 与修订 6 那一趟用的比（参照表）", "执行员", "第 0 步", ("start",), check_h11),
    Placeholder("H14", "第 6、10 步 q5 states= 的期望与丙类精确数", "执行员", "拍树之前", ("before-snapshot", "start"), check_h14),
    Placeholder("H15", "修订 6 那一趟拿来对照的产物与导出目录清单", "主 agent", "出对照产物之前", ("contrast",), check_h15),
    Placeholder("H16", "RUN_DATE、RUN_SUFFIX、导出根、日志目录", "主 agent", "起跑时", ("start", "contrast"), check_h16),
    Placeholder("H17", "跑法、判法、编排器副本、续跑核对、对照脚本的 sha256", "执行员写完，主 agent 认", "起跑之前", ("start",), check_h17),
    Placeholder("H18", "别的台的三个落点", "执行员", "写跑法时", ("before-snapshot", "start"), check_h18),
    Placeholder("H20", "第 3、9 步分不分到别的台", "执行员写，主 agent 认", "起跑之前", ("before-snapshot", "start"), check_h20),
)


def check_config(repo: str, config: dict, phase: str) -> tuple[list[str], list[str], list[str]]:
    """这一阶段要的每个占位：打一行，交回 (没填的, 核不过的, 打出的行)。现取的四个只打一行说明。"""
    unfilled_keys, failing, printed = [], [], []
    for item in PLACEHOLDERS:
        if phase not in item.phases:
            continue
        outcome = item.check(repo, config, phase)
        if not outcome.filled:
            unfilled_keys.append(item.key)
        elif not outcome.holds:
            failing.append(item.key)
        printed.append(f"name=placeholder key={item.key} phase={phase} filled={str(outcome.filled).lower()} holds={str(outcome.holds).lower()} "
                       f"who={item.who} when={item.when} what={item.what} detail={outcome.detail}")
    if phase == "start":
        for key, where in RUNTIME_PLACEHOLDERS.items():
            printed.append(f"name=placeholder key={key} phase={phase} filled=runtime where={where}")
    return unfilled_keys, failing, printed


def check_command(repo: str, config_path: str, phase: str) -> int:
    try:
        config = judge.load_config(config_path)
    except judge.RunConditionError as error:
        print(f"✗ {error}")
        return EXIT_RUN_CONDITION
    unfilled_keys, failing, printed = check_config(repo, config, phase)
    for line in printed:
        print(line)
    print(f"name=placeholders_verdict phase={phase} config={config_path} config_sha256={judge.sha256_of_file(config_path)} dry_run={str(bool(config.get('dry_run'))).lower()} "
          f"unfilled={','.join(unfilled_keys) or 'none'} failing={','.join(failing) or 'none'} runtime={','.join(RUNTIME_PLACEHOLDERS) if phase == 'start' else 'none'}")
    return EXIT_RUN_CONDITION if unfilled_keys or failing else 0


def main(argument_list: list[str]) -> int:
    refusal = harness.refuse_break_outside_selftest(BREAK_VARIABLE, SELFTEST_VARIABLE)
    if refusal:
        print(refusal, file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 修订 8 那一趟：占位 H1–H20 的核对")
    parser.add_argument("mode", choices=("check",))
    parser.add_argument("--config", required=True)
    parser.add_argument("--phase", required=True, choices=PHASES)
    parser.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    return check_command(os.path.abspath(arguments.repo), arguments.config, arguments.phase)


# ── 自证：只在假的配置与假仓上核几条已知答案（判法本身的端到端在 frozen_tree_lines_revision_8.py 的自证里） ──
def template() -> dict:
    with open(os.path.join(HERE, "revision_8_placeholders.json"), encoding="utf-8") as handle:
        return json.load(handle)


def cell_template_shape(scratch: str) -> tuple[bool, str]:
    config = template()
    keys = {item.key for item in PLACEHOLDERS} | set(RUNTIME_PLACEHOLDERS) | {"H19"}
    present = sorted(key for key in keys if key in config or key == "tree")
    return set(present) == keys and config.get("dry_run") is False, f"模板里有的占位 {len(present)} 个（要 {len(keys)}），dry_run={config.get('dry_run')}"


def cell_unfilled_named(scratch: str) -> tuple[bool, str]:
    unfilled_keys, failing, _ = check_config(scratch, template(), "before-snapshot")
    expected = ["H1", "H5", "H6", "H14"]
    return unfilled_keys == expected, f"模板（值全空）在拍树之前那一阶段点名的没填 {unfilled_keys}（要 {expected}），核不过的 {failing}"


def cell_h14_formula(scratch: str) -> tuple[bool, str]:
    product = judge.write_text(os.path.join(scratch, "draft.out"), "E7RESULT name=s1 check=confirmed_loss_key_choices local=72 crates=80 holds=false\n"
                               "E7RESULT name=s1 check=confirmed_loss_in_flight_writes local=24 crates=24 holds=true\n"
                               f"E7RESULT name=s1 check=confirmed_loss_states local=1 crates={80 * 2 ** 24} holds=false\n")
    good = {"H6": {"draft_copy_stream_constants_product": product},
            "H14": {"key_choices": 80, "in_flight_writes": 24, "reverse_states": 80, "main_states": 80 * 2 ** 24, "piece_panic_states": [1280, 1280, 640]}}
    # 错的那一份：主取样点少乘一个 2，草稿产物也跟着写成那个数（只有式子核得出它错）
    bad_product = judge.write_text(os.path.join(scratch, "draft-bad.out"), open(product, encoding="utf-8").read().replace(str(80 * 2 ** 24), str(80 * 2 ** 23)))
    bad = json.loads(json.dumps(good))
    bad["H14"]["main_states"] = 80 * 2 ** 23
    bad["H6"]["draft_copy_stream_constants_product"] = bad_product
    good_outcome, bad_outcome = check_h14(scratch, good, "start"), check_h14(scratch, bad, "start")
    return good_outcome.holds and not bad_outcome.holds, f"对的过 {good_outcome.holds}；主取样点少乘一个 2 的 {bad_outcome.holds}（要 False）"


def cell_h16_collision(scratch: str) -> tuple[bool, str]:
    repo = os.path.join(scratch, "repo")
    os.makedirs(os.path.join(repo, "research/results"))
    config = {"H16": {"run_date": "2026-10-09", "run_suffix": "-r8", "dumps_root": "scripts/experiments/target/e171-r1-dumps/revision-8-2026-10-09",
                      "logs_directory": os.path.join(WORK_ROOT, "e171-r8-selftest-never-created", "sequence")}}
    judge.write_text(os.path.join(repo, product_prefix("2026-10-09") + "-tree-snapshot-r8.out"), "拍树那一刻的记录\n")
    clean = check_h16(repo, config, "start")
    judge.write_text(os.path.join(repo, product_prefix("2026-10-09") + "-frozen-tree-r8.out"), "早先一份\n")
    clashing = check_h16(repo, config, "start")
    return clean.holds and not clashing.holds, f"只有拍树那几份时不算撞 {clean.holds}；已有一份跑法会写的同名产物 {clashing.holds}（要 False）"


def cell_h17_files(scratch: str) -> tuple[bool, str]:
    files = {path: judge.sha256_or_missing(judge.resolve(judge_repo(), path)) for path in H17_REQUIRED_FILES}
    good = check_h17(judge_repo(), {"H17": {"files": files}}, "start")
    files[H17_REQUIRED_FILES[0]] = "0" * 64
    bad = check_h17(judge_repo(), {"H17": {"files": files}}, "start")
    return good.holds and not bad.holds, f"现算的 sha256 登进去过 {good.holds}；改一个字 {bad.holds}（要 False）"


def judge_repo() -> str:
    return os.path.abspath(os.path.join(HERE, "../../.."))


def cell_h18_names(scratch: str) -> tuple[bool, str]:
    template_names = check_h18(scratch, template(), "start")
    old = check_h18(scratch, {"H18": dict(zip(("peer_kit", "peer_snapshot", "peer_out"), REVISION_6_PEER_NAMES))}, "start")
    return template_names.holds and not old.holds, f"模板的三个落点过 {template_names.holds}；修订 6 那三个名字 {old.holds}（要 False）"


def cell_h20_not_supported(scratch: str) -> tuple[bool, str]:
    yes = check_h20(scratch, {"H20": {"steps_3_and_9_on_other_hosts": True, "why": "想分到第二台去跑"}}, "start")
    no = check_h20(scratch, template(), "start")
    return no.holds and not yes.holds, f"模板（留本机）过 {no.holds}；写分到别的台 {yes.holds}（要 False）"


def cell_dry_run_refused(scratch: str) -> tuple[bool, str]:
    dry = check_tree(scratch, {"dry_run": True, "dry_run_note": "自证", "tree": scratch}, "start")
    plan = check_tree(scratch, {"dry_run": True, "dry_run_note": "自证", "tree": scratch}, "before-snapshot")
    return not dry.holds and plan.holds, f"干跑配置在 start 阶段 {dry.holds}（要 False）；别的阶段 {plan.holds}"


SELFTEST_CELLS = {"template-shape": cell_template_shape, "unfilled-named": cell_unfilled_named, "h14-formula": cell_h14_formula,
                  "h16-collision": cell_h16_collision, "h17-files": cell_h17_files, "h18-names": cell_h18_names,
                  "h20-not-supported": cell_h20_not_supported, "dry-run-refused": cell_dry_run_refused}
SELFTEST_SWITCHES = {"unfilled-passes": "unfilled-named", "h14-formula-skipped": "h14-formula", "h16-collision-skipped": "h16-collision",
                     "h17-sha-skipped": "h17-files", "h18-revision-6-names": "h18-names", "h20-true-accepted": "h20-not-supported",
                     "dry-run-accepted": "dry-run-refused"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
