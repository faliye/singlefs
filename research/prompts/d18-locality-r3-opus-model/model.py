#!/usr/bin/env python3
# D18（块里携带什么信息） 已定项 3 那条提议（自描述头加 locality_id）第三轮，攻方腿（Opus）的模型。只用 std。
# 复跑（仓根）：
#   nice -n 19 python3 -B research/prompts/d18-locality-r3-opus-model/model.py > /tmp/claude-1000/d18-locality-r3-opus/out.txt
#   diff /tmp/claude-1000/d18-locality-r3-opus/out.txt research/prompts/d18-locality-r3-opus-model/output.txt
# 口径：确定性模型（无随机源、无 I/O），跑 N 遍与跑 1 遍信息量相同；证据强度来自钉绝对值的断言、阳性对照与检查的判别力自证。
# 与前两轮模型（research/prompts/d18-locality-r{1,2}-opus-model/，只读、没改、没拷）的差别：
#   ① 单元归属不再当神谕：扫描候选 = 同对象号、按祖先表对这个头可见、按实例表判已发布（C113 定案 P2 的全局谓词）、
#      没被死亡写序更大的墓碑杀掉的物理件；实例表与祖先表都可以读不出（实例表读不出 ⇒ 一条行都没有 ⇒ 旧实例全部
#      「无行 ⇒ 已发布」，D18 已定项 11；祖先表读不出 ⇒ 克隆头把 origin 的全部件与墓碑都当成可见）。
#   ② 删除与截断写墓碑区间记录（D18 已定项 10：对象 ID + 对象出生代 + 区间，带死亡写序），旧单元留在盘上。
#   ③ 管理员回退（D23 已定项 14 的显式例外：新实例，回退行 (r_old, T_old, 0) 与中间实例 (i, 0, 0)，新根 txg = 最大 + 1）
#      与崩溃（最后一次发布没落、它的单元成孤儿，恢复写 (i, 所选根 txg, 0)、换实例、txg 重发）。回退与崩溃都让
#      inode 号水位退回去（C143），号会被重发。
#   ④ 读者两种读法：R-view 读重建视图里的记录值；R-disk 在记录容器没坏时读盘上那条权威记录（D18 已定项 5 纪律 3：
#      权威态不得由重建结果就地修复），容器坏了才读重建出来的值。
#   ⑤ 甲-S 的打包对象规则两种读法：p2 = 只有被选中的版本是槽时用丙′ 的规则；p1 = ≤ 4 KiB 的对象全部 key 用丙′ 的规则。
# 没建的：reflink、实例切换（它按新写序重做同一批事务，孤儿与重做件同内容同首段）、同一次发布里几个事务之间的中间态；
#   删除 / 截断只在对象不在迁移中时做；每容器 2 条记录、每对象 2 个 offset、locality 两个值是小取样点，不是真值。
import copy
import itertools
import sys

RECORDS_PER_CONTAINER = 2   # 真值 233（D8 已定项 6）；取 2 让「丢一个容器连带丢邻居」在短历史里出现
OFFSETS = (0, 1)
LOCALITIES = (1, 2)
ZERO_LOCALITY = 0           # 两样都没有时取 0（第一版无父目录也取 0）
ROLLBACK_DEPTH = 2          # 记录容器当前那条链上最新的 k 版读不出，k 取 1..2
ORIGIN_TREE, CLONE_TREE = 11, 21
INFINITE_OFFSET = 1 << 30


class World:
    """一次历史跑完之后的盘面。物理件、墓碑、记录容器的每一版写出后都不可变、永不从盘上消失。"""

    def __init__(self, family):
        self.family = family      # rewrite：甲的写路径（重编 key 就重写单元）；copy_write：'recopy'（S8）/ 'both'；small：对象 ≤ 4 KiB
        self.txg = 0              # 最近一次发布的 checkpoint_txg
        self.max_txg = 0
        self.instance = 1
        self.max_instance = 1
        self.next_txn = 1
        self.next_id = 1
        self.next_content = 1
        self.items = {}           # 物理件 id -> 码 1 单元或容器里的槽
        self.tombstones = []      # 墓碑区间记录
        self.cversions = {}       # 记录容器每一版 id -> 版本
        self.chist = {}           # (树, 容器组) -> 这个头的容器指针取过的全部版本（含被回退抛弃的、克隆继承的）
        self.heads = {}
        self.rows = {}            # 当前根那一版实例表：实例 -> (T_pub, W)
        self.roots = []           # 当前时间线上每次发布之后的逻辑态
        self.clone_txg = None
        self.last_kind = None
        self.counters = dict(forced_rewrites=0, unshared=0, orphan_items=0, double_writes=0, rollbacks=0, crashes=0)

    def new_id(self):
        value = self.next_id
        self.next_id += 1
        return value

    def new_write_order(self):
        """一事务一单元（D16 已定项 5）：每写一个单元、每条墓碑取一个新写序 (实例代号, 事务号)。"""
        value = (self.instance, self.next_txn)
        self.next_txn += 1
        return value

    def new_content(self):
        value = self.next_content
        self.next_content += 1
        return value

    def open_txg(self):
        return self.txg + 1

    def publish(self, kind):
        self.txg += 1
        self.max_txg = max(self.max_txg, self.txg)
        self.last_kind = kind
        self.roots.append(dict(txg=self.txg, instance=self.instance, heads=copy.deepcopy(self.heads),
                               rows=dict(self.rows), clone_txg=self.clone_txg))


def new_head(world, tree_id):
    world.heads[tree_id] = dict(tree=tree_id, watermark=1, creates=0, records={}, cptr={}, extents={}, truth={},
                                ever={}, migration={}, writes={}, rekeys={}, packs={}, truncs={}, deletes=0)


def container_group(inode):
    return (inode - 1) // RECORDS_PER_CONTAINER


def cow_record_container(world, head, inode):
    """改一条记录 = COW 它所在的容器（D8 已定项 6）；码 3 的择新键 (诞生代号, 实例代号, 事务号)。旧版留在盘上。"""
    group = container_group(inode)
    version_id = world.new_id()
    world.cversions[version_id] = dict(
        group=group, tree=head["tree"], txg=world.open_txg(), write_order=world.new_write_order(),
        parent=head["cptr"].get(group),
        snapshot={member: copy.deepcopy(record) for member, record in head["records"].items()
                  if container_group(member) == group})
    head["cptr"][group] = version_id
    world.chist.setdefault((head["tree"], group), []).append(version_id)


def write_unit(world, tree_id, inode, offset, header_locality, content):
    head = world.heads[tree_id]
    item_id = world.new_id()
    world.items[item_id] = dict(kind="unit", tree=tree_id, object=inode, object_birth=head["records"][inode]["birth"],
                                anchor=offset, header_locality=header_locality, write_order=world.new_write_order(),
                                birth_txg=world.open_txg(), content=content)
    return item_id


def rewrite_unit_like(world, tree_id, item, header_locality):
    """甲的重写：同一份内容换一个头（新 locality、新写序、出生树取做重写的这个头），整单元重加密成新的码 1 单元。"""
    item_id = world.new_id()
    world.items[item_id] = dict(item, kind="unit", tree=tree_id, header_locality=header_locality,
                                write_order=world.new_write_order(), birth_txg=world.open_txg())
    return item_id


def is_published(world, table_readable, instance, birth_txg, transaction, code1):
    """C113 定案 P2 的全局已发布谓词（I-1.2）：码 1 看 b ≤ T_pub ∨ n ≤ W，码 3 只看 b ≤ T_pub。
    实例表读不出时一条行都没有：旧实例全部「无行 ⇒ 已发布」（D18 已定项 11「表不可读时」）。"""
    if instance == world.instance:
        return birth_txg <= world.txg
    if instance > world.instance:
        return False
    row = world.rows.get(instance) if table_readable else None
    if row is None:
        return True
    published_txg, applied_transaction = row
    return birth_txg <= published_txg or (code1 and transaction <= applied_transaction)


def keys_of(head, inode, offset=None):
    return sorted(key for key in head["extents"] if key[1] == inode and (offset is None or key[2] == offset))


def primary_locality(head, inode):
    return head["records"][inode]["loc"][0]


# ---------------- 用户操作与三种「重编 key」协议（形状同第二轮模型） ----------------
def op_create(world, tree_id, locality):
    head = world.heads[tree_id]
    inode = head["watermark"]
    head["watermark"] += 1
    head["creates"] += 1
    head["records"][inode] = dict(loc=(locality,), birth=world.open_txg(), alive=True)
    head["truth"][inode] = {}
    head["ever"].setdefault(inode, set())
    cow_record_container(world, head, inode)
    world.publish("create")


def write_segments(head, inode, offset):
    existing = [key[0] for key in keys_of(head, inode, offset)]
    if existing:
        return existing
    migration = head["migration"].get(inode)
    if migration is not None and migration["protocol"] == "perunit":
        return [migration["new"]]
    return [primary_locality(head, inode)]


