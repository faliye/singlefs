# e171-conclusion-r3 攻方腿（Opus）报告

轮名 e171-conclusion-r3（第三轮，最终轮），立场：攻方。攻 K2、K3，兼 K1；不重复前两轮攻过的角度。kb 引文写 kb 文件自己的行号；代码引文写树的绝对路径。日期 2026-10-03。

两棵树：「s4 树」= E171 跑的 `/tmp/claude-1000/e171-s4/frozen-tree/`（E171 的行级读数与导出目录都是它的）；「这一轮的冻结副本」= `/tmp/claude-1000/e171-conclusion-r3/tree/`（方案按它谈）。我的探针与变异全部打在 s4 树的副本上（草稿目录 `ws_s4`），副本上的数注明是副本。

## 复跑与文件摘要

模型目录 `research/prompts/e171-conclusion-r3-opus-model/`。复跑（只写给定的工作目录；编译与探针经 `research/scripts/run-with-memory-cap.sh`；第 4 步默认只跑 13 行的子集约 4 分钟，第三个参数写 `all` 跑全部 232 行约 45 分钟，主 agent 放后台跑）：

```
bash research/prompts/e171-conclusion-r3-opus-model/tools/rerun.sh /tmp/claude-1000/<新的工作目录> 15 hidden
```

它依次：① 281 行变异在两棵树上打不打得上；② 第 15 行首次发布每次写的落点；③ 冻结 O3 与三个变坏的 O3（v1 已确认下界丢了、v2 只判恢复得出、v3 A(c) 取全部各版）在 Q2 坏镜像与两种「已确认的丢了」写流上判，再拿第二轮那一形的判官（我照字面写成 `c407_judge.py`）判入库产物与三份造出来的产物；④ s4 树副本上返回值探针跑基线、与 E171 导出 `ops.tsv` 逐条比，再把「未触达」的行逐行打进去重编重跑；⑤ 交回值不同的行在调试输出里改了哪个字段；末尾逐个与 `out/` 下同名文件 `cmp`。第 1–3 步我在草稿里用截掉第 4、5 步的同一份脚本跑过，`cmp` 无输出；第 4、5 步的命令与我手跑的逐条相同。

每个文件的 sha256（在模型目录里 `find . -type f | sort | xargs sha256sum`）：

```
383a7d86eca111b2c6d921ba1ea284b28ddc1162f6bef9d974f967422f1b4fbc  ./anchor_survival.py
79352c7c5e66fe32ea442f67a382cc81fb3fe7b5c8ebac01ad85c42f2a11aeab  ./c407_judge.py
03079abeb01fd67da9f29145d97609dcf9b1a7f70514dc04d5bf85fbc5e25e98  ./compare_ops.py
a1c0ee3c121a87d91d125bc4e4fcb379a4c67b25ad9ecce35ddca06ecca1cb4c  ./e165_pin_overlap.py
39e39e59e3011ade48666388b8faa87938b1380a3c0c36b6c1f526f4e35b3402  ./compare_return.py
778d046a7cf938fa9a3aad4cf0b15ab62387a4f2145ecd20ca7af79b503a7200  ./hidden_diffs.py
a2b6871b98896dd46a825e8d02d177a97b374f6f5eb1a62304570eca5fcf9c35  ./make_o3_variants.py
bd4a3da6f6e93df3722225a36f5c39a52d2c60148c10ab137237dd7e9b99384e  ./make_products.py
675dfbb30ce746d7b120144a3c547bc51a44025d46eccf96b6abd2fc30b2cdc8  ./mutate_by_row.py
baab80699cba0d63ad43c2ec8b6a52edfbccc26c31878d920830a9ec76350935  ./o3_ack_shapes.py
bc89ecb0adb7289899bb5b326bd4491550e1c4afde3838bbd27f5dc663b63041  ./opus_r3_return_probe.rs
d36421dac049654f166a207d7726b77e93c5ed02d86ed56e1ef937724412b70b  ./out/anchor-survival.out
f7b80a3c32339fc6e5089376e74bfb0e5d09fcfb74d2ae6f745dc3220684a09c  ./out/compare-ops-base.out
60f23a0f99e40f9f4588d9985c63f4e2ab3c01b1b516b2b0fe94bce91259cf87  ./out/e165-pin-overlap.out
a74469f343ad913f5f7eeb6212bc817cebdef881ab2c044f2ac05d768086d2d7  ./out/hidden-diffs.out
36c4bcff978b5609dff4cee933e1041de0fcfc118fcf9a5b7353e27aa08291ed  ./out/judge-runs.out
86a1ea27b918157204e648d56e211e3fe5905ec5d1ca0c85172fb24de27fcbdc  ./out/o3-controls.out
ad0c04f0a250e561fe2bb07ad5d0c3756427da29f75e398fc7d9f4e68fa3b996  ./out/q2-o3-only.out
ba7ec604b222acaf53db7eef7402dd5e35b750f2e7a972d0288285527f093ac7  ./out/ret-base-1.tsv
0d1f17765624ed3a21b7a53c7266d932ce813ec1c7a254c56935cf2a78c675db  ./out/return-rows.out
62218b01a9ffcfede7deffb63d18753a6a8a6726a87fbaad1898b6906c707c09  ./out/return-rows-subset.out
aced9ae34004b3098c341cefced7d7ee028e598e20a5413599e956214e5f472e  ./out/row15-offsets.out
04e1282f8a866b210fda6046c266412fc81e4b6637a9b33acb578d0a5439bb3b  ./out/shapes-variants.out
f492997d16aa734d2508f5f74205a8902ebc26dcd110e76aac57e7127fd5e5b5  ./q2_o3_only.py
58158eb1553aaffdfd3d7a599e44616ab419c6e1911d68f5c89a4ff4bd9b2796  ./row15_offsets.py
e424c898d68e3988c63a38015ca4440116991fc58c0a76faf7e77b9e4c284410  ./run_return_batch.sh
35315ed935a9d994f28a54d3c54154f3eb9c6b6adcab9c5bba920fabf89261f6  ./tools/rerun.sh
f61afb4eec0b73ea60e32ea61a8f0e10cdfeae4cf2d3996d7a59046d9293b916  ./untouched-s4.tsv
f49077341fd375eb97076490d6ddf4f486cb02aa6a0c4eff36e94053152673ef  ./untouched-subset.tsv
```

