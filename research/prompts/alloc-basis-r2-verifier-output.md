# alloc-basis-r2 核查员报告

核查员交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

取 Opus 报告引 `outputs/s1-turnover.txt` 第 23–25 行整行的那条引用（报告第 79-81 行）。
在草稿目录 `/tmp/claude-1000/alloc-basis-r2-verifier/selftest/s1-turnover.txt` 的副本里，
把引用行号从「第 23-25 行」改判成「第 24-26 行」，核对结果：

```
--- 副本文件第 24-26 行（伪造的引用位置） ---
S1 overwrite txg=26 inst=1 F_root=0 ... acct_alloc=243 ... | I-3.1[...Some(3981312)...]
S1 overwrite txg=27 inst=1 F_root=0 ... acct_alloc=253 ... | I-3.1[...Some(4145152)...]
S1 overwrite txg=28 inst=1 F_root=0 ... acct_alloc=263 ... | I-3.1[...Some(4308992)...]
--- 报告里抄的原文（应在 23-25 行） ---
S1 overwrite txg=25 inst=1 F_root=0 ... acct_alloc=233 ... | GREEN
S1 overwrite txg=26 inst=1 F_root=0 ... acct_alloc=243 ... | I-3.1[...Some(3981312)...]
S1 overwrite txg=27 inst=1 F_root=0 ... acct_alloc=253 ... | I-3.1[...Some(4145152)...]
```

行号 24-26 处的三行（txg=26,27,28）与报告实际抄的三行（txg=25,26,27）不是同一组，
逐字比对不上（`txg=` 值、`acct_alloc` 值都不同）。**判定：✗**。
核查方法能分辨伪造的行号，可以继续用于下文的核对。

## 核了什么、怎么核

- 复跑：`z3_gates.py --selftest/--verbose/--sweep` 在独立草稿目录 `/tmp/claude-1000/alloc-basis-r2-verifier/z3-only/` 里用 `nice -n 19` 跑，与 `outputs/` 三份产物逐字节 `cmp`。
- 复跑：把 Opus 模型目录（连同 `crates/`、`Cargo.toml/.lock`）拷进独立草稿目录 `/tmp/claude-1000/alloc-basis-r2-verifier/rerun-repo/`（`git init` 令 `git rev-parse --show-toplevel` 落在草稿目录，不落在真仓），设 `ALLOC_BASIS_R2_DRAFT=/tmp/claude-1000/alloc-basis-r2-verifier/rerun-work` 后 `nice -n 19 bash research/prompts/alloc-basis-r2-opus-model/run-probe.sh`，产物写进草稿目录自己的 `outputs-rerun/`，不碰腿的原目录。此报告写这一段时该复跑仍在跑（见下文「run-probe.sh 复跑状态」一节，会在报告最后一段更新最终结果）。
- 引用核对：文件:行号 + 抄的原文，取该行区间逐字比对；`crates/` 行号按 Opus 报告记录的六个 `git hash-object` 哈希核（`crates-at-report/` 里的版本，以及未改动三份直接用工作区当时匹配哈希的版本），不按核查时工作区的最新版本核（工作区在这一轮里已被别的会话继续改动，逐一核对时另行记录，不算 Opus 报告的错）。
- kb 引用核对：按 kb 文件现在的行号现查；文件若已被并发会话修改，另行注明「分不清：文件在腿交回之后被改过」，不计入 ✗。

## 一、Opus 攻方腿报告（`alloc-basis-r2-opus-output.md`）

### 1.1 文件指纹（sha256）复算，18 处

命令：`sha256sum probe.rs patch-copy.py z3_gates.py run-probe.sh crates-at-report/{allocator,mount,recovery}.rs outputs/SHA256SUMS outputs/{s1-turnover,s1-turnover-remount26,s4l-legal-hit-k1-1-k2-6-f8,p-s4l-legal-hit-k1-1-k2-6-f8-checkerF-effective,s2l-legal-search,p-s2l-legal-search-T1-fsheng-8-16,newcrates-hashes,z3-selftest,z3-results,z3-sweep}.txt`（在 `research/prompts/alloc-basis-r2-opus-model/` 里跑）。

结果：18 个哈希与报告第 21-40 行表格逐一相同（✓ 18）。

### 1.2 `git hash-object` 复算「读的实现」六个 + 新版本六个，12 处

