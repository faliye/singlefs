# alloc-basis-r3 Opus 攻方腿报告（Z1′、Z2′、Z4′，Z5′ 攻方一侧）

轮名 `alloc-basis-r3`；腿 `three-way-attack`（Opus）；立场：假设主 agent 的倾向（Z1′ 甲-T1 + 甲-b′、Z2′ G7、Z4′ G5 + 第九项改式、Z5′ G8 不冲突）是错的。不碰 Z3′。副本探针 06:43–07:19 UTC 写成并跑完，报告 07:19 UTC 起写。**下文所有读数都是副本装置上的数，不是入库装置上的数。**

## 复跑命令与文件指纹

```
cd /home/fy5090/code/singlefs
bash research/prompts/alloc-basis-r3-opus-model/run-probe.sh            # 约 1 分钟；拷两份 crates 副本、打补丁、编译、跑 53 份 .txt 产物、生成 4 张表，写 outputs-rerun/ 并与 outputs/ 逐个 cmp
python3 research/prompts/alloc-basis-r3-opus-model/extract-tables.py research/prompts/alloc-basis-r3-opus-model/outputs/h4c-T1.txt   # 逐次发布表从产物生成，不手抄
```

`run-probe.sh` 先逐个核 6 个文件的 `git hash-object`，仓里改过就从 `crates-at-report/` 换回本腿读的那一版，对不上退出码 3；然后建两份副本：一份只改可见性（`patch-copy.py --visibility-only`，5 处），一份打全部补丁（19 处）；五段历史在环境变量全不设时两份逐字节相同才往下走（退出码 5）。07:14–07:19 UTC 之间先用这个脚本生成 `outputs/`、再原样复跑一次，57 份文件（53 份 .txt、4 张表）与 `outputs/` 逐个相同（脚本末行 `复跑与 outputs/ 逐个相同`），复跑目录随后删掉了。

| 文件 | sha256 |
|---|---|
| `probe.rs` | `ac4199aa2eb330023c4babdcba28103670dba9578ee4e028c97551292a34052b` |
| `patch-copy.py` | `b0f2ded68dd4682756ecf540190b6415ee61361f92ec1206a7072d257d4b3b1b` |
| `extract-tables.py` | `b6bf4c27c38b44de64dcd81f8144d928ea4edf1b526fd7929a6a27d79e21ec29` |
| `run-probe.sh` | `901c17ff9c39a93e7245bbfc51a3885f959550cd82d111bf10f5b65190f29db0` |
| `crates-at-report/allocator.rs`（hash-object `66c9eb1d…`） | `b25aa3f974e59870031bbdceca86eb23cbabf79b05e5e2e1e21ef5ab36582e06` |
| `crates-at-report/mount.rs`（`b228b7f4…`） | `349f1834ebbc2e8331434fe8c4b1315563cdce1cec95cc171906a4d8dc9a4fcd` |
| `crates-at-report/transaction.rs`（`6dd2ef9a…`） | `9c76bd1c4c26074f0d75199448a4e6dd7175b92ff7f3aee5e4e3b1deacd340d7` |
| `crates-at-report/recovery.rs`（`ef4da610…`） | `5281d94d92bc4e940016bc4077ccfe4d6d75cf65bbfd4a1542b2afbbf6e8e694` |
| `crates-at-report/walk.rs`（`1caa43f0…`） | `c9fad43aa6f5ec2e10dba9076ecbb9dd633b3f661a0a2ab5c7920fec2d76c786` |
| `crates-at-report/image.rs`（`bea51e16…`） | `a5d26bfaf540c1273f1507a7d2c9eb5ee326102ac859448933222d7ecd83de00` |
| `outputs/SHA256SUMS`（57 行：53 份 .txt 产物、4 张表各一行；.txt 里 10 份是自证） | `3380d6573f47b5425033313c3202a62cd841770024772344489536f0d666bbe6` |
| `outputs/h1-T1.txt` | `3f5bf3f832fd7c6ba1f126cda7364321c08fc5dca9edcfb7926052d7da53ab20` |
| `outputs/h1f-T1.txt` | `aabc41947e3631045a75c5ed0be31eebde3d5ecc297134e8e3aff2cb9dc51c31` |
| `outputs/h4-earlier-T1.txt` | `f57291cec83ac137d9f5cde12934acc44edcf234bbfb243a1c00d307e134fc97` |
| `outputs/h4-later-T1.txt` | `d49cc5e73eb6acc2403d9688ca9c68dfc1d6bd4e63523551aeeee3a8e6cc6b30` |
| `outputs/h4c-T1.txt` | `63f42eaba8c2686ac530095280e426926bb4b424eb00a590ceb6fe733ee6b809` |
| `outputs/h5-hole-advance-one-T1.txt` | `2ab7fe537c076984ca78a19ac0586a4a39b7e3769368e941c1a4453d0f750995` |
| `outputs/h6-z5-bug-at5-T1-G8.txt` | `af38eec813e45aeebb35e39410f3d55e182950a07ce1ea47372f9882babe840a` |
| `outputs/z2adv-Fkou-cap.txt` | `976d80376000ef2360a25d4d165703235bc6a9dc58acb4f49d15cd7886f4b3e2` |
| `outputs/z2-badimage-hold-removed-checkerNewestF.txt` | `8c726f46cb75a458fb5ae14c1ffb60b5a9a624fe139fc0b22238201734534029` |
| `outputs/x8a.txt` | `7ccacc0189ffe478b1c8bbfee46e6df112ff133c49a0441f6d6726d12433a06d` |

**读的实现**：06:43 UTC 拷副本时逐个 `git hash-object`，07:11 UTC 回仓再核一次，六个都没变：`allocator.rs` `66c9eb1d…`、`mount.rs` `b228b7f4…`、`transaction.rs` `6dd2ef9a…`、`recovery.rs` `ef4da610…`、`image.rs` `bea51e16…` 与正文第三节相同；**`walk.rs` 是 `1caa43f0…`，不是正文记的 `b09366bf…`**（两个哈希在 object 库里都没有，`git cat-file -t` 都报 `could not get object info`，旧版拿不回来）。`git diff HEAD -- crates/singlefs-checker/src/walk.rs` 的 hunk 里有 I-7.4 / I-4.8 也在最新根上判一格（步 6 checker 第一轮判决第五节 1 的改法）；候选集的 F 取法（`walk.rs:799` 起取 `record_bytes[130..138]`）没变。下文 `crates/` 行号都是这六个哈希里的行号。

**读的 kb**（行号都是这些版本里的）：`23-journal的角色与格式.md` `59128ca4…`、`16-发布语义.md` `80ce26aa…`、`28-挂载期承诺量.md` `6032feab…`、`05-快照-空间记账机制.md` `9218db1a…`、`03-空间分配.md` `d5dfe952…`、`invariants.md` `2fef0bfd…`、`milestone/02-second-txn.md` `cad6ac0c…`、`.claude/rules/fs-design.md` `7e4d1fab…`。

**产物一行的读法**（`probe.rs` 的 `line`）：`txg`、`inst` 是这次发布；`F_root` 这条根带的 F；`F_eff` 按恢复的定义（`recovery.rs:351` `effective_rollback_floor`）从盘上现算；`floor_now` = max(F_eff, 环里最旧有效根)，此刻盘上的回收门槛；`acct_*` 这次发布写进记账行的盘 0 的数（单位 16 KiB 槽）；`mem_*` 内存分配器此刻；`iso` 影子账隔离槽数；`held` 扣住槽数；`wrote` 这次发布写出的槽；`hit_abandoned` / `hit_candidate` 这次写出的槽落在发布之前环里哪条被抛弃根 / 候选根（按实例表有效 ∧ txg ≥ F_eff）引用的槽上，`window` 表示那条根正是被这次发布的根盖掉的（单元先写、根后写，崩在中间它还在环里），`durable` 表示这次发布持久之后它仍在环里；`|` 后是 checker 判红的不变量与第一处细节，全绿写 `GREEN`。`ninth` 一行见第三节。

**四种分配器 / 影子账形态在探针里怎么落**（`probe.rs` 的 `before_publish`；只作用在探针驱动的发布上，挂载与回退内部的写行与暖机照 crates 原样）：

| 标签 | 分配器在每次发布之前 | 记账行 | 影子账 |
|---|---|---|---|
| T0 | 不回收（今天的实现） | 今天的 | 挂载、回退、抬 F 时算（今天的实现，G5） |
| T1 | 按此刻盘上的门槛 max(F_eff, 环里最旧有效根) 回收（甲-T1 的分配器那一半） | `SFPROBE_T1`：按这条根持久之后的环与 F_生效（甲-T1 / G7 的记账那一半） | 同 T0（G5：只在挂载、回退、抬 F 时算） |
| T1-G5turnover | 同 T1，回收之后按此刻的候选集调 `mount.rs:213` 那个函数重算影子账（本腿提的，被攻过零轮） | 同 T1 | G5 + 每次回收时重算 |
| T1-conservative | 同 T1 | 同 T1 | 回退之后被抛弃根账里未释放的槽全部隔离（保守读法） |

## 各格判定一览

| # | 格 | 历史 / 镜像（副本上跑的，产物在 `outputs/`） | 中的臂 | 分不分辨臂 | 满足判据字面哪一句 | 判定 |
|---|---|---|---|---|---|---|
| 1 | Z1′ 甲-T1 × 多根共享单元 | H7：可写挂载后「覆盖写 2 次、空发布 3 次」与「1 / 1」两种节奏交替到 txg 70 / 72，中途 G7 形态抬 F | — | — | V1 / V2 | **没打中**：两段 134 个状态 checker 全绿、G8 / G8′ 全绿、没有一次写落在候选根或被抛弃根引用的槽上（1.1） |
| 2 | Z1′ × Z4′ 甲-T1 × 回退之后接着转环，影子账 G5 | H1：A(3)、同实例覆盖写 4–8，回退到 (1,3)（行 9、暖机 10），覆盖写到 33 | **甲-T1 + G5**（主 agent 两格倾向的组合） | 分辨 Z4′ 的臂：同一段历史 G5 + 转环重算、保守读法都是 0 次；不分辨 Z1′ 的活臂（凡挂载内按谓词回收的臂都暴露），T0 挂载内不回收所以不中 | V2 后半「被抛弃根（主句）引用的槽被发出去」 | **打中**：txg 28 的数据单元写在 50176–50177，被抛弃根 (1,5)(1,6)(1,7)(1,8) 还在环里引用它，checker 全绿；把回退实例的根槽抹掉之后恢复挂上 (1,8)，`UnitUnreadable { slot: 50176 }`（1.2） |
| 3 | 同上，里程碑脚本的形状 | H1m：A、B、重开、C、回退到 (1,3)，覆盖写到 30 | 甲-T1 + G5 | 同 #2 | V2 后半，只在崩溃窗口里 | 打中但只在 txg 28 的崩溃窗口（单元写了、根没写）：B (1,4) 那一刻还在环里（1.2 末） |
| 4 | Z4′ 两次回退 | H4：A(3)、覆盖写 4–7，回退到 (1,5)，覆盖写 11–12，第二次回退到更早的 (1,4) / 更晚的 (2,11)，覆盖写到 40 | 甲-T1 + G5 | 同 #2（两个变体 G5 + 转环重算都是 0） | V2 后半 | **打中**：更早那一支 txg 29、更晚那一支 txg 30，同形（3.2） |
| 5 | Z4′ 回退之前抬过 F | H4c：A、覆盖写 4–9，重开，覆盖写 12–15，抬 F 到 12，覆盖写 18–20，回退到 (2,14)，覆盖写到 30，再抬 F 到 27，覆盖写到 46 | — | — | V2 | 没打中（抬 F 那一刻重算影子账把这一族罩住了）（3.3） |
| 6 | Z4′ 第九项原式 | H4、H4c 每一次发布 | 原式「被抛弃根的已分配 − R_old 的已分配」 | 分辨（集合差读法在回退与抬 F 那一刻与 G5 逐个相等） | 正文 Z4′ 推翻观测「第九项与隔离数对不上」 | **打中**：两次回退 44 对 64、10 对 30（少了第一次被抛弃的那一支独占的 20 槽）；回退之前抬过 F 是 −33 对 48（3.4） |
| 7 | Z4′ 第九项改式 | 同上 | 「被抛弃根引用 − 候选集里的根引用、下界 0」 | — | 同上 | 按集合差读：回退、抬 F 那一刻与 G5 相等；同一次挂载里转环之后集合差降到 0，G5 的隔离数不降（陈旧隔离 50 / 64 / 30 / 50 槽到下一次挂载）。按数值差读：两次回退那一刻是 34 对 64，不是同一个量（3.4） |
| 8 | Z4′ 多扣 | 同上 | G5（今天的实现与 G5 + 转环重算都只加不清） | 不分辨 G5 的两种重算时机 | V3 | 隔离位在被抛弃根离开根环之后不清，与 `28-挂载期承诺量.md:30` 第九项「被抛弃的根被轮转覆写时清零」不一致；准入没实现，假性 ENOSPC 是推的（3.5） |
| 9 | Z1′ 甲-T1 × 根槽写失败推进一格重发 | H5：单实例覆盖写，txg 26 的根槽写报错、换 27 重发，38 之后重开，覆盖写到 56 | 甲-T1（甲家同形） | 不分辨甲家臂（与第二轮静默丢写同形） | V1 | 打中：txg 27–49 I-3.1 红（+10 起，重开那次 +140，49 是 +230），50 盖掉陈旧根才绿（1.3） |
| 10 | Z1′ 甲-b′ | H5 每个状态同时算「环里有效根的分配记录并集」 | 甲-b′ | 分辨（甲-T1 的 I-5.2 在洞上绿） | V1 | **打中两处**：① 第 0 代树表的根（txg 0–2）没有分配记录树，并集比走读少 mkfs 树表那 1 槽，txg 3–26 每个状态都少；② 洞上 I-3.1 要绿只能把差挪到 I-5.2：并集 + 空闲 − 单元区 = −11 … −231，重开那次 −141（1.4） |
| 11 | Z1′ 甲-b′ 两套算法 | 按定义 | 甲-b′ | — | V4 后半 | 按 `.claude/rules/fs-design.md:24` 字面够得着（推的，甲-b′ 没实现）（1.4） |
| 12 | Z2′ 合法崩溃态 | Z2 底：A、B、重开（5–7）、覆盖写 8–13，抬 F 到 8，崩在 txg 14 持久之后 | G7′、F-扣′ | 分辨（G7、F-扣 同一状态绿） | V1 | 打中 G7′（记账 101 对并集 66）与 F-扣′（66 对 101）；两组第二轮已有同形，这一轮在今天的哈希上重做（2.1） |
| 13 | Z2′ 崩溃态之后重开 / 回退到 (1,3) | 同上 | — | — | V1 | 四组都绿（2.1） |
| 14 | Z2′ 坏镜像「扣住位被撤掉」 | 同一段历史，回收时不扣住 | F-扣、G7′（checker 取最新根自己的 F） | 分辨（G7、F-扣′ 红） | V4 | 打中：txg 14 把记账树写在 50240（A 的 extent 根），崩溃态全绿；回退到 (1,3) `UnitUnreadable { slot: 50240 }`（2.2） |
| 15 | Z2′ 推进一格重发 | 同一段历史，抬 F 时 txg 16（盘 1）根槽写报错、换 17 重发，循环照 `mount.rs:480` 按成功次数 < 3 截断 | F-扣（今天的循环形状 + 条款的推进一格） | 分辨（G7 同一循环形状不回收） | V2 前半「候选集里的根引用的槽被发出去」 | 打中（今天 crates 里没有推进一格重发，这一格是按条款补上之后的形状，推的）：没落满盘 1 就放开扣住，txg 18 的数据写在 50176，候选根 (1,3)(1,4) 还引用它；最新根 F 的 checker 全绿（2.3） |
| 16 | Z2′ X8-A | 分配器层：每盘 128 槽，段 0 全占、段 1 全已释放，抬 F 到 8 | 四组 | **不分辨** | V3「进程发不出固定点」 | 四组都拿不到固定点；F-扣 的空闲计数报 64 而 G7 报 0。正文给 G7 写的推翻观测「在 X8-A 那一格上同样卡死」按字面触发，但病根是保留池没实现（2.4） |
| 17 | Z2′ Y1（已知，并进来不重攻） | 步 6 checker 第一轮判决第四节 Y1 | F-扣、G7′ 漏判 | 分辨 | V4 | 并表（2.5） |
| 18 | Z5′ G8 | H6：单实例覆盖写，txg 5 那次发布照抄实例表却把实例表单元也放进释放表（活单元记成已释放），T1，覆盖写到 31 | G8 | 分辨（本腿的 G8′ 在同一段 23 个状态上红） | V4「某份坏镜像全部检查判绿」 | **打中**：txg 5–27 共 23 个状态 checker 26 条与 G8 全绿；28 I-3.1 红；29 数据写到 50176，实例表坏（4.1） |
| 19 | Z5′ 正文举的跨统计量等式 | 第一个事务 A 那一行的数 | 「defer + 空闲 + 已分配里仍被候选根引用的 == 单元区」 | — | V1 | 按字面在第一个事务上就不成立（1 + 211955 + 13 = 211969 ≠ 211968）；换成「最新根」就是 G8′（4.2） |

