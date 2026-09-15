#!/usr/bin/env python3
# D18（块里携带什么信息） 已定项 3 那条提议（自描述头加 locality_id）第二轮，攻方腿（Opus）的模型。只用 std。
# 复跑（仓根）：
#   python3 -B research/prompts/d18-locality-r2-opus-model/model.py > /tmp/claude-1000/d18-locality-r2-opus.out
#   diff /tmp/claude-1000/d18-locality-r2-opus.out research/prompts/d18-locality-r2-opus-model/output.txt
# 口径：确定性模型（无随机源、无 I/O），跑 N 遍与跑 1 遍信息量相同；证据强度来自钉绝对值的断言与阳性对照。
# 与第一轮模型（research/prompts/d18-locality-r1-opus-model/model.py，只读、没改）的差别：
#   ① 盘上保留每一个写出过的物理件：被覆写、被甲重写、被打包、被搬走之后旧的那份仍能被扫描读到、仍判已发布；
#      扫描重建按 C113 定案 P3 的形状择版本（先分组，组内写序最大者胜），分组键取三种读法。
#   ② 重新聚簇按两种定义各建：重编 key（三种协议：一次发布做完 / 逐单元跨发布且记录带双值 / 先抄后切）
#      与只搬物理位置；每一步之间都发布（D23 已定项 14 后的施加单位是一次发布，发布之间的状态都可能是恢复结果）。
#   ③ D27 打包容器：槽的字段逐字节取打包前那一版单元头（D19 已定项 6 ⑩㈠）；容器写出后不可变，改对象 = 搬出去，旧槽不标死。
#   ④ 克隆之后在任一头里逐单元重新聚簇。
# 建模范围：单元归属（哪个物理件属于哪个头的哪个对象）当神谕：这个头的这个对象曾经引用过的全部物理件；
#   只量 locality_id 这一维。删除 / 墓碑、reflink、被抛弃时间线的孤儿（第一轮 G）不建。
import itertools
import sys

RECORDS_PER_CONTAINER = 2   # 真值 233（D8 已定项 6）；取 2 让「丢一个容器连带丢邻居」在短历史里出现
OFFSETS = (0, 1)
DIRECTORY_LOCALITIES = (1, 2)
V1_LOCALITY = 0             # 第一版无父目录取 0（D8 已定项 6 字段表偏移 16）
ROLLBACK_DEPTH = 2          # 记录容器最新的 k 版读不出、第 k+1 新的那版可读，k 取 1..2


class World:
    """一次历史跑完之后的盘面。物理件（码 1 单元或容器里的槽）写出后不可变、永不从盘上消失。"""

    def __init__(self, family):
        # family 的键：rewrite（甲：重编 key 时按 AAD 重写单元）、slot_loc（'none' 槽不带 / 'copy' 槽 +8 逐字节取打包前头）、
        #   claim_has_locality（容器内认领比不比 locality）、packed_rekey（'leave' 槽不动 / 'move_out' 重编时把对象搬出去）、
        #   mac（加密开着：读路径按查找路径的首段核 AAD）
        self.family = family
        self.copy_write = "both"   # 先抄后切期间的用户写：both 两份 key 都改；recopy 只写主段、另一段那条 key 作废之后重抄
        self.next_physical_id = 1
        self.next_write_order = 1
        self.next_content = 1
        self.next_container = 1
        self.txg = 1
        self.units = {}
        self.heads = {}
        self.container_previous = {}
        self.container_snapshot = {}
        self.counters = dict(forced_rewrites=0, unshared=0, stale_hints=0, move_outs=0)

    def allocate_physical_id(self):
        physical_id = self.next_physical_id
        self.next_physical_id += 1
        return physical_id

    def allocate_write_order(self):
        write_order = self.next_write_order   # 一事务一单元（D16 已定项 5 末段）：每写一个单元取一个新写序
        self.next_write_order += 1
        return write_order

    def allocate_content(self):
        content = self.next_content
        self.next_content += 1
        return content

    def allocate_container(self):
        container = self.next_container
        self.next_container += 1
        return container

    def publish(self):
        self.txg += 1


def new_head(world, tree_id):
    world.heads[tree_id] = {
        "tree_id": tree_id,
        "watermark": 1,
        "creates": 0,
        "records": {},           # inode -> {"localities": (L,) 或 (L_new, L_old), "object_birth": b}
        "record_container": {},  # 容器组号 -> container_version_id
        "extents": {},           # (locality, inode, offset) -> physical_id
        "refs": {},              # inode -> 这个头的这个对象引用过的全部物理件（归属神谕）
        "truth": {},             # inode -> {offset: content}：用户最后一次写进去的内容（真值）
        "migration": {},         # inode -> 迁移状态
        "writes": {},
        "rekeys": {},
        "relocations": {},
        "packs": {},
    }


def container_group(inode):
    return (inode - 1) // RECORDS_PER_CONTAINER


def cow_record_container(world, head, inode):
    """改一条记录 = COW 它所在的容器（D8 已定项 6：更新一条记录 = COW 叶 + 祖先）。旧版留在盘上。"""
    group = container_group(inode)
    previous_version = head["record_container"].get(group)
    new_version = world.allocate_physical_id()
    world.container_previous[new_version] = previous_version
    world.container_snapshot[new_version] = {
        member_inode: record["localities"]
        for member_inode, record in head["records"].items()
        if container_group(member_inode) == group
    }
    head["record_container"][group] = new_version


def add_physical(world, owner_tree_id, inode, fields):
    """写出一个物理件并记进这个头这个对象的归属神谕。fields 是它的头：出生树、对象、出生代、锚点、写序、头里的 locality……"""
    physical_id = world.allocate_physical_id()
    world.units[physical_id] = dict(fields)
    world.heads[owner_tree_id]["refs"].setdefault(inode, set()).add(physical_id)
    return physical_id


def new_data_unit(world, tree_id, inode, offset, header_locality, content):
    head = world.heads[tree_id]
    return add_physical(world, tree_id, inode, {
        "kind": "unit", "birth_tree": tree_id, "object_id": inode,
        "object_birth": head["records"][inode]["object_birth"], "anchor_offset": offset,
        "header_locality": header_locality, "write_order": world.allocate_write_order(),
        "content": content, "container": None})


def keys_of(head, inode, offset=None):
    return sorted(key for key in head["extents"] if key[1] == inode and (offset is None or key[2] == offset))


def set_key(head, key, physical_id):
    head["extents"][key] = physical_id
    head["refs"].setdefault(key[1], set()).add(physical_id)


def primary_locality(head, inode):
    return head["records"][inode]["localities"][0]


def units_referenced_by_other_heads(world, tree_id):
    referenced = set()
    for other_tree_id, other_head in world.heads.items():
        if other_tree_id != tree_id:
            referenced.update(other_head["extents"].values())
    return referenced


# ---------------- 用户操作与三种「重编 key」协议 ----------------
def op_create(world, tree_id, locality):
    head = world.heads[tree_id]
    inode = head["watermark"]
    head["watermark"] += 1
    head["creates"] += 1
    head["records"][inode] = {"localities": (locality,), "object_birth": world.txg}
    head["truth"][inode] = {}
    cow_record_container(world, head, inode)
    world.publish()


