# c381-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

轮名：`c381-r1`（设计轮，没有开工快照）。按派发提示：这一轮对主树核（`git status` 显示 `.claude/kb/checks-owed.md`、
`.claude/kb/decisions/16-发布语义.md` 当前有另一会话未提交的「决策正文瘦身」改动，`.claude/kb/decisions/23-journal的角色与格式.md`
经查此刻无未提交改动）。引用对不上时先对 `git show HEAD:` 那一版核，再决定记「分不清」还是 ✗；`.claude/kb/invariants.md`、
`.claude/kb/decisions/22-单元原子性怎么合成.md`、全部 `crates/*.rs` 经查均无未提交改动，按主树直接核、不设「分不清」。

草稿目录：`/tmp/claude-1000/c381-r1-verifier/`（`repo/` 是打了 `c381-arms-core.diff` / `c381-arms-harness.diff` 的副本，
`pristine/` 是 HEAD 版 core 的未打补丁副本，两者 harness 均按 opus 报告第零节的做法把 `mutations.tsv`、
`history.rs`、`lib.rs`、`second_transaction_supplement_three_random_history.rs` 换回 `git show HEAD:` 那一版、
删掉未跟踪的 `model.rs` / `model_comparison.rs`；`checker-rerun/` 是本地攻方两份样本的重跑目录；`selftest/` 是判别力自证用的副本）。

## 〇、判别力自证

抽 opus 报告第 142 行的引用「`transaction.rs` 第 2113 行 `writer.perform(CommitStep::WriteRootRecordForceUnitAccess {`」，
在草稿目录里拷一份 `transaction.rs`、在原第 2113 行前插入一个空行制造「行号 +1」的错位，再按同样方法核这份错位副本：

```
$ awk 'NR==2112{print ""}1' transaction.rs > transaction.rs.shifted   # 制造错位
$ awk 'NR==2113{print NR": "$0}' transaction.rs.shifted
2113:         writer.perform(CommitStep::Barrier)?;
```

第 2113 行变成了 `writer.perform(CommitStep::Barrier)?;`，不是引用要求的 `WriteRootRecordForceUnitAccess`——
判定：**✗**。核对方法能分辨错位引用，往下开工。

## 一、云端攻方（Opus）

时刻：本次核查 2026-09-19（东京时间，本机时钟 UTC）。腿报告 `research/prompts/c381-r1-opus-output.md`
（sha256 `be1fbc19c29152e3ae61f31804484c327f694f215499607c2a7f41742bee6972`，交回给的值；本次核查时报告文件现有
sha256 与该值一致，`sha256sum research/prompts/c381-r1-opus-output.md`）。模型目录
`research/prompts/c381-r1-opus-model/`（21 个文件，逐个 sha256 与报告第零节表格核对，全部相同——见下表）。

### 1.1 模型目录完整性（21 个文件 sha256）

`sha256sum research/prompts/c381-r1-opus-model/*` 逐个与报告第 33–55 行的表格比对，**21 个文件全部相同**，
不逐行贴（命令与两份表格本身可比对，不重复抄）。

### 1.2 file:行号 + 抄的原文

