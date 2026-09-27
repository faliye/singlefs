# m2-safety-r3 三方核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证

抽 Sonnet 报告里的一条引用：`.claude/kb/decisions/16-发布语义.md:36`（声称是「抬 F 的上限」那一行）。
在草稿目录副本 `/tmp/claude-1000/m2-safety-r3-verifier/selftest/sonnet-output-corrupted.md` 里把行号
`:36` 改成 `:37`，按下面「做什么」第 2 步核：到 `.claude/kb/decisions/16-发布语义.md` 取第 37 行——

```
第 37 行原文：| 生效 | F 另记进每块盘的系统配置（「抬 F 那一串」那一行写明先写后抬）；恢复后生效值 F_生效 = max(...
```

第 37 行是「生效」那一行，不是「抬 F 的上限」那一行——内容对不上，**判 ✗**。判别力确认：方法分辨得出错误行号。

## 一、sha256 核对（独立复核）

| 对象 | 命令 | 结果 |
|---|---|---|
| 快照 crates 哈希（130 个）vs 冻结副本 | `cd /tmp/claude-1000/safety-r3-frozen && sha256sum -c .../crates-sha256sums.txt` | 130/130 OK（`grep -v ': OK'` 输出为空） |
| 快照 crates 哈希（130 个）vs 主树 `crates/` | 同上命令对主树跑 | 130/130 OK |
| 快照 kb 哈希（7 个）vs 主树 | `sha256sum -c research/prompts/m2-safety-r3-snapshot/kb-sha256sums.txt` | 7/7 OK |
| `m2-safety-r3-sonnet-output.md` sha256 | `sha256sum m2-safety-r3-sonnet-output.md` | `7d5f82dc0d53b6388ee2e526f6955c4595b2de3202ba2084db62d6aaeaf23f29`，与交回给的前缀 `7d5f82dc` 一致 |
| `m2-safety-r3-opus-output.md` sha256 | 同上 | `b6c9bb61c0f6a13c1dc4ed94c88dfed863e09c0624e2e4455f3503a6af4ca055`，与交回给的整串**逐字符**一致 |
| `m2-safety-r3-opus-model/SHA256SUMS`（40 个文件） | `cd .../m2-safety-r3-opus-model && sha256sum -c SHA256SUMS` | 40/40 OK |
| `m2-safety-r3-sonnet-model/SHA256SUMS`（8 个文件） | 同上 | 8/8 OK |

三方主 agent 派我之前刚核过的两份（130、7）复核结果相同；本轮我独立重跑，不是抄主 agent 的结论。

## 二、特别核：冻结副本被改过这件事

### 2.1 攻方（Opus）开工时拷的那一份，是不是早于 04:31

- `/tmp/claude-1000/m2-safety-r3-opus/tree` 目录自身 mtime：`2026-09-26 03:53:49.856951908 +0000`；
  快照文件 `research/prompts/m2-safety-r3-snapshot/crates-sha256sums.txt` 的 mtime：
  `2026-09-26 03:53:49.866725347 +0000`——两者相差 10 毫秒，即这份拷贝与开工快照几乎同时生成，**早于 04:31**。
- **树相对快照的差异，逐字节核过正好等于 `core-arms.patch`**：把 `tree/` 拷进草稿目录，
  `patch -p1 -R < core-arms.patch` 反打（`patching file crates/singlefs-core/src/{admission,mount,transaction}.rs`，
  退出码 0），再把这条腿自己加的测试文件 `crates/singlefs-harness/tests/opus_r3_p3_attack.rs` 移出去，
  剩下的整棵 `crates/` 树对 `crates-sha256sums.txt` 跑 `sha256sum -c`：**130/130 全 OK**（`grep -v ': OK'` 输出为空）。
  即：`tree` = 快照 + `core-arms.patch`（仅 admission/mount/transaction 三个文件）+ 一个新增测试文件，**没有第四处未解释的差异**。
- 结论：这条腿的拷贝干净、早于冻结副本被改的时刻，它自己的报告「这条腿量的拷贝是开工时 rsync 的那一份」这句话成立。

### 2.2 辩方（Sonnet）有没有哪一格的数是在被改过的冻结副本上量的

