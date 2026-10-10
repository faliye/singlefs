# m3-batch52-54-code-r1 攻方腿（Opus）报告：删除线第 52–54 批代码轮第一轮（2026-10-10）

立场：云端攻方，攻 K1–K5（K6 顺带），在冻结副本的拷贝上造反例。被判对象是冻结副本 `~/.local/share/singlefs/work/m3-fixes-code-r1/repo/`（`crates/` 与 `research/prompts/m3-batch52-54-code-r1-snapshot/crates-sha256.txt` 25 行逐个对上；主工作区 `crates/` 这 25 份此刻也对上，下面的代码行号两边通用）。前几轮判决：`research/prompts/m3-batch50-code-r1-main-verification.md`。

## 复跑

模型目录 `research/prompts/m3-batch52-54-code-r1-opus-model/`。从空目录起：
```
SINGLEFS_REPO=$HOME/.local/share/singlefs/work/m3-fixes-code-r1/repo \
  bash research/prompts/m3-batch52-54-code-r1-opus-model/rerun.sh <空工作目录> all
```
`all` 依次跑 base-build、base-k1、base-k2-numbers、base-k2-random、base-k2-product、base-k4、base-k3、k5、15 种变异、compare，挂钟约 25 分钟（另加内存包装排队）；前台有超时的环境按阶段名分几次起，同一个工作目录接着跑。只拷 `Cargo.toml`、`Cargo.lock`、`crates/`；base 与每一种变异各一份副本、各用自己的 `CARGO_TARGET_DIR`；同一时刻只起一条 cargo；内存上限 `OPUS_MEMORY_CAP`（缺省 10G），线程 `OPUS_THREADS`（缺省 12）。k5 阶段读第 52、54 批实现员草稿目录里的证据（`OPUS_B52_EVIDENCE`、`OPUS_B54_EVIDENCE`，缺省就是那两份目录）。

交回前在空目录 `~/.local/share/singlefs/work/m3-batch52-54-code-r1-opus/rerun-c/` 里从头跑通一次，compare 的原样输出：
```
一致 base-k1.txt
一致 base-k2-numbers.txt
一致 base-k2-product.txt
一致 base-k2-random.txt
一致 base-k3.txt
一致 base-k4.txt
一致 k5.txt
一致 mut-c-incr-true.txt
一致 mut-k1-fix-judge-entries-before-count.txt
一致 mut-k1-revert-b52.txt
一致 mut-k3-revert-b53.txt
一致 mut-k3-treeid-not-abandoned.txt
一致 mut-k4-borrow-ceil.txt
一致 mut-k4-drop-start-right.txt
一致 mut-k4-floor-116.txt
一致 mut-k4-no-absorb-after-borrow.txt
一致 mut-k4-no-borrow.txt
一致 mut-k4-no-chain.txt
一致 mut-w-eager.txt
一致 mut-w-gap2.txt
一致 mut-w-same.txt
一致 mut-w-start2.txt
compare：22 份，不一致 0 份
```

每个文件的 sha256（`sha256sum`，在 `research/prompts/` 下跑）：
```
62cda2ad41288fe280519d2cf0b9a8315d2cb6fd334cb4de5cfa723ebfa9d366  m3-batch52-54-code-r1-opus-model/compare.py
bb74216a0c96359fdeabbf91f2ba1cb43ef695f7c4542265b4186adfa5dce992  m3-batch52-54-code-r1-opus-model/expected/base-k1.txt
978f4ede766c644143304ec2ac3aa288a05cbd6e4b7c6658745c1535fbcc60d7  m3-batch52-54-code-r1-opus-model/expected/base-k2-numbers.txt
efef3cc65eaf36841a8c8f12e97fd900b6f71ea0165b6122b929487aca828e05  m3-batch52-54-code-r1-opus-model/expected/base-k2-product.txt
239394d2d81b9dedadf632c07aab8167a1f6d98f8ecacf1acaeccec155bcbc44  m3-batch52-54-code-r1-opus-model/expected/base-k2-random.txt
1b27968bbaf09a694be84bcb3a0dc87db09341bbf0f3340a4e30bace41c21039  m3-batch52-54-code-r1-opus-model/expected/base-k3.txt
ab8cbeda0e969648fd2b49c8c25fad76bdaea5d3c0abbae47efab4ed826f95ed  m3-batch52-54-code-r1-opus-model/expected/base-k4.txt
341bab7c6f5e78817630a5adebf7dcde533f1546316c8ef93d595c4c7d82d6aa  m3-batch52-54-code-r1-opus-model/expected/k5.txt
85b3682a1a7f5c531768b85a2c0684c82a7daba3502f6685eca7a849244a0828  m3-batch52-54-code-r1-opus-model/expected/mut-c-incr-true.txt
4ce2003abbe206cd01f7544527522b970dad2d354d583d40fa338697ca57aea7  m3-batch52-54-code-r1-opus-model/expected/mut-k1-fix-judge-entries-before-count.txt
4e3a84a7f94f8f7b54dd91917fa3d05ede7adfb1d9d00ce380ff3e7e8ef4c10f  m3-batch52-54-code-r1-opus-model/expected/mut-k1-revert-b52.txt
a02d8a6a26cfb1e2d243e418b69261aab59506549072917d6fd6bbfb7271dd71  m3-batch52-54-code-r1-opus-model/expected/mut-k3-revert-b53.txt
7f6745c72620e1d4eb4c2e057c6299aead4a7fdfbc8aa130982bf04e935686ff  m3-batch52-54-code-r1-opus-model/expected/mut-k3-treeid-not-abandoned.txt
c948c1b483afe770323de7e76bb845dffcb435f9a39f168c174304c4a4c5f948  m3-batch52-54-code-r1-opus-model/expected/mut-k4-borrow-ceil.txt
9594da71391cb982731dc275b41e85c8af749e6f6697a99a08013274bdbfa7ac  m3-batch52-54-code-r1-opus-model/expected/mut-k4-drop-start-right.txt
c948c1b483afe770323de7e76bb845dffcb435f9a39f168c174304c4a4c5f948  m3-batch52-54-code-r1-opus-model/expected/mut-k4-floor-116.txt
9594da71391cb982731dc275b41e85c8af749e6f6697a99a08013274bdbfa7ac  m3-batch52-54-code-r1-opus-model/expected/mut-k4-no-absorb-after-borrow.txt
8b194f1b09c430a7ee97cd250cfb1abd1609852e0297e7a20c59c570bff21cda  m3-batch52-54-code-r1-opus-model/expected/mut-k4-no-borrow.txt
9594da71391cb982731dc275b41e85c8af749e6f6697a99a08013274bdbfa7ac  m3-batch52-54-code-r1-opus-model/expected/mut-k4-no-chain.txt
b66ef51a9c4d323efe11335c03615c6875d42d302e6ba032b04676b156918062  m3-batch52-54-code-r1-opus-model/expected/mut-w-eager.txt
2cab43b70469a0e665610e6ba4fd4b88f1c27b52951b214f2b54f37eda81f214  m3-batch52-54-code-r1-opus-model/expected/mut-w-gap2.txt
9de3a2f967f6439d13b1da3820b39d6e3a74223ae875203e223e6fa1ddb3ccdf  m3-batch52-54-code-r1-opus-model/expected/mut-w-same.txt
9762bd1a76764f4cd7d927ff5544ceec429c379241cf7acfee09fe0b56993a18  m3-batch52-54-code-r1-opus-model/expected/mut-w-start2.txt
5a3df67f3a7fb5bdff596fbab33e080d3bd77aac4d5625ee780336e9ae49f25f  m3-batch52-54-code-r1-opus-model/k5.py
eac0afadf82e7949c5a5036e40311973cf1efa84aff37370755672a99ee99fbb  m3-batch52-54-code-r1-opus-model/mutants.py
f9eb842bab80b48a43ce05c3073604b7ea70a1b57dee23224ab3f392e8a4fc37  m3-batch52-54-code-r1-opus-model/probes/corrupt_on_disk_k1_append.rs
15da24c59ba43713190be8b5ea4a8e5662cc8129990631ad918e43e7557bc5c5  m3-batch52-54-code-r1-opus-model/probes/opus_probe_k2_k4.rs
c22acad8bd16fdeac8384c3e7c678e8379d977099f3e0800b6f34f0d9a281d3e  m3-batch52-54-code-r1-opus-model/rerun.sh
c361b984c8e2d82cadba418e5afa24a53d2a0fbeadf7ce1e82a49d34a7965357  m3-batch52-54-code-r1-opus-model/summarize_random.py
```

