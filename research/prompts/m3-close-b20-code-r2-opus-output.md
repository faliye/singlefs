# m3-close-b20-code-r2 攻方腿报告（C971 与 C973 代码轮第二轮）

批次：`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 20 批。腿的模型与盘面用例都没跑（子 agent 不交任务），下面凡写「推的，待主 agent 跑」的读数都等「要主 agent 跑的命令」一节的结果。

**复跑命令（在仓根起，整条）**：见「要主 agent 跑的命令」第 1 条。**模型目录** `research/prompts/m3-close-b20-code-r2-opus-model/`，每个文件的 sha256（`sha256sum` 现量）：

```
2d51151198a1128997a019f781119d474b47096d2597fb12264a0bf1ddeb7c00  attack_cases_appended_to_the_test_target.rs
b352cfc631af9fc4a599dd1c4913edbf30e20b2c05099cf54e91e08d79a175a0  genesis_closed_form_appended_to_the_sweep.rs
7c4386f5f82df485e3e9581312b36c3b208048dc4f45440e7058e3800df3344b  ring_pass_model.py
8b1b0a47a06f17730d69b96c62e58cd849c5279718984882e7ad7d34509ab53e  tools/rerun.sh
```

## 各格判定一览

| 格 | 判 | 依据 | 读数状态 |
|---|---|---|---|
| 别池 × 撕裂 × 全零 × 读错的组合（实现自己） | 没打中：12 种组合实现都照条款分流 | 第一节表 | 推的（按代码走） |
| 判官择系统配置不筛 fsid：合法的拒判成不合法（A1） | **打中** | `crates/singlefs-verification/src/mount_transparency.rs:2550`、用例 `the_judge_calls_a_legal_refusal_illegal_when_it_chooses_the_other_pool` | 推的，待主 agent 跑 |
| 判官择系统配置不筛 fsid：不该有的拒判成合法（A2） | **打中**（判别力缺口，今天的实现没走到） | `crates/singlefs-verification/src/mount_transparency.rs:2550-2552`、用例 `the_judge_cannot_see_a_wrong_refusal_when_one_device_chooses_the_other_pool` | 推的，待主 agent 跑 |
| 判官与实现查一槽的次序不同（A3） | **打中（轻）**：只有手造才有 | 用例 `the_judge_and_the_product_disagree_on_a_slot_with_an_unknown_format_version_and_wrong_node_bytes` | 推的，待主 agent 跑 |
| 只读挂载的出路（B1） | **打中**：别池的槽世代号最大、每块盘都有时可写拒、只读也拒，只剩离线出路 | 用例 `newest_generation_of_another_pool_on_every_device_leaves_neither_writable_nor_read_only` | 推的，待主 agent 跑 |
| 离线修复路径 | 没打中：别池的槽算不自证，重写之后可写挂载做成 | 同上用例后半 | 推的，待主 agent 跑 |
| 「没有一块盘两槽都自证过」放过落后盘（B2） | **打中**（条款射程，不是实现照错）：比六格少坏一槽，新实例第一次发布落到被藏的根之下 | 用例 `a_device_holding_two_stale_self_verified_slots_lets_the_inverted_first_txg_through` | 推的，待主 agent 跑；要一次丢写，不是纯崩溃能到 |
| 别池 + 布局位不认的槽（B3） | **打中（线索）**：登记两句相撞，实现取「不算坏」，倒挂放行 | 用例 `a_slot_of_another_pool_with_unknown_incompat_bits_lets_the_inverted_first_txg_through` | 推的，待主 agent 跑 |
| 取号、F2 退回、逐盘核不读新字段 | 成立 | 第三节 `grep` 原样输出 | 量的（源文件 `grep`） |
| 随机历史新步与理想模型判的是同一件事 | 在它的取样域上是同一件事；没打中，但模型是常量、前提用被测函数算 | 第四节 | 推的（读代码） |
| C973 判据：S 不是 2 的幂、环绕一圈以上 | 没打中（错位读）：有证明与闭式核 | 第五节 | 证明是推的；闭式核与模型待主 agent 跑 |
| C973 射程句（C1） | **打中（轻）**：不说谎的设备也走得到，「最多停一圈」要加「跳号不重复」 | 第五节 | 推的，待主 agent 跑 |
| 检查单八格 | 全部成立，第 7 格带旁注 | 第六节 | 量的（`grep`、`task log`）；第 7 格 10312 等结局 |

**判决之外的提醒**：A1、A2、B1、B3 的盘面都是手造（一块盘上出现别的池的槽，崩溃走不到这一形：mkfs 先把 `[0, 根环基址)` 清零过屏障才写别的，`crates/singlefs-core/src/make_filesystem.rs:694`）。B2 要一次吞掉的轮换写或读回旧内容。这几处都不是一次普通断电能到的。

## 一、别池的槽与撕裂、全零、读错同时出现

实现的判法（`crates/singlefs-core/src/mount.rs:9191-9197`）：先 R2，再乙；乙只看两样之一（`crates/singlefs-core/src/mount.rs:9245-9253`、`crates/singlefs-core/src/mount.rs:9254-9258`）。符号：A 本池自证过，B 别的池（校验和过），T 撕裂或全零，E 设备报读错。盘面两块盘各两槽。下表按代码走出来（推的），「判官」一栏指判官用 `choose_system_configuration`（不筛 fsid）时的走向，「择到」是它择到哪个池：

| 盘 0 | 盘 1 | 实现 | 判官择到 | 判官判据 | 对不对得上 |
|---|---|---|---|---|---|
| A A | A B | 过读阶段，做成 | A | 不判（没拒） | 是 |
| A B | A B（B 世代号小） | 乙拒 | A | 真 | 是（实现员用例） |
| A B | A B（B 世代号大） | 乙拒 | B | 真（A 算别池） | 结论对、理由错（见 A1） |
| A A | A B（B 世代号大） | 做成 | 对不上，`choose` 报错 | 按真 | **A2** |
| B B | B A（盘 1 的 B 世代号大） | 乙拒 | B | **假** | **A1 不合法** |
| A T | A B | 乙拒 | A | 真 | 是 |
| T B | A T | 乙拒 | 对不上 | 按真 | 是 |
| E B | A A | 过乙（盘 1 两槽都过），逐盘核拒（读错） | 对不上 | U6 那一拒按真 | 是 |
| T B | A A | 过乙，走补写闸 | 对不上 | 不判或 U6 按真 | 是 |
| A B | A E | R2 拒 | A 或对不上 | 真 | 是 |
| A E | A B（重读才出现） | 第一遍 R2 真，重读乙真，只按重读那一遍判 | 判官每槽读一次 | 按当下盘面 | 是 |
| A X | A X（X 格式版本不认又节点大小不对） | 乙拒 | A | **假** | **A3** |

实现这一列全部照 `research/prompts/e205-preregistration.md:46` 与 `research/prompts/e205-preregistration.md:47` 走，没有一格是实现自己错。走不到的另一样：别的池的槽每区槽数越界时，可写挂载筛 fsid 在解析之前（`crates/singlefs-core/src/recovery.rs:1622-1627`），不因它整池拒，这一点与注释「越界的池根本挂不上」（`crates/singlefs-core/src/recovery.rs:5142`）说的是本池，不矛盾。

### 打中 A1：判官把合法的拒判成不合法

**盘面**：C971 最小历史；盘 0 两槽都换成别的池的（世代号比本池大 200、201）；盘 1 较旧位置换成别的池的（世代号大 100），盘 1 另一槽是本池最新那一槽。

**走法**：可写挂载按调用方的 fsid 择到本池那一份（「choose_system_configuration_of_the_pool(devices, filesystem_identifier_handed_in);」（`crates/singlefs-core/src/mount.rs:9679`）），盘 0 两槽别池、盘 1 一槽本池一槽别池，没有一块盘两槽都自证过，乙拒，盘上不变。判官不筛：「let Ok(system_configuration) = choose_system_configuration(crash_image) else {」（`crates/singlefs-verification/src/mount_transparency.rs:2550`），两块盘都择到别的池那一份，拿它的 fsid 当本池读每块盘两槽（`crates/singlefs-verification/src/mount_transparency.rs:2559-2564`），盘 0 两槽「都自证过」，乙与 R2 都假；只读恢复按同一份择到别的池、根一条都读不到，`selected_version` 是 None，「let selected_version = selected_version?;」（`crates/singlefs-verification/src/mount_transparency.rs:2612`）交回假。判官报 `CriterionFalseWhenRecomputedOnTheCrashImage`。

**四句**：
1. 分不分辨臂：不分辨。甲、乙、丙的实现都走这份判官。
2. 被判系统当时看不看得到判别它的东西：**看不到**，这是判官的毛病：`judge_writable_mount_refusal` 的参数里没有本池 fsid（`crates/singlefs-verification/src/mount_transparency.rs:3079-3085`），两个池的槽分不出哪个是本池。形态是「判别子观测不到」。
3. 满足的是字面哪一分句：正文第 2 条「透明判官择系统配置不按 fsid 筛、可写挂载按 fsid 筛能不能让判官判错」。
4. 改法在打中的格上还中不中：判官加一个「本池 fsid」参数、择系统配置改走筛 fsid 的择法，修 A1、A2。**只在我的推理上过、被攻过零轮**；用例读数推的，待主 agent 跑。

### 打中 A2：不该有的拒，判官判合法

**盘面**：只有盘 1 较旧位置换成世代号大 100 的别的池的槽（单故障）。实现做成（用例前半断言）。同一份盘面喂一个声称的 N-配置 之拒：判官 `choose` 报各盘 fsid 对不上，「return true;」（`crates/singlefs-verification/src/mount_transparency.rs:2551`），判据按真，判合法。对照盘面（别的池的槽世代号小，实现员用例那一形）同样喂，判官判不合法。

今天的实现在这一形不拒，判官判错不外露；缺口在：实现若在这一形错拒，判官抓不到。变异行 `crates/mutations.tsv:7062`（「别的池那一判丢了 R2 的另一半」）的盘面是世代号小那一形，正好绕开它。**推翻条件**：用例前半实现没做成，或判官判不合法。

### 打中 A3（轻）：判官与实现查一槽的次序不同

一槽整槽校验和过、格式版本不认、节点大小又不对：实现先查 incompat 再查节点大小、不查格式版本（「if !incompat_bits_are_mountable(bytes) {」（`crates/singlefs-core/src/system_configuration.rs:720`）、「if u64::from(recorded_node_bytes) != NODE_BYTES {」（`crates/singlefs-core/src/system_configuration.rs:730`）），报 `NodeBytesNotTheFormatConstant`，读的一方记读坏，乙拒。判官用 checker，格式版本排在最前（「if read_u16(slot, FORMAT_VERSION_OFFSET)」（`crates/singlefs-checker/src/lib.rs:364`）），报 `FormatVersionNotRecognized`，不在判官认的三样里（`crates/singlefs-verification/src/mount_transparency.rs:2344-2349`），不算坏，判据复算为假、判不合法。

四句：不分辨臂；判官看得到字节，是判官查的次序不同；满足「能不能让判官判错」；改法：判官那一判自己按 magic、校验和、节点大小三样读字节，不经 `check_system_configuration_slot` 的次序。只有手造才有，优先级低。

## 二、只读挂载与离线修复

### 打中 B1：可写拒、只读也拒

**盘面**：C971 最小历史，不注读故障；每块盘较旧位置换成世代号比本池最新大 100 的别的池的槽。

**走法**：
- 可写挂载乙拒。这一形在这一批之前做成，是这一批新加的一支（`crates/singlefs-core/src/mount.rs:9245-9253`）翻过来的。
- 只读挂载不筛：「choose_system_configuration_naming_devices_without_a_mountable_slot(reader)」（`crates/singlefs-core/src/mounted_read.rs:650`）。两块盘都择到别的池那一份，`choose_root` 按它的 fsid 一条根都读不到，报 `NoValidRoot`（`crates/singlefs-core/src/mounted_read.rs:653-654`）。这一格这一批之前就是这样，条款写过：「只接在可写挂载与离线出路上，只读挂载与恢复照旧不筛」（`.claude/kb/checks-owed.md:424`）。
- 恢复同一个择法（`crates/singlefs-core/src/recovery.rs:8275`）。
- 离线出路走得通：按调用方 fsid 择（`crates/singlefs-core/src/system_configuration_repair.rs:236`），别的池的槽算不自证（`crates/singlefs-core/src/system_configuration_repair.rs:198-201`），重写之后可写挂载做成（用例后半）。

**对照条款**：D22 写乙的出路是「照它拒可写、只给只读挂载」（`.claude/kb/decisions/22-单元原子性怎么合成.md:185`），那一句管的是坏的槽撕裂的态。「fsid 与本池不同而校验和过的槽算读得出而自证不过」（`.claude/kb/decisions/22-单元原子性怎么合成.md:185`）把别池的槽并进了乙，只读这一半没跟上：世代号小的那一形只读照常（对照用例 `older_generation_of_another_pool_on_every_device_still_mounts_read_only`），世代号大的那一形只读不通。这一批之前这一形可写做成、只读不通；这一批之后可写也不通。全仓条款没写这一形只剩离线出路。

**四句**：
1. 不分辨臂：只在乙入判不出才翻（甲、丙不并进别池的槽，乙直拒那一行也只写全零与自证不过，`research/prompts/e205-preregistration.md:45`；乙直拒要不要算别池的槽靠第 47 行）。
2. 系统看不看得到：只读挂载**看不到**：入口只收一个读者（`crates/singlefs-core/src/mounted_read.rs:648`），手里没有本池 fsid，`AnyPool` 一支的注释写的就是这个（「读者手里没有本池的 fsid（恢复、只读挂载、用例与装置）：各盘择到的 fsid 对不上整池拒」，`crates/singlefs-core/src/recovery.rs:1367`）。
3. 满足的分句：正文第 2 条「只读挂载与离线修复的路径会不会因为新字段或乙这一拒走错」。新字段本身没进只读路径；走错的是「拒可写之后还剩只读」这一句。
4. 改法：① 条款把这一形写明「只剩离线出路」，不改代码，把盘面立成用例（我倾向这一条，只在我的推理上过、被攻过零轮）；② 只读入口收一个可选的 fsid，要改入口形状，零轮；③ 这一形不并进乙，与第 47 行、C707 相反，要用户定。①、② 不改打中那一格上「可写拒」的结局，只有 ③ 改。

**推翻条件**：用例里只读挂载做成，或离线出路报拒。

### 没打中：离线修复

`rewrite_system_configuration_slots_for_the_asserted_version` 不读新字段（`grep` 见第三节）。试过：每块盘一槽别池（用例）；盘 0 两槽别池盘 1 一槽别池（读代码：盘 0 两次写，世代号 1、2，`crates/singlefs-core/src/system_configuration_repair.rs:370-371`）；别池的槽落在下一写之外（I-7.16 成立时走不到：存活的本池槽世代号与槽号同奇偶，下一写落另一槽，`crates/singlefs-core/src/system_configuration_repair.rs:354-356`）。

## 二之二、条款的另一半放过的形状

### 打中 B2：一块落后盘让乙不判，比六格少坏一槽

**历史**：C971 最小历史（实例 1 暖机 txg 1、2，第一个文件 txg 3，B、C、D 是 txg 4、5、6，D 的轮换没持久）。盘 0 收 C 那次轮换的写被吞（说谎的设备，或一次读回旧内容），盘 0 两槽都本池自证过、见证值都不超过 B。盘 1 正常。

**故障**：可写挂载这一次，C、D 的根与记录读坏（与六格同一组），只有盘 1 最新那一槽读回全 0。

**走法**：盘 0 两槽都自证过，「R2 的另一半（没有一块盘两槽都自证过）照旧」（`research/prompts/e205-preregistration.md:46`）那一半不成立，乙不判（`crates/singlefs-core/src/mount.rs:9256-9258`）。c_见证 取自证过的槽的最大值，是 B 的 tail（`crates/singlefs-core/src/mount.rs:9185-9190`），所选那一版也是 B，判据为假，放行。新实例第一次发布 = `max(环, 记录) + 1` = txg 5（「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」，`crates/singlefs-core/src/mount.rs:1534`），落到被藏的 D（txg 6）之下。与六格丙那一列是同一种倒挂（推的：没在这份盘面上崩在暖机第二次发布之前跑池级 checker，只断言了第一次发布的 txg）。

**对照**：同一组读坏、盘 0 没落后，盘 0 最新那一槽见证到 C，c_见证 大于所选那一版 B，判据比出来为真，拒（用例 `the_same_faults_are_refused_when_device_zero_is_not_behind`）。分辨的就是「落后」这一件事。

**用户动作放开扫过（按代码推的，没跑）**：盘面里落后那一槽是照较旧那一槽改世代号造的，见证值等于 B，不是从历史拍下来的。这条历史里用户决定的步只有写了几次；C971 那几次写之前的次数不改结局，只要落后盘的见证值不超过所选那一版。落后一次（纯崩溃走得到：D 的轮换只落盘 1）时，见证值到 C，要藏得让所选那一版不旧于 C，就只能藏 D 一条，新实例第一次发布落在 D 那个 txg、盖掉 D，不倒挂。所以这一形要落后两次轮换，纯崩溃走不到，要一次丢写（轮换之后那道屏障挡住了纯崩溃，`crates/singlefs-core/src/transaction.rs:1668`）。

**对照条款**：验收写「造这段最小历史（根槽三种读坏 × 系统配置槽两种读坏）当用例，判可写挂载要么拒、要么写不出池级 checker 红的镜像」（`.claude/kb/checks-owed.md:611`）。这一形故障数：一次丢写、一槽读坏、两条根读坏；C971 原形：两槽读坏、两条根读坏。

**四句**：
1. **分辨臂**：乙直拒「有一槽读回全零或读得出而自证不过就重读那一槽一次，仍是这两种之一就拒可写，不再看 R2 的两半」（`research/prompts/e205-preregistration.md:45`）在这一形上拒；甲越过读坏的根槽，第一个 txg 大于 D，也不中（甲的改法在 `crates/singlefs-core/src/mount.rs:1593-1601`，推的）；乙入判不出与丙中。用户 2026-11-07 选的是乙入判不出；E205 页（`.claude/kb/experiments/205-多处读坏倒挂三条改法的代价.md`）`grep` 「落后」「两槽都自证过」「丢写」「吞」都是 0 行，这一形大概没进当时的岔路表（只核了这四个词，没通读）。
2. 看不看得到：看不到盘 0 落后：两槽都自证过，见证值只有比盘 1 小，而盘 1 那一槽读回全 0。实现只能按「有一槽坏」保守拒（乙直拒那种），分不出落后还是没落后。
3. 满足的分句：C971「怎么拦」一格那一句的「要么拒、要么写不出」。按 D22 第 185 行「R2 的另一半照旧」，实现是照条款的，不是实现错。
4. 改法在这一格上：乙直拒修、甲修、乙入判不出不修。交主 agent 判要不要写进 C971 射程（丢写归 C6（块层语义假设写错）那一类，可以写「不罩」）。我提的是写进射程、不改代码，**只在我的推理上过、被攻过零轮**。

**推翻条件**：用例里可写挂载拒了，或第一次发布的 txg 大于 6。

### 打中 B3（线索）：别池又布局位不认的槽

**盘面**：每块盘最新那一槽换成 fsid 改一个字节、incompat 第一个字节翻一位、校验和重封的槽；C、D 的根与记录读坏。

**走法**：`parse_slot` 先查 incompat（`crates/singlefs-core/src/system_configuration.rs:720`），不解 fsid，两个计数都不进（`crates/singlefs-core/src/recovery.rs:5179-5182`）；挂载筛 fsid 那一处却按自证不过跳过它（`crates/singlefs-core/src/recovery.rs:1623-1626`）。乙不判，倒挂放行。

**对照条款**：「fsid 与本池不同而校验和过的槽算读得出而自证不过」与「布局号不认、每区槽数越界的槽不算坏、不并进」（都在 `.claude/kb/decisions/22-单元原子性怎么合成.md:185`）在这一槽上相撞，条款没写谁优先。判官与实现一致（都不算）。

**四句**：不分辨臂（甲修、乙两读法都不并进）；系统看得到（校验和过，fsid 能读）；满足「别池的槽」那一句与「布局号不认」那一句各一半，判据自己没写全；改法：条款定优先次序。这一槽要有别的版本布局的池，第一版没有，优先级低。交主 agent 判要不要记一笔。

## 三、取号、F2 退回、逐盘核确实不读新字段

`grep -n 'slots_of_another_pool_whose_checksum_verifies' -r crates/singlefs-core/src crates/singlefs-verification/src crates/singlefs-checker/src` 原样输出（截掉行尾注释）：

```
crates/singlefs-core/src/recovery.rs:5129:    pub slots_of_another_pool_whose_checksum_verifies: u64,
crates/singlefs-core/src/recovery.rs:5154:    let mut slots_of_another_pool_whose_checksum_verifies = 0;
crates/singlefs-core/src/recovery.rs:5165:                        slots_of_another_pool_whose_checksum_verifies += 1;
crates/singlefs-core/src/recovery.rs:5190:        slots_of_another_pool_whose_checksum_verifies,
crates/singlefs-core/src/mount.rs:9236:/// （[`SystemConfigurationSlotsOfOneDevice::slots_of_another_pool_whose_checksum_verifies`]：…
crates/singlefs-core/src/mount.rs:9247:        .any(|slots| slots.slots_of_another_pool_whose_checksum_verifies > 0)
```

读它的只有 `crates/singlefs-core/src/mount.rs:9247`。三处各核一眼：取号「if system_configuration_witness_is_undecidable(&slots_of_every_device) {」（`crates/singlefs-core/src/transaction.rs:964`）只判 R2；F2 退回（`crates/singlefs-core/src/recovery.rs:8150`）只判 R2、c_见证 取 `self_verified`（`crates/singlefs-core/src/recovery.rs:8153-8158`）；逐盘核解构时 `..` 跳过它（`crates/singlefs-core/src/mount.rs:6785-6789`）。`newer_publish_witness` 只有 `read_stage` 一个调用点（`crates/singlefs-core/src/mount.rs:9280`）。**成立**。什么现象会推翻：这三处 `grep` 再出现新字段。

一条旁证：取号那一刻没有乙（`crates/singlefs-core/src/mount.rs:563`）。读阶段判得出、取号之前盘面变成乙那一形（瞬时）照 R2 放行，这是登记的（「只加一道拒」），不算打中。

## 四、随机历史新步与理想模型

读过：`crates/singlefs-verification/src/history.rs:7917-8042`、`crates/singlefs-verification/src/model.rs:8052-8066`、`crates/singlefs-verification/src/model_comparison.rs:1418-1424`。

| 问 | 答 |
|---|---|
| 模型判什么 | 不判。「answer_mount_writable_with_no_device_holding_two_self_verified_system_configuration_slots」那一步答拒是常量（`crates/singlefs-verification/src/model.rs:8052`），判别全靠执行器选哪一支（`crates/singlefs-verification/src/history.rs:7997-8005`） |
| 前提谁算 | 被测函数：`verified_system_configuration_slots`（「let self_verified = singlefs_core::recovery::verified_system_configuration_slots(」，`crates/singlefs-verification/src/history.rs:7926`）就是 `system_configuration_slots_of_one_device_read_once` 的 `.self_verified`（`crates/singlefs-core/src/recovery.rs:5108-5114`）。read_once 把两槽都自证过读错时，这一步走 `not_applicable`，不报对不上（`crates/singlefs-verification/src/history.rs:7990-7992`） |
| 产品判的是不是这一件 | 在取样域上是：前提保证藏之前每块盘两槽都自证过，每块盘都藏则每块盘一槽全 0、一槽自证过，产品走乙（全 0 记读坏，没有一块盘两槽都自证过），模型答拒。只藏第一块盘，另一块盘两槽都自证过，乙不判，模型答一次普通挂载 |
| 拒因比到哪一层 | 比到 `PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion` 一层（`crates/singlefs-verification/src/model_comparison.rs:1420`），不比两遍的比法。走 R2 那一支或比出来为真的，模型分不出。这一步的盘面上不会走别的支，只有实现把全 0 记成设备读错时才看不出 |
| 取样域 | 只有全 0、只有每块盘或第一块盘。撕裂的非零字节、别的池的槽、读错与全 0 同现都不进随机历史（`grep -n 'another_pool\|别的池' crates/singlefs-verification/src/history.rs crates/singlefs-verification/src/model.rs` 0 行）。别池那一支只有手造用例与变异行 `crates/mutations.tsv:7061-7065` |

**没打中**：没造出模型与产品结局不同的历史。共用前提有两处：前提用被测函数算；「拒不看根环里有没有比所选那一版新的发布」（`crates/singlefs-verification/src/model.rs:8050`）与产品同一句条款。前提这一处的后果是漏（read_once 坏了这一步悄悄跳过），不是判错。提一笔：执行器可以不调 core、用判官那一份 `system_configuration_slots_read_on_the_crash_image` 算前提（它是私有的，要开口子），**只在我的推理上过、零轮**。什么现象会推翻「没打中」：`random_histories` 这一组比重上出现 `SucceededWhenModelRequiresRefusal`。

## 五、C973 判据

### S 不是 2 的幂、环绕一圈以上

落点是「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:172`）、「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`crates/singlefs-core/src/root_ring.rs:173`）。`(t mod 3, ⌊t/3⌋ mod S)` 在任意 S 上是连续 3S 个 txg 到 3S 个槽的双射，不用 S 是 2 的幂。判据的 `.take(3S)` 从 txg 1 起（`crates/singlefs-core/src/instance_table_sweep.rs:611-613`），覆盖 1..=3S，闭式是：区域 r 槽 j 第一次被写的 txg 是 3j + r，区域 0 槽 0 是 3S；判据为真 ⟺ 最大 txg 不小于它。环绕之后最大 txg ≥ 3S，三个槽 0 的判据都恒真，再多环几圈不变。追加进副本的 `genesis_criterion_matches_the_closed_form_for_every_slots_per_region` 在 S = 4..=16、最大 txg 到三圈多上逐格比（调实现的私有函数，闭式是我写的，不共用代码）；推的，待主 agent 跑。

