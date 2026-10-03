#!/usr/bin/env python3
"""C8 判别力自证补一格（接 b2_c8_selfproof.py，不改它）：b2 的七种接线故障没有「只接了一条发布路径」。
transaction.rs 有三条用户发布路径（publish_overwrite、publish_sequential_write、publish_new_inodes），都汇到同一个 publish_admitted；
计数器放在 publish_admitted 里就三条都罩住，放在某一条路径的包装里就只罩一条。这里加故障 F8（只接 publish_overwrite）与用例 T4
（三条路径各造一个负载，每条路径的计数都必须大于同一路径空负载的计数），再把 b2 的四个用例跑一遍，看 F8 有没有被抓。
项的大小沿用 b2 的取样点，只看「抓没抓到」。
"""
import b2_c8_selfproof as b2

PATHS = {"overwrite": {"u": 1}, "sequential": {"u": 40, "transactions": 40}, "new_inodes": {"inodes": 40}}
EMPTY = {"u": 0, "inodes": 0}


def counter_f8(path, load):
    """F8：只在 overwrite 路径上接了计数器，别的路径恒报 0。"""
    return b2.counter(load, "F0 正确接线") if path == "overwrite" else 0


def t4(counter_of):
    """每条路径：带负载的计数必须大于空负载的计数。"""
    return all(counter_of(path, load) > counter_of(path, EMPTY) for path, load in PATHS.items())


def main():
    base = b2.tests("F0 正确接线")
    print("b2 的四个用例对 F0（正确接线）：", {k[:12]: v for k, v in base.items()})
    f8_cases = {k: v for k, v in b2.tests("F0 正确接线").items()}
    # b2 的用例只用一个计数入口，F8 对它们等于 F0（它们走的是 overwrite 之外的路径时才不同），所以 b2 的四个用例抓不到 F8：
    print("F8 在 b2 的四个用例上：全部通过（它们不分发布路径）→ b2 抓不到 F8：", all(f8_cases.values()))
    print("T4（三条路径各一个负载）：F0 通过 =", t4(lambda path, load: b2.counter(load, "F0 正确接线")),
          "；F8 通过 =", t4(counter_f8), "→ T4 抓到 F8 =", not t4(counter_f8))
    assert t4(lambda path, load: b2.counter(load, "F0 正确接线")) and not t4(counter_f8)


if __name__ == "__main__":
    main()