## 一、Z1′ 甲-T1 与环上的洞（含甲-b′）

### 1.1 多根共享单元：没打中

空发布照抄四个文件单元（`transaction.rs:1146` 起那一支，第 1147 行注释整行 `// 照抄：数据指针、inode 记录、四个单元的字节都取上一版；inode 叶的指针从上一版 inode 根的那条条目解出来。`），所以一个文件单元被「写它那次」到「下一次覆盖写之前」的每条根共享；写行与暖机同理。H7 在可写挂载之后按「覆盖写 2、空发布 3」与「1、1」两种节奏交替，txg 30 附近按 G7 形态抬 F 到上限（两块盘都带上新 F 的根之后才回收），再交替到 txg 72 / 70；记账 T1，checker 取现算 F_生效，另开 G8 / G8′。`outputs/h7-shared-2-3-T1-G7-G8.txt` 第 27–30 行与第 70 行整行：

```
H7 raise arm=G7 F=21 ceiling=21 reclaimed_before_first_new_F_root=0
H7 raise-publish txg=31 inst=2 F_root=21 F_eff=0 floor_now=8 acct_alloc=152 acct_free=211816 acct_defer=140 mem_alloc=156 mem_defer=144 mem_free=211812 iso=0 held=0 root_write_failed_at=[] wrote=[50455-50458] hit_abandoned=- hit_candidate=- | GREEN
H7 raise-publish txg=32 inst=2 F_root=21 F_eff=21 floor_now=21 acct_alloc=74 acct_free=211894 acct_defer=62 mem_alloc=156 mem_defer=144 mem_free=211812 iso=0 held=0 root_write_failed_at=[] wrote=[50459-50462] hit_abandoned=- hit_candidate=- | GREEN
H7 G7 reclaimed after both devices carry F: 72
H7 empty txg=72 inst=2 F_root=21 F_eff=21 floor_now=49 acct_alloc=152 acct_free=211816 acct_defer=140 mem_alloc=162 mem_defer=150 mem_free=211806 iso=0 held=0 wrote=[50427-50430] hit_abandoned=- hit_candidate=- | GREEN
```

两份产物 134 个状态行（`grep -c 'acct_alloc=.* | '`：68、66）全部 `GREEN`，没有一行 `hit_candidate=(` 或 `hit_abandoned=(`。机理：一个文件单元的释放代是第一条不再引用它的根的 txg，单一时间线上引用它的根 txg 都小于释放代，「释放代 ≤ 环里最旧有效根」成立时它们都已不在环里；共享只拉长引用区间，不打破这条序。回退之后照抄 R_old 单元的那一支在 1.2 的 H1 / H4 / H4c 里，I-3.1 同样全绿。

### 1.2 回退之后接着转环：I-3.1 绿，主句被违反（打中，甲-T1 与 G5 合起来）

**机理**。`mount.rs:213` 的影子账只隔离「只被被抛弃根引用」的槽，豁免集是当前账里未释放的落点加上候选根（`mount.rs:228` `.filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)`）引用的落点。回退到 (1,3) 那一刻，mkfs 实例表 m1（50176–50177）被 A 的账与候选根 0–3 引用，所以豁免、不隔离；而同实例的覆盖写 4–8 照抄 A 的实例表，被抛弃根 (1,4)–(1,8) 也引用它。回退的写行（txg 9）重写实例表，把 m1 释放，释放代 9。环转到 txg 27 持久之后，0–3 都被盖掉，环里最旧有效根跳到 9（4–8 被实例表判为被抛弃），`16-发布语义.md:371` 的谓词「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」对 m1 成立；甲-T1 的分配器在下一次分配之前回收它，而影子账只在挂载、回退、抬 F 三处算（今天的实现与正文 G5 的定义「抬 F 后重算」一致），没人给它补隔离，于是 `lowest_user_data_slot`（`allocator.rs:317`）把它发出去。被抛弃根 (1,5)–(1,8) 要到 txg 29–32 才被盖掉。

`outputs/h1-T1.txt` 第 41、42 行整行：

```
H1 ninth txg=27 F_eff=0 candidates=[9-27] abandoned=[4-8] below_F=[] | orig((1,8)−(1,3))=50 set_reading=52 g5_def=52 narrow_by_valid=52 count_reading=0 G5_isolated=50[50182-50191,50249-50288] stale_isolated=[] missing_isolation=[50176-50177] lowest_user_slot=50224
H1 overwrite txg=28 inst=2 F_root=0 F_eff=0 floor_now=9 acct_alloc=196 acct_free=211772 acct_defer=184 mem_alloc=196 mem_defer=184 mem_free=211772 iso=50 held=0 wrote=[50176-50177,50451-50458] hit_abandoned=(1,6)durable[50176-50177](1,4)window[50176-50177](1,7)durable[50176-50177](1,5)durable[50176-50177](1,8)durable[50176-50177] hit_candidate=- | GREEN
```

（`lowest_user_slot` 是这次发布之后、下一次回收之前算的，所以 txg 27 那一行还是 50224。）

**对照**，同一段历史 txg 28 那一行整行：

```
outputs/h1-T0.txt:42:H1 overwrite txg=28 inst=2 F_root=0 F_eff=0 floor_now=9 acct_alloc=203 acct_free=211765 acct_defer=191 mem_alloc=203 mem_defer=191 mem_free=211765 iso=50 held=0 wrote=[50226-50227,50451-50458] hit_abandoned=- hit_candidate=- | I-3.1[盘 0：记账的已分配 Some(3325952)，遍历全部有效根得到 3211264]
outputs/h1-T1-G5turnover.txt:42:H1 overwrite txg=28 inst=2 F_root=0 F_eff=0 floor_now=9 acct_alloc=196 acct_free=211772 acct_defer=184 mem_alloc=196 mem_defer=184 mem_free=211772 iso=52 held=0 wrote=[50224-50225,50451-50458] hit_abandoned=- hit_candidate=- | GREEN
outputs/h1-T1-conservative.txt:42:H1 overwrite txg=28 inst=2 F_root=0 F_eff=0 floor_now=9 acct_alloc=196 acct_free=211772 acct_defer=184 mem_alloc=196 mem_defer=184 mem_free=211772 iso=52 held=0 wrote=[50224-50225,50451-50458] hit_abandoned=- hit_candidate=- | GREEN
```

T0（今天）挂载内不回收，所以不发 m1，但 I-3.1 红（第二轮 Z1-0）；G5 + 转环重算在 txg 28 之前把 m1 补进隔离（`iso=52`）；保守读法回退那一刻就隔离了它。四份产物里 `grep -c 'hit_abandoned=('`：T1 1、T0 0、G5turnover 0、conservative 0（H1），H4 两个变体同样 T1 各 1、G5turnover 各 0（3.2）。

**后果**。`23-journal的角色与格式.md:1243` 整行：

```
⚠️ **回退例外里「那次发布之前都不生效」这一句要靠影子账才成立**：回退那次发布先写它自己的 COW 单元（D16（发布语义） 已定项 7 的顺序），而分配器从 R_old 的账重新载入、I-7.4（近 K 代块未被复用） 不护被抛弃的根，那些单元可以落在最新那个根引用的单元上；崩在记录之前，下一次恢复挂上那个根、读到被复用的单元（第二轮反推腿 Y-R，零故障）。回退确认之后同样可以复用，再让回退实例的根全读不出就挂上被抛弃的根。落 [checks-owed.md](../checks-owed.md) C314（回退可以复用被抛弃的根引用的单元）。
```

照其中「回退确认之后同样可以复用，再让回退实例的根全读不出就挂上被抛弃的根。」那一种，探针在 txg 29 之后把回退实例（实例 2）发布过的每个根槽抹成零，再可写挂载，`outputs/h1f-T1.txt` 第 2、3 行与两份对照的第 3 行整行：

```
H1F wiped root slots of instance 2: txgs=[24, 27, 9, 12, 15, 18, 21, 25, 28, 10, 13, 16, 19, 22, 26, 29, 11, 14, 17, 20, 23] ; recovery now chooses (1,8) ; checker on the wiped image: I-2.1[实例表单元 在盘 0 槽 50176 的那一份与位置条目里的校验和对不上] ; I-3.1[盘 0：记账的已分配 Some(1032192)，遍历全部有效根得到 524288] ; I-4.8[最新根（txg 8）出发的遍历有单元对不上或读不出] ; I-7.2[最新的根走不完：实例表单元两份都读不到对得上的] ; I-7.4[最新根（txg 8）引用的单元已被复用或抹头（校验和对不上或头用不了）]
H1F mount on the fallback root failed: Recovery(UnitUnreadable { slot: SlotNumber(50176) })
outputs/h1f-T0.txt:3:H1F mount on the fallback root ok: instance=3 txg=31
outputs/h1f-T1-G5turnover.txt:3:H1F mount on the fallback root ok: instance=3 txg=31
```

（抹过的镜像上 I-3.1 三份都红，是挂上 (1,8) 之后它自己那份账与候选集的差，不分辨。）

