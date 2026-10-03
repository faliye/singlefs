"""E170 第 2 次重跑第二段：Q2、Q3 比对器（`research/prompts/e170-r2-prereg.md` 第一节读法 7、8，第六节 Q2-a…Q2-pc、Q3-a…Q3-pc，第八节轨迹）。

只读 v1、v2 的判定行与参照谓词 W_ref、T_ref 的输出（`witness_reference.py`），不重算判定（5.4）。K = 第 242、269 行上 W_ref 为真的态，
K270 = 第 270 行上 T_ref 为假的态（读法 7、8：一律取参照谓词，不取 v2 自己的 W、T，否则 Q2-a、Q3-a 恒为 0）。误红 = 合法态上前判定「在」、
v2 严格判「不在」、`failed_predicates` 含 T（读法 8）；W 在合法态上造成的翻面另报（第十节 F3「W 那一支记 Q1 乙」）。

跑法：python3 comparator_witness_classes.py --row <行号>:<v1 产物>:<v2 产物>:<参照产物> [...]
      --mutated <行号>:<X21|X27>:<变异后的 v2 产物> [...]
      --legal <输入名>:<组>:<states|images>:<批，逗号分隔或 -（坏镜像）>:<v2 产物>:<参照产物> [...]
      --positive-control-legal-input <输入名> --scratch <草稿目录> --output <新文件>
"""

from __future__ import annotations

import argparse
import os
import re
import sys

sys.dont_write_bytecode = True

FIELD_PATTERN = re.compile(r"(\w+)=(\S*)")
WITNESS_ROWS = ("242", "269")  # Q2（读法 7）
TAIL_ROW = "270"  # Q3（读法 8）
IN, NOT_IN, SPEC_GAP = "in", "not_in", "spec_gap"
STREAMS = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")


def fields_of(line: str) -> dict[str, str]:
    return dict(FIELD_PATTERN.findall(line.rstrip("\n").removeprefix("E7RESULT ")))


def keyed(path: str, name: str, batches: set[str] | None, legal_images_only: bool = False) -> dict[tuple, dict[str, str]]:
    """判定行与参照行都按 (来源, 流, 态号) 对齐；executor_main 的态来源记 state。坏镜像只取不打补丁的底态（合法态，读法 6）。"""
    with open(path, encoding="utf-8") as handle:
        parsed = [fields_of(line) for line in handle if line.startswith(f"name={name} ")]
    result = {}
    for fields in parsed:
        source = fields.get("source", "state")
        if legal_images_only and source != "base":
            continue
        if batches is not None and source == "state" and fields.get("batch") not in batches:
            continue
        result[(source, fields["stream"], fields["state"])] = fields
    return result


def v2_witness_value(fields: dict[str, str]) -> str:
    return "not_evaluated" if fields.get("witness_branch") == "not_evaluated" else fields.get("witness_true", "missing")


def reference_witness_value(fields: dict[str, str]) -> str:
    return "not_evaluated" if fields.get("w_ref_evaluated") != "true" else fields.get("w_ref", "missing")


def joined(v2_lines: dict, reference_lines: dict) -> list[tuple[tuple, dict, dict]]:
    return [(key, v2_lines[key], reference_lines[key]) for key in sorted(v2_lines) if key in reference_lines]


def trajectory_line(label: str, stream: str, quantity: str, member_keys: set[tuple], v2_lines: dict, emit) -> None:
    """第八节：按段序报峰值、为正的段数、最后一段的值（段号取判定行的 segment=）。"""
    segments = sorted({int(fields.get("segment", "-1")) for key, fields in v2_lines.items() if key[1] == stream})
    by_segment = [sum(1 for key in member_keys if key[1] == stream and int(v2_lines[key].get("segment", "-1")) == segment) for segment in segments]
    emit(f"name=trajectory input={label} stream={stream} quantity={quantity} segments={len(segments)} peak={max(by_segment, default=0)} "
         f"positive_segments={sum(count > 0 for count in by_segment)} last_segment_value={by_segment[-1] if by_segment else 0}")


def row_red_on_e171_reading_three(v_lines: dict, batch: str) -> str:
    """E171 原登记读法 3：一行在 oracle 上算红 ⟺ 这一行两条流的批 1 里至少判红一处（O3 按状态算；这里红 = 严格跑法判不在）。批 2 另报。"""
    judged = [fields for key, fields in v_lines.items() if fields.get("batch") == batch]
    return "red" if any(fields.get("strict") == NOT_IN for fields in judged) else ("pass" if judged else "no_states")