## 各格判定一览

| 格 | 判定 | 依据（详见各节） |
|---|---|---|
| K1-1 别的坏法错误成员变没变 | **打中（不分辨批次，第 52 批前后一样）**：台阶行的代写到根之后（5 > 4），只读恢复与可写挂载都报「记账条目数…不是 3 + 6 × 盘数」，同一坏法落在别的行上报「记账条目的代晚于根或 seq 为 0」 | 量过：`base-k1.txt`、`mut-k1-revert-b52.txt` |
| K1-1 合法镜像被拒 | 没打中 | 量过：干净对照两个读者都收下 |
| K1-2 两个读者次序 | 没打中：六个坏法上两个读者交回的成员逐个相同；可写挂载走重建那一条 | 量过：`base-k1.txt` |
| K2-1 合法镜像误报 | 没打中：旋钮下 12 段随机历史 1800 次判 0 违例；产品容量退役 960 片、回退 1360 次，I-9.36 Holds | 量过：`base-k2-random.txt`、`base-k2-product.txt` |
| K2-2 坏镜像漏报 | **打中**：容器号不连（1, 3）、不从 1 起（2, 3）、最右多一片空容器，三棵容器树带旋钮值交新入口一条不红 | 量过：`mut-w-gap2.txt`、`mut-w-start2.txt`、`mut-w-eager.txt` |
| K2-3 CPU 与 GPU | 推的：GPU 谱系记录表上限 256 条，产品容量下谱系树 I-9.36 有对象（≥ 1360 条）的状态 GPU 一个都判不了、放量拒跑；没有判得不同的状态 | 读码，没跑 GPU |
| K3 抽函数前后、三条单测 | 没打中（行为）：改回内联前后四段随机回退历史的根记录与水位逐步摘要相同；**判别力缺口（推的等价）**：「树 ID 水位照旧取环里全部根」没有会红的用例，改成只取没被抛弃的根，回退那三份 harness 用例 50 条全绿 | 量过：`mut-k3-revert-b53.txt`、`mut-k3-treeid-not-abandoned.txt` |
| K4-1 参照的独立性 | 部分：参照与实现照同一份 E176 `remove` 抄成同一结构，共用 233 与 ⌈233 ÷ 2⌉，读侧读的是发布路径交回的内存版本、不读盘 | 读码 |
| K4-2 参照覆盖 | **打中（覆盖）**：种子 53 那段历史只走到一次右吸收、一次借；删空摘片、左吸收、借后吸收、连锁吸收都是 0 次（另 12 个种子同样）。「借完不从借出那片接着并」的变异在参照用例与 core 的 21 条 inode_tree 单测上照绿 | 量过：`base-k4.txt`、`mut-k4-no-absorb-after-borrow.txt` |
| K5 钉值 | 没打中：dispatch 四个钉值、unlink 两个闭式按原理重算相同；分片两个打的行哈希与两份进度文件按证据重算相同、去掉新不变量那几项后 47 行逐字相同、CRC32C 47/47 | 量过：`k5.txt` |
| K6 判别力 | **打中**：I-9.36 / I-9.37 里「容器号沿叶序严格递增」那一半改成恒真，坏镜像整份 230 条、容器树长高 6 条全绿；墓碑那一半有变异行与坏镜像，这两条没有 | 量过：`mut-c-incr-true.txt`、`mut-w-same.txt` |

表里「量过」指本腿副本上的原样输出（复跑在 `research/prompts/m3-batch52-54-code-r1-opus-model/`，产物名是 `expected/` 下的文件），不是入库装置上的数。

## K1　恢复里两判的次序（第 52 批）

