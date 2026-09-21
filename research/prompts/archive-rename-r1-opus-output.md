# 云端攻方腿（Opus）：J1 / J2 / J8 —— archive-rename-r1（2026-09-21）

立场：假设「改这四处字面量是安全的」是错的。分到 J1、J2、J8，J3–J7 不碰。
所有行号是 2026-09-21 现读工作区（`HEAD` = 3cff909，改动在暂存区）用 `grep -nF` 取的。
时刻一律 UTC（本机时钟）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| J1 | **打中（量过）** | 区域名那个串是 E142「量 5」装置↔`crates/` 逐字节比对的**配对键**（`e142_first_transaction_dry_run.rs:3932` 拿它做 `find`）。两端名字对不上时，比对不是判红，是**静默少比两格**：实测 `regions=21 equal=19 unequal=0 snapshot_given=true`，两行 `equal=unknown … reason=no_impl_snapshot_given`（而同一行写着 `snapshot_given=true`，这个 reason 是假的）。`equal=false` 才是 F1 停机条款的触发条件，`equal=unknown` 不是 |
| J2 | **打中（现查）** | 四处载体里第四处 `research/perf-by-milestone.md` 跟上了**字面**却对不上**出处**：它第 350 行点名 `research/results/e142-first-txn-dry-run-2026-09-14-round2-slot4096.out`「里逐字是」，而那份产物已归档（工作区没有），版本库里它那一行写的是 `superblock_slot×2` |
| J8 | **打中（现查，三条，一条最干净）** | ① `research/perf-by-milestone.md:350` 那句「里逐字是」这次变成假的，而门禁 88 号只扫 `.claude/kb/**/*.md`（`88-quoted-result-lines.sh:37`），这份文件在任何门禁的射程之外 ② 新加的门禁 90 号在出路里逐字要求「源码与留存产物同改」（`90-term-renames.sh:17`），与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:49`「原样保存的证据不许事后改」正面打架；本次就这么改了 `research/results/e115-system-configuration-completeness-2026-09-07.out` 四行，没有任何检查会报 ③ `.claude/kb/experiments/152-…md:240` 那句「整行抄自 `e152-…final-2026-09-17.out` 第 21、26–29、32 行」变成假的——门禁 88 号**会红**，但报的是「产物里找不到」，给的出路是删行，看不出那几行被改写过 |

**反向接受条款按本文的判定**：J1 打中 ⇒ 按正文第三节，这四处字面量改动整个撤回、术语清扫射程收回到「不进产物的标识符」。**这一条我不替主 agent 采纳**——J1 我量到的是「这个串是判断用的键、对不上时静默降级」，不是「这一次提交改坏了什么」；这一次两端同时改了，`check-segment-registry.py` 与我跑的控制臂都对得上。采不采纳看主 agent 怎么读 J1 那一格的字面（见第二节末「打中之后的四句」）。

## 复跑命令与文件指纹

这条腿跑过的两条臂，都只用**仓里已有的、别人编好的**二进制，没有编译、没有改仓里任何文件；
工作副本与产出都在 `/tmp/claude-1000/archive-rename-r1-opus/`（未入库，见末节）。

```
# 臂甲（现场那份过期的 crates 侧产出，编于 01:32:03 UTC，早于改名的 02:21:05 UTC）
/home/fy5090/code/singlefs/target/release/first_transaction_region_bytes > impl-snapshot-current.txt
/home/fy5090/code/singlefs/research/target/release/e142-first-txn-dry-run impl-snapshot-current.txt > run-current.out
# 臂乙（对照：取留存产物尾部那 21 行改名后的 crates 快照）
grep "name=impl_region_bytes " research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out > impl-snapshot-renamed.txt
/home/fy5090/code/singlefs/research/target/release/e142-first-txn-dry-run impl-snapshot-renamed.txt > run-renamed.out
```

| 文件 | sha256 |
|---|---|
| `impl-snapshot-current.txt` | `be639d075ddf4f013eee70ea687e95034e498b3d150aa5db87b02da127365822` |
| `impl-snapshot-renamed.txt` | `641069185cc1bf1ef2e77e9f349a90e16da3631442e72e08e1c6c0d985b5fa23` |
| `run-current.out` | `d98824b6f1d757a34c2fb969fd820f519026ed887c9f1a6cd906567a20c92566` |
| `run-renamed.out` | `800c43c73dbdf306d5c6787d71cc5781a0262e42c45dffa12e960f8ce53c8243` |

开跑前 `ps -o pid,args -u "$(id -u)"` 看到的：没有性能测量在跑（没有 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`）；
只有另一个会话的 `bash .claude/scripts/../singlefs-ai-sop/scripts/gate.sh … --staged`（pid 1474915）在跑。两条臂都加了 `nice -n 19`，没有等锁。