def op_write(world, tree_id, inode, offset):
    """写一个 offset；打包对象上的写就是「搬出去」（D27：走未改动的独立单元写路径写一个新单元，容器不动）。"""
    head = world.heads[tree_id]
    content = world.new_content()
    segments = write_segments(head, inode, offset)
    migration = head["migration"].get(inode)
    if migration is not None and migration["protocol"] == "copy" and world.family["copy_write"] == "recopy":
        segments = [primary_locality(head, inode)]   # S8：只写主段，另一段那条 key 作废、之后重抄
    for key in keys_of(head, inode, offset):
        del head["extents"][key]
    if world.family["rewrite"]:
        for segment in segments:   # 甲：每个首段一份单元，头里的 locality 等于那个首段（AAD 绑它）
            head["extents"][(segment, inode, offset)] = write_unit(world, tree_id, inode, offset, segment, content)
        if len(segments) > 1:
            world.counters["double_writes"] += 1
            world.counters["forced_rewrites"] += len(segments) - 1
    else:
        primary = primary_locality(head, inode)
        header_locality = primary if primary in segments else segments[0]
        item_id = write_unit(world, tree_id, inode, offset, header_locality, content)
        for segment in segments:
            head["extents"][(segment, inode, offset)] = item_id
    head["truth"][inode][offset] = content
    head["ever"][inode].add(offset)
    head["writes"][inode] = head["writes"].get(inode, 0) + 1
    if migration is not None and migration["protocol"] == "copy" and migration["phase"] == "drop" \
            and not any(key[0] == migration["old"] for key in keys_of(head, inode)):
        del head["migration"][inode]
    world.publish("write")


def other_heads_reference(world, tree_id):
    referenced = set()
    for other_tree_id, other_head in world.heads.items():
        if other_tree_id != tree_id:
            referenced.update(other_head["extents"].values())
    return referenced


def rekey_item(world, tree_id, item_id, new_locality, shared_elsewhere):
    """甲-S：不论加不加密，重编 key 都重写被重编的码 1 单元；槽不带 locality（槽 +0）不动。乙′ / 丙′ 不动物理件。"""
    item = world.items[item_id]
    if item["kind"] == "unit" and world.family["rewrite"]:
        world.counters["forced_rewrites"] += 1
        if item_id in shared_elsewhere:
            world.counters["unshared"] += 1
        return rewrite_unit_like(world, tree_id, item, new_locality)
    return item_id


def op_rekey_atomic(world, tree_id, inode, new_locality):
    """协议 A：一次发布里把记录与这个对象全部 extent key 换首段（对象装得进一次发布时）。"""
    head = world.heads[tree_id]
    head["records"][inode]["loc"] = (new_locality,)
    cow_record_container(world, head, inode)
    shared = other_heads_reference(world, tree_id)
    for key in keys_of(head, inode):
        item_id = head["extents"].pop(key)
        head["extents"][(new_locality, inode, key[2])] = rekey_item(world, tree_id, item_id, new_locality, shared)
    head["rekeys"][inode] = head["rekeys"].get(inode, 0) + 1
    world.publish("rekey")


def op_perunit_begin(world, tree_id, inode, new_locality):
    """协议 P：记录先带双值 (L_new, L_old)，每次发布搬一条 key，最后收成单值；读者两个值都试。"""
    head = world.heads[tree_id]
    old_locality = primary_locality(head, inode)
    head["migration"][inode] = dict(protocol="perunit", old=old_locality, new=new_locality, phase="move")
    head["records"][inode]["loc"] = (new_locality, old_locality)
    cow_record_container(world, head, inode)
    head["rekeys"][inode] = head["rekeys"].get(inode, 0) + 1
    world.publish("p_begin")


def op_perunit_step(world, tree_id, inode):
    head = world.heads[tree_id]
    migration = head["migration"][inode]
    key = min((key for key in keys_of(head, inode) if key[0] == migration["old"]), key=lambda item: item[2])
    item_id = head["extents"].pop(key)
    head["extents"][(migration["new"], inode, key[2])] = rekey_item(world, tree_id, item_id, migration["new"],
                                                                    other_heads_reference(world, tree_id))
    world.publish("p_step")


def op_perunit_end(world, tree_id, inode):
    head = world.heads[tree_id]
    migration = head["migration"].pop(inode)
    head["records"][inode]["loc"] = (migration["new"],)
    cow_record_container(world, head, inode)
    world.publish("p_end")


def offsets_needing_copy(head, inode, migration):
    old_offsets = {key[2] for key in keys_of(head, inode) if key[0] == migration["old"]}
    new_offsets = {key[2] for key in keys_of(head, inode) if key[0] == migration["new"]}
    return sorted(old_offsets - new_offsets)


def op_copy_step(world, tree_id, inode, new_locality):
    """协议 C（先抄后切，记录单值）：给还没有新段 key 的最小 offset 在新段加一条 key；甲抄一份新头的单元。"""
    head = world.heads[tree_id]
    migration = head["migration"].get(inode)
    if migration is None:
        migration = dict(protocol="copy", old=primary_locality(head, inode), new=new_locality, phase="copy")
        head["migration"][inode] = migration
        head["rekeys"][inode] = head["rekeys"].get(inode, 0) + 1
    offset = offsets_needing_copy(head, inode, migration)[0]
    item_id = head["extents"][(migration["old"], inode, offset)]
    head["extents"][(migration["new"], inode, offset)] = rekey_item(world, tree_id, item_id, migration["new"], set())
    world.publish("c_step")


def op_copy_flip(world, tree_id, inode):
    head = world.heads[tree_id]
    migration = head["migration"][inode]
    head["records"][inode]["loc"] = (migration["new"],)
    migration["phase"] = "drop"
    cow_record_container(world, head, inode)
    world.publish("c_flip")


def op_copy_drop(world, tree_id, inode):
    head = world.heads[tree_id]
    migration = head["migration"][inode]
    key = min((key for key in keys_of(head, inode) if key[0] == migration["old"]), key=lambda item: item[2])
    del head["extents"][key]
    if not any(key[0] == migration["old"] for key in keys_of(head, inode)):
        del head["migration"][inode]
    world.publish("c_drop")


# ---------------- 删除、截断（写墓碑）、打包、克隆、回退、崩溃 ----------------
def write_tombstone(world, tree_id, inode, low_offset):
    record = world.heads[tree_id]["records"][inode]
    world.tombstones.append(dict(tree=tree_id, object=inode, object_birth=record["birth"], low=low_offset,
                                 high=INFINITE_OFFSET, death_order=world.new_write_order(), birth_txg=world.open_txg()))


def op_truncate(world, tree_id, inode, low_offset):
    """截断到 low_offset：删掉 offset ≥ low_offset 的 key，写一条区间墓碑（D18 已定项 10「每段连续 extent 区间一条」）。"""
    head = world.heads[tree_id]
    for key in keys_of(head, inode):
        if key[2] >= low_offset:
            del head["extents"][key]
    write_tombstone(world, tree_id, inode, low_offset)
    for offset in [offset for offset in head["truth"][inode] if offset >= low_offset]:
        del head["truth"][inode][offset]
    head["truncs"][inode] = head["truncs"].get(inode, 0) + 1
    world.publish("truncate")


def op_delete(world, tree_id, inode):
    """整个删除：全区间一条墓碑；记录留在 inode 树里（nlink = 0，D8 已定项 6「回收完成前留在 inode 树里」）。"""
    head = world.heads[tree_id]
    for key in keys_of(head, inode):
        del head["extents"][key]
    write_tombstone(world, tree_id, inode, 0)
    head["records"][inode]["alive"] = False
    cow_record_container(world, head, inode)
    head["truth"][inode] = {}
    head["deletes"] += 1
    world.publish("delete")


def op_pack(world, tree_id, inode):
    """D26 常驻整理把一个 ≤ 4 KiB 对象搬进一个新容器的槽：槽的字段逐字节取打包前那一版单元头（D19 已定项 6 ⑩㈠），
    槽不带 locality；extent key 不变（走映射、不改引用者）；打包前那个码 1 单元留在盘上。槽按码 3 判已发布。"""
    head = world.heads[tree_id]
    (key,) = keys_of(head, inode)
    item = world.items[head["extents"][key]]
    item_id = world.new_id()
    world.items[item_id] = dict(item, kind="slot", header_locality=None, container_instance=world.instance,
                                container_txg=world.open_txg())
    head["extents"][key] = item_id
    head["packs"][inode] = head["packs"].get(inode, 0) + 1
    world.publish("pack")


def op_repack(world, tree_id, inode):
    """整容器重写（D27：回收死槽只能整个重写）：活槽逐字节抄进一个新容器，旧容器留在盘上。"""
    head = world.heads[tree_id]
    (key,) = keys_of(head, inode)
    item_id = world.new_id()
    world.items[item_id] = dict(world.items[head["extents"][key]], container_instance=world.instance,
                                container_txg=world.open_txg())
    head["extents"][key] = item_id
    head["packs"][inode] = head["packs"].get(inode, 0) + 1
    world.publish("repack")


def op_clone(world, source_tree_id, clone_tree_id):
    """克隆头（D6 已定项 1 每头一棵树）：记录容器与 extent 条目按物理 id 共享，水位取 origin 那一刻；克隆点记进祖先表。"""
    source = world.heads[source_tree_id]
    new_head(world, clone_tree_id)
    clone = world.heads[clone_tree_id]
    for field in ("watermark", "records", "cptr", "extents", "truth", "ever"):
        clone[field] = copy.deepcopy(source[field])
    for (tree_id, group), versions in list(world.chist.items()):
        if tree_id == source_tree_id:
            world.chist[(clone_tree_id, group)] = list(versions)
    world.clone_txg = world.open_txg()
    world.publish("clone")


