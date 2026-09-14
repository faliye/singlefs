#!/usr/bin/env bash
# gate-stage: 格式常量文件里的占位（每个占位都指得到一条真实存在的分项或欠账）
#
# 里程碑「第一个事务」步 0 要求：一个格式常量文件，第一个事务要写的每个宽度都在里面，
# 占位的常量带占位标记与分项号；门禁数出常量文件里还有几个占位，每个占位都指得到分项号，指不到判红。
# 占位的写法（crates/singlefs-format/src/*.rs）：常量前一行
#   // placeholder: D22（单元原子性怎么合成） 已定项 2 —— 为什么还是预想
#   // placeholder: C323（镜像大小全仓没有条款） —— 为什么还是预想
# 判据：编号带简称；D<n> 的分项号在 .claude/kb/decisions/<n>-*.md 的「### 已定项 / ### 未定项」两张表里能找到那一行，
# C<n> 在 .claude/kb/checks-owed.md 里有登记行；简称与登记位一致由 doc-lint 管，这里只核编号与分项号。
# 成功那句报出检查了多少个占位（rules/show-me-test.md：扫到 0 项也不是通过——0 个占位要明说）。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -d crates/singlefs-format/src ]] || { echo "  ! 没有 crates/singlefs-format/src，本阶段跳过（步 0 之前没有常量文件）"; exit 0; }

python3 - <<'PY'
import glob, os, re, sys
bad = []
placeholders = []
for path in sorted(glob.glob('crates/singlefs-format/src/**/*.rs', recursive=True)):
    for number, line in enumerate(open(path, encoding='utf-8'), 1):
        m = re.match(r'\s*//\s*placeholder:\s*(.*)$', line)
        if not m:
            continue
        text = m.group(1).strip()
        placeholders.append((path, number, text))
        d = re.match(r'D(\d+)（[^）]+）\s*(已定项|未定项)\s*(\d+)', text)
        c = re.match(r'C(\d+)（[^）]+）', text)
        if d:
            n, kind, k = d.group(1), d.group(2), d.group(3)
            files = glob.glob(f'.claude/kb/decisions/{int(n):02d}-*.md')
            if not files:
                bad.append((path, number, f'D{n} 没有决策文件')); continue
            body = open(files[0], encoding='utf-8').read()
            # 索引行 `| k | **…` 或正文标题 `#### 已定项 k（`，两种形态任一命中即可
            hit = re.search(rf'^\|\s*{k}\s*\|', body, re.M) or re.search(rf'^#{{3,5}}\s*{kind}\s*{k}[（:：]', body, re.M)
            if not hit:
                bad.append((path, number, f'D{n} 里找不到 {kind} {k}'))
        elif c:
            n = c.group(1)
            owed = open('.claude/kb/checks-owed.md', encoding='utf-8').read()
            if not re.search(rf'^\|\s*C{n}\s*\|', owed, re.M):
                bad.append((path, number, f'checks-owed.md 里找不到 C{n} 的登记行'))
        else:
            bad.append((path, number, '占位没写成「D<n>（简称） 已定项/未定项 k」或「C<n>（简称）」'))
if bad:
    print(f'  ✗ 格式常量文件里有 {len(bad)} 个占位指不到分项或欠账：')
    for path, number, why in bad:
        print(f'     {path}:{number}  {why}')  # gate-lint:detail
    print('     → 怎么办：占位那一行写成「// placeholder: D<n>（简称） 已定项 k —— 为什么还是预想」或「// placeholder: C<n>（简称） —— …」，')
    print('       编号要在 .claude/kb/decisions/ 的索引表或 .claude/kb/checks-owed.md 里真的有那一行；分项定了就把占位行删掉。')
    sys.exit(1)
print(f'  ✓ 格式常量文件里的占位都指得到分项或欠账（{len(placeholders)} 个占位：' + '；'.join(t.split(' —— ')[0] for _, _, t in placeholders) + '）')
PY
