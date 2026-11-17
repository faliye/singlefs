# m4-close-batch2-code-r1：攻方腿承接报告（Opus，s2，2026-11-10）

立场：攻方。承接 `research/prompts/m4-close-batch2-code-r1-opus-output.md`，只攻第三节第 1 条里的 A2（暂停删意图那一次自己的落点也受 R 排他）。这一段只做一件事：上一条腿的探针二造前提时 panic，改它的造法，让前提成立；A1 已由主 agent 复跑坐实，这里不碰。方案按 `crates/` 今天的实现谈：主工作区五份产品文件与 `research/prompts/m4-close-batch2-code-r1-snapshot/crates-sha256.txt` 逐份对得上（量的，这一段现跑 `sha256sum -c`，五行都是 `OK`）。

## 复跑

模型目录 `research/prompts/m4-close-batch2-code-r1-opus-s2-model/`。复跑命令（整条交服务，见末尾「要主 agent 跑的命令」）：`bash research/prompts/m4-close-batch2-code-r1-opus-s2-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新目录>`。`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps` 那一段（`diff` 现跑：模板那边只少 3 行，即模板自带的 `rerun_steps` 正文）；做法照上一份：`minimal-copy.py` 拷冻结副本、核五份产品文件 sha256、把探针追加进 `compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs` 末尾（要用那份文件里的私有辅助函数）、只跑名字以 `attack_` 起头的那一条。探针一不再带。

```
27cc9584136b0da9a7f85684c2dbce798a47713a13f30f1aedf0c34d09bf649f  probes/attack_drop_refused_after_the_batches_freed_slots_in_the_region.rs
1c76b0630630c64ea49716f43dc58c75e0e7fc5a07baf62eb8f3918762be9e12  tools/rerun.sh
```

我什么都没编、没跑（子 agent 不交任务）：探针只过了 `rustfmt --edition 2021 --check` 的解析（三处排版差异，没有解析错），`tools/rerun.sh` 只过了 `bash -n`。下面凡是没标「量的」的读数都是推的。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| 三·1 A2 | **打中（推的，待探针跑）** | 同上一份：R 之外没有空槽而 R 里有时，暂停删意图那一次照 R 排他取落点被拒，意图与 R 排他一直留着；放开 R 发同一次删意图就发得成 | 读码（推的）＋新探针（待主 agent 跑） |
| 上一条探针二为什么没造成前提 | 找到病根（推的） | 它挑的那一段过不了签发：强制选段的四道资格判里有一道不过，签发那一次停在 `NoEligibleSegment`；而在这个池上，照首次适配的落法，非满的那一段大概率过不了签发前的空洞判 | 主 agent 复跑的日志（量的）＋读码（推的） |
| 上一条探针二的断言 | **对不上，当场说** | 它把「对照臂之后用户改动发成」写进打中那一边；R 之外全标满之后，前台写还要过合取 7（全空聚簇段数 ≥ 1）与式子那一判，两臂都会被拒（推的）。前提造成了它也会判红。这一项不在上一份的推翻条件里，新探针只观测、不进断言 | 读码（推的） |

## A2：新探针怎么造前提

### 上一条探针二卡在哪（推的，除注明的一行）

- 主 agent 复跑的日志（量的）：`/home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r1-main-rerun/rerun-1/probes.log` 第 64–65 行，探针在 `crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:268` 起头的那条 `assert!` 上 panic，消息「在 CPU 上只发了签发那一次，意图在强制选的那一段上：Stopped(NoEligibleSegment)」（消息模板在同文件第 272 行）。
- 上一条探针挑 R 的办法是：第一块盘上有两槽记录的段里，两块盘同槽号空槽最多的那一段。强制选段时这一段要过四道判（`crates/singlefs-core/src/mounted_session.rs:5345` 起：是不是这次挂载开过的聚簇段、段里有没有钉住或隔离的槽、扫描拒过没有，再加扫描里要有可搬的单元），不过就落进 `(false, Ok(_)) | (true, _) => None,`（`crates/singlefs-core/src/mounted_session.rs:5359`），停在 `NoEligibleSegment`。哪一道没过我没量到。最像的是：两槽记录不只是数据单元（码 3 的两槽容器也是两槽），空槽最多的那一段多半是装提交内生块的那一段（同类用例写它在 50240：`crates/singlefs-verification/tests/compaction_off_the_calling_thread_and_admission.rs:60`），里面有被根直接指着的单元（钉住），或者只有不可搬的。
- 换一个非满的数据段也不行（推的）：签发前要判 R 之外装着用户数据的段里没挡的槽数不少于 R 里要搬的单元数（`crates/singlefs-core/src/mounted_session.rs:5474` 那一句 `if !holes_admit_the_issue(holes, units_to_move) {`）。这个池写 40 个数据单元，数据最多的那一段里有 29 个（`crates/singlefs-verification/tests/compaction_off_the_calling_thread_and_admission.rs:73` 的注释，那份用例量过），按首次适配的落法只有最后那一段有空洞；取它当 R，R 之外装数据的段几乎没有空洞，过不了这一判。

