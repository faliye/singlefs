"""Q3 第 4 问：GPU 没有对拍镜像的两形（往下认两层、第 0 代根那一种）各一张镜像在今天的 CPU 判法（ding_today 转写）上判什么，推的模型。
几何 R = 3、S = 2、DEVS = (0, 1, 0)；落点式 I-7.17：区域 txg mod 3、槽 (txg div 3) mod 2。根是 (txg, 实例, F, state, 盘)，
state 是用户可见树根指针的代号，ABSENT 是两棵都没有；journal 是 {txg: [(实例, F, state)]}（带提交标记、重放读得到的）。
用法：python3 q3_mirrors.py
"""
from ding_today import ABSENT, judge_raise

R, S, DEVS = 3, 2, (0, 1, 0)


def root(t, inst, f, st, region):
    return (t, inst, f, st, DEVS[region])


def genesis_form(raised):
    # mkfs；实例 1：1 写行、2 暖机（都 ABSENT）、3 A、4 B、5 C 三版带文件；6 D 记录落了、根槽没落（崩）。
    # 实例 2：恢复施加 6，写行 (1, T=6)；7 写行、8 暖机（D）、9 抬 F（D，F=raised）、10（D）、11 E、12 F_（盖掉区域 0 槽 0 的第 0 代根）。
    image = {(0, 0): root(12, 2, raised, "F_", 0), (1, 0): root(7, 2, 0, "D", 1), (2, 0): root(8, 2, 0, "D", 2),
             (0, 1): root(9, 2, raised, "D", 0), (1, 1): root(10, 2, raised, "D", 1), (2, 1): root(11, 2, raised, "E", 2)}
    journal = {3: [(1, 0, "A")], 4: [(1, 0, "B")], 5: [(1, 0, "C")], 6: [(1, 0, "D")], 7: [(2, 0, "D")], 8: [(2, 0, "D")],
               9: [(2, raised, "D")], 10: [(2, raised, "D")], 11: [(2, raised, "E")], 12: [(2, raised, "F_")]}
    rows = {1: 6}
    return image, journal, rows, image[(0, 1)]


def two_layer_form(raised):
    # 实例 1：1 写行、2 暖机（ABSENT）、3 A 记录落了、根槽没落（崩）。实例 2：恢复施加 3，写行 (1, T=3)；4 写行、5 暖机（A）、6 B、7 C、8 D、
    # 9 E 记录落了、根槽没落（崩）。实例 3：恢复施加 9，写行 (2, T=9)；10 写行、11 暖机（E）、12 抬 F（E，F=raised）、13（E）、14 G、15 H。
    # 区域 0 槽 1 落点式住 3、9、15：3、9 都没落，抬 F 那一刻全零；镜像上住 15。
    image = {(0, 0): root(12, 3, raised, "E", 0), (1, 0): root(13, 3, raised, "E", 1), (2, 0): root(14, 3, raised, "G", 2),
             (0, 1): root(15, 3, raised, "H", 0), (1, 1): root(10, 3, 0, "E", 1), (2, 1): root(11, 3, 0, "E", 2)}
    journal = {3: [(1, 0, "A")], 6: [(2, 0, "B")], 7: [(2, 0, "C")], 8: [(2, 0, "D")], 9: [(2, 0, "E")],
               10: [(3, 0, "E")], 11: [(3, 0, "E")], 12: [(3, raised, "E")], 13: [(3, raised, "E")], 14: [(3, raised, "G")],
               15: [(3, raised, "H")]}
    rows = {1: 3, 2: 9}
    return image, journal, rows, image[(0, 0)]