### 错位读藏不住根（证明，推的）

清扫不停、交回根环的前提是每一槽都读出自证过且合落点的根（`crates/singlefs-core/src/instance_table_sweep.rs:496-500`、`crates/singlefs-core/src/instance_table_sweep.rs:573-578`）。S ≥ 4 时区域 2 槽 S−1 只能住 txg ≡ 3S−1 (mod 3S) 的根，所以一遍读出的最大 txg H ≥ 3S−1。于是：
1. 区域 1、2 的槽 0 读出第 0 代根时判据恒真（第一次写的 txg 是 1、2，都 ≤ H），要过这一关只能重读读回别的根。
2. 判据为假只剩区域 0 槽 0 且 H < 3S。这时要藏住的是它的真根：真根 txg ≡ 0 (mod 3S)、≥ 3S，错位读读回第 0 代根，字节要从别的槽来，只能来自区域 1 或 2 的槽 0 里真的第 0 代根。
3. 那一槽真的住着第 0 代根时，它自己按第 1 条判真、重读仍是第 0 代根、这一次不删（`crates/singlefs-core/src/instance_table_sweep.rs:542-548`）。它第一读错成别的东西也没用：落点不合则重读读回第 0 代根，「self_verified_on_the_first_read」为假，直接不删（`crates/singlefs-core/src/instance_table_sweep.rs:527-533`）。