| 引用（报告行号） | 核的结果 | 命令 |
|---|---|---|
| `transaction.rs:2113` `writer.perform(CommitStep::WriteRootRecordForceUnitAccess {`（第 142 行） | ✓ | `awk 'NR==2113' crates/singlefs-core/src/transaction.rs` |
| `transaction.rs:2117`「之后第 2117 行才轮换超级块槽」（第 142 行） | ✓（第 2117 行是 `writer.perform(CommitStep::RotateSuperblockSlots {`） | `awk 'NR==2117' crates/singlefs-core/src/transaction.rs` |
| `transaction.rs:1260` `*allocator = allocator_before_this_publish;`（第 142、87 行） | ✓ | `awk 'NR==1260' crates/singlefs-core/src/transaction.rs` |
| `transaction.rs:885` `ReleaseTargetAlreadyReleased`（第 87、143 行） | ✓（第 885 行是 `return Err(PublishError::ReleaseTargetAlreadyReleased {`） | `awk 'NR==885' crates/singlefs-core/src/transaction.rs` |
| `recovery.rs:897` `if !all_verified {`（第 142、229 行） | ✓ | `awk 'NR==897' crates/singlefs-core/src/recovery.rs` |
| `walk.rs:1463` `format!("盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}")`（第 247 行） | ✓ | `awk 'NR==1463' crates/singlefs-checker/src/walk.rs` |
| 背景材料第 53 行「不退回；这次发布已成立，调用方拿到这次的版本」（第 143 行）、「不退回；这次发布按『结局不确定』对待……」（第 235 行） | ✓（正确标注为背景材料、非 kb 行号） | `awk 'NR==53' research/prompts/_c381-r1-background.md` |
| `research/prompts/m2-wave2-code-r1-opus-output.md` 第三节 Z2-c（第 107 行「原历史」出处） | ✓（文件存在，第 221 行标题「### Z2-c 最后一步失败时根已 FUA……」，槽号 50257 与本轮 k4.log 一致） | `grep -n "^## 三\|Z2-c" research/prompts/m2-wave2-code-r1-opus-output.md` |

`transaction.rs`、`recovery.rs`、`walk.rs` 三份代码经查此刻在主工作区均无未提交改动（`git status --porcelain -- crates/` 只列
`mutations.tsv`、`history.rs`、`lib.rs`、`second_transaction_supplement_three_random_history.rs` 与两个未跟踪的 `model*.rs`，
均是 opus 报告第零节自己交代要换回 `HEAD:` 的那几个，不含上面引用的三个文件），因此上表 7 处不设「分不清」。

### 1.3 引产物（逐字找）

| 引用（报告位置） | 核的结果 | 命令 |
|---|---|---|
| 第 109–130 行 `k4.log` 摘录（甲/乙/丙/丁「原历史」与乙「从现行版」四段） | ✓ 逐字节在 `research/prompts/c381-r1-opus-model/k4.log` 里找到，与报告贴的文本相同 | `grep -A4 '^\[甲\]\|^\[乙\]\|^\[丙\]\|^\[丁\]' research/prompts/c381-r1-opus-model/k4.log` |
| 第 135 行甲的 checker 五条红（I-2.1/I-3.1/I-4.8/I-7.2/I-7.4，槽 50257，540672/475136） | ✓ 逐字节找到 | 同上 |
| 第 149–213 行 phase-a/b/c/d/e.log 的 `runs=… {…}` 汇总（抽查 [甲 S3]、[乙 S1]phase-b、[丙 S3]phase-c、[乙 写行/暖机]phase-d、[丁 S3（报错但已落盘）]phase-e 共 5 处） | ✓ 5 处全部逐字节相同 | `grep '\[甲 S3\]' research/prompts/c381-r1-opus-model/phase-a.log`（其余 4 处同法） |
| 第 98 行表格「甲 173171」等 8 个分臂合计数（阶段 A 四臂、B/C/D/E 各甲一列） | ✓ 用日志里 S1+S2(+S3)/S4+写行暖机 分段数相加，8 个合计数全部与表格一致 | `python3 -c "print(133168+24163+15840)"`（其余 7 个同法，见下方「命令」小节） |
| 第 237–247 行 `s2-retry.log` 摘录（乙 k=16 段） | ✓ 逐字节找到 | `grep -A5 'k=Some(16)' research/prompts/c381-r1-opus-model/s2-retry.log` |
| 第 271–279 行 `no-remount.log` 摘录（甲/乙/乙′ 各 k=16/17/18 一行） | ✓ 逐字节找到（丙、丁两行报告本就没有引，报告自己写明「丁、丙没跑这一核」，不算欠账） | `grep -n "不带重开的段" research/prompts/c381-r1-opus-model/no-remount.log` |
| 第 287–293 行 `b-prime.log` 摘录（乙′ k=17 报错但已落盘=false 一段） | ✓ 逐字节找到 | `grep -A6 "乙′ k=17" research/prompts/c381-r1-opus-model/b-prime.log` |
| 第 247、312 行算术「704512 − 540672 = 163840 = 10×16384」「704512 − 638976 = 65536 = 4×16384」 | ✓ 算对 | `python3 -c "print(704512-540672,(704512-540672)/16384);print(704512-638976,(704512-638976)/16384)"` |