def op_rollback(world, depth):
    """管理员回退（D23 已定项 14 的显式例外）：R_old = 当前时间线上 depth 次发布之前的根；取新实例代号；
    在 R_old 那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例 (i, 0, 0)；新根 txg = 根环最大 + 1。"""
    target = world.roots[-1 - depth]
    new_instance = world.max_instance + 1
    world.heads = copy.deepcopy(target["heads"])
    world.rows = dict(target["rows"])
    world.clone_txg = target["clone_txg"]
    world.rows[target["instance"]] = (target["txg"], 0)
    for instance in range(target["instance"] + 1, new_instance):
        world.rows[instance] = (0, 0)
    world.instance = world.max_instance = new_instance
    world.roots = world.roots[:len(world.roots) - depth]
    world.txg = world.max_txg
    world.counters["rollbacks"] += 1
    world.publish("rollback")


def op_crash(world):
    """崩溃：最后一次发布的根没落盘，它写出的单元成了孤儿；恢复选前一个根，给那个实例写 (i, 所选根 txg, 0)，
    取新实例代号，被抛弃的 txg 重发（恢复这次挂载的第一次发布写行，不写用户数据）。"""
    world.roots.pop()
    base = world.roots[-1]
    new_instance = world.max_instance + 1
    world.heads = copy.deepcopy(base["heads"])
    world.rows = dict(base["rows"])
    world.clone_txg = base["clone_txg"]
    world.rows[base["instance"]] = (base["txg"], 0)
    for instance in range(base["instance"] + 1, new_instance):
        world.rows[instance] = (0, 0)
    world.instance = world.max_instance = new_instance
    world.txg = base["txg"]
    world.counters["crashes"] += 1
    world.publish("crash")


OPERATIONS = {
    "create": lambda world, op: op_create(world, op[1], op[2]),
    "write": lambda world, op: op_write(world, op[1], op[2], op[3]),
    "rekey": lambda world, op: op_rekey_atomic(world, op[1], op[2], op[3]),
    "p_begin": lambda world, op: op_perunit_begin(world, op[1], op[2], op[3]),
    "p_step": lambda world, op: op_perunit_step(world, op[1], op[2]),
    "p_end": lambda world, op: op_perunit_end(world, op[1], op[2]),
    "c_step": lambda world, op: op_copy_step(world, op[1], op[2], op[3]),
    "c_flip": lambda world, op: op_copy_flip(world, op[1], op[2]),
    "c_drop": lambda world, op: op_copy_drop(world, op[1], op[2]),
    "truncate": lambda world, op: op_truncate(world, op[1], op[2], op[3]),
    "delete": lambda world, op: op_delete(world, op[1], op[2]),
    "pack": lambda world, op: op_pack(world, op[1], op[2]),
    "repack": lambda world, op: op_repack(world, op[1], op[2]),
    "clone": lambda world, op: op_clone(world, op[1], op[2]),
    "rollback": lambda world, op: op_rollback(world, op[1]),
    "crash": lambda world, op: op_crash(world),
}


def migration_operations(head, tree_id, inode, enabled, others, fresh):
    migration = head["migration"].get(inode)
    keys = keys_of(head, inode)
    operations = []
    if "perunit" in enabled:
        if migration is None and keys and fresh:
            operations.extend(("p_begin", tree_id, inode, locality) for locality in others)
        elif migration is not None and migration["protocol"] == "perunit":
            if any(key[0] == migration["old"] for key in keys):
                operations.append(("p_step", tree_id, inode))
            else:
                operations.append(("p_end", tree_id, inode))
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
            if not head["records"][inode]["alive"]:
                continue
            migration = head["migration"].get(inode)
            keys = keys_of(head, inode)
            others = [locality for locality in config["localities"] if locality != primary_locality(head, inode)]
            fresh = head["rekeys"].get(inode, 0) < config["max_rekeys"]
            if head["writes"].get(inode, 0) < config["max_writes"]:
                operations.extend(("write", tree_id, inode, offset) for offset in config["offsets"])
            if "rekey" in enabled and migration is None and keys and fresh:
                operations.extend(("rekey", tree_id, inode, locality) for locality in others)
            operations.extend(migration_operations(head, tree_id, inode, enabled, others, fresh))
            if "pack" in enabled and migration is None and len(keys) == 1 and keys[0][2] == 0 \
                    and head["packs"].get(inode, 0) < config["max_packs"]:
                kind = world.items[head["extents"][keys[0]]]["kind"]
                operations.append(("pack" if kind == "unit" else "repack", tree_id, inode))
            if "truncate" in enabled and migration is None and any(key[2] >= 1 for key in keys) \
                    and head["truncs"].get(inode, 0) < config["max_truncs"]:
                operations.append(("truncate", tree_id, inode, 1))
            if "delete" in enabled and migration is None and head["deletes"] < config["max_deletes"]:
                operations.append(("delete", tree_id, inode))
    if "clone" in enabled and len(world.heads) == 1 and world.heads[ORIGIN_TREE]["creates"] \
            and not any(head["migration"] for head in world.heads.values()):
        operations.append(("clone", ORIGIN_TREE, CLONE_TREE))
    fresh_root = world.last_kind not in (None, "rollback", "crash")
    if "rollback" in enabled and world.counters["rollbacks"] < config["max_rollbacks"] and fresh_root:
        operations.extend(("rollback", depth) for depth in (1, 2) if len(world.roots) > depth)
    if "crash" in enabled and world.counters["crashes"] < config["max_crashes"] and fresh_root and len(world.roots) >= 2:
        operations.append(("crash",))
    return operations


def replay(history, family, config=None, check=True):
    """逐步重放；check 为真时核这一步在这个家族的世界里合不合法，走不通返回 None。"""
    world = World(family)
    new_head(world, ORIGIN_TREE)
    for operation in history:
        if config is not None and check and operation not in legal_operations(world, config):
            return None
        OPERATIONS[operation[0]](world, operation)
    return world


# ---------------- 家族（写路径怎么走）与臂（重建怎么走） ----------------
FAMILIES = {
    "plain": dict(rewrite=False, copy_write="both"),     # 乙′ / 丙′ 的写路径：重编 key 不动单元
    "jia": dict(rewrite=True, copy_write="recopy"),      # 甲-S：重编 key 重写单元；先抄后切期间只写主段（S8）
    "jia_both": dict(rewrite=True, copy_write="both"),   # 甲去掉 S8：迁移期间一次写两份单元
}
# keying：unify 这个对象全部 key 编同一个首段；per_unit 按单元头编（槽没有头时按丙′ 的规则编）。
# grouping：anchor 版本分组只按 (对象, 锚点)（S6）；extent_key 按「要放回的 extent key」分组（甲字面，去掉 S6）。
# order：选中值的来源次序。record：chosen 记录取选中值；derived 取重建出的首段集合（S7）；record_first 记录读得出就用记录。
# small：p2 只有被选中的版本是槽时用丙′ 的规则；p1 ≤ 4 KiB 的对象全部 key 用丙′ 的规则（两种读法）。
ARMS = {
    "bing_rec": dict(keying="unify", grouping="anchor", order=("record", "keys", "zero"), record="chosen", small="p2"),
    "bing_s1": dict(keying="unify", grouping="anchor", order=("keys", "record", "zero"), record="chosen", small="p2"),
    "yi_s3": dict(keying="unify", grouping="anchor", order=("keys", "record", "header", "zero"), record="chosen",
                  small="p2"),
    "jia_s": dict(keying="per_unit", grouping="anchor", order=("record", "keys", "zero"), record="derived", small="p2"),
    "jia_s_p1": dict(keying="per_unit", grouping="anchor", order=("record", "keys", "zero"), record="derived",
                     small="p1"),
    "jia_noS6": dict(keying="per_unit", grouping="extent_key", order=("record", "keys", "zero"), record="derived",
                     small="p2"),
    "jia_noS7": dict(keying="per_unit", grouping="anchor", order=("record", "keys", "zero"), record="record_first",
                     small="p2"),
}
PLAIN_ARMS = ("bing_rec", "bing_s1", "yi_s3")
JIA_ARMS = ("jia_s", "jia_noS6", "jia_noS7")
READERS = ("view", "disk")


# ---------------- 损坏 ----------------
def damage_patterns(world, target_tree_id, config):
    """目标头：每个记录容器 在 / 丢 / 当前链上最新 k 版读不出；每个写过数据的对象的 extent 条目 在 / 全丢 /
    丢某一个首段的那片叶 / 丢最低或最高 offset；另按世界加墓碑容器（在 / 丢）、实例表（读得出 / 读不出）、
    祖先表（读得出 / 读不出，只对克隆头）。"""
    head = world.heads[target_tree_id]
    container_choices = []
    for group in sorted(head["cptr"]):
        states = ["intact", "lost"]
        version = head["cptr"][group]
        for depth in range(1, ROLLBACK_DEPTH + 1):
            version = world.cversions[version]["parent"]
            if version is None:
                break
            states.append(("rolled_back", depth))
        container_choices.append([(group, state) for state in states])
    extent_choices = []
    for inode in sorted(inode for inode, offsets in head["ever"].items() if offsets):
        keys = keys_of(head, inode)
        states = ["intact", "all_lost"]
        segments = sorted({key[0] for key in keys})
        if len(segments) > 1:
            states.extend(("lose_segment", segment) for segment in segments)
        offsets = sorted({key[2] for key in keys})
        if len(offsets) > 1:
            states.extend([("lose_offset", offsets[0]), ("lose_offset", offsets[-1])])
        extent_choices.append([(inode, state) for state in states])
    tomb_states = ("intact", "lost") if "tomb" in config["damage"] else ("intact",)
    table_states = ("readable", "unreadable") if "table" in config["damage"] else ("readable",)
    ancestor_states = ("readable", "unreadable") if "ancestor" in config["damage"] and target_tree_id == CLONE_TREE \
        and world.clone_txg is not None else ("readable",)
    for container_combo in itertools.product(*container_choices):
        for extent_combo in itertools.product(*extent_choices):
            for tomb, table, ancestor in itertools.product(tomb_states, table_states, ancestor_states):
                yield dict(containers=dict(container_combo), extents=dict(extent_combo), tomb=tomb, table=table,
                           ancestor=ancestor)