**找到具体证据，比 Sonnet 报告自己披露的更精确**：`/tmp/claude-1000/m2-safety-r3-sonnet/run2.log` 的编译头原样是：

```
   Compiling singlefs-harness v0.1.0 (/tmp/claude-1000/safety-r3-frozen/crates/singlefs-harness)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
```

即：这条腿确实**直接在共享冻结副本自己的目录**（不是它的拷贝）里编译、跑过它的候选代码。三份早期日志的
mtime：`run1.log` 04:39:09（`Finished ... in 0.00s`，复用已编译的二进制）、`run2.log` 04:40:43（如上，重新编译 harness）、
`run3.log` 04:41:55（`Finished ... in 0.00s`）。冻结副本四个文件的当前 mtime（复原之后）全部是
`2026-09-26 04:49:20`，即复原动作发生在 run3 之后、`diff/*.patch`（04:47:56）与 `rerun.sh`（04:48:13）生成之后。
对照 Opus 报告披露的「2026-09-26 04:41 UTC 再核时四个 FAILED（修改时刻 04:31–04:33 UTC）」：**修改内容从
04:31 一直留到 04:49:20 才被换回，窗口约 18 分钟，比 Opus 报告字面给出的「04:31–04:33」更长**；run1/run2/run3
这三次编译 + 跑测试全部落在这段被改过的窗口之内。

- **辩方报告自己的披露不够具体**：第八节只写「打了补丁、跑了测试（应该在它的一份拷贝上做）」，没有点出
  编译目录字面就是 `/tmp/claude-1000/safety-r3-frozen/crates/singlefs-harness`——这一点靠 `run2.log` 的编译头才坐实。
- **但被点名引用为证据的那份 `rerun-output.log`（04:52:02 生成）不是这三次污染跑的产物**：它自己的编译头是
  `Compiling ... (/tmp/claude-1000/m2-safety-r3-sonnet-rerun2/crates/singlefs-harness)`——`rerun2` 是
  `rerun.sh` 从主仓 `crates/`（不是冻结副本）拷出来的独立目录；`rerun-check.log`（04:48:55）同理编译在
  `m2-safety-r3-sonnet-rerun-check/`。报告第六节表格引的证据文件正是 `rerun-output.log`，不是 run1/2/3.log
  ——**核对表里没有一处把 run1/2/3.log 当证据引用**。
- **我在自己的草稿目录独立复跑了一遍**（不碰冻结副本，`rerun.sh /home/fy5090/code/singlefs
  /tmp/claude-1000/m2-safety-r3-verifier/sonnet-rerun-verify`），48 条组合的数据行与交回的 `rerun-output.log`
  逐行排序后 `diff` **完全一致**（114/114 行，退出码 0；细节见第四节表）。
- 结论：**污染真实发生过，且持续时间比 Opus 报告字面暗示的更长**；但报告表格实际引用、并被我独立复现的
  那份证据文件不是从被污染的冻结副本产生的。这条留给主 agent 判断：报告第八节这句披露本身不够精确
  （少了「literally 编译在冻结副本自己的目录里」这个具体事实），要不要因此要求它补一句更准确的披露。
- **未核项**：run1.log 首次编译（在这之前谁在冻结副本里第一次跑起了完整编译）没有留下更早的日志可查，
  无法确定 04:31–04:39 之间冻结副本是否被除 Sonnet 之外的别的进程读取过；本轮材料里没有第三方证据可核。

## 三、云端辩方（Sonnet）引用核对

「快照」= 本轮开工快照（本轮是代码轮，快照已给）；kb 引用因辩方声明未建 kb 快照拷贝、按第一步规则对主树核（主树 7/7 与快照一致，见第一节）。

