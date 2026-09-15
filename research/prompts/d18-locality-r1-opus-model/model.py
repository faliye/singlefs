#!/usr/bin/env python3
# D18（块里携带什么信息） 已定项 3 那条提议（自描述头加 locality_id）第一轮，攻方腿（Opus）的模型。只用 std。
# 复跑（仓根）：
#   python3 research/prompts/d18-locality-r1-opus-model/model.py > /tmp/claude-1000/d18-locality-opus.out
#   diff /tmp/claude-1000/d18-locality-opus.out research/prompts/d18-locality-r1-opus-model/output.txt
# 口径：确定性模型（无随机源、无 I/O），跑 N 遍与跑 1 遍信息量相同；证据强度来自钉绝对值的断言与阳性对照。
# 建模范围：单元归属（哪个单元属于哪个头的哪个对象）与现行版本判定由神谕给出（C113 / 克隆祖先表视为已解），
# 只量 locality_id 这一维：重建之后记录的 locality 与该对象每条 extent key 首段是否一致、读者按记录找不找得到数据。
import itertools
import sys

RECORDS_PER_CONTAINER = 2   # 真值 233（D8 已定项 6）；取 2 让「丢一个容器连带丢邻居」在短历史里出现
DIRECTORY_LOCALITIES = (1, 2)
V1_LOCALITY = 0             # 第一版无父目录取 0（D8 已定项 6 字段表偏移 16）
OFFSETS = (0, 1)
MAX_CREATES_PER_HEAD = 2


class World:
    """一次历史跑完之后的盘面。unit / 记录容器版本 / extent 条目各有物理 id，克隆之后按 id 共享。"""

    def __init__(self, rewrite_units_on_rekey):
        self.rewrite_units_on_rekey = rewrite_units_on_rekey  # 甲：AAD 绑 locality ⇒ 重编 key 必须重写单元
        self.next_physical_id = 1
        self.txg = 1
        self.units = {}              # unit_id -> dict
        self.heads = {}              # tree_id -> dict
        self.container_previous = {}  # container_version_id -> 上一版 id 或 None
        self.container_snapshot = {}  # container_version_id -> {inode: (locality, object_birth)}
        self.forced_unit_rewrites = 0
        self.unshared_by_rewrite = 0
        self.stale_header_hints = 0

    def allocate_physical_id(self):
        physical_id = self.next_physical_id
        self.next_physical_id += 1
        return physical_id

    def publish(self):
        self.txg += 1


def new_head(world, tree_id):
    world.heads[tree_id] = {
        "tree_id": tree_id,
        "watermark": 1,          # 下一个 inode 号；inode 1 起（0 无效）
        "creates": 0,
        "records": {},           # inode -> {"locality": L, "object_birth": b}
        "record_container": {},  # 容器组号 -> container_version_id
        "extents": {},           # (locality, inode, offset) -> (entry_id, unit_id)
    }


def container_group(inode):
    return (inode - 1) // RECORDS_PER_CONTAINER


def cow_record_container(world, head, inode):
    """改一条记录 = COW 它所在的容器（D8 已定项 6：更新一条记录 = COW 叶 + 祖先）。"""
    group = container_group(inode)
    previous_version = head["record_container"].get(group)
    new_version = world.allocate_physical_id()
    world.container_previous[new_version] = previous_version
    world.container_snapshot[new_version] = {
        member_inode: (record["locality"], record["object_birth"])
        for member_inode, record in head["records"].items()
        if container_group(member_inode) == group
    }
    head["record_container"][group] = new_version


def op_create(world, tree_id, locality):
    head = world.heads[tree_id]
    inode = head["watermark"]
    head["watermark"] += 1
    head["creates"] += 1
    head["records"][inode] = {"locality": locality, "object_birth": world.txg}
    cow_record_container(world, head, inode)
    world.publish()


