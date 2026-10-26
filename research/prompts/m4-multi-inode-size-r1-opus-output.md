# m4-multi-inode-size-r1 攻方腿（Opus）报告（2026-10-26）

<!-- doc-lint:not-numbers S2 S3 S4 S5 S6 S7 H1 H2 H3 H4 H5 H6 K1 K3 K5 K6 K9 R2 R3 -->

下文「冻结副本」指 `/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/`（只读，引代码都写它里面的路径与行号）。副本上量的数都在我的副本里：冻结副本拷进草稿目录，只删了 `publish_size_change` 里「不是第一个文件而 N ≠ S 就拒」那一支（补丁见模型目录 `src/publish-size-change-refusal-dropped.patch`），别处一行不改；这个副本就是「只放开边界、别处照今天的实现」那一种放开。副本上的数不是入库装置上的数。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-multi-inode-size-r1-opus-model/`：

```
d5f784f9dbe656ccb1f0e29f1ebac049fe8141ca7924bb7a49844ea80944eb3c  src/attack_multi_inode_size_probe.rs
bc5b2bee61054d53673809c24816d6bdcbf4bdf15c4998330da2427155e894e5  src/publish-size-change-refusal-dropped.patch
7b396a28c0f20c620366c064c66bae209b3e26f9995665b8ff1746183b3408e4  out/probe.txt
b7b5f4156772838c814c9351924b5787126aa432f843b2e713686b1a58042cee  tools/rerun.sh
```

复跑（工作目录随意，`--work` 给一个新目录）：

```
bash scripts/hosts/run-with-memory-cap.sh 3G bash research/prompts/m4-multi-inode-size-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work <新目录>
```

我在交回前用同一条命令（`--work` 取草稿目录下的 `rerun-work`）跑过一次，退 0，末两行原样：

```
patching file crates/singlefs-core/src/transaction.rs
name=rerun_compare same_as_model_out=yes lines=73
```

`tools/rerun.sh` 是从 `scripts/agents/three-way-rerun-template.sh` 拷的，只改了 `rerun_steps`；模板自带的 `--selftest` 在假仓上跑我的 `rerun_steps`（要拷假仓里没有的 `.cargo`），所以自证那一格红，这是改了 `rerun_steps` 的必然结果，不是脚本坏。

探针的取样：一个起点（mkfs、取号、暖机、新池新建文件 3000 字节、建 inode 2 与 3、inode 3 写 100 字节）与一个三单元起点（inode 3 写 3 个满净荷），各只建一次池，在内存稀疏盘（`SparseBlockDevice`）上，每段历史从内存里那一份拷。历史 7 段（H1–H6 加手造的 H1 对照），每段之后在同一份镜像上跑四个读者（挂载态读者 `open_file` + `read_at`、`rebuild_version`、`recover`、`mount_writable`）与池级 checker `check_pool_image`（109 条判据）。没有调崩溃枚举，没有崩溃状态（这一轮的格都是单镜像判法；见「这条腿自己的限度」）。全量跑一趟 1.74 秒，不需要缩样。

## 各格判定一览

| 格 | 判定 | 一句话 | 证据种类 |
|---|---|---|---|
| S2 别的文件的洞 | **打中**（打中不分辨臂，另立一笔账） | 「别的 inode size > 0、上段没有条目」这一份镜像，今天挂载态读者拒、`rebuild_version` 与 `mount_writable` 收、CPU 池级 checker 109 条全不红、模型胶水判解不开；GPU 两个内核照代码读与 CPU 同一判（推的）。写标签 0 与不写两条臂各有一半组件在这份镜像上判错 | 副本上量的（`out/probe.txt` 第 8–20 行）；GPU 是读码 |
| S3 截断续做与计划 | **打中**（实现缺口，不分辨候选） | 只删拒绝那一支，写者对别的文件 5 段历史全「发成」，其中 4 段 checker 红 I-9.38 / I-4.6：别的文件的单元数在 `publish_size_change` 里写死 0，尾单元不重写、截断字段不写、续做认不出别的号；K3 的 extent 计划带 `inode` 参数却只换第一个文件的下段角色 | 副本上量的（第 21–60 行）；计划那一条是读码 |
| S4 截断条目至多一条 | **没打中**两条条目；**打中**一条副作用 | 回退那一步先删现行的全部截断条目、再放 R_old 的第一条，R_old 自己读进来就至多一条：造不出两条。副作用：留 K1 而某个别的号的条目续不下去时，会话「先走空」让全池每一次用户改动都停住 | 读码；副作用在副本上量的（第 42、56 行） |
| S5 对拍对象 | **打中** | 今天的模型胶水对别的文件要求「单元接起来恰好等于 size」，截断中间态与任何洞都判解不开；模型只比第一个文件那条记录的截断字段 | 副本上量的胶水输出（第 15、40、54 行）；字段比较范围是读码 |
| S6 拒绝边界的用例与变异 | **打中** | 把钉拒绝的用例直接翻成「发成就算过」零判别力：5 段坏历史 publish 全部 Ok，其中 H1 连池级 checker 都绿；钉这条边界的变异第 4389 行，它的变异体就是我的副本，换上的正例要在这五段上红；按 `is_the_first_file` grep 会多带进一行 GPU 恢复内核分批读（第 4779 行），与这条边界无关 | 副本上量的；变异表是 grep |
| S7 只认第一个文件的清单 | **打中**（判法漏） | `grep -rn FIRST_INODE_NUMBER crates/singlefs-core/src` 的 61 行里 truncate.rs 的 4 行全在测试模块，K3 计划本体一行不中；mount.rs、mounted_session.rs、mounted_read.rs、admission.rs 零行；结构上只认第一个文件的写法（`TruncationBatch` 不带号、`TransactionUnit::Data(`、`ExtentLowerNode(`、冷恢复对别的文件写死「尾部没有洞」）不带这个常量 | 命令数的；读码 |

## S2　别的文件的洞怎么表示

### 打中：「size > 0、上段没有条目」这一份镜像，读者与 checker 各判各的

**历史 H1**（副本，只删了拒绝那一支）：起点镜像上 inode 2 是 `publish_new_inodes` 建的（size 0、上段没有条目），会话外直接 `publish_size_change(inode 2, 5000)`。写者发成：别的文件的单元数在 `publish_size_change` 里写死 0（「let data_units_existing = if is_the_first_file {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:10494`），else 分支是 0），`size_change_of_the_first_file` 对 inode ≠ 1 交 None（「} if record.inode == FIRST_INODE_NUMBER => record.size,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:8933`）），extent 树整棵照抄，于是只改了记录的 size。这正是「不写标签 0」那一臂会写出的形状。**H1 手造**：不经写者，起点镜像上 inode 2 那条记录的 size 直接改成 5000、重封叶、校验和补到根（`common_deletion_in_progress::change_the_inode_record`），两份镜像的判法逐条相同，说明这不是副本写者造出来的。

原样输出（`research/prompts/m4-multi-inode-size-r1-opus-model/out/probe.txt` 第 8–20 行）：

```
name=h1 publish_size_change(inode 2, 5000) 发成
name=h1 inode2_tag=None
name=h1 reader=mounted_read inode=2 打开拒：ExtentRecordCountDoesNotMatchTheFileSize { inode: InodeNumber(2), extent_records: 0, data_units_implied_by_the_file_size: 1, file_size_in_bytes: 5000 }
name=h1 reader=rebuild_version inode=2 收：inode 2 的 size Some(5000)、第一个文件之外有单元的文件 [(3, 1)]
name=h1 reader=recover 读回第一个文件 3000 字节
name=h1 reader=mount_writable 收
name=h1 checker=pool judged=109 violated=0 
name=h1 model_glue=Some(Undecodable { what: "0 个数据单元接起来 0 字节，记录的 size 是 5000" })
name=h1-crafted reader=mounted_read inode=2 打开拒：ExtentRecordCountDoesNotMatchTheFileSize { inode: InodeNumber(2), extent_records: 0, data_units_implied_by_the_file_size: 1, file_size_in_bytes: 5000 }
name=h1-crafted reader=rebuild_version inode=2 收：inode 2 的 size Some(5000)、第一个文件之外有单元的文件 [(3, 1)]
name=h1-crafted reader=recover 读回第一个文件 3000 字节
name=h1-crafted reader=mount_writable 收
name=h1-crafted checker=pool judged=109 violated=0 
```

（`recover` 只走第一个文件，这一行与 inode 2 无关，列出来是为了说明它没拦。）

每一处判法指到许可它的那一句：

- 挂载态读者拒：没有条目映到「None => DataUnitsOfTheInode::NoneWritten,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_read.rs:809`），size ≠ 0 时「DataUnitsOfTheInode::NoneWritten => refused,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/write_request_split.rs:106`）。
- `rebuild_version` 与 `mount_writable` 收：冷恢复只对 extent 树里有条目的文件核记录（「.zip(&data_unit_contents.files_other_than_the_first_in_file_order)」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/recovery.rs:3845`）那一圈），有记录、没有条目的别的号不进这一圈；第一个文件同一形是拒的（同一函数 `inode_record_of(FIRST_INODE_NUMBER)` 那一支）。
- CPU 池级 checker 不红：I-4.6 的单元数判把「没有条目」与「标签 0」并成一支、不看 size（「(None | Some(DataUnitsDeclaredByTheUpperLeafEntry::NoDataUnit), _) => (」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker/src/walk.rs:13057`），说明串「"上段叶条目标签 0 或没有上段叶条目：一个单元都没有".to_string(),」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker/src/walk.rs:13059`））。
- GPU 检查内核同一判（推的，没起 GPU）：「var count_matches_the_declaration = data_unit_count == 0u;」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3302`），只有声明是「之后没有洞」「之后到 size 是洞」两种时才改判，没有条目照「零个单元就成立」。
- GPU 恢复内核收（推的）：建上一版那一遍只按 extent 树里的文件逐行走（「for (var file = 0u; file < files_of_the_extent_tree; file = file + 1u) {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:412`）），与 `rebuild_version` 同形。

**H6 手造**（「别的文件也写标签 0」那一臂的目标形）：起点镜像上 inode 3 那一条上段叶条目改成标签 0、载荷清零，映射条目拿掉，记录 size 改成 5000（对照：size 留 0）。原样（第 63–66 行）：

```
name=h6-tag-zero-size-5000 reader=mounted_read inode=3 收：size 5000、读出 5000 字节、非零 0 个
name=h6-tag-zero-size-5000 reader=rebuild_version inode=3 拒：Walk(UnitMalformed { what: "extent 树里第一个文件之外的文件没有单元、单元有洞或尾部有洞（按 inode 号写数据写不出来，第一版不支持）" })
name=h6-tag-zero-size-5000 reader=recover 拒：UnitMalformed { what: "extent 树里第一个文件之外的文件没有单元、单元有洞或尾部有洞（按 inode 号写数据写不出来，第一版不支持）" }
name=h6-tag-zero-size-5000 reader=mount_writable 拒：Recovery(UnitMalformed { what: "extent 树里第一个文件之外的文件没有单元、单元有洞或尾部有洞（按 inode 号写数据写不出来，第一版不支持）" })
```

拒的那一句是「if data_pointers_of_the_file.is_empty()」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/recovery.rs:5939`），背景材料已点名。H6 上 checker 红的 4 条（I-3.1、I-3.11、I-3.15、I-1.13）全是我手造时把那个单元留在分配记录与盘上造出来的（I-1.13 的明细点名「inode 3、偏移 0」那个被扫描重建认回来的单元），不拿它判 checker 对标签 0 的判法。

### 四句

1. **分不分辨臂**：不分辨。同一份镜像（size > 0、没有条目）在「不写标签 0」臂下是合法镜像、挂载态读者与模型胶水在它上面红；在「写标签 0」臂下是非法镜像（标签 0 那一条丢了），CPU checker、GPU 检查内核、`rebuild_version`、`mount_writable`、GPU 恢复内核在它上面绿。两臂各有一半组件要改，病根是它们共用的前提：今天的四处读法对「有记录、没有条目、size > 0」各判各的。照证据纪律「打中不分辨臂」，这一格不拿来判臂，另记一笔账：不管选哪一臂，挂载态读者、`rebuild_version` / 冷恢复、CPU I-4.6 的「没有条目」那一支、GPU 两个内核要对这一形给同一个判法，并配一个必红世界。
2. **被判的系统看不看得到判别它的东西**：看得到。记录的 size 与上段有没有这一条都在盘上，单镜像就判得了。
3. **满足的是判据字面的哪一个分句**：正文 S2「写或不写，三处读者……、池级 checker 与 GPU 恢复内核各怎么判」与分工表攻方那一行「在合法镜像上判红，或在非法镜像上判绿」——按臂分开，两个分句各中一次。
4. **跑前条款给的改法在打中的那几格上还中不中**：正文没写跑前改法。按两臂各自推（推的，没实现）：「写标签 0」只改写者与冷恢复那一句，CPU I-4.6 与 GPU 检查内核的「没有条目」一支不动的话，这份镜像照样绿，所以这一臂要连 checker 一起改成「没有条目 ⇒ size = 0」；「不写」只改挂载态读者的话，冷恢复与 checker 本来就收，但「丢了整条上段条目、单元也被放掉」的坏镜像从此与合法的整文件洞逐字节无法区分，I-4.6 判不出，只有单元还留在盘上时 CPU 的 I-1.13 认得出（GPU 侧 I-1.13 内核不置位，截断判决第六节 S4）。这一条是我推的，只在我的副本上量过上面那几行、被攻过零轮。

**推翻条件**：在入库装置上同一份镜像（size > 0、没有条目）挂载态读者收下，或 CPU I-4.6 / `rebuild_version` 把它判红，这一格就不成立。

### 「只红自己」那一问

改成「没有条目 ⇒ size = 0」会不会让既有镜像多红：我在两个起点与 7 段历史上没有见到 size > 0 而没有条目的合法镜像（起点 `name=start checker=pool judged=109 violated=0`，第 7 行）；今天每条写路径至少写一个单元，unlink 的各批保留至少一个单元、最后一批同一次发布删记录与条目（D8 已定项 16 第 2 条）。这是推的，没有扫既有用例的全部镜像。

## S3　截断续做与截断计划对别的文件

### 打中：只放开边界，写者照第一个文件那条路写别的文件，每一段都「发成」而镜像是坏的

同一个副本、同一起点，四段历史（H2 inode 3 内联 100 字节变长到 5000；H3 变长跨单元到 2 个净荷 + 7；H4 截到 50 再走空；H5 三个满单元截到 100 再走空），原样（`out/probe.txt` 第 21–60 行里的这几行）：

```
name=h2 publish_size_change(inode 3, 5000) 发成
name=h2 checker=pool judged=109 violated=1 I-9.38〔根（实例 1、txg 6）：inode 3 偏移 0 的数据单元声明长度 100，记录 size 5000 要 min(size − 起点, 净荷容量 32634) = 5000〕
name=h3 publish_size_change(inode 3, 65275) 发成
name=h3 inode3_tag=Some("InlineDataUnit(DataPointer { head: Point")
name=h4 publish_size_change(inode 3, 50) 发成
name=h4-intent checker=pool judged=109 violated=1 I-9.38〔根（实例 1、txg 6）：inode 3 偏移 0 的数据单元声明长度 100，记录 size 50 要 min(size − 起点, 净荷容量 32634) = 50〕
name=h4 走空：发成 0 步，停在 Err("TruncationRefused(TruncationIntentDoesNotNameTheLiveRecord { intent: TruncationIntent { inode: InodeNumber(3), object_birth: CheckpointTxg(4), target_size_in_bytes: 50 } })")
name=h5 publish_size_change(inode 3, 100) 发成
name=h5-intent checker=pool judged=109 violated=1 I-9.38〔根（实例 1、txg 6）：inode 3 偏移 0 的数据单元声明长度 32634，记录 size 100 要 min(size − 起点, 净荷容量 32634) = 100〕
name=h5-intent inode3_truncation_field=Some(InodeRecordTruncationField { truncation_txg: CheckpointTxg(0), truncated_size_in_bytes: 0 })
name=h5 走空：发成 0 步，停在 Err("TruncationRefused(TruncationIntentDoesNotNameTheLiveRecord { intent: TruncationIntent { inode: InodeNumber(3), object_birth: CheckpointTxg(4), target_size_in_bytes: 100 } })")
```

H3 那一版 checker 另红 I-4.6（单元数 1、size 要 3、标签仍是内联）。H2–H5 每一版挂载态读者、`rebuild_version`、`mount_writable` 都拒（第 22–60 行）。

许可每一步的那一句：

- 尾单元不重写、截断字段不写：别的文件的单元数在 `publish_size_change` 里写死 0（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:10494` 那一句的 else 分支），交给 `tail_data_unit_rewrite_of_a_truncation` / `tail_data_unit_rewrite_of_an_extension` 时「单元不存在」；交给截断字段那一判时「if data_units_existing_before_the_intent > data_units_a_size_needs(target_size_in_bytes) {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/truncate.rs:122`）恒假，于是 H5 截掉两个单元的意图那次 T 仍是 0（上面第 55 行）。这一形若续做走得下去，就是截断判决第二轮 R2 要防的「截到边界再变长、扫描重建认回放掉的单元」（推的：续做在今天的副本上走不下去，没量到认回那一步）。
- 续做认不出别的号：「let names_the_live_first_file = intent.inode.0 == FIRST_INODE_NUMBER」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:10604`），之后取单元数取的是第一个文件的（「let data_units_existing = u64::try_from(previous.data_pointers.len()).expect("单元数");」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:10616`）），批本身不带号（「TruncationBatch {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:5826`））。
- 标签不换：`size_change_of_the_first_file` 对别的号交 None（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:8933`），extent 树整棵照抄，H3 跨单元变长之后仍是内联标签 2，没有标签 3、4 可写——`FileOfAnInodeOtherThanTheFirst` 没有尾部那一项（「pub struct FileOfAnInodeOtherThanTheFirst {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:3673`））。

### 「照 unlink 那一份直接搬」够不够：K3 计划带着 `inode` 参数，换下的却是第一个文件的下段（读码，推的）

`plan_the_extent_tree_after_a_size_change(previous, inode, change)` 收一个 `inode`，上段那一串按它取（「let path_of_the_file = upper_path_of_inode(inode, upper_root_level);」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/truncate.rs:275`）），下段却是 `ExtentTreeVersion` 里第一个文件那一份：照抄的前提断言 `previous.lower_pointer_of(position)`（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/truncate.rs:259`），换下的角色写死成第一个文件的下段角色（「.map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/truncate.rs:297`））。把别的号直接传进去：别的文件自己的下段不在 `previous.lower_nodes` 里，第一个文件的下段被当成这个文件的换下（第一个文件有下段时）或断言失败（没有时）。unlink 那一份能搬的只有「按 inode 取剩下几个单元」（`next_unlink_step_after` 那一段）与别的文件的下段截法 `plan_the_lower_segment_after_a_deletion_batch`；后者只认「被截那一片叶」，`FileOtherThanTheFirstOrigin` 只有照抄、整份重写、被一批截三种来源（「CutByADeletionBatch」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:7210`）），没有「这次只重写单元 k、罩着它的叶到根恒重写」那一种——K9 那条叶边界（留下 144、288 个单元时罩着单元 k 的叶重写）搬到别的文件上，要新加一种来源；照搬 unlink 的批计划在截断各批上是对的（意图与各批是不同的发布，单元 k 在意图那次已经连叶重写），漏的只会在意图那次。这一段是读码推的，没跑。

### K1 回退取 R_old 的截断条目对别的号怎么认

回退那一步与号无关：先删现行那一版全部截断条目（「entries.retain(|entry| entry.operation.kind() != LoggedOperationKind::Truncation);」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:6048`）），再放 R_old 的那一条（「.find(|entry| entry.operation.kind() == LoggedOperationKind::Truncation),」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mount.rs:9938`）），inode 与 extent 树同取 R_old 的，所以条目点名的 (号, 对象出生代) 在新那一版里有同一条记录。要改的只在续做一侧：`publish_truncation_step` 核「条目点名的是活记录」时拿的是 `previous.inode_record`（第一个文件那一条，上面 10604 那一句），要改成按 `intent.inode` 在 inode 叶容器里找。

### 四句

1. **分不分辨臂**：S3 没有两条候选臂可分；打中的是「只删拒绝那一支」这一种放开，它说明放开的最小改动面不止拒绝那一行。
2. **看不看得到**：看得到，写者手里有上一版这个文件的单元数（`file_other_than_the_first(inode)`）。
3. **满足的是哪一个分句**：正文 S3「照 unlink 已泛化的按 inode 取单元那一份能不能直接搬」与「续做入口按 inode 认领时」；攻方立场「放开边界之后……在非法镜像上」的另一面：写者在放开之后写出非法镜像而交回成功。
4. **改法还中不中**：没有跑前改法。我推的最小改动面（零轮）：单元数按号取（三处：`publish_size_change` 的单元数、截断字段那一判、续做的单元数）、批带号、计划收这个文件自己的下段并加「重写单元 k」那一种来源、续做按号找记录、尾部标签进 `FileOfAnInodeOtherThanTheFirst`。每一处只在我的副本上看到它在上面那几行里不成立，被攻过零轮。

**推翻条件**：入库装置上只删拒绝那一支的放开，H2–H5 任一段 checker 全绿且四个读者都收。

## S4　「全池活截断条目至多一条」（K1）留不留

### 没打中：回退合不出两条截断条目

试的形状（读码逐步走，没有在副本上跑回退，因为副本上别的号的截断续做走不下去，见 S3）：

1. 现行那一版截 inode 2 截到一半（条目 A 挂着），回退到截 inode 3 截到一半的 R_old（条目 B 挂着）。回退那一步先删现行那一版的全部截断条目（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:6048` 那一句），再放 R_old 的第一条（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mount.rs:9938` 那一句）：新那一版只有 B。
2. R_old 自己读进来就至多一条：读者对多于一条拒成类型化错误（「if truncation_intents > 1 {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/logged_operations.rs:401`））。
3. 回退之后在 B 挂着时再发一条：会话里每一次用户改动先走空，发截断意图的入口断言三样队列都空（「"发截断意图之前三样队列（deleted_inodes、销毁中的快照、截断条目）都已走空（判决 m4-trunc-r1 K1）"」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:10546`））。
4. 回退目标里有 B、现行那一版有整理条目：整理条目随现行那一版留着，截断条目只有 B——两条不同种类，不违反 K1。

于是在「每一版都由上一版经这几步得到」的归纳下，任一版至多一条：意图入口要上一版没有，回退那一步整份换成 R_old 的、R_old 至多一条，各批与最后一批只删不加。取样范围：上面四种回退形状加「回退之后先做哪一种用户改动」放开（写、建 inode、unlink、截断、空发布：每一种在 `mounted_session` 里都先走空，读码），没有找到第五条加条目的路。只抽了一次样，照规则不当「没打中」引。

### 打中：留 K1 时，一个续不下去的别的号的条目让全池停写

副本上 H4、H5（只删拒绝那一支）意图那次签了点名 inode 3 的条目，之后走空一步都发不出：

```
name=h4 走空：发成 0 步，停在 Err("TruncationRefused(TruncationIntentDoesNotNameTheLiveRecord { intent: TruncationIntent { inode: InodeNumber(3), object_birth: CheckpointTxg(4), target_size_in_bytes: 50 } })")
```

（`out/probe.txt` 第 42 行，H5 同形在第 56 行。）会话「先走空再做自己」那一条让之后对任何 inode 的任何用户改动都先撞这一步。这一形在正确实现下不出现，打中的是：K1 加「先走空」把「某一个号的续做坏了」放大成「整池不能写」，所以续做的按号认领（S3）是 K1 能留的前提，不是可以放到一·3 的事。

### 放开成「每个 inode 至多一条」要改的地方（读码，推的）

- 回退那一步只取 R_old 的第一条（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mount.rs:9938` 的 `.find`）：R_old 有两条时丢一条，被丢的那个号记录已是 size = N、size 之外的单元还挂着，I-4.10 红、单元没人放。
- 读者拒多于一条（`logged_operations.rs:401`）；内存里取活条目的那一份断言至多一条（「"全池活截断意图至多一条（判决 m4-trunc-r1 K1）：签发入口断言上一版没有截断条目"」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/logged_operations.rs:454`）），多一条就 panic，不是类型化错误。
- checker 的「全池至多一条」那一判（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker/src/walk.rs:18385` 那一段）与 GPU 判器同一判、续做的次序（几条条目谁先）、发意图入口的断言。

### 四句

1. 分不分辨臂：「留」与「放开」两臂都要求续做按号认领，停写那一条不分辨臂；回退那一条只打「放开」臂。
2. 看不看得到：看得到，条目带号与对象出生代。
3. 分句：正文 S4「管理员回退取 R_old 的截断条目时能不能合出两条」——没合出；「放开的话哪几处读者与 checker 要改」——上面一节。
4. 改法：正文没给跑前改法。

## S5　对拍对象

### 打中：今天的模型对拍对别的文件只认「整份写、没有洞、没有截断中间态」，崩溃读回只认第一个文件

1. **胶水把合法的截断中间态与洞判成解不开**。别的文件读回的内容由胶水把这一版里这个文件的全部单元接起来、要求恰好等于 size（「if u64::try_from(content_read_back.len()).expect("内容长度装得进 u64") != size_in_bytes {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/model_comparison.rs:262`）），单元按角色 `DataOfAFileOtherThanTheFirst` 全收、不看 size 与标签。副本上的原样（第 15、54 行）：

```
name=h1 model_glue=Some(Undecodable { what: "0 个数据单元接起来 0 字节，记录的 size 是 5000" })
name=h5-intent model_glue=Some("Undecodable { what: \"3 个数据单元接起来 97902 字节，记录的 size 是 100\" }")
```

H5 那一版本身是坏的（尾单元没重写），但同一句也会判一个正确的截断中间态（单元 0 重写到 100 字节、单元 1、2 还挂着）：接起来 100 + 2 × 32634 字节 ≠ 100，解不开（推的，没造出正确实现的那一版）。H1 在「不写标签 0」臂下是合法的整文件洞，同样解不开。
2. **截断字段只比第一个文件那条记录**：观测那一项只有一格（「pub first_file_record_truncation_field: Option<ObservedTruncationField>,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/model.rs:2510`））。S3 里 H5 那种「截掉两个单元而 T 仍 0」（第 55 行）在别的文件上，模型对拍看不见；能看见的只有扫描重建一侧（CPU 的 I-1.13 与读法 Z 第四条），GPU 侧 I-1.13 内核不置位。
3. **崩溃读回的允许集合只装一份内容**：模型判崩溃读回的那一份按根只记一份内容（「committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/model.rs:8139`），④「读回的内容要与模型记的那一版逐字节相同，那一版树表 0 条时只许报没有文件。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/model.rs:8132`）），喂给它的是 `recover` 读回的第一个文件（第 12 行「name=h1 reader=recover 读回第一个文件 3000 字节」那一类）。RefFS 翻译层对按 inode 号写停下（「"RefFS 第一版只有第一个文件"」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/reference_filesystem_translation.rs:999`））。所以「只对模型时崩溃后读回的允许集合由谁给」今天的答案是：别的文件没有人给。

### 四句

1. 分不分辨候选：分辨。「只对 harness 模型」那一候选要先改三样（胶水按标签与截断条目读、截断字段按号比、崩溃读回按号给允许集合）才判得了这一行的验收；「RefFS 扩成多文件」那一候选不省掉第 2、3 条里模型那一半，但给崩溃读回一份与模型不共用代码的允许集合。
2. 看不看得到：看得到，胶水手里有这一版的记录、单元与 logged_ops。
3. 分句：正文 S5「只对 harness 模型够不够」「只对模型时崩溃后读回的允许集合由谁给」。
4. 改法：没有跑前改法；第 1 条我只在副本上看到胶水的输出，正确实现下的那一句是推的。

## S6　现有拒绝边界的用例与变异

### 打中：把钉拒绝的用例直接翻成「发成就算过」，判别力是零

钉边界的用例 `a_size_change_of_an_inode_other_than_the_first_file_is_refused_unless_the_size_stays`（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/tests/truncation_intent_batches_and_final_batch.rs:257`）断言 inode 2 改成 10 字节「expect_err」。最省事的翻法是改成 `expect`、断言发成。副本上只删拒绝那一支，H1–H5 五段 `publish_size_change` 全部「发成」（`out/probe.txt` 第 8、21、27、34、48 行），而五段里 H2–H5 四段的镜像 checker 红、四个读者里三个拒，H1 那一段连池级 checker 109 条都绿、只有挂载态读者拒（S2 那一节的原样）。所以：

- 翻成正例时，每一段至少要过「挂载态读者读回逐字节等于期望 + `rebuild_version` / 可写重挂收下 + 池级 checker 零红」三样；只断言发成、或只跑 checker，H1 那一形都放过（只跑 checker 放过 H1，量的；只断言发成五段全放过，量的）。
- 判别力用这五段当必红世界最直接：它们正是「只删拒绝、别处不改」那一种错误放开写出来的镜像，正例用例在这五段上必须红。这一条是我提的，被攻过零轮。

### 变异：钉这条边界的那一行，它的变异体就是我的副本

冻结副本 `crates/mutations.tsv` 里点名这条边界的行（按第 3 列原文与拒绝成员名 grep，`grep -n 'SizeChangeOfAnInodeOtherThanTheFirst\|(_, true) => {}'`）有四行：第 4389 行（`crates/singlefs-core/src/transaction.rs`：把 `(_, true) => {}` 改成 `(_, true) | (_, false) => {}`，即「别的 inode 改 size 不拒」，判它的是上面那条用例）、第 4456 行（模型里溢出先于这一拒）、第 4462 行（胶水把这一拒映射成说不清的理由）、第 4469 行（执行器拒绝的成员名）。按 `is_the_first_file` grep 另命中第 4779 行，那一行锚在 GPU 恢复内核分批读数据指针（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl`），与这条边界无关，不该进「改成正例」那一批。

第 4389 行的变异体与我副本里的补丁做的是同一件事（让别的 inode 的变长、截断照第一个文件那条路走下去）。所以拒绝删掉之后这一行要换成什么，答案就在副本上量的那几行里：换成的正例用例必须在这个变异体上红——H1–H5 五段，断言「四个读者收、读回等于期望、checker 零红」，在变异体上 H1 红在挂载态读者、H2–H5 红在读者与 checker（量的）。只断言发成的正例在这个变异体上全绿，判别力是零。第 4456、4462、4469 三行锚在模型与胶水里拒绝成员的那几处，成员删掉之后原文找不到，要换锚到新的正例分支（例如胶水按标签读洞那一支），不是翻成正例。

四句：不分辨候选（S6 是改法题）；看得到；满足正文 S6「钉它的用例与 3 行变异改成正例的哪种写法才不丢判别力」；没有跑前改法。

## S7　射程之外还剩什么只认第一个文件

### 打中：「grep FIRST_INODE_NUMBER」这个判法漏掉会让验收不成立的几处

命令与原样输出（冻结副本 `crates/singlefs-core/src` 下跑）：

```
$ grep -rn FIRST_INODE_NUMBER . | wc -l
61
$ for f in *.rs; do n=$(grep -c FIRST_INODE_NUMBER "$f"); [ "$n" != 0 ] && echo "$f 有 $n 行"; done
recovery.rs 有 5 行
transaction.rs 有 41 行
truncate.rs 有 4 行
unlink.rs 有 11 行
$ for f in mount.rs mounted_session.rs mounted_read.rs admission.rs; do echo "$f 有 $(grep -c FIRST_INODE_NUMBER "$f") 行"; done
mount.rs 有 0 行
mounted_session.rs 有 0 行
mounted_read.rs 有 0 行
admission.rs 有 0 行
$ grep -n FIRST_INODE_NUMBER truncate.rs; grep -n "^mod tests" truncate.rs
320:    use crate::transaction::FIRST_INODE_NUMBER;
438:            FIRST_INODE_NUMBER,
608:            FIRST_INODE_NUMBER,
657:            FIRST_INODE_NUMBER,
316:mod tests {
$ 不带常量、结构上只认第一个文件的写法各几行（去掉注释行）
TruncationBatch {	14
TransactionUnit::Data(	51
TransactionUnit::ExtentLowerNode(	47
previous\.data_pointers	6
TailOfAFile::NoHoleAfterTheLastDataUnit	18
```

truncate.rs 的 4 行全在测试模块里（第 316 行起），K3 的计划本体（`plan_the_extent_tree_after_a_size_change`）一行不中，却是只认第一个文件的（S3 那一节）。按「不改就让验收不成立」逐条，grep 不中而必须一起改的（读码，每条带验收里它挡的那一样）：

| 处 | 为什么只认第一个文件 | 不改挡哪一样验收 |
|---|---|---|
| `truncate.rs:297` 那一句（K3 计划的换下角色） | 下段取 `ExtentTreeVersion` 里第一个文件那一份，换下角色写死 `ExtentLowerNode` | 意图续做（截断的每一次发布） |
| `transaction.rs:5826` 起的 `TruncationBatch` / `TruncationFinalBatch` | 不带 inode 号，`data_units_after_this_publish` 把留下的个数记到第一个文件上 | 意图续做、崩溃恢复 |
| `transaction.rs:10616` 那一句 | 续做取单元数取 `previous.data_pointers`（第一个文件的） | 意图续做 |
| `transaction.rs:10494` 那一句的 else 分支 | 别的文件单元数写死 0：尾单元不重写、截断字段不写（副本上 H2–H5 量的） | 关闭重挂、崩溃后读回（截断字段不写让扫描重建认回放掉的单元，推的） |
| 冷恢复对别的文件写死「尾部没有洞」（「TailOfAFile::NoHoleAfterTheLastDataUnit,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/recovery.rs:3853`）） | 别的文件单元数对 size 按标签 1、2 判，标签 3、4、0 进不来（`recovery.rs:5939` 那一拒之外的第二处） | 关闭重挂、崩溃后读回 |
| `FileOfAnInodeOtherThanTheFirst`（`transaction.rs:3673`） | 没有尾部那一项，写者写不出标签 3、4、0 | 变长、快照共享下的变长 |

grep 之外、`crates/singlefs-core/src` 之外而同样挡验收的（S7 题面只问 core，这几处列出来是因为「崩溃后读回对拍」要它们）：GPU 恢复内核对别的文件不认尾部洞（「let tail_is_a_hole = is_the_first_file && (tag == 0u || tag == 3u || tag == 4u);」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:432`））；模型胶水、截断字段观测与崩溃读回判法（S5 那一节三条）。

可以留给一·3 的：我没有找到一处「不改也不挡这一行验收」而又只认第一个文件、且不是 K-U5（删完第一个文件之后的写拦截，`transaction.rs` 里 `death_txg_of_the_first_file` 那几行）那一族的；K-U5 那一族与 C618 同一题，留给一·3 不挡这一行（推的：这一行的验收不含「删完第一个文件之后再变长别的文件」，若含就要改）。

四句：不分辨候选；看得到；满足正文 S7「给出清单与判法」里「判法」那一半——题面给的判法会漏；没有跑前改法。我给的替代判法（零轮）：在 grep 常量之外，再 grep 四样结构写法（`TransactionUnit::Data(`、`ExtentLowerNode(`、`TruncationBatch`、`previous.data_pointers` / `.inode_record`），并以「只删拒绝那一支的副本上 H1–H5 每一段都红」为会红的判法——清单改完之后这五段的写法应当变成合法镜像、四个读者都收。

## 改法各修哪一格（全部是我提的，只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | H1 | H2–H5 | 标注 |
|---|---|---|---|---|
| 挂载态读者、冷恢复、CPU I-4.6、GPU 两个内核对「有记录、没有条目、size > 0」给同一个判法（选「写标签 0」就都判红，选「不写」就都收） | S2 | 这几处今天各判各的：量过（第 8–20 行）；改完之后的判法：推的 | 不涉及 | 量过一半 |
| 选「写标签 0」：I-4.6 与 GPU 检查内核的「没有条目」那一支加「size = 0」 | S2 | 推的 | 不涉及 | 推的 |
| 单元数按号取（`publish_size_change`、截断字段那一判、续做），批带号，计划收这个文件自己的下段并加「重写单元 k」那一种来源，续做按号找记录，`FileOfAnInodeOtherThanTheFirst` 带尾部 | S3、S7 | 不涉及 | 今天不成立：量过（第 21–60 行）；改完成不成立：推的 | 推的 |
| K1 留，并把「续做按号认领」写成留 K1 的前提 | S4 | 不涉及 | 停写那一形量过（第 42、56 行） | 量过一半 |
| 模型胶水按标签与截断条目读别的文件、截断字段按号比、崩溃读回按号给允许集合 | S5 | 胶水今天判解不开：量过（第 15 行） | 胶水今天判解不开：量过（第 40、54 行） | 量过一半 |
| 正例用例在第 4389 行那个变异体（就是我的副本）上必须红：H1–H5 五段都断言四个读者收、读回等于期望、checker 零红 | S6 | 变异体上 H1 只有挂载态读者红：量过 | 变异体上读者与 checker 都红：量过 | 量过 |
| S7 判法在常量之外再 grep 四样结构写法，并以 H1–H5 当会红的判法 | S7 | 同上 | 同上 | 计数量过、判法推的 |

## 没打中的形状

- **S4 两条截断条目**：现行截 inode 2 一半 → 回退到截 inode 3 一半的 R_old；现行截 inode 2 一半 → 回退到没有截断条目的 R_old；R_old 有截断条目、现行有整理条目；回退之后先做写、建 inode、unlink、截断、空发布五种用户改动（读码逐步走，都先走空）。都只得到至多一条。只抽了一次样，按规则不当「没打中」引。
- **S2 只红自己**：两个起点、7 段历史上没有见到 size > 0 而没有条目的合法镜像（起点 109 条全绿，第 7 行）；没有扫既有用例的全部镜像。
- **S3 K9 叶边界在截断各批上漏重写**：意图与各批是不同的发布，单元 k 在意图那次连叶重写，照搬 unlink 的批计划在各批上不漏；没造出漏重写的历史（读码，推的；副本上续做走不下去，跑不到 144、288 个单元的那几批）。
- **GPU 两个内核**：只读码比了 CPU 那一判的同形，没起 GPU（正文第五节禁起）。

## 这条腿自己的限度

- 副本只删了一个拒绝分支，量出的是「只放开边界」那一种错误放开写出来的镜像，不是任何一条正确候选实现写出来的镜像；S2 的 H1 同时是「不写标签 0」那一臂的合法目标形，这一点是按形状认的。H6（「写标签 0」那一臂的目标形）是手造的，checker 那一行被我手造时留下的孤儿单元污染，只用了读者那几行。
- 没有调崩溃枚举，没有一个崩溃状态：这一轮的打中都是单镜像判法；崩溃流上的读回对拍（S5 第 3 条）只是读码。
- 正推腿的报告与模型在禁读清单里，没读；这一份与它重不重复由主 agent 对。
- 只抽一次样。没打中的各条不支撑「没打中」。
- 读码那几条（K3 计划换下第一个文件的下段、回退 `.find` 丢条目、GPU 两个内核、模型崩溃读回）都没有在副本上跑出来，标了「推的」。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑门禁、没跑 checker 档测试、没起 GPU；编译与运行只在草稿目录的副本里，`singlefs-harness` 一个测试目标，经 `run-with-memory-cap.sh`（3G）与 `capped.sh 31`。第一次以 8G 起的那一趟在内存包装里排队，我停了它（逐个 `proc.py stop`）改 3G 重起，那一趟一行都没跑（包装报「命令一行都没跑」）。
- 模板 `--selftest` 在我改过的 `rerun.sh` 上红（见开头），没有改模板。
- 草稿目录里的副本 `tree/`（39M）与编译目录 `target/`（1.6G）交回前删掉了；复跑目录 `rerun-work/` 里的副本与编译目录由 `rerun.sh` 跑完自己删了，只剩两份输出（36K）。留下的只有几份运行输出（`run1.txt`、`run2.txt`、`run3.txt`、`rerun-check.txt`、`s7-counts.txt`、`s7-counts-2.txt`、`cite.txt` 与 `rerun-work/` 那两份），不入库：它们是这份报告与模型目录 `out/probe.txt` 的来源，`out/probe.txt` 已进模型目录。