**里程碑脚本的形状**（H1m：A、B、重开、C、回退到 (1,3)）只有 B (1,4) 照抄 m1，实例 2 的写行（txg 5）已经把它换掉，所以只剩 txg 28 的崩溃窗口：`outputs/h1m-T1-crash28.txt` 第 40 行整行 `H1m CRASH-WINDOW txg=28 (units + record written, root not) wrote=[50176-50177,50515-50522] hit_abandoned=(1,4)window[50176-50177] hit_candidate=- | disk-state GREEN`。回退之前同实例多写几次（H1 的 4–8、H4 的 4–7）窗口就变成持久的。

四句：
1. 分不分辨臂：分辨 Z4′ 的影子账形态（G5 中；G5 + 转环重算、保守读法同一段历史 0 次）。不分辨 Z1′ 的活臂：甲-T1、甲-b′（挂载内按甲-T1 回收）、甲-a、乙′ 的分配器都「按分配那一刻」回收，都暴露；甲-T0 不暴露但已在第二轮 W1 输。按反向接受条款，这一格记 G5 输一次，另记一笔 Z1′ 各臂共用的前提：**凡挂载内回收的臂，影子账要跟着候选集变**。
2. 系统当时看不看得到判别它的东西：看得到。写者自己在回退时算过影子账，知道哪些根被抛弃；每次回收之前按此刻的环再算一次就是 0（G5turnover 那份）。
3. 满足判据字面哪一句：V2 后半「被抛弃根（主句）引用的槽被发出去」；主句是 `23-journal的角色与格式.md:1209` 里的「被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」（到句号为止的整句抄在第三节开头），`invariants.md:50` I-7.4 那一行也写「它们引用的块在离开根环之前同样不许重新分配、不许抹头」。checker 不走被抛弃根（步 6 checker 第一轮判决第五节 3 那条欠账），所以全绿。
4. 跑前条款给的改法在这几格上还中不中：正文 Z4′ 的改法是第九项改式，不碰这一格；「G5 在每次回收门槛抬高（抬 F 或环里最旧有效根变大）时按此刻候选集重算」与保守读法在这几格上 0 次，前者是本腿提的、被攻过零轮（第六节）。

### 1.3 根槽写失败按条款推进一格重发：洞与静默丢写同形

条款逐字（`23-journal的角色与格式.md:691` 已定项 14 索引行里那一句）：「根槽写失败重发时 checkpoint_txg 推进一格再发（根环区域 = `txg mod R`，D22（单元原子性怎么合成） 已定项 2，不推进就是反复重写同一个槽、把 R 个失败域用成一个）；」。探针的 `MemDevice` 在 txg 26 的根槽偏移上报 `InputOutput` 错，`publish_version` 把分配器退回进来时的样子（`transaction.rs:955-959`），探针换 txg 27、jsn + 1 重发同一个事务。txg 26 的槽里留着 txg 2 的根，txg 27 盖掉的是 txg 3（A）。`outputs/h5-hole-advance-one-T1.txt` 第 46、70、88、90 行整行：

```
H5 overwrite txg=27 inst=1 F_root=0 F_eff=0 floor_now=2 acct_alloc=243 acct_free=211725 acct_defer=231 mem_alloc=243 mem_defer=231 mem_free=211725 iso=0 held=0 root_write_failed_at=[26] wrote=[50226-50227,50425-50432] hit_abandoned=- hit_candidate=- | I-3.1[盘 0：记账的已分配 Some(3981312)，遍历全部有效根得到 3817472]
H5 remount(row+warm) txg=40 inst=2 F_root=0 F_eff=0 floor_now=2 acct_alloc=363 acct_free=211605 acct_defer=351 mem_alloc=363 mem_defer=351 mem_free=211605 iso=0 held=0  | I-3.1[盘 0：记账的已分配 Some(5947392)，遍历全部有效根得到 3653632]
H5 overwrite txg=49 inst=2 F_root=0 F_eff=0 floor_now=2 acct_alloc=453 acct_free=211515 acct_defer=441 mem_alloc=453 mem_defer=441 mem_free=211515 iso=0 held=0 wrote=[50522-50523,50699-50706] hit_abandoned=- hit_candidate=- | I-3.1[盘 0：记账的已分配 Some(7421952)，遍历全部有效根得到 3653632]
H5 overwrite txg=50 inst=2 F_root=0 F_eff=0 floor_now=27 acct_alloc=232 acct_free=211736 acct_defer=220 mem_alloc=463 mem_defer=451 mem_free=211505 iso=0 held=0 wrote=[50524-50525,50707-50714] hit_abandoned=- hit_candidate=- | GREEN
```

3981312 / 16384 = 243 对 233（+10），重开之后的暖机状态 363 对 223（+140），txg 49 是 453 对 223（+230），txg 50 盖掉陈旧的 txg 2 之后回 0。`floor_now` 在 27–49 停在 2（陈旧根按 `16-发布语义.md:372`「写失败的槽按旧内容算」算进环里最旧有效根）。与第二轮静默丢写那一格同形、同幅度；跨一次重开也救不回来（重开按此刻环里最旧有效根 2 回收，与挂载内同一个门槛）。

四句：① 不分辨甲家臂（甲-T0、甲-T1 同一个门槛；甲-b′ 见 1.4）；② 写者知道 txg 26 那次失败，但跨一次重开之后只看得到槽里的陈旧根，看不出它是洞；③ V1「写失败推进重发之后的状态」；④ 没有跑前条款的改法。这一格按反向接受条款不拿来判甲家三条臂之间的高低，是第二轮账 3 在条款要求的写失败路径上的再现。

### 1.4 甲-b′：洞没变干净，而且在第一个事务上就与 checker 差一槽

甲-b′ 在正文第四节的定义：「挂载与回退时读环里每条有效根的分配记录树精确重建「仍被某条有效根引用」，挂载之内写者在内存里记着哪次根槽写失败、按甲-T1 回收」。探针没实现甲-b′ 的写者，只在 H5 每个状态上按这个定义算出它挂载那一刻会写的「精确并集」（`bprime_line`：环里按实例表有效的根，逐条 `allocation_records_under_root` 取未释放的落点，盘 0），与记账行、走读并排。

**① 第 0 代树表的根没有分配记录树**。`recovery.rs:373` 文档注释整行：`/// 一条根引用的分配记录（树表 → 分配记录树根节点）：回退的影子账要读每条被抛弃根的账。第 0 代树表（没有分配记录树）给空。`。txg 0（mkfs）与第一次挂载暖机的 txg 1、2 指着第 0 代树表，它们引用的 mkfs 树表单元 50178 在任何分配记录树里都找不到「被它们引用」。第一个事务 A 之后 50178 只被它们引用（A 已把它释放），于是「精确并集」比走读少 1。`outputs/h5-hole-advance-one-T1.txt` 第 1、45 行整行：

```
H5 A txg=3 ring_txgs=[0-3] acct1=13 acct2=211955 acct5=1 capacity=211968 | T1: acct1+acct2−cap=0 | b′: precise_candidates=12 precise_valid=12 acct1−precise=1 ; b′-I52(precise+acct2−cap)=-1 ; b′-I52(precise+mem_free−cap)=-1
H5 b′ txg=25 ring_txgs=[2-25] acct1=233 acct2=211735 acct5=221 capacity=211968 | T1: acct1+acct2−cap=0 | b′: precise_candidates=232 precise_valid=232 acct1−precise=1 ; b′-I52(precise+acct2−cap)=-1 ; b′-I52(precise+mem_free−cap)=-1
```

这两个状态上 checker 都是 `GREEN`：txg 3 见 `outputs/h6-z5-control-T1-G8.txt` 第 1 行（同一段 A，`acct_alloc=13 … | GREEN`），txg 25 见本份第 44 行（`acct_alloc=233 … | GREEN`），也就是 checker 的并集 = 记账第 1 项 = 13 / 233。所以按定义写的甲-b′，在 txg 3–26 之间任何一次挂载或回退写出的第 1 项都比 checker 少 1，I-3.1 红。要绿就得给第 0 代树表的根单开一支「它们引用 mkfs 的两个单元」——这一支正是走读与分配记录不同的地方。（这一格只在我的装置上量过「并集少 1」这个数；「会红」是按 `walk.rs:871` 的等式推的，没实现甲-b′ 的写者。）

**② 洞上：I-3.1 要绿，差就挪进 I-5.2**。同一份产物第 47、71、89、91 行整行（第 1 项 − 精确并集里含 ① 的那 1 槽，陈旧根 txg 2 在洞里一直在环）：

```
H5 b′ txg=27 ring_txgs=[2,4-25,27] acct1=243 acct2=211725 acct5=231 capacity=211968 | T1: acct1+acct2−cap=0 | b′: precise_candidates=232 precise_valid=232 acct1−precise=11 ; b′-I52(precise+acct2−cap)=-11 ; b′-I52(precise+mem_free−cap)=-11
H5 remount(row+warm) b′ txg=40 ring_txgs=[2,17-25,27-40] acct1=363 acct2=211605 acct5=351 capacity=211968 | T1: acct1+acct2−cap=0 | b′: precise_candidates=222 precise_valid=222 acct1−precise=141 ; b′-I52(precise+acct2−cap)=-141 ; b′-I52(precise+mem_free−cap)=-141
H5 b′ txg=49 ring_txgs=[2,27-49] acct1=453 acct2=211515 acct5=441 capacity=211968 | T1: acct1+acct2−cap=0 | b′: precise_candidates=222 precise_valid=222 acct1−precise=231 ; b′-I52(precise+acct2−cap)=-231 ; b′-I52(precise+mem_free−cap)=-231
H5 b′ txg=50 ring_txgs=[27-50] acct1=232 acct2=211736 acct5=220 capacity=211968 | T1: acct1+acct2−cap=0 | b′: precise_candidates=232 precise_valid=232 acct1−precise=0 ; b′-I52(precise+acct2−cap)=0 ; b′-I52(precise+mem_free−cap)=-231
```

甲-b′ 按甲-T1 回收，洞里那些「没有有效根引用、释放代却大于门槛」的槽分配器不放回空闲。第 1 项改成精确并集之后，I-5.2（`invariants.md:167`「空闲空间统计 == 总空间 − 已分配空间」）的两边差的正是洞里多扣的量：txg 27 −11、重开那次 −141、txg 49 −231（各含 ① 的 1）。要让 I-5.2 也绿只剩三条路：把第 2 项改成「单元区 − 精确并集」现算，`05-快照-空间记账机制.md:398-401` 四行接起来逐字「⚠️ **第 2 项必须独立维护，不许由 `容量 − 已分配` 现算。** I-5.2（空闲统计对得上） 逐字就是「空闲统计 == 总空间 − 已分配空间」——若空闲就是这么算出来的，**那条不变量是恒真式，判别力为零**（`.claude/singlefs-ai-sop/rules/test-discipline.md`「写完一个检查，问：如果被测对象真的坏了，它会不会红」）。两个数要走两条不共享的加减路径。」；或者把洞里那些槽算进空闲而分配器不发，那是 V3「报着空闲写不进」；或者给 I-5.2 加一项「分配器扣着而没有有效根引用的量」，那一项就是回收谓词在洞上多扣的量，与第二轮甲-a 精确读法同一个东西。**跨一次重开**：重开那次（txg 40）仍是 −141，没有变好——挂载那一刻的精确并集罩得住 I-3.1，罩不住分配器按谓词扣着的槽。

**③ 两套算法**。`.claude/rules/fs-design.md:24` 整行：`| **checker / 审计** | **必须**遍历 | 若运行时也用遍历算，checker 的遍历与运行时就是**同一次计算**，对照关系当场归零 |`。甲-b′ 挂载时对环里每条有效根遍历分配记录树、按同一个根集合（按实例表有效，再按正文的读法可能加 txg ≥ F）取并集；checker 对同一个根集合遍历指针取并集。两边只差「记录」与「指针」这一层输入，而 ① 说明这一层恰好在第 0 代树表的根上不同——那是要打补丁的地方，不是独立性的来源；根集合怎么选两边共用，写错了两边一起错。按字面够得着那一行（推的，甲-b′ 没实现）。对照：甲-T1 的第 1 项由分配器按释放代增量维护、不遍历，与 checker 的遍历并集是两种算法，它盯住的正是释放代簿记——甲-b′ 把第 1 项从释放代上摘下来之后，这一层只剩 I-5.2 在看，而 ② 里 I-5.2 在洞上本来就红，分不出是洞还是簿记错了。

四句（① ② 合并）：① 分辨甲-b′（甲-T1 在 ① 那些状态上全绿、在 ② 那些状态上 I-5.2 绿）；② 写者挂载时看得到环里每条根的树表是不是第 0 代，看不出槽是不是「被洞扣着」除非它另记；③ V1（① 是合法持久状态上 I-3.1 与第 1 项差 1，推的；② 是写失败推进重发之后的状态上 I-5.2 差 11–231，按定义算出来的数）；④ 正文没给甲-b′ 改法。

## 二、Z2′ 两个 F 的四组

