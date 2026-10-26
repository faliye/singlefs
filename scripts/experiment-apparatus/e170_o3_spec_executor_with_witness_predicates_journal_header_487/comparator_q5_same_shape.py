"""E170 第 2 次重跑第二段 Q5 比对器（`research/prompts/e170-r2-prereg.md` 读法 13、14、19，12.7 的 Q5-a…Q5-c、Q5-e、Q5-h、Q5-pc2，
12.8 的 B4-487、B15、轨迹与几何 G6–G9，12.14 的 B17 的 Z4 那一半）。

只读判定行（今天这一边是 Z 族上的判定，参照那一边是 v2 在 311 冻结导出上的判定）与两边导出目录里的行名，不重算判定。
格（读法 14）：
- 对照格：(输入, 流, 批) × 严格判定值（in / not_in / spec_gap）有没有；另比落点类（version / none / outside_version_table）有没有。
  批 1、批 2 进判；批 3 附带（attached），不进 Q5-a。
- 底态格：(输入, 流, 底态 F / Mk / Mk+ / Mk-) × (严格判定, 落点 (实例代号, checkpoint_txg))。
- 行格：(输入, 流, 行名) × (严格判定, failed_predicates, 落点类)。行名取各自那一边 `bad-expected.tsv` 第 2 列；补丁描述 = 第 2–5 列
  （行名、族、底态、底态那次调用）加第 9 列（底态备注）去掉流序位置（return_point、root_operation、first_unit_write 的数）——
  这几个数是录制流里的绝对位置，读法 14 写「不含绝对偏移」；描述不同或一边没有这一行记不可比（Q5-c）。
- 已知坏镜像（读法 19）：行名 `known_bad_<序号>|<目标名>`，序号取 `known-bad-states.tsv` 第 1 列（判定行的 state=），目标名取
  `known-bad-o2.tsv` 第 2 列；两边都有才可比，一边没有进 Q5-c，今天那一边多出的行逐行列进 Q5-h。
不同的格：今天那一边把「只因格式补丁新记的欠一句」（--new-gaps）的 spec_gap 态换成它的暂代判定之后与参照相同的，记 Q5-a(b)，不进 Q5-a；
其余进 Q5-a，再分 (i)（两边的 witness_true、tail_check、failed_predicates 有一样不同；对照格比那一 (流, 批) 里出现过的值的集合）
与 (ii)（其余）。
跑法：python3 comparator_q5_same_shape.py --pair <名>:<states|images>:<judged|attached>:<族或 rows>:<今天的产物>:<参照的产物> [...]
      --today-names <名>:<导出目录>[:<已知坏镜像目录>] --reference-names <名>:<导出目录>[:<已知坏镜像目录>] [...]
      --new-gaps G23 [--e171-tree <名>,<名>] [--expected-freeze <带冻结行的产物>] [--record-paths <Q5-e 第一项产物>]
      [--anchors-b15 <名>,<名>] [--b17 <照抄核对产物>:<Z4 已知坏镜像目录>] [--positive-control-row-input <名> --scratch <草稿目录>]
      --output <新文件>
文件名以 comparator 起头，不进冻结。
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass

sys.dont_write_bytecode = True

FIELD_PATTERN = re.compile(r"(\w+)=(\S*)")
STREAM_NAMES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")
JUDGED_BATCHES = ("1", "2")
ATTACHED_BATCHES = ("3",)
STRICT_VALUES = ("in", "not_in", "spec_gap")
LANDING_CLASSES = ("version", "none", "outside_version_table")
WITNESS_FIELDS = ("witness_true", "tail_check", "failed_predicates")
FAMILIES = ("e170_control", "e171_control", "base", "p_rows", "n_rows", "blind", "known_bad")
ROW_FAMILY_BY_COLUMN = {"P": "p_rows", "P10": "p_rows", "N": "n_rows", "B": "blind"}
POSITIONAL_NOTE_KEYS = re.compile(r"\b(return_point|root_operation|first_unit_write)=\d+")
STANDARD_BASE_LABELS = ("F", "Mk", "Mk+", "Mk-")
JUDGED, ATTACHED = "judged", "attached"


def fields_of(line: str) -> dict[str, str]:
    return dict(FIELD_PATTERN.findall(line.rstrip("\n").removeprefix("E7RESULT ")))


@dataclass
class JudgementOutput:
    path: str
    states: list[dict[str, str]]
    images: list[dict[str, str]]
    versions: dict[str, set[str]]
    freeze_lines: list[str]
    freeze_checks: list[dict[str, str]]


def read_output(path: str) -> JudgementOutput:
    states, images, freeze_lines, freeze_checks = [], [], [], []
    versions: dict[str, set[str]] = {stream: set() for stream in STREAM_NAMES}
    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            line = raw.rstrip("\n").removeprefix("E7RESULT ")
            fields = fields_of(line)
            name = fields.get("name")
            if name == "state":
                states.append(fields)
            elif name == "image_judgement":
                images.append(fields)
            elif name == "version":
                versions[fields["stream"]].add(fields["projection"])
            elif name in ("freeze", "freeze_seed") and " arm=" not in line:
                freeze_lines.append(line)
            elif name == "freeze_check":
                freeze_checks.append(fields)
    return JudgementOutput(path, states, images, versions, freeze_lines, freeze_checks)


def landing_class(fields: dict[str, str], versions: set[str]) -> str:
    landed = fields.get("landed", "none")
    if landed == "none":
        return "none"
    return "version" if landed in versions else "outside_version_table"


def landing_pair(fields: dict[str, str]) -> str:
    match = re.match(r"\((\d+),(\d+),", fields.get("landed", "none"))
    return f"({match.group(1)},{match.group(2)})" if match else fields.get("landed", "none")


def strict_without_new_gaps(fields: dict[str, str], new_gaps: set[str]) -> str:
    """今天那一边：严格判定是 spec_gap 而撞上的欠一句全是格式补丁新记的那几行，换成暂代判定（Q5-a(b) 用）。"""
    gaps = {item for item in fields.get("gaps", "-").split(",") if item and item != "-"}
    if fields.get("strict") == "spec_gap" and gaps and gaps <= new_gaps:
        return fields.get("provisional", "-")
    return fields.get("strict", "-")


def read_rows(path: str) -> list[list[str]]:
    with open(path, encoding="utf-8") as handle:
        return [line.rstrip("\n").split("\t") for line in handle if line.strip() and line.split("\t")[0].isdigit()]


@dataclass(frozen=True)
class RowName:
    name: str
    family: str
    description: tuple[str, ...]


def row_names_of(dumps: str, known_bad_dir: str | None) -> dict[tuple, RowName]:
    """(source, 流, state) → 行名；判定行判完之后才读（同第二段 Q1 比对器的取法，`comparator_two_versions.row_names`）。"""
    names: dict[tuple, RowName] = {}
    for stream in STREAM_NAMES:
        for columns in read_rows(os.path.join(dumps, stream, "bad-expected.tsv")):
            notes = POSITIONAL_NOTE_KEYS.sub(lambda match: f"{match.group(1)}=#", columns[8] if len(columns) > 8 else "-")
            family = ROW_FAMILY_BY_COLUMN.get(columns[2], f"unknown:{columns[2]}")
            names[("e170", stream, columns[0])] = RowName(columns[1], family, (columns[1], columns[2], columns[3], columns[4], notes))
    if known_bad_dir:
        for columns in read_rows(os.path.join(known_bad_dir, "known-bad-o2.tsv")):
            names[("known_bad", STREAM_NAMES[0], columns[0])] = RowName(f"known_bad_{columns[0]}|{columns[1]}", "known_bad", ())
    return names


@dataclass(frozen=True)
class Cell:
    input_label: str
    role: str  # judged / attached
    family: str
    kind: str  # control / base / row / known_bad
    key: tuple
    comparable: bool
    reason: str  # 不可比的原因；可比时 "-"
    today: tuple
    reference: tuple
    today_without_new_gaps: tuple
    today_witness: tuple
    reference_witness: tuple

    @property
    def status(self) -> str:
        if not self.comparable:
            return "not_comparable"
        if self.today == self.reference:
            return "same"
        if self.today_without_new_gaps == self.reference:
            return "differs_only_by_new_gaps"
        return "differs"

    @property
    def difference_class(self) -> str:
        return "i" if self.today_witness != self.reference_witness else "ii"


def witness_of(fields: dict[str, str]) -> tuple:
    return tuple(fields.get(name, "-") for name in WITNESS_FIELDS)


def witness_sets(group: list[dict[str, str]]) -> tuple:
    return tuple(tuple(sorted({fields.get(name, "-") for fields in group})) for name in WITNESS_FIELDS)


def control_cells(label: str, role: str, family: str, today: JudgementOutput, reference: JudgementOutput, new_gaps: set[str]) -> list[Cell]:
    cells = []
    for stream in STREAM_NAMES:
        for batch in JUDGED_BATCHES + ATTACHED_BATCHES:
            today_group = [fields for fields in today.states if fields["stream"] == stream and fields["batch"] == batch]
            reference_group = [fields for fields in reference.states if fields["stream"] == stream and fields["batch"] == batch]
            cell_role = role if batch in JUDGED_BATCHES else ATTACHED
            comparable = bool(today_group) and bool(reference_group)
            reason = "-" if comparable else f"states_today={len(today_group)}_reference={len(reference_group)}"
            today_strict = {fields.get("strict") for fields in today_group}
            today_mapped = {strict_without_new_gaps(fields, new_gaps) for fields in today_group}
            reference_strict = {fields.get("strict") for fields in reference_group}
            today_classes = {landing_class(fields, today.versions[stream]) for fields in today_group}
            reference_classes = {landing_class(fields, reference.versions[stream]) for fields in reference_group}
            today_witness, reference_witness = witness_sets(today_group), witness_sets(reference_group)
            for value in STRICT_VALUES:
                cells.append(Cell(label, cell_role, family, "control", (stream, f"batch={batch}", f"strict={value}"), comparable, reason,
                                  (value in today_strict,), (value in reference_strict,), (value in today_mapped,), today_witness, reference_witness))
            for value in LANDING_CLASSES:
                cells.append(Cell(label, cell_role, family, "control", (stream, f"batch={batch}", f"landing={value}"), comparable, reason,
                                  (value in today_classes,), (value in reference_classes,), (value in today_classes,), today_witness, reference_witness))
    return cells


def image_cells(label: str, role: str, today: JudgementOutput, reference: JudgementOutput, today_names: dict, reference_names: dict,
                new_gaps: set[str]) -> list[Cell]:
    cells = []
    today_bases = {(fields["stream"], fields["label"]): fields for fields in today.images if fields["source"] == "base"}
    reference_bases = {(fields["stream"], fields["label"]): fields for fields in reference.images if fields["source"] == "base"}
    for key in sorted(set(today_bases) | set(reference_bases)):
        one, two = today_bases.get(key), reference_bases.get(key)
        comparable = one is not None and two is not None
        today_value = (one.get("strict"), landing_pair(one)) if one else ()
        cells.append(Cell(label, role, "base", "base", key, comparable, "-" if comparable else "one_side_has_no_such_base",
                          today_value, (two.get("strict"), landing_pair(two)) if two else (),
                          (strict_without_new_gaps(one, new_gaps), landing_pair(one)) if one else (),
                          witness_of(one) if one else (), witness_of(two) if two else ()))

    def by_row_name(output: JudgementOutput, names: dict) -> dict[tuple, tuple[RowName, dict[str, str]]]:
        rows = {}
        for fields in output.images:
            if fields["source"] in ("e170", "known_bad"):
                name = names.get((fields["source"], fields["stream"], fields["state"]))
                assert name is not None, f"{output.path}：{fields['source']} {fields['stream']} {fields['state']} 在导出目录里没有行名"
                rows[(fields["source"], fields["stream"], name.name)] = (name, fields)
        return rows

    today_rows, reference_rows = by_row_name(today, today_names), by_row_name(reference, reference_names)
    for key in sorted(set(today_rows) | set(reference_rows)):
        one, two = today_rows.get(key), reference_rows.get(key)
        family = (one or two)[0].family
        if one is None or two is None:
            comparable, reason = False, "one_side_has_no_such_row"
        elif one[0].description != two[0].description:
            comparable, reason = False, "patch_description_differs"
        else:
            comparable, reason = True, "-"

        def value_of(entry, output, mapped=False):
            if entry is None:
                return ()
            fields = entry[1]
            strict = strict_without_new_gaps(fields, new_gaps) if mapped else fields.get("strict")
            return strict, fields.get("failed_predicates", "-"), landing_class(fields, output.versions[fields["stream"]])

        cells.append(Cell(label, role, family, "known_bad" if key[0] == "known_bad" else "row", key, comparable, reason,
                          value_of(one, today), value_of(two, reference), value_of(one, today, mapped=True),
                          witness_of(one[1]) if one else (), witness_of(two[1]) if two else ()))
    return cells


def segment_trajectory(today: JudgementOutput, reference: JudgementOutput, new_gaps: set[str]) -> dict[str, list[tuple[int, int]]]:
    """轨迹（12.8）：批 1、批 2 的态按 (流, 段) 分组，逐段比严格判定值与落点类有没有；交回 {流: [(段, 不同的格数)]}。"""
    trajectory = {}
    for stream in STREAM_NAMES:
        def grouped(output, mapped):
            groups: dict[int, tuple[set, set]] = {}
            for fields in output.states:
                if fields["stream"] == stream and fields["batch"] in JUDGED_BATCHES:
                    strict = strict_without_new_gaps(fields, new_gaps) if mapped else fields.get("strict")
                    entry = groups.setdefault(int(fields["segment"]), (set(), set()))
                    entry[0].add(strict)
                    entry[1].add(landing_class(fields, output.versions[stream]))
            return groups
        today_groups, reference_groups = grouped(today, True), grouped(reference, False)
        rows = []
        for segment in sorted(set(today_groups) | set(reference_groups)):
            one, two = today_groups.get(segment, (set(), set())), reference_groups.get(segment, (set(), set()))
            differing = sum((value in one[0]) != (value in two[0]) for value in STRICT_VALUES)
            differing += sum((value in one[1]) != (value in two[1]) for value in LANDING_CLASSES)
            rows.append((segment, differing))
        trajectory[stream] = rows
    return trajectory


def b4_487(label: str, kind: str, output: JudgementOutput, dumps: str, known_bad_dir: str | None, emit) -> int:
    """B4-487（12.8）：态行数等于导出目录的 states.tsv 按批行数、bad-states.tsv / known-bad-states.tsv 行数、底态 4 张每流；
    另数摘要不符与判定出错的行（V4、V11）。交回不符的格数。"""
    failures = 0
    if kind == "states":
        for stream in STREAM_NAMES:
            expected: dict[str, int] = {}
            for columns in read_rows(os.path.join(dumps, stream, "states.tsv")):
                expected[columns[3]] = expected.get(columns[3], 0) + 1
            for batch in sorted({fields["batch"] for fields in output.states if fields["stream"] == stream}):
                got = sum(1 for fields in output.states if fields["stream"] == stream and fields["batch"] == batch)
                holds = got == expected.get(batch, -1)
                failures += 0 if holds else 1
                emit(f"name=b4_487 input={label} stream={stream} batch={batch} state_lines={got} states_tsv={expected.get(batch, 0)} holds={str(holds).lower()}")
        lines = output.states
    else:
        for stream in STREAM_NAMES:
            for source, expected in (("e170", len(read_rows(os.path.join(dumps, stream, "bad-states.tsv")))), ("base", len(STANDARD_BASE_LABELS))):
                got = sum(1 for fields in output.images if fields["stream"] == stream and fields["source"] == source)
                failures += 0 if got == expected else 1
                emit(f"name=b4_487 input={label} stream={stream} source={source} lines={got} expected={expected} holds={str(got == expected).lower()}")
        if known_bad_dir:
            expected = len(read_rows(os.path.join(known_bad_dir, "known-bad-states.tsv")))
            got = sum(1 for fields in output.images if fields["source"] == "known_bad")
            failures += 0 if got == expected else 1
            emit(f"name=b4_487 input={label} source=known_bad lines={got} expected={expected} holds={str(got == expected).lower()}")
        lines = output.images
    mismatches = sum(1 for fields in lines if fields.get("digest") != "match" or fields.get("base_digest", "match") != "match")
    errors = sum(1 for fields in lines if fields.get("strict") == "error")
    failures += int(mismatches > 0) + int(errors > 0)
    emit(f"name=b4_487_integrity input={label} lines={len(lines)} digest_or_base_digest_mismatches={mismatches} error_lines={errors}")
    return failures


def b15(label: str, output: JudgementOutput, names: dict, emit) -> list[tuple[str, bool]]:
    """B15（12.8）：P8 八格 v2-487 严格 not_in、failed_predicates=W；流一 P8-F witness_counter=3、witness_compared=2（同 B5）。"""
    results = []
    for fields in output.images:
        name = names.get((fields["source"], fields["stream"], fields["state"])) if fields["source"] == "e170" else None
        if name is None or name.name not in ("P8-F", "P8-Mk"):
            continue
        holds = fields.get("strict") == "not_in" and fields.get("failed_predicates") == "W"
        if fields["stream"] == STREAM_NAMES[0] and name.name == "P8-F":
            holds = holds and fields.get("witness_counter") == "3" and fields.get("witness_compared") == "2"
        results.append((name.name.split("-")[1], holds))
        emit(f"name=b15 input={label} stream={fields['stream']} row={name.name} holds={str(holds).lower()} strict={fields.get('strict')} "
             f"failed_predicates={fields.get('failed_predicates')} witness_counter={fields.get('witness_counter')} "
             f"witness_branch={fields.get('witness_branch')} witness_compared={fields.get('witness_compared')}")
    return results


def b17(recopy_product: str, known_bad_dir: str, emit) -> bool:
    """B17 的 Z4 那一半（12.14）：Z4 的 known-bad-states.tsv 行数、known-bad-o2.tsv 第 2 列按序等于照抄核对数出的目标名。"""
    with open(recopy_product, encoding="utf-8") as handle:
        lines = [line.rstrip("\n") for line in handle]
    # `name=known_bad_entry_name index=… name=…` 一行里有两个 name=：取行首认行、取最后一个 name= 的值当目标名
    names = [re.findall(r"\bname=(\S*)", line)[-1] for line in lines if line.startswith("name=known_bad_entry_name ")]
    source = next(fields_of(line)["sha256"] for line in lines if line.startswith("name=known_bad_source "))
    states = read_rows(os.path.join(known_bad_dir, "known-bad-states.tsv"))
    targets = [columns[1] for columns in read_rows(os.path.join(known_bad_dir, "known-bad-o2.tsv"))]
    holds = len(states) == len(names) == len(targets) and targets == names
    emit(f"name=b17 known_bad_states_rows={len(states)} known_bad_o2_rows={len(targets)} entry_names_from_the_recopy_check={len(names)} "
         f"target_column_equals_in_order={str(targets == names).lower()} test_file_sha256_at_recopy={source} holds={str(holds).lower()}")
    return holds


def build_cells(pairs, outputs, today_names, reference_names, new_gaps) -> list[Cell]:
    cells = []
    for label, kind, role, family, today_path, reference_path in pairs:
        today, reference = outputs[today_path], outputs[reference_path]
        if kind == "states":
            cells += control_cells(label, role, family, today, reference, new_gaps)
        else:
            cells += image_cells(label, role, today, reference, today_names[label], reference_names[label], new_gaps)
    return cells


def stream_of(cell: Cell) -> str:
    """对照格与底态格的键以流起头；行格与已知坏镜像的键是 (source, 流, 行名)。"""
    return cell.key[1] if cell.kind in ("row", "known_bad") else cell.key[0]


def q5a_of(cells: list[Cell]) -> list[Cell]:
    return [cell for cell in cells if cell.role == JUDGED and cell.status == "differs"]


def cell_text(cell: Cell) -> str:
    return (f"input={cell.input_label} role={cell.role} family={cell.family} kind={cell.kind} key={'|'.join(map(str, cell.key))} "
            f"status={cell.status} class={cell.difference_class if cell.status.startswith('differs') else '-'} reason={cell.reason} "
            f"today={','.join(map(str, cell.today)) or '-'} reference={','.join(map(str, cell.reference)) or '-'} "
            f"today_witness={'/'.join(map(str, cell.today_witness)).replace(' ', '') or '-'} "
            f"reference_witness={'/'.join(map(str, cell.reference_witness)).replace(' ', '') or '-'}")


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑 Q5 比对器")
    parser.add_argument("--pair", action="append", required=True)
    parser.add_argument("--today-names", action="append", default=[])
    parser.add_argument("--reference-names", action="append", default=[])
    parser.add_argument("--new-gaps", default="")
    parser.add_argument("--e171-tree", default="")
    parser.add_argument("--expected-freeze")
    parser.add_argument("--record-paths")
    parser.add_argument("--anchors-b15", default="")
    parser.add_argument("--b17")
    parser.add_argument("--positive-control-row-input")
    parser.add_argument("--scratch")
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    output_file = open(arguments.output, "x", encoding="utf-8")
    emitted = 0

    def emit(line: str) -> None:
        nonlocal emitted
        emitted += 1
        output_file.write(line + "\n")

    new_gaps = {item for item in arguments.new_gaps.split(",") if item}
    pairs = [tuple(specification.split(":", 5)) for specification in arguments.pair]
    split_names = lambda specification: (specification.split(":")[0], specification.split(":")[1:])
    today_dirs = dict(split_names(item) for item in arguments.today_names)
    reference_dirs = dict(split_names(item) for item in arguments.reference_names)
    image_labels = {pair[0] for pair in pairs if pair[1] == "images"}  # 对照格的导出目录没有 bad-expected.tsv，只给 B4-487 用
    today_names = {label: row_names_of(directories[0], directories[1] if len(directories) > 1 else None)
                   for label, directories in today_dirs.items() if label in image_labels}
    reference_names = {label: row_names_of(directories[0], directories[1] if len(directories) > 1 else None)
                       for label, directories in reference_dirs.items() if label in image_labels}
    outputs = {path: read_output(path) for pair in pairs for path in pair[4:6]}
    emit(f"name=config pairs={len(pairs)} new_gaps={','.join(sorted(new_gaps)) or '-'} e171_tree={arguments.e171_tree or '-'} "
         f"judged_batches={','.join(JUDGED_BATCHES)} attached_batches={','.join(ATTACHED_BATCHES)}")
    for label, kind, role, family, today_path, reference_path in pairs:
        emit(f"name=pair input={label} kind={kind} role={role} family={family} today={os.path.basename(today_path)} reference={os.path.basename(reference_path)}")

    expected_freeze = read_output(arguments.expected_freeze).freeze_lines if arguments.expected_freeze else []
    integrity_failures = 0
    for path in sorted({pair[4] for pair in pairs}):
        output = outputs[path]
        if output.freeze_lines:
            identical = output.freeze_lines == expected_freeze and bool(expected_freeze)
            emit(f"name=freeze_check_today output={os.path.basename(path)} source=executor_main_freeze_lines identical={str(identical).lower()} lines={len(output.freeze_lines)}")
        for check in output.freeze_checks:
            emit(f"name=freeze_check_today output={os.path.basename(path)} source=judge_{check.get('when')} identical={check.get('identical')}")
        integrity_failures += sum(1 for check in output.freeze_checks if check.get("identical") != "true")
        integrity_failures += int(bool(output.freeze_lines) and output.freeze_lines != expected_freeze)
    seen_today = set()
    for label, kind, role, family, today_path, reference_path in pairs:
        if today_path not in seen_today and label in today_dirs:
            seen_today.add(today_path)
            directories = today_dirs[label]
            integrity_failures += b4_487(label, kind, outputs[today_path], directories[0], directories[1] if len(directories) > 1 else None, emit)
    b15_results = []
    for label in [item for item in arguments.anchors_b15.split(",") if item]:
        today_path = next(pair[4] for pair in pairs if pair[0] == label)
        b15_results += b15(label, outputs[today_path], today_names[label], emit)
    b17_holds = b17(*arguments.b17.split(":"), emit) if arguments.b17 else None

    cells = build_cells(pairs, outputs, today_names, reference_names, new_gaps)
    for cell in cells:
        if cell.status != "same":
            emit("name=q5_cell " + cell_text(cell))
    for cell in cells:
        if cell.kind == "known_bad" and cell.reason == "one_side_has_no_such_row" and cell.today:
            today_path = next(pair[4] for pair in pairs if pair[0] == cell.input_label)
            fields = next(item for item in outputs[today_path].images if item["source"] == "known_bad"
                          and today_names[cell.input_label][("known_bad", item["stream"], item["state"])].name == cell.key[2])
            emit(f"name=q5h_row input={cell.input_label} row={cell.key[2]} strict={fields.get('strict')} failed_predicates={fields.get('failed_predicates')} "
                 f"landing_class={cell.today[2]} landed={fields.get('landed')} witness_true={fields.get('witness_true')} tail_check={fields.get('tail_check')}")
    e171_tree = {item for item in arguments.e171_tree.split(",") if item}
    for label in [pair[0] for pair in pairs]:
        group = [cell for cell in cells if cell.input_label == label]
        emit(f"name=q5a_counts input={label} cells={len(group)} judged={sum(cell.role == JUDGED for cell in group)} "
             f"comparable_judged={sum(cell.role == JUDGED and cell.comparable for cell in group)} "
             + " ".join(f"{status}={sum(cell.role == JUDGED and cell.status == status for cell in group)}"
                        for status in ("same", "differs", "differs_only_by_new_gaps", "not_comparable"))
             + f" class_i={sum(cell.role == JUDGED and cell.status == 'differs' and cell.difference_class == 'i' for cell in group)}"
             + f" attached_differs={sum(cell.role == ATTACHED and cell.status == 'differs' for cell in group)}")
    for family in FAMILIES:
        group = [cell for cell in cells if cell.family == family and cell.role == JUDGED]
        emit(f"name=q5b_family family={family} comparable={sum(cell.comparable for cell in group)} "
             f"differs={sum(cell.status == 'differs' for cell in group)} not_comparable={sum(not cell.comparable for cell in group)}")
    for label, kind, role, family, today_path, reference_path in pairs:
        if kind == "states" and role == JUDGED:
            for stream, rows in segment_trajectory(outputs[today_path], outputs[reference_path], new_gaps).items():
                values = [differing for _, differing in rows]
                emit(f"name=q5_trajectory input={label} stream={stream} segments={len(rows)} peak={max(values, default=0)} "
                     f"positive_segments={sum(value > 0 for value in values)} last_segment_value={values[-1] if values else '-'}")
    judged_differs = q5a_of(cells)
    points = {"G6": [(stream, sum(stream_of(cell) == stream for cell in judged_differs)) for stream in STREAM_NAMES],
              "G7": [(f"batch={batch}", sum(cell.kind == "control" and cell.key[1] == f"batch={batch}" for cell in judged_differs))
                     for batch in JUDGED_BATCHES],
              "G8": [(pair[0], sum(cell.input_label == pair[0] for cell in judged_differs)) for pair in pairs if pair[1] == "states" and pair[2] == JUDGED]}
    if b15_results:
        points["G9"] = [(base, sum(1 for label, holds in b15_results if label == base and not holds)) for base in ("F", "Mk")]
    for point, values in points.items():
        stable = len({value > 0 for _, value in values}) <= 1
        emit(f"name=q5_geometry point={point} " + " ".join(f"{name}={value}" for name, value in values) + f" stable={str(stable).lower()}")
    attached_batch_three = sum(1 for cell in cells if cell.role == ATTACHED and cell.kind == "control" and cell.key[1] == "batch=3" and cell.status == "differs")
    emit(f"name=q5_geometry point=G7_batch_3_attached differs={attached_batch_three}")

    q5e_landed_by_replay = q5e_last_record = 0
    for path in sorted({pair[4] for pair in pairs}):
        lines = outputs[path].states + [fields for fields in outputs[path].images]
        for fields in lines:
            chosen = re.match(r"\((\d+),(\d+)\)@", fields.get("chosen_root", "none"))
            if chosen and fields.get("applied", "0") != "0" and landing_pair(fields) != f"({chosen.group(1)},{chosen.group(2)})":
                q5e_landed_by_replay += 1
            q5e_last_record += fields.get("witness_branch") == "last_record"
    q5e_every_device = "not_given"
    if arguments.record_paths:
        with open(arguments.record_paths, encoding="utf-8") as handle:
            q5e_every_device = sum(int(fields["every_device_has_a_self_verified_record"]) for fields in map(fields_of, handle)
                                   if fields.get("name") == "q5e_records_on_every_device")
    emit(f"name=q5e every_device_has_a_self_verified_record={q5e_every_device} landed_by_replay_above_the_chosen_root={q5e_landed_by_replay} "
         f"witness_branch_last_record={q5e_last_record}")

    positive_control = "not_run"
    if arguments.positive_control_row_input:
        label = arguments.positive_control_row_input
        candidates = [cell for cell in cells if cell.input_label == label and cell.kind == "row" and cell.role == JUDGED
                      and cell.status == "same" and cell.today and cell.today[0] in ("in", "not_in")]
        chosen_cell = candidates[0]
        pair = next(item for item in pairs if item[0] == label)
        state = next(key[2] for key, name in today_names[label].items() if key[0] == chosen_cell.key[0] and key[1] == chosen_cell.key[1]
                     and name.name == chosen_cell.key[2])
        flipped = "not_in" if chosen_cell.today[0] == "in" else "in"
        copy_path = os.path.join(arguments.scratch, "q5-positive-control-" + os.path.basename(pair[4]))
        marker = f" source={chosen_cell.key[0]} "
        changed = 0
        with open(pair[4], encoding="utf-8") as source_handle, open(copy_path, "x", encoding="utf-8") as copy_handle:
            for line in source_handle:
                if (line.startswith("name=image_judgement ") and marker in line and f" stream={chosen_cell.key[1]} state={state} " in line):
                    line = line.replace(f" strict={chosen_cell.today[0]} ", f" strict={flipped} ", 1)
                    changed += 1
                copy_handle.write(line)
        assert changed == 1, f"阳性对照改了 {changed} 行"
        outputs_after = dict(outputs)
        outputs_after[copy_path] = read_output(copy_path)
        pairs_after = [item if item[0] != label else item[:4] + (copy_path, item[5]) for item in pairs]
        before, after = len(judged_differs), len(q5a_of(build_cells(pairs_after, outputs_after, today_names, reference_names, new_gaps)))
        positive_control = str(after - before == 1).lower()
        emit(f"name=q5_pc2 input={label} changed_key={'|'.join(chosen_cell.key)} state={state} from={chosen_cell.today[0]} to={flipped} "
             f"q5a_before={before} q5a_after={after} exactly_one={positive_control} copy={copy_path}")

    families_without = [family for family in FAMILIES
                        if not any(cell.family == family and cell.role == JUDGED and cell.comparable for cell in cells)]
    by_new_gaps = sum(cell.role == JUDGED and cell.status == "differs_only_by_new_gaps" for cell in cells)
    class_i = sum(cell.difference_class == "i" for cell in judged_differs)
    if judged_differs:
        verdict = "yi"
    elif families_without:
        verdict = "not_decidable"
    else:
        verdict = "jia"
    subset = [cell for cell in judged_differs if cell.input_label in e171_tree]
    emit(f"name=verdict q5a={len(judged_differs)} q5a_class_i={class_i} q5a_class_ii={len(judged_differs) - class_i} "
         f"q5a_only_new_gaps={by_new_gaps} families_without_comparable_cells={','.join(families_without) or 'none'} q5_verdict={verdict} "
         f"q5a_all_class_ii={str(bool(judged_differs) and class_i == 0).lower()} e171_tree_subset_q5a={len(subset)} "
         f"b15_failures={sum(not holds for _, holds in b15_results)} b15_cells={len(b15_results)} "
         f"b17_holds={'not_run' if b17_holds is None else str(b17_holds).lower()} integrity_failures={integrity_failures} "
         f"q5_pc2_exactly_one={positive_control}")
    emitted += 1
    output_file.write(f"name=done emitted={emitted}\n")
    output_file.close()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