def genesis_copy_alone_on_a_device(raised):
    # R = 3、S = 4（环长 12）。区域 1 那块盘（盘 1）上落点式住的 txg 1、4、7、10、13 每一版都是「记录落了、根槽没落」崩掉：
    # 实例 1：1 写行（崩；它是实例 1 的第一条、跟在第 0 代根后面，前缀不跨实例，不施加）。实例 2：写行 (1, T=0)；2 写行、3 暖机（ABSENT）、4 A（崩）。
    # 实例 3：施加 4，行 (2, 4)；5 写行、6 暖机（A）、7 B（崩）。实例 4：施加 7，行 (3, 7)；8 写行、9 暖机（B）、10 C（崩）。
    # 实例 5：施加 10，行 (4, 10)；11 写行、12 暖机（C）、13 D（崩）。实例 6：施加 13，行 (5, 13)；14 写行、15 暖机（D）、16 抬 F、17 带新 F。
    # 区域 1 槽 0 一直是 mkfs 种的第 0 代根，槽 2、3 全零，槽 1 抬 F 之后住 16。
    image = {(0, 0): root(12, 5, 0, "C", 0), (0, 1): root(15, 6, 0, "D", 0), (0, 2): root(6, 3, 0, "A", 0), (0, 3): root(9, 4, 0, "B", 0),
             (1, 0): root(0, 0, 0, ABSENT, 1), (1, 1): root(16, 6, raised, "D", 1), (1, 2): None, (1, 3): None,
             (2, 0): root(14, 6, 0, "D", 2), (2, 1): root(17, 6, raised, "D", 2), (2, 2): root(8, 4, 0, "B", 2), (2, 3): root(11, 5, 0, "C", 2)}
    journal = {1: [(1, 0, ABSENT)], 2: [(2, 0, ABSENT)], 3: [(2, 0, ABSENT)], 4: [(2, 0, "A")], 5: [(3, 0, "A")], 6: [(3, 0, "A")],
               7: [(3, 0, "B")], 8: [(4, 0, "B")], 9: [(4, 0, "B")], 10: [(4, 0, "C")], 11: [(5, 0, "C")], 12: [(5, 0, "C")],
               13: [(5, 0, "D")], 14: [(6, 0, "D")], 15: [(6, 0, "D")], 16: [(6, raised, "D")], 17: [(6, raised, "D")]}
    rows = {1: 0, 2: 4, 3: 7, 4: 10, 5: 13}
    return image, journal, rows, image[(1, 1)]


def with_genesis_on_device_zero(image):
    """写者 mount.rs 3958–3968 的口径：第 0 代根按 target_for_publish(0) 落区域 0 的盘。"""
    return {k: (None if v is None else (v[:4] + (DEVS[0],) if v[0] == 0 else v)) for k, v in image.items()}


def main():

    for name, form, legal_f, bad_f in (("genesis_option", genesis_form, 3, 4), ("two_layers_below_u", two_layer_form, 6, 7)):
        for raised in (legal_f, bad_f):
            image, journal, rows, raising = form(raised)
            for variant_name, variant in (("today", {}), ("genesis_known", {"genesis_state": ABSENT})):
                j = judge_raise(image, journal, rows, raising, DEVS, R, S, **variant)
                print(f"name=q3_mirror form={name} raised_F={raised} legal={raised == legal_f} variant={variant_name} "
                      f"window={j['window']} possible_rings={j['rings']} final={j['final']}")
    # 第 0 代根的盘归属：写者按区域 0 的盘算上限是 5，checker 按它所在那一槽的区域（盘 1）算
    for raised in (5, 6):
        image, journal, rows, raising = genesis_copy_alone_on_a_device(raised)
        for variant_name, img, variant in (("today", image, {}),
                                           ("writer_device_for_genesis", with_genesis_on_device_zero(image), {"genesis_device": "txg"})):
            j = judge_raise(img, journal, rows, raising, DEVS, R, 4, **variant)
            print(f"name=q3_mirror form=genesis_copy_alone_on_a_device raised_F={raised} writer_ceiling=5 variant={variant_name} "
                  f"window={j['window']} possible_rings={j['rings']} final={j['final']}")


if __name__ == "__main__":
    main()
