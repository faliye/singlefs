"""每个几何点一行：各流数的逐行分类（只看指定臂对），另报这个几何点上哪些流数不是「丙行」、有覆写的行数。"""
import re, sys
arm = sys.argv[1]
rows = {}
order = []
for path in sys.argv[2:]:
    for line in open(path, encoding='utf-8'):
        if 'name=row_summary' not in line or f'arm_pair={arm} ' not in line:
            continue
        record = dict(re.findall(r'(\w+)=(\S+)', line))
        key = (record['geometry'], record['node_levels'], record['root_children'], record['fanout'])
        if key not in rows:
            rows[key] = []
            order.append(key)
        rows[key].append(record)
for key in order:
    records = rows[key]
    text = ' '.join(f"{r['streams']}:{r['classification']}" + ('*' if r['overwrite_cells'] != '0' else '') for r in records)
    rising = [r['streams'] for r in records if r['classification'] != '丙行']
    print(f"L={key[1]} rc={key[2]} fanout={key[3]} 非丙行流数={','.join(rising) or '无'} | {text}")