## J1：改这四处字面量有没有改变任何行为

正文第三节给 J1 的触发观测逐字是「给出一条路径：这个串除了被打印/被断言之外，还被拿去做过判断（解析、比较、查表、当 key）」。给出这条路径。

### 一、那个串是「量 5」的配对键，不是打印出来的装饰

`crates/singlefs-harness/src/first_transaction_regions.rs:168` 与 `:174` 这次把区域名从 `"superblock"` 改成 `"system_configuration"`。
这份文件自己第 78 行逐字写着它是什么：

```
/// 区域清单的一行。`name` 与 E142 装置里同一个结构的标签同名，量 5 按 `region=` 加 `device=` 两边配对。
```

配对在装置这一侧，`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:3932`：

```rust
            fields.get("region").map(String::as_str) == Some(*region) && fields.get("device").and_then(|value| value.parse::<u32>().ok()) == Some(*device)
```

装置那一侧的同名常量在 `:3905`：`region_geometry.push(("system_configuration", …))`。
两侧分属两个互相 `exclude` 的 cargo workspace（`research/scripts/replay.sh` 第 370–374 行的注释写了这件事），
编译期没有任何东西把这两个串钉在一起；把它们钉在一起的只有上面那句 `///` 注释和 `.claude/kb/experiments/142-第一个事务的干跑.md:154` 的一句
「`region=` 与装置 `descriptive_tag()` 同名同序」。

### 二、配对不上时会发生什么：不是红，是静默少比两格（量过）

`find` 落空走的是 `:3935` 那一行：

```rust
            emit(&mut emitter, &format!("name=impl_bytes_equal region={region} device={device} equal=unknown first_diff_offset=none mismatch_bytes=0 reason=no_impl_snapshot_given"));
```

**实测**（臂甲，原样输出，取自 `run-current.out` 末四行）：

```text
E7RESULT name=impl_bytes_equal region=journal_record device=1 equal=true first_diff_offset=none mismatch_bytes=0
E7RESULT name=impl_bytes_equal region=system_configuration device=0 equal=unknown first_diff_offset=none mismatch_bytes=0 reason=no_impl_snapshot_given
E7RESULT name=impl_bytes_equal region=system_configuration device=1 equal=unknown first_diff_offset=none mismatch_bytes=0 reason=no_impl_snapshot_given
E7RESULT name=impl_bytes_equal_summary regions=21 equal=19 unequal=0 snapshot_given=true
```

对照（臂乙，`run-renamed.out` 末行，原样）：

```text
E7RESULT name=impl_bytes_equal_summary regions=21 equal=21 unequal=0 snapshot_given=true
```

臂甲喂进去的那份 `crates/` 侧产出是**现场真有的一份**：`/home/fy5090/code/singlefs/target/release/first_transaction_region_bytes`
编于 `2026-09-21 01:32:03 UTC`，而 `crates/singlefs-harness/src/first_transaction_regions.rs` 改于 `2026-09-21 02:21:05 UTC`——
这个二进制是改名之前编的，它今天吐出来的还是 `region=superblock`（`impl-snapshot-current.txt` 第 21、22 行）。
这不是我捏出来的历史，是这台机器此刻 `target/` 里的状态；我另做的「只改一侧」的 `sed` 变体（`impl-snapshot-oneside.txt`）因此是空操作，两臂输出相同，就不单列了。

三件事要一起看：

1. **21 个区域掉成 19 个，而 `unequal` 仍是 0。** `.claude/kb/experiments/142-第一个事务的干跑.md:154` 记的 F1 停机条款是拿 `equal=false` 触发的
   （那一格逐字：「**第一次真跑触发 F1**：`equal=10`、`equal=false` 11 行」）。`equal=unknown` 不是 `equal=false`，F1 不触发。