所以「错位读回别的槽此刻的内容」这个故障模型里 `hidden` 没有取样点，三个槽 0 同时错也一样。上一轮的模型取样点是 S = 4、5、8、16、至多两槽；这条证明不靠取样，`ring_pass_model.py` 把 S 补全到 4..=16、三槽同错、跳号一起再扫一遍，**预期 `mode=misread` 的 `hidden` 全 0**（推的，待主 agent 跑）。推翻条件：哪一档 `hidden` 非 0。

### 读回旧内容（故障模型之外的线索）

一次读回**这一槽上一次的内容**（不是别的槽的）不在 C973 射程里（条款写的是错位读）。模型的 `mode=stale` 预期 `hidden` 非 0：H_true = 3S、真根在区域 0 槽 0、读回第 0 代根，H = 3S−1，判据假。到产品上要多一层：清扫在挂载的写行与暖机之后判（`crates/singlefs-core/src/mount.rs:7921`，挂载交回之前那一判），挂载自己的根落在 3S+1、3S+2，也要读回旧内容才把 H 压下去，就是三个槽 0 同时读回旧内容；这时那两槽判据真、重读读回真根放行，「判据只用上面那一遍读出的最大 txg，重读读出的不改它」（`crates/singlefs-core/src/instance_table_sweep.rs:522`），区域 0 槽 0 那一判不回头。模型另数 `hidden-caught-by-rejudging-after-rereads`：按重读之后的最大 txg 再判一遍能不能拦。这一形要读回已被覆写的旧字节三次，**不算打中**，写下来给主 agent 判要不要进射程；要改是再判一遍，只在我的模型上量过（还没跑）、被攻过零轮。