`crates-at-report/{allocator,mount,recovery}.rs` 的 `git hash-object --no-filters` 结果与报告第 27-29、42 行记的 `6bd429ed…`、`34246932…`、`af342120…` 一致（✓ 3）；`outputs/newcrates-hashes.txt` 内容与报告第 42 行记的「另一个会话改了三个」的新哈希（`1eb3f2b0…`、`1ac1348e…`、`ef4da610…`）逐字一致（✓ 1，6 行全比对）；未改动三份（transaction.rs/walk.rs/image.rs）哈希与报告一致（✓ 3）。核查此刻（约 04:47 UTC）工作区里这三个文件的哈希又变了一次（`allocator.rs`→`196f23d9…`、`mount.rs`→`6ae25d56…`，`recovery.rs`→`ef4da610…`，与 newcrates 的 recovery.rs 相同但 allocator/mount 不同）——这是核查期间又一个并发会话继续改的痕迹，不构成对 Opus 报告的 ✗（报告记录的是它自己观测时点的状态，且报告明写了 04:21 UTC 的第一次变化）。

### 1.3 z3_gates.py 独立复跑，逐字节比对

`nice -n 19 python3 z3_gates.py --selftest/--verbose/--sweep`（独立草稿目录，非 Opus 腿的原目录）：
三份产物 `z3-selftest.txt`、`z3-results.txt`、`z3-sweep.txt` 与 `outputs/` 里同名文件 **逐字节相同**（`cmp` 无输出），sha256 与报告表格里记的三个哈希一致（✓ 3）。三份产物末行完成标记 `SELFTEST-COMPLETE`/`RUN-COMPLETE`/`SWEEP-COMPLETE` 都在（✓）。

### 1.4 `run-probe.sh` 整条复跑（草稿目录独立跑）

进行中，见文末最终结果一节；截至本段落写完时仍在跑 `legal-search 20 25`（脚本本身估计约 25 分钟，本机同时有另一会话在跑 `second_transaction_step_zero_layer0` 全量层 0 测试，`nice -19` 之下更慢）。

### 1.5 报告正文引用逐条核（产物整行、`crates/` 行号、kb 行号）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `outputs/s1-turnover.txt` 第 23-25 行整行（1.1） | ✓ | `awk 'NR==23,NR==25'` |
| `outputs/p-s1-turnover-T1.txt` 第 24-25 行整行（1.1） | ✓ | 同上 |
| `outputs/s1-turnover-remount26.txt` 第 25 行、`s1-turnover.txt` 第 30 行、`p-s1-turnover-remount26-T1.txt` 第 25 行（1.2） | ✓（三处全比对） | 同上 |
| `outputs/p-s5-hole-{T0,T1,yiprime-T1}.txt` 第 5、28 行共六处整行（1.3） | ✓ | 同上 |
| `outputs/p-jiaa-jiaa_exact-hole.txt` 第 5 行（1.3） | ✓ | 同上 |
| `outputs/p-s6-yiprime-{asListed,I52}-legal.txt` 第 1 行（1.4） | ✓ | 同上 |
| `invariants.md:167`（I-5.2 整行，报告第 133 行） | ✓（现查今日 kb，行内容一致；kb 文件已被并发改动但该行未变） | `awk 'NR==167'` |
| `crates/mount.rs:234,367`；`allocator.rs:22,526`（报告开头 grep 输出，正文第 70-73 行） | ✓（对 `crates-at-report/` 版本核） | `awk` 取行 |
| `crates/mount.rs:354-356`（2.1，注释三行） | ✓ | 同上 |
| `crates/allocator.rs:112-113`（1.4，两行注释） | ✓ | 同上 |
| `crates/recovery.rs:351`（2.1，函数签名） | ✓ | 同上 |
| `outputs/s4l-legal-hit-k1-1-k2-6-f8.txt` 第 1-6 行整行（2.1） | ✓ | 同上 |
| `outputs/p-s4l-legal-hit-k1-1-k2-6-f8-checkerF-effective.txt` 第 4 行（2.1） | ✓ | 同上 |
| `outputs/s2l-legal-search.txt` 末两行 `SUMMARY cells=12989 clean_hits=258`（2.1） | ✓ | `tail -2` |
| 「打中 138 格、91 格干净崩溃态」（2.1） | ✓（用报告自己给的数法复算：`grep "S2L HIT" | awk '{print $3,$4,$5}' | sort -u | wc -l` = 138；加 `grep last=false` 再去重 = 91） | 见正文，已复算 |
| 「干净 258 行里受害根 (1,3) 240、(1,10) 6、(1,17)-(1,22) 各 2」（2.1） | ✓（`grep last=false` 后按 `victim_root=` 分组计数，逐项吻合） | 同上 |
| `outputs/p-s2l-legal-search-T1-fsheng-8-16.txt` 整份两行（2.1 第 209 行） | ✓ | `cat` |
| `outputs/p-s3-crash-between-checkerF-effective.txt` 第 3、12、16 行（2.2） | ✓ | `awk` |
| `outputs/p-s3-crash-between-T1-Fsheng-checkerEff.txt` 第 12、16 行（2.2） | ✓ | 同上 |
| `outputs/s3-crash-between.txt` 第 3、4、5 行（2.2） | ✓ | 同上 |
| `grep -l "I-5.2\[" *.txt` 命中三份（2.3） | ✓（命中集合完全相同：`p-s6-yiprime-asListed-deferplus.txt`、`p-s6-yiprime-asListed-legal.txt`、`p-s6-yiprime-I52-deferplus.txt`） | `grep -l` |
| `z3-results.txt` 第 25-33、67-74、210-213、244/247/251/252 行（3.2、3.3） | ✓（全部，用报告自带的 awk 行号标签核） | `awk` |
| `z3-selftest.txt` 第 3、6、16-23 行（3.3、3.4） | ✓ | `awk` |
| `z3-sweep.txt` 第 1、4、7、10、13、16 行「乙′串-重判 steps=7」（3.4） | ✓（size=4 的六行里五行 steps=7，其余 size 更大档不同，报告只引用 size=4 那几行，行号对应正确） | `awk` |
| `crates/allocator.rs:297`（`lowest_empty_segment` 邻近，2.1「开单元区内最低的全空段」） | ✓（函数在 299 行、297 行是紧邻的文档注释，与报告描述的区间一致） | `awk`/`grep -n` |
| `crates/allocator.rs:495`（`allocate_commit_generated`，2.1「提交内生块从开放聚簇段 bump」） | ✓ | 同上 |

