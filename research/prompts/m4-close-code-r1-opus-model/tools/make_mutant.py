#!/usr/bin/env python3
"""证红用的三个变异体：把改后那一份 core 拷成 <工作目录>/mut-<名>，改一处，再搭一份连它的探针工作区 <工作目录>/probe-mut-<名>。
用法：python3 make_mutant.py <工作目录> <名> <探针源码目录 src> <探针 Cargo.toml 模板>
工作目录里要已有 new/（冻结副本的拷贝）与 old/（HEAD 那一份、已改名）。"""
import shutil
import sys

MUTANTS = {
    # 编解码：全零判法从第二个字节起看（漏掉第一个字节）
    'every-byte-is-zero-skips-the-first-byte': ('crates/singlefs-core/src/bytes.rs',
        '    let mut checked_count: usize = 0;\n    while checked_count < bytes.len()',
        '    let mut checked_count: usize = 1;\n    while checked_count < bytes.len()'),
    # 容器小树：最高那一层装满了才长一层，写成「超过」才长
    'container-tree-grows-only-past-the-fanout': ('crates/singlefs-core/src/container_tree.rs',
        'let grows = stop_nodes[stop_nodes.len() - 1] >= entries_per_node;',
        'let grows = stop_nodes[stop_nodes.len() - 1] > entries_per_node;'),
    # inode 树：左从右借的条数向上取整（差是奇数时多借一条）
    'inode-leaf-borrow-rounds-up': ('crates/singlefs-core/src/inode_tree.rs',
        '            Some(difference) => difference / 2,',
        '            Some(difference) => (difference + 1) / 2,'),
}

work, name, probe_src, manifest_template = sys.argv[1:5]
path, anchor, replacement = MUTANTS[name]
tree = f'{work}/mut-{name}'
shutil.copytree(f'{work}/new', tree, symlinks=True)
text = open(f'{tree}/{path}', encoding='utf-8').read()
if text.count(anchor) != 1:
    sys.exit(f'{name}: 原文在 {path} 里命中 {text.count(anchor)} 次，不是 1 次')
open(f'{tree}/{path}', 'w', encoding='utf-8').write(text.replace(anchor, replacement))
# 格式与 replay-prefix 两个路径依赖指到 new/ 那一份（与改名之后的改前那一份同一份），不然锁文件里同名同版本的包有两个落点
manifest_path = f'{tree}/crates/singlefs-core/Cargo.toml'
core_manifest = open(manifest_path, encoding='utf-8').read()
for old, new in [('{ path = "../singlefs-format" }', '{ path = "../../../new/crates/singlefs-format" }'),
                 ('{ path = "../singlefs-replay-prefix" }', '{ path = "../../../new/crates/singlefs-replay-prefix" }')]:
    if core_manifest.count(old) != 1:
        sys.exit(f'{name}: core 清单里 {old!r} 不是恰好一处')
    core_manifest = core_manifest.replace(old, new)
open(manifest_path, 'w', encoding='utf-8').write(core_manifest)
probe = f'{work}/probe-mut-{name}'
shutil.copytree(probe_src, f'{probe}/src')
manifest = open(manifest_template, encoding='utf-8').read().replace('NEW_TREE', tree).replace('OLD_TREE', f'{work}/old')
open(f'{probe}/Cargo.toml', 'w', encoding='utf-8').write(manifest)
shutil.copy(f'{work}/new/Cargo.lock', f'{probe}/Cargo.lock')
print(f'name=mutant_made mutant={name} file={path}')
