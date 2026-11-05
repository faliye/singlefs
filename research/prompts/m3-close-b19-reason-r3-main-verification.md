# m3-close-b19-reason-r3 判决（第三轮，最后一轮，2026-11-03）

正文 `research/prompts/_m3-close-b19-reason-r3-body.md`。三件合一轮：reason-r2 的第三轮、批 19 对 `crates/singlefs-core/src/` 改动的代码轮、E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 与 E197（扫描 journal 粘连内存随设备数） 两份装置计数分配器的 unsafe 登记。腿：正推 `research/prompts/m3-close-b19-reason-r3-sonnet-output.md`（ebb498c5…，没建模型）；攻方 `research/prompts/m3-close-b19-reason-r3-opus-output.md`（08cfcf1f…，没建模型）；攻方第二样本 `research/prompts/m3-close-b19-reason-r3-opus-s2-output.md`（592be1fe…，模型 `research/prompts/m3-close-b19-reason-r3-opus-s2-model/`）。两条腿在 Q1–Q3 几问上报「造不出」，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」用同一份提示派了第二样本。

被判的代码文件（开工快照 `research/prompts/m3-close-b19-reason-r3-snapshot/crates-sha256.txt`）：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/mounted_session.rs`（只改 `//!` 文档注释）。

## 一、核查

