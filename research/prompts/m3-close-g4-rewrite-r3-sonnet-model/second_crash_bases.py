"""第二轮攻方「第三截那一形」八个基线：remount 流的四个系统配置段（段号 0、1、5、9）各摆「只落了盘 0 那一写」「只落了盘 1 那一写」
的整写域第一次崩溃镜像；在每个镜像上再起一次 remount 形的流（取号两轮 + 两次发布），数态数与死盘。"""
import sys
sys.path.insert(0, '.')
import protocol_histories as P
from enumerate_states import tally, dead_devices
from slot_flow_model import Recorder, V


def main():
    base = P.base_after_first_file()
    first = P.remount_flow(base)
    seg_writes = {si: [i for i in seg if first.writes[i].kind == 'sys'] for si, seg in enumerate(first.segments)}
    total = 0
    for si in (0, 1, 5, 9):
        for landed in (0, 1):
            disk = {d: list(first.base[d]) for d in first.devices}
            for sj, seg in enumerate(first.segments):
                if sj < si:
                    for i in seg:
                        w = first.writes[i]
                        if w.kind == 'sys':
                            disk[w.dev][w.slot] = w.content
            for i in seg_writes[si]:
                w = first.writes[i]
                if w.dev == landed:
                    disk[w.dev][w.slot] = w.content
            second = P.remount_flow(disk)
            t = tally(second)
            total += t['states']
            print(f"name=second_crash_base segment={si} landed_device={landed} states={t['states']} whole={t['whole']} torn={t['torn']} "
                  f"dead_states(whole+torn)={t['single_dead']['whole'] + t['single_dead']['torn'] + t['all_dead']['whole'] + t['all_dead']['torn']}")
    print('sum_of_the_eight_in_my_model=', total)


if __name__ == '__main__':
    main()