### 打中 C1（轻）：C973 射程句写窄

「丢了写进槽 0 的根槽写（说谎的设备吞掉 txg 1 那一次）时，清扫每轮都停，最多停一圈（3S 次发布），只少删不多删（只影响活性）」（`.claude/kb/checks-owed.md:613`）。

**历史**（不要说谎的设备）：mkfs；第一次可写挂载，暖机第一次发布（txg 1）记录落了、根 FUA 之前崩。持久顺序是「新池新建文件（D16（发布语义） 已定项 7）= 单元写 × 12（默认几何）→ 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障。」（`crates/singlefs-core/src/transaction.rs:7`），暖机同形（`crates/singlefs-core/src/transaction.rs:6`）。下一个实例的第一个 txg = max(环, 记录) + 1 = 2（`crates/singlefs-core/src/mount.rs:1534`），txg 1 那一槽（区域 1 槽 0）这一圈不写，真住着第 0 代根，判据为真、清扫停，到 txg 3S+1 才写掉。

**两处与射程句不合**：① 不要说谎的设备，一次崩溃就够；② 「最多停一圈」要加条件：txg 3S+1 又在同一个窗口崩，再停一圈。模型 `name=sweep_goes_through` 打这几行，预期 S=4 时跳 `[1]` 从 txg 13 起放行、跳 `[1, 13]` 从 25 起、跳 `[1, 13, 25]` 从 37 起（推的，待主 agent 跑）。