底子（`probe.rs` 的 `z2_base`，与第二轮 2.1 那一格、步 5 的扣住用例同形）：A(3)、覆盖写 4、重开（行 5、暖机 6、7）、覆盖写 8–13，抬 F 到 8（上限 10）。抬 F 在探针里照 `mount.rs:400-513` 一步一步做（`raise_manual`），能停在第一次发布之后、能注入根槽写失败；F-扣 那一支照 `mount.rs:452-473` 先按新 F 重算影子账（调的就是 `mount.rs:213` 那个函数）再回收并扣住，G7 那一支落满两块盘之后才重算影子账、回收。四组的记账与 checker：F-扣 = 今天的记账 + checker 取最新根自己的 F；F-扣′ = 今天的记账 + checker 现算 F_生效（`SFPROBE_CHECKER_FLOOR=effective`）；G7 = `SFPROBE_T1` 的记账 + 现算 F_生效；G7′ = `SFPROBE_T1` 的记账 + 最新根自己的 F。

### 2.1 合法崩溃态（崩在 txg 14 持久之后）与之后的重开、回退

整行（每份产物第 4 或 5 行是崩溃态，下一行是崩溃态上重开，再下一行是崩溃态上回退到 (1,3)）：

```
outputs/z2-Fkou.txt:4:Z2[fkou] crash-after-first-new-F-root txg=14 inst=2 F_root=8 F_eff=0 floor_now=0 acct_alloc=66 acct_free=211902 acct_defer=54 mem_alloc=66 mem_defer=54 mem_free=211902 iso=0 held=35  | GREEN
outputs/z2-Fkou-prime.txt:4:Z2[fkou] crash-after-first-new-F-root txg=14 inst=2 F_root=8 F_eff=0 floor_now=0 acct_alloc=66 acct_free=211902 acct_defer=54 mem_alloc=66 mem_defer=54 mem_free=211902 iso=0 held=35  | I-3.1[盘 0：记账的已分配 Some(1081344)，遍历全部有效根得到 1654784]
outputs/z2-G7.txt:5:Z2[g7] crash-after-first-new-F-root txg=14 inst=2 F_root=8 F_eff=0 floor_now=0 acct_alloc=101 acct_free=211867 acct_defer=89 mem_alloc=101 mem_defer=89 mem_free=211867 iso=0 held=0  | GREEN
outputs/z2-G7-prime.txt:5:Z2[g7] crash-after-first-new-F-root txg=14 inst=2 F_root=8 F_eff=0 floor_now=0 acct_alloc=101 acct_free=211867 acct_defer=89 mem_alloc=101 mem_defer=89 mem_free=211867 iso=0 held=0  | I-3.1[盘 0：记账的已分配 Some(1654784)，遍历全部有效根得到 1081344]
outputs/z2-Fkou-prime.txt:5:Z2[fkou] crash->remount txg=16 inst=3 F_root=0 F_eff=0 floor_now=0 acct_alloc=111 acct_free=211857 acct_defer=99 mem_alloc=111 mem_defer=99 mem_free=211857 iso=0 held=0  | GREEN
outputs/z2-G7-prime.txt:7:Z2[g7] crash->rollback(1,3) txg=16 inst=3 F_root=0 F_eff=0 floor_now=0 acct_alloc=23 acct_free=211945 acct_defer=11 mem_alloc=23 mem_defer=11 mem_free=211945 iso=88 held=0  | GREEN
```

1081344 / 16384 = 66、1654784 / 16384 = 101。崩溃态上 F-扣′ 与 G7′ 各红、方向相反；两份产物里不绿的状态行（`grep -n "acct_alloc=.* | " | grep -v "GREEN$"`）各 4 行，都是 txg 14、15 那两个「只有盘 0 带着新 F」的状态（停在第一条根之后那一段两行、完整抬 F 那一段两行）；txg 16 落满盘 1 之后、崩溃态之后重开、回退到 (1,3)、E（txg 17）四组都绿，F-扣 与 G7 两份产物没有一行不绿。重开把新实例的根写成 F_生效 = 0，两种 checker 取的 F 重新相等。

四句（F-扣′、G7′ 那两格）：① 分辨（同一状态 F-扣、G7 绿）；② 看得到：各盘所带 F 都在盘上；③ V1 字面「崩溃恢复状态」上 I-3.1 红；④ 正文没给这两组改法——它们就是「记账与 checker 按不同时刻的 F 说话」的两个方向，第二轮 2.2 已有同形，这一轮在今天的哈希上重做。

### 2.2 坏镜像「扣住位被撤掉、回收的槽在生效之前被发出去」

同一段历史，回收时用 `ReclaimedReuse::Immediately`（不扣住）。txg 14 的记账树落在 50240、树表落在 50242，正是 A (1,3) 的 extent 根与 inode 叶。整行：

```
outputs/z2-badimage-hold-removed-checkerNewestF.txt:3:Z2[fkou-nohold] raise-publish txg=14 inst=2 F_root=8 F_eff=0 floor_now=0 acct_alloc=66 acct_free=211902 acct_defer=54 mem_alloc=66 mem_defer=54 mem_free=211902 iso=0 held=0 root_write_failed_at=[] wrote=[50240-50242,50367] hit_abandoned=- hit_candidate=(1,3)durable[50240,50242] | GREEN
outputs/z2-badimage-hold-removed-checkerNewestF.txt:6:Z2[fkou-nohold] crash->rollback(1,3) failed: Recovery(UnitUnreadable { slot: SlotNumber(50240) })
outputs/z2-badimage-hold-removed-checkerEffective.txt:4:Z2[fkou-nohold] crash-after-first-new-F-root txg=14 inst=2 F_root=8 F_eff=0 floor_now=0 acct_alloc=66 acct_free=211902 acct_defer=54 mem_alloc=66 mem_defer=54 mem_free=211902 iso=0 held=0  | I-2.1[树 11（种类 1）的根 在盘 0 槽 50240 的那一份与位置条目里的校验和对不上] ; I-3.1[盘 0：记账的已分配 Some(1081344)，遍历全部有效根得到 1605632] ; I-4.8[候选根 txg 3 出发的遍历有单元对不上或读不出] ; I-5.1[盘 0：树表单元（槽 50242 跨 1）与 inode 树的孩子（槽 50242）重叠] ; I-7.4[候选根 txg 3 引用的单元已被复用或抹头（校验和对不上或头用不了）]
```

（第 3 行的 checker 判的是 txg 14 持久之后的盘，与第 4 行崩溃态是同一个盘面。）取最新根 F 的 checker（F-扣、G7′）全绿；现算 F_生效 的 checker（G7、F-扣′）五条红。四句：① 分辨；② 看得到（F_生效 从盘上算得出）；③ V4「某份坏镜像全部检查判绿」；④ 这正是第二轮 2.1 那个真坏的镜像换成「今天的实现少了扣住」的说法——F-扣 的 checker 对「扣住有没有生效」没有任何判别力，扣住这件事只有步 5 的用例与 `crates/mutations.tsv` 第 54、55 行那两条变异（「抬 F 回收的槽不扣住、生效之前就能发出去」「抬 F 生效之后不放开扣住的槽」）在守。

### 2.3 根槽写失败推进一格重发：F-扣 按今天的循环形状在生效之前放开扣住

`mount.rs:477-481` 的循环条件整行：

```
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
```

`mount.rs:506` 在循环之后无条件 `allocator.release_reclaim_holds();`。步 4 / 步 5 代码三方第三轮判决第五节 4 把它记成欠账：「`release_reclaim_holds` 在循环之后无条件放开、靠区域循环长度 = 3 担保落满每块盘（Sonnet T3a）」。今天 `publish_version` 报写错时 `?` 直接返回（`mount.rs:498`），crates 里没有推进一格重发；探针按 `23-journal的角色与格式.md:691` 那一句补上重发，循环照原样按成功发布次数截断。txg 14（盘 0）、15（盘 0）、16（盘 1）写失败 → 17（盘 0）：三次成功都在盘 0，循环退出，扣住放开。整行：

```
outputs/z2adv-Fkou-cap.txt:6:Z2adv[fkou,cap=true] released holds covered=false
outputs/z2adv-Fkou-cap.txt:8:Z2adv[fkou,cap=true] overwrite txg=18 inst=2 F_root=8 F_eff=0 floor_now=0 acct_alloc=84 acct_free=211884 acct_defer=72 mem_alloc=84 mem_defer=72 mem_free=211884 iso=0 held=0 wrote=[50176-50177,50379-50386] hit_abandoned=- hit_candidate=(1,3)durable[50176-50177](1,4)durable[50176-50177] | GREEN
outputs/z2adv-Fkou-cap-checkerEffective.txt:8:Z2adv[fkou,cap=true] overwrite txg=18 inst=2 F_root=8 F_eff=0 floor_now=0 acct_alloc=84 acct_free=211884 acct_defer=72 mem_alloc=84 mem_defer=72 mem_free=211884 iso=0 held=0 wrote=[50176-50177,50379-50386] hit_abandoned=- hit_candidate=(1,3)durable[50176-50177](1,4)durable[50176-50177] | I-2.1[实例表单元 在盘 0 槽 50176 的那一份与位置条目里的校验和对不上] ; I-3.1[盘 0：记账的已分配 Some(1376256)，遍历全部有效根得到 1916928] ; I-4.8[候选根 txg 0 出发的遍历有单元对不上或读不出] ; I-7.4[候选根 txg 0 引用的单元已被复用或抹头（校验和对不上或头用不了）]
outputs/z2adv-G7-cap.txt:6:Z2adv[g7,cap=true] G7 not covered: nothing reclaimed
outputs/z2adv-G7-cap.txt:8:Z2adv[g7,cap=true] overwrite txg=18 inst=2 F_root=8 F_eff=0 floor_now=0 acct_alloc=119 acct_free=211849 acct_defer=107 mem_alloc=119 mem_defer=107 mem_free=211849 iso=0 held=0 wrote=[50196-50197,50379-50386] hit_abandoned=- hit_candidate=- | GREEN
```

txg 18 的数据写在 m1（50176），而盘 1 上还没有带 F = 8 的根（F_eff = 0），A (1,3)、B (1,4) 与第 0 代根都是回退候选。不截断的循环（`z2adv-Fkou-nocap.txt`、`z2adv-G7-nocap.txt`）落满 19 才放开 / 回收，三次覆盖写都没撞槽。

四句：① 分辨：同一循环形状 G7 不回收（它回收的时点就是「落满」本身）；② 看得到：循环自己维护 `covered`；③ V2 前半「候选集里的根引用的槽被发出去」；④ 条款层的 F-扣（「生效之后放开」）不中，中的是「F-扣 + 今天的循环上限 + 条款要求的推进一格重发」这个组合——**推的**：推进一格重发没实现，实现它的人照现有循环加一个重试，步 4 / 步 5 第三轮判决第五节 4 那条「今天不可达」就变成可达。

### 2.4 X8-A 那一格：四组都拿不到固定点，不分辨

步 4 / 步 5 第三轮判决第四节 X8-A 当输入读，不重攻；这里只在分配器层把「G7 在同一格上怎样」补上。每盘 128 槽、两段，段 0 全部仍分配、段 1 全部已释放（释放代 5），抬 F 到 8，要一个单槽固定点。`outputs/x8a.txt` 第 1–3 行整行：

```
X8A F-扣: reclaimed=64 first_fixed_point=None retry_reclaimed=0 retry_fixed_point=None free_slots=64 allocated_slots=64 deferred=0
X8A G7: first_fixed_point=None free_slots=0 allocated_slots=128 deferred=64
X8A F-扣+α(失败路径放开扣住): first=None after_release=Some(50240)（发出去的是生效之前回收的槽）
```

G7 不先回收，段 1 的 `used_per_segment` 不为 0，`lowest_empty_segment`（`allocator.rs:337-352`）照样给不出段；F-扣 回收了但段 1 被扣住。两组都发不出第一条带新 F 的根，也就都抬不了 F。差别只在内存里的空闲计数：F-扣 报 64（重试时 `reclaim_released_up_to` 跳过已回收的，扣住位永远不放，这个进程里一直报 64 而发不出），G7 报 0。正文给 G7 写的推翻观测逐字「「两块盘都带上新 F 之后才回收」在 X8-A 那一格上同样卡死」按字面触发；按反向接受条款这一格四组一起中，不拿来判它们，病根是 checkpoint 保留池没实现：`28-挂载期承诺量.md:28` 第八项那句「它保证 checkpoint 自己的固定点写得出去（E19（defer 窗口下的假性 ENOSPC）：预留 0 时卡死 199 次）」，步 4 / 步 5 第三轮判决第六节 3 也把 X8-A 压在这上面。V3 那一句「进程发不出固定点而空闲报着」只 F-扣 字面全中（G7 的第 2 项不报它们；D16（发布语义） 已定项 1 的 `df` 把「已释放的」算进来，两组的 `df` 都会报，推的）。

### 2.5 Y1 并表（已知打中，没重跑）

步 6 checker 第一轮判决第四节 Y1（`m2-step6-checker-r1-main-verification.md:42`）：checker 候选集下界取最新根自己带的 F，一块盘上的载体根坏掉之后 F_生效 回落，更早的根按条款仍是回退候选、它们的单元已被合法复用，I-7.4 / I-4.8 全绿。按取 F 的方式并进四组：F-扣、G7′ 漏判；G7、F-扣′ 的 checker 现算 F_生效，按那一格的机理会红（推的，没在本腿装置上重做）。

### 2.6 Z2′ 四组一览

