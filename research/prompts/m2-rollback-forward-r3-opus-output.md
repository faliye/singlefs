# m2-rollback-forward-r3 云端攻方腿（Opus）报告

轮名 m2-rollback-forward-r3（设计轮第三轮，最后一轮）；攻击面：正文第一节 K1–K7，照第四节分工表「云端攻方」那一行——式子本身、K3 反过来多保留的那一边、K4 三条臂各自新开的失败面、K6 各改法丢掉的判别力、K7 绕过去的路；第二轮攻过的挂载时那一形、按字面求差、B1 各盘取小不再攻。写于 2026-09-25 UTC 16:0x–17:1x（JST 2026-09-26 01:0x–02:1x）。冻结副本 `/tmp/claude-1000/m2-rollback-forward-r3/tree/` 开工与交回前各按 `research/prompts/m2-rollback-forward-r3-snapshot/crates-sha256.txt` 核一次，`sha256sum -c` 全 OK；它与第二轮开工快照的清单逐行相同（`crates/` 两轮之间没改）。

## 〇、复跑命令与文件

```
bash research/prompts/m2-rollback-forward-r3-opus-model/rerun.sh /tmp/claude-1000/m2-rollback-forward-r3/tree <草稿目录> [fast|full]
```

它把冻结副本拷到草稿目录、拷根目录的 `Cargo.toml` 与 `Cargo.lock`，先打 `prototype-r2.patch`（第二轮攻方原型，与 `m2-rollback-forward-r2-opus-model/prototype.patch` 逐字节相同，sha256 同为 `fb99d587…`），再打 `prototype3.patch`，放进 `rbf3_attack.rs`，经 `run-with-memory-cap.sh 16G` + `capped.sh 12` 跑 `cargo test --release -p singlefs-harness --test rbf3_attack`，逐条 `--exact`。`fast` 跑这一轮的 10 条用例（约 5 分钟，编译另计），`full` 另跑 K1 式子那一臂的用户动作扫描（单跑 566 秒）。产出只留 `RBF2` / `RBF3` 行与 `test result` 行。

- `rerun-fast.out`：第二份从冻结副本经 `rerun.sh` 重建的副本上跑出来的；与草稿副本上逐条跑的输出比，除挂钟与「filtered out」计数外逐行相同（316 行）。
- `rerun-sweep-formula.out`：草稿副本上跑的（`RBF2_SWEEP_ARM=formula RBF2_SWEEP_LEN=3`）；草稿副本的 `crates/` 与复跑副本逐文件比，只多一份第二轮的 `rbf2_attack.rs`（这一轮的用例文件由它扩出来，不参与编译这一条）。**全部是副本上的数，不是入库装置上的数；主 agent 要引，按规则在入库装置上重做。**

`SHA256SUMS`（原样）：

```
fb99d58713a97864d00056493d28b342e883cf95b53a49a0dd127859b0380755  prototype-r2.patch
f9dd70c210f039d9e0725edb97d14d42e53a65c829dabdc60f9109f7ec18f369  prototype3.patch
74f4b5e905063b3ce6d10969fbe1b9bfbe0316068164306c109dc1d4543774e6  rbf3_attack.rs
4cff261d78c431abe508bf8f9e97f9f6e9a76b695ac0ae96c2852918860f6645  rerun.sh
c54dd062dd96f65b1eaefa1c4aa6d2fa47f1ba4683a53b4a991dd93a2754d394  rerun-fast.out
d5d29c2f4fd82e568c507ecc10844f60ebcd9b21cbfa2e6302088ae2440a8448  rerun-sweep-formula.out
```

## 各格判定一览

「量过」= 这条腿在冻结副本拷贝上的原型里跑出来的（原样行在各节与模型目录的 `.out`）；「推的」= 按代码推、没实现没跑。**全部是副本上的数，不是入库装置上的数。**「原型」指第一节那份最小规格（被攻过零轮）。共用问句「那一版引用的块有没有被复用、读不读得对」每格在各节答。

| 编号 | 格 | 攻什么 | 判定 | 量过 / 推的 |
|---|---|---|---|---|
| Y1 | K1 | 式子照字面的 inode 号水位（只取环里读得出的根） | **打中，一次瞬时读错，分辨读法**：现行那一条根在回退那一刻读不出，号 2、3、4 被重发，checker 红 I-9.6；另取现行内存水位的那一读 0 | 量过 |
| Y2 | K1 × K4 | 式子里没有「同槽同分配代」核 | **打中，分辨 C419 臂、不分辨 K1 读法**：今天的规则下第二轮 X2 那一形 15 格全接受、12 格中、5 格新根读不出；Max、MaxHold、SysPre、SysRot 各 0 / 15 | 量过 |
| Y3 | K1 | 式子没定义的两种情形（现行账里没有记录；仍分配而分配代不同） | 只在 Y4 那几格走到，新根照指过去，checker 红 I-3.1、I-3.11、I-4.2；开着 K2 那一判时一次都没走到 | 量过 |
| — | K1 | 式子那一臂放开用户动作；释放「全部落点」比「只在用户可见单元上」多出的孤儿 | 没打中：2400 段、5316 次回退，六个计数 0，孤儿 0 | 量过 |
| Y4 | K2 | 关掉「按现行那一版的实例表判仍然有效」 | **打中，两次瞬时读错，分辨臂**：根 7、8 与记录 7 在重开那一刻读不出，之后读得出，挂着回退进去 16 / 16 接受、8 格新根读不出、16 格 checker 红；开着 16 / 16 拒 | 量过 |
| — | K2 | 开着这一判还有没有别的路 | 没打中：中间实例行罩住「实例的根全读不出」、记录读得出时前缀施加到头、根与记录都读不出时被抛弃的根被新实例的根盖掉 | 量过（「只有这三种来源」是推的） |
| — | K3 | 第二轮 X7 改完之后 | 没打中：5 种来回模式全回到 4 个不同状态；「写—撤回」交替不去重 3 个、去重 4 个 | 量过 |
| — | K3 | 反过来多保留、F 抬不上去 | 没打中：F 停住只到第 4 个状态的根被环转掉（两态 18 步、三态 16 步），候选坏 0 | 量过 |
| Y5 | K3 | 两态来回超过一圈根环 | **打中定案字面，不分辨去重 / 不去重**：去重在第 19 步也掉到 2 个（环里 24 条根全是 A、D），不去重第 2 步就掉 | 量过 |
| — | K4 | 第二轮 X2 / X3 / X4 在五臂上 | X2：只今天的规则中。X3（两次崩溃枚举）：Max 65 格、SysRot 3 格、MaxHold 0、SysPre 0。X4：Max、MaxHold 坏 6..8 根槽就中，SYSCFG 另要坏 4 份系统配置 | 量过 |
| Y6 | K4 | SYSCFG 只随系统配置轮换带 F | **打中，分辨臂**：抬 F 那一串第一条根落盘、轮换还没落时崩，第二轮 X3 那一形照样中（模型 3 个前缀）；抬 F 之前先写一次的 SysPre 0 | 量过（真实窗口 1 个前缀是推的） |
| Y7 | K4 | SYSCFG 下 checker 用哪个 F | **打中，分辨臂**：带 F 的根全坏后系统 F = 7、候选坏 0、恢复读对，checker 按根上的 F 判红 I-2.1、I-3.1、I-3.10、I-4.8、I-7.4（6 / 6 格） | 量过 |
| — | K4 | MaxHold 新开的失败面 | 没找到：690 + 434 格里只有「带 F 的根全坏」中 | 量过 |
| — | K4 | SYSCFG 的代价 | 每份 8 字节；SysRot 不多写调用（随每次轮换照抄）、SysPre 每次抬 F 多每盘一写 + 一道屏障；checker 要跟着读 | 模型写数量过，真实字节推的 |
| — | K5 | B1 × 五臂、每个崩溃点、之后最新 j 条根坏 | 恢复失败 0、挂载失败 0（3594 格）；SYSCFG 两读三流 960 点 0（I-7.9 除外） | 量过 |
| Y8 | K6 | I-7.9 改法 (a) | **打中判别力**：准入抬过头到现行 txg（与卸载逐字节相同）与抬到上限 + 1（字节不同）两格 (a) 都放过，今天都红 | 量过 |
| Y9 | K6 | I-7.9 改法 (b)、(c) | (b) 信记号：带卸载记号的准入抬过头放过；动根记录 flags 一位（改「非 0 拒收」的口径，动格式）。(c) 池级 checker 在抬过头与抬过现行 txg 上都不判 | 量过（记号 / 入口是旁路信息） |
| Y10 | K7 | (ii) 见证非空只拒可写 | **打中，分辨臂**：只读恢复在 13 个改坏深度里 4 个落到被管理员回退掉的 C | 量过 |
| Y11 | K7 | (i) 「改格式标记」按版本号读 | **打中，判据字面两读**：今天的读者不核系统配置的版本号，旧镜像版本号改 2 照挂、删见证代码落 C；按 incompat 位读，旧读者挂不上新镜像（没有降级再升级的绕路） | 量过 |

