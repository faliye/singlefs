"""把副本模型 reproduce 模式的逐格与逐行结果，与入库产物逐字段比。用法：compare_with_product.py 副本输出 产物 [产物…]"""
import re
import sys

def fields_of(line):
    return dict(re.findall(r'(\w+)=(\S+)', line))

model_path, product_paths = sys.argv[1], sys.argv[2:]
model_cells, model_rows = {}, {}
for line in open(model_path, encoding='utf-8'):
    record = fields_of(line)
    if 'name=cell ' in line:
        model_cells[(record['geometry'], record['streams'], record['batch'])] = record
    elif 'name=row_summary ' in line and record.get('arm_pair') == 'intent_over_write_ahead_log_leaf':
        model_rows[(record['geometry'], record['streams'])] = record
geometries = {key[0] for key in model_cells}
product_cells, product_rows = {}, {}
for path in product_paths:
    for line in open(path, encoding='utf-8'):
        record = fields_of(line)
        if record.get('geometry') not in geometries:
            continue
        if 'name=cell ' in line:
            product_cells[(record['geometry'], record['streams'], record['batch'])] = record
        elif 'name=row_summary ' in line:
            product_rows[(record['geometry'], record['streams'])] = record
cell_differences = 0
for key, product in product_cells.items():
    model = model_cells.get(key)
    if model is None:
        cell_differences += 1
        print('缺格', key)
        continue
    for field in ('intent_avg', 'write_ahead_log_leaf_avg', 'ratio'):
        if model[field] != product[field]:
            cell_differences += 1
            print('格不同', key, field, model[field], product[field])
row_differences = 0
for key, product in product_rows.items():
    model = model_rows.get(key)
    for field in ('classification', 'peak_ratio', 'peak_batch_low', 'peak_batch_high'):
        if model is None or model[field] != product[field]:
            row_differences += 1
            print('行不同', key, field, None if model is None else model[field], product[field])
print(f'几何点 {sorted(geometries)}；产物格 {len(product_cells)}、副本格 {len(model_cells)}、字段不同 {cell_differences}；产物行 {len(product_rows)}、副本行 {len(model_rows)}、字段不同 {row_differences}')
