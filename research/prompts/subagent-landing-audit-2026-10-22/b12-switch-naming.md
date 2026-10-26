# 第十二批任务 42：弄坏开关点名逐格相同、doc-registries 与比对对象表的判法（接续）

日期：2026-10-23。批次 `records/2026-10-22-子agent结果落地审计.md#12`。前一个同类 agent 中途停下，这一份接着它做完；它已换上主工作区的几处先核了现场（自证与样本重跑），没重做。

## 结论

| 条 | 结论 | 推翻条件 |
|---|---|---|
| 1 ㉖(a) | 门禁 doc-registries 读登记位的六格共用 `kb_roots_or_red`，另四格（audit-contradictions、declared-counts、invariant-anchors、invariant-prerequisites）读不出登记位判红；四份红样本经 `stage-selftest.sh --item doc-registries.sh` 判得对（55 个用例全对），开关 `DOC_REGISTRIES_BREAK=kb-roots-failure-ignored` 下 5 个用例判错，正好是点名的那几份 | 主 agent 暂存后照「要主 agent 跑的命令」重跑，有一份新样本判错 |
| 2 ㉖(e)、㉟ | doc-experiments 的 replay-comparison-units 格：每行「不登记的比对对象」要写「欠在 C<号>（简称）」，号在 `.claude/kb/checks-owed.md` 开着；没写、号不在、号已还清、欠账表不在都判红，出路写「先在 `.claude/kb/checks-owed.md` 登记欠账号，同一行写上」。红样本 exclusion-owed-red、绿样本 table-green 判得对，开关下各自判错。**真仓此刻会红**：e185-r3 三行没写欠账号，另有 e161-r3 比对对象表第 1 行没登记（见「交给别的会话的」） | 真仓那四处补上之后这一格仍红 |
| 3 ㉖ | `lib-lag-baseline.py` 文件头调用方表照 `grep` 现查补全（补了 decision-links 的 baseline_text、lib_long_run 的 effective_moves、E171 两份 frozen_o3 都调 rewrite_moved_bytes、`scripts/agents/snapshot-tree.py`）；NOT_RUN_HERE 那一行改成「调它的三个脚本（replace-once.py、insert-row.py、replace-batch.py）」；`replace-batch.py` 接上 `lib_notation_check.py` 的钟点判法，红样本退 6、绿样本退 0、`REPLACE_BATCH_BREAK=no-clock-guard` 下红样本退 0 | 另有调用方 `grep` 得到而表里没有 |
| 4 ㉟ | break-switch-replay 格改成逐格相同（先跑一趟不设开关的基准趟，开关趟的失败行减去基准趟的；少红、多红各自点名；汇总行不要求点名）。四份样本在格的原样 python 上逐个判对，七条格开关各红在点名的样本。全仓复跑 1257 条：登记的 24 份里 22 份逐格相同，两份兄弟任务的不相同（completion-cleanup-guard 11 条、lib_prompt_scratch_line 4 条），**换上之后这一格在真仓会红在这两份上**。没登记的那些，能只改开关表补齐的都补了 | 兄弟任务改完那两份之后这一格仍红在别的登记份上 |
| 5 ㉟ | 造了两份探针确认改前判法：单独的 `\r` 留在行里，「✗ 自检：甲\r  乙在下一行」整行当一行，点名「甲…乙在下一行」判对（按文本收会拆成两行，判法与按文本收不一致）；坏字节一出就只记坏字节、点名不比。改成：`\r` 与 `\r\n` 都当换行；坏字节照 U+FFFD 换掉接着比点名，另记一条坏字节红。两条改前次序各有开关（replay-carriage-return-kept、replay-bad-bytes-first）与样本 | 有输出按文本收与按字节收认出的行不一样 |

## 第 1 条的证据（门禁 doc-registries）

改法：`.claude/gate.d/doc-registries.sh` 新函数 `kb_roots_or_red <格名> <当成空的会被误认成什么> <候选目录写到哪>`，六格开头都调它；弄坏开关 `DOC_REGISTRIES_BREAK=kb-roots-failure-ignored` 退回不看退出码。四份新红样本：`audit-contradictions-kb-directories-unreadable-red`、`declared-counts-kb-directories-unreadable-red`、`invariant-anchors-kb-directories-unreadable-red`、`invariant-prerequisites-kb-directories-unreadable-red`（各自 `setup.sh` 往 `.claude/kb-directories` 写一个坏字节）。绿样本沿用原有各格的 green。

命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-registries.sh 的样本档; 范围=1 道; 为什么=核这一批新加的登记位读不出红样本' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-registries.sh`，退 0。新样本那几行与末行原样：

```
  ✓ doc-registries               audit-contradictions-kb-directories-unreadable-red 判得对
  ✓ doc-registries               declared-counts-kb-directories-unreadable-red 判得对
  ✓ doc-registries               invariant-anchors-kb-directories-unreadable-red 判得对
  ✓ doc-registries               invariant-prerequisites-kb-directories-unreadable-red 判得对
  ✓ doc-registries               kb-directories-unreadable-red 判得对
  ✓ doc-registries               references-unreadable-red 判得对
  ✓ 点名的 1 道阶段判得都对（55 个用例）：doc-registries；没点名的这一趟不喂
```

同一条命令前面加 `DOC_REGISTRIES_BREAK=kb-roots-failure-ignored`，退 1，判错的行原样：

```
  ✗ doc-registries               audit-contradictions-kb-directories-unreadable-red 输出里找不到「audit-contradictions 没判：kb 目录登记位读不出」
  ✗ doc-registries               declared-counts-kb-directories-unreadable-red 输出里找不到「declared-counts 没判：kb 目录登记位读不出」
  ✗ doc-registries               invariant-anchors-kb-directories-unreadable-red 期望退出 1，实测 77
  ✗ doc-registries               invariant-anchors-kb-directories-unreadable-red 输出里找不到「invariant-anchors 没判：kb 目录登记位读不出」
  ✗ doc-registries               invariant-anchors-kb-directories-unreadable-red 输出里找不到「  invariant-anchors：红（退出 1）」
  ✗ doc-registries               invariant-anchors-kb-directories-unreadable-red 输出里找不到「doc-registries.sh：1 格里 1 格红（invariant-anchors）」
  ✗ doc-registries               invariant-prerequisites-kb-directories-unreadable-red 期望退出 1，实测 77
  ✗ doc-registries               invariant-prerequisites-kb-directories-unreadable-red 输出里找不到「invariant-prerequisites 没判：kb 目录登记位读不出」
  ✗ doc-registries               invariant-prerequisites-kb-directories-unreadable-red 输出里找不到「  invariant-prerequisites：红（退出 1）」
  ✗ doc-registries               invariant-prerequisites-kb-directories-unreadable-red 输出里找不到「doc-registries.sh：1 格里 1 格红（invariant-prerequisites）」
  ✗ doc-registries               kb-directories-unreadable-red 期望退出 1，实测 77
  ✗ doc-registries               kb-directories-unreadable-red 输出里找不到「invariant-count-elsewhere 没判：kb 目录登记位读不出」
  ✗ doc-registries               kb-directories-unreadable-red 输出里找不到「inline-sums 没判：kb 目录登记位读不出」
  ✗ doc-registries               kb-directories-unreadable-red 输出里找不到「  invariant-count-elsewhere：红（退出 1）」
  ✗ doc-registries               kb-directories-unreadable-red 输出里找不到「  inline-sums：红（退出 1）」
  ✗ doc-registries               kb-directories-unreadable-red 输出里找不到「doc-registries.sh：2 格里 2 格红」
  ✗ 5 个用例判错（判对 50 个，0 个阶段没有用例）
```

## 第 2 条的证据（门禁 doc-experiments 的 replay-comparison-units 格）

改法：格里新判 `exclusion_owed_problem`（读欠账表经 `.claude/gate.d/lib-owed.py`，与读欠账表的各格同一份）；弄坏开关 `EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-exclusion-owed-unchecked`。登记表 `.claude/gate.d/replay-comparison-units.tsv` 文件头写明这一条；九十节 ㉖ 原来的「点名行与基准比、只缩不涨」不做（格注释写明由这一判替掉）。
样本：红 `replay-comparison-units-exclusion-owed-red`（四行点名不登记：没写、号不在、号已还清各一行判红，欠在开着的 C92 那一行不报），绿 `replay-comparison-units-table-green`（点名行写了开着的欠账号）。

这几份样本还没暂存进 stage-selftest 认的那一版（doc-experiments.sh 的样本里别的会话另有没暂存的 decision-links-switch-inserted-green 等，闸拒），这里是把格的那段 python 原样取出（与主工作区 `doc-experiments.sh` 里那段逐字节相同，`diff` 无输出）在样本拷贝上跑，比 expect 的退出码与 want：

```
replay-comparison-units-exclusion-owed-red （不设开关）：判得对
replay-comparison-units-exclusion-owed-red comparison-exclusion-owed-unchecked：判错
replay-comparison-units-table-green （不设开关）：判得对
replay-comparison-units-table-green comparison-exclusion-owed-unchecked：判错
replay-comparison-units-table-red （不设开关）：判得对
replay-comparison-units-table-red comparison-exclusion-owed-unchecked：判得对
```

（开关下 exclusion-owed-red 判错是「没判出红」，table-green 判错是成功句里少了「（点名行都欠在开着的欠账上）」；table-red 不受这条开关影响。）

红样本那一趟的输出原样（前 5 行）：

```
  ✗ .claude/gate.d/replay-comparison-units.tsv 里 3 处对不上：
     replay-comparison-units.tsv 第 3 行：「不登记的比对对象」没写欠在哪一条账（理由里要写「欠在 C<号>（简称）」）：# 不登记的比对对象：research/prompts/e991-r2-prereg.md #2：样本里这条正式跑在单元表里没有单元
     replay-comparison-units.tsv 第 4 行：写的 C998 在 .claude/kb/checks-owed.md 里没有登记：# 不登记的比对对象：research/prompts/e991-r2-prereg.md #3：样本里这条正式跑在单元表里没有单元；欠在 C998（欠账表里没有的号）
     replay-comparison-units.tsv 第 5 行：写的 C991（样本已还）在 .claude/kb/checks-owed.md 里已还清：还清了就该补上单元、改登记成正式行，不再点名不登记：# 不登记的比对对象：research/prompts/e991-r2-prereg.md #4：样本里这条正式跑在单元表里没有单元；欠在 C991（样本已还）
     → 怎么办：列义在 .claude/gate.d/replay-comparison-units.tsv 文件头。登记出处写仓内现存路径；单元名照单元表里的字符串字面量逐字抄，要在单元表出处里恰好出现一次；
```

真仓此刻（同一段 python，在主工作区仓根读真表，基准传空）：退 1，红行原样（截到 200 字）：

```
  ✗ .claude/gate.d/replay-comparison-units.tsv 里 4 处对不上：
     replay-comparison-units.tsv 第 27 行：「不登记的比对对象」没写欠在哪一条账（理由里要写「欠在 C<号>（简称）」）：# 不登记的比对对象：research/prompts
     replay-comparison-units.tsv 第 28 行：「不登记的比对对象」没写欠在哪一条账（理由里要写「欠在 C<号>（简称）」）：# 不登记的比对对象：research/prompts
     replay-comparison-units.tsv 第 29 行：「不登记的比对对象」没写欠在哪一条账（理由里要写「欠在 C<号>（简称）」）：# 不登记的比对对象：research/prompts
     research/prompts/e161-r3-prereg.md 比对对象表 #1（比对对象 e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out，重跑产物名 e161-crash-state-dedup-and-time-split-fe
```

## 第 3 条的证据

- `lib-lag-baseline.py` 文件头「调它的各处」：对着 `grep -rln lib-lag-baseline .claude/gate.d .claude/hooks scripts`（fixtures 除外）逐份核用到的函数，补了四处（结论表第 3 行）。只改文件头，`python3 .claude/gate.d/lib-lag-baseline.py --selftest` 末行：

```
  ✓ lib-lag-baseline.py 自证：32 项都过
```

- `code-tooling.sh` NOT_RUN_HERE 那一行现在是：

```
835:  "scripts/kb/lib_notation_check.py	它自己没有自证：只被 import，调它的三个脚本（replace-once.py、insert-row.py、replace-batch.py）各自的 --selftest 走一遍拒绝与放行"
```

- `scripts/kb/replace-batch.py`：`run()` 在锚点核完之后、判会话号之前调 `clock_refusals`（共用 `lib_notation_check.py` 的 `clock_refusal`，与 replace-once.py、insert-row.py 同一份），退出码 6；弄坏开关 `REPLACE_BATCH_BREAK=no-clock-guard`；自证新加「写 .md 之前判新串里的钟点」一组。`lib_notation_check.py` 只改文档串（三份都判钟点）。

红样本 `[{"file": "scripts/kb/README.md", "old": "# scripts/kb", "new": "# scripts/kb 运行目录 <紧凑时间戳>"}]`（时间戳在规格文件 `specs/clock-red-sample.json` 里，这里不抄），绿样本新串写「（2026-10-23）」，都 `--dry-run`（README.md 的 sha256 前后相同）：

```
红：rc=6，末行 ✗ 1 处里 1 处的新串在描述里写了钟点、时区词或时间戳，--dry-run 没写
绿：rc=0，✓ 1 处都命中得对（1 个文件），--dry-run 没写
红 + REPLACE_BATCH_BREAK=no-clock-guard：rc=0，✓ 1 处都命中得对（1 个文件），--dry-run 没写
```