命令补齐（第 98/166/179/192/201 行 8 个合计数逐一核）：
`python3 -c "print(133168+24163+15840,133168+17656+14896,133168+12011+7392,133168+24163+15840,130672+23695+15528,27232+4118+2416,93+868,133168+23760+15840)"`
→ `173171 165720 152571 173171 169895 33766 961 172768`，与报告第 98 行「甲 173171、乙 165720、丙 152571、丁 173171」、
第 99 行「甲 169895」、第 100 行「甲 33766」、第 101 行「甲 961」、第 102 行「甲 172768」逐个相同。

### 1.4 复跑（草稿目录自己的副本，不在腿的原目录里跑）

按报告第零节的做法，在 `/tmp/claude-1000/c381-r1-verifier/repo`（打补丁）与 `/tmp/claude-1000/c381-r1-verifier/pristine`
（不打补丁）各自 `rsync -a --exclude target --exclude .git` 拷主工作区、把 4 个 harness 文件换回 `HEAD:`、删 2 个未跟踪文件、
应用两个 diff、拷测试文件。开工前 `ps -o pid,etimes,args` 查过一遍，没有 `qemu-system` / `vm-bench.sh` / `e152-*` / `fio` 在跑。

**hash-object 核对（对应报告第 138 行「HEAD 版 core……与 `HEAD:` 那一版逐个相同」）**：

```
$ cd /tmp/claude-1000/c381-r1-verifier/pristine
$ git -C /home/fy5090/code/singlefs hash-object crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/mount.rs crates/singlefs-core/src/allocator.rs
d7de039cc15f5da48a98238e78f8799f4755051c
a23acac654d333314c5d669ae498fa0b46401233
6544913c2b18e243b7dfa76bcb4702fb7c7a358f
```
与主仓 `git rev-parse HEAD:crates/singlefs-core/src/{transaction,mount,allocator}.rs` 逐个相同，也与报告第 8、138 行给的三个哈希相同。**✓**

**复跑一：K4 原历史（打补丁的 `repo` 副本）**——对应派发提示「至少复跑……K4 原历史那一段」，即报告第三节：

```
$ cd /tmp/claude-1000/c381-r1-verifier/repo
$ nice -n 19 cargo test --release -p singlefs-harness --test c381_k4 -- --nocapture --test-threads=1
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
```
输出（甲/乙/丙/丁「原历史」与「从现行版」四段，含甲的五条 checker 红）与报告第 109–136 行贴的 `k4.log` 内容**逐字节相同**——
甲仍是 `remount=REFUSED Recovery(UnitUnreadable { slot: SlotNumber(50257) })`、同样五条红、同样 540672/475136；
乙、丙、丁仍是 `checker@crash=[]`、`checker@after=[]`、`remount=ok(chosen 5)`。**✓**
（不比对整份日志的 sha256：编译横幅、警告行、时间戳这些随本机环境变化，报告自己也只贴了过滤后的 `grep -A4/-A5` 结果，
这里按同样字段比对，不按整份哈希判。）

**复跑二：K4 原历史（不打补丁的 `pristine` 副本）**——对应报告第 138 行「HEAD 版 core……上用同一个探针……甲这一条逐字相同」：

```
$ cd /tmp/claude-1000/c381-r1-verifier/pristine
$ nice -n 19 cargo test --release -p singlefs-harness --test c381_k4 -- --nocapture --test-threads=1
```
甲那一段与复跑一的甲逐字节相同（同样五条红、同样槽 50257、同样两个数）。**✓**

**复跑三：HEAD 版 core 上「只让 C 在记录与根槽之间断电、可写重开、I-3.1 红」**——对应报告第六节：