def write_segments(head, inode, offset):
    """一次用户写落在哪几个首段下：跟着这个 offset 现在的 key 走（先抄后切时两份 key 都要改）；新 offset 落主段或迁移目标段。"""
    existing = [key[0] for key in keys_of(head, inode, offset)]
    if existing:
        return existing
    migration = head["migration"].get(inode)
    if migration is not None and migration["protocol"] in ("perunit", "naive"):
        return [migration["new"]]
    return [primary_locality(head, inode)]


def op_write(world, tree_id, inode, offset):
    head = world.heads[tree_id]
    content = world.allocate_content()
    segments = write_segments(head, inode, offset)
    migration = head["migration"].get(inode)
    if migration is not None and migration["protocol"] == "copy" and world.copy_write == "recopy":
        segments = [primary_locality(head, inode)]   # 只写主段；另一段那条 key 随下面一起删掉，之后由抄的那一步重抄
    for key in keys_of(head, inode, offset):
        del head["extents"][key]
    if world.family["rewrite"]:
        # 甲：每个首段一份单元，头里的 locality 等于那个首段（AAD 绑它）
        for segment in segments:
            set_key(head, (segment, inode, offset), new_data_unit(world, tree_id, inode, offset, segment, content))
        world.counters["forced_rewrites"] += len(segments) - 1
    else:
        # 乙′ / 丙′：一个单元；头里记写那一刻主段（在其中时）或这个 key 的首段——乙′ 的提示，丙′ 的世界里照样生成、重建不读
        primary = primary_locality(head, inode)
        header_locality = primary if primary in segments else segments[0]
        physical_id = new_data_unit(world, tree_id, inode, offset, header_locality, content)
        for segment in segments:
            set_key(head, (segment, inode, offset), physical_id)
    head["truth"][inode][offset] = content
    head["writes"][inode] = head["writes"].get(inode, 0) + 1
    if migration is not None and migration["protocol"] == "copy" and migration["phase"] == "drop" \
            and not any(key[0] == migration["old"] for key in keys_of(head, inode)):
        del head["migration"][inode]   # 删旧段阶段的写把最后一条旧段 key 也带走了：迁移就此结束
    world.publish()


def new_data_unit_like(world, tree_id, inode, unit, header_locality):
    """甲的重写：同一份内容换一个头（新 locality、新写序、出生树取做重写的这个头），整单元重加密成新的码 1 单元。"""
    return add_physical(world, tree_id, inode, dict(unit, kind="unit", birth_tree=tree_id, container=None,
                                                    header_locality=header_locality,
                                                    write_order=world.allocate_write_order()))


def rekey_physical(world, tree_id, inode, physical_id, new_locality, shared_elsewhere):
    """重编 key 时这个物理件换不换：甲按 AAD 重写码 1 单元；槽看政策；乙′ / 丙′ 不动（头里的提示就此过期）。"""
    unit = world.units[physical_id]
    family = world.family
    if unit["kind"] == "slot" and family["rewrite"] and family["slot_loc"] == "copy" \
            and family["claim_has_locality"] and family["packed_rekey"] == "move_out":
        world.counters["move_outs"] += 1
        world.counters["forced_rewrites"] += 1
        return new_data_unit_like(world, tree_id, inode, unit, new_locality)
    if unit["kind"] == "unit" and family["rewrite"]:
        world.counters["forced_rewrites"] += 1
        if physical_id in shared_elsewhere:
            world.counters["unshared"] += 1
        return new_data_unit_like(world, tree_id, inode, unit, new_locality)
    if unit["header_locality"] is not None and unit["header_locality"] != new_locality:
        world.counters["stale_hints"] += 1
    return physical_id


def op_rekey_atomic(world, tree_id, inode, new_locality):
    """协议 A：一次发布里把记录与这个对象全部 extent key 换首段（对象装得进一次发布时）。"""
    head = world.heads[tree_id]
    head["records"][inode]["localities"] = (new_locality,)
    cow_record_container(world, head, inode)
    shared_elsewhere = units_referenced_by_other_heads(world, tree_id)
    for key in keys_of(head, inode):
        physical_id = head["extents"].pop(key)
        set_key(head, (new_locality, inode, key[2]),
                rekey_physical(world, tree_id, inode, physical_id, new_locality, shared_elsewhere))
    head["rekeys"][inode] = head["rekeys"].get(inode, 0) + 1
    world.publish()


def op_migration_begin(world, tree_id, inode, new_locality, protocol):
    """协议 P：记录先带双值 (L_new, L_old)，读者两个都试；协议 N（阳性对照）：记录直接换成 L_new、不带旧值。"""
    head = world.heads[tree_id]
    old_locality = primary_locality(head, inode)
    head["migration"][inode] = {"protocol": protocol, "old": old_locality, "new": new_locality, "phase": "move"}
    if protocol == "perunit":
        head["records"][inode]["localities"] = (new_locality, old_locality)
    else:
        head["records"][inode]["localities"] = (new_locality,)
    cow_record_container(world, head, inode)
    head["rekeys"][inode] = head["rekeys"].get(inode, 0) + 1
    world.publish()


def op_migration_step(world, tree_id, inode):
    """把还在旧段、offset 最小的那条 key 搬到新段（甲：连单元一起重写，一事务一单元）。一批 = 一次发布。"""
    head = world.heads[tree_id]
    migration = head["migration"][inode]
    key = min((key for key in keys_of(head, inode) if key[0] == migration["old"]), key=lambda item: item[2])
    physical_id = head["extents"].pop(key)
    shared_elsewhere = units_referenced_by_other_heads(world, tree_id)
    set_key(head, (migration["new"], inode, key[2]),
            rekey_physical(world, tree_id, inode, physical_id, migration["new"], shared_elsewhere))
    world.publish()


def op_migration_end(world, tree_id, inode):
    head = world.heads[tree_id]
    migration = head["migration"].pop(inode)
    head["records"][inode]["localities"] = (migration["new"],)
    cow_record_container(world, head, inode)
    world.publish()


def offsets_needing_copy(head, inode, migration):
    old_offsets = {key[2] for key in keys_of(head, inode) if key[0] == migration["old"]}
    new_offsets = {key[2] for key in keys_of(head, inode) if key[0] == migration["new"]}
    return sorted(old_offsets - new_offsets)


def op_copy_step(world, tree_id, inode, new_locality):
    """协议 C（先抄后切，记录不带双值）：给还没有新段 key 的最小 offset 在新段加一条 key。
    乙′ / 丙′ 指向同一个物理件；甲要一份新头的新单元。"""
    head = world.heads[tree_id]
    migration = head["migration"].get(inode)
    if migration is None:
        migration = {"protocol": "copy", "old": primary_locality(head, inode), "new": new_locality, "phase": "copy"}
        head["migration"][inode] = migration
        head["rekeys"][inode] = head["rekeys"].get(inode, 0) + 1
    offset = offsets_needing_copy(head, inode, migration)[0]
    physical_id = head["extents"][(migration["old"], inode, offset)]
    set_key(head, (migration["new"], inode, offset),
            rekey_physical(world, tree_id, inode, physical_id, migration["new"], set()))
    world.publish()


def op_copy_flip(world, tree_id, inode):
    head = world.heads[tree_id]
    migration = head["migration"][inode]
    head["records"][inode]["localities"] = (migration["new"],)
    migration["phase"] = "drop"
    cow_record_container(world, head, inode)
    world.publish()