def q2_counts(row: str, v2_lines: dict, reference_lines: dict, emit, quiet: bool = False) -> dict[str, int]:
    pairs = joined(v2_lines, reference_lines)
    members = {key for key, _, reference in pairs if reference_witness_value(reference) == "true"}
    judged_in = [key for key in sorted(members) if v2_lines[key].get("strict") == IN]
    counts = {"K": len(members), "q2a_K_judged_in": len(judged_in),
              "q2d_K_spec_gap": sum(v2_lines[key].get("strict") == SPEC_GAP for key in members),
              "q2c_witness_disagreements": sum(v2_witness_value(v2) != reference_witness_value(reference) for _, v2, reference in pairs)}
    if quiet:
        return counts
    for key in judged_in:
        fields, reference = v2_lines[key], reference_lines[key]
        emit(f"name=q2a_state row={row} stream={key[1]} state={key[2]} batch={fields.get('batch')} landed={fields.get('landed')} "
             f"v2_witness_branch={fields.get('witness_branch')} ref_w_branch={reference.get('w_branch')} c_witness={reference.get('c_witness')} "
             f"ref_compared={reference.get('w_compared')} v2_witness_true={fields.get('witness_true')}")
    for key, v2, reference in pairs:
        if v2_witness_value(v2) != reference_witness_value(reference):
            emit(f"name=q2c_disagreement row={row} stream={key[1]} state={key[2]} v2_witness={v2_witness_value(v2)} "
                 f"v2_branch={v2.get('witness_branch')} v2_counter={v2.get('witness_counter')} v2_compared={v2.get('witness_compared')} "
                 f"ref_witness={reference_witness_value(reference)} ref_branch={reference.get('w_branch')} ref_counter={reference.get('c_witness')} "
                 f"ref_compared={reference.get('w_compared')} landed={v2.get('landed')}")
    for stream in STREAMS:
        for batch in ("1", "2"):
            subset = [key for key in members if key[1] == stream and v2_lines[key].get("batch") == batch]
            emit(f"name=q2b_size row={row} stream={stream} batch={batch} K={len(subset)} "
                 f"K_judged_in={sum(v2_lines[key].get('strict') == IN for key in subset)} "
                 f"K_judged_not_in={sum(v2_lines[key].get('strict') == NOT_IN for key in subset)}")
    branches = {}
    for key in members:
        name = f"ref_{reference_lines[key].get('w_branch')}"
        branches[name] = branches.get(name, 0) + 1
        name = f"v2_{v2_lines[key].get('witness_branch')}"
        branches[name] = branches.get(name, 0) + 1
    emit(f"name=q2e_branches row={row} " + (" ".join(f"{name}={count}" for name, count in sorted(branches.items())) or "none=0"))
    fewer = sum(reference.get("disks_with_fewer_than_two_self_verified_slots", "0") != "0" for _, _, reference in pairs)
    emit(f"name=q2i_disk_with_fewer_than_two_self_verified_slots row={row} states={fewer} of={len(pairs)}")
    for batch in ("1", "2"):
        subset = {key for key in members if v2_lines[key].get("batch") == batch}
        verdict = "not_enough" if not subset else ("jia_not_in" if all(v2_lines[key].get("strict") == NOT_IN for key in subset) and
                                                  not any(v2_lines[key].get("strict") == IN for key in subset) else
                                                  "yi_in" if any(v2_lines[key].get("strict") == IN for key in subset) else "spec_gap_only")
        emit(f"name=q2g_batch_verdict row={row} batch={batch} K={len(subset)} verdict={verdict}")
    for stream in STREAMS:
        trajectory_line(f"row-{row}", stream, "K", members, v2_lines, emit)
        trajectory_line(f"row-{row}", stream, "K_judged_in", set(judged_in), v2_lines, emit)
    emit(f"name=q2_counts row={row} " + " ".join(f"{key}={value}" for key, value in counts.items()))
    return counts


def tail_check_disagreements(label: str, v2_lines: dict, reference_lines: dict, emit, quiet: bool) -> int:
    """Q3-c：v2 的 `tail_check` 与 T_ref 不同的态数（not_evaluated 算一个值）。"""
    differing = 0
    for key, v2, reference in joined(v2_lines, reference_lines):
        if v2.get("tail_check") != reference.get("t_ref"):
            differing += 1
            if not quiet:
                emit(f"name=q3c_disagreement input={label} source={key[0]} stream={key[1]} state={key[2]} v2_tail_check={v2.get('tail_check')} "
                     f"v2_tau={v2.get('ack_rotation_tail')} v2_counter={v2.get('witness_counter')} ref_t={reference.get('t_ref')} "
                     f"ref_reason={reference.get('t_reason')} ref_tau={reference.get('tau')} ref_counter={reference.get('c_witness')} ack={v2.get('ack')}")
    return differing


