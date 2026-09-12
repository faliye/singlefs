#!/usr/bin/env bash
# gate-stage: 引用写着「已定项」，紧跟着却说它没定
#
# 一条分项从未定翻成已定之后，全仓的引用要从「未定项 k」改写成「已定项 k」（22 号阶段逼的）；
# 改完标签，**句子本身**常常还在说它没定：「D19 已定项 6 未定」「那个 key 是 D19 已定项 6、今天没定」。
# 22 号只比标签，28 号只认紧贴「D<n>（简称）」的「未定」，29 号只扫已定项小节里的自指——三个都看不见这一型。
# 实测（2026-09-11，D19 已定项 6 定案）：改完标签之后这样的句子留了六处（三份实验源码的注释、D06、两份实验正文），
# 是逐句手找出来的。
#
# 判据：归属按 22 号的规则（同一份库 `lib-item-ref-status.py`，不另抄）；归属到的那一条是已定、引用处也写着「已定项」，
# 而它后面紧跟着「未定 / 没定 / 定不下 / 待定 / 空着」（中间只许有一段括注、标点与「今天 / 仍 / 还」这类词）⇒ 判红。
# 两份变更史与 records/ 不扫：它们写的是当时的状态，按「今天的编号」改写之后本来就会读成这样。
set -uo pipefail
LIB="$(cd "$(dirname "$0")" && pwd)/lib-item-ref-status.py"
cd "${1:-$(dirname "$0")/../..}" || exit 2
exec python3 - "$LIB" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location('item_ref_status', sys.argv[1])
lib = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib)
item_map, names = lib.load_map()
self_map = lib.self_decisions()
files = [f for f in lib.scanned_files()
         if not f.endswith(('decisions-history.md', 'experiments-history.md')) and '/decisions-history/' not in f
         and not f.startswith('records/')]
bad = []; seen = 0
for path in files:
    for ln, line, m, want, owner, missing, _ in lib.references(path, item_map, self_map):
        if missing or owner is None or want != '已' or item_map[owner][int(m.group(2))] != '已':
            continue
        seen += 1
        if lib.says_open_after(line, m.end()):
            snippet = line[max(0, m.start() - 20):m.end() + 30].strip()
            bad.append(f"{path}:{ln} 「{owner}（{names[owner]}） {m.group(0)}」后面紧跟着说它没定：…{snippet}…")
if bad:
    print(f"  ✗ 引用写着已定项、紧跟着却说它没定 {len(bad)} 处")
    for b in bad[:40]: print("    ", b)
    print("     → 那条分项已经定了：把句子改成它定下来之后的说法（定成了什么、这一处按它重算了没有），别只改标签。")
    sys.exit(1)
print(f"  ✓ 扫了 {len(files)} 个文件、{seen} 处已定项引用，没有一处紧跟着说它没定")
PY