def op_copy_drop(world, tree_id, inode):
    head = world.heads[tree_id]
    migration = head["migration"][inode]
    key = min((key for key in keys_of(head, inode) if key[0] == migration["old"]), key=lambda item: item[2])
    del head["extents"][key]
    if not any(key[0] == migration["old"] for key in keys_of(head, inode)):
        del head["migration"][inode]
    world.publish()


# ---------------- 克隆、打包、只搬物理位置 ----------------
def op_clone(world, source_tree_id, clone_tree_id):
    """克隆头（D6 已定项 1 每头一棵树）：记录容器与 extent 条目按物理 id 共享，水位取 origin 那一刻；归属神谕随之继承。"""
    source = world.heads[source_tree_id]
    new_head(world, clone_tree_id)
    clone = world.heads[clone_tree_id]
    clone["watermark"] = source["watermark"]
    clone["records"] = {inode: dict(record) for inode, record in source["records"].items()}
    clone["record_container"] = dict(source["record_container"])
    clone["extents"] = dict(source["extents"])
    clone["refs"] = {inode: set(physical_ids) for inode, physical_ids in source["refs"].items()}
    clone["truth"] = {inode: dict(offsets) for inode, offsets in source["truth"].items()}
    world.publish()


def op_pack(world, tree_id, inode):
    """D26 常驻整理把一个小对象搬进一个新容器的槽：槽的字段逐字节取打包前那一版单元头（D19 已定项 6 ⑩㈠），
    槽带不带 locality 看 slot_loc；extent key 不变（走映射、不改引用者）；打包前那个码 1 单元留在盘上。"""
    head = world.heads[tree_id]
    (key,) = keys_of(head, inode)
    unit = world.units[head["extents"][key]]
    slot_locality = unit["header_locality"] if world.family["slot_loc"] == "copy" else None
    head["extents"][key] = add_physical(world, tree_id, inode, dict(unit, kind="slot", header_locality=slot_locality,
                                                                    container=world.allocate_container()))
    head["packs"][inode] = head["packs"].get(inode, 0) + 1
    world.publish()


def op_repack(world, tree_id, inode):
    """整容器重写（D27：回收死槽只能整个重写）：活槽逐字节抄进一个新容器，旧容器留在盘上。"""
    head = world.heads[tree_id]
    (key,) = keys_of(head, inode)
    slot = world.units[head["extents"][key]]
    head["extents"][key] = add_physical(world, tree_id, inode, dict(slot, container=world.allocate_container()))
    head["packs"][inode] = head["packs"].get(inode, 0) + 1
    world.publish()


def op_relocate(world, tree_id, inode, offset):
    """「后台重新聚簇」按只搬物理位置读：物理件换落点、头一个字节不变（D26 已定项 4 走映射、不改引用者），旧落点那份留在盘上。"""
    head = world.heads[tree_id]
    for key in keys_of(head, inode, offset):
        head["extents"][key] = add_physical(world, tree_id, inode, dict(world.units[head["extents"][key]]))
    head["relocations"][inode] = head["relocations"].get(inode, 0) + 1
    world.publish()


OPERATIONS = {
    "create": lambda world, operation: op_create(world, operation[1], operation[2]),
    "write": lambda world, operation: op_write(world, operation[1], operation[2], operation[3]),
    "rekey": lambda world, operation: op_rekey_atomic(world, operation[1], operation[2], operation[3]),
    "p_begin": lambda world, operation: op_migration_begin(world, operation[1], operation[2], operation[3], "perunit"),
    "n_begin": lambda world, operation: op_migration_begin(world, operation[1], operation[2], operation[3], "naive"),
    "m_step": lambda world, operation: op_migration_step(world, operation[1], operation[2]),
    "m_end": lambda world, operation: op_migration_end(world, operation[1], operation[2]),
    "c_step": lambda world, operation: op_copy_step(world, operation[1], operation[2], operation[3]),
    "c_flip": lambda world, operation: op_copy_flip(world, operation[1], operation[2]),
    "c_drop": lambda world, operation: op_copy_drop(world, operation[1], operation[2]),
    "clone": lambda world, operation: op_clone(world, operation[1], operation[2]),
    "pack": lambda world, operation: op_pack(world, operation[1], operation[2]),
    "repack": lambda world, operation: op_repack(world, operation[1], operation[2]),
    "relocate": lambda world, operation: op_relocate(world, operation[1], operation[2], operation[3]),
}


def migration_operations(head, tree_id, inode, enabled, others, fresh):
    migration = head["migration"].get(inode)
    keys = keys_of(head, inode)
    operations = []
    for protocol, begin in (("perunit", "p_begin"), ("naive", "n_begin")):
        if protocol not in enabled:
            continue
        if migration is None and keys and fresh:
            operations.extend((begin, tree_id, inode, locality) for locality in others)
        elif migration is not None and migration["protocol"] == protocol:
            if any(key[0] == migration["old"] for key in keys):
                operations.append(("m_step", tree_id, inode))
            else:
                operations.append(("m_end", tree_id, inode))
    if "copy" in enabled:
        if migration is None and keys and fresh:
            operations.extend(("c_step", tree_id, inode, locality) for locality in others)
        elif migration is not None and migration["protocol"] == "copy":
            if migration["phase"] == "copy" and offsets_needing_copy(head, inode, migration):
                operations.append(("c_step", tree_id, inode, migration["new"]))
            elif migration["phase"] == "copy":
                operations.append(("c_flip", tree_id, inode))
            else:
                operations.append(("c_drop", tree_id, inode))
    return operations


def legal_operations(world, config):
    enabled = config["ops"]
    operations = []
    for tree_id in sorted(world.heads):
        head = world.heads[tree_id]
        if head["creates"] < config["max_creates"]:
            operations.extend(("create", tree_id, locality) for locality in config["localities"])
        for inode in sorted(head["records"]):
            migration = head["migration"].get(inode)
            keys = keys_of(head, inode)
            others = [locality for locality in config["localities"] if locality != primary_locality(head, inode)]
            fresh = head["rekeys"].get(inode, 0) < config["max_rekeys"]
            if head["writes"].get(inode, 0) < config["max_writes"]:
                operations.extend(("write", tree_id, inode, offset) for offset in config["offsets"])
            if "rekey" in enabled and migration is None and keys and fresh:
                operations.extend(("rekey", tree_id, inode, locality) for locality in others)
            operations.extend(migration_operations(head, tree_id, inode, enabled, others, fresh))
            if "relocate" in enabled and head["relocations"].get(inode, 0) < config["max_relocations"]:
                operations.extend(("relocate", tree_id, inode, offset) for offset in sorted({key[2] for key in keys}))
            if "pack" in enabled and migration is None and len(keys) == 1 and keys[0][2] == 0 \
                    and head["packs"].get(inode, 0) < config["max_packs"]:
                kind = world.units[head["extents"][keys[0]]]["kind"]
                operations.append(("pack" if kind == "unit" else "repack", tree_id, inode))
    if "clone" in enabled and len(world.heads) == 1 and not any(head["migration"] for head in world.heads.values()):
        operations.append(("clone", 11, 21))
    return operations


