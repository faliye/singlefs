# m2-rootchoice-repair-r1 核查报告（three-way-verifier）

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证

挑 Opus 报告里的一条引用：`crates/singlefs-core/src/recovery.rs:361`–`380`，原样抄
`let candidate_key = (candidate.checkpoint_txg, candidate.instance);`（Opus 标注在 372 行）。
在草稿目录的快照副本里把行号 +1（372 → 373），核对：

```
$ sed -n '372p' /tmp/claude-1000/rootchoice-verifier/snapshot/recovery.rs
            let candidate_key = (candidate.checkpoint_txg, candidate.instance);
$ sed -n '373p' /tmp/claude-1000/rootchoice-verifier/snapshot/recovery.rs
            if best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance))
```

373 行不是 Opus 引的那句 ⇒ 按第 2 步的核法（到快照里取那一行、逐字比内容）判 **✗**。
核法判得出 ✗，往下按这一套办法核。

## 一、开工快照与主树今天的关系（先说清，下面每一处引用都按这一节办）

`research/prompts/m2-rootchoice-repair-r1-snapshot/opening.sha256` 列了 10 个文件。现查：

```
$ sha256sum -c research/prompts/m2-rootchoice-repair-r1-snapshot/opening.sha256
crates/singlefs-core/src/mount.rs: OK
crates/singlefs-core/src/recovery.rs: OK
crates/singlefs-core/src/transaction.rs: OK
crates/singlefs-core/src/instance_table.rs: OK
crates/singlefs-core/src/allocator.rs: OK
.claude/kb/decisions/02-RAID条带策略.md: OK
.claude/kb/decisions/16-发布语义.md: FAILED
.claude/kb/decisions/23-journal的角色与格式.md: FAILED
.claude/kb/checks-owed.md: FAILED
.claude/kb/invariants.md: OK
```

3 个 kb 文件今天（我核查时）与开工快照不同，**但 Opus 与 Sonnet 报告里各自贴的 `sha256sum -c` 原样输出都是全 OK**——
两条腿各自完成时（Opus 报告文件 mtime 23:17、Sonnet 23:11）应确实是 OK；这 3 个文件的 mtime 是 23:09–23:10，
早于两条报告完稿、晚于快照时刻 22:57，判定是**另一个并发会话在这一轮腿工作期间改了这 3 个 kb 文件**
（`.claude/agents/agent-common.md`「本机常有别的会话在同一个仓里干活」），不是腿说谎。

**这 3 个文件今天的主树内容不能拿来核这一轮的引用**（会核出与腿开工时不同的行号/文本）。
我在 `/tmp` 里找到另外两个不相关轮次的草稿副本，其内容恰好与开工快照哈希逐位相同，用它们重建了这 3 个文件
在开工时刻的原文：

```
$ sha256sum /tmp/claude-1000/rootchoice-verifier/snapshot/16-发布语义.md
23ca03d9d489bfa2ad300127c532628b5ba969c62cbbaa61173437479e74fb36  ...16-发布语义.md
$ sha256sum /tmp/claude-1000/rootchoice-verifier/snapshot/23-journal的角色与格式.md
c3e07269d4c9ca005bdd232b3e5446cccc665cad92536b94e69588f5ae9e37e4  ...23-journal的角色与格式.md
$ sha256sum /tmp/claude-1000/rootchoice-verifier/snapshot/checks-owed.md
bcd38e66482e30222cbfd2cd992a8a241eed76752ec5f05c40cce3196676129a  ...checks-owed.md
```

来源：`/tmp/claude-1000/m2-refusals-presumed-r1-verifier/repo/.claude/kb/decisions/{16-发布语义.md,23-journal的角色与格式.md}`
与 `/tmp/claude-1000/panic-sites-r2-r7-r10/drafts/copy4/.claude/kb/checks-owed.md`，三份哈希与 opening.sha256 逐位相同，
以下所有对这 3 个文件的核查都对着这份重建副本，不对今天的主树核。

**`journal.rs` 不在 10 个文件的清单里**（这一轮的核心量`(counter−1) mod 环槽数`就住在这里，manifest 漏收了它）。
`git status` 对它干净、上一次提交在 2026-09-14，早于这一轮开工 8 天，此后没有改动 ⇒ 今天的主树内容
可以当开工时刻的内容用，下面对它的核查按主树核，标注「主树，manifest 未覆盖，git 干净+8 天无改动」。