## 一、原型的最小规格（这条腿自己定的，不是条款，被攻过零轮）

在冻结副本的拷贝上先整份打第二轮攻方原型（`prototype-r2.patch`，与 `m2-rollback-forward-r2-opus-model/prototype.patch` 逐字节相同，`sha256sum` 见开头），再打这一轮的 `prototype3.patch`（`crates/singlefs-core/src/` 下 `mount.rs`、`recovery.rs`、`transaction.rs`、`prototype_rbf2.rs` 四处）。开关全关时与第二轮原型同。

- **K1 式子那一臂**（`Rbf2Arm::Formula`）：照正文第一节 K1 那一行（`_m2-rollback-forward-r3-body.md:13`）的式子做：
  - 复活 = R_old 的用户可见落点里现行账记作「已释放」的，改回仍分配、分配代写 R_old 那条记录的；**没有核**（第二轮原型那道「同槽同分配代」核在式子里没有）。现行账里「没有记录」或「仍分配而分配代不同」的，式子不说做什么，原型什么都不做、计数（`revive_skipped`）；「已释放且已回收」的，式子要改回仍分配，原型只计数（`formula_revive_reclaimed`）、跳过。
  - 释放 = 现行账里仍分配、新根不引用的**全部**落点：第二轮原型只在用户可见单元上求差、固定点由发布路径自己换下，式子比它多出的是「现行账里仍分配、现行那一版自己也不引用」的孤儿，原型数它（`orphans_allocated_not_referenced`）并照式子一起释放。
  - inode 号水位 = 环里读得出的根的 max，**不另取现行那一版内存里的**（式子字面）。
  - 候选照 K2：在环里、txg ≥ max(盘上 F_生效, 现行 F)、按现行那一版的实例表仍然有效。另一臂 `FormulaWithoutTableValidityCheck` 关掉最后这一判。
- **K3 去重**（`PROTOTYPE_CEILING_DEDUP`）：`rollback_floor_ceiling` 里照今天的规则认出非空根之后，从新到旧再过一遍：一条只有在它的 (inode, extent) 根指针与每一条更新的、已计入的根都不同时才计入；上限的其余部分不动。
- **C419 五条臂**（`Arm4`）：`MinOfMax`（今天）、`Max`、`MaxHold`（第二轮原型的扣住）、`SysPre`、`SysRot`。SYSCFG 两读都建在取最大值上（没有 HOLD），F_生效 = max(max_d F_d, 系统配置里读得出的 F 的最大值)：
  - **模型的载体**：不动真的系统配置槽（改它是格式改动），F 的另一份住每块盘末尾两块，各放 [magic 4 | seq 8 | F 8 | crc 4]，按 seq 轮换写较旧的一份，读取取读得出的份里 F 的最大值。
  - `SysRot`：每次发布在系统配置轮换（根槽 FUA 之后，`crates/singlefs-core/src/transaction.rs:899` 那一段的顺序）紧接着把 max(这条根的 F, 系统配置里已有的最大值) 写进每块盘一份，不另发屏障。真实实现里这 8 字节就在轮换写的那一槽里，模型多出一次写调用。
  - `SysPre`：同 `SysRot`，另在抬 F 那一串预演过了、第一次发布之前，把新 F 写进每块盘一份并发一道屏障。
- 其余同第二轮原型：两块 4 GiB 内存稀疏盘，根环三区归属盘 0 / 1 / 0、每区 8 槽，起点池只建一次（mkfs、暖机 1、2，A = (1, 3)），每段历史在内存里拷。

**这份规格的已知毛病**：`Formula` 对式子没定义的两种情形（没有记录、分配代不同）选了「什么都不做」；SYSCFG 的载体是模拟的，崩溃窗口比真实实现宽一次写（见 Y6）；I-7.9 的 (b)(c) 两条改法的记号与入口是装置给的旁路信息，没改盘上字节。

## 二、K1　挂着时回退的释放与复活（式子本身）

### Y1　式子照字面的 inode 号水位：现行那一条根在回退那一刻读不出，号被重发（打中，一次瞬时读错，分辨读法）

**历史**（`k1_watermark_literal_when_the_current_root_is_unreadable`）：

1. 起点池 A = (1, 3)；重开 → 实例 2；覆盖写 B；建 5 个 inode → X（txg 7，水位 7）。现行那一版 = X。
2. 故障：盘上 X 那条根读不出（翻一个字节；内存里的现行那一版照旧）。
3. 挂着回退到 B（txg 8）。
4. 建 3 个 inode（txg 9）。
5. 故障撤掉（同一字节再翻一次），X 又读得出。

每一步许可它的一句：第 3 步的水位是正文 K1 那一行「inode 号水位与树 ID 水位取环里全部读得出的根的 max」；X 在第 5 步是候选，按 `16-发布语义.md:38` 整行 `| 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |`。

**结果**（副本，原样）：

```
RBF3 k1_watermark shape=inodes-are-newest arm=Formula X=7 watermark_X=7 rollback=Ok((8, 2)) watermark_after_rollback=2 create3=Ok(9) conflicts_after_restore=["inode 2 births {7, 9}", "inode 3 births {7, 9}", "inode 4 births {7, 9}"] cand_bad=[] checker=["I-9.6:记账里的 inode 号水位 5 不大于 inode 树内最大 key 6"]
RBF3 k1_watermark shape=inodes-are-newest arm=Full X=7 watermark_X=7 rollback=Ok((8, 2)) watermark_after_rollback=7 create3=Ok(9) conflicts_after_restore=[] cand_bad=[] checker=[]
```

「X 之后又覆盖写一次、只坏 X」那一格两臂都不中（前一条根也带水位 7）。

**共用问句**：块没有被复用，每条候选根读得对（`cand_bad=[]`）；打中的是 inode 号：候选 X 与新根下同号 2、3、4 是不同对象（出生代 7 与 9）。

**四句**：

1. 分辨臂：分辨水位的两种读法——「只取环里读得出的根」（式子字面）与「另取现行那一版内存里的」（第二轮原型）。同一段历史、同一个故障，只差这一处。
2. 系统看得到：现行那一版在内存里，它的水位就在记账行里。
3. 满足的是 I-9.6 那一行的判据（`invariants.md:276`，checker 红的就是它），与第一轮判决 F1 第 3 条（跟着 R_old 退回去号会被重发）要挡的那一形；满足的是 K1 式子水位那一分句。
4. 改法：式子改成「max(现行那一版的水位, 环里读得出的根)」——`Full` 臂量过这一格不中。只在我的模型上量过、被攻过零轮。树 ID 水位同一句话，第一版八棵树的号恒定，没造得出差别。