def op_write(world, tree_id, inode, offset):
    """写一个 extent：新单元的头取写那一刻记录里的 locality（甲 / 乙的第六个字段）。"""
    head = world.heads[tree_id]
    record = head["records"][inode]
    unit_id = world.allocate_physical_id()
    world.units[unit_id] = {
        "birth_tree": tree_id, "object_id": inode, "object_birth": record["object_birth"],
        "anchor_offset": offset, "header_locality": record["locality"], "write_order": world.txg,
    }
    for key in [key for key in head["extents"] if key[1] == inode and key[2] == offset]:
        del head["extents"][key]
    head["extents"][(record["locality"], inode, offset)] = (world.allocate_physical_id(), unit_id)
    world.publish()


def units_referenced_by_other_heads(world, tree_id):
    referenced = set()
    for other_tree_id, other_head in world.heads.items():
        if other_tree_id != tree_id:
            referenced.update(unit_id for _entry_id, unit_id in other_head["extents"].values())
    return referenced


def op_rekey(world, tree_id, inode, new_locality):
    """后台重新聚簇（D8 已定项 3「批量改名」第 3 条设想的那个动作）：记录与该对象全部 extent key 同一事务改首段。"""
    head = world.heads[tree_id]
    head["records"][inode]["locality"] = new_locality
    cow_record_container(world, head, inode)
    shared_elsewhere = units_referenced_by_other_heads(world, tree_id)
    for key in sorted(key for key in head["extents"] if key[1] == inode):
        _old_entry_id, unit_id = head["extents"].pop(key)
        if world.rewrite_units_on_rekey:
            old_unit = world.units[unit_id]
            rewritten_unit_id = world.allocate_physical_id()
            world.units[rewritten_unit_id] = dict(old_unit, birth_tree=tree_id,
                                                  header_locality=new_locality, write_order=world.txg)
            world.forced_unit_rewrites += 1
            if unit_id in shared_elsewhere:
                world.unshared_by_rewrite += 1
            unit_id = rewritten_unit_id
        elif world.units[unit_id]["header_locality"] != new_locality:
            world.stale_header_hints += 1
        head["extents"][(new_locality, inode, key[2])] = (world.allocate_physical_id(), unit_id)
    world.publish()


def op_clone(world, source_tree_id, clone_tree_id):
    """克隆头（D6 已定项 1 每头一棵树）：记录容器与 extent 条目按物理 id 共享，水位取 origin 那一刻。"""
    source = world.heads[source_tree_id]
    new_head(world, clone_tree_id)
    clone = world.heads[clone_tree_id]
    clone["watermark"] = source["watermark"]
    clone["records"] = {inode: dict(record) for inode, record in source["records"].items()}
    clone["record_container"] = dict(source["record_container"])
    clone["extents"] = dict(source["extents"])
    world.publish()


def replay(history, rewrite_units_on_rekey):
    world = World(rewrite_units_on_rekey)
    new_head(world, 11)
    for operation in history:
        kind = operation[0]
        if kind == "create":
            op_create(world, operation[1], operation[2])
        elif kind == "write":
            op_write(world, operation[1], operation[2], operation[3])
        elif kind == "rekey":
            op_rekey(world, operation[1], operation[2], operation[3])
        elif kind == "clone":
            op_clone(world, operation[1], operation[2])
        else:
            raise AssertionError("未知操作 " + repr(operation))
    return world


