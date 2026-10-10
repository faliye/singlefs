# m3-batch35-36-code-r1 攻方（Opus）报告：K2 哪一格会错（2026-10-08）

立场：攻方，假设第 35 批 W、第 36 批 V 的改法是错的，在冻结副本的拷贝上造反例。只跑 harness 档；checker 档测试与 GPU 不跑，GPU 一侧读 wgsl、按 CPU 判法算期望（推的）。
被判对象：冻结副本 `/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/`（20 份 sha256 与 `research/prompts/m3-batch35-36-code-r1-snapshot/crates-sha256.txt` 逐份相同，拷进草稿目录之后 `sha256sum -c` 0 份不符）。
副本上量的数只算副本上的数，不算入库装置上的数。下文 `out/<名>.log` 指 `rerun.sh` 工作目录里的日志；报告里抄的每一行判定都在 `research/prompts/m3-batch35-36-code-r1-opus-model/expected-summary.txt` 里（行首是日志名加一个制表符），按行核过：17 行代码块里的判定行 0 行不在。

## 各格判定一览

| 格（正文第三节 K2） | 判定（一次抽样） | 一句话 | 节 |
|---|---|---|---|
| W1 窄读：被豁免的槽这次挂载里释放、过回收门槛、根槽没写掉时发出去 | 没打中（读码） | 记根、抬 F 两条回收路都按还没解开的 C 补隔离；装置上没走到回收门槛 | 八 |
| W1 / W2：F2 退回之后读得出的较新根 R、它的记录全读不出（H1） | **打中**（双故障） | C 与 C_ACCOUNT 只从记录起算，影子账只管已被抛弃的根，R 两边都不管；M1 写行之前写掉 R 的 14 份单元，checker 红 I-2.1 / I-4.8 / I-7.2 / I-7.4；ow 每个 k 22 格中 9，sw3 10 格 | 一 |
| W3：F1 按 F2 退之前的水位判（H2） | **打中**（双故障） | 读阶段以 R 为水位判 F1，退到 P 之后不再判；按字面该拒的 12 格挂得上、用户动作之后写掉 R 的数据单元 | 二 |
| W2：C_ACCOUNT「账读不出交 F1」（H3） | **打中**（三故障） | 缺末条、账上节点读不出时 F1 只看得见数据单元、不拒；末条单独点名的单元没人护（根藏 12 格、根读得出另 12 格） | 三 |
| W3：F1_ANY 放过其实读不出的单元 | 没打中 | 单份坏 182 格（R 看得见）0 拒、0 写掉 | 八 |
| W4：F2_EXACT 仍退过已确认的一版（H4） | **打中**（双故障，落在用户接受的那一形之外；F2 字面许可） | R 的单元 + P 那次轮换写的较新系统配置槽读回全零 ⇒ 三处都落 P 之前那一版，P 一个字节没坏；4 / 4 组 | 四 |
| W4：F2_EXACT 该退不退 | 不另报 | 见证记录读不出不退是条款原话 | 八 |
| V1、V3、V4：GPU 与 CPU 判得不同 | 没打中（读码推的） | F2 逐支、拼读、并成一个调用点都对得上 | 五 |
| V2：最终折叠表下 I-3.5 漏判 | 抽象模型打中（推的，真盘面没造） | 穷举 165888 个字面要判的落点里 96 个只在最终折时漏；净减 912；形在 I-2.1 已红的坏镜像上 | 六 |

攻方自提四个改法（`OPUS_C_NEWER_ROOTS`、`OPUS_F1_AFTER_F2`、`OPUS_C_MAPPING`、`OPUS_F2_NEWEST`）各修哪一格见第七节，**只在我的模型上量过、被攻过零轮**。

## 复跑

模型目录 `research/prompts/m3-batch35-36-code-r1-opus-model/`。在空目录里串行逐阶段跑（每个阶段前台 4 分钟内；这一次 `prep build` 13 秒，最长的 `sw3single` 180 秒）：

```
SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree SINGLEFS_TOOLS=<仓根> THREADS=7 \
  bash research/prompts/m3-batch35-36-code-r1-opus-model/rerun.sh <空工作目录> [prep build h1 ow sw3vis sw3hid sw3f1 sw3both sw3all3 sw3single w4 exist v2model summary]
bash research/prompts/m3-batch35-36-code-r1-opus-model/compare.sh <空工作目录>/summary.txt
```

交回前在空目录 `~/.local/share/singlefs/work/m3-batch35-36-code-r1-opus/rerun1/` 从头跑通，原样：「COMPARE same lines=1511」。之后那份工作目录里的仓副本与编译目录已删，日志与 `summary.txt` 留在原处。

| 文件 | sha256 |
|---|---|
| `compare.sh` | 0360141e0ec401f7cba2fe1a00222428240b33ff81ca88a9b8ecea5e548efa6c |
| `expected-summary.txt` | c8d0166843391f466e29e8081a31f931505d1a4468e7b5237c9be97ad6247ea1 |
| `patch_arms.py` | 6f847fcd6e8e1c78975bcaefb03f53562e0bb2d562baeaaea885e78cdabfcc5b |
| `probe/opus_b3536_probe.rs` | f0d360d424a044b9403c958db484e2447e9724e585ce70e0dffa5182fc1f42e4 |
| `rerun.sh` | 99bd6c4be64df1668d4f6d9c7b96ee88f59344d30d7b5e727a2df4a29c69206d |
| `summarize.py` | 2642bdbbf6d2afb010ee561aefbb8605de1a24d9ca1498d51c7c91aa890117d0 |
| `v2_fold_model.py` | 31b828fb73a30cca1e655d07a50491758855b335c2c3a7cf119cf12e1e71a778 |