自证：不设开关 rc=0；`REPLACE_BATCH_BREAK=no-clock-guard` rc=1 红在「✗ 自证：写 .md 之前判新串里的钟点没判对」；`LIB_FORBIDDEN_NOTATIONS_BREAK=clock-times` rc=1 红在同一格。replace-once.py、insert-row.py 的 `--selftest` 也各跑一次，都退 0。

## 第 4 条的证据（门禁 code-tooling 的 break-switch-replay 格）

判法写在 `code-tooling.sh` 里这一格函数上方的注释（「先不设开关起一趟……逐格相同」那一段），要点：
- 基准趟（不设开关）要退 0，它的失败行是自证本来就有的；开关趟的失败行减去基准趟的（去掉临时路径与数字之后整行比、按次数抵）。
- 点名的项都要在剩下的行里（少红判「点名的项没在失败行里」）；剩下的行里以 ✗ 起头的每一行都要被某一项照出来（多红判「失败行没被点名」，逐行列）。行中间才带 ✗ 的只拿来照点名，不要求点名。
- 汇总行：同一份脚本里红了的开关有一半以上（至少两条）都多红的同一行，不要求点名。
- 管不到的（注释里写了）：失败行不带 ✗ 的自证；汇总行是按次数推的。

新弄坏开关（`CODE_TOOLING_BREAK=`）：replay-unnamed-rows-unchecked、replay-baseline-unsubtracted、replay-summary-rows-required、replay-carriage-return-kept、replay-bad-bytes-first；原有 replay-items-unchecked、replay-output-strict-decode 的点名照新样本改了。
样本：红 break-switch-replay-red、break-switch-replay-unnamed-red（新）、break-switch-replay-undecodable-red（加了 garbled-mismatch.py），绿 break-switch-replay-green（加了 noisy.py、chatty.py）。

code-tooling.sh 的新样本还没暂存（stage-selftest 闸拒），这里把格的两段 python（开关表库 `PY_BREAK_SWITCH_TABLE` 与格本身）原样取出（与主工作区那份逐字节相同，`diff -q` 无输出），在样本拷贝上逐个跑，比 expect 的退出码与 want。4 份样本 × （不设开关 + 7 条开关）共 32 趟，收回 32 个退出码文件：

```
[new] break-switch-replay-red （不设开关）：判得对
[new] break-switch-replay-red replay-items-unchecked：判错
[new] break-switch-replay-red replay-unnamed-rows-unchecked：判得对
[new] break-switch-replay-red replay-baseline-unsubtracted：判得对
[new] break-switch-replay-red replay-summary-rows-required：判得对
[new] break-switch-replay-red replay-carriage-return-kept：判得对
[new] break-switch-replay-red replay-bad-bytes-first：判得对
[new] break-switch-replay-red replay-output-strict-decode：判得对
[new] break-switch-replay-unnamed-red （不设开关）：判得对
[new] break-switch-replay-unnamed-red replay-items-unchecked：判得对
[new] break-switch-replay-unnamed-red replay-unnamed-rows-unchecked：判错
[new] break-switch-replay-unnamed-red replay-baseline-unsubtracted：判得对
[new] break-switch-replay-unnamed-red replay-summary-rows-required：判得对
[new] break-switch-replay-unnamed-red replay-carriage-return-kept：判错
[new] break-switch-replay-unnamed-red replay-bad-bytes-first：判得对
[new] break-switch-replay-unnamed-red replay-output-strict-decode：判得对
[new] break-switch-replay-undecodable-red （不设开关）：判得对
[new] break-switch-replay-undecodable-red replay-items-unchecked：判错
[new] break-switch-replay-undecodable-red replay-unnamed-rows-unchecked：判得对
[new] break-switch-replay-undecodable-red replay-baseline-unsubtracted：判得对
[new] break-switch-replay-undecodable-red replay-summary-rows-required：判得对
[new] break-switch-replay-undecodable-red replay-carriage-return-kept：判得对
[new] break-switch-replay-undecodable-red replay-bad-bytes-first：判错
[new] break-switch-replay-undecodable-red replay-output-strict-decode：判错
[new] break-switch-replay-green （不设开关）：判得对
[new] break-switch-replay-green replay-items-unchecked：判得对
[new] break-switch-replay-green replay-unnamed-rows-unchecked：判得对
[new] break-switch-replay-green replay-baseline-unsubtracted：判错
[new] break-switch-replay-green replay-summary-rows-required：判错
[new] break-switch-replay-green replay-carriage-return-kept：判得对
[new] break-switch-replay-green replay-bad-bytes-first：判得对
[new] break-switch-replay-green replay-output-strict-decode：判得对
```

判错的 8 趟正是文件头每条开关点名的样本（不设开关 4 趟都判得对）。

## 第 5 条的证据（\r 与坏字节）

探针根 `probe-decode/`（草稿目录里）：`probe-cr.py` 在 `CR_BREAK=split` 下打「  ✗ 自检：甲\r  乙在下一行」、点名「✗ 自检：甲…乙在下一行」；`probe-guard-bytes.sh` 在外面设开关时退 2、拒绝说明末尾带一个 0xff 字节。改前与改后的格 python 各跑一趟（输出在 `verify/probe-old.txt`、`verify/probe-new.txt`，下面各省掉了末尾「→ 怎么办」那一行）：

```
=== 改前
  ✗ scripts/gate/probe-guard-bytes.sh GUARD_BYTES_BREAK=skip：坏字节，stderr 里有不合法的 UTF-8（第 82 字节起：ff 0a），点名的项比不了；改那份脚本让它在这条开关下只吐 UTF-8
  ✗ 脚本与钩子的弄坏开关复跑：1 处不对（复跑了 2 条、2 份）
exit=1
=== 改后
  ✗ scripts/gate/probe-cr.py CR_BREAK=split：不符，点名的项没在失败行里：「✗ 自检：甲…乙在下一行」；失败行没被点名：「✗ 自检：甲」
  ✗ scripts/gate/probe-guard-bytes.sh GUARD_BYTES_BREAK=skip：坏字节，stderr 里有不合法的 UTF-8（第 82 字节起：ff 0a），改那份脚本让它在这条开关下只吐 UTF-8
  ✗ 脚本与钩子的弄坏开关复跑：2 处不对（复跑了 2 条、2 份）
exit=1
```

改前：`\r` 那一条判「对」（两行被当成一行，点名跨行也照得到），坏字节那一条在比守卫之前就停在坏字节。改后两条都红，且坏字节之外照比点名。改前两种次序由 replay-carriage-return-kept、replay-bad-bytes-first 两条开关保留，样本 break-switch-replay-unnamed-red（carriage.py）与 break-switch-replay-undecodable-red（garbled-mismatch.py）证红。

## 全仓复跑：怎么跑的、补了哪些

范围：`scripts/` 与 `.claude/hooks/` 下读弄坏开关、文件头开关表写全、自证登记成 light 的每一份（登记进 break-switch-replay-files.tsv 的 24 份与没登记的，共 66 份 1257 条开关）。工具在草稿目录 `tools/replay.py`（判法与格同一份：开关表库从 code-tooling.sh 现取，基准趟、按次数抵、汇总行同一算法），续跑表是 `runs/<脚本 sha256 前 16 位>-<开关>.json`，脚本没变的沿用；判「不同」的另起一趟（`tools/recheck.py`，记成 `…#recheck.json`），两趟都多红 / 都少红的才算稳的、才去改头注。本机跑，`capped.sh 31`、`nice -n 19`，并行 16 到 28。

补法（`tools/fixnames.py` 出规格、逐份看过再换上；`scripts/` 下经 `hook-swap.py` 脚本规格、`--verify` 跑那份的 `--selftest`，钩子 `lib_heavy_tests.py` 经 `hook-swap.py --hook .claude/hooks/heavy-test-guard.sh`）：
- 稳的多红行：那一条开关那一块末尾加一行「连带红的格：「…」、…」，项照失败行截到「应当 / 实际」之前、临时路径与长数字换成 …；
- 稳的少红项（点名的是别的脚本的格、写着「不会走到」「被挡在前面」的格、或写的是实测值不是失败行）：外层「」换成『』，原话留着、不再当点名的项；
- 少红项与多红行一一对得上、只是写法旧了的，就地换成实测写法（prove-red.sh 四处、e165 的 zfirstfour、insert-row.py 的 no-clock-guard、replay.sh 的 inputs-not-checked）；
- `gpu-quiesce-run.sh` 的自证自己按「格名（「…」）」读开关表、核红在点名的格，加行会让它的开关轮红，改成在原括号里就地写「✗ 自证 <格名>：…」。

这一趟换上的（逐份 `--selftest` 退 0、换上后复跑全部开关）：`.claude/hooks/lib_heavy_tests.py`（44 条）、`scripts/kb/history-consolidate.py`、`scripts/kb/insert-row.py`、`scripts/kb/quote-kb.py`、`scripts/kb/archive-past-rounds.py`、`scripts/mutation/prove-red.sh`、`scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh`、`scripts/experiment-apparatus/replay.sh`、`scripts/gate/merge-and-verify.sh`（含 `WORKSPACE_LOCK_BREAK=trust-dead-holder` 那一处，前一个 agent 补的，这一趟复核相同）、`scripts/hosts/detached-run.sh`、`scripts/crash/gpu-quiesce-run.sh`、`scripts/hosts/multi-host-run.sh`、`.claude/gate.d/break-switch-replay-files.tsv`（文件头那条加行规矩改成逐格相同）。前一个 agent 换上、这一趟复核相同的：`scripts/agents/agent-cost.py`、`cite-check.py`、`three-way-rerun-template.sh`、`scripts/gate/apply-writer-patch.py`、`scripts/kb/batch-target.py`、`kb-spec-check.py`、`stale-candidates.py`、`sweep-term.py`、`scripts/kb/replace-batch.py`。

### 还不相同的，与为什么没改

| 份 | 条数 | 为什么 | 归谁 |
|---|---|---|---|
| `.claude/hooks/completion-cleanup-guard.sh`（登记） | 11 不相同 | 兄弟任务的文件：都是多红（开关弄红的格比点名的多，逐格清单在 `verify/show-sib.txt`）。换上新判法之后 break-switch-replay 格会红在这里 | 兄弟任务 |
| `.claude/hooks/lib_prompt_scratch_line.py`（登记） | 4 不相同 | 同上：label-anywhere 多红 2、code-blocks-read 多红 8、roots-unfiltered 多红 1、unclosed-fence-read 多红 3 | 兄弟任务 |
| `.claude/hooks/runner-dispatch-guard.sh`、`scripts/agents/dispatch-prompt.py`、`scripts/agents/snapshot-tree.py` | 128、20、9 | 都相同（snapshot-tree 前一份是 3 条不相同，兄弟任务改过之后相同） | — |
| `scripts/crash/check-segment-registry.py`、`scripts/gate/gate-structure-check.py`、`scripts/gate/stage-mine.py`、`scripts/gate/changed-paths.sh`、`scripts/kb/kb-sections.py`、`scripts/kb/replace-once.py` | 2、4、3、7、1、1 | 自证的失败细节不以 ✗ 起头（「bad：…」「selftest: …」「实测「…」」或缩进的细节行），带 ✗ 的只有一句汇总；只改开关表补不齐，要改自证的打印。没登记，格不判它们 | 各份的主人 |
| `scripts/kb/migrate-changelog-format.py` | 10 | 开关趟退 78：admission 是 inputs-changed，基准趟跑成功记了指纹，同一份输入的开关趟被 preflight 拒。接不进复跑 | 那份的主人 |
| `scripts/hosts/multi-host-run.sh` | 18 | 两样：开关趟输出里有不合法的 UTF-8（坏字节，脚本那一侧要修）；「Ⓝ 第二台起登记数那么多条线程忙等」一格在并行复跑下时红时不红（核数量出来低于阈值，两趟不一样），不点名。稳的多红已补 | 那份的主人（另有会话在改） |
| `scripts/hosts/detached-run.sh` | 不相同 1、超时 1、没红 1、没点名 39 | start-failed-trusted 多红的「核数②」两趟不一样（负载下时红时不红），没点名；exit-dropped 两次都过 200 秒；watch-skips-unknown 开关下自证退 0；39 条开关表的点名不带「」，比不了 | 那份的主人 |
| `scripts/hosts/resource_release_broadcast.py` | 13 | 别的会话新建、还没进 git 的文件，这一趟里还在改；多红的是「✗ … 自证 N 格不对（查了 13 格）：<格名>」这类汇总句，每条开关点的格名不同，汇总行认不出 | 那份的主人 |
| `scripts/experiment-apparatus/replay.sh` 4 条、`scripts/gate/changed-paths.sh` 1 条与 write-guard.sh、cpu-judging-sites-check.py、crash-amplification-lib.sh、crash-flow-coverage.py、gpu-exemptions-check.py、e197 两份、milestone-close-owed-check.py、crates-mutation-light-rows.py | 没点名 | 点名的项不带「」，格比不了（break-switch-headers 格另判写全没有） | — |
| `scripts/gate/verus-proof-check.py` | 64 没跑 | 基准趟就红：自证 123 格里 24 格判错（reuse-* 那一组与 host-lookup-*，「包装里找不到 wait_thread」），疑与别的会话此刻在改的 `scripts/hosts/run-with-memory-cap.sh` 有关；开关趟不起 | 改 run-with-memory-cap.sh 的会话 |
| `scripts/agents/agent-watch.py`（含 memoryscan）、`scripts/crash/crash-amplification.sh`、`scripts/crash/crash-point-replay.sh`、`scripts/mutation/broad-mutation.py`、`scripts/mutation/mutate.sh`、`scripts/mutation/mutation-shard-run.sh`、`scripts/hosts/run-with-memory-cap.sh` | 119+1、50、41、77、33、16、70 | 单趟自证接近或超过前台时限（基准趟：agent-watch 191 秒、mutate 202 秒、run-with-memory-cap 199 秒；crash-point-replay、broad-mutation、mutation-shard-run、crash-amplification 225 秒内没跑完），memoryscan 那一条 225 秒超时。子 agent 不起后台，交主 agent（命令在下面） | 主 agent |