`o3_ack_shapes.py` 与 `compare_ops.py` 原样拷自 `research/prompts/e171-conclusion-r2-opus-model/`（sha256 与那边相同）；`opus_r3_return_probe.rs` 由那边的 `opus_r2_drive_probe.rs` 改出（驱动与写流记录照抄，多记交回值与分配器的 Debug 摘要）。

## 各格判定一览

| 格 | 对象 | 攻方判定 | 依据 |
|---|---|---|---|
| K2 | 未触达 238 行 | 照前两轮「这一批取到的态不合法」的口径，一行都进不了名单：写流摘要相同 ⇒ 每个崩溃态与基线逐字节相同。但「未触达」里有 7 行改动走到了、而且改了成功调用交回的值或内存里的分配器（第 49、110、155、161、162、174、252 行），第 155、161 行是已知的那两行，另 5 行 O1 不比那几个字段。第二轮改法表「装置记成功调用的交回值｜155、161 同形」写窄了：照它改触达，D 多 7 行、甲读法全放行多 5 行（17 行） | 量过（s4 树副本，232 / 238 行） |
| K2 | 名单里的第 15 行 | 留在名单，但第二轮写的依据错了：「t1@50180」（layout 第 309 行）是分配政策，D3 已定项 8 射程明写「镜像完全合法」「登记的是字段与宽度不是取值」，照第 34 行出名单的同一口径它撑不住第 15 行。撑得住的是 ⑤：变异把 t2–t12 十一个提交内生块整体挪了 64 槽（50240 → 50304 起），⑤ 与 layout 第 307 行钉的是这些落点。实验页对第 15 行「和基线不同在哪」的描述漏了这一半 | 量过（导出目录逐条 op） |
| K2 | 只靠一条的 23 行、名单另外 6 行 | 没找到按口径该进、该出的 | 读产物 + 条款（见「没打中的形状」） |
| K3 | 坏掉的 O3 | 「永远答在」「永远答红」两种判官都抓到。抓不到的是「已确认下界丢了」（v1）与「A(c) 取全部各版」（v3）：O3 独有的那一类（第二轮判决：丢掉的那一版内容看不出）在两种写流上 27 → 0，而判官两格都不动——① 因为 O3 在 37 行上本来 0 红、这两种变坏只会让它更少红；② 因为 Q2 的 29 张 O3 独有坏镜像 v1、v3 下仍是 29 张 | 量过（坏镜像、写流）+ 推的（行上那一半按单调性推，没重跑整表） |
| K3 | 少跑了行的整表 | 骗得过：只留登记的 12 行加一行只靠 O1、一行只靠 O2，判官判绿（计入分母 14 行，F4 门槛 30）。判官不读 D、`rows_run`、编不过与自崩的行数 | 量过（造出来的产物） |
| K3 | 过期产物 | 骗得过：补位前的 `…-s4-f4-comparator.out` 与补位后的那一份判官结果逐字相同；产物里没有树的摘要，判官分不出产物是 s4 树还是今天的树跑的；补位那一份在 `replay.sh` 里没有复跑行 | 量过 + 读 `replay.sh` |
| K3 | 与 `replay.sh` E171 复跑行的重叠 | 判官「读已入库产物、不重跑」时，任何代码或 oracle 的改动它都看不见：复跑绿则产物没变、判官结果没变；复跑红则判官读的还是旧产物。判官唯一独立的内容是「登记表与入库产物对得上」这件记账 | 读 `replay.sh` 与判官字面 |
| K3 | 与门禁 checker-tier-formal-verification 红例钉表的重叠 | 登记集合里的第 112 行，变异就钉在 E165 红例表里（`e165-pinned-red-examples.tsv` 第 26 行），那一格今天已经要求它红；给它再挂一个「补 oracle」欠账号是重复的。另有 8 行 E171 未触达的变异也在那张表里 | 读表 |
| K1 | 第二轮写回形态与 C407 改写 | 写宽 4 处、写窄 2 处、依据放错 1 处（第三节逐句） | 读 kb 与产物 + 上面量过的 |

## 一、K2：真盲区候选名单完整不完整（攻击面 ①）

### 1. 未触达的 238 行：照口径一行都进不了名单，但其中 7 行的改动走到了

**口径为什么挡住它们**：比对器的触达照三样算，第一样是「两条流任一条的 ops.tsv、ops-payload.bin、calls.tsv、base-sectors.bin 的 sha256」（`research/e7-index-bench/src/bin/e171_three_oracle_collusion/collusion_matrix.py:74`）。写流摘要相同，每个崩溃态与基线那一态逐字节相同；前两轮的口径是「这一批取到的态在已定条款下不合法、三条都放行」，这一批的态与基线相同，基线的态被当成合法，所以 238 行没有一行能按这个口径进名单。这是推的（推理只用到触达定义与口径字面），没有逐行再核。

**口径没罩住的那一样**：成功调用交回的值。实验页自己写了这一格看不出来：「别的未触达行有没有同类差异，装置没记成功调用的交回值，看不出来」（`.claude/kb/experiments/171-三oracle合谋测试.md:462`）。我把它量了。

做法（s4 树副本，harness 档自写探针 `opus_r3_return_probe.rs`）：照 E171 流二（取号、暖机、新池新建文件、覆盖写、进程重开之后可写挂载、覆盖写、回退、五次覆盖写、抬 F 到 11、覆盖写、卸载）在内存稀疏盘上驱动一遍，记每条写（下标、盘、种类、偏移、长度、内容摘要）、每次调用的结局、成功调用交回值 `{:?}` 文本的 sha256，末了分配器与现行那一版的 `{:?}` 摘要。流一是流二前三次调用，不另跑。基线跑两遍逐字节相同，写流与 E171 导出逐条相同：

```
name=compare_ops label=base dump=baseline-run1/fixed_script_through_e_then_unmount probe_ops=595 dump_ops=595 compared=595 identical_prefix=595 differing_in_compared=0 first_difference=-
```