def q3_row_counts(v2_lines: dict, reference_lines: dict, emit, quiet: bool = False) -> dict[str, int]:
    pairs = joined(v2_lines, reference_lines)
    members = {key for key, _, reference in pairs if reference.get("t_ref") == "false"}
    judged_in = [key for key in sorted(members) if v2_lines[key].get("strict") == IN]
    counts = {"K270": len(members), "q3a_K270_judged_in": len(judged_in),
              "K270_spec_gap": sum(v2_lines[key].get("strict") == SPEC_GAP for key in members),
              "q3c_tail_check_disagreements": tail_check_disagreements(f"row-{TAIL_ROW}", v2_lines, reference_lines, emit, quiet)}
    if quiet:
        return counts
    for key in judged_in:
        fields, reference = v2_lines[key], reference_lines[key]
        emit(f"name=q3a_state row={TAIL_ROW} stream={key[1]} state={key[2]} batch={fields.get('batch')} ack={fields.get('ack')} "
             f"tau={reference.get('tau')} c_witness={reference.get('c_witness')} v2_tail_check={fields.get('tail_check')}")
    for stream in STREAMS:
        for batch in ("1", "2"):
            subset = [key for key in members if key[1] == stream and v2_lines[key].get("batch") == batch]
            emit(f"name=q3b_size row={TAIL_ROW} stream={stream} batch={batch} K270={len(subset)} "
                 f"K270_judged_in={sum(v2_lines[key].get('strict') == IN for key in subset)} "
                 f"K270_judged_not_in={sum(v2_lines[key].get('strict') == NOT_IN for key in subset)}")
    for batch in ("1", "2"):
        subset = {key for key in members if v2_lines[key].get("batch") == batch}
        verdict = "not_enough" if not subset else ("yi_in" if any(v2_lines[key].get("strict") == IN for key in subset) else
                                                  "not_in" if all(v2_lines[key].get("strict") == NOT_IN for key in subset) else "spec_gap_only")
        emit(f"name=q3_row_batch_verdict row={TAIL_ROW} batch={batch} K270={len(subset)} verdict={verdict}")
    for stream in STREAMS:
        trajectory_line(f"row-{TAIL_ROW}", stream, "K270", members, v2_lines, emit)
        trajectory_line(f"row-{TAIL_ROW}", stream, "K270_judged_in", set(judged_in), v2_lines, emit)
    emit(f"name=q3_row_counts row={TAIL_ROW} " + " ".join(f"{key}={value}" for key, value in counts.items()))
    return counts


def legal_counts(label: str, group: str, v2_lines: dict, reference_lines: dict, emit, quiet: bool = False) -> dict[str, int]:
    """Q3-d（误红）、W 在合法态上的翻面、Q3-e（T 的路径计数）、Q3-c（合法历史上 T 与 T_ref 的分歧）。"""
    misred = [key for key, fields in sorted(v2_lines.items())
              if fields.get("strict_before_witness") == IN and fields.get("strict") == NOT_IN and "T" in fields.get("failed_predicates", "-").split(",")]
    witness_flips = [key for key, fields in sorted(v2_lines.items())
                     if fields.get("strict_before_witness") == IN and fields.get("strict") == NOT_IN and "W" in fields.get("failed_predicates", "-").split(",")]
    evaluated = [fields for fields in v2_lines.values() if fields.get("tail_check") in ("true", "false")]
    counts = {"states": len(v2_lines), "q3d_misred_T": len(misred), "legal_flip_W": len(witness_flips),
              "t_evaluated": len(evaluated), "t_true": sum(fields["tail_check"] == "true" for fields in evaluated),
              "t_false": sum(fields["tail_check"] == "false" for fields in evaluated),
              "t_not_evaluated": sum(fields.get("tail_check") == "not_evaluated" for fields in v2_lines.values()),
              "t_counter_equals_tau": sum(fields.get("witness_counter") == fields.get("ack_rotation_tail") for fields in evaluated),
              "t_counter_above_tau": sum(int(fields["witness_counter"]) > int(fields["ack_rotation_tail"]) for fields in evaluated),
              "ref_t_evaluated": sum(fields.get("t_ref") in ("true", "false") for fields in reference_lines.values()),
              "ref_counter_equals_tau": sum(fields.get("c_witness_vs_tau") == "eq" for fields in reference_lines.values()),
              "ref_w_true": sum(fields.get("w_ref") == "true" for fields in reference_lines.values()),
              "q3c_tail_check_disagreements": tail_check_disagreements(label, v2_lines, reference_lines, emit, quiet),
              "witness_disagreements": sum(v2_witness_value(v2) != reference_witness_value(reference) for _, v2, reference in joined(v2_lines, reference_lines)),
              "digest_mismatches": sum(fields.get("digest") != "match" for fields in v2_lines.values()),
              "reference_digest_mismatches": sum(fields.get("digest") != "match" for fields in reference_lines.values()),
              "reference_missing": len(set(v2_lines) - set(reference_lines))}
    if quiet:
        return counts
    for key in misred + witness_flips:
        fields = v2_lines[key]
        emit(f"name=legal_flip input={label} group={group} source={key[0]} stream={key[1]} state={key[2]} failed_predicates={fields.get('failed_predicates')} "
             f"ack={fields.get('ack')} tau={fields.get('ack_rotation_tail')} c_witness={fields.get('witness_counter')} landed={fields.get('landed')}")
    for stream in STREAMS:
        if any(key[1] == stream for key in v2_lines) and all(key[0] == "state" for key in v2_lines):
            trajectory_line(label, stream, "misred_T", set(misred), v2_lines, emit)
    emit(f"name=legal_counts input={label} group={group} " + " ".join(f"{key}={value}" for key, value in counts.items()))
    return counts