### Y2　式子里没有核：复活与释放读不读得对，全押在候选集上（打中，分辨 C419 的臂，不分辨 K1 的读法）

第二轮原型那道「同槽同分配代」核在 K1 式子里没有。式子对「R_old 引用的落点在现行账里仍分配、分配代不同」（被复用了）什么都不说，照做就是新根指着别人的单元。它走得到的前提是 F 回落，由 K4 那条臂定。

**历史**（`k4_x2_cells_under_five_arms`，第二轮 X2 那一形，回退换成 K1 式子）：实例 1 A(3) B(4) C(5) → B1 卸载（6 盘 0、7 盘 1）→ 重开写 0..4 次、关掉 → 盘 1 上带 F 的根全坏 → 重开、挂着回退到 A / B / C。写几次、回退到哪放开扫，15 格一条臂。

| 臂 | 格 | 接受 | 打中 | 其中新根读不出 |
|---|---|---|---|---|
| MinOfMax | 15 | 15 | 12 | 5 |
| Max / MaxHold / SysPre / SysRot | 15 | 5（全是回退到 C） | 0 | 0 |

原样行（节选一行，`try_rollback_on_copy` 的判词装置自己截在 400 字）：

```
RBF3 k4_x2 arm=MinOfMax writes=3 target=(1, 3) chain=[6, 7] killed_dev1=[7, 10, 13] F_after=0 syscfg=[] rollback=accepted F=0 read_ok=Err("MappingStillUnreadable { slot: SlotNumber(50180) }") cand_bad=["(0,0) 走读失败 UnitUnreadable { slot: SlotNumber(50178) }", "(1,1) 走读失败 UnitUnreadable { slot: SlotNumber(50178) }", "(1,2) 走读失败 UnitUnreadable { slot: SlotNumber(50178) }", "(1,3) 走读失败 MappingStillUnreadable { slot: SlotNumber(50180) }", "(1,4) 走读失败 MappingStillUnreadable { slot: SlotNumber(50182) }", "(3,17) 走读 hit=true
```

第二轮 `Full` 臂（有核）同一形 15 格里核拒 2 格；式子没有核，那 2 格也接受（15 / 15）。

**共用问句**：MinOfMax 下回退到的那一版（A、B）引用的块被复用、5 格新根读不出（量过）；其余四臂 0 格。

**四句**：1. 不分辨 K1 的读法（有核的第二轮原型同样中，只少 2 格），分辨 C419 的臂：它是第二轮 X2 的延续。2. 核那一半系统看不到：已释放的记录记释放代（`crates/singlefs-core/src/allocator.rs:38` 整行 `    /// 仍分配时是分配代；已释放时是释放代。`）；生效规则那一半看得到。3. 字面：I-7.4（`invariants.md:54`）与共用问句。4. 改法：C419 取 Max 以下四臂量过 15 格全不中；给式子补「复活之前按 R_old 的指针读单元、核校验和」推的、没实现（第二轮已提，补的是看不到的那一半）。

### Y3　式子没定义的两种情形只在 K2 关掉那一判时走到（记账红，量过）

「现行账里没有这条记录」与「仍分配而分配代不同」：扫描、X2、K4 各批里只在 Y4 那几格出现（`skipped=3..7`），新根照指过去，checker 红 I-3.1、I-3.11、I-4.2（Y4 的原样行）。K2 那一判开着时一次都没走到。

### K1 释放集合：式子多释放的孤儿

见第九节扫描那一行的 `orphans_sum`（现行账里仍分配、现行那一版不引用的落点，每次回退数一遍）。

## 三、K2　挂着时回退的候选（按现行那一版的实例表判仍然有效）

### Y4　关掉这一判：落到被崩溃恢复抛弃的时间线上（打中，两次瞬时读错，分辨臂）

**历史**（`k2_candidates_on_the_abandoned_timeline`，形 `newest-two-roots-and-the-older-record`）：

1. 实例 2 覆盖写 B、C、D（6、7、8）；关掉。
2. 故障（瞬时）：重开那一刻根 7、8 读不出，txg 7 那条 journal 记录也读不出（记录 8 读得出，所以新实例的第一个 txg 越过 8，根 7、8 的槽不被盖）。
3. 重开 → 实例 3 从根 6 施加前缀停在 6（记录 7 断了），写行 (2, 6)，按它根 7、8 被抛弃；影子账读不到它们的账（第一轮 H12 那一形）。实例 3 写 0..3 次。
4. 故障撤掉：根 7、8 又读得出（记录 7 的槽已被实例 3 接着的计数器写过，不复原）。
5. 挂着回退（K1 式子）到 (2, 7) 或 (2, 8)，开 / 关这一判。

许可：第 5 步候选按正文 K2 那一行（`_m2-rollback-forward-r3-body.md:14`），关掉的那一臂只剩「在环里 ∧ txg ≥ F_生效 ∧ 带文件」。

| 形 | 臂 | 格 | 接受 | 新根读不出 | checker 红 |
|---|---|---|---|---|---|
| 根 7、8 + 记录 7 读不出（覆盖写 ×3） | 开着 | 8 | 0（全是 `OnAbandonedTimeline`） | 0 | 0 |
| 同上 | 关掉 | 8 | 8 | 5 | 8 |
| 覆盖写、顺序写 3 单元、覆盖写，根 7、8 + 记录 7 读不出 | 开着 | 8 | 0 | 0 | 0 |
| 同上 | 关掉 | 8 | 8 | 3 | 8 |

原样行（写 1 次、回退到 (2,7)，两臂）：

```
RBF3 k2 shape=newest-two-roots-and-the-older-record writes=1 hidden=[8, 7] landed=(3, 10) rows=Ok([(1, 3), (2, 6)]) target=(2,7) arm=FormulaWithoutTableValidityCheck accepted txg=12 revived=0 skipped=4 frr=0 read_ok=Err("MappingStillUnreadable { slot: SlotNumber(50184) }") checker=["I-2.1", "I-3.1", "I-3.11", "I-4.2", "I-4.8", "I-7.2", "I-7.4"]
RBF3 k2 shape=newest-two-roots-and-the-older-record writes=1 hidden=[8, 7] landed=(3, 10) rows=Ok([(1, 3), (2, 6)]) target=(2,7) arm=Formula refused OnAbandonedTimeline
```

「读得对」的那几格（关掉那一臂 16 格里 8 格 `read_ok=Ok(true)`）照样红 I-3.1、I-3.11、I-4.2：新根指着现行账里没有记录的落点（`skipped=4`，Y3），下一次分配就会把它们发出去。

**共用问句**：关掉那一判，回退到的被抛弃版本引用的块已被实例 3 复用（写行、暖机、之后的写都能落进去），16 格里 8 格新根读不出；开着 0 格。

**四句**：1. 分辨臂：同一段历史只差这一判。2. 系统看得到：现行那一版的实例表就在内存里，行 (2, 6) 写着。3. 字面：I-7.4（候选根引用的块被重新分配）、共用问句；故障是两次瞬时读错（根、记录），不需要崩溃。4. 改法：这一判本身就是改法，量过 16 格全拒。

### K2 开着这一判：别的路（没打中）

| 形 | 格 | 结果 |
|---|---|---|
| 实例 2 的根全部读不出、重开 | 20（每臂） | 实例 3 落在 (1, 3)，写行 (1, 3) 与中间实例行 (2, 0)（`crates/singlefs-core/src/mount.rs:1736` 那一段文档：`/// 中间实例 (i, 0, 0)；实例 0（mkfs）不写行（D18（块里携带什么信息） 已定项 11）。纯算，取号之前就列得出来。`），实例 2 的根全被抛弃：开着 20 / 20 拒；关掉那一臂也 20 / 20 拒，拒它的是 R_old 重建失败（`UnitUnreadable { slot: SlotNumber(50304) }`，写行那次就复用了），不是这一判 |
| 根 7、8 读不出、记录都读得出 | 8（每臂） | 前缀施加到 8，写行 (2, 8)，不抛弃任何根；两臂 16 格全接受、读对、checker 绿 |
| 根与记录都读不出（覆盖写 ×3；或顺序写那一版单独） | 12（每臂） | 新实例的第一个 txg 落回 7，被抛弃的根的槽被新实例的根盖掉，回退报 `NotInRing`（这两形的「撤掉故障」翻的是实例 3 的根，那之后的镜像只用来读这一个判词） |