| 组 | 崩溃态 txg 14（2.1） | 崩溃态后重开 / 回退（2.1） | 坏镜像扣住位被撤掉（2.2） | 推进一格重发 + 今天的循环上限（2.3） | X8-A（2.4） | Y1（2.5） |
|---|---|---|---|---|---|---|
| G7 | 绿 | 绿 / 绿 | **红**（五条） | 不回收，0 次撞槽 | 卡死（空闲报 0） | 红（推的） |
| G7′ | **I-3.1 红**（V1） | 绿 / 绿 | 绿（V4） | 同 G7 | 卡死 | 绿（V4） |
| F-扣 | 绿 | 绿 / 绿 | 绿（V4） | **生效前发出候选根引用的槽，checker 绿**（V2，推的） | 卡死（空闲报 64） | 绿（V4） |
| F-扣′ | **I-3.1 红**（V1） | 绿 / 绿 | 红 | 同 F-扣，checker 红 | 卡死 | 红 |

G7 在这一轮本腿造的格上没有单独输；它的记账口径与甲-T1 同一个时点，所以 1.3 的洞它一样暴露；与甲-T1 的挂载内回收合用时，1.2 / 3.2 的影子账那一格也一样（都不分辨，不在这张表里）。

## 三、Z4′ G5 与第九项

主句整句（`23-journal的角色与格式.md:1209` 里那一句，到句号为止）：

```
被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账、只隔离其中**只被被抛弃根引用**的槽——查账的集合 2026-09-16 用户定案取窄读法：按主语「被抛弃时间线的根」读，仍被有效根引用的槽不在其内；宽读法（凡被环里任一可读根引用过的槽都隔离）会让抬 F 在第一版 24 槽里买不到任何东西，里程碑「第二个事务」三方第一轮攻方腿指出两种读法（`research/prompts/m2-r1-main-verification.md` 第三节）（影子账；2026-09-13 用户定案，C314（回退可以复用被抛弃的根引用的单元） 取 F-A，E150（回退复用被抛弃的根引用的单元） 两格零违例、多隔离 2–3 个单元到被抛弃的最新根被轮转覆写为止；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。
```

G5 在今天的实现里的形态：`mount.rs:213` 那个函数，豁免集 = 当前账里未释放的落点 ∪ 候选根（按最新根实例表有效 ∧ txg ≥ floor）引用的落点，隔离「被抛弃根引用 ∧ 不在豁免集」的槽；在 `rebuilt_allocator`（挂载与回退，floor = F_生效）与 `raise_rollback_floor`（floor = 新 F）两处调用；隔离位只置不清（`mount.rs:208` 文档注释整行 `` /// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销。 ``）。

**第九项那几列怎么算**（`probe.rs` 的 `ninth_item_line`，全部从盘上现算，盘 0，单位槽）：`candidates` 候选根 txg；`abandoned` 环里按最新根实例表判被抛弃的根；`below_F` 有效而 txg < F_生效 的根；`orig` 原式——`28-挂载期承诺量.md:30` 第九项「上界 = 被抛弃根的已分配统计量 − R_old 的已分配统计量」，条款与里程碑步 4 那一句（`02-second-txn.md:170`「上界 = 被抛弃根的已分配 − A 的已分配」）都没写环里有多条被抛弃根时取哪一条的已分配，本腿取回退那一刻环里最新的那条（参数没绑，见 3.4），两个数取各自那一版记账行的第 1 项，那条被抛弃根离开环之后标 `→cleared0`；`set_reading` 改式按集合读：|∪ 环里被抛弃根引用的槽 \ ∪ 候选根引用的槽|；`g5_def` 再减去当前账里未释放的；`count_reading` 改式按数读：max(0, |∪被抛弃根引用| − |∪候选根引用|)；`narrow_by_valid` 用户措辞「仍被有效根引用的槽不在其内」按「有效只看实例表」读出的集合；`G5_isolated` 分配器此刻真隔离着的槽；`stale_isolated` 隔离着而环里已没有被抛弃根引用的；`missing_isolation` 按 `g5_def` 该隔离而分配器没隔离的；`lowest_user_slot` 这次发布之后、下一次回收之前的最低可发用户数据槽。「引用」取那条根那一版分配记录里未释放的落点——第 0 代树表的根取不到（1.4 ①），这几列在 txg ≤ 2 的根上少算 mkfs 树表那 1 槽，不影响下面几张表（那几段里被抛弃根都不是第 0 代根）。

### 3.1 两种历史各一张逐次发布表（甲-T1 + G5，产物 `outputs/table-*.md` 由 `extract-tables.py` 生成）

**两次回退，第二次回退到更早的 (1,4)**（`outputs/table-h4-earlier-T1.md` 整份）。前 4 行是第一段时间线里 A(3) 之后的覆盖写 4–7，还没有回退，没有第九项；「—」表示那一行没有第九项那几列：

| 事件 | txg | 实例 | 根带的 F / F_生效 | 候选集 txg | 环里被抛弃根 txg | F 之下的有效根 | G5 隔离数 | 改式·集合差 | 改式·数值差 | 原式（被抛弃最新根 − R_old） | 窄读（有效只按实例表） | 陈旧隔离槽数 | 该隔离而没隔离 | 最低可发用户槽 | 这次写撞被抛弃根的槽 | checker |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| first-timeline | 4 | 1 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| first-timeline | 5 | 1 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| first-timeline | 6 | 1 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| first-timeline | 7 | 1 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| after-rollback-1 | 10 | 2 | 0/0 | [0-5,8-10] | [6-7] | [] | 20 | 20 | 0 | 20 | 20 | 0 | [] | 50190 | — | 绿 |
| second-timeline | 11 | 2 | 0/0 | [0-5,8-11] | [6-7] | [] | 20 | 20 | 0 | 20 | 20 | 0 | [] | 50192 | - | 绿 |
| second-timeline | 12 | 2 | 0/0 | [0-5,8-12] | [6-7] | [] | 20 | 20 | 0 | 20 | 20 | 0 | [] | 50194 | - | 绿 |
| after-rollback-2 | 14 | 3 | 0/0 | [0-4,13-14] | [5-12] | [] | 64 | 64 | 34 | 44 | 64 | 0 | [] | 50194 | — | 绿 |
| overwrite | 15 | 3 | 0/0 | [0-4,13-15] | [5-12] | [] | 64 | 64 | 24 | 44 | 64 | 0 | [] | 50196 | - | 绿 |
| overwrite | 16 | 3 | 0/0 | [0-4,13-16] | [5-12] | [] | 64 | 64 | 14 | 44 | 64 | 0 | [] | 50198 | - | 绿 |
| overwrite | 17 | 3 | 0/0 | [0-4,13-17] | [5-12] | [] | 64 | 64 | 4 | 44 | 64 | 0 | [] | 50200 | - | 绿 |
| overwrite | 18 | 3 | 0/0 | [0-4,13-18] | [5-12] | [] | 64 | 64 | 0 | 44 | 64 | 0 | [] | 50202 | - | 绿 |
| overwrite | 19 | 3 | 0/0 | [0-4,13-19] | [5-12] | [] | 64 | 64 | 0 | 44 | 64 | 0 | [] | 50204 | - | 绿 |
| overwrite | 20 | 3 | 0/0 | [0-4,13-20] | [5-12] | [] | 64 | 64 | 0 | 44 | 64 | 0 | [] | 50206 | - | 绿 |
| overwrite | 21 | 3 | 0/0 | [0-4,13-21] | [5-12] | [] | 64 | 64 | 0 | 44 | 64 | 0 | [] | 50208 | - | 绿 |
| overwrite | 22 | 3 | 0/0 | [0-4,13-22] | [5-12] | [] | 64 | 64 | 0 | 44 | 64 | 0 | [] | 50210 | - | 绿 |
| overwrite | 23 | 3 | 0/0 | [0-4,13-23] | [5-12] | [] | 64 | 64 | 0 | 44 | 64 | 0 | [] | 50212 | - | 绿 |
| overwrite | 24 | 3 | 0/0 | [1-4,13-24] | [5-12] | [] | 64 | 64 | 0 | 44 | 64 | 0 | [] | 50214 | - | 绿 |
| overwrite | 25 | 3 | 0/0 | [2-4,13-25] | [5-12] | [] | 64 | 64 | 0 | 44 | 64 | 0 | [] | 50216 | - | 绿 |
| overwrite | 26 | 3 | 0/0 | [3-4,13-26] | [5-12] | [] | 64 | 64 | 0 | 44 | 64 | 0 | [] | 50218 | - | 绿 |
| overwrite | 27 | 3 | 0/0 | [4,13-27] | [5-12] | [] | 64 | 64 | 0 | 44 | 64 | 0 | [] | 50218 | - | 绿 |
| overwrite | 28 | 3 | 0/0 | [13-28] | [5-12] | [] | 64 | 66 | 0 | 44 | 66 | 0 | [50176-50177] | 50218 | - | 绿 |
| overwrite | 29 | 3 | 0/0 | [13-29] | [6-12] | [] | 64 | 60 | 0 | 44 | 60 | 4 | [] | 50218 | (1,6)durable[50176-50177](1,7)durable[50176-50177](1,5)window[50176-50177] | 绿 |
| overwrite | 30 | 3 | 0/0 | [13-30] | [7-12] | [] | 64 | 50 | 0 | 44 | 50 | 14 | [] | 50220 | - | 绿 |
| overwrite | 31 | 3 | 0/0 | [13-31] | [8-12] | [] | 64 | 40 | 0 | 44 | 40 | 24 | [] | 50222 | - | 绿 |
| overwrite | 32 | 3 | 0/0 | [13-32] | [9-12] | [] | 64 | 36 | 0 | 44 | 36 | 28 | [] | 50224 | - | 绿 |
| overwrite | 33 | 3 | 0/0 | [13-33] | [10-12] | [] | 64 | 32 | 0 | 44 | 32 | 32 | [] | 50226 | - | 绿 |
| overwrite | 34 | 3 | 0/0 | [13-34] | [11-12] | [] | 64 | 22 | 0 | 44 | 22 | 42 | [] | 50228 | - | 绿 |
| overwrite | 35 | 3 | 0/0 | [13-35] | [12] | [] | 64 | 12 | 0 | 44 | 12 | 52 | [] | 50230 | - | 绿 |
| overwrite | 36 | 3 | 0/0 | [13-36] | [] | [] | 64 | 0 | 0 | 44→cleared0 | 0 | 64 | [] | 50232 | - | 绿 |
| overwrite | 37 | 3 | 0/0 | [14-37] | [] | [] | 64 | 0 | 0 | 44→cleared0 | 0 | 64 | [] | 50234 | - | 绿 |
| overwrite | 38 | 3 | 0/0 | [15-38] | [] | [] | 64 | 0 | 0 | 44→cleared0 | 0 | 64 | [] | 50236 | - | 绿 |
| overwrite | 39 | 3 | 0/0 | [16-39] | [] | [] | 64 | 0 | 0 | 44→cleared0 | 0 | 64 | [] | 50236 | - | 绿 |
| overwrite | 40 | 3 | 0/0 | [17-40] | [] | [] | 64 | 0 | 0 | 44→cleared0 | 0 | 64 | [] | 50236 | - | 绿 |

**第二次回退到更晚的 (2,11)**（`outputs/table-h4-later-T1.md`）只抄与上表不同形状的四行，全表在产物里：

| 事件 | txg | 实例 | 根带的 F / F_生效 | 候选集 txg | 环里被抛弃根 txg | F 之下的有效根 | G5 隔离数 | 改式·集合差 | 改式·数值差 | 原式（被抛弃最新根 − R_old） | 窄读（有效只按实例表） | 陈旧隔离槽数 | 该隔离而没隔离 | 最低可发用户槽 | 这次写撞被抛弃根的槽 | checker |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| after-rollback-2 | 14 | 3 | 0/0 | [0-5,8-11,13-14] | [6-7,12] | [] | 30 | 30 | 0 | 10 | 30 | 0 | [] | 50194 | — | 绿 |
| overwrite | 29 | 3 | 0/0 | [8-11,13-29] | [6-7,12] | [] | 30 | 32 | 0 | 10 | 32 | 0 | [50176-50177] | 50218 | - | 绿 |
| overwrite | 30 | 3 | 0/0 | [8-11,13-30] | [7,12] | [] | 30 | 20 | 0 | 10 | 20 | 10 | [] | 50218 | (1,6)window[50176-50177](1,7)durable[50176-50177] | 绿 |
| overwrite | 31 | 3 | 0/0 | [8-11,13-31] | [12] | [] | 30 | 10 | 0 | 10 | 10 | 20 | [] | 50220 | - | 绿 |

### 3.2 两次回退：被抛弃根引用的槽被发出去（与 1.2 同一个机理）

两支都中，形状与 1.2 相同，释放 m1 的那次写行不同：更早那一支是第二次回退的写行（txg 13）把 (1,4) 的实例表 m1 换掉，豁免它的是候选根 0–4，被抛弃的 (1,5)–(1,7) 同实例照抄它；更晚那一支是第一次回退的写行（txg 8）把 m1 换掉，第二次回退重算影子账时候选根 0–5 仍引用它、照样豁免，被抛弃的 (1,6)(1,7) 照抄它。整行：