## 要主 agent 跑的命令

工作目录都是主工作区仓根 `/home/fy5090/code/singlefs`；内存上限派发没给，下面这几样只起 light 自证与样本，不起 cargo。

1. 暂存这一批新样本之后跑两道样本档（code-tooling.sh 的 break-switch-replay-unnamed-red 与 green 里的 noisy.py、chatty.py、undecodable-red 里的 garbled-mismatch.py 还没进 git；doc-experiments.sh 的样本里另有别的会话没暂存的 decision-links-switch-inserted-green 等，闸照拒，要一并暂存或等它们）：
   ```
   SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的样本档; 范围=1 道; 为什么=break-switch-replay 逐格相同的新样本' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item code-tooling.sh
   SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的样本档; 范围=1 道; 为什么=比对对象表点名不登记要指到欠账号' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-experiments.sh
   ```
   看末行「点名的 1 道阶段判得都对（N 个用例）」。各开关逐条证红归门禁 code-tooling 的 stage-break-switch-replay 格（提交前那一趟）。
2. 慢的那几份的开关复跑（单趟接近或超过 200 秒，前台跑不完；推的：每份不到 2 小时，可分开 run_in_background）。`run` 同一条起两遍：头一遍只跑还没记的基准趟、退 3，第二遍跑开关趟、全记完退 0；再 judge：
   ```
   D=~/.local/share/singlefs/work/landing-audit-b12-switch-naming
   REPLAY_EXCLUDE=none REPLAY_SLOW_LIMIT=100000 REPLAY_SECONDS=900 bash scripts/hosts/capped.sh 31 nice -n 19 python3 -B $D/tools/replay.py run --deadline 100000 --jobs 8 --only scripts/agents/agent-watch.py
   （--only 依次换成 scripts/crash/crash-amplification.sh、scripts/crash/crash-point-replay.sh、scripts/mutation/broad-mutation.py、scripts/mutation/mutate.sh、scripts/mutation/mutation-shard-run.sh、scripts/hosts/run-with-memory-cap.sh）
   REPLAY_EXCLUDE=none python3 -B $D/tools/replay.py judge --only <同一份> --json $D/verify/judge-slow.jsonl
   ```
   看 judge 打的「条数 N；对 …、不同 …」；不同的逐条在 jsonl 的 missing / unnamed 里，补法照上面「补法」一节，或派同类 agent 接着补。crash-point-replay、broad-mutation、mutation-shard-run、crash-amplification 的基准趟在负载 45 到 75 时 225 秒没跑完，并行度按当时负载再降。
3. 命名检查不用跑：这一批没改 `.rs`。

## 交给别的会话的

- 门禁 doc-experiments 的 replay-comparison-units 格在真仓会红（新判法加旧缺口）：`.claude/gate.d/replay-comparison-units.tsv` 第 27 至 29 行（e185-r3 #1 至 #3，别的会话加的）没写「欠在 C<号>（简称）」，要先在 `.claude/kb/checks-owed.md` 登记欠账号再写上；`research/prompts/e161-r3-prereg.md` 比对对象表 #1 在表里没有一行对得上、文件头也没点名（与这一批无关的旧缺口）。kb 不归 tooling-writer 写。
- 门禁 code-tooling 的 break-switch-replay 格换上新判法之后会红在两份登记了的兄弟任务文件上：completion-cleanup-guard.sh 11 条、lib_prompt_scratch_line.py 4 条，都是多红（逐条在 `verify/show-sib.txt`）。
- verus-proof-check.py 自证基准趟红 24 格（见上表），疑与 run-with-memory-cap.sh 正在改有关。
- multi-host-run.sh 开关趟吐坏字节；resource_release_broadcast.py 的汇总句带格名、判法认不出汇总行。

## 新写的 gate-similar / hook-events

没有：这一批没新建门禁阶段与钩子，只改已有的格与文件头。

## 改过的文件

这一批（前一个 agent 加这一趟）写过的仓内文件：门禁 `.claude/gate.d/code-tooling.sh`（break-switch-replay 格、文件头开关表与样本说明、NOT_RUN_HERE 那一行）、`.claude/gate.d/doc-registries.sh`、`.claude/gate.d/doc-experiments.sh`（replay-comparison-units 格）、`.claude/gate.d/replay-comparison-units.tsv`（只改文件头说明）、`.claude/gate.d/lib-lag-baseline.py`（只改文件头）、`.claude/gate.d/break-switch-replay-files.tsv`（只改文件头一行）；`scripts/kb/replace-batch.py`、`scripts/kb/lib_notation_check.py`（文档串）；开关表那几行：上面「这一趟换上的」与「前一个 agent 换上」两串文件；样本：`.claude/gate.d/fixtures/doc-registries.sh/` 下四份 `*-kb-directories-unreadable-red`，`fixtures/doc-experiments.sh/replay-comparison-units-exclusion-owed-red`（新）与 table-green、table-red 的 `checks-owed.md`，`fixtures/code-tooling.sh/break-switch-replay-unnamed-red`（新）与 green、undecodable-red 里加的脚本与改的 expect、登记表。

这几份里有别的会话没提交的改动（code-tooling.sh 的 change-range-single-source 那一段、doc-experiments.sh 的 decision-links 语言切换行、lib-lag-baseline.py 的搬迁那一层、detached-run.sh、multi-host-run.sh、gpu-quiesce-run.sh 的大段），照现状定点改，没回退；下面的行数含它们。

`git diff --stat -- <这些文件>`（工作区对暂存区；doc-registries.sh 与它的样本主 agent 已暂存，不在这张里）原样：

```
 .claude/gate.d/break-switch-replay-files.tsv       |   3 +-
 .claude/gate.d/code-tooling.sh                     | 219 +++++++++++++++------
 .claude/gate.d/doc-experiments.sh                  |  28 ++-
 .../.claude/gate.d/break-switch-replay-files.tsv   |   2 +
 .../.claude/hooks/selftest-registry.tsv            |   2 +
 .../break-switch-replay-green/expect               |   2 +-
 .../.claude/gate.d/break-switch-replay-files.tsv   |   1 +
 .../.claude/hooks/selftest-registry.tsv            |   1 +
 .../break-switch-replay-undecodable-red/expect     |   4 +-
 .claude/gate.d/lib-lag-baseline.py                 |  40 ++--
 .claude/hooks/lib_heavy_tests.py                   | 177 +++++++++--------
 scripts/agents/agent-cost.py                       |   3 +
 scripts/agents/cite-check.py                       |   1 +
 scripts/agents/three-way-rerun-template.sh         |   1 +
 scripts/crash/gpu-quiesce-run.sh                   |  73 ++++++-
 .../e165-replay-mutation-kill-bounds.sh            |  18 +-
 scripts/experiment-apparatus/replay.sh             |  19 +-
 scripts/gate/apply-writer-patch.py                 |   2 +
 scripts/gate/merge-and-verify.sh                   |  18 +-
 scripts/hosts/detached-run.sh                      | 101 +++++++++-
 scripts/hosts/multi-host-run.sh                    |  72 ++++++-
 scripts/kb/archive-past-rounds.py                  |  10 +-
 scripts/kb/batch-target.py                         |   2 +
 scripts/kb/history-consolidate.py                  |   2 +-
 scripts/kb/insert-row.py                           |   5 +-
 scripts/kb/kb-spec-check.py                        |   6 +
 scripts/kb/lib_notation_check.py                   |   4 +-
 scripts/kb/quote-kb.py                             |   7 +-
 scripts/kb/replace-batch.py                        | 140 ++++++++++++-
 scripts/kb/stale-candidates.py                     |   1 +
 scripts/kb/sweep-term.py                           |   1 +
 scripts/mutation/prove-red.sh                      |   9 +-
 32 files changed, 765 insertions(+), 209 deletions(-)
```

改动前的原件留在 `originals/`（`python3 scripts/agents/copy-originals.py check <草稿目录>/originals` 列出改过的），每次换上的反向规格在 `swap/hook-swap/reverse-*.json`。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 broad-mutation.py 起 cargo 的用法、gate.sh、全量 cargo test、check.sh）；没走定义三方；没提交。
- 没跑门禁阶段与各类 lint；code-tooling.sh 与 doc-experiments.sh 的新样本没经 stage-selftest.sh（没暂存、闸拒），用格的原样 python 在样本拷贝上核的，正式的交主 agent（上面第 1 条）。
- 慢的七份（agent-watch.py 的 memoryscan 在内）开关复跑没跑完，交主 agent（上面第 2 条）；verus-proof-check.py 基准趟就红，没跑开关趟。
- 兄弟任务那五份只报，没改；别的会话在改的 resource_release_broadcast.py、run-with-memory-cap.sh 没改。
- 「没点名」的 96 条（点名不带「」）与失败行不带 ✗ 的那几份没改：只改开关表补不齐，要改自证的打印或开关表的写法，不在这一条里。
- multi-host-run.sh 的坏字节、Ⓝ 那一格在并行下时红时不红、detached-run.sh 的核数② 时红时不红，都没点名（点了会在负载低时变成少红）。
- 草稿目录里没有仓副本、编译目录与库拷贝（`du -sh` 整个目录 51M，大头是 `runs/` 的续跑表 41M），不用删；表与工具留着给接着跑的人。

## 全仓复跑表（按份汇总）

「改前」取这一批改头注之前最早一次判出的结局（前一个 agent 的 judge-pre2、judge-all-1，这一趟开工时的 judge-now 与 judge-2，依次取第一个不是「没跑」的），「改后」取 `verify/judge-final.jsonl`。相同 = 点名的格与实际红的格逐格相同。

