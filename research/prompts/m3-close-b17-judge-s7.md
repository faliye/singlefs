# 批 17 阶段同步第二段：F19、F21、F22、F23 逐行判（2026-10-26）

## 结论

- 范围：`--base 841b3beabb98`，结束提交对象 `1e3079d7a07304ceaf5a9de84f080853d61f52b5`（`git cat-file -t` 得 commit，还在）；候选表 `/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-facts/candidates.tsv`（sha256 `2e508f2bf76f47f66a2fea7300f53e2e1ec2b7a4fa71fa539b38d50836a94ee4`，与派发提示相同）。分到的四组 158 行逐行判完：F19 68、F21 24、F22 19、F23 47。
- 判定计数（`awk -F' \\| ' '{print $3}' table-*.md | sort | uniq -c`）：要改 10、事件句不改 60、不相干 88；要补 0、要人看 0。要改的 10 行全在 F19：
  - E142 页第 22、310 行：把装置那一版的 16777240 / 16777260 说成门禁现在枚举的、「今天的」新池新建文件流；结束那一版 `crash_enumeration_new_pool_file_creation_stream.rs` 第 757、761 行钉 16_777_243 / 16_777_268。
  - E184 页第 245 行（岔路表「还差什么」）：「候选甲追不追原登记第十二节由主 agent 定」已由调度记录第 875 行定为「不追，乙已成」。
  - E184 页第 577 行（路径与结论登记第 6 行）：只写了旧段形算 16842782，表下第 579 行已比 16842785 / 16777243。
  - 布局 01 第八节那段说明（中 406、英 405、日 402）：「取号、暖机、普通发布三行与整条流那一句同产物第 36–39 行逐字相同」不成立（产物第 36、39 行是取号写一轮的 `2`、16777240，表里已是 `2+2`、16777243）。
  - 里程碑 01 步 5 现状（中 161、英 134、日 134）：世代号、验收段序列与闭式、取号定义句都是取号写一轮那一版（验收用例第 379、415、502 行已是两轮）。