```
outputs/h4-earlier-T1.txt:41:H4(1,4) overwrite txg=29 inst=3 F_root=0 F_eff=0 floor_now=13 acct_alloc=166 acct_free=211802 acct_defer=154 mem_alloc=166 mem_defer=154 mem_free=211802 iso=64 held=0 wrote=[50176-50177,50491-50498] hit_abandoned=(1,6)durable[50176-50177](1,7)durable[50176-50177](1,5)window[50176-50177] hit_candidate=- | GREEN
outputs/h4-later-T1.txt:43:H4(2,11) overwrite txg=30 inst=3 F_root=0 F_eff=0 floor_now=8 acct_alloc=200 acct_free=211768 acct_defer=188 mem_alloc=200 mem_defer=188 mem_free=211768 iso=30 held=0 wrote=[50176-50177,50499-50506] hit_abandoned=(1,6)window[50176-50177](1,7)durable[50176-50177] hit_candidate=- | GREEN
```

同一段历史 T0 与 T1-G5turnover 的产物 `grep -c 'hit_abandoned=('` 都是 0（`outputs/h4-{earlier,later}-{T0,T1-G5turnover}.txt`）；T0 从 txg 26 起 I-3.1 红（两份各 15 个状态行，第二轮 Z1-0 的形状）。四句同 1.2：① 分辨 G5 与「G5 + 转环重算」；不分辨挂载内回收的 Z1′ 活臂；② 看得到；③ V2 后半；④ 第九项改式不碰这一格。

### 3.3 回退之前已经抬过 F（回退目标在 F 之上）：没打中主句

`outputs/table-h4c-T1.md` 整份：

| 事件 | txg | 实例 | 根带的 F / F_生效 | 候选集 txg | 环里被抛弃根 txg | F 之下的有效根 | G5 隔离数 | 改式·集合差 | 改式·数值差 | 原式（被抛弃最新根 − R_old） | 窄读（有效只按实例表） | 陈旧隔离槽数 | 该隔离而没隔离 | 最低可发用户槽 | 这次写撞被抛弃根的槽 | checker |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| inst1 | 4 | 1 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| inst1 | 5 | 1 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| inst1 | 6 | 1 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| inst1 | 7 | 1 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| inst1 | 8 | 1 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| inst1 | 9 | 1 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| remount(row+warm) | 11 | 2 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | — | 绿 |
| inst2 | 12 | 2 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| inst2 | 13 | 2 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| inst2 | 14 | 2 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| inst2 | 15 | 2 | 0/0 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| after-raise | 17 | 2 | 12/12 | [12-17] | [] | [0-11] | 0 | 0 | 0 | - | 0 | 0 | [] | 50176 | — | 绿 |
| abandoned-to-be | 18 | 2 | 12/12 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| abandoned-to-be | 19 | 2 | 12/12 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| abandoned-to-be | 20 | 2 | 12/12 | — | — | — | — | — | — | — | — | — | — | — | - | 绿 |
| after-rollback | 22 | 3 | 12/12 | [12-14,21-22] | [15-20] | [0-11] | 48 | 48 | 8 | -33 | 44 | 0 | [] | 50182 | — | 绿 |
| overwrite | 23 | 3 | 12/12 | [12-14,21-23] | [15-20] | [0-11] | 48 | 48 | 0 | -33 | 44 | 0 | [] | 50184 | - | 绿 |
| overwrite | 24 | 3 | 12/12 | [12-14,21-24] | [15-20] | [1-11] | 48 | 48 | 0 | -33 | 44 | 0 | [] | 50186 | - | 绿 |
| overwrite | 25 | 3 | 12/12 | [12-14,21-25] | [15-20] | [2-11] | 48 | 48 | 0 | -33 | 44 | 0 | [] | 50188 | - | 绿 |
| overwrite | 26 | 3 | 12/12 | [12-14,21-26] | [15-20] | [3-11] | 48 | 48 | 0 | -33 | 44 | 0 | [] | 50190 | - | 绿 |
| overwrite | 27 | 3 | 12/12 | [12-14,21-27] | [15-20] | [4-11] | 48 | 48 | 0 | -33 | 44 | 0 | [] | 50192 | - | 绿 |
| overwrite | 28 | 3 | 12/12 | [12-14,21-28] | [15-20] | [5-11] | 48 | 48 | 0 | -33 | 46 | 0 | [] | 50202 | - | 绿 |
| overwrite | 29 | 3 | 12/12 | [12-14,21-29] | [15-20] | [6-11] | 48 | 48 | 0 | -33 | 46 | 0 | [] | 50204 | - | 绿 |
| overwrite | 30 | 3 | 12/12 | [12-14,21-30] | [15-20] | [7-11] | 48 | 48 | 0 | -33 | 46 | 0 | [] | 50206 | - | 绿 |
| after-raise | 32 | 3 | 27/27 | [27-32] | [15-20] | [9-14,21-26] | 50 | 50 | 0 | -33 | 46 | 0 | [] | 50182 | — | 绿 |
| overwrite | 33 | 3 | 27/27 | [27-33] | [15-20] | [10-14,21-26] | 50 | 50 | 0 | -33 | 48 | 0 | [] | 50184 | - | 绿 |
| overwrite | 34 | 3 | 27/27 | [27-34] | [15-20] | [11-14,21-26] | 50 | 50 | 0 | -33 | 48 | 0 | [] | 50186 | - | 绿 |
| overwrite | 35 | 3 | 27/27 | [27-35] | [15-20] | [12-14,21-26] | 50 | 50 | 0 | -33 | 48 | 0 | [] | 50188 | - | 绿 |
| overwrite | 36 | 3 | 27/27 | [27-36] | [15-20] | [13-14,21-26] | 50 | 50 | 0 | -33 | 48 | 0 | [] | 50194 | - | 绿 |
| overwrite | 37 | 3 | 27/27 | [27-37] | [15-20] | [14,21-26] | 50 | 50 | 0 | -33 | 48 | 0 | [] | 50196 | - | 绿 |
| overwrite | 38 | 3 | 27/27 | [27-38] | [15-20] | [21-26] | 50 | 50 | 0 | -33 | 50 | 0 | [] | 50198 | - | 绿 |
| overwrite | 39 | 3 | 27/27 | [27-39] | [16-20] | [21-26] | 50 | 46 | 0 | -33 | 46 | 4 | [] | 50206 | - | 绿 |
| overwrite | 40 | 3 | 27/27 | [27-40] | [17-20] | [21-26] | 50 | 42 | 0 | -33 | 42 | 8 | [] | 50208 | - | 绿 |
| overwrite | 41 | 3 | 27/27 | [27-41] | [18-20] | [21-26] | 50 | 32 | 0 | -33 | 32 | 18 | [] | 50210 | - | 绿 |
| overwrite | 42 | 3 | 27/27 | [27-42] | [19-20] | [21-26] | 50 | 22 | 0 | -33 | 22 | 28 | [] | 50212 | - | 绿 |
| overwrite | 43 | 3 | 27/27 | [27-43] | [20] | [21-26] | 50 | 12 | 0 | -33 | 12 | 38 | [] | 50214 | - | 绿 |
| overwrite | 44 | 3 | 27/27 | [27-44] | [] | [21-26] | 50 | 0 | 0 | -33→cleared0 | 0 | 50 | [] | 50216 | - | 绿 |
| overwrite | 45 | 3 | 27/27 | [27-45] | [] | [22-26] | 50 | 0 | 0 | -33→cleared0 | 0 | 50 | [] | 50218 | - | 绿 |
| overwrite | 46 | 3 | 27/27 | [27-46] | [] | [23-26] | 50 | 0 | 0 | -33→cleared0 | 0 | 50 | [] | 50220 | - | 绿 |

H4c 里三份形态（T0、T1、T1-G5turnover）`hit_abandoned=(` 都是 0，状态行全绿。抬 F 的那两次（txg 16–17、31–32）都在回收之前按新 F 重算了影子账，而回退目标 (2,14) 的账里 m1 早在 txg 10 的写行被换掉、抬 F 到 12 时（第一条带新 F 的根 txg 16 之前）回收并扣住，被抛弃时间线的 txg 18 把数据写在它上面（回退之后 G5 隔离集里的 50176–50181 就是这一族），没有 1.2 那种「回退那一刻被候选根豁免、之后豁免失效」的槽。

这张表同时显出两件与主句无关的事：① 窄读（「有效」只按实例表）比 G5 少 4 槽（txg 22–27：44 对 48），那 4 槽被 F 之下的有效根与被抛弃根一起引用——按用户措辞的第一种读法它们被豁免、会被回收发出，G5 隔离它们；这是第一、二轮「窄读法原文」那一格在「回退之前抬过 F」上的数，不另记。② F_生效 在回退之后仍是 12：被抛弃的抬 F 空发布（16、17）带着 F = 12 算进 `recovery.rs:351` 的「各幸存盘所带 F 最大值的最小值」，回退目标 (2,14) 本身在两次抬 F 之间；这一格不违反任何一条（新实例的根也写 12），是第一轮账 5 P1「数哪些根的 F」在回退上的样子。

### 3.4 第九项：原式在两种历史上都对不上；改式按集合读才与 G5 同数，但它不再是「不遍历树」的量

回退与抬 F 那几个时刻（影子账刚重算）的读数，出自各份产物的 `ninth` 行（`grep -h "after-rollback ninth\|after-rollback-1 ninth\|after-rollback-2 ninth\|after-raise ninth txg=32"`）：

| 历史 | 时刻 | G5 隔离数 | 改式·集合差 | 改式·数值差 | 原式 | 窄读（有效只按实例表） |
|---|---|---|---|---|---|---|
| H1 回退到 (1,3) | txg 10 | 50 | 50 | 30 | 50 = (1,8) − (1,3) | 50 |
| H1m 回退到 (1,3)（里程碑形状） | txg 10 | 34 | 34 | 14 | 34 = (2,8) − (1,3) | 34 |
| H4 第一次回退到 (1,5) | txg 10 | 20 | 20 | 0 | 20 = (1,7) − (1,5) | 20 |
| H4 第二次回退到更早的 (1,4) | txg 14 | 64 | 64 | 34 | **44** = (2,12) − (1,4) | 64 |
| H4 第二次回退到更晚的 (2,11) | txg 14 | 30 | 30 | 0 | **10** = (2,12) − (2,11) | 30 |
| H4c 回退到 (2,14)，回退之前抬过 F 到 12 | txg 22 | 48 | 48 | 8 | **−33** = (2,20) − (2,14) | 44 |
| H4c 回退之后再抬 F 到 27（越过 R_old 14） | txg 32 | 50 | 50 | 0 | **−33**（原式回退那一刻算定，不随抬 F 重算） | 46 |

读法：
- **原式**只在「一次回退、回退之前没抬过 F」（H1、H1m、H4 第一次）上与 G5 同数。两次回退差 20：第二次回退时「被抛弃的最新根」(2,12) 的账里没有第一次被抛弃的 (1,6)(1,7) 独占的那 20 槽（它们在 (2,12) 的时间线里从没分配过），而它们仍在环里、仍被 G5 隔离。回退之前抬过 F 是 −33 对 48：被抛弃的 (2,20) 那一版的已分配是抬 F 回收之后写的（抬 F 回收了 66 个落点，`outputs/h4c-T1.txt` 第 12 行整行 `H4c raise F wanted=12 ceiling=12 used=12 publishes=[16, 17] reclaimed=66`），而 R_old (2,14) 那一版是回收之前写的，两个已分配之差连符号都不对。步 4 / 步 5 第三轮判决第六节决策点第 1 行（`m2-step45-code-r3-main-verification.md:64`）写的「F 越过 R_old 之后公式比独占集少 2（A 也引用的 mkfs 实例表）」「净释放多于分配时公式为负」，是这两种形状里较温和的那一种。**原式对不上的触发不需要 F 越过 R_old**。
- 原式还有一个没绑的参数：环里有多条被抛弃根时「被抛弃根的已分配」取哪一条（`28-挂载期承诺量.md:30` 与 `02-second-txn.md:170` 都没写）。本腿取最新的；取最大、取和、取「每条减 R_old 再取最大」各给不同的数，没逐个量。
- **改式按集合读**（「被抛弃根引用的槽 − 候选集里的根引用的槽」的集合差再数槽）在上表每个重算时刻都与 G5 的隔离数相等（再减当前账里未释放的 `g5_def` 也相等）；**按数值读**（两个并集各数槽再相减、下界 0）不是同一个量（64 对 34、48 对 8），「下界 0」只对数值读法有意义。
- 集合读法算得出来要读每条被抛弃根与每条候选根的分配记录树逐槽求并，它就是 `mount.rs:213` 那个函数；`28-挂载期承诺量.md:30` 原式那半句「两个数各从那条根可达的记账树根读、不遍历树、不加盘上字段」对它不再成立。第九项若改成集合读，那一句要跟着改，`05-快照-空间记账机制.md:351` 已定项 4「式子逐项对照」表里「被抛弃根独占量」那一行（「两个数各从那条根可达的记账树根读，不遍历树」）也一样（V5 连带，推的）。

四句（原式那一格）：① 分辨：集合读法同一时刻同数；② 看得到：写者回退时两种都算得出；③ 正文 Z4′ 推翻观测「第九项与隔离数对不上」字面触发；④ 改式按集合读在这几格上不中，按数值读仍中。