def extent_survives(state, key):
    if state == "all_lost":
        return False
    if isinstance(state, tuple) and state[0] == "lose_segment" and key[0] == state[1]:
        return False
    if isinstance(state, tuple) and state[0] == "lose_offset" and key[2] == state[1]:
        return False
    return True


# ---------------- 扫描：候选、记录、墓碑（这一轮不再当神谕） ----------------
def visible_to(world, target_tree_id, owner_tree_id, birth_txg, ancestor_readable):
    """祖先表判「对这棵树可不可见」（D11 已定项 4 那张表）：自己的件；克隆头另见 origin 在克隆点之前的件。
    祖先表读不出时克隆头拿不到克隆点，把 origin 的件全当可见（最宽的读法）。"""
    if owner_tree_id == target_tree_id:
        return True
    if target_tree_id == CLONE_TREE and owner_tree_id == ORIGIN_TREE and world.clone_txg is not None:
        return (not ancestor_readable) or birth_txg < world.clone_txg
    return False


def newest_key(world, item_id):
    """择新键：写序（码 1）；槽与打包前那个单元写序逐字节相同时单元排前（甲-S 按单元头编的最弱读法）。"""
    item = world.items[item_id]
    return (item["write_order"], 1 if item["kind"] == "unit" else 0, item_id)


def record_input(world, head, inode, damage):
    """记录容器没坏 ⇒ 盘上那条（'disk'）；坏了 ⇒ 在读得出、判已发布的版本里取 (诞生代号, 写序) 最大的那一版
    （'rebuilt'，被回退抛弃的版本在实例表读不出时也在内）；全丢 ⇒ 没有。"""
    group = container_group(inode)
    state = damage["containers"].get(group, "intact")
    if state == "intact":
        record = head["records"].get(inode)
        return (record, "disk") if record is not None else (None, None)
    if state == "lost" or group not in head["cptr"]:
        return None, None
    unreadable, version = set(), head["cptr"][group]
    for _ in range(state[1]):
        unreadable.add(version)
        version = world.cversions[version]["parent"]
    pool = list(world.chist.get((head["tree"], group), ()))
    if head["tree"] == CLONE_TREE and damage["ancestor"] == "unreadable":
        pool += world.chist.get((ORIGIN_TREE, group), [])
    best = None
    for version_id in pool:
        version = world.cversions[version_id]
        if version_id in unreadable or not is_published(world, damage["table"] == "readable", version["write_order"][0],
                                                         version["txg"], version["write_order"][1], False):
            continue
        if best is None or (version["txg"], version["write_order"]) > (best["txg"], best["write_order"]):
            best = version
    record = best["snapshot"].get(inode) if best is not None else None
    return (record, "rebuilt") if record is not None else (None, None)


def item_birth(item):
    return item["container_txg"] if item["kind"] == "slot" else item["birth_txg"]


def scan_candidates(world, target_tree_id, inode, object_birth, damage):
    """这个头这个对象的扫描候选：可见、判已发布（码 1 单元按码 1 规则，槽按容器的码 3 规则）、对象出生代对得上
    （记录读得出就取记录的，否则取最新那个候选的）、没被死亡写序更大的可见墓碑盖住（I-1.9 的「可见墓碑压住」）。"""
    table_ok = damage["table"] == "readable"
    ancestor_ok = damage["ancestor"] == "readable"
    published = []
    for item_id, item in world.items.items():
        if item["object"] != inode or not visible_to(world, target_tree_id, item["tree"], item_birth(item), ancestor_ok):
            continue
        if item["kind"] == "unit":
            ok = is_published(world, table_ok, item["write_order"][0], item["birth_txg"], item["write_order"][1], True)
        else:
            ok = is_published(world, table_ok, item["container_instance"], item["container_txg"], 0, False)
        if ok:
            published.append(item_id)
    if not published:
        return []
    if object_birth is None:
        object_birth = world.items[max(published, key=lambda item_id: newest_key(world, item_id))]["object_birth"]
    tombs = []
    if damage["tomb"] == "intact":
        tombs = [tomb for tomb in world.tombstones if tomb["object"] == inode and tomb["object_birth"] == object_birth
                 and visible_to(world, target_tree_id, tomb["tree"], tomb["birth_txg"], ancestor_ok)
                 and is_published(world, table_ok, tomb["death_order"][0], tomb["birth_txg"], tomb["death_order"][1],
                                  False)]
    live = []
    for item_id in published:
        item = world.items[item_id]
        if item["object_birth"] != object_birth:
            continue
        if any(tomb["low"] <= item["anchor"] < tomb["high"] and tomb["death_order"] > item["write_order"]
               for tomb in tombs):
            continue
        live.append(item_id)
    return live


def rebuild_object(world, spec, inode, record, source, surviving, lost, candidates):
    """一个对象：选中值 → 幸存 key 放回（unify 重编到选中值）→ 没有幸存 key 的锚点从扫描候选里择新补上 → 视图里的记录。
    这个对象的记录没坏、extent 条目一条没丢时原样不动。返回 (视图记录, extent, 选中值来源)。"""
    record_locs = record["loc"] if record is not None else None
    counts = {}
    for key in surviving:
        counts[key[0]] = counts.get(key[0], 0) + 1
    key_choice = min(counts, key=lambda segment: (-counts[segment], segment)) if counts else None
    with_header = [item_id for item_id in candidates if world.items[item_id]["header_locality"] is not None]
    header_hint = world.items[max(with_header, key=lambda item_id: newest_key(world, item_id))]["header_locality"] \
        if with_header else None
    sources = dict(record=record_locs[0] if record_locs else None, keys=key_choice, header=header_hint,
                   zero=ZERO_LOCALITY)
    chosen_name = next(name for name in spec["order"] if sources[name] is not None)
    chosen = sources[chosen_name]
    bing_value = next(sources[name] for name in ("record", "keys", "zero") if sources[name] is not None)
    if lost is None and source == "disk":
        segments = tuple(sorted({key[0] for key in surviving}))
        return record_locs, dict(surviving), "untouched"
    whole_small = spec["small"] == "p1" and world.family.get("small", False)

    def segment_of(item_id):
        item = world.items[item_id]
        if spec["keying"] == "unify":
            return chosen
        if whole_small or item["header_locality"] is None:
            return bing_value
        return item["header_locality"]

    extents = {}

    def place(key, item_id):
        present = extents.get(key)
        if present is None or newest_key(world, item_id) > newest_key(world, present):
            extents[key] = item_id

    for key, item_id in sorted(surviving.items()):
        if spec["keying"] == "unify":
            place((chosen, inode, key[2]), item_id)
        elif whole_small:
            place((bing_value, inode, key[2]), item_id)
        else:
            place(key, item_id)
    if lost is not None:
        covered = {key[2] for key in surviving}
        if spec["grouping"] == "anchor":
            by_anchor = {}
            for item_id in candidates:
                anchor = world.items[item_id]["anchor"]
                if anchor not in by_anchor or newest_key(world, item_id) > newest_key(world, by_anchor[anchor]):
                    by_anchor[anchor] = item_id
            for anchor in sorted(by_anchor):
                if anchor not in covered:
                    place((segment_of(by_anchor[anchor]), inode, anchor), by_anchor[anchor])
        else:
            groups = {}
            for item_id in candidates:
                group = (segment_of(item_id), world.items[item_id]["anchor"])
                if group not in groups or newest_key(world, item_id) > newest_key(world, groups[group]):
                    groups[group] = item_id
            for (segment, anchor), item_id in sorted(groups.items()):
                if lost == "all" or (segment, inode, anchor) in lost:
                    place((segment, inode, anchor), item_id)
    segments = tuple(sorted({key[0] for key in extents}))
    if whole_small:
        view = (bing_value,)
    elif spec["record"] == "chosen":
        view = (chosen,)
    elif spec["record"] == "derived":
        view = segments or (chosen,)
    else:
        view = record_locs if record_locs else (segments or (chosen,))
    return view, extents, chosen_name


