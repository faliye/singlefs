#!/usr/bin/env python3
"""辩方腿的纯文本量：records/ 下以 | 起头的表格里，批号落在哪一列。不起判据库，只读仓里的 .md。
用法：python3 scan_table_columns.py --repo <仓根>
打出三行：name=table_header_position（表头里有「批次」那一格，它在最后一列的与不在最后一列的表数）、
name=table_batch_cells（批号落在表头是「批次」那一格里的单元格数，与落在别的列里的单元格数）、name=table_batch_other_headers（落在别的列里的，各列表头出现次数前 12）。
退出码：0 跑完；2 用法错。批号的认法是这份脚本自己的近似（批 / 批次 / 批号 + 数字，或第 N 批），不是判据库的正则。"""
import argparse
import collections
import pathlib
import re
import sys

BATCH = re.compile(r'(?<![每分A-Za-z0-9])批(?:次号|次|号)?\s*[:：]?\s*#?\s*\d+|(?<![#A-Za-z0-9])第\s*\d+\s*批')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--repo', required=True)
    arguments = parser.parse_args()
    header_position = collections.Counter()
    in_batch_column = in_other_column = 0
    other_headers = collections.Counter()
    for path in sorted(pathlib.Path(arguments.repo, 'records').glob('*.md')):
        header = None
        for line in path.read_text(errors='replace').splitlines():
            stripped = line.strip()
            if not stripped.startswith('|'):
                header = None
                continue
            cells = [cell.strip() for cell in stripped.strip('|').split('|')]
            if header is None:
                header = cells
                if '批次' in cells:
                    header_position['last' if cells[-1] == '批次' else 'not_last'] += 1
                continue
            if re.fullmatch(r'[-: ]+', ''.join(cells)):
                continue
            for index, cell in enumerate(cells):
                if not BATCH.search(cell):
                    continue
                name = header[index] if index < len(header) else '?'
                if name == '批次':
                    in_batch_column += 1
                else:
                    in_other_column += 1
                    other_headers[name[:20]] += 1
    print(f'name=table_header_position last={header_position["last"]} not_last={header_position["not_last"]}')
    print(f'name=table_batch_cells in_batch_column={in_batch_column} in_other_columns={in_other_column}')
    print('name=table_batch_other_headers ' + ' '.join(f'{name}={count}' for name, count in other_headers.most_common(12)))
    return 0


if __name__ == '__main__':
    sys.exit(main())