没找到第三条路：候选的有效判据只看现行那一版的行，被抛弃的来源只有「前缀施加停在所选根之前」一种（推的，按 `mount.rs` 写行的代码读），而那一刻写下的行（含中间实例行）罩住了这几形（量过）。第一轮 H12（恢复直接落在被抛弃时间线上）与第二轮 X8（旧镜像）是恢复的落点，不是这一判的候选，这一轮没重攻。

## 四、K3　「4 个可退到的不同状态」的去重

### 第二轮 X7 改完之后（没打中：去重把几种来回回退都拉回 4 个）

`k3_dedup_patterns_and_long_cycles`（取最大值规则；实例 2 覆盖写 B C D，按模式挂着回退（K1 式子），再按今天的上限抬 F，数候选里按内容不同的状态）：

| 回退模式 | 不去重：上限 / 不同状态 | 去重：上限 / 不同状态 |
|---|---|---|
| 不回退 | 3 / 4 | 3 / 4 |
| A-D | 7 / 3 | 6 / 4 |
| A-D-A | 8 / 2 | 6 / 4 |
| A-D-A-D | 9 / 2 | 6 / 4 |
| A-D-A-D-A-D | 11 / 2 | 6 / 4 |

原样行（节选两行）：

```
RBF3 k3_pattern dedup=false pattern=A-D-A ceiling=Ok(8) raise=Ok(Ok(2)) F=8 candidate_roots_with_file=6 distinct_states=2 cand_bad=[]
RBF3 k3_pattern dedup=true pattern=A-D-A ceiling=Ok(6) raise=Ok(Ok(2)) F=6 candidate_roots_with_file=8 distinct_states=4 cand_bad=[]
```

### 反过来多保留的那一边（没找到抬不上去的历史；另有一格与去重无关的保证缺口，Y5）

长跑三种（每一步之后按上限抬 F；回退的目标取「候选里内容是那个状态的最新一条根」，用户要的是状态不是当初那条根）：两态 A/D 来回 30 步、三态 A/B/D 轮转 30 步、「写一个新状态、撤回到写之前」交替 30 步。

| 长跑 | 去重 | 候选坏的步数 | F 连着不动最多几步 | 候选里不同状态的最小值 |
|---|---|---|---|---|
| 两态 A/D | 不去重 | 0 | 0 | 2 |
| 两态 A/D | 去重 | 0 | 18 | 2 |
| 三态 A/B/D | 不去重 | 0 | 0 | 3 |
| 三态 A/B/D | 去重 | 0 | 16 | 3 |
| 写—撤回 | 不去重 | 0 | 0 | 3 |
| 写—撤回 | 去重 | 0 | 1 | 4 |

原样行（去重的三行，去掉末尾逐步轨迹）：

```
RBF3 k3_cycle dedup=true cycle=two-state-A-D refused=0 first_refusal=None cand_bad_steps=0 min_distinct=2 max_steps_F_unchanged=18 final_txg=62
RBF3 k3_cycle dedup=true cycle=three-state-A-B-D refused=0 first_refusal=None cand_bad_steps=0 min_distinct=3 max_steps_F_unchanged=16 final_txg=67
RBF3 k3_cycle dedup=true cycle=write-then-undo refused=0 first_refusal=None cand_bad_steps=0 min_distinct=4 max_steps_F_unchanged=1 final_txg=75
```

- **F 停住是保留，不是卡死**：去重下两态来回时上限钉在第 4 个不同状态（B）最新那条根上，F 停 18 步；B、C 的根被根环转掉之后「非空有效根不足 4 个时取最旧的有效根」（`16-发布语义.md:36`）接手，F 从下一步起每步跟着抬。候选坏 0。停住的那 18 步里被扣着的只有回退换下又会被下一次回退复活的块，与每次发布换下的固定点（推的）；后者本来就按「环里最旧有效根」回收（`16-发布语义.md:33`），与 F 无关。没造出「F 永远抬不上去」的历史：只要环在转，上限就跟着最旧有效根走。
- **写—撤回**是第二轮 X7 的又一形（不回退到老状态，只撤回上一次写）：不去重只剩 3 个，去重 4 个。

### Y5　两态来回超过一圈根环，「4 个不同状态」在去重下也保不住（打中定案字面，不分辨去重 / 不去重）

去重那一臂在第 19 步掉到 2 个。那一刻环里 24 条根全是 A、D 两个状态（原样）：

```
RBF3 k3_drop dedup=true cycle=two-state-A-D step=19 distinct=2 F=7 ring(txg:F:state)=["9:0:s0", "10:6:s0", "11:6:s0", "12:6:s1", "13:6:s0", "14:6:s1", "15:6:s0", "16:6:s1", "17:6:s0", "18:6:s1", "19:6:s0", "20:6:s1", "21:6:s0", "22:6:s1", "23:6:s0", "24:6:s1", "25:6:s0", "26:6:s1", "27:6:s0", "28:6:s1", "29:6:s0", "30:6:s1", "31:7:s1", "32:7:s1"]
```

B、C 不是被 F 抛下的，是它们的根槽被环转掉了；三态轮转同样在第 18 步从 4 掉到 3。

**共用问句**：没有块被复用、候选全读得对；打中的是 D16（发布语义） 已定项 1 定案的第一句（`16-发布语义.md:29`，转述：回退候选集保留最近 4 个可退到的不同状态）。

**四句**：1. 不分辨 K3 的两种数法：两边都掉到 2，去重只把掉的那一刻从第 2 步推到第 19 步。2. 系统看得到（环里全是 A、D）。3. 字面：定案那一句，不是 I-7.4。4. 改法：F 与去重都碰不到它，病根是一个状态只住在根槽里、根槽随发布轮换；要保就得让「第 4 个状态」的根不被盖（钉槽或另存），推的、没实现、动根环。条款也可以把「4 个」写成「根环容量之内」，那是改保证，不是修。

## 五、K4　C419 的三条臂（MAX、MAX+HOLD、SYSCFG 两读）

### 两次崩溃逐点枚举（K4 与 K5 同一张表）

`k4_k5_chain_crash_then_mount_crash`：实例 2 覆盖写到下一个 txg 落盘 0（现行 7），B1 卸载那一串 8（盘 0）、9（盘 0）、10（盘 1）。崩溃一：这一串写的每一个前缀 p1（0..全串）。重开：录下那次挂载的写，崩溃二：挂载写流里每一次根槽写之前、以及挂完（p2）。之后的故障三种：无、盘 0 上带 F 的根全坏、带 F 的根全坏。每格判恢复落点与内容、候选根逐条走读（共用问句），候选有坏的另照 K1 式子挂着回退到每一条坏候选。

| 臂 | 格 | 盘 0 带 F 的根坏：候选坏的格 | 带 F 的根全坏：候选坏的格 | 候选坏出现在哪些 p1 |
|---|---|---|---|---|
| MinOfMax | 690 | 12 | 12 | 65..67（这一串落满两块盘之后：第二轮 X2 那一类） |
| Max | 690 | 65 | 140 | 盘 0 坏：23..64（第一条根落盘之后、盘 1 那条根落盘之前）；全坏：23..67 |
| MaxHold | 690 | 0 | 12 | 全坏：65..67 |
| SysPre | 774 | 0 | 0 | — |
| SysRot | 750 | 3 | 3 | 23..25 |

