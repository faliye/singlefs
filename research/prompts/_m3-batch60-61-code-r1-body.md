# 删除线第 60、61 批：代码轮第一轮正文（2026-10-11）

<!-- doc-lint:not-numbers N1 N2 N3 N4 P1 P2 P3 P4 P5 -->

## 一、题面

这一轮攻第 60、61 两批四份补丁，已合进主工作区（底是根环线 E9–E11 合入之后）。第 60 批照判决 `research/prompts/m3-batch59-code-r1-main-verification.md` 第三节 N1–N4 改，只改测试与变异表；第 61 批照调查报告 `~/.local/share/singlefs/work/m3-batch60/exponential-stage/report.md`「改法 1（主）」改崩溃放量双机跑法的导入，改了 `crates/singlefs-checker-tier/src/`。

| 份 | 关的 | 落点 |
|---|---|---|
| 甲 | N1 GPU 三棵树各一条旋钮 2 下 [2,1,2,2,1]、转一圈根环的对拍；L1 三条与新三条先断言 CPU 违例集恰好是这一棵那一条；删「慢档」注释；变异三行 k = 2 | checker 档 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` |
| 乙 | N2 L4 那条改补 (7, 0, 0xFFFF_FFFF, txg)、注释改实情；N3 钉「分配记录判在条目宽之前」（冷走读、可写挂载各一条，变异两行） | harness `core_review_tree_table_duplicates_and_slot_one_search.rs`、`corrupt_on_disk_content_is_refused_instead_of_panicking.rs` |
| 丙 | N4 变异轻型核心部分红的 13 行：两条新取样点、8 行整行替换、1 行改名换改坏处 | harness `system_configuration_slot_is_overwritten_only_after_a_barrier.rs`、`shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs`；`crates/mutations.tsv`；`.claude/gate.d/crates-mutation-light-rows.tsv` 第 54 行跟着改名 |
| 丁 | 双机导入只导这一趟计划里不归本机那一份的块：`VerdictStore::import_the_planned_rows_from`（本库已有且相同跳过、不同报错、成批同步写）、`record_path` 先比再写；流水线导入那一段按计划排键；给定状态表那一路判到别机的表时点名导入；新旧导入对拍、调用方排键用例、变异 10 行追加 2 行替换 | `crates/singlefs-checker-tier/src/verdict_store.rs`、`crash_amplification.rs`、`tests/crash_amplification_pipeline_records_checks_and_reuses.rs` |

条款：`.claude/kb/invariants.md` I-9.22、I-9.37、I-3.13；`.claude/kb/decisions/05-快照-空间记账机制.md` 已定项 8（「没有对象的统计量不写行」）；`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「编译与算法都要快」「判定可复用：按输入存，按前缀剪」。I-9.36、I-9.37 定义在 `~/.local/share/singlefs/work/m3-batch57/container-judges/report.md`。

改动全文：`research/prompts/_m3-batch60-61-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-batch60/gpu-far-left/report.md`、`m3-batch60/watermark-and-width-r2/report.md`、`m3-batch60/core-light-rows/report.md`、`m3-batch61/import-planned-blocks/report.md`、`m3-batch61/import-planned-blocks-2/report.md`。上一轮判决与腿报告：`research/prompts/m3-batch59-code-r1-main-verification.md`、`m3-batch59-code-r1-opus-output.md`、`m3-batch59-code-r1-sonnet-output.md`；攻方模型 `research/prompts/m3-batch59-code-r1-opus-model/`（可以拷来改臂）。