### 新造法

1. R 取 `pool_after_a_sequential_write_in_an_earlier_mount` 交回的那一段，即数据单元最多的那一段。同文件 `a_compensating_run_on_the_cpu_while_stopped_after_a_gpu_failure_ends_exactly_as_the_cpu_mode` 在 CPU 上用同一个辅助函数签发过它（`crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:2204` 与第 2208 行 `"在 CPU 上",`），A1 的探针也在这一段上签发成了（主 agent 复跑的观测行 `start_slot: SlotNumber(50176)`，量的）。
2. 签发之后把分配器的复用窗口切到只供测试的 `ReuseWindow::ForcedToZero`（「只供测试：释放那一刻就回收、当场可再分配，F 与延迟窗口都不看。」，`crates/singlefs-core/src/allocator.rs:1105`），一趟只推一次地推几批（`run_one_compaction_publish`，一批至多 2 个），每批把 R 里的单元搬到 R 之外，R 里搬空的槽当场回收成空槽。每块盘 R 里的空槽到 32 个就停（上界 16 批；R 里 29 个单元，8 批搬 16 个，搬不空，意图不会在这一步做完）。这一步代 A2 历史第 2 步「R 里搬空的槽在抬 F 之后回收」：产品路径要先抬 F、等带新 F 的根落满，结果同样是 R 里那几槽变成空槽（推的）。之后复用窗口放回产品那一档 `GatedByTheRollbackFloor`。
3. 在分配器里把 R 那段槽号之外的每一个空槽标成已分配（`mark_allocated`，与上一条探针同一个直接标法），代「意图活着期间前台的写都落在 R 之外、把 R 之外写满」。
4. 前提成立的判法：意图活在 R 上、分配器的 R 排他是 `Some(R)`、每块盘 R 里的空槽数 > 0、每块盘 R 之外按 `is_free` 数的空槽数 = 0。不成立就停在前提那一道断言上（红），观测行照打。
5. 之后照上一份：显式入口推两趟（每趟预算放满）；一次用户改动（只观测）；对照臂撤掉分配器这一侧的 R 排他，经 `transaction::publish_compaction_step(…, CompactionIntentStep::Drop, …)` 发同一次删意图，发成就换上那一版、再发一次用户改动（只观测）。

### 比上一条探针多核的一样：失败是不是删意图那一次被空间拒

上一条探针的「一趟」只看 `Failed`。可一批的拒绝不是空间那一类时，会话不走暂停、原样交回（`if !compaction_publish_refused_short_of_space(&refused) {`，`crates/singlefs-core/src/mounted_session.rs:5859`），一样是 `Failed`、意图一样留着，却与 A2 无关。新探针多算一项 `refused_short_of_space`：结局是 `Failed(Batch(Publish(cause)))` 且 `refusal_is_short_of_space(cause)`（空间那一类就是 `PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,`，`crates/singlefs-core/src/mounted_session.rs:2092`）。推法：一批被空间拒才去删意图，删意图那一次发成就不是 `Failed`、没发成就交回它自己的拒绝；所以停在空间那一类的 `Failed` 上，就是删意图那一次试过、被空间拒。再加一项「丢意图数（`observations.intents_dropped`）这一趟前后不变」。这是收严，不改上一份的推翻条件。

删意图那一次为什么只能在 R 之外取落点（推的）：它是只删整理意图、不搬单元的发布，式子那一判不判（`&& !deletes_only_the_compaction_intent(deletion_and_lineage, relocations)`，`crates/singlefs-core/src/transaction.rs:13405`），合取 7 只拦前台写（`if writes_in_the_foreground && !is_a_rollback && relocations.is_empty() {`，`crates/singlefs-core/src/transaction.rs:13263`），剩下的就是固定点取落点；提交内生块开放段装不下、又没有全空段时回落到槽号最小的空槽，「同样排除 `R`」（`crates/singlefs-core/src/allocator.rs:1024`），取落点那一处读的是分配器此刻的 R 排他（`crates/singlefs-core/src/allocator.rs:2589`）。

### 观测行与期望读数（都是推的，待主 agent 跑）

观测行都以 `m4-close-batch2-attack:` 起头，照上一份「探针二」那一段的名字，字段只多不少：

