#!/usr/bin/env python3
"""m3-tomb-r1 正推腿的小模型：两部分，都是纯算术 / 纯逻辑，不读 crates，不依赖任何装置。

第一部分（算术）：墓碑记录宽、一个 32 KiB 容器装几条、墓碑树一层根装几个容器、增长表、
  每种形态一次墓碑发布多写几个单元、对那一段崩溃状态数的倍数（一个单元两副本 = 两次整写，每次整写取两态，D13 已定项 4）。
  输入常量全部取自 crates 冻结副本的字面量或 kb 条款，每个常量旁写出处。
第二部分（逻辑）：一个按 E104 摘要里 P2 / P3 规则（已发布的版本里按 (诞生代号, 实例代号, 事务号) 取最大者）写的
  扫描重建玩具，只核管理员回退（向前发布、不写实例表行）之后三种回退形态在三个世界里把几个活对象扫成死、几个死对象扫成活。
  它不是 E104 的复跑；E104 的世界与规则比它全得多，这里只回答「回退那次发布对墓碑容器要不要重写、要不要写退役记录」。
"""
import sys

# ---------- 第一部分：算术 ----------
UNIT_BYTES = 32768                # crates/singlefs-format/src/lib.rs:20 DATA_UNIT_BYTES（码 3 容器恒 32 KiB）
PACKED_RECORDS_OFFSET = 136       # crates/singlefs-format/src/lib.rs:44 PACKED_UNIT_RECORDS_OFFSET（含 29 字节预留位的头）
CODE2_NODE_BYTES = 16384          # D8 已定项 2 节点 16 KiB；树表单元 16 KiB（D22 已定项 7）
CODE2_HEADER_BYTES = 131          # D8 已定项 8：树表单元自己的码 2 头 131（key 宽 8）
INODE_STYLE_INTERNAL_ENTRY = 120  # crates/singlefs-format/src/lib.rs:107 INODE_INTERNAL_ENTRY：分隔 key 8 + 身份引用 26 + 子指针 86
SLOT = 16384                      # 16 KiB 槽（D19 已定项 4 的槽号单位）
REPLICAS = 2                      # D2 已定项 9：第一版 2 盘恒 w = 2


def cap(width):
    return (UNIT_BYTES - PACKED_RECORDS_OFFSET) // width


def emit(**fields):
    print("E " + " ".join(f"{k}={v}" for k, v in fields.items()))


