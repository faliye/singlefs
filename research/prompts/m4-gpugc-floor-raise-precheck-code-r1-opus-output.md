# m4-gpugc-floor-raise-precheck-code-r1 攻方腿（Opus）报告（2026-11-10）

立场：假设「抬 F 之前先算」那一判是错的，在冻结副本的拷贝上找反例。主攻 T1、T3，T2 与检查单八格逐格回答。
这一腿一条命令都没跑（子 agent 不交任务）：下面凡是没写「核过」的数都是读码推的；探针的读数等主 agent 跑完「要主 agent 跑的命令」第 1 条、新派同立场的腿承接。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-model/`（被判的冻结副本 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1/repo/`，被引的代码文件与仓里此刻那一份 sha256 相同，逐个比过）。

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1-opus/rerun-1
```

| 文件 | sha256 |
|---|---|
| `probe/Cargo.toml` | `8371dfe90094a93768ef22d529c123a548d2507a2b69d616b8ab134ba1ad0359` |
| `probe/core_probe_module.rs` | `5dc041b956f43d12cd1cc93053ab0dc2838758e5f98a8bf442f3ffa67a2d33dc` |
| `probe/src/history.rs` | `693a404570f09348437e68269ae8959967f394d701079db3b6333fce6fce6e79` |
| `probe/src/main.rs` | `0a520709c4bb2f275085d11236fbde1654a1e902124acbfdb09b00c4a08c0abc` |
| `probe/src/memory_device.rs` | `c53449b99f30c817799194b40d75064b8d55090a91700db5c86e31bf7df9358c` |
| `probe/src/pool.rs` | `f4ec69956e6e79784240477449938a157a4c4257bde85c142c64ce9c933495dd` |
| `probe/src/tally.rs` | `620c77dbeba0b68ffd2b65294d7cb3be0e0771fee8225d6c8acc6b46c45584a3` |
| `tools/patch_the_copy.py` | `d9b675e4ea4b18785dbb3c7edf6f86ceafa2a26940c018466585797b2bf826df` |
| `tools/rerun.sh` | `d01d4346f80627e3f42d0eec2dc5d6f2aed2c213f870067e2212c6a9bc699c74` |

`out/` 是空的：模型一次没跑过，复跑的输出写在 `--work` 那个目录的 `out/` 里（`probe.out` 末行 `probe PROBE_RESULT …`）。

探针做什么（文件头各有一段）：
- `tools/rerun.sh`：模板拷来的，只换了 `rerun_steps`。冻结副本 rsync 进 `$WORK/copy`（不带 target、.git），`tools/patch_the_copy.py` 拼探针，`cargo build --offline --release -p floor-raise-precheck-probe`，跑两条自证（各 62 条历史）与全量（1240 条历史），自证有一条没红就退 1、全量的数不算。
- `tools/patch_the_copy.py`：只改拷贝里的 `crates/singlefs-core/src/mounted_session.rs` 三处（原文都要恰好命中一次）：式子那一判的上界读一个只给自证用的「上界不算 defer」开关；先算判出放不下时先记一笔，探针的旁路开关开着就照推（等于先算之前的推法）；每一串推成前后各读一次。文件末尾拼上 `probe/core_probe_module.rs`（按线程记的探针模块）。工作区成员加探针那个包。
- 探针二进制：两块内存盘（只存写过的扇区，自己写的一份，不引验证包，免得拖上判定库客户端与 RocksDB 的编译），几何取验证包 384 槽小盘那一档（journal 环 6 MiB、单元区 1007 + 384 槽），两块盘各自再加 0 / 64 / 256 / 1024 槽（可以不等大）。mkfs、取号、暖机、第一个文件之后，每条历史 60–120 步，每一步按种子放开用户动作：建 inode（1–3 个）、整份写某个文件（单元数上限在 1、4、16、48、96、160、240、400 里挑）、覆盖写、unlink、截断（缩或长）、建快照（至多 3 个）、发布边界上崩了再挂、正常卸载再挂、重发上一次被拒的改动。
- 每一次经会话的调用之前拷一份池（盘上写过的扇区与会话的每个字段）。真池走带先算的会话；它交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 时，在拷贝上旁路先算、发同一个改动，推满仍不够就照用户重发那样再发，至多一共三次；写进了记 `HIT_MISREFUSAL`（头一次就写进）或 `HIT_MISREFUSAL_AFTER_RESENDING`。拷贝上推满仍不够而报「被宽护卡住」的记 `STUCK_REPORT_LOST`。
- 每一串推成，按推之前的读数核两个上界：式子那一判的可用涨了多少不超过（defer + 隔离）的字节；段外没挡的槽对涨了多少不超过 defer + 隔离 + 空闲而被挡的槽数。超出记 `UNSOUND_FORMULA` / `UNSOUND_PAIRS`，另记最小余量、每一串前后 ckpt_cost 变了多少、同一次调用里先后两次被拒同一块盘同一判的需求变了多少、没判出放不下的头一次被拒上界比需求多几槽与这次调用收没收下。
- 估时：先跑一批（与线程数一样多），按它估全量挂钟，超过 40 分钟就停、退 3、只印那一批。线程数照 `SINGLEFS_THREAD_CAP`，没设取登记文件的数。
- 崩溃状态：这一题判的是拒不拒、推不推，探针只在发布边界上崩（会话丢掉、盘停在最后一次落盘之后），不枚举崩溃状态，没有调枚举函数，也没有要算的状态数。宽护那一形要读坏的被抛弃根才造得出，探针造不出（见「这条腿自己的限度」）。

## 各格判定一览

| 格 | 判定 | 一句话 | 分量 |
|---|---|---|---|
| T1-1 上界算小了误拒 | 没打中（推的，待探针） | 式子那一判与槽对那一判的上界，按「推一串能放出什么」逐项对过都是真上界；唯一的缝是推完之后需求或 ckpt_cost 变小，探针量它 | 读码 |
| T1-2 越位到不该拦的拒绝 | 没打中 | `PlacementRefused` 与提交内生块那一判照推；`let … else` 把别的拒绝全放过去 | 读码 |
| T1-3 第四处为空间推串没过先算 | 打中（推的） | 改动之后另发的那一批搬迁被空间拒时推抬 F，走 `publish_the_relocation_batch_raising_the_floor_for_space`，不经先算 | 读码 |
| T1-4 提交内生块那一判「判不准」 | 打中（推的，可达性低） | 这一判照样有上界（没挡的槽加三样放得开的槽）；推的那一串自己也占槽只说明上界松，不说明算不出 | 读码 |
| T3-1 上界恰好等于需求 | 成立（不误拒）；附：等于那一格推了必定写不进（推的） | 每一串至少一次空发布、新写的单元记在已分配里，推完的可用严格小于上界（ckpt_cost 不变时） | 读码，探针核 |
| T3-2 多块盘一块短一块够 | 成立 | 逐盘判、只判被点名的盘，与 D28 已定项 1 逐设备合取同形；探针的盘可以不等大 | 读码，探针核 |
| T3-3 短缺种类混着来 | 打中（推的） | 落得下那一判排在式子那一判前面；落得下先拒、它的上界够时照推一串，推完式子那一判才判出放不下，`floor_raises` 带着白推的那一串 | 读码，探针数 `later_round_fired` |
| T3-4 被宽护卡住那条出路 | 式子那一判没打中；槽对那一判打中（推的） | 式子那一判：卸载重挂与推环放出来的都在 defer 加隔离里。槽对那一判：卸载重挂把这次挂载开过的聚簇段清空，段里没挡的槽对不在上界里，先算报放不下，「被宽护卡住」与它一定列的卸载重挂那条出路一起没了 | 读码 |
| T2 同一份分配器、带回的东西 | 成立 | 判法与推串读写同一个 `allocator`；三层记账都认新成员 | 读码 |
| T2 条款与代码 | 不一致一处 | D16 已定项 10 射程写「盘上逐字节不变」不分头一次与推过之后 | 读码 |
| 检查单：走不到的理由二次查实 | 成立 | 两处 `expect` 的不变量都核到计数的维护点 | 读码 |
| 检查单：类型与名字 | 成立（字面） | 没有 `_ =>`；`let … else` 兜底了 `PublishError` 别的成员，另报 | 读码 |
| 检查单：不兼容旧代码与旧格式 | 成立 | 补丁 15 个文件里没有 | 读补丁 |
| 检查单：不新建判定库 | 成立 | 补丁里没有新的存放处 | 读补丁 |
| 检查单：所有任务多机可用 | 成立 | 新用例都在内存盘上，补丁里没有读环境、读文件、主机名 | 读补丁 |
| 检查单：测试优先多线程 | 不成立 | 运行库两条崩溃循环用例单线程串着走，不能并行的理由没写 | 读补丁 |
| 检查单：带 --item 的命令与项 | 成立 | 三个崩溃点重放项、五行变异名在登记表里各命中一次 | 读登记 |
| 检查单：新崩溃流多罩了什么 | 成立 | 没有新流；头一次判出放不下一个写都没有，运行库用例钉着录制流为空 | 读码 |

## T1　过度外推与越位

### T1-1　上界算小了会不会误拒一个推了本来写得进的写

判法：先算只在被拒的那一次发布的拒绝上判，`SpaceAdmissionRefused` 才判得出需求（「let PublishError::SpaceAdmissionRefused(refused) = refusal else {」（`crates/singlefs-core/src/mounted_session.rs:5889`）），一块盘上界严格小于需求就交回新成员（「(upper_bound.0 < i128::from(short.demand.0)).then_some(」（`crates/singlefs-core/src/mounted_session.rs:5874`））。误拒要的是：推之前那一刻上界小于需求，而推一串（或几次请求累起来推的几串）之后同一个改动放得下。逐项核「推一串能让被拒那一判的量涨多少」：

**式子那一判**，上界是可用加 defer 与隔离的字节（「short.available.0 + bytes_of_slots(slots.deferred) + bytes_of_slots(slots.isolated),」（`crates/singlefs-core/src/mounted_session.rs:5851`））。式子九项里推一串动得到的：
- 已分配：读的是占着的槽数，defer 里的已释放槽算在里面（「allocated: BytesOnOneDevice::of_slots(device_map.allocated_slots()),」（`crates/singlefs-core/src/admission.rs:766`））。回收把槽从 defer 挪回空闲（「self.deferred_slots -= span;」（`crates/singlefs-core/src/allocator.rs:853`）），至多挪出全部 defer。推的每一次空发布新写的单元照常记成已分配（「self.allocated_slots += span;」（`crates/singlefs-core/src/allocator.rs:950`）），换下的进 defer、释放代高于这一串的上限、这一串回收不到。所以已分配至多降 defer、而且一定再加上这一串新写的单元。
- 第九项：读分配器隔离的槽（「BytesOnOneDevice::of_slots(device_map.isolated_slots()),」（`crates/singlefs-core/src/admission.rs:770`）），写掉被抛弃根或 C 那一圈的槽时解开，至多降隔离的全部。
- 抬 F 扣住的槽：式子本来不扣（`admission.rs` 第 750 行起那段文档写明扣住的槽「计数上已回到空闲」、式子里没有一项装它们），放开不改可用。上界不算它对。
- 挂载期承诺量与 checkpoint 保留池：两样都按「要接着的那一版」现算 ckpt_cost（「instance_switch_reserve_on_one_device(instance_rows_of_the_switch_reserve, checkpoint_cost),」（`crates/singlefs-core/src/admission.rs:1723`）），切换预留里暖机那一半乘 3 次、再乘 4 次切换（「.checked_mul(WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH)」（`crates/singlefs-core/src/admission.rs:638`），「.checked_mul(INSTANCE_SWITCHES_ALLOWED_PER_MOUNT + 1)」（`crates/singlefs-core/src/admission.rs:645`））。推一串之后要接着的那一版换成最后一次空发布，ckpt_cost 降 1 块，每块盘的可用就涨 13 槽（切换预留 12 加保留池 1，推的）。上界里没有这一项。
- 环窗口、待删占用、已承诺预留、不可回收：按几何或按这次发布之后的标签现算，空发布不改它们（推的）。

合起来（推的）：推一串之后每块盘的可用 ≤ 推之前的可用 + defer + 隔离 − 这一串新写的单元 + 13 ×（ckpt_cost 降了几块）。需求那一边是走到固定点之后的全部角色（「let demand = demand_of_the_roles_on_each_device(」（`crates/singlefs-core/src/transaction.rs:13386`）），分配记录树改几片叶随落点变，推完重判时同一个改动的需求可以与推之前不同。所以式子那一判的误拒只剩一条缝：推完之后需求少掉的加上 13 × ckpt_cost 降掉的，大过这一串自己新写的单元。按读码推不出这条缝开得开：空发布只照抄条目，ckpt_cost 的几项（分配记录树按几何、记账树按行数、中央映射树按条目数与形状）推一串不该变小；需求那一边变的是分配记录树几片叶，量级是几个节点，而一串至少一次空发布（运行库用例写的是每串 3 次），新写的单元不少于这个量级。探针把这两样都量出来：每一串前后的 ckpt_cost 差（`checkpoint_cost_delta`）、同一次调用先后两次被拒的需求差（`demand_delta_slots`）、式子上界的最小余量（`formula_margin_minimum_slots`）。

**槽对那一判**，上界是段外槽对加每个放得开的槽各补一对（「slots.deferred + slots.isolated + slots.free_but_blocked;」（`crates/singlefs-core/src/mounted_session.rs:5856`））。段外槽对只在一个槽从「挡着」变成「没挡」时涨，一个槽至多补成一对；推一串能把槽变成没挡的只有回收（defer）、解隔离、放开扣住（空闲而被挡），都在上界里。反方向能让段外槽对涨而上界不算的，只有「这次挂载开过的聚簇段」变少，而这个集合在一次挂载里只加不减：「self.cluster_segments.insert(segment_start);」（`crates/singlefs-core/src/allocator.rs:2625`）是它唯一的改动点，字段的文档写「段里的块被回收空了也仍算聚簇段」（`crates/singlefs-core/src/allocator.rs:1890`）。需求是这次数据单元个数 × 2 槽，推一串不变。所以槽对那一判的上界是真上界，推一串（几串）不会让它误拒（推的，探针核 `unsound_pairs`）。

判定：没打中（推的）。翻面条件（跑前写下）：
- 探针全量 `PROBE_RESULT` 里 `hits` 大于 0、而两条自证都红：误拒坐实，打中；看命中那一行的 `named=`（哪一判、上界、需求）与同一条历史的 `UNSOUND_*` 行定是哪一项算小了。
- `hits` 为 0 而 `unsound_formula` 或 `unsound_pairs` 大于 0：上界不是真上界，这一份取样里没撞上误拒，改判「打中（上界不是真上界）」，看 `checkpoint_cost_delta` 有没有负值。
- 两者都是 0、两条自证都红：这一份取样里没打中；照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，要再抽一次才记没打中。
- 自证有一条没红：探针判不出它要判的，全量的数作废，交下一条腿修探针。

四句（给的是「没打中」，四句只答前两句）：不分辨臂（被判的只有一种实现）；被判的系统那一刻看得到 defer、隔离、空闲而被挡三个计数，看不到推完之后的需求与 ckpt_cost。

### T1-2　判法有没有越位到不该拦的拒绝

- `PlacementRefused` 走 `let … else` 那一支交空表，照旧推（`crates/singlefs-core/src/mounted_session.rs:5889` 那一行之后是 `return Vec::new();`）。
- 提交内生块那一判交 `None`，不点名（「JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,」（`crates/singlefs-core/src/mounted_session.rs:5864`））。
- 不是空间不够的拒绝：先算交空表，之后照旧原样交回 `Publish`（先算排在 `refusal_is_short_of_space` 之前，「devices_short_even_with_everything_a_floor_raise_could_release(allocator, &refusal);」（`crates/singlefs-core/src/mounted_session.rs:5955`））。
- 删除类发布（unlink 与截断的意图、各批、删快照）不判式子，数据单元是 0，槽对那一判不会拒，只剩提交内生块那一判，照推（推的）。

判定：没打中。附带一处写法：`let … else` 把 `PublishError` 的别的成员一律当「判不出需求」，以后再加一种带需求的空间拒绝，这里不会编不过、会静默照推（见检查单「类型与名字」）。

### T1-3　改动之后另发的那一批搬迁：第四处为空间推串，没过先算

`publish_user_change` 发成这次改动之后，另发一批搬迁（修读回时只坏一份的旧单元），这一批被空间拒也推抬 F，走的是 `fn publish_the_relocation_batch_raising_the_floor_for_space<`（`crates/singlefs-core/src/mounted_session.rs:2389`）：被拒先过「if !compaction_publish_refused_short_of_space(&refusal_of_the_relocation) {」（`crates/singlefs-core/src/mounted_session.rs:2434`），是空间不够就直接「push_one_floor_raise_within_the_admission_budget(」（`crates/singlefs-core/src/mounted_session.rs:2447`），中间没有先算。正文说判法「3 处调用共用」，会话里为空间推串的是四处。

可达的历史（推的，探针没罩：探针不注读故障）：可写挂载期间读某个单元只坏一份 → 下一次用户改动发成 → 另发的搬迁批被式子那一判拒（带搬迁的发布把批次最坏占用加进已承诺预留，`crates/singlefs-core/src/transaction.rs` 判式子那一段的注释）→ 这一批的需求大过可用加 defer 加隔离 → 照旧推一串、两串，推满仍不够交回 `RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NoSpaceAfterRaisingTheFloor`。每一步许可它的那一句：搬迁批被空间拒就推（2434、2447 两行）；推的预算照 `push_one_floor_raise_within_the_admission_budget`。

四句：
1. 分不分辨臂：分辨「先算罩住会话全部为空间推的串」与「只罩三处」。
2. 系统那一刻看得到吗：看得到。拒绝是同一种 `PublishError::SpaceAdmissionRefused`（式子那一判），分配器是同一个，先算那个函数只要这两样。
3. 字面分句：D16（发布语义） 已定项 10 射程「推抬 F 放得出的全放出来也放不下就一串都不推」（`.claude/kb/decisions/16-发布语义.md:275`）。这一句的主语是 `accept` 被空间拒时自己抬 F 那一段，搬迁批算不算在射程里，条款没写；用户定案「改：推之前先算」也没限到哪几处。打中归哪一句要主 agent 定：归射程空白，还是归实现漏了一处。
4. 改法：跑前条款没给改法。这一腿提的：搬迁批被空间拒时同样过 `devices_short_even_with_everything_a_floor_raise_could_release`，放不下就交回一个新的 `RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing` 成员、一串不推（只在读码上推的、没实现没跑、被攻过零轮）。

挂载处的两处推串（`crates/singlefs-core/src/mount.rs` 里可写挂载写行之后与续做删除那两处）是挂载自己的准入，不在这一题的射程里，不判。

### T1-4　提交内生块那一判交 `None` 的理由不成立

那一行的文档写「提交内生块那一判交 `None`：这一判数的槽抬 F 那一串自己也要占，不在这里算。」（`crates/singlefs-core/src/mounted_session.rs:5842`）。同 T1-1 槽对那一判的论证：没挡的槽只在回收、解隔离、放开扣住时涨，一个槽涨一个，所以推几串之后「数据单元取走之后剩下没挡的槽」至多涨 defer + 隔离 + 空闲而被挡；抬 F 那一串自己也占槽只让真正推得出的更少，上界照样是上界，只是松。所以这一判算得出上界（可用加三样放得开的槽的字节），代码选了一律照推。

后果：落得下那一判排在走固定点与式子那一判之前（「admit_the_units_landing_on_every_device(」（`crates/singlefs-core/src/transaction.rs:13315`）在「let settled = settle_the_allocation_record_tree(」（`crates/singlefs-core/src/transaction.rs:13328`）之前），一次写在提交内生块那一判上被拒、加上三样也不够时，照旧推一串到几串、推满仍不够才拒，D16 已定项 10 射程那句「一串都不推」在这一判上不兑现。

可达性（推的）：这一判要拒，得段外槽对还够数据单元、剩下没挡的槽却不够提交内生块（普通的加整个 ckpt_cost）。式子那一判放行的盘上，没挡的槽至少有环窗口那么多（这一档盘约 1007 槽），远大于 ckpt_cost；要拒只能是数据单元几乎取光没挡的槽对、聚簇段里又几乎没有空着的槽，窗口很窄。探针按「named slots-left fired/pushed」数它出现几次。

四句：分辨「这一判算上界」与「这一判照推」两种实现；系统看得到（三样计数与拒绝里的可用、需求都在）；字面分句同 T1-3 第 3 句；改法：这一判也照槽对那一判的写法算上界（`available` 加 defer + 隔离 + 空闲而被挡的字节），只在读码上推的、被攻过零轮。

判定：打中（推的，可达性低）。

## T3　哪一条最脆弱

### T3-1　上界恰好等于需求照推

代码按严格小于判（`crates/singlefs-core/src/mounted_session.rs:5874`），等于照推，钉在纯判法用例 `fn a_formula_demand_exactly_covered_by_available_deferred_and_isolated_bytes_is_still_pushed_for(`（`crates/singlefs-core/src/mounted_session.rs:8104`）上，整池用例没有。

不误拒：等于那一格推了，T1-1 的不等式给出推完的可用 ≤ 上界 − 这一串新写的单元 + 13 × ckpt_cost 降的块数；ckpt_cost 不降时推完严格小于需求，照样被拒，结局是 `NoSpaceAfterRaisingTheFloor` 或下一轮的新成员，不是写进。所以「等于照推」不会造成误拒，判定：成立。

附（推的）：同一个不等式说明等于那一格推了一定写不进（ckpt_cost 不降时），推的那一串是白写的；更一般地，需求落在「推之前可用 + 这一串回收得到的 defer + 解得开的隔离 − 这一串新写的」与「可用 + defer + 隔离」之间的写都推了也写不进。这是「宁大勿小」选的松，不是错；量多大由探针的 `slack_of_the_pushed formula published …`（推了而写进的那几次上界比需求多几槽）与 `pushed_then_refused_calls` 给。翻面条件：`slack_of_the_pushed formula published 0` 计数大于 0（上界恰好等于需求而推了写进）就说明上界在那一格是紧的，这句附言撤回，同时回看 T1-1（多半是 ckpt_cost 降了）。

### T3-2　多块盘一块短一块够

先算逐块判、只判拒绝点名的盘，任一块放不下就整次不推。准入本来就是逐设备合取（D28（挂载期承诺量） 已定项 1「逐设备算，每块盘各自满足」，`.claude/kb/decisions/28-挂载期承诺量.md:26`），一块放不下，别的盘的富余补不了它，不推是对的；放得下的那块不被点名，不影响判。盘不等大只改各盘自己的三样计数与可用，判法不变。探针每条历史两块盘各自加 0 / 64 / 256 / 1024 槽，不等大占多数。判定：成立（读码，探针核）。

### T3-3　短缺种类混着来：落得下那一判先拒，式子那一判看不见

发布路径的次序：落得下那一判（`crates/singlefs-core/src/transaction.rs:13315`）→ 走到固定点（`crates/singlefs-core/src/transaction.rs:13328`）→ 式子那一判（「admit_on_every_device(」（`crates/singlefs-core/src/transaction.rs:13415`））。落得下那一判拒了，式子那一判就没判；先算只看手里这一次拒绝。

可达的历史（推的，每一步的许可）：
1. 一次挂载里发布得多，聚簇段一段段开出来、只加不减（`allocator.rs:2625` 是唯一的加入点）；段里换下、回收的槽还算在聚簇段里，用户数据用不上。
2. 用户整份写 N 个单元，N 大于此刻段外槽对、小于等于段外槽对 + defer + 隔离 + 空闲而被挡（defer 里有一大截是聚簇段里换下的元数据，算进了槽对的上界，放开它们补不成段外的槽对）。落得下那一判拒在槽对上，先算的上界 ≥ 需求，照推（`mounted_session.rs:5856` 那一行把三样都算进上界）。
3. 推一串（写盘、用掉根环的槽）。重判：
   - 段外槽对仍不够：这次的 defer 已被回收掉一截，上界掉到需求以下，先算交回新成员，`floor_raises` 带着刚推的那一串；或者
   - 段外槽对够了（聚簇段外回收出来几对），落得下那一判过，式子那一判判出可用 + defer + 隔离 < 需求，交回新成员，同样带着那一串。
4. 两种收场都是「推了也写不进、推之后才判出放不下」，D16 已定项 10 射程「推抬 F 放得出的全放出来也放不下就一串都不推」（`.claude/kb/decisions/16-发布语义.md:275`）在这一形上不兑现。

四句：
1. 分不分辨臂：分辨「先算只判被拒的那一判」与「先算把判得出的都判」。
2. 系统那一刻看得到吗：一半。式子那一判的需求要走到固定点才知道，落得下被拒那一刻没有；但可用（分配器现读）与需求的一个下界（这次数据单元与普通分配那一半的槽，都在需求里）看得到，按下界判「放不下」不会误拒。
3. 字面分句：同上第 4 步那一句；用户定案「可用加 defer 里全部字节仍小于需求就不推」里的「需求」在这一刻没定义，这一形判不判、按什么判，条款没写，交主 agent 定是射程空白还是实现漏。
4. 改法：这一腿提的——落得下那一判拒了、它的上界够时，再拿式子那一判的可用加 defer 加隔离对需求下界判一次（只在读码上推的、没实现没跑、被攻过零轮）。槽对上界里那一截「聚簇段里的 defer」放开也补不成段外槽对，按段分开数可以收紧，但要多维护一个按段的 defer 计数，同样零轮。

探针怎么数：`call later-round-shape first=<头一次被拒点名的判> fired=<判出放不下那一次点名的判>`。`first=slot-pairs` 打头的那几行就是这一形；`first=formula fired=formula` 是 T3-1 说的那条松带。翻面条件：全量里 `later-round-shape first=slot-pairs` 计数为 0、`later_round_fired` 也为 0，这一形在这份取样里没出现，判定改「推的、取样没撞上」。

判定：打中（推的，待探针数）。

### T3-4　被宽护卡住那条出路有没有被这一判吃掉

先算之前：推满停在「F 已在上限」（「FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}」（`crates/singlefs-core/src/mounted_session.rs:1960`）），C 那一圈还隔离着槽，就报「被宽护卡住」，要出路时卸载重挂一定列（「出路：卸载重挂一定列」（`crates/singlefs-core/src/mounted_session.rs:760`））。先算判出放不下时根本不推，走不到「停在上限」，这一报连同出路都不出现。要问的是：先算判出放不下的写，走出路之后会不会写得进。

- 式子那一判（推的）：推环只写掉 R 那几槽、F 不变，放出来的是隔离，自己还新写单元；卸载把 F 抬到现行那一版、能回收的至多是全部 defer，再挂时 C 那一圈按新择的根重算、宽护不再出现，放出来的至多是全部隔离，切换预留按新的 rows0 只多不少。两条出路放得出的都在 defer + 隔离里，先算判出放不下时两条都走不通，吃掉的是一条本来也没用的出路。判定：没打中。
- 槽对那一判（推的）：再挂时分配器重建，这次挂载开过的聚簇段清空（「cluster_segments: BTreeSet::new(),」（`crates/singlefs-core/src/allocator.rs:1955`）），上一次挂载聚簇段里没挡的槽对重新算进段外槽对。这一截不在先算的上界里（抬 F 放不出它），却是卸载重挂放得出的。所以在「C 那一圈隔离着槽 + 一次挂载里聚簇段攒了很多空着的槽对 + 一次大写短在段外槽对上、N 大于段外槽对 + 三样放得开的槽」这一形上：先算之前报「被宽护卡住」、列卸载重挂，卸载重挂之后写得进；先算之后报新成员，不带宽护、不带出路。

T3-4 槽对那一形的四句：
1. 分不分辨臂：分辨先算之前与之后（同一段历史上一个带出路、一个不带）。
2. 系统那一刻看得到吗：看得到。报「被宽护卡住」那一判只读分配器（`crates/singlefs-core/src/mounted_session.rs:1966` 起，`root_ring_push_target` 与 `isolated_slots_covered_by`），不用推、不用预演。
3. 字面分句：D16 已定项 1「根槽这一次读坏」那一行的那一句前提是「准入推抬 F 停在上限、C 那一圈这一刻还隔离着槽时」（`.claude/kb/decisions/16-发布语义.md:39`）；先算之后这个前提不发生，字面上不违。违的是正文第一节主 agent 定的射程「保住被宽护卡住那条出路」。
4. 改法：这一腿提的——新成员也带一份同样算法的 `wide_protection`（C 那一圈隔离着槽就点名、列卸载重挂；推环那条照旧到要出路时预演）。只在读码上推的、被攻过零轮。槽对上界不该加聚簇段里的槽对（抬 F 放不出它们），加了就会推一串白写。

可达性没量：宽护要读坏的被抛弃根才造得出，这一腿的探针造不出（见「这条腿自己的限度」）。验证包里被宽护卡住的那一族（例如 `fn a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting()`（`crates/singlefs-verification/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:1603`））在实现员副本上绿（任务 11701，正文第二节的观测，不是这一腿核的），说明那几形上先算没判出放不下；它们短在哪一判、把写放大到多少单元先算开始拦，没有人量过。翻面条件：在那一族的场景上把写的单元数往上扫，出现「先算之前报被宽护卡住、卸载重挂之后写得进，先算之后报新成员」的一格，就坐实；扫到先算开始拦的那一格卸载重挂之后也写不进，就撤回。

判定：式子那一判没打中；槽对那一判打中（推的，可达性没量）。

## T2　内部矛盾与循环依赖（次攻）

- 判法与推串读写的是同一份分配器：先算在 `push_one_floor_raise_for_a_refusal_for_space` 里读它的参数 `allocator`（`crates/singlefs-core/src/mounted_session.rs:5955`），同一个函数里接着拿它推。`accept` 那一路判的时候读 `&self.session.allocator`（`journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write(` 在 7003、7013 两行），被拒之后从 `&mut self.session` 拆出同一个分配器交给先算与推串。一次挂载里盘不增不减，拒绝点名的盘一定在这个分配器里。成立。
- 带回去的东西：新成员带 `floor_raises`、`refusals_that_pushed_the_floor_raises`、`versions_published`；`accept` 的入口认它（「UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) => {」（`crates/singlefs-core/src/mounted_session.rs:6510`）），运行库没收下时报的串从这个入口取（`crates/singlefs-mount-runtime/src/lib.rs:902` 调 `floor_raises_pushed_by_this_accept()`）。`accept` 那一路被拒时照样把 `current` 换成推到的那一版（「*current = PoolVersion::of_a_shared_version_of_trees(version_pushed);」（`crates/singlefs-core/src/mounted_session.rs:6915`）），头一次就判出放不下时那一版就是原来那一版的共享拷贝，等于不变。预算：新成员不推，`publishes_pushed` 不加。成立。
- 条款与代码一处不一致：D16 已定项 10 射程写「推抬 F 放得出的全放出来也放不下就一串都不推，报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，盘上逐字节不变」（`.claude/kb/decisions/16-发布语义.md:275`）。代码的文档分了头一次与推过之后：「头一次被拒就判到这里时它们是空的，盘上逐字节不变」（`crates/singlefs-core/src/mounted_session.rs:631`）。推过一串之后才判出（T3-3 那一形）、unlink 意图发成之后走空才判出，盘上都有这次调用写的东西。条款那一句少了「头一次」的限定，照字面读会以为新成员一律盘上不变。另：工作区里那一行以「两样已知边角：」收尾，下面没有列出来（这一行此刻有没提交的改动，可能正被别的会话写，只报看到的）。
- D16 已定项 10「过不了就在任何读写之前拒」与 `accept` 抬 F：射程已写 `accept` 被空间拒时自己抬 F（同一行），先算之后头一次判出放不下的 `accept` 在任何写之前拒，与「任何读写之前拒」同向；推过之后才判出的那一形（T3-3）不在「之前」，归 `accept` 抬 F 那一句的例外。没有新的矛盾。

## 检查单八格

### 走不到的理由二次查实：成立

- `SlotsAFloorRaiseCouldRelease::of_the_device` 的 `checked_sub … expect`（`crates/singlefs-core/src/mounted_session.rs:5835`）：空闲槽数与已分配槽数只在 `mark_reclaimed` 与 `mark_allocated` 两处成对改（「self.free_slots += span;」（`crates/singlefs-core/src/allocator.rs:854`）、「self.free_slots -= span;」（`crates/singlefs-core/src/allocator.rs:951`）），所以空闲槽数等于分配位没置的槽数；没挡的槽数数的是三种位都没置的槽（`fn is_unblocked_at_index(&self, index: usize) -> bool {`（`crates/singlefs-core/src/allocator.rs:517`）那一判），三种位都没置的一定分配位没置。没挡 ≤ 空闲恒成立，走不到。冻结副本里它只有一处调用（`crates/singlefs-core/src/mounted_session.rs:5903`，在 `devices_short_even_with_everything_a_floor_raise_could_release` 里）；这一腿的探针补丁在拷贝里另调它，只读。
- `devices_short_even_with_everything_a_floor_raise_could_release` 的 `find … expect`（`crates/singlefs-core/src/mounted_session.rs:5900`）：拒绝里的盘来自 `AdmissionReading::of_allocator` 与 `PlacementRoomOnOneDevice::of_every_device_of_the_allocator`，两份读数都按 `allocator.devices` 逐块建；调用点只有 `push_one_floor_raise_for_a_refusal_for_space`，三处调用交进来的分配器与判出拒绝的是同一个（T2 第一条），一次挂载里盘不变。走不到。

### 类型与名字：成立（字面）

新名字 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`、`DeviceShortEvenWithEverythingAFloorRaiseCouldRelease`、`SlotsAFloorRaiseCouldRelease`、`most_a_floor_raise_could_make_available`、`device_short_even_with_everything_a_floor_raise_could_release`、`devices_short_even_with_everything_a_floor_raise_could_release`、`no_space_even_with_everything_a_floor_raise_could_release` 都说出内容、不缩写。字节与槽：`SlotsAFloorRaiseCouldRelease` 三个字段都是槽数（同一单位、各有字段名），换成字节只经 `BytesOnOneDevice::of_slots` 一处；上界是 `AvailableBytesOnOneDevice`、需求是 `BytesOnOneDevice`，没有裸整数混用。`match short.judged_short_by` 三个成员逐个写，没有 `_ =>`。两处 `expect` 都写了依赖的不变量。

另报一处同目的的兜底：「let PublishError::SpaceAdmissionRefused(refused) = refusal else {」（`crates/singlefs-core/src/mounted_session.rs:5889`）把 `PublishError` 的别的成员一律交空表照推。字面不是 `_ =>`，但起的作用相同：以后加一种带需求的空间拒绝，这里编得过、静默照推。归不归这一格由主 agent 判。

### 不兼容旧代码与旧格式：成立

读了补丁目录 `crates.patch` 的 15 个文件（`git apply --stat` 那一份清单，`apply-outcome.tsv` 列的同一组）：没有认旧版本的分支、旧布局号、迁移、只为旧数据放宽的读法。

### 不新建判定库：成立

同一份补丁里没有新的目录、TSV、sqlite、RocksDB 或别的存放处；新用例的断言都在进程内。

### 所有任务多机可用：成立

补丁新增行里搜过 `env::var`、`std::fs`、`/tmp`、`HOME`、主机名、`File::open`、`read_to_string`，一处都没有；新用例都在内存盘上现造。

### 测试优先多线程：不成立

运行库两条崩溃循环用例（`descending_sizes_…` 与 `thirty_two_unit_files_…`，补丁里 `a_write_that_never_fits_pushes_no_series_on_any_mount_of_a_crash_loop`）在一个线程里串着走 9 次挂载、每次 3 次 `accept`。按正文那一格「不能并行的写明为什么」，文档里没写为什么不并行。实质上这条历史前后相依（每次挂载接着上一次崩溃留下的盘），不拆不了；两种填法已经是两个测试函数、由测试框架并行。所以打中的是「理由没写」，不是「该并行没并行」。核心层十条是纯函数小用例，验证包四条各是一段场景，不涉及。

### 带 --item 的命令与项：成立

`.claude/gate.d/crash-point-replay-items.tsv` 里 `truncate-to-zero-on-a-nearly-full-pool`、`last-fitting-unit-then-refusal-and-unmount-on-a-nearly-full-pool`、`floor-raise-pushed-by-the-session` 各命中一行（行首逐字、制表符结尾，仓里与冻结副本都查了）。`crates/mutations.tsv` 里五行变异名各命中一次：「抬 F 先算：式子那一判的上界不算 defer」「抬 F 先算：式子那一判的上界不算隔离」「抬 F 先算：槽对那一判一个槽只换一个槽的量不换一对」「抬 F 先算：拒绝里丢掉推串的那几次被拒」「accept 抬 F：拒绝里不带推成的串」。命令带了范围声明与 `<根>`（实现员报告第 9、10 条）。只读登记，没跑 `--list-items`。

### 新崩溃流多罩了什么：成立

本轮没有新流。头一次判出放不下一个写都不发，录制流里这一步是空的，崩溃域里没有新状态；运行库用例钉着「{label}：一个写都不发（录制流记每个写、写零、discard 与屏障）」（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1040`），不用另加小用例。推过一串之后才判出的那一形（T3-3），盘上的写是先算之前那条推法写的前缀，崩溃状态是旧流状态的子集，`floor-raise-pushed-by-the-session` 那一项罩着推串本身。贴边那条流严格写着 `NoSpaceAfterRaisingTheFloor`（「let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = &refusal else {」（`crates/singlefs-verification/tests/crash_enumeration_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream.rs:397`））：那一步之前第一个文件被顺序写过几十遍，换下的旧版本单元都在 defer 里，再写一个单元的需求远小于可用加 defer，先算不会判出放不下，这一项应当不变红（推的，等第 9 条命令的结局）。

## 这一腿提的改法各修哪一格

都是这一腿提的，只在读码上推、没实现没跑（连探针都还没跑），被攻过零轮。

| 改法 | 修哪一格 | 在打中的那几格上还中不中 | 标 |
|---|---|---|---|
| 搬迁批被空间拒也过同一个先算，放不下交回一个新的搬迁结局成员、一串不推 | T1-3 | 只修 T1-3；对 T3-3、T3-4 不起作用 | 推的 |
| 提交内生块那一判也算上界：可用加 defer + 隔离 + 空闲而被挡的字节 | T1-4 | 只修 T1-4；T3-3 第一种收场（槽对仍不够）不受影响 | 推的 |
| 落得下那一判拒了、它的上界够时，再拿式子那一判的可用 + defer + 隔离对需求下界（这次数据单元与普通分配那一半的槽）判一次 | T3-3 第二种收场（推完式子才判出） | 不修 T3-3 第一种收场（槽对上界把聚簇段里的 defer 算进去了）；要修它得按段分开数 defer | 推的 |
| 槽对那一判的上界里，defer 只数聚簇段外的（按段多维护一个 defer 计数） | T3-3 第一种收场 | 收紧上界；按 T1-1 的论证仍是真上界，不引入误拒 | 推的 |
| 新成员也带一份同样算法的 `wide_protection`（C 那一圈隔离着槽就点名、列卸载重挂） | T3-4 槽对那一形 | 只修 T3-4；不改推不推 | 推的 |
| D16 已定项 10 射程那一句补「头一次被拒就判出时」 | T2 条款与代码 | 只改条款文字 | 推的 |

## 要主 agent 跑的命令

1. 探针整份复跑（`tools/rerun.sh`，模板拷来、只换了 `rerun_steps`）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1-opus/rerun-1
```

   - 在哪起：仓根（命令里的路径都是绝对路径）；`--work` 指到这一腿认领目录下还不存在的 `rerun-1`（认领目录的 `.deliverables` 已写成交接）。
   - 跑完看：`TASK_SERVER_RESULT` 那一行 `exit=0` 算跑完。`task-server/task log <号>` 里依次有 `PATCH_THE_COPY …`、`name=build done …`、`name=exit_codes bound_without_deferred=0 soundness_without_deferred=0 probe=0`、三行 `… ESTIMATE …` 与三行 `… PROBE_RESULT …`、`name=self_checks_red …`、`name=rerun_done …`。退 1 而日志尾巴是 `cargo build` 的报错：这一腿没编过探针，编不过是它的错，交承接的腿修（`$WORK/out/build.log`）。退 1 而写「自证没红」：探针判不出它要判的，全量的数作废。`probe=3` 是估时超过 40 分钟，`probe.out` 里有 `ESTIMATE_EXCEEDS_FORTY_MINUTES` 与先跑那一批的计数，交承接的腿缩条数。
   - 内存与时间（推的，没量）：384 槽那一档的盘每块单元区约 22 MiB、journal 环 6 MiB，只存写过的扇区；每条历史同时持真池、一份拷贝，判出放不下时再多一份；线程数是登记值，峰值估几 GiB。编译只编核心层、格式层与探针，不编验证包与判定库客户端。全量 1240 条历史，挂钟由探针先跑一批自己估，超过 40 分钟自己停。
   - 承接：新派同立场的腿，派发提示写「承接：research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-output.md」与任务号，它读 `task-server/task log <号> --all` 与 `rerun-1/out/` 下三份 `.out`，按本报告 T1-1、T3-1、T3-3 写下的翻面条件改判定。

## 没打中的形状

读码逐格试过、判不出误拒的（都是推的）：
- 式子那一判：已分配里的 defer、第九项隔离、扣住的槽、切换预留与保留池（ckpt_cost）、环窗口、待删占用、已承诺预留，九项逐项问「推一串能让它降多少」（T1-1）。
- 槽对那一判：回收、解隔离、放开扣住三样，聚簇段集合只加不减（T1-1）。
- 等于那一格（T3-1）、多块盘与不等大（T3-2）、`PlacementRefused` 与不是空间不够的拒绝（T1-2）、删除类发布只过落得下那一判（T1-2）。
- 跨几次请求累起来推：上限卡在读坏或全零的根槽上时一次请求只推不抬 F 的空发布、下一次请求从零预算接着推（D16 已定项 1「抬 F 的上限」那一行），推到写掉那一槽之后才回收得到的 defer 一直在先算的 defer 计数里，所以几次请求累起来放得出的也不超过上界。
- `accept` 的「待发连同这次」那一批：与「这次单独」走同一个先算、同一种需求，判法相同。
- 被宽护卡住、短在式子那一判上的写（T3-4 第一条）。

探针要扫的范围（等主 agent 跑）：1240 条历史、种子 1–1240，每条 60–120 步；盘宽两块各在 0 / 64 / 256 / 1024 槽里挑；动作九种按权重放开（见开头）。两条自证各 62 条、种子 900001 起。这一份取样只算一次观测，即使 `hits` 为 0，照规矩要再抽一次（换种子起点）才记没打中。

## 这条腿自己的限度

- 一条命令都没跑：判定全是读码推的；探针连编都没编过（子 agent 不起 cargo，`rustfmt --check` 只核过语法），编不过的可能不小。
- 探针只在发布边界上崩，不枚举崩溃状态；不注读故障，所以宽护（T3-4）与搬迁批（T1-3）两形它造不出，只走 `publish_user_change`，没走 `accept`（`accept` 与它共用先算，需求按「待发连同这次」算，读码判同形）。
- 盘只取验证包 384 槽那一档加宽，每区根槽数取 mkfs 默认，没扫；历史至多 120 步、中间会崩了再挂与卸载再挂，聚簇段攒不到很多，T3-3 第一种收场与 T3-4 槽对那一形要的「一次挂载里聚簇段攒了很多空着的槽对」可能取不到。
- 探针在拷贝里改了核心层三处（加记录与旁路，判法本身不动；只给自证用的「上界不算 defer」开关默认关）。旁路那一臂等于先算之前的推法，这一点按补丁原文对过（先算之前那一段代码就是去掉先算的这一个函数），没另核。
- 没读禁读清单里的文件；正推腿的结论不知道。

## 草稿与清理

- 认领目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1-opus/`：这一腿没建仓副本、没建编译目录（被拒的那条拼补丁的命令在执行前就拒了，什么都没建）。三份 `rustfmt --check` 的输出（共约 42 KiB）已删。留着的只有 `.deliverables`（`rerun-1` 写成交接，给主 agent 的复跑用）。
- 模型目录在仓里 `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-model/`，`out/` 是空的。
