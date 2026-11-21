"""fix-configured-root 那一臂的缺省工作根（只在这个模型上量过、被攻过零轮）：多机配置 SHARED_DATA_ROOT 下的 work，不看 SINGLEFS_DATA_ROOT。

编译根（merge-and-verify.sh 的 SLOT_BUILD_ROOT_PROGRAM）只从多机配置读；这一臂让缺省工作根也只从同一份配置读，两样出自同一处。
用法：python3 configured-default-work-root.py <仓根>：打一行；取不到（配置没写、写法不对、不在共享根下）退 2、原因写标准错误。
"""
import importlib.util
import os
import sys

root = sys.argv[1]
specification = importlib.util.spec_from_file_location("lib_work_root_for_pairing_probe", os.path.join(root, ".claude", "hooks", "lib_work_root.py"))
module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(module)
try:
    print(os.path.join(module.configured_data_root(os.environ), "work"))
except ValueError as error:
    print(f"取不到多机配置里的数据根：{error}", file=sys.stderr)
    sys.exit(2)