| 引用（文件:行） | 抄的/转述的内容 | 核的结果 | 命令 |
|---|---|---|---|
| `.claude/kb/decisions/16-发布语义.md:36` | 「D16 已定项 1『抬 F 的上限』那一行」 | ✓ 第 36 行确是「抬 F 的上限」行，含 min(...)/第 4 新非空根 | `awk 'NR==36' .claude/kb/decisions/16-发布语义.md` |
| 同文件 `:41` | 「一次准入最多 8 次发布，B=4+2k_tol，k_tol=2」 | ✓ 第 41 行「准入」行原文含 B=4+2k_tol、k_tol=2⇒8 | `awk 'NR==41' ...` |
| `m2-safety-r2-main-verification.md:30` | 「判：出局（两种读法都中）」 | ✓ 逐字相同 | `awk 'NR==30' ...` |
| 同文件 `:36` | A1 Sonnet 读法判词整句（放行覆盖写只多 1 次…4/5、5/6、9/10） | ✓ 逐字相同 | `awk 'NR==36' ...` |
| 同文件 `:51` | 「判：单独用出局（第一、二两轮都中，三轮里已经过半）」 | ✓ 逐字相同 | `awk 'NR==51' ...` |
| 同文件 `:53` | 「第一轮：384 槽 k=20、21 上，24 次回退全被准入拒」 | ✓ 逐字相同 | `awk 'NR==53' ...` |
| 同文件 `:54` | 「这一轮 Opus E5：768 槽…根环 24 条根逐条回退全被拒」 | ✓ 逐字相同 | `awk 'NR==54' ...` |
| 同文件 `:61` | 「判：出局」（A3 小节头） | ✓ 逐字相同 | `awk 'NR==61' ...` |
| 同文件 `:71` | 「豁免只接在发布路径上，挂载那一半一点没改善」 | ✓ 逐字相同 | `awk 'NR==71' ...` |
| 同文件 `:75` | 「判：出局」（A4 小节头） | ✓ 逐字相同 | `awk 'NR==75' ...` |
| 同文件 `:77` | 「池从『挂不上』变成『挂得上、写不了、删不了』」 | ✓ 内容相同（原文用「」、辩方嵌套改用『』属合法的嵌套引号写法，不算摘句） | `awk 'NR==77' ...` |
| 同文件 `:93` | 「抬 F 能救，但今天只有测试入口…每条臂上都是 ok」 | ✓ 但**只引了半行**，原文该行还有「可抬 F 只有测试入口（冻结副本 `mount.rs:1084`）」未被这处引文包含（未标注省略号，是提前收尾的引号）；观察记录，不判 ✗——内容在被引部分没有出入 | `awk 'NR==93' ...` |
| `m2-safety-r1-main-verification.md:42` | 「候选 A（去掉 defer 重复扣）……384 槽 k=20、21 上 24 次回退全被准入拒」 | ✓ 用「……」显式省略中段，首尾两截逐字相同 | `awk 'NR==42' ...` |
| `m2-safety-r2-opus-output.md:321` | 「ND（第一轮候选 A）768 槽：3 个崩溃点挂载被拒…全被拒」 | ✓ 逐字相同（含加粗标记） | `awk 'NR==321' ...` |
| 同文件 `:314` | 「各改法：A1–A4 都不含抬 F，都中…三格上也中」 | ✓ 逐字相同；辩方据此指出判决 `:93` 漏列 A3ge——核过站得住 | `awk 'NR==314' ...` |
| `.claude/kb/checks-owed.md:246` | 「今天没有任何条款、任何检查兑现它」 | ✓ 该子串确实出现在 C283 行的正文里 | `grep -n "checks-owed.md:246" ...`、`awk 'NR==246' checks-owed.md` |
| `diff/diff-mount.rs.patch:151` | `fn mount_writable_with_admission_candidate_and_c283` | ✓ 逐字相同 | `awk 'NR==151' diff/diff-mount.rs.patch` |
| 同文件 `:175` | `fn raise_the_floor_once_for_c283` | ✓ 逐字相同 | `awk 'NR==175' ...` |
| 同文件 `:212` | 「D16 已定项 1『写行那次发布之前不推』在这里成立…」 | ✓ 逐字相同（注释） | `awk 'NR==212' ...` |
| `tests/r3_defense_common/mod.rs:194` | `fn overwrite_with_c283` | ✓ 逐字相同 | `awk 'NR==194' mod.rs` |
| 同文件 `:207` | `fn raise_the_floor_once` | ✓ 逐字相同 | `awk 'NR==207' mod.rs` |
| 同文件 `:279` | `rollback_once` 注释「这里不另包 C283——只测今天字面代码能不能回退出这个格」 | ✓ 注释紧邻该行，内容相同（原文有 `**不**` 加粗标记，报告转述去掉了标记，字面不变） | `sed -n '275,280p' mod.rs` |
| 「跑前登记见附『验证』」 | 声称有附录 | **✗ 引用的位置不存在**——报告全文（222 行）没有任何「附」「附录」小节，`m2-safety-r3-sonnet-model/` 下也没有对应文件 | `grep -n "附「验证」\|附录\|## 附" m2-safety-r3-sonnet-output.md`（只命中这一处自身）；`find m2-safety-r3-sonnet-model -type f` |