### 造法
原型 `probes/corrupt_on_disk_k1_append.rs`（追加在 `corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 副本末尾，复用它的 `pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row` 与 `rewrite_the_root_node_of_a_tree_of_the_newest_root`）：两块内存稀疏盘，第一个文件（txg 3）后再建一个空 inode（txg 4），记账叶根 16 条里有一级代段 3 的台阶行。改一条条目、按 (统计量, 树, 盘, 代) 重排、头里的最小 / 最大 key 换成首末两条、节点与上面每一道校验和重算，再交给只读恢复（`recover`）与可写挂载（`mount_writable`）。六种：不改；台阶行代 3 → 5；台阶行 seq → 0；这一版自己那一行水位代 4 → 5；池级统计量 4 那一行代 4 → 5；这一版自己那一行水位代 4 → 2。

### 量到的（`expected/base-k1.txt`，副本上的原样输出）
```
OPUS_K1 case=step_row_generation_5_after_root_4 recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数" } }
OPUS_K1 case=step_row_generation_5_after_root_4 writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数" })
OPUS_K1 case=current_watermark_row_generation_5 recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "记账条目的代晚于根或 seq 为 0" } }
OPUS_K1 case=statistic_4_pool_row_generation_5 recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "记账条目的代晚于根或 seq 为 0" } }
OPUS_K1 case=statistic_4_pool_row_generation_5 writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目的代晚于根或 seq 为 0" })
```
第 52 批那两处条目宽之判拿掉（`mutants.py` 的 `k1-revert-b52`，即第 52 批之前的次序）之后同一张坏镜像（`expected/mut-k1-revert-b52.txt`）：
```
command=1 args=--release -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking filter=opus_probe rc=0
OPUS_K1 case=step_row_generation_5_after_root_4 recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数" } }
OPUS_K1 case=step_row_generation_5_after_root_4 writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数" })
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out
command=2 args=--release -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking filter=--skip opus_probe rc=101
test result: FAILED. 31 passed; 2 failed; 0 ignored; 0 measured; 1 filtered out
test a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery ... FAILED
test a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_writable_mount ... FAILED
```

### 判
同一个坏法（记账条目的代晚于根），落在台阶行上两个读者都报成条目数，落在池级那一行或这一版自己那一行上报「代晚于根或 seq 为 0」。病根与第 52 批修的那一处同形：数条目那一判按「统计量 12 且代早于根」认台阶行（「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg」（`crates/singlefs-core/src/recovery.rs:4792`）），代写坏的台阶行不再被认成台阶行、被算进 3 + 6 × 盘数；逐条判代与 seq 的那一步（「accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(」（`crates/singlefs-core/src/recovery.rs:2927`）、「accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;」（`crates/singlefs-core/src/recovery.rs:5411`））排在数条目（「accounting_entry_count_is_three_plus_six_per_device(」（`crates/singlefs-core/src/recovery.rs:2919`））之后。第 52 批只把条目宽挪到前面（「every_accounting_entry_is_as_wide_as_its_field_table(」（`crates/singlefs-core/src/recovery.rs:2916`）、同句在 `crates/singlefs-core/src/recovery.rs:5390`），它自己写的理由是「条目宽坏了时台阶行解不开、被算进 3 + 6 × 盘数，条目宽这一坏法就报成了条目数不对，不是它自己的成员。」（`crates/singlefs-core/src/recovery.rs:4770`）；代那一坏法是同一句话的另一个实例，没罩到。

四句：
1. 分不分辨批次：不分辨。第 52 批前后这一格的成员相同（上面两段输出），它不是第 52 批引入的回归，是第 52 批那条理由没覆盖到的同形格。
2. 读者当时看不看得到判别它的东西：看得到。条目宽 34、解得开，代那 8 字节就在条目里；把逐条判提到数条目之前，两个读者都报出代（下面「改法」）。
3. 满足的是哪一个分句：K1 第 1 问写的是「交回的错误成员变了」——字面上没变（第 52 批前后都是条目数）；满足的是第 52 批自己那条理由「报成了条目数不对，不是它自己的成员」。两个成员是同一个 `RecoveryFailure::InvariantViolated` 变体、`detail` 不同，「成员」按变体算这一格不成立，按判定算才成立，交主 agent 定按哪个读。
4. 跑前改法在这一格上中不中：第 52 批的改法（只挪条目宽）在这一格上照样报条目数（`mut-k1-revert-b52.txt` 与 `base-k1.txt` 这一行相同），对这一格不起作用。

第 ⑥ 种（这一版自己那一行的代改成 2，成了第二级台阶）两个读者也报条目数：这一版少了一条非台阶行，条目数确实不对，判它报哪个成员都说得通，不算打中。

### 改法（只在我的模型上量过、被攻过零轮）
两个读者里，把条目宽那一判换成「逐条解开并判代与 seq」（`mutants.py` 的 `k1-fix-judge-entries-before-count`），数条目排在它之后。

| 改法 | 台阶行代晚于根 | 窄成 key 宽的记账根（第 52 批两条用例） | 别的 |
|---|---|---|---|
| 第 52 批（只挪条目宽） | 报条目数（量过，`base-k1.txt`） | 报条目宽（没在本腿 base 上单跑；拿掉它这两条红，量过，`mut-k1-revert-b52.txt`） | — |
| 攻方：逐条判提前 | 报代晚于根（量过，下面输出） | 报条目宽（量过：除攻方原型之外那 33 条全过，含第 52 批两条） | `writable_mount_of_a_formatted_pool` 15 条全过（量过）；checker 档坏盘战役没跑（推的：那一类坏法同样先红在解条目那一步） |

`expected/mut-k1-fix-judge-entries-before-count.txt`：
```
command=1 args=--release -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking filter=opus_probe rc=0
OPUS_K1 case=step_row_generation_5_after_root_4 recover=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: InvariantViolated { invariant: "E142 走读同款", detail: "记账条目的代晚于根或 seq 为 0" } }
OPUS_K1 case=step_row_generation_5_after_root_4 writable_mount=Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目的代晚于根或 seq 为 0" })
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out
command=2 args=--release -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking filter=--skip opus_probe rc=0
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out
command=3 args=--release -p singlefs-harness --test writable_mount_of_a_formatted_pool filter= rc=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## K2　I-9.36、I-9.37（第 53 批）

### K2-1 合法镜像误报：没打中
原型 `probes/opus_probe_k2_k4.rs`：内存稀疏盘，mkfs 与第一个文件只做一次，之后每段历史从它 `clone`。

