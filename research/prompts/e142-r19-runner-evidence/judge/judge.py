#!/usr/bin/env python3
"""E142 第十九次跑：主产物模型那一侧对登记锚点逐格比（Q142.47、Q142.48 新旧格、Q142.57、Q142.60、Q142.61、R41 行数）。
用法：judge.py <主产物> <臂 N18 产物> [--self-proof <几号>]
--self-proof 1：G10a 的期望段序列换成 G10b 的（登记 8.2 判别力自证 ①）；2：mkfs 的期望 m 改成 2+0+0+0+0（②）；
3：G11 整条的期望段序列换成 C5 的（R19C-7 ③）；4：C5 整条的期望换成 B5 的（R19C-7 ④）。"""
import argparse, collections, copy, re, sys
sys.path.insert(0, '/tmp/claude-1000/e142-r19-runner/judge')
import anchors

def parse(line):
    return dict(re.findall(r'(\S+?)=(\S*)', line.split(' ', 1)[1] if line.startswith('E7RESULT ') else line))

def model_side(path):
    rows = []
    for raw in open(path, encoding='utf-8'):
        line = raw.rstrip('\n')
        if not line.startswith('E7RESULT '):
            continue
        fields = parse(line)
        if fields.get('name') == 'done':
            break
        rows.append(fields)
    return rows

def multiset_segments(kinds_text):
    out = []
    for segment in kinds_text.split('|'):
        counts = collections.Counter()
        for item in segment.strip('[]').split(','):
            if '×' in item:
                kind, count = item.split('×')
                counts[kind] += int(count)
            else:
                counts[item] += 1
        out.append(dict(counts))
    return out

def verdict(cell, field, model, expected, source):
    equal = model == expected
    print(f'name={cell} {field} model={model} expected={expected} equal={str(equal).lower()} source={source}')
    return equal

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('main_product'); parser.add_argument('arm_n18_product')
    parser.add_argument('--self-proof', type=int, default=0)
    arguments = parser.parse_args()
    n19c, g11, swallowed = copy.deepcopy(anchors.N19C), copy.deepcopy(anchors.G11), copy.deepcopy(anchors.SWALLOWED)
    if arguments.self_proof == 1:
        for path in ('transaction', 'post_mkfs_stream'):
            swallowed[('G10a', path)]['segments'] = swallowed[('G10b', path)]['segments']
    elif arguments.self_proof == 2:
        n19c['mkfs']['in_place_per_segment'] = '2+0+0+0+0'
    elif arguments.self_proof == 3:
        g11['post_mkfs_stream']['segments'] = anchors.N19C['post_mkfs_stream']['segments']
    elif arguments.self_proof == 4:
        n19c['post_mkfs_stream']['segments'] = anchors.G11['post_mkfs_stream']['segments']
        n19c['post_mkfs_stream']['closed_form'] = anchors.G11['post_mkfs_stream']['closed_form']
        n19c['post_mkfs_stream']['closed_form_three_state'] = anchors.G11['post_mkfs_stream']['closed_form_three_state']
    rows = model_side(arguments.main_product)
    n18_rows = model_side(arguments.arm_n18_product)
    segments_rows = [row for row in rows if row.get('name') == 'segments']
    three_rows = [row for row in rows if row.get('name') == 'segments_three_state']
    sensitivity_rows = [row for row in rows if row.get('name') == 'segments_sensitivity']
    paths = ['mkfs', 'instance_acquisition', 'warm_up', 'transaction', 'post_mkfs_stream']
    # R41 / V19e（按 R19C-6 第 8 格）：行数与 path= 集合。
    print(f'name=row_counts segments={len(segments_rows)} segments_three_state={len(three_rows)} segments_sensitivity={len(sensitivity_rows)} '
          f'segments_paths_ok={str(sorted(row["path"] for row in segments_rows) == sorted(paths)).lower()} '
          f'three_state_paths_ok={str(sorted(row["path"] for row in three_rows) == sorted(paths)).lower()}')
    by_path = {row['path']: row for row in segments_rows}
    three_by_path = {row['path']: row for row in three_rows}
    for path in paths:
        anchor = n19c[path]
        for field in ('operations', 'segments', 'closed_form', 'kinds'):
            verdict('q48', f'arm=N19C path={path} field={field}', by_path[path][field], anchor[field], anchor['source'])
        for field in ('in_place_per_segment', 'in_place_kinds'):
            verdict('q48', f'arm=N19C path={path} field={field}', three_by_path[path][field], anchor[field], anchor['source'])
        for field in ('closed_form_three_state', 'in_place_per_segment', 'in_place_overwrites'):
            verdict('q47', f'arm=N19C path={path} field={field}', three_by_path[path][field], anchor[field], anchor['source'])
        verdict('q47', f'arm=N19C path={path} field=closed_form_two_state_vs_segments_row', three_by_path[path]['closed_form_two_state'], by_path[path]['closed_form'], 'same_product')
    write_list = next(row for row in rows if row.get('name') == 'write_list')
    for field in ('writes', 'barriers', 'fua'):
        verdict('q48', f'arm=N19C row=write_list field={field}', write_list[field], anchors.WRITE_LIST[field], anchors.WRITE_LIST['source'])
    verdict_row = next(row for row in rows if row.get('name') == 'verdict')
    print(f'name=q48 arm=N19C row=verdict field=write_list_ok value={verdict_row["write_list_ok"]}')
    before_window = next(row for row in rows if row.get('name') == 'before_window_summary' and row.get('side') == 'model')
    for field in ('operations', 'writes'):
        verdict('c10', f'arm=N19C row=before_window_summary field={field}', before_window[field], anchors.BEFORE_WINDOW[field], anchors.BEFORE_WINDOW['source'])
    layer0 = next(row for row in rows if row.get('name') == 'layer0')
    verdict('c12', 'arm=N19C row=layer0 field=closed_form', layer0['closed_form'], anchors.LAYER0_CLOSED_FORM, 'C12')
    verdict('c12', 'arm=N19C row=layer0 field=segments', layer0['segments'], n19c['post_mkfs_stream']['segments'], 'C12=C5')
    # 吞一步屏障的四点与 G11。
    n18_by_path = {row['path']: row for row in n18_rows if row.get('name') == 'segments'}
    for row in sensitivity_rows:
        point, path = row['point'], row['path']
        if point == 'G11':
            anchor, cell = g11[path], 'q60'
            for field in ('segments', 'closed_form'):
                verdict('q60', f'point=G11 path={path} field={field}_vs_arm_n18', row[field], n18_by_path[path][field], 'arm_n18_product')
        else:
            anchor, cell = swallowed[(point, path)], ('q57' if point.startswith('G10') else 'q61')
        for field in ('operations', 'segments', 'closed_form', 'closed_form_three_state', 'in_place_per_segment'):
            verdict(cell, f'point={point} path={path} field={field}', row[field], anchor[field], anchor['source'])
        if isinstance(anchor['kinds'], str):
            verdict(cell, f'point={point} path={path} field=kinds', row['kinds'], anchor['kinds'], anchor['source'])
        else:
            equal = multiset_segments(row['kinds']) == anchor['kinds']
            print(f'name={cell} point={point} path={path} field=kinds_multiset model={row["kinds"]} multiset_equal={str(equal).lower()} source={anchor["source"]}')

if __name__ == '__main__':
    main()