**点名复跑（A3 + C283，三个盘宽的 normal_unmount 分支）**：

```
bash research/prompts/m2-safety-r3-sonnet-model/rerun.sh /home/fy5090/code/singlefs \
    /tmp/claude-1000/m2-safety-r3-verifier/sonnet-rerun-verify
```

（在草稿目录的独立仓副本上跑，不碰冻结副本，`nice -n 19` 内建在 rerun.sh 里）。结果：

```
S4 width=240 candidate=a3 branch=normal_unmount c283=false ... remount=RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite
S4 width=240 candidate=a3 branch=normal_unmount c283=true  ... remount=RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite
S4 width=256 candidate=a3 branch=normal_unmount c283=false ... remount=RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite
S4 width=256 candidate=a3 branch=normal_unmount c283=true  ... remount=RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite
S4 width=384 candidate=a3 branch=normal_unmount c283=false ... remount=RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite
S4 width=384 candidate=a3 branch=normal_unmount c283=true  ... remount=RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite
```

三个盘宽（240/256/384）、带不带 C283 都出现 `RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`——**与报告表格一致，✓**。
更进一步：把整份复跑输出的全部数据行（`^(S4|S4b|D3I9|ABSORBING) `，114 行）排序后与交回的 `rerun-output.log`
同样处理的 114 行逐行 `diff`，**退出码 0，逐行相同**（命令：`diff myrun-sorted.txt delivered-sorted.txt`）。

**核了 25 处，✓ 24 处，✗ 1 处**（「附『验证』」引用的位置不存在）。复跑 1 项（A3+C283 全 3 盘宽×2 c283 值），结果与交回产物逐字节一致。

## 四、云端攻方（Opus）引用核对

代码行号对快照核（对当前主树核，主树 130/130 与快照一致，见第一节；对 Opus 自己拷贝的 `tree/` 反打补丁后同样一致，见第二节）。