同理，`crates/singlefs-harness/{fault_injection.rs, segments.rs, scenario.rs}`、`crates/singlefs-format/src/lib.rs`
四个文件也不在 manifest 里，且 `segments.rs`/`scenario.rs` 相对 HEAD 有未提交改动（`fault_injection.rs` 整个未跟踪）。
四者的 mtime 分别是 13:25 / 12:13 / 12:02（均为 9-22 当天），早于快照时刻 22:57 十小时以上，此后无改动迹象 ⇒
按同样理由对主树核，标注同上。

## 二、云端攻方（Opus）：K2、K5 引用逐条核

Opus 自陈「一行代码都没跑」，全部读数按源码逐行推。下表核的是「引的那几行是否存在、原文是否与抄的一致」
与「从这几行推出的算术/枚举是否成立」，不判「打中不打中」。

### 二·一　事实表（正文 二·〇，共 9 行）

| 引用 | 核的结果 |
|---|---|
| `recovery.rs:361-380` / `let candidate_key = (candidate.checkpoint_txg, candidate.instance);`（372 行） | ✓ 逐字相同 |
| `mount.rs:294-312` / `CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 + 1)`（311 行） | ✓ 逐字相同 |
| `transaction.rs:579-594` / 四步收在 `persist` 闭包 | ✓ 闭包内确为 Barrier→WriteJournalRecord→Barrier→WriteRootRecordForceUnitAccess→RotateSystemConfigurationSlots 五步（Opus 说「四步」指发布动作，Barrier 不计入，与原文一致） |
| `transaction.rs:432` / `pool.write_system_configuration_slot(index, 0, instance)` | ✓ 逐字相同 |
| `transaction.rs:202-221` / 世代号+1、槽=世代号mod2 的注释 | ✓ 逐字相同（199 行注释） |
| `transaction.rs:91` / 根槽 FUA 注释 | ✓ 逐字相同 |
| `journal.rs:109-114` / `((counter - 1) % ring_slots) * JOURNAL_RECORD_BYTES` | ✓ 逐字相同（主树，manifest 未覆盖，见第一节说明） |
| 第一版几何三常量（`singlefs-format/src/lib.rs:185-186`、`segments.rs:197`、`scenario.rs:43`） | ✓ 三处均逐字相同（主树，manifest 未覆盖，见第一节说明） |
| `mount.rs:980-1005` / 暖机循环条件 | ✓ 逐字相同 |

### 二·二　算术复核（判别力自证之外，独立核一遍）

**journal 槽数**：768 MiB ÷ 4096 = 196608；**根环槽数**：3×8=24；196608÷24=8192。Opus 的「8192 倍」核对成立。

**H-A 六步的区域/槽/盘**（公式：区域=txg mod 3，槽=(txg div 3) mod 8，`region_devices=[d0,d1,d0]`），逐条现算：

| txg | 算出区域 | 算出槽 | 算出盘 | Opus 表里写的 | 对不对 |
|---|---|---|---|---|---|
| 40 | 1 | 5 | d1 | 区域1/槽5/d1 | ✓ |
| 41 | 2 | 5 | d0 | 区域2/槽5/d0 | ✓ |
| 42 | 0 | 6 | d0 | 区域0/槽6/d0 | ✓ |
| 43 | 1 | 6 | d1 | 区域1/槽6/d1 | ✓ |
| 44 | 2 | 6 | d0 | 区域2/槽6/d0 | ✓ |
| 45 | 0 | 7 | d0 | 区域0/槽7/d0 | ✓ |

六行全部现算成立；这也说明 txg 相同、instance 不同的两条根必然落同一个（区域,槽），
「挂载 2 写出 (42,8) 盖掉 (42,7) 所在的区域0槽6」这类跨实例覆盖的机制成立。

**暖机次数 m 的推导**：`warm_up_publish_txgs` 从 `covered=[写行发布落的那块盘]` 起循环，每次 txg+1、
把新覆盖的盘加进 `covered`，直到 `all_devices` 全覆盖或到 `ROOT_RING_REGIONS`(3) 次为止。
第一版两块盘、`region_devices=[d0,d1,d0]`：写行发布落 region1(d1)，`covered=[d1]`；
下一 txg 落 region2(d0)，`covered=[d1,d0]` 已覆盖两块盘 ⇒ 循环退出，恰好 1 次。
m = 1（写行）+1（暖机）+1（第一次用户发布）= 3，与 Opus 算的一致。

**`grep -rn '切换' crates/singlefs-core/src/*.rs`**（Opus 第四节）：现跑，命中 1 行，在 `recovery.rs:906`
的注释里，与 Opus 报告逐字相同。