def rebuild(world, target_tree_id, damage, arm_names):
    """按每条臂重建目标头。每个对象的输入（记录、幸存 key、丢了哪些 key、扫描候选）对各臂相同，只算一遍。"""
    head = world.heads[target_tree_id]
    ancestor_ok = damage["ancestor"] == "readable"
    inodes = set(head["records"]) | {inode for inode, offsets in head["ever"].items() if offsets}
    inodes |= {item["object"] for item in world.items.values()
               if visible_to(world, target_tree_id, item["tree"], item_birth(item), ancestor_ok)}
    listed = damage["extents"]
    whole_tree_lost = bool(listed) and all(state == "all_lost" for state in listed.values())
    prepared = {}
    for inode in sorted(inodes):
        state = listed.get(inode, "all_lost" if whole_tree_lost else "intact")
        surviving = {key: item_id for key, item_id in head["extents"].items()
                     if key[1] == inode and extent_survives(state, key)}
        lost = None if state == "intact" else \
            ("all" if state == "all_lost" else {key for key in keys_of(head, inode) if key not in surviving})
        record, source = record_input(world, head, inode, damage)
        candidates = scan_candidates(world, target_tree_id, inode, record["birth"] if record else None, damage) \
            if lost is not None else []
        prepared[inode] = (record, source, surviving, lost, candidates)
    results = {}
    for arm in arm_names:
        views, extents, info = {}, {}, {}
        for inode, (record, source, surviving, lost, candidates) in prepared.items():
            view, object_extents, chosen_name = rebuild_object(world, ARMS[arm], inode, record, source, surviving,
                                                               lost, candidates)
            views[inode] = view
            extents.update(object_extents)
            info[inode] = (source, lost, chosen_name)
        results[arm] = (views, extents, info)
    return prepared, results


# ---------------- 检查 ----------------
FAILURE_FIELDS = ("missing", "misread", "mac_fail", "split_new")


def reader_localities(reader, view, source, record):
    """R-view：读者用重建视图里的记录值；R-disk：记录容器没坏时读盘上那条权威记录（纪律 3 不许就地修复它）。"""
    if reader == "disk" and source == "disk":
        return record["loc"]
    return view


def read_object(world, head, inode, localities, extents):
    """读者按记录里的 locality 次序找，第一个找到的算数；甲的码 1 单元按查找路径的首段核 AAD（槽的 AAD 绑容器身份）。
    missing = 找不到真值；misread = 读到的不是真值，或者在一个已删 / 已截掉 / 从没写过的位置读到数据。"""
    truth = head["truth"].get(inode, {})
    offsets = set(head["ever"].get(inode, ())) | {key[2] for key in extents if key[1] == inode}
    tally = dict(missing=0, misread=0, mac_fail=0)
    for offset in sorted(offsets):
        want = truth.get(offset)
        found = None
        for locality in localities:
            item_id = extents.get((locality, inode, offset))
            if item_id is not None:
                found = (locality, item_id)
                break
        if found is None:
            if want is not None:
                tally["missing"] += 1
            continue
        item = world.items[found[1]]
        if world.family["rewrite"] and item["kind"] == "unit" and item["header_locality"] != found[0]:
            tally["mac_fail"] += 1
            continue
        if want is None or item["content"] != want:
            tally["misread"] += 1
    return tally


def s10_verdict(spec, reader, source, lost, chosen_name, localities, segments):
    """S10：I-9.9 按侧判——一侧是从另一侧编出来的（或两侧同源）就输出「不可判定」，否则按字面判红 / 绿。"""
    rebuilt_keys = lost is not None
    if spec["keying"] == "unify":
        keys_side = ("chosen", chosen_name) if rebuilt_keys else ("orig",)
    else:
        keys_side = ("header",) if rebuilt_keys else ("orig",)
    if reader == "disk" and source == "disk":
        record_side = ("disk",)
    elif spec["keying"] == "unify":
        record_side = ("chosen", chosen_name)
    elif spec["record"] == "derived" or source is None:
        record_side = ("derived",)
    else:
        record_side = ("disk",) if source == "disk" else ("rebuilt",)
    same = record_side == keys_side or record_side == ("derived",) or record_side == ("chosen", "keys") \
        or (keys_side == ("chosen", "record") and record_side in (("disk",), ("rebuilt",), ("chosen", "record")))
    if same:
        return "undecidable"
    if not segments:
        return "green"
    return "red" if len(localities) != 1 or any(segment != localities[0] for segment in segments) else "green"


def s6_red(world, extents, prepared):
    """S6 的检查：没有幸存 key 的锚点，重建只许补一条，而且是该锚点可见候选里择新键最大的那个。"""
    red, filled = 0, {}
    for key, item_id in extents.items():
        surviving = prepared[key[1]][2]
        if item_id not in surviving.values():
            filled.setdefault((key[1], key[2]), set()).add(item_id)
    for (inode, anchor), item_ids in filled.items():
        at_anchor = [item_id for item_id in prepared[inode][4] if world.items[item_id]["anchor"] == anchor]
        newest = max(at_anchor, key=lambda item_id: newest_key(world, item_id)) if at_anchor else None
        if len(item_ids) > 1 or newest not in item_ids:
            red += 1
    return red


def s7_red(views, extents, prepared):
    """S7 的检查：被重建过的对象，每条 key 的首段都在视图记录的集合里（读者逐个试找得到它）。"""
    return sum(1 for key in extents if (prepared[key[1]][3] is not None or prepared[key[1]][1] != "disk")
               and key[0] not in views[key[1]])


def normal_path_tally(world):
    tally = dict(missing=0, misread=0, mac_fail=0)
    for head in world.heads.values():
        for inode, record in head["records"].items():
            for field, value in read_object(world, head, inode, record["loc"], head["extents"]).items():
                tally[field] += value
    return tally


def i99_literal_red(world):
    """I-9.9 按字面判一个没损坏、合法的镜像：记录只许一个值，且等于该对象每条 extent key 的首段。"""
    red = 0
    for head in world.heads.values():
        for inode, record in head["records"].items():
            segments = {key[0] for key in keys_of(head, inode)}
            if segments and (len(record["loc"]) != 1 or segments != {record["loc"][0]}):
                red += 1
    return red


def check_rebuild(world, target_tree_id, prepared, result, arm, reader, s10_table):
    head = world.heads[target_tree_id]
    views, extents, info = result
    spec = ARMS[arm]
    tally = dict.fromkeys(FAILURE_FIELDS, 0)
    tally.update(objects=0, locality_changed=0)
    for inode, (record, source, _surviving, lost, _candidates) in prepared.items():
        segments = sorted({key[0] for key in extents if key[1] == inode})
        if not segments and not head["ever"].get(inode) and not head["truth"].get(inode):
            continue
        tally["objects"] += 1
        localities = reader_localities(reader, views[inode], source, record)
        object_tally = read_object(world, head, inode, localities, extents)
        original = {key[0] for key in keys_of(head, inode)}
        object_tally["split_new"] = 1 if len(segments) > 1 and len(original) <= 1 else 0
        for field in FAILURE_FIELDS:
            tally[field] += object_tally[field]
        current = head["records"].get(inode)
        if current is not None and set(localities) != set(current["loc"]):
            tally["locality_changed"] += 1
        if lost is not None or source != "disk":
            bad = object_tally["missing"] or object_tally["split_new"] or object_tally["mac_fail"]
            outcome = "loc_bad" if bad else ("content_bad" if object_tally["misread"] else "ok")
            verdict = s10_verdict(spec, reader, source, lost, info[inode][2], localities, segments)
            s10_table[(verdict, outcome)] = s10_table.get((verdict, outcome), 0) + 1
    return tally


# ---------------- 穷举 ----------------
STAT_FIELDS = ("histories", "patterns", "objects", "failures") + FAILURE_FIELDS + ("locality_changed", "s6_red", "s7_red")
FAMILY_FIELDS = ("histories", "skipped", "normal_fail", "i99_legal_red", "forced_rewrites", "unshared", "double_writes")


def new_accumulator(config):
    keys = [(family, arm, reader) for family, arms in config["families"] for arm in arms for reader in READERS]
    return dict(stats={key: dict.fromkeys(STAT_FIELDS, 0) for key in keys},
                family={family: dict.fromkeys(FAMILY_FIELDS, 0) for family, _arms in config["families"]},
                shortest={}, normal_shortest={}, dist={key: dict(only_arm=0, only_base=0, both=0) for key in keys},
                dist_example={}, rdist_example={}, common=dict.fromkeys(READERS, 0), patterns=dict.fromkeys(READERS, 0),
                s10={key: {} for key in keys})