| 脚本 | 登记 | 兄弟任务 | 改前 | 改后 |
|---|---|---|---|---|
| `.claude/hooks/completion-cleanup-guard.sh` | 登记 | 兄弟任务 | 不相同 11、相同 17 | 不相同 11、相同 17 |
| `.claude/hooks/continuation-guard.sh` | 登记 |  | 相同 8 | 相同 8 |
| `.claude/hooks/handback-guard.sh` | 登记 |  | 相同 45 | 相同 45 |
| `.claude/hooks/lib_heavy_tests.py` | 没登记 |  | 不相同 44、相同 9 | 相同 53 |
| `.claude/hooks/lib_prompt_scratch_line.py` | 登记 | 兄弟任务 | 不相同 4、相同 2 | 不相同 4、相同 2 |
| `.claude/hooks/run-scope-guard.sh` | 没登记 |  | 相同 17 | 相同 17 |
| `.claude/hooks/runner-dispatch-guard.sh` | 登记 | 兄弟任务 | 相同 128 | 相同 128 |
| `.claude/hooks/two-logs-guard.sh` | 没登记 |  | 相同 11 | 相同 11 |
| `.claude/hooks/write-guard.sh` | 没登记 |  | 没点名 1 | 没点名 1 |
| `scripts/agents/agent-cost.py` | 登记 |  | 不相同 3、相同 11 | 相同 14 |
| `scripts/agents/agent-handover.py` | 登记 |  | 相同 3 | 相同 3 |
| `scripts/agents/agent-watch.py` | 没登记 |  | 没跑 119、超时 1 | 没跑 119、超时 1 |
| `scripts/agents/checklist-specs.py` | 登记 |  | 相同 3 | 相同 3 |
| `scripts/agents/cite-check.py` | 登记 |  | 不相同 1、相同 6 | 相同 7 |
| `scripts/agents/copy-originals.py` | 登记 |  | 相同 8 | 相同 8 |
| `scripts/agents/dispatch-prompt.py` | 没登记 | 兄弟任务 | 相同 20 | 相同 20 |
| `scripts/agents/snapshot-tree.py` | 登记 | 兄弟任务 | 不相同 3、相同 6 | 相同 9 |
| `scripts/agents/subagent_transcript.py` | 登记 |  | 相同 3 | 相同 3 |
| `scripts/agents/three-way-materials.py` | 登记 |  | 相同 6 | 相同 6 |
| `scripts/agents/three-way-rerun-template.sh` | 登记 |  | 不相同 1 | 相同 1 |
| `scripts/crash/check-segment-registry.py` | 没登记 |  | 不相同 2、相同 1 | 不相同 2、相同 1 |
| `scripts/crash/cpu-judging-sites-check.py` | 没登记 |  | 没点名 8 | 没点名 8 |
| `scripts/crash/crash-amplification-lib.sh` | 没登记 |  | 没点名 4 | 没点名 4 |
| `scripts/crash/crash-amplification.sh` | 没登记 |  | 没跑 50 | 没跑 50 |
| `scripts/crash/crash-case-check.py` | 登记 |  | 相同 23 | 相同 23 |
| `scripts/crash/crash-flow-coverage.py` | 没登记 |  | 没点名 11 | 没点名 11 |
| `scripts/crash/crash-point-replay.sh` | 没登记 |  | 没跑 41 | 没跑 41 |
| `scripts/crash/gpu-exemptions-check.py` | 没登记 |  | 没点名 4 | 没点名 4 |
| `scripts/crash/gpu-quiesce-run.sh` | 没登记 |  | 不相同 7、相同 9 | 相同 16 |
| `scripts/devices/snapshot-check.sh` | 没登记 |  | 相同 2 | 相同 2 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | 没登记 |  | 不相同 15、相同 12 | 相同 27 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | 没登记 |  | 没点名 5 | 没点名 5 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | 没登记 |  | 没点名 6 | 没点名 6 |
| `scripts/experiment-apparatus/replay.sh` | 没登记 |  | 不相同 10、没点名 4、相同 18 | 没点名 4、相同 28 |
| `scripts/gate/apply-writer-patch.py` | 登记 |  | 不相同 2、相同 8 | 相同 10 |
| `scripts/gate/changed-paths.sh` | 没登记 |  | 不相同 7、没点名 1、相同 1 | 不相同 7、没点名 1、相同 1 |
| `scripts/gate/closeout-status.py` | 登记 |  | 相同 1 | 相同 1 |
| `scripts/gate/gate-structure-check.py` | 没登记 |  | 不相同 4 | 不相同 4 |
| `scripts/gate/merge-and-verify.sh` | 没登记 |  | 不相同 2、相同 9、超时 31 | 相同 42 |
| `scripts/gate/stage-mine.py` | 没登记 |  | 不相同 3 | 不相同 3 |
| `scripts/gate/verus-proof-check.py` | 没登记 |  | 没跑 64 | 没跑 64 |
| `scripts/hosts/detached-run.sh` | 没登记 |  | 不相同 6、没点名 31、没红 1、相同 3、超时 12 | 不相同 1、没点名 39、没红 1、相同 11、超时 1 |
| `scripts/hosts/multi-host-run.sh` | 没登记 |  | 不相同 5、坏字节+不同 13、坏字节+对 1、相同 9 | 坏字节+不同 16、坏字节+对 2、相同 10 |
| `scripts/hosts/resource_release_broadcast.py` | 没登记 |  | 不相同 13 | 不相同 13 |
| `scripts/hosts/run-build-directory.sh` | 登记 |  | 相同 4 | 相同 4 |
| `scripts/hosts/run-with-memory-cap.sh` | 没登记 |  | 没跑 70 | 没跑 70 |
| `scripts/kb/archive-past-rounds.py` | 没登记 |  | 不相同 7、相同 6 | 相同 13 |
| `scripts/kb/batch-target.py` | 登记 |  | 不相同 2、相同 1 | 相同 3 |
| `scripts/kb/claim-experiment.sh` | 登记 |  | 相同 3 | 相同 3 |
| `scripts/kb/doc-consolidate.py` | 登记 |  | 相同 3 | 相同 3 |
| `scripts/kb/history-consolidate.py` | 没登记 |  | 不相同 1、相同 2 | 相同 3 |
| `scripts/kb/insert-row.py` | 没登记 |  | 不相同 2、相同 2 | 相同 4 |
| `scripts/kb/kb-sections.py` | 没登记 |  | 不相同 1 | 不相同 1 |
| `scripts/kb/kb-spec-check.py` | 登记 |  | 不相同 6、相同 10 | 相同 16 |
| `scripts/kb/migrate-changelog-format.py` | 没登记 |  | 不相同 10 | 不相同 10 |
| `scripts/kb/milestone-close-owed-check.py` | 没登记 |  | 没点名 7 | 没点名 7 |
| `scripts/kb/quote-kb.py` | 没登记 |  | 不相同 3、相同 7 | 相同 10 |
| `scripts/kb/replace-batch.py` | 登记 |  | 不相同 1、相同 4 | 相同 5 |
| `scripts/kb/replace-once.py` | 没登记 |  | 不相同 1、相同 2 | 不相同 1、相同 2 |
| `scripts/kb/stale-candidates.py` | 登记 |  | 不相同 1、相同 5 | 相同 6 |
| `scripts/kb/sweep-term.py` | 没登记 |  | 不相同 1、相同 9 | 相同 10 |
| `scripts/mutation/broad-mutation.py` | 没登记 |  | 没跑 77 | 没跑 77 |
| `scripts/mutation/crates-mutation-light-rows.py` | 没登记 |  | 没点名 6 | 没点名 6 |
| `scripts/mutation/mutate.sh` | 没登记 |  | 没跑 33 | 没跑 33 |
| `scripts/mutation/mutation-shard-run.sh` | 没登记 |  | 没跑 16 | 没跑 16 |
| `scripts/mutation/prove-red.sh` | 没登记 |  | 不相同 4 | 相同 4 |

## 全仓复跑表（逐条）

共 1257 条，次序按脚本、开关。