- 开工快照倒推：任务 5549 `python3 scripts/agents/snapshot-tree.py m3-close-b19-reason-r3 --out ~/.local/share/singlefs/work/m3-close-b19-reason-r3-main/m3-close-b19-reason-r3-tree --repository <仓根>`：「清单点名 44 份，倒推出 42 份、sha256 对上 42 份；倒推不出 2 份」。倒推不出的是 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md` 与 `.claude/kb/experiments/173-上限卡在读坏槽的三种处置.md`：腿跑着时这一批写了 E183 第二段、在 E173 第 34 行一行之内补点了四份分臂 tsv，违了「腿跑着时主 agent 不改这些文件」。三份腿报告引这两页共 5 处：E183 页 0 处（引的是登记 `research/prompts/e183-r2-prereg.md`）；E173 页第 359 行（正推第 170 行、攻方第 151 行）人工核过，原文此刻逐字在，第 34 行的改动不挪行号。
- 引文：`python3 scripts/agents/cite-check.py <三份腿报告> --root <倒推的树> --background research/prompts/_m3-close-b19-reason-r3-background.md`：「核了 178 处引文、95 处裸行号，对上 122 处，没判 151 处（其中裸行号没有引文可比 72 处）」；同一条命令 `--root <仓根>`：「核了 178 处引文、95 处裸行号，对上 178 处，没判 95 处（其中裸行号没有引文可比 95 处）」。0 处对不上。
- 复跑：第二轮辩方模型拷进 `~/.local/share/singlefs/work/m3-close-b19-reason-r3-main/r2-sonnet-model` 交 `tools/rerun.sh`（任务 5550）：九件 `_output_identical`。正推腿点的缺格 `p_rootfail = 0.03` 补跑 `today_both_fixed`：S = 4「cfg=today … red=216 partial=1292」「cfg=today_both_fixed … red=216 partial=1239」（5551），S = 8「cfg=today … red=246 partial=955」「cfg=today_both_fixed … red=246 partial=597」（5552）。攻方第二样本模型拷进同一草稿目录交 `tools/rerun.sh`（5603）：「E7RESULT name=zero_slot_merge_blind S=4 histories=57 drop_writer_exceeds_the_literal_ceiling=39 checker_holds_on_those=39」、S = 8「histories=385 … =343 … =343」、S = 16「histories=1905 … =1815 … =1815」、「E7RESULT name=first_example S=4 hole_txg=3 raise_at=8 drop_writer_floor=4 literal_ceiling=3 checker_ceiling=4」；模型目录里没有冻结输出可比（`result=no_reference_in_the_model_directory`），数只在这一次复跑上。
- 现读坐实（量的）：
  - 第五判 `crates/singlefs-checker/src/walk.rs:10592`–`10596` 只跳过两条都已释放的那一对，比的是「至少一条未释放」的每一对，不看是不是被改记的记录。
  - `crates/singlefs-checker/src/walk.rs:10331` 树表指针全零交回 `tree_table_entries: 0`，`walk.rs:10770`–`10775` 那一支报不适用；`crates/singlefs-checker/invariants/05-空间记账.md` 第 28 行 I-3.21 的前置是「最新根的树表带种类 3 的分配记录树、全部读得出」。
  - `crates/singlefs-harness/src/history.rs:5141`–`5165` `judge_one_floor_raise_pushed_by_the_session` 不看 `floor_of_the_publishes`，`crates/singlefs-harness/src` 下 grep `floor_of_the_publishes|FloorOfThePushedPublishes` 0 处（只在 `tests/` 的推空用例里）；`crates/singlefs-harness/src/model.rs:8790` 按上限逐字比。
  - `crates/singlefs-checker/src/walk.rs` 第 12665 行附近文档注释 ③ 写「txg ≥ 最新根带的 F」，攻方第二样本说调用处传的是 F_生效（推的，调用处这一轮没逐行核）。
  - 冻结输出 `research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt`：「cfg=today … partial=976」「cfg=today_genesis_only … partial=974」「cfg=today_both_fixed … partial=604」。第二轮判决写的「合法部分没判降 370」是从 today_genesis_only 量起的，从 today 量起是 372；S = 4 同文件 `q3-admission-4-128.txt`「cfg=today … partial=1348」「cfg=today_both_fixed … partial=1293」，降 55。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| Q1 第 1 问　措辞更正（至少一条未释放就比） | **站得住（三份一致）**：判决字面「未释放对未释放」抓不到判决自己的隔离尾槽验收镜像，`05-空间记账.md` 第 8 行与 `walk.rs:10595` 已按「与别的记录（未释放或已释放）相交」写 | 三份腿报告 |
| Q1 第 1 问　合法镜像误红、两条都已释放的漏判 | **造不出（攻方两份样本与正推一致）**：第五判只在 I-3.18 红的盘面里进（`walk.rs:21691`、`10523`），两条都已释放的相交由 I-5.4 判 | 三份腿报告 |
| Q1 第 2 问　与 I-5.4 重复报 | **重复、不矛盾（三份一致）**：两种读法都相交的那一对 I-5.4 与第五判各报一次；只在 I-3.18 已红的盘面上出现。攻方第一样本给的收窄（只比含被改记记录、相交落在改记多罩那几槽上的一对）是零轮形态，不采：条文 `05-空间记账.md` 第 8 行明写比到「别的记录」，收窄要连条文改，换来的只是红的集合少一项。违例文字在两条都没被改记时仍写「改记之后相交」（攻方第二样本）：文字改成按实际情形说，随下一批 | 攻方两份第 Q1-2 节；正推 Q1 第 2 问 |
| Q1 第 2 问外带　第二轮判决「不算病」的 18 个隔离形照样判红 | **推的（攻方第二样本，没造盘）**：两条都带隔离记号的改宽相交今天由第五判判红，与第二轮判决第二节 Q1 第 1 问那一格「改宽罩进隔离豁免的槽（18 个）两条记录都带记号、不复用，不算病」不一致；只在 I-3.18 已红的盘面上。零轮，登欠账 | 攻方第二样本 |
| Q1 第 3 问　部分没判说明里的数 | **能分开（三份一致）**：改记后相交走红的那一支，说明只在相交为空时才写（`walk.rs:21740`） | 三份腿报告 |
| Q2 第 1 问　「0 条不适用」 | **更正站得住；字面打中一处（攻方第二样本，现读坐实）**：树表指针全零也落进「0 条」那一支、报不适用（CPU `walk.rs:10331`、`10770`；GPU 内核同形，攻方第二样本点 `crash_judge_checker_scan.wgsl` 第 2251、3103 行），条款前置把它算读不出、应报部分没判。最新根这时另由走读失败判红（正推引 `walk.rs:2698`），镜像照红，差在 I-3.21 这一格的结论。**定（零轮）**：全零指针改报部分没判，CPU 与 GPU 同改，随下一批 | 现读；攻方第二样本 Q2-1；正推 Q2 第 1 问 |
| Q2 第 1 问外带　「树表 0 条、写过行」那一版上写行发布漏记自己新写的单元 | **推的（攻方第二样本）**：I-3.21 不适用、I-3.1 没有记账可比，没有别的不变量罩着。零轮，登欠账 | 攻方第二样本 |
| Q2 第 2 问　记录集合措辞（E(θ_high)，不是「未释放」） | **站得住（正推另点出）**：判决「未释放记录」字面会在有覆盖写的合法历史上误红，实现与条文取 E(θ_high) | 正推 Q2 第 2 问 |
| Q2 第 2 问　合法误红、C944 形漏判 | **造不出（三份一致）**：走到的版本 txg ≥ θ_high，换下的单元释放代都高于 θ_high（攻方 `walk.rs:20962`、`12715`、`9826`）；记录过滤与第二轮攻方模型 `c944_starts` 同式（推的，1195 形没在实现上复跑）。文档注释 ③ 与调用处的 F 口径不一致（推的），注释随下一批 | 三份腿报告 |
| Q2 第 3 问　两形比例 | **合法写者写不出（三份一致，推的）**；崩溃点重放各族 n/a 占比没量 | 三份腿报告 |
| Q3 第 1 问　第 0 代根两处改法 | **今天的 Rust 与第二轮辩方 `today_both_fixed` 逐支同语义（正推、攻方第一样本读码对出，没拿端口对拍 Rust）**。数照复跑：坏写者红 +7（S = 4）、+3（S = 8）；合法部分没判降 55（S = 4）、372（S = 8；第二轮判决写的 370 是从 today_genesis_only 量起）；`p_rootfail = 0.03` 档红不变（216、246），部分没判降 53、358。合法环变红：**造不出（三份一致）** | 一节复跑 |
| Q3 第 2 问　I-7.9 不改 | **结论站住，理由要换（攻方两份样本打中，正推认原理由）**：写者把读坏、全零槽并进上限只会压低上限，合法写者不会因此被判红（三份一致）。但实现员那句「按条款式子本来就不进上限」字面不对：`.claude/kb/decisions/16-发布语义.md` 第 38 行正是把这两类槽并进上限。反过来，不并全零槽的坏写者越过字面上限而 I-7.9 判成立：攻方第二样本模型在 S = 4 / 8 / 16 上 39 / 57、343 / 385、1815 / 1905 段历史都是这一形（模型上量的）。读坏那一半 checker 看不到挂载那一刻的读数（判别子观测不到）；全零那一半镜像上看得到（攻方第二样本），能不能照写者的「能住的最大 txg」并进 checker 的上限是零轮形态，登欠账 | 攻方两份 Q3-2；正推 Q3 第 2 问；一节复跑 |
| Q4 第 1 问　全零并 | **相符（三份一致）**：与读坏槽同一个函数、同一个窗口锚；回收门槛的值没变 | 三份腿报告 |
| Q4 第 2 问　推空 | **相符（三份一致）**：「已推 + 1 + 1 > 8」（`mount.rs:4810`）与 E173 登记推空B 同形，带现行 F、txg 加一，写掉那一槽后下一轮走抬 F。两处只差次序不差结局：预算连一次推空都放不下时实现报预算用完、装置报抬不动（攻方第一样本）；放得下推空、放不下抬 F 那一串时先读环再报预算用完，读盘错会盖过它（正推）。两边都拒这次请求、盘上一样，不改 | 三份腿报告 |
| Q4 第 3 问　按槽认盘 | **准入那一处在真写者上与按 txg 结局相同（三份一致，推的）**：推抬 F 排在暖机之后（攻方 `mount.rs:8356` 在前、`8415` 在后），每块盘上都已有本实例的根。另有三处仍按 txg 认盘或认槽（`mount.rs:5044`、`5053`、`5275`），只在第 0 代根上与按槽不同、算出来的数相同。**定（零轮）**：三处一并改成按槽，与准入那一处同一个口径，随下一批 | 三份腿报告 |
| Q4 第 4 问　理想模型 | **打中（攻方两份、正推各一处，现读坐实一处）**：随机历史胶水把每一份抬 F 都交给模型比上限，不看 `floor_of_the_publishes`，推空那一份走到就假红（今天随机历史走不到推空，也一次都没对拍过推空）；故障注入「被吞的根槽写」的解释口径 `left_by_the_lying_device` 只认不并的上限，留下全零槽之后重挂再抬 F 会报假的不一致（推的）。**定（零轮）**：胶水按 `floor_of_the_publishes` 分开，推空那一份记「模型不答」；故障注入的解释口径认全零并与推空；随下一批，配会红的用例 | 现读；三份腿报告 |
| Q5　计数分配器 | **E183 离不开（三份一致）**：量的是一次调用前后线程本地的 Layout 字节，VmHWM 是页粒度，`Cargo.lock` 与本机缓存里没有安全计数库。**E197 判的数离得开（三份一致）**：判的是服务记的 cgroup 峰值，分配器只做互核与停机条款。**门禁今天两份都登不进（三份一致）**：放开表按 `lib.rs` 认根、`unsafe fn` 签名与 `unsafe impl` 在任何函数体之外、同名方法撞键。E183 的 `// SAFETY:` 说「计数是原子的」，加了线程本地计数之后只对一半（攻方第二样本），注释随下一批。登不登、怎么登交用户 | 三份腿报告 |