2. **同一行里 `reason=no_impl_snapshot_given` 与 `snapshot_given=true` 自相矛盾。** 快照是给了的，落空的是配对键；
   这个 reason 把「名字对不上」讲成「没给快照」，读产物的人会往调用方式上找，找不到真因。这是装置里早就有的一处写法，改名把它变成可达的。
3. **系统配置那两格恰好是量 5 里最强的两格。** 按 `.claude/kb/experiments/142-第一个事务的干跑.md:201`，21 个区域里只有 5 个（根记录、journal 记录 ×2、系统配置 ×2）
   是 ≤4096 字节、**逐字节全比**的，其余 16 个只抽样头尾各 32 字节。掉出去的两格是那 5 个全比格里的 2 个。

### 三、打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」）

- **分不分辨臂**：分辨。臂甲（两端名字不一致）`equal=19`，臂乙（两端一致）`equal=21`，同一个二进制、同一条历史，只有那一个串不同。
- **被判的系统当时看不看得到判别它的东西**：看得到一半。产物里 `equal=19` 与 `regions=21` 都印出来了，人比一眼就看得出；
  但**没有任何检查比这一眼**——装置不 panic、退出码 0（两臂都 `exit=0`）、F1 不触发。
- **满足的是判据字面的哪一个分句**：「当 key」与「比较」两个分句。`fields.get("region") == Some(*region)` 是逐字的相等比较，`find` 是查表。
- **跑前条款给的每个改法在打中的这两格上还中不中**：J1 的反向接受条款是「四处字面量改动整个撤回」。撤回之后这两格**照样中**——
  撤回只是把两端一起换回 `"superblock"`，键还是那个键，两端还是两个 workspace、还是只有一句注释钉着。
  能让这两格不中的改法不在跑前条款里（见下一条）。

### 四、我提的改法（只在我的臂上量过、被攻过零轮）

| 改法 | 改哪一格 | 量过 / 推的 |
|---|---|---|
| A：`find` 落空时改报 `equal=false reason=region_name_not_in_snapshot`（而不是 `unknown` + 假 reason） | 让 F1 在配对键错位时触发 | **推的**（按 `:3932`–`:3935` 的代码推；没实现没跑） |
| B：装置末尾断言 `equal + unequal == regions`，不等就非零退出 | 让「静默少比」变成跑不完 | **推的** |
| C：把区域名清单从 `crates/` 侧导出成一份登记，装置读它而不是自己写一份 | 从结构上拆掉「两个 workspace 各写一份同名串」 | **推的**；代价没量（装置要多读一份输入，`replay.sh` 的 driver 要改） |

三条都没在副本上实现过，只在我的两臂上验证了「今天的写法会静默降级」这一件事。

## J2：段序列那个串横跨四处，有没有一处没跟上

正文第三节给 J2 的触发观测逐字是「四处里任一处仍写旧名，或四处之间的值对不上」。

### 先说没打中的三处（现查，都跟上了）

- **`crates/` 断言**：`crates/singlefs-harness/src/segments.rs:31` 与 6 份测试里的断言全是新名；全仓 `grep -rni superblock`（排 `.git` / `target` / `research/prompts/`）
  剩下 30 处命中分在 9 份文件里，没有一处是「该改而没改」的代码或产物：`research/scripts/sweep-term.py` 10（清扫脚本自己的测试数据与文案）、
  `.claude/kb/term-renames.md` 8（对照表自己）、`.claude/warnings/2026-09-21.md` 4（`en-words.txt` 那条警示）、`.claude/kb/prior-art.md` 2（别家引文）、
  `.claude/kb/decisions-history/2026-09.md` 2（历史节）、`.claude/term-rename-exempt` 1、`.claude/kb/checks-owed.md` 1（C430 立账时逐字抄的旧名）、
  `.claude/kb/experiments/23-journal几何.md` 1（jbd2 的 journal superblock，别家术语）、`records/2026-09-07-D14项2第一轮.md` 1（历史记录）。这几处该不该留归 J7，不归我。