# ---------------- 损坏 ----------------
EXTENT_DAMAGE_PER_OBJECT = ("intact", "all_lost", "first_lost")
HEADER_ARMS = ("jia", "yi_per_unit", "yi_per_object")
PER_UNIT_KEYING_ARMS = ("jia", "yi_per_unit")   # 甲：AAD 逼着按单元头编首段；乙的最弱读法同形
ARMS = ("jia", "yi_per_unit", "yi_per_object", "bing", "bing_inode_only")
PRECEDENCES = ("record_first", "keys_first")
# 重建记录的 locality 取哪个来源、按什么次序（第一个非空者胜）。record_first 是各臂的字面读法，keys_first 是收严。
SOURCE_ORDER = {
    "jia": {"record_first": ("record", "header", "zero"), "keys_first": ("keys", "header", "record", "zero")},
    "yi_per_unit": {"record_first": ("record", "header", "zero"), "keys_first": ("keys", "header", "record", "zero")},
    "yi_per_object": {"record_first": ("record", "keys", "header", "zero"),
                      "keys_first": ("keys", "record", "header", "zero")},
    "bing": {"record_first": ("record", "keys", "zero"), "keys_first": ("keys", "record", "zero")},
    "bing_inode_only": {"record_first": ("record", "keys", "zero"), "keys_first": ("keys", "record", "zero")},
}


def damage_patterns(world, target_tree_id):
    """目标头上：每个记录容器组 intact / lost / rolled_back（最新版读不出、上一版可读）；每个对象的 extent 条目
    全在 / 全丢 / 只丢最低 offset 那条。别的头共享的物理件跟着一起坏，但只重建并检查目标头。"""
    head = world.heads[target_tree_id]
    container_choices = []
    for group in sorted(head["record_container"]):
        states = ["intact", "lost"]
        if world.container_previous.get(head["record_container"][group]) is not None:
            states.append("rolled_back")
        container_choices.append([(group, state) for state in states])
    objects_with_extents = sorted({key[1] for key in head["extents"]})
    extent_choices = [[(inode, state) for state in EXTENT_DAMAGE_PER_OBJECT] for inode in objects_with_extents]
    for container_combo in itertools.product(*container_choices):
        for extent_combo in itertools.product(*extent_choices):
            yield dict(container_combo), dict(extent_combo)


def surviving_records(world, head, container_damage):
    survivors = {}
    for group, version in head["record_container"].items():
        state = container_damage[group]
        if state == "lost":
            continue
        chosen = version if state == "intact" else world.container_previous[version]
        snapshot = world.container_snapshot[chosen]
        if state == "intact":
            current = {inode: (record["locality"], record["object_birth"])
                       for inode, record in head["records"].items() if container_group(inode) == group}
            assert snapshot == current, "容器快照与头里的记录对不上：模型自身的 bug"
        for inode, (locality, _object_birth) in snapshot.items():
            survivors[inode] = locality
    return survivors


def split_extents(head, extent_damage):
    """返回 (幸存条目, 丢了条目的单元, 丢了的物理条目 id)。丢了条目的单元由扫描按单元头找回（归属神谕）。"""
    by_object = {}
    for key, (entry_id, unit_id) in head["extents"].items():
        by_object.setdefault(key[1], []).append((key, entry_id, unit_id))
    surviving, lost_units, lost_entry_ids = {}, [], set()
    for inode, entries in by_object.items():
        entries.sort(key=lambda item: item[0][2])
        state = extent_damage.get(inode, "intact")
        for position, (key, entry_id, unit_id) in enumerate(entries):
            if state == "all_lost" or (state == "first_lost" and position == 0):
                lost_units.append((inode, key[2], unit_id))
                lost_entry_ids.add(entry_id)
            else:
                surviving[key] = unit_id
    return surviving, lost_units, lost_entry_ids


