# 根环实现第 10 批：代码轮第二轮判决（2026-10-05）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 Q3 Q4 R1 rr2 G4 -->

正文 `research/prompts/_m3-batch10-code-r2-body.md`；改动全文 `research/prompts/_m3-batch10-code-r2-diff.md`（第 10g 批一份补丁）；冻结副本 `/tmp/claude-1000/m3-batch10-code-r2/tree/`；三份腿报告：正推 `research/prompts/m3-batch10-code-r2-sonnet-output.md`（237 行，sha256 5bd33329…ebfd），攻方第一次抽样 `research/prompts/m3-batch10-code-r2-opus-output.md`（266 行，sha256 b24587df…9f01，模型 `-opus-model/`；原腿在客户端重启时被停，接续那一条写完），攻方第二次抽样 `research/prompts/m3-batch10-code-r2-opus-s2-output.md`（235 行，sha256 2c0ca9b3…c1be9，模型 `-opus-s2-model/`）。

## 一、核查

| 核了什么 | 结果 |
|---|---|
| 开工快照 | 冻结副本对 `research/prompts/m3-batch10-code-r2-snapshot/crates-sha256.txt` 312 份逐份相同（冻结副本没被改过）。主工作区在腿跑着时被别的会话改了快照里的文件：形式化接入那条线 32-1（8 份）、32-2（4 份，其中 `allocator.rs` 只改一段字段注释）、33（`crash_injection.rs` 等 4 份）、33-3（`walk.rs`、`checker_known_bad_images.rs`），删除与并发那条线第 22 批（12 份）、`checks-owed.md` 三行原地替换与两行插入。各次都留了合前原样（`/tmp/claude-1000/-home-fy5090-code-singlefs/3c4dae2e-4622-4601-9f3b-0db4b8e8ce93/scratchpad/b32-1-before/`、`b32-2-before/`、`b33-before/`，`/tmp/claude-1000/m3-batch22/before/`、`/tmp/claude-1000/m3-batch22/k3-before/checks-owed.md`），主 agent 逐份与快照比过 sha256 相同；腿引的行一律在冻结副本上核，不受这些改动影响 |
| 引文 | `cite-check.py`（`--root` 冻结副本、`--background`）：正推与攻方第一次合跑「✓ 核了 55 处引文，对上 55 处，没判 0 处」；攻方第二次「✓ 核了 26 处引文，对上 26 处，没判 0 处」 |
| 攻方第一次抽样复跑 | 主 agent 拷到自己的副本（`verify-r2-opus.sh`，副本已删）：mkfs 盘面 `roots={0: [0, 0], 1: [0]}`；rr2_on 在 mkfs 起点 `outcome=Ok(reread)`、按槽归盘那一臂 `Ok(first)`，无故障挂载后盘面哈希三臂相同；入库用例五个文件：rr2_on 全绿，按槽归盘那一臂只红 `the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread`，rr2 自己的三份全绿 |
| 攻方第二次抽样复跑 | 主 agent 照它的 `rerun.sh` 在自己的草稿目录（`/tmp/claude-1000/m3-r1/verify-r2-s2/`）经 `run-with-memory-cap.sh 4000M` 重跑：末行「比了 13 份，0 份不同」；用它的 `summarize.sh` 从重跑日志重抽，打中那几格原样：`pass1:dev1-all-e reread:c-Error \| rr2: n=4 red=4 iso0=4 refused=0 \| before: n=4 red=0 iso0=0 refused=0 \| merge: n=4 red=0 iso0=0 refused=0`（`c-Garbage`、`c-Zeros` 两行同数）；`pass1:dev1-all-e reread:all-e \| rr2: n=4 red=4 iso0=0 refused=4 \| before: n=4 red=0 … \| merge: n=4 red=0 …`；`pass1:dev1-all-e+c-e reread:- \| rr2: n=4 red=0 … \| before: n=4 red=4 iso0=4 refused=0 \| merge: n=4 red=0 …`；`no-fault` 三臂都 0 |
| mkfs 写第 0 代根的槽 | 主 agent 在冻结副本上现查：`crates/singlefs-core/src/make_filesystem.rs:619`–`627` 对 `genesis_root_ring_slots()` 每个槽按 `region_devices` 写到那一区所在的盘；`crates/singlefs-core/src/root_ring.rs:109` 的 `target_for_publish` 把 txg 0 落到区域 0 |

腿的数都在冻结副本的拷贝或攻方原型上，是副本上的数，不进 kb；判决只拿来分辨候选。

## 二、逐格判决

判据照正文第五节。「没打中」两次抽样都没打中才记「站住」。

### K1　Q3 拒因与它的复算