| 行 | 期望 |
|---|---|
| `drop_refused_setup` | `region_start_slot=50176`；`batches_freeing_the_region` 在 1 到 8 之间；`free_slots_in_the_region` 每块盘 ≥ 32；`free_slots_outside_the_region=[0, 0]`；`live_intent_on_the_region=true`；`region_excluded_from_placement=Some(...50176...)`；`reuse_window=reuse_window_gated_by_the_rollback_floor`；`data_units_in_the_region` 比 29 少搬走的那几个 |
| `drop_refused_trip trip=0`、`trip=1` | `outcome=Failed(Batch(Publish(PlacementRefused …)))` 或 `SpaceAdmissionRefused`；`refused_short_of_space=true`；`versions_published=0`；`txg_before` 与 `txg_after` 相同；`live_intent_on_disk=true`；R 排他仍是 `Some(R)`；`intents_dropped_before` 等于 `intents_dropped_after` |
| `drop_refused_user_change_with_the_region_excluded` | `refused=true`（只观测，见下） |
| `drop_contrast_with_the_region_open` | `withdrawn=Some(R)`、`published=true`、`live_intent_on_disk=false`、`rewritten_slots_in_the_region` > 0（R 之外没有空槽，删意图那一版的固定点只能落进 R） |
| `drop_contrast_user_change` | `published=false`，`detail` 里是 `EmptyClusterSegmentsBelowTheFloor` 或空间准入那一类（只观测，见下） |

断言两道：前提（第 4 步那四样）；打中（两趟都满足上表 `trip` 那一行的全部，且对照臂删意图发成）。绿 = 打中。

**用户改动那两行为什么只观测、不进断言**（推的）：`mark_allocated` 给它标的每一槽都记成罩在未释放记录下（`crates/singlefs-core/src/allocator.rs:945` 那一句 `self.note_slots_under_an_unreleased_record(`），R 之外每一整段都不再全空，R 里还有单元也不全空，记录口径的全空段数掉到 0。新建 inode 是前台写，合取 7 照判（`N₀` 第一版取 1），两臂都会被拒；式子那一判在 R 之外标满之后同样可能拒。所以上一份「对照臂之后用户改动发成」这一期望不成立，它也不分辨两臂；上一条探针把它写进了断言，前提造成了也会判红。我把它从断言里拿掉，只打观测行。这一项不在上一份的三条推翻条件里，拿掉不改推翻条件；要不要认，主 agent 定。

### 推翻条件（照上一份原样，加一条收严）

照上一份原样的三条：
1. `drop_refused_setup` 里 `free_slots_in_the_region` 有 0：R 里本来就没空槽，这一形退化成真满，不分辨臂。
2. `drop_refused_trip` 的 `outcome` 不是 `Failed`，或意图不在：删意图那一次另有落点来源，A2 不成立。
3. 对照臂也被拒：放开 R 也不够，改法碰不到这一格。

加的一条（收严）：`outcome` 是 `Failed` 而 `refused_short_of_space=false`：失败不是删意图那一次被空间拒，这一趟不支撑 A2，按没打中记。

另：前提那一道红（意图在第 2 步就做完了、`free_slots_outside_the_region` 不是 0、R 排他不在），是造法的问题，不是 A2 的输赢；看 `outcome_of_the_last_freeing_run` 与 `batches_freeing_the_region` 判卡在哪一步。

### 四句

- 分不分辨臂：分辨。「删意图那一次照 R 排他取落点」（产品）与「删意图那一次放开 R 取落点」（对照臂）在同一段历史上一臂被拒一臂发成。
- 系统当时看不看得到判别它的东西：看得到。`drop_the_live_intent_on_a_stop` 自己交的就是 `CompactionIntentStep::Drop`，发布路径分得出这一次是只删意图。
- 满足的是哪一个分句：用户定案「整理暂停 删意图 下次重来」（`records/2026-10-13-里程碑四预开发.md:1589`）的「删意图」在这一形上做不到；`drop_the_live_intent_on_a_stop` 文档串「下一次整理再走到停下时再删。」（`crates/singlefs-core/src/mounted_session.rs:5883`）承诺的「再删」在这一形上永远删不成（两趟都被拒）。
- 跑前条款给的改法在这几格上中不中：见下表。

### 改法表

| 改法 | 修 A2 吗 | 标注 |
|---|---|---|
| 删意图那一次（`CompactionIntentStep::Drop` 与达成删意图）取落点时不排除 R | 修 | 推的，待主 agent 跑新探针的对照臂；只在我的模型上、被攻过零轮 |
| 删意图被拒时退回撤 R 排他、意图留着 | 修卡死，回到 I-5.6 红 | 推的；用户已否 |
| 不改 | 不修 | 推的 |

### 用户动作放开扫