然后把「未触达」的行逐行打进副本（编不过的 6 行取补位换上的那一行，同产物的 `name=substitution` 行）、重编、跑、还原（还原后 `touch`，每行的编译日志都有 `Compiling singlefs-core` 或 `singlefs-replay-prefix`；全部跑完副本 `crates/` 与 s4 树 `diff -rq` 无输出）。

取样与估时：先跑 3 行，每行重编加跑约 11–12 秒（编译为主），238 行估 45 分钟，超过 40 分钟；缩法是去掉 `make_filesystem.rs` 的行（mkfs 不是一次调用，探针不记它的交回值，那几行在这把尺子上本来就量不全）。去掉之前已跑了其中 4 行（第 28–31 行），没跑的是第 149–154 行。缩前 238 行，缩后 232 行，逐行耗时相加 2 863 秒，约 48 分钟（239 次，含因草稿事故重跑的 7 次：第 1、3、16、17 行输出被追加两遍或误删，第 140–142 行跑的时候另两批在同一副本上打过又还原变异；重跑前核过副本与 s4 树 `diff -rq` 无输出；14–20 行一批，每批前台跑完）。崩溃态不在这把尺子里（它比的是调用与写流，不枚举崩溃态）。

原样输出（`research/prompts/e171-conclusion-r3-opus-model/out/return-rows.out`，交回值不同的 7 行与汇总行）：

```
name=return_row row=49 status=hidden_difference ops_same=True call_words_same=True returned_differs_at=4:mount_writable final_allocator_same=True final_output_same=True file=crates/singlefs-core/src/mount.rs row_name=C554_乙：c_见证_取系统配置槽_tail_的最小值（只剩较旧那一槽的见证）
name=return_row row=110 status=hidden_difference ops_same=True call_words_same=True returned_differs_at=2:publish,3:publish,4:mount_writable,5:publish,7:publish,8:publish,9:publish,10:publish,11:publish,12:raise_floor,13:publish,14:unmount final_allocator_same=True final_output_same=False file=crates/singlefs-core/src/write_accounting.rs row_name=增补_1：树表单元的写记到中央映射树名下（合计不变，按种类钉的字节表红）
name=return_row row=155 status=hidden_difference ops_same=True call_words_same=True returned_differs_at=12:raise_floor final_allocator_same=True final_output_same=True file=crates/singlefs-core/src/mount.rs row_name=步_5：抬_F_的上限不看第_4_新的非空根
name=return_row row=161 status=hidden_difference ops_same=True call_words_same=True returned_differs_at=12:raise_floor final_allocator_same=True final_output_same=True file=crates/singlefs-core/src/mount.rs row_name=增补_3_第_2_件（理想模型，r1_攻方变异_B3）：抬_F_上限不取第_4_新的非空根（只取每盘最新有效根）
name=return_row row=162 status=hidden_difference ops_same=True call_words_same=True returned_differs_at=4:mount_writable final_allocator_same=True final_output_same=True file=crates/singlefs-core/src/mount.rs row_name=影子账两臂报同一个分支名（运行时看不出走了哪一条，五条硬要求第_4_条）
name=return_row row=174 status=hidden_difference ops_same=True call_words_same=True returned_differs_at=12:raise_floor final_allocator_same=True final_output_same=True file=crates/singlefs-core/src/mount.rs row_name=实二（回退改形态：SysPre_与_B1，D16_已定项_1_/_7、D22_已定项_7_/_9、I-7.9_/_I-7.12、C556_/_C557）：抬_F_做成时先写系统配置那一步的写账丢掉
name=return_row row=252 status=hidden_difference ops_same=True call_words_same=True returned_differs_at=4:mount_writable final_allocator_same=False final_output_same=True file=crates/singlefs-core/src/transaction.rs row_name=增补_2_收口表第_39_行那一族：发布取完落点之后不记根（不转环、不回收；取号之前的预演与真发走同一段，两边一起不回收）
name=return_summary hidden_difference=7 identical=225 no_output=6
```

`no_output=6` 是缩掉的第 149–154 行。阳性对照是现成的：第 155、161 行就是 E171 自己报的「没触达却红」那两行（产物 `name=untouched_but_red rows=2 row_numbers=155,161`），这把尺子在同一次调用（第 12 次 `raise_floor`）上抓到它们，交回的上限 11 → 13（`out/hidden-diffs.out`：「name=hidden_diff row=155 item=call12-raise_floor lines_removed=1 lines_added=1 base=11, mutant=13,」）。

另外 5 行改了什么（`research/prompts/e171-conclusion-r3-opus-model/out/hidden-diffs.out`，调试输出逐字段比，每行截前 3 处）：

```
name=hidden_diff row=49 item=call4-mount_writable lines_removed=1 lines_added=1 base=witnessed_journal_counter: 4, mutant=witnessed_journal_counter: 3,
name=hidden_diff row=110 item=call2-publish lines_removed=6 lines_added=2 base=write_calls: 2, | written_bytes: 32768, | }, mutant=write_calls: 4, | written_bytes: 65536,
name=hidden_diff row=162 item=call4-mount_writable lines_removed=1 lines_added=1 base=shadow_ledger_branch: "shadow_ledger=on", mutant=shadow_ledger_branch: "shadow_ledger",
name=hidden_diff row=174 item=call12-raise_floor lines_removed=6 lines_added=1 base=counted: { | SystemConfigurationSlot: WriteCallsAndBytes { | write_calls: 2, mutant=counted: {},
name=hidden_diff row=252 item=final-allocator lines_removed=121 lines_added=1 base=region: 0, | slot: 2, | }: ValidRoot { mutant=4,
```

读法（每行的条款归属是推的，只到「属于哪一类」，没逐条量害处）：