| 格 | 正推 | 攻方（两次） | 判 |
|---|---|---|---|
| 新拒因只在一处交回 | 一致：构造点只有 `mount.rs:1974`，读阶段择不到根仍报 `Recovery(NoValidRoot)` | 两次都没打中 | **站住**。正推另报准入抬 F 上限里 `mount.rs:2463`、`2506` 仍报同名 `Recovery(NoValidRoot)`（第一轮 H3 的残留，没造盘面）：记线索 |
| 复算判据与实现判拒同一个定义 | 一致：被抛弃谓词逐字同式（`recovery.rs:1540` 对 `mount_transparency.rs:2559`） | 两次都没打中 | **站住** |
| 没注读故障的合法镜像上复算为真（假红） | 一致（推的） | 第一次 18 张起点镜像、33 个层 0 崩溃态复算都为假；第二次推的 | **站住** |
| 实例表读不出时「不按表滤」（读法 ③） | 一致：两种挂载下这一支都走不到 | 两次都没打中（推的） | **站住**；条文写回时写明 |
| 第三截报红口径 | 没打中，既有口径 | 两次都判与 G4 同口径：只报判据复算为真，判据为假的误拒只计数，由第二截抓 | **站住**（既有设计） |

另：正推报 Q3 判官里「按表判被抛弃」那半句 harness 档没有用例也没有变异行（改成「表里有该实例的行就算被抛弃」5 条仍全绿；推的是合法盘面上走不到）。下一批补一行变异或写明它在合法盘面上走不到的留档用例。

### K2　rr2 与它的两个后果

| 格 | 攻方第一次 | 攻方第二次 | 判 |
|---|---|---|---|
| 触发与取 m 是不是同一处定义 | **打中（读法错）**：同一处 `newest_valid_root_txg_on_every_device`，它按 `target_for_publish` 把根归盘；mkfs 在三区槽 0 各写一份第 0 代根，公式把三份都归到盘 0，环里只剩第 0 代根时盘 1 实际读出了有效根，触发照样重读 | 没打中：两处调同一个函数 | **打中**（一次坐实即采：主 agent 复跑与现查都对上）。第二次没打中是它没去比公式与盘面 |
| 实现员读法 ①「新池与年轻的环每次可写挂载都重读」 | **打中（原因写错，射程写大）**：只在环里只剩第 0 代根时重读，A 段 13 个起点里 5 个，L1 33 个层 0 崩溃态里 12 个 | 与写的相同，射程量过：mkfs 之后第一次可写挂载与它在第 1–5 个写上掉电之后的下一次 | **打中**：两次射程一致，原因按第一次（落点公式归盘）。代码照槽归盘之后不再重读 |
| 有没有用例钉错 | **打中**：`the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread` 钉 `OnTheOneReread`、注释理由与盘面不符 | 一处名字与钉值不符 | **打中**；照槽归盘之后改回 `OnTheFirstRead` |
| 重读比第一遍坏：多拒（读法 ②） | 没打中：多拒的 20 条全是重读整环都坏那一形 | 与写的相同：rr2 臂 4 / 4 拒成 Q3 | **站住**（代价与写的一样大）；Q3 字面与岔路 R1 一起交用户 |
| 重读比第一遍坏：影子账 | 没打中 | **打中**：第一遍读出被抛弃根 C、重读只坏 C 的槽，C 掉出影子账、隔离 0，第一次覆盖写就复用 C 的单元，池级 checker 红 I-7.4；rr2 臂 12 / 12 段红，rr2 之前与「按槽并两遍」两臂 0 / 12 | **打中**（一次坐实即采：见第一节复跑）。违反 `.claude/kb/decisions/23-journal的角色与格式.md:390`「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」；C554 同族、rr2 新开的进入法（C 的槽第一遍读得出，只坏重读那一遍） |
| 重读比第一遍坏：回收门槛、抬 F 上限越过有效根 | 没打中：B 段 865 条臂、L2 8223 个层 0 崩溃态 × 9 条臂，checker 0 违例 | 没打中：36 段三臂不红、F 同数 | **站住** |
| 重读之后三样同一遍读数 | 没打中 | — | 站住（同一遍；正是影子账那一格的病根：同一遍丢了第一遍读到的根） |
| rr2 用例「丢盘整读与崩溃之前相同」 | **打中（判别力缺口）**：两个起点丢任一块盘后都没有活文件，比的是两个空表 | — | **打中**；下一批换一个丢盘后有活文件的起点 |

### K3　两处覆盖用例（正推一条腿）

一致：变异 2772、2773 各红自己那条（14 对 12；2392064 对 2260992，红在门槛），门槛里的 max 项另有用例钉着。实现员报的「槽 1088、释放代 36、F=24」正推复核不了（它造的盘面 F=24 被拒）：不进 kb。

### K4　checker 档装置