| 脚本 | 开关 | 登记 | 改前 | 改后 |
|---|---|---|---|---|
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=all-sessions` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=attribution-by-window` | 是 | 不相同 | 不相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=background-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=build-directory-declarable` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=build-directory-unrecognized` | 是 | 不相同 | 不相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=creator-alive-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=deliverables-ignored` | 是 | 不相同 | 不相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=guard-off` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=handover-exempt-for-main` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=ledger-ignored` | 是 | 不相同 | 不相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=library-copy-unrecognized` | 是 | 不相同 | 不相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=notifications-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=open-files-ignored` | 是 | 不相同 | 不相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=prompt-scratch-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=prompt-scratch-relative-by-cwd` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=release-never` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=release-on-any-continuation` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=reminder-counts-as-running` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=repository-copy-unrecognized` | 是 | 不相同 | 不相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=run-holders-leaf-only` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=run-windows-ignored` | 是 | 不相同 | 不相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=scratchpad-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=size-omitted` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=subagent-run-session-level` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=subagent-sees-whole-session` | 是 | 不相同 | 不相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=subagents-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=unjudged-pass-silent` | 是 | 不相同 | 不相同 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=unjudged-skip-silent` | 是 | 不相同 | 不相同 |
| `.claude/hooks/continuation-guard.sh` | `CONTINUATION_GUARD_BREAK=anyrecord` | 是 | 相同 | 相同 |
| `.claude/hooks/continuation-guard.sh` | `CONTINUATION_GUARD_BREAK=disable-check` | 是 | 相同 | 相同 |
| `.claude/hooks/continuation-guard.sh` | `CONTINUATION_GUARD_BREAK=honor-outside-selftest` | 是 | 相同 | 相同 |
| `.claude/hooks/continuation-guard.sh` | `CONTINUATION_GUARD_BREAK=ignore-main-handback` | 是 | 相同 | 相同 |
| `.claude/hooks/continuation-guard.sh` | `CONTINUATION_GUARD_BREAK=library-unread` | 是 | 相同 | 相同 |
| `.claude/hooks/continuation-guard.sh` | `CONTINUATION_GUARD_BREAK=missing-record-passes` | 是 | 相同 | 相同 |
| `.claude/hooks/continuation-guard.sh` | `CONTINUATION_GUARD_BREAK=unparsable-input-silent` | 是 | 相同 | 相同 |
| `.claude/hooks/continuation-guard.sh` | `SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=cite-any-agent` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=honor-break-outside-selftest` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=judge-general-purpose` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=landing-applies-always` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=landing-case-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=landing-exemption-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=landing-location-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=leg-report-citation-root-main` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=leg-report-git-directory-trusted` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=leg-report-outside-root-kept` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=leg-report-relative-root-only` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=leg-report-subdirectory-kept` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=leg-report-work-root-token-kept` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=leg-report-worktree-error-unnamed` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=leg-report-worktrees-unread` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=leg-types-hardcoded` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=limit-hint-unnamed` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-any-file` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-bash-writes-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-copy-judged` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-echo-unread` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-exit-code-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-git-toplevel-unknown` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-persisted-unread` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-pipestatus-unread` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-position-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-refusal-reasons-merged` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-refused-counts` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-root-unmatched` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-saved-status-unread` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-timeout-refused` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=naming-tool-exit-trusted` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=no-background` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=no-cite` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=no-green-count` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=no-landing-checklist` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=no-limit` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=no-naming-check` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=no-scribe-targets` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=scribe-section-anywhere` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=scribe-section-unread` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `HANDBACK_GUARD_BREAK=zero-count-ok` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown` | 是 | 相同 | 相同 |
| `.claude/hooks/handback-guard.sh` | `SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` | 是 | 相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=bench-unjudged` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=break-variable-fact-dropped` | 否 | 相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=cargo-run-device-replay-allowed` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=checker-dependents-unjudged` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=command-word-variables-unexpanded` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=crash-amplification-unnamed` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=crash-point-replay-unnamed` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=device-main-as-test` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=device-main-replay-allowed` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=device-replay-registry-unreadable-allowed` | 否 | 相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=e165-pin-backfill-as-append` | 否 | 相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=e165-pin-check-table-not-a-mode` | 否 | 相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=e165-pin-old-library-option` | 否 | 相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=e165-pin-unexpanded-as-replay` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=experiment-replay-unnamed` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=gate-top-library-as-stage` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=gpu-quiesce-opaque` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=judged-by-name-path-tail` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=light-scale-variable-ignored` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=light-tier-any-arguments` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=light-tier-arrays-unexpanded` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=light-tier-break-variable-ignored` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=light-tier-local-only-unrecognized` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=light-tier-memory-cap-unrequired` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=light-tier-unrecognized` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=light-tier-variables-unexpanded` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=manifest-path-home-unexpanded` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=manifest-path-variable-unjudged` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=package-flag-members-only` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=package-flag-needs-directory` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=posix-brackets-by-python-glob` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=posix-brackets-unresolved-literal` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=prove-red-unjudged` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=qemu-build-unnamed` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=run-build-directory-opaque` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=selftest-work-copy-content-unchecked` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=selftest-work-copy-howto-short` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=selftest-work-copy-row-unchecked` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=selftest-work-copy-unrecognized` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=two-logs-opaque` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=unexpanded-binary-unjudged` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=unexpanded-pin-same-message` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_HEAVY_TESTS_BREAK=verus-read-only-needs-literal-arguments` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_SHELL_WORDS_BREAK=arguments-as-executed` | 否 | 相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_SHELL_WORDS_BREAK=array-appends-unseen` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_SHELL_WORDS_BREAK=detached-run-force-kept` | 否 | 相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_SHELL_WORDS_BREAK=env-chdir-ignored` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_SHELL_WORDS_BREAK=env-option-clusters-unparsed` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_SHELL_WORDS_BREAK=env-split-string-unexpanded` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_SHELL_WORDS_BREAK=python-options-by-dash` | 否 | 不相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_SHELL_WORDS_BREAK=shell-standard-input-unread` | 否 | 相同 | 相同 |
| `.claude/hooks/lib_heavy_tests.py` | `LIB_SHELL_WORDS_BREAK=variable-rebinding-unseen` | 否 | 相同 | 相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=code-blocks-read` | 是 | 不相同 | 不相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=fence-info-with-backtick` | 是 | 相同 | 相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere` | 是 | 不相同 | 不相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=relative-by-cwd` | 是 | 相同 | 相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=roots-unfiltered` | 是 | 不相同 | 不相同 |
| `.claude/hooks/lib_prompt_scratch_line.py` | `LIB_PROMPT_SCRATCH_LINE_BREAK=unclosed-fence-read` | 是 | 不相同 | 不相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=allowed-unrecorded` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=cargo-targets-as-whole` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=count-unchecked` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=declaration-optional` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=failure-unrecorded` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=honor-outside-selftest` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=kind-unchecked` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=last-runs-unread` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=query-forms-declared` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=reason-unchecked` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=replay-list-items-declared` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=scripts-unread` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=segments-unchecked` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=subagent-whole-allowed` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=template-keeps-declaration` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=tooling-scripts-read` | 否 | 相同 | 相同 |
| `.claude/hooks/run-scope-guard.sh` | `RUN_SCOPE_GUARD_BREAK=wrappers-opaque` | 否 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `LIB_LONG_RUN_BREAK=hosts-canonical-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `LIB_LONG_RUN_BREAK=hosts-suffix-accepted` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `LIB_LONG_RUN_BREAK=hosts-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `LIB_LONG_RUN_BREAK=partial-accepted` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `LIB_LONG_RUN_BREAK=unshardable-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=absent-task-counted` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=admission-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=batch-file-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=batch-label-digits-only` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=batch-line-optional` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=batch-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=check-mode-registers` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=code-block-ambiguity-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=code-block-fallback-unmasked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=code-block-library-missing` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=code-block-line-rule` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=code-fence-loose` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=concurrency-no-limit` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=disable-check` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=disable-heavy-test` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=disable-snapshot` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=general-purpose-allowed` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=general-purpose-free` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=heavy-permission-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=host-quota-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=host-quota-whole-prompt` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=host-quota-wide` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=hosts-valve-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=inputs-hint-dropped` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=inputs-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=json-error-denies` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=json-error-silent` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=leg-report-any-round` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=leg-report-forbidden-counted` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=leg-report-forbidden-own-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=leg-report-line-optional` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=leg-report-line-root-only` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=leg-report-name-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=leg-report-outside-root-kept` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=leg-report-placeholder-kept` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=leg-report-work-root-token-kept` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=leg-types-hardcoded` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=light-tier-commands-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=light-tier-quick-counted` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=long-run-codeblock-judged` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=long-run-duplicate-label-first-wins` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=long-run-reuse-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=long-run-types-narrowed` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=long-run-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=long-run-values-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=mutation-table-free` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=placeholders-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=quote-pairs-per-line` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=real-prompts-absent-green` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=real-prompts-digest-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=registry-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=registry-no-prune` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=report-line-labels-split` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=report-line-position-anchored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=report-line-unanchored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=report-name-case-sensitive` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=report-name-codeblock-judged` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=report-name-negation-judged` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=report-name-prefix-extension-lowercase` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=report-name-prefixes-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=report-name-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=report-name-whole-path` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=runner-backfill-refused` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=scope-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=scope-library-unread` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=scope-negation-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=scope-token-unexpanded` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=scratch-line-legacy-root-accepted` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=scratch-line-root-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=selftest-location-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-line-unread` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-name-substring` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-quote-short-ok` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-round-name-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-source-any-path` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-source-escapes` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-abandoned-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-any-agent` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-body-optional` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-closed-round-open` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-error-unrecorded` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-loose-lines` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-missing-spec-denies` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-no-edges` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-no-normalize` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-spec-unread` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-unreadable-aborts` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-unreadable-denies` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-unreadable-unmarked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-valve-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=snapshot-valve-loose` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=spec-check-skipped` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=stop-at-first` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-angle-dropped` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-codeblock-judged` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-formula-accepted` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-limit-any-refused` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-limit-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-line-optional` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-note-accepted` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-required-only-writer` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-required-six-types` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-valve-hint-merged` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-valve-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=thread-valve-loose` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=three-way-report-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=valve-reason-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=verdict-not-spec` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=wall-clock-optional` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=window-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=window-never-lifted` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=writer-lock-fence-ambiguity-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=writer-lock-live-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=writer-lock-retry-refused` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=writer-lock-root-locked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=writer-lock-stale-kept` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=writer-lock-unchecked` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=writer-lock-unreadable-absent` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `RUNNER_DISPATCH_GUARD_BREAK=writer-lock-valve-ignored` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown` | 是 | 相同 | 相同 |
| `.claude/hooks/runner-dispatch-guard.sh` | `SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` | 是 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=all-cargo-guarded` | 否 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=failure-unrecorded` | 否 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=group-redirect-unchecked` | 否 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=honor-outside-selftest` | 否 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=nested-code-unread` | 否 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=no-way-out` | 否 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=non-log-targets-guarded` | 否 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=query-forms-guarded` | 否 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=redirect-unchecked` | 否 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=tee-unchecked` | 否 | 相同 | 相同 |
| `.claude/hooks/two-logs-guard.sh` | `TWO_LOGS_GUARD_BREAK=variable-names-ignored` | 否 | 相同 | 相同 |
| `.claude/hooks/write-guard.sh` | `LIB_WRITER_LOCK_BREAK=takeover-sweeps-new-claims` | 否 | 没点名 | 没点名 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=day-cut-by-record-zone` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=draft-anywhere-in-line` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=fix-no-word-end` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=leg-types-by-prefix` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=main-session-by-mtime` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=narrow-rework-words` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=no-bash-writes` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=no-dedupe` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=no-path-normalize` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=outcome-out-clobber` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=three-way-by-type` | 是 | 不相同 | 相同 |
| `scripts/agents/agent-cost.py` | `AGENT_COST_BREAK=three-way-history-unread` | 是 | 不相同 | 相同 |
| `scripts/agents/agent-cost.py` | `SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` | 是 | 相同 | 相同 |
| `scripts/agents/agent-cost.py` | `SUBAGENT_TRANSCRIPT_BREAK=start-by-mtime` | 是 | 不相同 | 相同 |
| `scripts/agents/agent-handover.py` | `AGENT_HANDOVER_BREAK=continuations` | 是 | 相同 | 相同 |
| `scripts/agents/agent-handover.py` | `AGENT_HANDOVER_BREAK=keepscratch` | 是 | 相同 | 相同 |
| `scripts/agents/agent-handover.py` | `AGENT_HANDOVER_BREAK=written` | 是 | 相同 | 相同 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=ack` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=acksneverpruned` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=acksnotpersisted` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=backgroundwait` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=banned` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=bannedrefused` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=bannedwindow` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=context` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=contextzero` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=corerecords` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=coreusage` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=cost` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=cpu` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=deadline` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=detections` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=detectiontype` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=discoverfallback` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=discovernone` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=display` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=finished` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=handback` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=handoveranytype` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavycommitcheck` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyexempt` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavylighttier` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavylighttierargs` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyprefix` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavysampleselftest` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyscan` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyselftestancestor` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyselftestanycopy` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyselftestcondition` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyselftestpathonly` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyselftesttemporary` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavytopmost` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavywatchdog` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavywrapper` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=helpformat` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heredoc` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=holders` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=idle` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=infoalertwakes` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=infowakes` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquiry` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquiryack` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquirydelivered` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquiryended` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquiryreset` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquirysent` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=keepscratch` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=keptcopy` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=keptcopyage` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=keptcopyexists` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=leftover` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=leftoverack` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=leftoverexit` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=leftovergrace` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=leftovernotified` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=loop` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=loopnoise` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryfixedline` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memorygrew` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memorygrewfloor` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryoutcome` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memorypeakfloor` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memorypeakfloorfixed` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memorypoll` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryqueue` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryscan` | 否 | 超时 | 超时 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryunit` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=midwaymerged` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=movedbackground` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=mutationrows` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=nosingleinstance` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=nowake` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=orphan` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=otheragent` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=outcomeignored` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=ownrefusal` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=pipeancestor` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=pipeupstream` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=pipeupstreamalways` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=processexit` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=progressanypath` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=progressnote` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=progressstale` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=refusedwakes` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=remindertimer` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=repeat` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=repeatsforgotten` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=restarthint` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=resume` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=resumeturn` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=rootfree` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=scopeunlisted` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=scratchhog` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=scratchhogage` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=scratchhogtag` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=settle` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=stale` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=stalepending` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=staleselftestscratch` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=stalesession` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=stalesessionrestart` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=taskstop` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=terminateacked` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=terminatezero` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=timeout` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=toolrevoke` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictlines` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftest` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftestanyfile` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftestpathonly` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftestprivateignored` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftestsharedtmpdir` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftestwrapperseen` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=zombie` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` | 否 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` | 否 | 没跑 | 没跑 |
| `scripts/agents/checklist-specs.py` | `CHECKLIST_SPECS_BREAK=preamble-without-range` | 是 | 相同 | 相同 |
| `scripts/agents/checklist-specs.py` | `CHECKLIST_SPECS_BREAK=raw-arguments` | 是 | 相同 | 相同 |
| `scripts/agents/checklist-specs.py` | `CHECKLIST_SPECS_BREAK=trust-follow` | 是 | 相同 | 相同 |
| `scripts/agents/cite-check.py` | `CITE_CHECK_BREAK=always-ok` | 是 | 相同 | 相同 |
| `scripts/agents/cite-check.py` | `CITE_CHECK_BREAK=bare-line-refs-ignored` | 是 | 相同 | 相同 |
| `scripts/agents/cite-check.py` | `CITE_CHECK_BREAK=ignore-changed` | 是 | 相同 | 相同 |
| `scripts/agents/cite-check.py` | `CITE_CHECK_BREAK=no-background-check` | 是 | 相同 | 相同 |
| `scripts/agents/cite-check.py` | `CITE_CHECK_BREAK=no-heading-check` | 是 | 相同 | 相同 |
| `scripts/agents/cite-check.py` | `CITE_CHECK_BREAK=raw-arguments` | 是 | 不相同 | 相同 |
| `scripts/agents/cite-check.py` | `CITE_CHECK_BREAK=title-any-section` | 是 | 相同 | 相同 |
| `scripts/agents/copy-originals.py` | `COPY_ORIGINALS_BREAK=check-ignores-changes` | 是 | 相同 | 相同 |
| `scripts/agents/copy-originals.py` | `COPY_ORIGINALS_BREAK=differing-target-overwritten` | 是 | 相同 | 相同 |
| `scripts/agents/copy-originals.py` | `COPY_ORIGINALS_BREAK=empty-file-list-accepted` | 是 | 相同 | 相同 |
| `scripts/agents/copy-originals.py` | `COPY_ORIGINALS_BREAK=guard-off` | 是 | 相同 | 相同 |
| `scripts/agents/copy-originals.py` | `COPY_ORIGINALS_BREAK=inside-repository-accepted` | 是 | 相同 | 相同 |
| `scripts/agents/copy-originals.py` | `COPY_ORIGINALS_BREAK=outside-scratch-roots-accepted` | 是 | 相同 | 相同 |
| `scripts/agents/copy-originals.py` | `COPY_ORIGINALS_BREAK=relative-scratch-accepted` | 是 | 相同 | 相同 |
| `scripts/agents/copy-originals.py` | `COPY_ORIGINALS_BREAK=source-checks-skipped` | 是 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=claim-check-skipped` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=claim-line-dropped` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=draft-line-own-regex` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=guard-skipped` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=hosts-valve-hint-dropped` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=ignore-placeholders` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=leg-types-hardcoded` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=live-words-narrowed` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=long-run-hosts-live-restored` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=long-run-lines-dropped` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=long-run-types-narrowed` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=report-default-dropped` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=reuse-line-dropped` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=selftest-location-dropped` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=thread-formula-restored` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=three-way-report-default-dropped` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `DISPATCH_PROMPT_BREAK=types-before-nine` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `RUNNER_DISPATCH_GUARD_BREAK=placeholders-ignored` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `RUNNER_DISPATCH_GUARD_BREAK=thread-limit-any-refused` | 否 | 相同 | 相同 |
| `scripts/agents/dispatch-prompt.py` | `RUNNER_DISPATCH_GUARD_BREAK=three-way-report-unchecked` | 否 | 相同 | 相同 |
| `scripts/agents/snapshot-tree.py` | `SNAPSHOT_TREE_BREAK=diff-markers-kept` | 是 | 相同 | 相同 |
| `scripts/agents/snapshot-tree.py` | `SNAPSHOT_TREE_BREAK=guard-off` | 是 | 相同 | 相同 |
| `scripts/agents/snapshot-tree.py` | `SNAPSHOT_TREE_BREAK=head-only` | 是 | 不相同 | 相同 |
| `scripts/agents/snapshot-tree.py` | `SNAPSHOT_TREE_BREAK=index-diff-ignored` | 是 | 不相同 | 相同 |
| `scripts/agents/snapshot-tree.py` | `SNAPSHOT_TREE_BREAK=no-unquote` | 是 | 不相同 | 相同 |
| `scripts/agents/snapshot-tree.py` | `SNAPSHOT_TREE_BREAK=path-moves-ignored` | 是 | 相同 | 相同 |
| `scripts/agents/snapshot-tree.py` | `SNAPSHOT_TREE_BREAK=result-blob-ignored` | 是 | 相同 | 相同 |
| `scripts/agents/snapshot-tree.py` | `SNAPSHOT_TREE_BREAK=skip-sha-check` | 是 | 相同 | 相同 |
| `scripts/agents/snapshot-tree.py` | `SNAPSHOT_TREE_BREAK=worktree-ignored` | 是 | 相同 | 相同 |
| `scripts/agents/subagent_transcript.py` | `SUBAGENT_TRANSCRIPT_BREAK=honor-break-outside-selftest` | 是 | 相同 | 相同 |
| `scripts/agents/subagent_transcript.py` | `SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` | 是 | 相同 | 相同 |
| `scripts/agents/subagent_transcript.py` | `SUBAGENT_TRANSCRIPT_BREAK=start-by-mtime` | 是 | 相同 | 相同 |
| `scripts/agents/three-way-materials.py` | `QUOTE_KB_BREAK=invariants-to-readme` | 是 | 相同 | 相同 |
| `scripts/agents/three-way-materials.py` | `THREE_WAY_MATERIALS_BREAK=ignore-not-numbers` | 是 | 相同 | 相同 |
| `scripts/agents/three-way-materials.py` | `THREE_WAY_MATERIALS_BREAK=no-resume` | 是 | 相同 | 相同 |
| `scripts/agents/three-way-materials.py` | `THREE_WAY_MATERIALS_BREAK=own-number-reading` | 是 | 相同 | 相同 |
| `scripts/agents/three-way-materials.py` | `THREE_WAY_MATERIALS_BREAK=skip-declared-kind` | 是 | 相同 | 相同 |
| `scripts/agents/three-way-materials.py` | `THREE_WAY_MATERIALS_BREAK=skip-index-extra` | 是 | 相同 | 相同 |
| `scripts/agents/three-way-rerun-template.sh` | `RERUN_TEMPLATE_BREAK=root-from-self-path` | 是 | 不相同 | 相同 |
| `scripts/crash/check-segment-registry.py` | `CHECK_SEGMENT_REGISTRY_BREAK=accept-extra-path` | 否 | 不相同 | 不相同 |
| `scripts/crash/check-segment-registry.py` | `CHECK_SEGMENT_REGISTRY_BREAK=path-rows-to-product` | 否 | 不相同 | 不相同 |
| `scripts/crash/check-segment-registry.py` | `CHECK_SEGMENT_REGISTRY_BREAK=product-required` | 否 | 相同 | 相同 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=judge-self-test-covers-whole-file` | 否 | 没点名 | 没点名 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=judge-self-tests-not-followed` | 否 | 没点名 | 没点名 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=judging-bodies-not-followed` | 否 | 没点名 | 没点名 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=local-function-of-the-same-name-followed` | 否 | 没点名 | 没点名 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=marker-anywhere-in-file` | 否 | 没点名 | 没点名 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=receiver-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=table-targets-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/crash/cpu-judging-sites-check.py` | `CPU_JUDGING_SITES_CHECK_BREAK=unmarked-sites-accepted` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-amplification-lib.sh` | `CRASH_AMPLIFICATION_LIB_BREAK=rocksdb-entry-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-amplification-lib.sh` | `CRASH_AMPLIFICATION_LIB_BREAK=term-not-forwarded` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-amplification-lib.sh` | `CRASH_AMPLIFICATION_LIB_BREAK=term-not-waited` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-amplification-lib.sh` | `CRASH_AMPLIFICATION_LIB_BREAK=term-trap-not-restored` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=accept-bad-red-field` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=accept-expand-mismatch` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=accept-not-exhaustive` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=accept-unknown-item` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=accept-unknown-reuse` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=accept-zero-full-item` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=accept-zero-lines` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=beyond-expansion-ignored` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=beyond-expansion-missing-as-dash` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=beyond-expansion-unreported` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=condition-before-red` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=count-stopped-as-run` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=dash-counts-as-registered` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=domain-unlisted-segment-accepted` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=domains-missing-accepted` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=driver-77-as-failure` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=driver-not-run-as-zero-lines` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=excuse-share-lines-without-import` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=flow-share-lines-unexcused` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=hide-scaled-not-exhaustive` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=honor-test-variables-outside-selftest` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=ignore-red-count` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=ignore-red-on-78` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=ignore-summed-per-row` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=ignore-unknown-as-zero` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=import-line-as-share-line` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=item-78-as-failure` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=judge-not-run-ignored` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=logs-single` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=no-quiesce` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=registered-excuses-the-whole-item` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=registered-segments-ignored` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=registered-segments-unreported` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=reuse-across-shares` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=reuse-as-zero` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=rocksdb-entry-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=run-target-kept` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=segments-registry-not-sent` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=segments-registry-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=skip-collection-count` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=skip-filter` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=skip-ignore-report` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=skip-not-exhaustive-in-full-scope` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=slice-domains-unreported` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=slice-progress-zero-accepted` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=slice-zero-despite-reuse` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=sum-reuse-of-every-line` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `CRASH_AMPLIFICATION_BREAK=swallow-exit` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `GPU_QUIESCE_BREAK=restore-after-run` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-amplification.sh` | `GPU_QUIESCE_BREAK=run-despite-failed-quiesce` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=crash-case-rows-only` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=fresh-pool-loop-unseen` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=given-states-unlisted` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=helpers-unseen` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=ignore-unseen` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=items-table-unchecked` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=knob-basis-unchecked` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=knob-source-unchecked` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=milestone-names-unseen` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=module-declaration-scans-strings` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=module-declaration-unchecked` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=must-red-package-ignored` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=must-red-unchecked` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=one-scenario-tier-unscanned` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=only-ignored` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=package-unchecked` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=quick-tier-counts` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=rows-ignored` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=self-sweep-unseen` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=test-names-exempt-stale-unseen` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=test-names-unseen` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=waiver-ignored` | 是 | 相同 | 相同 |
| `scripts/crash/crash-case-check.py` | `CRASH_CASE_CHECK_BREAK=writable-mount-phases-unlisted` | 是 | 相同 | 相同 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=any-expansion-covers` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=block-count-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=empty-reason-accepted` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=entry-absence-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=ignored-growth-accepted` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=incomplete-judged-as-complete` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=reuse-without-recall` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=step-kind-coverage-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=unregistered-entry-accepted` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=unseeded-marker-ignored` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-flow-coverage.py` | `CRASH_FLOW_COVERAGE_BREAK=zero-growth-accepted` | 否 | 没点名 | 没点名 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=accept-unknown-item` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=enumeration-exit-ignored` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=fast-tier-classes-sent` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=fast-tier-red-continues` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=fast-tier-reuse-not-counted` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=fast-tier-slice-states-not-set` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=fast-tier-slice-states-override-caller` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=fast-tier-two-hosts-throwaway-library` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=flow-coverage-table-skipped` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=full-in-a-library-of-its-own` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=full-keeps-reuse` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=given-states-run-condition-as-red` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=gpu-cards-not-sent` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=gpu-test-threads-unset` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=gpu-without-quiesce` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=heavy-slice-states-set` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=heavy-without-fast-tier` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=honor-test-variables-outside-selftest` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=list-items-builds` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=local-only-accepted-with-list-items` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=local-only-ignored` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=logs-single` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=memory-cap-not-forwarded` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=not-run-as-green` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=pipeline-78-as-red` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=pipeline-red-ignored` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=quick-slice-states-not-set` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=quick-slice-states-override-caller` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=quick-throwaway-library` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=quick-without-both-tiers` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=red-as-green` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=rocksdb-entry-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=run-target-kept` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=segments-registry-not-sent` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=segments-registry-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=skip-collection-count` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=threads-not-defaulted` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=two-hosts-check-failure-as-single` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=two-hosts-not-used` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=two-hosts-unreachable-as-red` | 否 | 没跑 | 没跑 |
| `scripts/crash/crash-point-replay.sh` | `CRASH_POINT_REPLAY_BREAK=two-hosts-when-off` | 否 | 没跑 | 没跑 |
| `scripts/crash/gpu-exemptions-check.py` | `GPU_EXEMPTIONS_CHECK_BREAK=marker-column-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/crash/gpu-exemptions-check.py` | `GPU_EXEMPTIONS_CHECK_BREAK=row-fields-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/crash/gpu-exemptions-check.py` | `GPU_EXEMPTIONS_CHECK_BREAK=unmarked-rows-accepted` | 否 | 没点名 | 没点名 |
| `scripts/crash/gpu-exemptions-check.py` | `GPU_EXEMPTIONS_CHECK_BREAK=unregistered-markers-accepted` | 否 | 没点名 | 没点名 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=accept-missing-quiesce-commands` | 否 | 相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=broadcast-exit-swallowed` | 否 | 相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=broadcast-none` | 否 | 不相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=broadcast-unstarted` | 否 | 不相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=cards-from-unquiesced-key` | 否 | 相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=check-every-card` | 否 | 不相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=drop-stop-exit` | 否 | 不相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=exempt-holder-session` | 否 | 相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=failure-log-in-runtime-dir` | 否 | 不相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=failure-log-on-stderr` | 否 | 不相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=keep-inherited-check-variables` | 否 | 相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=no-failure-log` | 否 | 不相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=no-holder-descendants` | 否 | 相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=restore-after-run` | 否 | 相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=run-despite-failed-quiesce` | 否 | 相同 | 相同 |
| `scripts/crash/gpu-quiesce-run.sh` | `GPU_QUIESCE_BREAK=trust-stale-holders` | 否 | 相同 | 相同 |
| `scripts/devices/snapshot-check.sh` | `SNAPSHOT_CHECK_BREAK=ignore-mismatch` | 否 | 相同 | 相同 |
| `scripts/devices/snapshot-check.sh` | `SNAPSHOT_CHECK_BREAK=skip-head-presence` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=byrownumber` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=defaultone` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=gpumemoryasresult` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=j6qnfromj6` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=noanchorcheck` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=nodupcheck` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=nolocationcheck` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=noprofile` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=noprofilecheck` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=owntablesplit` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=pincheckskipped` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=progressnever` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r3firstredswap` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r3itemsmissing` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r3kapinunchecked` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r3noprofile` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r4firstredswap` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r4gateignored` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r4itemsmissing` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r4kgcountsasred` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r4kgpinunchecked` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r4noprofile` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=r5pinsignored` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=recoveryonly` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=rerunall` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=retainedmissingred` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `E165_KILL_BOUNDS_BREAK=zfirstfour` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | `E197_DRIVE_BREAK=cap-not-raised` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | `E197_DRIVE_BREAK=lost-cell-dropped` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | `E197_DRIVE_BREAK=missing-peak-as-zero` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | `E197_DRIVE_BREAK=s1-always-same` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` | `E197_DRIVE_BREAK=segment-not-skipped` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=digest-mismatch-ignored` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=empty-cell-not-subtracted` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=line-ignores-the-sign` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=median-replaced-by-minimum` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=pc-red-threshold-ignored` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py` | `E197_REPORT_BREAK=slope-uses-the-top-point-only` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=archived-not-itemized` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=build-directory-kept` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=build-everything` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=build-log-single` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=build-output-dropped` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=cards-not-from-quiesced-list` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=copy-build-directories-kept` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=copy-lines-without-registry` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=driver-build-output-dropped` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=emitted-unchecked` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=exit-code-ignored` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=gpu-features-dropped` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=gpu-without-quiesce` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=guard-off` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=gz-read-raw` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=inputs-not-checked` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=jobs-from-nproc` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=load-errors-unchecked` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=load-field-ignored` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=not-run-as-broken` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=not-run-not-itemized` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=previous-output-kept` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=quiesce-failure-ignored` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=reuse-ignored` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=reuse-key-by-row-only` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=row-key-by-experiment` | 否 | 不相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=run-lines-without-features` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=run-lines-without-quiesce` | 否 | 没点名 | 没点名 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=term-not-forwarded` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=thread-registry-not-copied` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REPLAY_BREAK=unknown-experiment-accepted` | 否 | 相同 | 相同 |
| `scripts/experiment-apparatus/replay.sh` | `REUSE_CHECKS_BREAK=rejudged-sentinel-passes` | 否 | 相同 | 相同 |
| `scripts/gate/apply-writer-patch.py` | `APPLY_WRITER_PATCH_BREAK=append-duplicates` | 是 | 相同 | 相同 |
| `scripts/gate/apply-writer-patch.py` | `APPLY_WRITER_PATCH_BREAK=append-internal-duplicates` | 是 | 相同 | 相同 |
| `scripts/gate/apply-writer-patch.py` | `APPLY_WRITER_PATCH_BREAK=apply-unchecked` | 是 | 相同 | 相同 |
| `scripts/gate/apply-writer-patch.py` | `APPLY_WRITER_PATCH_BREAK=checker-section-ignored` | 是 | 相同 | 相同 |
| `scripts/gate/apply-writer-patch.py` | `APPLY_WRITER_PATCH_BREAK=copy-lock-ignored` | 是 | 相同 | 相同 |
| `scripts/gate/apply-writer-patch.py` | `APPLY_WRITER_PATCH_BREAK=ignore-workspace-lock` | 是 | 相同 | 相同 |
| `scripts/gate/apply-writer-patch.py` | `APPLY_WRITER_PATCH_BREAK=light-rows-unchecked` | 是 | 不相同 | 相同 |
| `scripts/gate/apply-writer-patch.py` | `APPLY_WRITER_PATCH_BREAK=no-hint` | 是 | 相同 | 相同 |
| `scripts/gate/apply-writer-patch.py` | `APPLY_WRITER_PATCH_BREAK=snapshot-ignored` | 是 | 不相同 | 相同 |
| `scripts/gate/apply-writer-patch.py` | `APPLY_WRITER_PATCH_BREAK=snapshot-unreadable-unmarked` | 是 | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=binary` | 否 | 相同 | 相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=empty-merge-base` | 否 | 不相同 | 不相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=fallback` | 否 | 没点名 | 没点名 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=header-anywhere` | 否 | 不相同 | 不相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=moved-kept` | 否 | 不相同 | 不相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=quotepath` | 否 | 不相同 | 不相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=swallow` | 否 | 不相同 | 不相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=untracked` | 否 | 不相同 | 不相同 |
| `scripts/gate/changed-paths.sh` | `LIB_CHANGED_PATHS_BREAK=untracked-lines` | 否 | 不相同 | 不相同 |
| `scripts/gate/closeout-status.py` | `CLOSEOUT_STATUS_BREAK=everything-done` | 是 | 相同 | 相同 |
| `scripts/gate/gate-structure-check.py` | `GATE_STRUCTURE_CHECK_BREAK=heavy-ignored` | 否 | 不相同 | 不相同 |
| `scripts/gate/gate-structure-check.py` | `GATE_STRUCTURE_CHECK_BREAK=list-ignored` | 否 | 不相同 | 不相同 |
| `scripts/gate/gate-structure-check.py` | `GATE_STRUCTURE_CHECK_BREAK=table-ignored` | 否 | 不相同 | 不相同 |
| `scripts/gate/gate-structure-check.py` | `GATE_STRUCTURE_CHECK_BREAK=unknown-ignored` | 否 | 不相同 | 不相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=all-packages-always` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=checker-quick-always-run` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=checker-quick-dependencies-ignored` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=checker-quick-elsewhere-run` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=checker-quick-prefix-unchecked` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=checker-quick-red-ignored` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=checker-quick-sample-exit-anywhere` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=checker-quick-skipped` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=cleanup-only-on-success` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=dependents-dropped` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=drop-discipline-lint` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=fixed-harness-packages` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=gpu-memory-as-test-red` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=ignore-red` | 否 | 不相同 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=ignore-workspace-lock` | 否 | 相同 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=keep-copy` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=keep-run-target` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=lint-table-optional` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=lock-changes-narrowed` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=logs-single` | 否 | 相同 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=no-checker-tier-guard` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=no-peer-hint` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=no-source-stamp` | 否 | 相同 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=no-touch-patched` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=peer-components-unchecked` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=peer-failure-unreported` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=peer-ignored` | 否 | 不相同 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=quick-memory-below-memory` | 否 | 相同 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=quick-memory-oom-as-red` | 否 | 相同 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=quick-memory-oom-exits-zero` | 否 | 相同 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=quick-memory-same-tier` | 否 | 相同 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=quick-memory-shared` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=quick-memory-table-ignored` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=refused-keeps-going` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=rocksdb-entry-unchecked` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=run-target-open-files-unchecked` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=skip-apply-unchecked` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=skip-build` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=skip-rebuild-check` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=tests-one-binary-at-a-time` | 否 | 相同 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `MERGE_AND_VERIFY_BREAK=zero-tests-green` | 否 | 超时 | 相同 |
| `scripts/gate/merge-and-verify.sh` | `SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` | 否 | 相同 | 相同 |
| `scripts/gate/stage-mine.py` | `STAGE_MINE_BREAK=by-row-takes-block` | 否 | 不相同 | 不相同 |
| `scripts/gate/stage-mine.py` | `STAGE_MINE_BREAK=first-occurrence-anchor` | 否 | 不相同 | 不相同 |
| `scripts/gate/stage-mine.py` | `STAGE_MINE_BREAK=write-index-unchecked` | 否 | 不相同 | 不相同 |
| `scripts/gate/verus-proof-check.py` | `REUSE_CHECKS_BREAK=rejudged-sentinel-passes` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=absent-kernel-not-run` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=anchor-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=cheating-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=commit-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=count-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=derived-clone-not-item` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=expectation-form-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=fingerprint-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=host-lookup-lines-kept` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=item-filter-ignored` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=key-mismatch-by-item-only` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=killed-as-proof-red` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=landing-by-count` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=landing-form-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=landing-missing-accepted` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=landing-read-from-unmutated-text` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=landing-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=landings-write-skipped` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=list-items-needs-landing` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=list-items-pipe-unhandled` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=must-red-key-with-name-and-path` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=mutation-accepts-green` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=mutation-rlimit-attribute-accepted` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=nonzero-errors-accepted` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=not-run-masks-red` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=queue-refused-keeps-queueing` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=require-entire-crate-per-function` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=require-success-key-per-function` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-ignores-key` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-key-function-only` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-key-whole-crate` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-key-with-timeout` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-key-without-judging-units` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-key-without-row` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-key-without-toolchain` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-key-without-units` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-lookup-skipped` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-records-red` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=reuse-records-whole-cell-only` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=rlimit-any-line-voids` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=rlimit-attribute-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=rlimit-counted-in-proof-errors` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=rlimit-counts-as-red` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=rlimit-unpinned` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=spec-decreases-not-item` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=spec-fn-as-item` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=submodule-verify-root` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=threads-default-cpu-count` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=timeout-as-pass` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=tool-versions-bare-z3-from-current-directory` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=tool-versions-empty-z3-path-as-unset` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=tool-versions-five-files-only` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=tool-versions-launcher-only` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=tool-versions-rustup-ignored` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=tool-versions-verus-environment-ignored` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=vacuity-accepts-verified` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=vacuity-all-at-once` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=vacuity-key-by-line-number` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=wrapper-252-one-howto` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=wrapper-exit-as-proof-red` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=wrapper-output-dropped` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=wrapper-queue-lines-swallowed` | 否 | 没跑 | 没跑 |
| `scripts/gate/verus-proof-check.py` | `VERUS_PROOF_CHECK_BREAK=zero-verified-accepted` | 否 | 没跑 | 没跑 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=archive-not-restored` | 否 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=assignment-name-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=break-honored-anywhere` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=broadcast-exit-swallowed` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=broadcast-none` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=broadcast-test-world` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=core-usage-unwatched` | 否 | 不相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=default-stop-timeout-unset` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=environment-dropped` | 否 | 超时 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=exit-dropped` | 否 | 超时 | 超时 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=in-caller-tree` | 否 | 超时 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=linger-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=log-unchecked` | 否 | 相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=long-run-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=observe-overwrites-unit` | 否 | 不相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=path-home-dropped` | 否 | 超时 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=query-check-after-archive` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=query-failure-as-gone` | 否 | 超时 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=ran-without-pid-trusted` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=record-write-failure-as-vanished` | 否 | 不相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=same-label-allowed` | 否 | 超时 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=same-second-restart-allowed` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=scopes-left-running` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=serial-declarations-unset` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=start-failed-trusted` | 否 | 相同 | 不相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=start-failure-not-rechecked` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=start-failure-unmarked` | 否 | 超时 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=stop-kill-unrecorded` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=stop-nonblocking` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=stop-timeout-ignored` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=unknown-alarm-missing` | 否 | 不相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=unregistered-units-hidden` | 否 | 不相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=wall-clock-optional` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-keys-by-path` | 否 | 超时 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-misses-short-runs` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-skips-unknown` | 否 | 没红 | 没红 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-stops-in-trap` | 否 | 超时 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-stops-units` | 否 | 超时 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-stops-units-on-hup` | 否 | 超时 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-stops-units-on-int` | 否 | 超时 | 没点名 |
| `scripts/hosts/detached-run.sh` | `DETACHED_RUN_BREAK=watch-trusts-registry` | 否 | 不相同 | 相同 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=exempt-sha256-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=header-live-reading` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=hosts-canonical-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=hosts-suffix-accepted` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=hosts-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=live-exempt-ignored` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=partial-accepted` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=script-unresolved-accepted` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=two-logs-not-unwrapped` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=unshardable-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_LONG_RUN_BREAK=wrappers-not-skipped` | 否 | 没点名 | 没点名 |
| `scripts/hosts/detached-run.sh` | `LIB_SHELL_WORDS_BREAK=detached-run-force-kept` | 否 | 没点名 | 没点名 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=break-honored-anywhere` | 否 | 坏字节+不同 | 相同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=broadcast-exit-swallowed` | 否 | 不相同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=broadcast-none` | 否 | 坏字节+不同 | 相同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=broadcast-test-world` | 否 | 坏字节+不同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=copy-requires-git` | 否 | 坏字节+不同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=core-usage-unwatched` | 否 | 相同 | 相同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=fetch-altered` | 否 | 不相同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=fetch-under-tmp` | 否 | 相同 | 相同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=gpu-memory-unexplained` | 否 | 相同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=local-reads-configuration` | 否 | 不相同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=local-target-kept` | 否 | 坏字节+不同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=local-without-memory-cap` | 否 | 不相同 | 相同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=no-peer-cap-limit` | 否 | 相同 | 相同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=no-peer-cleanup` | 否 | 坏字节+不同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=no-peer-stop-on-signal` | 否 | 坏字节+不同 | 相同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=no-redact` | 否 | 坏字节+对 | 坏字节+对 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=no-toolchain-check` | 否 | 相同 | 相同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=open-files-unchecked` | 否 | 坏字节+不同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=outside-target-accepted` | 否 | 相同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=peer-exit-dropped` | 否 | 相同 | 坏字节+对 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=peer-rocksdb-unchecked` | 否 | 坏字节+不同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=peer-without-memory-cap` | 否 | 相同 | 相同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=serial-declarations-unset` | 否 | 坏字节+不同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=switch-off-falls-back-to-local` | 否 | 坏字节+不同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=sync-ignored-files` | 否 | 不相同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=tmp-files-copied` | 否 | 坏字节+不同 | 相同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=vanished-tracked-tolerated` | 否 | 相同 | 坏字节+不同 |
| `scripts/hosts/multi-host-run.sh` | `MULTI_HOST_RUN_BREAK=vanished-untracked-fails` | 否 | 坏字节+不同 | 坏字节+不同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=fault-without-mark` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=follow-since-ignored` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=host-name-in-event` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=label-path-kept` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=no-local-write` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=no-prune` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=problems-on-stderr` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=push-despite-switch-off` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=push-unbounded` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=sequence-unlocked` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=skip-ignored` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=unreachable-exit` | 否 | 不相同 | 不相同 |
| `scripts/hosts/resource_release_broadcast.py` | `RESOURCE_BROADCAST_BREAK=wait-ignores-events` | 否 | 不相同 | 不相同 |
| `scripts/hosts/run-build-directory.sh` | `RUN_BUILD_DIRECTORY_BREAK=foreground` | 是 | 相同 | 相同 |
| `scripts/hosts/run-build-directory.sh` | `RUN_BUILD_DIRECTORY_BREAK=kept` | 是 | 相同 | 相同 |
| `scripts/hosts/run-build-directory.sh` | `RUN_BUILD_DIRECTORY_BREAK=owner-left-alone` | 是 | 相同 | 相同 |
| `scripts/hosts/run-build-directory.sh` | `RUN_BUILD_DIRECTORY_BREAK=owner-unrecorded` | 是 | 相同 | 相同 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=alertsignorepeaks` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=alertspathtext` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=anynvrm` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=autonodefault` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=broadcast-exit-swallowed` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=broadcast-lines-leak` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=broadcast-nested` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=broadcast-none` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=broadcast-unadmitted` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=broadcast-wake-any` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=broadcast-wake-ignored` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=broadcast-wake-unthrottled` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=check-reports-foreign-reserve` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=dailynoqueue` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=exitcode` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=fallback` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=gpuoomonsuccess` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=growthanysource` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=guard-off` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=hostlookupblocking` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=hostlookupjoinunbounded` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=hostlookuplockblocking` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=hostlookupnocatch` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=hostlookupnorecord` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=hostlookupnotimeout` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=hostlookupunthrottled` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=ignorequeuebycap` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=ignoretable` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=journalasclean` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=keepstale` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=lastpeakonly` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nestedbyenvonly` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nestedinouterscope` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nestedlargercap` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nestedqueues` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nocap` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nodefault` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nogpuoom` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nogrowthalert` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nohostlookup` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=noledger` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nolock` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nooomadjust` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nooomfirst` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nooutcomealert` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nopeakmin` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nopredict` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=noprivate` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=noresult` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=noslice` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=noslicehit` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=noslicemark` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=notimelimit` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=oomfirstalerts` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=overpeakreject` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=peakqueue` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=queuebaseline` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=queuehopeless` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=recordkilledpeak` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=reject-foreign-reserve` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=sessiononejob` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=slicefallback` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=temporarycallerdeleted` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=temporarycallerreplaced` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=temporarykeepignored` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=temporaryleft` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=temporaryshared` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=term-leaves-scope` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=threadsdefaultunset` | 否 | 没跑 | 没跑 |
| `scripts/hosts/run-with-memory-cap.sh` | `RUN_WITH_MEMORY_CAP_BREAK=wholejournal` | 否 | 没跑 | 没跑 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=check-counts-index` | 否 | 相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=dangling-registry-unreported` | 否 | 不相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=exclusion-colon-as-path` | 否 | 相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=flat-registries-unread` | 否 | 不相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=gate-directory-default-only` | 否 | 相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=gitlinks-kept` | 否 | 不相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=glob-range-protects` | 否 | 不相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=history-carriers-ignored` | 否 | 不相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=mutation-replacement-counted` | 否 | 相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=registry-check-skipped` | 否 | 不相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=rerun-crashes` | 否 | 相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=rewrite-basename-only` | 否 | 不相同 | 相同 |
| `scripts/kb/archive-past-rounds.py` | `ARCHIVE_PAST_ROUNDS_BREAK=still-input-by-basename` | 否 | 相同 | 相同 |
| `scripts/kb/batch-target.py` | `BATCH_TARGET_BREAK=match-whole-file` | 是 | 不相同 | 相同 |
| `scripts/kb/batch-target.py` | `BATCH_TARGET_BREAK=real-index` | 是 | 相同 | 相同 |
| `scripts/kb/batch-target.py` | `BATCH_TARGET_BREAK=whole-worktree` | 是 | 不相同 | 相同 |
| `scripts/kb/claim-experiment.sh` | `CLAIM_EXPERIMENT_BREAK=registry-ignored` | 是 | 相同 | 相同 |
| `scripts/kb/claim-experiment.sh` | `CLAIM_EXPERIMENT_BREAK=used-check-skipped` | 是 | 相同 | 相同 |
| `scripts/kb/claim-experiment.sh` | `CLAIM_EXPERIMENT_BREAK=worktrees-skipped` | 是 | 相同 | 相同 |
| `scripts/kb/doc-consolidate.py` | `DOC_CONSOLIDATE_BREAK=drop-number` | 是 | 相同 | 相同 |
| `scripts/kb/doc-consolidate.py` | `DOC_CONSOLIDATE_BREAK=history-heading-chinese-only` | 是 | 相同 | 相同 |
| `scripts/kb/doc-consolidate.py` | `DOC_CONSOLIDATE_BREAK=migrated-not-recognized` | 是 | 相同 | 相同 |
| `scripts/kb/history-consolidate.py` | `HISTORY_CONSOLIDATE_BREAK=drop-number` | 否 | 相同 | 相同 |
| `scripts/kb/history-consolidate.py` | `HISTORY_CONSOLIDATE_BREAK=english-quotes-unprotected` | 否 | 相同 | 相同 |
| `scripts/kb/history-consolidate.py` | `HISTORY_CONSOLIDATE_BREAK=git-show-ignores-path` | 否 | 不相同 | 相同 |
| `scripts/kb/insert-row.py` | `INSERT_ROW_BREAK=literal-as-regex` | 否 | 相同 | 相同 |
| `scripts/kb/insert-row.py` | `INSERT_ROW_BREAK=no-clock-guard` | 否 | 不相同 | 相同 |
| `scripts/kb/insert-row.py` | `INSERT_ROW_BREAK=no-session-guard` | 否 | 相同 | 相同 |
| `scripts/kb/insert-row.py` | `INSERT_ROW_BREAK=vague-anchor-hint` | 否 | 不相同 | 相同 |
| `scripts/kb/kb-sections.py` | `KB_SECTIONS_BREAK=raw-arguments` | 否 | 不相同 | 不相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=any-hits` | 是 | 相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=any-name` | 是 | 不相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=basis-unchecked` | 是 | 不相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=cross-spec-any-word` | 是 | 相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=cross-spec-unchecked` | 是 | 不相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=dated-time` | 是 | 相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=flat-parens` | 是 | 相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=no-history` | 是 | 相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=no-pairing` | 是 | 相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=no-pointers` | 是 | 相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=no-references` | 是 | 不相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=raw-arguments` | 是 | 不相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=translation-chinese-name-ok` | 是 | 相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=translation-fullwidth-ok` | 是 | 相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=translations-as-chinese` | 是 | 相同 | 相同 |
| `scripts/kb/kb-spec-check.py` | `KB_SPEC_CHECK_BREAK=translations-as-registry` | 是 | 不相同 | 相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=abort-on-conflict` | 否 | 不相同 | 不相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=content-lenient` | 否 | 不相同 | 不相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=copies-ignored` | 否 | 不相同 | 不相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=fill-skip-one` | 否 | 不相同 | 不相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=fill-touch-existing` | 否 | 不相同 | 不相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=fold-single-dates` | 否 | 不相同 | 不相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=no-recheck` | 否 | 不相同 | 不相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=ordinal-always` | 否 | 不相同 | 不相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=untouched-off` | 否 | 不相同 | 不相同 |
| `scripts/kb/migrate-changelog-format.py` | `MIGRATE_CHANGELOG_BREAK=write-order-file` | 否 | 不相同 | 不相同 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=file-form-ignored` | 否 | 没点名 | 没点名 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=filename-dates-counted` | 否 | 没点名 | 没点名 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=item-suffix-kept` | 否 | 没点名 | 没点名 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=paid-heading-optional` | 否 | 没点名 | 没点名 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=paid-rows-counted` | 否 | 没点名 | 没点名 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=status-ignored` | 否 | 没点名 | 没点名 |
| `scripts/kb/milestone-close-owed-check.py` | `MILESTONE_CLOSE_OWED_BREAK=whole-row-scanned` | 否 | 没点名 | 没点名 |
| `scripts/kb/quote-kb.py` | `QUOTE_KB_BREAK=bare-lacking-hint` | 否 | 不相同 | 相同 |
| `scripts/kb/quote-kb.py` | `QUOTE_KB_BREAK=checklist-reasons-unchecked` | 否 | 相同 | 相同 |
| `scripts/kb/quote-kb.py` | `QUOTE_KB_BREAK=colon-first` | 否 | 相同 | 相同 |
| `scripts/kb/quote-kb.py` | `QUOTE_KB_BREAK=fence-not-skipped` | 否 | 不相同 | 相同 |
| `scripts/kb/quote-kb.py` | `QUOTE_KB_BREAK=ignore-not-numbers` | 否 | 相同 | 相同 |
| `scripts/kb/quote-kb.py` | `QUOTE_KB_BREAK=invariants-to-readme` | 否 | 相同 | 相同 |
| `scripts/kb/quote-kb.py` | `QUOTE_KB_BREAK=kb-directories-raw` | 否 | 不相同 | 相同 |
| `scripts/kb/quote-kb.py` | `QUOTE_KB_BREAK=keepscratch` | 否 | 相同 | 相同 |
| `scripts/kb/quote-kb.py` | `QUOTE_KB_BREAK=no-readme-fallback` | 否 | 相同 | 相同 |
| `scripts/kb/quote-kb.py` | `QUOTE_KB_BREAK=not-numbers-in-fence` | 否 | 相同 | 相同 |
| `scripts/kb/replace-batch.py` | `LIB_FORBIDDEN_NOTATIONS_BREAK=clock-times` | 是 | 相同 | 相同 |
| `scripts/kb/replace-batch.py` | `LIB_FORBIDDEN_NOTATIONS_BREAK=session-names` | 是 | 相同 | 相同 |
| `scripts/kb/replace-batch.py` | `REPLACE_BATCH_BREAK=no-clock-guard` | 是 | 相同 | 相同 |
| `scripts/kb/replace-batch.py` | `REPLACE_BATCH_BREAK=no-session-guard` | 是 | 相同 | 相同 |
| `scripts/kb/replace-batch.py` | `REPLACE_BATCH_BREAK=stop-at-first` | 是 | 不相同 | 相同 |
| `scripts/kb/replace-once.py` | `REPLACE_ONCE_BREAK=no-clock-guard` | 否 | 相同 | 相同 |
| `scripts/kb/replace-once.py` | `REPLACE_ONCE_BREAK=no-nearest` | 否 | 不相同 | 不相同 |
| `scripts/kb/replace-once.py` | `REPLACE_ONCE_BREAK=no-session-guard` | 否 | 相同 | 相同 |
| `scripts/kb/stale-candidates.py` | `QUOTE_KB_BREAK=kb-directories-raw` | 是 | 相同 | 相同 |
| `scripts/kb/stale-candidates.py` | `STALE_CANDIDATES_BREAK=dirty-worktree-unchecked` | 是 | 相同 | 相同 |
| `scripts/kb/stale-candidates.py` | `STALE_CANDIDATES_BREAK=handover-unchecked` | 是 | 相同 | 相同 |
| `scripts/kb/stale-candidates.py` | `STALE_CANDIDATES_BREAK=invariants-carrier-missing` | 是 | 相同 | 相同 |
| `scripts/kb/stale-candidates.py` | `STALE_CANDIDATES_BREAK=provenance-unchecked` | 是 | 不相同 | 相同 |
| `scripts/kb/stale-candidates.py` | `STALE_CANDIDATES_BREAK=wide-unchecked` | 是 | 相同 | 相同 |
| `scripts/kb/sweep-term.py` | `SWEEP_TERM_BREAK=archived-names-unprotected` | 否 | 相同 | 相同 |
| `scripts/kb/sweep-term.py` | `SWEEP_TERM_BREAK=defer-value-as-root` | 否 | 相同 | 相同 |
| `scripts/kb/sweep-term.py` | `SWEEP_TERM_BREAK=header-as-row` | 否 | 相同 | 相同 |
| `scripts/kb/sweep-term.py` | `SWEEP_TERM_BREAK=ignored-files-walked` | 否 | 不相同 | 相同 |
| `scripts/kb/sweep-term.py` | `SWEEP_TERM_BREAK=line-exemption-covers-the-file` | 否 | 相同 | 相同 |
| `scripts/kb/sweep-term.py` | `SWEEP_TERM_BREAK=own-fixtures-swept` | 否 | 相同 | 相同 |
| `scripts/kb/sweep-term.py` | `SWEEP_TERM_BREAK=path-shaped-replaced` | 否 | 相同 | 相同 |
| `scripts/kb/sweep-term.py` | `SWEEP_TERM_BREAK=product-name-no-left-boundary` | 否 | 相同 | 相同 |
| `scripts/kb/sweep-term.py` | `SWEEP_TERM_BREAK=regex-assertions-dropped` | 否 | 相同 | 相同 |
| `scripts/kb/sweep-term.py` | `SWEEP_TERM_BREAK=slash-words-protected` | 否 | 相同 | 相同 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=accept-unknown-item` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=baseline-alerts-beside-peaks` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=baseline-any-threads` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=baseline-by-packages` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=baseline-refused-step-whole` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=break-guard-after-parsing` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=build-directory-in-home` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=carry-without-store-check` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=coverage-selection-ignored` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=cpu-limit-ignored` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=doc-tests-skipped` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=environmental-not-reselected` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=escalated-not-summarised` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=escalation-evidence-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=escalation-queues-by-peak` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=escalation-same-limit` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=escalation-skipped` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=exclusion-filter-dropped` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=gpu-all-items` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=gpu-also-when-cpu-caught` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=gpu-carry-ignored` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=gpu-mac-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=gpu-memory-as-driver-error` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=gpu-not-run-as-missed` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=gpu-quiesce-dropped` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=gpu-red-not-caught` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=gpu-test-threads-dropped` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=ignored-scan-skips-nested-modules` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=import-gpu-conflict-ignored` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=import-gpu-keys-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=import-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=import-verdicts-only` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=interrupted-as-done` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=interrupted-run-directory-kept` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=item-counts-dropped` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=judge-units-raw-text` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=limits-in-key` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=list-cache-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=list-items-pipe-unhandled` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=malformed-exclusion-row-skipped` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=mutation-steps-not-oom-first` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=named-items-always-rerun` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=no-carry-without-arguments` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=no-environment-check` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=no-row-flush` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=old-layout-accepted` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=own-libtest-failure-line` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=partial-row-kept` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=progress-never` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=read-only-volatile-refused` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=refused-carried` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=removed-arguments-accepted` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=rerun-done` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=result-mac-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=rocksdb-entry-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=run-directory-kept` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=run-directory-open-files-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=select-by-commit` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=slow-step-keeps-old-name` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=stage-two-baseline-red-generic` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=stage-two-full-cargo-test` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=stage-two-missed-as-caught` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=stage-two-not-by-binary` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=stage-two-without-stage-one` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=stale-exclusion-passes` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=state-is-last-row` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=summary-gate-silent` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=summary-mismatch-silent` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=term-leaves-step-running` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=test-variables-anywhere` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=unit-environment-fixed-list` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=unit-environment-toolchain-ignored` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=unknown-exit-stops` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=unregistered-ignored-passes` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=unviable-by-exit-code` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `BROAD_MUTATION_BREAK=whole-step-time-limit` | 否 | 没跑 | 没跑 |
| `scripts/mutation/broad-mutation.py` | `REUSE_CHECKS_BREAK=rejudged-sentinel-passes` | 否 | 没跑 | 没跑 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=duplicate-targets-accepted` | 否 | 没点名 | 没点名 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=missing-targets-accepted` | 否 | 没点名 | 没点名 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=non-core-rows-accepted` | 否 | 没点名 | 没点名 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=non-core-targets-required` | 否 | 没点名 | 没点名 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=target-column-unchecked` | 否 | 没点名 | 没点名 |
| `scripts/mutation/crates-mutation-light-rows.py` | `CRATES_MUTATION_LIGHT_ROWS_BREAK=unknown-rows-accepted` | 否 | 没点名 | 没点名 |
| `scripts/mutation/mutate.sh` | `LIBTEST_RESULT_LINES_BREAK=should-panic-in-name` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=anchorstopsall` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=bareusage` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=baselinelogdropped` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=baselinequeueasred` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=baselinesqueezedasred` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=calleesignored` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=compileassuccess` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=compilenontest` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=countblind` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=dependenciesbycrate` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=droprow` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=exportparallelreason` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=gpumemoryascaught` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=honorbreakoutsideselftest` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=includesunlinked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=libraryunread` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=memoryascaught` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=memorynotfail` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=no-cap-lints` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=nofeatures` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=noparallelreason` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=outertimeout` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=overridecallerreason` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=ownnameparse` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=queueasinvalid` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=rocksdb-entry-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=runtargetkept` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=sharedcopy` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=squeezedascrashed` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=swallowwrapper` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `REUSE_CHECKS_BREAK=rejudged-sentinel-passes` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutate.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nocap` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=anchor-broken-dispatched` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=break-honored-anywhere` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=environment-opens` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=gpu-memory-unexplained` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=import-skipped` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=no-final-pass` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=no-peer-cleanup-on-failure` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=peer-crash-library-home-unset` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=peer-library-kept` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=peer-llvm-tools-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=peer-rocksdb-unchecked` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=peer-without-memory-cap` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=selftest-tmpdir-outside` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=start-timeout-judged` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=store-not-fetched` | 否 | 没跑 | 没跑 |
| `scripts/mutation/mutation-shard-run.sh` | `MUTATION_SHARD_RUN_BREAK=switch-off-ignored` | 否 | 没跑 | 没跑 |
| `scripts/mutation/prove-red.sh` | `PROVE_RED_BREAK=build-jobs-six` | 否 | 不相同 | 相同 |
| `scripts/mutation/prove-red.sh` | `PROVE_RED_BREAK=caller-target-overridden` | 否 | 不相同 | 相同 |
| `scripts/mutation/prove-red.sh` | `PROVE_RED_BREAK=second-argument-target` | 否 | 不相同 | 相同 |
| `scripts/mutation/prove-red.sh` | `PROVE_RED_BREAK=unbound-second-argument` | 否 | 不相同 | 相同 |
