# m4-publish-trigger-r1 判决：串行下时间与脏量两种发布触发、跨请求攒批（主 agent，2026-10-26，第一轮）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 G8 -->

被判的条款：`.claude/kb/decisions/16-发布语义.md` 已定项 2、5、10、11、14；`.claude/kb/checks-owed.md` C348、C422、C424、C425、C733。射程由用户 2026-10-26 定：「串行下时间与脏量两种触发」——时钟由调用方给、测试里能换，阈值先用只供测试的开关；挂载选项通道与对外 fsync 留里程碑十的一·2，并发与在飞深度留里程碑六。冻结副本 `~/.local/share/singlefs/work/m4-close-rounds-frozen/`。

两条腿：正推 `research/prompts/m4-publish-trigger-r1-sonnet-output.md`（sha256 c1a59adda5263cb9b0871e43bb8c669f2bb46b1e223974ca217f343b4ae46d79，模型目录 `research/prompts/m4-publish-trigger-r1-sonnet-model/`），攻方 `research/prompts/m4-publish-trigger-r1-opus-output.md`（sha256 66fdbba8e5df9381b9fba2dfdad5fa370b4540b3937ee661f1122779e40ef662，模型目录 `research/prompts/m4-publish-trigger-r1-opus-model/`）。

## 一、核查