**四句**：不分辨臂（条款写法）；清扫看不到这一槽为什么住着第 0 代根，分不出丢写、跳号、错位读，只能停，停是对的；满足射程句里「说谎的设备」那一半与「最多一圈」那一半；改法：改射程句（写回批），代码不动。**结局仍是只少删不多删**，只影响活性。推翻条件：txg 1 记录落盘而根没落时，下一实例从 txg 1 重发（读代码是 `crates/singlefs-core/src/mount.rs:1534`，没跑崩溃用例）。

## 六、检查单八格

| 格 | 判 | 依据（这一次现量） |
|---|---|---|
| 走不到的理由二次查实 | **成立** | 补丁 `~/.local/share/singlefs/work/m3-close-b20-code-r1-fixes/patch/crates.patch` 第 1–187 行（三份 `src`，第 188 行起是测试文件）`grep -nE '^\+.*(todo!\|unreachable!\|assert!\|panic!\|\.expect\(\|unwrap\(\|unimplemented!)'` 0 行。新分支的调用点只有 `crates/singlefs-core/src/mount.rs:9193` 一处，没有走不到的臂 |
| 类型与名字 | **成立** | 字段是 `u64` 计数（`crates/singlefs-core/src/recovery.rs:5129`）；同一段 `grep -nE '^\+.*(_ =>\|as u[0-9]\|as usize)'` 0 行；没有缩写。旁注：判官那一份字段叫 `slots_of_another_pool`（`crates/singlefs-verification/src/mount_transparency.rs:2337`），两份不同名，判官是独立写的，不算错 |
| 不兼容旧代码与旧格式 | **成立** | 同一段 `grep -nE '^\+.*(legacy\|兼容\|旧版\|旧格式\|迁移\|version)'` 0 行 |
| 不新建判定库 | **成立** | 整份补丁 `grep -nE '^\+.*(rocksdb\|sqlite\|prune\|\.tsv\|File::create\|fs::write)'` 0 行 |
| 所有任务多机可用 | **成立** | `crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs` 全文 `grep -nE 'std::env\|env::var\|hostname\|/home/\|std::fs\|std::process'` 0 行；盘是内存稀疏盘 |
| 测试优先多线程 | **成立，不该并行** | 三条新用例各一个盘面一次挂载（`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:1093`、`:1125`、`:1157` 起），没有互相独立的单位可切 |
| 带 --item 的命令与项 | **成立，10312 等结局** | 变异行 6 行还在，行号今天是 `crates/mutations.tsv:7061-7066`（正文写 7064–7069，别的会话删了前面三行，`grep -n` 现量）。证红输出原样见下。`task-server/task status 10312`：第 1 次撞内存上限（退 250）、第 2 次在跑。没有新崩溃枚举项：`.claude/gate.d/crash-point-replay-items.tsv` 里 `grep -c writable_mount_counts_system_configuration` 是 0 |
| 新崩溃流多罩了什么 | **成立（没有）** | 补丁只动四个文件（`grep -n '^diff --git'` 四行），没有 `crash_enumeration_` 与崩溃点重放项 |