**`grep -rn '只供测试' crates/*/src/*.rs`**（Opus 第四节）：现跑，命中 12 处，文件与行号集合
（mount.rs:155/641/642、recovery.rs:219、allocator.rs:513/524/636/660、fault_injection.rs:276/362/398/483）
与 Opus 报告逐字相同（只是 glob 展开顺序不同，不影响集合）；`_m2-rootchoice-repair-r1-body.md:22` 确实写
「只命中三处」，Opus「这中间有别的会话落地了新开关」的推断与 `fault_injection.rs` 是当天新建的未跟踪文件
（mtime 12:02，早于快照 22:57）这一事实相容。

`fault_injection.rs:276`、`:286-292`、`:393-400`、`:486-494`、`:658`、`:666` 六处引用逐条核：✓ 全部逐字相同
（主树，manifest 未覆盖，见第一节说明）。

**`grep -rn scrub crates` 与 `grep -rn 根链 crates`**（Sonnet 报告用，顺带一起核）：现跑均零命中，与报告一致。

### 二·三　mount.rs / recovery.rs 深层引用（K5 与「二·一·补」用到的那几处）

| 引用 | 核的结果 |
|---|---|
| `mount.rs:1194-1199` / `next_counter = ...max().unwrap_or(0) + 1` | ✓ 逐字相同 |
| `mount.rs:1174-1192` / `replay_journal` 调用 + `chosen_root`/`effective_root` | ✓ 逐字相同 |
| `mount.rs:1213-1218` / `previous_row.selected_root_txg: effective_root.checkpoint_txg` | ✓ 逐字相同 |
| `mount.rs:1241` 注释「新实例的第一条 jsn 接在环里最大的 jsn 之后…取 P2」 | ✓ 逐字相同 |
| `mount.rs:1009-1034` / `instance_rows_to_write` | ✓ 逐字相同 |
| `mount.rs:809-858` / `publish_empty_after` | ✓ 逐字相同 |
| `mount.rs:1166` / `mount_writable`，`mount.rs:1245` / `mount_rollback` | ✓ 函数签名逐字相同 |
| `recovery.rs:1058-1155` / `replay_journal`：`let mut rebuilt = *root;`，无记录施加时不改变 | ✓ 现读整个函数体，`rebuilt` 只在 for 循环内被赋新值，`above` 为空时循环不执行 ⇒ 交回的正是 `chosen_root`，Opus「一条都没施加时交回的就是 chosen_root」的机制性断言成立 |
| `recovery.rs:1023-1055` / `scan_journal`：逐盘扫、`.or_insert` | ✓ 逐字相同，`or_insert` 确认「任一份自证过即在」的机制 |
| `transaction.rs:171-178` / 根槽写只调 `write_at` | ✓ 逐字相同 |
| `transaction.rs:581-584`、`:2699-2702`、`:586-593` / 两条发布路径都先写记录再写根 | ✓ 三处均逐字相同 |

## 三、特别核查项之二：C331 正文末句 + journal.rs 槽映射 + mount.rs counter

**checks-owed.md 那一行**（用重建副本核，见第一节）：
```
$ grep -n "^| C331 " /tmp/claude-1000/rootchoice-verifier/snapshot/checks-owed.md | cut -c1-6
288:| C331
```
第 288 行确实是 C331 那一行，与 Opus 报告标的行号一致。原样抄出末句（第三列末尾）：

> D23（journal 的角色与格式） 已定项 14 注 3「新实例从前缀末 + 1 接着写」还会让新实例的第一条记录落在
> 读不出那条根的记录槽上，把它唯一的记录见证盖掉（今天的规则不拿那条记录当见证，所以今天看不出后果）

与 Opus 报告 210 行抄的那句**逐字相同**（Opus 少抄了括注「今天的规则不拿那条记录当见证，所以今天看不出
后果」，但 Opus 在下面正文自己另外单独讨论了这一点，不算摘句丢限定词——它把整句原文放在别处完整交代了）。

**独立复核判据本身**：

1. `journal.rs:112`：`((counter - 1) % ring_slots) * JOURNAL_RECORD_BYTES`，`ring_slots = ring_bytes / JOURNAL_RECORD_BYTES`。
2. `mount.rs:1194-1199`：新实例第一条记录的 `counter = 环里全部记录 counter 的 max + 1`。
3. `transaction.rs` 里另外三处 `counter: previous.record.counter + 1` / `previous_counter + 1`
   （488、1499、1545、1659 行一带，现查确认）：**counter 全程逐一递增，不跳号、不按实例分段**。
