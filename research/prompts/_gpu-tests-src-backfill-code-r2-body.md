# gpu-tests-src-backfill-code-r2：产品 GPU 测试包 E202 装置第一轮改法的复核（2026-11-18）

代码轮第二轮。第一轮判决 `research/prompts/gpu-tests-src-backfill-code-r1-main-verification.md` 第三节的改法已合进主工作区（打补丁任务 19651、回读任务 19654），这一轮攻合入之后的样子。开工快照 `research/prompts/gpu-tests-src-backfill-code-r2-snapshot/`（`crates-sha256.txt` 七份、`kb-sha256.txt` 两份、`changes.patch` 是合进主工作区的那一份补丁、三份变异行文件是这一批追加 25 行、换 3 行、删 2 行的原样）。

方案按 `crates/` 今天的实现来谈。

## 实现今天的样子（主 agent 的观测，2026-11-18 现读）

- `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs` 共 13881 行。文件头第 32–42 行写卡的规矩：卡只用服务交的；「全部卡」臂由 `lockstep --cards <张数>` 设；两条用卡的臂开的卡逐张记进产物（`name=every_card_arm_cards`、`name=one_card_arm_card`）；续跑时对不上打 `name=stopped clause=cards_differ_from_the_product_being_resumed` 退 2；一条结果行都没跑出来的那一趟不挡以后的续跑。
- 续跑核卡：`enum ArmWhoseCardsAreRecorded`（`:5966`）、`struct CardsRecordedForAnArm`（`:5997`）、`fn cards_recorded_differing_from_the_product_being_resumed`（`:6104`），停下那一行在 `:6206` 打。
- crates 摘要：`STARTING_PATHS_OF_THE_CRATES_DIGEST`（`:4135`，五处起走的路径）、`fn source_files_and_digests_walked_with`（`:4184`，目录逐层走到每个文件）。
- 取卡：`fn cards_handed_to_the_apparatus`（`:5754`）。
- 按位置取臂的写法还在的几处：`LOCKSTEP_ARMS[0]`（`:5378`、`:7986`、`:8038`、`:10942`）、`LOCKSTEP_ARMS[1]`（`:7986`）、`LOCKSTEP_ARMS[4]`（`:10665`）、`LOCKSTEP_ARMS[3]`（`:10943`）。
- 新测试 `crates/singlefs-background-gpu-tests/tests/reference_units_entries_name_functions_that_exist.rs`（137 行）：逐组核参照一侧登记表的入口。
- 任务服务此刻不给任务分卡（卡归 GPU 批处理验证服务）：装置在任务里拿不到卡串，要卡的四个子命令此刻跑不出来；文件头第 4 行 `// task-server-gpu-cards: all` 与讲服务交卡的几句因此过时，没在这一批改（管 E202 的会话等「交 GPU 服务代跑」的接口定型后一次改）。

## 这一轮攻什么

第一轮判决第三节的七条改法，逐条给出改后的落点，腿判「改法做没做到判决写的那一条」「改法自己有没有新带进来的错」：

1. **续跑核卡的身份**（第三节第 1 条，零轮）：卡的身份取 PCI 地址加型号，记进两种行，续跑逐张比。实现员自己报的边角：认不出「两台同一个槽插同一种型号」那一形。攻：比的字段全不全（张数、开法、每张的地址与型号、次序）；产物里有这条臂的结果行而没有记卡行时停不停；并发跑（`name=concurrent_run`）算不算点名一张卡臂跑出的；`--product` 文件里夹着别的臂的行时会不会误停或漏停。
2. **一张都没交的那一趟不挡续跑**（第 2 条，零轮）：攻「一条结果行都没跑出来」怎么判的（读产物里哪种行）、`--cards` 多于交来的张数与领卡没成两种出路是不是同一条路、那一趟写下的行会不会让之后的核卡把它当成「有结果行而没有记卡行」。
3. **crates 摘要算上装置自己**（第 3 条）：五处起走的路径罩不罩全装置读的源文件（装置自己、`#[path]` 引进来的共用模块、参照登记表）；验证包里装置用到的模块（按需领卡、判定库）没进摘要，判决没点这一条，攻它该不该进。
4. **跑前登记修订第 12 条**（第 4 条）：`research/prompts/e202-preregistration.md` 第十二节新加的第 12 条与装置今天的行为逐句对不对得上；它写「服务交卡」而任务服务已不分卡，这一句该怎么记。
5. **参照登记表的测试与变异、三条新单测的变异行、按名字取臂**（第 5 条）：新测试判的是不是「登记错一组入口会红」；`LOCKSTEP_ARMS[下标]` 还剩七处，第一轮判决点的新加的三处（`:12651`、`:12654`、`:12656`，改前行号）改没改掉、剩下的是不是都是旧代码。
6. **并行下一次性库用例与真卡用例的冲突**（第 6 条）：加了守卫；证红里那一行（「E202 真卡对拍经全仓那一份库时不拿守卫…」）抓到了（任务 19641）。攻守卫罩不罩全开真库的每条用例。
7. **旧代码里的线索**（第 7 条）：登记欠账没有，在哪。

