"""一键复跑：第三轮正推腿的全部量（纯 Python，几秒）。每行 `name=… key=value`。"""
import sys
sys.path.insert(0, '.')
import protocol_histories as P
import closure_check as C
from enumerate_states import tally, states, dead_devices
from slot_flow_model import V, Z, T, valid, Recorder


def line(name, rec):
    t = tally(rec)
    print(f"name={name} segments={P.seg_sizes(rec)} states={t['states']} whole={t['whole']} torn={t['torn']} "
          f"single_dead(whole/torn)={t['single_dead']['whole']}/{t['single_dead']['torn']} "
          f"all_dead(whole/torn)={t['all_dead']['whole']}/{t['all_dead']['torn']}")
    return t


def second_attack_numbers():
    print('# 第二轮攻方的数（我的模型照切段规则 + 原地覆写第三态判据 + 写协议独立推出）')
    P.MUT.clear()
    base = P.base_after_first_file()
    line('remount_correct', P.remount_flow(base))
    P.MUT.add('B1')
    line('MUT_B1_base_built_with_the_mutation_like_the_prototype', P.remount_flow(P.base_after_first_file()))
    line('MUT_B1_base_built_clean', P.remount_flow(base))
    P.MUT.clear(); P.MUT.add('B2')
    line('MUT_B2', P.remount_flow(base))
    P.MUT.clear()
    rec = Recorder(base)
    P.acquisition(rec)
    P.publish(rec, 14)
    P.publish(rec, 12)
    P.publish(rec, 12)
    line('remount_plus_one_more_12_unit_publish_the_segment_list_the_attacker_printed_for_its_bases_at_segment_5', rec.finish())
    P.MUT.clear(); P.MUT.add('B2')
    per_device = {}
    for si, dom, dsk, dead, wgt in states(P.remount_flow(base)):
        if dead:
            per_device[(dom, dead)] = per_device.get((dom, dead), 0) + wgt
    print('name=MUT_B2_dead_states_by_domain_and_device', sorted(per_device.items()))
    P.MUT.clear()
    print('second_crash_group_arithmetic 6*20519+2*24626 =', 6 * 20519 + 2 * 24626, ' (also = 6*(20499+20)+2*(24601+25))')
    print('four_groups_total 3*20519+172366 =', 3 * 20519 + 172366)


def mkfs_tail():
    print('# mkfs 末段（四个系统配置槽写同一段、没有屏障）')
    for label, base_slots in (('used_pool', [V(77), V(77)]), ('fresh_pool', [Z, Z])):
        rec = P.mkfs_flow(base_slots)
        t = line(f'mkfs_{label}', rec)
        by_segment = {}
        for si, dom, dsk, dead, wgt in states(rec):
            if len(dead) == 1:
                by_segment[(si, dom)] = by_segment.get((si, dom), 0) + wgt
        print(f"name=mkfs_{label}_single_dead_by_segment(index,domain)={sorted(by_segment.items())}")
    P.MUT.clear()


def indistinguishable():
    print('# 同一张盘面能不能分出「mkfs 中途」与「mkfs 之后」')
    rec = P.mkfs_flow([V(77), V(77)])
    last = len(rec.segments) - 1
    mid_images = {}
    for si, dom, dsk, dead, wgt in states(rec):
        if si == last:
            mid_images[tuple((d, tuple(dsk[d])) for d in sorted(dsk))] = (dom, dead)
    post = {d: [V(0), V(0)] for d in (0, 1)}
    lost = {}
    for dev in (0, 1):
        for dead_pair in ((Z, Z), (T, Z), (Z, T), (T, T)):
            img = {d: list(post[d]) for d in post}
            img[dev] = list(dead_pair)
            lost[tuple((d, tuple(img[d])) for d in sorted(img))] = dev
    found = [k for k in lost if k in mid_images]
    print(f"name=post_mkfs_pool_with_one_device_lost images={len(lost)} "
          f"of_them_equal_to_a_state_inside_the_mkfs_tail_segment={len(found)} "
          f"(by_domain={sorted(set(mid_images[k][0] for k in found))})")