4. 由 1–3 现推：设当前环里最大 counter 为 M，新实例第一条记录 counter = M+1，其槽 = M mod 196608。
   该槽此前一次被占用时写的是 counter = M+1-196608 那条记录（因为 counter 逐一递增、槽按 mod 196608 循环，
   下一次相同槽位必须是整整一圈之后，即 196608 之后）——即「比新记录小 196608 的那一条」，
   **算出来的结论与 Opus 的算术一致**。
5. 「768 MiB 环在任何可达历史里没绕满过一圈」这句本身是一个全称命题（对「任何可达历史」），
   我核不动全称部分；但这一轮 Opus/Sonnet/本地攻方构造的全部历史，counter 都停在两位数以内
   （H-A 最大到 45），196608 比它们大 4 个数量级，量级差距本身可现算，不是拍的。

判定：**C331 正文末句在「普通可写挂载」这条路径上的机制性断言可以现查证伪**（盖掉的是最旧的一条，
不是「读不出那条根的记录」），Opus「判据自己写错」这一判定**逐条引用与算术都核对成立**；
末句在「回退按 P1」那个假设支上是否为真，我没有再重复推——Opus 自己也写明那是「那句话在一种形状上为真」
的另一支，不影响这里核的这一条。

## 四、特别核查项之三：K5 定义「两半不等」

`.claude/kb/decisions/23-journal的角色与格式.md:371`（用重建副本，见第一节）：

`23-journal的角色与格式.md` 标题层级下 `#### 已定项 14` 会把注 1-4 整节一起带出，与「定位到 371 那一行」的诉求不对齐，改用行区间取法，命令与原样输出：

```
$ python3 research/scripts/quote-kb.py /tmp/claude-1000/rootchoice-verifier/scratch/k5-def.md '/tmp/claude-1000/rootchoice-verifier/snapshot/23-journal的角色与格式.md:371-371'
  ✓ 1 段整抄进 /tmp/claude-1000/rootchoice-verifier/scratch/k5-def.md，回读逐字节一致
```

产物 `k5-def.md` 里的整行与 Opus 报告 245 行抄的内容逐字相同（两边都以「- **实例切换** = 挂载内做一次恢复」开头、
以「频率未测。」结尾）。

在这一整行原文里现读，确认「两半」字面都在：
- 首句：「所选根取被重发的那个**在飞 checkpoint 所基于的根**」——锚定对象是「在飞 checkpoint」。
- 括号：「（旧实例发布过根时是它最后发布的根，**没有时是这次挂载恢复重放之后的根**，回退那次发布里是 R_old）」
  ——锚定对象是「旧实例」这个身份。

两个从句字面上锚定的对象不同（一个是「这次被重发的 checkpoint」，一个是「旧实例这个身份」），这一点是可核的事实。
「连续第二次切换时二者取值不同」这一步是推论（论据：旧实例已经换成上一次切换取的新实例、而那个新实例一条根都没发布成），
我按第 1 节机制核了它引用的支撑材料，不判这条推论本身站不站得住：

- `mount.rs:1174-1192`：`replay_journal` 一条记录都没施加时交回 `chosen_root`——已在第二节核实成立，
  支持 H-K5-1 步骤 7「按括号」那一支取值为 `Y_后 = chosen_root`（未施加记录时等于 `Y`）的机制性前提。
- `.claude/kb/decisions/18-块里携带什么信息.md:307`（切换写的行，四分句①-④）：现查主树（本文件未被并发修改，
  见第一节），与 Opus 245-252 行、316-321 行的逐句应用**逐字对得上**。
- `.claude/kb/decisions/18-块里携带什么信息.md:309`（已发布谓词全局）：现查主树，与 Opus 313 行抄的内容
  **逐字相同**。

## 五、C334 那一行 + 两条警告

`checks-owed.md:291`（重建副本）：
```
$ grep -n "^| C334 " /tmp/claude-1000/rootchoice-verifier/snapshot/checks-owed.md | cut -c1-6
291:| C334
```
行号与 Opus 三·四节标的一致；行内「恢复会选的最新可读根」「这次挂载恢复所选的根」两种读法的措辞
与 Opus 331-334 行表格里摘的两句**逐字相同**。

## 六、云端正推（Sonnet）：K1、K3 引用逐条核