def rebuild(world, target_tree_id, container_damage, extent_damage, arm, precedence, extra_scanned_units=()):
    """按臂的规则重建目标头：返回 (重建后的记录 locality, 重建后的 extent 映射, 歧义对象数)。"""
    head = world.heads[target_tree_id]
    records_left = surviving_records(world, head, container_damage)
    surviving, lost_units, lost_entry_ids = split_extents(head, extent_damage)
    lost_units = lost_units + list(extra_scanned_units)
    rebuilt_records, rebuilt_extents, ambiguous_objects = {}, dict(surviving), 0
    candidate_objects = sorted(set(records_left) | {key[1] for key in surviving} | {item[0] for item in lost_units})
    for inode in candidate_objects:
        if arm == "bing_inode_only":   # 最弱读法：「这个对象」按 inode 号认，不按 (树 ID, inode)
            segments = sorted({key[0] for other in world.heads.values()
                               for key, (entry_id, _unit) in other["extents"].items()
                               if key[1] == inode and entry_id not in lost_entry_ids})
        else:
            segments = sorted({key[0] for key in surviving if key[1] == inode})
        object_units = [world.units[unit_id] for key, unit_id in surviving.items() if key[1] == inode]
        object_units += [world.units[unit_id] for lost_inode, _offset, unit_id in lost_units if lost_inode == inode]
        header_values = sorted({unit["header_locality"] for unit in object_units})
        newest_header = None
        if arm in HEADER_ARMS and object_units:
            newest_header = max(object_units, key=lambda unit: unit["write_order"])["header_locality"]
        if len(segments) > 1 or (arm in HEADER_ARMS and len(header_values) > 1):
            ambiguous_objects += 1
        sources = {"record": records_left.get(inode), "keys": segments[0] if segments else None,
                   "header": newest_header, "zero": V1_LOCALITY}
        chosen = next(sources[name] for name in SOURCE_ORDER[arm][precedence] if sources[name] is not None)
        rebuilt_records[inode] = chosen
        for lost_inode, offset, unit_id in lost_units:
            if lost_inode != inode:
                continue
            if arm in PER_UNIT_KEYING_ARMS:
                first_segment = world.units[unit_id]["header_locality"]
            else:
                first_segment = chosen
            key = (first_segment, inode, offset)
            if key in rebuilt_extents:   # 同 key 两版：按写序择新（C113 的形状）
                if world.units[rebuilt_extents[key]]["write_order"] > world.units[unit_id]["write_order"]:
                    continue
            rebuilt_extents[key] = unit_id
    return rebuilt_records, rebuilt_extents, ambiguous_objects


# ---------------- 检查 ----------------
FAILURE_FIELDS = ("missing", "misread", "mac_fail", "resurrected", "split_objects")


def check_reads(world, target_tree_id, rebuilt_records, rebuilt_extents, arm, expected_override=None):
    """读者按重建后记录里的 locality 查每个本该有数据的 offset；另数「一个对象两个首段」与复活的数据。"""
    head = world.heads[target_tree_id]
    expected_by_object = {}
    for key, (_entry_id, unit_id) in head["extents"].items():
        expected_by_object.setdefault(key[1], {})[key[2]] = unit_id
    if expected_override is not None:
        expected_by_object = expected_override
    tally = dict(objects_checked=0, extents_checked=0, missing=0, misread=0, mac_fail=0, resurrected=0,
                 split_objects=0, locality_changed=0)
    for inode, expected in sorted(expected_by_object.items()):
        tally["objects_checked"] += 1
        locality = rebuilt_records.get(inode)
        if locality is None:
            tally["missing"] += len(expected)
            continue
        if inode in head["records"] and locality != head["records"][inode]["locality"]:
            tally["locality_changed"] += 1
        object_keys = [key for key in rebuilt_extents if key[1] == inode]
        if len({key[0] for key in object_keys}) > 1:
            tally["split_objects"] += 1
        for offset, expected_unit in sorted(expected.items()):
            tally["extents_checked"] += 1
            found = rebuilt_extents.get((locality, inode, offset))
            if found is None:
                tally["missing"] += 1
            elif found != expected_unit:
                tally["misread"] += 1
            elif arm == "jia" and world.units[found]["header_locality"] != locality:
                tally["mac_fail"] += 1   # 甲：AAD 期望值取查找路径的首段，与单元里绑的那个不等 ⇒ MAC 败
        for key in object_keys:
            if key[0] == locality and key[2] not in expected:
                tally["resurrected"] += 1  # 读者按记录看得见、而这个头在损坏前没有的数据
    return tally