```
$ cd /tmp/claude-1000/c381-r1-verifier/pristine
$ for c in 17 18; do C381_CRASH=$c C381_ACTIONS= nice -n 19 cargo test --release -p singlefs-harness --test c381_single -- --nocapture; done
```
`crash=Some(18)`（根槽 FUA 之前）四条「臂名」（实为同一份 HEAD 代码）输出均为：
`steps=["err(BlockDevice)"] writes_after_c=19 writes=19`、`checker@crash=[]`、`remount=ok(chosen 4)`、
`checker@after=["I-3.1: Violated(\"盘 0：记账的已分配 Some(704512)，遍历全部有效根得到 638976\")"]`——
与报告第 305–309 行贴的 `pristine-crash.log` 摘录逐字节相同。`crash=Some(17)`（只有盘 0 一份记录）同样四条全红、
同样两个数（704512/638976），与报告第 311 行「断电点 17……同样红、同样的两个数」一致。**✓**

三次复跑均前台跑、单条命令数十秒到几分钟内完成，没有触发「长活」条款，未使用 `run_in_background`。

### 1.5 计数与没做什么（云端攻方）

核了 39 处（模型目录完整性 21 个文件 + 引用 7 处 + 产物核对 8 处 + 复跑 3 次），**✓ 39，✗ 0，核不动 0**。

没做什么：
- 没有重跑阶段 A–E 的完整扫描（分别约 77 分钟、57 分钟等，累计数小时），只重跑了派发提示点名的 K4 原历史与
  「记录与根槽之间断电」那一段纯崩溃历史；阶段 A–E 的汇总数字只做了「产物里逐字找得到 + 内部算术自洽（分臂合计数）」的核对，
  没有从零重新扫一遍 156 条动作序列 × 21 个故障点。
- 没有核对「乙′」「checker 并集改法」这两条推的、被攻过零轮的改法本身对不对——报告自己已注明「只在我的模型上量过或推过」。
- 没有判 K1、K3、K5、K6，不在这条腿的任务范围。
- 没有对第七、八节列出的「没打中的形状」逐条复算，那些是范围声明，不是待验证的量化结论。

## 二、云端辩方（Sonnet）

腿报告 `research/prompts/c381-r1-sonnet-output.md`（sha256 `50cf19a723b82e40a66cc4206a981c58c0101b429537de06a7d608f9cc234a4d`，
交回给的值，与本次核查现算的 sha256 一致）。这条腿没有模型目录、没有复跑命令（报告自己写明「没有跑代码、没有编译」），
核的是它的全部 `文件:行号 + 引原文` 与一处 grep 结论。