各臂根槽写在这一串写流里的位置（原样）：MinOfMax / Max / MaxHold `["22:RootRecordFua@0", "43:RootRecordFua@0", "64:RootRecordFua@1"]`；SysPre `["24:RootRecordFua@0", "47:RootRecordFua@0", "70:RootRecordFua@1"]`；SysRot `["22:RootRecordFua@0", "45:RootRecordFua@0", "68:RootRecordFua@1"]`。

原样行（每臂汇总，去掉末尾 p1 列表）：

```
RBF3 k4k5 arm=MinOfMax newest_before_chain=7 chain=[8, 9, 10] chain_writes=67 summary={"cells": 690, "kill_all_carriers:cand_bad": 12, "kill_dev0_carriers:cand_bad": 12}
RBF3 k4k5 arm=Max newest_before_chain=7 chain=[8, 9, 10] chain_writes=67 summary={"cells": 690, "kill_all_carriers:cand_bad": 140, "kill_dev0_carriers:cand_bad": 65}
RBF3 k4k5 arm=MaxHold newest_before_chain=7 chain=[8, 9, 10] chain_writes=67 summary={"cells": 690, "kill_all_carriers:cand_bad": 12}
RBF3 k4k5 arm=SysPre newest_before_chain=7 chain=[8, 9, 10] chain_writes=75 summary={"cells": 774}
RBF3 k4k5 arm=SysRot newest_before_chain=7 chain=[8, 9, 10] chain_writes=73 summary={"cells": 750, "kill_all_carriers:cand_bad": 3, "kill_dev0_carriers:cand_bad": 3}
```

照 K1 式子挂着回退到坏候选：五臂一次都没被接受，全在重建 R_old 时读不出被拒（`Mount(Recovery(UnitUnreadable …))`）。这一形里先被复用的是 R_old 的固定点；只坏数据单元、固定点完好的格在 Y2（MinOfMax）里有，回退被接受而新根读不出。

**用户动作放开扫**（`k4_user_steps_opened`，只固定前缀与故障）：这一串崩在 7 个代表前缀（第一条根之前、之后、两条根之间、第二条根之后、盘 1 那条根之前、之后、整串完），重开（挂载做完），之后用户动作长度 0..2（覆盖写、建 3 个 inode、回退到上一个、回退到最旧、关掉重开，31 种），关掉，再坏盘 0 带 F 的根 / 全部带 F 的根：

| 臂 | 盘 0 带 F 的根坏：候选坏 / 格 | 带 F 的根全坏：候选坏 / 格 |
|---|---|---|
| MinOfMax | 62 / 217 | 62 / 217 |
| Max | 0 / 217 | 186 / 217 |
| MaxHold | 0 / 217 | 134 / 217 |
| SysPre | 0 / 217 | 0 / 217 |
| SysRot | 0 / 217 | 0 / 217 |

挂载做完时 Max 的「盘 0 坏」0 格：写行与暖机的根已带 F 落满两块盘，X3 那一形要挂载自己崩在根写之前（上一张表）。

### Y6　SYSCFG 只随系统配置轮换带 F：第一条抬 F 根落盘到轮换写之间，第二轮 X3 那一形照样中（打中，分辨臂）

**历史**（上一张表 SysRot 那 3 个 p1）：

1. 实例 2 覆盖写到现行 7；B1 卸载，这一串第一次发布 8 的根槽 FUA 落盘（盘 0，带 F = 7）。
2. 崩溃一：系统配置轮换那两次写与 F 那两份之前（p1 = 23..25）。盘上只有根 8 带 F = 7，系统配置里的 F 仍是 0。
3. 重开：取最大值 ⇒ F_生效 = 7，按 7 回收；写行那次发布的单元落进刚回收的槽。
4. 崩溃二：写行那次的单元落了、根没落（p2 = 26）。
5. 故障：盘 0 上根 8 坏。F_生效 = max(0, 系统配置 0) = 0，候选回到 (1,3)、(2,4)、(2,5)。

原样行：

```
RBF3 k4k5-hit arm=SysRot p1=23/73 p2=26/54 fault=kill_dev0_carriers killed=[8] F_after=0 syscfg=[(0, 0, 3, 0), (0, 1, 4, 0), (1, 0, 3, 0), (1, 1, 4, 0)] recover=Ok((2, 8)) cand_bad=["(1,3) 走读失败 UnitUnreadable { slot: SlotNumber(50251) }", "(2,4) 走读失败 MappingStillUnreadable { slot: SlotNumber(50240) }", "(2,5) 走读失败 MappingStillUnreadable { slot: SlotNumber(50240) }"] accepted_rollbacks_into_bad=[]
```

同一格 Max 臂的原样行（窗口宽得多，p1 = 23..64）：

```
RBF3 k4k5-hit arm=Max p1=23/67 p2=26/50 fault=kill_dev0_carriers killed=[8] F_after=0 syscfg=[] recover=Ok((2, 8)) cand_bad=["(1,3) 走读失败 UnitUnreadable { slot: SlotNumber(50251) }", "(2,4) 走读失败 MappingStillUnreadable { slot: SlotNumber(50240) }", "(2,5) 走读失败 MappingStillUnreadable { slot: SlotNumber(50240) }"] accepted_rollbacks_into_bad=[]
```

**共用问句**：三条候选根引用的块被写行那次复用，读不出（量过）；恢复落 (2, 8) 读对。

**四句**：

1. 分辨臂：同一段历史 SysPre 0、MaxHold 0、SysRot 3、Max 65。SysRot 与 SysPre 只差「抬 F 之前先写一次系统配置、带屏障」。
2. 系统看得到：第 3 步盘 1 上没有带 F 的根、系统配置里 F 还是 0——这正是「F 还没落稳」的判别子。
3. 字面：I-7.4（`invariants.md:54`，候选根引用的块被重新分配）与共用问句。故障数同第二轮 X3：两次崩溃 + 盘 0 上一个根槽坏（这一格只要 1 个，第二轮那一格是 2 个，因为崩得更早）。
4. 改法：`SysPre` 在这张 774 格的表与用户动作扫里都量过不中；`MaxHold` 同样量过不中。模型把 F 写成轮换之后的单独一次写，窗口多了一次写：真实实现里 F 就在轮换写的那一槽里，窗口只剩「根 FUA 落了、轮换一次都没落」那一个前缀（推的，p1 = 23）。**只在我的模型上量过、被攻过零轮。**

### Y7　SYSCFG 让 checker 与系统分叉：系统读得对的镜像上 checker 红（打中，判据字面，分辨臂）

`k4_x4_carriers_syscfg_and_checker`：这一串完整落盘，重开写 0..2 次，关掉，带 F 的根全坏（6..8 个根槽）。SYSCFG 两读下系统配置里 F = 7 还在，F_生效 = 7、候选坏 0、恢复读对；checker 照今天按根上带的 F 判（最新那条读得出的根带 0），把 7 之下已被合法复用的根算进候选，红 I-7.4 等五条。原样行（SysRot 写 2 次那一格，系统配置不坏 / 再坏 4 份）：

```
RBF3 k4_x4 arm=SysRot chain=[8, 9, 10] chain_F=7 writes_after_reopen=2 fault=kill_all_carriers roots_killed=8 syscfg_killed=0 F_eff=7 newest_root_F=Some(0) syscfg_F=7 cand_bad=0 recover=Ok((2, 10)) checker=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-7.4"]
RBF3 k4_x4 arm=SysRot chain=[8, 9, 10] chain_F=7 writes_after_reopen=2 fault=kill_all_carriers+syscfg roots_killed=8 syscfg_killed=4 F_eff=0 newest_root_F=Some(0) syscfg_F=0 cand_bad=6 recover=Ok((2, 10)) checker=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-7.4"]
```

SysPre、SysRot 两读 × 写 0..2 共 6 格都是这样（系统 0 格坏、checker 红）；MaxHold 同一故障下 F 回落、候选坏 6，checker 红的是真的。

