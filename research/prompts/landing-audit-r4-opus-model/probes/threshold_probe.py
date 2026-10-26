"""K3 ⑬ 探针：按文件路径导入草稿里拷的 agent-watch.py，直接调 memory_threshold，峰值从 0 到 20 GiB 扫一遍，看阈值随峰值怎么走。"""
import importlib.util, sys, types
spec = importlib.util.spec_from_file_location("agent_watch_copy", sys.argv[1])
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
GiB, MiB = 1024 ** 3, 1024 ** 2
thresholds = types.SimpleNamespace(process_rss_gibibytes=8, process_rss_floor_mebibytes=512, process_rss_cap_percent=90)
for peak in (0, 1, 4096, 100 * MiB, 300 * MiB, 341 * MiB, 400 * MiB, 2 * GiB, 5 * GiB, 6 * GiB, 20 * GiB):
    limit, text, source = module.memory_threshold(1, None, thresholds, 8 * GiB, lambda pid: None, lambda pid, p=peak: p)
    print(f"峰值 {peak / MiB:10.3f} MiB → 阈值 {limit / MiB:10.1f} MiB（{source}）")
