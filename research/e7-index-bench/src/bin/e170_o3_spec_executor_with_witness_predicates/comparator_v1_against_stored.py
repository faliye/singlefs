"""E170 第 2 次重跑第二段：v1 这次现跑与它入库的产物逐态比（`research/prompts/e170-r2-prereg.md` 5.4 末条、第六节 Q1-d、第十一节 V9）。

比两边都有的字段（`time_ns`、`worker` 除外）；行格式不同的按字段名对齐：v1 的 executor_main 判定行与入库的同格式逐字段比；
坏状态与坏镜像那几份，入库产物的 `patched_digest=` 对这次的 `digest=`（都是叠补丁之后的镜像摘要与导出写的比没比上）。
没有入库对照的态（I3 那 8 张不打补丁的底态：第 1 次重跑第三段只判了 P、N、盲测各行）照数报，不算不同。

跑法：python3 comparator_v1_against_stored.py --pair <输入名>:<kind>:<这次的产物>:<入库的产物> [...] --output <新文件>
  kind = states（两边都是 executor_main 的 name=state 行）、bad_states（入库的是 comparator_q3_bad_states 的 bad_judgement 行）、
         images（入库的是 E171 o3_bad_image_judge 的 image_judgement 行）。
"""

from __future__ import annotations

import argparse
import re
import sys

sys.dont_write_bytecode = True

FIELD_PATTERN = re.compile(r"(\w+)=(\S*)")
NOT_COMPARED = {"time_ns", "worker", "name"}
FIELD_RENAMES_OF_THE_STORED_SIDE = {"patched_digest": "digest"}


def fields_of(line: str) -> dict[str, str]:
    return dict(FIELD_PATTERN.findall(line.rstrip("\n").removeprefix("E7RESULT ")))


def lines_named(path: str, name: str) -> list[dict[str, str]]:
    with open(path, encoding="utf-8") as handle:
        return [fields_of(line) for line in handle if line.removeprefix("E7RESULT ").startswith(f"name={name} ")]


def keyed_now(path: str, kind: str) -> dict[tuple, dict[str, str]]:
    if kind == "states":
        return {(fields["stream"], fields["state"]): fields for fields in lines_named(path, "state")}
    return {(fields["source"], fields["stream"], fields["state"]): fields for fields in lines_named(path, "image_judgement")}


def keyed_stored(path: str, kind: str) -> dict[tuple, dict[str, str]]:
    if kind == "states":
        return {(fields["stream"], fields["state"]): fields for fields in lines_named(path, "state")}
    if kind == "bad_states":
        return {("e170", fields["stream"], fields["row"]): {FIELD_RENAMES_OF_THE_STORED_SIDE.get(key, key): value for key, value in fields.items()}
                for fields in lines_named(path, "bad_judgement")}
    return {(fields["source"], fields["stream"], fields["row"]): {FIELD_RENAMES_OF_THE_STORED_SIDE.get(key, key): value for key, value in fields.items()}
            for fields in lines_named(path, "image_judgement")}


def compare_pair(label: str, kind: str, now_path: str, stored_path: str, emit) -> int:
    now = keyed_now(now_path, kind)
    stored = keyed_stored(stored_path, kind)
    compared = differing = 0
    for key in sorted(set(now) & set(stored)):
        now_fields, stored_fields = now[key], stored[key]
        common = sorted((set(now_fields) & set(stored_fields)) - NOT_COMPARED - {"row", "state", "source", "stream"})
        different = [field for field in common if now_fields[field] != stored_fields[field]]
        compared += 1
        if different:
            differing += 1
            detail = ";".join(f"{field}:{stored_fields[field]}->{now_fields[field]}" for field in different)
            emit(f"name=v1_against_stored_difference input={label} key={':'.join(key)} fields={detail}")
    only_now = sorted(set(now) - set(stored))
    only_stored = sorted(set(stored) - set(now))
    common_example = sorted((set(next(iter(now.values()), {})) & set(next(iter(stored.values()), {}))) - NOT_COMPARED)
    emit(f"name=v1_against_stored input={label} kind={kind} now_states={len(now)} stored_states={len(stored)} compared={compared} "
         f"differing_states={differing} only_in_this_run={len(only_now)} only_in_the_stored_product={len(only_stored)} "
         f"only_in_this_run_sources={','.join(sorted({key[0] for key in only_now})) if kind != 'states' and only_now else '-'} "
         f"compared_fields={','.join(common_example)}")
    return differing + len(only_stored)


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="v1 现跑与入库产物逐态比（V9）")
    parser.add_argument("--pair", action="append", required=True, help="<输入名>:<kind>:<这次的产物>:<入库的产物>")
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    output = open(arguments.output, "x", encoding="utf-8")
    emitted = 0

    def emit(line: str) -> None:
        nonlocal emitted
        emitted += 1
        output.write(line + "\n")

    total = 0
    for pair in arguments.pair:
        label, kind, now_path, stored_path = pair.split(":", 3)
        emit(f"name=pair input={label} kind={kind} this_run={now_path} stored={stored_path}")
        total += compare_pair(label, kind, now_path, stored_path, emit)
    emit(f"name=verdict q1d_differing_or_missing_states={total} v9_triggered={str(total != 0).lower()}")
    emitted += 1
    output.write(f"name=done emitted={emitted}\n")
    output.close()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