- **kb 登记表 + 留存产物**：`nice -n 19 python3 research/scripts/check-segment-registry.py` 这一次跑出来（原样末行）：

```text
  ✓ 比对了 5 处登记（表格 4 行 + 整条流提示 1 处，段序列与每段步骤种类多重集都与 e142-first-txn-dry-run-2026-09-18-change-count-three.out 的 name=segments 行逐字一致），跳过 0 条标预想的表格行；5 条标「装置钉住」的表格行不与产物比（形状从第二条流推得），第二条流的登记 1 处与钉它的用例相符：第二条流 `2+2+1+2+2+1+18+2+1+18+2+1+4+10+2+1+10+2+1+10+2+1+18+2+1+4+10+2+1+10+2+1+18+2+1+18+2+1+18+2+1+18+2+1+10+2+1+10+2+1+18+2+1+2`（279 次写、2104413 个状态，数组在 crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs 里出现 1 次）
```

  退出码 0。这两处逐字一致，且比对的是**内容**（`STEP_KINDS_PATTERN` 解析出来的 `kinds=` 串，见脚本第 165–166、474 行），不是只比数字。

### 打中的是第四处：`research/perf-by-milestone.md`

这份文件里带 `kinds=` 的只有一行（`grep -n 'kinds=' research/perf-by-milestone.md` 命中 1 行，第 354 行），它已经写成新名。
问题在它上面那句（`research/perf-by-milestone.md:350`，逐字）：

```text
第一个事务写出什么，E142（第一个事务的干跑） 的产物 `research/results/e142-first-txn-dry-run-2026-09-14-round2-slot4096.out` 里逐字是：
```

- 这份产物**工作区里没有**：`ls research/results/e142-first-txn-dry-run-2026-09-14-round2-slot4096.out` → `No such file or directory`（已按归档删掉）。
- 版本库里它那一行是旧名（`git show 3cff909^:research/results/e142-first-txn-dry-run-2026-09-14-round2-slot4096.out | grep -n 'name=segments path=transaction'`，原样）：

```text
34:E7RESULT name=segments path=transaction operations=23 segments=16+2+1+2 closed_form=65543 kinds=[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]
```

- 工作区 `research/perf-by-milestone.md:354` 现在写的是 `…|[system_configuration_slot×2]`。

⇒ 第四处载体上的那一行，与它**自己点名的那份出处**对不上：出处里写 `superblock_slot`，抄件里写 `system_configuration_slot`。
这正是 J2 触发观测后半句「四处之间的值对不上」。

还有一条同型的（在 kb 里，因此门禁看得见、但报错原因是另一回事，放 J8 讲）：
`.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:240` 声明下面那一块「整行抄自 `e152-file-system-benchmark-second-transaction-final-2026-09-17.out` 第 21、26–29、32 行」，
而 `:243` 已改成 `…|[system_configuration_slot×4,barrier]`；那份产物第 21 行在版本库里逐字是：

```text
E7RESULT name=singlefs_inner configuration=singlefs round=2 at_nanoseconds=19555763 inner=segments path=mkfs operations=13 segments=4+1+1+1+4 closed_form=34 kinds=[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[superblock_slot×4,barrier]
```

（取法：`git show 3cff909^:research/results/e152-file-system-benchmark-second-transaction-final-2026-09-17.out | sed -n '21p'`，就是正文点名的第 21 行；
工作区 `152-…md:243` 那一行的 `at_nanoseconds=19555763` 与它逐字相同，只有 `superblock_slot` 换成了 `system_configuration_slot`。E152 按 `research/on-request-experiments.tsv` 只在用户说跑时跑，
`.claude/kb/experiments/152-…md:5` 逐字「**只在用户说跑时才跑**……装置改了不重跑」⇒ 不会有哪一次跑把这一行变成真的。）

### 打中之后的四句

- **分不分辨臂**：分辨。「抄件与它点名的出处逐字相同」这条性质，改名前成立、改名后不成立，两个状态可区分。
- **被判的系统当时看不看得到判别它的东西**：`perf-by-milestone.md` 这一处**看不到**——出处已归档，工作区里没有可比的对照；
  要看得到只能像我这样去 `git show`。`152-…md` 那一处看得到（门禁 88 号会红），但报的原因不对。