| 行 | 交回的哪一样不同 | 三条为什么都不红 | 归哪一类 |
|---|---|---|---|
| 49 | 可写挂载交回的见证值 4 → 3（取了最小）。条款是「取全部自证过的槽 journal tail 的最大值 c_见证」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）；这个值进见证判据「} => self.witness.witnessed_journal_counter > selected_version_last_record_counter,」（`/tmp/claude-1000/e171-s4/frozen-tree/crates/singlefs-core/src/mount.rs:397`），这段历史里判据结局没变 | O1-对拍比 `ObservedRoot`，不带见证值；写流与基线相同，O1-崩溃判官、O2、O3 看到的态与基线相同 | 取样点（历史）不敏感：要一段让最小值与最大值落在判据两边的历史 |
| 110 | 每次发布、挂载、抬 F、卸载交回的按结构种类写账：树表单元的写记到别的种类名下（合计不变） | 写账不进 `ObservedRoot`，不进盘 | 运行时读数，三条按设计都不判（与第 34 行「拦它只能靠运行时读数」同类） |
| 162 | 可写挂载交回的影子账分支名 | 同上 | 运行时读数（「让 `MountOutput::shadow_ledger_branch` 报得出走了哪一条（五条硬要求第 4 条）」，`/tmp/claude-1000/e171-s4/frozen-tree/crates/singlefs-core/src/mount.rs:1130`） |
| 174 | 抬 F 交回的写账丢了系统配置那一步 | 同上 | 运行时读数 |
| 252 | 可写挂载交回值与末了的分配器里少了根槽登记（内存里） | 写流相同 | 取样点（历史）不敏感：根环转得到被漏登的槽时才分得开 |

**这一格打中的是什么**：不是名单（照口径一行不进），是三件事。
- 第二轮判决改法表那一行「| E171 装置记成功调用的交回值 | 第一轮两腿 | 155、161 同形 |」（`research/prompts/e171-conclusion-r2-main-verification.md:63`）写窄了：同形的不是 2 行，是这 232 行里的 7 行。照这条改法把交回值算进触达，第 49、110、162、174、252 行变成计入分母、甲读法三条全放行的行（O1-对拍在这 5 行上不红：产物里没触达却红的只有第 155、161 行）。第二轮判决说「连同第 155、161 行改触达定义 D = 40」（同文件第 33 行），量到的是 D = 45、甲读法全放行 12 → 17 行（加第 125 行与这 7 行，是算术，没在入库装置上跑）。C407 改写的「登记集合」随这一条改法变 5 行。
- 这 5 行不是真盲区候选（态与基线相同），也不进第二轮的另几类（27、112、277 的恢复一侧；34 的分配政策；92 的条款欠一句）。名单旁边的另几类少一类：「交回值或内存状态错、写流不变」，其中 110、162、174 是运行时读数，49、252 要更长的历史。
- 实验页「它答不了的」那一句「未触达只说那一处改动在这两条流上没走到，不说它被哪条 oracle 罩着」（`.claude/kb/experiments/171-三oracle合谋测试.md:461`）对这 7 行不成立：改动走到了，只是没写到盘上。

### 2. 名单里的第 15 行：留下，但第二轮写的依据撑不住它，撑得住的是 ⑤

第二轮判决对第 15 行写的是「D3 已定项 10 ⑤ 落点表与 `layout/01-first-txn.md` 第 309 行「新池新建文件 t1@50180」钉死字节」（`research/prompts/e171-conclusion-r2-main-verification.md:25`）。同一份判决把第 34 行挪出名单，依据是 D3 已定项 8 射程（同文件第 26 行）。两句放在一起：

- 第 309 行那一格是「用户数据落点政策」，原文「不在开放段与 R 内的空槽（新池新建文件 t1@50180）」（`.claude/kb/layout/01-first-txn.md:309`），它点的决策是 D3 已定项 8 / 已定项 10。
- 已定项 8 射程：「它是分配器政策不是格式位，与已定项 5 同一类——盘上没有字段声明它，违反它的实现写出的镜像完全合法」（`.claude/kb/decisions/03-空间分配.md:171`），同一段还写「登记的是字段与宽度不是取值」（`.claude/kb/decisions/03-空间分配.md:171`）。

照第 34 行出名单的那个口径，单凭 t1 落在别处，第 15 行的态也是合法的。我去导出目录逐条 op 看第 15 行到底挪了什么（`out/row15-offsets.out`，两条流相同）：

```
name=row15_first_publish stream=new_pool_file_creation device0_writes_moved=12 op22:50180->50240(32768) op24:50240->50304(16384) op26:50242->50306(32768) op28:50244->50308(16384) op30:50245->50309(16384) op32:50246->50310(16384) op34:50247->50311(16384) op36:50248->50312(16384) op38:50249->50313(16384) op40:50250->50314(16384) op42:50251->50315(16384) op44:50252->50316(16384)
```

用户数据落到 50240，那一段不再全空，开放段整段后移到 [50304, 50368)，十一个提交内生块全部挪了 64 槽。⑤ 钉的正是它们：「⑤ 在新池新建文件上的落点（开放段 [50240, 50304)，bump 游标从 50240 起）」（`.claude/kb/decisions/03-空间分配.md:206`），射程「改新池新建文件的字节：是——七个提交内生块的落点整体重排」（`.claude/kb/decisions/03-空间分配.md:220`），字节表「t2 到 t12 依次 50240、50242–50243、50244、50245、50246、50247、50248、50249、50250、50251、50252」（`.claude/kb/layout/01-first-txn.md:307`）。

判：第 15 行留在名单，依据换成「⑤ 与 layout 第 307 行：t2–t12 的落点」，不写 t1@50180 与第 309 行。实验页对它的描述「用户数据单元两块盘各 32 KiB 落点 822149120 → 823132160，内容逐字节相同；指向它的 16 KiB 节点里指针与校验和随之变」（`.claude/kb/experiments/171-三oracle合谋测试.md:86`）漏了提交内生块整体挪位那一半，三分那一格写的「条款没定」是第一轮之前的读法。推翻条件：⑤ 被认作「新池那一次的示例」而不是条款（那时第 15 行与第 34 行同类、出名单）。

### 3. 只靠一条的 23 行与名单另外 6 行：没找到该进、该出的

- 只靠一条的 23 行：按口径进名单要三条都放行，这 23 行批 1 各有一条红；第二轮已判这 23 行的红没有误红（一次抽样），这一轮不重攻那个角度。批 2 上全放行的第 102、233（丙）、265 行，态在批 2 上合法（取样点不敏感），第一、二轮都记了「不稳定」，不进名单与口径一致。
- 第 23、42、242、269、270、271 行：各自的条款句在实验页与前两轮判决里，我读了产物的 `name=q1_quiet_row` 行与实验页的逐行说明，没找到与口径不符的；没另跑、没逐态核。