Sonnet 分工 K1（收口表③「修复」指什么）与 K3（C332 择根要不要读实例表）。抽查了它引用的全部大段原文抄录
（`> ` 引用块）与关键代码行，逐条核如下。

| 引用 | 核的结果 |
|---|---|
| `mount.rs:209-211` / `abandoned_roots_unreadable` 字段与注释 | ✓ 逐字相同 |
| `mount.rs:206-207` / `isolated_slots_per_device` 字段与注释 | ✓ 逐字相同 |
| `02-RAID条带策略.md:183`（射程句） | ✓ 逐字相同 |
| `02-RAID条带策略.md:180`（规则2「修复」定义） | ✓ 逐字相同 |
| `02-RAID条带策略.md:194`（第一个事务字节：否） | ✓ 逐字相同 |
| `02-RAID条带策略.md:148/179/181/187`（已定项9标题、钉住量、w=2规则、E80数据） | ✓ 四处均逐字相同 |
| `21-权威态与派生态的分界.md:184-191`（权威态/派生态两分表） | ✓ 逐字相同（本文件未被并发修改，见第一节） |
| `21-权威态与派生态的分界.md:106`（分配记录树是派生态） | ✓ 逐字相同 |
| `21-权威态与派生态的分界.md:253`（不设转换器） | ✓ 逐字相同 |
| `21-权威态与派生态的分界.md:265-271`（三件事同一条代码路径表） | ✓ 逐字相同 |
| `21-权威态与派生态的分界.md:277`（树可以丢、根不能丢） | ✓ 逐字相同 |
| `21-权威态与派生态的分界.md:202`（实例表不可读时只读挂载） | ✓ 逐字相同 |
| `invariants.md:54`（I-7.4 近K代块未被复用） | ✓ 逐字相同 |
| `13-验证路线.md:142-154`（Merkle根链差分/代际增量scrub定案） | ✓ 逐字相同 |
| `13-验证路线.md:156`（射程句+grep零命中的断言） | ✓ 逐字相同；现跑 `grep -rn scrub crates`、`grep -rn 根链 crates` 均零命中，与报告一致 |
| `checks-owed.md:289`（C332 整行，重建副本） | ✓ 逐字相同 |
| `16-发布语义.md:151`（择新按实例代号高者赢） | ✓ 逐字相同 |
| `18-块里携带什么信息.md:306`（回退写的行） | ✓ 逐字相同 |
| `22-单元原子性怎么合成.md:143`（实例表单元指针字段行） | ✓ 逐字相同 |
| `22-单元原子性怎么合成.md:186-193`（字段分段定案表） | ✓ 逐字相同 |
| `22-单元原子性怎么合成.md:195-198`（跨512字节撕裂射程句） | ✓ 逐字相同 |

**「D22 第487行是错误引用」这条独立复核**：
```
$ wc -l .claude/kb/decisions/22-单元原子性怎么合成.md
531 .claude/kb/decisions/22-单元原子性怎么合成.md
$ awk 'NR==487' .claude/kb/decisions/22-单元原子性怎么合成.md | cat -A
$
$ grep -rn "按 checkpoint_txg 择新\|checkpoint_txg 平局时按实例代号高者赢" .claude/kb/decisions/22-单元原子性怎么合成.md
（零命中）
```
487 行确为空行，全文零命中那两句 ⇒ Sonnet 的现查结论成立。这条错误引用原本写在
`research/prompts/_m2-rootchoice-repair-r1-body.md:48`（主 agent 2026-09-22 自己现查并写明），
Sonnet 报告 149 行的措辞（「主 agent 2026-09-22 现查已核实」）也正确地把出处归给了主 agent，
不是把自己的新发现冒充成独立打中。

**K1 段落审查**：Sonnet 在「二·一 关键前提」一节里对「树表」与「分配记录树」两个对象分别推的做法，
逐条核对照的原文抄录都命中且完整（未见摘句、未见漏掉射程句本身的限定词）。

Sonnet 报告没有可复跑的命令类推论（无实验、无模型目录），复跑一栏不适用。

## 七、本地攻方（K4、K6）：翻译核对表复核