### 1.6 小计

核了 39 处（18 处 sha256 + 12 处 `git hash-object` + 3 处 z3 产物整体 cmp + 26 处正文引用逐条核，其中「打中 138/91 格」「干净 258 行分组」按报告自带数法复算各算一处）；✓ 39；✗ 0；核不动 0。`run-probe.sh` 整条复跑结果另记文末。

## 二、Sonnet 辩方腿报告（`alloc-basis-r2-sonnet-output.md`）

复跑不适用（无独立产物、无复跑命令；报告本身是对第一轮判决与第一轮产物的复核，核对方式是逐条现查引用）。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `alloc-basis-r1-main-verification.md:26`（Y4-a/b 判决原文） | ✓ | `awk 'NR==26'` |
| `alloc-basis-r1-opus-output.md:206`、`:221-222`（丙现 (i)/(ii) 读数） | ✓ | `awk` |
| `_alloc-basis-r1-body.md:77`（Y4 判据）、`:58`（候选表丙现/乙仍定义）、`:21`（D16 已定项 1 七行）、`:82`（反向接受条款） | ✓ 四处全部逐字匹配 | `awk` |
| `alloc-basis-r1-opus-output.md:186`（Y3-c 四句第 1 句「整条臂仍算中」） | ✓ | `awk` |
| `alloc-basis-r1-opus-output.md:243`（Y4-c 读数、`D23主句违例=[5,6,7,8]`） | ✓ | `awk` |
| `alloc-basis-r1-opus-output.md:266`（点 4，I-7.4 归因） | ✓ | `awk` |
| `alloc-basis-r1-opus-output.md:287`（Y4-e 分辨） | ✓ | `awk` |
| `alloc-basis-r1-opus-output.md:336`（G3 三条收法候选） | ✓ | `awk` |
| `alloc-basis-r1-opus-output.md:379`（账 2 机理，标「整行抄」） | **✗**：漏抄「抬 F 时」三字（实际列 1 原文「`mount.rs:291-294`：**抬 F 时**「回收在写第一条带新 F 的根之前……」」，引用处该三字缺失）；且该行是三列表格行，标「整行抄」但只抄了第 1 列，未含第 2、3 列 | `python3 -c` 精确 repr 逐字符核对 |
| `alloc-basis-r1-opus-output.md:380`（账 2 按语，`mount.rs:387`） | ✓ | `awk` |
| `checks-owed.md:292`（C314 机理，标「整行抄的相关句」——非「整行抄」，用词已留了口子） | ✓（引的那句在整行内逐字能找到） | `awk` |
| `.claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:35`（E150 模型表「臂 \| 分配器 \| 回退多写什么」） | **✗**：行号错。第 35 行实为标题「### 模型」；被引表格实际在第 41、44 行 | `awk 'NR>=30&&NR<=44'` |
| `.claude/kb/decisions/23-journal的角色与格式.md:1209`（旁 1 段，主句 + 窄读法定义） | ✓ | `awk` |
| `.claude/kb/checks-owed.md:292` C314 F-A 机理（3.4 段） | ✓（与上一行同一处，两次引用都核） | `awk` |
| `.claude/kb/decisions-history/2026-09.md:8` 现查（账 1，「变更史现在有 2026-09-17 条目」这句核实是否成立） | ✓：现查确有 `### 2026-09-17：D5 已定项 8 那张 15 行表的 defer 待释放行从 0 改成 16 384……`，与 Sonnet 报告第 150 行的陈述逐字一致；`grep -c "2026-09-17"` = 3（非零），证实 Sonnet「r1 判决那句『零命中』现在不成立」的说法成立 | `awk`、`grep -c` |
| `.claude/kb/milestone/02-second-txn.md:189`（账 6，「E 落 50178、50176 被隔离」现查是否成立） | ✓：现查该行确有「E（txg 17）的数据单元落回……50178……mkfs 实例表那片 50176 也回收了但 B 的根还引用它、影子账隔离着」，与 Sonnet 报告第 160 行的陈述逐字一致 | `awk` |
| `alloc-basis-r1-opus-output.md:199`、`:358`、`:300`（旁 1 段「推的」标注、判别力自证分不开两种读法） | ✓ 三处 | `awk` |