| 引用（报告行号） | 核的结果 | 命令 |
|---|---|---|
| `transaction.rs:1259-1260` `if outcome.is_err() { *allocator = allocator_before_this_publish; }`（第 11 行） | ✓ | `awk 'NR==1259,NR==1260' crates/singlefs-core/src/transaction.rs` |
| `invariants.md:53`（I-7.4 定义，第 12 行） | ✓（第 53 行整行即 I-7.4 表格行，引文「回退候选集里每一个根……也未被清扫抹头」逐字在内） | `awk 'NR==53' .claude/kb/invariants.md` |
| `decisions/22-单元原子性怎么合成.md:233`（已定项 16，第 13 行） | ✓（引文「每盘恒 2 个槽……择槽取校验和过且世代号最大的」逐字在该行内） | `awk 'NR==233' ".claude/kb/decisions/22-单元原子性怎么合成.md"` |
| `invariants.md:51`（I-7.2 定义，第 27 行） | ✓ | `awk 'NR==51' .claude/kb/invariants.md` |
| `transaction.rs:172-174`（`RotateSuperblockSlots` 逐盘调用 `write_superblock_slot`，第 13 行） | ✓（172–174 行正是 `for index in 0..self.devices.len() { self.write_superblock_slot(...)?; }` 循环体，失败经 `?` 整体返回 `Err`，与引用的行为描述一致） | `awk 'NR==172,NR==174' crates/singlefs-core/src/transaction.rs` |
| docstring（1223 行）「准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立」（第 11 行） | ✓（第 1223 行整行含此子句，前半「按 D16……的持久顺序落盘」未引，属合理截取，非摘句违规） | `awk 'NR==1223' crates/singlefs-core/src/transaction.rs` |
| `invariants.md:12`「已实现（2026-09-17……层 0 每个崩溃状态都判）」（第 12 行） | **✗**：第 12 行是「池级 checker……层 0 的每个崩溃状态都跑它」，措辞不同（「都跑它」非「都判」），且不含「已实现（2026-09-17」这个起手；引用的这句原文实际是 **I-7.4 那一行（第 53 行）自己的状态列**：「已实现（2026-09-17，池级 checker `walk::check_pool_image`，按每条回退候选集里更早的根各判一格……层 0 每个崩溃状态都判）」。`invariants.md` 经查此刻无未提交改动，不设「分不清」 | `awk 'NR==12' .claude/kb/invariants.md`；`awk 'NR==53' .claude/kb/invariants.md` |
| `checks-owed.md:355`（C381 自己那一行，第 24、37、46 行） | **分不清**：现在的工作区第 355 行是别的条目（C1/C2 事务层那一段），C381 现在实际在第 348 行；对 `git show HEAD:.claude/kb/checks-owed.md` 取同一行号，第 355 行正是 C381 那一整行、内容与引文逐字相同。`git diff --unified=0` 显示另一会话未提交的「决策正文瘦身」在第 355 行之前删了 7 行（`@@ -86 +85,0@@`、`-254 +252,0@@`、`-298/-321/-331/-334/-337 各 +N,0@@` 共 7 处单行删除），355 − 7 = 348，与工作区现状吻合 | `git show HEAD:.claude/kb/checks-owed.md \| awk 'NR==355'`；`awk 'NR==348' .claude/kb/checks-owed.md`；`git diff --unified=0 -- .claude/kb/checks-owed.md \| grep -E '^@@'` |
| 全仓 grep「不许再发布\|不许接着发布……」唯一命中 `checks-owed.md` 的 C381 行（第 24 行） | ✓（现在重跑同一条 grep，唯一命中仍是 C381 那一行，行号 348，内容含「失败之后这个写入口还许不许接着发布」） | `grep -rn "不许再发布\|不许接着发布\|不许继续发布\|此后不许发布\|失败之后.*不许.*发布\|发布失败.*不许" .claude/kb/` |
| 背景材料第 50–56 行候选表（辩护 D，甲/乙/丙/丁 S3 行为） | ✓ | `awk 'NR==50,NR==56' research/prompts/_c381-r1-background.md` |

计数：核了 10 处，**✓ 8，✗ 1（`invariants.md:12`），分不清 1（`checks-owed.md:355`）**。

没做什么：
- 没有重跑代码或 checker——这条腿本身没有产物或模型目录，没有可复跑的东西。
- 没有判 K1–K5，不在这条腿的任务范围（报告自己写明）。
- 没有替攻方找反例、没有评估辩护 A–D 之外的其他可能读法穷尽性——这两点报告自己在「没做什么」里已如实列出，不是本核查的欠账。
- `invariants.md:12` 那一处 ✗ 只影响引用的行号，不影响它所支撑的论点本身（I-7.4 已实现、已被 checker 判定为不变量这句话，核到 kb 文件里确有其事，只是所指的具体行号错了，出处应改指第 53 行）——这句判断由主 agent 做，这里只报观测。

## 三、本地攻方

四份材料：提示 `research/prompts/c381-r1-local-attack.md`、转述核对表
`research/prompts/c381-r1-local-attack-translation-audit.md`、样本 `-output-s1.md` / `-output-s2.md`（`-output-void-gateway1.md`
是网关报错留下的 0 字节文件，不算样本）、运行记录 `-local-attack-runlog.md`。这条腿没有独立“报告”文件，核的是转述核对表里
每一处「原文文件:行」、以及运行记录里可复核的工具判定。

### 3.1 转述核对表——逐处「原文文件:行」