- **满足判据字面的哪一个分句**：后半句「四处之间的值对不上」。前半句（「任一处仍写旧名」）**不成立**——四处都写了新名。
- **跑前条款给的每个改法在这几格上还中不中**：J2 的条款是「补上漏的那一处并给它一道会红的检查」。
  补那一处有两种补法，只有一种在这几格上不中：把 `perf-by-milestone.md:350` 与 `152-…md:240` 的措辞从「逐字是 / 整行抄自」
  降成「按当时产物转述、原始产物见版本库 3cff909^」——这样两句都重新为真。
  另一种补法（把门禁 88 号的射程扩到 `research/*.md`）在这一格上**不中**：出处已归档，扩了射程只会多一条判不了的红（与 J3 是同一个坑）。

## J8：这次改动让哪些先前成立的断言不再成立

正文第三节给 J8 的触发观测逐字是「找出一句仓里写着的话，它在这次改动之后变成假的，而没有任何检查会报」。三条，按「有没有检查会报」排序。

### ① `research/perf-by-milestone.md:350`：变成假的，且在所有门禁射程之外（最干净的一条）

那一句与它下面那一行的对不上已在 J2 里给了证据。这里只补「为什么没有检查会报」：
门禁 88 号（kb 正文里整行抄的产物行，产物里逐字找得到）的对象集写死在 `.claude/gate.d/88-quoted-result-lines.sh:37`：

```python
kb = sorted(f for f in glob.glob('.claude/kb/**/*.md', recursive=True)
```

`research/perf-by-milestone.md` 不在 `.claude/kb/` 下 ⇒ 88 号一行都不判它。
我另扫了一遍 `.claude/gate.d/*.sh` 里点名 `research/results` 的 16 个阶段，没有一个把 `research/*.md` 收进对象集
（`grep -rn 'perf-by-milestone' .claude/gate.d/` 命中 0 行）。这份文件是 `CLAUDE.md:89` 登记的「给人读的对比表」，正文里抄了产物行，而没有任何阶段管它。

### ② 门禁 90 号的出路与 `evidence-discipline.md`「原样保存的证据不许事后改」正面打架

这次新加的 `.claude/gate.d/90-term-renames.sh` 第 17 行逐字：

```text
echo "     → 怎么办：python3 research/scripts/sweep-term.py --apply 一次换完，源码与留存产物同改；"
```

而 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:49` 是一整节的标题：

```text
## 原样保存的证据不许事后改
```

同一节 `:52` 与 `:55` 逐字：

```text
**改一个字，产物就不再对应它的输入**，而外面看不出来：文件还在，日期还在，
```
```text
所以改提示只能连同重跑一起改。要补说明就另开一份文件，别动原件。
```

**这一次就这么做了**：`git diff --cached -- research/results/e115-system-configuration-completeness-2026-09-07.out` 改了 4 行，全是把产物里
`逐字「写进超级块」`/`逐字「超级块声明的常量」`/`逐字「与超级块声明的一致」` 改成系统配置的说法。
这几行是产物在 2026-09-07 那次跑里**当作条款原文抄出来的**；`.claude/kb/term-renames.md:9` 写的改名起始日是 2026-09-20 ⇒
改完之后，这份名叫 `-2026-09-07.out` 的产物在断言「2026-09-07 那天 I-8.1 逐字写着『写进系统配置』」，而那天它写的是超级块。

**没有任何检查会报**——这一条我量过，不是推的。`research/scripts/replay.sh:81` 登记 E115 的复跑模式是 `exact`（逐字节比）：

```text
E115|e115-system-configuration-completeness||e115-system-configuration-completeness-2026-09-07.out|exact
```

用仓里已编好的、改名之后编的二进制（`research/target/release/e115-system-configuration-completeness`，编于 2026-09-21 02:26:04 UTC，
源码改于 02:21:06 UTC）现跑一次：

```
cd research && nice -n 19 ./target/release/e115-system-configuration-completeness > /tmp/claude-1000/archive-rename-r1-opus/e115-fresh.out
diff /tmp/claude-1000/archive-rename-r1-opus/e115-fresh.out research/results/e115-system-configuration-completeness-2026-09-07.out   # 无输出
```

退出码 0、`diff` 无输出：**手改过的产物与改名后装置的新鲜输出逐字节相同**，所以 `replay.sh E115` 判绿。
（`e115-fresh.out` 的 sha256 = `a22fbf30bb7688354bff885668c0f638090472312d2df4a9f771875da3ac3243`。）
换句话说：源码与产物一起改之后，「逐字节一致」这道闸**仍然全绿，但它证的已经不是「这份输出是那次跑出来的」**，
只是「这份输出是今天的源码跑出来的」。`evidence-discipline.md:52` 那句「而外面看不出来：文件还在，日期还在」说的就是这个形状。

### ③ `.claude/kb/experiments/152-…md:240`：变成假的，门禁会红，但报的原因不对

那一句声明下面那一块是「整行抄自 `e152-…final-2026-09-17.out` 第 21、26–29、32 行」（J2 里给了该产物第 21 行的原文）。
门禁 88 号这一次确实红了（我现跑 `nice -n 19 bash .claude/gate.d/88-quoted-result-lines.sh`，退出码 1，原样首行）：

```text
  ✗ 这次改动新增或改写的 kb 正文里有 59 行整行抄的产物行，在 research/results/ 的 12 份产物里一份都找不到：