第 7 格旁注：任务 10241 的证红跑在改名之前。`task-server/task log 10241` 第 8 行原样：

```
✓ 点名 3 条：现判 3 条、复用 0 条、拒算 0 条；跑了 3 条，抓到 3 条（其中复用 0 条），本次未跑 0 条，跑了的都抓到了
```

同一份日志第 4 行红的测试名是 `a_hand_made_image_with_the_older_slot_of_every_device_from_another_pool_is_refused`，今天文件里与变异行里是 `hand_made_image_with_the_older_slot_of_every_device_from_another_pool_is_refused`（`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:1093`）。6 行变异对今天这些名字的证红只有 10312 那一趟，所以这一格等它。另外三行（`crates/mutations.tsv:7063-7065`）只靠它。

## 没打中的形状

| 试过什么 | 范围 | 结果 |
|---|---|---|
| 别池、撕裂、全零、读错同现，实现自己分流错 | 第一节 12 种盘面（按代码走，没跑）；3 块盘以上没走，判法只有 `any`，不随盘数变 | 没打中 |
| 第一遍与重读那一遍形状不同 | 第一遍 R2 真重读乙真、第一遍乙真重读假（实现员有用例）、第一遍假不重读 | 没打中 |
| 新字段被别的路径读到 | `grep` 三个包的 `src` | 没打中 |
| 离线修复路径走错 | 每块盘一槽别池（用例）；一块盘两槽别池（读代码） | 没打中 |
| 只读挂载走错 | 别池世代号小（用例对照）：照常；世代号大：B1 | B1 |
| 判官把实现的合法拒判错 | 第一节 12 种盘面；别池世代号大、每块盘各一时择到别池，结论碰巧对（第一节第 3 行） | A1、A3 之外没打中 |
| 随机历史新步判的与产品不同 | 读代码：每块盘、只第一块盘两支 | 没打中 |
| C973 判据：S 不是 2 的幂 | S = 4..=16 闭式（待跑）、证明 | 没打中 |
| C973 判据：环绕、跳号、三槽同错 | 同上，模型 `misread`（待跑）；跳号取 11 组（空、1、2、1+2、3S、3S+1、3S+2、1+(3S+1)、2+(3S+2)、3S+6S、1+(3S+1)+(6S+1)），最大 txg 从 3S−1 到 9S+3 每一个 | 错位读没打中；射程句 C1 |
| 第一次崩溃之后 F2 退回读到别池的槽 | 读代码：只读 `self_verified`（第三节） | 没打中 |