| 引用（核对表行） | 核的结果 | 命令 |
|---|---|---|
| Fact 1 → `decisions/16-发布语义.md:287-288` | ✓ | `awk 'NR==287,NR==288' ".claude/kb/decisions/16-发布语义.md"` |
| Fact 2 → `decisions/16-发布语义.md:293` | ✓ | `awk 'NR==293' ".claude/kb/decisions/16-发布语义.md"` |
| Fact 5 → `decisions/23-journal的角色与格式.md:691`（「槽」/「区域」区分） | ✓（内容含「不推进就是反复重写同一个槽、把 R 个失败域用成一个」；核对表转写成「……槽，把……」，顿号写成逗号，标点小误、不改语义，不计 ✗） | `awk 'NR==691' ".claude/kb/decisions/23-journal的角色与格式.md"` |
| Fact 6 part 5 → `decisions/23-journal的角色与格式.md:1225` | ✓（引文含「否则一次落在 R_old 上……回退被静默撤销」半句，该半句实际连续到第 1226 行——核对表只写单一行号 1225，未写成区间 1225-1226，属轻微行号范围不精确，内容本身逐字准确，不计 ✗） | `awk 'NR==1225,NR==1226' ".claude/kb/decisions/23-journal的角色与格式.md"` |
| Fact 9（不译的例子）→ `_c381-r1-background.md:34` | ✓ | `awk 'NR==34' research/prompts/_c381-r1-background.md` |
| S1 stage → `_c381-r1-background.md:42` | ✓ | `awk 'NR==42' research/prompts/_c381-r1-background.md` |
| S2 stage 主句 → `_c381-r1-background.md:42` | ✓ | 同上 |
| **S2 stage 附注「两块盘的事实见 `decisions/16-发布语义.md:542`」** | **✗，误写成背景材料第 542 行，原文件（`decisions/16-发布语义.md`）实为第 208 行**：工作区与 `git show HEAD:` 该文件第 542 行均不存在（全文件只有 423 行）；`_c381-r1-background.md` 第 542 行恰好逐字是「……两次正好覆盖两块盘……」；该句在 kb 原文件里出现在第 189、208、221、291 行，第 208 行是最完整的一处（含「两次正好覆盖两块盘 ⇒ 次数由归属唯一确定」整句） | `wc -l ".claude/kb/decisions/16-发布语义.md"`；`awk 'NR==542' research/prompts/_c381-r1-background.md`；`grep -n "覆盖两块盘" ".claude/kb/decisions/16-发布语义.md"` |
| Arm A stage S4 → `_c381-r1-background.md:52` | ✓ | `awk 'NR==52' research/prompts/_c381-r1-background.md` |
| mount.rs 注释 → `crates/singlefs-core/src/mount.rs:849` | ✓ | `awk 'NR==849' crates/singlefs-core/src/mount.rs` |
| Fact 3 reason 1 → `decisions/16-发布语义.md:91` | ✓ | `awk 'NR==91' ".claude/kb/decisions/16-发布语义.md"` |
| Fact 4 主引用 → `decisions/16-发布语义.md:399`（「最少保留 4 个状态」） | ✓ | `awk 'NR==399' ".claude/kb/decisions/16-发布语义.md"` |
| **Fact 4 附注「16-发布语义.md:615『最少保留4个』」** | **✗，误写成背景材料第 615 行，原文件（`decisions/16-发布语义.md`）实为第 361 行**：文件只有 423 行，第 615 行不存在；`_c381-r1-background.md` 第 615 行逐字是「……最少保留 4 个……『4』数的是可退到的不同状态」；kb 原文件里同一句在第 361 行（「2026-09-11 用户定案盘紧时候选集可以一直缩、最少保留 4 个……2026-09-12 用户澄清『4』数的是可退到的不同状态」），一次命中，与核对表引文逐字相同 | `awk 'NR==615' research/prompts/_c381-r1-background.md`；`grep -n "最少保留" ".claude/kb/decisions/16-发布语义.md"` |
| Fact 11 → `decisions/23-journal的角色与格式.md:1206`「一带」 | 核对表自己标「一带」（未点名精确行号），不作为精确引用核；第 1206 行附近确有「链接得上」一类表述所在的记录格式讨论段落，属合理的近似指路，不计 ✗、不计 ✓ | `awk 'NR==1200,NR==1210' ".claude/kb/decisions/23-journal的角色与格式.md"` |
| Fact 8「resulting version object」← `TransactionOutput`（无行号声明） | 无行号声明，跳过（不是本节核的对象） | — |

