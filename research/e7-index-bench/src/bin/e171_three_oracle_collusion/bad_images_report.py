"""E171 Q2（坏镜像）的行文（登记 5.5、第六节 Q2-全放行、Q2-阴性、Q2-族覆盖、第八节几何）：四族坏镜像在 O2、O3 上的判定。

O2 那一列来自装置的 `known-bad-images` 模式（41 条已知坏镜像在流一全持久态上的 `known-bad-o2.tsv`；E170 `bad-states` 模式造的 P、N、盲测各行的
`e170-bad-rows-o2.tsv`，每行还带不打补丁的底态上 O2 的判定）；族名、行名、底态名从 E170 的 `bad-expected.tsv` 前四列取，第六列是 E170 设计员照条款推的期望
（只用来核登记 A2 与失败条款 F3，不进任何判定）。O3 那两列来自 `o3_bad_image_judge.py judge` 的 `name=image_judgement` 行（冻结的执行器判，严格与暂代各一）。
O3 严格判「规约欠一句」、出错都算弃权、不红（登记第一节读法 7）；`None` 表示没有这一行的 O3 判定（没跑）。盲测族的种子取冻结的那一版（登记 5.4），N 行附带。
"""

from __future__ import annotations

import os
from dataclasses import dataclass

from result_lines import ResultLines, field
from run_directory import STREAM_NAMES, parse_result_line, read_rows

FAMILY_POSITIVE = "P"
FAMILY_NEGATIVE = "N"
FAMILY_BLIND = "B"
FAMILY_KNOWN_BAD = "K"
FAMILIES_IN_THE_Q2_TABLE = (FAMILY_POSITIVE, FAMILY_BLIND, FAMILY_KNOWN_BAD)
EXPECTED_BASE_LABELS = ("F", "Mk", "Mk+", "Mk-")
P_ROWS_EXPECTED = 26
N_ROWS_EXPECTED = 10
KNOWN_BAD_ROWS_EXPECTED = 41
BLIND_ROWS_EXPECTED = 40
NEGATIVE_BASES_EXPECTED = 8
EXPECTED_COLUMN_OF_THE_VERDICT = 5
O3_NOT_IN = "not_in"
O3_IN = "in"
SOURCE_E170, SOURCE_KNOWN_BAD, SOURCE_BASE = "e170", "known_bad", "base"


@dataclass(frozen=True)
class BadImageRow:
    family: str
    stream: str
    row_id: int
    name: str
    base_label: str
    o2: str
    o2_items: str
    base_o2_items: str
    o3_strict: str | None = None
    o3_provisional: str | None = None
    expected_verdict: str | None = None

    def o2_is_red(self) -> bool:
        return self.o2 == "red"

    def o3_strict_is_red(self) -> bool:
        return self.o3_strict == O3_NOT_IN

    def o3_provisional_is_red(self) -> bool:
        return self.o3_provisional == O3_NOT_IN

    def quiet_on_o2_and_strict_o3(self) -> bool:
        """甲：O2 不红且 O3 严格不红（欠一句、出错、没跑都不红）。"""
        return not self.o2_is_red() and not self.o3_strict_is_red()

    def quiet_on_o2_and_provisional_o3(self) -> bool:
        return not self.o2_is_red() and not self.o3_provisional_is_red()

    def patch_kind(self) -> str:
        """P 行名 `P1-F` 的补丁种类 `P1`（Q2 几何：逐种补丁比两个底态）。"""
        return self.name.split("-", 1)[0]


@dataclass(frozen=True)
class ImageJudgement:
    strict: str
    provisional: str
    base_digest_ok: bool
    patched_digest_ok: bool


def read_known_bad_rows(known_bad_directory: str) -> list[BadImageRow]:
    """41 条已知坏镜像（流一全持久态 F，施加在 `build_pool` 走完的干净镜像上）。列：序号、目标不变量、补丁叠回之后判红的不变量、直接改坏的判红的不变量、两者相等、目标红。"""
    rows = []
    for columns in read_rows(os.path.join(known_bad_directory, "known-bad-o2.tsv")):
        index, target, patched_red = int(columns[0]), columns[1], columns[2]
        rows.append(BadImageRow(FAMILY_KNOWN_BAD, STREAM_NAMES[0], index, target, "F", "green" if patched_red == "-" else "red", patched_red, "-"))
    return rows


