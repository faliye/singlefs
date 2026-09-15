#!/usr/bin/env python3
"""C161（缓冲两级并存而衔接没定） 第二轮攻方腿：丙（主 agent 的倾向）单独往深处穷举。
通用负载（两棵子树 + grow / shrink / rebalance + 部分下推）到深度 12，记账负载 K = 2 / 3 到深度 15；
同一深度上跑一次改坏的丙（N4：前端树上多一个直落叶的写者），它必须被抓到，证明这套检查在这个深度上分得出差别。
复跑：python3 -B c161_r2_bing_deep.py > c161-r2-bing-deep-output.txt（产物末行 emitted=N，行数对不上即作废）。
"""
import sys

import c161_r2_model as model


def main():
    depth_generic = int(sys.argv[1]) if len(sys.argv) > 1 else 12
    depth_accounting = int(sys.argv[2]) if len(sys.argv) > 2 else 15
    lines = []
    bing = model.make_arm('bing')
    model.run_u1(lines.append, 'generic', bing, model.generic_operations(bing), depth_generic)
    # 阳性对照：N4 第 2 步就中，深度封顶 8，只证明检查在这套装置上会红
    broken = model.make_arm('bing+N4', 'bing', direct_target='leaf_blind')
    model.run_u1(lines.append, 'generic', broken, model.generic_operations(broken), min(depth_generic, 8))
    for retained in (2, 3):
        model.run_u1(lines.append, f'accounting_K{retained}', bing, model.accounting_operations(bing, retained),
                     depth_accounting)
    lines.append(f"emitted={len(lines)}")
    print('\n'.join(lines))


if __name__ == '__main__':
    main()