### 小计

核了 18 处；✓ 16；✗ 2（379 行漏抄限定词 + 只抄一列却标「整行」、experiments/150.md:35 行号误引）；核不动 0。

## 三、本地攻方腿（`alloc-basis-r2-local-attack*`）

不涉及独立产物复跑（提示与样本，无脚本产物）。

### 3.1 转述核对表（`alloc-basis-r2-local-attack-translation-audit.md`）逐条核「原文文件:行」

| 表格行（摘要） | 核的结果 | 命令 |
|---|---|---|
| 行 1「窄读法」`23-journal的角色与格式.md:1209` | ✓ | `awk 'NR==1209'` |
| 行 2「保守读法」`m2-step45-code-r1-main-verification.md:56` | ✓ | `awk 'NR==56'` |
| 行 3「G5」（背景材料自身，不查 kb） | 不适用（正文自引） | — |
| 行 4「不许重新分配也不许抹头」`23-journal的角色与格式.md:1209` | ✓ | 同上 |
| 行 5「可再分配定义」`16-发布语义.md:371` | ✓ | `awk 'NR==371'` |
| 行 6「第一版环里最旧有效根恒 0」`milestone/02-second-txn.md:189` | ✓（今日行内容与引文一致） | `awk 'NR==189'` |
| 行 7「F 生效」`16-发布语义.md:375` | ✓ | `awk 'NR==375'` |
| 行 8「U1 两次释放」`milestone/02-second-txn.md:135`、`:160` | ✓（两行均核） | `awk` |
| 行 9「U2 释放代 3，早于任何被抛弃根」`layout/01-first-txn.md:285` | ✓ | `awk 'NR==285'` |
| 行 10 Fact4a「隔离 36」`milestone/02-second-txn.md:160` | **分不清：文件在腿交回之后被改过。** 背景材料快照（`_alloc-basis-r2-background.md:1388`、`_alloc-basis-r2-appendix.md:889`，冻结于 03:20-03:21 UTC）里该行仍是「被抛弃根独占的槽逐盘 34、隔离 36（加 A 与 B 都引用的 mkfs 实例表 2 槽）」，与提示引文一致；但**现在**（约 04:5x UTC）live 的 `milestone/02-second-txn.md:160` 已改写成「隔离也是 34」（G5 读法已落地，另一并发会话所为，即本轮禁读清单里的 `m2-step45-code-r2/r3`）。这不算腿的错——提示写于 03:37，取的是当时的 kb 状态 | `grep -n` 背景材料 vs 现查 kb 文件 |
| 行 11 Fact4b「50178 不是 50176」`m2-step45-code-r1-main-verification.md:56` | ✓ | `awk 'NR==56'` |
| 行 12「checker 候选集用最新根自己的 F，非 F_生效、非根自己的 F」`invariants.md:120`、`milestone/02-second-txn.md:189` | ✓ 两处均核（invariants.md 已被并发改动但第 120 行内容仍与引文一致） | `awk` |
| 行 13「中间实例行 T=0」`23-journal的角色与格式.md:1209`、`milestone/02-second-txn.md:160` | ✓ 两处 | `awk` |
| 行 14「根环几何公式 R=3 S=8」`22-单元原子性怎么合成.md:1006`、`:257` | ✓ 两处 | `awk` |
| 行 15「区域-设备归属 0/1/0」`22-单元原子性怎么合成.md:1014` | ✓ | `awk 'NR==1014'` |

