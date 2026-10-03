#!/usr/bin/env python3
"""C1（Q1）：三个候选各自能抬哪一项，纯代数。除下面标 E44、E152 的两个数外，参数都是取样点（没量），只用来看哪个参数让排序翻面，不当本机的数。

每个事务的墙钟 T（吞吐 = 1 ÷ T），流水线稳态取最慢一段，只有比值 c ÷ S 是自由的（把 S 当单位 1）：
  c   一次发布的准备耗时（CPU）：c = c_ser + c_par。c_ser = 必须在号序里串行的一段——transaction.rs 的 prepare_the_version_publish 收
      `allocator: &mut PoolAllocator` 与 `previous`，publish_admitted 里「落点先于内容」，所以取号、准入、取落点、建各棵树都在号序里；
      c_par = 取完落点之后能并行的一段（build_file_version_units 里装数据单元字节与校验和）。f = c_par ÷ c 没量过。
  S   一次发布的设备同步（D16 已定项 7 四个序点）。本机已量：E44 sync_1 = 2785 次/秒 → 359 µs 一次 fdatasync；E152 虚机发布 B 每次 6.6 ms（含 CPU）。
  m   同发：一次发布装 m 个事务；c = c_f + c_p，c_f 是每次发布的固定项，c_p 是每个事务的边际项。
  q   丙：同时在飞的发布里设备同步能重叠几个——设备 flush 的并发度，E44 只量了 sync_1 与 sync_8（8 块一次），没量并发线程数。
形态：
  串行  T0 = c + S
  甲    T_A = max(c_ser + c_par / p, S)        准备与上一个事务的落盘重叠；p 个准备线程
  乙    T_B = (c_f + S) / m + c_p              每个事务摊 1/m 个固定项与 1/m 个同步
  丙    T_C = max(c_ser + c_par / p, S / q)
"""
F_PAR, FIXED_SHARE, P = 0.3, 0.5, 4        # 取样点


def forms(c, m, q, f_par=F_PAR, fixed_share=FIXED_SHARE, p=P, s=1.0):
    c_par, c_ser = c * f_par, c * (1 - f_par)
    c_f, c_p = c * fixed_share, c * (1 - fixed_share)
    return {"串行": c + s, "甲": max(c_ser + c_par / p, s), "乙": (c_f + s) / m + c_p, "丙": max(c_ser + c_par / p, s / q)}


def main():
    print(f"取样点：f={F_PAR}、c_f 占 c 的 {FIXED_SHARE}、p={P}；表里是 T ÷ T0，S 取 1")
    print("① 甲的上限：T_A ÷ T0 ≥ S ÷ (c + S)，即 S 占大头时甲几乎不动（与 m、q 无关）")
    for ratio in (0.05, 0.3, 1.0):
        t = forms(ratio, m=16, q=4)
        print(f"   c/S={ratio:4.2f}  甲 = {t['甲'] / t['串行']:.3f}  （下限 {1 / (1 + ratio):.3f}）")
    print("② 乙与丙谁更好随 (m, q) 翻面（c/S = 0.3；格内：乙 / 丙 的 T ÷ T0，较小的带 *）")
    ms, qs = (2, 4, 8, 16, 32), (1, 2, 4, 8, 16)
    print("      q \\ m  " + "".join(f"{m:>16d}" for m in ms))
    for q in qs:
        cells = []
        for m in ms:
            t = forms(0.3, m=m, q=q)
            b, c_ = t["乙"] / t["串行"], t["丙"] / t["串行"]
            cells.append(f"{b:6.3f}{'*' if b <= c_ else ' '}/{c_:6.3f}{'*' if c_ < b else ' '}")
        print(f"      {q:>4d}   " + "".join(f"{x:>16s}" for x in cells))
    print("③ 乙不输给丙所需的最小 m（c/S = 0.3，按 q 列）：")
    for q in qs:
        need = next((m for m in range(1, 129) if forms(0.3, m=m, q=q)["乙"] <= forms(0.3, m=m, q=q)["丙"]), None)
        print(f"   q={q:2d}: m ≥ {need}")
    print("结论只取方向：甲只抬 c 那一项，S 占大头时上限 1 + c/S；乙靠摊薄，丙靠 q。排序翻面的参数是 q、m、c/S、f——四个都没量过。")


if __name__ == "__main__":
    main()