估时：先跑一小段（ow、深度 1，44 格）20 秒，按格数与结点数放大估全量约 20 分钟，没超 40 分钟，历史、候选与几何都没缩。并行：没有，全部串行、同一时刻一条 cargo。开跑前 `ps` 看到别的会话有一条 `cargo test -p singlefs-harness`（另一组测试），没有性能测量在跑；加 `nice -n 19` 照跑，没等锁。

## 一、W1 / W2：F2 退回之后，读得出的那条较新根 R 谁都不护（H1，打中）

冻结副本记作 T = `/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree`，下面引代码一律写 T 下的绝对路径。

**形**（双故障，同 T 那一轮 C 盲区的故障数）：384 槽小盘、第一个文件之后覆盖写 k 次，录一次覆盖写 R（一条记录），崩在 R 的根槽落盘之后、系统配置轮换之前（入库装置 `tests/common_unwitnessed_publish/mod.rs` 原样）。取新实例的那次可写挂载 M1 里读回全零：R 的某一个元数据单元（两份）**加** R 那一条记录（两块盘各一份）。R 的根槽读得出。

**每一步许可它的那一句**：
1. R 的根自证、txg 最大，被择；走读失败，系统配置没见证 R（R 的末条读不出，计数器 = c_见证 的那条是 P 的、比 R 旧）⇒ 退回 P：`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/recovery.rs:5488`：「(None, Some(witnessed_publish)) => {」；可写挂载照它退：`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/mount.rs:6429`：「之后的一切（取号之前逐盘核、写行、影子账与 C）都按退到的那一版。」
2. 影子账只隔离按最新根的实例表算被抛弃的根：`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/mount.rs:1534`：「for root in roots.iter().filter(|root| is_abandoned(root)) {」。M1 那一刻最新根就是 R，R 按它自己的实例表有效，不在这一圈里。
3. C 只看记录：点名那一份与 C_ACCOUNT 都从水位之上读得出的记录起算（`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/mount.rs:1754`：「for record in records.values().filter(|record| {」）。R 唯一那条记录读不出，C 是空的。
4. R 新分配的单元在 P 的账里是空闲槽，M1 写行那次发布取的提交内生块落在上面——代码自己的注释就写着这一后果：`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/mount.rs:1569`：「它比所选那一版新，它新分配的单元不在现行账里、回收门槛也罩不到，挂载之后当空闲槽发出去。」W1 的修法（`mount.rs:1574` 只豁免现行账）只在 R 的记录读得出时起作用。

**原样输出**（副本，`out/h1-base.log`；探针 `opus_h1_the_k4_1_form_with_the_record_of_r_also_reading_back_zero`，上一行是入库用例 `a_power_loss_before_the_row_root_of_the_mount_still_reads_back_a_candidate_root_whose_mapping_root_read_back_zero_at_that_mount` 那一形原样，下一行只多藏 R 的记录）：

```
OPUS H1 form=K4-1:mapping_root M1=Ok effective=P isolated=[12, 12] hidden_ranges=2 before_row_root:r_in_ring=true r_copies_overwritten=0 at=[] chk={} rec=FileRead(R content=R)
OPUS H1 form=H1:mapping_root+record M1=Ok effective=P isolated=[0, 0] hidden_ranges=4 before_row_root:r_in_ring=true r_copies_overwritten=14 at=["d0@25165824", "d1@25165824", "d0@25182208", "d1@25182208", "d0@25198592", "d1@25198592", "d0@25214976", "d1@25214976", "d0@25231360", "d1@25231360", "d0@25247744", "d1@25247744", "d0@25264128", "d1@25264128"] chk={"I-2.1", "I-4.8", "I-7.2", "I-7.4"} rec=FileRead(P content=P)
```

读法：多一个读回全零的段，C 隔离的槽从每盘 12 掉到 0；写行那次的根落盘之前 M1 写掉 R 的 14 份单元，R 还在根环里；断电之后那份盘面上池级 checker 红 I-2.1、I-4.8、I-7.2、I-7.4，恢复读回 P——同一份盘面不多藏记录时恢复读回 R（入库用例断言的就是这一句）。

**放开用户动作扫**（只固定前缀与故障）：每格可写挂载之后用户动作取 {o 覆盖写, s 顺序写两个数据单元, n 建 inode} 长度 ≤ 2 的全部序列（13 个结点），数「R 的根还在环里时 R 的单元被写掉」的格。藏法取全部组合：R 的哪个单元（不藏、10 个单元各两份）× 记录藏不藏 × 根槽藏不藏。`vis=root` 是根槽读得出的那一半。副本原样（`out/cbd2-ow-k*-base.log` 经 `summarize.py`）：

```
cbd2-ow-k1-base	TALLY kind=ow vis=root M1=Ok r_units=overwritten cells=9
cbd2-ow-k3-base	TALLY kind=ow vis=root M1=Ok r_units=overwritten cells=9
cbd2-ow-k5-base	TALLY kind=ow vis=root M1=Ok r_units=overwritten cells=9
cbd2-ow-k7-base	TALLY kind=ow vis=root M1=Ok r_units=overwritten cells=9
```

四个 k 上都是 22 格里 9 格：记录藏了、9 个元数据单元任藏一个（数据单元 u0 藏了 R 照样建得出上一版、不退，不算）。记录不藏的 11 格 R 的单元一份不少（C 点名那一份护住）。三条记录的顺序写 R（`sw3`，k = 5，藏法 13 单元 × 8 种记录子集 × 根槽两态 = 224 格）上同形：根槽读得出、三条记录全藏、10 个元数据单元任藏一个，10 格全中（`out/cbd2-sw3-k5-rootvisible-base.log`，`rec_hidden=111` 那一行）。

**四句**：
- 分不分辨臂：不分辨 W1 的两臂（豁免候选根 / 只豁免现行账）——两臂下 C 都是空的，病根是 C 只从记录起算这一共用前提。分辨的是「C 只看记录」与「C 也看环里比所选那一版新、按实例表有效、读得出的根」。
- 系统当时看不看得到：看得到。R 的根槽在 M1 那一刻自证过、txg 比所选那一版大、按实例表有效；F2 正是因为读到了它才退。藏的单元不在 R 的账的路上时（ow 上 u1、u2、u3、u7、u8 五个；u4、u5、u6、u9 在账的路上，u9 是树表），R 的账也读得出。
- 满足判据字面的哪一句：D23 已定项 14 主句，`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`：「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」——R 正是 M1 的 F2 退回所抛弃的那条根，离开根环之前它的单元被重新分配。C 那一段的分工句（同一节：「它护的是挂载那一刻根槽读不出、系统配置又没见证到的那条根新分配的单元：影子账只查环里读得出的根的账，看不见这条根」）默认读得出的根归影子账管，而影子账只管按实例表已被抛弃的根，F2 刚退掉、实例表还没写的 R 两边都不管。
- 跑前条款给的改法在这几格上还中不中：T 判决给的 `C_ACCOUNT`（记录拼根）与 K4-1（去掉候选豁免）在这几格上**照样中**（上面的数就是冻结副本，即两样都已落地）。

不中的形（对照）：根槽与全部记录都藏（`vis=none`，R 在 M1 那一刻整个看不见）在 k = 5 上也被写掉（11 / 11 格），那一形 M1 看不见判别子，按「判别子观测不到」记，不算打中；T 判决与入库用例 `urJ` 那一形已认它。

## 二、W3：F1 按 F2 退之前的水位判，退到 P 之后不再判（H2，打中）

**形**（双故障）：同一装置，R 取三条记录的顺序写（`sw3`：每个数据单元一条记录，末条另点名全部元数据单元），k = 5，崩在 R 的根槽之后、轮换之前。M1 里读回全零：R 的第 1 条记录（两份）**加** R 账上的一个节点 u7（两份；它只被末条点名）。R 的根槽读得出。

**每一步**：
1. 读阶段的 F1 拿重放之后的所选那一版当水位：`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/mount.rs:6203-6207`：「let publishes_missing_records_whose_named_units_do_not_verify = … &effective_root,」。这时所选那一版是 R 本身，R 的记录不在水位之上，F1 一条都不判。
2. 之后建上一版走读失败、F2 退回 P（同第一节第 1 步），`mount.rs:6429` 那一句说退了之后「影子账与 C」都按 P，F1 不在其内，不再判。
3. C 按 P 当水位：记录 2、3 点名的落点隔离（每盘 15 槽）；C_ACCOUNT 拿记录 2 的新根段拼根读账，u7 读不出，整份账不交（`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/mount.rs:1735`：「账读不出、解不开的那次发布不交：那时它的单元多半也验不过，交 F1 乙。」）——而 F1 已经在第 1 步过去了。只被第 1 条记录点名的数据单元没人护。
4. 按条款字面这一形该拒：`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`：「水位之上一次没施加的发布，读得出的那几条记录本次发布内序号不连续或缺末条、而它们点名的单元里有一个两份副本都验不过」——以 P 为水位，R 读得出的是第 2、3 条（序号不从 1 起），第 3 条点名的 u7 两份都验不过。

**原样输出**（副本；上一行冻结副本原样，`out/cbd2-sw3-k5-rootvisible-base.log`；下一行加臂 `OPUS_F1_AFTER_F2`，F2 退了之后按 P 再判一次 F1，`out/cb-sw3-k5-rootvisible-f1.log`）：

```
OPUS CB kind=sw3 k=5 unit=u7c2 rec_hidden=001 root_hidden=false vis=root M1=Ok effective=P isolated=[15, 15] before_row_root:r_copies_overwritten=0 chk={} rec=FileRead(R content=R) sweep(depth=2,nodes=13):max_r_copies_overwritten_while_r_in_ring=2 at=ns
OPUS CB kind=sw3 k=5 unit=u7c2 rec_hidden=001 root_hidden=false vis=root M1=Refused:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify
```

读法：M1 做成，用户先建一个 inode 再顺序写（`at=ns`），R 还在环里时 R 的第一个数据单元两份被写掉。按字面重判 F1 时这一格在任何写之前拒。

扫描：`sw3`、k = 5、根槽读得出的 112 格里，记录子集 {1}、{2}、{1,2}（`rec_hidden` = 001 / 010 / 011）× 账上的四个节点（u7、u8、u9、u12，u12 是树表）这 12 格全中；同样三种记录子集配别的单元（u3–u6、u10、u11，不在账的路上）时 C_ACCOUNT 读得出账、护得住，不中，但按字面也该拒——`OPUS_F1_AFTER_F2` 臂下这 30 格全拒（`TALLY kind=sw3 vis=root M1=Refused:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify r_units=kept cells=30`）。

**四句**：
- 分不分辨臂：分辨 F1 的「判时取的水位」（退前的所选那一版 / 退后的那一版），不分辨 F1_ANY 与「两份都要验过」两臂（这几格里 u7 两份都读回全零，两臂判得一样）。
- 系统当时看不看得到：看得到。F2 退回之后 P、R 的记录、u7 两份都验不过，都在挂载进程手里（`rebuilt_allocator` 拿的就是同一份 `records`）。
- 满足判据字面的哪一句：上面第 4 步那一句；水位按 D23 已定项 14 是「所选那一版」，F2 退回之后所选那一版是 P（`mount.rs:6429`）。
- 改法还中不中：T 判决给 F1 的改法 `F1_ANY` 碰不到这一格（它改的是「验不过」的判法，不是判时的水位）；`C_ACCOUNT` 的「交 F1」那一半在这一格上落空。

## 三、W2：C_ACCOUNT「账读不出就交 F1」，而 F1 只看读得出的记录点名的单元（H3，打中）

**形**（三故障，与 T 那一轮 C 盲区同族、多一个）：`sw3`、k = 5。M1 里读回全零：R 的根槽、R 的末条记录（第 3 条）、账上的节点 u7（两份）。

**每一步**：R 的根槽读不出，所选那一版是 P，不经 F2。F1 以 P 为水位判 R：读得出第 1、2 条（缺末条），它们只点名数据单元 1、2，两份都验得过 ⇒ 不拒（`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/mount.rs:6099`：「!crate::recovery::every_named_unit_of_the_record_has_a_copy_that_verifies(」，只遍历读得出的记录）。C_ACCOUNT 拼出的根读账读到 u7 读不出，`mount.rs:1770` 起那一段 `let Some(placements) = … else { continue; };` 跳过。只被末条点名的那些单元（数据单元 3 与全部元数据单元）没人护。`mount.rs:1735` 那句的前提「那时它的单元多半也验不过」在这里不成立：验不过的 u7 只被读不出的那条记录点名，F1 看不见它。

**原样输出**（`out/cbd2-sw3-k5-roothidden-base.log`）：

```
OPUS CB kind=sw3 k=5 unit=u7c2 rec_hidden=100 root_hidden=true vis=rec M1=Ok effective=P isolated=[6, 6] before_row_root:r_copies_overwritten=0 chk={} rec=FileRead(R content=R) sweep(depth=2,nodes=13):max_r_copies_overwritten_while_r_in_ring=2 at=ns
```

扫描：根槽藏着的 112 格里 `TALLY kind=sw3 vis=rec M1=Ok r_units=overwritten cells=12`：记录子集 {3}、{1,3}、{2,3} × u7、u8、u9、u12。根槽读得出时同样这 12 格加 {1,2,3} 那一行也中（第一节）。

**四句**：分辨「账读不出时什么都不加」与「账读不出时照被抛弃根同一份办法（从中央映射现算，C393 (b)）」两臂；系统看得到（拼出来的根的中央映射树根 u11 读得出）；字面：D23 已定项 14 C 那一段「账读不出就不加，交「记录有缺口时拒可写」」——这一形交过去 F1 不拒，C 段的「它护的是…那条根新分配的单元」没兑现；改法：`C_ACCOUNT` 本身就是这一格的现行改法，照样中。

## 四、W4：F2_EXACT 退过一版已确认、单元全读得出的 P（H4，打中，落在用户接受的那一形之外）

**形**（双故障）：同一装置，崩在 R 的根槽之后、轮换之前（R 没见证；P 是 R 之前那一版，它那次轮换写的系统配置槽每块盘一个，是较新的那一槽，P 已确认）。读的那一刻读回全零：R 的单元（设计里那一退）**加** 每块盘上较新的那个系统配置槽（P 那次轮换写的）。P 的根、记录、单元都读得出。

**每一步**：c_见证 取读得出的槽里 tail 的最大值（`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/recovery.rs:5465`：「.map(|slot| slot.quantities.journal_tail)」），较新那一槽读不出，c_见证 落成 P 之前那一版 Pm 的计数器；R 的末条读得出、计数器大于 c_见证 ⇒ 没见证；只退到计数器 = c_见证 那条记录所属那一版本身（`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/recovery.rs:5509`：「order == witnessed_publish && order < chosen」）⇒ 落到 Pm，跳过 P。

**原样输出**（副本，`out/w4-base.log`；两种 R × k ∈ {3, 5} 四组全同形，这里抄 ow、k = 3 那一组；`W4INFO` 行是每块盘两槽的世代号与 tail）：

```
OPUS W4INFO kind=ow k=3 R=(1,7) P=(1,6) Pm=(1,5) slots=["d0@0:gen8:tail6", "d0@4096:gen7:tail5", "d1@0:gen8:tail6", "d1@4096:gen7:tail5"]
OPUS W4 kind=ow k=3 hidden=units_of_R rec=FileRead(P content=P) ro=Ok(P) rw=Ok(P)
OPUS W4 kind=ow k=3 hidden=newer_cfg_slots rec=FileRead(R content=R) ro=Ok(R) rw=Ok(R)
OPUS W4 kind=ow k=3 hidden=units_of_R+newer_cfg_slots rec=FileRead(Pm content=Pm) ro=Ok(Pm) rw=Ok(Pm)
```

读法：两样故障各自单独出现时都落对（R 的单元藏 ⇒ P；较新槽藏 ⇒ R）；一起出现时恢复、只读挂载、可写挂载三处都悄悄读回 Pm，可写挂载在 Pm 上取号、写行，已确认、一个字节都没坏的 P 被新实例抛弃。

**四句**：
- 分不分辨臂：不分辨 F2_EXACT 与 T 之前的「不超过上限的最新可读根」（两臂都落 Pm，上限同样是 Pm）。分辨的是「只认 c_见证 那一版」与下面「退到 [见证到的那一版, 所选根) 里最新、不被抛弃的那条根」。
- 系统当时看不看得到：看得到。P 的根读得出、在 Pm 与 R 之间；R 的记录与根都在，按 D16（发布语义） 已定项 7 的持久顺序，R 写记录之前 P 那次轮换已过屏障——R 存在这件事本身就证明 P 被确认过（这一条是按持久顺序推的，没在装置上逐写核）。
- 满足判据字面的哪一句：F2 那一段的字面许可这一退（D23 已定项 14：「只退到系统配置见证到的那一版本身」，c_见证 按读得出的槽算）。用户知情接受的代价只有一形：`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`：「已确认的版本在较新的系统配置槽与它的单元同时读回全 0 时，恢复、可写挂载与只读挂载悄悄读回上一版」——这一形里读回全零的是 R 的单元，P 的单元读得出，不在那一形里，退的也不是「上一版」而是上上一版。所以按的是「打中落在用户接受的那一形之外」，不是「F2 字面被违反」；这一格要主 agent 判是条款写宽了还是代价句写窄了。
- 改法还中不中：T 判决的 `F2_EXACT` 照样中（数就是冻结副本）。

攻方自提改法（只在我的模型上量过、被攻过零轮）`OPUS_F2_NEWEST`：退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根；实例表读不出时只认恰好相等的那条；见证记录读不出照旧不退。副本上原样（`out/w4-f2newest.log`）：

```
OPUS W4 kind=ow k=3 hidden=units_of_R+newer_cfg_slots rec=FileRead(P content=P) ro=Ok(P) rw=Ok(P)
```

四组全变成 P；同一臂下入库的 F2 两份用例原样过：「f2newest recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed rc=0 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out」「f2newest read_recovery_fields_follow_the_fallback_to_the_witnessed_version rc=0 test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out」（T 那一轮的 A、B 两形与 S1 都在前一份里）。它要 GPU 恢复内核同改一处（下限、实例表滤），没改、没量。

## 五、V1、V3、V4：GPU 与 CPU 判得不同的一形（没打中，读码推的，GPU 不跑）

逐支对 CPU（`recovery.rs:5425` 起那个函数、`recover`）读 wgsl，按 CPU 判法算期望，没找到判得不同的取值：

| 支 | GPU（冻结副本） | CPU | 判 |
|---|---|---|---|
| c_见证 | 每盘两槽，自证过取 tail 最大；槽距不按 512 对齐算报读错（`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:111`：「if ((offset % SECTOR_BYTES_OF_THE_MEMORY_POOL) != 0u) {」） | `system_configuration_slots_of_one_device_read_once`，内存池按扇区（512）判对齐与越界 | 崩溃态里两边的「读错」只有不对齐一种，同（推的） |
| 见证过 / c_见证 = 0 / 记录读不出 | `/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:160`：「if (is_witnessed || is_zero64(witnessed_counter) || !has_witnessed_record) {」 | `(Some, _)`、`(None, Some)`、`(None, None)` 三支，`record_at_the_witnessed_counter?` | 同；计数器从 1 起，c_见证 = 0 时 CPU 那一支也是「记录读不出」 |
| 恰好相等的根 | 按 R_ROOTS（choose_root 收下的根，已滤掉指针头不是第一版的）取第一条（`crash_judge_recovery.wgsl:170`） | `readable_roots` 里 `find` | 同一个 (txg, 实例) 一个环里只有一条，取第一条与 `find` 同 |
| 走两遍 | `/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:55`：「if (walk_round == 1u || walked.outcome != OUTCOME_FAILED || !consult) {」；退回那一版走不通照报第一遍 | 同；`Ignore` 不退 | 同 |
| 拼读 | 空段最后一槽、读两槽、整 32 KiB 在盘内才拼（`crash_judge_common.wgsl:561`）；拼出的 CRC 对得上而要往下读字节的记判不了（`crash_judge_common.wgsl:668`） | 稀疏读者读「16 KiB 全零 + 下一落点前 16 KiB」 | 单元测试钉了 CRC 拼法；不对得上的照判不上，对得上的回落 CPU，不会判出不同（推的） |
| 并成一个调用点 | `/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:678`：「for (var copy = 0u; copy < 2u; copy = copy + 1u) {」 | `read_unit_via_locations` 先第 0 条后第 1 条 | 次序、早退、`no_slice`、判不了记号与改前逐条同 |

我的 H1–H4 都是 core 的行为：H1–H3 是挂载期隔离（GPU 不判挂载），H4 的 F2 GPU 照 `F2_EXACT` 同判，CPU、GPU 一起落 Pm，不分叉。若采 `OPUS_F2_NEWEST`，恢复内核那一支要同改（第四节末）。

## 六、V2：最终折叠表下 I-3.5 漏判的一形（抽象模型打中，真盘面没造）

V2 把每版走完当场按折叠表折，改成全部走完之后按最终的表折（`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-checker/src/walk.rs:11414`：「&walk.versions_folded_into_units_by_the_final_fold_table(),」）。表只在一处被改：读回第 1 条那一份、那个落点此刻还折在这条指针第 0 条上时改记成它自己（`walk.rs:1347`）；记引用时 `or_insert`（`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-checker/src/walk.rs:1395`：「self.unit_of_each_placement.entry(placement).or_insert(unit);」）。

模型 `v2_fold_model.py` 照这两条规则穷举（落点 4 或 5 个、3 版、每版一条指针、每条第 0 份好 / 坏、走读次序全排列），数「条款字面（按落点）要判、折成单元之后不判」的落点。原样（`out/v2model-4-3-1.log`、`out/v2model-5-3-1.log`）：

```
V2MODEL P=4 V=3 M=1 cases=82944 literal_judged_placements=165888 missed_snapshot=40752 missed_final=39936 missed_final_not_snapshot=96 missed_snapshot_not_final=912
V2MODEL_EXAMPLE ((((0, 1),), ((0, 1),), ((1, 2),)), ((1,), (0,), (0,)), (1, 0, 2), 1, 2, 2)
V2MODEL P=5 V=3 M=1 cases=384000 literal_judged_placements=921600 missed_snapshot=169800 missed_final=167760 missed_final_not_snapshot=240 missed_snapshot_not_final=2280
```

例子读法：txg 0、1 两版都指着落点对 (0, 1)，txg 2 那一版指着 (1, 2)；先走 txg 1（读第 0 份）、再走 txg 0（读第 1 份，把落点 1 从单元 0 拆成它自己）、最后走 txg 2。相邻对 (txg 1, txg 2) 里落点 2 是新出现的、字面要判；按最终表，落点 2 折进单元 1，而 txg 1 的落点 1 此刻也折成单元 1 ⇒ 不判；按改前的当场折，txg 1 那时的单元只有 0 ⇒ 判。改法净减漏判（912 减、96 增；5 落点 2280 减、240 增），但不是只减不增。

四句：分辨「当场折」与「最终折」两臂（96 格只在后者漏）；判别子看得到（走读手里有每一版的落点集合，不折就是字面）；字面：I-3.5 出现规则「R 引用而 R⁻ 不引用的每个落点 b」（`.claude/kb/invariants.md` 第 146 行那一行）；改法：S 判决给的 F1（就是 V2）在这 96 格上造出漏判，「按落点直接比、只在取诞生代号时折」两边都不漏（推的，没实现）。限度：例子要两版同一对落点却校验和不同（同一处原地换了内容），盘面上 I-2.1 必已红，与 S 判决认的那一形同类；真盘面要照 S 那一轮攻方的造镜像法改指针校验和，我没造，算推的。

## 七、几个改法各修哪一格

「量过」= 副本上贴了原样输出（`out/` 下同名日志，复跑见文件头）；「推的」= 按代码推、没实现或没跑。全部臂都是攻方自提：**只在我的模型上量过、被攻过零轮**。臂的代码在 `patch_arms.py`，环境变量开。

| 格 | 现行（K4-1 + C_ACCOUNT + F1_ANY + F2_EXACT） | `OPUS_C_NEWER_ROOTS`：C 并上环里比所选那一版新、按实例表有效、读得出的根自己的账 | `OPUS_F1_AFTER_F2`：F2 退了之后按退到的那一版再判 F1 | 加 `OPUS_C_MAPPING`：C 的账读不出时照被抛弃根那一份（中央映射现算，仍不成拒） | `OPUS_F2_NEWEST`：退到 [见证到的那一版, 所选根) 里最新、不被所选根实例表抛弃的根 |
|---|---|---|---|---|---|
| H1 ow，根读得出、记录藏、元数据单元藏（9 格 / k） | 中 9（量过） | 中 4：u4、u5、u6、u9（账在路上）（量过，`cbd2-ow-k5-newer`：`TALLY kind=ow vis=root M1=Ok r_units=overwritten cells=4`） | 碰不到（推的） | 与前两臂合用 0（量过，`cbd2-ow-k5-all3`：`TALLY kind=ow vis=root M1=Ok r_units=kept cells=22`） | 碰不到 |
| H1 sw3，三条记录全藏（10 格） | 中 10（量过） | 中 4：u7、u8、u9、u12（量过，`cb-sw3-k5-rootvisible-newer` 里 `rec_hidden=111` 那四格） | 碰不到（量过：仍中） | 合用 0（量过） | 碰不到 |
| H2 sw3，根读得出、缺第 1 / 2 条、账上节点藏（12 格） | 中 12（量过） | 碰不到 | 拒 12（量过；同一族 30 格全拒） | — | 碰不到 |
| H3 缺末条、账上节点藏（根藏 12 格，根读得出另 12 格） | 中（量过） | 根读得出那 12 格仍中（量过） | 仍中（量过：`OPUS_F1_AFTER_F2` 下 `100`、`101`、`110` 三行仍中） | 合用 0（量过：根藏那一半 `TALLY kind=sw3 vis=rec M1=Ok r_units=kept cells=59`） | 碰不到 |
| H4 R 的单元 + P 那次轮换的较新槽（4 组） | 落 Pm（量过） | 碰不到 | 碰不到 | 碰不到 | 落 P（量过） |
| 入库用例（5 份、25 条：C 两份、F1、F2、W5） | 全过 | 全过（量过） | 全过（量过） | **红 7 条**（量过：C 那份 3、F2 那份 4，都是 `AccountOfAnAbandonedRoot` 拒——`ur` 那几形 R 的单元全读不出、映射也现算不成，按「仍不成拒」拒了设计里该做成的挂载） | F2、W5 两份 11 条全过（量过；另三份没在这一臂下跑） |

读法：H1 要「C 也看较新的可读根」，H2 要「F1 跟着 F2 换水位」，H3 与 H1 的剩下那几格要「账读不出时不是什么都不加」。第三样照被抛弃根那一份办会在 `ur` 那几形上误拒（上表末行），要收窄成「只在那次发布读得出的记录不齐（缺序号或缺末条）时才现算、才拒」——这一收窄是推的，没实现。H4 那一臂只碰 F2 的取根，与前面几样不相干。

## 八、没打中的形状

- **W1 字面那一形**（现行账里仍分配、被豁免的槽这次挂载里释放又过回收门槛、R 的根槽还没写掉时发出去）：读码没找到出口。回收出来的槽三条路都补隔离：进程内记根那一路按还没解开的 C 补（`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/allocator.rs:2103`：「self.isolate_reclaimed_slots_the_isolation_still_protects(」，保护集含 C：`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/allocator.rs:1105`：「protected.extend(self.placements_named_by_unapplied_journal_records());」）；抬 F 那一路回收之前按此刻的现行账重算（`/home/fy5090/.local/share/singlefs/work/m3-batch35-36-code-r1/tree/crates/singlefs-core/src/mount.rs:3778`：「&current.allocation_records,」），这次挂载里释放了的槽在现行账里是已释放、不豁免、补隔离；挂载那一刻的回收在影子账之前，隔离位独立于分配位。另外按回收门槛 max(F, 环里最旧有效根)，这次挂载释放的槽（释放代 > R 的 txg）要等环转到 R 的槽之后才过门槛，只有抬 F 走得到前面——那一路重算过。装置上没走到回收门槛与抬 F（深度 2 的用户动作转不过环），这一条是读码的，取样上限写在第九节。
- **W2「拼根读错一版」**：一次发布每条记录都带整次发布的新根段，按最小计数器那条拼与按别的拼一样（I-8.10 钉着）；同一 (实例, txg) 两次不同的发布只在失败重发时出现，而重发是原样重发。没造出读错一版的形。「读到被抛弃根之外的账」：管理员回退那一形里水位之上的整串旧时间线都进 C（K4-1 去掉候选豁免之后它们不再被豁免），它们本来就该护到离开根环，不算多隔离。
- **W3 只换判法（F1_ANY）那一半**：sw3、k = 5、根槽藏，藏法加上「某个单元只藏一份」（13 单元 × 2 份），记录取全部 8 种子集，深度 1（`out/cb-sw3-k5-roothidden-singlecopy-base.log`）：`TALLY kind=sw3 vis=rec M1=Ok r_units=overwritten cells=12`、`… M1=Refused:… cells=39` 与不加单份那一组相同——一份坏的 208 格里，R 看得见（`vis=rec`）的 182 格没有一格被拒、也没有一格写掉 R 的单元（C 护住；F1_ANY 放过的单元另一份读得出）；三条记录全藏的 26 格是 R 整个看不见的形（`vis=none`，那一行 40 格），不计。没打中「F1_ANY 放过一形单元其实读不出的」。
- **W4「该退不退」**：R 的单元 + P 的末条读不出 ⇒ 照报读不出——D23 已定项 14 的 F2 句明写「那条记录读不出…都不退」，T 判决认的 A、B 两形同一机制，不另报。
- **V1、V3、V4**：第五节逐支对过，没找到判得不同的取值（读码）。
- 用户动作：每格 {o, s, n} 长度 ≤ 2 的全部 13 个结点；ow 取 k ∈ {1, 3, 5, 7}，sw3 取 k = 5。

## 九、这条腿自己的限度

- 只用了一种装置（384 槽小盘、两块盘、第一个文件之后覆盖写 k 次再录一次 R）与两种 R（一条记录的覆盖写、三条记录的顺序写）；k 在 ow 上取 1、3、5、7，在 sw3 上只取 5。故障模型只有「这一次挂载读回全零」（入库装置那一种），没注设备报读错。
- 判据是「R 的根还在环里时 R 写出的单元被逐字节改掉」，加写行那次根落盘之前那一刻的池级 checker 与恢复；没调层 0 枚举（`enumerate_layer0` 一类）把 M1 与用户动作的每个崩溃状态展开——这个判据对持久写的子集是单调的（一份单元被改掉只能是某一次写落在它上面），全部写都落了时没改掉，任何子集都不会改掉；这是推的。没跑重型测试与 GPU，`closed_form_state_count` 的配额一个状态都没用。
- 用户动作只扫了 {o, s, n} 长度 ≤ 2（13 个结点）；没扫删除、抬 F、卸载，转不过根环，所以第八节 W1 字面那一形只有读码。
- 修法臂都只在副本上、只对这几格量过；`OPUS_C_MAPPING` 已知在入库用例上误拒 7 条，按第七节的收窄是推的。
- H4 里「R 存在即证明 P 被确认过」按 D16 已定项 7 的持久顺序推，没在装置上逐写核。
- V2 只有抽象模型，规则照 `walk.rs` 两处抄，没走真的 `check_pool_image`；GPU 一侧全是读码。
- 一次抽样：没打中的几格照规则不算「没打中」，要再抽一次。

## 十、要主 agent 跑的命令

这条腿的结论不依赖要主 agent 跑的东西。若采 `OPUS_F2_NEWEST`，GPU 恢复内核同改之后要上卡跑对拍文件（checker 档，子 agent 不跑），命令照第一节材料里主 agent 跑 V 合后那一次。

## 十一、没做什么

- 没判正推那几格（K1、K3），没替主 agent 采纳；副本上的数不算入库装置上的数。
- 没造 V2 的真盘面；没上卡；没跑层 0 枚举与 checker 档。
- 没覆盖：多于两块盘、1024 MiB 环那套几何、管理员回退那一路的 C。

## 附：引到的 D23 已定项 14 原文整行

下面四行从 `.claude/kb/decisions/23-journal的角色与格式.md` 现在那一份原样抄（第 390、392、396、421 行；正文里引的短句都出自这四行，背景材料里冻结的那一版同样有这几句）。

```
崩在回退那次发布的任何一点，恢复落到的与任何一次普通发布没成时相同；回退的根持久之后才向管理员确认（D16（发布语义） 已定项 7）。回退深度由候选集定：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。**被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账、只隔离其中**只被被抛弃根引用**的槽——查账的集合取窄读法：按主语「被抛弃时间线的根」读，仍被有效根引用的槽不在其内——「有效根」指回退候选集里的根（按实例表判仍然有效 ∧ txg ≥ F_生效），每次挂载与每次抬 F 按当时的候选集重算；被抛弃根的账（树表或分配记录树）读不出时不跳过：从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报，映射也现算不成就重读一次，仍不成在取号之前拒可写、只读照常（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定，被攻过零轮，判决 `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）；被抛弃根的账读不出时，正常卸载、会话写与再可写挂载都该做得成，拒只在没有凭据时留着：凭据是内存里的现行账有一条代大于那条根的 txg、罩住它树表槽（与映射根槽）的记录，只看账、不读那一槽的内容，有它就不拒，没有它照拒（C694（被抛弃的根的账读不出时正常卸载被拒） 的凭据放行，用户 2026-10-07 定「C 与 B 都做」，被攻过零轮）；抬 F 那一串（准入与卸载）不走这一道：被抛弃根照挂载时记在根环表里的落点算影子账，不从盘上重读它的账、读不出也不拒，这一拒只留在再可写挂载那一道（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-07 定，被攻过零轮）；宽读法（凡被环里任一可读根引用过的槽都隔离）会让抬 F 在第一版 24 槽里买不到任何东西（影子账；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 同一次定案（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）把事务按单元切分 ⇒ 崩溃原子性的单位是一个单元、不是一次写请求（仓里没有任何决策承诺过写请求级原子性，这是一次对外语义的收缩）。
**可写挂载时的预隔离**（C694（被抛弃的根的账读不出时正常卸载被拒） 的 C，用户 2026-10-07 定「C 与 B 都做」，被攻过零轮）：可写挂载读阶段、重建分配器之后，把水位之上、没被施加的全部 journal 记录点名的落点隔离，不论那条根是不是回退候选根，只除去现行账里仍分配着的槽（它们本来分配着；这次挂载里释放、回收时分配器照补隔离，D28（挂载期承诺量） 已定项 1 第九项）；还用那次发布任一条读得出的记录的新根段读它的账，账里的落点一并隔离（账读不出就不加，交「记录有缺口时拒可写」）（`research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「T1 C 的候选豁免」「T1 C 的盲区」两行，被攻过零轮）；只在这个进程写掉那条记录所属 txg 对应的根环槽、那次发布取完落点之后解开，不随 D16（发布语义） 已定项 1「根槽这一次读坏」那一行给读回全零的槽的进程内重读时机解开。它护的是挂载那一刻根槽读不出、系统配置又没见证到的那条根新分配的单元：影子账只查环里读得出的根的账，看不见这条根；系统配置见证过的发布由乙-配置续（「可写挂载读到的更新状态读不出」那一段）管。隔离的槽算进准入不等式第九项（D28（挂载期承诺量） 已定项 1）。**记录有缺口时拒可写**（用户 2026-10-07 定 F1「乙：缺记录就拒可写」，被攻过零轮）：水位之上一次没施加的发布，读得出的那几条记录本次发布内序号不连续或缺末条、而它们点名的单元里有一个两份副本都验不过（`research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 行，被攻过零轮），重读一次，仍这样就拒可写挂载。
**所选根走读失败、系统配置没见证它时退回见证到的那一版**（用户 2026-10-07 定 F2「乙：三处都做」，被攻过零轮）：恢复、可写挂载读阶段与只读挂载择根三处用同一套逻辑，只退到系统配置见证到的那一版本身：根环里与计数器 = c_见证 那条记录 (txg, 实例) 恰好相等、读得出的那条根；那条记录读不出、c_见证 = 0、那条根读不出都不退（`research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行，收严，被攻过零轮）。所选那一版算「见证过」：它那次发布带末条标志的记录读得出、计数器 ≤ c_见证；末条读不出时拿计数器 = c_见证 的那条记录比 (txg, 实例)，不比所选那一版旧就算见证过；两条都读不出时当见证过、不退。
- ⚠️ **退回见证到的那一版的代价**（用户 2026-10-07 定 F2 时知情接受）：已确认的版本在较新的系统配置槽与它的单元同时读回全 0 时，恢复、可写挂载与只读挂载悄悄读回上一版；可写挂载在上一版上取号之后，那一版被抛弃（推的，没量）（`research/prompts/m3-c694-r2-main-verification.md` 第二节「K3 恢复一侧补法」那一行、第三节第 7 条与第四节 F2）。
```