def replay(history, family, config=None, check=True):
    """config 给了就带上它的写路径开关；check 为真时逐步核这一步在这个家族的世界里合不合法，
    走不通返回 None（例如甲把对象搬出去之后那一格已没有槽可重写）。"""
    world = World(family)
    world.copy_write = config.get("copy_write", "both") if config is not None else "both"
    new_head(world, 11)
    for operation in history:
        if config is not None and check and operation not in legal_operations(world, config):
            return None
        OPERATIONS[operation[0]](world, operation)
    return world


# ---------------- 家族（写路径怎么走）与臂（重建怎么走） ----------------
FAMILIES = {
    "plain": dict(rewrite=False, slot_loc="none", claim_has_locality=False, packed_rekey="leave", mac=False),
    "plain_slot8": dict(rewrite=False, slot_loc="copy", claim_has_locality=False, packed_rekey="leave", mac=False),
    "jia": dict(rewrite=True, slot_loc="none", claim_has_locality=False, packed_rekey="leave", mac=True),
    "jia_slot8_claim5": dict(rewrite=True, slot_loc="copy", claim_has_locality=False, packed_rekey="leave", mac=True),
    "jia_slot8_moveout": dict(rewrite=True, slot_loc="copy", claim_has_locality=True, packed_rekey="move_out", mac=True),
    # 阳性对照：槽带 locality、容器内认领比它，而重编 key 时不动槽 ⇒ 常规读就该认领失败
    "jia_slot8_claim6_leave": dict(rewrite=True, slot_loc="copy", claim_has_locality=True, packed_rekey="leave", mac=True),
}
# keying：unify 这个对象全部 key 编同一个首段；keep 幸存 key 原样、找回的编选中值（第一轮丙字面）；per_unit 按单元头编。
# grouping：extent_key 版本分组键 = 它要被放回的 extent key（I-1.8「同 key」按甲的字面读：头里的 locality 是 key 的一段）；
#   anchor 只按锚点偏移分组（不管首段、不管出生树）。order：选中值的来源次序。record：重建后记录带什么。
ARMS = {
    "bing_s1": dict(keying="unify", grouping="anchor", order=("keys", "record", "zero"), record="chosen"),
    "bing_rec": dict(keying="unify", grouping="anchor", order=("record", "keys", "zero"), record="chosen"),
    "bing_r1lit": dict(keying="keep", grouping="anchor", order=("record", "keys", "zero"), record="chosen"),
    "yi_s3": dict(keying="unify", grouping="anchor", order=("keys", "record", "header", "zero"), record="chosen"),
    "yi_unit": dict(keying="per_unit", grouping="extent_key", order=("record", "header", "zero"), record="chosen"),
    "jia_key_rec": dict(keying="per_unit", grouping="extent_key", order=("record", "keys", "zero"), record="record_first"),
    "jia_key_der": dict(keying="per_unit", grouping="extent_key", order=("keys", "record", "zero"), record="derived"),
    "jia_anc_rec": dict(keying="per_unit", grouping="anchor", order=("record", "keys", "zero"), record="record_first"),
    "jia_anc_der": dict(keying="per_unit", grouping="anchor", order=("keys", "record", "zero"), record="derived"),
    "jia_unify": dict(keying="unify", grouping="anchor", order=("keys", "record", "zero"), record="chosen"),
}
PLAIN_ARMS = ("bing_s1", "bing_rec", "bing_r1lit", "yi_s3", "yi_unit")
JIA_ARMS = ("jia_key_rec", "jia_key_der", "jia_anc_rec", "jia_anc_der", "jia_unify")


# ---------------- 损坏 ----------------
def damage_patterns(world, target_tree_id):
    """目标头：每个记录容器 在 / 丢 / 最新 k 版读不出（k = 1..ROLLBACK_DEPTH）；每个对象的 extent 条目
    在 / 全丢 / 丢某一个首段的那片叶（对象有两个首段时）/ 丢最低或最高 offset 的全部条目（对象有两个 offset 时）。"""
    head = world.heads[target_tree_id]
    container_choices = []
    for group in sorted(head["record_container"]):
        states = ["intact", "lost"]
        version = head["record_container"][group]
        for depth in range(1, ROLLBACK_DEPTH + 1):
            version = world.container_previous.get(version)
            if version is None:
                break
            states.append(("rolled_back", depth))
        container_choices.append([(group, state) for state in states])
    extent_choices = []
    for inode in sorted({key[1] for key in head["extents"]}):
        keys = keys_of(head, inode)
        states = ["intact", "all_lost"]
        segments = sorted({key[0] for key in keys})
        if len(segments) > 1:
            states.extend(("lose_segment", segment) for segment in segments)
        offsets = sorted({key[2] for key in keys})
        if len(offsets) > 1:
            states.extend([("lose_offset", offsets[0]), ("lose_offset", offsets[-1])])
        extent_choices.append([(inode, state) for state in states])
    for container_combo in itertools.product(*container_choices):
        for extent_combo in itertools.product(*extent_choices):
            yield dict(container_combo), dict(extent_combo)


def surviving_records(world, head, container_damage):
    survivors = {}
    for group, version in head["record_container"].items():
        state = container_damage[group]
        if state == "lost":
            continue
        chosen = version
        if state != "intact":
            for _ in range(state[1]):
                chosen = world.container_previous[chosen]
        survivors.update(world.container_snapshot[chosen])
    return survivors


def surviving_extents(head, extent_damage):
    surviving = {}
    for key, physical_id in head["extents"].items():
        state = extent_damage.get(key[1], "intact")
        if state == "all_lost":
            continue
        if isinstance(state, tuple) and state[0] == "lose_segment" and key[0] == state[1]:
            continue
        if isinstance(state, tuple) and state[0] == "lose_offset" and key[2] == state[1]:
            continue
        surviving[key] = physical_id
    return surviving


# ---------------- 扫描重建 ----------------
def unit_segment(unit, spec, chosen):
    if spec["keying"] == "per_unit" and unit["header_locality"] is not None:
        return unit["header_locality"]
    return chosen


