#!/usr/bin/env python3
"""L1 P1/P2 的另一半：同一串参数经 e165 判法脚本自己的 __main__ 那两步（preflight 摘 --force、main 切参数）落到哪个模式。
replay / check / append 三个函数换成只打模式的桩，不编、不起装置。用法：python3 l1-e165-parse.py <仓根>"""
import importlib.util, os, sys

root = sys.argv[1]
scripts = os.path.join(root, "research", "scripts")
sys.path.insert(0, scripts)
path = os.path.join(scripts, "e165-pin-red-examples.py")
spec = importlib.util.spec_from_file_location("pin_under_probe", path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
module.replay = lambda root_, table, options: print(f"    真跑的模式：replay（target={options['target']!r}）") or 0
module.check = lambda root_, table, ledgers: print("    真跑的模式：check") or 0
module.append = lambda root_, table, ledgers, kill_bounds: print("    真跑的模式：append") or 0
for label, arguments in [("P1", ["--replay", "--target", "--force", "--check"]),
                         ("P2", ["--root", root, "--replay", "--target", "--force", "--check"]),
                         ("P4 对照", ["--check", "--force"])]:
    sys.argv = [path, *arguments]
    print(f"{label} 参数：{' '.join(arguments)}")
    remaining = module.preflight(path)          # 与脚本末尾 `preflight(__file__)` 同一步：摘掉 --force
    print(f"    preflight 之后 sys.argv[1:]：{' '.join(remaining)}")
    print(f"    main 退 {module.main(sys.argv[1:])}")
