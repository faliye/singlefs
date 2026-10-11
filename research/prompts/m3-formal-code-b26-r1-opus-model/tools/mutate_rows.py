#!/usr/bin/env python3
"""在冻结副本的拷贝上逐条跑 crates 变异表格式的行（六段：名、文件、原文、替换文、cargo test 参数、必须红的测试名）。

用法：mutate_rows.py <仓副本根> <编译目录> <行文件.tsv> <输出.txt>
每条：原文恰好命中一次才改；经 run-with-memory-cap.sh 4G 与 capped.sh 7 跑 `cargo test --offline <参数>`；
输出里有「test <必须红的名> ... FAILED」或那条测试 panicked 记「红」，测试跑过且 ok 记「绿」，别的记「没判」；跑完还原原文件。
只在副本上改，不碰主仓。"""
import os, subprocess, sys

REPO = '/home/fy5090/code/singlefs'

def unescape(text):
    return text.replace('\\n', '\n').replace('\\t', '\t')

def main():
    root, target, rows_path, out_path = sys.argv[1:5]
    env = dict(os.environ)
    env['CARGO_TARGET_DIR'] = target
    env.setdefault('RUN_WITH_MEMORY_CAP_PARALLEL_REASON', '三方攻方腿与正推腿各跑各的探针')
    env.setdefault('RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON', '变异副本上编 harness 档测试，峰值表记的是别的参数')
    env.setdefault('RUN_WITH_MEMORY_CAP_WAIT_SECONDS', '200')
    with open(rows_path, encoding='utf-8') as handle, open(out_path, 'a', encoding='utf-8') as out:
        for line in handle:
            if not line.strip() or line.startswith('#'):
                continue
            name, path, old, new, args, must_red = line.rstrip('\n').split('\t')
            file_path = os.path.join(root, path)
            original = open(file_path, encoding='utf-8').read()
            hits = original.count(unescape(old))
            if hits != 1:
                out.write(f'MUT {name} => 锚点命中 {hits} 次，没跑\n'); out.flush(); continue
            open(file_path, 'w', encoding='utf-8').write(original.replace(unescape(old), unescape(new)))
            try:
                command = ['nice', '-n', '19', 'bash', f'{REPO}/scripts/hosts/run-with-memory-cap.sh', '4G',
                           'bash', f'{REPO}/scripts/hosts/capped.sh', '7', 'cargo', 'test', '--offline'] + args.split()
                result = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True)
                text = result.stdout + result.stderr
                if f'test {must_red} ... FAILED' in text:
                    verdict = '红'
                elif f'test {must_red} ... ok' in text:
                    verdict = '绿'
                else:
                    verdict = '没判'
                tail = ' | '.join(l for l in text.splitlines() if l.startswith('test result') or 'error[' in l)[:300]
                out.write(f'MUT {name} => {verdict}（退出码 {result.returncode}；{tail}）\n'); out.flush()
            finally:
                open(file_path, 'w', encoding='utf-8').write(original)

if __name__ == '__main__':
    main()
