# m4-exit5-relocation-proof-r1 判决：出口 5 的证明罩到 core 搬迁执行（主 agent，2026-10-26，第一轮）

<!-- doc-lint:not-numbers X1 X2 X3 X4 G7 N6 P4 -->

被判的条款：里程碑四出口 5（`.claude/kb/milestone/04-fourth-txn.md` 出口表第 5 行）；判决 `research/prompts/m4-gc-r1-main-verification.md` G7；`.claude/kb/decisions/13-验证路线.md` 已定项 1、5、13、14；`.claude/rules/verification.md` 两句。用户 2026-10-26 定这一出口「要罩到 core 搬迁执行」。冻结副本 `~/.local/share/singlefs/work/m4-close-rounds-frozen/`。

两条腿：正推 `research/prompts/m4-exit5-relocation-proof-r1-sonnet-output.md`（sha256 8595fa2269fa8916cc4cfd1a794b8e5f680b8114f0b0042691cae5a0d90146a3，模型目录 `research/prompts/m4-exit5-relocation-proof-r1-sonnet-model/`），攻方 `research/prompts/m4-exit5-relocation-proof-r1-opus-output.md`（sha256 96e4627cdb850ff34a83e5f4fd5d4a69edd18b3632ce077f60172dbbdd30c975，模型目录 `research/prompts/m4-exit5-relocation-proof-r1-opus-model/`）。

## 一、核查

- 开工快照：`python3 scripts/agents/snapshot-tree.py m4-exit5-relocation-proof-r1 --out /tmp/claude-1000/-home-fy5090-code-singlefs/361bb28f-a369-4461-967b-925319d2cf5d/scratchpad/x5/tree`，「清单点名 2 份，倒推出 2 份，sha256 对上 2 份」。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-exit5-relocation-proof-r1-sonnet-output.md research/prompts/m4-exit5-relocation-proof-r1-opus-output.md --root <上一条的树> --root /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --background research/prompts/_m4-exit5-relocation-proof-r1-background.md`，末行「✓ 核了 44 处引文、85 处裸行号，对上 36 处，没判 93 处（其中裸行号没有引文可比 56 处；逐处列在上面）」，没有对不上的。
- 两条腿的模型复跑：拷到 `~/.local/share/singlefs/work/m4-exit5-relocation-proof-r1-verify/`，经 `multi-host-run.sh --peer --memory-cap 8G` 在第二台跑各自的 `tools/rerun.sh`；结果见第五节。复跑核到之前，判决里引的数都标「腿量的」。

## 二、逐格判定

| 格 | 两条腿 | 判定 |
|---|---|---|
| X1 抽象文件系统状态 | 正推推荐 `gc_crash_proofs` 的 `logical`（P4 是它当前视图那一支），O1 的 POSIX 可见内容留给后半句的对拍；限制：同一发布重写派生态的角色换新 key，恒等只能说被搬的 key。攻方打中（腿量的）：带管理员回退、抬 F 的历史上「带 GC 与不跑 GC 逐项相同」不成立，三种定义一起中，病根是 G7「整理不改用户看得见的东西」——整理的发布占根环槽、把旧用户版本挤出环外，搬根那一批算一个新状态、抬 F 的上限跟着挪；另打中 P4 的目的地前提只要求起点槽号互异、Verus 证出两槽目的地罩住一槽单元也过 | **攻方两处挂起，进第二轮**：恒等性质的主语要连「回退能退回的版本集合」一起说，G7 重写；P4 目的地前提改成按区间不重叠。抽象状态取 `logical` 并补「回退目标视图」一类，第二轮攻这一形 |
| X2 罩到哪一层 | 正推：最弱读法是「被证明的判定由 core 真调」，今天连它都没做到（核内 12/18057 行）；最强是 core 直验加盘面。攻方打中：只按 `publish_relocation_batch` 划界，漏了 GC 真改抽象状态的回退路径（判决 m4-segment-code-r1 的 T5 / F1 在 `mount.rs` 的回退路径上） | **用户 2026-10-26 定「core 直验，连胶水一起证」**。射程连回退路径（攻方 X2）一起算 |
| X3 办法与代价 | 正推写了原型核（条目判定、目的地判定、写序前缀定理），直接起 verus 88 → 102 verified、0 errors（腿量的）；core 直验今天走不通：闭包 836 个 fn、下界 ≥ 836 条义务，`Option::copied` 要 `assume_specification`，判法禁它、只认一个 crate。攻方打中：`gc_crash_proofs` 协议里没有持久的回退目标视图，Verus 证出「搬迁、checkpoint 之后立刻回收覆写源槽」每步合法、定理照样成立而旧版映射读坏；核外提第四样、执行模型对拍、前提运行时核共用这份协议当 spec，一起中 | **攻方挂起，进第二轮**：协议先补持久回退目标视图与多逻辑版本；第二轮照用户定的层攻「core 直验」的做法、射程、判法改动与代价 |
| X4 条款怎么改 | 正推给 G7 改写句、出口 5 追加句、D13 新立已定项草稿；攻方给改写句必须交代的四样（版本怎么对应、回退窗口与 F、协议补持久回退视图与多逻辑版本、目的地按区间不重叠） | 第二轮定 |

## 三、交用户

| 项 | 形态 | 被攻过几轮 | 去向 |
|---|---|---|---|
| 层：core 直验连胶水 | 用户定 | 零轮 | 第二轮 `m4-exit5-relocation-proof-r2` 攻它的做法与代价 |
| G7 重写、协议补回退视图、目的地按区间 | 攻方提 | 零轮 | 第二轮 |

## 四、写回与后续

- 这一轮不写回；第二轮判完一起写。
- 第二轮正文由主 agent 写：照用户定的层，问 core 直验怎么接（vstd 依赖、`verus!` 包法或 cfg 注解、哪些 std 规格算可用、门禁只认一个 crate 与禁 `assume_specification` 两条怎么改）、射程（发布闭包加回退闭包）、分阶段与代价（先证哪几个函数、义务数怎么现数）、罩不到的（设备、校验和只能当未解释函数）、攻方三处怎么收。

## 五、复跑

第二台没有 Verus，两份复跑在那里退 1，不算结果；改在本机仓外目录跑：`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=7200 bash scripts/hosts/run-with-memory-cap.sh 8G bash ~/.local/share/singlefs/work/m4-exit5-relocation-proof-r1-verify/<腿>-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work ~/.local/share/singlefs/work/m4-exit5-relocation-proof-r1-verify/<腿>-rerun`。攻方那一份末行「name=rerun-compare out/summary.txt same=yes」（含 baseline 32 verified、攻方反例 36 verified、两个必红变体各 1 errors 那四行）；正推那一份末行「name=rerun_result mismatches=0」。第一节里标「腿量的」的数据都复出了。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 备料 | 2026-10-26 不受影响：第一轮只挂起，第二轮判完再回看 |
| D13（验证路线） 已定项 5 | 不影响 | 2026-10-26 不受影响：core 直验不让 checker 调 core |
| D13（验证路线） 已定项 14 | 不影响 | 2026-10-26 不受影响：校验和照旧当未解释函数 |
