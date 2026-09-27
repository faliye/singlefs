#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 全仓不许用角标写法（撇号类字符 U+2032、U+2033、U+2034、U+02B9、U+02BA）给变体起名：K9 的变体起一个新名字，不写成 K9 加一撇
#
# 判据：`git ls-files --cached --others --exclude-standard` 列出的每一份文本文件（不在 git 仓里时走目录树），
# 任何一行出现 U+2032、U+2033、U+2034、U+02B9、U+02BA 之一就判红，逐处列「文件:行」。
# 射程是全仓，冻结证据（research/prompts/、records/、变更史、research/results/）也在内（用户 2026-09-24 定）。
# 没扫的：二进制文件（前 8 KiB 里有 NUL 的）、列出了却不是普通文件的（工作区里删掉还没暂存的），
# 与上游副本 .claude/singlefs-ai-sop/（只能在上游改）；成功行报出扫了几份，前两类逐个列名，上游副本报份数。
# 一份文本文件都没扫到：无对象可判，退 77（不记通过）。
# 管不到的：ASCII 单引号当角标（`S1'`、`丙'`）——它与代码里的引号、带引号的词分不开，机器判不了，靠写的人与 review。
# 怎么起新名字见 .claude/rules/path-moves.md「变体起新名字，不用角标」。
# 字符集在同目录的 lib-prime-marks.py（写成转义，不出现字面），.claude/hooks/write-guard.sh 在写入那一刻拒绝时读同一份，不各抄一份。
# 样本：fixtures/12-no-prime-marks.sh/red 放一份带「K9」加一撇的 md 与一份带「G5」加两撇的 rs，逐处点名判红；green 只有正常写法，判绿。
#
#   bash .claude/gate.d/12-no-prime-marks.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
LIBRARY="$(cd "$(dirname "$0")" && pwd)/lib-prime-marks.py"
cd "$ROOT" 2>/dev/null || exit 2
python3 - "$LIBRARY" <<'PY'
import importlib.util, os, subprocess, sys

try:
    spec = importlib.util.spec_from_file_location('lib_prime_marks', sys.argv[1])
    prime_marks_library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(prime_marks_library)
except OSError as error:
    print(f'  ✗ 读不到共用字符集 {sys.argv[1]}：{error}')
    print('     → 怎么办：撇号类字符只在那一份里定义（.claude/hooks/write-guard.sh 读同一份），从 git 恢复它，别在阶段里再抄一份。')
    sys.exit(1)
PRIME_MARKS = prime_marks_library.PRIME_MARKS
MARKS_SHOWN = prime_marks_library.MARKS_SHOWN
UPSTREAM_COPY = '.claude/singlefs-ai-sop/'

listed = subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'],
                        capture_output=True)
if listed.returncode == 0:
    paths = sorted({name for name in listed.stdout.decode('utf-8', 'replace').split('\0') if name})
else:
    paths = sorted(os.path.relpath(os.path.join(directory, name), '.')
                   for directory, subdirectories, names in os.walk('.')
                   if not subdirectories.__setitem__(slice(None), [s for s in subdirectories if s != '.git'])
                   for name in names)

hits, scanned, binary_paths, upstream, not_regular_paths = [], 0, [], 0, []
for path in paths:
    if path.startswith(UPSTREAM_COPY):
        upstream += 1
        continue
    if not os.path.isfile(path):
        not_regular_paths.append(path)   # 工作区里删掉、还没暂存的
        continue
    with open(path, 'rb') as handle:
        raw = handle.read()
    if b'\0' in raw[:8192]:
        binary_paths.append(path)
        continue
    scanned += 1
    for number, line in enumerate(raw.decode('utf-8', 'replace').split('\n'), 1):
        if any(mark in line for mark in PRIME_MARKS):
            position = min(line.find(mark) for mark in PRIME_MARKS if mark in line)
            hits.append((path, number, line[max(0, position - 20):position + 10].strip()))

if hits:
    files = sorted({path for path, _, _ in hits})
    print(f'  ✗ {len(hits)} 行用了角标写法（{MARKS_SHOWN}），在 {len(files)} 份文件里：')   # gate-lint:summary
    for path, number, snippet in hits[:200]:
        print(f'      {path}:{number}  …{snippet}…')   # gate-lint:detail
    if len(hits) > 200:
        print(f'      （另有 {len(hits) - 200} 行没列出，每份文件的行数见下）')   # gate-lint:detail
        for path in files:
            print(f'      {path}：{sum(1 for hit_path, _, _ in hits if hit_path == path)} 行')   # gate-lint:detail
    print('     → 怎么办：给这个变体起一个新名字——在它那一族里取下一个没用过的号，或起一个短的描述性名字，全仓一次改完；')
    print('               撞号先查（新名字在它那一族的全部文件里零命中）。做法见 .claude/rules/path-moves.md「变体起新名字，不用角标」。')
    sys.exit(1)
if scanned == 0:
    print(f'  ! 本次无对象可判：一份文本文件都没扫到（列出 {len(paths)} 个路径：上游副本 {upstream}、'
          f'二进制 {len(binary_paths)}、不是普通文件 {len(not_regular_paths)}）')
    sys.exit(77)
print(f'  ✓ 查了 {scanned} 份文本文件，没有角标写法（{MARKS_SHOWN}）')
print(f'     没扫的：上游副本 {UPSTREAM_COPY} 下 {upstream} 份（只能在上游改）；'
      f'二进制 {len(binary_paths)} 份、列出了却不是普通文件 {len(not_regular_paths)} 份，逐个列在下面')
for path in binary_paths:
    print(f'       二进制：{path}')
for path in not_regular_paths:
    print(f'       不是普通文件（工作区里删掉还没暂存的）：{path}')
PY