def is_failure(tally):
    return any(tally[field] for field in FAILURE_FIELDS)


# ---------------- 穷举 ----------------
def legal_operations(world, rekey_enabled, v1_mode):
    operations = []
    localities = (V1_LOCALITY,) if v1_mode else DIRECTORY_LOCALITIES
    for tree_id in sorted(world.heads):
        head = world.heads[tree_id]
        if head["creates"] < MAX_CREATES_PER_HEAD:
            operations.extend(("create", tree_id, locality) for locality in localities)
        for inode in sorted(head["records"]):
            written_by_this_head = {key[2] for key, (_entry_id, unit_id) in head["extents"].items()
                                    if key[1] == inode and world.units[unit_id]["birth_tree"] == tree_id}
            operations.extend(("write", tree_id, inode, offset)
                              for offset in OFFSETS if offset not in written_by_this_head)
            has_extents = any(key[1] == inode for key in head["extents"])
            if rekey_enabled and has_extents and not v1_mode:
                current_locality = head["records"][inode]["locality"]
                operations.extend(("rekey", tree_id, inode, locality)
                                  for locality in DIRECTORY_LOCALITIES if locality != current_locality)
    if len(world.heads) == 1:
        operations.append(("clone", 11, 21))
    return operations


def explore(max_depth, rekey_enabled, v1_mode):
    """按深度宽搜全部历史；每条历史 × 每个头 × 全部损坏形态 × 每条臂（含两种优先级读法）重建并检查。"""
    variants = [(arm, precedence) for arm in ARMS for precedence in PRECEDENCES]
    stats = {variant: dict(histories=0, patterns=0, objects=0, failures=0, ambiguous=0, locality_changed=0,
                           forced_rewrites=0, unshared=0, stale_hints=0) for variant in variants}
    shortest = {}
    frontier = [()]
    for depth in range(1, max_depth + 1):
        next_frontier = []
        for history in frontier:
            base_world = replay(history, rewrite_units_on_rekey=False)
            next_frontier.extend(history + (operation,)
                                 for operation in legal_operations(base_world, rekey_enabled, v1_mode))
        frontier = next_frontier
        for history in frontier:
            worlds = {False: replay(history, False), True: replay(history, True)}
            for arm, precedence in variants:
                world = worlds[arm == "jia"]
                stat = stats[(arm, precedence)]
                stat["histories"] += 1
                stat["forced_rewrites"] += world.forced_unit_rewrites
                stat["unshared"] += world.unshared_by_rewrite
                stat["stale_hints"] += world.stale_header_hints
                for target_tree_id in sorted(world.heads):
                    for container_damage, extent_damage in damage_patterns(world, target_tree_id):
                        stat["patterns"] += 1
                        records, extents, ambiguous = rebuild(world, target_tree_id, container_damage,
                                                              extent_damage, arm, precedence)
                        tally = check_reads(world, target_tree_id, records, extents, arm)
                        stat["objects"] += tally["objects_checked"]
                        stat["ambiguous"] += ambiguous
                        stat["locality_changed"] += tally["locality_changed"]
                        if is_failure(tally):
                            stat["failures"] += 1
                            if (arm, precedence) not in shortest:
                                shortest[(arm, precedence)] = (depth, history, target_tree_id,
                                                               container_damage, extent_damage, tally)
    return stats, shortest


# ---------------- 手造历史：钉绝对值（每条要么是最短反例、要么是阳性对照） ----------------
def expect(label, actual, wanted):
    verdict = "ok" if actual == wanted else "FAIL"
    print("assert %-62s got=%-6s want=%-6s %s" % (label, actual, wanted, verdict))
    if actual != wanted:
        raise SystemExit("断言失败：" + label)


def no_damage(world, tree_id):
    return {group: "intact" for group in world.heads[tree_id]["record_container"]}, {}