## 三、交用户的

| 岔路 | 候选 | 代价（腿报告给的，没量的标推的） |
|---|---|---|
| 两份装置的计数分配器 unsafe | 甲：改门禁那一格认 bin 根、`unsafe fn` 签名与 `unsafe impl`，两份各登 5 行、共 10 行，checker 档包根改 deny、补四条 `// SAFETY:` | 改门禁一格加样本；E197 那几行理由只能写成本理由 |
| | 乙：计数分配器挪进一个小 crate，两份装置引它，放开只登那一个 crate | 新 crate 一份、两份装置改引用（推的） |
| | 丙：E197 改登记不再用分配器（A7–A9 与停机条款 S3 改读 cgroup 峰值），只 E183 登 | E197 产物没出，改登记成本最低；E183 照样要门禁认 bin 根（推的） |

## 四、被攻过零轮的

第三轮之后停，不再开轮。这一轮新冒出来的零轮形态：Q1 违例文字按实际情形说；Q2 全零树表指针改报部分没判（CPU 与 GPU）；Q2 注释 ③ 口径；Q4 三处按槽认盘；Q4 胶水与故障注入解释口径认推空与全零并；E183 `// SAFETY:` 注释。前五样随下一批实现，配会红的用例；不另开轮，交用户时标「被攻过零轮」。登欠账的三样见第五节。