```

`.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:243` 在那 59 行里。
但 88 号报的原因是「在 12 份产物里找不到」（产物已归档），它给的出路逐字是
「产物确实没留存，就删掉这几行并写明『原始输出未留存』」——照这条出路做，那几行被**删掉**，
「它们曾被改写成新名」这件事就此消失，没有任何地方会记下来。
59 行里绝大多数与改名无关（E139、E144、E145、E146 与 E152 的 summary 行，都是产物归档造成的），改名造成的那一行混在里面不显眼。

### 打中之后的四句

- **分不分辨臂**：三条都分辨。①②③ 各自「那句话为真」与「为假」是两个可区分的仓库状态。
- **被判的系统当时看不看得到判别它的东西**：①看不到（门禁射程之外，出处已归档）；②看不到（闸是绿的，而且**因为两边同改所以是绿的**）；③看得到一半（红了，原因不对）。
- **满足判据字面的哪一个分句**：①②满足整句（「变成假的」+「没有任何检查会报」）；③只满足前半句，后半句不满足，按判据字面**③不算打中**，只作旁证。
- **跑前条款给的每个改法在这几格上还中不中**：J8 的条款是「逐条记进欠账，不阻塞提交」。记进欠账对①③有效（措辞改掉就为真）；
  对②**不够**——②是两条规矩在打架，记欠账不会让任何一条退让，下一次全仓改名还会照 90 号的出路再改一次产物。
  ②要么改 90 号的出路（改成「源码改完重跑，产物由重跑产生」），要么在 `evidence-discipline.md` 那一节里为「纯改名的全仓清扫」开一个写明代价的例外。这两条我都没实现、没跑。

## 没打中的形状（试过、取样范围、为什么没中）

按「这四个串被当成数据用」逐条找过，下面几条**没中**，写下来免得下一轮重扫：

1. **`StepKind::name()` 的串被当成排序键**（`crates/singlefs-harness/src/segments.rs:26` 起那个 `match`）。
   没中：种类多重集按 `BTreeMap<StepKind, usize>` 聚合（该文件 `:146`），`StepKind` 的 `Ord` 是 derive 出来的声明序（`:15` 的 `#[derive(… PartialOrd, Ord)]` + `:16`–`:21` 的枚举体），不是字符串序；
   产物里 `[unit_write×16,system_configuration_slot×2,barrier]` 的次序与声明序一致、与字典序不一致，能反证这一点。
   同型地查了 `crates/singlefs-core/src/write_accounting.rs:65` 的 `report_name()`（那个串在 `:78`）：聚合容器是 `:108` 的 `BTreeMap<WrittenStructureKind, WriteCallsAndBytes>`，同样是枚举序。
   顺带算过：即使按字符串排，`superblock_slot` 与 `system_configuration_slot` 在 `root_slot` 与 `unit_write` 之间的相对位置也不变，这条本来也打不响。
