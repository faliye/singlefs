# m4-c145-destruction-bound-r1 判决：未完成销毁意图的区间上界在第一版怎么判（主 agent，2026-10-26，第一轮）

<!-- doc-lint:not-numbers B1 B2 B3 -->

被判的条款：`.claude/kb/checks-owed.md` C145 第二条判定「未完成意图的区间里没有纪元 ≥ 上界的条目」；`.claude/kb/decisions/05-快照-空间记账机制.md` 已定项 12；`crates/singlefs-checker/invariants/05-空间记账.md` 的 I-3.5、I-3.6；d5-item12 三轮判决。用户 2026-10-26 定这一条开推论轮（同一次弹窗另定第一条并进 I-3.6、第三条加断言并留档）。冻结副本 `~/.local/share/singlefs/work/m4-close-rounds-frozen/`。

两条腿：正推 `research/prompts/m4-c145-destruction-bound-r1-sonnet-output.md`（sha256 82feb4694071f0f82b8e60a27993ee2a60794564834e963255a17dcee30699dc，模型目录 `research/prompts/m4-c145-destruction-bound-r1-sonnet-model/`），攻方 `research/prompts/m4-c145-destruction-bound-r1-opus-output.md`（sha256 2eb549d1ab1d69d3ab80d226c5682b0bbd5e232deb9d5b3e2958854fe97bcc15，模型目录 `research/prompts/m4-c145-destruction-bound-r1-opus-model/`）。

## 一、核查

