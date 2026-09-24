#!/usr/bin/env python3
"""checkpoint-trigger-r1 云端攻方腿 K2 的算术探针。

只算术，不跑文件系统：把三件事按今天盘上的数算一遍。
输入全部是今天已落盘的常量，出处写在下面的 SOURCES 里。
量纲一律字节，记录数一律条。
"""

SOURCES = {
    "ring_bytes": "crates/singlefs-format/src/lib.rs:162 JOURNAL_RING_DEFAULT_BYTES = 768 MiB",
    "record_bytes": "crates/singlefs-format/src/lib.rs:139 JOURNAL_RECORD_BYTES = 4096",
    "safety_factor": "crates/singlefs-format/src/lib.rs:165 JOURNAL_SAFETY_FACTOR = 3",
    "declared_worst": ".claude/kb/layout/01-first-txn.md:131 journal 最坏占用 = 268435456",
    "real_worst_txn": ".claude/kb/checks-owed.md:276 C310「I-8.1 按「任一事务」读只要 12 KiB」",
    "concurrency": ".claude/kb/decisions/25-目标负载优先级.md:141 并发 fsync 不设上限、下限 16",
    "in_flight_depth": ".claude/kb/decisions/16-发布语义.md:124 在飞 checkpoint 深度 = 开放 + 发布中两个",
}

RING = 768 * 1024 * 1024
REC = 4096
DECLARED_WORST = 268_435_456
REAL_WORST_TXN = 12 * 1024


def geometry(f):
    return {
        "F": f,
        "环字节": RING,
        "环槽数": RING // REC,
        "环÷F 字节": RING // f,
        "环÷F 条": RING // f // REC,
        "触发后余量字节": RING - RING // f,
    }


def i81_as_instantiated(f):
    """I-8.1 逐字：环大小 ≥ F × 任一事务的最坏 journal 占用。
    今天盘上那个「最坏占用」字段填的是 环÷F 自己（mkfs 用 journal_in_flight_record_limit
    × 记录字节算出来，system_configuration.rs:343-346）。"""
    field = RING // f * REC // REC  # = RING // f，写清楚它就是 环÷F
    return {
        "F": f,
        "字段值（环÷F）": field,
        "F × 字段": f * field,
        "环": RING,
        "判据成立": RING >= f * field,
        "余量倍数": RING / (f * field),
        "按 C310 真算一个事务": REAL_WORST_TXN,
        "真算下的余量倍数": RING / (f * REAL_WORST_TXN),
    }


def headroom_vs_concurrency(f, c, per_txn):
    """第三支在 已占用 ≥ 环÷F 触发；触发只关闭成员资格，已取号的 c 个事务
    还要把各自的记录写完（16-发布语义.md:124）。问余量够不够。"""
    headroom = RING - RING // f
    need = c * per_txn
    return {
        "F": f,
        "并发事务数": c,
        "每事务占用": per_txn,
        "触发后余量": headroom,
        "触发后仍要写": need,
        "装得下": headroom >= need,
        # headroom = RING(1 - 1/F) >= c*w  <=>  F >= 1/(1 - c*w/RING)，c*w >= RING 时无解
        "要的最小 F": None if need >= RING else round(1 / (1 - need / RING), 4),
    }


def stall_seconds(f, lam, bytes_per_fsync):
    """触发之后到发布真正完成那段时间，新来的事务照样写 journal。
    问：按这台机器自己的速率，余量能撑多久。lam 是 fsync/秒。"""
    headroom = RING - RING // f
    trigger = RING // f
    return {
        "F": f,
        "lambda": lam,
        "每 fsync 字节": bytes_per_fsync,
        "触发点撑住的秒数（今天没有第三支时越界的时刻）": round(trigger / (lam * bytes_per_fsync), 2),
        "触发后余量撑住的秒数（第三支买到的那段）": round(headroom / (lam * bytes_per_fsync), 2),
        "倍数": round(headroom / trigger, 4),
    }


def interval_vs_physical(f, depth):
    """第三支量的是「这个 checkpoint 间隔里」的占用，而 tail 只在发布完成时推进
    （23-journal的角色与格式.md:116）。在飞深度 depth 个 checkpoint 的记录同时占着环。"""
    per_interval = RING // f
    return {
        "F": f,
        "在飞深度": depth,
        "间隔计数触发点": per_interval,
        "同刻物理占用上界": per_interval * depth,
        "物理占用是否已满环": per_interval * depth >= RING,
    }


def one_fsync_interval(f):
    """读法甲（checkpoint = 一次发布，fsync 触发的发布也是发布）下，
    「一个 checkpoint 间隔」= 一次 fsync 的那一次发布。那一次 fsync 有多大由用户定。
    journal 恒占用户数据 12.5%（checks-owed.md:276 C310 / E143）。"""
    ratio = 0.125
    effective_t_dirty = RING // f  # 挂载时有效 T_dirty = min(字段, 环长 ÷ F)
    return {
        "F": f,
        "环÷F": RING // f,
        "一次 fsync 写多少用户数据才占满 环÷F": int(RING // f / ratio),
        "一次 fsync 写多少用户数据才占满整环": int(RING / ratio),
        "有效 T_dirty（用户脏字节）": effective_t_dirty,
        "T_dirty 满窗时的 journal 占用": int(effective_t_dirty * ratio),
        "T_dirty 满窗占用 / (环÷F)": round(effective_t_dirty * ratio / (RING // f), 4),
        "T_dirty 夹取留的余量倍数": round((RING // f) / (effective_t_dirty * ratio), 2),
    }


def main():
    print("== 出处 ==")
    for k, v in SOURCES.items():
        print(f"  {k}: {v}")
    print("\n== 一、环几何 ==")
    for f in (2, 3):
        print("  " + repr(geometry(f)))
    print("\n== 二、I-8.1 按今天字段值的实例化 ==")
    for f in (2, 3):
        print("  " + repr(i81_as_instantiated(f)))
    print("\n== 三、触发后余量 vs 已取号事务（每事务按两种读法）==")
    for per_txn, name in ((REAL_WORST_TXN, "C310 真算 12 KiB"), (DECLARED_WORST, "盘上字段 256 MiB")):
        print(f"  --- 每事务占用取：{name}")
        for f in (2, 3):
            for c in (1, 2, 3, 16, 64, 1024):
                print("    " + repr(headroom_vs_concurrency(f, c, per_txn)))
    print("\n== 四、间隔计数 vs 物理占用（在飞深度 2）==")
    for f in (2, 3):
        print("  " + repr(interval_vs_physical(f, 2)))
    print("\n== 五、触发到发布完那段，余量按本机速率撑多久 ==")
    print("  lambda = 2785 fsync/秒（E44 本机实测，m2-s1-r2 判决 I7 用的同一根柱子）")
    print("  每 fsync 8 条 4 KiB 记录 = 32768 字节（D25 已定项 1 + D23 已定项 12 射程）")
    for f in (2, 3):
        print("  " + repr(stall_seconds(f, 2785, 8 * 4096)))
    for f in (3,):
        for lam in (278, 2785, 27850):
            print("  " + repr(stall_seconds(f, lam, 8 * 4096)))
    print("\n== 六、读法甲下「一个间隔 = 一次 fsync」，那次 fsync 多大由用户定 ==")
    for f in (2, 3):
        print("  " + repr(one_fsync_interval(f)))


if __name__ == "__main__":
    main()