2. **设备日志键名 `system_configuration_slot_write_calls=` 被脚本解析**（`crates/singlefs-harness/src/bin/first_transaction_on_device.rs:1083`，
   现读那一行在 `mod tests` 的一条断言里，真正发这一行的是 `describe_publish_writes`）。
   没中：`grep -rn '_write_calls\|_written_bytes\|report_name\|structure=' research/scripts/*.py research/scripts/*.sh .claude/gate.d/*.sh`
   只命中 `research/scripts/e152-tables.py:81` 一行，而它取的是 `singlefs_second_transaction_written_bytes_both_devices` 这个汇总指标名，
   不是按结构种类拆的那组键。真设备那一侧的比对（`publish_writes_against_device`）用的是枚举，不是串。
3. **变异表的锚点里带着这个串**（锚点是按字面在源码里定位的，是标准的「当 key」）。
   没中：`grep -n 'system_configuration_slot\|superblock' crates/mutations.tsv` 命中 0 行；
   `git diff --cached -- crates/mutations.tsv` 这次只改了两条与 checker `walked` 变量有关的锚点、加了一条新变异，与改名无关。
   `research/mutations/e142_first_transaction_dry_run.tsv` 确实有 4 行（第 21、34、65、66 行）带这个词，但带的是 Rust 标识符
   （`StepKind::SystemConfigurationSlot`、`SYSTEM_CONFIGURATION_SLOT_BYTES`）与变异条目名，不是这一轮改的那四个字符串字面量；
   `git diff --cached --stat -- research/mutations/e142_first_transaction_dry_run.tsv` 这次是空的（那一批标识符在更早的提交里就换过了）。
4. **`name=width structure=…` 行被门禁或脚本解析**。没中：`grep -rn 'structure=\|name=width' research/scripts/replay.sh .claude/gate.d/ .claude/kb/layout/01-first-txn.md`
   只命中 `replay.sh:294`（E1 的 `name=width unit=512 jsn_bytes=12`，另一个实验、另一组字段）与 layout 里两句「`name=width` 28 行 expected == actual」的散文。没有解析器。
5. **kb 登记表、`crates/` 断言、留存产物三处之间对不上**。没中：`check-segment-registry.py` 这一次跑绿（原样末行已抄在 J2 一节），
   它比的是解析出来的 `kinds=` 串本身（脚本 `:165–166` 抓串、`:474` 逐字比），不是只比 `4+1+1+1+4` 那串数字。
6. **`research/perf-by-milestone.md` 里还有别的段序列串没跟上**。没中：`grep -n 'kinds=' research/perf-by-milestone.md` 全文只命中 1 行（第 354 行）。

取样范围（数是命令数出来的，不是手数的）：

```
grep -rni "superblock" . --exclude-dir=.git --exclude-dir=target | grep -v "^research/prompts/" | wc -l        # 30（9 份文件）
grep -rn "system_configuration_slot\|superblock_slot" . --exclude-dir=.git --exclude-dir=target \
  | grep -v "^research/prompts/" | grep -v "^research/results/e155-fourth" | wc -l                             # 127（42 份文件）
```

30 处旧名逐处看过（分布：`research/scripts/sweep-term.py` 10、`.claude/kb/term-renames.md` 8、`.claude/warnings/2026-09-21.md` 4、
`.claude/kb/prior-art.md` 2、`.claude/kb/decisions-history/2026-09.md` 2、`.claude/term-rename-exempt` / `.claude/kb/checks-owed.md` /
`.claude/kb/experiments/23-journal几何.md` / `records/2026-09-07-D14项2第一轮.md` 各 1）。
127 处新名**没有逐处看**。42 份文件按目录分：`crates/` 18、`research/e7-index-bench/` 9、`.claude/kb/` 8、
`research/results/` 2、`records/` 2、`research/mutations/` 1、`research/` 1（就是 `perf-by-milestone.md`）、`.claude/` 1。
逐行看过的是 `crates/` 那 18 份（第一遍 `grep -n` 的输出整份读过）、`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`、
`crates/mutations.tsv` 与 `research/mutations/e142_first_transaction_dry_run.tsv`、`research/results/` 那 2 份、`.claude/kb/` 那 8 份、`research/perf-by-milestone.md`。
**没看的**：`research/e7-index-bench/src/bin/` 另外 8 份装置（E100 / E102 / E126 / E147 一类，那几份里这个串是各自实验的字段表标签，不进第一个事务的四处载体）与 `records/` 那 2 份历史记录。
看过的那些按「谁读它」分了五类（打印 / 断言 / 配对键 / 人读的散文 / 对照表自己），只有配对键那一类进了 J1。