**四句**：1. 分辨臂：只在 SYSCFG 下出现（其余臂系统与 checker 同时回落）。2. checker 看得到：F 就在盘上系统配置里，是它没去读。3. 字面：I-7.4（`invariants.md:54`）的候选集照 `16-发布语义.md:38` 整行 `| 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |`——checker 用的不是这个 F_生效。4. 改法：checker（与层 0 的判定）跟着读系统配置里的 F，推的、没实现；这是 SYSCFG 的一笔代价，不改它，SYSCFG 下每一次「带 F 的根全坏」都会在 checker 上误红。

### 第二轮 X4（带 F 的根全坏）在各臂上

| 臂 | 要坏的根槽 | 另要坏的系统配置份数 | 候选坏（写 0 / 1 / 2 次） |
|---|---|---|---|
| MinOfMax | 盘 1 上带 F 的 2 个就够 | — | 3 / 6 / 6 |
| Max、MaxHold | 带 F 的全部 6 / 7 / 8 个 | — | 3 / 6 / 6 |
| SysPre、SysRot | 6 / 7 / 8 个 | 4（两块盘各两份都带 7） | 3 / 6 / 6 |

（`k4_x4` 那张的 45 行；本地攻方那一问算的是同一件事的最少数。）

### SYSCFG 的代价（量过 / 推的）

| 读法 | 这一串 3 次发布里写了几份 F | 其中另发、带屏障的 | 每份字节 | 哪几次发布要写 |
|---|---|---|---|---|
| SysRot | 6（每次发布每块盘一份） | 0 | 8（模型里一块 4 KiB） | 每一次发布的系统配置轮换都带着（F 不变也要照抄，不然两次轮换之后就被旧值盖掉） |
| SysPre | 8 | 2（抬 F 之前每块盘一份 + 一道屏障） | 8 | 同上，另加每次抬 F 前一次 |

量过的是模型的写调用数（`k4_cost` 那几行）。真实实现（推的）：系统配置字段表（D22（单元原子性怎么合成） 已定项 9）多 8 字节；SysRot 不多一次写调用，SysPre 每次抬 F（准入与卸载）多「每盘一次系统配置写 + 一道屏障」；另加 checker 读它（Y7）。

### K4 各臂新开的失败面（汇总）

| 臂 | 新开的 | 量过 / 推的 |
|---|---|---|
| Max | 第二轮 X3（窗口 = 第一条抬 F 根落盘到盘 1 那条根落盘，42 个前缀） | 量过 |
| MaxHold | 这一轮没找到新的：只剩 X4（带 F 的根全坏） | 量过（690 格 + 434 格） |
| SysRot | Y6（窗口 = 抬 F 根落盘到系统配置轮换）、Y7（checker 误红） | 量过 |
| SysPre | Y7（checker 误红）；另多一次带屏障的写 | 量过 |

## 六、K5　B1 与 K4 各臂配着崩

- **两次崩溃枚举**（第五节第一张表）就是 K5 那一问在五臂上的答案：这一串的每一点 × 重开那次挂载的每一次根写之前 × 三种故障。恢复失败 0 格、可写挂载失败 0 格（汇总里没有 `recover_bad`、`mount_err`）；候选坏的格见那张表。
- **SYSCFG 两读的逐点崩溃**（`k5_b1_crash_points_under_syscfg`，第二轮 `g1_g4_crash_points_prefix` 那三条 B1 流：这一串 + 重开 + 写一次，每点另改坏最新 1..3 条根）：960 个崩溃点，落点错 0、改坏之后错 0、候选坏 0，checker 只红 I-7.9（第二轮 X6 那一条）。原样行（节选一行）：

```
RBF2 prefix name=RBF3-k5 arm=SysRot b1-chain[8, 9, 10]-remount-write points=182 bad=0 first_bad=None probe_bad(kill1,kill2,kill3)=[0, 0, 0] probe_first=[None, None, None] audit_bad=0 audit_first=None checker_reds_by_invariant={"I-7.9": 159}
```

（今天的规则、Max、MaxHold 三臂的这三条流第二轮量过，1323 个点，这一轮没重跑。）

## 七、K6　I-7.9 的三条改法各丢了哪一格判别力

`k6_i79_variants`：同一个起点（实例 2 覆盖写 5 次，现行 10，今天的上限 7）走六种抬 F，逐张镜像判。(a) = 抬 F 的根带的 F ≤ 它之前最新那条根的 txg（抬 F 的根的认法照今天：同一实例前一条根带的 F 比它低）；(b) = 带记号的根按 (a)、不带的按今天；(c) = 只在录制流上判、入口已知（卸载按 (a)、准入按今天），池级 checker 不判。(b) 的记号与 (c) 的入口是装置给的旁路信息，没动盘上字节。

| 镜像 | 与 S1 逐字节相同 | 今天 | (a) | (b) | (c) 录制流 | (c) 池级 |
|---|---|---|---|---|---|---|
| S1 B1 卸载那一串（F = 10） | — | 红（该绿） | 绿 | 绿 | 绿 | 不判 |
| S2 准入抬到上限（F = 7） | 否 | 绿 | 绿 | 绿 | 绿 | 不判 |
| S3 准入抬过头到现行 txg（F = 10） | 是 | 红 | **绿（丢）** | 红 | 红 | **不判（丢）** |
| S4 准入抬过头到上限 + 1（F = 8） | 否 | 红 | **绿（丢）** | 红 | 红 | **不判（丢）** |
| S5 抬过现行 txg（缺陷形，F = 13） | 否 | 红 | 红 | 红 | 红 | **不判（丢）** |
| S6 准入抬过头、却带了卸载记号（缺陷形） | 是 | 红 | 绿 | **绿（丢）** | 红 | 不判 |

逐点崩溃（这一串的每个写前缀，64 点）：S1、S3 两串今天各红 45 点，(a) 各红 0 点。

原样行（节选三行）：

```
RBF3 k6 S3 准入抬过头到现行 txg entry=admission cur=10 ceiling=7 F=10 chain=[11, 12, 13] same_bytes_as_S1=true today_red_root=Some(11) a_red=[] b_red=true c_layer0_red=true c_pool=not_judged
RBF3 k6 S4 准入抬过头到上限+1 entry=admission cur=10 ceiling=7 F=8 chain=[11, 12, 13] same_bytes_as_S1=false today_red_root=Some(11) a_red=[] b_red=true c_layer0_red=true c_pool=not_judged
RBF3 k6 S6 准入抬过头、却带了卸载记号（缺陷形） entry=admission cur=10 ceiling=7 F=10 chain=[11, 12, 13] same_bytes_as_S1=true today_red_root=Some(11) a_red=[] b_red=false c_layer0_red=true c_pool=not_judged
```

### Y8　(a) 丢掉「准入抬过头」整类，不只是抬到现行 txg 那一格（打中，判别力，量过）

**四句**：1. 分辨 I-7.9 的改法，不分辨 B1 本身。2. S3 与 S1 逐字节相同，任何只看盘的判法都分不开（第二轮 X6）；S4 与 S1 不同字节，今天分得开，(a) 也放过了——(a) 只判「F 不越过已有的根」，D16（发布语义） 已定项 1 上限里「第 4 新的非空根」那一半（`16-发布语义.md:36`）它一个字都不判。3. 字面：I-7.9 那一行（`invariants.md:59`，转述：抬 F 的那一条根带的 F 不高于用它之前的根算出的 D16 已定项 1 上限）。4. 改法：(b)、(c) 录制流在 S3、S4 上都红（量过）；(a) 这两格不起作用。

### Y9　(b) 信记号、(c) 丢池级：各有一格丢（打中，判别力，量过）

