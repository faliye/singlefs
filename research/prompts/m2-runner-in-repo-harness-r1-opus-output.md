# m2-runner-in-repo-harness-r1 云端攻方（Opus）报告：W1

腿：云端攻方（Opus），分到 W1（入库装置与「不与实现共用代码」）。写于 2026-09-23 02:50 UTC（JST 11:50）。
开工快照 `research/prompts/m2-runner-in-repo-harness-r1-snapshot/opening.sha256` 四个文件全部 `OK`（本次 `sha256sum -c` 现跑）。

## 各格判定一览

| 格 | 判定 | 判据第 1 条要的三样 | 在哪一节 |
|---|---|---|---|
| W1 | **站不住** | ① 量：E156 报出的代价数 `beta2_hr_end` 的 `deferred0` / `allocated0`、`hr_rollback_target.candidate_count`、`q7b.total_l1`；② 共用的那段代码：`crates/singlefs-format/src/lib.rs:186` 的 `pub const ROOT_RING_SLOTS_PER_REGION: u64 = 8`（实现侧 `root_ring.rs:36`、`recovery.rs:344`、`system_configuration.rs:349`；装置侧 `e156_allocation_basis_counts.rs:29,472,473`）；③ 一起错的机制：装置的工作负载长度 `6 × (R × S)` 与实现的环长 `R × S` 是同一个 `S` 的两个下游，`S` 改了两边同向同倍地动，逐格自洽 | 二、三 |

**实测已打中**：在仓副本上把 `S` 由 8 改成 4，E156 报出的代价数从 `deferred0=484` 变成 `deferred0=370`（β2）、回退候选数从 24 变成 12、`q7b.total_l1` 从 96 变成 85，
而 `matches_registered=true`、`g8_holds=true`、`row0_eq_row1=true`、`i31_red` / `i52_red` 的每一个布尔位**逐格不变**。见第三节的原样 diff。

## 复跑

仓副本 `/tmp/claude-1000/runner-harness-opus/repo`（`rsync -a --exclude target --exclude .git`，2026-09-23 02:30 UTC 取），两个变异点跑完已逐字节还原（`diff` 与工作区相同，本次现跑）。
副本上量出的数不入库，下面所有数都标着是副本上的。

```
cd /tmp/claude-1000/runner-harness-opus/repo
export CARGO_TARGET_DIR=/tmp/claude-1000/runner-harness-opus/target
nice -n 19 cargo build -j 6 --release -p singlefs-harness --bin e156_allocation_basis_counts
nice -n 19 /tmp/claude-1000/runner-harness-opus/target/release/e156_allocation_basis_counts > out/baseline.out
# 变异：crates/singlefs-format/src/lib.rs:186 的 8 改成 4，重编重跑
```

| 文件 | sha256 |
|---|---|
| `/tmp/claude-1000/runner-harness-opus/out/baseline.out` | `5b7381b9d84f8cb12bc266db89f9322bcfd60fc04cd02d695089f872677cb3de` |
| `/tmp/claude-1000/runner-harness-opus/out/m1_ring4.out` | `f2edd6fef74c7c19d9232b65d1449ad0c6a6834311c34f3d7b23a4d6635c9ade` |
| `/tmp/claude-1000/runner-harness-opus/out/m2_onslot2.out` | `50bfd07e0e2b926ae0dcc368ddebce9569ac5c3fdcb7c8c071bc37a2432f7a19` |

