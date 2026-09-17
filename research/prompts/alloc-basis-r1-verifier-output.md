# 核查报告：alloc-basis-r1

核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证（开工先做）

抽 Opus 攻方腿附录引用之一：`.claude/kb/decisions/28-挂载期承诺量.md:21`
（可用公式那一行）。在草稿目录的副本里，把这一行的行号加 1（在原第 21 行之前插入一个空行，
让原内容后移到第 22 行），按第 2 步方法核（取引用给的行号那一行的内容，与被引原文逐字比对）。

```
$ sed -n '21p' .claude/kb/decisions/28-挂载期承诺量.md
可用 = Σ设备( 容量 − 已分配 − 不可回收 − defer 待释放 − 挂载期承诺量 − 被抛弃根独占量 ) − 待删占用 − 已承诺预留 − checkpoint 保留池
$ # 副本：把这一行加 1（原第 21 行之前插入空行）
$ sed -n '21p' /tmp/claude-1000/alloc-basis-r1-verifier/selftest/28-shifted.md
（空行）
$ sed -n '22p' /tmp/claude-1000/alloc-basis-r1-verifier/selftest/28-shifted.md
可用 = Σ设备( 容量 − 已分配 − 不可回收 − defer 待释放 − 挂载期承诺量 − 被抛弃根独占量 ) − 待删占用 − 已承诺预留 − checkpoint 保留池
```

**结果：判 ✗。** 副本第 21 行（引用给的行号）是空行，与被引原文不符，核查方法正确报出不匹配；
顺藤摸瓜找到内容实际去处（第 22 行）。判别力自证通过，核查方法分辨得出行号漂移。

## 一、Opus 攻方腿（`alloc-basis-r1-opus-output.md`，模型目录 `alloc-basis-r1-opus-model/`）

### 1.1 sha256 与复跑

草稿目录：`/tmp/claude-1000/alloc-basis-r1-verifier/opus-model/`（拷贝腿的模型目录，`nice -n 19` 在副本里跑，不在腿的原目录跑）。

| 文件 | 报告里的 sha256 | 副本里 `sha256sum` 实测 | 一致 |
|---|---|---|---|
| model.py | `e1b6e55f…4aea29` | 同 | ✓ |
| mutate.py | `79a3cf72…88d8` | 同 | ✓ |
| results.txt | `3695733a…71fd3` | 同 | ✓ |
| selftest.txt | `b868458f…a388a` | 同 | ✓ |
| history.txt | `ca70a3a3…fba10` | 同 | ✓ |
| scan.txt | `49b7ca05…4beeb5` | 同 | ✓ |
| mutations.txt | `517050da…51963` | 同 | ✓ |

7 处 sha256 全部核对逐字节相同（`sed` 抽表格与 `sha256sum` 输出逐字符比对，64 位十六进制全同）。

**复跑（`nice -n 19 python3 model.py [--selftest|--history|--scan]`、`nice -n 19 python3 mutate.py`），
产物与腿留存的原文件 `diff` 逐字节比对**：

| 命令 | 退出码 | 与报告留存产物 diff | sha256 一致 |
|---|---|---|---|
| `python3 model.py > results_rerun.txt` | 0 | 无差异（IDENTICAL） | ✓（同 `3695733a…`）|
| `python3 model.py --selftest > selftest_rerun.txt` | 0 | 无差异 | ✓ |
| `python3 model.py --history > history_rerun.txt` | 0 | 无差异 | ✓ |
| `python3 model.py --scan > scan_rerun.txt`（耗时 1m8s） | 0 | 无差异 | ✓ |
| `python3 mutate.py > mutations_rerun.txt` | 0 | 无差异（7 条全红） | ✓ |

5 份产物全部字节级复现；`mutations.txt` 报的「M2 是 selftest 里一个场景撞了模型自己的『同一单元释放两次』
断言而退出，不是某条 expect 判红」核实：单独重跑该条变异，Traceback 落在
`assert state == "allocated", f"单元 {identifier} 释放了两次"`，`AssertionError: 单元 2 释放了两次`——
与报告原样引述相符。


### 1.2 引用核对（kb 文件:行号、crates/ 文件:行号）