def rebuild_object(world, head, inode, spec, record_tuple, own_surviving, lost_filter):
    """一个对象：选中值 → 幸存 key 放回 → 扫描到的物理件按分组择新、放回 → 重建后的记录。返回 (记录, extent, 冲突数)。
    lost_filter：None = 这个对象的 extent 条目一条没丢（重建不往它的 key 空间里补）；"all" = 整段 key 区间都读不出
    （不知道原来有哪些 key，按单元头能放回的都放回）；否则是读不出的那几条原 key 的集合——按单元头编 key 的臂
    只往这些 key 上补（对甲最有利的读法：重建知道丢的是哪几片叶的区间）。"""
    scanned = sorted(head["refs"].get(inode, ()))
    segment_counts = {}
    for key in own_surviving:
        segment_counts[key[0]] = segment_counts.get(key[0], 0) + 1
    key_choice = min(segment_counts, key=lambda segment: (-segment_counts[segment], segment)) if segment_counts else None
    with_header = [world.units[physical_id] for physical_id in scanned
                   if world.units[physical_id]["header_locality"] is not None]
    header_hint = max(with_header, key=lambda unit: unit["write_order"])["header_locality"] if with_header else None
    sources = {"keys": key_choice, "record": record_tuple[0] if record_tuple else None,
               "header": header_hint, "zero": V1_LOCALITY}
    chosen = next(sources[name] for name in spec["order"] if sources[name] is not None)
    object_extents, collisions = {}, 0

    def place(key, physical_id):
        nonlocal collisions
        present = object_extents.get(key)
        if present is None or present == physical_id:
            object_extents[key] = physical_id
            return
        if world.units[present]["content"] != world.units[physical_id]["content"]:
            collisions += 1   # 同一个 extent key 两个内容不同的现行件：规则没写怎么办，按写序择新
        if world.units[physical_id]["write_order"] > world.units[present]["write_order"]:
            object_extents[key] = physical_id

    for key, physical_id in sorted(own_surviving.items()):
        place((chosen, inode, key[2]) if spec["keying"] == "unify" else key, physical_id)
    referenced = set(own_surviving.values())
    groups = {}
    for physical_id in scanned:
        unit = world.units[physical_id]
        segment = unit_segment(unit, spec, chosen)
        group_key = (unit["anchor_offset"],) if spec["grouping"] == "anchor" else (segment, unit["anchor_offset"])
        groups.setdefault(group_key, []).append(physical_id)
    for group_key in sorted(groups) if lost_filter is not None else ():
        newest = max(groups[group_key], key=lambda physical_id: (world.units[physical_id]["write_order"], physical_id))
        if newest in referenced:
            continue
        unit = world.units[newest]
        key = (unit_segment(unit, spec, chosen), inode, unit["anchor_offset"])
        if spec["keying"] == "per_unit" and lost_filter != "all" and key not in lost_filter:
            continue
        place(key, newest)
    segments_present = tuple(sorted({key[0] for key in object_extents}))
    if spec["record"] == "chosen":
        record = (chosen,)
    elif spec["record"] == "derived":
        record = segments_present or (chosen,)
    else:
        record = record_tuple if record_tuple is not None else (segments_present or (chosen,))
    return record, object_extents, collisions


def rebuild(world, target_tree_id, container_damage, extent_damage, arm):
    """按臂的规则重建目标头：返回 (重建后的记录 locality 元组, 重建后的 extent 映射, 内容不同的 key 冲突数)。"""
    spec = ARMS[arm]
    head = world.heads[target_tree_id]
    records_left = surviving_records(world, head, container_damage)
    surviving = surviving_extents(head, extent_damage)
    rebuilt_records, rebuilt_extents, collisions = {}, {}, 0
    for inode in sorted(set(records_left) | {key[1] for key in surviving} | set(head["refs"])):
        own_surviving = {key: physical_id for key, physical_id in surviving.items() if key[1] == inode}
        state = extent_damage.get(inode, "intact")
        lost_keys = {key for key in keys_of(head, inode) if key not in surviving}
        lost_filter = None if state == "intact" else ("all" if state == "all_lost" else lost_keys)
        record, extents, object_collisions = rebuild_object(world, head, inode, spec, records_left.get(inode),
                                                            own_surviving, lost_filter)
        rebuilt_records[inode] = record
        rebuilt_extents.update(extents)
        collisions += object_collisions
    return rebuilt_records, rebuilt_extents, collisions


# ---------------- 检查 ----------------
FAILURE_FIELDS = ("missing", "misread", "mac_fail", "claim_fail", "split_new")
READ_FAILURES = ("missing", "misread", "mac_fail", "claim_fail")


def read_offset(world, record_localities, extents, inode, offset, want):
    """读者按记录里的 locality 次序找，第一个找到的算数；甲在加密开着时按查找路径的首段核 AAD（只对码 1 单元，
    槽的 AAD 绑容器身份）；槽带 locality 且认领比它时按查找路径的首段认领。"""
    family = world.family
    for locality in record_localities:
        physical_id = extents.get((locality, inode, offset))
        if physical_id is None:
            continue
        unit = world.units[physical_id]
        if unit["kind"] == "slot" and family["claim_has_locality"] and unit["header_locality"] != locality:
            return "claim_fail"
        if unit["kind"] == "unit" and family["mac"] and unit["header_locality"] != locality:
            return "mac_fail"
        return "ok" if unit["content"] == want else "misread"
    return "missing"


def check_reads(world, tree_id, rebuilt_records, rebuilt_extents):
    """读者按重建后记录里的 locality 查每个本该有数据的 offset（真值 = 用户最后一次写进去的内容）。
    misread = 读到的内容不是真值（被覆写掉的旧版、死槽）：「读到损坏前（在这个位置）没有的数据」。"""
    head = world.heads[tree_id]
    tally = dict(objects_checked=0, extents_checked=0, missing=0, misread=0, mac_fail=0, claim_fail=0,
                 split_new=0, locality_changed=0)
    for inode, truth in sorted(head["truth"].items()):
        if not truth:
            continue
        tally["objects_checked"] += 1
        record = rebuilt_records.get(inode)
        if record is None:
            tally["missing"] += len(truth)
            continue
        if set(record) != set(head["records"][inode]["localities"]):
            tally["locality_changed"] += 1
        original_segments = {key[0] for key in keys_of(head, inode)}
        rebuilt_segments = {key[0] for key in rebuilt_extents if key[1] == inode}
        if len(rebuilt_segments) > 1 and len(original_segments) <= 1:
            tally["split_new"] += 1
        for offset, content in sorted(truth.items()):
            tally["extents_checked"] += 1
            verdict = read_offset(world, record, rebuilt_extents, inode, offset, content)
            if verdict != "ok":
                tally[verdict] += 1
    return tally


def is_failure(tally):
    return any(tally[field] for field in FAILURE_FIELDS)


def normal_path_tally(world):
    """没有损坏时的常规读：协议本身对不对。阳性对照（记录直接换值、槽认领比 locality 而不动槽）在这里就该红。"""
    tally = dict.fromkeys(READ_FAILURES, 0)
    for head in world.heads.values():
        for inode, truth in head["truth"].items():
            for offset, content in truth.items():
                verdict = read_offset(world, head["records"][inode]["localities"], head["extents"], inode, offset, content)
                if verdict != "ok":
                    tally[verdict] += 1
    return tally


def i99_literal_red(world):
    """I-9.9（locality 三值对照）按字面判一个没损坏、合法的镜像：记录只许一个值，且等于该对象每条 extent key 的首段。"""
    red = 0
    for head in world.heads.values():
        for inode, record in head["records"].items():
            segments = {key[0] for key in keys_of(head, inode)}
            if segments and (len(record["localities"]) != 1 or segments != {record["localities"][0]}):
                red += 1
    return red


# ---------------- 穷举 ----------------
STAT_FIELDS = ("histories", "patterns", "objects", "failures") + FAILURE_FIELDS + ("collisions", "locality_changed")
FAMILY_FIELDS = ("histories", "skipped", "normal_fail", "i99_legal_red", "forced_rewrites", "unshared",
                 "stale_hints", "move_outs")


