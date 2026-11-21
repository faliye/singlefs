#!/usr/bin/env python3
"""E210 行 1 的第二条路：把可避免误拒那 30 条历史逐条打出来（正推腿草稿，纯 python3 标准库，不读仓里代码）。

照 `scripts/experiment-apparatus/e210_capacity_grow_reader_rules.rs` 的口径从头写一遍，不抄它的代码：
  池两块盘、每盘两槽；取号写两轮、每轮逐盘一写、轮后一道池屏障；每次槽写取 没持久 / 持久 / 新旧都读不出 三态；
  历史 = 形状 x 点位 x 断电点 x 用户动作；观测类 = (盘面的规范形, 探测值)；
  可避免误拒 = 可写被拒、而这一类里没有任何历史的落点越过探测值（落点严格大于盘尾才算越界）。
这一份只建四条臂（甲、乙、丁、基线臂；丙要六写序、这里不建）。输出每条臂的读数，格式与 E210 产物里的
`E7RESULT name=reading ...` 行逐字相同，另打出丁的可避免误拒逐条明细与分组。

用法：python3 -I e210_avoidable_witnesses.py [--artifact <E210 行 1 产物路径>]
带 --artifact 时，把自己打出的 reading 行与产物里同行逐行比，不一致退 1。
"""
import itertools
import sys

GROW_STEP = 100
DISKS = 2


def table_of(slot):
    return slot[1]


def next_target(board, disk):
    generations = [slot[0] for slot in board[disk] if slot is not None]
    generation = (max(generations) if generations else 0) + 1
    return generation, generation % 2


def start_board(newest_slot, table):
    board = [[None, None] for _ in range(DISKS)]
    for disk in range(DISKS):
        newest = newest_slot[disk]
        newest_generation = 3 if newest == 1 else 4
        board[disk][newest] = (newest_generation, table)
        board[disk][1 - newest] = (newest_generation - 1, table)
    return board


def copy_board(board):
    return [list(slots) for slots in board]


def crash_points(start, back):
    """四写序：两段、每段逐盘各一写。交回 [(盘面, 是不是终点已用, 各次写的态)]，态 0 没持久、1 持久、2 新旧都读不出。"""
    segments = [[0, 1], [0, 1]]
    points = []
    board_before = copy_board(start)
    for segment in segments:
        whole = copy_board(board_before)
        targets = []
        for disk in segment:
            generation, slot_index = next_target(whole, disk)
            content = (generation, back)
            whole[disk][slot_index] = content
            targets.append((disk, slot_index, content))
        for outcomes in itertools.product(range(3), repeat=len(segment)):
            board = copy_board(board_before)
            for (disk, slot_index, content), outcome in zip(targets, outcomes):
                if outcome == 1:
                    board[disk][slot_index] = content
                elif outcome == 2:
                    board[disk][slot_index] = None
            points.append((board, False, outcomes))
        board_before = whole
    points.append((board_before, True, None))
    return points


SHAPES = [
    ("D", (100, 100), (200, 200), (300, 300)),
    ("D", (100, 100), (200, 200), (300, 200)),
    ("D", (100, 100), (200, 100), (200, 300)),
    ("N", (100, 100), (100, 100), (200, 200)),
    ("N", (100, 100), (100, 100), (200, 100)),
]


def actions_of(family):
    names = ["不动"]
    if family == "D":
        names += ["全盘缩回前尺寸", "只盘0缩回前尺寸"]
    names += ["全盘缩回早尺寸", "只盘0缩回早尺寸", "全盘再变大", "只盘0再变大",
              "盘0换回早镜像", "盘1换回早镜像"]
    if family == "D":
        names += ["盘0换回前镜像", "盘1换回前镜像"]
    return names


def mirrored(table):
    return [(1, table), (1, table)]


def after_action(name, shape, board):
    _, early, front, back = shape
    board = copy_board(board)
    probe = list(back)
    if name == "全盘缩回前尺寸":
        probe = list(front)
    elif name == "只盘0缩回前尺寸":
        probe[0] = front[0]
    elif name == "全盘缩回早尺寸":
        probe = list(early)
    elif name == "只盘0缩回早尺寸":
        probe[0] = early[0]
    elif name == "全盘再变大":
        probe = [size + GROW_STEP for size in probe]
    elif name == "只盘0再变大":
        probe[0] += GROW_STEP
    elif name.endswith("换回早镜像"):
        disk = int(name[1])
        probe[disk] = early[disk]
        board[disk] = mirrored(early)
    elif name.endswith("换回前镜像"):
        disk = int(name[1])
        probe[disk] = front[disk]
        board[disk] = mirrored(front)
    return board, tuple(probe)


def canonical(board):
    result = []
    for slots in board:
        keys = sorted((0,) if slot is None else (1, table_of(slot)) for slot in slots)
        result.append(tuple(keys))
    return tuple(result)