腿读的 4 个 crates 文件哈希（`6858e09c…`、`97cf1061…`、`7e4292e7…`、`92b2a44c…`）与正文第三节相同；
腿自己在 `/tmp/claude-1000/alloc-basis-r1-opus/snap/` 留了这个时点的快照（`git hash-object` 核对
逐字节相同），01:25 UTC 之后另一个会话又改了 `allocator.rs`/`transaction.rs`/`walk.rs`/`mount.rs`；
下表凡引这四个文件的行号，都在该快照上核（与腿报告的哈希一致，不是当前工作区）。

| 引用 | 抄的原文（节选） | 核的结果 | 命令 |
|---|---|---|---|
| `.claude/kb/decisions/28-挂载期承诺量.md:21` | 「可用 = Σ设备( 容量 − 已分配 − 不可回收 − defer 待释放 − 挂载期承诺量 − 被抛弃根独占量 ) − 待删占用 − 已承诺预留 − checkpoint 保留池」 | ✓ 逐字相符 | `sed -n '21p' .claude/kb/decisions/28-挂载期承诺量.md` |
| `28-挂载期承诺量.md:40`、`:42` | 「容量、已分配 \| I-3.1 的被审计对象」「defer 待释放 \| D16：checkpoint C 中产生的释放在 C 发布之前不得进入可分配集合」 | ✓ 逐字相符 | `sed -n '40p;42p' 28-挂载期承诺量.md` |
| `05-快照-空间记账机制.md:347`、`:358`、`:362` | 「defer 待释放 \| 第 5 项…」「1 \| 已分配字节 \| 分配、准入、I-3.1 \| 带 \| 不带」「5 \| defer 队列待释放（按代） \| defer 窗口、分配、准入 \| 带 \| 不带」 | ✓ 逐字相符 | `sed -n '347p;358p;362p' 05-快照-空间记账机制.md` |
| `.claude/kb/invariants.md:120`（附录 A3） | I-3.1 整行，含「2026-09-17 起『有效根』= 回退候选集里的根」 | ✓ 逐字相符 | `sed -n '120p' invariants.md` |
| `crates/singlefs-core/src/allocator.rs:107-114` | 「占着的槽数：仍分配的加上已释放、还在 defer 窗口里的……」 | ✓ 逐字相符（快照第 107-114 行） | `sed -n '105,116p' snap/allocator.rs` |
| `crates/singlefs-core/src/transaction.rs:1257` | 「// 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。」 | ✓ 逐字相符（快照第 1257 行） | `sed -n '1245,1261p' snap/transaction.rs` |
| `crates/singlefs-core/src/transaction.rs:74` | 「/// 根槽 FUA 写：落区域 `txg mod 3` 的槽 `(txg div 3) mod 8`……」 | ✓ 逐字相符 | `sed -n '74p' snap/transaction.rs` |
| `crates/singlefs-core/src/transaction.rs:899-904` | 「None => Vec::new(),」第一个事务不释放任何落点 | ✓（`None => Vec::new(),` 在第 902 行，落在引用区间内） | `sed -n '895,910p' snap/transaction.rs` |
| `16-发布语义.md:371,372,375,376`（附录 A2） | 可再分配、环里最旧有效根、生效、回退候选集 四行 | ✓ 逐字相符 | `sed -n '371p;372p;375p;376p' 16-发布语义.md` |
| `23-journal的角色与格式.md:1209`（附录 A4） | 管理员回退整段 | ✓ 逐字相符（长段落，逐字核对） | `sed -n '1209p' 23-journal的角色与格式.md` |
| `invariants.md:50`（附录 A5） | I-7.4 整行 | ✓ 逐字相符 | `sed -n '50p' invariants.md` |
| `crates/singlefs-core/src/allocator.rs:164-173`（Sonnet 也引） | `mark_released` 只加 `deferred_slots` | ✓ 逐字相符（快照第 172 行 `self.deferred_slots += span;` 是函数体唯一赋值） | `sed -n '164,173p' snap/allocator.rs` |
| `allocator.rs:209-216` | `mark_allocated` 已分配加、空闲减 | ✓ 逐字相符（第 214-215 行） | `sed -n '209,216p' snap/allocator.rs` |
| `crates/singlefs-checker/src/walk.rs:746-778` | 先走最新根、再走其余全部 `valid_roots` | ✓ 逐字相符（770 行 `walk.walk_root(&roots[newest_index]…, true)`；774-778 行 for 循环） | `sed -n '746,778p' snap/walk.rs` |
| `crates/singlefs-checker/src/image.rs:206-221` | `valid_roots` 不按实例表过滤、不看 F | ✓ 逐字相符 | `sed -n '206,221p' snap/image.rs` |