单次跑 2.29 秒（`/usr/bin/time -v` 的 `Elapsed (wall clock) time (h:mm:ss or m:ss): 0:02.29`），峰值常驻 186584 KiB。
开跑前 `ps -o pid,args -u "$(id -u)"` 现看：没有 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`；
有另一条会话的 `cargo test -j 8 -p singlefs-harness --test second_transaction_step_zero_layer0`（pid 3818460）在 `/tmp/claude-1000/placement-opus/` 下跑，我用 `-j 6` + `nice -n 19`，副本自带独立 `CARGO_TARGET_DIR`，没等锁。

## 二、W1 的共错：`ROOT_RING_SLOTS_PER_REGION`

### 2.1 那个量

E156 第一段为 alloc-basis 四条岔路交出的代价数，具体是这四个 E7RESULT 字段（副本基线跑出的原样值）：

```
E7RESULT name=geometry ring_length=24 baseline_workload_length=144 hr_prefix_length=72 hr_tail_length=72
E7RESULT name=hr_rollback_target instance=1 txg=29 candidate_count=24
E7RESULT name=beta2_hr_end txg=102 instance=2 allocated0=496 free0=211472 deferred0=484 allocated1=496 free1=211472 deferred1=484 referenced=12 g8_holds=true row0_eq_row1=true
E7RESULT name=q7b red_l1=0 total_l1=96 first_red=
```

`deferred0=484`（延迟释放槽数的稳态）是这一段要交的代价数本身；`candidate_count=24` 是回退可选的根槽数；`total_l1=96` 是岔路 7 扫了多少个状态。

### 2.2 那段共用代码

`crates/singlefs-format/src/lib.rs` 第 185–186 行，两行原样：

```
pub const ROOT_RING_REGIONS: u64 = 3;
pub const ROOT_RING_SLOTS_PER_REGION: u64 = 8;
```

**实现侧的四个下游**（本次 `grep -n` 现取）：

- `crates/singlefs-core/src/root_ring.rs:36`：`        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % ROOT_RING_SLOTS_PER_REGION,` —— 决定 txg t 落哪个环槽，也就是环多久绕一圈。
- `crates/singlefs-core/src/recovery.rs:344`：`        for slot in 0..ROOT_RING_SLOTS_PER_REGION {` —— 决定 `readable_roots` 枚举出多少个可回退的根。
- `crates/singlefs-core/src/system_configuration.rs:349`：`            writer.put_u8(u8::try_from(ROOT_RING_SLOTS_PER_REGION).expect("S"));` —— 把同一个数写进盘上的系统配置。
- `crates/singlefs-core/src/root_ring.rs:44`：`    ROOT_RING_SLOTS_PER_REGION * u64::from(fixed_structure_slot_spacing)`。

**装置侧的三处**（`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`，本次 `grep -n` 现取）：

- 第 29 行：`    JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS, ROOT_RING_SLOTS_PER_REGION, SLOT_BYTES,`
- 第 472 行：`    let ring_length = ROOT_RING_REGIONS * ROOT_RING_SLOTS_PER_REGION;`
- 第 473 行：`    let baseline_workload_length = 6 * ring_length;`

### 2.3 两边为什么会一起错

E156 的跑前登记 `research/prompts/e156-preregistration.md:290` 把 H0 段写成「mkfs → 首次挂载暖机 → 工作负载段 6N 次」，`N = 3S`（同文件 278 行：「记 N = 3S」）。
**`S` 在这句话里不是一个被独立钉死的数，是从被测实现的编译期常量读出来的**：装置第 472 行直接 `use` 了 `singlefs_format` 的那两个 `pub const`。于是

- 实现的环长 = `R × S` 槽（`root_ring.rs:36` 取模的那个 S）；
- 装置的工作负载长度 = `6 × R × S` 次覆盖写（`e156:472–473` 的同一个 S）；
- 盘上系统配置里的 S 也是它（`system_configuration.rs:349`），checker 再从那个字节把它读回来（`crates/singlefs-checker/src/image.rs:147`：`        slots_per_region: u64::from(slot[362]),`，再由 `walk.rs:2494` 算 `root_ring_slot_count`）。

⇒ **S 不论取什么值，「跑满 6 圈」这句话都自洽**。环短了，装置的负载跟着短，绕圈数仍是 6；checker 读到的 S 也跟着短，它判的「环槽都有效」也仍然成立。
装置、实现、checker 三条路径上的 S 是同一个字面量经三条路送到的同一个数，谁都没有拿它跟「定案该是多少」比过。
这正是 `.claude/singlefs-ai-sop/rules/test-discipline.md:114` 那一节的名字：「只让多条臂互相比，测不出「所有臂一起错」」；它第 120 行开的方子逐字是
「⇒ **每写一条互比断言，旁边就要有一条把绝对值钉死的断言。**」——S 这一维上那条钉绝对值的断言**不存在**。

## 三、实测：副本上把 S 由 8 改成 4，代价数整体平移而一格不红

变异只改一行（副本 `crates/singlefs-format/src/lib.rs:186`）：`pub const ROOT_RING_SLOTS_PER_REGION: u64 = 8;` → `... = 4;`。
重编重跑之后 `diff baseline.out m1_ring4.out` 的原样输出（本次现跑）：

```
7,8c7,8
< E7RESULT name=geometry ring_length=24 baseline_workload_length=144 hr_prefix_length=72 hr_tail_length=72
< E7RESULT name=baseline_workload_truncated_by_write_failure requested_length=144 actual_length=49 failed_at_step=50 failure=AllocationRecordsExceedOneNode { records: 820, capacity: 812 }
---
> E7RESULT name=geometry ring_length=12 baseline_workload_length=72 hr_prefix_length=36 hr_tail_length=36
> E7RESULT name=baseline_workload_truncated_by_write_failure requested_length=72 actual_length=49 failed_at_step=50 failure=AllocationRecordsExceedOneNode { records: 820, capacity: 812 }
11,15c11,13
< E7RESULT name=hr_prefix_truncated_by_write_failure requested_length=72 actual_length=49 failed_at_step=50 failure=AllocationRecordsExceedOneNode { records: 820, capacity: 812 }
< E7RESULT name=hr_rollback_target instance=1 txg=29 candidate_count=24
< E7RESULT name=hr_tail_truncated_by_write_failure requested_length=72 actual_length=47 failed_at_step=48 failure=AllocationRecordsExceedOneNode { records: 816, capacity: 812 }
< E7RESULT name=hr_lengths requested_prefix=72 actual_prefix=49 requested_tail=72 actual_tail=47
< E7RESULT name=beta2_hr_end txg=102 instance=2 allocated0=496 free0=211472 deferred0=484 allocated1=496 free1=211472 deferred1=484 referenced=12 g8_holds=true row0_eq_row1=true
---
> E7RESULT name=hr_rollback_target instance=1 txg=28 candidate_count=12
> E7RESULT name=hr_lengths requested_prefix=36 actual_prefix=36 requested_tail=36 actual_tail=36
> E7RESULT name=beta2_hr_end txg=77 instance=2 allocated0=382 free0=211586 deferred0=370 allocated1=382 free1=211586 deferred1=370 referenced=12 g8_holds=true row0_eq_row1=true
17c15
< E7RESULT name=q7b red_l1=0 total_l1=96 first_red=
---
> E7RESULT name=q7b red_l1=0 total_l1=85 first_red=
35c33
< E7RESULT name=done emitted=35
---
> E7RESULT name=done emitted=33
```

退出码 0（`echo "exit=$?"` 打出 `exit=0`）。**逐格相等的是哪几格**：两份产物有 26 行逐字相同（`comm -12 <(sort baseline.out) <(sort m1_ring4.out) | wc -l` ⇒ `26`，基线共 35 行），其中包括

- `name=k1_after_first_transaction … registered_item1_slots=13 registered_item5_slots=1 matches_registered=true`——装置里唯一那条钉绝对值的断言（`e156:397–399` 的 `allocated0 == 13 && deferred0 == 1`）照样 `true`；
- `beta0_k1_end` / `beta1_h0_end` / `beta2_hr_end` 三行的 `referenced=12 g8_holds=true`，`beta1` / `beta2` 的 `row0_eq_row1=true`；
- `beta0_today_checker` / `beta1_today_checker` / `beta2_today_checker` 三行的 `i31_red` / `i52_red` 一位不差；
- 全部 7 行 `q7a` 的六个布尔位（`comm -12` 的共同行里 `q7a` 7 行、`pc_check` 6 行，均与基线逐字相同）、两行 `q7c2_threshold_at_least` 的 `at_least_holds=true flips_red_to_green=true`、六行 `pc_check_bk*`。

⇒ **代价数 `deferred0` 从 484 掉到 370（差 114 槽 = 1.78 MiB），回退候选从 24 掉到 12，而装置里每一条互比断言、每一个 checker 判定都逐格不变**。
装置报的是「6 圈之后的稳态」，实现给的确实是 6 圈之后的稳态——两边一致，一起错在「一圈有多长」上。

### 3.1 checker 为什么不会说

`check_pool_image` 走的是字节级，但它读的 S 来自盘上系统配置那一个字节，而那个字节是实现从同一个常量写下去的：

- 写：`crates/singlefs-core/src/system_configuration.rs:349`　`            writer.put_u8(u8::try_from(ROOT_RING_SLOTS_PER_REGION).expect("S"));`
- 读：`crates/singlefs-checker/src/image.rs:147`　`        slots_per_region: u64::from(slot[362]),`
- 用：`crates/singlefs-checker/src/walk.rs:2494`　`    let root_ring_slot_count = geometry.regions * geometry.slots_per_region;`

全仓对 `slots_per_region` 只有这三处加 `image.rs:111` 的字段声明、`image.rs:229` 的循环、`history.rs:2121` 的乘法（`grep -rn 'slots_per_region' crates/ --include=*.rs`，6 个命中），
**没有任何一处判它落不落在区间里**（`grep -rn '4..16\|(4..=16)\|slots_per_region <\|slots_per_region >' crates/ --include=*.rs` 对 `slots_per_region` 零命中）。
所以 checker 判的是「盘上这一份自洽」，不是「S 取得对」。

## 四、这个错是可达的，不是我编出来的

要让「S 写错」算一条可达的历史，得说清今天谁有资格说 S 该是 8。四处现查，四处都说不出：

1. **定案没定 8**。`.claude/kb/decisions/22-单元原子性怎么合成.md:58` 那一行逐字（整行抄）：

   `| **每区槽数 S** | 住系统配置。**下界 k_tol + 2 = 4**（D16（发布语义） 已定项 1：**每区槽数 S** 要装得下 4 个状态加一次带 k_tol 次根槽写失败的处置——那一项是直接在 S 上量的，不是在 R × S 上；「环深」这个词在 C131（栈深与保护窗口的量纲） 出过一次事，这里不用它），**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**；4..16 之间取值不碰任何一条边。⚠️ **具体取多少不是格式决策**——格式承诺的是「S 是系统配置字段 + 挂载时判区间」，取值由第一版实现按 D25（目标负载优先级） 的负载形态定 |`

   定案只给区间 `4..16`，并且明写 S 是**系统配置字段**、**挂载时判区间**。8 与 4 都在区间里；实现把它写成了编译期 `pub const`，区间判也没写（见 3.1）。

2. **kb 里唯一提到 8 的那句话自称是猜的**。`.claude/kb/milestone/01-first-txn.md:54` 整行：

   `- 根环三个区域落在 `1 MiB + r × P × chunk`。**预想**：P = 3、`chunk` = 1 MiB、每区 8 槽、槽距 4096；区域 0 / 1 / 2 归盘 0 / 1 / 0——`

3. **门禁 27 号绑不到它**。kb 里一共登记了 21 个 `format-const` 名（`grep -rho 'format-const: *[A-Z_0-9]*' .claude/kb/ --include=*.md | sed 's/.*format-const: *//' | sort -u | wc -l` ⇒ `21`），
   `ROOT_RING_SLOTS_PER_REGION` 与 `SLOT_BYTES` 都不在里面（`grep -rn 'format-const: *ROOT_RING_SLOTS_PER_REGION\|format-const: *SLOT_BYTES' .claude/kb/ | wc -l` ⇒ `0`）。
   27 号自陈（`.claude/gate.d/27-format-constants.sh` 第 22–24 行整段抄）：

   ```
   # ⚠️ **它管得了哪一半**：只检查**已经登记了标记**的常量。一个新加的、没登记标记的
   # 格式常量仍然可以静默漂移——那一半靠人，落点见 kb/checks-owed.md。
   ```

   门禁 92 号（`layouts.tsv` 那一行确实把 `crates/singlefs-format/src/lib.rs` 与三个 checker 文件配了对）判的是「格式常量变了而 checker 路径没跟」，不判取值对不对；它自陈「④ 判的是「格式变了没人跟」，判不了 checker 跟得对不对——后者要人看」。

4. **跑前登记的锚点 A2 引的是源码自己**。`research/prompts/e156-preregistration.md:427` 那一行的验证依据列写的是 ``` `singlefs-format/src/lib.rs` 185–192；`root_ring.rs` 33–38 ``` ——
   即「S = 8」这个锚点的权威就是被测实现的那一行常量。A2 能证的是「装置读到的 S 等于实现里的 S」，不是「S 是 8」。
   这是同一条互比：`.claude/singlefs-ai-sop/rules/test-discipline.md:114` 那一节的标题。

⇒ 这条历史只要：实现员按 D22 的区间随手取 8（或任何 4..16 的数）写成 `const`，执行员照「入库装置」分支 `use` 它写负载长度，就成立。**不需要任何人犯错**——
今天仓里就是这个状态，**E156 已经交出去的 `deferred0=484` 就是在这个状态下量的**，而它到底是不是定案几何下的稳态，仓里没有任何东西回答得了。

## 五、定义里那三条限制挡不挡得住

`.claude/agents/experiment-runner.md` 新分支自带的三条（正文第二节转述的）逐条对着上面这个场景过：

| 限制 | 挡不挡得住 | 为什么 |
|---|---|---|
| 「只读、只驱动与观测」 | **挡不住** | 装置确实只读、只驱动。`use ROOT_RING_SLOTS_PER_REGION` 就是「读」。共错发生在读到的那个值本身上，跟读写无关 |
| 「不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`」 | **挡不住** | 装置一个字都没改这两个 crate。它改的是「拿什么数当负载参数」，那是装置自己的第 472–473 行 |
| 「变异表追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑」 | **挡不住** | 变异测试证的是「断言会红」。S 这一维上根本没有断言可红——把 S 改掉，装置里每一条断言（含 `assert_eq!(allocated, 13)`、`assert_eq!(referenced, 12)`）都照样绿，见第六节 |

⇒ W1 判 **站不住**：`.claude/rules/implementation-first.md:14` 第 4 条逐字「4. **`research/` 下的装置与模型只用来独立验证**：它们不与实现共用代码，用来核实现、核推论。装置的数与实现对不上时两边都查，不许默认装置对，也不许默认实现对。」
最后那半句「装置的数与实现对不上时两边都查」预设的是**两边会对不上**；入库装置共用常量之后，这一类错**永远对得上**，那条纪律的触发条件被消掉了。

### 缺的那一句（我提的改法，只在我的模型上量过、被攻过零轮）

> **入库装置不许从 `crates/` 里 `use` 任何决定工作量、几何或判据门槛的常量**：这类参数一律在装置里写成本地 `const`，值抄自 kb 的 `format-const` 登记位并在文件头注明登记位路径与行号；
> 装置**必须**另有一条断言把它与实现侧的同名常量比（`assert_eq!(LOCAL_S, singlefs_format::ROOT_RING_SLOTS_PER_REGION)`），两边不等时红。

装置里已有一个现成的样板：`e156_allocation_basis_counts.rs:40` 的 `const INSTANCE_TABLE_SPAN_SLOTS: u64 = 2;` 就是本地抄了一份，而不是 `use` 过来的——只是它今天没有那条回比断言，也没注明登记位。

## 六、打中之后的四句（evidence-discipline.md:176 那一节）

1. **分不分辨臂？** 分辨。三格 W1/W2/W3 的三条结论（站得住 / 站得住但限制不够 / 站不住）里，这个构造只把「站得住」一条打掉：
   它给出的是「定义里已有的三条限制**挡不住**一个具体的共错场景」，而不是所有结论一起出局。W2、W3 不受它影响（那两格另有腿）。
2. **被判的系统当时看不看得到判别它的东西？** 看不到，而这正是打中的内容。E156 跑完那一刻，装置、实现、checker 手里的 S 是同一个数；
   要区分「S=8 是定案」与「S=8 是随手取的」，得去 `.claude/kb/decisions/22-单元原子性怎么合成.md:58` 看——那不在装置能观测的东西里。
   这不构成「判别子观测不到」那一种形态：判据第 1 条要的是**我**给出量与共用代码并说清机制，不是要装置自己分辨。
3. **满足的是判据字面的哪一个分句？** 第 1 条的三个分句全中：「给出了一个具体的量」＝`beta2_hr_end.deferred0`（484，副本上 S=4 时 370）；
   「那段共用代码」＝`singlefs-format/src/lib.rs:186` 经 `e156:472–473` 与 `root_ring.rs:36` 的两条下游；
   「说得出两边为什么会一起错」＝装置的负载长度与实现的环长是同一个 S 的同倍下游，第二节 2.3 有机制、第三节有实测 diff。
   **不是**判据点名「大概率触发」的那个通道：它猜的是「E156 用到的某个**格式常量**」，我打中的确实是格式常量，但不是 C94 那一半（登记值与正文增量对不上）——
   `ROOT_RING_SLOTS_PER_REGION` **根本没登记**，属于 C94 第四列里自陈仍欠的覆盖面那一半。归账时别记进「登记值错了」。
4. **跑前条款给的每个改法，在打中的那几格上还中不中？** 这一轮跑前条款没给改法（判据第 4 条明写「这一轮不许下定案」）。
   我自己在第五节提的那一句，在打中的两格上**还中一半**：把 S 写成本地 `const` 并加回比断言，能挡住「装置与实现对 S 的理解不一致」，**挡不住**「kb 与实现一起取错」——
   因为 kb 今天对 S 只有 `milestone/01-first-txn.md:54` 那句「**预想**」，没有 `format-const` 登记位可抄。要真挡住，前置是先把 S 登记进 kb（这是 C94 第四列那半笔欠账的具体一例）。

## 七、没打中的形状

**M2：`TransactionUnit::span_slots()` 那一维打不穿**（副本上实跑，`out/m2_onslot2.out`）。
把 `crates/singlefs-core/src/transaction.rs:697` 那一档 `PlacementRule::CommitGenerated(UnitFootprint::OneSlot) => 1,` 改成 `=> 2,` 之后，原样头三行：

```
E7RESULT name=k1_after_first_transaction txg=3 allocated_slots=13 free_slots=211955 deferred_slots=1 registered_item1_slots=13 registered_item5_slots=1 matches_registered=true
E7RESULT name=k1_placements txg=3 units=50180:2,50240:2,50242:2,50244:2,50245:2,50246:2,50247:2,50248:2
E7RESULT name=beta0_k1_end txg=3 allocated0=13 free0=211955 deferred0=1 referenced=18 g8_holds=false
```

`allocated_slots` 停在 13 不动而 `referenced` 从 12 变 18 ⇒ **G27 当场转红**，随后装置在 `e156:656` panic（退出码 101）：
`HR 管理员回退: RollbackToVersionWithoutFileUnsupported(...)`。
⇒ 分配落点的跨度这一维上，`PoolAllocator` 的计数与 `span_slots()` **不是**同一条路（allocator 有自己的一份），G27 在这一维上有真判别力，且 K1 那条钉绝对值的断言（13 / 12）也盯着。
**这条攻击试过的形状与取样范围**：只试了 `OneSlot` 这一档由 1 改 2；没扫 `UserData`、`TwoSlotsAligned` 两档，也没扫「只改 `InstanceTable` 的 `placement()`」那一路（后者 E156 跑不到，`referenced` 三个基底上恒为 12，实例表从没进过 `units`）。

**没试的第二类（推的，没跑）**：`SLOT_BYTES`（`crates/singlefs-format/src/lib.rs:12` `pub const SLOT_BYTES: u64 = 16384;`，同样不在 21 个登记名里）。
装置在 `e156:446`、`e156:770`、`e156:831–832`、`e156:847` 用它把「注入 k 个槽的延迟释放偏差」换算成字节，而实现在 `crates/singlefs-core/src/transaction.rs:2292/2297/2303` 用同一个常量把记账三项由槽换成字节写上盘。
⇒ 岔路 7 的判别力自证（`q7a` / `q7c2_threshold_at_least`）所谓「一个槽的偏差」是拿被测实现自己的槽宽定义的，SLOT_BYTES 错了自证照样通过。**这条只推、没跑**：没试是因为 `SLOT_BYTES` 与 `DATA_UNIT_BYTES = 32768`、`NODE_BYTES = 16384` 之间还有别处的耦合，单改一个大概率编译期或运行期直接炸，不构成「没人会说」的干净反例。

## 八、这条腿自己的限度

- **只攻了 W1**，W2、W3 一个字没判（正文第四节分工）。不替主 agent 采纳任何改法。
- **副本上的数不入库**：第三节、第七节全部数出自 `/tmp/claude-1000/runner-harness-opus/`，工作区 `crates/` 一个字节没动（跑完 `diff` 过两个变异点，与工作区逐字节相同）。
  草稿产物停在 `/tmp/claude-1000/runner-harness-opus/out/`（三份 `.out` 加一份 `baseline.time`），**现在不入库**的理由：这一轮是三方论证的一条腿，不是实验执行，没有跑前登记、没有实验页，按 `.claude/rules/implementation-first.md:14` 也不该拿副本上的数当实现的数。主 agent 要留就从那个路径拷。
- **没跑门禁**：`.claude/gate.d/stage-owners.tsv` 里没有登记给「云端攻方」这个名字的阶段。没编译工作区、没跑 `gate.sh`。
- **S=4 是我挑的一个区间内取值，不是「正确值」**。我没有、也不可能给出 S 的定案值——D22 已定项 2 明写取值不是格式决策。这条攻击证的是「装置报的代价数随一个没人钉的参数整体平移而无人报警」，不是「484 是错的」。
- **只跑了一次**（每个变异一次）。`.claude/rules/three-way-inference.md:147`「一条腿只抽一次样不算一次观测」管的是否定结论；这里是肯定结论（打中），但 S 只取了 4 这一个反事实值，没扫 16。

## 九、什么观测会推翻这条结论

1. 仓里出现一处把 S（或由它导出的环长 24）**独立于 `ROOT_RING_SLOTS_PER_REGION` 钉死**的断言或门禁——例如 kb 里给 S 加 `format-const` 登记位并被门禁 27 号绑住，或 checker 加上 D22 要的「挂载时判区间」。那时第四节第 3、4 条塌掉，W1 至少降为「站得住但限制不够」。
2. `.claude/agents/experiment-runner.md` 那三条限制里能读出「不许 `use` 实现侧常量」的字面——我核的是正文第二节转述的三条，主 agent 若贴出定义原文里另有一句，第五节那张表要重做。
3. 副本上把 S 改回 8 之后 `deferred0` 不回到 484——那说明我的 diff 有别的自变量混进来，第三节作废。
4. `deferred0` 这个量其实不进 alloc-basis 的任何一条判据（即 E156 第一段交出去的结论不依赖它）——那样「代价数错了没人说」就没有后果，打中降级为无害。