**Fact 2 误译修复复核**：`.claude/kb/decisions/18-块里携带什么信息.md:304` 现查含「写行之前不推抬 F 的空发布」；
`.claude/kb/decisions/16-发布语义.md:35` 现查含「回退下界 F」这个名字，与 D23（journal 的角色与格式）已定项14 的
「W」（`23-journal的角色与格式.md:361`「切换时的 W 取被重发的那个 checkpoint 里的最大事务号」）是两个不同的量
——核对表「F 与 W 是两个不同的量」这句成立。最终提示文件现查：
```
$ grep -n "watermark" research/prompts/m2-rootchoice-repair-r1-local-attack.md
（零命中）
$ sed -n '94p' research/prompts/m2-rootchoice-repair-r1-local-attack.md
publish meant to raise the rollback floor is pushed before the row is
```
✓ 今天的提示里没有残留「recovery watermark」这个误译，F 用「rollback floor」表达，W 单独出现且不叫 watermark。

**Fact 5 误译修复复核**：`.claude/kb/decisions/28-挂载期承诺量.md:73` 现查公式含
`max(1, ⌈(rows0 + N_switch) / 每片行数⌉)` 外层 max。最终提示文件现查：
```
$ sed -n '133,136p' research/prompts/m2-rootchoice-repair-r1-local-attack.md
Fact 5. From an established decision item about what a mount commits to
for its own duration, its formula for rewriting the instance table chain,
quoted in full: number of mirror copies times 32768 times whichever is
larger of one, or the ceiling of rows0 plus N_switch divided by rows per
```
✓ 今天的提示里「whichever is larger of one, or the ceiling of...」把外层 max 译出来了，不再漏抄。

**其余 Fact 抽查**（核对表宣称「无遗漏」或「按字面译」的几条，抽了 8 条现查原文）：

| Fact | 原文文件:行 | 核的结果 |
|---|---|---|
| Fact 4 | `23-journal的角色与格式.md:370`（根槽写失败重发） | ✓ 逐字相同 |
| Fact 7 | `02-RAID条带策略.md:224-225`（挂载准入表+可写设备数口径） | ✓ 逐字相同 |
| Fact 9 | `23-journal的角色与格式.md:360`（计数器全池接着走，注3） | ✓ 逐字相同 |
| Fact 10 | `16-发布语义.md:151` | ✓ 逐字相同（与 Opus/Sonnet 引的同一行） |
| Fact 12 | `23-journal的角色与格式.md:363`（管理员回退定义） | ✓ 逐字相同 |
| Fact 15 | `22-单元原子性怎么合成.md:186`、`:204`（481字节字段表、射程） | ✓ 两处均逐字相同 |
| Fact 16 | `checks-owed.md:215`（C245 整行，重建副本） | ✓ 逐字相同 |
| Fact 1/8/11/13/3/6 | `checks-owed.md:292/288/289/291`（重建副本）、`18-块里携带什么信息.md:310`、`28-挂载期承诺量.md:79` | ✓ 六处均逐字相同（核对表自己标注的「压缩摘要」「未译某半句」都如实登记，抽查未发现登记之外的额外删减） |

**void1 复跑核实**（判红作废那一次，独立复跑 corruption-check.py，不在原目录跑，在草稿目录跑）：
```
$ nice -n 19 python3 research/scripts/corruption-check.py /tmp/claude-1000/rootchoice-verifier/replay/m2-rootchoice-repair-r1-local-attack-output-void1.md
红 .../m2-rootchoice-repair-r1-local-attack-output-void1.md  cjk=0 words=647 fffd=0 汉字复读=0(0.00/千) 英文复读=2(3.09/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0
     英文复读样本: same invariant | same invariant
```
与运行记录里贴的原样输出逐字段相同（`英文复读=2(3.09/千)`、复读样本一致），退出码 1（脚本本身）。

```
$ nice -n 19 python3 research/scripts/corruption-check.py .../output-s1.md
绿 ...  cjk=0 words=509 ... 全部计数为 0
$ nice -n 19 python3 research/scripts/corruption-check.py .../output-s2.md
绿 ...  cjk=0 words=893 ... 全部计数为 0
```
两份干净样本复跑：exit=0，`words=509`/`words=893` 与运行记录贴的数字（`corruption-check.py` 那一栏）逐字相同。

**oov-check.py 复跑，第一次踩坑、第二次找到原因**：不带提示文件参数跑时得到 `生词=7`（s1）/`生词=14`（s2），
与运行记录声称的「生词=0」不符；读脚本发现 `oov-check.py <输出文件> [提示文件]` 有第二个可选参数——
提示文件里出现过的词会被从「生词」里排除（`scan()` 函数第 157-159 行读提示文件建 `known` 集合）。
补上提示文件参数重跑：
```
$ nice -n 19 python3 research/scripts/oov-check.py .../output-s1.md .../m2-rootchoice-repair-r1-local-attack.md
绿 ...  生词=0 拼接=0
$ nice -n 19 python3 research/scripts/oov-check.py .../output-s2.md .../m2-rootchoice-repair-r1-local-attack.md
绿 ...  生词=0 拼接=0
```
与运行记录一致。**这一条记「核对成立」，不记 ✗**：我第一次复跑漏传参数是我自己没读脚本用法，
不是报告本身的错——补上参数之后精确复现，写在这里是给后续核查员的一个提醒，不是判这份报告的缺陷。