### 1.3 模型产物里的数字核对（在副本里复跑之后对照）

| 报告里引的数 | 产物文件 | 核的结果 |
|---|---|---|
| `scan.txt` 第 1、2 行「序列数 3937、状态数 40377、甲候post非0状态 0 / 12771」 | `scan_rerun.txt` | ✓ 逐字相符（复跑产物与腿留存产物逐字节一致） |
| 旁 1「扫描里 3937 条序列有 1664 条」 | `scan.txt` 第 1、3 行「含D23违例的序列: 1664」 | ✓ |
| 3.5「窄读法按槽比、按单元比 … 4608 个状态违例；候选豁免 0」 | `scan.txt` 第 1、3、5 行 | ✓ 逐字相符 |
| 2.2 history.txt「m2 不释放」txg 25/26/27 整段引文 | `history_rerun.txt` 第 6-8 行 | ✓ 逐字相符 |
| 2.2 history.txt「m2 在 A 释放」txg 26/27 整段引文 | `history_rerun.txt` 第 14-15 行 | ✓ 逐字相符 |
| 2.4 history.txt「txg 30 写失败」整段（29/30/31/32/53/54） | `history_rerun.txt` 第 19-24 行 | ✓ 逐字相符 |
| 2.4 `results.txt`「覆写31/53/54」pre/post 各 +10/+230/+240/+0 | `results_rerun.txt` 第 63、65、66、77、79、80 行 | ✓ 逐字相符 |
| 3.4 表「±71」及其余 6 行数据 | `results_rerun.txt` 第 242-321 行（16 段） | ✓ 逐格核对，8 行全部对应正确（甲候post 数值、走读(iii)换内容非空状态数均一致，含「0（八段都是 0）」这一行） |

### 1.4 工作区漂移的观测（核不动的一半）

Opus 报告第九节自陈 01:25 UTC 后另一个会话改了 4 个 crates 文件与 `layout/01-first-txn.md`。
现查当前工作区：

```
$ git hash-object crates/singlefs-core/src/allocator.rs crates/singlefs-core/src/transaction.rs \
    crates/singlefs-checker/src/walk.rs crates/singlefs-core/src/mount.rs .claude/kb/layout/01-first-txn.md
a2759761e11cbe25bf9510cf7df5ddaf4a112e4d   # 与报告第九节记的新哈希相同
6dd2ef9a01801b325d16a517bd5b1c9e57c2288f   # 同上
6f539f57db4ece2971418799c06cf34eaffdf9c2   # 同上
431d050b5b0408250ea151b1fe63d4a6a186604a   # 同上
c8fa8459b6ec5065f7a833a04ca84a418a69b524   # 与报告记的「210cd201…」不同——layout/01-first-txn.md 又被改过
```

四个 crates 文件的当前哈希与报告第九节记的「新版本」哈希逐字相同——那次改动之后没有再变；
`layout/01-first-txn.md` 又变了一次（报告写完之后），**核不动**：报告第九节对这份文件的引用只对
它当时读到的那个哈希成立，当前内容已经是第三个版本，我没有再往下追。这不构成对报告的否定——
报告自己已声明这几处是「没量」「推的」，且给出了每一句的判断依据。

顺带一提：当前 `layout/01-first-txn.md` 第 82 行已出现「2026-09-17 里程碑『第二个事务』步 5 逼出：
不释放它，txg 0 的根离开候选集之后这一槽永远占着、I-3.1 红」，与 Opus 报告 Y3-a 的攻击结论
（m2 从不释放导致 I-3.1 迟早红）方向一致；这是另一个会话独立在改代码，不是本轮论证的产物，
仅作背景记录，不代入判断。


### 1.5 Opus 腿计数