def run_history(depth, history, config, acc):
    worlds = {}
    for family_name, arms in config["families"]:
        world = replay(history, dict(FAMILIES[family_name], small=config["small"]), config)
        family_stat = acc["family"][family_name]
        if world is None:
            family_stat["skipped"] += 1
            continue
        family_stat["histories"] += 1
        for counter in ("forced_rewrites", "unshared", "double_writes"):
            family_stat[counter] += world.counters[counter]
        normal = normal_path_tally(world)
        if any(normal.values()):
            family_stat["normal_fail"] += 1
            acc["normal_shortest"].setdefault(family_name, (depth, history, normal))
        family_stat["i99_legal_red"] += i99_literal_red(world)
        worlds[family_name] = world
        for arm in arms:
            for reader in READERS:
                acc["stats"][(family_name, arm, reader)]["histories"] += 1
    if "plain" not in worlds:
        return
    for target_tree_id in sorted(worlds["plain"].heads):
        for damage in damage_patterns(worlds["plain"], target_tree_id, config):
            fails = {}
            for family_name, arms in config["families"]:
                world = worlds.get(family_name)
                if world is None or target_tree_id not in world.heads:
                    continue
                prepared, results = rebuild(world, target_tree_id, damage, arms)
                for arm in arms:
                    views, extents, _info = results[arm]
                    red6, red7 = s6_red(world, extents, prepared), s7_red(views, extents, prepared)
                    for reader in READERS:
                        key = (family_name, arm, reader)
                        tally = check_rebuild(world, target_tree_id, prepared, results[arm], arm, reader,
                                              acc["s10"][key])
                        stat = acc["stats"][key]
                        stat["patterns"] += 1
                        stat["objects"] += tally["objects"]
                        stat["locality_changed"] += tally["locality_changed"]
                        stat["s6_red"] += red6
                        stat["s7_red"] += red7
                        for field in FAILURE_FIELDS:
                            stat[field] += tally[field]
                        fails[key] = any(tally[field] for field in FAILURE_FIELDS)
                        if fails[key]:
                            stat["failures"] += 1
                            acc["shortest"].setdefault(key, (depth, history, target_tree_id, damage, tally))
            for reader in READERS:
                base = fails.get(("plain", "bing_rec", reader))
                if base is None:
                    continue
                acc["patterns"][reader] += 1
                reader_keys = [key for key in fails if key[2] == reader]
                if all(fails[key] for key in reader_keys):
                    acc["common"][reader] += 1
                for key in reader_keys:
                    if fails[key] and not base:
                        acc["dist"][key]["only_arm"] += 1
                        acc["dist_example"].setdefault(key, (depth, history, target_tree_id, damage))
                    elif base and not fails[key]:
                        acc["dist"][key]["only_base"] += 1
                        acc["rdist_example"].setdefault(key, (depth, history, target_tree_id, damage))
                    elif base and fails[key]:
                        acc["dist"][key]["both"] += 1


def explore(config, max_depth):
    acc = new_accumulator(config)
    plain_family = dict(FAMILIES["plain"], small=config["small"])
    frontier = [()]
    for depth in range(1, max_depth + 1):
        next_frontier = []
        for history in frontier:
            base_world = replay(history, plain_family, config, check=False)
            next_frontier.extend(history + (operation,) for operation in legal_operations(base_world, config))
        frontier = next_frontier
        for history in frontier:
            run_history(depth, history, config, acc)
    open_after = sum(len(legal_operations(replay(history, plain_family, config, check=False), config))
                     for history in frontier)
    return acc, open_after


# ---------------- 世界 ----------------
BASE = dict(localities=LOCALITIES, offsets=OFFSETS, max_creates=1, max_writes=2, max_rekeys=1, max_packs=0,
            max_truncs=0, max_deletes=0, max_rollbacks=0, max_crashes=0, ops=frozenset(), damage=frozenset(), small=False)
PJ = (("plain", PLAIN_ARMS), ("jia", JIA_ARMS))
PJB = PJ + (("jia_both", ("jia_s",)),)
WORLDS = (
    # 协议 C（先抄后切、记录单值）：两种读者读法各跑一遍；jia_both 是甲去掉 S8。
    ("copy_r", dict(BASE, ops={"copy"}, max_writes=3, depth=8, families=PJB)),
    # 删除与截断写墓碑、旧单元留在盘上；墓碑容器 在 / 丢；重编 key 协议 A。
    ("tomb_a", dict(BASE, ops={"rekey", "truncate", "delete"}, max_truncs=1, max_deletes=1, damage={"tomb"}, depth=7,
                    families=PJ)),
    # 截断 + 协议 C。
    ("tomb_c", dict(BASE, ops={"copy", "truncate"}, max_truncs=1, max_writes=3, damage={"tomb"}, depth=8, families=PJB)),
    # 管理员回退：回退之前的时间线上重编过、写过；实例表 读得出 / 读不出。两个对象（号会被重发，C143）。
    ("rollback_a", dict(BASE, ops={"rekey", "rollback"}, max_rollbacks=1, max_creates=2, damage={"table"}, depth=7,
                        families=PJ)),
    ("rollback_p", dict(BASE, ops={"perunit", "rollback"}, max_rollbacks=1, damage={"table"}, depth=8, families=PJ)),
    ("rollback_c", dict(BASE, ops={"copy", "rollback"}, max_rollbacks=1, damage={"table"}, depth=8, families=PJB)),
    # 崩溃：最后一次发布没落，它的单元成孤儿，号与 txg 重发；实例表 读得出 / 读不出。
    ("crash_a", dict(BASE, ops={"rekey", "crash"}, max_crashes=1, max_creates=2, damage={"table"}, depth=7,
                     families=PJ)),
    # D27 打包：打包、整容器重写、打包之后重编（协议 A）、打包之后写（搬出去）、删除；对象 ≤ 4 KiB。
    ("pack3", dict(BASE, ops={"pack", "rekey", "delete"}, offsets=(0,), max_packs=2, max_deletes=1, small=True,
                   depth=8, families=(("plain", PLAIN_ARMS), ("jia", JIA_ARMS + ("jia_s_p1",))))),
    # 克隆之后两个头各自重编同一个共享对象（协议 A）、各自截断；祖先表 读得出 / 读不出。
    ("clone3", dict(BASE, ops={"clone", "rekey", "truncate"}, max_truncs=1, damage={"ancestor"}, depth=7, families=PJ)),
    ("clone_p", dict(BASE, ops={"clone", "perunit"}, damage={"ancestor"}, depth=7, families=PJ)),
)


def describe_history(history):
    return " ; ".join("%s(%s)" % (operation[0], ",".join(str(part) for part in operation[1:])) for operation in history)


def describe_damage(damage):
    parts = ["containers=%s" % sorted(damage["containers"].items(), key=str),
             "extents=%s" % sorted(damage["extents"].items(), key=str)]
    parts.extend("%s=%s" % (field, damage[field]) for field, default in
                 (("tomb", "intact"), ("table", "readable"), ("ancestor", "readable")) if damage[field] != default)
    return " ".join(parts)


def print_world(label, config, max_depth):
    acc, open_after = explore(config, max_depth)
    print("coverage", label, "max_depth=%d" % max_depth, "histories_one_step_longer=%d" % open_after,
          "(0 = 在这个世界的操作上限之内穷举完了)")
    for family, _arms in config["families"]:
        stat = acc["family"][family]
        print("family", label, family, " ".join("%s=%d" % (field, stat[field]) for field in FAMILY_FIELDS))
        if family in acc["normal_shortest"]:
            depth, history, normal = acc["normal_shortest"][family]
            print("normal_shortest", label, family, "depth=%d" % depth, "history=[%s]" % describe_history(history),
                  "tally=%s" % ",".join("%s=%d" % item for item in sorted(normal.items())))
    keys = [(family, arm, reader) for family, arms in config["families"] for arm in arms for reader in READERS]
    for key in keys:
        print("row", label, *key, max_depth, " ".join(str(acc["stats"][key][field]) for field in STAT_FIELDS))
    for reader in READERS:
        print("common", label, reader, "patterns=%d" % acc["patterns"][reader],
              "all_arms_fail=%d" % acc["common"][reader])
    for key in keys:
        dist = acc["dist"][key]
        print("dist", label, *key, "fails_where_bing_rec_ok=%d" % dist["only_arm"],
              "ok_where_bing_rec_fails=%d" % dist["only_base"], "both_fail=%d" % dist["both"])
    for key in keys:
        if key not in acc["shortest"]:
            print("shortest", label, *key, "none_through_depth=%d" % max_depth)
            continue
        depth, history, tree_id, damage, tally = acc["shortest"][key]
        print("shortest", label, *key, "depth=%d" % depth, "head=%d" % tree_id, "history=[%s]" % describe_history(history),
              describe_damage(damage), "tally=%s" % ",".join("%s=%d" % (field, tally[field]) for field in FAILURE_FIELDS))
    for key in keys:
        if key in acc["dist_example"]:
            depth, history, tree_id, damage = acc["dist_example"][key]
            print("dist_shortest", label, *key, "depth=%d" % depth, "head=%d" % tree_id,
                  "history=[%s]" % describe_history(history), describe_damage(damage))
    for key in keys:
        if key in acc["rdist_example"]:
            depth, history, tree_id, damage = acc["rdist_example"][key]
            print("rdist_shortest", label, *key, "depth=%d" % depth, "head=%d" % tree_id,
                  "history=[%s]" % describe_history(history), describe_damage(damage))
    for key in keys:
        table = acc["s10"][key]
        print("s10", label, *key, " ".join("%s/%s=%d" % (verdict, outcome, table[(verdict, outcome)])
                                            for verdict, outcome in sorted(table)))
    return acc


def main():
    depth_override = int(sys.argv[1]) if len(sys.argv) > 1 and sys.argv[1] != "0" else None
    only = sys.argv[2].split(",") if len(sys.argv) > 2 else None
    print("# D18 locality_id 第三轮攻方腿模型输出（确定性）RECORDS_PER_CONTAINER=%d OFFSETS=%s LOCALITIES=%s ROLLBACK_DEPTH=%d"
          % (RECORDS_PER_CONTAINER, OFFSETS, LOCALITIES, ROLLBACK_DEPTH))
    if only is None:
        hand_built_cases()
    print("## 穷举：每条历史 × 每个家族 × 每个头 × 全部损坏形态 × 每条臂 × 两种读者读法")
    print("columns(row): world family arm reader max_depth " + " ".join(STAT_FIELDS))
    for label, config in WORLDS:
        if only is not None and label not in only:
            continue
        print_world(label, config, depth_override or config["depth"])
    print("done")


