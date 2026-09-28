"""逐 (文件, 几何, 流数) 比：甲/乙 的逐行分类与 甲/丙（两种点名口径）的是否相同；报峰值比值的最大相对差。"""
import re, sys
total = same = 0
largest_peak_gap = 0.0
for path in sys.argv[1:]:
    by_key = {}
    for line in open(path, encoding='utf-8'):
        if 'row_summary' not in line:
            continue
        record = dict(re.findall(r'(\w+)=(\S+)', line))
        by_key.setdefault((record['geometry'], record['streams']), {})[record['arm_pair']] = record
    for key, arms in by_key.items():
        base = arms['intent_over_write_ahead_log_leaf']
        for other in ('intent_over_third_arm_deduplicated', 'intent_over_third_arm_whole_path'):
            total += 1
            if arms[other]['classification'] == base['classification']:
                same += 1
            else:
                print('分类不同', path, key, other, base['classification'], arms[other]['classification'])
            gap = 1 - float(arms[other]['peak_ratio']) / float(base['peak_ratio'])
            largest_peak_gap = max(largest_peak_gap, gap)
print(f'比了 {total} 对，分类相同 {same}；甲/丙 峰值比 甲/乙 峰值低得最多 {largest_peak_gap*100:.3f}%')