核了 41 处（用命令数出来，不是手数）：7 处 sha256（1.1 表，`awk` 数据行数）+ 5 处产物复跑对比 + 1 处
mutate.py M2 断言文本核对 + 15 处文件:行号引用（1.2 表）+ 8 处产物数字核对（1.3 表）+ 5 处工作区当前哈希
核对（1.4，4 个 crates 文件 + 1 个 kb 文件）。附录 A1-A5 已计入 15 处引用表中，不重复计数。
**✓ 40 处、✗ 0 处、核不动 1 处**（`layout/01-first-txn.md` 的当前内容，因工作区在本轮核查期间被第三次修改，
其余 4 个 crates 文件当前哈希与报告第九节记的「新版本」哈希逐字相同，判 ✓）。

## 二、Sonnet 正推腿（`alloc-basis-r1-sonnet-output.md`）

Sonnet 报告用脚本核对了附录 30 处引文与 kb 逐字节 diff（脚本在它自己 `/tmp/claude-1000/alloc-basis-r1-sonnet/`
下，未随报告交付、不在我的复跑范围），本节改用直接现查每处引用原文的办法独立核，覆盖它「四处发现」
与关键代码事实。

### 2.1 第一节「四处发现」（正推腿指出前提表摘要遗漏了什么）——任务书明确要求逐条核

| 发现 | 引用 | 抄的原文 | 核的结果 |
|---|---|---|---|
| ① 行三 df 报可用 | `.claude/kb/decisions/28-挂载期承诺量.md:78-95`，引句在第 90 行 | 「`df`：可用已经扣掉这一项，`df` 报可用（D3 已定项 9 第 1 条）；第四轮探针实测没扣时 > 60 块（容量 4000）就出 24 次假性 ENOSPC，扣了就是真 ENOSPC（D16 已定项 1）。」 | ✓ 逐字相符，第 90 行落在引用区间 78-95 内 |
| ② 行七 D16 已定项 1「准入」 | `.claude/kb/decisions/16-发布语义.md:358-416`，引句在第 377 行 | 「准入 \| 可分配 = min(可再分配 + 活元数据 − 保留池, df)；准入不够时先推空发布抬 F（写行那次发布之前不推：……）……」 | ✓ 逐字相符，含加粗的限定分句，第 377 行落在区间内 |
| ③ 行九 D3 已定项 9 ⚠️ 射程注 | `.claude/kb/decisions/03-空间分配.md:359-389`，引句在第 369 行 | 「⚠️ 界 3 是按字面判的，E139 上有越过它的格：……」 | ✓ 逐字相符（主 agent 已核，本次独立复核同样相符），第 369 行落在区间内 |
| ④ 行十二 R=3/S=8 出处归属 | `.claude/kb/decisions/22-单元原子性怎么合成.md`，引句在第 258 行 | 「每区槽数 S \|……⚠️ 具体取多少不是格式决策——格式承诺的是『S 是超级块字段 + 挂载时判区间』，取值由第一版实现按 D25 的负载形态定」 | ✓ 逐字相符（主 agent 已核，本次独立复核同样相符）；连带核了 `crates/singlefs-format/src/lib.rs:185` `ROOT_RING_SLOTS_PER_REGION: u64 = 8;` 确无 `format-const:` 标记（对照同文件 181 行 `SUPERBLOCK_SLOT_BYTES` 带标记），与发现④「S=8 是实现选的、不是冻结的格式常量」一致；`layout/01-first-txn.md:134` 「每区槽数 S \| 1 \| 8 \| D22 已定项 2」逐字相符 |

四处发现全部核实：Sonnet 的引文与判断（「表格摘要遗漏了限定/例外」）成立，不是摘句造假，是
概括时省略了限定句——报告自己也如实标注了这一点，不是隐瞒。


### 2.2 代码事实与 Y6 引用核对（抽样，与 Opus 共享的行号只算一次）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `allocator.rs:107-114` 注释 | ✓（同 1.2 表，已核） | 同上 |
| `allocator.rs:164-173` `mark_released` | ✓（同 1.2 表） | 同上 |
| `allocator.rs:209-216` `mark_allocated` | ✓（同 1.2 表） | 同上 |
| `transaction.rs:1245-1260` 三处 `per_device` 写记账行 | ✓ 逐字相符（`allocated_slots()`、`free_slots()`、`deferred_slots()` 各乘 `SLOT_BYTES`） | `sed -n '1245,1261p' snap/transaction.rs` |
| `walk.rs:746-778`、`image.rs:206-221` | ✓（同 1.2 表） | 同上 |
| `.claude/gate.d/51-admission-terms-covered.sh` 判据描述（第 72-74、113-119、141-155 行） | ✓ 脚本存在，行号区间与描述的「从代码块取『可用 =』一行、按『−』拆项，与表格左列逐一相等比较」逻辑吻合（现读脚本确认） | `sed -n '60,160p' .claude/gate.d/51-admission-terms-covered.sh` |
| `grep -rn "可用 = Σ设备" .claude/kb/decisions/*.md` 命中 4 处 | ✓ 复跑命中数一致 | 见下方命令与输出 |