def expect(label, actual, wanted):
    verdict = "ok" if actual == wanted else "FAIL"
    print("assert %-72s got=%-10s want=%-10s %s" % (label, actual, wanted, verdict))
    if actual != wanted:
        raise SystemExit("断言失败：" + label)


def case(history, family_name, arm, reader, damage_overrides, config_small=False, target=ORIGIN_TREE):
    """手造一段历史 + 一种损坏，返回 (世界, 重建输入, 视图记录, extent, 读者的 tally)。"""
    world = replay(history, dict(FAMILIES[family_name], small=config_small))
    head = world.heads[target]
    damage = dict(containers={group: "intact" for group in head["cptr"]}, extents={}, tomb="intact", table="readable",
                  ancestor="readable")
    damage.update({field: value for field, value in damage_overrides.items() if field != "containers"})
    damage["containers"].update(damage_overrides.get("containers", {}))
    prepared, results = rebuild(world, target, damage, (arm,))
    tally = check_rebuild(world, target, prepared, results[arm], arm, reader, {})
    views, extents, _info = results[arm]
    return world, prepared, views, extents, tally


def failures(tally):
    return sum(tally[field] for field in FAILURE_FIELDS)


def fields(tally):
    return tuple(tally[field] for field in FAILURE_FIELDS)


def hand_built_cases():
    """钉绝对值：每条要么是最短反例，要么是它的阴性对照。tally 顺序 (missing, misread, mac_fail, split_new)。"""
    print("## 手造历史（钉绝对值）")
    zero = (0, 0, 0, 0)
    copy_mid = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("c_step", ORIGIN_TREE, 1, 2))
    lose_one = {"extents": {1: ("lose_segment", 1)}}
    expect("A1 丙′-记录优先：先抄后切抄了一份、丢首段 1 那片叶，R-view", fields(case(copy_mid, "plain", "bing_rec", "view", lose_one)[4]), zero)
    expect("A1 同上 R-disk", fields(case(copy_mid, "plain", "bing_rec", "disk", lose_one)[4]), zero)
    expect("A1 丙′-S1 R-view（第二轮模型的读法）", fields(case(copy_mid, "plain", "bing_s1", "view", lose_one)[4]), zero)
    expect("A1 丙′-S1 R-disk：幸存 key 在 2、盘上记录说 1，找不到", fields(case(copy_mid, "plain", "bing_s1", "disk", lose_one)[4]),
           (1, 0, 0, 0))
    expect("A1 乙′（S3）R-view", fields(case(copy_mid, "plain", "yi_s3", "view", lose_one)[4]), zero)
    expect("A1 乙′（S3）R-disk：同上，找不到", fields(case(copy_mid, "plain", "yi_s3", "disk", lose_one)[4]), (1, 0, 0, 0))
    flipped = copy_mid + (("c_flip", ORIGIN_TREE, 1),)
    lose_two = {"extents": {1: ("lose_segment", 2)}}
    expect("A2 翻记录之后丢首段 2 的叶：丙′-记录优先 R-disk", fields(case(flipped, "plain", "bing_rec", "disk", lose_two)[4]), zero)
    expect("A2 同上 丙′-S1 R-disk：幸存 key 在 1、记录说 2", fields(case(flipped, "plain", "bing_s1", "disk", lose_two)[4]),
           (1, 0, 0, 0))
    expect("A2 同上 乙′ R-disk", fields(case(flipped, "plain", "yi_s3", "disk", lose_two)[4]), (1, 0, 0, 0))
    lose_all = {"extents": {1: "all_lost"}}
    expect("A3 甲-S 同一段历史 extent 全丢 R-view（S7 管用）", fields(case(copy_mid, "jia", "jia_s", "view", lose_all)[4]), zero)
    expect("A3 甲-S R-disk（= 第二轮打中 B 的记录优先读法）", fields(case(copy_mid, "jia", "jia_s", "disk", lose_all)[4]),
           (1, 0, 0, 0))

    abandoned_p = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("write", ORIGIN_TREE, 1, 1),
                   ("p_begin", ORIGIN_TREE, 1, 2), ("p_step", ORIGIN_TREE, 1), ("rollback", 2))
    for table, wanted in (("readable", zero), ("unreadable", (0, 0, 0, 1))):
        damage = {"extents": {1: "all_lost"}, "table": table}
        expect("B1 回退抛弃了协议 P 搬的一条（甲重写过那个单元）、实例表 %s：甲-S" % table,
               fields(case(abandoned_p, "jia", "jia_s", "view", damage)[4]), wanted)
        expect("B1 同上 丙′-记录优先", fields(case(abandoned_p, "plain", "bing_rec", "view", damage)[4]), zero)
        expect("B1 同上 乙′", fields(case(abandoned_p, "plain", "yi_s3", "view", damage)[4]), zero)
    abandoned_a = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("rekey", ORIGIN_TREE, 1, 2), ("rollback", 1))
    unreadable = {"extents": {1: "all_lost"}, "table": "unreadable"}
    expect("B2 回退抛弃了一次协议 A 重编、实例表读不出：丙′-记录优先 R-disk", fields(case(abandoned_a, "plain", "bing_rec", "disk", unreadable)[4]), zero)
    expect("B2 同上 甲-S R-view（首段跑到被抛弃的 2，内容相同）", fields(case(abandoned_a, "jia", "jia_s", "view", unreadable)[4]), zero)
    expect("B2 同上 甲-S R-disk", fields(case(abandoned_a, "jia", "jia_s", "disk", unreadable)[4]), (1, 0, 0, 0))

    abandoned_write = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("rekey", ORIGIN_TREE, 1, 2),
                       ("write", ORIGIN_TREE, 1, 0), ("rollback", 2))
    expect("C1 被抛弃的时间线重编后又写、实例表读不出：丙′-记录优先读到被抛弃的版本",
           fields(case(abandoned_write, "plain", "bing_rec", "view", unreadable)[4]), (0, 1, 0, 0))
    expect("C1 同上 甲-S R-view（S6 让写序最大的孤儿胜出）", fields(case(abandoned_write, "jia", "jia_s", "view", unreadable)[4]),
           (0, 1, 0, 0))
    expect("C1 同上 甲去掉 S6（字面分组）：两个首段、读者按 1 读到现行版本",
           fields(case(abandoned_write, "jia", "jia_noS6", "view", unreadable)[4]), (0, 0, 0, 1))
    expect("C1 阴性对照：实例表读得出时甲-S", fields(case(abandoned_write, "jia", "jia_s", "view", lose_all)[4]), zero)

    small_new = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0))
    both_lost = {"containers": {0: "lost"}, "extents": {1: "all_lost"}}
    expect("D1 甲-S 打包规则读成 p1：没打包的 ≤ 4 KiB 对象、记录与 extent 都丢，AAD 失配",
           fields(case(small_new, "jia", "jia_s_p1", "view", both_lost, True)[4]), (0, 0, 1, 0))
    expect("D1 同上读成 p2", fields(case(small_new, "jia", "jia_s", "view", both_lost, True)[4]), zero)
    small_rekey = small_new + (("rekey", ORIGIN_TREE, 1, 2),)
    expect("D2 p1：重编之后记录容器回到上一版，AAD 失配",
           fields(case(small_rekey, "jia", "jia_s_p1", "view", {"containers": {0: ("rolled_back", 1)}}, True)[4]), (0, 0, 1, 0))
    packed = small_new + (("pack", ORIGIN_TREE, 1), ("rekey", ORIGIN_TREE, 1, 2))
    expect("E1 打包之后重编、extent 全丢：丙′-记录优先 R-disk", fields(case(packed, "plain", "bing_rec", "disk", lose_all, True)[4]), zero)
    expect("E1 同上 甲-S R-view（槽与打包前单元同写序，单元胜出）", fields(case(packed, "jia", "jia_s", "view", lose_all, True)[4]), zero)
    expect("E1 同上 甲-S R-disk", fields(case(packed, "jia", "jia_s", "disk", lose_all, True)[4]), (1, 0, 0, 0))

    clone_origin_rekey = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("clone", ORIGIN_TREE, CLONE_TREE),
                          ("rekey", ORIGIN_TREE, 1, 2))
    for ancestor, wanted_disk in (("readable", zero), ("unreadable", (1, 0, 0, 0))):
        damage = {"extents": {1: "all_lost"}, "ancestor": ancestor}
        expect("F1 克隆后 origin 重编、克隆头 extent 全丢、祖先表 %s：丙′-记录优先 R-disk" % ancestor,
               fields(case(clone_origin_rekey, "plain", "bing_rec", "disk", damage, False, CLONE_TREE)[4]), zero)
        expect("F1 同上 甲-S R-view", fields(case(clone_origin_rekey, "jia", "jia_s", "view", damage, False, CLONE_TREE)[4]), zero)
        expect("F1 同上 甲-S R-disk", fields(case(clone_origin_rekey, "jia", "jia_s", "disk", damage, False, CLONE_TREE)[4]),
               wanted_disk)
    clone_both = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("write", ORIGIN_TREE, 1, 1),
                  ("clone", ORIGIN_TREE, CLONE_TREE), ("rekey", CLONE_TREE, 1, 2), ("truncate", ORIGIN_TREE, 1, 1))
    for ancestor, wanted in (("readable", zero), ("unreadable", (1, 0, 0, 0))):
        damage = {"extents": {1: "all_lost"}, "ancestor": ancestor}
        expect("F2 克隆头重编、origin 之后截断、祖先表 %s：丙′-记录优先" % ancestor,
               fields(case(clone_both, "plain", "bing_rec", "view", damage, False, CLONE_TREE)[4]), wanted)
        expect("F2 同上 甲-S（克隆头重写过的单元照样被 origin 的墓碑盖住）",
               fields(case(clone_both, "jia", "jia_s", "view", damage, False, CLONE_TREE)[4]), wanted)

    trunc_rekey = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("write", ORIGIN_TREE, 1, 1),
                   ("truncate", ORIGIN_TREE, 1, 1), ("rekey", ORIGIN_TREE, 1, 2), ("write", ORIGIN_TREE, 1, 1))
    for tomb, wanted_literal in (("intact", (0, 0, 0, 1)), ("lost", (0, 1, 0, 1))):
        damage = {"extents": {1: "all_lost"}, "tomb": tomb}
        expect("G1 截掉 offset 1、重编、再写回 offset 1，墓碑 %s：丙′-记录优先" % tomb,
               fields(case(trunc_rekey, "plain", "bing_rec", "view", damage)[4]), zero)
        expect("G1 同上 甲-S", fields(case(trunc_rekey, "jia", "jia_s", "view", damage)[4]), zero)
        expect("G1 同上 甲去掉 S6（被截掉的旧单元按头里的 1 自成一组放回）",
               fields(case(trunc_rekey, "jia", "jia_noS6", "view", damage)[4]), wanted_literal)
    for label, history in (("截断", (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 1), ("truncate", ORIGIN_TREE, 1, 1))),
                           ("删除", (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("delete", ORIGIN_TREE, 1)))):
        for tomb, wanted in (("intact", zero), ("lost", (0, 1, 0, 0))):
            damage = {"extents": {1: "all_lost"}, "tomb": tomb}
            for family, arm in (("plain", "bing_rec"), ("plain", "yi_s3"), ("jia", "jia_s")):
                expect("G2 %s之后墓碑 %s：%s 读到被删掉的版本数" % (label, tomb, arm),
                       fields(case(history, family, arm, "view", damage)[4]), wanted)
    orphan_base = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("write", ORIGIN_TREE, 1, 1),
                   ("rekey", ORIGIN_TREE, 1, 2))
    lose_first = {"extents": {1: ("lose_offset", 0)}}
    for label, history, target, extra in (
            ("回退抛弃了一次重编", orphan_base + (("rollback", 1),), ORIGIN_TREE, {"table": "unreadable"}),
            ("崩溃丢了一次重编", orphan_base + (("crash",),), ORIGIN_TREE, {"table": "unreadable"}),
            ("克隆之后 origin 重编", orphan_base[:3] + (("clone", ORIGIN_TREE, CLONE_TREE), ("rekey", ORIGIN_TREE, 1, 2)),
             CLONE_TREE, {"ancestor": "unreadable"})):
        damage = dict(lose_first, **extra)
        expect("H %s、丢 offset 0 那片叶、%s：丙′-记录优先" % (label, extra),
               fields(case(history, "plain", "bing_rec", "view", damage, False, target)[4]), zero)
        expect("H 同上 乙′", fields(case(history, "plain", "yi_s3", "view", damage, False, target)[4]), zero)
        expect("H 同上 甲-S R-view：写序更大、头里是 2 的那个单元胜出，一个对象两个首段",
               fields(case(history, "jia", "jia_s", "view", damage, False, target)[4]), (0, 0, 0, 1))
        expect("H 同上 甲去掉 S6：只往丢了的原 key (1, 1, 0) 上补，补回现行版本",
               fields(case(history, "jia", "jia_noS6", "view", damage, False, target)[4]), zero)
        expect("H 阴性对照：表读得出时甲-S", fields(case(history, "jia", "jia_s", "view", lose_first, False, target)[4]), zero)
    u4_cases()
    detector_cases()