### 4. 打中之后的四句（K2 两格）

| 问 | 交回值那一格（第 1 小节） | 第 15 行依据那一格（第 2 小节） |
|---|---|---|
| 分不分辨臂 | 分辨「触达只算三样」与「触达加交回值」两种读法：前者 D = 37（加第 125、155、161 行是 40），后者 45；名单不变，C407 登记集合变 5 行 | 不分辨名单成员（第 15 行两种依据下都在）；分辨依据句 |
| 被判的系统当时看不看得到 | 看不到：E171 装置不记成功调用的交回值，产物里没有这一列；O1-对拍看得到交回值，但比的字段里没有见证值、写账、分支名、分配器的根登记 | 看得到：导出目录 `ops.tsv` 里每条写的偏移都在，只是实验页那一格没数提交内生块 |
| 满足判据字面哪一个分句 | 不满足名单口径的任何分句（态不合法那一句不成立）；满足的是登记读法 4 自己写的前提「三样都相同的行……在这一批上与正常实现逐字节同行为」不成立的那一半 | 名单口径「这一批取到的态在已定条款下不合法」：成立，经 ⑤ 而不经第 309 行 |
| 跑前条款给的改法在这几格上还中不中 | 第二轮改法「装置记成功调用的交回值」：让这 7 行进分母，但 O1 照样不比那几个字段，5 行变成新的全放行行，不是补上了；要补得另写 O1 的答（见证值、写账、分支名），或照第 34 行的先例归「运行时读数，三条按设计不判」 | 第二轮的改法「O2 新池新建文件落点照 D3 已定项 10 ⑤ 核」对这一格仍中，而且核的对象要写成 t2–t12，不是 t1 |

## 二、K3：C407 改写的判官骗不骗得过（攻击面 ②）

### 1. 判官写成代码

第二轮判决第三节的字面：「改写成——判官读已入库的 E171 比对器产物（不重跑，重跑归 experiment-replay）」（`research/prompts/e171-conclusion-r2-main-verification.md:48`），① 全放行集合等于登记集合，② 每一列至少一样只有它红、O3 那一列用 Q2 的 O3 独有坏镜像。字面没写的几处我取最宽的读法写进 `c407_judge.py` 文件头：甲读法（O1 = 对拍或崩溃判官、O2、O3 严格）、批 1；O3 那一列取 `name=q2_family` P、B 两族 `o3_only_strict` 之和。登记集合取第二轮的 12 行。它不读的：D、`rows_run`、冻结核对、阳性 / 阴性对照、批 2、丙读法、树。

### 2. 坏掉的 O3：「永远答在」「永远答红」抓得到，「已确认下界丢了」抓不到

在 O3 冻结执行器的副本上造三个变坏的 O3（`make_o3_variants.py`，原目录不动）：

- v1 已确认下界丢了：「acknowledged = version.version_index」（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:98`）改成恒 0；
- v2 只判恢复得出：「landed_is_allowed = recovery.landed is not None and any(」（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/executor_core.py:127`）去掉后半句；
- v3 A(c) 取全部各版：v1 再加 begun 恒为最后一版。

先核 v0（冻结那一份原样）在 Q2 坏镜像上的判定与入库产物 `…-s2-images-o3.out` 的 `name=image_judgement` 行逐字节相同（`diff` 无输出）。再拿比对器自己的 Q2 读入函数（`bad_images_report.py` 原样调用）数各族「O2 不红而 O3 红」（`out/q2-o3-only.out`）：

```
name=q2_o3_only variant=v0 P:o3_red=22/o3_only=12 B:o3_red=40/o3_only=17 K:o3_red=28/o3_only=0 N:o3_red=0/o3_only=0 o3_only_total=29 …
name=q2_o3_only variant=v1 P:o3_red=22/o3_only=12 B:o3_red=40/o3_only=17 K:o3_red=28/o3_only=0 N:o3_red=0/o3_only=0 o3_only_total=29 …
name=q2_o3_only variant=v2 P:o3_red=4/o3_only=0 B:o3_red=0/o3_only=0 K:o3_red=2/o3_only=0 N:o3_red=0/o3_only=0 o3_only_total=0 o3_only_rows=none
name=q2_o3_only variant=v3 P:o3_red=22/o3_only=12 B:o3_red=40/o3_only=17 K:o3_red=28/o3_only=0 N:o3_red=0/o3_only=0 o3_only_total=29 …
```

（v0、v1、v3 三行末尾的 29 张名单逐字相同，这里截了；全文在文件里。）v0 的 12 + 17 与产物第 4759、4760 行 `o3_only_strict=12`、`o3_only_strict=17` 相同。

同样四个 O3 在第一、二轮攻方那两种「已确认的调用丢了」写流上（`o3_ack_shapes.py` 原样，末段全枚举 288 态；第二轮判决认定这一形是 O3 独有的那一类，`out/shapes-variants.out`）：

```
variant=v0 name=o3_ack_shape shape=warmup tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=27 o3_provisional_not_in=27 spec_gap=0 anomalies=1
variant=v0 name=o3_ack_shape shape=content tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=27 o3_provisional_not_in=27 spec_gap=0 anomalies=1
variant=v1 name=o3_ack_shape shape=warmup tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=0 o3_provisional_not_in=0 spec_gap=0 anomalies=1
variant=v1 name=o3_ack_shape shape=content tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=0 o3_provisional_not_in=0 spec_gap=0 anomalies=1
variant=v2 name=o3_ack_shape shape=warmup tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=0 o3_provisional_not_in=0 spec_gap=0 anomalies=1
variant=v2 name=o3_ack_shape shape=content tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=0 o3_provisional_not_in=0 spec_gap=0 anomalies=1
variant=v3 name=o3_ack_shape shape=warmup tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=0 o3_provisional_not_in=0 spec_gap=0 anomalies=1
variant=v3 name=o3_ack_shape shape=content tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=0 o3_provisional_not_in=0 spec_gap=0 anomalies=1
```