### 3.2 事实表（fact table）算术自证：样本 s1、s2 的九格表

提示 Section 1/3/4/5/6/7/8 给出的物理事实（U1/U2/U3 的释放代、F_effective 时间线、reclaim-floor 公式），逐格手算核对样本 s1、s2 的 `Z4A.{narrow,conservative,G5}.{txg10,txg16,txg17}` 九格：

- (a) 是否隔离：按 narrow/conservative/G5 三种读法定义逐格代入——9 格全部与手算结果一致（✓）。
- (b)/(c) 计数与差值：34/36、0/+2，与 Section 8「固定值 34」+ U1 两槽是否隔离，9 格全部吻合（✓）。
- (d) 可再分配：release-generation 9 与 reclaim-floor（txg10 时 0、txg16/17 时 11）比较，9 格全部吻合（✓）。
- (e) 最低可发落点：结合 (a)(d) 推导出的槽对，9 格全部吻合，且 conservative 读法两格与 Fact 4a/4b 给定事实**逐字**一致（提示明确要求核对这一点）。
- s1、s2 两份样本彼此一致，无矛盾。

小计（本节）：核了 15 处转述引用 + 1 处过时检测 + 9 格算术（记为 1 组，两份样本共 18 个单元格全部手算核对）；✓ 15（转述）+ 1（过时判定成立）+ 18（算术格）= 34；✗ 0；分不清 1（milestone/02-second-txn.md:160 已被并发改动）。

## 四、本地辩方腿（`alloc-basis-r2-local-defense*`）

不涉及独立产物复跑（提示与样本，无脚本产物）。

### 4.1 F1-F8 逐条对附录原文核（`alloc-basis-r2-local-defense-translation-audit.md`）

| F 项 | 原文文件:行 | 核的结果 |
|---|---|---|
| SERIAL 定义（「第一道不过就拒」） | `_alloc-basis-r2-body.md:69` | ✓（该行原文「串：照 D3（空间分配） 已定项 12 两道闸串联，第一道不过就拒」与引文一致） |
| F1 D3 已定项12 + ⚠️ 未走三方 | `03-空间分配.md:430`、`:432` | ✓ 两行均核，含 432 行「⚠️ 这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻」逐字一致 |
| F2 D28 已定项1 九项式子 | `28-挂载期承诺量.md:21` | ✓ |
| F3 D5 已定项4 式子对照 | `05-快照-空间记账机制.md:345`、`:347`、`:362` | ✓ 三行均核 |
| F4 D16 已定项1 七行 | `16-发布语义.md:371-375,377,378` | ✓ 七行全部逐字核对（371/375 与 Opus 报告核对重叠，372-374/377/378 单独核） |
| F5 D28 已定项3 df 句 + 「这一项」范围 | `28-挂载期承诺量.md:90` | ✓（且核对表指出的关键点——「这一项」在原文语境里指切换预留子项、非全部 F2 式——经上下文（第 87-88 行「暖机那一半」段）确认成立，不是模型/审计员自己编的读法） |
| F6 D3 已定项9 两条 + ⚠️ 警告 | `03-空间分配.md:363`、`:365`、`:369` | ✓ 三行全部核，含 369 行嵌套引用 E139 自己那句「按它自己的 `df_S`……」逐字一致 |
| F7「合」「串-重判」两读法（背景，不辩护） | `_alloc-basis-r2-body.md:69` | ✓（同 SERIAL 定义引同一行，三种读法定义都在该行内） |
| F8 今天实现事实 | `_alloc-basis-r2-body.md:38`、`:39` | ✓ 两行均核，含哈希 `6bd429ed…`/`34246932…` 与正文一致 |

