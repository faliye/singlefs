# 删除线第 63 批：代码轮第一轮正文（2026-10-11）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 R1 R2 R3 R4 R5 S1 S2 S3 T1 T2 T3 T4 T5 T6 -->

## 一、题面

这一轮攻第 63 批一份补丁，已合进主工作区（底是第 60、61、62 批与根环线 E12–E14 合入之后）。第 63 批照判决 `research/prompts/m3-batch62-code-r1-main-verification.md` 第三节 S1–S3 改：只改测试与变异表，产品代码一行没改。同一轮另外复攻那份判决第四节点名「只抽了一次样」的 R2、R4、R5 三格（第二次抽样）。

| 关的 | 落点 |
|---|---|
| S1 Q3 用例在 `drop` 之前经照产品批长导完的那一份库按内容读回、再登记一种新向量看编号接在后面、从第二份对方库 C 导入同内容的新向量不另起编号、重开再比；变异三行（攻方 `take1`、`rev`，只去掉按内容查编号那一句 `insert`） | `crates/singlefs-checker-tier/src/verdict_store.rs` 单测：构造函数 `build_the_second_other_library_c_with_the_same_new_verdict_vectors`、用例 `planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes` |
| S2 `crates/mutations.tsv` 第 62 批 Q3 换批判据那一行换成只在小批长下改行为的替换 | `crates/mutations.tsv` 第 3987 行 |
| S3 流水线用例删掉「核对行点名的事实行与一台判完的相同」那一句断言与文件头那半句 | `crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs` |

条款与依据：判决 `research/prompts/m3-batch62-code-r1-main-verification.md` 第二节 R1、R3 与第三节 S1–S3；`.claude/singlefs-ai-sop/rules/test-discipline.md`「只让多条臂互相比，测不出「所有臂一起错」」「变异测试证明的是断言会红，不是覆盖」；`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」。

改动全文：`research/prompts/_m3-batch63-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-batch63/import-number-pins/report.md`（含手算的账 `(1, 1, 7, 0, 0, 0, rows 6, synced 1)`）。上一轮判决与腿报告：`research/prompts/m3-batch62-code-r1-main-verification.md`、`m3-batch62-code-r1-opus-output.md`、`m3-batch62-code-r1-sonnet-output.md`；上一轮攻方模型 `research/prompts/m3-batch62-code-r1-opus-model/`（可以拷来改臂）。第 62 批的正文 `research/prompts/_m3-batch62-code-r1-body.md` 第三节是 R2、R4、R5 三格的原题。

## 二、实现今天的样子（主 agent 的观测，2026-10-11）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch63-code-r1/repo/`（主工作区合入第 63 批之后的 `crates/`、`Cargo.toml`、`Cargo.lock`）。被判文件、`crash_amplification.rs`、GPU 那份测试文件与 `crates/mutations.tsv`（3997 行）的 sha256 在 `research/prompts/m3-batch63-code-r1-snapshot/crates-sha256.txt`。代码行号按冻结副本。
- 主 agent 在实现员副本里跑过：`verdict_store` 库单测点名那一组「29 passed」，流水线那份测试文件「16 passed」；证红「✓ 点名 4 条：跑了 4 条，抓到 4 条，本次未跑 0 条」，四行各红在 `verdict_store.rs` 第 4254 行（`take1`：编号 3 左边 `None`；`rev`：编号 2 是第 3 块那一种）、第 4295 行（去掉 `insert`：`rows_written: 10` 对 `6`）、第 4223 行（S2：「一字节一批：同样的 28 行分 7 批写」，`synced_writes: 1` 对 `7`）。合入主工作区后 `merge-and-verify.sh --skip-apply`「✓ 测试全绿（194 个测试二进制，共 2019 条测试）」，fmt、clippy、build 过。
- 第 62 批卡恢复之后补跑的：GPU 那份测试文件基线「2 passed」、日志里 `NOT_RUN_NO_GPU_ADAPTER` 0 行、`judge=gpu card=0` 的块行在；GPU 证红「✓ 点名 3 条：跑了 3 条，抓到 3 条，本次未跑 0 条」。主工作区的轻型崩溃与轻型变异在跑，结果写进判决。
- 实现员自定、主 agent 接受、被攻过零轮的：C 里两种新向量的编号次序与本库相反；从 C 导入时核对行不点（实现员报告「S1」第 4 步的理由）；S3 留下了「一台判完：核对行点名的事实行在库里」那一句。
- 已知、记作欠账、报出来不算打中的：`.claude/kb/checks-owed.md` C786（按计划导入几份库同一块各数一次）、C784（只判最右之前 k 片的变异有限组关不死）；`.claude/kb/milestone/03-third-txn.md`「## 待优化：流程、门禁与装置上踩过的坑」第 22、24 条；上一轮不采纳的攻方 G2、G3（判决第三节末段）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　S1 钉得住没有
`verdict_store.rs` 第 2069–2076 行写完一批之后把这一批登记的向量放进两张内存表。除了 `take1`、`rev`、只去掉 `insert` 三种，这几行还有哪些错法（编号取错一位、先 push 后算编号、两张表只更新一张的另一半、只在最后一批更新）在新用例下照绿？C 的编号次序与本库相反，这一步分辨的到底是哪一种错法，换成同序会少抓什么？账 `(1, 1, 7, 0, 0, 0, rows 6, synced 1)` 照实现员报告的拆法逐项数一遍，对不对？核对行不点这一选择有没有让某一种错法逃掉？