### 3.5 同一次挂载里转环之后：隔离不清零，第九项与 G5 又对不上（V3 方向，推的）

`28-挂载期承诺量.md:30` 第九项那句写「被抛弃的根被轮转覆写时清零」，`02-second-txn.md:172` 整行：`- 「被抛弃根独占量」在被抛弃的根被轮转覆写时清零（D28（挂载期承诺量） 已定项 1 第九项逐字），影子账隔离到被抛弃的最新根被轮转覆写为止（D23（journal 的角色与格式） 已定项 14 逐字）——第一版 24 个根槽，这个里程碑的脚本走不到那一刻，被抛弃时间线独占的两个单元一直隔离着。`。探针的脚本走到了那一刻：

| 历史 | 被抛弃根全部离开环之后的第一个状态 | 改式·集合差 | 原式按字面 | G5 隔离数（分配器真扣着的） |
|---|---|---|---|---|
| H1（T1） | txg 32 | 0 | 0（cleared） | 50 |
| H4 更早（T1） | txg 36 | 0 | 0 | 64 |
| H4 更晚（T1） | txg 36 | 0 | 0 | 30 |
| H4c（T1） | txg 44 | 0 | 0 | 50 |

隔离位只置不清（`mount.rs:208`），下一次挂载的 `rebuilt_allocator` 才重新算。本腿提的「G5 + 转环重算」同样只置不清（它调的是同一个函数，`outputs/h1-T1-G5turnover.txt` 第 51 行 txg 32 `G5_isolated=52` 而 `set_reading=0`）。步 4 的验收那一句（`02-second-txn.md:179` 整行 `- 准入读数：可用比回退前少了正好被隔离的那几块（C318（影子账隔离的单元没进准入不等式） 的判别力自证：把第九项置 0 必须多报正好那几块）。`）把第九项接到隔离数上；接上之后，被抛弃根离开环到下一次挂载之间，准入多扣 30–64 槽，`df` 少报同样多。反过来按原式（回退那一刻算定、离开环时清零）接准入，H4c 那一段从回退到 txg 43 都是 −33，比真扣着的 48–50 少扣 81–83 槽，`df` 多报——那是 V3 第一句「`df` 报出 ≥ s 而写 s 失败」的形状。两种接法各错一个方向；准入今天没实现（正文第三节 grep 零命中），这一格是推的。

## 四、Z5′ 攻方一侧：G8 能不能被一份坏镜像骗过

G8 在探针 checker 里的实现（`patch-copy.py` 的 `SFPROBE_G8`，`walk.rs` 副本）：从最新根走读时顺手收它的分配记录树条目（今天的 `walk.rs:298` 那一支只判层级、不解条目），门槛 = max(checker 用的 F, 按最新根实例表判有效的根的最小 txg)，逐盘数「已释放 ∧ 释放代 > 门槛」的跨度，与记账第 5 项比。同一处另加本腿的 G8′：逐盘判「第 1 项 − 第 5 项 == 最新根走读得到的引用」（被攻过零轮，第六节）。两条都登记进副本 `image.rs` 的清单（`IMPLEMENTED_INVARIANTS` 26 → 28），环境变量不设时不判。

### 4.1 坏镜像：活单元被记成已释放，运行时与 G8 一起错，checker 与 G8 全绿（打中）

造法（`SFPROBE_BUG_RELEASE_CARRIED_INSTANCE_TABLE_AT=5`，改的是写者）：txg 5 那次覆盖写照抄实例表（`transaction.rs:899` `instance_table: InstanceTablePlan::Carry(previous.root.instance_table),`），却把实例表单元 m1 的落点也放进释放表。于是分配记录把 m1 改写成「已释放、释放代 5」，第 5 项 +2，第 1 项不变（甲家的第 1 项含 defer），而最新根仍经实例表指针引用 m1。这是「释放表算错了一个角色」这一类写者错误，不需要故障。记账 T1、分配器 T1，覆盖写到 31。

逐 txg 判红的名字（`outputs/h6-z5-bug-at5-T1-G8.txt` 每个状态行 `|` 之后取不变量名去重；txg 3、4 为 `GREEN`）：

| txg | 判红 |
|---|---|
| 5–27（23 个状态） | 只有 G8′（`G8p`） |
| 28 | G8′、I-3.1 |
| 29 | I-2.1、I-4.8、I-7.2、I-7.4 |
| 30–31 | G8′、I-2.1、I-4.8、I-7.2、I-7.4 |

同一份第 3、25、26、27 行整行：

```
H6 overwrite txg=5 inst=1 F_root=0 F_eff=0 floor_now=0 acct_alloc=33 acct_free=211935 acct_defer=23 mem_alloc=33 mem_defer=23 mem_free=211935 iso=0 held=0 wrote=[50184-50185,50257-50264] hit_abandoned=- hit_candidate=- | G8p[盘 0：第 1 项 Some(540672) − 第 5 项 Some(376832) ≠ 最新根走读 196608]
H6 overwrite txg=27 inst=1 F_root=0 F_eff=0 floor_now=4 acct_alloc=242 acct_free=211726 acct_defer=232 mem_alloc=252 mem_defer=242 mem_free=211716 iso=0 held=0 wrote=[50178-50179,50433-50440] hit_abandoned=- hit_candidate=- | G8p[盘 0：第 1 项 Some(3964928) − 第 5 项 Some(3801088) ≠ 最新根走读 196608]
H6 overwrite txg=28 inst=1 F_root=0 F_eff=0 floor_now=5 acct_alloc=240 acct_free=211728 acct_defer=230 mem_alloc=252 mem_defer=242 mem_free=211716 iso=0 held=0 wrote=[50180-50181,50441-50448] hit_abandoned=- hit_candidate=- | G8p[盘 0：第 1 项 Some(3932160) − 第 5 项 Some(3768320) ≠ 最新根走读 196608] ; I-3.1[盘 0：记账的已分配 Some(3932160)，遍历全部有效根得到 3964928]
H6 overwrite txg=29 inst=1 F_root=0 F_eff=0 floor_now=6 acct_alloc=240 acct_free=211728 acct_defer=230 mem_alloc=250 mem_defer=240 mem_free=211718 iso=0 held=0 wrote=[50176-50177,50449-50456] hit_abandoned=- hit_candidate=- | I-2.1[实例表单元 在盘 0 槽 50176 的那一份与位置条目里的校验和对不上] ; I-4.8[最新根（txg 29）出发的遍历有单元对不上或读不出] ; I-7.2[最新的根走不完：实例表单元两份都读不到对得上的] ; I-7.4[最新根（txg 29）引用的单元已被复用或抹头（校验和对不上或头用不了）]
```

txg 5–27：G8 数的是分配记录里「已释放 ∧ 释放代 > 门槛」，m1 那条就在里面（释放代 5 > 门槛 0–4），第 5 项也含它，两边相等；I-3.1 的第 1 项含 defer，走读并集含 m1，也相等。txg 28 持久之后环里最旧有效根变成 5，记账按 T1 把 m1 当成可再分配扣掉，而最新根还引用它，I-3.1 才红——这一刻 G8 两边又一起把 m1 排除，照样绿。txg 29 分配器把 m1 当最低空闲偶数槽对发给数据单元，实例表被盖。对照 `outputs/h6-z5-control-T1-G8.txt`（不注入）29 个状态行全部 `GREEN`。

四句：① 分辨：G8′ 在 txg 5–27 每个状态上红，G8 全绿；② checker 看得到判别它的东西：最新根走读得到的引用集与第 1、5 项都在镜像里；③ V4「某份坏镜像全部检查判绿」（G8 算进「全部检查」）；④ 正文 Z5′ 倾向写的推翻观测逐字「一份坏镜像让运行时与 G8 一起错、全部检查全绿」，按字面触发。

**为什么 G8 骗得过**：运行时的第 5 项 = 分配器在释放时加、在谓词成立时减；G8 = 对同一批分配记录按同一个谓词数一遍。两边的输入（记录里的已释放位与释放代）与谓词（`16-发布语义.md:371`）相同，G8 能核的只有「加减做没做漏」，核不到「记录本身记对没有」。`.claude/rules/fs-design.md:36` 整行：`第二条不依赖任何关于性能的假设——它要求的恰恰是**运行时与 checker 必须用不同的算法**。`。G8 与运行时代码不共用，但算的是同一个函数、喂的是同一份输入；上面这份镜像就是那条禁令要防的形状（「对照关系当场归零」，`fs-design.md:24`）。这一句是本腿对「冲不冲突」的判断，辩方那一侧归本地辩方腿。

### 4.2 不借用回收谓词的核法：正文举的形式按字面不成立，换成「最新根」就是 G8′

正文举的「defer + 空闲 + 已分配里仍被候选根引用的 == 单元区」在第一个事务上：`outputs/h6-z5-control-T1-G8.txt` 第 1 行整行 `H6 A txg=3 inst=1 F_root=0 F_eff=0 floor_now=0 acct_alloc=13 acct_free=211955 acct_defer=1 mem_alloc=13 mem_defer=1 mem_free=211955 iso=0 held=0  | GREEN`，候选根并集 = 第 1 项 = 13（I-3.1 绿），1 + 211955 + 13 = 211969 ≠ 211968（单元区 211968 槽，`allocator.rs` 单测 `unit_area_of_a_4_gib_image_has_211968_slots_in_3312_segments`）。甲家的第 1 项已含 defer，候选根并集又含 defer 里那些槽，defer 算了两次。换成「最新根引用的」：1 + 211955 + 12 = 211968，在 I-5.2 成立时等价于 G8′「第 1 项 − 第 5 项 == 最新根走读」。

G8′ 在合法历史上的判定（`outputs/g8/*.txt` 八份、`h7-shared-*.txt` 两份、`h6-z5-control-T1-G8.txt`，每份数状态行里 `G8p[` 的次数）：

| 产物 | 状态行 | G8′ 红 | G8 红 |
|---|---|---|---|
| `g8/h1m-T0.txt`（今天的写者，里程碑形状转环） | 24 | 0 | 5（T0 挂载内不回收，第 5 项比谓词多） |
| `g8/h1.txt`（回退之后转环，T1 + G5 转环重算，checker 现算 F_生效） | 36 | 0 | 0 |
| `g8/h4-earlier.txt` / `g8/h4-later.txt`（两次回退，同上） | 34 / 34 | 0 / 0 | 0 / 0 |
| `g8/h4c.txt`（回退之前抬过 F，同上） | 39 | 0 | 0 |
| `g8/z2-g7.txt`（崩在两条带新 F 的根之间、重开、回退，G7） | 10 | 0 | 0 |
| `g8/z2-fkou-writer-checkerNewestF.txt`（同上，F-扣） | 10 | 0 | 0 |
| `g8/h5.txt`（推进一格重发造洞，T1） | 48 | 0 | 0（I-3.1 红 22 个，洞） |
| `h7-shared-2-3` / `h7-shared-1-1`（共享单元 + G7 抬 F） | 68 / 66 | 0 / 0 | 0 / 0 |
| `h6-z5-control-T1-G8.txt` | 29 | 0 | 0 |

合计 398 个状态行，G8′ 0 次红。洞上 G8′ 不红：洞让第 1、5 项一起多出同样的量，差不变。G8′ 抓不到的形状：第 1、5 项一起多报或一起少报同样的量（第二轮 ③′「一直不回收」），那一类甲家的 I-3.1 抓（第二轮 4.1）。所以 G8′ 不替代 I-3.1，只补上「第 5 项自己记错、或记录把活单元记成已释放」这一类；它不读分配记录、不用回收谓词，与运行时的第 5 项是两种算法（被攻过零轮）。

## 五、按反向接受条款记账（主 agent 判，本腿只列）

| 臂 | 在哪一格输一次（最弱读法） | 一起中、不拿来判它的格 |
|---|---|---|
| 甲-T1 | —（1.1 共享单元、1.2 回退之后转环的 I-3.1 都没打中） | 1.3 洞（甲家同形）；1.2 / 3.2 主句（凡挂载内回收的臂与 G5 合用都中） |
| 甲-b′ | 1.4 ①：txg 3–26 之间挂载或回退写出的第 1 项比 checker 少 mkfs 树表那 1 槽（数是量的，「会红」是推的）；1.4 ②：洞上 I-5.2 差 11–231 | 1.2 / 3.2 主句（挂载内按甲-T1 回收） |
| G5（Z4′ 倾向） | 1.2、3.2：同一次挂载里环转过去之后，被抛弃根引用的槽被发出去（与挂载内回收合用；「G5 + 转环重算」与保守读法同一段历史 0 次） | 3.5 陈旧隔离（「G5 + 转环重算」同样只置不清） |
| 第九项原式 | 3.4：两次回退少 20；回退之前抬过 F 是 −33 对 48 | — |
| 第九项改式（按数值读，下界 0） | 3.4：两次回退 34 对 64、回退之前抬过 F 8 对 48 | — |
| 第九项改式（按集合读） | — | 3.5（同一次挂载转环之后与 G5 对不上，病根在隔离位不清） |
| G7（Z2′ 倾向） | — | 1.3 洞（记账时点同甲-T1）；与甲-T1 的挂载内回收合用时 1.2 主句；2.4 X8-A（四组一起卡死） |
| G7′ | 2.1 V1（崩溃态 I-3.1 红）；2.2 V4 | 2.4 |
| F-扣（今天的实现） | 2.2 V4；2.3 V2（推的：条款要求的推进一格重发 + 今天的循环上限）；Y1 V4（已知） | 2.4 |
| F-扣′ | 2.1 V1 | 2.4 |
| G8（Z5′） | 4.1 V4 | — |