def read_e170_rows(known_bad_directory: str, expected_directory: str) -> list[BadImageRow]:
    """E170 坏状态各行：`e170-bad-rows-o2.tsv` 列为流名、行号、O2 红绿、O2 红的不变量、E170 池级 checker 那一列、两者相等、摘要相等、底态上 O2 判红的不变量、底态摘要；
    族名、行名、底态名、E170 设计员的期望取 `bad-expected.tsv`。"""
    names: dict[tuple[str, int], tuple[str, str, str, str]] = {}
    for stream in STREAM_NAMES:
        for columns in read_rows(os.path.join(expected_directory, stream, "bad-expected.tsv")):
            names[(stream, int(columns[0]))] = (columns[2], columns[1], columns[3], columns[EXPECTED_COLUMN_OF_THE_VERDICT])
    rows = []
    for columns in read_rows(os.path.join(known_bad_directory, "e170-bad-rows-o2.tsv")):
        stream, row_id = columns[0], int(columns[1])
        family, name, base_label, expected = names[(stream, row_id)]
        rows.append(BadImageRow(family, stream, row_id, name, base_label, columns[2], columns[3], columns[7], expected_verdict=expected))
    return rows


def read_image_judgements(path: str) -> dict[tuple[str, str, int, str], ImageJudgement]:
    """`o3_bad_image_judge.py judge` 的产物里每张镜像一行 `name=image_judgement`；键是 (来源, 流名, 行号, 底态名)，底态名只有 `base` 来源才不是 `-`。"""
    judgements = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            fields = parse_result_line(line.rstrip("\n"))
            if fields is None or fields.get("name") != "image_judgement":
                continue
            key = (fields["source"], fields["stream"], int(fields["row"]), fields["label"] if fields["source"] == SOURCE_BASE else "-")
            judgements[key] = ImageJudgement(fields["strict"], fields["provisional"], fields.get("base_digest", "match") == "match",
                                             fields.get("patched_digest", "match") == "match")
    return judgements


def attach_o3(rows: list[BadImageRow], judgements: dict[tuple[str, str, int, str], ImageJudgement] | None) -> list[BadImageRow]:
    """每张镜像并上它的 O3 两个判定（没有判定文件、或这一张没有判定行时留 None）。"""
    if judgements is None:
        return rows
    attached = []
    for row in rows:
        source = SOURCE_KNOWN_BAD if row.family == FAMILY_KNOWN_BAD else SOURCE_E170
        found = judgements.get((source, row.stream, row.row_id, "-"))
        attached.append(row if found is None else BadImageRow(row.family, row.stream, row.row_id, row.name, row.base_label, row.o2, row.o2_items,
                                                              row.base_o2_items, found.strict, found.provisional, row.expected_verdict))
    return attached


def negative_bases(rows: list[BadImageRow]) -> dict[tuple[str, str], str]:
    """不打补丁的底态（Q2-阴性）：每条流的 F、Mk、Mk⁺、Mk⁻ 各一张，取 P 行与 N 行用到的底态名；值是底态上 O2 判红的不变量（`-` 是不红）。"""
    bases: dict[tuple[str, str], str] = {}
    for row in rows:
        if row.family in (FAMILY_POSITIVE, FAMILY_NEGATIVE) and row.base_label in EXPECTED_BASE_LABELS:
            bases.setdefault((row.stream, row.base_label), row.base_o2_items)
    return bases


def o3_text(verdict: str | None) -> str:
    return "not_run" if verdict is None else verdict