- 开工快照：`python3 scripts/agents/snapshot-tree.py m4-publish-trigger-r1 --out /tmp/claude-1000/-home-fy5090-code-singlefs/361bb28f-a369-4461-967b-925319d2cf5d/scratchpad/pt/tree`：清单 4 份，倒推出 3 份、sha256 对上 3 份；`.claude/kb/checks-owed.md` 倒推不出（快照之后别的会话改过工作区那一份）。腿引它的行逐处人工核（第六节）。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-publish-trigger-r1-sonnet-output.md research/prompts/m4-publish-trigger-r1-opus-output.md --root <上一条的树> --root /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --background research/prompts/_m4-publish-trigger-r1-background.md`，末行「✓ 核了 124 处引文、4 处裸行号，对上 113 处，没判 15 处（其中裸行号没有引文可比 4 处；逐处列在上面）」，没有对不上的。
- 攻方模型复跑：拷到 `~/.local/share/singlefs/work/m4-publish-trigger-r1-verify/opus-model/`，经 `multi-host-run.sh --peer --memory-cap 3G` 在第二台跑 `tools/rerun.sh`，退 0，三份输出 `reference_filesystem_publish_points.out`、`test_only_switch_in_a_release_binary.out`、`idle_trigger_model.out` 都 `equal=yes`，末行「name=rerun_summary files=3 differing=0」。
- 正推模型复跑：第二台那一趟退 101，原因是工作目录落在第二台那棵仓树里、cargo 把 RefFS 拷贝认成工作区成员（`refs_cargo.txt`「current package believes it's in a workspace when it's not」），不是模型的问题；python 那几份输出（`cases.sha256`、`count_assumptions.txt`、`model_dirty.txt`、`model_firstcross.txt`、`model_window.txt`、`refs_cases_summary.txt`）与模型 `out/` 逐份相同。真 RefFS 那一步改在本机仓外目录重跑，结果见第六节。

## 二、逐格判定

| 格 | 两条腿 | 判定 |
|---|---|---|
| P1 时间那一支 | 正推推荐：`accept` 进门判一次，加调用方按周期调的 `publish_when_due(now)`，钟由调用方给、复用 `CallerMonotonicClockReading`；守门「待发非空才判」；丢窗口式子加调用方轮询周期 P 与发布耗时 d。攻方量过（计数模型）：只在进门判时空闲丢窗口无界；定时入口的窗口还要加 tick 周期、调用方线程上排在前面的长调用、钟回拨量；照字面式子脏字节为 0 时也触发，空闲时每 T_time 一次空发布，S = 8 时 120 秒把回退候选集挤成 1 个 | **采正推的入口形并收进攻方三条**：触发 ⟺ 待发非空 ∧（时间支 ∨ 脏量支）；时间由调用方给单调读数，读数倒退时当作没过时间（不回拨）；丢窗口式子写成 T_time + P + d + 调用方线程上排在前面那次调用的时长，后两项在里程碑四是调用方的责任、条款里点名 |
| P2 脏字节口径 | 正推推荐折叠后待发数据单元字节；攻方打中三处（只改元数据的不计、与 min(字段, 环长÷F) 量纲不合、批内同一单元覆盖写 K 次），提按 journal 字节算 | **用户 2026-10-26 定「按待发改动要写的 journal 字节算」**：与夹取式同量纲，元数据改动也计入；D16 已定项 5「它就是脏页内存预算本身」那一句跟着改。批内覆盖写 K 次按 journal 字节计 K 次的记录（每次改动的记录照实算），不折叠 |
| P3 一次发布里攒什么 | 正推：数据写类可并（同 inode 折叠、建 inode 求和），删除类、截断、快照、整理单发并当屏障；新不变量「已 accept 的改动必须发布得出去」。攻方打中四处：删除类步并进别的改动违 D8 已定项 16 第 2 条；攒批里没发布的 unlink 现行那一版看不见、同批接着写会写活；整理的 G8 第三样保证锚在「一次调用发成」；一批的条数上限与准入只在发布时判 | **采正推的分类，攻方四处都收成条款**：unlink、截断、建删快照、整理、回退、卸载是屏障，进门先排空待发、自己单发；数据写类同 inode 折叠、建 inode 求和；`accept` 进门先做这一批合起来的准入与条数预判，过不了就在任何读写之前拒（不在返回成功之后拒）。正推交的待核前提「accept 进门能否做不拷分配器的只读预判」进实现的第一步现核，做不到停下报主 agent |
| P4 返回即落盘 | 两腿一致：要有立即排空入口；原入口不动、另加新入口。攻方量过：34 条崩溃枚举流 33 条直接调事务层、只有 1 条经会话，「现有崩溃流仍绿」不是攒批的证据 | **采**：原 `publish_user_change` 语义不变（多一条先排空待发），另加 `accept`、`publish_when_due`、`publish_pending_now`；新加经会话、带攒批的崩溃流（攒了几个请求还没发布时崩、自动发布中途崩），随机历史、模型对拍、RefFS 翻译三处的「返回即落盘」假设改成按会话报告的发布点判 |
| P5 RefFS | 正推量过（真 RefFS，32152 条）：今天的接法 3501 条合法恢复态判红、2084 条该红的放过；翻译把会话报告的每次发布翻成显式发布点后两个数都是 0，RefFS 本体不改。攻方量过（真 RefFS 代码）：一次请求被触发切开的那几格要按单元键序前缀才 0 红；自动发布写了之后报错、攒批又收下改动时 RefFS 判不了 | **采正推的接法**，条件是触发只在 `accept` 收下一个完整改动之后或 `publish_when_due` 里判，一个请求不被触发切开（攻方那 4 格的前提不出现）；发布写失败冻结之后 `accept` 在任何读写之前拒（P7），攻方那一形不出现 |
| P6 只供测试的开关 | 正推：照整理开关、删除开关的先例装在分配器上、只住内存，产品入口恒取有效值。攻方量过（release 编译）：今天那几个「只供测试」的开关是 pub 成员加 pub 入口、没有 cfg 门，发布构建的普通 bin 够得着，会话持有者一句话就能关掉空间准入；阈值开关照抄就是产品可达的第二条配置通道 | **阈值开关放进默认关的 cargo 特性**（只在测试与 checker 档打开），不照抄今天的 pub 写法。今天已有的那几个开关同一个问题，另立一条欠账，不在这一行改 |
| P7 串行下不立的 | 两腿一致：C425 原形串行下走不到、实例切换没有对象，留里程碑六；里程碑四补两条：守门（同 P1）与冻结时 `accept` 拒 | **采** |

## 三、交用户

| 项 | 形态 | 被攻过几轮 | 去向 |
|---|---|---|---|
| 三个新入口、触发式带「待发非空」、单调读数不回拨、窗口式子加 P、d 与排队时长 | 正推提、攻方补 | 零轮 | 写回 D16 已定项 2、5、10，进实现 |
| 脏量按 journal 字节 | 用户定 | 零轮 | 写回 D16 已定项 5、C348，进实现 |
| 屏障分类、进门预判准入 | 两腿合 | 零轮 | 写回 D16 已定项 10，进实现 |
| RefFS 按会话报告的发布点、请求不切开 | 正推提 | 零轮 | 进实现 |
| 阈值开关进默认关的特性；已有开关另立欠账 | 攻方提 | 零轮 | 进实现；欠账写进 `checks-owed.md` |

## 四、写回与后续

- 写回（kb-scribe，等这一批推论轮判完一起写）：D16 已定项 2 触发式加「待发非空」；已定项 5 脏量口径改成 journal 字节、丢窗口式子加 P、d 与排队时长、读数倒退不回拨；已定项 10 写屏障分类与进门预判；C348 改写；新立一条欠账「只供测试的开关在发布构建里够得着」（今天的整理、删除、空间准入开关）；里程碑四待优化表那一行写成「判决 m4-publish-trigger-r1」。
- 实现（implementation-writer，等 GPU 垃圾回收第 39 批交回、`mounted_session.rs`、`transaction.rs`、`admission.rs`、`mount.rs` 放出来；排在 m4-multi-inode-size-r1 与 m4-partial-write-r1 之后，因为它改全部改动共用的发布节奏）：第二节各格；新崩溃流照 P4。

## 五、复跑

见第一节第三、四条与第六节。

## 六、人工核的引文与正推真 RefFS 那一步

- 正推模型本机复跑：`bash scripts/hosts/run-with-memory-cap.sh 3G bash ~/.local/share/singlefs/work/m4-publish-trigger-r1-verify/sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work ~/.local/share/singlefs/work/m4-publish-trigger-r1-verify/sonnet-rerun`（工作目录在仓外），退 0，末行「name=rerun_summary mismatches=0」，真 RefFS 那一步与 python 各份都 `same_as_stored`。
- 腿引 `checks-owed.md` 的五处（`:249` C348、`:297` C422、`:299` C424、`:300` C425、`:459` C733）在此刻工作区仍是这五个编号的那一行，引法与行相合。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 2 | 支撑 | 2026-10-26 改了：触发式加待发非空，并进写回批 |
| D16（发布语义） 已定项 5 | 推翻 | 2026-10-26 改了：脏量口径照用户定改成 journal 字节，丢窗口式子加项，并进写回批 |
| D16（发布语义） 已定项 10 | 支撑 | 2026-10-26 改了：屏障分类与进门预判，并进写回批 |
| D16（发布语义） 已定项 11 | 不影响 | 2026-10-26 不受影响：对外 fsync 留里程碑十，会话层的立即排空入口与它同义 |
| D16（发布语义） 已定项 14 | 不影响 | 2026-10-26 不受影响：建删快照是屏障、单发，快照发布点仍在 checkpoint 全部数据变更之后 |