def evidence_heuristic():
    print('# 丙-I 若用「盘面上有 mkfs 已返回的证据」分中途与之后：证据 = 某个自证过的槽世代号 ≥ 1')
    def evidence(dsk):
        return any(valid(c) and c[1] >= 1 for d in dsk for c in dsk[d])
    for label, mut in (('correct', None), ('MUT_B2', 'B2')):
        P.MUT.clear()
        if mut:
            P.MUT.add(mut)
        rec = Recorder({d: [V(0), V(0)] for d in (0, 1)})
        P.acquisition(rec)
        P.publish(rec, 1)
        rec.finish()
        single = missed = 0
        for si, dom, dsk, dead, wgt in states(rec):
            if len(dead) == 1:
                single += wgt
                if not evidence(dsk):
                    missed += wgt
        print(f"name=first_mount_after_mkfs_{label} single_dead_states={single} without_evidence_of_mkfs_return={missed}")
    P.MUT.clear()


def inherited_dead_flow():
    print('# 补写流：起点就有一块盘两槽全废（手造），正确实现的取号两轮 + 两次发布')
    P.MUT.clear()
    base = {0: [V(4), V(5)], 1: [Z, Z]}
    rec = P.remount_flow(base)
    t = line('rewrite_flow_from_a_dead_device', rec)
    ok = sum(w for si, dom, dsk, dead, w in states(rec) if not dead)
    print(f"name=rewrite_flow_states_still_dead_before_the_second_round_persisted dead_states={t['states'] - ok}")


def closure():
    print('# 可达盘面闭包（正确实现）：mkfs 返回后反复在任意崩溃状态上再挂载 / 抬 F / 发布')
    start = C.canon({0: [V(0), V(0)], 1: [V(0), V(0)]})
    P.MUT.clear()
    seen, dead, rounds = C.run(C.FLOWS, start)
    bad = sum(1 for img in seen if any(not any(valid(c) for c in dv) for dv in img))
    print(f"name=closure_correct fixpoint_rounds={rounds} reachable_images={len(seen)} images_with_a_dead_device={bad} dead_states_met={len(dead)}")
    base = C.canon(P.base_after_first_file())
    rec = P.remount_flow(P.base_after_first_file())
    firsts = {}
    for si, dom, dsk, d, w in states(rec):
        if si in (0, 1, 5, 9) and dom == 'whole':
            firsts[C.canon(dsk)] = si
    print(f"name=second_crash_bases whole_domain_images_of_the_four_system_configuration_segments={len(firsts)} "
          f"all_inside_the_closure={all(c in seen for c in firsts)}")


def miss_classes():
    print('# 现行 I-8.5（只判全池皆废）漏掉哪一类：只让一块盘的两槽落进同一段的坏实现')
    P.MUT.clear()
    base = P.base_after_first_file()
    rec = Recorder(base)
    for rnd in (1, 2):
        for dev in rec.devices:
            P.sys_write(rec, dev)
        if rnd == 1:
            rec.barrier(1)  # 只有盘 0 的屏障被吞，盘 1 照发
        else:
            rec.barrier_all()
    P.publish(rec, 14)
    P.publish(rec, 12)
    line('B3_only_device_zero_skips_its_barrier_after_round_one', rec.finish())
    rec = Recorder(base)
    P.sys_write(rec, 0)
    P.sys_write(rec, 0)
    P.sys_write(rec, 1)
    rec.barrier_all()
    line('B4_device_zero_writes_both_slots_in_one_segment_device_one_once', rec.finish())


def double_write_equivalence():
    print('# 单盘全废态可达 ⟺ 某一段里有一块盘的系统配置槽写不止一个（健康起点、写协议照 transaction.rs、基镜像两槽非零）')
    import itertools
    base = {0: [V(4), V(5)], 1: [V(4), V(5)]}
    total = dead_without_double = dead_with_double = double_without_dead = 0
    for per_segment_writes in itertools.product(itertools.product((0, 1, 2), repeat=2), repeat=3):
        rec = Recorder(base)
        for segment in per_segment_writes:
            for dev, k in enumerate(segment):
                for _ in range(k):
                    P.sys_write(rec, dev)
            rec.barrier_all()
        rec.finish()
        dead = any(d for si, dom, dsk, d, w in states(rec))
        double = any(k == 2 for segment in per_segment_writes for k in segment)
        total += 1
        dead_without_double += dead and not double
        dead_with_double += dead and double
        double_without_dead += double and not dead
    print(f"name=double_write_equivalence flows={total} (3_segments_each_device_0_1_or_2_slot_writes) "
          f"dead_state_reachable_without_a_double_write={dead_without_double} "
          f"dead_state_reachable_with_a_double_write={dead_with_double} double_write_without_any_dead_state={double_without_dead}")


if __name__ == '__main__':
    second_attack_numbers()
    mkfs_tail()
    indistinguishable()
    evidence_heuristic()
    miss_classes()
    double_write_equivalence()
    inherited_dead_flow()
    closure()