```
$ grep -rn "可用 = Σ设备" .claude/kb/decisions/*.md | wc -l
4
```

### 2.3 Sonnet 腿计数

核了 7 处新引用（四处发现 4 + 2.2 表 7 行里除去 4 行与 Opus 1.2 表重叠——allocator.rs:107-114、
:164-173、:209-216、walk.rs:746-778/image.rs:206-221 那一行——剩下的 3 行新引用：
`transaction.rs:1245-1260`、门禁 51 号脚本描述、`grep` 命中数）。**✓ 7 处、✗ 0 处、核不动 0 处**（这 7 处）。
另有 1 项**不计入上面的 ✓/✗/核不动**、单列出来：Sonnet 报告自称「附录 30 处引文逐字节 diff 全部
IDENTICAL」，那份 diff 脚本住在它私有草稿目录，未随报告交付，本轮没有找到、没有复跑，这句自称
**核不动**——它说的「30 处」是附录层面的引用集合，与本节独立核对的 7 处（Sonnet 正文自己的引用）
不是同一批，不能互相顶替。


## 三、本地攻方腿（`alloc-basis-r1-local-attack-output-s1.md`/`-s2.md`）

### 3.1 字词损坏闸复跑

```
$ wc -w research/prompts/alloc-basis-r1-local-attack-output-s1.md
462 ...s1.md
$ nice -n 19 python3 research/scripts/corruption-check.py research/prompts/alloc-basis-r1-local-attack-output-s1.md
绿 ...s1.md  cjk=0 words=462 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0（exit 0）
$ nice -n 19 python3 research/scripts/oov-check.py research/prompts/alloc-basis-r1-local-attack-output-s1.md
绿 ...s1.md  生词=0 拼接=0（exit 0）
```
s2 同样复跑：`wc -w`=459，`corruption-check.py` 自报 `words=495`，全部计数 0，exit 0；`oov-check.py`
生词=0 拼接=0，exit 0。**与运行记录逐字相符**（包括「两个计数器口径不同」那句：`wc -w` 与脚本自报的
`words` 确实不同，459 对 495）。

### 3.2 翻译核对表核对（原表 22 行，本节抽核 15 行，逐条核引用原文）

| 核对表行（英文项节选） | 表里给的出处 | 核的结果 |
|---|---|---|
| oldest-valid-root 定义 (a)(b) 两个并列限定 | `16-发布语义.md:372` | ✓ 逐字相符 |
| 写失败的槽「按旧内容算」 | `16-发布语义.md:372` | ✓ 逐字相符（同一行） |
| 在飞发布「不算」 | `16-发布语义.md:372` | ✓ 逐字相符（同一行） |
| 可再分配 ⟺ 两个合取项 | `16-发布语义.md:371` | ✓ 逐字相符 |
| 两道闸先后顺序「先 D28 再 D16」 | `03-空间分配.md:430`（表述为 `28-挂载期承诺量.md:11` 索引行复述） | ✓ 逐字相符（现读 03-空间分配.md:430：「先过 D28（挂载期承诺量） 的『可用 ≥ 需求』，再过 D16（发布语义） 的抬 F 逻辑」） |
| 「df 报可用」 | `28-挂载期承诺量.md:90` | ✓ 逐字相符（同 1.2 表①） |
| 保留池/残留公式 | `16-发布语义.md:378` | ✓ 逐字相符 |
| 滞后量两个合取项 | `16-发布语义.md:378` | ✓ 逐字相符（同一行） |
| B=8、做满才报 ENOSPC | `16-发布语义.md:377` | ✓ 逐字相符 |
| df≥s 必须成功 | `03-空间分配.md:363` | ✓ 逐字相符 |
| 界 3、空发布不算 | `03-空间分配.md:365, 367` | **✗ 行号不准**：「界是 3 次改变用户可见状态的发布」实际在**第 367 行**（不是 365）；「文件系统自己推的空发布不算这 3 次」实际在**第 368 行**（不是 367）。现查见下方命令。内容本身逐字无误，只是行号标注偏了 1 行 |
| Candidate A 回退候选集定义 | `23-journal的角色与格式.md:691` 及 `16-发布语义.md:290` | **✗ 第二处行号错**：`16-发布语义.md:290` 处是「fsync 返回条件」段落，与「回退候选集 = 按实例表判仍然有效 ∧ txg ≥ F_生效」无关；该句实际在**第 376 行**。`grep -n "回退候选集" 16-发布语义.md` 只命中第 203、376、408 行，没有 290。`23-journal的角色与格式.md:691` 这一半核对无误（管理员回退定义段落） |
| Candidate C checker 并集范围 | `_alloc-basis-r1-body.md` 第三节 | ✓ 逐字相符（与 body.md 第 38 行 `valid_roots` 那一格一致） |
| `txg div R` 整数除法 | `22-单元原子性怎么合成.md:257`、`:1006` | ✓ 逐字相符（两处均含公式） |
| checkpoint_txg 推进一格 | `23-journal的角色与格式.md:691` | ✓ 逐字相符（长段落中含「不推进就是反复重写同一个槽、把 R 个失败域用成一个」） |

