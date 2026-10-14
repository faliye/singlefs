#!/usr/bin/env python3
"""host-dispatch.py ⓪ 的 header_hosts 判法函数（它经 judge_script_header 判被起脚本的文件头）：打 multi（多台）或 single（只占一台）与理由。
用法：hd_header.py <冻结副本根> <脚本路径>…"""
import importlib.util, os, sys
root = os.path.realpath(sys.argv[1])
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("host_dispatch_under_test", os.path.join(root, "scripts/hosts/host-dispatch.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
for script in sys.argv[2:]:
    hosts, why = module.header_hosts(["bash", os.path.abspath(script)], root)
    print(f"{os.path.basename(script)}\t{hosts}\t{str(why)[:200]}")