由用户决定的有两样：R 之外被什么写满（数据写、建 inode、快照钉住的旧单元），与 R 里的空槽是怎么来的（几批搬走之后抬 F 回收）。前一样不改第 4 步以后：删意图那一次只要固定点落点（推的）。后一样我只造了「`ForcedToZero` 下推几批」这一种，抬 F 的那一条没造；两条留下的分配器状态都是 R 里那几槽空着（推的）。只固定了前缀（写 40 个数据单元、在 R 上签发）与「R 之外写满」。

## 没打中的形状

这一段只改造法，没有新攻别的形状。造法上试过、弃掉的两种（都是读码，推的）：取 R 之外空槽最多、装着两槽记录的那一段（上一条探针的办法，主 agent 复跑停在 `NoEligibleSegment`，量的）；取数据单元较少的那一段（签发前的空洞判在这个池上过不了）。

## 这一条腿自己的限度

- 探针没编过、没跑过。API 照同文件已有用例与上一条探针（上一条已编过，主 agent 的复跑走到了运行期）的写法；新用到的只有 `set_reuse_window`、`reuse_window().name()`、`mounted_session::CompactionRunFailure`、`transaction::RelocationBatchRefused::Publish`、`mounted_session::refusal_is_short_of_space`、`observations.intents_dropped`、`batches_published`，都在 `crates/` 里查到是 `pub`。编不过的话 `^error` 行会打出来，交承接的攻方腿改。
- 造前提用了两样只供测试的手段：`ReuseWindow::ForcedToZero` 代抬 F 回收 R 里搬空的槽；`mark_allocated` 直接标满 R 之外（标的槽没有分配记录，池级 checker 会判 I-3.1 不对，探针不跑 checker）。两样都只在前提那一段，进被判路径之前复用窗口已放回产品那一档。`ForcedToZero` 下推的那几批，旧根还指着被复用的槽（C22 那一格），这里不重挂、不回退，用不上。
- 32 槽是估的（删意图那一版要重写的固定点估不过十来槽），没量过；对照臂若因 R 里槽不够被拒，`detail` 会写出来，那时把 `FREE_SLOTS_IN_THE_REGION_WANTED` 往上调再跑，不算推翻条件 3。
- 哪一道资格判挡下了上一条探针挑的那一段，我没量到，只给了最像的推法。
- 删意图之前、途中、之后崩溃那一格没造；没写调崩溃枚举函数的原型，本腿原型跑的崩溃状态数 0（承接的上一条腿也是 0，合计 0）。

## 要主 agent 跑的命令

1. 新探针（判决要引 A2 的数靠它）。在仓根起，`--frozen` 给开工那一刻的树（主工作区此刻五份产品文件与快照 sha256 相同，量的；脚本里先核一遍），`--work` 给一个新目录：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 compaction_on_the_cpu_while_stopped_after_a_gpu_failure 里攻方追加的一条 attack_ 探针; 范围=1 个测试目标里 1 条; 为什么=攻方腿核暂停删意图在只剩 R 有空槽时删不删得掉' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r1-opus-s2 -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-batch2-code-r1-opus-s2-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r1-opus-s2/rerun-1`
   看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=`（0 = 打中；1 = 快照核不过、前提那一道红或没打中；别的码是脚本或编译那一步的错）；日志里 `name=m4_close_batch2_attack_snapshot rc=0`、`name=m4_close_batch2_attack_probes rc=`、`name=m4_close_batch2_attack_summary drop_refused_hit_reproduced=`；`panicked at` 之后那一行以「前提不成立」还是「打中 A2」起头；以及五种观测行 `drop_refused_setup`、`drop_refused_trip`（两行）、`drop_refused_user_change_with_the_region_excluded`、`drop_contrast_with_the_region_open`、`drop_contrast_user_change`，对上面「观测行与期望读数」那张表与推翻条件逐行判。日志与观测落在 `--work` 那个目录（`probes.log`、`observations.txt`、`snapshot-check.txt`），每次尝试的副本在 `rerun-1.attempts/attempt.*/copy`，脚本跑完自己删；编译目录由服务给，交的人用完 `task-server/task clean <号>`。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，上面那条与判决要引的数都没复跑，报告里没标「量的」的读数都是推的。
- A1 不碰（主 agent 已复跑坐实）；探针一没带进新模型目录。
- 背景材料我只读了正文 `research/prompts/_m4-close-batch2-code-r1-body.md`、上一份报告与 `crates/` 现码；派发之后主 agent 更正背景材料路径为带下划线的 `research/prompts/_m4-close-batch2-code-r1-background.md`，这一段没引它的内容。
- 副本上的数（探针跑出来之后）不算入库装置上的数。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r1-opus-s2` 里只有两份文本（`syntax-check.rs` 是探针的拷贝给 `rustfmt --check` 解析用的、`rustfmt.txt` 是它的输出）与认领文件，没建仓副本、没建编译目录，没有要删的。