```
$ sed -n '363,369p' .claude/kb/decisions/03-空间分配.md | cat -n
     5	   **界是 3 次改变用户可见状态的发布**（……）：删掉的块被最近 4 个状态里更旧的 3 个钉着，
     6	   要再发生 3 次这样的发布才回可分配集合；……文件系统自己推的空发布不算这 3 次（空发布不改用户可见状态）。
$ grep -n "回退候选集" .claude/kb/decisions/16-发布语义.md
203:...
376:| 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |
408:...
```

### 3.3 本地攻方腿计数

核了 19 处：3.1 节 4 处字词损坏闸复跑（s1 corruption-check、s1 oov-check、s2 corruption-check、
s2 oov-check）+ 3.2 节核对表 15 行（`awk` 数据行数）。**✓ 17 处、✗ 2 处（行号偏差，内容本身无误）、
核不动 0 处。**


## 四、本地辩方腿（`alloc-basis-r1-local-defense-output-s1.md`/`-s2.md`/`-void1.md`）

### 4.1 字词损坏闸复跑（标准调用：带提示文件为第二参数，与 `ask-local.sh` 内部调用一致）

```
$ nice -n 19 python3 research/scripts/corruption-check.py .../s1.md   → 绿 words=462... exit 0（与运行记录一致）
$ nice -n 19 python3 research/scripts/oov-check.py .../s1.md .../alloc-basis-r1-local-defense.md
绿 ...s1.md  生词=0 拼接=0
$ nice -n 19 python3 research/scripts/oov-check.py .../s2.md .../alloc-basis-r1-local-defense.md
绿 ...s2.md  生词=0 拼接=0
$ nice -n 19 python3 research/scripts/oov-check.py .../void1.md .../alloc-basis-r1-local-defense.md
红 ...void1.md  生词=2 拼接=1  拼接: reclaimedclaimed(=reclaimed+claimed)  生词: rechecked reclaimedclaimed
```

**void1 的数字（生词=2、拼接=1）与运行记录逐字相符**；但 s1、s2 的 oov 数字**核不完全对上**：
运行记录写的「s1 生词=4」「s2 生词=1」那两组词表，是不带提示文件第二参数单独跑 `oov-check.py`
时的结果；`ask-local.sh` 源码里实际调用是 `python3 "$CHECK" "$TXT" "$1"`（带提示文件），按这个
真实调用法复跑，s1、s2 的生词都是 0，不是运行记录写的那两个数：

```
$ nice -n 19 python3 research/scripts/oov-check.py .../s1.md   # 不带提示文件（运行记录数字的来源）
绿 ...s1.md  生词=4 拼接=0   （词表两词，其中一词含撇号，此处省略原样文本）
$ nice -n 19 python3 research/scripts/oov-check.py .../s2.md   # 不带提示文件，复核 s2 是否同一模式
绿 ...s2.md  生词=1 拼接=0   （同上一个词）
```