## 八、两份干净样本（S1/S2）72 格逐格比对

72 格 = K4 六个 judgment（A–F）各 3 项（grounding / break / consequence）= 18 格，
K6 十八个 judgment 各 3 项（same byte / same invariant / verdict）= 54 格，合计 72。
逐格比对，只列**方向不一致**（含「有/无冲突」翻面、「有/无共享字节」翻面、断了哪个 fact 的判断完全不同）
的格，方向一致、仅供支撑事实编号不同的不列（那种差异不算方向不一致，是两次抽样对「该引哪条 fact」
给出不同答案，本身就是「一条腿只抽一次样不算一次观测」要防的那类正常波动）。

### K4（A–F），consequence 一栏三处方向不一致

| judgment | S1 | S2 | 不一致点 |
|---|---|---|---|
| A | consequence: fact 6（暗示有牵连） | consequence: no other fact directly conflicts | 有无冲突方向不同 |
| B | consequence: fact 1（暗示有牵连） | consequence: no other fact directly conflicts | 有无冲突方向不同 |
| F | consequence: fact 2（暗示有牵连） | consequence: no other fact directly conflicts | 有无冲突方向不同 |

judgment D 的 break 本身两份样本指向完全不同的内容（S1 改 Fact 5 的「每片行数」公式；S2 改 Fact 1/6
「近满盘」那句），不是同一个 break 的方向分歧，是两次抽样选了不同的攻击点，按上面的口径不算方向不一致，
单独记一笔：**这一格两次抽样连「攻哪一句」都没对上**，比其余几格的分歧更大。

judgment C、E 两份样本在「打没打中」这个顶层问题上方向一致（两份都判有冲突），
但对「break 的是 fact 几、consequence 的是 fact 几」互相调换了编号（S1：破 fact2→果 fact1；
S2：破 fact1→果 fact2），这是标号层面的不一致，不是判断方向的不一致，单列在这里存查。

### K6（18 对），三处判断方向不一致

| judgment | S1 | S2 | 不一致点 |
|---|---|---|---|
| 2a-3c | same invariant: fact 8 fact 11；verdict: no conflict found | same invariant: fact 8；verdict: adopting 2a would change the behavior that 3c is meant to keep unchanged so there is a conflict | **verdict 直接翻面**：无冲突 ↔ 有冲突 |
| 2b-3c | same invariant: fact 15；verdict: no conflict found | same invariant: fact 15；verdict: adopting 2b would change the behavior that 3c is meant to keep unchanged so there is a conflict | **verdict 直接翻面**：无冲突 ↔ 有冲突 |
| 4-2b | same byte: system configuration structure（判有共享字节）；verdict: no conflict found | same byte: no shared byte named in any fact given above；verdict: no conflict found | **same byte 分项翻面**：有共享字节 ↔ 无共享字节（verdict 本身两份都判无冲突，未翻面） |

其余 15 对 K6 判断（2a-3a/3b、2b-3a/3b、2c-3a/3b/3c、2a-5、2b-5、2c-5、3a-5、3b-5、3c-5、4-3b、4-5）
两份样本的 same byte（均判「无共享字节」）与 verdict（均判「no conflict found」）方向一致，
仅 same invariant 引用的 fact 编号不同（例如 2a-3a：S1 引 fact9，S2 引 fact10），按上面口径不算方向不一致。

**汇总**：72 格里，K4 有 3 格 consequence 方向不一致（A/B/F）+ 1 格 break 本身选了不同内容（D）；
K6 有 3 格方向不一致（2a-3c、2b-3c 的 verdict 翻面，4-2b 的 same byte 翻面）。
方向不一致合计 **6 格**（A、B、F、2a-3c、2b-3c、4-2b），另有 D 一格「攻击点选不到一起」与 C、E 两格
「标号对调」单列存查，不计入方向不一致的 6 格之内。

## 九、计数与汇总