### T2　S2 替换之后的那一行
新替换文 `batch_size_in_bytes < PLANNED_IMPORT_BATCH_BYTES || …` 在产品批长下与原样相同、小批长下一批都不换：它与 `crates/mutations.tsv` 里「按计划导入 批长判据反过来（每项都同步写一批）」那一行（原文相同、替换文不同）改的是不是两种不同的行为？有没有别的点名测试在这一行下也该红而没点名？

### T3　S3 删掉之后
删掉的那一句原来有没有被哪一行变异靠着红（点名流水线那条用例的四行：「按计划导入 排键时选块条件反过来」「按计划导入 排键漏掉观察者那一种结论」「第62批 Q1 … mv」「第62批 Q2 … mi-pipe」）？导入补写事实行这件事，现在由 `verdict_store.rs` 库层的哪一条用例、哪一句断言判，那一句在「核对行已有且相同、事实行缺」的库上真的走到吗？

### T4　复攻第 62 批 R2：重导计数（第二次抽样）
原题照抄：「第二次的 `imported_blocks` 等于第一次」钉的到底是哪个口径：数点名到的块，还是数写下的块？这个口径与 `crash_amplification.rs` 报出去的 `imported_blocks=` 那一行、与判决 P5-2 的改法是不是同一个量？给定状态表那一路的账四元组「与第一次相同」是不是把计数之外的东西也钉上了，有没有在正确实现下因无关的变化而红的风险？上一轮两条腿都只读码；这一轮尽量造出来。

### T5　复攻第 62 批 R4：GPU 两条与「本次未跑」（第二次抽样）
原题照抄：两条 GPU 用例在没卡、显存不够、判器报运行条件不满足时打「本次未跑」并返回：返回之后测试是绿的，门禁 `checker-tier-layer0-replay` 的 feature-gated-tests 格认不认得这一行，会不会把「跑了 0 条」当成跑过？`stopped_early` 那一支会不会在卡在、只是本机负载高的时候被走到，让一条真红被吞成「本次未跑」？2 块与 8 块两个绝对值另外怎么算出来的，前提断言能不能在错的配置下（GPU 没真判）照过？三行 GPU 变异各红在哪一条？（卡上的基线与证红已经绿，见第二节；腿不起 GPU。）

### T6　复攻第 62 批 R5：改动范围与产品路径（第二次抽样）
原题照抄：`crash_amplification.rs` 没改：Q1–Q4 判决点名的缺口（P4-1、P5-2、B1、B2、Q4）在产品代码里本来就对、只差测试，还是有一处要改产品代码而这一批没改？`verdict_store.rs` 的签名变化有没有影响别的调用方（旁库导回主库那一处、旧 `import_from`）？这一轮加问：S1 在测试里经同一个打开着的库接着导第二份对方库，产品里 `crash_amplification.rs` 逐份导入那一处（`for directory in list.split(':')`）走的是不是同一条路径、同一个库实例？

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T6 | 核「代码与测试做的是不是判决第三节 S1–S3 点名的那件事」，逐条对判决原文与冻结副本的行；T4–T6 照原题重核，不抄上一轮正推的结论 |
| 云端攻方（Opus） | T1–T6 | 在冻结副本的拷贝上造反例：用例在错的理由上绿或红、改坏照绿、绝对值算错、删掉的断言留下的空、产品路径上还有没罩到的缺口；T4–T6 换上一轮没用过的攻法（上一轮攻方报告在 `research/prompts/m3-batch62-code-r1-opus-output.md`，不重复它的角度），读码的结论尽量换成造出来的 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷编译目录当种子之前先 `find crates -type f -exec touch {} +`。建副本与编译目录之前先查自己草稿目录里有没有不用的，先删。读码推出的「没打中」尽量换成造出来的反例尝试（不跑 checker 档测试的前提下：拷出被测函数写独立小模型、或读码逐步数）。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；线程上限 15；用自己的编译目录，几份变异副本各用各的。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