限定词核对（抽查）：F5 行「df 报可用」在首稿被译成「df equals the full F2 formula」当作既定事实，核对表指出的问题（「这一项」字面只指切换预留、不是全式）经查原文 87-90 行上下文成立，不是编造；F4 行首稿曾想带上 372「回退候选集」与 379「环深下限」两行，核对表说「与本轮问题无关，删去」——经查这两行在 `16-发布语义.md` 确实存在且与 F4 七行分属同一份定义表，删去理由（避免无关事实分散模型注意力）站得住，不是遗漏限定词、是主动收窄范围且理由写明。

### 小计

核了 9 处（F1-F8 + SERIAL 定义，其中 F4 按 7 个子行、F1/F6 按多行分别计入上表已展开，本节按 F 项计数不重复累加子行）；✓ 9；✗ 0；核不动 0。

## 五、跨腿共性核对

- Opus、Sonnet、local-attack、local-defense 四份材料引用的 kb 文件行号，凡是能在今日 kb 里核到的（`decisions/16-发布语义.md`、`decisions/03-空间分配.md`、`decisions/28-挂载期承诺量.md`、`decisions/05-快照-空间记账机制.md`、`decisions/22-单元原子性怎么合成.md`、`decisions/23-journal的角色与格式.md`、`invariants.md`、`layout/01-first-txn.md`）逐条核对均一致，唯一的例外是 `milestone/02-second-txn.md:160`（隔离 36→34，见 3.1 表行 10）——这份文件是本轮唯一被并发会话正在改写的 kb 文件（`m2-step45-code-r2/r3`，在我的禁读清单内），四条腿引它的地方全部核对成立（因为都对齐的是背景材料冻结时的快照，不是 live 文件）。
- `_alloc-basis-r2-background.md`/`_alloc-basis-r2-appendix.md` 与 live kb 文件之间的偏差只在 `milestone/02-second-txn.md` 一处被发现（步 4/5 现状描述已被并发会话推进到 G5 读法），其余核对的 kb 文件今日内容与背景材料快照逐字一致（即今日虽然文件本身有改动痕迹如 `invariants.md` 的哈希变化，但被引用的具体行号内容未变）。

## 六、run-probe.sh 整条复跑：最终结果

### 2.1 补充：账 5「`mount.rs:460`」引用的时序核实

Sonnet 报告（`research/prompts/alloc-basis-r2-sonnet-output.md`，mtime 2026-09-17 03:40:10 UTC）第 158 行写「现查当前 `crates/singlefs-core/src/mount.rs:460`：`rollback_floor: start.effective_floor,`」。核查时（约 04:53 UTC）该文件当前第 460 行已是另一行代码（`&|root| abandoned_by_table(root, &table),`），`rollback_floor: start.effective_floor,` 现在落在第 562 行；`crates/singlefs-core/src/mount.rs` mtime 为 2026-09-17 04:34:50 UTC，**晚于 Sonnet 报告落笔时间近一小时**。**分不清：文件在腿交回之后被改过**，不计入 ✗——Sonnet 写下这句时引用的行号在当时是准确的（与 Opus 报告 `crates-at-report/mount.rs:460` 那一份快照的行号完全一致）。

## 七、没做什么

- **不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。** 本报告的每一处 ✓/✗ 都只回答「引用抄得准不准、产物对不对、命令复跑一不一致」，不回答「Y4-c 判『打中归错了判据』对不对」「窄读法站不站得住」这类推论问题——那些交主 agent 逐条现查。
- 没有核 Opus 报告第八节「没打中的形状」表格、第九节「这条腿自己的限度」、第十节「没做什么」——这些是腿自陈的方法论限度说明，不含可独立核对的文件引用或产物数字，本轮判定为不适用逐条核对（不计入核对数、不计入未核）。
- 没有核 Opus 报告第五节「按反向接受条款记账」表与第六节「本腿提的改法」表里每一格的判断依据——那是攻方腿自己的分析结论，不是可比对的产物引用；已核对它们所依据的每一处原始产物引用（见 1.5 表）。
- 没有核本地攻方 / 本地辩方两条腿的运行记录（`*-runlog.md`）逐字，只抽查了 `alloc-basis-r2-local-attack-runlog.md` 开头两个样本的字词损坏闸记录作为背景确认，未对损坏闸脚本本身复跑。
- 没有编译 `crates/`（工作区版本）；所有涉及 `crates/` 源码的比对，要么用 Opus 腿留存的 `crates-at-report/` 快照，要么用独立拷贝并 `git init` 的草稿仓（不影响真实仓与工作区）。
- 没有做 git 写操作，没有碰真实工作区里的任何文件。
- `run-probe.sh` 整条复跑截至写这一节时仍在跑（`legal-search 20 25` 那一步，机器 32 核、`nice -19`，本机另有一条 `second_transaction_step_zero_layer0` 全量层 0 测试同时占满一个核），最终结果记文末第六节；z3_gates.py 三个子命令已独立复跑、逐字节相同（见 1.3）。
- 未逐条核对本地攻方/本地辩方两份 `-output-s1/s2.md` 样本里除「九格表」之外的其余问题（Z4B/C/D/E 等定性问答）与附录原文的逐句对应——只核了明确要求的「九格表算术」与「转述核对表」本身列出的行。

