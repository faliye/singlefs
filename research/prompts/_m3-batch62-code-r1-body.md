# 删除线第 62 批：代码轮第一轮正文（2026-10-11）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 R1 R2 R3 R4 R5 -->

## 一、题面

这一轮攻第 62 批一份补丁，已合进主工作区（底是第 60、61 批与根环线 E12–E14 合入之后）。第 62 批照判决 `research/prompts/m3-batch60-61-code-r1-main-verification.md` 第三节 Q1–Q4 改：只改测试与 `crates/singlefs-checker-tier/src/verdict_store.rs` 里一个只给测试用的批长参数，`crash_amplification.rs` 一行没改。

| 关的 | 落点 |
|---|---|
| Q1 流水线导入那一趟设 `SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu`，断言导入那一趟三个缺口计数都是 0、按块读回的核对行与一台判完的那份相同；变异「排键时一行核对行都不点」 | `tests/crash_amplification_pipeline_records_checks_and_reuses.rs` 第 1219 行那条 |
| Q2 两条用例各在导入之后再导一次，钉第二次的 `imported_blocks` 等于第一次；变异两行「导入块数只数写下的」 | 同一文件第 1219、1481 行那两条 |
| Q3 一字节一批时换批、跨批编号接续、核对行已有时事实行补写；产品入口照旧取 64 MiB 常量，单测经私有的 `import_the_planned_rows_in_batches_of` 给小批长；变异三行追加、一行整行替换（批长判据跟到新写法） | `verdict_store.rs` 第 1772–1950 行、单测第 4151、4249 行 |
| Q4 GPU 配置下按计划导入带上 GPU 判器那一种结论与 GPU 核对代码那一把摘要；新开一份带 `gpu` 特性的测试文件两条；变异三行（判决要两行，实现员报告说明多一行的理由） | 新文件 `tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs` |

条款与依据：判决 `research/prompts/m3-batch60-61-code-r1-main-verification.md` 第二节 P4、P5 与第三节 Q1–Q4；`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」「用上现有的硬件加速」「测试脚本自己也要被测试证明有效」；`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「只让多条臂互相比，测不出「所有臂一起错」」。

改动全文：`research/prompts/_m3-batch62-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-batch62/planned-import-pins/report.md`（含手算的 28 行 / 7 批、GPU 那条 2 块与 8 块）。上一轮判决与腿报告：`research/prompts/m3-batch60-61-code-r1-main-verification.md`、`m3-batch60-61-code-r1-opus-output.md`、`m3-batch60-61-code-r1-sonnet-output.md`；攻方模型 `research/prompts/m3-batch60-61-code-r1-opus-model/`（可以拷来改臂）。

## 二、实现今天的样子（主 agent 的观测，2026-10-11）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch62-code-r1/repo/`（主工作区合入第 62 批之后的 `crates/`、`Cargo.toml`、`Cargo.lock`）。被判文件与 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m3-batch62-code-r1-snapshot/crates-sha256.txt`。代码行号按冻结副本。
- 主 agent 在实现员副本里跑过：流水线两条 CPU 用例「2 passed」，`verdict_store::tests::planned_import` 与 `importing_the_planned_rows_of_a_flow` 两组「4 passed」；CPU 证红「✓ 点名 7 条：跑了 7 条，抓到 7 条」。合入主工作区后 `merge-and-verify.sh`「✓ 测试全绿（194 个测试二进制，共 2019 条测试）」，fmt、clippy、build 过。
- 没跑成的：GPU 那份新测试文件的基线、GPU 证红 3 行、变异轻型 10 行，都因本机 NVIDIA 驱动内核模块与用户态库版本对不上（`nvidia-smi`「Failed to initialize NVML: Driver/library version mismatch」）退 78，卡恢复后由主 agent 重跑，结果写进判决。腿不起 GPU。
- 实现员自定、主 agent 接受、被攻过零轮的：批长从常量挪成私有参数（产品入口不变）；批长判据那一行变异整行替换、点名的测试不变；GPU 变异多出给定状态表那一路一行；Q4 两条在没卡、显存不够（`stopped_early`）、给定状态表判器报运行条件不满足时打「本次未跑」那一行并返回（照 `crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs` 的做法）。
- 已知、记作欠账、报出来不算打中的：`.claude/kb/checks-owed.md` C786（按计划导入几份库同一块各数一次）；导入改完之后的秒数没量、改法 2（按段预算前缀持久掩码）与拷整库给第二台（`.claude/kb/milestone/03-third-txn.md`「## 待优化：流程、门禁与装置上踩过的坑」第 22 条）；C784（只判最右之前 k 片的变异有限组关不死）；形式化轻型那份红例账缺失（同一张表第 24 条）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### R1　核对行钉得住没有（Q1）
导入那一趟设了 `VERIFY=cpu` 之后，「三个缺口计数都是 0」与「核对行逐项相同」两条断言，是不是只有排键时把核对行点全才过？攻方 h02 那个形状（第二台那一块只在第二台的库里核过）在用例里真的出现了吗（逐步数哪一块在哪台核）？变异「两种核对代码都不点」红在哪一条，会不会只红在别的断言上而让核对行那一条从来没起作用？