def family_line(family: str, in_family: list[BadImageRow]) -> str:
    """一族的 O2、O3 红的张数与交叉：O2 单独红、O3 单独红（甲、乙）、两条都红、两条都不红。"""
    def quiet_rows(predicate) -> list[str]:
        return [f"{row.stream[:3]}:{row.row_id}" for row in in_family if predicate(row)]
    o2_red = sum(row.o2_is_red() for row in in_family)
    strict_red = sum(row.o3_strict_is_red() for row in in_family)
    provisional_red = sum(row.o3_provisional_is_red() for row in in_family)
    return (f"name=q2_family family={family} rows={len(in_family)} o2_red={o2_red} o2_quiet={len(in_family) - o2_red} "
            f"o3_strict_red={strict_red} o3_provisional_red={provisional_red} "
            f"o2_only_strict={sum(row.o2_is_red() and not row.o3_strict_is_red() for row in in_family)} "
            f"o3_only_strict={sum(row.o3_strict_is_red() and not row.o2_is_red() for row in in_family)} "
            f"both_red_strict={sum(row.o2_is_red() and row.o3_strict_is_red() for row in in_family)} "
            f"o2_only_provisional={sum(row.o2_is_red() and not row.o3_provisional_is_red() for row in in_family)} "
            f"o3_only_provisional={sum(row.o3_provisional_is_red() and not row.o2_is_red() for row in in_family)} "
            f"both_red_provisional={sum(row.o2_is_red() and row.o3_provisional_is_red() for row in in_family)} "
            f"o3_strict_spec_gap={sum(row.o3_strict == 'spec_gap' for row in in_family)} "
            f"o3_error={sum(row.o3_strict == 'error' or row.o3_provisional == 'error' for row in in_family)} "
            f"o2_quiet_rows={','.join(quiet_rows(lambda row: not row.o2_is_red())) or 'none'}")


def geometry_lines(rows: list[BadImageRow], lines: ResultLines) -> int:
    """第八节几何：P 行逐种补丁比两个底态（F 对 Mk）上 (O2 红不红、O3 严格、O3 暂代) 三项，有一项不同记「不稳定」；P9 只有 Mk⁺ 一个底态、已知坏镜像族只有流一 F 一点，各写一句。交回不稳定的种类数。"""
    unstable = 0
    positive = [row for row in rows if row.family == FAMILY_POSITIVE]
    for stream in STREAM_NAMES:
        for kind in sorted({row.patch_kind() for row in positive if row.stream == stream}):
            by_base = {row.base_label: row for row in positive if row.stream == stream and row.patch_kind() == kind}
            if "F" in by_base and "Mk" in by_base:
                first, second = by_base["F"], by_base["Mk"]
                signature = lambda row: (row.o2_is_red(), o3_text(row.o3_strict), o3_text(row.o3_provisional))
                differs = signature(first) != signature(second)
                unstable += differs
                lines.result(f"name=q2_geometry stream={stream} kind={kind} base_f={signature(first)[0]}/{signature(first)[1]}/{signature(first)[2]} "
                             f"base_mk={signature(second)[0]}/{signature(second)[1]}/{signature(second)[2]} status={'unstable' if differs else 'same_on_both_points'}")
            else:
                lines.result(f"name=q2_geometry stream={stream} kind={kind} bases={','.join(sorted(by_base))} status=one_point_only")
    lines.result(f"name=q2_geometry_known_bad rows={sum(row.family == FAMILY_KNOWN_BAD for row in rows)} status=one_point_only_stream_one_all_persisted_state")
    return unstable


