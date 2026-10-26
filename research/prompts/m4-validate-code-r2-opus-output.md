# m4-validate-code-r2 攻方腿（Opus）报告（2026-10-24）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 -->

立场：假设这一批（标签 0 写者照写、隔离的盘上记号落点甲、测试临时目录守卫）是错的，在冻结副本的拷贝上造历史与镜像去打穿它。副本上量的数都是副本上的数，不进 kb。

## 复跑与文件

复跑（仓根跑；`--work` 给一个新的空目录；脚本自己经 `capped.sh 31` 与 `run-with-memory-cap.sh 8G` 起三组 cargo，三组各编一次，前台跑不完，交主 agent 用 run_in_background 起）：

```bash
bash research/prompts/m4-validate-code-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/m4-validate-code-r2/repo \
  --work /home/fy5090/.local/share/singlefs/work/m4-validate-code-r2-verify/opus-rerun
```

跑完看三行 `name=main same_as_model_out=…`、`name=old-readers …`、`name=fix-candidate …`，三行都该是 `same_as_model_out=yes exit=0`。三组：`main` 是冻结副本拷贝上的全部用例；`old-readers` 是 HEAD（841b3beab，隔离记号那一批之前）那一版的读者读 `main` 落下的两张镜像；`fix-candidate` 是同一份用例跑在打了改法甲（`tools/apply_fix_candidate_a.py`）的拷贝上。

模型目录 `research/prompts/m4-validate-code-r2-opus-model/` 的 SHA256SUMS（`find . -type f ! -name SHA256SUMS | sort | xargs sha256sum`）：

```text
412d971edd7600c5325c4b0a1210890ef3792026862050d02112abf1bac6cda2  ./out/fix-candidate.txt
c838822be7b1ea7dbbe4826a78267500bce700b926e7688e7181ada356c10aef  ./out/main.txt
fc019dd5b130224f91f161d2f2f4a33e52ca65873dafd347c572cfe65a07be06  ./out/old-readers.txt
c0d16ad222fc661b6a4474451abb125385ed2457f7be740e700ac5cfbb0b8993  ./prototype/old_readers_on_quarantine_marks.rs
cd52a87561e1627f9538afd8a83b928e95ff211365b3e40e9500f3eeef460844  ./prototype/validate_code_round_two_attack_cases.rs
5efd12b9ab451eacf3312d67d24060368be96c9cb24cb091ed80ff696ae34dd1  ./tools/apply_fix_candidate_a.py
7f51f630037021a63cf291109f81fdde6fef95a71eb12f81bf73a77f8e460cc6  ./tools/rerun.sh
```

`rerun.sh` 拷自 `scripts/agents/three-way-rerun-template.sh`，只改了 `rerun_steps` 那一段（加了「自证用的假冻结副本没有 Cargo.toml 就只核根」那一句，模板自证照旧走得通）。`out/*.txt` 是三组各自的 `grep -oE 'ATTACK-.*' | sort` 加一行 `exit=`；我这一次是按 rerun.sh 里同一串命令逐组在草稿目录里跑出来的（三组合起来编译要三分多钟，超过子 agent 前台的时限），没有整条跑过 rerun.sh，见「要主 agent 跑的命令」。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 | **打中** | 标签 0（size 3000）的第一个文件截到 0，之后任何一次重写第一片 extent 上段叶的发布（按号写 inode 2、顺序写 inode 2、写到 inode 143 让上段长高、写过 inode 2 再 unlink 它）写出的那一版里第一个文件 size 0、上段叶没有它的条目：冷走读、可写重挂、冷恢复三处都拒（「E142 走读同款」），重启之后池可写挂不上；接着变长到 7000，挂载态读者也拒。扫了 12 种后续，5 种中。改法甲（第一个文件没有单元时一律写标签 0）在我的副本上把 5 种全消掉，这一批自带的 4 个相关测试目标照绿 |
| T2 | 没打中 | 一块盘读坏、两块盘各坏一个单元、B 换下的 12 个单元全在盘 1 改坏：两块盘的记录都置 0x4000、分配代不变，读回、重挂、40 次覆盖写之后都在 |
| T3 | 没打中 | 重挂之后回退、回退到不引用它的版本再回退到引用它的、两单元、十个单元：引用到的清、没引用到的留，checker 108 条全不红；核心层今天没有别的路径走到「释放隔离记录」（40 次覆盖写里整理没起、抬 F 0 次，这一条只量了这么多） |
| T4 | 没打中 | 0xC000、0xC002（两盘、一盘）在分配记录树读者拒；0x4000（跨度 0）、0x4003 拒成越界或重叠；核心层没有读跨度段而不去掉 0x4000 的地方（核心层只有 `AllocationRecord::parse` 一个解析点） |
| T5 | 打中一处条款措辞，旁证两处 | HEAD 的旧读者：分配记录树读者、冷走读、冷恢复拒成「分配记录越界」，旧池级 checker 判 I-1.1、I-5.4 红并按跨度 16386 算 I-3.11 的豁免；**只读挂载照收、读得出文件**——D19 已定项 5 射程那句「不认记号的旧读者读到 0x4001、0x4002 都过不去」不全对（推的影响：只读挂载不读分配记录树，读出的数据是对的） |
| T6 | 没打中（只读码） | 守卫只换删除办法、不碰被测路径；`_removed` 这类绑定都活到作用域末尾 |

## T1　标签 0 配 size > 0：截到 0 的边界打中

### 历史（每一步指到许可它的那一句）