def run_case(history, tree_id, container_damage, extent_damage, arm, precedence, rewrite=None, extra=()):
    world = replay(history, (arm == "jia") if rewrite is None else rewrite)
    if container_damage is None:
        container_damage, extent_damage = no_damage(world, tree_id)
    records, extents, ambiguous = rebuild(world, tree_id, container_damage, extent_damage, arm, precedence, extra)
    return world, records, extents, ambiguous, check_reads(world, tree_id, records, extents, arm)


def i99_verdict(record_locality, key_segments, keys_derived_from_record):
    """I-9.9（locality 三值对照）：两侧同源 ⇒ 不可判定；否则相等绿、不等红。"""
    if keys_derived_from_record:
        return "undecidable"
    return "green" if set(key_segments) == {record_locality} else "red"


def hand_built_cases():
    print("## 手造历史（钉绝对值）")
    history_a = (("create", 11, 1), ("write", 11, 1, 0), ("write", 11, 1, 1), ("clone", 11, 21), ("rekey", 21, 1, 2))
    world_jia, world_plain = replay(history_a, True), replay(history_a, False)
    expect("A 甲：克隆里重新聚簇一个 2 单元对象，强制重写单元数", world_jia.forced_unit_rewrites, 2)
    expect("A 甲：其中本与 origin 共享、被重写拆开的单元数", world_jia.unshared_by_rewrite, 2)
    expect("A 乙/丙：同一历史强制重写单元数", world_plain.forced_unit_rewrites, 0)
    expect("A 乙：同一历史留下的过期头提示数", world_plain.stale_header_hints, 2)
    _w, _r, _e, _a, tally = run_case(history_a, 21, None, None, "jia", "record_first", rewrite=False)
    expect("B 阳性对照：甲不重写就重新聚簇，无损坏时 MAC 败数", tally["mac_fail"], 2)
    _w, _r, _e, _a, tally = run_case(history_a, 21, None, None, "jia", "record_first", rewrite=True)
    expect("B 甲按规矩重写之后，无损坏时 MAC 败数", tally["mac_fail"], 0)

    history_c = (("create", 11, 1), ("write", 11, 1, 0), ("rekey", 11, 1, 2), ("write", 11, 1, 1))
    lose_all = ({0: "lost"}, {1: "all_lost"})
    outcomes_c = {}
    for arm in ARMS:
        _w, records, _e, _a, tally = run_case(history_c, 11, lose_all[0], lose_all[1], arm, "record_first")
        outcomes_c[arm] = (tally["missing"], tally["split_objects"], tally["locality_changed"], records[1])
    expect("C 乙逐单元读法：记录与 extent 全丢后读者找不到的 extent 数", outcomes_c["yi_per_unit"][0], 1)
    expect("C 乙逐单元读法：一个对象两个首段的对象数", outcomes_c["yi_per_unit"][1], 1)
    expect("C 乙按对象读法：找不到数", outcomes_c["yi_per_object"][0], 0)
    expect("C 丙：找不到数", outcomes_c["bing"][0], 0)
    expect("C 丙：重建后的 locality（原值 2，丢了局部性）", outcomes_c["bing"][3], 0)
    expect("C 甲（重新聚簇时已重写）：找不到数", outcomes_c["jia"][0], 0)

    history_d = (("create", 11, 1), ("write", 11, 1, 0), ("rekey", 11, 1, 2))
    for arm in ARMS:
        for precedence, wanted in (("record_first", 1), ("keys_first", 0)):
            _w, _r, _e, _a, tally = run_case(history_d, 11, {0: "rolled_back"}, {}, arm, precedence)
            expect("D 容器回到上一版 + extent 在：%s/%s 找不到数" % (arm, precedence), tally["missing"], wanted)

    history_e = (("create", 11, 1), ("clone", 11, 21), ("create", 11, 1), ("create", 21, 2),
                 ("write", 11, 2, 0), ("write", 21, 2, 0))
    _w, records, _e, ambiguous, tally = run_case(history_e, 21, {0: "lost"}, {}, "bing_inode_only", "record_first")
    expect("E 丙按 inode 号认对象：克隆两头同号异物时找不到数", tally["missing"], 1)
    expect("E 丙按 inode 号认对象：歧义对象数", ambiguous, 1)
    _w, records, _e, ambiguous, tally = run_case(history_e, 21, {0: "lost"}, {}, "bing", "record_first")
    expect("E 丙按 (树 ID, inode) 认对象：找不到数", tally["missing"], 0)

    history_f = (("create", 11, V1_LOCALITY), ("write", 11, 1, 0), ("write", 11, 1, 1))
    for arm in ARMS:
        _w, records, _e, _a, tally = run_case(history_f, 11, {0: "lost"}, {1: "all_lost"}, arm, "record_first")
        expect("F 第一版恒 0：%s 找不到数 + 局部性变化数" % arm, tally["missing"] + tally["locality_changed"], 0)

    print("## G 实例表不可读时的孤儿（D18 已定项 11「表不可读时」：全部旧实例单元判已发布）")
    for abandoned_locality, reissued_locality in ((1, 2), (1, 1), (0, 0)):
        for arm in ARMS:
            world = replay((("create", 11, reissued_locality), ("write", 11, 1, 1)), arm == "jia")
            orphan_id = world.allocate_physical_id()
            world.units[orphan_id] = {"birth_tree": 11, "object_id": 1,
                                      "object_birth": world.heads[11]["records"][1]["object_birth"],
                                      "anchor_offset": 0, "header_locality": abandoned_locality, "write_order": 0}
            records, extents, _a = rebuild(world, 11, {0: "lost"}, {1: "all_lost"}, arm, "record_first",
                                           [(1, 0, orphan_id)])
            tally = check_reads(world, 11, records, extents, arm)
            print("G abandoned=%d reissued=%d %-16s resurrected=%d split=%d missing=%d"
                  % (abandoned_locality, reissued_locality, arm, tally["resurrected"],
                     tally["split_objects"], tally["missing"]))


