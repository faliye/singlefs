#!/usr/bin/env python3
"""B5：墓碑树根长高。每个容器装 RPC 条墓碑，根装 F 个容器（内部条目 120 字节，扇出 135）。
算：产品几何下多少次 unlink 之后根满；长高那次发布多写几个节点；树高随容器数怎么长；只供测试的旋钮能让流多短。
内部节点数照 D8 已定项 6「内部节点同样在右端溢出、末尾分裂」：每层按 ⌈下层 ÷ F⌉ 个，直到只剩一个根。
"""
F = (16384 - (115 + 2 * 8)) // 120      # 内部条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86），头 131
RPC = (32768 - 136) // 40
assert (F, RPC) == (135, 815), (F, RPC)
def internal_nodes(c, f=F):
    n, tot = c, 0
    while True:
        n = -(-n // f); tot += n
        if n == 1: return tot
def height(c, f=F):
    h, n = 0, c
    while True:
        n = -(-n // f); h += 1
        if n == 1: return h
print(f"E name=geometry fanout={F} records_per_container={RPC} containers_one_root_holds={F} unlinks_until_root_full={F * RPC}")
first_grow = F * RPC + 1
print(f"E name=first_growth unlink_number={first_grow} nodes_before={internal_nodes(F)} nodes_after={internal_nodes(F + 1)} "
      f"extra_nodes={internal_nodes(F + 1) - internal_nodes(F)} extra_bytes_per_device={16384 * (internal_nodes(F + 1) - internal_nodes(F))}")
for c in (1, 135, 136, 1227, 18225, 18226):
    print(f"E name=tree_shape containers={c} records={c * RPC} internal_nodes={internal_nodes(c)} height_incl_leaf={height(c) + 1}")
# 只供测试的旋钮：一个容器装 rpc 条、根装 f 个容器（f 下限 2，照 set_code_two_tree_node_capacities 的断言）
for rpc, f in ((815, 135), (1, 135), (1, 2), (3, 2)):
    print(f"E name=knob rpc={rpc} root_fanout={f} unlinks_until_growth={rpc * f + 1} unlinks_until_third_level={rpc * f * f + 1}")
# 第二次长高：根（135 个孩子，每个孩子装 135 个容器）装满之后
print(f"E name=second_growth containers={F * F} unlinks={F * F * RPC} nodes_before={internal_nodes(F * F)} nodes_after={internal_nodes(F * F + 1)}")
# 不长高时到满那一刻：拒成什么？D3 已定项 9 第 1 条 假性 ENOSPC：df 空闲 ≥ s 就必须成功；这里 df 报的空闲与结构上限无关
print("E name=refusal_class note=结构上限拒不是 ENOSPC：df 空闲不变、写 s 字节仍成功；拒 unlink 违反 D3 已定项 17 射程「墓碑树根满时长高，不因结构拒删除」")
