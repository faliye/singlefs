#!/usr/bin/env python3
"""复核节点引用与 journal 扫描：原版、仅临时引用检查对照、再次原版。"""
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / 'crates/singlefs-checker/Cargo.toml').exists())
WORK = Path(tempfile.mkdtemp(prefix='singlefs-checker-recheck-'))

def run(command, env):
    print('+ ' + ' '.join(map(str, command)), flush=True)
    subprocess.run(list(map(str, command)), cwd=ROOT, env=env, check=True)

def main():
    project = WORK / 'checker'
    (project / 'src').mkdir(parents=True)
    dependencies = ['singlefs-format', 'singlefs-core', 'singlefs-harness']
    manifest = '[package]\nname="singlefs-checker-audit"\nversion="0.0.0"\nedition="2021"\n[dependencies]\n'
    manifest += ''.join(name + '={path="' + str(ROOT / 'crates' / name) + '"}\n' for name in dependencies)
    manifest += 'original_checker={package="singlefs-checker",path="' + str(ROOT / 'crates/singlefs-checker') + '"}\n'
    manifest += '[profile.release]\nopt-level=3\ndebug=true\noverflow-checks=true\ndebug-assertions=true\n'
    (project / 'Cargo.toml').write_text(manifest)
    original_sources = {p.name: p.read_text() for p in (ROOT / 'crates/singlefs-checker/src').glob('*.rs')}
    for name, source in original_sources.items():
        (project / 'src' / name).write_text(source)
    (project / 'src/walk.rs').write_text(original_sources['walk.rs'])
    extra = (HERE / 'node_reference_controls.rs').read_text() + (HERE / 'recovery_image_and_journal_controls.rs').read_text().replace('__COMMON_FIXTURE_PATH__', str(ROOT / 'crates/singlefs-harness/tests/common/mod.rs'))
    source = original_sources['walk.rs']
    # 记录候选集及坏指针是否真的命中了已访问节点，排除“根根本未入选”。
    needle = '    for index in candidate_indexes.iter().copied() {'
    assert source.count(needle) >= 1
    at = source.index(needle, source.index("fn judge_pool_image_given_the_container_capacity_knobs"))
    source = source[:at] + '    eprintln!("AUDIT candidate_roots={:?}",candidate_indexes.iter().map(|i|(roots[*i].2.instance,roots[*i].2.checkpoint_txg)).collect::<Vec<_>>());\n' + source[at:]
    needle = '        if !self.visited_units.insert(placement_it_was_read_from) {'
    start = source.index('    fn read_index_node(')
    at = source.index(needle, start)
    trace = '        if pointer.birth_txg == 999 { eprintln!("AUDIT pointer_birth=999 cache_hit={}",self.visited_units.contains(&placement_it_was_read_from)); }\n'
    source = source[:at] + trace + source[at:]
    # 只用于因果对照；不修改仓库，也不将此简化改动作为正式修复。
    repair = '''        if self.visited_units.contains(&placement_it_was_read_from) {
            self.judge_birth_identity_of_a_referenced_unit(&unit, &pointer, placement_it_was_read_from, what);
            if let Ok(cached_view) = crate::index_node_view(&unit) {
                self.judgements.judge("I-1.3", cached_view.tree_identifier == expected_tree, || "audit recheck control".to_string());
            }
        }
'''
    at = source.index(needle, source.index('    fn read_index_node('))
    patched = source[:at] + repair + source[at:]
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(ROOT / 'target')
    print('临时目录：', WORK, flush=True)
    for label, text, control in [('original-first', source, False), ('reference-recheck-control', patched, True), ('original-second', source, False)]:
        print('\nROUND ' + label, flush=True)
        (project / 'src/walk.rs').write_text(text + extra)
        test_env = env.copy()
        if control: test_env['AUDIT_RECHECK_CONTROL'] = '1'
        else: test_env.pop('AUDIT_RECHECK_CONTROL', None)
        run(['cargo', 'test', '--release', '--offline', '--manifest-path', project / 'Cargo.toml', '--lib', 'audit_', '--', '--nocapture', '--test-threads=1'], test_env)
    for name, text in original_sources.items():
        assert (ROOT / 'crates/singlefs-checker/src' / name).read_text() == text, '审计期间源码发生变化：' + name
    print('CHECKER_RECHECK_COMPLETE original/control/original 各轮通过；产品源码未变。')

if __name__ == '__main__':
    main()