s1、s2 不带提示文件时的生词数字（4 与 1）与运行记录逐字相符，证实运行记录的两个数字确实来自
不带提示文件的调用法，而不是 `ask-local.sh` 真实会跑的调用法（带提示文件、生词都是 0）。

**这不影响「判定：干净」这句结论**：`oov-check.py` 源码第 189 行 `red = len(spliced) > 0`——
红绿只看拼接数，不看生词数；s1、s2 两种调用法拼接数都是 0，所以两种算法都判「绿」。**误差只在
生词数字这一栏**：s1、s2 各记一处数字对不上（判 ✗），运行记录「判定：干净」本身没有错。

### 4.2 F1–F15 事实清单核对（任务书点名的 F1 及相关几条）

提示文件 `alloc-basis-r1-local-defense.md` 里实际编号到 **F15**（不是 F1–F14；F1–F13 是主体事实、
F14 是例数、F15 是另一条独立的在飞合成规则）。逐条读了 F1–F15 原文（见上方 grep 输出），F1 本身
只是「外部公式」的定义加两个术语的理由，**没有任何一句提到「统计量每次发布刷新」或「第一道闸在
每次推空发布之后重判」**。

样本 2 的第 2 题答案写「the statistics are refreshed on every publish (F1), so ... the outer formula
is re-evaluated after each publish push」，把这个说法挂在 F1 头上——**核实：F1 原文不支持这句话**。
最接近的依据其实是 F13（「Each publish writes statistic item 1 ... directly from the allocated_slots
counter」），但 F13 只说统计量随每次发布被写入，**没有任何一条 F 项说明『外层公式会在推空发布过程中
被重新判一次』这件事**——这是样本 2 自己的推论，被误标成 F1 给出的事实。

样本 1 对同一题给出**相反**的答案：「F5 states that pushing empty publishes to raise F does not
change user-visible state, so statistics ... remain unchanged. F3 specifies the gates are in series:
outer formula is checked first, and if it fails, the inner formula's raise-F logic is never reached.」
核实：F3、F5 原文（见上方 grep 输出）确实支持这个读法——F3 说两道闸串联、先外后内；F5 关于「非空」
的定义确实排除了推 F 的空发布。样本 1 这条引用比样本 2 更贴合材料原文。

**两份样本在这一题上给出相反结论**，按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算
一次观测」，这属于观测不稳定，不下结论。现查 `alloc-basis-r1-local-defense-runlog.md` 与它的翻译核对表：
两份文件里都**没有像本地攻方腿运行记录那样列一张「两份样本之间的差异」表**——`grep -n "差异\|不一致\|分歧"`
在这两份文件与提示文件里零命中。本地辩方腿只核对比较了字词损坏闸的指标（退出码、词数、生词/拼接），
没有留意到 s1、s2 在 Y2 这一题上给出了字面相反的答案（一个说外层公式会被重新判、一个说外层公式检查
一次就把请求直接拒了）——这一处内容分歧完全没有被记录下来，是这份材料的一处空白，不是「漏记一条」
那么小。


### 4.3 F1–F15 与其余引用逐条核（对照原始 kb 条款，抽样）

| F 项 | 涉及的 kb 出处（材料本身没写文件:行号，属英文转述，按内容比对原文） | 核的结果 |
|---|---|---|
| F1 外层公式 | `28-挂载期承诺量.md:21`（式子）、`:40,42`（两项理由） | ✓ 内容与原文相符（式子、两条理由都译对） |
| F3 两道闸串联 | `03-空间分配.md:430` | ✓ 内容相符（先外后内） |
| F5 可再分配、F 生效上界 | `16-发布语义.md:371,372,375` | ✓ 内容相符 |
| F6 根环参数、t−24 | `22-单元原子性怎么合成.md:1006`、`layout/01-first-txn.md:134`（S=8 现值） | ✓ 内容相符 |
| F7 实例表有效性 | `23-journal的角色与格式.md:1209` 段内 (i,T) 判据句 | ✓ 内容相符 |
| F9 回退机制、隔离量 | `23-journal的角色与格式.md:1209` 全段 | ✓ 内容相符（含「只隔离只被被抛弃根引用的槽」） |
| F10 I-3.1 | `invariants.md:120` | ✓ 内容相符，含「2026-09-17 起」新读法 |
| F13 代码事实 | `allocator.rs`/`transaction.rs`/`walk.rs`/`image.rs` 对应段 | ✓ 与 1.2 表已核代码段一致 |
| F14 例数 23/10 槽 | `_alloc-basis-r1-body.md` 第三节「这条脚本上的数」 | ✓ 与 body.md 逐字相符（未独立重放，body.md 自己也标「里程碑步 2 现状」文档自陈值） |