def history_set():
    observations = []
    for shape_index, shape in enumerate(SHAPES):
        family, early, front, back = shape
        start = start_board((1, 1), front)
        points = crash_points(start, back)
        footprints = ("满", "少") if family == "D" else ("满",)
        for footprint in footprints:
            for point_index, (board_at_crash, is_end, outcomes) in enumerate(points):
                if footprint == "少" and is_end:
                    continue
                if footprint == "满":
                    landing = back if is_end else front
                else:
                    landing = early
                for name in actions_of(family):
                    board, probe = after_action(name, shape, board_at_crash)
                    if any(all(slot is None for slot in slots) for slots in board):
                        continue
                    observations.append({
                        "shape": shape_index, "footprint": footprint, "point": point_index,
                        "outcomes": outcomes, "action": name, "board": board,
                        "probe": probe, "landing": landing,
                    })
    union = {}
    for observation in observations:
        key = (canonical(observation["board"]), observation["probe"])
        beyond = tuple(observation["landing"][d] > observation["probe"][d] for d in range(DISKS))
        previous = union.get(key, (False,) * DISKS)
        union[key] = tuple(a or b for a, b in zip(previous, beyond))
    return observations, union


def tables_of(board):
    return [table_of(slot) for slots in board for slot in slots if slot is not None]


def minimum(tables):
    return tuple(min(table[d] for table in tables) for d in range(DISKS))


def maximum(tables):
    return tuple(max(table[d] for table in tables) for d in range(DISKS))


def is_chain(tables):
    def below(left, right):
        return all(left[d] <= right[d] for d in range(DISKS))
    return all(below(a, b) or below(b, a) for a in tables for b in tables)


def standard(table, probe):
    named = tuple(probe[d] < table[d] for d in range(DISKS))
    return (not any(named), True, named)


REFUSED = (False, False, (False,) * DISKS)


def decide(arm, board, probe):
    tables = tables_of(board)
    if arm == "chain_then_lowest":
        return standard(minimum(tables), probe) if is_chain(tables) else REFUSED
    if arm == "highest_table":
        return standard(maximum(tables), probe)
    if arm == "baseline_only_if_every_table_is_equal":
        return standard(tables[0], probe) if all(t == tables[0] for t in tables) else REFUSED
    if arm == "lowest_to_read_highest_to_probe":
        outcome = standard(maximum(tables), probe)
        lowest = minimum(tables)
        differ = any(t != tables[0] for t in tables)
        reaches = all(probe[d] >= lowest[d] for d in range(DISKS))
        return (outcome[0] or (differ and reaches), outcome[1], outcome[2])
    raise ValueError(arm)


ARMS = ("chain_then_lowest", "highest_table", "lowest_to_read_highest_to_probe",
        "baseline_only_if_every_table_is_equal")
READINGS = ("leak_writable", "leak_read_only", "misrefusal_literal", "misrefusal_avoidable", "writable_refused")


def flags_of(decision, observation, union):
    writable, read_only, named = decision
    beyond = tuple(observation["landing"][d] > observation["probe"][d] for d in range(DISKS))
    class_union = union[(canonical(observation["board"]), observation["probe"])]
    return (
        writable and any(beyond),
        read_only and any(beyond[d] and not named[d] for d in range(DISKS)),
        (not writable) and not any(beyond),
        (not writable) and not any(class_union),
        not writable,
    )


def describe_board(board):
    parts = []
    for slots in board:
        parts.append("[" + ";".join("读不出" if slot is None else f"g{slot[0]}{slot[1]}" for slot in slots) + "]")
    return "|".join("盘" + str(i) + parts[i] for i in range(DISKS))


def main(argv):
    artifact = None
    if len(argv) == 3 and argv[1] == "--artifact":
        artifact = argv[2]
    observations, union = history_set()
    lines = []
    witnesses = []
    for arm in ARMS:
        counts = [0] * len(READINGS)
        for observation in observations:
            decision = decide(arm, observation["board"], observation["probe"])
            flags = flags_of(decision, observation, union)
            for index, flag in enumerate(flags):
                if flag:
                    counts[index] += 1
            if arm == "lowest_to_read_highest_to_probe" and flags[3]:
                witnesses.append(observation)
        for reading, count in zip(READINGS, counts):
            lines.append(
                f"E7RESULT name=reading arm={arm} reading={reading} count={count} "
                f"denominator={len(observations)} exists={'true' if count else 'false'}")
    for line in lines:
        print(line)
    for observation in witnesses:
        shape = SHAPES[observation["shape"]]
        print("WITNESS arm=lowest_to_read_highest_to_probe reading=misrefusal_avoidable "
              f"family={shape[0]} early={shape[1]} front={shape[2]} back={shape[3]} footprint={observation['footprint']} "
              f"crash_point={observation['point']} write_outcomes_of_the_segment={observation['outcomes']} "
              f"action={observation['action']} board={describe_board(observation['board'])} "
              f"probe={observation['probe']} landing={observation['landing']}")
    by_point = {}
    by_action = {}
    for observation in witnesses:
        by_point[observation["point"]] = by_point.get(observation["point"], 0) + 1
        by_action[observation["action"]] = by_action.get(observation["action"], 0) + 1
    print("GROUP by_crash_point " + " ".join(f"{k}={v}" for k, v in sorted(by_point.items())))
    print("GROUP by_action " + " ".join(f"{k}={v}" for k, v in sorted(by_action.items())))
    print(f"GROUP witness_total={len(witnesses)}")
    if artifact is None:
        return 0
    with open(artifact, encoding="utf-8") as handle:
        artifact_lines = set(handle.read().splitlines())
    mismatches = [line for line in lines if line not in artifact_lines]
    for line in mismatches:
        print("MISMATCH 产物里没有这一行：" + line)
    print(f"ARTIFACT_COMPARE lines={len(lines)} mismatches={len(mismatches)}")
    return 1 if mismatches else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
