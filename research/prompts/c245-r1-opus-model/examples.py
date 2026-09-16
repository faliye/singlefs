#!/usr/bin/env python3
"""把 model.py 枚举里各类最短的打中状态单独重放一遍，逐臂打出盘面与恢复结果（报告第二到第五节引的就是这份输出）。"""
import model

ARMS_SHOWN = ('ORACLE', 'JIA_LITERAL', 'JIA_TAIL_FALLBACK', 'JIA_TAIL_CHAIN_ONLY', 'YI_INLINE', 'BING_STRICT',
              'BING_FIRST', 'CUR_IMPL')


def state_of(base, writes, wanted_key):
    for key, image, roots in model.crash_states_with_persisted_roots(base, writes):
        if key == wanted_key:
            return image, roots
    raise AssertionError(f'没有这个崩溃状态：{wanted_key}')


def first_mount_case(title, pattern, crash_key):
    base, writes = model.first_mount_stream(pattern)
    image, _ = state_of(base, writes, crash_key)
    print(f'## {title}')
    print(f'pattern={list(pattern)} crash={crash_key}')
    print(f'image: {model.describe_image(image)}')
    for rule in model.NEXT_RULES:
        for arm in ARMS_SHOWN:
            print('  ' + model.describe_recovery(model.recover(image, arm, rule)))
    print()


def rollback_case(title, pattern, crash1_key, rollback_txg, rollback_records, crash2_key, tail_values=(), isolate=False):
    model.ROLLBACK_ISOLATES_ABOVE_WATER_UNITS = isolate
    base, writes = model.first_mount_stream(pattern)
    image1, _ = state_of(base, writes, crash1_key)
    rollback_root = next(root for root in image1.readable_roots() if root.txg == rollback_txg)
    print(f'## {title}')
    print(f'pattern={list(pattern)} crash1={crash1_key} R_old=({rollback_root.instance},{rollback_root.txg}) '
          f'rollback_records={rollback_records} crash2={crash2_key} isolate_above_water_units={isolate}')
    print(f'image1: {model.describe_image(image1)}')
    for rule in model.NEXT_RULES:
        for arm in ARMS_SHOWN:
            baseline = model.recover(image1, arm, rule)
            writes2, state2 = model.mount_rollback(image1, baseline, rollback_root, rollback_records)
            image2, roots2 = state_of(image1, writes2, crash2_key)
            after = model.recover(image2, arm, rule)
            committed = any(root.instance == state2.instance for root in roots2)
            written = [write[1] for write in writes2 if write[0] == 'record']
            print(f'  [{arm}/{rule}] rollback wrote records {[(r.instance, r.counter) for r in written]}; '
                  f'rollback root persisted={committed}')
            print(f'    image2: {model.describe_image(image2)}')
            print(f'    no-rollback baseline: {model.describe_recovery(baseline)}')
            print(f'    after crash2:         {model.describe_recovery(after)}  '
                  f'same_as_baseline={after.outcome() == baseline.outcome()}')
            for tail in tail_values:
                perturbed = model.recover(image2, arm, rule, tail)
                print(f'    tail={tail}: {model.describe_recovery(perturbed)}')
    model.ROLLBACK_ISOLATES_ABOVE_WATER_UNITS = False
    print()


def counter_and_txg_divergence():
    """第二次挂载的第一条暖机记录里，计数器（前缀末 + 1）与 checkpoint_txg（CJ2）不相等的状态有多少。"""
    diverging, total, shortest = 0, 0, None
    for pattern in model.patterns(4):
        base, writes = model.first_mount_stream(pattern)
        for key, image, _ in model.crash_states_with_persisted_roots(base, writes):
            recovery = model.recover(image, 'ORACLE', 'prefix')
            writes2, _ = model.mount_normal(image, recovery, (1,))
            record = next(write[1] for write in writes2 if write[0] == 'record')
            total += 1
            if record.counter != record.checkpoint_txg:
                diverging += 1
                if shortest is None:
                    shortest = (list(pattern), key, record.counter, record.checkpoint_txg)
    print('## 第二次挂载第一条暖机记录：计数器与 checkpoint_txg 不相等')
    print(f'diverging={diverging} total={total} shortest(pattern, crash, counter, txg)={shortest}')
    print()


def main():
    first_mount_case('层 0 首次挂载：两条记录的发布只落了第二条（丙取「水位之上第一条」、现行实现打中）', (2,), (7, 2))
    first_mount_case('层 0 首次挂载：所选根是 mkfs 第 0 代根、暖机第一条记录落了（现行实现在链首跨实例）', (), (1, 1))
    rollback_case('层 0 回退：回退的记录盖掉所选根覆盖的最后一条（最短，空发布）', (), (4, 1), 0, 1, (1, 1),
                  tail_values=(0, 1))
    rollback_case('层 0 回退：同一形状，读回的文件字节不同（隔离了水位之上的单元，排除单元复用那一类）', (1,), (7, 1), 1, 1,
                  (1, 1), tail_values=(0, 1, 2), isolate=True)
    rollback_case('层 0 回退：回退写两条记录，盖掉基线要施加的那条（与臂无关，prefix 规则下真值也中）', (), (4, 1), 0, 2, (1, 2))
    rollback_case('层 0 回退：回退的单元写复用了基线要施加的那条记录点名的单元（与臂无关）', (1,), (7, 1), 1, 1, (0, 4))
    counter_and_txg_divergence()


if __name__ == '__main__':
    main()
