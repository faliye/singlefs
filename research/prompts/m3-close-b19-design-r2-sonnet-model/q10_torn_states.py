"""Q10（C874）小模型：补写流（D 两槽全废、L 完好）里，撕裂态能不能造出「新死盘」。
流：取号写两轮，每轮每块盘一次原地覆写，轮后一道屏障（transaction.rs ACQUISITION_WRITE_ROUNDS 两轮；轮内逐盘一写）。
D 第一轮写槽 1，第二轮写槽 0（世代号 mod 2）；L 第一轮覆写较旧的那一槽、第二轮覆写原来较新的那一槽。
每次写的落盘态：没落 / 整写 / 撕裂（新旧字节各一截，不自证）。屏障 = 前一轮的写全是整写。
「死」= 该盘两槽都不自证。基镜像上 D 已死，所以「新死盘」只看 L。
screen=True：两轮之间有屏障（真实流）；screen=False：去掉屏障的核心变异所录出的流（两轮并成一段）。
mutation_torn_as_whole：枚举侧让撕裂退化成整写（crash.rs 的 torn_image_of_in_place_overwrite 不拷旧字节）。
推的模型，不是入库装置；撕裂的字节级细节不模型，只用「撕裂 = 不自证」。"""
import itertools

STATES = ("not", "whole", "torn")

def slots_valid(initial, writes):
    """initial: 槽有效位的元组；writes: {槽: 落盘态}；原地覆写：没落=旧内容，整写=有效，撕裂=不自证"""
    out = list(initial)
    for s, st in writes.items():
        if st == "whole":
            out[s] = True
        elif st == "torn":
            out[s] = False
    return tuple(out)

def enumerate_flow(screen, torn_as_whole=False):
    D0 = (False, False); L0 = (True, True)
    rounds = [  # (D 写的槽, L 覆写的槽)；L 覆写的槽覆盖的是原来有效的内容
        (1, 0),
        (0, 1),
    ]
    states = []
    if screen:
        # 第 r 轮崩溃：之前的轮全是整写，本轮每块盘各 3 态
        for r, (d_slot, l_slot) in enumerate(rounds):
            d_prev = {rr[0]: "whole" for rr in rounds[:r]}
            l_prev = {rr[1]: "whole" for rr in rounds[:r]}
            for dst, lst in itertools.product(STATES, STATES):
                if torn_as_whole:
                    dst = "whole" if dst == "torn" else dst
                    lst = "whole" if lst == "torn" else lst
                d = slots_valid(D0, {**d_prev, d_slot: dst})
                l = slots_valid(L0, {**l_prev, l_slot: lst})
                states.append((r, dst, lst, d, l))
    else:
        # 去掉屏障：两轮并成一段，4 次写各 3 态
        for sts in itertools.product(STATES, repeat=4):
            if torn_as_whole:
                sts = tuple("whole" if s == "torn" else s for s in sts)
            d1, l1, d0, l0 = sts
            d = slots_valid(D0, {1: d1, 0: d0})
            l = slots_valid(L0, {0: l1, 1: l0})
            states.append((0, (d1, d0), (l1, l0), d, l))
    return states

def stats(states):
    new_dead_L = sum(1 for s in states if not any(s[4]))
    D_dead = sum(1 for s in states if not any(s[3]))
    return len(set(map(lambda s: s[1:3], states))), len(states), new_dead_L, D_dead

if __name__ == "__main__":
    for screen in (True, False):
        for mut in (False, True):
            states = enumerate_flow(screen, mut)
            _, n, newL, Dd = stats(states)
            print(f"screen_between_rounds={screen} torn_as_whole_mutation={mut} states_total={n} new_dead_L_states={newL} D_without_verified_slot_states={Dd}")