### 4.4 本地辩方腿计数

核了 14 处：4.1 节 4 处字词损坏闸复跑（corruption-check、void1 的 oov-check 判定相符 2 处 ✓；
s1、s2 的 oov 生词数字与真实调用法对不上 2 处 ✗）+ 4.2 节 1 处引用误标发现（样本 2 把一句 F1 原文
不含的推论挂在 F1 头上）+ 4.3 节 9 处 F 项事实核对。**✓ 11 处、✗ 3 处、核不动 0 处**；另记 1 处
材料空白（4.2 段：s1、s2 在 Y2 第 2 题给出字面相反答案，这处内容分歧完全没被本地辩方腿的运行记录
提及，不计入 ✓/✗，因为它不是引用核对，是覆盖面观察）。

## 五、全轮汇总

| 腿 | 核了 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| Opus 攻方 | 41 | 40 | 0 | 1 |
| Sonnet 正推 | 7（另有「附录 30 处引文 diff」这句自称，未复现，不计入本表） | 7 | 0 | 0 |
| 本地攻方 | 19 | 17 | 2（行号偏差 1 行，内容无误） | 0 |
| 本地辩方 | 14 | 11 | 3（s1/s2 生词数字各 1 处 + F1 被误引 1 处） | 0 |
| **合计** | **81** | **75** | **5** | **1** |

（数字均由 `awk`/`grep -c` 数表格行数或逐条列出后手数得到，命令见各节；Sonnet 自称的「附录 30 处
引文 diff」是另一批引用集合，未随报告交付脚本、本轮没有复现，单独作为「没做什么」的一项，不计入
上表的核了/核不动列，避免把两批不同的引用集合混进同一个数字里。）

## 没做什么

- **不判一条打中成不成立、该不该采纳**：Opus 报告里「打中／没打中」的判断、Sonnet 的「要改哪些地方」、
  两条本地腿对 Y2/Y5/Y7 的回答，本报告一概不予评判是否正确、是否该被主 agent 采纳——只核引用、
  产物、复跑。
- **不核推理本身**：例如 Opus 报告第 2.4 节「为什么只改时点修不掉」那段推理链是否成立，或 Sonnet
  报告 3.1-3.7 节「要改哪些地方」的推导是否周全，均不在核查范围。
- **不核 Y1-Y7 判据表本身定得对不对**、不核主 agent 的倾向该不该被推翻。
- **Sonnet 报告自称的「附录 30 处引文逐字节 diff 全部 IDENTICAL」完全没有复核**：那份 diff 脚本住在
  它私有草稿目录，未随报告交付，没有找到（不在我的写范围/读范围内主动去找）。本报告独立核对的是
  Sonnet 正文自己引用的 7 处（四处发现 4 处 + 与 Opus 报告不重叠的代码/门禁引用 3 处），这 7 处与
  Sonnet 自称的「30 处」是不同的引用集合，验证前者不能顶替后者。
- **Opus 报告第五节 G1-G6 改法「被攻过零轮」这件事本身不复核**——报告自己已经如实标注，不需要我再验证。
- **不编译、不跑 `crates/`**：所有 crates/ 源码核对都是用 `sed`/`git hash-object` 读文本，没有构建、
  没有跑单测。
- **layout/01-first-txn.md 当前内容与 Opus 报告第九节的「新版本」引用**：工作区在本轮核查期间被第三方
  会话又改了一次，第九节那几条引用现在核不动（见 1.4 节），不构成对 Opus 报告本身的否定——报告写作
  时那处引用是准确的。
- **未跑 `.claude/gate.d/stage-owners.tsv` 登记表**（不属于本轮任务范围，任务书未要求）。
- **未核 `alloc-basis-r1-local-attack.md`/`-local-defense.md` 提示文件全文的每一句转述**：只核了两份
  核对表列出的行（本地攻方 15 行 + 本地辩方 9 项 F），提示文件里没有被核对表点名的部分未逐句比对。
