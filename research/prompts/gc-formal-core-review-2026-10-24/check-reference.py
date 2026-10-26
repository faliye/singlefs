"""核对原算法与 cfg(test) 参考体；从已记录的改动前 Git blob 冻结函数体词法单元，之后使用冻结输入。"""
from pathlib import Path
import json
import subprocess
import sys

root = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(root / 'scripts/gate'))
from lib_function_units import rust_lex

relative = 'crates/singlefs-core/src/compaction_plan.rs'

def body(source, name):
    tokens = rust_lex(source)[1]
    starts = [i for i in range(len(tokens) - 1) if tokens[i] == 'fn' and tokens[i + 1] == name]
    assert len(starts) == 1, name
    start = tokens.index('{', starts[0])
    depth = 0
    for i in range(start, len(tokens)):
        depth += (tokens[i] == '{') - (tokens[i] == '}')
        if depth == 0:
            return tokens[start:i + 1]
    raise AssertionError(name)

snapshot = Path(__file__).with_name('original-pure-function-bodies.json')
if not snapshot.exists():
    # 首次扩展的 git diff 记录的旧 blob；暂存区会被其他会话更新，不能当作稳定基线。
    baseline = '808c46915'
    original = subprocess.check_output(['git', 'show', baseline], cwd=root, text=True)
    data = {
        'baseline_blob': subprocess.check_output(['git', 'rev-parse', baseline], cwd=root, text=True).strip(),
        'choose_region': body(original, 'choose_region'),
        'plan_batch': body(original, 'plan_batch'),
    }
    snapshot.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
data = json.loads(snapshot.read_text())
current = (root / relative).read_text()
assert data['choose_region'] == body(current, 'reference_choose_region')
print('CHOOSE_REFERENCE_IDENTICAL: 原函数体与参考体逐 token 一致（忽略注释/空白）')
original_plan = data['plan_batch']
normalized_plan = []
i = 0
while i < len(original_plan):
    if original_plan[i:i + 5] == ['batch', '.', 'units', '(', ')']:
        normalized_plan.append('batch_units')
        i += 5
    else:
        normalized_plan.append(original_plan[i])
        i += 1
assert normalized_plan == body(current, 'plan_of_the_independent_reference')
print('PLAN_REFERENCE_IDENTICAL: 仅将 batch.units() 参数展开为 batch_units，其余函数体逐 token 一致')
print('BASELINE_BLOB:', data['baseline_blob'])
