"""系统配置槽写协议与几类历史（每类一个函数往 Recorder 里录）。协议照 transaction.rs：
write_system_configuration_slot（世代号 = 这一刻盘上自证过的最大世代号 + 1，从 1 起；槽 = 世代号 mod 2）、
取号写两轮（每轮逐盘一写、轮后一道池屏障）、发布末尾轮换（根槽 FUA 之后，逐盘一写，一道池屏障）、
抬 F（先屏障、逐盘一写、再屏障）、取号失败回卷（先屏障、每块盘一写）。开关 MUT_B1 / MUT_B2 复现第二轮攻方的两个坏实现（读它的 toggles.patch
只为知道变异语义，代码自己写）。"""
from slot_flow_model import Recorder, V, Z, valid

MUT = set()  # 'B1' | 'B2' | 'B3'（B3：只有盘 0 的取号两轮之间少屏障，本模型自己加的）


def next_gen_slot(rec, dev):
    gens = [c[1] for c in rec.disk[dev] if valid(c)]
    g = (max(gens) if gens else 0) + 1
    return g, g % 2


def sys_write(rec, dev):
    g, s = next_gen_slot(rec, dev)
    rec.write(dev, 'sys', s, V(g))
    if 'B1' in MUT and dev == 0:  # 盘 0 写完新槽，同一段里把另一槽清成全 0
        rec.write(dev, 'sys', (g + 1) % 2, Z)
    return g


def acquisition(rec, barrier_between_rounds=True):
    for rnd in (1, 2):
        for dev in rec.devices:
            sys_write(rec, dev)
        if rnd == 1 and (not barrier_between_rounds or 'B2' in MUT):
            continue
        rec.barrier_all()


def opaque_segment(rec, n_writes, kind):
    """不改系统配置槽的段：n_writes 个写（交替落在两块盘上，每块盘都有写），然后每块盘一道屏障。"""
    for k in range(n_writes):
        rec.write(rec.devices[k % len(rec.devices)], kind)
    rec.barrier_all()


def rotation(rec):
    for dev in rec.devices:
        sys_write(rec, dev)
    rec.barrier_all()


def publish(rec, unit_writes, root_device=0):
    """屏障 → 单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置轮换 → 屏障（transaction.rs persist_publish_writes 的形）。"""
    opaque_segment(rec, unit_writes, 'unit')
    opaque_segment(rec, len(rec.devices), 'journal')
    rec.write(root_device, 'root', fua=True)
    rotation(rec)


def raise_floor(rec):
    rec.barrier_all()
    for dev in rec.devices:
        sys_write(rec, dev)
    rec.barrier_all()


def roll_back_acquisition(rec, written_devices):
    """取号失败回卷：先一道池屏障，每块写出过的盘只回卷一次（只在注入写错误的历史里出现，崩溃点重放不枚举它）。"""
    rec.barrier_all()
    for dev in written_devices:
        sys_write(rec, dev)
    rec.barrier_all()


def remount_flow(base, devices=(0, 1)):
    """第二轮攻方 remount 流的形（它自己印出来的段序列 `2,2,14,2,1,2,12,2,1,2`）：取号两轮、一次发布（14 个单元写）、再一次发布（12 个单元写）。"""
    rec = Recorder(base, devices)
    acquisition(rec)
    publish(rec, 14)
    publish(rec, 12)
    return rec.finish()


def seg_sizes(rec):
    return [len(s) for s in rec.segments]


def base_after_first_file(devices=(0, 1)):
    """mkfs 之后第一次可写挂载（取号两轮 + 暖机两次发布）+ 第一个文件那次发布 + 关闭：盘面终态。mkfs 写两槽世代号 0。"""
    rec = Recorder({d: [V(0), V(0)] for d in devices}, devices)
    acquisition(rec)
    publish(rec, 1)
    publish(rec, 1)
    publish(rec, 14)
    return {d: list(rec.disk[d]) for d in devices}


def mkfs_flow(base_slots, devices=(0, 1)):
    """make_filesystem 的写序（段序列 2+12+1+1+1+4）：清 [0,根环基址)、屏障；清根环与 journal 环 + 单元写、屏障；三条第 0 代根 FUA；
    逐盘两槽写（同一段，没有屏障）；逐盘屏障。base_slots 是用过的盘（旧池）或全零（新盘）的槽内容。"""
    rec = Recorder({d: list(base_slots) for d in devices}, devices)
    for d in devices:
        rec.write(d, 'zerofill')  # 清 [0, 根环基址)：罩住两个系统配置槽
    rec.barrier_all()
    for d in devices:
        for _ in range(4):
            rec.write(d, 'zerofill', covers_slots=False)  # 清根环三个区域与 journal 环：不碰系统配置槽
        for _ in range(2):
            rec.write(d, 'unit')
    rec.barrier_all()
    for k in range(3):
        rec.write(0 if k != 1 else 1, 'root', fua=True)
    for d in devices:
        for s in (0, 1):
            rec.write(d, 'sys', s, V(0))
    rec.barrier_all()
    return rec.finish()