## 二、实现今天的样子（主 agent 的观测，2026-10-11）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch60-61-code-r1/repo/`（主工作区合入第 60、61 批之后的 `crates/`、`Cargo.toml`、`Cargo.lock`）。被判文件与 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m3-batch60-61-code-r1-snapshot/crates-sha256.txt`。代码行号按冻结副本。
- 第 61 批在实现员副本里由主 agent 跑过：`crash_amplification_pipeline_records_checks_and_reuses` 「16 passed」、`verdict_store::tests::` 「27 passed」、`crash_injection_campaign` 点名两条「2 passed」；`prove-red.sh` 「点名 12 条：跑了 12 条，抓到 12 条」。合入后验证、卡上证红与两条轻型的结果写进判决。
- 实现员自定、主 agent 接受、被攻过零轮的：丙把「末条标志」那一行改名换改坏处（不删行，让那个 checker 档目标在轻型里留一条真会改行为的变异）；丁的事实行跟着核对行导、不由调用方点名；给定状态表那一路「这一趟的计划」取判到的、不归本机的表，判那张表时才导，导入失败挪到那时报；`imported_blocks=` 的含义改成这一趟点名导到的块数；旧 `import_from` 留着（对照旁库导回主库那一处还用，几条测试当参照）。
- 已知、记作欠账或下一批线索、报出来不算打中的：GPU 配置下点名导入漏掉 GPU 判器那一种没有用例会红；导入改完之后的秒数没量；改法 2（按段预算前缀持久掩码）先量再定；J4 与 3849 那一族 GPU 行等根环线；「只判最右之前 k 片」对足够大的 k 有限组排布关不死；形式化轻型 pinned-red-examples-complete 格因归档提交删掉一份红例账而红，不归这两批；第 59 批判决第四节与更早判决列的欠账照旧。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### P1　GPU 远左短片与精确红集（甲 N1）
新三条是不是只因旋钮支红、CPU 违例集恰好是这一棵那一条？精确红集断言会不会在正确判法下因造法顺带的别的违例而红？k = 2 那三行变异红在哪一条断言上，是不是逐位不一致那一条而不是精确红集那一条？L1 三条收成造法结构之后，第 59 批那三行变异还点不点得中、还红不红？

### P2　水位统计量 7 与分配记录判序（乙 N2、N3）
补进来的 (7, 0, 0xFFFF_FFFF, txg) 会不会被读者在别的理由上先拒，让「水位行缺」在错的理由上报出？「台阶行之外每一行代段等于这一版 txg」这句前提断言对不对？N3 那张盘是不是只由「分配记录判在条目宽之前」分得出，两行变异各只红自己那一条的理由成不成立？

### P3　轻型核心行的修补（丙 N4）
两条新取样点是不是在测变异行点名的那件事：取号回卷之后的屏障、影子账一圈窗口在抬 F 那一路的判别力？把根槽清零再写回那一手是不是硬造（产品会不会走到那一形）？8 行替换与改名那一行在原条款下是不是真会改行为、轻型里跑不跑得到？

### P4　按计划导入的正确性（丁）
按计划导入后，本机库对这条流的读回与整库导入逐项相同吗：排键的全局序号与 `my_blocks` 是不是同一套（含 ignored 的点）、观察者结论挂没挂全、两种判器各一把版本取得对不对、核对行与事实行带没带全？「本库已有且相同就跳过」比的是什么，比漏了会不会让一块旧判法的结论顶掉新的？成批写入中途断了，重跑同一条命令能不能补齐、块内原子还在不在？给定状态表那一路判那张表时才导，会不会让一张该导的表被现判、或让导入失败被吞？

### P5　按计划导入的代价与钉值（丁）
导入的读写次数是不是真的只随这一趟那一份的块数线性、与库里累计行数无关（逐步数）？钉的绝对值（21 行、导入块数 2、导入 4 复用 4 现判 0）是不是另外算出来的、改坏会不会红？旧 `import_from` 留着的几处调用方还有没有随库涨的路径？

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | P1–P5 | 核「代码与测试做的是不是条款、判决与调查报告点名的那件事」，逐条对条款原文与冻结副本的行 |
| 云端攻方（Opus） | P1–P5 | 在冻结副本的拷贝上造反例：用例在错的理由上绿或红、改坏照绿、导入后读回与整库导入不同、断了重跑补不齐、代价仍随库涨 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷编译目录当种子之前先 `find crates -type f -exec touch {} +`。建副本与编译目录之前先查自己草稿目录里有没有不用的，先删。读码推出的「没打中」尽量换成造出来的反例尝试。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；线程上限 15；用自己的编译目录，几份变异副本各用各的。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