- (b)：S6 带着卸载记号的准入抬过头，按 (a) 那一支放过。记号自己没有别的东西核它：盘上看不出「这一串真是卸载发的」。它比 (a) 窄——只在「准入入口误打记号」这一种缺陷上丢。
- (c)：池级 checker 在 S3、S4、S5 上都不判。录制流之外的镜像（真盘、修复工具、不经录制的坏镜像）上，抬过头、抬过现行 txg 都没人判。崩溃状态从录制流里枚举出来，入口已知，(c) 判得到（推的，装置里 (c) 的崩溃点没另跑）。

**(b) 动哪几字节**：根记录的 flags（4 字节，偏移 20）今天「第一版恒 0、非 0 拒收、位号表空」（`22-单元原子性怎么合成.md:136` 整行 `| flags | 4 | 预留：第一版恒 0、非 0 拒收、位号表空（D22（单元原子性怎么合成） 已定项 17） |`；代码 `crates/singlefs-core/src/root_record.rs:46` 整行 `        writer.put_u32(0); // flags：第一版恒 0、非 0 拒收`）。记号放那里一位：宽度一个字节不变，改的是「非 0 拒收」的解释口径，按 D15（格式冻结政策） 已定项 1 第 1 条（`15-格式冻结政策.md:27`，转述：改盘上字节或改已有字节的解释口径都算动格式）算动格式。旧读者见到带这一位的根按拒收处理，卸载那一串的根在旧读者眼里读不出（推的）。

**三条改法各修哪一格**：

| 改法 | S1 卸载误红 | S3 / S4 准入抬过头 | S5 抬过现行 txg | S6 记号打错 | 池级镜像 |
|---|---|---|---|---|---|
| 今天 | 红（误）（量过） | 红（量过） | 红（量过） | 红（量过） | 判 |
| (a) | 修（量过） | 丢（量过） | 红（量过） | 丢（量过） | 判 |
| (b) | 修（量过） | 红（量过） | 红（量过） | 丢（量过） | 判，动格式 |
| (c) | 修（量过，录制流） | 红（量过，录制流） | 红（量过，录制流） | 红（量过，录制流） | 不判（量过） |

## 八、K7　旧镜像

旧镜像同第二轮 X8（今天的代码造：实例 2 写 B(6)、C(7)，挂载时回退到 B → 实例 3，回退行 + 见证 1 条，写 D、E）。原样：`RBF3 k7 B=(2, 6) C=(2, 7) instance3_roots=[(3, 8), (3, 9), (3, 10), (3, 11), (3, 12)] witness_entries=1`。

### Y10　(ii)「见证表非空就拒可写挂载、只许只读」：只读那一路照样落到被管理员回退掉的 C（打中，分辨臂）

`k7_old_image_under_the_two_fixes`：删掉见证的代码 + (ii) 那道闸，逐条改坏更新的根，只读恢复落在哪：

| 改坏最新几条 | 见证非空 | (ii) 下可写挂载 | 只读恢复落在 |
|---|---|---|---|
| 0..4 | 是 | 拒 | (3, 12) |
| 5..8（实例 3 的根全坏） | 是 | 拒 | **(2, 7) = C** |
| 9..11 | 是 | 拒 | (1, 3) |
| 12 | 是 | 拒 | (0, 0) |

原样行（节选两行 + 汇总）：

```
RBF3 k7-ii killed_newest=4 witness_nonempty=true writable_under_ii=refused read_only_recover_landed=Some((3, 12)) judged=Ok((3, 12))
RBF3 k7-ii killed_newest=5 witness_nonempty=true writable_under_ii=refused read_only_recover_landed=Some((2, 7)) judged=Ok((2, 7))
RBF3 k7-ii-summary depths=13 read_only_landed_on_C=4
```

**共用问句**：C 那一版读得对（`judged=Ok((2, 7))`），块没被复用；打中的是「管理员的回退被静默撤销」：用户在只读挂载里看到的是他回退掉的那一版。

**四句**：1. 分辨臂：(i) 下这份镜像挂都挂不上（下一格），(ii) 下只读落到 C。2. 系统看得到：见证表就在系统配置里，(ii) 为了判「非空」本来就得读它。3. 字面：第二轮 X8 那一形（D23（journal 的角色与格式） 已定项 14「回退见证」要挡的被回退抛弃的根又被择中），只是换到只读那一路；`.claude/rules/fs-design.md:198` 的判据（整行 `> **一条格式分支值不值得，看它能不能把「用错了」变成「挂不上」。**`）要的是挂不上，(ii) 把可写那一半变成了挂不上、只读那一半仍是「用错了」。4. 改法：(ii) 改成「见证表非空就连只读也拒」，或只读择根保留见证那一判（留一个只读的见证读者，第二轮提过）——推的、没实现；(i) 见下一格。

### (i)「改格式标记」：两种读法一对一错（量过）

- **读成「格式版本号 1 → 2」**：今天的读者不看系统配置里的格式版本号——冻结副本里写它的是 `crates/singlefs-core/src/system_configuration.rs:315`（`            writer.put_u16(FORMAT_VERSION);`），`parse_slot`（同文件第 421 行起）三关是 magic、整槽校验和、incompat 位，没有版本号；全仓核版本号的只有单元头（`crates/singlefs-core/src/unit.rs:300`）。把旧镜像四个系统配置槽的版本号改成 2、重算校验和：

```
RBF3 k7-i-version code=today version_bytes=Some([2, 0]) writable_mount=Ok((4, 14)) instance3_killed_recover_landed=Some((2, 6))
RBF3 k7-i-version code=deleted version_bytes=Some([2, 0]) writable_mount=Ok((4, 14)) instance3_killed_recover_landed=Some((2, 7))
```

  两种代码都照挂；删见证的代码在实例 3 的根全坏时落到 C。只 bump 版本号、不加读侧的判，(i) 什么都没挡。**打中条款字面的一种读法**（Y11）：正文 K7 那一行（`_m2-rollback-forward-r3-body.md:19`）写「改格式标记，旧镜像挂载时按不认识的标记拒」，没说哪个标记、谁来判。
- **读成「换 incompat 布局身份位」**（新代码只认位 1、位 0 当不认识）：今天的 `incompat_bits_are_mountable`（`system_configuration.rs:513`）要求布局身份位 0 且没有不认识的位，旧镜像在新代码下按不认识的位拒（推的，没改新代码的判）。反方向量过：把旧镜像四槽的 incompat 字节改成只带位 1，今天的代码（旧读者）挂不上、只读恢复也读不出系统配置：

```
RBF3 k7-i-bit old_reader_writable_mount=Err("Recovery(NoValidSystemConfiguration { first_device_with_no_valid_system_configuration_slot: DeviceId") old_reader_recover=None
```

  所以没有「旧代码在新镜像上写出见证、再交给新代码」这条降级再升级的绕路。这一读法下旧镜像的代价是连只读都挂不上：`.claude/rules/format-evolution.md:7` 整行 `**在第一个外部用户出现之前，磁盘格式是软的**——随时可以拆了重做，不需要向后兼容。` 许可这个代价。

### Y11　(i) 按「版本号」读时一个旧镜像都拦不住（打中，判据字面两读）

**四句**：1. 分辨 (i) 的两种读法（版本号 / incompat 位），同一份镜像一个照挂、一个挂不上。2. 系统看得到：版本号就在槽里，只是没人判。3. 字面：正文 K7 (i) 的「格式标记」一词两读。4. 改法：条款写明「换 incompat 布局身份位（或加一个必需位），新读者不认旧位」，量过旧读者对新位挂不上；新读者对旧位拒是推的。

## 九、K1 式子那一臂的用户动作放开扫（没打中）

第二轮 `g1_sweep_user_steps` 原样借来，臂换成 `Formula`（`RBF2_SWEEP_ARM=formula RBF2_SWEEP_LEN=3`）：前缀 2 种 × 第一次回退目标 3 种 × 后缀 400 种（7 种动作长度 0..3）= 2400 段历史。每段末尾环里每条根走读 + 重建、对每条根把更新的全改坏之后恢复落点、候选之间 inode 号冲突、checker；每次回退之后内存与盘上逐项比。原样行（草稿副本上跑的，源码与 `rerun.sh` 重建的副本逐字节相同，见开头）：