| 引用（文件:行） | 抄的/转述的内容 | 核的结果 | 命令 |
|---|---|---|---|
| `admission.rs:401` | 「先推空发布抬 F 再判一次（D16 已定项 1）是调用方的事，这里不做。」 | ✓ 逐字相同（文档注释） | `awk 'NR==401' admission.rs` |
| `mount.rs:2157` | `establish_instance` 挂载准入判在取号之前 | ✓ 该行是 `fn establish_instance<...>` 签名 | `awk 'NR==2157' mount.rs` |
| `mount.rs:1236` | 正常卸载 `unmount` | ✓ 该行是 `pub fn unmount<...>` 签名 | `awk 'NR==1236' mount.rs` |
| `mount.rs:3021` | `roll_back_by_a_forward_publish` | ✓ 该行是该函数签名 | `awk 'NR==3021' mount.rs` |
| `mount.rs:2163` | `isolated_slots_per_device` 挂载时算 | ✓ 该行定义该局部变量 | `awk 'NR==2163' mount.rs` |
| `mount.rs:626` | `isolate_slots_referenced_only_by_abandoned_roots` | ✓ 该行是该函数签名 | `awk 'NR==626' mount.rs` |
| `mount.rs:1114`（连同上三行 `.get(3)`） | F 上限取 `get(3)` 连 `.or(oldest_valid_root)?;` | ✓ 逐字相同，且上三行（1111–1113）正是 `.get(3)` 调用链 | `sed -n '1108,1115p' mount.rs` |
| `transaction.rs:4718` | `demand_is_judged`「有普通分配才判」 | ✓ 该行定义 `let demand_is_judged = match allocator.space_admission()` | `awk 'NR==4718' transaction.rs` |
| `.claude/kb/decisions/03-空间分配.md:185` | 「界是 3 次改变用户可见状态的发布」 | ✓ 逐字出现在该行 | `awk 'NR==185' 03-空间分配.md` |
| `.claude/kb/decisions/28-挂载期承诺量.md:89` | 「近满盘 + 坏扇区的池上『预留拿不到⇒只读』可能成吸收态（推的，每 369 行才多一片）」 | ✓ 逐字出现在该行 | `awk 'NR==89' 28-挂载期承诺量.md` |
| `.claude/kb/decisions/16-发布语义.md:29` | 「根环容量是这句的边界」，只写两个状态来回交替 | ✓ 逐字出现且原文确只写两状态交替 | `awk 'NR==29' 16-发布语义.md` |
| `.claude/kb/decisions/16-发布语义.md:41` | 「先推空发布抬 F…B=4+2k_tol」 | ✓（与辩方引用同一行，核过一致） | 同第三节 |
| `research/prompts/m2-safety-r2-main-verification.md:86` | 「会多扣 12–14 槽（推的，没量过）」 | ✓ 逐字相同 | `awk 'NR==86' m2-safety-r2-main-verification.md` |
| `out/e9.log`（384 槽 step49/50，P3、P1 各两行） | 报告正文原样引的 4 行日志 | ✓ 4 行在 `e9.log` 里逐字节可查（`grep -c` 各命中 1） | `grep -n "E9 384 P3 a-ow150 step49..." out/e9.log` 等 4 条 |
| `out/e7.log`（rows0=366，384 槽那一行） | 报告引的完整日志行 | ✓ `grep -c` 命中 1，且 `diff` 与报告贴出的行逐字节相同 | `diff <(awk 'NR==248' opus-output.md) e10line...`（同法用于 e7 行，命中 1） |
| `out/e8.log`（k=6, order=no-rollback-ow-abandon 那一行） | 报告引的完整日志行 | ✓ `grep -c` 命中 1 | `grep -c "E8 slots=384 arm=P3 k=6..." out/e8.log` |
| `out/e10.log`（slots=240, grow=0, branch=c 那一行） | 报告引的完整日志行 | ✓ `grep -c` 命中 1，且逐字节 `diff` 相同 | 见上「特别核」用到的同一条命令 |
| `out/e5.log`（E5a slots=240 script=c-grow6-truncate-x10 step=8 那一行） | 报告引的完整日志行 | ✓ `grep -c` 命中 1 | `grep -c "E5a slots=240 arm=P3 script=c-grow6-truncate-x10 step=8" out/e5.log` |

**点名复跑（E1、E10，自己的仓副本 `/tmp/claude-1000/m2-safety-r3-verifier/opus-rerun/`，源仓用主树，未碰冻结副本）**：

```
# 建副本、打补丁（与 rerun.sh 相同步骤，主树替代已被污染又已复原的冻结副本）
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/{crates,Cargo.toml,Cargo.lock} tree/
patch -p1 --forward < core-arms.patch
# E1
env OPUS_ARMS=T,P1,P2,P3,P3pl nice -n 19 bash run-with-memory-cap.sh 10G bash capped.sh 16 \
    cargo test --release -p singlefs-harness --test opus_r3_p3_attack e1_ -- --nocapture
# E10（同上，filter 换成 e10_）
```

E1 结果：`test result: ok. ... finished in 137.52s`；跑 `summarize_e1.py` 生成的汇总与交回的
`out/e1-summary.txt` **`diff` 退出码 0，逐字节相同**，其中：

```
P3 {'PlacementRefused[Data]': 1240}
P1 {'SAR': 1122, 'PlacementRefused[Data]': 15}
```

——P3 1240 次全是 `PlacementRefused[Data]`，P1 1122+15=1137 次里 15 次是这一种，**与点名的数字一致，✓**。

