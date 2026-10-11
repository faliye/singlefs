#!/usr/bin/env python3
"""攻方腿模型（m3-run-compare-split-r2）：双机按块亲和（哪台录就在哪台比、录入不跨机）时，权重每趟重量一次，
同一块在两趟之间换了归属的比例 = 判器改了之后要在新归属那台重录的比例。

WeightedShares::owner_of_block 照 E5 副本 crates/singlefs-checker-tier/src/crash_amplification.rs 的平滑加权轮转逐行改写成 Python；
先用同文件单测 weighted_shares_hand_out_blocks_in_proportion_and_never_skip_a_block 的两组钉值自证，对不上退 1。
权重取 ~/.local/share/singlefs/crash-amplification/runs/ 下各趟日志里的 CRASH_AMPLIFICATION_HOST_WEIGHTS 行（命令行给）。
块号两种：流水线是 0..总块数（crash_amplification.rs 里 (0..total_blocks).filter(...owner_of_block(*block)...)），
给定状态表是块键输入指纹头 8 字节（owns_the_table），这里用 sha256(序号) 头 8 字节代替（均匀随机的替身）。
"""
import hashlib
import sys


def cycle(weights):
    """一整个周期（位置 0..total-1）每个位置的归属，与 owner_of_block 逐位置相同（owner_of_block 每次从 0 重走到 position）。"""
    total = sum(weights)
    current = [0] * len(weights)
    owners = []
    for _ in range(total):
        for index, weight in enumerate(weights):
            current[index] += weight
        # max_by_key((current, Reverse(index)))：值相同取下标小的
        owner = max(range(len(current)), key=lambda index: (current[index], -index))
        current[owner] -= total
        owners.append(owner)
    return owners


def owner_of_block(owners, block_sequence):
    return owners[block_sequence % len(owners)]


def selftest():
    three_one = cycle([3, 1])
    first_eight = [owner_of_block(three_one, block) for block in range(8)]
    equal = cycle([1, 1])
    ok = first_eight.count(0) == 6 and first_eight.count(1) == 2 and \
        [owner_of_block(equal, block) for block in range(6)] == [0, 1, 0, 1, 0, 1]
    print(f"SELFTEST weights=3,1 first8={first_eight} weights=1,1 first6={[owner_of_block(equal, b) for b in range(6)]} ok={ok}")
    return ok


def fingerprint_block_number(ordinal):
    return int.from_bytes(hashlib.sha256(f"table-{ordinal}".encode()).digest()[:8], "little")


def main():
    if not selftest():
        sys.exit(1)
    weight_pairs = [tuple(int(x) for x in pair.split(",")) for pair in sys.argv[1:]]
    pipeline_blocks = 39917  # 26.16 亿态 / 65536 态每块（r1 攻方 volume_and_cpu.py 的 blocks=39917）
    table_blocks = 20000
    cycles = {pair: cycle(list(pair)) for pair in weight_pairs}
    for index, before in enumerate(weight_pairs):
        for after in weight_pairs[index + 1:]:
            for kind, numbers in (("pipeline", range(pipeline_blocks)),
                                  ("given_states", [fingerprint_block_number(i) for i in range(table_blocks)])):
                moved = sum(1 for number in numbers
                            if owner_of_block(cycles[before], number) != owner_of_block(cycles[after], number))
                count = len(numbers)
                print(f"OWNER_DRIFT kind={kind} before={before[0]},{before[1]} after={after[0]},{after[1]} "
                      f"blocks={count} moved={moved} moved_fraction={moved / count:.4f}")


if __name__ == "__main__":
    main()