（更正：上面「六、run-probe.sh 整条复跑：最终结果」下误插入了一段与账 5 相关的补充，那段内容属于第二节，编号应为「2.1」；内容本身准确，只是标题位置放错，特此更正，结果不受影响。）

## 八、第四节判红表补充核对（原属 1.5，续）

| 引用 | 核的结果 |
|---|---|
| `p-s6-T0-Fjin-leak5.txt` 第 3、7、9 行（4.1 甲-T0①） | ✓ |
| `p-s6-T1-leak5.txt` 第 9 行（4.1 甲-T1①） | ✓ |
| `p-s6-T0-Fjin-deferplus.txt`、`p-s6-T1-deferplus.txt` 第 10 行（4.1 甲-T0/T1③） | ✓ 两份 |
| `p-s7-reuse-candidate-T0.txt`、`-yiprime.txt` 第 2 行（4.1②） | ✓ 两份 |
| `p-s6-yiprime-I52-leak5.txt` 第 3 行、`-deferplus.txt` 第 1 行（4.1 乙′①③） | ✓ 两份 |

## 九、`run-probe.sh` 整条复跑：最终状态（诚实中止说明）

复跑在草稿目录（`/tmp/claude-1000/alloc-basis-r2-verifier/rerun-repo/`、`.../rerun-work/`）里持续运行，写报告到此处时（2026-09-17 约 04:56 UTC，job 从 04:40 UTC 起跑）仍停在第一个 `legal-search 20 25` 步骤，CPU 时间已超 9 分钟（原腿自述整条脚本约 25 分钟，两次 legal-search 是大头；本机同时有另一会话的 `second_transaction_step_zero_layer0` 全量层 0 测试占满一核，`nice -19` 之下更慢）。在可用的核查时间预算内，这条复跑没能在本报告完稿前跑完全部 57 个场景 + 3 个 z3 子命令。

**已经坐实、不依赖这条复跑完成的部分**（构成对 Opus 报告可信度的强支持）：
1. **环境搭建本身正确**：草稿仓的 `git rev-parse --show-toplevel` 落在草稿目录（验证过 `git init` 生效）；6 个 `crates/` 文件哈希核对与脚本内置的 `crates-at-report/` 回退逻辑按预期触发（`allocator.rs`、`mount.rs`、`recovery.rs` 三个已被工作区并发改动的文件，脚本按报告所述逻辑自动换回报告读的那一版，日志里的三行「在仓里已被改过……换回」与 Opus 报告记录的机理完全对应）。
2. **`z3_gates.py --selftest/--verbose/--sweep`（独立复跑）与 `outputs/` 三份产物逐字节相同**——这是报告里「Z3 条款层计数模型」全部结论的直接依据，已完整验证。
3. **`patched 18 sites`**（`patch-copy.py` 输出）与报告第 442 行「`patch-copy.py` 18 处替换」一致。

**没坐实的部分**：57 个 Rust 探针场景产物与 `outputs/` 的逐字节比对——这是报告里 Z1/Z2/Z5（账本形态、F 的两个读法、可验证性）全部具体数字的产物来源，本轮核查改用「逐条引用取行区间比对」的方式核对（见 1.5、八两节，26+5=31 处具体引用全部核对一致），但**没有做「整份产物文件逐字节 `cmp`」这一步**——引用取样核对不能排除产物里没被引用到的行存在偏差的可能性，虽然可能性较低（Rust 探针是确定性计算，报告自称「无随机源」，且已核对的取样点全部吻合）。

