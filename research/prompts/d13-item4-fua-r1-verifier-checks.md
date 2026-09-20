# 核查员核对表：d13-item4-fua-r1

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证（先做）

抽的引用：Opus 报告里 `crates/singlefs-harness/src/crash.rs:1108` `for segment in &self.segments[..segment_of_state] {`。
在草稿目录 `/tmp/claude-1000/-home-fy5090-code-singlefs/71b89f6d-fc2d-4479-bae1-ac8ab41e21bd/scratchpad/verifier-selftest/crash.rs`
（`crash.rs` 的副本）里把行号加 1，去核第 1109 行：

```
$ awk 'NR==1109{print NR": "$0}' crash.rs
1109:             for write_index in segment {
```

第 1109 行是 `for write_index in segment {`，与待核文本 `for segment in &self.segments[..segment_of_state] {` 不同字。
**判定：✗（行号+1 后判红，核查方法分辨得出不一致）。** 自证通过，往下按此方法核。

## 一、云端攻方（Opus）腿：`d13-item4-fua-r1-opus-output.md`

### 1.1 模型复跑

把 `research/prompts/d13-item4-fua-r1-opus-model/` 拷到草稿目录
`/tmp/claude-1000/.../scratchpad/verifier-opus-model/`，加 `nice -n 19` 在副本里跑：

```
$ nice -n 19 python3 fua_segmentation_domains.py > domains.rerun.out 2>&1; echo exit=$?
exit=0
$ nice -n 19 python3 selftest_distinguishing_power.py > selftest.rerun.out 2>&1; echo exit=$?
exit=0
$ sha256sum fua_segmentation_domains.py selftest_distinguishing_power.py domains.rerun.out selftest.rerun.out
80f2d2f8889edaadb7a3c39a48f64b8f4cb2a10bd0270ea59515a29ff1bb1023  fua_segmentation_domains.py
eebca3173de917629c12ba8a6a81b4deeba1d27fa8f88ffcbb5baadda1d2ac0f  selftest_distinguishing_power.py
580ddf41a280bd96b0b15b387f98838663c0a253449cb758240f2a403b2cd921  domains.rerun.out
4d21d2fa5c635667c1ed0d6566282b27754aae9efa9f7d2c9cf846fc59c7b811  selftest.rerun.out
```

两个 sha256（脚本、产物）与报告里给的逐字相同；两条脚本退出码都是 0（第二条自证退出码 0）。
**判定：✓。**

### 1.2 报的数与仓里三处独立登记对不对得上

| 数 | 报告里的出处 | 现查 | 判定 |
|---|---|---|---|
| 段序列 `2+2+1+2+2+1+18+2+1+2`、闭式 262165 | `.claude/kb/layout/01-first-txn.md:400` | `awk 'NR==400'` 该行原文含 `2+2+1+2+2+1+18+2+1+2`、`262165` | ✓ |
| mkfs 段序列 `4+1+1+1+4`、34 个崩溃状态 | 同文件 `:390` | `awk 'NR==390'` 该行原文含 `4+1+1+1+4`、`34 个崩溃状态` | ✓ |
| 段序列 `2+2+3+2+19+2+3`、闭式 524314 | `.claude/kb/experiments/142-第一个事务的干跑.md:88` | `awk 'NR==88'` 原文 `E7RESULT name=layer0_fua_not_boundary segments=2+2+3+2+19+2+3 closed_form=524314` | ✓ |
| 262171、1048609（仓里无登记位） | 报告称「这次跑出来的」 | `domains.rerun.out` 第二段（施加变异表第 67 行）：`甲 …闭式=262171`、`丁 向量数=1048609`，与本轮独立重跑输出逐字相同 | ✓（确系这次重跑产物，非编造） |

### 1.3 代码行引用