- 这几处在基准那一版已经过时（取号写两轮在基准之前的提交 0b926bd3f 就进了代码），本批的 E184 改动只把 E184 自己的页与索引跟上了。
- 推翻条件：结束那一版 `crash_enumeration_new_pool_file_creation_stream.rs` 不钉 16_777_243 / 16_777_268，或 `new_pool_file_creation_publish.rs` 不钉 `(6, "2+2", 7)` 与 61 步 16_777_243，十行要改就不成立；调度记录第 875 行不是主 agent 定「候选甲不追」，第 245 行那一行就该判不相干。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F19 | .claude/kb/experiments.en.md:28 | 不相干 | 已是新写法：索引行写「the new numbers of anchors A1, A2, A5, A7 and X3 are all settled」「now records 1358954642」「range S contains 16 pinned smoke streams」，与第 2 次重跑的新数一致 |
| F19 | .claude/kb/experiments.ja.md:28 | 不相干 | 已是新写法：索引行写「アンカーA1、A2、A5、A7とX3の新値はすべて確定」「現在の記録は1358954642」「区間Sは固定済みスモーク16本」，与第 2 次重跑的新数一致 |
| F19 | .claude/kb/experiments.md:208 | 不相干 | 已是新写法：索引行写「锚 A1、A2、A5、A7 与 X3 的新数全部定下（建快照流全量 16842815 等）」「现录 1358954642」「区间 S 含 16 条钉过的冒烟流」，与第 2 次重跑一致 |
| F19 | .claude/kb/experiments/142-新池新建文件的干跑.md:22 | 要改 | 改成：崩溃点重放主臂本页装置不枚举（`name=layer0 … states=not_run closed_form=16777240`：本页装置还是取号写一轮那一版的流，两态闭式 16777240，补第三态后全量 16777260；`crates/` 那条流取号写两轮之后两态 16777243、全量 16777268，全量由 `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` 在门禁 checker-tier-crash-point-replay 里枚举）。理由：原句把「补第三态后全量 16777260」与「全量由 … 在门禁里枚举」连成一句，结束那一版该测试钉 `FULL_STATES = 16_777_268`、两态 16_777_243（第 757、761 行），门禁枚举的不是 16777260 |
| F19 | .claude/kb/experiments/142-新池新建文件的干跑.md:103 | 事件句不改 | 代码围栏里整行抄的第十九次跑产物行「E7RESULT name=segments path=post_mkfs_stream operations=57」，说的是那一份产物，不改 |
| F19 | .claude/kb/experiments/142-新池新建文件的干跑.md:109 | 事件句不改 | 代码围栏里整行抄的产物行「E7RESULT name=layer0 arm=settled_two_devices」，closed_form=16777240 是那一份产物的读数，不改 |
| F19 | .claude/kb/experiments/142-新池新建文件的干跑.md:190 | 不相干 | 第十九次跑判决表第 4 行「这一格量的是第十七次跑那条流」，16777240 是本页装置第十九次跑那条流的闭式（装置 `e142_new_pool_file_creation_dry_run.rs` 仍钉 16_777_240），「这条流还没枚举过」说的是本页装置，取号写两轮改的是 `crates/` 那条流，不受这件事实影响 |
| F19 | .claude/kb/experiments/142-新池新建文件的干跑.md:215 | 事件句不改 | 代码围栏里整行抄的产物行「E7RESULT name=segments path=post_mkfs_stream operations=57」（第十九次跑的段序列小节），说的是那一份产物，不改 |
| F19 | .claude/kb/experiments/142-新池新建文件的干跑.md:225 | 事件句不改 | 代码围栏里整行抄的产物行「E7RESULT name=segments_three_state path=post_mkfs_stream」，16777240 / 16777260 是那一份产物的读数，不改 |
| F19 | .claude/kb/experiments/142-新池新建文件的干跑.md:310 | 要改 | 把「今天的新池新建文件流是 12 单元、两态闭式 16777240、补第三态全量 16777260，本页装置不枚举它」改成：今天 `crates/` 的新池新建文件流是 12 单元、取号写两轮之后两态闭式 16777243、补第三态全量 16777268（本页装置的模型仍是取号写一轮那一版，16777240 / 16777260），本页装置不枚举它。理由：「今天」是现状句，结束那一版 `crash_enumeration_new_pool_file_creation_stream.rs` 第 757、761 行钉 16_777_243 / 16_777_268 |
| F19 | .claude/kb/experiments/142-新池新建文件的干跑.md:316 | 不相干 | 「第十九次跑：与 `crates/` 逐字比只在主几何一个取样点上量过」这一条说的是本页装置第十九次跑那条流，「整条流 16777240 个两态状态还没枚举过」指本页装置的模型（仍是取号写一轮），不说 `crates/` 现在的流，不受这件事实影响 |
| F19 | .claude/kb/experiments/167-多在飞发布的崩溃枚举切法.md:24 | 事件句不改 | 「第一段做了什么」里的「停机条款 S1（登记表两条）、S2（发布归属手推）在产物之前用 `halt` 子命令真跑」，16777240 是那一次 S1 核对的数，说的是那一次 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:13 | 不相干 | 已是新写法：现状第一条「第 2 次重跑（2026-10-24，重跑登记」写的就是新数 16842785、16777243、16842815、1358954642 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:15 | 事件句不改 | 这一条在现状里「第 1 次跑的产物（2026-10-19，取号写一轮那一版的树；本条起到「单测 82 个」那一条，读数都是这份产物的）」的范围内，「区间 S 取 14 条钉过的冒烟流」「下界 16777240」是那一份产物的读数，新区间在第 13 行另写 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:18 | 不相干 | 已是新写法：「条款这一格：锚 B1」一条已并写第一段产物的 1358954634 与取号写两轮之后现录 1358954642、`match=false` |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:21 | 事件句不改 | 「上游写序变化（取号写两轮：mkfs 之后多一个 2 写的系统配置槽段）之后只修钉值」是 2026-10-23 那一次的事；其中现状分句「X3 写死的 16842807 … 没改，登记的新值是 16842815，第三段开跑前要改」在结束那一版仍真（装置常量仍是 16_842_807） |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:31 | 事件句不改 | 「做了什么」第 6 条「2026-10-23 只修钉值」，说的是那一次怎么改的钉值，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:145 | 事件句不改 | 「第 2 次重跑」一节的锚表行「A1 建快照流两态」，写的是那一趟的改前改后与产物行号，已是新数 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:146 | 事件句不改 | 「第 2 次重跑」一节的锚表行「A2 新池两态」，16777240 → 16777243 是那一趟的改前改后 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:150 | 事件句不改 | 「第 2 次重跑」一节的锚表行「A5 建快照流全量」，16842807 → 16842815 是那一趟的改前改后 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:151 | 事件句不改 | 「第 2 次重跑」一节的锚表行「A6 多记录流全量（附带）」，1358954634 → 1358954642 是那一趟的改前改后 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:152 | 事件句不改 | 「第 2 次重跑」一节的锚表行「A7 的后半与 X3 小项」，16842807 → 16842815 是那一趟的改前改后 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:153 | 事件句不改 | 「第 2 次重跑」一节的锚表行「B1 条款」，现录 1358954634 → 1358954642 是那一趟的读数 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:156 | 不相干 | 已是新写法：「X3 的新值：小项 `states` 不等于 16842815 即作废」，装置常量「这一趟没改，仍是 16842807」「第三段开跑之前要改成 16842815」在结束那一版仍真（装置常量仍是 16_842_807） |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:183 | 事件句不改 | 「占比在动，词没动：区间 S 变了。改前 S 下界 16777240」写的是第 2 次重跑那一趟的改前改后对比 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:229 | 事件句不改 | 代码围栏里整行抄的第 2 次重跑产物行「77:E7RESULT name=anchor identifier=multi_record_full_named_by_the_clause」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:236 | 事件句不改 | 「第 2 次重跑」一节的没过字段表那一行「`match=false`（B1）与 `clause_anchor_matches=false`」，1358954642 是那一趟产物的读数 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:245 | 要改 | 「还差什么」那一格改成：X3 新值 16842815 写进装置常量与两条手造读数用例（第三段开跑之前做，延后登在重跑登记 `research/prompts/e184-r2-prereg.md` 第一节「未定项」那一行；谁改由主 agent 定）；候选甲（往原登记第十二节追加）不追，乙已成。理由：原格写「候选甲追不追原登记第十二节由主 agent 定」，结束那一版调度记录第 835、875 行记主 agent 已定「候选甲（往原登记第十二节追加）不追，乙已成」与「X3 进装置的时点：第三段开跑之前」 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:253 | 不相干 | 已是新写法：「X3 在新段形下是否真按 16842815 判、A7 前半，要等第三段的 J2 冷跑」在结束那一版仍真（第三段没跑） |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:333 | 事件句不改 | 「读数：态数区间重叠与共用块键」一节读的是 2026-10-19 那份产物（第 273 行点名产物），「区间 S 是冒烟层 14 条已钉流的两态闭式」「下界 16777240」是那份产物的读数；新区间写在第 13、183 行 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:370 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「927:E7RESULT name=overlap scope=family:overwrite_chain_main」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:372 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「929:E7RESULT name=overlap scope=family:random_history_main」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:374 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「931:E7RESULT name=overlap scope=group:overwrite_chain-01」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:376 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「933:E7RESULT name=overlap scope=group:overwrite_chain-04」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:378 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「935:E7RESULT name=overlap scope=group:overwrite_chain-16」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:380 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「937:E7RESULT name=overlap scope=group:random_history-broad-F」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:382 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「939:E7RESULT name=overlap scope=group:random_history-broad-F」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:384 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「941:E7RESULT name=overlap scope=group:random_history-broad-F」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:386 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「943:E7RESULT name=overlap scope=group:random_history-broad-F」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:388 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「945:E7RESULT name=overlap scope=group:random_history-broad-U」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:390 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「947:E7RESULT name=overlap scope=group:random_history-broad_w」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:441 | 事件句不改 | 代码围栏里整行抄的 2026-10-19 产物行「68:E7RESULT name=anchor id=B1_multi_record_full_named_by_the」，1358954634 是那一份的读数 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:447 | 事件句不改 | 代码围栏里整行抄的留存产物 selfcheck-2026-10-19 行「E7RESULT name=anchor identifier=multi_record_full_named_by_t」，不改 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:454 | 事件句不改 | 「判决行与没过的字段」一节是 2026-10-19 产物的判决，「`match=false`（B1）与 `b1_matches=false`」那一行的 1358954634 是那份产物第 68 行的读数 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:501 | 事件句不改 | 「作废、失败、停机条款逐条（第一段）」表里「F6 条款这一格可能错」引的是 2026-10-19 产物第 68 行的 1358954634；第 2 次重跑的同一格另写在第 236 行 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:577 | 要改 | 第三格改成：按段形手写 python 一行算两态闭式：取号写一轮那一版 `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2]` 得 A1 的 16842782（登记第十三节第 13.4 条第 3 点）；取号写两轮之后 `[2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2]` 得 A1 的 16842785、前 12 段得 A2 的 16777243（「做了什么」第 6 条）；第四格改成：无实现：两次都是 `python3 -c` 一行，没有存进仓的脚本。理由：原行「手写 python 一行算 A1 的 16842782」只写了旧段形，表下第 579 行已改成路径 6 对 1 比的是 16842785、16777243 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:579 | 不相干 | 已是新写法：「跨路径的比对断言钉在源码和产物里」那一段已写「建快照流两态闭式 16842785 与新池新建文件流两态闭式 16777243 两个锚」并注明 10-19 产物里是 16842782、16777240 |
| F19 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:585 | 不相干 | 已是新写法：「D13（验证路线） 已定项 9 … 备料 … 2026-10-24 不受影响」那一行写 B1 现录 1358954642，与第 2 次重跑一致 |
| F19 | .claude/kb/layout/01-first-txn.en.md:405 | 要改 | Replace "acquiring number, warm-up, regular publish three rows and entire stream sentence verbatim identical to artifact lines 36–39" with: "warm-up and regular publish rows verbatim identical to artifact lines 37–38; the acquiring-number row and the entire stream sentence are the shape after number acquisition writes two rounds (acquisition `2+2`, 6 operations, 7 crash states; entire stream `2+2+2+1+2+2+1+2+24+2+1+2`, 61 operations, 16777243), pinned by `recorded_paths_match_the_registered_segment_sequences` in `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs`, while artifact lines 36 and 39 still carry the one-round `2`, `2+2+1+2+2+1+2+24+2+1+2`, 16777240 (the E142 rig was not rerun after two-round acquisition)"; and replace the last clause "state count in entire stream sentence calculated in closed form, not enumerated" with: "the 16777243 in the entire stream sentence is the two-state closed form pinned by the crates test; the three-state full count 16777268 is enumerated by `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` in gate checker-tier-crash-point-replay". 照中文第 406 行同一改法（理由见 F19 那一行） |
| F19 | .claude/kb/layout/01-first-txn.en.md:417 | 不相干 | 已是新写法：「closed form 16777243 (pinned by `the_formatted_pool_mount_and_first_file_stream_keeps_the_new_pool_file_creation_segment_sequence`」，与取号写两轮之后一致 |
| F19 | .claude/kb/layout/01-first-txn.en.md:419 | 不相干 | 已是新写法：「New-pool file creation stream is entire stream `2+2+2+1+2+2+1+2+24+2+1+2`, 16777243 states」，并注明产物那一行是取号写一轮时的 16777260 |
| F19 | .claude/kb/layout/01-first-txn.ja.md:402 | 要改 | 「発番、暖機、通常パブリッシュの 3 行およびストリーム全体の文は成果物第 36–39 行と逐語的に同一」を次に置き換え：「暖機、通常パブリッシュの 2 行は成果物第 37–38 行と逐語的に同一；発番の行とストリーム全体の文は発番書き込み 2 ラウンド化後の形（発番 `2+2`、6 回操作、7 クラッシュ状態；ストリーム全体 `2+2+2+1+2+2+1+2+24+2+1+2`、61 回操作、16777243）で、`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` の `recorded_paths_match_the_registered_segment_sequences` が固定する；成果物第 36、39 行は発番 1 ラウンド時の `2`、`2+2+1+2+2+1+2+24+2+1+2`、16777240 のまま（E142 装置は 2 ラウンド化後に再実行していない）」。末尾「ストリーム全体の文の状態数は閉形式で計算されたものであり、列挙されたものではない」は「ストリーム全体の文の 16777243 は crates のテストが固定する 2 状態閉形式で、3 状態の全量 16777268 は `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` がゲート checker-tier-crash-point-replay で列挙する」に。中文第 406 行同一改法 |
| F19 | .claude/kb/layout/01-first-txn.ja.md:414 | 不相干 | 已是新写法：「ストリーム全体で `2+2+2+1+2+2+1+2+24+2+1+2`」「閉形式 16777243」，与取号写两轮之后一致 |
| F19 | .claude/kb/layout/01-first-txn.ja.md:416 | 不相干 | 已是新写法：「新規プールでのファイル作成ストリームはストリーム全体で `2+2+2+1+2+2+1+2+24+2+1+2`、16777243 状態」，与取号写两轮之后一致 |
| F19 | .claude/kb/layout/01-first-txn.md:406 | 要改 | 把「取号、暖机、普通发布三行与整条流那一句同产物第 36–39 行逐字相同」改成：暖机、普通发布两行同产物第 37、38 行逐字相同；取号那一行与整条流那一句是取号写两轮之后的形状（取号 `2+2`、6 次操作、7 个崩溃状态；整条流 `2+2+2+1+2+2+1+2+24+2+1+2`、61 次操作、16777243），由 `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` 的 `recorded_paths_match_the_registered_segment_sequences` 钉住，产物第 36、39 行仍是取号写一轮的 `2`、`2+2+1+2+2+1+2+24+2+1+2`、16777240（E142（新池新建文件的干跑） 装置没按取号写两轮重跑）；末句「整条流那一句的状态数是闭式算出来的，不是枚举出来的」改成：整条流那一句的 16777243 是 `crates/` 测试钉的两态闭式，取三态的全量 16777268 由 `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` 在门禁 checker-tier-crash-point-replay 里枚举。理由：结束那一版产物 `e142-…-r25-main-2026-10-13.out` 第 36、39 行是 `segments=2`、`2+2+1+…` `closed_form=16777240`，表里取号行与第 420 行已是 `2+2`、16777243，「逐字相同」不成立 |
| F19 | .claude/kb/layout/01-first-txn.md:418 | 不相干 | 已是新写法：「整条流 `2+2+2+1+2+2+1+2+24+2+1+2`，与新池新建文件那条流逐段相同、闭式 16777243」，与取号写两轮之后一致 |
| F19 | .claude/kb/layout/01-first-txn.md:420 | 不相干 | 已是新写法：「新池新建文件那条流是整条流 `2+2+2+1+2+2+1+2+24+2+1+2`、16777243 个状态」，并注明产物那一行是取号写一轮时的 16777260 |
| F19 | .claude/kb/milestone/01-first-txn.en.md:134 | 要改 | Three replacements, same as the Chinese line 161. (1) "Instance acquisition writes generation 2 slot 0, warm-up txg 1 / 2 writes generations 3 / 4, transaction txg 3 writes generation 5 slot 1" becomes "Instance acquisition writes generations 2 (slot 0) and 3 (slot 1) in two rounds, warm-up txg 1 / 2 writes generations 4 / 5, transaction txg 3 writes generation 6 slot 0". (2) "match verbatim with Section 8 and E142 (Dry Run of New File in New Pool) run 8 artifacts (…closed form 16777240)" becomes "match verbatim with Section 8 (acquisition `2+2`, warm-up `2+1+2+2+1+2`, transaction `24+2+1+2`, entire sequence after mkfs `2+2+2+1+2+2+1+2+24+2+1+2`, closed form 16777243; the E142 run 8 artifact is the one-round shape, acquisition `2`, entire sequence `2+2+1+2+2+1+2+24+2+1+2`, closed form 16777240, and the rig was not rerun after two-round acquisition)". (3) "Instance acquisition = one system configuration slot write per disk (…) + one barrier" and "on initial mount path, that barrier is the opening barrier of warm-up (write entry does not issue second time)" become "Instance acquisition = two rounds, each round one system configuration slot write per disk (generation = max validated slot on that disk + 1) followed by one pool barrier; on the initial mount the second-round barrier and the opening barrier of warm-up are back to back and merged into one", and append "; after two-round acquisition 12 segments, 16777243" to the parenthesis "new-pool file creation stream has 11 segments, closed form 16,777,240" |
| F19 | .claude/kb/milestone/01-first-txn.ja.md:134 | 要改 | 中文第 161 行同一改法，三処。① 「番号取得は世代 2 スロット 0、ウォームアップ txg 1 / 2 は世代 3 / 4、トランザクション txg 3 は世代 5 スロット 1」→「番号取得は 2 ラウンドで世代 2（スロット 0）、3（スロット 1）、ウォームアップ txg 1 / 2 は世代 4 / 5、トランザクション txg 3 は世代 6 スロット 0」。② 「8 節および E142 (新規プール新規ファイルのドライラン) 第 8 回実行成果物と完全に一致（番号取得 `2`、…閉形式 16777240）」→「8 節と完全に一致（番号取得 `2+2`、ウォームアップ `2+1+2+2+1+2`、トランザクション `24+2+1+2`、mkfs 後全体 `2+2+2+1+2+2+1+2+24+2+1+2`、閉形式 16777243；E142 第 8 回実行成果物は発番 1 ラウンド時の `2`、`2+2+1+2+2+1+2+24+2+1+2`、16777240 で、装置は 2 ラウンド化後に再実行していない）」。③ 「番号取得 = ディスクごとにシステム構成スロット書き込み 1 回（…）+ バリア 1 回」「初回マウント経路上のバリアはウォームアップ冒頭のバリア（書き込みエントリは 2 回目を発行しない）」→「番号取得 = 2 ラウンド、各ラウンドでディスクごとにシステム構成スロット書き込み 1 回（世代番号はそのディスクで自己検証済みスロットの最大値 + 1）、各ラウンド後にプールバリア 1 回；初回マウントでは第 2 ラウンド後のバリアとウォームアップ冒頭のバリアが連続し 1 つにマージされる」、括弧内「11 セグメント、閉形式 16 777 240」の後に「；発番 2 ラウンド化後は 12 セグメント、16777243」を追加 |
| F19 | .claude/kb/milestone/01-first-txn.md:161 | 要改 | 三处改。① 把「取号写世代 2 槽 0、暖机 txg 1 / 2 写世代 3 / 4、事务 txg 3 写世代 5 槽 1」改成：取号两轮写世代 2（槽 0）、3（槽 1），暖机 txg 1 / 2 写世代 4 / 5、事务 txg 3 写世代 6 槽 0。② 把「与八那一节及 E142（新池新建文件的干跑） 第八次跑产物逐字相同（取号 `2`、暖机 `2+1+2+2+1+2`、事务 `24+2+1+2`、mkfs 之后整条 `2+2+1+2+2+1+2+24+2+1+2`、闭式 16777240）」改成：与八那一节逐字相同（取号 `2+2`、暖机 `2+1+2+2+1+2`、事务 `24+2+1+2`、mkfs 之后整条 `2+2+2+1+2+2+1+2+24+2+1+2`、闭式 16777243；E142（新池新建文件的干跑） 第八次跑产物是取号写一轮那一版，取号 `2`、整条 `2+2+1+2+2+1+2+24+2+1+2`、闭式 16777240，装置没按取号写两轮重跑）。③ 把「取号 = 逐盘一次系统配置槽写（世代号按那块盘自证过的槽里最大的 + 1）+ 一道屏障」「首次挂载路径上那道屏障就是暖机开场那道（写入口不再发第二次）」改成：取号 = 两轮，每轮逐盘一次系统配置槽写（世代号按那块盘自证过的槽里最大的 + 1）、每轮之后一道池屏障；首次挂载上第二轮那道与暖机开场那道背靠背、并成一道（布局 01 第八节取号那一行），括注「新池新建文件流是 11 段、闭式 16 777 240」后补「；取号写两轮之后 12 段、16777243」。理由：「现状（2026-09-14 开工，验收六条全过）」是现状句；结束那一版验收用例 `new_pool_file_creation_publish.rs` 第 379、415 行钉 `(6, "2+2", 7)` 与 61 步、16_777_243，第 502 行注释「取号两轮写世代 2（槽 0）、3（槽 1），暖机写 4 / 5，事务写 6 槽 0」 |
| F19 | records/2026-09-24-里程碑二收尾调度.md:282 | 事件句不改 | 调度记录 09-24「C577 交回、与 A3b 一起合入（2026-09-27）」那一行，「新池新建文件流 67108885 → 16777240」是那一次盘点的数 |
| F19 | records/2026-10-13-复用键与全仓改名.md:940 | 事件句不改 | 调度记录「监控第 32 轮（重起后，量的）」，states=1358954634 是那一轮监控读到的数 |
| F19 | records/2026-10-13-里程碑四预开发.md:168 | 事件句不改 | 调度记录「快照崩溃流交回（Opus，54 分，52.8 万 token）」，「全量 16842807 态」是那一次交回时按真写表数的 |
| F19 | records/2026-10-18-里程碑三收尾与近期欠账.md:232 | 事件句不改 | 调度记录「E184 第六位（报告」那一条记的是 E184 第一段交回的读数，「重录 1358954634」是那一次的数 |
| F19 | records/2026-10-18-里程碑三收尾与近期欠账.md:835 | 事件句不改 | 调度记录「X3 进装置的时点：第三段开跑之前改」是主 agent 那一次认下的处置，事件句 |
| F19 | records/2026-10-18-里程碑三收尾与近期欠账.md:874 | 事件句不改 | 调度记录「E184 第 2 次重跑收尾交回」那一条，新数是那一次交回的内容 |
| F19 | records/2026-10-18-里程碑三收尾与近期欠账.md:875 | 事件句不改 | 调度记录「E184 收尾留给主 agent 定的：X3 新值 16842815」，是那一次主 agent 的定案 |
| F19 | records/2026-10-18-里程碑三收尾与近期欠账.md:891 | 事件句不改 | 调度记录「E184 英日两份那一块：判法只认去掉首尾空白后以」那一条，记的是那一次门禁红的处置 |
| F21 | .claude/kb/checks-owed.en.md:30 | 不相干 | 已是新写法：C911 那一行「torn image tables are counted in one place only, and direct table builds go unguarded」，记的是改完之后还剩的口子（取样判段一处计数），与开线程之前建一份的新事实一致 |
| F21 | .claude/kb/checks-owed.en.md:39 | 不相干 | C16「管道声明失败模型例外」那一行只把 `WritesWithTornImages` 造原地覆写的撕裂镜像当作现有覆盖举例，不说撕裂表按段还是按阶段建，不受这件事实影响 |
| F21 | .claude/kb/checks-owed.ja.md:28 | 不相干 | C16「管道声明失败模型例外」那一行只把 `WritesWithTornImages` 造撕裂镜像当作现有覆盖举例，不说撕裂表怎么建，不受这件事实影响 |
| F21 | .claude/kb/checks-owed.ja.md:587 | 不相干 | 已是新写法：C911「断裂イメージ表の作成数は一箇所でしか数えず」，记的是改完之后还剩的口子 |
| F21 | .claude/kb/checks-owed.md:27 | 不相干 | C16「管道声明失败模型例外」那一行只把 `WritesWithTornImages` 造原地覆写的撕裂镜像当作现有覆盖举例，不说撕裂表按段还是按阶段建，不受这件事实影响 |
| F21 | .claude/kb/checks-owed.md:585 | 不相干 | 已是新写法：C911「撕裂表建表计数只在一处，直调建表的管不住」，记的是开线程之前建一份之后还剩的口子 |
| F21 | .claude/kb/experiments/142-新池新建文件的干跑.md:316 | 不相干 | E142「它答不了的」第 15 条「三态只算闭式，不枚举、不生成撕裂镜像」说的是本页装置，不是 checker 档判段怎么建撕裂表 |
| F21 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:422 | 不相干 | E161 路径登记第 1 行「装置自己的枚举计划」说的是 E161 装置源文件自己的撕裂镜像表，不是 checker 档取样判段的撕裂表 |
| F21 | .claude/kb/experiments/78-重放的起点.md:22 | 不相干 | E78 表头「健康镜像 撕裂镜像（记录在、单元没落）」是重放起点那个计数模型的列名，与撕裂表怎么建无关 |
| F21 | records/2026-09-24-里程碑二收尾调度.md:220 | 不相干 | 调度记录「实审 B3a-2 交回（2026-09-27，暂停期间）」讲撕裂镜像真生成与快档数，说的是那一次交回，不涉及按段新建撕裂表 |
| F21 | records/2026-09-24-里程碑二收尾调度.md:241 | 不相干 | 调度记录「攻方腿交回：Z3-A、Z1Q-B、Z1Q-A 三处打中」里「40312 个撕裂镜像换撕裂点不改恢复结局」，与撕裂表怎么建无关 |
| F21 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:54 | 不相干 | 调度记录「复用验证过：同库同代码重跑一个状态都没再核」那一行提到崩溃事实里的撕裂镜像，不是撕裂表按段新建这件事 |
| F21 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:56 | 不相干 | 调度记录「用户定 GPU 判器现在做」那一行列崩溃事实的抽取项（三态写、撕裂镜像与重放），不是撕裂表按段新建这件事 |
| F21 | records/2026-09-30-里程碑三开工第一轮.md:1013 | 不相干 | 调度记录「J4 二分（E5 副本」那一条提到「未计撕裂镜像与重放」的随机历史封顶，与撕裂表怎么建无关 |
| F21 | records/2026-10-13-复用键与全仓改名.md:971 | 不相干 | 调度记录「第 12 条红的调查交回（2026-10-18」讲 GPU 判表容量与撕裂镜像占掩码位，不是 CPU 判段每段新建撕裂表 |
| F21 | records/2026-10-13-复用键与全仓改名.md:1013 | 不相干 | 调度记录表行「实现员乙：判表容量（给定状态路不补从不落的撕裂镜像」，说的是判表容量那一件，与撕裂表按段新建无关 |
| F21 | records/2026-10-13-复用键与全仓改名.md:1022 | 不相干 | 调度记录「乙的证红第一趟（`scratchpad/capacity-prove-red.log`）」提到撕裂镜像那行变异的 cargo 参数，与撕裂表怎么建无关 |
| F21 | records/2026-10-13-复用键与全仓改名.md:1045 | 不相干 | 调度记录「后 8 条第二趟第 1 轮监控（量的）」推测撕裂镜像 `Appended` / `LeftOut` 两路块键不一致，与撕裂表按段新建无关 |
| F21 | records/2026-10-18-里程碑三收尾与近期欠账.md:736 | 事件句不改 | 调度记录「根因：CPU 判段每条线程每领一段，就在」那一条是那一次调查的根因结论，说的是那一次 |
| F21 | records/2026-10-18-里程碑三收尾与近期欠账.md:740 | 事件句不改 | 调度记录「B（撕裂镜像按扇区稀疏存）是设计判断，进收拢表」是那一次的处置，之后收拢表第 900 行判「不做」另有记 |
| F21 | records/2026-10-18-里程碑三收尾与近期欠账.md:762 | 事件句不改 | 调度记录「主 agent 定 front_hole 照 C773 定案改全绿。派 implementation-writer」是那一次派发，事件句 |
| F21 | records/2026-10-18-里程碑三收尾与近期欠账.md:779 | 事件句不改 | 调度记录「mkfs 撕裂表在开线程之前建一份、各线程借用，断言恰好建一份」是修红第二批交回的内容，事件句 |
| F21 | records/2026-10-18-里程碑三收尾与近期欠账.md:900 | 事件句不改 | 调度记录「不做：10（撕裂镜像按扇区稀疏存；改法 A 已让那一格在登记上限内跑完」是收拢表那一次的去向判定 |
| F21 | records/2026-10-18-里程碑三收尾与近期欠账.md:906 | 事件句不改 | 调度记录「收拢表登号交回」那一条记 C908–C912 登号，事件句 |
| F22 | .claude/kb/milestone/03-third-txn.en.md:113 | 不相干 | 五·7「Pinned values of known reds left from Milestone 2 closure」点名的 findings_log 那条是 `red_states_are_deduplicated_by_signature…`（签名跨 10 个状态），不是快档态数 46 → 54 那一处钉值，不受这件事实影响 |
| F22 | .claude/kb/milestone/03-third-txn.ja.md:113 | 不相干 | 五·7「マイルストーン二収束時に残された既知の赤の固定値」点名的 findings_log 那条是签名去重用例，不是快档态数 46 → 54 那一处钉值 |
| F22 | .claude/kb/milestone/03-third-txn.md:113 | 不相干 | 五·7「里程碑二收尾留下的已知红钉值」点名的 findings_log 那条是 `red_states_are_deduplicated_by_signature_and_the_findings_log_is_readable_while_running`（签名跨 10 个状态），不是快档态数 46 → 54，不受这件事实影响 |
| F22 | records/2026-09-24-里程碑二收尾调度.md:256 | 事件句不改 | 调度记录「崩溃点重放发现日志（crates）交回并打上」，记的是那一次新建 findings_log 测试 |
| F22 | records/2026-09-24-里程碑二收尾调度.md:299 | 不相干 | 调度记录「合入后验证一交回（没做完）、主补丁合入」里 findings_log 是打不上的补丁名，与快档态数无关 |
| F22 | records/2026-09-27-验证两档拆分.md:41 | 事件句不改 | 调度记录「checker-tier-crash-point-replay 快档真跑红」那一行记 C577 之后 `QUICK_TIER_STATES = 54` 没改，是那一次的归因 |
| F22 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:38 | 事件句不改 | 调度记录「捶打：checker 档快档在 HEAD 上就有红」，「现在 46 态」是那一次的读数 |
| F22 | records/2026-09-30-里程碑三开工第一轮.md:594 | 事件句不改 | 调度记录「提前跑的 checker 档集成测试」列 findings_log 那一处红，是那一次的读数 |
| F22 | records/2026-10-01-对接上游测试脚本规范.md:664 | 事件句不改 | 调度记录「r2 冻结副本 checker 档库与集成测试」里 findings_log 三条红在钉值上，是那一次的归因 |
| F22 | records/2026-10-01-对接上游测试脚本规范.md:683 | 事件句不改 | 调度记录「收尾副本重取后带 `=commit` 跑 checker 档库与集成测试」里 findings_log 那 3 条红，是那一次的读数 |
| F22 | records/2026-10-13-代码欠账批.md:59 | 不相干 | 调度记录「2026-10-15 五·10 回扫交回」里 `findings_log_begin_line` 注释与拼法，与快档态数无关 |
| F22 | records/2026-10-13-复用键与全仓改名.md:86 | 不相干 | 调度记录「英文改名登记 clerk 交回」里 findings_log 测试漏改的序数名，与快档态数无关 |
| F22 | records/2026-10-13-复用键与全仓改名.md:801 | 不相干 | 调度记录「`crash_enumeration_findings_log_dedupes_by_signature_and_survives_re」那一条讲库里录入段那一块不在，与快档态数 46 → 54 无关 |
| F22 | records/2026-10-13-复用键与全仓改名.md:811 | 不相干 | 调度记录「第四轮 E 留下的两问，主 agent 定：三条发现日志用例都算空库起跑，照用一次性库」，与快档态数无关 |
| F22 | records/2026-10-18-里程碑三收尾与近期欠账.md:725 | 事件句不改 | 调度记录「ee 报 checker 档带 gpu、verdict-store 特性单跑」列 findings_log 三条红，是那一次的报告 |
| F22 | records/2026-10-18-里程碑三收尾与近期欠账.md:758 | 事件句不改 | 调度记录「findings_log 三条：HEAD 0b926bd3f「取号写两轮」，同类早先已认，46 → 54」是那一次的归因 |
| F22 | records/2026-10-18-里程碑三收尾与近期欠账.md:762 | 事件句不改 | 调度记录「主 agent 定 front_hole 照 C773 定案改全绿」那一条派发里的 findings_log 46 → 54，事件句 |
| F22 | records/2026-10-18-里程碑三收尾与近期欠账.md:782 | 事件句不改 | 调度记录「findings_log 46 → 54（按段长现算）」是修红第二批交回的内容 |
| F22 | records/2026-10-18-里程碑三收尾与近期欠账.md:814 | 事件句不改 | 调度记录「验证段第 2 步，三条轻型同时起」那一条记的是那一趟带特性跑过的范围 |
| F23 | .claude/kb/checks-owed.en.md:25 | 不相干 | C893「the local stream-1 segment-length table copied into the e170」那一行只拿这份装置的段长表行号举例，与第三条流一次性库无关 |
| F23 | .claude/kb/checks-owed.en.md:496 | 不相干 | C734「on_device_modes 的root ring表自己拼, 没走 core 的函数」那一行只拿这份装置里同形的比法举例，与一次性库无关 |
| F23 | .claude/kb/checks-owed.ja.md:485 | 不相干 | C734「on_device_modes 的ルートリング表自己拼」那一行只拿这份装置里同形的比法举例，与一次性库无关 |
| F23 | .claude/kb/checks-owed.ja.md:582 | 不相干 | C893「e170、e171 の装置にローカルに写したストリーム一の段長表」那一行只拿段长表行号举例，与一次性库无关 |
| F23 | .claude/kb/checks-owed.md:481 | 不相干 | C734「on_device_modes 的根环表自己拼、没走 core 的函数」那一行只拿这份装置里同形的比法举例，与一次性库无关 |
| F23 | .claude/kb/checks-owed.md:580 | 不相干 | C893「e170、e171 装置本地抄的流一段长表与今天的 crates 对不上」那一行只拿段长表行号举例，与第三条流一次性库无关 |
| F23 | .claude/kb/experiments/171-三oracle合谋测试.md:411 | 不相干 | E171 修订 6 那一趟的登记「登记 `research/prompts/e171-r1-prereg.md`（修订 6、7」只写装置路径与 sha，与内联用例改用一次性库无关 |
| F23 | .claude/kb/experiments/171-三oracle合谋测试.md:646 | 不相干 | E171「比对器 `scripts/experiments/e171_three_oracle_collusion_after_the_oracl」那一条讲比对器改法，与一次性库无关 |
| F23 | .claude/kb/experiments/171-三oracle合谋测试.md:647 | 不相干 | E171「单测 41 条」那一条讲研究一侧变异表，与 crates 内联用例的一次性库无关 |
| F23 | .claude/kb/experiments/171-三oracle合谋测试.md:923 | 不相干 | E171 第二段「比对器 `scripts/experiments/e171_three_oracle_collusion_after_the_oracl」讲第二段比对器，与一次性库无关 |
| F23 | .claude/kb/experiments/171-三oracle合谋测试.md:925 | 不相干 | E171「研究一侧变异表 `scripts/mutation/tables/e171_three_oracle_collusion_after_t」讲第二段变异表，与一次性库无关 |
| F23 | .claude/kb/experiments/171-三oracle合谋测试.md:1035 | 不相干 | E171 路径登记第 7 行「第 1 次重跑：O1」只引装置函数落点，与一次性库无关 |
| F23 | .claude/kb/experiments/171-三oracle合谋测试.md:1036 | 不相干 | E171 路径登记第 8 行「第 1 次重跑：O1」是比对器现算，与一次性库无关 |
| F23 | .claude/kb/experiments/171-三oracle合谋测试.md:1179 | 不相干 | E171「它答不了的」「这一页的行级读数（全放行、只靠一条、未触达的名单与每行的态）只属于快照摘要那一棵冻住的树」，与一次性库无关 |
| F23 | .claude/kb/experiments/171-三oracle合谋测试.md:1183 | 不相干 | E171「它答不了的」「新树重跑装置」那一条讲 `piece_panic_states=` 的计法，与一次性库无关 |
| F23 | .claude/kb/experiments/171-三oracle合谋测试.md:1184 | 不相干 | E171「变异表旧第 38 行已改名」那一条讲变异名与补位，与一次性库无关 |
| F23 | .claude/main-agent.md:93 | 不相干 | main-agent.md「一批实现员交回到齐、合入之前」那一行只拿 `stream-constants` 子命令举例，与一次性库无关 |
| F23 | records/2026-09-30-里程碑三开工第一轮.md:578 | 不相干 | 调度记录「点名项门槛（去掉 e171 一段的副本」那一条讲 clippy 红在这份装置里，与一次性库无关 |
| F23 | records/2026-09-30-里程碑三开工第一轮.md:828 | 不相干 | 调度记录「形式化线（E171）回：冻结在两条线各回」讲装置 sha 与合入次序，与一次性库无关 |
| F23 | records/2026-10-01-删除与并发事务.md:750 | 不相干 | 调度记录「2026-10-05 形式化线请合入验证再带」讲 E171 装置返工补丁，与一次性库无关 |
| F23 | records/2026-10-01-删除与并发事务.md:1158 | 不相干 | 调度记录「2026-10-07 S 合入的两边回话」点名这份装置不在补丁里，与一次性库无关 |
| F23 | records/2026-10-01-删除与并发事务.md:1190 | 不相干 | 调度记录「2026-10-07 T 合入前两边回话」讲穷举 match 编不编得过，与一次性库无关 |
| F23 | records/2026-10-01-里程碑三形式化接入.md:703 | 不相干 | 调度记录「16-5 交回（登记」讲新装置另起，与一次性库无关 |
| F23 | records/2026-10-01-里程碑三形式化接入.md:905 | 不相干 | 调度记录「22-6 派出（opus，E171 第 1 次重跑第一段」讲装置起头与冻结树，与一次性库无关 |
| F23 | records/2026-10-01-里程碑三形式化接入.md:922 | 不相干 | 调度记录「第 24 批合入验证（单元」讲 clippy 红，与一次性库无关 |
| F23 | records/2026-10-01-里程碑三形式化接入.md:923 | 不相干 | 调度记录「22-6 交回（E171 第 1 次重跑第一段」列装置与变异行，与一次性库无关 |
| F23 | records/2026-10-01-里程碑三形式化接入.md:1079 | 事件句不改 | 调度记录「调查员交回（报告」那一条是 2026-10 早先流三 I-3.1 红的归因（`LiveStream::start` 没装根环表），说的是那一次，与这一批复用 33 张表的红不是同一个病根 |
| F23 | records/2026-10-01-里程碑三形式化接入.md:1299 | 不相干 | 调度记录「用户 2026-10-07：「把能做的往前做」」讲装置内联用例证红怎么起，与一次性库无关 |
| F23 | records/2026-10-01-里程碑三形式化接入.md:1305 | 不相干 | 调度记录表行「第 37 批 + 修订 8 乙类常量换进仓里装置 + 代码轮」只点名改到的文件，与一次性库无关 |
| F23 | records/2026-10-01-里程碑三形式化接入.md:1409 | 不相干 | 调度记录「更正：修订 8 第 8.1 节里的「装置 sha」指装置源文件」，与一次性库无关 |
| F23 | records/2026-10-11-scripts审计.md:19 | 不相干 | scripts 审计表第 6 行「`scripts/agents/agent-cost.py:37`」讲写死路径，与一次性库无关 |
| F23 | records/2026-10-11-scripts审计.md:31 | 不相干 | scripts 审计表第 13 行「revision_10_placeholders.py」讲引不存在的路径，与一次性库无关 |
| F23 | records/2026-10-11-崩溃放量两段与脚本审计.md:53 | 不相干 | 调度记录「2026-10-11：快档带 `gpu,verdict-store` 特性那一趟」里 `tables_reused=6` 是崩溃注入对拍用例，不是 e171 第三条流 |
| F23 | records/2026-10-11-崩溃放量两段与脚本审计.md:117 | 不相干 | 调度记录「2026-10-13：合入前的 E171（三oracle合谋测试） 常量核对」，与一次性库无关 |
| F23 | records/2026-10-12-里程碑三工具与治理批.md:75 | 不相干 | 调度记录「批 2 收尾（工具戊）交回」讲 rows_orchestrator 进排除表，与一次性库无关 |
| F23 | records/2026-10-13-复用键与全仓改名.md:201 | 不相干 | 调度记录「`read-faulted-states-never-hold-the-witness-criterion`」里 `tables_reused: 13` 是另一份测试的计数，不是 e171 |
| F23 | records/2026-10-13-复用键与全仓改名.md:382 | 不相干 | 调度记录「合入后的 checker 档快档第一趟（不带特性」里 e171 装置红的是 panic 归属两条，与一次性库无关 |
| F23 | records/2026-10-13-复用键与全仓改名.md:384 | 不相干 | 调度记录「这一趟快档作废、停掉（退 143）」讲运行时现算结论版本，与一次性库无关 |
| F23 | records/2026-10-13-复用键与全仓改名.md:584 | 不相干 | 调度记录「4-2a（U3）交回」里「任一单元拒算整趟走一次性库」是崩溃放量那一件，不是 e171 第三条流 |
| F23 | records/2026-10-13-复用键与全仓改名.md:596 | 不相干 | 调度记录「病根（现查，量的）」里「录入键拒算→一次性库」是崩溃放量那一件，不是 e171 |
| F23 | records/2026-10-13-复用键与全仓改名.md:785 | 不相干 | 调度记录「第四轮 D 交回」讲一次性库目录怎么建与跨进程交不过去，不是 e171 第三条流 |
| F23 | records/2026-10-13-复用键与全仓改名.md:809 | 不相干 | 调度记录「第四轮 E 交回」里「同文件三条用例改用一次性库」是发现日志那份测试，不是 e171 |
| F23 | records/2026-10-13-复用键与全仓改名.md:811 | 不相干 | 调度记录「第四轮 E 留下的两问」里三条发现日志用例照用一次性库，不是 e171 |
| F23 | records/2026-10-13-复用键与全仓改名.md:969 | 不相干 | 调度记录「监控第 55 轮（量的，2026-10-18）」里 `tables_reused=21` 是随机历史与故障注入那一条的计数，不是 e171 |
| F23 | records/2026-10-18-里程碑三收尾与近期欠账.md:165 | 不相干 | 调度记录「调查员交回双机导入冲突」里三条用例改用一次性库是崩溃注入那份测试，不是 e171 |
| F23 | records/2026-10-18-里程碑三收尾与近期欠账.md:239 | 不相干 | 调度记录「b6 告：映射条目表那件也打进主工作区」里叫停、占位两条用一次性库是崩溃注入那份测试，不是 e171 |
| F23 | records/2026-10-18-里程碑三收尾与近期欠账.md:783 | 事件句不改 | 调度记录「e171 每次新建一次性库、断言 `tables_reused == 0`」是修红第二批交回的内容，事件句 |

## 反向问（第 10 步）：做成的事在该记它的载体里有没有

只问分到的四组背后的做成的事（范围清单「做成的事」第 7、10 条），没加 M 行：

- 撕裂表开线程之前建一份（第 7 条）：剩下的口子记在 `.claude/kb/checks-owed.md` C911（结束那一版第 585 行，英日各一行）；修法与根因在调度记录第 736、779 行。kb 里没有别处写过「每段新建撕裂表」的现状句要跟改（`git grep` 结束那一版 kb 与不变量清单里「mkfs 用过的池」「撞内存」「71.6」，只命中无关的几处）。
- findings_log 快档 46 → 54（第 7 条）：kb 里不记这个钉值，载体只有测试本身与调度记录第 758、782 行，不用补。
- e171 第三条流改一次性库（第 7 条）：符合 `.claude/kb/verification-build.md`「八」已有的「空库起跑的单测只在一次性副本里建库」，kb 不用另记；调度记录第 783 行有。
- E184 第 2 次重跑与 X3 待改（第 10 条）：E184 页现状第 13 行、「第 2 次重跑」一节、第 21、156、253 行、索引三语第 28 / 208 行都记了；X3 的时点与「候选甲不追」只在调度记录，E184 页第 245 行要按上表改。
- 范围清单之外主 agent 另给的 `scripts/kb/batch-target.py`（事实表 F45，结束那一版 0 命中）：主工作区暂存的 `.claude/main-agent.md` 第 99 行已经写了它，但这一行与那份脚本登在 `.claude/batch-scope` 第 339 行「2026-10-24 审计与文档门禁治理」那一批，不在这一批的对象里。不归 F19–F23，没加 M 行，交主 agent 判它归哪一批。

## 交实现员

0 行。分到的 158 行里载体在 `crates/` 下的是 0 行（`awk -F' \\| ' '$2 ~ /^crates\//' table-*.md | wc -l` 得 0），判「要改」的 10 行都在 `.claude/kb/` 下。

## 顺带看到的（不在分到的组里，没判，只报）

- 结束对象与主工作区的行号对不上：E184 页第 21、156 行与调度记录第 835 行写 X3 常量在装置 `:2913`、用处 `:3035`，结束对象（与 HEAD）里是第 2912、3033 行，主工作区（暂存了别的批的改动）里是第 2913、3035 行；C911 一行写的 `crash_state_enumeration.rs` 第 554、642 行，结束对象里是 548、636，主工作区是 554、642。kb 是照主工作区写的，那一批提交之后就对得上；这一批要是单独提交，这几处行号会偏。
- E184 页第 576 行（路径登记第 5 行）写 `crash_enumeration_new_pool_file_creation_stream.rs:762 FULL_STATES_WITH_TWO_STATES_PER_WRITE`，结束对象里这个常量在第 757 行，主工作区在第 774 行，哪一版都不是 762。
- E142 的英日两页（`142-新池新建文件的干跑.en.md`、`.ja.md`）在主工作区里有，不在结束对象里，所以没进候选；第 22、310 行改的时候英日对应两处要一起改。
- 结束对象里 `crates/singlefs-checker-tier/tests/in_place_overwrite_torn_state_count.rs:317` 还钉 `today_full, 16_777_240`（注释「今天的全量闭式」），主工作区那一份已是 16_777_243（第 318 行，别的会话没提交的改动）；`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:4051` `ANCHOR_FIRST_STREAM_STATES = 16_777_260`、`scripts/crash/crash-point-replay-shard-run.sh:24` 注释「闭式 16777260 态」「multi-record-publish-stream 1358954634」，都是取号写一轮的数。这些不是现状载体、不在候选表里，没判。
- 欠账表里没有「E142 装置与产物还是取号写一轮的段形」这一行（`git grep '取号写两轮\|取号两轮'` 结束那一版 `checks-owed.md` 只命中 C555）。C893 只管 e170、e171；`crash_enumeration_new_pool_file_creation_stream.rs` 第 756 行注释写「E142 产物 `closed_form=` 那个数要跟着重出」。要不要立账交主 agent 定。
- 里程碑三五·7（中英日第 113 行）写「要等下一次提交时 checker 档实跑才算数」：这一批带特性单跑过一趟（调度记录第 725、814 行），只是那一趟不是提交时跑的门禁。我判的是它与 F22 的快档态数无关，要不要改由主 agent 看。

## `--check-report` 输出末行

```text
$ python3 scripts/kb/stale-candidates.py --check-report /home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-facts/candidates.tsv /home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-s7/result.md --groups F19,F21,F22,F23
rc=0
  ✓ 报告判全了：158 行候选都有逐行判定；crates/ 下判要改要补的 0 行都在「交实现员」一节里（不变量清单的 .md 除外）
```

## 没做什么

- 没改仓里任何文件，没跑门禁、lint、重型测试，没提交。
- 只判了 F19、F21、F22、F23；F20 不出候选。候选表只罩事实表写到的事实与检索词搜得到的句子：E142 页里写成 `16 777 240`（带空格）的旧数、英日 E142 页（不在结束对象里）都搜不到。里程碑 01 中英日那一行的「11 段、闭式 16 777 240」是我读整行时看到、并进改法的。
- 判的依据是结束对象 `1e3079d7…` 里的文件（`git show`）；主工作区里别的会话没提交的改动只用来解释行号差，没拿来判。
- 要改那几行的英日改法是我照中文改法写的译文，没走三语门禁；写回员可以照自己的译法改。
- 草稿目录里留着 `tools/`（判定数据与生成脚本）、`table-F*.md`、`e184.md`、`f19.txt`、`check-report.txt`，没有仓副本、编译目录或库拷贝。
