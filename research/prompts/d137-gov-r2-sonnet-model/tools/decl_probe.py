#!/usr/bin/env python3
"""declaration_excuses_low 的边界：它把窗口里全部 CPU 秒算给「没声明」的那一截（上界），而声明的串行段至少占一个核。
用法：decl_probe.py <仓根>   只读 import scripts/hosts/core_usage.py"""
import importlib.util, sys
spec = importlib.util.spec_from_file_location("core_usage", sys.argv[1] + "/scripts/hosts/core_usage.py")
core = importlib.util.module_from_spec(spec); spec.loader.exec_module(core)
for cores, span, declared in ((4.0, 600, 300), (3.0, 600, 300), (2.0, 600, 400), (1.0, 600, 600), (1.0, 600, 590)):
    reading = {"cores": cores, "span": span, "threshold": 7.75, "declared_in_window": declared}
    remainder = (cores * span - 1.0 * declared) / (span - declared) if span > declared else float("nan")
    print(f"name=decl_probe cores={cores} span={span} declared={declared} threshold=7.75 excused={core.declaration_excuses_low(reading)} "
          f"remainder_average_if_serial_part_used_one_core={remainder:.2f}")