`decisions/16-发布语义.md` 此刻有另一会话未提交的改动（只涉及第 107–113 行「已定项 4」一处单行替换，未增删行数），
`decisions/23-journal的角色与格式.md` 此刻无未提交改动；上表两处 ✗（542、615 行）与工作区改动所在位置（107–113 行）无关，
`git show HEAD:` 同一份文件同样是 423 行、同样没有第 542、615 行——两处按定义判 ✗，不判「分不清」。

### 3.2 运行记录——复跑字词损坏闸（草稿目录 `checker-rerun/` 里的副本）

运行记录 `-local-attack-runlog.md` 第 11、60 行给出的调用方式是
`nice -n 19 bash research/scripts/ask-local.sh research/prompts/c381-r1-local-attack.md`；`ask-local.sh` 第 86 行起对两个检测器的
调用是 `python3 "$CHECK" "$TXT" "$1"`——第二个参数是提示文件本身。在草稿目录里把两份样本原样拷贝后，按同样的两参数形式复跑：

```
$ cp research/prompts/c381-r1-local-attack-output-s{1,2}.md /tmp/claude-1000/c381-r1-verifier/checker-rerun/
$ python3 research/scripts/oov-check.py checker-rerun/s1.txt research/prompts/c381-r1-local-attack.md
绿 checker-rerun/s1.txt  生词=15 拼接=0
     生词: advancement
$ python3 research/scripts/oov-check.py checker-rerun/s2.txt research/prompts/c381-r1-local-attack.md
绿 checker-rerun/s2.txt  生词=0 拼接=0
$ python3 research/scripts/corruption-check.py checker-rerun/s1.txt
绿 checker-rerun/s1.txt  cjk=0 words=1694 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0
$ python3 research/scripts/corruption-check.py checker-rerun/s2.txt
绿 checker-rerun/s2.txt  cjk=0 words=2000 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0
```

（`oov-check.py`、`corruption-check.py`、`en-words.txt` 经查此刻均无未提交改动，与本轮腿运行时状态相同——最后一次改动是
`852e4b3`（2026-09-19 01:40 UTC），早于本地攻方腿运行时刻 06:09–06:27 UTC，且那次改动只给 `oov-check.py` 的自检分支加
`# gate-lint:detail` 注释，不改 `scan()` 的匹配逻辑，故不设「分不清」。）

样本文件比对：`wc -w`、`diff`、`sha256sum` 均确认草稿副本与仓里原文件逐字节相同（s1 1765 词、s2 2056 词，与运行记录第 11、60 行
一致）。corruption-check.py 两份样本的复跑结果与运行记录第 11、60 行给出的判定**逐项相同（✓）**。

oov-check.py 的复跑结果与运行记录**不同**：

| 样本 | 运行记录声称（第 11/60 行） | 本次复跑（同样两参数调用） | 判定 |
|---|---|---|---|
| s1 | 绿，生词=1（advancement，非拼接） | 绿，**生词=15**（advancement，非拼接） | **✗**：判红/判绿结论一致（都绿），但工具原样输出的数字不一致——`advancement` 在正文里出现 15 次（`grep -o -i advancement checker-rerun/s1.txt \| wc -l` = 15），脚本按「出现次数」计数、不按去重词数计数，第 249 行 `dict.fromkeys(oov)` 只用于折叠**显示**、不折叠**计数**（第 243 行 `len(oov)`）；运行记录把展示出来的一个词名误当成了计数 |
| s2 | 绿，生词=1（checkpoint's，非拼接，是所有格缩写） | 绿，**生词=0** 拼接=0 | **✗**：`checkpoint's` 逐字出现在提示文件 `c381-r1-local-attack.md`（Fact 9：「the checkpoint's txg number is advanced by one」，`grep -c -o -i "checkpoint's" research/prompts/c381-r1-local-attack.md` = 1），因此会被 `oov-check.py` 的第二参数（`known` 集合，取自提示文件）排除，按 `ask-local.sh` 实际调用形式根本不会被计入生词；只有**省略第二参数**单独手跑 `oov-check.py checker-rerun/s2.txt`（不带提示文件）才会复现「生词=1，checkpoint's」——本次单独试过（见下），确认这正是数字的来源 |

补充复现（不带第二参数，验证上表判断的来源）：

```
$ python3 research/scripts/oov-check.py checker-rerun/s1.txt
绿 checker-rerun/s1.txt  生词=15 拼接=0
$ python3 research/scripts/oov-check.py checker-rerun/s2.txt
绿 checker-rerun/s2.txt  生词=1 拼接=0
     生词: checkpoint's
