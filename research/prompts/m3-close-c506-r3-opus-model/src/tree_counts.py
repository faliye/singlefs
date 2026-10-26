#!/usr/bin/env python3
"""m3-close-c506-r3 攻方腿：在冻结副本上数 X2 / X3 要的落点与计数（只读，不编译）。

用法：tree_counts.py <冻结副本根>（里面要有 crates/、.claude/kb/、scripts/experiment-apparatus/e50_ring_slots.rs、
.claude/gate.d/crash-point-replay-items.tsv）。每一行 name=… 是一格，报告逐格引。
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(sys.argv[1])
D22_ITEM_ONE = re.compile(r"D22（单元原子性怎么合成） 已定项 1(?![0-9])")


def rust_and_wgsl():
    for path in sorted((ROOT / "crates").rglob("*")):
        if path.suffix in (".rs", ".wgsl") and "/target/" not in str(path):
            yield path


def rel(path):
    return str(path.relative_to(ROOT))


def lines_matching(pattern):
    hits = []
    for path in rust_and_wgsl():
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if re.search(pattern, line):
                hits.append((rel(path), number, line))
    return hits


def main():
    associated = lines_matching(r"RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM")
    docs = [h for h in associated if h[2].lstrip().startswith("//")]
    print(f"name=at_make_filesystem_associated lines={len(associated)} doc_comment_lines={len(docs)} "
          f"code_lines={len(associated) - len(docs)} files={len({h[0] for h in associated})}")
    direct = lines_matching(r"ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM")
    print(f"name=at_make_filesystem_format_constant lines={len(direct)} files={len({h[0] for h in direct})}")
    copies = lines_matching(r"const [A-Z_]*SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 =")
    for path, number, line in copies:
        print(f"name=local_copy_of_eight at={path}:{number} text={line.strip()}")
    for path, number, line in lines_matching(D22_ITEM_ONE.pattern):
        print(f"name=cites_d22_item_one at={path}:{number}")
    lib = ROOT / "crates/singlefs-format/src/lib.rs"
    lib_text = lib.read_text(encoding="utf-8")
    for constant in ("ROOT_RING_CHUNK_BYTES", "ROOT_RING_PRIME_STEP", "ROOT_RING_SLOTS_PER_REGION_MINIMUM",
                     "ROOT_RING_SLOTS_PER_REGION_MAXIMUM"):
        print(f"name=format_const_marker_in_lib constant={constant} "
              f"markers={lib_text.count('format-const: ' + constant)}")
    layout = (ROOT / ".claude/kb/layout/01-first-txn.md").read_text(encoding="utf-8").splitlines()
    for number, line in enumerate(layout, 1):
        if line.startswith("| 几何 | 素数步长 P ") or line.startswith("| 几何 | 根环 chunk ") \
                or line.startswith("| 几何 | 每区槽数 S "):
            print(f"name=layout_row at=.claude/kb/layout/01-first-txn.md:{number} "
                  f"has_format_const_marker={'format-const:' in line} text={line}")
    for path, number, line in lines_matching(r"const SLOTS_PER_REGION_OF_THE_[A-Z_]+_POOLS: u64 ="):
        print(f"name=edge_pool_constant at={path}:{number} text={line.strip()}")
    users = sorted({h[0] for h in lines_matching(r"common_root_ring_holes|SLOTS_PER_REGION_OF_THE_(RING_HOLE|BIRTH|CLAIM)_POOLS")})
    print(f"name=edge_pool_users files={len(users)} checker_tier_files="
          f"{[u for u in users if 'singlefs-checker-tier' in u]}")
    items = (ROOT / ".claude/gate.d/crash-point-replay-items.tsv").read_text(encoding="utf-8").splitlines()
    for number, line in enumerate(items, 1):
        if line.startswith("instance-table-sweep\t"):
            print(f"name=crash_replay_item at=.claude/gate.d/crash-point-replay-items.tsv:{number} "
                  f"text={line.split(chr(9))[0]}")
    c568 = lines_matching(r"fn one_mebibyte_minimum_input_output_size_is_refused_before_any_write_at_")
    print(f"name=c568_mkfs_cases_passing_S count={len(c568)}")
    for path, number, line in lines_matching(r"for declared in \[4u64, 8, 16\]|\(4u64, 16_384u64\)"):
        if "singlefs-harness/tests/" in path:
            print(f"name=mkfs_builds_edge_S at={path}:{number} text={line.strip()}")
    for path in rust_and_wgsl():
        if "singlefs-checker-tier/src/bin/" not in rel(path):
            continue
        text = path.read_text(encoding="utf-8").splitlines()
        for number, line in enumerate(text, 1):
            if line.strip().startswith("root_ring_slots_per_region:"):
                window = " ".join(x.strip() for x in text[number - 1:number + 5])
                said = re.search(r'expect\("([^"]*)"\)', window)
                print(f"name=experiment_mkfs_parameter_S at={rel(path)}:{number} "
                      f"expect={said.group(1) if said else None}")
    for path, number, line in lines_matching(r"const ROOT_RING_(REGION_STRIDE|BASE_BYTES): u32"):
        print(f"name=gpu_literal at={path}:{number} text={line.strip()}")
    prior = (ROOT / ".claude/kb/prior-art.md").read_text(encoding="utf-8").splitlines()
    for number, line in enumerate(prior, 1):
        if "区域多大不是格式常量" in line or "要先把「区域大小固定」写成条款" in line or "下界 1、上界由" in line:
            print(f"name=prior_art_line at=.claude/kb/prior-art.md:{number}")
    e50 = (ROOT / "scripts/experiment-apparatus/e50_ring_slots.rs").read_text(encoding="utf-8")
    scanned = [int(x) for x in re.search(r"TOTAL_SLOT_COUNTS_SCANNED: \[u64; \d+\] = \[([^\]]*)\]", e50).group(1).split(",")]
    for label, blocks in (("coarse", 13), ("scattered", 34)):
        first_scanned = next(n for n in scanned if n * blocks >= 1000)
        first_true = next(n for n in range(1, 2000) if n * blocks >= 1000)
        print(f"name=e50_first_crossing load={label} blocks_per_fsync={blocks} scanned={scanned} "
              f"first_in_scan={first_scanned} first_over_all_n={first_true} "
              f"largest_S_below_line_R3={max(s for s in range(1, 400) if blocks * 3 * s < 1000)} "
              f"smallest_S_on_line_R3={min(s for s in range(1, 400) if blocks * 3 * s >= 1000)}")
    e50_page = (ROOT / ".claude/kb/experiments/50-根环槽数的上下界.md").read_text(encoding="utf-8").splitlines()
    for number, line in enumerate(e50_page, 1):
        if "得到 **256 槽**" in line or "S 在 1..16 这个区间里怎么取都不碰任何一条边" in line:
            print(f"name=e50_page_line at=.claude/kb/experiments/50-根环槽数的上下界.md:{number}")
    d22 = (ROOT / ".claude/kb/decisions/22-单元原子性怎么合成.md").read_text(encoding="utf-8").splitlines()
    old = "**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**；4..16 之间取值不碰任何一条边。"
    for number, line in enumerate(d22, 1):
        if old in line:
            after = line.split(old, 1)[1]
            print(f"name=d22_s_cell at=.claude/kb/decisions/22-单元原子性怎么合成.md:{number} replaced_span_found=1 "
                  f"kept_after_span_has_not_a_format_decision={'具体取多少不是格式决策' in after} "
                  f"kept_after_span_has_load_shape={'按 D25（目标负载优先级） 的负载形态定' in after}")
    inv = (ROOT / "crates/singlefs-checker/invariants/02-根环.md").read_text(encoding="utf-8").splitlines()
    for number, line in enumerate(inv, 1):
        if line.startswith("| I-7.13 "):
            print(f"name=i713_row at=crates/singlefs-checker/invariants/02-根环.md:{number} "
                  f"says_eight_members={'共八项' in line} names_prime_step_363={'363' in line} "
                  f"names_chunk_367={'367' in line} pbs_lower_457={'[457' in line}")


if __name__ == "__main__":
    main()