def inject_record_locality(world, tree_id, inode, wrong_locality):
    """模拟一次实现 bug：记录里的 locality 被写错，extent key 仍在原首段。"""
    head = world.heads[tree_id]
    head["records"][inode]["locality"] = wrong_locality
    version = head["record_container"][container_group(inode)]
    world.container_snapshot[version][inode] = (wrong_locality, head["records"][inode]["object_birth"])


def i99_cases():
    print("## H I-9.9（locality 三值对照）判别力自证：往记录里注入一个错的 locality，extent key 仍在原首段")
    history_h = (("create", 11, 1), ("write", 11, 1, 0), ("write", 11, 1, 1))
    verdicts = {}
    for injected in (False, True):
        for arm in ARMS:
            world = replay(history_h, arm == "jia")
            if injected:
                inject_record_locality(world, 11, 1, 2)
            head = world.heads[11]
            intact_segments = sorted({key[0] for key in head["extents"] if key[1] == 1})
            verdicts[("intact", injected, arm)] = i99_verdict(head["records"][1]["locality"], intact_segments, False)
            records, extents, _ambiguous = rebuild(world, 11, {0: "intact"}, {1: "all_lost"}, arm, "record_first")
            rebuilt_segments = sorted({key[0] for key in extents if key[1] == 1})
            keys_derived_from_record = arm not in PER_UNIT_KEYING_ARMS
            verdicts[("rebuilt", injected, arm)] = i99_verdict(records[1], rebuilt_segments, keys_derived_from_record)
            tally = check_reads(world, 11, records, extents, arm)
            print("H injected=%-5s %-16s intact=%-11s rebuilt=%-11s rebuilt_missing=%d"
                  % (injected, arm, verdicts[("intact", injected, arm)], verdicts[("rebuilt", injected, arm)],
                     tally["missing"]))
    for arm in ARMS:
        expect("H 未经重建 %s：干净绿、注入红" % arm,
               (verdicts[("intact", False, arm)], verdicts[("intact", True, arm)]), ("green", "red"))
    expect("H extent 树重建后 丙：干净与注入同为不可判定",
           (verdicts[("rebuilt", False, "bing")], verdicts[("rebuilt", True, "bing")]), ("undecidable", "undecidable"))
    expect("H extent 树重建后 甲：干净绿、注入红",
           (verdicts[("rebuilt", False, "jia")], verdicts[("rebuilt", True, "jia")]), ("green", "red"))
    print("## H′ 乙逐单元读法：合法的重新聚簇之后（不注入任何错），记录在、extent 树重建")
    history_legal = (("create", 11, 1), ("write", 11, 1, 0), ("rekey", 11, 1, 2))
    world = replay(history_legal, False)
    records, extents, _ambiguous = rebuild(world, 11, {0: "intact"}, {1: "all_lost"}, "yi_per_unit", "record_first")
    verdict = i99_verdict(records[1], sorted({key[0] for key in extents if key[1] == 1}), False)
    tally = check_reads(world, 11, records, extents, "yi_per_unit")
    expect("H′ 乙逐单元：合法镜像上重建后 I-9.9 判红（误报）", verdict, "red")
    expect("H′ 乙逐单元：记录还在也找不到的 extent 数", tally["missing"], 1)