| 格 | 判 |
|---|---|
| 第三截与第二截口径 | 两次攻方都判同口径（与 G4、N-配置 一样只报判据复算为真）：站住 |
| 读序号后挪一遍根环 | 两次都是线索。主 agent 在 10g 合后跑过 checker 档库与全部集成测试（调度记录第 10g 批合后 checker 档那一条）：`crash_injection_campaign.rs`、`bad_disk_input_campaign.rs` 都没红，红的 4 条归属都不在 10g（形式化接入那条线 28-1、31-1、30-3 与删除与并发那条线第 18 批）。照槽归盘之后只剩第 0 代根时也不重读，读序号回到 rr2 之前。E158、E171 实验复跑归提交时 |

### K5　数能不能核（正推一条腿）

一致：14 / 12、8 条红且拒的成员各 8 次、钩子那一刻每槽读 6 次（常量改 5 则 8 条全红）、手工盘面（变异 2768 只红这一条）都有断言或变异行钉着。第一轮判决里「36 到 28」是攻方装置的数，不进 kb。

### K6　欠账与读法清单（正推一条腿）

缺一格：kb 里 rr2 与 Q3 成员 0 命中、E158 重读计数在调度记录里 0 命中、第一轮判决 K7 写的「第 10 批收拢表」不存在。去向：rr2、Q3 随第 10 批 kb 写回；E158 重读计数归提交时的实验复跑；收拢表在第 10 批 kb 写回那一批的调度记录里建。

## 三、改法与去向

| 改法 | 出处 | 被攻轮数 | 去向 |
|---|---|---|---|
| rr2 触发与取 m 按读出的槽归盘，不按 txg 落点公式（第 0 代根三区各一份） | 攻方第一次 K2-1 | 零轮；攻方副本上量过：只剩第 0 代根时不再重读，rr2 的 14 条用例全绿，挂载写下的盘面不变 | 下一批（实现员）：条款 Q4「任一块盘没读出」的字面就是按盘面判，不是改条款 |
| `the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread` 改回 `OnTheFirstRead` | 攻方两次 | — | 下一批，随上一行 |
| rr2 用例丢盘整读换一个有活文件的起点 | 攻方第一次 K2-7 | — | 下一批 |
| Q3 判官「按表判被抛弃」补变异或留档 | 正推 K1-3 | — | 下一批 |
| 重读那一遍比第一遍坏时的读数取法 | 攻方第二次 K2-2 | 零轮 | 交用户（岔路 R1） |
| `rollback_by_a_forward_publish` 三条慢档用例改成「只许 I-3.1 红在记账多的一边」、差值各钉 819200、917504 | 调查员（调度记录慢档 4 条那一条） | — | 下一批 |

## 四、交用户的岔路

岔路单 `research/prompts/m3-batch10-code-r2-forks.md`（R1 一行）。

## 五、旁及与去向

| 线索 | 依据 | 去向 |
|---|---|---|
| 准入抬 F 上限里仍报同名 `Recovery(NoValidRoot)`（`mount.rs:2463`、`2506`），带故障层第三遍才全坏时不在单子上 | 正推 K1-1，没造盘面 | 记欠账，与第一轮 H3 同族 |
| rr2 让「第一遍读得出、重读读回全零」的槽进上限的计算，是 Q2 代价的一条新进入法 | 攻方第二次 K2-3 末段，推的 | 随 R1 一起看：按槽并之后这一形没有 |
| 用例名 `…does_not_reread` 与钉值读起来相反 | 正推、攻方第二次 | 照槽归盘之后名字与钉值一致，不另改 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-05 不受影响：rr2 的触发按读出的槽归盘是实现照条款改，不改条文；「根槽这一次读坏」那一行补 rr2 随第 10 批 kb 写回 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-05 不受影响：单子加 Q3 那一行随第 10 批 kb 写回；岔路 R1 选按槽并时 Q3 那一行的字面跟着改（重读那一遍改成两遍并起来） |

## 被判的 `crates/` 文件

第 10g 批补丁改到的每一份，这一轮都在被判范围里：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`、`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`、`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs`、`crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs`、`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs`、`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs`、`crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs`、`crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs`。

## 六、用户定案

用户 2026-10-05 弹窗定（岔路单 `research/prompts/m3-batch10-code-r2-forks.md`）：

| 岔路 | 定的 | 去向 |
|---|---|---|
| R1 重读那一遍比第一遍坏时用哪一份读数 | 按槽并两遍 | 下一批实现：重读之后逐槽取两遍里较好的那一份（任一遍自证过的根，两遍都自证过取重读那一条 ＞ 任一遍读坏不是全零 ＞ 全零），根环表、回收门槛、影子账都用并出来的那一份；Q3 改成两遍并起来一条不被抛弃的自证有效根都没有才拒，`mount_transparency.rs` 的复算判据跟着改；补 K2-2 那 12 段与反方向 4 段的用例。条款随第 10 批 kb 写回（D16 已定项 1「根槽这一次读坏」补 rr2 与按槽并、D23 已定项 14 单子那一行写成「两遍并起来」）。攻方提的改法，被攻过零轮，合入后交代码三方攻 |