v0 两份逐态判定与第二轮攻方模型的 `out/shape-*-o3-verdicts.tsv` 逐字节相同。O3 的阳性对照（P1-F）与 8 张底态在四个 O3 上都不变（`out/o3-controls.out`）：

```
name=o3_controls variant=v1 p1_f_new_pool_strict=not_in base_images_strict_not_in=0 freeze_check_differences=2
name=o3_controls variant=v3 p1_f_new_pool_strict=not_in base_images_strict_not_in=0 freeze_check_differences=2
```

判官的 ① 在 v1、v3 下不动，这一半是推的、没重跑整表：v1、v3 只把 A(c) 放大（「return AllowedVersions(largest_index, acknowledged, begun, table.versions[acknowledged:begun + 1])」，`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:102`，下界变小、上界变大），「不在」只会变成「在」；O3 在 37 行上本来 0 红（产物 `name=q1_column_red_rows … O3甲=0 O3乙=0`），少红不会让任何一行进出全放行集合。② 的 O3 那一格照上面量的仍是 29 张。所以：**v1、v3 让 O3 丢掉它在 D13 意义上独有的那一类（27 → 0），判官两格都绿**。v2（只判恢复得出）与「永远答在」判官抓得到（Q2 独有归 0）；「永远答红」也抓得到（全放行集合变空）。

判官在入库产物与造出来的产物上（`out/judge-runs.out`，`short.out` 只留登记的 12 行与第 32、8 行的逐行结果行，`o3red.out` 每行批 1 的 O3甲 改红，`o3in.out` Q2 的 O3 独有改 0）：

```
e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out name=c407_judge verdict=green counted_rows=37 all_quiet=15,23,27,34,42,92,112,242,269,270,271,277 equals_registered=true only_O1_rows=10 only_O2_rows=13 only_O3_rows=0 q2_o3_only_images=29 column_has_a_unique_red=O1,O2,O3
e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out name=c407_judge verdict=green counted_rows=37 all_quiet=15,23,27,34,42,92,112,242,269,270,271,277 equals_registered=true only_O1_rows=10 only_O2_rows=13 only_O3_rows=0 q2_o3_only_images=29 column_has_a_unique_red=O1,O2,O3
e171-three-oracle-collusion-2026-10-03-s4-comparator.out name=c407_judge verdict=red counted_rows=19 all_quiet=15,23,27,34,42,92,112 equals_registered=false only_O1_rows=4 only_O2_rows=7 only_O3_rows=0 q2_o3_only_images=29 column_has_a_unique_red=O1,O2,O3
short.out name=c407_judge verdict=green counted_rows=14 all_quiet=15,23,27,34,42,92,112,242,269,270,271,277 equals_registered=true only_O1_rows=1 only_O2_rows=1 only_O3_rows=0 q2_o3_only_images=29 column_has_a_unique_red=O1,O2,O3
o3red.out name=c407_judge verdict=red counted_rows=37 all_quiet=none equals_registered=false only_O1_rows=0 only_O2_rows=0 only_O3_rows=12 q2_o3_only_images=29 column_has_a_unique_red=O3
o3in.out name=c407_judge verdict=red counted_rows=37 all_quiet=15,23,27,34,42,92,112,242,269,270,271,277 equals_registered=true only_O1_rows=10 only_O2_rows=13 only_O3_rows=0 q2_o3_only_images=0 column_has_a_unique_red=O1,O2
```

- **少跑了行的整表**：`short.out` 判绿，计入分母 14 行；F4 自己的门槛是 30（产物 `enough_to_judge_none_side`）。真实的一次少跑（锚点腐化、编不过、自崩的行变多）只要不碰登记的 12 行与各列各一行只靠它的行，判官同样绿。117 行那一份判红，是因为登记集合里的扩表 5 行不在，不是判官看了分母。
- **过期产物**：补位前后两份产物判官输出逐字相同。产物里没有树的摘要（`grep -c 'snapshot\|d1586465'` 在补位那一份上是 0），判官分不出产物是 s4 树还是今天的树跑的。补位那一份在 `replay.sh` 里没有复跑行：「还没有入库产物，这里不跑；补位产物入库之后另登一行」（`research/scripts/replay.sh:1012`），E171 的十行登记只到 `…-s4-f4-comparator.out`。
- **今天的树**：281 行的变异在这一轮的冻结副本上 279 行仍打得上（`out/anchor-survival.out`：「name=anchor_survival root=/tmp/claude-1000/e171-conclusion-r3/tree rows=281 applicable=279 not_applicable=2 by_category=untouched:2」），登记的 12 行都打得上；但 `replay.sh` 的 s4、F4 两行「读数读今天的 `crates/`」（`research/scripts/replay.sh:1011`），第二轮攻方量过同一份驱动在第一轮的冻结副本上第 40 条 op 起就与 s4 树不同（这一轮的冻结副本我没重驱）。复跑一次，产物就换成今天的树的读数，与 D13 写回形态「E171 的行级读数只属于 s4 树」两不相容。

### 3. 与 `replay.sh` 的 E171 复跑行、与红例钉表重叠在哪

**与复跑行**：E171 在 `replay.sh` 的十行都是 `exact`，s4 与 F4 两行分别是「E171|@driver_e171_s4_comparator||e171-three-oracle-collusion-2026-10-03-s4-comparator.out|exact」（`research/scripts/replay.sh:261`）与「E171|@driver_e171_s4_f4_comparator||e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out|exact」（`research/scripts/replay.sh:262`）。判官「读已入库产物、不重跑」，两者的关系是：

| 情形 | 复跑行 | 判官 |
|---|---|---|
| 代码、oracle 都没变 | 绿 | 绿 |
| oracle 或实现变了、产物也跟着变 | 红（逐字节比） | 绿：它读的还是入库那一份 |
| 复跑红之后有人把新产物入库 | 绿 | 才第一次看到变化；v1、v3 这种变坏照样判绿（上一小节） |
| 登记表改了、产物没改 | 绿 | 红 |

判官在任何代码或 oracle 的改动面前都不先于复跑红；它独立判的只有最后一行，即「登记表与入库产物对得上」这件记账。「每列至少一样只有它红」在入库产物上是一个固定的事实，读一次之后每次都同样成立。补位那一份产物没有复跑行，判官若读它，连复跑这道保险都没有。