E10 结果：`test result: ok. ... finished in 445.16s`；`summarize_e10.py` 汇总与交回的
`out/e10-summary.txt` **`diff` 退出码 0，逐字节相同**，其中 240/256 槽在 grow=4/10 两档下的首次拒绝步数落在
7–9 之间（`P1/T/P2/P3/P3pl` 在 `c 4`、`c 10` 两行都非 `None`），末行
`{'T': '13/13', 'P1': '13/13', 'P2': '6/6', 'P3': '6/6', 'P3pl': '6/6'}` 显示**五条臂都命中且全部吸收态，与点名的描述一致，✓**。

**核了 20 处，✓ 20 处，✗ 0 处**。复跑 2 项（E1、E10），两项汇总产物均与交回产物逐字节相同。

## 五、本地攻方（转述核对表 + 运行记录）核对

本地腿的提示是英文转述，逐条核对表要求的是「原文文件:行」核对——按定义第 5 步逐条核，不重新判本地模型答案对不对（那是推理本身，不归我核）。

| 原文文件:行（核对表声称） | 转述内容 | 核的结果 | 命令 |
|---|---|---|---|
| `_m2-safety-r3-body.md:13`（Rule T 行） | 容量(d) − 已分配(d) − …；Σ Demand 角色 span_slots；报拒两种错误码 | ✓ 第 13 行确是 T 行，内容逐字相同 | `nl -ba _m2-safety-r3-body.md \| sed -n '10,17p'`；`grep -n '^| T（今天）'` |
| `_m2-safety-r3-body.md:14`（Rule P1 行） | 同 T；同 T；先推空发布抬 F… | ✓ 第 14 行确是 P1 行 | `grep -n '^| P1（T + C283）'` |
| `_m2-safety-r3-body.md:15`（Rule P2 行） | 去掉 defer 待释放；Σ 全部角色不加换下的槽；报拒 | ✓ 第 15 行确是 P2 行 | `grep -n '^| P2（ND+A1n）'` |
| `_m2-safety-r3-body.md:16`（Rule P3 行） | 同 P2；同 P2；同 P1 | ✓ 第 16 行确是 P3 行 | `grep -n '^| P3（ND+A1n'` |
| `.claude/kb/decisions/16-发布语义.md:41`（Fact B1） | 「准入」整行，含 ENOSPC 与 B=4+2k_tol | ✓ 逐字相同，含新实例第一次发布不推那句括注 | `awk 'NR==41' 16-发布语义.md` |
| 同文件 `:36`（Fact B2） | 「抬 F 的上限」整行，含卸载抬 F 等价关系那半句 | ✓ 逐字相同；核对表自己发现首稿漏译这半句并已回补（见下） | `awk 'NR==36' 16-发布语义.md` |
| `m2-safety-r2-opus-output.md:76`（Fact A5 解释句） | 「21−20=1，实际掉了 28…CheckpointReservePool」 | ✓ 逐字相同（含「space_budget_of_role 归到保留池那一格」，回译为 CheckpointReservePool 不是 Demand，语义对应） | `awk 'NR==76' m2-safety-r2-opus-output.md` |
| `m2-safety-r2-opus-model/out/e7.log:87`（Fact A6） | 「E7 240 A1 a-ow150 step3…」整行 | ✓ 逐字相同 | `awk 'NR==87' e7.log` |
| 同文件 `:88`（Fact A1） | 「E7 240 A1 a-ow150 step4…」整行 | ✓ 逐字相同 | `awk 'NR==88' e7.log` |
| `m2-safety-r2-opus-model/opus_s4_attack.rs:177`（Fact A2） | `[DeviceIdentity(0), DeviceIdentity(1)]...` | ✓ 逐字相同 | `awk 'NR==177' opus_s4_attack.rs` |
| `m2-safety-r2-opus-model/out/e4.log:22`（Fact B3, arm=T） | 「E4 slots=240 arm=T filler=raiseF N=4…」 | ✓ 逐字相同 | `awk 'NR==22' e4.log` |
| 同文件 `:27`（Fact B5, arm=A1） | 「E4 slots=240 arm=A1…8 次都 F-at-ceiling(6)」 | ✓ 逐字相同 | `awk 'NR==27' e4.log` |
| 同文件 `:37`（Fact B4, arm=ND） | 「E4 slots=240 arm=ND filler=raiseF N=7…」 | ✓ 逐字相同 | `awk 'NR==37' e4.log` |
| `crates/singlefs-core/src/admission.rs:514`（补的解释括注来源） | 「普通分配：用户数据单元，与…extent 树、inode 树的节点」 | ✓ 逐字相同（核对表只引用该句前半，未引「保留池对它是纯税」，属声明范围内的节选，不算摘句—对照表本身已注明这是「加的解释」不是整段引用） | `awk 'NR==514' admission.rs` |