这条复跑判定：**核不动（本轮时间预算内）**，不计入 ✓ 也不计入 ✗。留给后续核查或主 agent：若需要逐字节级别的完整确证，可在更长的时间预算下重跑 `/tmp/claude-1000/alloc-basis-r2-verifier/rerun-repo/` 里已经搭好的环境（`ALLOC_BASIS_R2_DRAFT=/tmp/claude-1000/alloc-basis-r2-verifier/rerun-work bash research/prompts/alloc-basis-r2-opus-model/run-probe.sh`，从 `rerun-repo` 目录里跑），或者直接杀掉当前后台进程（pid 见 `ps aux | grep alloc_basis_r2_probe`）等它自然跑完（脚本仍在这一轮派发的 Bash 后台任务里运行，不会因为本报告写完而被中止）。

## 十、全轮总计

按各节小计相加（一节 39 + 二节 18 + 三节 34 + 四节 9 + 八节 5 = 105 处逐条核对，另加二节文末补充的 `mount.rs:460` 时序核实 1 处）：

- **核了：106 处**
- **✓：103 处**
- **✗：2 处**（均在 Sonnet 报告：`alloc-basis-r1-opus-output.md:379` 标「整行抄」但漏抄「抬 F 时」三字且只抄表格第 1 列；`.claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:35` 行号误引，实际内容在第 41、44 行）
- **分不清（文件在腿交回之后被改过，不计入 ✓/✗）：2 处**（`milestone/02-second-txn.md:160` 隔离数「36→34」被并发会话 `m2-step45-code-r2/r3` 改写；`crates/singlefs-core/src/mount.rs:460` 的目标行被并发改动挪到第 562 行）
- **核不动：1 处**（`run-probe.sh` 整条复跑，见第九节；本机同时有另一全量层 0 测试占满一核，`nice -19` 之下这条独立复跑没能在预算内跑完，但其内置的 `crates-at-report/` 版本回退机制、`z3_gates.py` 全部三个子命令均已独立验证）

判别力自证（本报告开头）通过：把一处引用行号错判 1，核对方法判红。

## 十一、事后更新：`run-probe.sh` 整条复跑已完成（更正第九、十节）

第九节写完、报告已交回主 agent 之后，后台复跑任务（草稿目录 `/tmp/claude-1000/alloc-basis-r2-verifier/rerun-repo/`）继续跑完，`RUN-PROBE-COMPLETE`、`EXIT=0`（日志：`/tmp/claude-1000/alloc-basis-r2-verifier/run-probe.log`）。补做完整逐字节比对：

```
cd .../rerun-repo/research/prompts/alloc-basis-r2-opus-model
for f in outputs-rerun/*.txt; do cmp -s "$f" "$O/$(basename $f)" || echo DIFFERS; done
=== total rerun files: 53, diffs: 0 ===
```

- **53 份产物（含 z3-selftest.txt、z3-results.txt、z3-sweep.txt）与 `outputs/` 全部逐字节相同，0 处差异。**
- `p-s5-hole-yiprime-I52.txt`（本轮才加的场景名）与 `outputs/p-s5-hole-yiprime-T1.txt` 逐字节相同——与 Opus 报告第 9 行「是同一组环境变量换了个名字」的说明一致。
- `outputs/` 里比 `outputs-rerun/` 多出的 5 份（`newcrates-*.txt` 四份、`p-s3-crash-between.txt` 一份）：前四份报告已自陈是「04:21 UTC 之后手跑、不在 `run-probe.sh` 里」（第 42 行），核对一致；`p-s3-crash-between.txt` 的 mtime（04:03）早于本轮 `run-probe.sh` 正式定稿，是腿早期自检遗留的文件、不由当前脚本生成，不构成差异。
- 编译日志里的 `crates-at-report/` 回退提示三行与首次运行时逐字相同，「在仓里已被改过……换回」的机理确认按报告所述工作。
- `patch-copy.py` 输出 `patched 18 sites`，与报告一致。

**更正第十节总计**：核不动那 1 处（`run-probe.sh` 整条复跑）现已坐实，改记 **✓**。

**更正后的全轮总计**：核了 107 处；**✓ 104**；✗ 2；分不清 2；核不动 **0**。

判别力自证仍然有效（本更新不改变前述任何一处 ✗ 或分不清的判定）。
