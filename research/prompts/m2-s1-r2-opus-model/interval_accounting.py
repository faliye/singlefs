#!/usr/bin/env python3
"""m2-s1 第二轮 云端攻方腿的计数模型：I3（K9′ 推迟回收的账）与 I7（周期与环账）。

纯算术，不读盘、不引 crates。全部输入常量在 INPUTS 里标了出处；出处是 kb 条款或 E155 产物的，
行号写在旁边，都在 2026-09-21 现查过。模型自己不产生任何「入库的数」——它只是把条款里的常量
按一条公式算到底（F3：副本 / 模型上量出的数不进 kb）。

复跑：  nice -n 19 python3 research/prompts/m2-s1-r2-opus-model/interval_accounting.py
"""

# ---------------------------------------------------------------- 输入常量
INPUTS = {
    # 记录几何
    "record_bytes": (4096, "D23 已定项 12 的 4 KiB 定长记录；crates/singlefs-format/src/lib.rs:148"),
    "records_per_fsync_seq": (8, ".claude/kb/decisions/16-发布语义.md:130「每次 fsync 的记录数等于数据单元数（seq 批 1 是 8 条…）」"),
    # 速率与周期
    "fsync_per_second": (2785, ".claude/kb/decisions/16-发布语义.md:138「本机 2785 fsync/秒」（E44）"),
    "t_time_default_s": (5.0, ".claude/kb/decisions/16-发布语义.md:121「T_time 默认值 5 s」"),
    # 环几何
    "ring_bytes_default": (768 * 1024 * 1024, ".claude/kb/decisions/16-发布语义.md:121「环长是 mkfs 参数（默认 768 MiB…）」"),
    "F_min": (2, ".claude/kb/invariants.md:76 I-8.1「F ≥ 2」"),
    "F_used_in_clause": (3, ".claude/kb/decisions/16-发布语义.md:121「默认环下有效值 = min(2 GiB, 256 MiB) = 256 MiB」⇒ 768÷256 = 3"),
    "t_dirty_field": (2 * 1024**3, ".claude/kb/decisions/16-发布语义.md:121「T_dirty = 2 GiB（初值…）」"),
    # 负载
    "leaves_per_fsync": (8, ".claude/kb/decisions/25-目标负载优先级.md:20「一次 fsync 带 8 个叶子，落在 1 条共享脊柱上」"),
    "write_amp_seq": (1.75, ".claude/kb/decisions/25-目标负载优先级.md:20 的表：seq 每次 fsync 的写放大 1.75"),
    # 槽与单元
    "slot_bytes": (16384, ".claude/kb/decisions/03-空间分配.md:117「落点粒度取 16384 字节（一个 16 KiB 槽）」"),
    # 分配记录一个节点的容量（今天的实现硬墙）
    "node_bytes": (16384, "crates/singlefs-format/src/lib.rs:15 NODE_BYTES"),
    "index_node_header_no_keyrange": (86, "crates/singlefs-format/src/lib.rs:47"),
    "nonce_mac_reserved": (29, "crates/singlefs-format/src/lib.rs:30"),
    "alloc_key_bytes": (10, "crates/singlefs-format/src/lib.rs:104"),
    "alloc_record_bytes": (20, "crates/singlefs-format/src/lib.rs:106（自证 lib.rs:285 assert = 20）"),
    "devices": (2, ".claude/kb/decisions/28-挂载期承诺量.md:71 的式子按两块盘写"),
    # E155 第二次跑第一段的产物数（背景材料第四节整行抄；模型只当输入，不改写）
    "e155_mid_versions_wal_N16": (60, "E155 第二次跑：K9′ 下 wal_full 一个间隔（N=16）的中间版 60"),
    "e155_mid_versions_yi_N16": (45, "E155 第二次跑：乙-M-K9′ 中间版 45"),
    "e155_alloc_records_K9": (402, "E155 第二次跑：K9 下分配记录树仍 [1]、402 条"),
    "defer_states": (4, ".claude/kb/decisions/16-发布语义.md:32 所在的表：候选集保留最近 4 个可退到的不同状态"),
    "bound_three": (3, ".claude/kb/decisions/03-空间分配.md:180「界是 3 次改变用户可见状态的发布」"),
}
V = {k: v[0] for k, v in INPUTS.items()}
MiB = 1024 * 1024