def run_history(depth, history, config, stats, family_stats, shortest, normal_shortest):
    for family_name, arms in config["families"]:
        world = replay(history, FAMILIES[family_name], config)
        family_stat = family_stats[family_name]
        if world is None:
            family_stat["skipped"] += 1
            continue
        family_stat["histories"] += 1
        for counter in ("forced_rewrites", "unshared", "stale_hints", "move_outs"):
            family_stat[counter] += world.counters[counter]
        normal = normal_path_tally(world)
        if any(normal.values()):
            family_stat["normal_fail"] += 1
            normal_shortest.setdefault(family_name, (depth, history, normal))
        family_stat["i99_legal_red"] += i99_literal_red(world)
        for arm in arms:
            stats[(family_name, arm)]["histories"] += 1
        for target_tree_id in sorted(world.heads):
            for container_damage, extent_damage in damage_patterns(world, target_tree_id):
                for arm in arms:
                    records, extents, collisions = rebuild(world, target_tree_id, container_damage, extent_damage, arm)
                    tally = check_reads(world, target_tree_id, records, extents)
                    stat = stats[(family_name, arm)]
                    stat["patterns"] += 1
                    stat["objects"] += tally["objects_checked"]
                    stat["collisions"] += collisions
                    stat["locality_changed"] += tally["locality_changed"]
                    for field in FAILURE_FIELDS:
                        stat[field] += tally[field]
                    if is_failure(tally):
                        stat["failures"] += 1
                        shortest.setdefault((family_name, arm), (depth, history, target_tree_id, container_damage,
                                                                 extent_damage, tally))


def explore(config, max_depth):
    stats = {(family, arm): dict.fromkeys(STAT_FIELDS, 0) for family, arms in config["families"] for arm in arms}
    family_stats = {family: dict.fromkeys(FAMILY_FIELDS, 0) for family, _arms in config["families"]}
    shortest, normal_shortest = {}, {}
    frontier = [()]
    for depth in range(1, max_depth + 1):
        next_frontier = []
        for history in frontier:
            base_world = replay(history, FAMILIES["plain"], config, check=False)
            next_frontier.extend(history + (operation,) for operation in legal_operations(base_world, config))
        frontier = next_frontier
        for history in frontier:
            run_history(depth, history, config, stats, family_stats, shortest, normal_shortest)
    open_after = sum(len(legal_operations(replay(history, FAMILIES["plain"], config, check=False), config)) for history in frontier)
    return stats, family_stats, shortest, normal_shortest, open_after


# ---------------- 世界 ----------------
BASE = dict(localities=DIRECTORY_LOCALITIES, offsets=OFFSETS, max_creates=1, max_writes=3, max_rekeys=1,
            max_relocations=0, max_packs=0)
BOTH = (("plain", PLAIN_ARMS), ("jia", JIA_ARMS))
WORLDS = (
    # 重编 key，协议 A：对象装得进一次发布。第二个家族让甲的五种读法跑在「不重写单元」的写路径上（加密关着、没有 AAD 核它）。
    ("atomic", dict(BASE, ops={"rekey"}, max_rekeys=2, depth=7,
                    families=(("plain", PLAIN_ARMS + JIA_ARMS), ("jia", JIA_ARMS)))),
    # 重编 key，协议 P：逐单元跨发布，记录带双值 (L_new, L_old)。
    ("perunit", dict(BASE, ops={"perunit"}, depth=8, families=BOTH)),
    # 阳性对照：协议 N（记录直接换成 L_new、不带旧值）常规读就该找不到还没搬的那条。
    ("naive", dict(BASE, ops={"naive"}, depth=8, families=(("plain", ("bing_s1",)), ("jia", ("jia_anc_der",))))),
    # 重编 key，协议 C：先抄后切，记录不带双值（乙′ / 丙′ 的新 key 指同一个单元；甲要抄一份新头的单元）。
    ("copy", dict(BASE, ops={"copy"}, depth=9, families=BOTH)),
    # 同一协议，只把迁移期间的用户写换成「只写主段、另一段作废重抄」（甲不再一次写两份单元）。
    ("copy_recopy", dict(BASE, ops={"copy"}, copy_write="recopy", depth=10, families=BOTH)),
    # 只搬物理位置：头一个字节不变，locality 创建之后永不改。
    ("physical", dict(BASE, ops={"relocate"}, max_relocations=2, depth=7, families=BOTH)),
    # 同一个记录容器里两个对象，其中一个逐单元重编 key（容器回退连带邻居）。
    ("perunit2", dict(BASE, ops={"perunit"}, max_creates=2, max_writes=2, depth=7, families=BOTH)),
    # D27 打包：槽 +0 / +8，甲的三种槽政策 + 阳性对照；重编 key 用协议 A 与协议 P。
    ("pack", dict(BASE, ops={"pack", "rekey", "perunit"}, offsets=(0,), max_packs=2, max_writes=2, depth=8,
                  families=(("plain", PLAIN_ARMS), ("plain_slot8", PLAIN_ARMS), ("jia", JIA_ARMS),
                            ("jia_slot8_claim5", JIA_ARMS), ("jia_slot8_moveout", JIA_ARMS),
                            ("jia_slot8_claim6_leave", ("jia_anc_der",))))),
    # 克隆之后在任一头里逐单元重编 key（协议 P）。
    ("clone", dict(BASE, ops={"perunit", "clone"}, max_writes=2, depth=7, families=BOTH)),
    # 第一版：locality 恒 0，打包与克隆都开。
    ("v1", dict(BASE, localities=(V1_LOCALITY,), ops={"pack", "clone"}, max_creates=2, max_packs=1, max_writes=2,
                depth=7, families=(("plain_slot8", PLAIN_ARMS), ("jia_slot8_claim5", JIA_ARMS)))),
)


def describe_history(history):
    return " ; ".join("%s(%s)" % (operation[0], ",".join(str(part) for part in operation[1:])) for operation in history)


def describe_damage(container_damage, extent_damage):
    return "containers=%s extents=%s" % (sorted(container_damage.items(), key=str), sorted(extent_damage.items(), key=str))


def print_world(label, config, max_depth):
    stats, family_stats, shortest, normal_shortest, open_after = explore(config, max_depth)
    print("coverage", label, "max_depth=%d" % max_depth, "histories_one_step_longer=%d" % open_after,
          "(0 = 在这个世界的操作上限之内穷举完了)")
    for family, _arms in config["families"]:
        family_stat = family_stats[family]
        print("family", label, family, max_depth, " ".join("%s=%d" % (field, family_stat[field]) for field in FAMILY_FIELDS))
        if family in normal_shortest:
            depth, history, normal = normal_shortest[family]
            print("normal_shortest", label, family, "depth=%d" % depth, "history=[%s]" % describe_history(history),
                  "tally=%s" % ",".join("%s=%d" % item for item in sorted(normal.items())))
    for family, arms in config["families"]:
        for arm in arms:
            stat = stats[(family, arm)]
            print("row", label, family, arm, max_depth, " ".join(str(stat[field]) for field in STAT_FIELDS))
    for family, arms in config["families"]:
        for arm in arms:
            if (family, arm) not in shortest:
                print("shortest", label, family, arm, "none_through_depth=%d" % max_depth)
                continue
            depth, history, tree_id, container_damage, extent_damage, tally = shortest[(family, arm)]
            print("shortest", label, family, arm, "depth=%d" % depth, "head=%d" % tree_id,
                  "history=[%s]" % describe_history(history), describe_damage(container_damage, extent_damage),
                  "tally=%s" % ",".join("%s=%d" % (field, tally[field]) for field in FAILURE_FIELDS))
    return stats, family_stats