- 开工快照：`python3 scripts/agents/snapshot-tree.py m4-c145-destruction-bound-r1 --out /tmp/claude-1000/-home-fy5090-code-singlefs/361bb28f-a369-4461-967b-925319d2cf5d/scratchpad/c145/tree`：清单 4 份，倒推出 3 份、sha256 对上 3 份；`.claude/kb/checks-owed.md` 倒推不出（快照之后别的会话改过工作区那一份）。腿引它的只有攻方第 111、125 行两处 `checks-owed.md:121`，人工核：此刻工作区第 121 行是 C145 那一行，引法相合。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-c145-destruction-bound-r1-sonnet-output.md research/prompts/m4-c145-destruction-bound-r1-opus-output.md --root <上一条的树> --root /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --background research/prompts/_m4-c145-destruction-bound-r1-background.md`，末行「✓ 核了 31 处引文、54 处裸行号，对上 28 处，没判 57 处（其中裸行号没有引文可比 52 处；逐处列在上面）」，没有对不上的。
- 两条腿模型复跑：拷到 `~/.local/share/singlefs/work/m4-c145-r1-verify/`，经 `multi-host-run.sh --peer` 在第二台跑各自的 `tools/rerun.sh`；结果见第五节。

## 二、逐格判定

| 格 | 两条腿 | 判定 |
|---|---|---|
| B1 等价论证 | 两腿都没找到反例。正推三条互不共用论证的路子：代码论证（条目纪元只有一处取值、值是上一版活头前驱；活头前驱只在建快照与写意图两处变，二者在意图挂着时都被拒，`transaction.rs:9450`、`:9527`）、纯 Python 小模型（5957 个状态，三种读法差 0；两道拒绝都拿掉分叉 5 例）、真写路径探针（535 个历史、197 条待放候选，纪元 ≥ 写意图 txg 的 0 条；拿掉 `transaction.rs:14131` 那一支真 checker 红 47 次、红在 I-9.25）。攻方：四种判法在模型两种调度深 10 步上 0 个状态不同，副本 9 段删快照 49 版每一版最大纪元都小于意图 txg；拿掉守卫后 8486 个状态出现纪元 ≥ 意图 txg 的条目、380 个状态两种判法不同 | **成立，但承重点不是 checker 注释那一段**：两腿一致指出撑住结论的是 core 那两道拒绝（意图写下之后、最后一批之前不准建快照与发第二个销毁意图）。checker 注释（`walk.rs` 里 `entry_is_pending_release` 上那三行）只论证了「之后新进来的」条目，core 注释「中间都不杀用户可见单元」那一句不是承重的那一条（攻方实跑过批与批之间插覆盖写）；两处注释照实改写。条款里 hi 的三种写法（D5 已定项 12 正文两处与 core 实际传的这一批 txg）照「下一侧是头时上界取被判那一版自己的 checkpoint txg」统一 |
| B2 第一版的形态 | 正推推荐：并进 I-3.6 的豁免句与 I-3.5 逆向共用的待放集谓词，下一侧是头时 hi 取被判这一版根记录的 checkpoint_txg，零字节；补丁在副本上试过（新单测过、改回 ∞ 红）。攻方打中三处：以 ∞ 为上界时这一条判定是空真（纪元抬高 10⁶ 的坏镜像在意图那一版 0 条违例）；C145 登记的判别力自证「把上界改成已发布的号」在实现里是等价变体（四个测试目标全绿）；缺「纪元 > lo 的待放条目」那一种不许红的世界（把待放上界收成 lo + 1，已有用例全绿） | **采正推的形，攻方三处收成验收**：上界取被判那一版的 checkpoint txg（攻方「另判纪元 < V」与它同形）；必红世界换成「纪元抬到本版 txg 以上」那一种（∞ 判法放过、新判法红）；判别力自证换成「上界改回 ∞ 必红」，不用登记的那一句；补「纪元 > lo 的合法待放条目」那一形不许红的世界。GPU 扫描内核 `s_entry_is_pending_release` 同改，内核侧本版 txg 的来源由实现现核 |
| B3 不成立时要加什么 | 两腿一致：第一版不用加格式；正推给条件式答案（码 14 树表条目预留区存 8 字节），攻方指出串行放开之后「意图里存 hi = 意图 txg」是错的量，要配 core 在意图那一刻定死 hi | **第一版不加格式**。两腿给的条件式答案写进 D5 已定项 12 射程：放开并行销毁或放开意图挂着时建快照时，这一条要重开 |

## 三、交用户

| 项 | 形态 | 被攻过几轮 | 去向 |
|---|---|---|---|
| 上界取被判那一版的 checkpoint txg，并进 I-3.6 与 I-3.5 逆向的待放集谓词 | 两腿同 | 零轮 | 进实现（与 C145 第一条同一段 walk.rs 一起），写回 D5 已定项 12、C145 |
| 两处注释照实改写，条款 hi 的三种写法统一 | 两腿同 | 零轮 | 进实现与写回 |

## 四、写回与后续

- 写回（kb-scribe）：D5 已定项 12 正文 hi 那两句与射程；I-3.6 判别句；C145 第二条收成「已判」，三条都有落点后销账。
- 实现（implementation-writer，排在多 inode 实现员交回之后，因为同改 `walk.rs`）：CPU 与 GPU 两侧待放集谓词、必红与不许红的世界、变异行；并进 C145 第一条（并进 I-3.6）与 C300 checker 一半、C410 已发布谓词扫描侧另写一份那一簇。

## 五、复跑

两条腿的模型目录拷到 `~/.local/share/singlefs/work/m4-c145-r1-verify/<腿>-model/`（拷完 `diff -rq` 与 `research/prompts/m4-c145-destruction-bound-r1-<腿>-model/out` 相同），在第二台经 `bash scripts/hosts/detached-run.sh start … -- bash scripts/hosts/multi-host-run.sh --peer --memory-cap <8G|5G> --send <冻结副本> c145-frozen --send <模型目录拷贝> c145-<腿>-model -- bash c145-<腿>-model/tools/rerun.sh --repo . --frozen c145-frozen --work /tmp/c145-<腿>-rerun` 跑，两趟都退 0：

- 攻方（`m4-c145-r1-opus-rerun-peer`）：`name=model_output_matches` 三份（`model-product-transaction-depth10.jsonl`、`model-unguarded-depth9.jsonl`、`model-summary.txt`）、`name=exit_codes_match file=exit-codes.txt`；伪造纪元续做那一形照报 `name=attack_forged_epoch_continuation txg=9 snapshots_left=0 deadlist_entries_left=4 violations_after_the_continuation=2`，两条违规是 I-3.5 与 I-3.6，与报告相同。
- 正推（`m4-c145-r1-sonnet-rerun-peer`）：rerun.sh 拿这一趟的输出与模型目录 `out/` 逐行比，12 份（`model-baseline.txt`、五份 `model-mutant-*.txt`、`model-stray-entry.txt`、`probe.txt`、`probe-patched.txt`、`patched-unit-test.txt`、`control-unit-test.txt`、`control-core-mutation.txt`）都打「逐行相同」，两条控制都照红。

两份报告里判决引的数都复出。模型目录拷贝留到这一题不再开下一轮时删。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D5（快照 / 空间记账机制） 已定项 12 | 支撑 | 2026-10-26 改了：hi 统一取被判那一版的 checkpoint txg，射程写重开条件，并进写回批 |
| D15（格式冻结政策） 已定项 4 | 不影响 | 2026-10-26 不受影响：第一版不加格式 |