**工具闸复核（核对表第 8 节的具体数字）**：

```
python3 research/scripts/corruption-check.py research/prompts/m2-safety-r3-local-attack.md
绿 ... cjk=0 words=2176 ... 粘连=0
```

与核对表声称的「`cjk=0 words=2176 粘连=0`」**逐字段一致，✓**。提示文件里 6 处 `::`/`+:`/`-:` 后插空格、
3 处 `decision file D16` 替换，`grep -n` 逐一命中且计数一致（`::`/`+:`/`-:` 类共 7 处出现、涵盖核对表列出的
全部 6 个模式；`decision file D16` 命中 3 处），✓。

**运行记录（runlog）核对**：

| 声称 | 核的结果 | 命令 |
|---|---|---|
| s1（现存）258 词、s2 282 词、void1 285 词 | ✓ `wc -w` 逐一相同 | `wc -w s1.md s2.md void1.md` |
| s1、s2 判绿，唯一生词 `CheckpointReservePool` | ✓ `oov-check.py` 复跑结果逐字段相同 | `python3 oov-check.py s1.md`、`s2.md` |
| s1、s2 `corruption-check.py` 判绿 | ✓ 两份都 `绿`，`粘连=0` | `python3 corruption-check.py s1.md`、`s2.md` |
| void1.md 是脚本自己写、私有权限、1692 字节 | ✓ `-rw-------`，1692 字节，285 词 | `ls -la`、`wc -cw` |
| void1 作废是因为提示文件判红、不是模型答复判红 | ✓ 一致：void1.md 内容本身语言通顺、无缺词/粘连痕迹，与「判红的是提示不是答复」这一说法自洽（非结论性证据，仅观察） | 通读 void1.md 首 20 行 |

**核了 20 处，✓ 20 处，✗ 0 处**。

## 六、没做什么

- 不判三条腿任何一条打中成不成立、该不该采纳；不核推理本身（A1/A3/A4 出不出局、C283 的实现选择对不对），只核引用、产物与复跑。
- 没有复跑 Opus 报告里 E2–E9（除已点名的 E1、E10）、E5/E6/E7-p3r/E1-p3r 等其余段，也没有重新执行它自己的
  「复跑核对」（E1/E3/E4 用最终源码重跑一遍）——那是它自己已经做过并入库的复核，我只复核了主 agent 点名的两段。
- 没有判断本地攻方 s1/s2 的算术答案对不对（Q1–Q5 的具体数值是否是这套式子的正确解）——那需要重新推导
  C283 的完整算法，属于推理本身，不在我的射程内；只核了它引用的每条 fact 的文件行号与转述。
- 没有查清 04:31–04:39 之间冻结副本是否被 Sonnet 之外的进程读取过：没有更早的日志可查，材料里也没有第三方证据。
- 没有对 Sonnet 报告 diff 补丁与 Opus 报告 `core-arms.patch` 之外的产品代码文件（如 `allocator.rs` 的其余部分、
  `checker`）做全文比对，只核了报告点名引用的具体行。
- 没有跑 `layer0` 测试目标，没有跑 54 号门禁阶段（按派发要求）。
- 没有删除 `/tmp/claude-1000/m2-safety-r3-verifier/opus-rerun/out/`、`sonnet-rerun-verify.log`、`selftest/`、
  几个 `*-sorted.txt` 对照文件——这些是本报告引用的复跑产物与判别力自证材料，留着供主 agent 核对；
  已删除的编译目录与仓副本：`opus-rerun/target`（约 800MB）、`opus-rerun/tree`（约 6MB）、`opus-tree-check`（约 6MB）、
  `sonnet-rerun-verify/target`（约 1.3GB）、`sonnet-rerun-verify/crates`（约 6MB）。