随机历史 `opus_probe_k2_random_knob_histories`：每个种子随机取一组旋钮（墓碑 1–3、谱系 1–2、退役 1–2、码 2 节点 2 / 3 / 产品），写入口与分配器开关装同一组，先建 234–241 个空 inode，再走 150 步：25% 建 1–3 个，25% 从记录最少的那片叶里删一个，25% 删最大的活号（最右那片删空就退役），10% 随便删一个，15% 挂着回退到最近 12 版里随便一版。用户动作（建几个、删哪个、退到哪一版）全放开，只固定第一个文件那一前缀。每一步之后带同一组旋钮值交 `check_pool_image_given_the_container_capacity_knobs`。`python3 summarize_random.py expected/base-k2-random.txt` 的原样输出（列：种子 | 旋钮 | 回退成功 | 墓碑树末版 | 谱系树末版 | 退役树末版 | 三棵树根的层级 墓碑/谱系/退役 | I-9.37 成立的判次 | 有违例的判次）：
```
合计：判 1800 次，有违例的 0 次
| 1 | tombstone: 3, lineage: 2, retirement: 1, node_entries: 3 | 20 | 34 片 100 条 | 10 片 20 条 | 4 片 4 条 | 4/3/2 | 148 | 0 |
| 2 | tombstone: 2, lineage: 1, retirement: 2, node_entries: 0 | 17 | 48 片 96 条 | 17 片 17 条 | 2 片 4 条 | 1/1/1 | 146 | 0 |
| 3 | tombstone: 1, lineage: 2, retirement: 2, node_entries: 3 | 27 | 87 片 87 条 | 14 片 27 条 | 0 片 0 条 | 5/3/0 | 140 | 0 |
| 4 | tombstone: 2, lineage: 1, retirement: 2, node_entries: 0 | 25 | 43 片 85 条 | 25 片 25 条 | 1 片 2 条 | 1/1/1 | 148 | 0 |
| 5 | tombstone: 3, lineage: 1, retirement: 2, node_entries: 3 | 22 | 31 片 92 条 | 22 片 22 条 | 2 片 3 条 | 4/3/1 | 148 | 0 |
| 6 | tombstone: 3, lineage: 2, retirement: 1, node_entries: 0 | 28 | 28 片 82 条 | 14 片 28 条 | 0 片 0 条 | 1/1/0 | 129 | 0 |
| 7 | tombstone: 1, lineage: 1, retirement: 1, node_entries: 0 | 20 | 98 片 98 条 | 20 片 20 条 | 2 片 2 条 | 1/1/1 | 144 | 0 |
| 8 | tombstone: 2, lineage: 2, retirement: 2, node_entries: 2 | 21 | 46 片 92 条 | 11 片 21 条 | 2 片 3 条 | 6/4/1 | 143 | 0 |
| 9 | tombstone: 2, lineage: 1, retirement: 1, node_entries: 0 | 27 | 45 片 90 条 | 27 片 27 条 | 7 片 7 条 | 1/1/1 | 139 | 0 |
| 10 | tombstone: 2, lineage: 1, retirement: 2, node_entries: 2 | 21 | 43 片 85 条 | 21 片 21 条 | 0 片 0 条 | 6/5/0 | 147 | 0 |
| 11 | tombstone: 1, lineage: 2, retirement: 2, node_entries: 3 | 23 | 87 片 87 条 | 12 片 23 条 | 4 片 8 条 | 5/3/2 | 148 | 0 |
| 12 | tombstone: 1, lineage: 2, retirement: 1, node_entries: 0 | 22 | 84 片 84 条 | 11 片 22 条 | 1 片 1 条 | 1/1/1 | 140 | 0 |
```
覆盖到：根长高（墓碑到层级 6、谱系到 5、退役到 2）、回退照抄现行那一版的谱系与退役树（每段 17–28 次回退）、谱系与退役混在一棵池里、三棵树旋钮各不相同。没造到：崩溃恢复抛弃段里写的容器（见「这条腿自己的限度」）。

产品容量的边 `opus_probe_k2_product_boundary_retirement` / `_lineage`（`expected/base-k2-product.txt`）：
```
OPUS_K2_PRODUCT_RETIREMENT retirements=960 retirement_counts=[959, 1] I-9.36=Holds violations=[]
OPUS_K2_PRODUCT_LINEAGE rollbacks=1360 lineage_counts=[1359, 1] I-9.36=Holds violations=[]
```
产品容量下第一片退役容器装满 959 条、第一片谱系容器装满 1359 条才开第二片，产品入口判 I-9.36 成立。

### K2-2 坏镜像漏报：打中
写侧改坏容器号，画出镜像交池级 checker（`opus_probe_k2_container_numbers_under_the_knobs`：三棵树旋钮都是一片 1 条，删两个号、回退两次，三棵树各两片）。不改时（`expected/base-k2-numbers.txt`）：
```
OPUS_K2_NUMBERS trees tombstone=[(1, 1), (2, 1)] lineage=[(1, 1), (2, 1)] retirement=[(1, 1), (2, 1)] levels(t,l,r)=(1,1,1) leaves=[1]
OPUS_K2_NUMBERS knob_entry_violations=[]
OPUS_K2_NUMBERS knob_entry I-9.37=Holds I-9.22=Holds
OPUS_K2_NUMBERS product_entry_violated=["I-9.21", "I-9.36"]
```
`expected/mut-w-gap2.txt`：
```
OPUS_K2_NUMBERS trees tombstone=[(1, 1), (3, 1)] lineage=[(1, 1), (3, 1)] retirement=[(1, 1), (3, 1)] levels(t,l,r)=(1,1,1) leaves=[1]
OPUS_K2_NUMBERS knob_entry_violations=[]
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
command=2 args=--release -p singlefs-core --lib filter=_tree:: rc=101
test result: FAILED. 51 passed; 4 failed; 0 ignored; 0 measured; 189 filtered out
```
`expected/mut-w-start2.txt`：
```
OPUS_K2_NUMBERS trees tombstone=[(2, 1), (3, 1)] lineage=[(2, 1), (3, 1)] retirement=[(2, 1), (3, 1)] levels(t,l,r)=(1,1,1) leaves=[1]
OPUS_K2_NUMBERS knob_entry_violations=[]
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
command=2 args=--release -p singlefs-core --lib filter=_tree:: rc=101
test result: FAILED. 50 passed; 5 failed; 0 ignored; 0 measured; 189 filtered out
```
`expected/mut-w-eager.txt`：
```
OPUS_K2_NUMBERS trees tombstone=[(1, 1), (2, 1), (3, 0)] lineage=[(1, 1), (2, 1), (3, 0)] retirement=[(1, 1), (2, 1), (3, 0)] levels(t,l,r)=(1,1,1) leaves=[1]
OPUS_K2_NUMBERS knob_entry_violations=[]
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
command=2 args=--release -p singlefs-core --lib filter=_tree:: rc=101
test result: FAILED. 48 passed; 7 failed; 0 ignored; 0 measured; 189 filtered out
```
`expected/mut-w-same.txt`：
```
OPUS_K2_NUMBERS trees tombstone=[(1, 1), (1, 1)] lineage=[(1, 1), (1, 1)] retirement=[(1, 1), (1, 1)] levels(t,l,r)=(1,1,1) leaves=[1]
OPUS_K2_NUMBERS knob_entry_violations=["I-1.1: 树 65545（种类 10）的根：key 宽 / key 区间与条目不符（Err(KeysNotStrictlyAscending)）", "I-9.22: 根（实例 1、txg 9）：墓碑树（树 65545）沿叶序容器号不严格递增：容器 1 之后是容器 1", "I-9.37: 根（实例 1、txg 9）：退役树（树 65548）沿叶序容器号不严格递增：容器 1 之后是容器 1"]
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
command=2 args=--release -p singlefs-core --lib filter=_tree:: rc=101
test result: FAILED. 51 passed; 4 failed; 0 ignored; 0 measured; 189 filtered out
```