**「没打中」只这一份样本**，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，C973 判据要算第二份样本还要主 agent 复跑模型，且模型是手抄的。

## 这条腿自己的限度

- 一条用例、一个模型都没跑。第一节表、四句里的走向全是读代码推的。用例断言里写的就是我的预期，**红了说明我推错，不说明产品错**，主 agent 跑完逐条对。
- `ring_pass_model.py` 照代码手抄，与上一轮模型同一种，不是独立校验路子。闭式核调实现的私有函数，只比判据一个函数，不比整遍清扫。
- B2、B3 的「倒挂」只断言第一次发布的 txg ≤ 6，没崩在暖机第二次发布之前跑池级 checker。「池级 checker 红」是拿 E205 六格丙那一列类推的。
- A1 的盘面里盘 0 与盘 1 的别池槽是同一个 fsid；两个不同的别池 fsid 时 `choose` 报对不上、判据按真，不打中 A1，落回 A2。
- E205 页只 `grep` 了四个词，没通读。C971 的实现员目录、D22 变更史没读。
- 禁读的两份没读；上一轮模型读了（要知道它的取样范围，不算禁读清单），上一轮判决读了。
- 改法都是我提的，**只在我的推理上过、被攻过零轮**；下表每格都是「推的」。

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| 判官收本池 fsid、择系统配置走筛 fsid 的择法 | A1、A2 | 推的，待主 agent 跑（要改副本再跑，这一轮没实现） |
| 判官判槽自证不过不经 checker 的次序 | A3 | 推的（没实现没跑） |
| D22 射程写明别池世代号大的那一形只剩离线出路 | B1 | 推的（改条款） |
| C971 射程写明落后盘那一形归 C6、不罩，或换乙直拒（要用户定） | B2 | 推的（改条款；换读法没实现） |
| 条款定「别池」与「布局不认」谁优先 | B3 | 推的 |
| C973 射程句改成「根槽写没落（丢写或记录落了根没写的崩溃），最多停到下一次写进这一槽」 | C1 | 推的（改条款） |