共用前提的账（不回落到任何一条臂）：
1. **挂载内回收要影子账跟着候选集走**（1.2、3.2）：「环里最旧有效根」变大与抬 F 一样让候选集缩小，而 G5 的定义只写「抬 F 后重算」。今天的实现（甲-T0）挂载内不回收，所以没暴露；第二轮 Z1-0 要求的挂载内回收一接上，这一格就可达。
2. **隔离位只置不清**（3.5）：与 `28-挂载期承诺量.md:30` 第九项「被抛弃的根被轮转覆写时清零」不一致，任何按隔离数接准入的读法在被抛弃根离开环到下一次挂载之间多扣。
3. **checker 不走被抛弃根**（步 6 checker 第一轮判决第五节 3 的欠账）：1.2、3.2 那几份违反主句的镜像全绿，只有「把回退实例的根抹掉再挂」（1.2 后果那一段）才显出来。
4. **保留池没实现**（2.4）：X8-A 四组一起卡死。
5. **洞**（第二轮账 3）：条款要求的写失败路径（推进一格重发）与静默丢写留下同一个洞（1.3）。
6. **第九项原式的参数没绑**（3.4）：环里有多条被抛弃根时取哪一条的已分配。

## 六、本腿提的改法（每一条都只在本腿的模型上量过、被攻过零轮）

| 编号 | 改法 | 写成对实现的改动 | 本腿量到的 | 在哪几格上不起作用 / 没量 |
|---|---|---|---|---|
| R3-1 | 影子账在每次回收门槛抬高（抬 F，或环里最旧有效根变大）之前按此刻的候选集重算 | 挂载内回收的那一处（今天没有，第二轮 Z1-0 要加）先调 `mount.rs:213` 那个函数，参数同 `rebuilt_allocator`（floor = F_生效，当前账 = 这一版的分配记录） | `outputs/h1-T1-G5turnover.txt`、`h4-{earlier,later}-T1-G5turnover.txt`、`h4c-T1-G5turnover.txt`：`hit_abandoned=(` 0 次；1.2 后果那一段挂上被抛弃根成功 | 每次回收都要读环里每条被抛弃根与候选根的分配记录树，与 `.claude/rules/fs-design.md` 第一格「代价不许随盘容量增长」有张力（推的；也可以回退时读一次、按根离环增量维护，没量）；不修 3.5（只置不清） |
| R3-2 | 隔离位随重算可以清：被抛弃根离开环、或它引用的槽已不在任何环里被抛弃根的账里，就清掉 | `DeviceFreeMap::isolate` 之外加「按集合重设隔离位」，`isolated_slots` 跟着减 | 没实现、没量；3.5 表里的「改式·集合差」是它清完之后应有的数 | 与 R3-1 一起才对得上第九项「清零」 |
| R3-3 | 第九项取集合读法（= R3-1 重算之后的隔离数），删掉原式与「不遍历树」那半句 | `28-挂载期承诺量.md:30`、`05-快照-空间记账机制.md:351` 两处；`mount.rs` 的 `isolated_slots_per_device` 已经是这个量 | 3.4 七个时刻与 G5 同数 | 3.5 要 R3-2 |
| R3-4 | G8′：checker 逐盘判「第 1 项 − 第 5 项 == 最新根走读得到的引用」，等价于「第 5 项 + 第 2 项 + 最新根走读 == 单元区」 | `walk.rs` 走完最新根之后留一份 `references`（副本 `patch-copy.py` 那一处），在 I-3.1 / I-5.2 旁加一条判定；`invariants.md` 立一条 | 398 个合法状态 0 次红；4.1 坏镜像 txg 5–27 每个状态红 | 第 1、5 项同量一起错抓不到（要 I-3.1）；甲-b′ 的第 1 项下洞上会红（推的） |
| R3-5 | 抬 F 的空发布循环只以「每块盘都带上新 F」为出口；截断时不放开扣住 | `mount.rs:477-481` 的循环条件、`mount.rs:506` | `outputs/z2adv-Fkou-nocap.txt`：落满盘 1 之后才放开，0 次撞槽 | 推进一格重发没实现时不可达；X8-A 卡死不管；暖机循环 `mount.rs:583-587` 同形（第七节 1） |

## 七、旁：读代码时看到、不归这一轮判的

1. **暖机循环与抬 F 循环同形**：`mount.rs:583-587` 整段

   ```
       while all_devices
           .iter()
           .any(|identity| !covered.contains(identity))
           && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS
       {
   ```

   按成功次数截断。推进一格重发接进去之后，写行落盘 0（txg ≡ 0 mod 3）、盘 1 上连续两次写失败（N+1、N+4）时，成功的三次 N+2、N+3、N+5 都在盘 0，循环退出而本实例的根没覆盖盘 1（`16-发布语义.md` 已定项 8 的暖机）。按落点公式推的（`target_for_publish` 的区域是 txg mod 3，区域设备 [0, 1, 0]），没跑；k_tol = 2 在射程里。抬 F 循环只要一次失败就够（2.3）。归步 4 / 步 5 那一轮（它第三轮判决第五节 4 那条欠账的同形）。
2. **第 0 代树表的根不在任何分配记录树里**（1.4 ①）：`recovery.rs:373` 已写明给空。影子账与本腿的撞槽检查都靠分配记录判「一条根引用了什么」，在 txg 0–2 那几条根上都少算 mkfs 树表 50178；今天第 0 代根不可能是被抛弃根（回退目标要有文件版本，0–2 恒 ≤ 目标），所以影子账不受影响，只记下来。
3. **回退之后 F_生效 由被抛弃的空发布撑着**（3.3 末）：H4c 里被抛弃的 16、17 带 F = 12，`effective_rollback_floor`（`recovery.rs:351`）把它们算进去。这一格与条款不冲突，是第一轮账 5 P1「数哪些根的 F」在回退上的样子。

## 八、没打中的形状

| 攻的 | 试过的形状 | 取样范围 | 结果 |
|---|---|---|---|
| 甲-T1 × 多根共享单元（V1 / V2） | 可写挂载后覆盖写与空发布按 2:3、1:1 交替，含写行、暖机、G7 抬 F 到上限 | 2 段，134 个状态行，到 txg 72 / 70 | 全绿，0 次撞槽 |
| 甲-T1 × 回退之后照抄 R_old 并转环（V1） | H1（回退到 (1,3) 后覆盖写到 33）、H1m（里程碑形状到 30）、H4 两个变体（到 40）、H4c（到 46） | 29 + 24 + 34 + 34 + 39 = 160 个状态行（T1） | I-3.1 / I-5.2 / I-7.4 / I-4.8 全绿（1.2 的打中在 V2，不在 V1） |
| G5 × 回退之前抬过 F、回退目标在 F 之上（V2） | H4c，T0 / T1 / T1-G5turnover 三种形态 | 各 39 个状态行 | 0 次撞被抛弃根或候选根的槽 |
| G5 × 多扣成假性 ENOSPC（V3） | 准入与 `df` 没实现，只量了陈旧隔离的槽数（3.5） | 4 段历史 | 没法打中「`df` 报出 ≥ s 而写 s 失败」，只给了方向与数 |
| G7 × 合法崩溃态、之后重开、回退、生效之后的 E（V1） | Z2 底，崩在 txg 14 之后 | 10 个状态行 | 全绿 |
| G7 × 推进一格重发（V2） | txg 16（盘 1）写失败换 17；循环有上限 / 无上限 | 7 + 9 个状态行 | 0 次撞槽 |
| G7 × 坏镜像扣住位被撤掉（V4） | 同一段历史回收时不扣住 | 崩溃态 1 个 | 抓到（五条红），不是没打中 G7 的反例 |
| G8′ 误红（V1） | 11 份合法历史（第四节 4.2 那张表） | 398 个状态行 | 0 次 |
| 第九项改式按集合读 × 重算时刻（Z4′ 推翻观测） | 7 个回退 / 抬 F 时刻 | 3.4 表 | 与 G5 同数 |

全部是确定性计算（探针没有随机源），同一份输入跑 N 遍一样，07:14–07:19 UTC 的复跑逐字节相同只说明没有隐藏状态；强度来自对照形态（T0 / G5turnover / conservative、两种 checker）在同一段历史上分得开。

## 九、这条腿自己的限度

- **副本上的数不是入库装置上的数**，要引得由主 agent 在入库装置上重做（`.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」）。
- **四种形态是本腿在副本上写的**，没人审过：T1 的分配器那一半只作用在探针驱动的发布上，`mount_writable` / `mount_rollback` / `raise_rollback_floor` 内部的写行、暖机、抬 F 空发布之前不回收（那几次只分配提交内生块，少回收是保守的方向）；T1 的记账那一半是 `patch-copy.py` 里的 `probe_t1_floor`（与第二轮同一个算法，改成读盘上实例表的兜底）；G7 的「落满之后再回收」、F-扣 的「停在第一条根之后」是 `raise_manual` 照 `mount.rs:400-513` 一步一步重写的，不是直接调 `raise_rollback_floor`。
- **推进一格重发在 crates 里没有**，探针补的：换 txg + 1、jsn + 1、事务号不变、反向链接最后一次成功的那条记录。失败那次的记录留在环里（「只有记录、没有根」）；这种记录链上可写挂载走得通（H5 txg 38 之后那次重开成功），恢复路径的其余判定（前缀、断号）没单独核。
- **甲-b′ 没实现写者**。1.4 的「精确并集」是按正文定义现算的数；「写出来会红」「I-5.2 差多少」是拿这个数代进 `walk.rs:871`、`:878` 两个等式算的。
- **V2 的「撞槽」检查靠分配记录判「一条根引用了什么」**（`referenced_slots`：那一版分配记录里未释放的落点），第 0 代树表的根取不到（第七节 2）；checker 本身不走被抛弃根，所以 V2 那几格 checker 的 `GREEN` 不是证据，证据是撞槽那一列与 1.2 的「抹掉回退实例的根再挂」。
- **只建了两块盘**（区域设备 [0, 1, 0]）；每盘 4 GiB 镜像；只看盘 0 的数（两盘同槽）。
- **历史不是跑前写死的**：判据 V1–V5 是正文跑前写死的；本腿的 H1F（后果）、H7（共享单元）、G8′ 的合法历史批是看了 H1 / H6 第一版读数之后加的，模型第一版跑完之后改过两处（补 `image.rs` 的清单登记、补 H7 与 H1F 两个场景），改完从头重跑，`outputs/` 是改完之后脚本一次跑出来的。
- **层 0 没跑**；准入与 `df` 没实现，V3 那几格只给方向与数（3.5、2.4）。
- **Y1 没在本腿装置上重做**，2.5 按已知判决并表。
- **第九项原式**「被抛弃根」取回退那一刻环里最新的那条；别的取法没量（3.4）。

## 十、没做什么

- 没碰 Z3′；没判正推、辩方那几格；没替主 agent 采纳任何改法；没重攻已知打中的 X8-A、X3（被抛弃根读不出）、Y1。
- 没改 `crates/`、kb、门禁；编译与探针只在 `/tmp/claude-1000/alloc-basis-r3-opus/`（副本、`CARGO_TARGET_DIR` 都在那里，`run-probe.sh` 默认也写那里），一律 `nice -n 19`。写进仓的只有本报告与 `research/prompts/alloc-basis-r3-opus-model/`（`probe.rs`、`patch-copy.py`、`extract-tables.py`、`run-probe.sh`、`crates-at-report/` 六个文件、`outputs/`）；`outputs-rerun/` 复跑核对之后删掉了。
- `.claude/gate.d/stage-owners.tsv` 里没有登记给 `three-way-attack` 的阶段（`awk` 查询输出为空），没跑门禁。
- 开跑前 `ps` 看到：`second_transaction_step_zero_layer0` 测试进程占一个核（100%）、两个 `ray::RayWorkerProc` 各约 28%；本腿没有加并发（两次 `run-probe.sh` 顺序跑，编译各一次）。
- 背景材料只读了正文（第 1–96 行）与附录里的 kb 条款（第 385–1053 行）；小节清单（第 97–384 行）没读，附录后段的三份判决（第 1054 行起）没在背景材料里读，读的是它们的原文件。前两轮判决、第二轮本腿的报告与模型、步 4 / 步 5 第三轮判决、步 6 checker 第一轮判决直接读的原文件。
- 没读禁读清单里的文件（`research/prompts/alloc-basis-r3-sonnet*`、`alloc-basis-r3-local-*`、`alloc-basis-r3-main-verification.md`、`_m2-step45-code-r4-*`、`m2-step45-code-r4-*`）。
- 改自己的报告时只用 `research/scripts/replace-once.py`（每处命中一次、回读确认），模型目录与报告第一段都是排他新建（`set -o noclobber`）。