**与门禁 checker-tier-formal-verification 的红例钉表**：那一格判的是「pinned-red-examples-replay 钉住的每一行是 21 段整行抄、抄来的账行与自己的段对得上，点名的变异还在它的表里，装置在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同」（`.claude/gate.d/checker-tier-formal-verification.sh:18`）。E171 的 281 行里 9 行的变异名在那张表的第 6 段（`out/e165-pin-overlap.out`：「name=e165_pin_overlap_summary rows=9 counted=1 untouched=8」）：

- 登记集合里的第 112 行就在其中：「crates/mutations.tsv#步 3：前缀跨实例边界（把别的实例的记录也接上）」（`.claude/gate.d/e165-pinned-red-examples.tsv:26`）。E165 的有界穷举今天已经在它上面红、门禁按行复判。C407 改写要给第 112 行挂「补 oracle 的欠账号」，这一笔已经有一个会红的检查在还；第二轮自己也把第 27、112、277 行归到「M4 的落点是 E165 与 Verus」。
- 另 8 行（第 70、71、113、114、213、274、275、276 行）E171 记未触达，E165 红。两道检查都是重型、只在提交时跑；E171 那一道只证明「这两条流走不到」，钉表那一道证明「有界穷举里红」，两边不冲突，也互不替代。
- 第 277 行的变异在 E165 的挑行里出现过，标的是「not_judged_enough_to_judge_without_it」（`research/results/e165-replay-bounded-enumeration-2026-10-02-r2-segment2-row-selection.out:142`），没判；第 27 行的变异名在 `research/results/e165-*` 里零命中。所以「三条不判实现恢复里靠记录重建的那一段」三行里，只有第 112 行在别处有会红的检查。

### 4. 打中之后的四句（K3）

| 问 | v1、v3 判绿 | 少跑了行判绿 | 过期产物 / 树判绿 |
|---|---|---|---|
| 分不分辨臂 | 分辨：第二轮那一形（O3 用 Q2 坏镜像）判绿，第一轮的「钉四列矩阵」同样判绿（O3 一列本来全 0）；加一条崩溃态上 O3 独有的那一形（下一小节）判红 | 分辨：加「读 D 与行数」的形判红 | 分辨：加「产物带树摘要、判官比登记的摘要」的形判红 |
| 判官当时看不看得到 | 产物里看得到的只有冻结核对行 `differences=2`，任何 O3 的改动都会让它不为 0，分不出改好还是改坏；O3 在行上 0 红，产物里没有一样东西会因为 v1 变 | 看得到：`name=q1_denominator … counted_D=` 在产物里，判官字面不读 | 看不到：产物里没有树的摘要 |
| 满足判据字面哪一个分句 | ② 的「O3 那一列用 Q2 的 O3 独有坏镜像」；① 对 O3 在这张表上没有判别力 | ①、② 都满足，判官字面没有分母那一句 | 「读已入库产物」那一句：入库的是哪一份、哪棵树，字面没写 |
| 跑前条款给的改法还中不中 | 第二轮第三节没有给判官的备选改法；第二轮改法表里给 O3 的「加谓词 W」「见证值 ≥ tail」是补 O3 的判别力，与判官抓不抓得到 O3 变坏无关 | — | — |

### 5. 我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | v1、v3 | 少跑了行 | 过期产物 | 依据 |
|---|---|---|---|---|---|
| 甲：O3 那一列除了 Q2 坏镜像，再钉一形崩溃态上只有 O3 红的写流（暖机那一形，288 态，冻结 O3 判 27 态不在；第二轮量过 RefFS、O1-崩溃判官、O2 在它上面都绿） | O3 变坏 | 判红（27 → 0） | 不管 | 不管 | 量过（`out/shapes-variants.out` 前 8 行，副本上的数） |
| 乙：判官读 `name=q1_denominator` 的 `rows_run`、`counted_D`、`compile_failed`，要等于登记的值 | 少跑了行 | 不管 | 判红 | 不管 | 推的（`short.out` 上 `counted_rows=14`，判官字面加一句就红；没实现） |
| 丙：比对器把逐行驱动的树快照摘要写进 `name=config`，判官比登记的摘要；补位那一份登一行复跑 | 过期产物、树 | 不管 | 不管 | 判红 | 推的（产物里今天没有摘要；没实现） |
| 丁：判官读复跑那一趟写出的新产物，不读入库那一份 | 判官先于复跑没东西可判 | 甲在新产物上才起作用 | 同乙 | 同丙 | 推的 |

甲的代价：288 态在冻结 O3 上一趟的耗时我没单独量（第 3 步四个 O3、两形加坏镜像整步在草稿里两秒内跑完，只是一个上界）。甲修不了 O3 别的变坏方式（例如把 P、B 两族判出「不在」的那条投影比较弄坏，而保留窗口）——那一种推的是由 ② 的 Q2 那一格抓（没量），所以甲是加，不是换。

## 三、K1：第二轮写回形态里写宽、写窄的句子（攻击面 ③，只写上面量到的与它牵出来的）