def main():
    depth_override = int(sys.argv[1]) if len(sys.argv) > 1 else None
    only = sys.argv[2].split(",") if len(sys.argv) > 2 else None
    print("# D18 locality_id 第二轮攻方腿模型输出（确定性）RECORDS_PER_CONTAINER=%d OFFSETS=%s ROLLBACK_DEPTH=%d"
          % (RECORDS_PER_CONTAINER, OFFSETS, ROLLBACK_DEPTH))
    if only is None:
        hand_built_cases()
    print("## 穷举：每条历史 × 每个家族 × 每个头 × 全部损坏形态 × 每条臂")
    print("columns(row): world family arm max_depth " + " ".join(STAT_FIELDS))
    for label, config in WORLDS:
        if only is not None and label not in only:
            continue
        print_world(label, config, depth_override or config["depth"])
    print("done")


def expect(label, actual, wanted):
    verdict = "ok" if actual == wanted else "FAIL"
    print("assert %-66s got=%-8s want=%-8s %s" % (label, actual, wanted, verdict))
    if actual != wanted:
        raise SystemExit("断言失败：" + label)


def case(history, family_name, arm, tree_id, container_damage, extent_damage):
    world = replay(history, FAMILIES[family_name])
    containers = {group: "intact" for group in world.heads[tree_id]["record_container"]}
    containers.update(container_damage)
    records, extents, _collisions = rebuild(world, tree_id, containers, extent_damage, arm)
    return records, extents, check_reads(world, tree_id, records, extents)


def failures(tally):
    return sum(tally[field] for field in FAILURE_FIELDS)


def hand_built_cases():
    print("## 手造历史（钉绝对值；每条要么是最短反例、要么是阳性 / 阴性对照）")
    lose_all = {1: "all_lost"}
    stale = (("create", 11, 1), ("write", 11, 1, 0), ("rekey", 11, 1, 2), ("write", 11, 1, 0))
    records, _e, tally = case(stale, "jia", "jia_key_der", 11, {}, lose_all)
    expect("A 甲（版本分组键 = extent key）：重编后覆写、extent 全丢，读到旧版数", tally["misread"], 1)
    expect("A 同上：一个对象两个首段", tally["split_new"], 1)
    expect("A 同上：重建后的记录", records[1], (1, 2))
    _r, _e, tally = case(stale, "jia", "jia_key_rec", 11, {}, lose_all)
    expect("A 甲（extent key 分组、记录优先）：(读到旧版, 两个首段)", (tally["misread"], tally["split_new"]), (0, 1))
    for family, arm in (("jia", "jia_anc_der"), ("jia", "jia_anc_rec"), ("plain", "bing_s1"), ("plain", "bing_rec"),
                        ("plain", "yi_s3")):
        _r, _e, tally = case(stale, family, arm, 11, {}, lose_all)
        expect("A 同一历史 %s：失败字段合计" % arm, failures(tally), 0)

    copy_mid = (("create", 11, 1), ("write", 11, 1, 0), ("c_step", 11, 1, 2))
    _r, _e, tally = case(copy_mid, "jia", "jia_anc_rec", 11, {}, lose_all)
    expect("B 甲（只按锚点分组、记录优先）：先抄后切抄了一份、extent 全丢，找不到数", tally["missing"], 1)
    records, _e, tally = case(copy_mid, "jia", "jia_anc_der", 11, {}, lose_all)
    expect("B 甲（只按锚点分组、记录取重建出的首段集合）：找不到数", tally["missing"], 0)
    expect("B 同上：重建后的记录", records[1], (2,))
    for family, arm in (("jia", "jia_key_rec"), ("plain", "bing_s1"), ("plain", "bing_rec"), ("plain", "yi_s3")):
        _r, _e, tally = case(copy_mid, family, arm, 11, {}, lose_all)
        expect("B 同一历史 %s：失败字段合计" % arm, failures(tally), 0)

    dual = (("create", 11, 2), ("write", 11, 1, 0), ("c_step", 11, 1, 1), ("write", 11, 1, 0), ("c_flip", 11, 1),
            ("write", 11, 1, 1), ("c_drop", 11, 1))
    records, _e, tally = case(dual, "jia", "jia_anc_der", 11, {}, {1: ("lose_offset", 0)})
    expect("B′ 甲最强读法：先抄后切期间写了两份、翻记录、删旧段，丢 offset 0 的叶，找不到数", tally["missing"], 1)
    expect("B′ 同上：重建后的记录", records[1], (1,))
    _r, _e, tally = case(dual, "jia", "jia_anc_der", 11, {}, lose_all)
    expect("B′ 同一历史 extent 全丢：(找不到, 两个首段)", (tally["missing"], tally["split_new"]), (0, 1))
    for family, arm in (("plain", "bing_s1"), ("plain", "bing_rec"), ("plain", "yi_s3")):
        for extent_damage in ({1: ("lose_offset", 0)}, lose_all):
            _r, _e, tally = case(dual, family, arm, 11, {}, extent_damage)
            expect("B′ 同一历史 %s %s：失败字段合计" % (arm, sorted(extent_damage.values(), key=str)), failures(tally), 0)

    rolled = (("create", 11, 1), ("write", 11, 1, 0), ("rekey", 11, 1, 2))
    back = {0: ("rolled_back", 1)}
    _r, _e, tally = case(rolled, "jia", "jia_anc_rec", 11, back, {})
    expect("C 甲（记录优先）：记录容器回到上一版、extent 完好，找不到数", tally["missing"], 1)
    _r, _e, tally = case(rolled, "plain", "bing_r1lit", 11, back, {})
    expect("C 阳性对照（第一轮打中 1）：幸存 key 不重编的丙字面读法找不到数", tally["missing"], 1)
    for family, arm in (("jia", "jia_anc_der"), ("plain", "bing_rec"), ("plain", "bing_s1"), ("plain", "yi_s3")):
        _r, _e, tally = case(rolled, family, arm, 11, back, {})
        expect("C 同一历史 %s：失败字段合计" % arm, failures(tally), 0)
    records, _e, tally = case(rolled, "plain", "bing_rec", 11, back, {})
    expect("C 丙（记录优先 + 全部 key 重编）：重建后的记录与局部性变化数", (records[1], tally["locality_changed"]), ((1,), 1))

    packed = (("create", 11, 1), ("write", 11, 1, 0), ("pack", 11, 1), ("rekey", 11, 1, 2))
    world = replay(packed, FAMILIES["jia_slot8_claim6_leave"])
    expect("D 阳性对照：槽带 locality、认领比它、重编时不动槽，常规读认领失败数", normal_path_tally(world)["claim_fail"], 1)
    world = replay(packed, FAMILIES["jia_slot8_moveout"])
    expect("D 甲把对象搬出去：常规读失败数", sum(normal_path_tally(world).values()), 0)
    expect("D 同上：搬出去的 ≤ 4 KiB 对象数（每个换一个 32 KiB 码 1 单元）", world.counters["move_outs"], 1)
    world = replay(packed, FAMILIES["jia_slot8_claim5"])
    expect("D 槽带 locality、认领不比它：重编后槽里过期的 locality 数", world.counters["stale_hints"], 1)
    _r, _e, tally = case(packed, "jia_slot8_claim5", "jia_anc_rec", 11, {}, lose_all)
    expect("D 同上、记录优先：extent 全丢后找不到数", tally["missing"], 1)
    _r, _e, tally = case(packed, "jia", "jia_key_der", 11, {}, lose_all)
    expect("D 甲槽不带 locality、extent key 分组：打包前单元与槽落到两个首段", tally["split_new"], 1)

    migrate = (("create", 11, 1), ("write", 11, 1, 0), ("write", 11, 1, 1), ("p_begin", 11, 1, 2),
               ("m_step", 11, 1), ("m_step", 11, 1), ("m_end", 11, 1))
    reds = tuple(i99_literal_red(replay(migrate[:length], FAMILIES["plain"])) for length in range(3, 8))
    expect("E 协议 P：写完 / 开始 / 搬一条 / 搬两条 / 结束 各发布上 I-9.9 字面判红", reds, (0, 1, 1, 1, 0))
    expect("E 甲：两单元对象逐单元重编，被迫重写的单元数", replay(migrate, FAMILIES["jia"]).counters["forced_rewrites"], 2)
    expect("E 乙′ / 丙′：同一历史被迫重写的单元数", replay(migrate, FAMILIES["plain"]).counters["forced_rewrites"], 0)
    half = migrate[:5]
    _r, _e, tally = case(half, "jia", "jia_anc_rec", 11, {0: ("rolled_back", 1)}, lose_all)
    expect("E′ 甲（记录优先）：搬了一条时记录回到开始之前、extent 全丢，找不到数", tally["missing"], 1)
    for family, arm in (("jia", "jia_anc_der"), ("plain", "bing_s1"), ("plain", "bing_rec"), ("plain", "yi_s3")):
        _r, _e, tally = case(half, family, arm, 11, {0: ("rolled_back", 1)}, lose_all)
        expect("E′ 同一历史 %s：失败字段合计" % arm, failures(tally), 0)

    naive = (("create", 11, 1), ("write", 11, 1, 0), ("n_begin", 11, 1, 2))
    expect("F 阳性对照：协议 N（记录直接换值）常规读找不到数", normal_path_tally(replay(naive, FAMILIES["plain"]))["missing"], 1)
    _r, _e, tally = case(rolled[:2], "jia", "jia_unify", 11, {0: "lost"}, lose_all)
    expect("G 阴性对照：甲若照丙把首段重编成 0（记录与 extent 都丢），AAD 失配数", tally["mac_fail"], 1)
    moved = (("create", 11, 1), ("write", 11, 1, 0), ("relocate", 11, 1, 0), ("relocate", 11, 1, 0))
    for family, arms in BOTH:
        expect("H 只搬物理位置：%s 被迫重写的单元数" % family, replay(moved, FAMILIES[family]).counters["forced_rewrites"], 0)
        for arm in arms:
            if arm != "jia_unify":
                _r, _e, tally = case(moved, family, arm, 11, {0: "lost"}, lose_all)
                expect("H 只搬物理位置、记录与 extent 都丢：%s 失败字段合计" % arm, failures(tally), 0)
    u4_cases()
    slot_capacity_cases()