（每份的 `command=2` 是同一副本上 `cargo test -p singlefs-core --lib _tree::`；四种写侧改坏都被 core 的容器树单测拦下，`mut-w-*.txt` 里有红的用例名。）

判：容器号不连（1, 3）、不从 1 起（2, 3）、最右多出一片 0 条的容器，三棵容器树带旋钮值交新入口一条都不红；改成两片同号（1, 1）时 I-9.37 与 I-9.22 红（另带 I-1.1），说明「严格递增」那一半是活的。条款要的是「容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1」（`.claude/kb/decisions/18-块里携带什么信息.md:576`），谱系树同一句「谱系树根满同样长高，容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1」（`.claude/kb/decisions/18-块里携带什么信息.md:580`）；判法只判「judgements.judge(invariant, left_container < right_container, || {」（`crates/singlefs-checker/src/walk.rs:11994`），产品那一支只判非最右「judgements.judge(invariant, record_count == product_records, || {」（`crates/singlefs-checker/src/walk.rs:12012`）、最右那片「if is_the_rightmost {」（`crates/singlefs-checker/src/walk.rs:12008`）直接跳过，旋钮那一支只判「judgements.judge(invariant, record_count <= records_the_knob_allows, || {」（`crates/singlefs-checker/src/walk.rs:12019`）。所以起点、连号、最右那片至少一条都没有判法。产品容量下这三种照样漏：(1, 3) 与 (2, 3) 的非最右那片装满 959 / 1359 条就过 I-9.36，[959, 0] 的空片在最右被跳过（推的，按上面那几行）。

四句：
1. 分不分辨批次：不分辨。墓碑树的 I-9.21 / I-9.22 同一形（`.claude/kb/invariants.md:309` 写的就是「墓碑树里容器号沿叶序严格递增」），墓碑那一半在上面四份输出里同样不红；第 53 批照它的形抄过来，连这三个洞一起抄了。
2. 判别它的东西 checker 当时看得到吗：看得到。容器号与记录数就是它已经读进 (容器号, 记录数) 那张表的两列。
3. 满足的是哪一个分句：K2 第 2 问「坏镜像漏不漏：……容器号不连……」，字面打中「容器号不连」；「最右多一片空容器」是那三种之外的第四种（「提早开」）。按 I-9.36 / I-9.37 自己的文字（「容器号沿叶序严格递增」）这三张镜像都合法，打中的是 D18 已定项 24 第 1、5 条与判法之间的差，不是判法写错了自己。
4. 改法在这几格上中不中：第 53 批判决 F2 给的验收是「攻方那一形红，只红新判法；合法镜像不红」，没有要求连号；照 F2 的改法这几格仍不中。

改法（被攻过零轮，只推了没实现）：

| 改法 | (1, 3) / (2, 3) | 最右空容器 | 墓碑树同形 |
|---|---|---|---|
| I-9.36 / I-9.37 的「严格递增」改成「第一片是 1、相邻 +1」 | 推的：红 | 推的：不红 | 推的：I-9.21 / I-9.22 照改才红 |
| 每片至少 1 条（不论是不是最右） | 推的：不红 | 推的：红 | 推的：同上 |

写侧这四种改坏都有 core 单测拦（上面 `command=2`），漏的只是池级 checker 这一道独立的判。

### K2-3 CPU 与 GPU：推的，没跑 GPU
扫描内核读谱系树时先判容量「d_judge_lineage_or_retirement_container_capacity(DELETION_LINEAGE_CONTAINER_RECORDS);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3031`），接着把谱系记录装进一张表，表满即判不了「if (d_lineage_record_count >= D_MAX_LINEAGE_RECORDS) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3035`），上限「const D_MAX_LINEAGE_RECORDS: u32 = 256u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:853`）。产品容量下谱系树要 ≥ 1360 条记录才有第二片容器（上面 `base-k2-product.txt` 那一行），所以 I-9.36 在谱系树上有对象的每个状态 GPU 都判不了；放量遇到判不了的状态整块拒跑，「配置开了 GPU，不退回 CPU 判器」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2321`）。这不是判得不同（判不了的状态不出结论），是第 53 批「CPU 与 GPU 同判」在谱系树那一半只在旋钮压小（≤ 256 条）的镜像上成立；退役树那一半产品容量 960 条以内（容器表 256 片、两两比 4096 条）都判得了。上限是旧的（第 53 批没碰），判别力没有变差。改法：把 `D_MAX_LINEAGE_RECORDS` 抬到 ≥ 1360，或者容量那一判不依赖谱系记录表（它只要容器号与记录数）——都只是推的。

树表 12 条：「const C_MAX_TREES_PER_TABLE: u32 = 12u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:432`），第 50 批加退役树之后树表正好 12 条，第 53 批没有加树，没有更近一步；没造多于 12 条的树表。旋钮怎么传给 GPU：不传，GPU 恒按产品判 I-9.36、I-9.37 恒不置位，与 CPU 旧入口对拍，第 53 批两条 GPU 用例走的就是旧入口；没有判得不同的格。

## K3　Opt2 环那一项（第 53 批）

### 行为是否逐字节相同：没打中
读码：抽出的函数与原内联写法同一条表达式（过滤、交给盘上重建、与 cur 那一项取 max）；差别只在 `current.inode_number_watermark()` 早一步求值，它是纯读，没有副作用。量：把第 53 批那一段改回内联（`mutants.py` 的 `k3-revert-b53`），在同一组四段随机历史（产品容量，建 / 删 / 挂着回退到最近 12 版，每段 80 步）上逐步把根记录、inode 号水位与谱系记录记进摘要，改前改后摘要相同。`expected/base-k3.txt`：
```
OPUS_K3_DIGEST seed=1 ops=80 rollbacks_ok=11 rollbacks_refused=0 final_txg=134 final_watermark=276 digest=e1c4753ea9a4d00d7da013ce3dbbd4d50a4d1375a74b9653af5b16591f887e1e
OPUS_K3_DIGEST seed=2 ops=80 rollbacks_ok=13 rollbacks_refused=0 final_txg=131 final_watermark=281 digest=8288edc0c09285e85be54bd7a0d82f8145fbbc173a584b1badf9f2876c366e7d
OPUS_K3_DIGEST seed=3 ops=80 rollbacks_ok=10 rollbacks_refused=0 final_txg=131 final_watermark=285 digest=9ca6c87e28fa567bb6714c34df900938c4003692f6b5067782994de7f97c7715
OPUS_K3_DIGEST seed=4 ops=80 rollbacks_ok=18 rollbacks_refused=0 final_txg=124 final_watermark=281 digest=3a762c82f9de60001aad00843b9f5e1d57b8e2f8106dcd9496e5a2dbc3f5dec0
```
`expected/mut-k3-revert-b53.txt`：
```
OPUS_K3_DIGEST seed=1 ops=80 rollbacks_ok=11 rollbacks_refused=0 final_txg=134 final_watermark=276 digest=e1c4753ea9a4d00d7da013ce3dbbd4d50a4d1375a74b9653af5b16591f887e1e
OPUS_K3_DIGEST seed=2 ops=80 rollbacks_ok=13 rollbacks_refused=0 final_txg=131 final_watermark=281 digest=8288edc0c09285e85be54bd7a0d82f8145fbbc173a584b1badf9f2876c366e7d
OPUS_K3_DIGEST seed=3 ops=80 rollbacks_ok=10 rollbacks_refused=0 final_txg=131 final_watermark=285 digest=9ca6c87e28fa567bb6714c34df900938c4003692f6b5067782994de7f97c7715
OPUS_K3_DIGEST seed=4 ops=80 rollbacks_ok=18 rollbacks_refused=0 final_txg=124 final_watermark=281 digest=3a762c82f9de60001aad00843b9f5e1d57b8e2f8106dcd9496e5a2dbc3f5dec0
```
射程：这几段历史里没有崩溃恢复抛弃根，环那一项在这些状态上都不大于 cur（上一轮判决 K1 那一句），所以只核了「cur 那一项胜出」那条路上相同；环那一项胜出的路只有三条构造态单测走到，改前改后那条路的等价是读码。

### 三条单测钉的是不是条款那件事
条款那一格：「树 ID 水位照旧 = max(cur 那一版内存里的水位, 环里读得出的全部根的水位)」（`.claude/kb/decisions/23-journal的角色与格式.md:385`）。三条单测钉「没被抛弃的根算」「被抛弃的不算」「与 cur 取 max」，对得上 inode 号水位那半句；树 ID 水位那半句不在抽出的函数里（它在「let tree_identifier_watermark = ring_roots」（`crates/singlefs-core/src/mount.rs:8131`）那几行，「.fold(current.root.tree_identifier_watermark, u64::max);」（`crates/singlefs-core/src/mount.rs:8134`）），三条单测碰不到它。

判别力（量过）：把树 ID 水位也改成只取没被抛弃的根（`k3-treeid-not-abandoned`），回退那三份 harness 用例全绿（`expected/mut-k3-treeid-not-abandoned.txt`）：
```
command=1 args=--release -p singlefs-harness --test rollback_by_a_forward_publish filter=--include-ignored rc=0
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
command=2 args=--release -p singlefs-harness --test tree_identifier_watermark_after_the_first_file_root_is_abandoned filter= rc=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
command=3 args=--release -p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over filter= rc=0
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
推的：这一改在走得到的状态上多半是等价变异——树 ID 只在第一个文件那次发布从水位发号，被抛弃的根要比 cur 那条线的水位高，得是被抛弃的那次发布自己发过号，而那一形（被抛弃的第一个文件版本）之后 cur 那条线重新发号时已经取过环里的 max（`tree_identifier_watermark_after_the_first_file_root_is_abandoned` 这一份照绿就是它）。checker 档 `crash_points_tree_identifier_watermark_orphans.rs` 没跑。四句：不分辨批次（第 53 批没碰这几行）；看得到（根记录里的水位）；满足的是 K3「树 ID 水位照旧取全部」那一问的「单测钉没钉」；改法若要钉，得造一个被抛弃的根比 cur 线高的树 ID 水位（推的，可能造不出）。

## K4　M3 的独立参照（第 53 批）

### K4-1 独立到哪一步（读码）
- 不调 `inode_tree.rs` 的合并、借、摘片：对。参照自己的 `merge_from`、`borrow_from_the_right_if_below_the_floor`、`remove` 都在用例文件里。
- 共用的前提：① 算法是照 E176 `remove` 抄的，与 `inode_tree.rs` 抄自同一份（两边都是「先右后左、并完从幸存那片接着查、一次都没并才借、借完从借出那片起并」），E176 那一臂若与 D8 已定项 6 不符，两边一起错；② 容量 `INODE_LEAF_RECORDS` 与 ⌈233 ÷ 2⌉ 两边同一个来源；③ 读侧读的是发布路径交回的内存版本（「.inode_leaf_containers」（`crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs:225`）），不读盘：只把内存版本写坏、盘上不坏（或反过来）的错，这条用例看不见（推的，没造）；④ 不比分隔 key，借那一次右片分隔 key 改没改（N1）不在它的比较里。

### K4-2 覆盖：打中
把参照照抄一份加形状计数，在内存盘上按同一个种子、同一组常量、同一个随机数取用次序把那段历史走一遍（`opus_probe_k4_shapes_reached_by_the_reference_history`，每一步照样与发布路径逐片比），另换 12 个种子。`expected/base-k4.txt`：
```
OPUS_K4 seed=53 ShapeCounts { removes: 246, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=1 ShapeCounts { removes: 249, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=2 ShapeCounts { removes: 255, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=3 ShapeCounts { removes: 243, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=4 ShapeCounts { removes: 249, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=5 ShapeCounts { removes: 249, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=6 ShapeCounts { removes: 254, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=7 ShapeCounts { removes: 239, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=8 ShapeCounts { removes: 250, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=9 ShapeCounts { removes: 249, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=10 ShapeCounts { removes: 249, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=11 ShapeCounts { removes: 239, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
OPUS_K4 seed=12 ShapeCounts { removes: 248, drops: 0, drop_then_absorb: 0, right_absorbs: 1, left_absorbs: 0, removes_with_two_or_more_absorbs: 0, both_pairs_fit: 1, both_pairs_fit_outside_the_first_step: 0, borrows: 1, borrow_then_absorb: 0, borrow_skipped_rightmost: 0, leaves_left_at_116: 0 }
```
13 个种子的计数逐项相同：每段 239–255 次删，只走到一次「两对都装得下」时的右吸收与一次借；删空摘片、左吸收、借后吸收、连锁吸收全是 0。原因在历史的形状：删的一段永远挑「可删条数最多的那片」（「let position = reference.position_of_the_leaf_with_the_most_removable_records();」（`crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs:360`）），三片被均匀削到各 117 条，第一次「两对都装得下」就停（「while reference.times_both_neighbour_pairs_fit == 0 {」（`crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs:350`）），之后的 6 次随机删落在三片都 ≥ 116 的树上。种子只换得动建的批次大小，换不动形状。

判别力（量过，`mutants.py` 的 k4 六种；每种各一份副本、各用自己的编译目录，跑 core 的 `inode_tree::` 21 条与第 53 批参照用例）：

| 变异 | core `inode_tree::` | 第 53 批参照用例 | 走得到吗 |
|---|---|---|---|
| 借完不从借出那片接着并（`k4-no-absorb-after-borrow`） | 21 过 | 过 | 走得到：变异表已有同一处的行，「第 50 批 M3：借完不从借出的那一片接着看合并」（`crates/mutations.tsv:3678`），由另一份 harness 用例拦（本腿没跑它） |
| 借的下限 117 → 116（`k4-floor-116`） | 1 红 | 红 | — |
| 借的条数向上取整（`k4-borrow-ceil`） | 1 红 | 红 | — |
| 不借（`k4-no-borrow`） | 2 红 | 红 | — |
| 删空摘片之后从右邻起并（`k4-drop-start-right`） | 21 过 | 过 | 推的：等价。改前相邻两片之和都 ≥ 234，右邻与它右边那一对没变、仍装不下，第一步照样落到新相邻的那一对 |
| 并一次就停（`k4-no-chain`） | 21 过 | 过 | 推的：等价。并完的幸存片与两边邻居之和都 ≥ 234（改前那两对已 ≥ 234），连锁并走不到；与上一轮「删除类发布至多一片叶身份消失」同一个前提 |

四句（针对第一行）：不分辨批次（参照是第 53 批新加的，借后吸收由第 50 批的用例罩着，这一行不是回归）；看得到（参照自己就有借后接着并那一步，只是历史走不到）；满足的是 K4 第 2 问「覆盖到借后吸收、连锁吸收、先右与先左结局不同的那几形了吗」——先右与先左结局不同那一形覆盖到了（1 次），借后吸收没有，连锁吸收走不到；F1 的验收（「M4（先左）在新用例上红」）这一条本身满足，打中的是「随机历史逐次比」这个说法的射程。

改法（被攻过零轮，推的）：历史里加两段——删到某片叶空（删最大的活号，最右那片删空摘片）与「删左片到借、再删右片」那一形——参照里那几条分支才有输入。

## K5　钉值（第 52、54 批）：没打中

`python3 k5.py <仓根> <第 52 批证据目录> <第 54 批证据根>`（证据是第 52、54 批实现员草稿目录里留的进度文件与打的行，`rerun.sh k5` 阶段；`expected/k5.txt`）：
```
K5_DISPATCH scratch_words=320688 per_state_bytes=1282752 states_per_lane=1674 states_in_12_gib=10044 mebibytes_for_45=57
K5_UNLINK_CLOSED_FORM two_states_per_write=1463812228 domain=1463812278
K5_PROGRESS_B52 old=previous-golden-progress-1d92a7ad.txt sha256=1d92a7ad8c48c01f new=layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-555baf1b8e6301e8874a473d44d0ce43260f946b695dfc0fbb3b0c5024bbbba6.txt sha256=1553ba8eee2fd282 lines=47/47 identical_after_dropping=47 new_crc32c_ok=47
K5_PROGRESS_B54 old=progress-b50-1553ba8e.txt sha256=1553ba8eee2fd282 new=progress-b53-92ff4595.txt sha256=92ff4595a87332b8 lines=47/47 identical_after_dropping=47 new_crc32c_ok=47
K5_PRINTED_B54 lines=58 sha256=a4643c92e6836ef465aba245ee096127bebc0b65f7342a090fd1570d9c48bea6 without_two_sha256=4d42c7ca5ca9e8e16e5c7bdde36198f576d40a98ad9a8dfb35fb33947161e354 removing_the_two_items_gives_the_without_file=True
```
- dispatch 草稿区四个钉值（1674、10 044、57 MiB、`1674 * MAXIMUM_SCRATCH_LANES`）全由 `SCRATCH_WORDS` = 320 688 算出，与补丁里的数相同；`SCRATCH_WORDS` 本身是第 50 批的表变大带来的，第 52、53 批没动草稿区布局（第 53 批的 I-9.36 复用容器表）。
- unlink 全量两个闭式：只把最后一批的单元段从 2^20 − 1 换成 2^24 − 1，算出来正是补丁里的 1 463 812 228 与 1 463 812 278。12 个单元的来历（退役容器与退役树根各一个）是第 50 批的写侧行为，上一轮已判过；段数的断言（`prepare` 里那张 35 段表）在快档里跑，钉值与行为对得上（主 agent 合后快档那一份照绿）。
- 分片进度文件两次换值（1d92a7ad → 1553ba8e → 92ff4595）：47 行去掉新不变量那几项与计划哈希之后逐字相同，新文件每行 CRC32C 都对得上；打的行 a4643c92 去掉两项 ` I-9.36=0/0/46 I-9.37=0/0/46` 之后正是 4d42c7ca。没有藏着的行为变化。
- AccountingLeafSplit「L5 L5 L6」、AccountingRootSplit「L8 L8」：读码。两处都比旧钉值多 1 条，正是第一个文件那一版那一行水位在空发布时成了台阶行、照抄不删（W3 的台阶行语义，上一轮已判），按补丁注释那一路插入次序推得出 5 + 5 + 6 与 8 + 8；checker 档那条流没跑。
- 第 54 批的豁免表六行逐条对得上池级 checker 在没有删除的会话上报的理由常量（见第 54 批补丁），I-9.35 不进豁免、要求成立；变异六行各删一行豁免。没造反例。

## K6　判别力

打中一处（量过）：I-9.36 / I-9.37 的「容器号沿叶序严格递增」那一半改成恒真（`c-incr-true`：「judgements.judge(invariant, left_container < right_container, || {」（`crates/singlefs-checker/src/walk.rs:11994`）换成 `true`），`expected/mut-c-incr-true.txt`：
```
command=1 args=--release -p singlefs-harness --test checker_known_bad_images filter= rc=0
test result: ok. 230 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
command=2 args=--release -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full filter= rc=0
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
同一张「两片同号」的镜像在不改的 checker 上红 I-9.37（`mut-w-same.txt` 那一行），所以这一半是活的、只是没有会红的用例。墓碑树那一半有：「第24批 K M5：墓碑容器号沿叶序严格递增那一句恒真」（`crates/mutations.tsv:2884`）与它的坏镜像。第 53 批变异表里 I-9.36 / I-9.37 那五行只改「装满」与「不超旋钮」。GPU 扫描内核那一半（`lt64(d_container_number(…))` 那一判）同样没有用例，推的（GPU 没跑）。四句：不分辨批次（新加的判法，墓碑那一形是旧的且罩着）；看得到；满足 K6「还有哪几种改坏的写法照绿」；改法：照墓碑那一条补一张「两片同号」的坏镜像（谱系、退役各一张），加两行变异——被攻过零轮。

另照绿的（K2-2 那三种是写侧改坏，被 core 单测拦，算 checker 的漏报不算判别力缺口；K3 的树 ID 水位、K4 的借后吸收见各节）。第 52 批两行变异（两个读者把条目宽排回数条目之后）在 `mut-k1-revert-b52.txt` 里那两条用例都红，罩得住。

## 没打中的形状

| 格 | 试过的形状 | 取样 |
|---|---|---|
| K1 合法镜像被拒 | 干净的 txg 4 镜像（带一级台阶行）交只读恢复与可写挂载 | 1 张；读码：新加的判只拒窄于 34 的条目，合法条目恒 34 |
| K1 两个读者次序 | 六种坏法 × 两个读者 | 12 次，成员两两相同 |
| K1 别的坏法 | 台阶行 seq = 0、这一版那一行代晚于根、池级行代晚于根 | 各 1 张，成员都是「代晚于根或 seq 为 0」 |
| K2-1 误报 | 旋钮下随机历史（建 / 删 / 回退，根长高、回退照抄、三棵树旋钮各异）；产品容量两条边 | 12 个种子 × 150 步 = 1800 次判；退役 960 片、回退 1360 次各 1 段 |
| K2-2 中间一片多出一条 | 读码：产品容量下 960 × 34、1360 × 24 都大于 32 768 − 136，I-1.7 先拦；旋钮下 I-9.37 逐片判 ≤ 旋钮值 | 没造镜像 |
| K3 行为 | 改回内联前后的逐步摘要 | 4 个种子 × 80 步，52 次回退 |
| K4 变异 | 下限 116、借的条数上取整、不借 | 三种都被 core 单测与参照用例拦下 |
| K5 | 八个钉值从原理或证据重算 | 全对上 |

## 这条腿自己的限度

- 没造崩溃状态：没有调层 0 的枚举（0 个崩溃状态，全量合计 0，离 10⁶ / 10⁷ 的上限都远）。理由：这一轮的几格是镜像判法、恢复里的次序与发布路径的形状，攻法是改坏镜像、改坏代码与放开用户动作的历史；I-9.36 / I-9.37 每一版只读那一版自己的树（`judge_lineage_or_retirement_container_capacity_of_a_version` 的参数只有这一版的 (容器号, 记录数)），崩溃恢复抛弃段里写的容器读码看不出能让哪一版自相矛盾——这一句是推的，K2-1 的「崩溃恢复抛弃段里写的容器」那一形没造。
- 改坏代码只在副本上量；副本上的数不算入库装置上的数。K2-2、K6、K4 的「改法」都被攻过零轮。
- GPU 那一半全是读码（K2-3、K6 的 GPU 半句）。checker 档一条都没跑。
- 内存稀疏盘上的池（`opus_probe_k2_k4.rs` 自己建），不是 harness `build_pool` 那份文件盘；跑第 53 批参照用例与几份回退用例时它们自己按设计建文件盘（`TMPDIR` 指到副本自己的目录）。
- 随机历史每批 6 个进程并行（各 2 个种子），先跑 2 个种子 × 20 步估时（`release` 下约 0.17 秒一步）再定每段 150 步；不设限时。
- 复跑时可写挂载、回退那几份用例经内存包装排过队（别的会话的 checker 档占着 36.5 GiB），排队不改结果。

## 没做什么

- 不判正推那几格，不替主 agent 采纳；没读禁读清单里的文件。
- 没跑 checker 档、GPU、门禁阶段；没跑第 50 批那份拦「借后吸收」的 harness 用例（`crates/mutations.tsv:3678` 点名的那一份），那一格写成「由它拦」是照变异表推的。
- AccountingLeafSplit / AccountingRootSplit 的形状、unlink 全量 12 个单元只读码，没在内存盘上重造那两条流。