| 引用 | 抄的原文 | 现查 | 判定 |
|---|---|---|---|
| `crash.rs:1108` | `for segment in &self.segments[..segment_of_state] {` | `awk 'NR==1108'` 逐字相同 | ✓ |
| `crash.rs:1067` | 「前面的段全持久 + 当前段任意真子集…」 | `awk 'NR==1067'` 逐字相同 | ✓ |
| `crash.rs:1105`（`persisted_writes_of_state` 起） | 函数签名 | `awk 'NR==1105'` = `fn persisted_writes_of_state(&self, ordinal: u64, write_count: usize) -> Vec<bool> {` | ✓ |
| `crash.rs:327-366`（切段函数） | 整个函数体 | 函数体 327 起、366 行 `}` 收尾，与报告摘录逐字对应（Sonnet 腿完整摘录过，见下） | ✓ |
| `crates/mutations.tsv:67` | 「步 3：零单元发布…少一道屏障」，改的是 `transaction.rs` 记录与根之间那道屏障 | `awk -F'\t' 'NR==67'` 内容含该标签、目标文件、且删的两行正是 `writer.perform(CommitStep::Barrier)?;` 与紧邻的 `WriteRootRecordForceUnitAccess` 那两行 | ✓ |
| `.claude/kb/experiments/77-发布的持久顺序.md:33` | 「记录→根槽的顺序买的是记录流完整性，不是数据…」 | `sed -n '33,35p'` 逐字对得上（含省略号处的换行） | ✓ |
| `evidence-discipline.md:171/172/178` | 「打中不分辨臂」「判别子观测不到」「打中之后先问四句」三行 | `grep -n` 三行逐字对得上 | ✓✓✓ |
| `second_transaction_supplement_three_crash_injection.rs:345-347` | `assert_eq!(injection.tally.record_root_without_record, 0, "…")` | 实际该 3 行内容是 345=`injection.tally.record_root_without_record, 0,`、346=消息串、347=`);`；`assert_eq!(` 本身在第 344 行，未计入引用范围，但引用范围内容与抄文一致 | ✓（范围少收一行 `assert_eq!(` 本身，属边界写法，不影响定位） |
| `the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence` 测试名 | 复跑命令里点名的测试 | `grep -n` 命中 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs:122` | ✓ |

### 1.4 `journal_tail` 「一处读都没有，七处命中全在 tests/」这句核实

按报告给的命令原样复跑：

```
$ grep -rn "\.journal_tail" --include=*.rs crates/ | wc -l
8
$ grep -rn "\.journal_tail" --include=*.rs crates/*/src/
crates/singlefs-core/src/superblock.rs:144:        writer.put_u64(self.journal_tail);
```

命中 8 处，`crates/*/src/` 下只有 1 处（`superblock.rs:144`，写入侧 `self.journal_tail` 序列化），其余 7 处全在 `tests/`——
这一半与报告数字（「命中的七处全在 tests/」）对得上。但报告原句「除了 `superblock.rs` 的字段定义**与 `transaction.rs` 的写入侧**，一处读都没有」
**这半句不准**：`\.journal_tail` 这个模式在 `transaction.rs` 里零命中（该文件写侧用的是不带点的字段初始化 `journal_tail: plan.counter,`，不是 `self.journal_tail`），
不存在报告所说的「transaction.rs 的写入侧」这个命中。

用更宽的模式核主 agent 的反例：

```
$ grep -rn "journal_tail" --include=*.rs crates/*/src/ | wc -l
15
$ grep -n "journal_tail" crates/singlefs-checker/src/lib.rs
125:    pub journal_tail: u64,
182:        journal_tail: read_u64(slot, SUPERBLOCK_TAIL_OFFSET),
```

`crates/singlefs-checker/src/lib.rs:182` 确实是一处「读」（从字节解析出字段），与主 agent 指出的一致；`src/` 下用宽口径命中 15 处，不是 10 处
（差异未查明，留给主 agent）。**实质核实**：用 `\.journal_tail`（真正的字段访问）在全仓 `.rs` 搜，除 `superblock.rs:144` 写入侧外零命中，
`checker::SuperblockView.journal_tail` 解析出来之后没有任何 `.journal_tail` 访问消费它（`choose_superblock` 按 `slot_generation` 择槽，不看 `journal_tail`）。
**判定**：Opus 报告「一处读都没有」按字面（含 checker 的解析）**不准**（✗，应为「除写入侧的字段序列化外，唯一一次字节解析在 `checker/lib.rs:182`，但解析出的字段没有下游消费者」）；
但它据以推出的实质结论——**这个字段今天没有消费者**——现查成立（✓）。

## 计数（第一节）

核了 18 项（含 evidence-discipline.md 三行各算一项）：✓ 17，✗ 1（`journal_tail` 措辞，实质结论仍站得住）。数由 `grep -o '✓'`/`grep -o '✗'` 在本节正文（不含本行）里现数得出。

## 二、云端正推（Sonnet）腿：`d13-item4-fua-r1-sonnet-output.md`

### 2.1 kb 与内核文档引用

| 引用 | 抄的原文 | 现查 | 判定 |
|---|---|---|---|
| `.claude/kb/decisions/13-验证路线.md:71` | 定案句整句 | `grep -n` 逐字相同 | ✓ |
| `writeback_cache_control.rst:23-27`（Linux 6.17 树） | PREFLUSH 段整段 | `sed -n '23,27p'`（`/home/fy5090/code/fs-refs/linux-6.17/`）逐字相同 | ✓ |
| 同文件 `:36-38` | FUA 段整段 | `sed -n '36,38p'` 逐字相同 | ✓ |
| 同文件 `:44-47` | 「两个标志可同设一个 bio」 | `sed -n '44,47p'` 逐字相同 | ✓ |
| 同文件 blk-mq 段（背景材料 :58，Sonnet 未标行号，只说「已引」） | `BLK_FEAT_FUA` 那句 | `grep -n` 命中该文件第 92 行，逐字相同 | ✓ |

### 2.2 virtio-blk / NVMe / SCSI 外部规范核实（现查网络，而非采信报告转述）

```
$ curl -sL -A "Mozilla/5.0" -o virtio-spec.html https://docs.oasis-open.org/virtio/virtio/v1.3/csd01/virtio-v1.3-csd01.html
$ grep -io -c "FUA" virtio-spec.html
0
$ grep -io -c "force unit access" virtio-spec.html
0
```
virtio 1.3 规范全文「FUA」「force unit access」零命中，与报告一致；`VIRTIO_BLK_T_*` 命令类型清单原文抽出后与报告引文逐字相同。**判定：✓。**

```
$ curl -sL -A "Mozilla/5.0" https://nvmexpress.org/wp-content/uploads/NVM-Express-Base-Specification-2.1-2024.09.17-Ratified.pdf -o x.html
$ grep -io -c "gravity" x.html
25
$ curl -sL -A "Mozilla/5.0" "https://www.t10.org/cgi-bin/ac.pl?t=f&f=sbc4r22.pdf" -o y.html
$ grep -io -c "File Access Monitor" y.html
9
```
NVMe 官网确系返回 Gravity Forms 网关页，T10.org 确系返回「File Access Monitor」访客登记页——报告说的两处访问闸真实存在，
**没有拿别的东西冒充规范原文**：报告在查不到之后只引用了 Linux 内核驱动源码，并明确标注「是内核对协议的翻译代码，不是规范原文」。**判定：✓。**

```
$ grep -n "FUA" /home/fy5090/linux-bug-fix/linux/drivers/block/virtio_blk.c | wc -l
0
$ grep -n "BLK_FEAT_WRITE_CACHE" /home/fy5090/linux-bug-fix/linux/drivers/block/virtio_blk.c
1112:  lim.features |= BLK_FEAT_WRITE_CACHE;
1114:  lim.features &= ~BLK_FEAT_WRITE_CACHE;
1508:  lim.features |= BLK_FEAT_WRITE_CACHE;
```
`virtio_blk.c` 全文件 `FUA` 零命中、`BLK_FEAT_WRITE_CACHE` 命中 3 处，与报告一致。**判定：✓。**

| 引用 | 抄的原文/断言 | 现查 | 判定 |
|---|---|---|---|
| `drivers/nvme/host/core.c:1025-1026` | `if (req->cmd_flags & REQ_FUA) control \|= NVME_RW_FUA;` | `sed -n '1025,1026p'` 逐字相同 | ✓ |
| `drivers/scsi/sd.c:1475` | `fua = rq->cmd_flags & REQ_FUA ? 0x8 : 0;` | `awk 'NR==1475'` 逐字相同 | ✓ |
| `drivers/scsi/sd.c:3231` | `sdkp->DPOFUA = (data.device_specific & 0x10) != 0;` | `awk 'NR==3231'` 逐字相同 | ✓ |

### 2.3 本工程 `block_device.rs` 双实现调同一 syscall（本轮最重的附带发现）

| 引用 | 抄的原文 | 现查 | 判定 |
|---|---|---|---|
| `block_device.rs:245-247`（`FileBackedBlockDevice`） | `WriteDurability::ForceUnitAccess => { self.file.sync_data()... }` | `awk` 逐行核，函数属 `impl BlockDevice for FileBackedBlockDevice`（233 行起） | ✓ |
| `:251-253`（同实现 `barrier()`） | `fn barrier(&mut self) -> ... { self.file.sync_data()... }` | 逐字相同 | ✓ |
| `:449-451`（`DirectInputOutputBlockDevice`） | 同上结构 | 函数属 `impl BlockDevice for DirectInputOutputBlockDevice`（393 行起，`write_at` 421 行起） | ✓ |
| `:455-457`（同实现 `barrier()`） | 同上 | 逐字相同 | ✓ |
| `block/fops.c:937` | `.fsync = blkdev_fsync,` | `grep -n`（`/home/fy5090/linux-bug-fix/linux`）逐字相同 | ✓ |
| `block/fops.c:590-609`（`blkdev_fsync` 函数体） | 报告贴的代码块 | 函数确从 590 行起，`blkdev_issue_flush(bdev)` 调用在函数体内 | ✓ |
| `block/blk-flush.c:470-476`（`blkdev_issue_flush`） | `bio_init(&bio, bdev, NULL, 0, REQ_OP_WRITE \| REQ_PREFLUSH);` | 函数恰好占 470-476 行，第 474 行正是这一句、只有 `REQ_PREFLUSH`、没有 `REQ_FUA` | ✓ |

**口径差核实**：`block/fops.c`、`block/blk-flush.c`、`drivers/nvme/host/core.c`、`drivers/scsi/sd.c`、`drivers/block/virtio_blk.c`、
`include/uapi/linux/virtio_blk.h` 全部取自 `/home/fy5090/linux-bug-fix/linux`：

```
$ cd /home/fy5090/linux-bug-fix/linux && git describe --tags
v7.3-rc1
```

而 `writeback_cache_control.rst`（含 `Documentation/block/writeback_cache_control.rst` 的三段核心引文）取自
`/home/fy5090/code/fs-refs/linux-6.17/`，是 Linux 6.17。**报告全文没有一处提到这两棵树版本不同**
（`grep -n "6.17\|v7.3\|版本\|口径" d13-item4-fua-r1-sonnet-output.md` 只命中报告自己第 21 行给 6.17 树标注版本号那一处，
没有第二处提及口径差）。**判定：✗（未披露）**——两棵树相隔至少几个大版本，`REQ_FUA`/`REQ_PREFLUSH` 的语义在这段跨度里没有变化史无法排除，
报告混用两棵树而不标注，读者拿不到这一层信息去自行判断风险。

### 2.4 `transaction.rs` 两处行号核实

| 引用 | 抄的原文/断言 | 现查 | 判定 |
|---|---|---|---|
| `transaction.rs:158-165`（根槽写用 `WriteDurability::ForceUnitAccess`） | `device.write_at(..., WriteDurability::ForceUnitAccess)` | `device.write_at(` 实际起于 **159** 行，`)?;` 收尾在 **166** 行；158 行是上一条语句的 `.expect(...)`，165 行只到 `WriteDurability::ForceUnitAccess,` 不含收尾括号 | ✗（应为 159-166，偏一行） |
| `transaction.rs:566` 起 `CommitStep::Barrier` 调 `device.barrier()` | — | `awk 'NR==566'` = `if let Err(cause) = persist(pool) {`，与 `CommitStep::Barrier`/`device.barrier()` 无关；`grep -n "CommitStep::Barrier\|device.barrier()"` 全文件命中在 124、178、181、405、548、553、2118、2123 行，其中真正执行 `device.barrier()` 的匹配分支在 **178-184** 行 | ✗（`:566` 附近没有这个调用，指错了位置；`device.barrier()` 的处理逻辑在 `:178-184`，`CommitStep::Barrier` 的两处调用点在发布路径的 `:548`、`:553`） |

第二处（`:566`）是本轮找到的最明显的一处行号错误：引用的行与断言的内容完全对不上，不是差一两行的边界写法。

### 2.5 `crash.rs` / `lib.rs` / `segments.rs` 结构核实

| 引用 | 抄的原文 | 现查 | 判定 |
|---|---|---|---|
| `crash.rs:327-366`（`writes_and_segments_with_stream_indexes`） | 报告贴的完整函数体（删了与切段无关的字段赋值） | 逐行核对，函数体确实 327 起、366 行 `}` 收尾，报告贴的控制流与原文一致 | ✓ |
| `RecordedOperationKind` 只有三个成员 | `lib.rs:30-34` | `sed -n '30,34p'`：`Write,`/`WriteForceUnitAccess,`/`Barrier,` 三个成员、无第四个 | ✓ |
| `segments.rs:71-74`（切段规则文档注释） | 四句整段 | `sed -n '71,74p'` 逐字相同 | ✓ |
| `segments.rs:266-274`（钉子测试） | `assert_eq!(segment_sizes_text(...), "2")` 与注释 | 逐行核对，266 行注释、267-270 构造、271-274 断言，逐字相同 | ✓ |

### 2.6 D19 / D20 / D4 三条已定项核实（Sonnet 附带发现，非 Q1/Q2 射程内但如实记录）

| 引用 | 抄的原文 | 现查 | 判定 |
|---|---|---|---|
| `.claude/kb/decisions/20-承重面单元的原子性与自包含.md:67` | 「有父指针的单元…不依赖任何宽度…」 | `awk 'NR==67'` 逐字相同 | ✓ |
| `.claude/kb/decisions/19-块指针的结构与宽度预算.md:39` | `#### 已定项 2：密文校验和取 32 位` | `awk 'NR==39'` 逐字相同 | ✓ |
| `.claude/kb/decisions/04-校验和位置.md:18` | `#### 已定项 1：校验和粒度与随机小读的张力` | `awk 'NR==18'` 逐字相同 | ✓ |
| Sonnet 的实质判断：D13 已定项 4「依据」引「D4 已定项 1」佐证「校验和内联进父指针」，但该说法实际在 D4 文件**引言段**（未编号），不在已定项 1（已定项 1 谈的是 32 KiB 粒度） | — | 读了 `04-校验和位置.md` 全文：第 2 行引言段原文正是「校验和内联进指向该块的父指针」；已定项 1（第 18-26 行）整段谈的是「数据 extent 的校验和/MAC 覆盖 32 KiB」的粒度取舍，完全没有提「内联进父指针」这件事。反查 D13 原文（`decisions/13-验证路线.md` 已定项 4「依据」段第二条）：确实写着「有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19」 | ✓（Sonnet 这条「分项号可能指错位置」的发现现查成立，且它自己也只标「如实记下，不据此下判」，未越权判决） |

## 计数（第二节）

核了 29 项：✓ 26，✗ 3（`transaction.rs:158-165` 偏一行、`transaction.rs:566` 指错位置、跨内核树版本口径差未披露）。数由 `grep -o` 在本节正文（不含本行）里现数得出。

## 三、本地辩方腿：`d13-item4-fua-r1-local-defense*`

### 3.1 判别力缺口：损坏闸判绿的两份样本

按报告命令原样复跑（不在腿的原目录里跑改成直接对已落盘的样本文件跑，两份样本文件本身就是要核的产物，未改动）：

```
$ sed -n '15p' d13-item4-fua-r1-local-defense-output-s1.md
...This would refuterefute row 2's reading because...
$ sed -n '9p' d13-item4-fua-r1-local-defense-output-s2.md
...which is exactly what the measured test observesing behavior shows...
$ python3 research/scripts/corruption-check.py d13-item4-fua-r1-local-defense-output-s1.md; echo $?
绿 ... cjk=0 words=1080 ... 粘连=0 实词自复读=0 ...
0
$ python3 research/scripts/oov-check.py d13-item4-fua-r1-local-defense-output-s1.md; echo $?
绿 ... 生词=2 拼接=0
     生词: strongest refuterefute
0
$ python3 research/scripts/corruption-check.py d13-item4-fua-r1-local-defense-output-s2.md; echo $?
绿 ... 粘连=0 ...
0
$ python3 research/scripts/oov-check.py d13-item4-fua-r1-local-defense-output-s2.md; echo $?
绿 ... 生词=3 拼接=0
     生词: strongest observesing wording's
0
```

第 15 行「refuterefute」、第 9 行「observesing」两处真实存在，两个脚本对两份样本都判绿（退出码 0），
`oov-check.py` 虽把两词列进「生词」但没有标「拼接」（拼接=0），因此不触发红。**判定：✓（报告称的判别力缺口真实存在，不是编造）。**
未产生 `-output-void*.md`（`ls` 确认目录下无此轮的 void 文件），与报告「四次调用全部退出码 0，没有作废」一致。

### 3.2 逐句转述核对表核实（`-translation-audit.md`）

逐行核对表里给出的「原文文件:行」，抽查如下（其余行内容已在第二节交叉验证过，如 kb 决策句、块层契约三段、`transaction.rs` 五步、`mutations.tsv:67`）：

| 核对表条目 | 原文文件:行 | 现查 | 判定 |
|---|---|---|---|
| `crash.rs` 切段函数文档注释 | `crash.rs:324-325` | `grep -n` 逐字相同 | ✓ |
| `segments.rs` 切段规则文档注释 | `segments.rs:71-74` | 同 2.5 节，逐字相同 | ✓ |
| 单测注释+断言值 "2" | `segments.rs:266,271-273` | 266 行注释逐字相同；271-273 为 `assert_eq!(` 开头两行 + `"2"`，274 行的 `);` 未计入但不影响内容判定 | ✓ |
| 判别力文档注释整段 | `second_transaction_supplement_three_crash_injection.rs:275-277` | `grep -n` 逐字相同（275 起、277 止） | ✓ |
| `root_without_record` 字段语义 | `crash.rs:461-463` | `awk 'NR>=458 && NR<=463'`：461 行是 `pub struct RecordCheck {`（结构体开括号，非语义描述），语义注释在 **462** 行、字段声明在 **463** 行 | ✗（应为 462-463，461 是无关的结构体声明行） |
| `crash_points_withholding_write_kind` 断言 | `:333-342`（代码）、`:341`（消息） | `awk` 核，333 行 `assert!(` 起、341 行正是消息串所在行；342 行是最后一个参数、收尾 `);` 在 343 行未计入 | ✓ |
| 变异表标签、发布五步、宿主/虚机读数、块层三段引文 | 见第二节 | 均已交叉核过 | ✓ |

## 四、主 agent 自己的现查产物

### 4.1 `d13-item4-fua-r1-device-readings.md`

宿主盘 sysfs 读数现场重量一遍：

```
$ cat /sys/block/nvme0n1/queue/write_cache /sys/block/nvme0n1/queue/fua /sys/block/nvme0n1/queue/rotational
write back
1
0
```

与文档第 12-16 行逐字相同。**判定：✓（活体现查，非快照）。** 虚机 virtio-blk 部分（`fua=0`）需要起 QEMU，**核不动**。

### 4.2 `d13-item4-fua-r1-device-model-readings.md`

三种 QEMU 设备型号（`virtio-blk-pci`/`virtio-scsi-pci`/`-device nvme`）的 `fua` 读数、insmod 次序、`blklogwrites` 录制、
`log0.img` 头 8 字节 `rhswfsj\0` 魔数——这些都要求起 QEMU 虚机才能复现。**核不动（要虚机）**，不计入 ✓/✗。

### 4.3 `d13-item4-fua-r1-device-log-normalisation.md`

| 引用 | 抄的原文 | 现查 | 判定 |
|---|---|---|---|
| `DeviceEvent` 枚举 | `crates/singlefs-harness/src/device_log.rs:24-36` | `grep -n "pub enum DeviceEvent"` 起于 24 行、闭括号在 36 行，四个成员逐字相同 | ✓ |
| `LOG_FUA_FLAG` 判断补 `Flush` | `:132-134` | `awk` 逐字相同 | ✓ |
| `LOG_FLUSH_FLAG` 判断补 `Flush` | `:136-138` | `awk` 逐字相同 | ✓ |
| 文档注释「FUA 写之后跟一个 FLUSH」 | `:147` | `awk 'NR==147'` 逐字相同 | ✓ |
| 期望侧「每个 FUA 写后面无条件补一个 Flush」 | `:167-169` | `awk 'NR>=166 && NR<=170'`：实际 `if operation.kind == ...` 在 **168** 行，`events.push(...)` 在 **169** 行，闭括号在 **170** 行；167 行是上一段 `Write` 推入语句的收尾 `});`，与这段代码无关 | ✗（应为 168-170，偏一行） |

### 4.4 `d13-item4-fua-r1-c313-row-rot.md`

| 引用/断言 | 现查 | 判定 |
|---|---|---|
| `.claude/kb/checks-owed.md:410`（C313 已还清行，含单测名 `fua_not_a_boundary_gives_524311_states`） | `awk 'NR==410'` 逐字相同 | ✓ |
| 仓里真实单测名是 `fua_not_a_boundary_gives_524314_states`，`524311` 那个名字全仓 `.rs` 零命中 | `grep -rn "fua_not_a_boundary" --include=*.rs .` 只命中 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:4445`，名字里是 `524314` | ✓ |
| `experiments/142-第一个事务的干跑.md:176、:235` 均已改成新值 262165/524314 | `awk 'NR==176'`、`awk 'NR==235'` 现查，两行都含 `524314`（`176` 行还写着「另一读法（524314）只报数不判」） | ✓ |
| `decisions-history/2026-09.md:7235` 记着新值、`:8164-8165` 的「改前/改后」分别记旧值/新值 | `sed -n '7235p;8164,8165p'` 现查：7235 行长段落里含「另一读法是 524314，由单测 `fua_not_a_boundary_gives_524314_states` 钉住」；8164 行「改前」句含「262162 / 524311」，8165 行「改后」句含「262165 与 524314」 | ✓ |
| 门禁 33 号、59 号、27 号存在，射程如文档所述 | `ls .claude/gate.d/` 命中 `27-format-constants.sh`、`33-mutation-tables.sh`、`59-crates-mutation-replay.sh` | ✓ |
| `.claude/rules/mutation-sampling.md` 「第七类」标题存在 | `grep -n "第七类"` 命中该文件第 54 行 | ✓ |

这份文档全部核对项通过，唯一的疵点是它自己另一份姊妹文档（4.3 节）里的一处行号偏差，与 c313-row-rot.md 本身无关。

## 计数（第三、四节）

第三节核了 8 项：✓ 7，✗ 1（`crash.rs:461-463` 应为 462-463）。
第四节核了 12 项：✓ 11，✗ 1（`device-log-normalisation.md` 的 `:167-169` 应为 168-170）；另有 1 份文档（device-model-readings.md）整份核不动，不计入这 12 项。数由 `grep -o`（排除含「不计入」字样的行）在两节正文里现数得出。

## 五、快照与前提说明

`research/prompts/d13-item4-fua-r1-start-snapshot.sha256` 给了 12 个文件的开工快照。现场重算全部 12 个 sha256：
只有 `crates/mutations.tsv` 与快照不同（快照 `dbaef664...`，现值 `1c273753...`），其余 11 个逐字节相同。
按主 agent 给的前提（这一轮里派的实现员重写了 `mutations.tsv` 第 189 行、追加第 192 行，第 67 行未变），
本报告第一、二节里对 `mutations.tsv:67` 的核对**按现在的主树核**（未去索取该文件在快照那一刻的原始内容单独重放，
因为改动范围明确不含第 67 行，且现读内容与三条腿的转述、与变异表本身「一条一行」的结构一致，两处独立佐证）。
其余 11 个文件因为哈希与快照相同，本报告里对它们的全部行号核对等同于对快照核。

## 总计数

| 腿/来源 | 核了 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| 云端攻方（Opus） | 18 | 17 | 1（实质结论仍站得住） | 0 |
| 云端正推（Sonnet） | 29 | 26 | 3 | 0 |
| 本地辩方（转述+抽样） | 8 | 7 | 1 | 0 |
| 主 agent 自查产物 | 12 | 11 | 1 | 1 份（device-model-readings.md，要虚机，不计入前一列） |
| **合计** | **67** | **61** | **6** | **1 份文档** |

四列数字均由 `sed -n '<节起>,<节止>p' 报告文件 | grep -v 不计入 | grep -o '✓'`（✗ 同理）在每节的原始判定文字上现数，见各节末的「计数」小节；未手数。

六处 ✗ 汇总：
1. Opus「journal_tail 一处读都没有」——字面不准（checker/lib.rs:182 确有一次字节解析），但「无消费者」的实质结论现查成立。
2. Sonnet `transaction.rs:158-165` 应为 159-166（偏一行）。
3. Sonnet `transaction.rs:566` 指错位置（`device.barrier()` 真正处理逻辑在 178-184，`CommitStep::Barrier` 调用点在 548/553）。
4. Sonnet 混用两棵不同版本的内核树（`fs-refs/linux-6.17` 与 `linux-bug-fix/linux` 即 v7.3-rc1）未披露口径差。
5. 本地辩方核对表 `crash.rs:461-463` 应为 462-463。
6. 主 agent 自查产物 `device-log-normalisation.md` 的 `:167-169` 应为 168-170。

## 没做什么

- 不判 Q1-Q5 任何一格该采纳哪个候选、不判「候选甲更对」这个结论成不成立；不核推理本身，只核引用、产物与复跑。
- 不判 Opus 第四节「本腿提的改法」（甲′、改法二/三/四）该不该写回 kb——那是判决，不是观测。
- 未起 QEMU 虚机复核 `d13-item4-fua-r1-device-model-readings.md` 与 `device-readings.md` 第二节（virtio-blk `fua=0`）；
  也未复核门禁 55 号今天用什么设备型号跑（Opus/主 agent 都称是 virtio-blk，未在本轮独立起门禁验证）。
- 未复核 `registered_segment_sequences_match_every_recorded_path`、`crash_points_withholding_write_kind` 等既有单测的实际运行结果
  （Sonnet 自己也标注未重跑，只核了源码逐字）——这需要 `cargo test`，本轮未编译。
- 未逐条核对 `_d13-item4-fua-r1-background.md` 小节清单表（第 99-291+ 行，几百行「抄/不抄」判断）里每一行的「理由」是否恰当——
  这是材料取舍的实质判断，不是引用核对；只抽查了被三条腿实际引用到的那几处背景材料行号（均对得上）。
- 未核 `_d13-item4-fua-r1-appendix.md`、`_d13-item4-fua-r1-checklist.md`、`_d13-item4-fua-r1-body.md` 三份材料文件的内部一致性
  （只核了三条腿实际抄用的那些行，未通读附录与清单全文）。
- 未收到任何一条腿报告的独立 sha256sum（派发提示未给），因此没有做「报告文件现在的 sha256 与交回里给的对不上」那一类判定。
- 未跑 `.claude/gate.d/stage-owners.tsv` 里登记给 `three-way-verifier` 的门禁阶段（未在共用约束或本轮派发提示里看到点名给这个角色的阶段）。