## 要主 agent 跑的命令

1. **模型与盘面用例整份跑一次（`tools/rerun.sh`）。** 在仓根起：

   ```
   task-server/task submit --cwd /home/fy5090/code/singlefs -- bash research/prompts/m3-close-b20-code-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r2-legs/copy --work /home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r2-legs/rerun-1
   ```

   范围声明在脚本里两条 `cargo test` 各自那一行（两条都是 `种类=target`、1 个测试目标）。它把冻结副本拷进 `<work>/tree`，追加两份 `.rs`（只进这棵树），跑模型、闭式核与盘面用例。

   跑完看：
   - `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`（退非 0 的件数为 0 算过）；输出末行 `name=rerun_exits model=0 core=0 verification=0`。
   - `<work>/out/attack-cases.out` 的 `test result: ok. 8 passed; 0 failed`（8 是 `grep -c '#\[test\]' attack_cases_appended_to_the_test_target.rs` 的数）。**某一条红就是我推错了那一格**，那一格打中作废：红在「读数：…」哪一句，它就推翻哪一句。
   - `<work>/out/core-closed-form.out` 的 `name=genesis_closed_form cells_compared=` 与 `test result: ok. 1 passed`。
   - `<work>/out/ring_pass.out`：每个 S 的 `mode=misread` 一行没有 `hidden=`；`mode=stale` 有 `hidden=`，另数两类 `hidden-caught-by-rejudging-after-rereads`、`hidden-not-caught-by-rejudging`；`name=sweep_goes_through S=4 skipped=[1]` 是 13、`[1, 13]` 是 25、`[1, 13, 25]` 是 37。

   全量估时（推的，没量）：S = 4、5、6 的三槽全枚举约 450 万次清扫，每次几十微秒，约 5 到 10 分钟，远低于 40 分钟；每个 S 的 `name=ring_pass_cost` 打实测秒数。这一条没有调崩溃枚举函数，崩溃状态数 0，盘面用例每条一个盘面。

2. 判决要引的数由主 agent 照 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做，不派子 agent 去核子 agent」第 3 步把模型目录拷到草稿目录再跑，第 1 条的 `--work` 与模型路径照改；副本上的数不进 kb，A1、B1、B2 要引就在入库用例上重做。

3. **要不要把 A1、A2、B1 做成入库用例**：判决定。用例草稿就是 `attack_cases_appended_to_the_test_target.rs` 里的同名几条（测试目标 `writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors`）；A1、A2 在判官改好之后要把断言翻成「判官判合法」「判官判不合法」，用来钉改法。

## 做完的、没做的、等什么

**做完**：读完正文、判决、实现员报告、被判三处代码、判官、只读挂载、离线出路、随机历史新步与理想模型；写了 8 条盘面用例、1 条闭式核、1 份模型与 `tools/rerun.sh`；八格逐格回了。

**没做**：
- 不判正推、辩方那几格；不替主 agent 采纳。
- 没往统一任务服务交任务，上面的用例与模型一次没跑；判决要引的数没复跑。副本上量出的数不算入库装置上的数。
- 没改 `crates/`、kb、记录；改法都只写在报告里。
- B2 没跑池级 checker；C1 没造真实的崩溃流（只读代码加算术模型）。

**等**：第 1 条的结果。A1、A2、A3、B1、B2、B3 与 C1 的「打中」都要它坐实；主 agent 跑完如要腿接着读，派同立场新腿带「承接：」本报告路径与任务号。

**没删** `/home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r2-legs/copy`（`du -sh` 量的 51M）：第 1 条的 `--frozen`，已在 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r2-legs/.deliverables` 写成交接，主 agent 复跑完删。