def copy_with_one_line_changed(source_path: str, copy_path: str, wanted_key: tuple, replacements: dict[str, str]) -> None:
    with open(source_path, encoding="utf-8") as handle:
        lines = handle.readlines()
    changed = 0
    for index, line in enumerate(lines):
        fields = fields_of(line)
        if line.startswith("name=state ") and (fields.get("source", "state"), fields.get("stream"), fields.get("state")) == wanted_key:
            for field, value in replacements.items():
                line = re.sub(rf"\b{field}=\S*", f"{field}={value}", line, count=1)
            lines[index] = line
            changed += 1
    assert changed == 1, f"复制件里要改的那一态命中 {changed} 次"
    with open(copy_path, "x", encoding="utf-8") as handle:
        handle.writelines(lines)


def positive_controls(rows: dict, legal: dict, legal_label: str, scratch: str, emit) -> None:
    """Q2-pc、Q3-pc：复制一份、把 K（K270）里一态的 `strict=` 改成「在」，Q2-a（Q3-a）必须恰好多 1；Q3-pc2：复制一份、把一个合法态改成
    `strict=not_in`、`failed_predicates=T`，误红必须恰好多 1（第六节；≠ 1 → V6）。"""
    for quantity, candidate_rows in (("q2_pc", WITNESS_ROWS), ("q3_pc", (TAIL_ROW,))):
        done = False
        for row in candidate_rows:
            if row not in rows or done:
                continue
            _, v2_path, reference_path = rows[row]
            v2_lines, reference_lines = keyed(v2_path, "state", {"1", "2"}), keyed(reference_path, "witness_reference", {"1", "2"})
            counter = (lambda v2, ref: q2_counts(row, v2, ref, emit, quiet=True)["q2a_K_judged_in"]) if quantity == "q2_pc" else \
                      (lambda v2, ref: q3_row_counts(v2, ref, emit, quiet=True)["q3a_K270_judged_in"])
            member_test = (lambda reference: reference_witness_value(reference) == "true") if quantity == "q2_pc" else \
                          (lambda reference: reference.get("t_ref") == "false")
            target = next((key for key, v2, reference in joined(v2_lines, reference_lines) if member_test(reference) and v2.get("strict") != IN), None)
            if target is None:
                continue
            copy_path = os.path.join(scratch, f"{quantity}-row-{row}.out")
            copy_with_one_line_changed(v2_path, copy_path, target, {"strict": IN})
            before, after = counter(v2_lines, reference_lines), counter(keyed(copy_path, "state", {"1", "2"}), reference_lines)
            emit(f"name={quantity} row={row} changed_key={':'.join(target)} before={before} after={after} added={after - before} "
                 f"exactly_one={str(after - before == 1).lower()} copy={copy_path}")
            done = True
        if not done:
            emit(f"name={quantity} changed_key=none added=0 exactly_one=false detail=no_member_to_flip")
    group, kind, batches, v2_path, reference_path = legal[legal_label]
    v2_lines = keyed(v2_path, "state", batches)
    target = next(key for key, fields in sorted(v2_lines.items()) if fields.get("strict_before_witness") == IN and fields.get("strict") == IN)
    copy_path = os.path.join(scratch, f"q3_pc2-{legal_label}.out")
    copy_with_one_line_changed(v2_path, copy_path, target, {"strict": NOT_IN, "failed_predicates": "T"})
    reference_lines = keyed(reference_path, "witness_reference", batches)
    before = legal_counts(legal_label, group, v2_lines, reference_lines, emit, quiet=True)["q3d_misred_T"]
    after = legal_counts(legal_label, group, keyed(copy_path, "state", batches), reference_lines, emit, quiet=True)["q3d_misred_T"]
    emit(f"name=q3_pc2 input={legal_label} changed_key={':'.join(target)} before={before} after={after} added={after - before} "
         f"exactly_one={str(after - before == 1).lower()} copy={copy_path}")


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑第二段：Q2、Q3 比对器")
    parser.add_argument("--row", action="append", required=True, help="<行号>:<v1 产物>:<v2 产物>:<参照产物>")
    parser.add_argument("--mutated", action="append", default=[], help="<行号>:<X21|X27>:<变异后的 v2 产物>")
    parser.add_argument("--legal", action="append", required=True, help="<输入名>:<组>:<states|images>:<批|->:<v2 产物>:<参照产物>")
    parser.add_argument("--positive-control-legal-input", required=True)
    parser.add_argument("--scratch", required=True)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    output = open(arguments.output, "x", encoding="utf-8")
    emitted = 0

    def emit(line: str) -> None:
        nonlocal emitted
        emitted += 1
        output.write(line + "\n")

    rows = {}
    for entry in arguments.row:
        row, version_one_path, v2_path, reference_path = entry.split(":", 3)
        rows[row] = (version_one_path, v2_path, reference_path)
    legal = {}
    for entry in arguments.legal:
        label, group, kind, batch_text, v2_path, reference_path = entry.split(":", 5)
        legal[label] = (group, kind, None if batch_text == "-" else set(batch_text.split(",")), v2_path, reference_path)
    emit(f"name=config rows={','.join(sorted(rows))} legal_inputs={','.join(legal)} mutated={len(arguments.mutated)}")
    for row, (version_one_path, v2_path, reference_path) in sorted(rows.items()):
        v1_lines, v2_lines = keyed(version_one_path, "state", {"1", "2"}), keyed(v2_path, "state", {"1", "2"})
        reference_lines = keyed(reference_path, "witness_reference", {"1", "2"})
        emit(f"name=b4_count row={row} v1_states={len(v1_lines)} v2_states={len(v2_lines)} reference_states={len(reference_lines)} "
             f"v2_digest_mismatches={sum(fields.get('digest') != 'match' for fields in v2_lines.values())} "
             f"reference_digest_mismatches={sum(fields.get('digest') != 'match' for fields in reference_lines.values())}")
        for batch in ("1", "2"):
            emit(f"name=row_verdict_e171_reading_three row={row} batch={batch} v1={row_red_on_e171_reading_three(v1_lines, batch)} "
                 f"v2={row_red_on_e171_reading_three(v2_lines, batch)}")
        if row in WITNESS_ROWS:
            q2_counts(row, v2_lines, reference_lines, emit)
        if row == TAIL_ROW:
            q3_row_counts(v2_lines, reference_lines, emit)
    for entry in arguments.mutated:
        row, mutation, mutated_path = entry.split(":", 2)
        mutated_lines = keyed(mutated_path, "state", {"1", "2"})
        reference_lines = keyed(rows[row][2], "witness_reference", {"1", "2"})
        if mutation == "X21":
            found = q2_counts(row, mutated_lines, reference_lines, emit, quiet=True)["q2a_K_judged_in"]
        else:
            found = q3_row_counts(mutated_lines, reference_lines, emit, quiet=True)["q3a_K270_judged_in"]
        emit(f"name=mutation_run mutation={mutation} row={row} states={len(mutated_lines)} judged_in_inside_the_class={found} "
             f"at_least_one={str(found >= 1).lower()}")
    totals: dict[str, dict[str, int]] = {}
    for label, (group, kind, batches, v2_path, reference_path) in legal.items():
        v2_lines = keyed(v2_path, "state" if kind == "states" else "image_judgement", batches, legal_images_only=kind == "images")
        reference_lines = keyed(reference_path, "witness_reference", batches, legal_images_only=kind == "images")
        counts = legal_counts(label, group, v2_lines, reference_lines, emit)
        group_totals = totals.setdefault(group, {})
        for key, value in counts.items():
            group_totals[key] = group_totals.get(key, 0) + value
    for group, counts in totals.items():
        emit(f"name=legal_group_totals group={group} " + " ".join(f"{key}={value}" for key, value in counts.items()))
    positive_controls(rows, legal, arguments.positive_control_legal_input, arguments.scratch, emit)
    emitted += 1
    output.write(f"name=done emitted={emitted}\n")
    output.close()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
