#!/usr/bin/env python3
"""攻方腿 m3-batch10t-code-r1 S5（K2-1 判红 / 压并发那一向）：起过卡池的产品路径，第 ① 段每张卡带得了几个状态照
「登记时克隆出来的卡」的 memory_budget_mebibytes 算，那张卡的空闲是开账那一刻读的（gpu_unit_checks.rs started_card），
不是账本里此刻的估计。纯算术，公式逐条照冻结副本抄（行号见报告）：
- budget_of：gpu_unit_checks.rs memory_budget_mebibytes_of（配了额度取额度与空闲的小者，没配取空闲的 3/4）；
- 账的容量：gpu_memory_ledger.rs card_ledger_capacity_mebibytes（没设额度变量：budget_of(配置, 估计) − 驱动那一份）；
- 按卡的额度带得了几个：crash_amplification.rs states_in_flight_of_each_card（额度 < 表 + 驱动那一份 ⇒ 0，否则
  (额度 − 表 − 驱动那一份) MiB ÷ (SCRATCH_WORDS × 4) 字节）；
- 每张卡装几个：两者取小（states_each_card_of_the_pool_lends）；全是 0 ⇒ 第 ① 段报错「没有一张卡装得下这条流的表…」（判红）。
场景：这个进程开池那一刻，另一个记账进程在卡上借着大半张卡（开账读到的空闲小），账本估计照夹法留在整卡；之后那个进程还了，卡空着。
"""
DRIVER = 2560
SCRATCH_BYTES_PER_STATE = 315_376 * 4
MIB = 1024 * 1024
TOTAL = 32_607
TABLES = 64


def budget_of(configured, free):
    return min(configured, free) if configured is not None else free // 4 * 3


def states_by_card_budget(budget):
    if budget < TABLES + DRIVER:
        return 0
    return (budget - TABLES - DRIVER) * MIB // SCRATCH_BYTES_PER_STATE


def states_fitting_the_ledger(capacity):
    # mebibytes_a_judge_needs：表 + ⌈状态 × (SCRATCH_WORDS×4 + 2×VERDICT_WORDS×4) ÷ MiB⌉，verdict 那几个字略去（只让结论更偏向账）
    if capacity < TABLES:
        return 0
    return (capacity - TABLES) * MIB // SCRATCH_BYTES_PER_STATE


for configured in (24 * 1024, None):
    for free_at_open in (2_000, 3_000, 10_000, 20_000, TOTAL):
        estimate = TOTAL  # 别的记账进程占着时，估计夹在 [F, F + K] 里、照旧是整卡
        ledger_capacity = max(budget_of(configured, estimate) - DRIVER, 0)
        by_budget = states_by_card_budget(budget_of(configured, free_at_open))
        by_ledger = states_fitting_the_ledger(ledger_capacity)
        placed = min(by_budget, by_ledger)
        outcome = "red_no_card_holds_the_tables_and_one_state" if placed == 0 else f"states_per_card={placed}"
        print(
            f"MODEL scenario=s5 configured={configured} free_at_open={free_at_open} card_free_now={TOTAL} "
            f"ledger_capacity={ledger_capacity} states_by_the_opening_free={by_budget} states_the_ledger_allows={by_ledger} "
            f"outcome={outcome}"
        )