def part_one():
    # 记录宽：整对象墓碑 = 对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 = 32（D18 已定项 10「这三个字段合计 32 字节」）
    for name, width in [("tomb_32", 32), ("tomb_40_with_death_txg", 40), ("tomb_56_E83_guess", 56),
                        ("retire_26_four_tuple", 8 + 2 + 8 + 8), ("instance_row_88", 88)]:
        emit(name="capacity", record=name, width=width, per_container=cap(width))
    fanout = (CODE2_NODE_BYTES - CODE2_HEADER_BYTES) // INODE_STYLE_INTERNAL_ENTRY
    emit(name="root_fanout", entry_bytes=INODE_STYLE_INTERNAL_ENTRY, containers_per_one_level_root=fanout,
         tombs_per_one_level_tree_at_32=fanout * cap(32))
    # 增长：只涨不缩（D18 已定项 10 的回收规则在第一版无从满足，见报告 W3）
    for deletes in (1, 1019, 1020, 10**3, 10**6, 10**9):
        containers = -(-deletes // cap(32))
        both_copies = containers * UNIT_BYTES * REPLICAS
        emit(name="growth", deletes=deletes, containers=containers, bytes_both_copies=both_copies,
             mib_both_copies=round(both_copies / 2**20, 2))
    # 一次墓碑发布多写几个单元；一个单元 = 两副本 = 两次整写，一段里 k 次整写取两态 ⇒ 2^k（D13 已定项 4）
    shapes = {
        "T_tree_container_plus_root": 2,    # 墓碑树：码 3 容器一个 + 码 2 根一个
        "chain_head_only": 1,               # 链：只有开放容器那一个单元（根指针在树表条目里）
        "single_container_in_tree_table": 1,
        "mapping_as_index_value_only": 1,   # 容器一个，映射条目另算（不是单元）
    }
    for name, units in shapes.items():
        writes = units * REPLICAS
        slots_per_copy = {"T_tree_container_plus_root": 2 + 1}.get(name, 2)
        emit(name="intent_publish_extra", shape=name, extra_units=units, extra_writes=writes,
             segment_state_multiplier=2 ** writes,
             bytes_both_copies=(UNIT_BYTES + (CODE2_NODE_BYTES if units == 2 else 0)) * REPLICAS,
             prepay_slots_per_copy=slots_per_copy, prepay_bytes_summed_over_replicas=slots_per_copy * SLOT * REPLICAS)
    for name, units in (("rollback_take_R_old_only", 0), ("rollback_recow_open_plus_root", 2),
                        ("rollback_recow_plus_root_plus_retire_container", 3)):
        emit(name="rollback_publish_extra", shape=name, extra_units=units, segment_state_multiplier=4 ** units)


# ---------- 第二部分：扫描玩具 ----------
class V:
    """一个已落盘的墓碑类容器版本。ident = (出生树, 类型, 容器号, 容器出生代)。"""
    def __init__(self, ident, txg, records, published=True, instance=1, txn=1):
        self.ident, self.txg, self.records = ident, txg, frozenset(records)
        self.published, self.order = published, (txg, instance, txn)


def scan_dead(versions, retire_key):
    latest = {}
    for v in versions:
        if v.published and (v.ident not in latest or v.order > latest[v.ident].order):
            latest[v.ident] = v
    retired = set()
    for ident, v in latest.items():
        if ident[1] == 3:
            retired |= {retire_key(r) for r in v.records}
    dead = set()
    for ident, v in latest.items():
        if ident[1] == 1 and retire_key(ident) not in retired:
            dead |= set(v.records)
    return dead


full_key = lambda ident: ident                       # 退役记录按完整四元组认
number_only_key = lambda ident: ident[:3]            # 消融：退役记录不带容器出生代


def world(name):
    """返回 (回退之前盘上全部墓碑类容器版本, R_old 那一版的活墓碑集 = 真值里的死者, R_old 里按容器身份列的版本, 回退发布的 txg, 回退之后又删的对象)。"""
    T = 1
    if name == "w1_rollback_between_two_deletes":
        c1_12 = V((T, 1, 1, 12), 12, [("A", 10)])
        c1_14 = V((T, 1, 1, 12), 14, [("A", 10), ("B", 10)])
        return [c1_12, c1_14], {("A", 10)}, {c1_12.ident: c1_12}, 16, []
    if name == "w2_rollback_before_any_delete_then_delete_again":
        c1_12 = V((T, 1, 1, 12), 12, [("A", 10)])
        c1_14 = V((T, 1, 1, 12), 14, [("A", 10), ("B", 10)])
        return [c1_12, c1_14], set(), {}, 16, [("A", 10)]
    if name == "w3_two_containers_capacity_two":
        c1_12 = V((T, 1, 1, 12), 12, [("A", 10)])
        c1_14 = V((T, 1, 1, 12), 14, [("A", 10), ("B", 10)])
        c2_16 = V((T, 1, 2, 16), 16, [("C", 10)])
        return [c1_12, c1_14, c2_16], {("A", 10)}, {c1_12.ident: c1_12}, 20, []
    raise KeyError(name)


def run(arm, name):
    versions, truth_dead, r_old_versions, t_roll, later_deletes = world(name)
    versions = list(versions)
    if arm != "A_take_R_old_only":
        retire_ids = []
        newest_by_ident = {}
        for v in versions:
            newest_by_ident[v.ident] = max(newest_by_ident.get(v.ident, v), v, key=lambda x: x.order)
        for ident, newest in newest_by_ident.items():
            if ident in r_old_versions:
                if newest.order != r_old_versions[ident].order:       # R_old 之后又被重写过：重写一遍 R_old 的内容，诞生代号压过陈旧版本
                    versions.append(V(ident, t_roll, r_old_versions[ident].records))
            else:                                                     # R_old 里没有这个容器：整身份退役
                retire_ids.append(ident)
        if retire_ids:
            versions.append(V((T_, 3, 1, t_roll), t_roll, retire_ids))
    truth = set(truth_dead)
    for k, obj in enumerate(later_deletes):                          # 回退之后用户又删了这个对象：新容器号 = 现行树里最大号 + 1，出生代 = 这次发布的 txg
        t = t_roll + 2 + 2 * k
        live_numbers = [i[2] for i in r_old_versions] or [0]
        versions.append(V((T_, 1, max(live_numbers) + 1, t), t, [obj]))
        truth.add(obj)
    key = number_only_key if arm == "B_retire_without_container_birth" else full_key
    dead = scan_dead(versions, key)
    emit(name="scan", arm=arm, world=name, phantom_dead=len(dead - truth), missed_dead=len(truth - dead),
         phantom=sorted(o[0] for o in dead - truth), missed=sorted(o[0] for o in truth - dead))
    return len(dead - truth), len(truth - dead)


T_ = 1

def part_two():
    results = {}
    for name in ("w1_rollback_between_two_deletes", "w2_rollback_before_any_delete_then_delete_again",
                 "w3_two_containers_capacity_two"):
        for arm in ("A_take_R_old_only", "B_recow_and_retire", "B_retire_without_container_birth"):
            results[(arm, name)] = run(arm, name)
    # 钉绝对值：陈旧版本压过 R_old 版本的世界里，只取 R_old 的形态扫出活对象被判死；重写 + 退役之后 0；退役不带容器出生代会漏掉回退之后的新死亡
    assert results[("A_take_R_old_only", "w1_rollback_between_two_deletes")] == (1, 0)
    assert results[("A_take_R_old_only", "w2_rollback_before_any_delete_then_delete_again")] == (1, 0)
    assert results[("A_take_R_old_only", "w3_two_containers_capacity_two")] == (2, 0)
    for name in ("w1_rollback_between_two_deletes", "w2_rollback_before_any_delete_then_delete_again", "w3_two_containers_capacity_two"):
        assert results[("B_recow_and_retire", name)] == (0, 0), name
    assert results[("B_retire_without_container_birth", "w2_rollback_before_any_delete_then_delete_again")] == (0, 1)


if __name__ == "__main__":
    part_one()
    part_two()
    print("E name=done asserts=ok")
    sys.exit(0)