另加第一轮判决里写明只抽查过、没看过的：攻方第八节列的「没有变异罩着的判红分支」逐处补没补全；26 行不带真卡特性的变异全抓到（任务 19641「计数：抓到 26 条、没红 0 条、无效 0 条、锚点失效 0 行」），带 `tests-on-a-real-gpu` 特性的 2 行（「E202 内联真卡对拍：持卡判拿每段多一槽的参照比…」「E202 小池 greedy 调用：丙段那一截的次数报成 0」）没判——任务拿不到卡，用户定记本次未跑。

已攻过而没打中的角度这一轮不重复：取卡只用服务交的卡、开卡表整个进程一份、先录后判经判定库、判定行追加进产物文件不算另开判定库。

## 验证读数（主 agent 经任务服务跑出的）

- 合入后验证任务 19622：「测试全绿（2 个测试二进制，共 21 条测试；跑了 2 项（1 个包），--item 点名的；按条复用：现判 21、复用 108、拒算 0）」，fmt、clippy（default 与 --all-features）、build 过。
- GPU 测试包整包（合入前的快照上，任务 19465）：166 条过、0 条败。
- 变异任务 19641：26 行全抓到。命名两条（任务 19639、19640）退 0。合入后 mutation-tables 格（任务 19655）绿。

### 检查单：走不到的理由二次查实

实现员报告（`~/.local/share/singlefs/work/e202-apparatus-backfill-fixes-second/patch/result.md`「走不到的理由」一节）原文：「无。产品代码 `crates/<包>/src/` 里这一次没新加 `todo!`、`unreachable!`、`assert!`，也没有条款没定的错误成员。」腿在 `changes.patch` 的非测试段里自己搜一遍这几样，有一处就是打中。

### 检查单：类型与名字

新加的名字：`ArmWhoseCardsAreRecorded`、`CardsRecordedForAnArm`、`cards_recorded_differing_from_the_product_being_resumed`、`background_compute_the_arm_mounts_with`（命名修补改的，原名 `compute`）、`STARTING_PATHS_OF_THE_CRATES_DIGEST`、`PathWalkedForTheCratesDigest`、`source_files_and_digests_walked_with`、`what_the_path_is_on_the_disk`。卡的身份（PCI 地址、型号）在 `CardsRecordedForAnArm` 里用什么类型装，腿现读 `:5997` 起判是不是裸字符串混用。`LOCKSTEP_ARMS[下标]` 七处见上。新加的 `expect` 逐处看消息写没写依赖的不变量。

### 检查单：不兼容旧代码与旧格式

续跑核卡对「产物里有结果行而没有记卡行」的处置是停下（文件头第 40 行），不是放行旧产物；腿核有没有哪一支为认旧产物而放宽。

### 检查单：不新建判定库

本轮改动没有新的判定存放处：记卡的两种行追加进 `--product` 文件（实验产物）；真卡对拍的输入与参照经验证包那一份库（`common_pairing_input_through_the_library`）。腿在 `changes.patch` 里搜新开的目录、TSV、sqlite、RocksDB。

### 检查单：所有任务多机可用

卡的身份取 PCI 地址加型号：两台同一个槽插同一种型号时认不出换了一台（实现员自己报的）。腿判这算不算「读会改变判定结局的本机独有状态」、该不该把机器也记进那一行。新测试读不读仓根之外的路径。

### 检查单：测试优先多线程

新加的单测都是内联的小用例与一份集成测试，不走崩溃枚举；真卡对拍那几条在同一个二进制里并行时拿全仓那一份库的守卫（第 6 条）。腿核守卫是不是把本可并行的用例也串起来了。

### 检查单：带 --item 的命令与项

实现员报告「要主 agent 跑的命令」里 crates-mutation-replay 带 `--item` 那一条点 28 行；主 agent 交了其中 26 行（任务 19641），2 行带真卡特性的没交。新测试目标 `reference_units_entries_name_functions_that_exist` 由合入后验证的 `--item singlefs-background-gpu-tests:tests/reference_units_entries_name_functions_that_exist.rs` 点到（任务 19622）。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新加崩溃点重放流或崩溃枚举用例（查过 `changes.patch` 里没有 `crash-case:` 登记行，也没有调 `enumerate_crash_states` 一族的新测试）。