## 五、去向

| 编号 | 去向 |
|---|---|
| C937（I-3.1 照 I-3.18 改记槽数取宽的条件被攻过零轮） | 关：第三轮三份样本都造不出合法镜像误红与两条已释放的漏判，措辞更正站住；与 I-5.4 重复报不改 |
| C944（走到的单元起点没有自己的记录） | 关：I-3.21 第三轮三份样本都造不出误红与漏判；全零树表指针那一支改报部分没判随下一批 |
| C953（写者与 checker 对第 0 代根记在哪块盘不一致） | 关：写者准入那一处已按槽认盘，三份一致与条款相符；残留三处随下一批统一 |
| 新登记（checker 不把镜像上读回全零的根槽并进上限，不并全零槽的坏写者越过 D16 字面上限而 I-7.9 判成立） | 登 `.claude/kb/checks-owed.md`，依据攻方第二样本模型 5603 那三行 |
| 新登记（第五判对两条都带隔离记号的改宽相交照判红，与第二轮判决「不算病」不一致；违例文字在两条都没被改记时写「改记之后」） | 登 `.claude/kb/checks-owed.md`；文字那一半随下一批 |
| 新登记（树表 0 条、写过行那一版上写行发布漏记自己新写的单元，没有不变量罩着） | 登 `.claude/kb/checks-owed.md`，推的 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-11-03 不受影响：代码轮核全零并、推空与条款第 38、39、44 行逐句相符；I-7.9 照旧不改，全零那一半的 checker 加强登欠账 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-11-03 不受影响：这一轮没动按 journal 重建根环的前提 |