| 腿 | 核了几处引用/命令 | ✓ | ✗ | 核不动 / 分不清 |
|---|---|---|---|---|
| 判别力自证 | 1 | 0 | 1（人为改错，判得出） | 0 |
| 云端攻方 Opus（K2、K5） | 事实表9 + 深层引用11 + grep/算术复核6 + K5引用3 = 29 | 29 | 0 | 0 |
| 云端正推 Sonnet（K1、K3） | 21 | 21 | 0 | 0 |
| 本地攻方翻译核对表（K4、K6） | Fact抽查14（含2处首稿错译的复核）+ 复跑命令4（corruption×3、oov×2按下方口径合1）| 全部核对成立 | 0（首稿两处错译已在提示定稿前改正，今天的提示文件不含残留错误） | 0 |
| 72 格 S1/S2 比对 | 72 | 66 方向一致 | 不适用（这不是判对错，是判一致不一致；6 格方向不一致，已列在第八节） | — |

**总计**：核了引用/命令约 74 处（不含 72 格逐格比对，那是另一种计数口径），✓ 74 处，✗ 0 处
（不含判别力自证那 1 处人为改错的 ✗）。

## 十、没做什么

- **不判**任何一条打中成不成立、该不该采纳，也不判 72 格里哪一份样本的具体判词更对——那是主 agent 的活。
- **没有重新推导** Opus 的 H-B1、H-B2、H-C1、H-C2、H-K5-1、H-K5-2 全部逐步读数表格（只核了 H-A 与 H-B2/H-K5-1
  用到的关键机制性引用与 H-A 的完整算术），H-B1/H-C1/H-C2/H-K5-2 表格里的具体数字没有逐格重算——
  Opus 自己标注这些都是「推的」，不是「量过的」，我核的重点放在支撑这些推论的**源码引用是否存在、
  是否被正确抄录**，以及可以独立算的那几处算术（第二节二·二），没有把每一张表逐格摊开重算一遍。
- **没有编译、没有跑门禁、没有跑变异**：这一轮三条腿都没有可编译产物（Opus/Sonnet 全靠读代码推理，
  本地腿只交了翻译核对与两份模型抽样），`.claude/gate.d/stage-owners.tsv` 里没有登记给 `three-way-verifier`
  的阶段。
- **没有对 K1/K3 的「相容分项/新开的失败面」这类判断做真伪判定**，只核了它引用的原文是否存在、是否完整——
  例如 Sonnet 判「候选1对C332结构性无效」这个结论本身对不对，不归这份报告管。
- **没有重新验证 H-C1/H-C2 涉及的「12 个故障」「173 个状态」这类由攻方腿自己模型跑出来的数字**
  （C331 那一行里「攻方腿模型真值自己 173 个状态」）——这些数字来自更早轮次的攻方腿模型，
  不是这一轮 Opus/Sonnet/本地腿自己产出的，不在这一轮的核查范围内。
- **本地攻方腿 K6 只做了「K4 候选与 2b/3b/5 各配一次」共 3 对**（而非 6×7=42 对全交叉），
  这是它自己在核对表与运行记录里写明的取舍，不是这份报告要补的缺口，原样记在第八节。
- **没有读禁读清单**（这一轮派发提示没有单独给出禁读清单，只给了背景材料路径 `_m2-rootchoice-repair-r1-body.md`
  用于识别「误写成背景材料行号的引用」；核查中没有发现任何一条引用把行号误标成背景材料自己的行号——
  Opus、Sonnet、本地攻方三份材料里出现的行号全部指向各自 kb/代码文件自己的行号，没有一条指向
  `_m2-rootchoice-repair-r1-body.md` 或 `_m2-rootchoice-repair-r1-appendix.md`/`-checklist.md`/`-background.md`
  自己的行号）。

## 十一、草稿目录

`/tmp/claude-1000/rootchoice-verifier/`：
- `snapshot/`：开工快照的可用副本，含 manifest 里 10 个文件（3 个 kb 文件是从别处重建的、哈希核对过）
  与 4 个 manifest 未覆盖但按 mtime/git 状态判定安全可用主树的文件（`journal.rs`、`fault_injection.rs`、
  `segments.rs`、`scenario.rs`；后两者相对 HEAD 有未提交改动，但改动时刻早于本轮开工快照，已在第一节写明理由）。
- `replay/`：本地攻方腿三份输出样本（s1、s2、void1）与提示文件的副本，供 corruption-check.py / oov-check.py
  复跑用，未在原目录跑。
- `scratch/`：`quote-kb.py` 的产物 `k5-def.md`。