def header_key_mismatches(world, extents):
    """甲 / 乙′ 盘上才有的一条见证：extent key 的首段 vs 它指的码 1 单元头里的 locality（丙′ 盘上没有这一格）。"""
    return sum(1 for key, physical_id in extents.items()
               if world.units[physical_id]["kind"] == "unit" and world.units[physical_id]["header_locality"] is not None
               and world.units[physical_id]["header_locality"] != key[0])


def u4_cases():
    print("## U4 记录丢了、extent 树没丢：往 extent key 里注入一个错的首段（写路径 bug：key 编到 2，记录与单元头都说 1）")
    base = (("create", 11, 1), ("write", 11, 1, 0))
    outcome = {}
    for family, arm in (("plain", "bing_s1"), ("plain", "yi_s3"), ("jia", "jia_anc_der")):
        world = replay(base, FAMILIES[family])
        head = world.heads[11]
        head["extents"][(2, 1, 0)] = head["extents"].pop((1, 1, 0))
        normal = normal_path_tally(world)
        before = i99_literal_red(world)
        records, extents, _collisions = rebuild(world, 11, {0: "lost"}, {}, arm)
        tally = check_reads(world, 11, records, extents)
        from_keys = records[1] == tuple(sorted({key[0] for key in extents if key[1] == 1}))
        witness = header_key_mismatches(world, extents) if arm != "bing_s1" else "n/a"
        outcome[arm] = (normal["missing"], before, tally["missing"], tally["mac_fail"], from_keys, witness)
        print("U4 %-5s %-11s before_normal_missing=%d before_i99_red=%d rebuilt_record=%s after_missing=%d after_mac_fail=%d "
              "record_from_keys=%s header_vs_key_mismatch=%s" % (family, arm, normal["missing"], before, records[1],
                                                                tally["missing"], tally["mac_fail"], from_keys, witness))
    expect("U4 丙′：(注入后常规读找不到, I-9.9 字面红, 重建后找不到, AAD 失配, 记录取自 key, 见证)", outcome["bing_s1"],
           (1, 1, 0, 0, True, "n/a"))
    expect("U4 乙′：同上（见证 = 单元头 vs key）", outcome["yi_s3"], (1, 1, 0, 0, True, 1))
    expect("U4 甲：同上（重建后读路径 AAD 失配，见证 = 单元头 vs key）", outcome["jia_anc_der"], (1, 1, 0, 1, True, 1))


def slot_capacity_cases():
    print("## U3 打包容器槽 +8（E114 口径：容器头 107、每槽全额自描述 47 = 五元组 33 + 写序 10 + 槽表 4；推算，没重跑 E114）")
    wanted = {64: (294, 274), 512: (58, 57), 1024: (30, 30), 4096: (7, 7)}
    for object_bytes in sorted(wanted):
        today = (32768 - 107) // (object_bytes + 47)
        plus8 = (32768 - 107) // (object_bytes + 55)
        expect("U3 %4d 字节对象：一个容器装几个（今天, 槽 +8）" % object_bytes, (today, plus8), wanted[object_bytes])
    print("## U3 一次发布装得下多大的对象的重编 key（推算：T_dirty 有效值 256 MiB（D16 已定项 5），extent 叶 16 KiB、"
          "头 163、记录 112 ⇒ 每叶 144 条；映射叶扇出 296（D19 已定项 6）；每条 key 换首段按删一处、插一处各摊一片叶）")
    dirty_budget = 256 * 1024 * 1024
    extent_leaf_records = (16384 - 163) // 112
    per_extent_metadata = 2 * 16384 / extent_leaf_records
    per_unit_jia = 32768 + per_extent_metadata + 2 * 16384 / 296
    extents_bing = int(dirty_budget // per_extent_metadata)
    units_jia = int(dirty_budget // per_unit_jia)
    print("U3 extent_leaf_records=%d per_extent_metadata_bytes=%.1f per_unit_jia_bytes=%.1f extents_per_publish_bing=%d "
          "units_per_publish_jia=%d" % (extent_leaf_records, per_extent_metadata, per_unit_jia, extents_bing, units_jia))
    expect("U3 extent 叶每叶条数", extent_leaf_records, 144)
    expect("U3 一次发布里乙′ / 丙′ 能重编的 extent 数（推算）", extents_bing, 1179648)
    expect("U3 一次发布里甲能重编的单元数（推算）", units_jia, 8108)


if __name__ == "__main__":
    main()