1. 起点：内存盘上新池新建文件（3000 字节），手造成标签 0 配 size 3000（办法与这一批自带的 `whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session.rs` 相同：上段叶那一条改标签 0、载荷清零，拿掉那个数据单元的映射条目）。三处读者都收（`out/main.txt` 第 28 行「第0步（起点）」）。D8 已定项 14 第 477 行：「标签 0 配 size > 0 的第一个文件，写者接得住这一形」（`.claude/kb/decisions/08-核心索引结构.md:477`）。
2. 可写挂载、截到 0（`UserChange::Truncate`，new_size 0）。截断意图与最后一批都做成；上段整棵照抄，标签 0 留着，size 0。许可它的：「let keeps_at_least_one_unit_or_had_none = data_units_after >= 1 || data_units_before == 0;」（`crates/singlefs-core/src/truncate.rs:238`）。照抄的原因：0 个单元时 size 3000 与 size 0 算出同一个尾部，「if data_units < size_in_bytes.div_ceil(payload_capacity_in_bytes()).max(1) {」（`crates/singlefs-core/src/extent_tree.rs:335`），上段只在「|| tail_before != tail_after;」（`crates/singlefs-core/src/truncate.rs:273`）这一类条件下重写，两端由「tail_before: TailOfAFile::of_data_units_and_size(data_units_before, record_before.size),」（`crates/singlefs-core/src/transaction.rs:8947`）与「tail_after: TailOfAFile::of_data_units_and_size(data_units_after, size_after),」（`crates/singlefs-core/src/transaction.rs:8948`）算。这一版三处读者都收（`out/main.txt` 第 29 行）。
3. 建 1 个 inode（不碰 extent 树），再按号覆盖写 inode 2（`UserChange::OverwriteOfAnInode`）：inode 2 与第一个文件在同一片上段叶，这片叶重写。写者按这一版记录的 size 0 取第一个文件那一条：「(None, None) => size_of_the_first_file_after_this_publish」（`crates/singlefs-core/src/transaction.rs:12339`），落到「size > 0 写标签 0，size 0 不写条目」（`crates/singlefs-core/src/extent_tree.rs:409`）——这一条丢了。
4. 这一版：第一个文件 size 0、0 个单元、上段叶没有它的条目。冷走读按「let data_units_within_the_size = data_unit_count_matches_the_inode_size(」（`crates/singlefs-core/src/recovery.rs:7305`）判，size 0 要的单元数是「content_length_in_bytes.div_ceil(payload_capacity).max(1)」（`crates/singlefs-core/src/write_request_split.rs:61`）= 1，没有条目按标签 1、2 那一臂判 0 ≥ 1 不成立，拒成「detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符",」（`crates/singlefs-core/src/recovery.rs:6692`）。可写重挂与冷恢复走同一段，一起拒。挂载态读者把没有条目读成「None => DataUnitsOfTheInode::NoneWritten,」（`crates/singlefs-core/src/mounted_read.rs:809`），size 0 时收（「DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0 && data_units_on_disk == 0 => {」，`crates/singlefs-core/src/write_request_split.rs:103`）。
5. 会话里接着变长到 5000 或 7000：上段照抄，仍没有条目；这一版挂载态读者也拒（`ExtentRecordCountDoesNotMatchTheFileSize`）。
6. 进程退出（或崩溃）之后：可写挂载拒，不退回更早的根（输出是 `Err(Recovery(InvariantViolated …))`），池只能只读挂。

这串操作里没有故障注入，全是用户改动；写者在会话内存里一直自洽（`内存里size/单元数=Some(0)/0`），问题只在落盘那一版。

### 命令与原样输出

在草稿目录的冻结副本拷贝里（`~/.local/share/singlefs/work/m4-validate-code-r2-opus/tree`，`find crates -type f -exec touch {} +` 之后）：

```bash
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 validate_code_round_two_attack_cases; 范围=1 个测试目标; 为什么=复跑 m4-validate-code-r2 攻方腿的标签0、隔离记号与旧读者用例' \
  nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 31 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 8G \
  cargo test --release -p singlefs-harness --test validate_code_round_two_attack_cases -- --test-threads 4 --nocapture
```

（环境：`CARGO_TARGET_DIR`、`TMPDIR` 指草稿目录，`ATTACK_IMAGE_DUMP_DIR` 指草稿目录的 images2，rocksdb 预编译库的四个变量由 `scripts/hosts/rocksdb-prebuilt.py env` 给。）末行 `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s`（用例只打 `ATTACK-` 行、不断言正确形，绿不代表没打中）。

`sed -n '29p;31p;49p;52p' research/prompts/m4-validate-code-r2-opus-model/out/main.txt` 的原样输出：

```text
ATTACK-T1 截到0再重写上段叶再变长 第1步 改size到0：做成2版 内存里size/单元数=Some(0)/0 上段条目=标签0 冷走读=Ok(len=0 same_as_expected=true) 可写重挂=Ok(size=Some(0) units=0) 挂载态读者=Ok(size=0 units=0 len=0 same_as_expected=true) 冷恢复=FileRead { root: (InstanceGeneration(2), CheckpointTxg(7)), content: [] }
ATTACK-T1 截到0再重写上段叶再变长 第3步 覆盖写inode2共100字节：做成1版 内存里size/单元数=Some(0)/0 上段条目=没有条目 冷走读=Err(InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" }) 可写重挂=Err(Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" })) 挂载态读者=Ok(size=0 units=0 len=0 same_as_expected=true) 冷恢复=Failed { root: Some((InstanceGeneration(2), CheckpointTxg(9))), failure: InvariantViolated
ATTACK-T1扫 建1个再覆盖写inode2：上段条目=没有条目 冷走读=Err(InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" }) 可写重挂=Err(Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" })) 挂载态读者=Ok(size=0 units=0 len=0 same_as_expected=true) 冷恢复=Failed { root: Some((InstanceGeneration(2), CheckpointTxg(9))), failure: InvariantViolated checker 判红 4 条 [I-3.1:盘 0：记账的已分配 Some(1376256)，遍历全部有效根得到 1343488（其中隔离豁免 0、F 之下仍在 defer 的 0、钉带出来的 0）；机理：根环槽数 24、最新根 txg 9、环里自证过的根槽 10 个、最老的自证过的根 txg 0、遍历的候选根槽 10 个 | I-3.11:盘 0：记账的已分配 Some(1376256) 减 defer 待释放 Some(1048576)，不等于从最新根（txg 9）走读到的 294912（其中隔离豁免 0） | I-1.13:扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 1 处，第一处 key （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1）：扫描重建 落点 {盘 0 槽 50180、盘 1 槽 50180}、遍历方向没有这一条；「每个 inode 的数据单元集合 | I-3.15:根（实例 1、txg 3）：盘 1 槽 50180 的分配记录（未释放、跨 2 槽）罩住的单元（码 1、诞生代号 3、写序实例 1、事务号 1）在这一版的中央映射里没有一把相符的 key 指着这一槽（指着这一槽的 key：没有；与它相符的 key 落在：没有）]
ATTACK-T1扫 建1个写inode2再变长到7000：上段条目=没有条目 冷走读=Err(InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" }) 可写重挂=Err(Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" })) 挂载态读者=打开Err(ExtentRecordCountDoesNotMatchTheFileSize { inode: InodeNumber(1), extent_records: 0, data_units_implied_by_the_file_size: 1, file_size_in_bytes: 7000 }) 冷恢复=Failed { root: Some((InstanceGeneration(2), CheckpointTxg(10))), failure: InvariantViolate checker 判红 4 条 [I-3.1:盘 0：记账的已分配 Some(1556480)，遍历全部有效根得到 1523712（其中隔离豁免 0、F 之下仍在 defer 的 0、钉带出来的 0）；机理：根环槽数 24、最新根 txg 10、环里自证过的根槽 11 个、最老的自证过的根 txg 0、遍历的候选根槽 11  | I-3.11:盘 0：记账的已分配 Some(1556480) 减 defer 待释放 Some(1228800)，不等于从最新根（txg 10）走读到的 294912（其中隔离豁免 0） | I-1.13:扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 1 处，第一处 key （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1）：扫描重建 落点 {盘 0 槽 50180、盘 1 槽 50180}、遍历方向没有这一条；「每个 inode 的数据单元集合 | I-3.15:根（实例 1、txg 3）：盘 1 槽 50180 的分配记录（未释放、跨 2 槽）罩住的单元（码 1、诞生代号 3、写序实例 1、事务号 1）在这一版的中央映射里没有一把相符的 key 指着这一槽（指着这一槽的 key：没有；与它相符的 key 落在：没有）]
```

冷走读对「没有第一个文件那一条」的读法是现查的：「上段里没有第一个文件那一条：第一个文件删完之后的零条目根」（`crates/singlefs-core/src/recovery.rs:5962`），尾部按「TailOfAFile::NoHoleAfterTheLastDataUnit,」（`crates/singlefs-core/src/recovery.rs:5968`）交，于是走到标签 1、2 那一臂、要 ⌈0 ÷ 净荷⌉.max(1) = 1 个单元。写者那一侧的注释说这一形「读者按没有条目收」，对挂载态读者成立、对冷走读不成立。

### 后续放开扫（前缀固定：标签 0 镜像可写挂载、截到 0）

照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节，截到 0 之后由用户定的那几步放开扫了 12 种（`t1_sweep_follow_ups_after_truncating_a_tag_zero_file_to_zero`，`out/main.txt` 第 42–53 行）：

| 后续 | 写出的那一版第一个文件的上段条目 | 冷走读 / 可写重挂 / 冷恢复 | 挂载态读者 |
|---|---|---|---|
| 不再做别的 | 标签 0 | 收 | 收 |
| 再截到 0（N = S） | 标签 0 | 收 | 收 |
| 建 1 个 inode | 标签 0 | 收 | 收 |
| 建 1 个、覆盖写 inode 2 | 没有条目 | **拒** | 收 |
| 建 1 个、顺序写 inode 2 两单元 | 没有条目 | **拒** | 收 |
| 建 142 个、覆盖写 inode 143（上段长高） | 没有条目 | **拒** | 收 |
| 建 1 个、写 inode 2、unlink inode 2 | 没有条目 | **拒** | 收 |
| 建 1 个、unlink inode 2（没写过） | 标签 0 | 收 | 收 |
| 建快照 | 标签 0 | 收 | 收 |
| 变长到 7000 | 标签 0 | 收 | 收 |
| 建 1 个、写 inode 2、变长到 7000 | 没有条目 | **拒** | **拒** |
| unlink 第一个文件 | 标签 0（记录已没了，这一条留成孤条目） | 收（没有文件） | 没有这个 inode |

前缀换成「截到 100、建 1 个、写 inode 2、截到 0、写 inode 2」同样中（`out/main.txt` 第 41 行）；不重写第一片上段叶的那几段都不中，截到 0 之后只变长、再截到 0 的也不中（第 19–27、34–40 行）。普通文件（有单元）截到 0 留一个声明长度 0 的单元、标签 2，不走这一臂（第 58 行）。

池级 checker 对这几张拒挂的镜像不多判一条红：起点镜像本来就有手造带来的 I-3.1、I-3.11、I-1.13、I-3.15 四条（拿掉映射条目留下的那条已分配无引用记录），拒挂的五张也恰好是这四条（`out/main.txt` 第 49 行与第 54 行判红的名字相同；同一条命令逐行抽名字，原样见下）。checker 的标签判法把「没有上段叶条目」与标签 0 并在一起，「size 7000、没有条目、0 个单元」它也判对得上。

```text
ATTACK-T1扫 unlink第一个文件 => I-3.1 I-3.11 I-3.15 
ATTACK-T1扫 再截到0一次（N=S） => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 变长到7000 => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 只截到0 => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 建142个再覆盖写inode143（上段长高） => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 建1个inode => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 建1个再unlink它（没写过） => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 建1个再覆盖写inode2 => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 建1个再顺序写inode2两单元 => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 建1个写inode2再unlink它 => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 建1个写inode2再变长到7000 => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 建快照 => I-3.1 I-3.11 I-1.13 I-3.15 
ATTACK-T1扫 标签0起点镜像 => I-3.1 I-3.11 I-1.13 I-3.15 
```

起点镜像带的这条泄漏与拒挂无关：拒的是 extent 树对 size 的判，不读分配记录。要排除它，可以另造一张把那条记录也改成已释放的起点再跑一次；我没造，记进限度。

### 改法甲在副本上（量过）

改法甲：`ExtentUpperLeafTarget::of_a_file_without_data_units` 不看 size、一律交标签 0（它只在第一个文件有记录时被调，记录没了照旧不写）。打在草稿目录的第二份拷贝上（`tools/apply_fix_candidate_a.py`），同一份用例：`sed -n '31p;42p;49p;52p' research/prompts/m4-validate-code-r2-opus-model/out/fix-candidate.txt` 原样输出：

```text
ATTACK-T1 截到0再重写上段叶再变长 第3步 覆盖写inode2共100字节：做成1版 内存里size/单元数=Some(0)/0 上段条目=标签0 冷走读=Ok(len=0 same_as_expected=true) 可写重挂=Ok(size=Some(0) units=0) 挂载态读者=Ok(size=0 units=0 len=0 same_as_expected=true) 冷恢复=FileRead { root: (InstanceGeneration(2), CheckpointTxg(9)), content: [] }
ATTACK-T1扫 unlink第一个文件：上段条目=没有条目 冷走读=Ok(None) 可写重挂=Ok(有树没有文件) 挂载态读者=打开Err(NoSuchInode { inode: InodeNumber(1) }) 冷恢复=NoFile { root: (InstanceGeneration(2), CheckpointTxg(9)) } checker 判红 3 条 [I-3.1:盘 0：记账的已分配 Some(1425408)，遍历全部有效根得到 1392640（其中隔离豁免 0、F 之下仍在 defer 的 0、钉带出来的 0）；机理：根环槽数 24、最新根 txg 9、环里自证过的根槽 10 个、最老的自证过的根 txg 0、遍历的候选根槽 10 个 | I-3.11:盘 0：记账的已分配 Some(1425408) 减 defer 待释放 Some(1064960)，不等于从最新根（txg 9）走读到的 327680（其中隔离豁免 0） | I-3.15:根（实例 1、txg 3）：盘 1 槽 50180 的分配记录（未释放、跨 2 槽）罩住的单元（码 1、诞生代号 3、写序实例 1、事务号 1）在这一版的中央映射里没有一把相符的 key 指着这一槽（指着这一槽的 key：没有；与它相符的 key 落在：没有）]
ATTACK-T1扫 建1个再覆盖写inode2：上段条目=标签0 冷走读=Ok(len=0 same_as_expected=true) 可写重挂=Ok(size=Some(0) units=0) 挂载态读者=Ok(size=0 units=0 len=0 same_as_expected=true) 冷恢复=FileRead { root: (InstanceGeneration(2), CheckpointTxg(9)), content: [] } checker 判红 4 条 [I-3.1:盘 0：记账的已分配 Some(1376256)，遍历全部有效根得到 1343488（其中隔离豁免 0、F 之下仍在 defer 的 0、钉带出来的 0）；机理：根环槽数 24、最新根 txg 9、环里自证过的根槽 10 个、最老的自证过的根 txg 0、遍历的候选根槽 10 个 | I-3.11:盘 0：记账的已分配 Some(1376256) 减 defer 待释放 Some(1048576)，不等于从最新根（txg 9）走读到的 294912（其中隔离豁免 0） | I-1.13:扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 1 处，第一处 key （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1）：扫描重建 落点 {盘 0 槽 50180、盘 1 槽 50180}、遍历方向没有这一条；「每个 inode 的数据单元集合 | I-3.15:根（实例 1、txg 3）：盘 1 槽 50180 的分配记录（未释放、跨 2 槽）罩住的单元（码 1、诞生代号 3、写序实例 1、事务号 1）在这一版的中央映射里没有一把相符的 key 指着这一槽（指着这一槽的 key：没有；与它相符的 key 落在：没有）]
ATTACK-T1扫 建1个写inode2再变长到7000：上段条目=标签0 冷走读=Ok(len=7000 same_as_expected=true) 可写重挂=Ok(size=Some(7000) units=0) 挂载态读者=Ok(size=7000 units=0 len=7000 same_as_expected=true) 冷恢复=FileRead { root: (InstanceGeneration(2), CheckpointTxg(10)), content: [0, 0, 0, 0, 0, 0, 0 checker 判红 4 条 [I-3.1:盘 0：记账的已分配 Some(1556480)，遍历全部有效根得到 1523712（其中隔离豁免 0、F 之下仍在 defer 的 0、钉带出来的 0）；机理：根环槽数 24、最新根 txg 10、环里自证过的根槽 11 个、最老的自证过的根 txg 0、遍历的候选根槽 11  | I-3.11:盘 0：记账的已分配 Some(1556480) 减 defer 待释放 Some(1228800)，不等于从最新根（txg 10）走读到的 294912（其中隔离豁免 0） | I-1.13:扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 1 处，第一处 key （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1）：扫描重建 落点 {盘 0 槽 50180、盘 1 槽 50180}、遍历方向没有这一条；「每个 inode 的数据单元集合 | I-3.15:根（实例 1、txg 3）：盘 1 槽 50180 的分配记录（未释放、跨 2 槽）罩住的单元（码 1、诞生代号 3、写序实例 1、事务号 1）在这一版的中央映射里没有一把相符的 key 指着这一槽（指着这一槽的 key：没有；与它相符的 key 落在：没有）]
```

`out/fix-candidate.txt` 里「E142 走读同款」那一句的个数（`grep -c 'Err(InvariantViolated'`）是 0，`out/main.txt` 是 8。这一批自带的四个相关测试目标在同一份打了改法甲的拷贝上照绿（每条命令带同一个范围声明形状、经 `capped.sh 31` 与 `run-with-memory-cap.sh 8G`，原样末行）：

```text
whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
whole_file_hole_of_upper_leaf_tag_zero_with_a_positive_size_read_by_every_reader: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
size_extension_and_the_hole_after_the_last_data_unit: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
truncation_intent_batches_and_final_batch: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s
```

只在我的模型上量过、被攻过零轮。它让「标签 0 配 size 0」成了写者的正常形；这一形挂载态读者与冷走读今天都收（`out/main.txt` 第 29 行），池级 checker 也收，但 K5、S1 的条款原文只写了 size > 0，要不要把 size 0 也写进条款归主 agent。

### 打中之后的四句

1. **分不分辨臂**：分辨。r1 判决「四、交用户」T5 甲的三个候选里，子（`publish_size_change` 交类型化拒绝）与寅（读者不收这一形）都到不了截到 0 这一步；这一格只打中「丑」今天的实现，而改法甲在「丑」之内就消掉它，打不到「丑」这个选择本身。
2. **被判的系统当时看不看得到判别它的东西**：看得到。重写上段叶那次发布手里有第一个文件这一版的记录（size 0）与空的数据指针，`build_extent_tree` 的 `(None, None)` 那一臂按 size 选写不写，选成了冷走读不收的那一形。
3. **满足的是判据字面的哪一个分句**：正文 T1「截到 0 与从 0 变长的边界」；D8 已定项 14 第 477 行「写者接得住这一形」——写者写出了可写挂载拒收的一版。另有一句代码注释不成立：`extent_tree.rs:409` 起那段说 size 0 不写条目时读者按「没有条目」收，冷走读不收。
4. **跑前条款给的每个改法在打中的那几格上还中不中**：这一格没有跑前条款；下表是我提的。

| 改法 | 修哪一格 | 在打中的 5 种后续上 | 依据 |
|---|---|---|---|
| 甲：第一个文件没有单元时一律写标签 0（size 0 也写） | 写者丢条目 | 5 种都不中，另外 7 种结局不变；顺带 unlink 第一个文件之后不再留孤条目（`out/fix-candidate.txt` 第 42 行「上段条目=没有条目」，`out/main.txt` 第 42 行是「标签0」） | 量过（副本） |
| 乙：冷走读、可写重挂、冷恢复把「没有条目、size 0、0 个单元」当成合法的第一个文件 | 读者拒 | 截到 0 之后重写叶那 4 种推的不中；「写 inode 2 再变长到 7000」那一种推的照中（没有条目而 size > 0，挂载态读者也拒） | 推的 |
| 丙：标签 0 的文件截到 0 交类型化拒绝 | 不让这一形出现 | 推的全不中，但截到 0 成了用户可见的拒绝，与「丑」相反 | 推的 |

### 推翻条件

在冻结副本上把同一张标签 0 镜像截到 0、按号覆盖写 inode 2 之后，可写重挂收下、冷走读读出 0 字节，这一格就不成立。另一种推翻：证明起点那张手造镜像本身不合法（K5、S1 不许它），那这一格退成「坏镜像上的行为」——但这一批自带的测试用的就是同一张起点。

## T2　隔离那次发布置记号：没打中

造过的形状（全在内存盘上，起点同 T1 的新池新建文件 A，txg 3）：

- 盘 1 上 A 的数据单元那一份改坏，覆盖写 B：隔离 2 份，两块盘的记录都是 0x4002、分配代 3（`out/main.txt` 第 18 行）。只坏一块盘时另一块盘照样置——发布路径传进去的是「任一份没对上交回两份」那一份清单（`crates/singlefs-core/src/transaction.rs:5074` 起那段说明），`release_leaving_the_record_allocated_on` 对清单里的盘置记号、跳过释放：「if devices_whose_record_stays_allocated.contains(&device.device) {」（`crates/singlefs-core/src/allocator.rs:2227`）。
- 两单元的文件 A2（txg 4），单元 0 坏在盘 0、单元 1 坏在盘 1，同一次覆盖写 B 隔离两个单元：4 份、四条记录都是 0x4002、分配代 4（第 4 行）。
- B 换下的 12 个单元在盘 1 上全改坏（数据单元与各树节点）：隔离 20 份、20 条记录带记号（单槽的 0x4001、两槽的 0x4002），树表与另一个不经映射释放的单元照常释放（第 2 行）。这张镜像重挂、三处读者都收（第 11 行）。
- 隔离之后重挂（分配器从盘上重建），记号跟着回来（第 12 行）；会话里接着覆盖写 40 次，第 9、19、29、39 次之后读回都还是 0x4002，checker 在旧根转出窗口之后 108 条全不红（第 14–17 行）。
- 记号与已释放标志的组合：写者一侧只写得出 0x0、0x4000、0x8000 三种（`to_bytes` 的断言拦 0xC000）；手改出来的组合归 T4。

```text
ATTACK-Q 两单元各坏一盘之后 B：隔离 4 份 盘上 盘0槽50182:仍分配代4跨度段0x4002 盘0槽50184:仍分配代4跨度段0x4002 盘1槽50182:仍分配代4跨度段0x4002 盘1槽50184:仍分配代4跨度段0x4002 checker 判红 3 条 [I-2.1:数据单元 在盘 0 槽 50182 的那一份与位置条目里的校验和对不上 | I-4.8:候选根 txg 4 出发的遍历有单元对不上或读不出 | I-7.4:候选根 txg 4 引用的单元已被复用或抹头（校验和对不上或头用不了）]
ATTACK-Q 重挂之后分配器里：盘0槽50180:仍分配代3跨度段0x4002 盘1槽50180:仍分配代3跨度段0x4002
ATTACK-Q 重挂后第19次覆盖写：抬F 0 次 整理 没起 盘上 盘0槽50180:仍分配代3跨度段0x4002 盘1槽50180:仍分配代3跨度段0x4002 checker 全不红（108 条）
ATTACK-Q 重挂后第9次覆盖写：抬F 0 次 整理 没起 盘上 盘0槽50180:仍分配代3跨度段0x4002 盘1槽50180:仍分配代3跨度段0x4002 checker 判红 3 条 [I-2.1:数据单元 在盘 1 槽 50180 的那一份与位置条目里的校验和对不上 | I-4.8:候选根 txg 3 出发的遍历有单元对不上或读不出 | I-7.4:候选根 txg 3 引用的单元已被复用或抹头（校验和对不上或头用不了）]
ATTACK-Q 隔离 A 的数据单元之后 B：隔离 2 份 带记号2条[盘0槽50180:仍分配代3跨度段0x4002 盘1槽50180:仍分配代3跨度段0x4002] checker 判红 3 条 [I-2.1:数据单元 在盘 1 槽 50180 的那一份与位置条目里的校验和对不上 | I-4.8:候选根 txg 3 出发的遍历有单元对不上或读不出 | I-7.4:候选根 txg 3 引用的单元已被复用或抹头（校验和对不上或头用不了）]
```

没打中的理由：置记号与释放走同一个循环、同一份清单，固定点的演练按「全释放」算哪几片叶变了，置记号改的是同一片叶，`settle_the_allocation_record_tree` 交出的重写集罩得住（真发出来的那一版读回逐条对得上，上面几行就是读回的）。这一格我只量了数据单元与树节点两类、一次发布至多 10 个单元。

## T3　清记号：回退复活与释放，没打中

| 历史 | 结局（`out/main.txt`） |
|---|---|
| 隔离、进程重开可写挂载、写回原样、回退到 A | 做成；两块盘的记录 0x0002、分配代 3；checker 108 条全不红；三处读者读出 A（第 13 行） |
| A2 两单元隔离、重挂、写回、回退到 A2 | 做成；四条记录清成 0x0002、分配代 4；checker 全不红（第 9 行） |
| A →（隔离）B → C → 回退到 B → 回退到 A | 回退到 B（不引用隔离单元）记号留着（第 3 行）；再回退到 A 清掉（第 6 行）；之后重挂做成（第 5 行） |
| B 换下的 12 个单元全隔离、重挂、写回、回退到 A | 槽 50180、50240、50242、50244 四个清掉；50245–50250 六个留着记号——新这一版不引用它们（checker 的 I-3.17 第二句没红），checker 全不红（第 7 行）。哪几个是回退从目标取的用户可见单元、哪几个是随现行那一版的树节点，我是按「清了的与留着的各自被不被新根引用」推的，没逐个对单元类 |
| 隔离、重挂、40 次覆盖写之后回退到 A | 拒 `NotInRing`（A 已转出根环），记号留着（第 8 行） |

```text
ATTACK-Q C 之后回退到 B：做成（新版 txg 6，复活 8 份、释放 4 个、隔离 0 份） 盘上 盘0槽50180:仍分配代3跨度段0x4002 盘1槽50180:仍分配代3跨度段0x4002 checker 判红 3 条 [I-2.1:数据单元 在盘 1 槽 50180 的那一份与位置条目里的校验和对不上 | I-4.8:候选根 txg 3 出发的遍历有单元对不上或读不出 | I-7.4:候选根 txg 3 引用的单元已被复用或抹头（校验和对不上或头用不了）]
ATTACK-Q 再回退到 A：做成（新版 txg 7，复活 6 份、释放 4 个、隔离 0 份） 盘上 盘0槽50180:仍分配代3跨度段0x0002 盘1槽50180:仍分配代3跨度段0x0002 checker 全不红（108 条） 读者 上段条目=标签2 冷走读=Ok(len=3000 same_as_expected=true) 可写重挂=Ok(size=Some(3000) units=1) 挂载态读者=Ok(size=3000 units=1 len=3000 same_as_expected=true) 冷恢复=FileRead { root: (InstanceGeneration(1), CheckpointTxg(7)), content: [0, 1, 2, 3, 4, 5, 6,
ATTACK-Q 写回原样回退到 A：做成（新版 txg 8，复活 0 份、释放 4 个、隔离 0 份） 盘上 盘0槽50180:仍分配代3跨度段0x0002 盘0槽50240:仍分配代3跨度段0x0001 盘0槽50242:仍分配代3跨度段0x0002 盘0槽50244:仍分配代3跨度段0x0001 盘0槽50245:仍分配代3跨度段0x4001 盘0槽50246:仍分配代3跨度段0x4001 盘0槽50247:仍分配代3跨度段0x4001 盘0槽50248:仍分配代3跨度段0x4001 盘0槽50249:仍分配代3跨度段0x4001 盘0槽50250:仍分配代3跨度段0x4001 盘0槽50251:已释放代4跨度段0x8001 盘0槽50252:已释放代4跨度段0x8001 盘1槽50180:仍分配代3跨度段0x0002 盘1槽50240:仍分配代3跨度段0x0001 盘1槽50242:仍分配代3跨度段0x0002 盘1槽50244:仍分配代3跨度段0x0001 盘1槽50245:仍分配代3跨度段0x4001 盘1槽50246:仍分配代3跨度段0x4001 盘1槽50247:仍分配代3跨度段0x4001 盘1槽50248:仍分配代3跨度段0x4001 盘1槽50249:仍分配代3跨度段0x4001 盘1槽50250:仍分配代3跨度段0x4001 盘1槽50251:已释放代4跨度段0x8001 盘1槽50252:已释放代4跨度段0x8001 checker 全不红（108 条） 读者 上段条目=标签2 冷走读=Ok(len=3000 same_as_expected=true) 可写重挂=Ok(size=Some(3000) units=1) 挂载态读者=Ok(size=3000 units=1 len=3000 same_as_expected=true) 冷恢复=FileRead { root: (InstanceGeneration(2), CheckpointTxg(8)), content: [0, 1, 2, 3, 4, 5, 6,
ATTACK-Q 重挂之后回退到 A：做成（新版 txg 8，复活 6 份、释放 4 个、隔离 0 份） 盘上 盘0槽50180:仍分配代3跨度段0x0002 盘1槽50180:仍分配代3跨度段0x0002 checker 全不红（108 条） 读者 上段条目=标签2 冷走读=Ok(len=3000 same_as_expected=true) 可写重挂=Ok(size=Some(3000) units=1) 挂载态读者=Ok(size=3000 units=1 len=3000 same_as_expected=true) 冷恢复=FileRead { root: (InstanceGeneration(2), CheckpointTxg(8)), content: [0, 1, 2, 3, 4, 5, 6,
```

清记号只在「仍分配、同分配代、带记号」那一臂入列：「quarantine_marks_to_clear.push((」（`crates/singlefs-core/src/mount.rs:9216`）。回退目标里没有那条隔离的，不进 `target_units`，不清——第 3 行就是这一格。清错的形状我造了一种（一块盘带记号、另一块不带，T4 第 67 行那张镜像）只到可写挂载收下为止，没接着回退：正文说这是已记的欠账「回退遇一块盘带记号照清照做」，不重复。

「核心层没有释放隔离记录的路径」被别的路径意外走到：40 次覆盖写里会话的整理一次都没起、抬 F 0 次（第 14–17 行「抬F 0 次 整理 没起」），扫描重建没调。这一句我只量到这么多，整理真起跑、扫描重建碰到带记号记录的形状没造，记进限度。

## T4　读者认记号：没打中

读跨度段的地方：核心层只有一个解析点 `AllocationRecord::parse`（`grep -rn 'ALLOCATION_RECORD_BYTES\|0x7fff\|0x7FFF\|RELEASED_FLAG'` 在 `crates/singlefs-core/src` 里，跨度段的读只在 `allocator.rs` 那一处，别处拿的都是解好的 `span_slots`），0xC000 判坏在「return Err(AllocationRecordEntryRefusal::QuarantineMarkOnAReleasedRecord);」（`crates/singlefs-core/src/allocator.rs:143`），落叶判按去掉记号的跨度判「record.span_slots > 0 && record.slot.0 + u64::from(record.span_slots) - 1 <= last_slot.0」（`crates/singlefs-core/src/allocation_record_tree.rs:282`）。区域扫描、整理计划、恢复、回退、实例表扫拿的都是解好的记录（`crates/singlefs-core/src/region_scan.rs:221`、`crates/singlefs-core/src/mount.rs:1630` 等处的 `u64::from(record.span_slots)`），没有一处自己切跨度段。

在 B（隔离之后）那张合法镜像上逐条手改跨度段、重封、校验和补到根槽，四种核心层读者与池级 checker 各判一遍（`t4_span_fields_set_by_hand_on_the_quarantined_and_on_a_live_record`）：

| 手改 | 可写挂载 / 读分配记录树 / 冷恢复 | 只读挂载 | 池级 checker 多判的 |
|---|---|---|---|
| 不改（合法镜像） | 收 | 收 | 无（只有那份坏副本带来的 I-2.1、I-4.8、I-7.4） |
| 两盘 0xC002 | 拒「隔离记号与已释放标志同时置」 | 收 | I-3.9、I-3.11、I-3.17 第一句 |
| 盘 0 一块 0xC002 | 拒，同上 | 收 | 同上 |
| 两盘 0xC000 | 拒，同上 | 收 | I-1.1 等 |
| 两盘 0x4000（跨度 0） | 拒「分配记录不在它所在叶……」 | 收 | I-1.1、I-3.11、I-3.18 |
| 两盘 0x4003（跨度 3） | 拒「同一块盘上两条分配记录罩住同一个槽」 | 收 | I-5.4、I-3.17 第二句 |
| 只剩盘 1 带记号 | 收 | 收 | I-3.11、I-3.15、I-3.17 第三句 |
| 两盘清成 0x0002（漏记号） | 收 | 收 | I-3.11、I-3.15 |
| B 引用着的活单元两盘带记号 | 收 | 收 | I-3.17 第二句 |

原样输出（`sed -n '60p;65p;67p;59p' out/main.txt`）：

```text
ATTACK-T4 B 引用着的活单元两盘带记号 0x4002：可写挂载=Ok 只读挂载=Ok 读分配记录树=Ok(52 条) 冷恢复=FileRead { root: (InstanceGeneration(1), CheckpointTxg(4)), content: [3, 10, 17, 24, 31, 38, 45, 52, 59, 66, 73, 80, 87, checker 判红 4 条 [I-2.1:数据单元 在盘 1 槽 50180 的那一份与位置条目里的校验和对不上 | I-4.8:候选根 txg 3 出发的遍历有单元对不上或读不出 | I-7.4:候选根 txg 3 引用的单元已被复用或抹头（校验和对不上或头用不了） | I-3.17:②：盘 1 槽 50182 的分配记录带隔离记号，它罩住的槽 [50182, 50183] 被最新根（txg 4）引用]
ATTACK-T4 合法的隔离镜像：可写挂载=Ok 只读挂载=Ok 读分配记录树=Ok(52 条) 冷恢复=FileRead { root: (InstanceGeneration(1), CheckpointTxg(4)), content: [3, 10, 17, 24, 31, 38, 45, 52, 59, 66, 73, 80, 87, checker 判红 3 条 [I-2.1:数据单元 在盘 1 槽 50180 的那一份与位置条目里的校验和对不上 | I-4.8:候选根 txg 3 出发的遍历有单元对不上或读不出 | I-7.4:候选根 txg 3 引用的单元已被复用或抹头（校验和对不上或头用不了）]
ATTACK-T4 隔离记录两盘 0xC002（记号加已释放）：可写挂载=Err(Recovery(UnitMalformed { what: "分配记录树叶里一条记录的跨度段隔离记号与已释放标志同时置" })) 只读挂载=Ok 读分配记录树=Err(UnitMalformed { what: "分配记录树叶里一条记录的跨度段隔离记号与已释放标志同时置" }) 冷恢复=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: UnitMalformed { what: "分配记录树叶里一条记录的跨度段隔离记号与已释放标 checker 判红 6 条 [I-2.1:数据单元 在盘 1 槽 50180 的那一份与位置条目里的校验和对不上 | I-3.9:盘 1 槽 50180 的分配记录释放代 3 不在 (3, 4] 里：还引用它的最新一条有效根是 txg 3，最早不再引用它的是 txg 4 | I-3.11:盘 0：记账的已分配 Some(507904) 减 defer 待释放 Some(212992)，不等于从最新根（txg 4）走读到的 262144（其中隔离豁免 0） | I-4.8:候选根 txg 3 出发的遍历有单元对不上或读不出 | I-7.4:候选根 txg 3 引用的单元已被复用或抹头（校验和对不上或头用不了） | I-3.17:①：盘 1 槽 50180 的分配记录隔离记号与已释放标志同时置（跨度段 0x4000 与 0x8000）]
ATTACK-T4 隔离记录只剩盘 1 带记号（盘 0 清成 0x0002）：可写挂载=Ok 只读挂载=Ok 读分配记录树=Ok(52 条) 冷恢复=FileRead { root: (InstanceGeneration(1), CheckpointTxg(4)), content: [3, 10, 17, 24, 31, 38, 45, 52, 59, 66, 73, 80, 87, checker 判红 6 条 [I-2.1:数据单元 在盘 1 槽 50180 的那一份与位置条目里的校验和对不上 | I-3.11:盘 0：记账的已分配 Some(507904) 减 defer 待释放 Some(212992)，不等于从最新根（txg 4）走读到的 262144（其中隔离豁免 0） | I-4.8:候选根 txg 3 出发的遍历有单元对不上或读不出 | I-7.4:候选根 txg 3 引用的单元已被复用或抹头（校验和对不上或头用不了） | I-3.15:根（实例 1、txg 4）：盘 1 槽 50180 的分配记录（未释放、跨 2 槽）罩住的单元（码 1、诞生代号 3、写序实例 1、事务号 1）在这一版的中央映射里没有一把相符的 key 指着这一槽（指着这一槽的 key：没有；与它相符的 key 落在：没有） | I-3.17:③：起点槽 50180、跨 2 槽的单元，盘 [1] 上的记录带隔离记号，盘 [0] 上的不带]
```

为什么不记打中：

- 只读挂载在每一种坏法上都收，是因为挂载态读者不读分配记录树（`crates/singlefs-core/src/mounted_read.rs` 里 `grep -n allocation` 只命中第 1438 行，那是测试里造根记录）。这在隔离记号那一批之前就是这样，与记号无关；读出来的文件内容也不依赖分配记录。
- 后三行（只剩一块盘带记号、漏记号、活单元带记号）核心层照收：D19 已定项 5 第 117 行只要求「池级 checker 只认记号」，核心层判不判这三句没有条款；三张都被 checker 判红。活单元带记号那一张可写挂载之后覆盖写一次，那条记录照常释放成 0x8002、记号随之清掉（`out/main.txt` 第 61 行），下一版 checker 不再多判——写者把这张坏镜像洗干净了，没有把坏扩散出去。
- 正文点名的「区域扫描、整理计划、恢复三处对带记号的记录各怎么处置」：恢复与回退量了（上面与 T3）；区域扫描与整理计划要整理真起跑才走得到，我没造到（T3 末段），正文第二节已记「区域扫描不认记号」为欠账，没重复。

## T5　第一轮的改法本身与从没被看过的地方

### 旧读者遇到带记号的池（量过，HEAD 那一版的代码）

做法：冻结副本拷贝上的写者写两张镜像（同一段历史，一张盘 1 那一份改坏、B 隔离之后带记号，一张不改坏、不带记号），只把写过的扇区落进草稿目录；`git archive 841b3beab` 导出 HEAD 那一版的 `crates/` 编一份，用 HEAD 的读者读这两张（`prototype/old_readers_on_quarantine_marks.rs`）。原样输出（`out/old-readers.txt` 全文）：

```text
ATTACK-T5旧读者 不带记号的对照：可写挂载=Err(NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: I) 只读挂载=Ok 读分配记录树=Ok(52 条) 冷走读=Ok(Some(2700) 字节) 冷恢复=FileRead { root: (InstanceGeneration(1), CheckpointTxg(4)), content: [3, 10, 17, 24, 31, 38, 45, 52, 59, 66, 73, 80, 87, 池级checker判红=[I-2.4:区域 0 槽 0 的根（实例 0、txg 0）：根槽偏移 436 的字节是 0xff，算法类型 / nonce / MAC 那 29 字节（428..457）加密关着时恒 0]
ATTACK-T5旧读者 带记号：可写挂载=Err(NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: I) 只读挂载=Ok 读分配记录树=Err(AllocationRecordOutsideThePoolGeometry { what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽" }) 冷走读=Err(AllocationRecordOutsideThePoolGeometry { what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽" }) 冷恢复=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: AllocationRecordOutsideThePoolGeometry { what:  池级checker判红=[I-1.1:分配记录树（树 65538）层级 0 盘 0 第 61 格的节点：盘 0 槽 50180 跨 16386 的记录不落在这片叶按位置罩的那一段里，或末槽越过叶的末槽 | I-2.1:数据单元 在盘 1 槽 50180 的那一份与位置条目里的校验和对不上 | I-2.4:区域 0 槽 0 的根（实例 0、txg 0）：根槽偏移 436 的字节是 0xff，算法类型 / nonce / MAC 那 29 字节（428..457）加密关着时恒 0 | I-3.11:盘 0：记账的已分配 Some(507904) 减 defer 待释放 Some(212992)，不等于从最新根（txg 4）走读到的 268730368（其中隔离豁免 268468224） | I-4.8:候选根 txg 3 出发的遍历有单元对不上或读不出 | I-5.4:根（实例 1、txg 4）那棵分配记录树、盘 0：槽 50180 跨 16386 的记录与槽 50182 跨 2 的记录罩住同一个槽 | I-7.4:候选根 txg 3 引用的单元已被复用或抹头（校验和对不上或头用不了） | I-3.16:盘 0：记账的全空聚簇段数 3310，全扫重算是 3055]
exit=0
```

读法：

- 可写挂载两张都拒，拒因一样（系统配置见证的那一版比择到的新，重读一次仍读不出）：不带记号的对照也拒，说明 HEAD 的写入口对冻结副本写出的池本来就挂不上，这一层分辨不了记号的效果。对照里池级 checker 判的 I-2.4（根槽偏移 436 那一字节 0xff）提示两版在根槽上已经不一致；这不是这一批改的（隔离记号那一批没碰根槽），我没追，算 HEAD 与工作区之间别的会话的改动。
- 分辨得出记号的四层：分配记录树读者、冷走读、冷恢复——对照收、带记号的拒成 `AllocationRecordOutsideThePoolGeometry`（报的是「越界」，不是「不认识的格式」）；只读挂载两张都收，文件读得出；HEAD 的池级 checker 判 I-1.1、I-5.4 红，并把这条按跨度 16386 记进 I-3.11 的隔离豁免（「其中隔离豁免 268468224」= 16386 槽 × 16384 字节）。
- 与条款比：D19 已定项 5 第 119 行写「不认记号的旧读者读到 0x4001、0x4002 都过不去——分配记录树读者拒整棵树（挂载与恢复失败），池级 checker 判违例，GPU 判器停止走盘」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:119`）。量下来「都过不去」不全对：只读挂载过得去。后果是推的：只读挂载不读分配记录树，读出来的数据对，所以只是条款措辞多说了；这一批没给记号配 incompat 位（`system_configuration.rs` 不在 diff 里），旧读者拒的那几层报的是「坏盘」而不是「格式不认识」，D15 已定项 9 判定表「改变已有字段含义」那一行要的是 incompat 位——E187 第一段已把落点甲落在那一行、「是哪一种实例」交了主 agent，我只补上这一轮量到的四层与只读挂载那一处例外。GPU 判器那一句没量（不起 GPU）。

### 第一轮的改法本身

- 「要改 K5 的『变长』一句与 `truncate.rs` 两处断言的前提」：代码一侧两处都改了——「let unit = DataUnitIndexInFile(data_units_existing.checked_sub(1)?);」（`crates/singlefs-core/src/truncate.rs:141`）与第 238 行那个前提。K5 那一句在 kb 里改没改是写回，我没核（辩方那一格）。改了的这两处在 size > 0 的那几段上都对（`out/main.txt` 第 19–27、34–40 行三处读者都收、内容都对），问题只在截到 0 那一条边界，见 T1。
- r1 判决第四节第二条岔路（大 size 冷读与 RefFS 整份分配，C878，用户定这一版不改）：这一批让标签 0 的文件也能变长（以前 panic），变长到多大照旧只改记录，所以「冷走读按 size 整份分配」现在多了一条到达路径；它与 C878 同一类，挡它的是用户那条定案，不是这一批的代码。推的，我没把标签 0 的文件变长到 2⁴⁰ 去量（会撞内存上限）。
- 从没被看过的地方：除 T1 那一处，`inodes_with_an_extent_upper_leaf_entry` 与 `build_extent_tree` 共用同一条判法，两处一致（它们一致地丢掉 size 0 那一条，所以 T1 不是两处不一致，是共同前提错了）。

## T6　测试临时目录守卫：没打中（只读码）

读的是 diff 第 905–1180 行那 18 个 hunk 与冻结副本 `crates/singlefs-core/src/block_device.rs` 测试模块里命中的 22 行（含函数与结构体自己的定义那 3 行，`grep -c` 数的；`grep -n 'temporary_image_path(\|TemporaryPathRemovedOnDrop('`）。守卫把原来那几条测试末尾的 `std::fs::remove_file(&path).expect("清理镜像")` 换成离开作用域时删，被测路径一处没动；`dirty_image_for_discards` 的六处调用都绑成 `_removed`（有名字的绑定活到作用域末尾，不是 `_` 当场丢）；「撞上同名文件一律报错」那一条在守卫之前自己先删、再看重开报什么，守卫后来删时文件不在，`NotFound` 直接返回。删掉的那几句 `.expect("清理镜像")` 顺带核过「文件还在」，这一核没了，但它不是被测行为。没跑：核心层 lib 测试要编整个 `singlefs-core` 的测试目标，这一格只读码。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| T1 | 标签 0 镜像上：截到 100、N = S、变长到 3 个多净荷、变长后覆盖写、顺序写两单元后截到 10 再截到 0、截到 100 后回退到标签 0 那一版、截到 0 后再变长再截到 0、建快照、unlink 第一个文件、截到 0 之后的 12 种后续 | 1 张起点镜像（size 3000）；size 取 0、10、100、3000、5000、7000、97919（3 个净荷加 17）；后续步数 1–5 |
| T2 | 一盘坏一个单元、两盘各坏一个单元、一盘坏 12 个单元；隔离后重挂、40 次覆盖写 | 文件 1 或 2 个单元；隔离 2、4、20 份 |
| T3 | 进程内回退、重挂之后回退、回退到不引用的再到引用的、回退目标已转出根环 | 回退 1 或 2 次；40 次覆盖写之后 1 次 |
| T4 | 跨度段手改成 0xC002（两盘、一盘）、0xC000、0x4000、0x4003、只一块盘带记号、两盘漏记号、活单元带记号，各过可写挂载、只读挂载、读分配记录树、冷恢复与池级 checker | 一张合法隔离镜像上的 8 种改法 |
| T5 | HEAD 读者读带记号与不带记号各一张 | 2 张镜像、6 个读者 |
| T6 | 读码 | 18 个 hunk、22 处命中 |

## 取样、规模与挂钟

- 盘全是内存稀疏盘（`SparseBlockDevice`）；mkfs、取号、暖机、新池新建文件只跑一次（`OnceLock`），标签 0 起点也只手造一次，之后每段历史从内存那一份拷。T5 为了把镜像交给另一版代码，把两张镜像写过的扇区落成草稿目录里两份各 1092540 字节的文件（不是整盘镜像），交接完删。
- 没用崩溃点枚举：这一轮的反例都是不带崩溃的用户操作序列，崩溃状态数 0、全量 0 个状态（不靠近第 3b 步约 10⁶ / 10⁷ 的上界）；T1 的后果「重启之后可写挂不上」是在每一步写完之后整份重挂量的，等于「在那次发布之后崩溃」那一个状态，崩在发布中途的状态没枚举。
- 估时：先单跑一格（`q_baseline_images_judged_by_the_pool_checker`，挂钟 2.5 秒，测试本身 0.04 秒），全量 16 格 `finished in 1.08s`，不用缩。编译每份拷贝约一分半（`Finished release … in 1m 27s`、`1m 28s`）。
- 本机负载：开跑前 `ps` 看到别的会话的 checker 档 `cargo test` 与一批 rustc；资源提醒最紧时写「本机：放不下：差内存 4.0 GiB」「结论：可以跑：在第二台」，我仍在本机加 `nice -n 19` 跑（每次峰值约 125 MiB，提醒里「历史：内存峰值 125 MiB」），没上第二台。

## 这条腿自己的限度

- T1 的起点是手造镜像（与这一批自带测试同一种造法），带一条已分配无引用的泄漏；拒挂与泄漏无关（拒的是 extent 判），但没造一张不带泄漏的起点去排除。
- T1 只扫了 12 种后续；「重写第一片上段叶」的别的触发（删除续做、整理搬上段叶、回退那次发布重写它）没造。
- 改法甲只跑了我的用例与这一批自带的 4 个测试目标，没跑 harness 全部、没跑变异、没跑 checker 档。
- T3 没造出整理真起跑、扫描重建碰到带记号记录的形状；「核心层没有别的路径走到释放隔离记录」只在 40 次覆盖写上量过。
- T5 的 HEAD 可写挂载对照本身就拒，那一层分辨不了；GPU 判器那一句没量。
- 「没打中」都只有这一次抽样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不单独支撑结论。

## 要主 agent 跑的命令

1. 复跑（见开头）：仓根起 `bash research/prompts/m4-validate-code-r2-opus-model/tools/rerun.sh --repo … --frozen … --work <新目录>`，用 run_in_background，三组共编三次约五分钟；看三行 `same_as_model_out=yes exit=0`。
2. 我没改 `crates/` 或 `scripts/` 下的 `.rs`（原型只在 `research/prompts/…-model/prototype/` 与草稿目录的拷贝里），命名检查两条不适用。

## 没做什么

- 不判正推、辩方的格；K5 那一句 kb 写回改没改没核（辩方格）。
- 没跑重型测试、没跑门禁、没起 GPU；副本上量的数不算入库装置上的数。
- 改法甲、乙、丙都是我提的，甲只在我的模型上量过、被攻过零轮，乙、丙是推的。
- 没整条跑 `rerun.sh`（前台时限内三组编不完），`out/*.txt` 是按它里面同一串命令逐组跑的。

## 结论表

| 格 | 结论 | 什么现象会推翻 |
|---|---|---|
| T1 | 打中：截到 0 之后重写第一片上段叶的发布丢掉标签 0 那一条，写出可写重挂、冷走读、冷恢复都拒的一版 | 同一历史在冻结副本上重挂收下 |
| T2 | 没打中 | 找到一次隔离只置了一块盘、或分配代变了、或重挂后记号没了的历史 |
| T3 | 没打中 | 找到回退清了目标不引用的记号、或漏清目标引用的记号的历史；或整理、扫描重建走到释放隔离记录 |
| T4 | 没打中 | 核心层里找到一处自己切跨度段而不去掉 0x4000 的读，或 0xC000 被哪个核心层读者收下 |
| T5 | 条款措辞打中一处：旧读者的只读挂载过得去；其余几层与 D19 的说法相符 | HEAD 的只读挂载拒带记号的池 |
| T6 | 没打中（只读码） | 守卫改了某条测试判的东西，或某个守卫当场被丢掉 |

## 草稿目录清理

交回前删了草稿目录 `~/.local/share/singlefs/work/m4-validate-code-r2-opus/` 里自己建的拷贝与编译目录（删之前 `du -sh`）：`target` 719M、`target-fix` 1.5G、`target-head` 700M、`tree` 38M、`tree-fix` 38M、`head-tree` 35M、`images` 2.1M、`images2` 2.1M、`tmp` 4.0K。留下的只有编译与运行日志（`out/`、`build*.txt`，共 368K）。