def node_capacity_entries() -> int:
    """crates/singlefs-core/src/unit.rs:126 index_node_entry_capacity(10, 20) 的算术复刻。"""
    header = V["index_node_header_no_keyrange"] + 2 * V["alloc_key_bytes"] + V["nonce_mac_reserved"]
    return (V["node_bytes"] - header) // V["alloc_record_bytes"]


def journal_occupancy_bytes(period_s: float, rate: float, records_per_fsync: int) -> float:
    """一个 checkpoint 间隔攒下的 journal 字节。

    间隔里 tail 不动：D16 已定项 7（16-发布语义.md:169）「系统配置槽在根槽持久之后再更新，每个
    checkpoint 一次」，而 tail 住系统配置槽（D23 已定项 3）⇒ WAL 类形态下每个间隔只推进一次。
    """
    return rate * period_s * records_per_fsync * V["record_bytes"]


def table_i7_occupancy() -> list:
    """I7 表 1：间隔占用 vs 环÷F。甲那一列是「一次发布的占用」——甲每次 fsync 发根，tail 每次 fsync 推进。"""
    rows = []
    ring = V["ring_bytes_default"]
    for period in (0.2, 1.0, 2.941, 4.412, 5.0, 30.0, 300.0):
        j = journal_occupancy_bytes(period, V["fsync_per_second"], V["records_per_fsync_seq"])
        rows.append({
            "周期 s": period,
            "WAL 类间隔占用 MiB": round(j / MiB, 2),
            "甲一次发布占用 KiB": round(V["records_per_fsync_seq"] * V["record_bytes"] / 1024, 2),
            "环÷F=2 MiB": round(ring / 2 / MiB, 2),
            "环÷F=3 MiB": round(ring / 3 / MiB, 2),
            "F=2 过不过": "过" if j <= ring / 2 else "不过",
            "F=3 过不过": "过" if j <= ring / 3 else "不过",
        })
    return rows


def crossover_period_s(F: int, ring_bytes: int) -> float:
    """周期取到多大，间隔占用就顶到环÷F。"""
    per_second = V["fsync_per_second"] * V["records_per_fsync_seq"] * V["record_bytes"]
    return (ring_bytes / F) / per_second


def table_i7_crossover() -> list:
    rows = []
    for ring_mib in (64, 256, 768, 2048):
        for F in (2, 3):
            rows.append({
                "环 MiB": ring_mib,
                "F": F,
                "顶到环÷F 的周期 s": round(crossover_period_s(F, ring_mib * MiB), 4),
                "默认 T_time 5 s 过不过": "过" if crossover_period_s(F, ring_mib * MiB) >= V["t_time_default_s"] else "不过",
            })
    return rows