def rebuild_injected(world, arm, reader, damage_overrides):
    head = world.heads[ORIGIN_TREE]
    damage = dict(containers={group: "intact" for group in head["cptr"]}, extents={}, tomb="intact", table="readable",
                  ancestor="readable")
    damage["containers"].update(damage_overrides.get("containers", {}))
    prepared, results = rebuild(world, ORIGIN_TREE, damage, (arm,))
    table = {}
    tally = check_rebuild(world, ORIGIN_TREE, prepared, results[arm], arm, reader, table)
    return tally, table


def u4_cases():
    print("## U4：S10（I-9.9 按侧判）在真错与合法中间态上各判什么")
    record_lost = {"containers": {0: "lost"}}
    zero = (0, 0, 0, 0)
    base = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0))
    for family, arm, wanted in (("plain", "bing_rec", ((1, 1), zero, "undecidable")), ("plain", "yi_s3", ((1, 1), zero, "undecidable")),
                                ("jia", "jia_s", ((1, 1), (0, 0, 1, 0), "undecidable"))):
        world = replay(base, dict(FAMILIES[family], small=False))
        head = world.heads[ORIGIN_TREE]
        head["extents"][(2, 1, 0)] = head["extents"].pop((1, 1, 0))   # 写路径 bug：key 编到 2，记录与单元头都说 1
        before = (normal_path_tally(world)["missing"], i99_literal_red(world))
        tally, table = rebuild_injected(world, arm, "view", record_lost)
        verdicts = sorted({verdict for verdict, _outcome in table})
        expect("U4-1 %s：(注入后常规读找不到, I-9.9 字面红) / 记录丢了之后读者 tally / S10" % arm,
               (before, fields(tally), ",".join(verdicts)), wanted)
    legal = replay((("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("write", ORIGIN_TREE, 1, 1),
                    ("p_begin", ORIGIN_TREE, 1, 2), ("p_step", ORIGIN_TREE, 1)), dict(FAMILIES["plain"], small=False))
    buggy = replay((("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("write", ORIGIN_TREE, 1, 1)),
                   dict(FAMILIES["plain"], small=False))
    buggy.heads[ORIGIN_TREE]["extents"][(2, 1, 0)] = buggy.heads[ORIGIN_TREE]["extents"].pop((1, 1, 0))
    outcome = []
    for world in (legal, buggy):
        literal_red = sorted({key[0] for key in keys_of(world.heads[ORIGIN_TREE], 1)}) != [1]
        _tally, table = rebuild_injected(world, "bing_rec", "view", record_lost)
        outcome.append((tuple(keys_of(world.heads[ORIGIN_TREE], 1)), literal_red, ",".join(sorted({v for v, _o in table}))))
    expect("U4-2 合法的协议 P 中间态 vs 写路径 bug，记录都丢了：(key 集合, 按记录取自 key 的字面红, S10)", outcome[0], outcome[1])
    expect("U4-2 同上两者的 S10", outcome[0][2], "undecidable")
    jia_macs = []
    legal_history = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("write", ORIGIN_TREE, 1, 1),
                     ("p_begin", ORIGIN_TREE, 1, 2), ("p_step", ORIGIN_TREE, 1))
    for history, inject in ((legal_history, False), (legal_history[:3], True)):
        world = replay(history, dict(FAMILIES["jia"], small=False))
        if inject:
            world.heads[ORIGIN_TREE]["extents"][(2, 1, 0)] = world.heads[ORIGIN_TREE]["extents"].pop((1, 1, 0))
        tally, _table = rebuild_injected(world, "jia_s", "view", record_lost)
        jia_macs.append(tally["mac_fail"])
    expect("U4-2 甲-S 同两幅镜像：AAD 失配数（合法, bug）——单元头替它作证", tuple(jia_macs), (0, 1))


def detector_cases():
    print("## S6 / S7 / S8 各配一条检查：甲-S 上判绿，去掉那一条收严的对照上判红（判别力自证）")
    stale = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("rekey", ORIGIN_TREE, 1, 2), ("write", ORIGIN_TREE, 1, 0))
    reds = []
    for arm in ("jia_s", "jia_noS6"):
        world, prepared, _views, extents, _tally = case(stale, "jia", arm, "view", {"extents": {1: "all_lost"}})
        reds.append(s6_red(world, extents, prepared))
    expect("S6 检查（没有幸存 key 的锚点只补一条、且是择新键最大的）：(甲-S, 去掉 S6)", tuple(reds), (0, 1))
    rolled = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("rekey", ORIGIN_TREE, 1, 2))
    reds = []
    for arm in ("jia_s", "jia_noS7"):
        _world, prepared, views, extents, _tally = case(rolled, "jia", arm, "view", {"containers": {0: ("rolled_back", 1)}})
        reds.append(s7_red(views, extents, prepared))
    expect("S7 检查（重建过的对象每条 key 的首段都在视图记录里）：(甲-S, 去掉 S7)", tuple(reds), (0, 1))
    during_copy = (("create", ORIGIN_TREE, 1), ("write", ORIGIN_TREE, 1, 0), ("c_step", ORIGIN_TREE, 1, 2), ("write", ORIGIN_TREE, 1, 0))
    counts = tuple(replay(during_copy, dict(FAMILIES[family], small=False)).counters["double_writes"] for family in ("jia", "jia_both"))
    expect("S8 检查（迁移期间一次写在两个首段各写一个码 1 单元的次数）：(甲-S, 去掉 S8)", counts, (0, 1))


if __name__ == "__main__":
    main()