| # | 句子（第二轮判决原文） | 宽 / 窄 / 放错 | 依据 | 改成（零轮） |
|---|---|---|---|---|
| 1 | 「每个登记行挂一个补 oracle 的欠账号」（`research/prompts/e171-conclusion-r2-main-verification.md:48`） | 写宽 | 登记集合 12 行里，第 34 行第二轮自己归「分配政策，镜像合法」，第 92 行归「条款欠一句」，第 27、112、277 行归「三条不判实现恢复里靠记录重建的那一段」、M4 落点在 E165 与 Verus；5 行没有 oracle 可补。第 112 行在 E165 红例钉表第 26 行，已有会红的检查 | 「每个登记行挂一个欠账号，写明是补 oracle、补条款，还是已由别处的检查判（点名那一道）」 |
| 2 | 「「三者全部放行」的行集合等于登记集合」（同上） | 写窄（没定口径） | 甲、乙 12 行，丙 13 行（多第 233 行），批 2 是 15 行（产物第 4529、4555 行）；第二轮改法表若采「装置记交回值」，甲还会多 5 行（第一节） | 写明「甲读法、批 1」，并写明触达定义改了之后登记集合要随一次复跑重登 |
| 3 | 「O3 那一列用 Q2 的 O3 独有坏镜像」（同上） | 写宽（它证明不了它被拿来证明的东西） | 这 29 张在 v1、v3 下仍是 29 张，而 O3 独有的那一类在 v1、v3 下 27 → 0（第二节）；判官判不出 O3 丢了窗口 | 「O3 那一列用 Q2 的 O3 独有坏镜像，再加一形崩溃态上只有 O3 红的写流（暖机那一形）」 |
| 4 | 「判官读已入库的 E171 比对器产物（不重跑，重跑归 experiment-replay）」（同上） | 写宽（说的是判官，做到的只是记账） | 读入库产物的判官在任何代码或 oracle 改动面前都不先于复跑红（第二节第 3 小节那张表）；补位那一份没有复跑行；产物不带树摘要 | 要么照实写成「登记表与入库产物的一致性检查」，不叫合谋测试的判官；要么改成读复跑那一趟的新产物，并补上乙、丙两条 |
| 5 | 「O3 有只靠它判出的坏镜像（Q2 `o3_only_strict` 12 + 17）」（`research/prompts/e171-conclusion-r2-main-verification.md:43`） | 写宽 | Q2 只比 O2 与 O3：O1 不碰盘，RefFS 没在坏镜像上跑，「只靠它」只对 O2 成立；而且这 29 张靠的是投影比较，不是第二轮认定 O3 独有的那一类（丢掉的那一版内容看不出） | 「O2 放行而 O3 判不在的坏镜像（Q2 `o3_only_strict` 12 + 17；O1 与 RefFS 没在坏镜像上跑）」 |
| 6 | 第 15 行的依据「D3 已定项 10 ⑤ 落点表与 `layout/01-first-txn.md` 第 309 行「新池新建文件 t1@50180」钉死字节」（同文件第 25 行） | 依据放错一半 | 第 309 行那一格是分配政策，D3 已定项 8 射程判它的违反者「镜像完全合法」（第一节第 2 小节）；撑得住第 15 行的是 ⑤ 与 layout 第 307 行，变异挪了 t2–t12 | 「D3 已定项 10 ⑤（`.claude/kb/decisions/03-空间分配.md` 第 206、220 行）与 layout 第 307 行：变异下 t2–t12 十一个提交内生块整体挪 64 槽」 |
| 7 | 改法表「| E171 装置记成功调用的交回值 | 第一轮两腿 | 155、161 同形 |」（同文件第 63 行） | 写窄 | 同形的是 7 行（第 49、110、155、161、162、174、252 行），另 5 行 O1 也不比；这条改法让它们进分母、成为全放行行，补不上 | 「E171 装置记成功调用的交回值：同形 7 行（s4 树副本上量的），其中 5 行 O1 不比那几个字段，要么给 O1 加答、要么归运行时读数与历史取样」 |

实验页两句随之要改（不是第二轮写回形态，是写回时引的读数源）：第 86 行第 15 行「和基线不同在哪」漏了提交内生块整体挪位；第 461 行「未触达只说那一处改动在这两条流上没走到」对上面 7 行不成立。

## 没打中的形状

- 未触达 238 行里按「态不合法」的口径进名单的：0 行。理由是触达的第一样就是写流摘要（第一节第 1 小节），不靠取样；没打中的是「口径本身」以外的东西，我没找到能绕过它的形状。
- 只靠一条的 23 行与名单其余 6 行：只读产物与条款，没另跑；没找到该进、该出的。
- K3 里「永远答在」「永远答红」的 O3、只判恢复得出的 O3（v2）：判官都抓到（`o3in.out`、`o3red.out` 判红，v2 让 Q2 独有归 0）。O1、O2 列的「只有它红」行各有 10、13 行，一列变坏让任何一行只靠它的行失手，① 就红；骗得过的只有同时被两条抓到的行（第 11、244 行）上丢掉一条，那一种第二轮已降为测量。
- 判官对「补 oracle 之后它红」的反应（例如第 271 行补上之后全放行集合少一行、判官红到登记表改）：照设计就该如此，不算骗过，没试。

## 这条腿自己的限度

- 第一节的交回值只看流二（流一是它的前三次调用），只看成功调用交回值、末了的分配器与现行那一版；崩溃态上实现的恢复、恢复之后内存里的状态没比。第 149–154 行（mkfs）没跑。交回值的「不同」用 `{:?}` 文本的摘要判，任何派生 `Debug` 的字段都算，包括纯诊断字段；我把 7 行逐个看了字段（`out/hidden-diffs.out`），归类是推的。
- 7 行的害处没量：第 49、252 行我推的是要更长的历史才分得开，没造那段历史。
- 第二节 ① 在 v1、v3 下不动是按单调性推的，没在 v1、v3 上重跑 37 行的批态（整表是重型，归提交）；v1、v3 在 Q2 坏镜像、两种写流、阳性与阴性对照上是量过的。
- 判官是我照第二轮字面写的模型，字面没写的地方取了最宽的读法（文件头列着）；主 agent 的实现若读了冻结核对或分母，第二节第 2 小节的「判绿」会变——冻结核对能分开「改了」与「没改」，分不开改好与改坏。
- 副本上的数不算入库装置上的数；这一轮没有入库装置可跑（E171 装置是 checker 档）。
- 引 kb 都是这一轮快照里的文件（开工时 `sha256sum -c` 三份 kb 无输出）。

## 没做什么

- 没跑任何 checker 档测试、没起 E171 装置或整表、没跑门禁；编译只编自写的 harness 档探针（s4 树副本），都经内存包装。
- 并行两条线道的那一批被内存包装按「同一个会话同时只许一条」退 252（命令没跑）；之后全部串行。有一批超过前台时限被移到后台，我用 `proc.py wait` 等它退出，期间起的两批在同一副本上打过又还原变异（编译都退 252 没跑），核过副本干净后重跑了可能被碰到的第 140–142 行。
- 草稿目录 `/tmp/claude-1000/e171-conclusion-r3-opus/` 里的两份 s4 树副本与编译目录、调试输出交回前删掉（见交回）。