```
不带第二参数时 s2 复现出「生词=1，checkpoint's」，与运行记录的数字吻合；s1 不带参数同样是 15（`advancement` 不在提示文件里，
带不带第二参数对 s1 无影响）。**这说明运行记录表里的两个 oov-check.py 判定，更像是省略提示文件参数单独手跑得到的，
不是 `ask-local.sh` 内部按第 86 行那样两参数调用时会产生的输出**——而 `ask-local.sh` 第 87–91 行的判红分支只在 `crc` 非 0
时才把 `$VERDICT` 打到 stderr，`crc=0`（绿）时第 89 行 `: ;;` 什么都不打印，运行记录表里这两个绿色判定的具体数字因此不可能是从
这次 `ask-local.sh` 调用的标准输出/错误里直接抄下来的，只能是另外单独跑出来再填进表格的。

判决由主 agent 做：这两处✗不改变「s1、s2 均无字词损坏、判绿」这个最终结论本身（不管带不带第二参数，两份样本都不触发
`splice_of` 判红），但运行记录表格里具体的「生词=N」数字与例词来源与实际工具行为对不上，且**s2 那一格给出的例词按工具实际
调用方式根本不会出现**，这一点需要主 agent 判断是否影响对这条腿「通读复查」步骤可信度的评估。

### 3.3 计数与没做什么（本地攻方）

转述核对表核了 13 处「原文文件:行」（另 2 处不计入：Fact 11 自己标「一带」未作精确声明、Fact 8 无行号声明），
**✓ 11，✗ 2**（均为「误写成背景材料行号」，已在 3.1 表格内逐条给出原文件实际行号）。
运行记录复跑了 6 项工具判定（oov-check.py × 2、corruption-check.py × 2、样本词数 × 2），**✓ 4，✗ 2**（均为 oov-check.py 的
「生词=N」计数与例词，判绿/判红结论本身不受影响）。合计核了 19 处，**✓ 15，✗ 4，核不动 0**。

没做什么：
- 没有核样本 s1、s2 对提示里第 1–5 题的作答内容本身对不对——那是 K1、K3 的实质判断，归主 agent。
- 没有重跑本地模型（Qwen3-Next-80B via ai-center 网关）本身——这不是「产物」或「复跑命令」的范畴，S1/S2 的正文
  是模型的一次性输出，核的只是对它的机械检测（字词损坏闸）与转述核对表，不是让模型再答一遍。
- 没有核转述核对表里「首稿缺的」那一栏所描述的编辑过程本身真实与否（比如「首稿写成……」）——那是过程自述，无法脱离
  当事会话的历史记录去验证，不属于「文件:行号 + 抄的原文」这一类可核对象。
- 没有查网关报错那次（`c381-r1-local-attack-output-void-gateway1.md`，退出码 3）的具体缘由，运行记录已如实留存原样报错文本，
  不构成待核的引用或产物声明。

## 四、没做什么（总）

- 不判一条打中成不成立、该不该采纳；不核推理本身（乙 S2 那一格的四句分析、甲在 S3/S2′ 上算不算缺陷、K6 站不站得住这些
  判断留给主 agent），只核引用、产物与复跑。
- 没有编译整仓门禁、没有跑 `gate.sh`。
- 没有对三条腿之间「一致 / 不一致」做任何评判——三条腿分工不同（K1/K3、K2/K4、K6），本轮任务范围不含裁决它们是否指向
  同一个结论。
- 除报告文件与草稿目录 `/tmp/claude-1000/c381-r1-verifier/`（含 `repo/`、`pristine/`、`checker-rerun/`、`selftest/`、
  三份重跑日志 `k4-rerun.log`、`k4-pristine-rerun.log`、`pristine-crash-rerun.log`）外没有写过任何文件；三份重跑日志与两份
  副本仓（各 154 MB）留在草稿目录，不入 `research/results/`——它们只是复核用的临时产物，实质数据已原样写进本报告。