def describe_history(history):
    return " ; ".join("%s(%s)" % (operation[0], ",".join(str(part) for part in operation[1:]))
                      for operation in history)


WORLDS = (("rekey_on", True, False, 6), ("rekey_off", False, False, 6), ("v1_all_zero", False, True, 7))


def main():
    depth_override = int(sys.argv[1]) if len(sys.argv) > 1 else None
    print("# D18 locality_id 第一轮攻方腿模型输出（确定性）RECORDS_PER_CONTAINER=%d MAX_CREATES_PER_HEAD=%d OFFSETS=%s"
          % (RECORDS_PER_CONTAINER, MAX_CREATES_PER_HEAD, OFFSETS))
    hand_built_cases()
    i99_cases()
    print("## 穷举：每条历史 × 每个头 × 全部损坏形态 × 每条臂（两种优先级读法）")
    print("columns: world arm precedence max_depth histories patterns objects failures ambiguous "
          "locality_changed forced_rewrites unshared stale_hints")
    for label, rekey_enabled, v1_mode, max_depth in WORLDS:
        depth = depth_override or max_depth
        stats, shortest = explore(depth, rekey_enabled, v1_mode)
        for arm in ARMS:
            for precedence in PRECEDENCES:
                stat = stats[(arm, precedence)]
                print("row", label, arm, precedence, depth, stat["histories"], stat["patterns"], stat["objects"],
                      stat["failures"], stat["ambiguous"], stat["locality_changed"], stat["forced_rewrites"],
                      stat["unshared"], stat["stale_hints"])
        for arm in ARMS:
            for precedence in PRECEDENCES:
                if (arm, precedence) not in shortest:
                    print("shortest", label, arm, precedence, "none_through_depth=%d" % depth)
                    continue
                found_depth, history, tree_id, container_damage, extent_damage, tally = shortest[(arm, precedence)]
                print("shortest", label, arm, precedence, "depth=%d" % found_depth, "head=%d" % tree_id,
                      "history=[%s]" % describe_history(history),
                      "containers=%s" % sorted(container_damage.items()),
                      "extents=%s" % sorted(extent_damage.items()),
                      "tally=%s" % ",".join("%s=%d" % (field, tally[field]) for field in FAILURE_FIELDS))
    print("done")


if __name__ == "__main__":
    main()