def bad_image_lines(rows: list[BadImageRow], base_judgements: dict[tuple[str, str], ImageJudgement] | None, lines: ResultLines,
                    tally: dict[str, int]) -> None:
    """Q2 的全部行文。`rows` 里已经并上 O3（`attach_o3`）；`base_judgements` 是不打补丁的底态上的 O3 判定（键 (流名, 底态名)）。"""
    o3_available = any(row.o3_strict is not None for row in rows)
    for row in rows:
        lines.result(f"name=q2_row family={row.family} stream={row.stream} row={row.row_id} bad_name={field(row.name)} "
                     f"base={field(row.base_label)} o2={row.o2} o2_red={field(row.o2_items)} o3_strict={o3_text(row.o3_strict)} "
                     f"o3_provisional={o3_text(row.o3_provisional)}")
    by_family = {family: [row for row in rows if row.family == family] for family in (FAMILY_POSITIVE, FAMILY_BLIND, FAMILY_KNOWN_BAD, FAMILY_NEGATIVE)}
    for family, in_family in by_family.items():
        lines.result(family_line(family, in_family))
    table_rows = [row for family in FAMILIES_IN_THE_Q2_TABLE for row in by_family[family]]
    for reading, predicate in (("alpha", BadImageRow.quiet_on_o2_and_strict_o3), ("beta", BadImageRow.quiet_on_o2_and_provisional_o3)):
        quiet = [row for row in table_rows if predicate(row)]
        if o3_available:
            tally[f"q2_all_quiet_{reading}"] = len(quiet)
        lines.result(f"name=q2_all_quiet reading={reading} o3={'run' if o3_available else 'not_run'} table_rows={len(table_rows)} all_quiet_rows={len(quiet)} "
                     f"verdict={'not_run' if not o3_available else ('yes_a_bad_image_passes_both' if quiet else 'no_such_image')}")
        for row in quiet if o3_available else []:
            lines.result(f"name=q2_all_quiet_row reading={reading} family={row.family} stream={row.stream} row={row.row_id} bad_name={field(row.name)} "
                         f"base={field(row.base_label)} o2_red={field(row.o2_items)} o3_strict={o3_text(row.o3_strict)} "
                         f"o3_provisional={o3_text(row.o3_provisional)} expected_by_the_e170_designer={field(row.expected_verdict or '-')}")
    p_rows = by_family[FAMILY_POSITIVE]
    o3_in = [row for row in p_rows if row.o3_strict == O3_IN or row.o3_provisional == O3_IN]
    tally["f3_p_rows_judged_in_by_o3"] = len(o3_in)
    lines.result(f"name=f3_p_rows_judged_in_by_o3 rows={len(o3_in)} "
                 f"names={','.join(f'{row.stream[:3]}:{row.name}' for row in o3_in) or 'none'}")
    expected_not_in = sum(row.expected_verdict == O3_NOT_IN for row in p_rows)
    tally["a2_p_rows_not_expected_not_in"] = len(p_rows) - expected_not_in
    lines.result(f"name=a2 p_rows={len(p_rows)} expected_not_in={expected_not_in} expected_other={len(p_rows) - expected_not_in}")
    unstable = geometry_lines(rows, lines)
    tally["q2_geometry_unstable_kinds"] = unstable
    bases = negative_bases(rows)
    red_bases = {key: items for key, items in bases.items() if items != "-"}
    base_o3 = {key: (base_judgements or {}).get(key) for key in bases}
    strict_red = [key for key, found in base_o3.items() if found is not None and found.strict == O3_NOT_IN]
    provisional_red = [key for key, found in base_o3.items() if found is not None and found.provisional == O3_NOT_IN]
    tally["negative_bases_with_o2_red"] = len(red_bases)
    tally["negative_bases_with_o3_red"] = len(set(strict_red) | set(provisional_red))
    lines.result(f"name=q2_negative_bases bases={len(bases)} expected={NEGATIVE_BASES_EXPECTED} o2_red_bases={len(red_bases)} "
                 f"o3_judged_bases={sum(found is not None for found in base_o3.values())} o3_strict_red_bases={len(strict_red)} "
                 f"o3_provisional_red_bases={len(provisional_red)} "
                 f"red_base_list={','.join(f'{stream[:3]}:{label}' for (stream, label) in sorted(set(red_bases) | set(strict_red) | set(provisional_red))) or 'none'}")
    families = {family: len(by_family[family]) for family in by_family}
    lines.result(
        f"name=a9_a10_a8 p_rows={families[FAMILY_POSITIVE]} p_rows_expected={P_ROWS_EXPECTED} n_rows={families[FAMILY_NEGATIVE]} "
        f"n_rows_expected={N_ROWS_EXPECTED} blind_rows={families[FAMILY_BLIND]} blind_rows_expected={BLIND_ROWS_EXPECTED} "
        f"known_bad_rows={families[FAMILY_KNOWN_BAD]} known_bad_rows_expected={KNOWN_BAD_ROWS_EXPECTED} q2_table_rows={len(table_rows)} "
        f"q2_table_rows_expected={P_ROWS_EXPECTED + BLIND_ROWS_EXPECTED + KNOWN_BAD_ROWS_EXPECTED}")
    lines.result("name=a1 o1_column_in_the_q2_table=absent")
    tally["known_bad_rows_with_o2_quiet"] = sum(not row.o2_is_red() for row in by_family[FAMILY_KNOWN_BAD])