def t_dirty_cut_s() -> dict:
    """T_dirty 夹取那一支在什么时候先触发——分两种读法。

    读法 A（T_dirty 数的是「还没落盘的用户数据字节」，D16 已定项 5 的字面「脏页内存预算本身」）：
      W1 / L1 的数据单元在 fsync 那一刻就落盘了（两臂的定义都写了「写脏数据单元」），
      所以这个量恒为 0 ⇒ 这一支永不触发，间隔只由 T_time 收。
    读法 B（T_dirty 数的是「这个间隔里写出去、还没被根罩住的用户数据字节」）：
      按 seq 一次 fsync 8 个数据单元、单元 32 KiB（16-发布语义.md:130 的 12.5% 反解）算。
    """
    effective = min(V["t_dirty_field"], V["ring_bytes_default"] // V["F_used_in_clause"])
    unit_bytes = 32 * 1024
    per_second_b = V["fsync_per_second"] * V["records_per_fsync_seq"] * unit_bytes
    return {
        "有效 T_dirty MiB": effective / MiB,
        "读法 A 触发时刻 s": None,
        "读法 B 触发时刻 s": round(effective / per_second_b, 4),
        "读法 B 下的间隔 journal 占用 MiB": round(
            journal_occupancy_bytes(effective / per_second_b, V["fsync_per_second"], V["records_per_fsync_seq"]) / MiB, 2),
    }


def pinned_released_units(N: int, released_per_fsync: float) -> dict:
    """I3 表：谓词钉住的已释放槽数（= df 的滞后量那一项数的东西）。

    谓词（16-发布语义.md:32）可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)；
    滞后量（16-发布语义.md:39）= 释放代 > max(第 4 新的非空有效根, 环里最旧有效根) 的已释放块。
    ⇒ 钉住的 = 最近 3 次「改变用户可见状态的发布」放掉的（03-空间分配.md:180 的界 3），
      WAL 类另加当前这个间隔里按 K9′ 扣着、还没落一条已释放记录的中间版。
    甲：一次 fsync = 一次发布 ⇒ 3 次发布 = 3 次 fsync。
    WAL：一次 checkpoint = 一次发布 ⇒ 3 次发布 = 3N 次 fsync，再加当前间隔的 N 次。
    """
    jia = V["bound_three"] * released_per_fsync
    wal = (V["bound_three"] + 1) * N * released_per_fsync
    return {
        "N": N,
        "甲 钉住槽数": round(jia, 1),
        "WAL 钉住槽数": round(wal, 1),
        "倍数": round(wal / jia, 1) if jia else None,
        "甲 钉住 MiB": round(jia * V["slot_bytes"] / MiB, 3),
        "WAL 钉住 MiB": round(wal * V["slot_bytes"] / MiB, 3),
    }


def table_i3_pinned() -> list:
    """每次 fsync 的释放量取 E155 第二次跑的中间版数除以 N=16（推的外推，标在报告里）。"""
    rho_wal = V["e155_mid_versions_wal_N16"] / 16
    rho_yi = V["e155_mid_versions_yi_N16"] / 16
    out = []
    for N in (16, 256, 1024, int(V["fsync_per_second"] * V["t_time_default_s"])):
        r = pinned_released_units(N, rho_wal); r["臂"] = "wal_full-K9′"; r["ρ 槽/fsync"] = rho_wal
        out.append(r)
        r = pinned_released_units(N, rho_yi); r["臂"] = "乙-M-K9′"; r["ρ 槽/fsync"] = rho_yi
        out.append(r)
    return out


def table_i3_alloc_records() -> list:
    """I3 / I4 表：间隔里攒下的分配记录条数 vs 今天实现的一节点硬墙。

    今天的实现（crates/singlefs-core/src/transaction.rs 的 admission_of_one_publish）
    按 records_before + rewritten.len() × 盘数 判，越过一个节点的容量就报
    AllocationRecordsExceedOneNode。K9′ 让间隔里每个中间版各占一条、间隔内回收不了。
    """
    cap = node_capacity_entries()
    base = V["e155_alloc_records_K9"]
    rows = []
    for N in (4, 8, 13, 16, 32, 64):
        for arm, mids16 in (("wal_full-K9′", V["e155_mid_versions_wal_N16"]),
                            ("乙-M-K9′", V["e155_mid_versions_yi_N16"])):
            new_entries = round(mids16 / 16 * N) * V["devices"]
            rows.append({
                "臂": arm, "N": N, "一个节点容量": cap, "基数(E155 K9)": base,
                "间隔新增条目": new_entries, "合计": base + new_entries,
                "今天的实现": "拒（AllocationRecordsExceedOneNode）" if base + new_entries > cap else "过",
            })
    return rows


def show(title, rows):
    print("\n## " + title)
    if isinstance(rows, dict):
        for k, v in rows.items():
            print(f"  {k}: {v}")
        return
    keys = list(rows[0].keys())
    print("  " + " | ".join(keys))
    for r in rows:
        print("  " + " | ".join(str(r[k]) for k in keys))


if __name__ == "__main__":
    print("# m2-s1-r2 云端攻方腿计数模型（纯算术；数不进 kb）")
    print(f"\n分配记录一个节点的容量 index_node_entry_capacity(10, 20) = {node_capacity_entries()} 条")
    show("I7 表 1：一个 checkpoint 间隔的 journal 占用 vs 环÷F（环 768 MiB）", table_i7_occupancy())
    show("I7 表 2：顶到环÷F 的周期", table_i7_crossover())
    show("I7 表 3：T_dirty 夹取那一支什么时候先触发", t_dirty_cut_s())
    show("I3 表 1：谓词钉住的已释放槽（甲 vs WAL 类）", table_i3_pinned())
    show("I3 表 2：间隔里攒下的分配记录条数 vs 一节点硬墙", table_i3_alloc_records())