## 这条腿自己的限度

1. **J1 我量的是「静默降级」，不是「这一次改坏了」。** 两端这一次是一起改的；`check-segment-registry.py` 绿、我的臂乙 `equal=21`。
   J1 那一格的字面只要求「被拿去做过判断」，这一条我给足了证据（`:3932` 的 `find` + 两臂分辨）；
   但若主 agent 读 J1 为「这次提交已经改变了行为」，那这一格**不算打中**——两种读法下的结论不同，判由主 agent 做。
2. **臂甲用的是机器上现成的一个过期构建产物，不是我构造的源码状态。** 我没有编译任何 Rust，也没有在副本上改过源码；
   `replay.sh` 的 `driver_e142` 每次 `cargo run` 现编 `crates/` 那一侧，所以**走 replay 这条路时过期二进制这个具体触发不会发生**。
   真正可达的同型历史是「两侧源码只改了一侧」（豁免表挡住一侧、或分两次提交），那一条我**没有实测**，只按代码推。
3. **E142 那份留存产物是不是真重跑出来的，我没能独立证实。** 它与 3cff909^ 的版本差 17 行（`diff` 计数：`<` 17 行、`>` 17 行，
   多出 0 行、少掉 0 行），其中 16 行是改名、1 行是 `name=gap id=G25` 里的中文「超级块 → 系统配置」。
   装置是确定性的 ⇒ 「重跑」与「文本清扫」两种历史产生的字节完全相同，从内容上分不开。这不构成指控，只是说这一格我分辨不了。
4. **E155 第四次跑的材料按禁读清单没读**，所以 `research/results/e155-fourth-run-group-commit-concurrency-2026-09-21.out` 里的 `system_configuration_slot` 我没核。
   `research/results/e155-third-run-release-cascade-2026-09-20-stage1.out` 是别的会话没提交的文件，按共用约束没碰。
5. **没有跑整轮门禁**，只单独跑了 88 号一个阶段（读性质）与 `check-segment-registry.py`（`stage-owners.tsv` 里没有登记给这个 agent 名的阶段）。

## 没做什么

- 不判 J3–J7，也没读这一轮别的腿的提示与产出；主 agent 通报的 J3 那次打中只当背景，没有复核、没有在 J3 上做任何工作。
  ⚠️ 收尾统计文件分布时，`grep -rl` 的文件名清单里出现过本轮本地腿的提示文件名一次（那一次 `grep -v` 的 `^\./` 前缀没匹配上，
  因为这台机器上的 grep 输出不带 `./`）。**只出现了文件名，没有读到它任何一行内容**：出内容的那几次 `grep` 都被 `head` 截在它之前，
  之后的统计只取了文件名与计数。后面的命令改成了 `grep -v "^research/prompts/"`。
- 不替主 agent 采纳任何反向接受条款；第三节的三条改法（A/B/C）**都只在纸上，被攻过零轮，一条都没实现没跑**。
- 没有编译 Rust、没有改仓里任何代码、没有做 git 写操作。只写了这一份报告文件。
- 草稿与产出全部在 `/tmp/claude-1000/archive-rename-r1-opus/`（`impl-snapshot-current.txt`、`impl-snapshot-renamed.txt`、`impl-snapshot-oneside.txt`、
  `run-current.out`、`run-renamed.out`、`run-oneside.out`、`e115-fresh.out`、`e142-old.out`、`e142-diff.txt`、`gate88.txt`）。
  **为什么不入库**：这些不是实验产物，是这条腿为了分辨臂临时跑的对照输出（用的还是现成的过期构建产物），
  按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「所有旧数据都只是参考」与共用约束「腿在副本上量的数不进 kb」，
  它们不该进 `research/results/`；要留存的话得走 E142 的正式复跑。主 agent 要拷走就在上面那个路径。