### R2　重导计数（Q2）
「第二次的 `imported_blocks` 等于第一次」钉的到底是哪个口径：数点名到的块，还是数写下的块？这个口径与 `crash_amplification.rs` 报出去的 `imported_blocks=` 那一行、与判决 P5-2 的改法是不是同一个量？给定状态表那一路的账四元组「与第一次相同」是不是把计数之外的东西也钉上了，有没有在正确实现下因无关的变化而红的风险？

### R3　换批、跨批编号与事实行补写（Q3）
私有批长参数是不是只有单测能给，产品路径上有没有一条能拿到非 64 MiB 的批长？28 行 / 7 批这个绝对值是手算的：照实现员报告的拆法逐批数一遍，对不对？一字节一批时「两份库逐字节相同」与「编号 2、3 是两种新向量」两条，能不能抓住编号只在一批之内接续的错法？事实行补写那条：核对行已有且相同、事实行缺的库，是不是产品里真会出现的形状（断了重跑那一形），只点核对行重导就补得齐吗？整行替换的那一行变异在新写法下还改不改行为？

### R4　GPU 两条与「本次未跑」（Q4）
两条 GPU 用例在没卡、显存不够、判器报运行条件不满足时打「本次未跑」并返回：返回之后测试是绿的，门禁 `checker-tier-layer0-replay` 的 feature-gated-tests 格认不认得这一行，会不会把「跑了 0 条」当成跑过？`stopped_early` 那一支会不会在卡在、只是本机负载高的时候被走到，让一条真红被吞成「本次未跑」？2 块与 8 块两个绝对值另外怎么算出来的，前提断言能不能在错的配置下（GPU 没真判）照过？三行 GPU 变异各红在哪一条？

### R5　改动范围与产品路径
`crash_amplification.rs` 没改：Q1–Q4 判决点名的缺口（P4-1、P5-2、B1、B2、Q4）在产品代码里本来就对、只差测试，还是有一处要改产品代码而这一批没改？`verdict_store.rs` 的签名变化有没有影响别的调用方（旁库导回主库那一处、旧 `import_from`）？

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | R1–R5 | 核「代码与测试做的是不是判决第三节 Q1–Q4 点名的那件事」，逐条对判决原文与冻结副本的行 |
| 云端攻方（Opus） | R1–R5 | 在冻结副本的拷贝上造反例：用例在错的理由上绿或红、改坏照绿、「本次未跑」吞掉真红、绝对值算错、产品路径上还有没罩到的缺口 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷编译目录当种子之前先 `find crates -type f -exec touch {} +`。建副本与编译目录之前先查自己草稿目录里有没有不用的，先删。读码推出的「没打中」尽量换成造出来的反例尝试（不跑 checker 档测试的前提下：拷出被测函数写独立小模型、或读码逐步数）。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；线程上限 15；用自己的编译目录，几份变异副本各用各的。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