```
RBF2 g1_sweep totals={"formula_revive_reclaimed_sum": 0, "histories": 2400, "orphans_sum": 0, "other_bad_roots": 0, "released_sum": 19833, "remounts": 972, "revive_skipped_sum": 0, "revived_sum": 20694, "rollback_done": 5316, "rollback_to_instance_1": 1772, "rollbacks_with_revive": 4688}
```

六个打中计数全 0（只打印非零键，失败的历史会另打一行，一行都没有）。5316 次回退里孤儿 0：「释放全部落点」与第二轮「只在用户可见单元上求差」在这批历史上释放的是同一批（`released_sum` 与第二轮 `Full` 臂同为 19833）；式子没定义的两种情形 0 次；已回收而要复活的 0 次。单跑 566 秒。

## 十、各改法修哪一格（量过 / 推的）

| 改法 | Y1 水位 | Y2 X2 | Y4 K2 关掉 | Y5 4 态与环 | Y6 SysRot 窗口 | Y7 checker 误红 | Y8 (a) 丢准入 | Y10 (ii) 只读 | Y11 (i) 版本号 |
|---|---|---|---|---|---|---|---|---|---|
| 水位取 max(现行, 环) | 修（量过） | — | — | — | — | — | — | — | — |
| C419 取 Max / MaxHold / SYSCFG | — | 修（量过） | — | — | Max 中、MaxHold 修、SysPre 修（量过） | SYSCFG 引入 | — | — | — |
| K2 开着实例表那一判 | — | — | 修（量过） | — | — | — | — | — | — |
| K3 去重 | — | — | — | 推迟、不修（量过） | — | — | — | — | — |
| 复活前按 R_old 指针核校验和 | — | 修一半（推的） | 修一半（推的） | — | — | — | — | — | — |
| checker 读系统配置里的 F | — | — | — | — | — | 修（推的） | — | — | — |
| I-7.9 (b) / (c) | — | — | — | — | — | — | 修（量过） | — | — |
| (ii) 连只读也拒，或只读留见证读者 | — | — | — | — | — | — | — | 修（推的） | — |
| (i) 写明换 incompat 位 | — | — | — | — | — | — | — | 修（推的） | 修（旧读者一侧量过） |

这条腿自己提的改法（水位取 max、SysPre 的「先写后抬」、checker 读系统配置、(ii) 连只读也拒、(i) 写明 incompat 位）**只在我的模型上量过或只是推的，被攻过零轮**。

## 十一、没打中的形状与取样范围

| 格 | 形状 | 取样 | 结果 |
|---|---|---|---|
| K1 | 式子那一臂放开用户动作 | 2400 段历史、5316 次回退 | 六个计数 0，孤儿 0 |
| K1 | 水位：现行根之前一条也带同一水位 | 2 臂 × 1 段 | 两臂都不中 |
| K2 | 开着这一判：实例 2 的根全读不出 / 只读不出根 / 根与记录都读不出 | 20 + 8 + 12 格（每臂） | 拒或不抛弃，没有第三条路 |
| K3 | 去重下来回回退 5 种模式、3 种长跑 30 步 | 各 1 段 | 候选坏 0；F 停住只到环转过 |
| K4 | MaxHold 两次崩溃枚举 + 用户动作扫 | 690 + 434 格 | 只有「带 F 的根全坏」中 |
| K4 | SysPre 同上 | 774 + 434 格 | 0 |
| K5 | SYSCFG 两读 B1 三流逐点崩溃，每点改坏最新 1..3 条 | 960 点 | 0（I-7.9 除外） |
| K5 | 五臂两次崩溃之后恢复与可写挂载 | 3594 格 | 恢复失败 0、挂载失败 0 |
| K7 | (i) incompat 位读法下旧读者挂新镜像 | 1 份 | 挂不上 |

单次观测：原型、用例、装置都是确定性的；同一套用例在草稿副本与 `rerun.sh` 从冻结副本重建的副本上各跑一遍（第十二节），按规则仍只算这一条腿的一次观测。

## 十二、这条腿自己的限度

- 原型是我自己定的最小规格（第一节），被攻过零轮；K1 式子没定义的两种情形我选了「什么都不做」。
- SYSCFG 的 F 不在真的系统配置槽里，是每块盘末尾两块模拟的；轮换之后另写一次，崩溃窗口比真实实现多一次写（Y6 的 3 个前缀里真实只剩 1 个是推的）。checker 没跟着改（Y7 要的就是这个对照）。
- MaxHold 沿用第二轮原型：扣住只罩挂载那一刻的回收，挂载之内按根环表回收那一路（`record_root_written_by_this_process`）照 F_生效 立刻回收——两次崩溃枚举与用户动作扫的挂载之后都有写，没中，但没专门造「挂载之内环转过之后」的形。
- 两次崩溃枚举的第二次崩溃只取「挂载写流里每一次根槽写之前」与挂完，不是那次挂载的每个写前缀。
- I-7.9 (b)(c) 的记号 / 入口是旁路信息；(c) 在崩溃状态上的判别力是推的。
- K7 (i) 的「新代码按不认识的旧位拒」没改代码量，是按今天 `incompat_bits_are_mountable` 的写法推的。
- 故障只有根槽翻字节、journal 记录翻字节、系统配置那几份翻字节、取写流前缀当崩溃四种；没有真的瞬时读错（「撤掉故障」是同一字节再翻一次）。
- 几何只有两块 4 GiB 盘、每区 8 槽；负载只有一个文件对象（单单元覆盖写、3 单元顺序写）与建 inode。

## 十三、没做什么

- 不判辩方、本地攻方那几格（辩方复核第二轮判决；本地的最少坏槽数与去重计数）；不替主 agent 采纳；副本上的数都没在入库装置上重做。
- 没读禁读清单里的文件（这一轮 Sonnet 与本地攻方的报告、模型、草稿目录，这一轮核查员报告）。
- 第二轮已攻过的挂载时那一形、按字面求差、B1 各盘取小没再攻；今天的规则、Max、MaxHold 三臂的 B1 三流逐点崩溃（第二轮 1323 点）没重跑。
- 没跑门禁、层 0、QEMU、全量 cargo test；只编译、跑了草稿副本与两份复跑副本里的 `singlefs-core`（库）与测试二进制 `rbf3_attack`，经 `run-with-memory-cap.sh 16G` + `capped.sh 12`，没撞上限。开跑前 `ps` 看过，没有性能测量在跑。
- 取样没为时长缩：这一轮最长的一条是 K1 扫描 566 秒，整批不到 40 分钟。
- 草稿目录 `/tmp/claude-1000/m2-rollback-forward-r3-opus/` 里的副本、日志与复跑目录没入库：模型目录里的两份 patch、`rbf3_attack.rs`、`rerun.sh` 能从冻结副本重建它们；各份 `target/` 交回之前删掉。

## 十四、推翻条件

- Y1：水位只取环里读得出的根，现行那一条根读不出时回退、再建 inode，撤掉故障之后候选之间没有同号不同出生代。
- Y4：关掉实例表那一判，被崩溃恢复抛弃的根在实例 3 写过之后仍全部读得对、checker 绿。
- Y5：去重下两态来回超过一圈根环，候选里仍有 4 个不同状态。
- Y6：SysRot 下这一串第一条根落盘、系统配置轮换还没落时崩，重开写行落进回收槽再崩、盘 0 上那条根坏，F_生效 不回落。
- Y7：SYSCFG 下带 F 的根全坏时 checker 用的 F 与系统一样（它读了系统配置）。
- Y8：(a) 在 S4（抬到上限 + 1）上判红。
- Y10：(ii) 下只读恢复在实例 3 的根全坏时不落 C。
- Y11：今天的系统配置读者核格式版本号（版本号改 2 之后挂不上）。
