# e171-conclusion-r2 攻方腿（Opus）报告

轮名 e171-conclusion-r2（第二轮），立场：攻方。攻 H1–H4，重点 H2、H4；不重复第一轮攻过的角度。kb 引文写 kb 文件自己的行号；代码引文写冻结副本的绝对路径与行号。日期 2026-10-03。

两棵树，先说清楚（这一条本身是 H4 的一格，见第四节）：

- 「s4 树」= E171 整表跑的那棵冻住的树 `/tmp/claude-1000/e171-s4/frozen-tree/`（执行员第七、八段报告里的快照摘要 `d1586465…`）。E171 的 37 行读数、导出目录 `research/target/e171-dumps/` 里的每一态都是它写出来的。
- 「今天的树」= 这一轮的冻结副本 `/tmp/claude-1000/e171-conclusion-r1/tree/`（正文第二节点名的那一份，方案按它谈）。
- 两棵树的 `crates/` 有 83 个文件不同（`diff -rq` 现数），`singlefs-core` 的 `transaction.rs` 差 2000 行、`recovery.rs` 781 行、`mount.rs` 446 行。

## 复跑与文件摘要

模型目录 `research/prompts/e171-conclusion-r2-opus-model/`。复跑（只写给定的工作目录；编译与探针都经 `research/scripts/run-with-memory-cap.sh`，线程数默认 15；从零编要十几分钟，主 agent 用后台跑）：

```
bash research/prompts/e171-conclusion-r2-opus-model/tools/rerun.sh /tmp/claude-1000/<新的工作目录> 15
```

它依次：① 两棵树各拷一份副本，编三个 harness 档自写探针（`opus_r2_drive_probe`、`opus_r2_judge_probe`、`opus_r2_history_probe_s4`），在 s4 树上另把第 125、34、23 行的变异各打一次、编、还原；② 流二驱动四遍，与 E171 导出的 `ops.tsv` 逐条比；③ 两种「已返回的调用丢了」写流喂冻结 O3，再逐态跑实现恢复、O2、O1-崩溃判官、RefFS、两次可写挂载；④ 单镜像谓词 W 在基线与 37 行上数；⑤ 第 242、269、270 行与基线的批 1、批 2 态上两次可写挂载、崩溃注入第二截那一道拒绝判定（两棵树各一遍）；⑥ 第 23 行在 s4 树上建 233 个 inode。末尾逐个与 `out/` 下同名文件 `cmp`，没有输出就是逐字节相同。

每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里）：

```
02befec7f52278bfdb15188d72c481f23029a6f634e7f55beb33088e6cec2eef  ./apply_mutation.py
03079abeb01fd67da9f29145d97609dcf9b1a7f70514dc04d5bf85fbc5e25e98  ./compare_ops.py
b9f10f569edbbd7c98ff01b8b5e2ad38ca3243904640b1c9bf709bdf7794bfb4  ./join_witness_and_mount.py
baab80699cba0d63ad43c2ec8b6a52edfbccc26c31878d920830a9ec76350935  ./o3_ack_shapes.py
c87e801febf0e939a9f15cfbd8599e0f93a1aa52d6293c79bc0e5d78019499c6  ./opus_r2_drive_probe.rs
1ac48ec2cb0c0af2b7397209cd3e01079f9a9d421da42402bf7e6f60f5c9fa89  ./opus_r2_history_probe_s4.rs
f2112261068ad6853d3d80c6172d6f01212109b22fb3380cfcb30da79de5f352  ./opus_r2_judge_probe.rs
6432bcdf4011b93410db4fe8d883b385c2a7e8b9d6ca4b470fc1891885127e3c  ./out/compare-ops.out
60de00cf7a6685eaab189be40c16301cc82df2a1e13c048622eff61fcc346504  ./out/drive-base.tsv
9bae46fe28cd8ab214a29112d65f1eca1bb46a8c5cb7ad4e51601a6285872619  ./out/drive-s4-base.tsv
e9f58db61bd843b86c51287864bd45f225a21090e19780ec5e870d7f167b1e9f  ./out/drive-s4-m125.tsv
8f4bc2e90f365a9143563ab779b37831a03f9ebdaa26267657a1636b69277e7a  ./out/drive-s4-m34.tsv
7b87c8fd4cd9c5737c22f61a076126472e3df5d99b9563d779bc476db9c574fb  ./out/hist-s4.tsv
4a7c83bc214ad85eab8d2ba39b85071ad780de61cd60109967fd564f759ebb9b  ./out/judge-judge_s4_v2-content.tsv
b01477fadbaa40e4190848423c17dff81bc5c8acfd757f6fdd7a5ad9d2304a47  ./out/judge-judge_s4_v2-warmup.tsv
4a7c83bc214ad85eab8d2ba39b85071ad780de61cd60109967fd564f759ebb9b  ./out/judge-judge_v2-content.tsv
b01477fadbaa40e4190848423c17dff81bc5c8acfd757f6fdd7a5ad9d2304a47  ./out/judge-judge_v2-warmup.tsv
6e621048ec3ad8c5f0a2568abea2c42838d8ad28514e260e1fbff4b91a706933  ./out/judge-summary.out
e0d6d2d3f648302c1a4d3d198df5f8ca932f91c88ae5be2e4b636e0f577e618c  ./out/refusal-judge-summary.out
d50a33e17f643ef8ae634ecf03dcc61c1c6d94008bfbbec48f0a7284fb6cd142  ./out/shape-content-o3-verdicts.tsv
624269a615f12539b7500f28ade7ea2ba5c4aea6b31e54a5d00883005f9e3fe3  ./out/shapes.out
2179634646c83752538145a0122637222c49782c63c08ac2587cec3615490ba7  ./out/shape-warmup-o3-verdicts.tsv
a8f70386ad04c51637672a14e02bc8adb4143395358c8b02fadb3a25a95dc995  ./out/witness-consistency-all.out
4f4174f1a80ace24d713dadbbaaebaf2cd82bf177354769096ee9836b5b76ea0  ./out/witness-per-state.tsv
5efe3b5b07c236dd15a7e9ad0247f8d2151470e68098f8c405f991c67663f29b  ./out/witness-vs-mount.out
d354b2a545d34237725a905231f6d70fa7f7a8ef3dc9de0aa9db3fc5e5cd6abb  ./run_judge_batch.sh
1529423c772de5dcf84f6e5f470dd6a90d43854f931e7891f94312c6cf770f42  ./summarize_judge.py
30c62b6e166f7becc96068fd5328b852ffece00421215ceaf01e02d584511bb3  ./summarize_refusals.py
abcb5b0e98351387553966254807c06b384fbd69c48ce9cc1b4b6b5ec922f17f  ./tools/rerun.sh
bc072b7804b635dd2f1e1380a99e74a5ddc1b120387f726d52af1196f661304f  ./witness_consistency.py
```

`judge-judge_*` 四份原始逐态文件是上一版判官探针（不带第二截判定那一列）跑的；复跑用现在这一版，多一列，所以只比它们的汇总 `judge-summary.out`（我在草稿里用现在这一版重跑过带内容那一形，汇总与上一版逐行同）。`apply_mutation.py` 原样拷自 e171-conclusion-r1 攻方模型目录。

## 各格判定一览

| 格 | 对象 | 攻方判定 | 依据 |
|---|---|---|---|
| H2 | 「只让 O3 红」那一形在 RefFS 上 | r1 那一形（暖机第二次空发布返回之后丢）：RefFS 27/27 态绿、O1-崩溃判官绿（这次在函数上跑了）、O2 绿，只 O3 红成立；同一种写流换到带内容的发布（新池新建文件）上，O3 红的 27 态 RefFS 全红，另有 9 态 RefFS 红而 O3 判在。O3 独有的只剩「丢的那一版内容看不出」那一类 | 量过（两棵树结果逐字节同） |
| H2 | E171 的 O1-崩溃判官一列 | 不是 D13 表里 O1 那一格：表把「崩溃之后允许看到的集合」归 RefFS，E171 登记照派发提示换成模型的崩溃判官、不接 RefFS。两者下界不同（盘上已持久的最新根 / 最后一次返回成功的发布），「已确认的丢了」那一类只有 RefFS 那一边判 | 读码 + 上一格量过 |
| H2 | 写序义务今天归谁判 | 不是「三条都不判」：O2 的 I-7.12 判抬 F 那一串的写序（第 102、265 行就是它抓的）；C577 点名测试判第 270 行；系统配置见证值那一侧（第 242、269 行）谁都不判——记录核对器两条判据不碰系统配置，崩溃注入第二截把这些拒可写判成合法 | 量过（第二截判定 61/61 legal）+ 读码 |
| H1 | R1（第 242、270 行按已定项 12 合法） | 打中一半：已定项 12 只管「根 + journal」，系统配置槽的合法性 kb 按写序判（I-7.12 自己写「任何合法状态上……」）。照 R1 字面，只靠 O2 的第 86、102、265 行（只红 I-7.12）成了在合法态上的误红；不照字面，第 242、269 行的态不合法。第 242、269 行的这些态在没有任何读故障时让 D23 已定项 14 的见证判据为真，池子断一次电就永远拒可写 | 量过（谓词 W：基线 0 / 576，37 行里只第 242 行 15 态、第 269 行 63 态；与实现两次挂载逐态对上）|
| H1 | R2（第 242、269 行归「取样点（历史）不敏感」，补 O1 历史） | 打中：「崩溃之后可写挂载」这段历史今天崩溃注入第二截已经有，它判拒绝用 `judge_writable_mount_refusal`，在第 242、269 行那 61 个见证拒上全判合法（基线 135 个撕裂系统配置的拒也判合法）。补历史不够，缺的是判据；第 23 行在 s4 树上重做，第一轮的读数成立 | 量过（s4 树与今天的树各一遍） |
| H1 | R4（三条按设计都不判 M4 等号） | 写宽：O1-崩溃判官 ③ 判得到 M4 落在「盘上已持久的最新根」以下的那一半（第 281 行就是这样抓到的）；盲的只是 j* 要靠记录重建、实现停在已持久根与 j* 之间那一段 | 读产物 + 读码 |
| H3 | C407「钉四列矩阵」 | 对 O3 零判别力：O3 一列 37 行 0 红，一个永远答「在」的 O3 在钉住的矩阵上一格都不翻；第一轮表里每条补 oracle 的改法一落地就翻格，门禁红在改好上 | 读产物 |
| H3 | C407「每行至少一条红 + O3 独有一形」 | 比钉矩阵合 D13「独立性要被验证」的字面，但 O3 独有那一形要取内容看不出的那种（否则带 RefFS 的列里它不独有）；E171 Q2 里已经有 O3 独有的坏镜像 29 张（P 12、B 17），不用另造 | 读产物 + 上面量过的两形 |
| H3 | G5 第 2、3、4 条文字 | 第 2 条依据漏了 Q2 的 O3 独有 29 张；第 3 条「实现自己发布过的版」写错一半（A(c) 是写过根槽的各版，窗口 [ack, begun] 才是 O3 推的那一半，新句子把它丢了；也没写系统配置槽不判）；第 4 条改写要把 RefFS 写进去（D13 自己的 O1 行已经把崩溃之后的集合交给它） | 读 kb 与码 |
| H4 | 自崩 6 行 | 第 85、235、272 行在第一次调用之前、第 6、7 行在第一次 publish 的第一条写之前就 panic，没有可判的前缀，不进分母对；第 125 行流二前 13 次调用 537 条 op 都走完才在卸载里 panic，装置对 panic 整条流不导出（对报错是截断照判）。在 s4 树上重驱：第 4 次调用之后每个全持久态 O2 判 I-3.11 红。照报错的规矩它是计入分母、O2 抓到的一行 | 量过（驱动与 E171 导出逐条对上之后） |
| H4 | 只靠一条的 23 行误红 | 照 R1 字面，第 86、102、265 行三行是误红（见 H1）；别的 20 行没找到误红 | 读产物 + 条款 |
| H4 | 第 155、161 行改触达定义 | 变的读数：D 37 → 39、只靠 O1 10 → 12、被抓 25 → 27、扩表那一张的分母 18 → 20、「关一条仍有别的腿红」2 / 25 → 2 / 27；Q1 三种读法名单不变 | 重跑 r1 正推的 `recount.py` |
| H4 | 今天的树不是 E171 跑的树 | 同一份驱动在 s4 树上逐字节复现 E171 基线流二 595 条 op，在今天的树上第 40 条 op 起就不同、回退那次多 4 条 op。E171 的行级读数是 s4 树的，「方案按今天的实现谈」时不能直接当今天实现的读数；第一轮「实现一侧」那几个数用今天的 core 挂 s4 写出的态，这次用 s4 的 core 在全部批 1、批 2 态上重做，见证类的拒同为 9 与 52 态 | 量过 |

## 一、H2：O3、RefFS 与 O1-崩溃判官的分工（攻击面 ②）

### 1. 「只让 O3 红」那一形在 RefFS 上：内容看不出的那一形只有 O3 判得到，内容看得出的那一形 RefFS 判得更严

做法（`o3_ack_shapes.py`，r1 攻方 `o3_ack_probe.py` 改参数，其余照抄）：

- 形 warmup = r1 那一形：流一，暖机第二次空发布的记录与根槽之间、轮换之后的屏障拿掉，根槽 FUA 改普通写；截到下一次调用（新池新建文件）的第一组单元写。O3 判定与 r1 的 `out/ack-o3-verdicts.tsv` 逐字节同（`cmp` 无输出）。
- 形 content = 这一轮加的：流二（E171 基线导出），第一次带文件的发布（调用 2，新池新建文件）的第 50、51、55、56 条屏障拿掉，第 52 条根槽 FUA 改普通写；截到调用 3（覆盖写）的第 57、58 条单元写。写的字节与基线逐字节相同。
- 两形都把末段合并出的 7 次写的整写子集全枚举、可撕裂的两次另加撕裂态：288 态一个不落；前面各段与基线相同，不重判。
- 逐态再喂判官探针（`opus_r2_judge_probe.rs`）：实现只读恢复（`recover`，看 journal）、O2、O1-崩溃判官（直接调 `crash_recovery_disagreement`，模型提交过的版本照两条流的调用写：warmup 是 (0,0)、(1,1)、(1,2) 无文件；content 另加 (1,3) 带第一个文件；盘上已持久的最新根逐态取 O3 版本表里落了的根槽写）、RefFS（直接调 `singlefs_reference_filesystem`，操作序列照 harness 翻译层的对应「发布第一个文件 = 建第一个文件 + 发布调用；覆盖写 = 覆盖 + 发布调用」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-harness/src/reference_filesystem_translation.rs:13`），崩溃落在最后一个操作里）、两次可写挂载。

O3 一侧原样输出（`research/prompts/e171-conclusion-r2-opus-model/out/shapes.out`）：

```
name=o3_ack_shape shape=warmup tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=27 o3_provisional_not_in=27 spec_gap=0 anomalies=1
name=o3_ack_shape shape=content tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=27 o3_provisional_not_in=27 spec_gap=0 anomalies=1
```

判官探针汇总（`out/judge-summary.out` 里 O3 判不在或 RefFS 判红的行，s4 树的 core；今天的树逐态输出与它逐字节相同，两份 sha256 见开头）：

```
name=judge_summary label=warmup-judge_s4_v2 count=3 o3_strict=not_in recover=1,1 read_back=no_file o1_crash_judge=green reffs=green o2_red=- mount1=ok unchanged_after_mount1=false mount2=ok digest_ok=true
name=judge_summary label=warmup-judge_s4_v2 count=9 o3_strict=not_in recover=1,1 read_back=no_file o1_crash_judge=green reffs=green o2_red=- mount1=refused:AgainstTheSelectedVersionsLastRecord unchanged_after_mount1=true mount2=refused:AgainstTheSelectedVersionsLastRecord digest_ok=true
name=judge_summary label=warmup-judge_s4_v2 count=15 o3_strict=not_in recover=1,1 read_back=no_file o1_crash_judge=green reffs=green o2_red=- mount1=refused:SomeSystemConfigurationSlotUnreadOrUnverified unchanged_after_mount1=true mount2=refused:SomeSystemConfigurationSlotUnreadOrUnverified digest_ok=true
name=judge_summary label=content-judge_s4_v2 count=3 o3_strict=not_in recover=1,2 read_back=no_file o1_crash_judge=green reffs=red o2_red=- mount1=ok unchanged_after_mount1=false mount2=ok digest_ok=true
name=judge_summary label=content-judge_s4_v2 count=9 o3_strict=not_in recover=1,2 read_back=no_file o1_crash_judge=green reffs=red o2_red=- mount1=refused:AgainstTheSelectedVersionsLastRecord unchanged_after_mount1=true mount2=refused:AgainstTheSelectedVersionsLastRecord digest_ok=true
name=judge_summary label=content-judge_s4_v2 count=15 o3_strict=not_in recover=1,2 read_back=no_file o1_crash_judge=green reffs=red o2_red=- mount1=refused:SomeSystemConfigurationSlotUnreadOrUnverified unchanged_after_mount1=true mount2=refused:SomeSystemConfigurationSlotUnreadOrUnverified digest_ok=true
name=judge_summary label=content-judge_s4_v2 count=1 o3_strict=in recover=1,2 read_back=no_file o1_crash_judge=green reffs=red o2_red=- mount1=ok unchanged_after_mount1=false mount2=ok digest_ok=true
name=judge_summary label=content-judge_s4_v2 count=3 o3_strict=in recover=1,2 read_back=no_file o1_crash_judge=green reffs=red o2_red=- mount1=refused:AgainstTheSelectedVersionsLastRecord unchanged_after_mount1=true mount2=refused:AgainstTheSelectedVersionsLastRecord digest_ok=true
name=judge_summary label=content-judge_s4_v2 count=5 o3_strict=in recover=1,2 read_back=no_file o1_crash_judge=green reffs=red o2_red=- mount1=refused:SomeSystemConfigurationSlotUnreadOrUnverified unchanged_after_mount1=true mount2=refused:SomeSystemConfigurationSlotUnreadOrUnverified digest_ok=true
```

读法：

- warmup 一形：27 态只 O3 红。O1-崩溃判官这次是在函数上跑的（r1 是按判据推的绿），绿：落到的 (1,1) 是模型提交过的、不比盘上已持久的最新根旧。RefFS 绿：暖机写的根在 RefFS 里不是一个操作，翻译层把它映到现行版本（「实现写下的每一条根（发布调用、回退、可写挂载的写行与暖机、抬 F、卸载）映到写它时的现行版本」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-harness/src/reference_filesystem_translation.rs:9`）），丢的那一版与留下的那一版读出来都是「没有文件」。
- content 一形：O3 红的 27 态 RefFS 全红（下界 = 最后一次返回成功的发布，「lo = 崩溃之前最后一次返回成功的发布调用、fsync 或回退的末尾」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-reference-filesystem/src/lib.rs:209`），读出「没有文件」不在 {第一个文件} 里）。O1-崩溃判官同样绿，O2 同样绿。
- 另有 9 态 RefFS 红而 O3 判「在」：这 9 态调用 3 的写一条都没落，O3 按「落了的最晚一条写」定已确认（「if version.return_point is not None and version.return_point <= largest_index:」（`/tmp/claude-1000/e171-conclusion-r1/tree/research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:97`）），认为发布还没返回；RefFS 的翻译层按「关上这一段的那道屏障由哪一步发出」归属崩溃时刻（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-harness/src/reference_filesystem_translation.rs:672`），这一段由调用 3 关，发布算已返回。两边的归属规则不同，这是 C447 要记的那种逐态分歧（我的 RefFS 输入 hi 是手给的，按这条归属规则推它与翻译层给的相同，没经翻译层跑）。
- 结论：「O3 有一类别人判不到的」只对「丢的那一版内容看不出」成立（暖机、抬 F 与卸载推的空发布、零单元发布）。丢的那一版带内容时，D13 指定的崩溃之后那一格（RefFS）判得到，而且比 O3 判得严。第一轮 G3 的「已确认的调用丢了」整类归 O3 独有，写宽了。

### 2. E171 的「O1-崩溃判官」一列不是 D13 表里 O1 的那一格

- D13 表 O1 行：「无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3）」（`.claude/kb/decisions/13-验证路线.md:161`）。E171 登记明写换掉了它：「本登记照派发提示把崩溃判官算进 O1」（`research/prompts/e171-preregistration.md:36`）；RefFS「本实验不用它」（`research/prompts/e171-preregistration.md:193`）。
- 两者的下界不同：崩溃判官是「盘上已持久的最新根」（「if chosen < persisted {」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-harness/src/model.rs:2639`）），RefFS 是最后一次返回成功的发布（上一段）。所以「已确认而丢了」整类在 E171 的四列里只剩 O3 判，在 D13 字面的三条里由 O1 那一格的 RefFS 判（内容看得出时）。
- 对 E171 的读数：37 行里没有「已确认的带内容发布丢了」那一类（r1 现数：拿掉屏障的变异只进了第 270 行，那一行根槽 FUA 仍在返回之前），所以换成 RefFS，Q1、Q3 的名单不会因这一类变（推的，RefFS 没在 37 行上跑）。变的是两句话：第一轮判决「O3 独有的那一类」与 C407 改写里「表加 O3 独有的那一形」都要写明列里有没有 RefFS。

### 3. 写序义务今天归谁判：不是「三条都不判」，系统配置见证那一侧才是谁都不判

第一轮判决写「写序义务今天三条都不判（O1 不碰盘、O2 判单镜像、O3 按已定项 12 判 replay 结果）」（`research/prompts/e171-conclusion-r1-main-verification.md:26`）。现查：

- O2 判一部分写序：I-7.12 的判据就是从写序推出来的单镜像性质，「所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里」（`.claude/kb/invariants.md:66`）。E171 里第 86、102、265 行只红它（实验页 `.claude/kb/experiments/171-三oracle合谋测试.md:108`、`:112`、`:194`）。
- 第 270 行（轮换之后那道屏障）：点名测试 `a_publish_returns_only_after_the_system_configuration_rotation_is_durable`（checker 档，`crates/mutations.tsv` 冻结副本第 1307 行那一条变异点名它）；r1 攻方给的「见证值 ≥ 已确认那一版轮换的 tail」是 O3 可加的一句。
- 第 242、269 行（系统配置见证值超前于根或取错值）：记录核对器在两行上都不红（产物 `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` 第 4537、4538 行 `layer0_or_record_checker_red=false`）；崩溃注入第二截有「崩溃之后可写挂载」，但它判拒绝合不合法时只在崩溃镜像上复算判据：「判一次拒可写合不合法。拒的判据在崩溃镜像（`crash_image`）上自己复算」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:1180`），合法的只计数（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2174`）。我在同一批态上调它（第二节第 2 段贴原样输出）：第 242、269 行的 61 个见证拒全判 legal。
- 所以没人判的是：「没有任何读故障的崩溃态，不许让 D23 已定项 14 那条见证判据为真」。它该进 D13 已定项 6 表的「验不到什么」或记一笔欠账；补法见第二节。

## 二、H1：第一轮改判够不够得着（攻击面 ①）

### 1. R1：已定项 12 的读法若成立，只靠 O2 的三行变成误红；不成立，第 242、269 行的态就不合法

- 已定项 12 的定案只说「根 + journal 任意前缀，replay 之后合法」（`.claude/kb/decisions/16-发布语义.md:264`），管的是根与 journal 两样。系统配置槽是盘上第三样，恢复读它（择几何、可写挂载的见证值），已定项 12 一个字没提它。
- 同一份 kb 里系统配置槽的合法性是按写序判的：已定项 7「它是发布序列的最后一步、进层 0 枚举，一次发布的段序列因此唯一」（`.claude/kb/decisions/16-发布语义.md:181`）；I-7.12 的判据就是从「抬 F 那一串先写系统配置」推的，并写「所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里」（`.claude/kb/invariants.md:66`），同一格还写「条款仍是主 agent 2026-09-26 按 SysPre 的写序推的，没三方」（`.claude/kb/invariants.md:66`）。
- 两种读法各自的后果：
  - 照 R1 的字面（合法 = replay 之后合法）：第 86、102、265 行只红 I-7.12，红的态都是「根 + journal」合法、只有系统配置的 F 落后于根的态——在 R1 的口径下是合法态上的误红。Q3 只靠 O2 的 13 行减三行（10 行），被抓行 25 → 22。
  - 照 kb 的写序口径（I-7.12 那样）：第 242 行「系统配置写先于根槽 FUA 落盘」的态与 I-7.12 抓的是同一种形（系统配置超前于根），不合法；R1 把这两行判成合法站不住。第 270 行不同：它的态单看字节是正确实现崩在返回之前也写得出的，不合法只是相对「已经返回」而言（同 O3 的 ack 窗口），这一半 R1 判对了。
- 两种读法不能同时成立。我取第二种（kb 两处正文与已实现的 I-7.12 都站这边），并量了第 242、269 行的态到底坏在哪。

谓词 W（`witness_consistency.py`，零轮）：照 D23 已定项 14 第 392 行可写挂载那一段的取法，c_见证 = 自证过的系统配置槽 journal_tail 的最大值；所选那一版（冻结 O3 重放之后落到的那一版）那次发布带末条标志的记录读得出 ⇒ c_见证 > 它的计数器为真；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, txg) 大于所选那一版为真；两条都读不出按真。W 为真的态，正确实现的可写挂载会把它读成「见证过的更新的一版读不出」。在 E171 导出的批 1、批 2 态上数（`out/witness-consistency-all.out`，基线两条流与计入分母的 37 行，76 条流，19 361 态），不为 0 的只有这四条，其余 72 条（含基线两条）都是 `w_true=0`：

```
name=witness_consistency label=base1 states=60 w_true=0 no_landing=0 first_states=-
name=witness_consistency label=base2 states=516 w_true=0 no_landing=0 first_states=-
name=witness_consistency label=r242new states=81 w_true=5 no_landing=0 first_states=13,16,17,18,62
name=witness_consistency label=r242fix states=649 w_true=10 no_landing=0 first_states=13,16,17,18,105,108,109,110,454,491
name=witness_consistency label=r269new states=60 w_true=8 no_landing=0 first_states=2,5,6,7,8,9,10,11
name=witness_consistency label=r269fix states=516 w_true=55 no_landing=0 first_states=2,5,6,7,8,9,10,11,74,77,78,79
name=witness_consistency label=r270new states=56 w_true=0 no_landing=0 first_states=-
name=witness_consistency label=r270fix states=460 w_true=0 no_landing=0 first_states=-
```

校验（第二条路子不共用代码：W 用冻结 O3 的 Python 读盘与重放，挂载是 s4 树 core 的 `mount_writable`，Rust）：同一批态上逐态挂两次（`out/witness-vs-mount.out`，W 为真的行全贴；W 为假的 16 行里没有一行是见证类的拒）：

```
name=witness_vs_mount label=r242fix w=True mount1=refused:AgainstTheRecordAtTheWitnessedCounter unchanged_after_mount1=true mount2=refused:AgainstTheRecordAtTheWitnessedCounter count=3
name=witness_vs_mount label=r242fix w=True mount1=refused:AgainstTheSelectedVersionsLastRecord unchanged_after_mount1=true mount2=refused:AgainstTheSelectedVersionsLastRecord count=3
name=witness_vs_mount label=r242fix w=True mount1=refused:SomeSystemConfigurationSlotUnreadOrUnverified unchanged_after_mount1=true mount2=refused:SomeSystemConfigurationSlotUnreadOrUnverified count=4
name=witness_vs_mount label=r242new w=True mount1=refused:AgainstTheRecordAtTheWitnessedCounter unchanged_after_mount1=true mount2=refused:AgainstTheRecordAtTheWitnessedCounter count=3
name=witness_vs_mount label=r242new w=True mount1=refused:SomeSystemConfigurationSlotUnreadOrUnverified unchanged_after_mount1=true mount2=refused:SomeSystemConfigurationSlotUnreadOrUnverified count=2
name=witness_vs_mount label=r269fix w=True mount1=refused:AgainstTheRecordAtTheWitnessedCounter unchanged_after_mount1=true mount2=refused:AgainstTheRecordAtTheWitnessedCounter count=3
name=witness_vs_mount label=r269fix w=True mount1=refused:AgainstTheSelectedVersionsLastRecord unchanged_after_mount1=true mount2=refused:AgainstTheSelectedVersionsLastRecord count=40
name=witness_vs_mount label=r269fix w=True mount1=refused:SomeSystemConfigurationSlotUnreadOrUnverified unchanged_after_mount1=true mount2=refused:SomeSystemConfigurationSlotUnreadOrUnverified count=9
name=witness_vs_mount label=r269fix w=True mount1=refused:Undecidable unchanged_after_mount1=true mount2=refused:Undecidable count=3
name=witness_vs_mount label=r269new w=True mount1=refused:AgainstTheRecordAtTheWitnessedCounter unchanged_after_mount1=true mount2=refused:AgainstTheRecordAtTheWitnessedCounter count=3
name=witness_vs_mount label=r269new w=True mount1=refused:SomeSystemConfigurationSlotUnreadOrUnverified unchanged_after_mount1=true mount2=refused:SomeSystemConfigurationSlotUnreadOrUnverified count=2
name=witness_vs_mount label=r269new w=True mount1=refused:Undecidable unchanged_after_mount1=true mount2=refused:Undecidable count=3
```

- 实现的见证类拒（三种理由）一共 61 态（第 242 行 9、第 269 行 52，与 r1 用今天的 core 量的 9 / 276 与 6 + 46 同数），全落在 W 为真的态上；W 为真而没走到见证判据的 17 态都先被撕裂系统配置那一种拒（C331 已知形，基线上同样有）。基线 W 为 0。
- 拒是永久的：第一次拒之后盘上逐字节不变，第二次照样拒。也就是一次普通断电（不需要任何读故障）之后，这个池每次可写挂载都拒，只剩只读挂载。按已定项 12 这些态「replay 之后合法」，按 D23 已定项 14 它们等于「出了一次读故障」。R1 的「两行的态合法」只看了根 + journal。

### 2. R2：第 242、269 行补「崩溃之后可写挂载」这段历史不够，这段历史今天已经有

第一轮改法表把第 242、269 行补在 O1 的历史里（`research/prompts/e171-conclusion-r1-main-verification.md:71`：「模型给答（先与 C331 的取舍对齐）」）。崩溃注入第二截已经在崩溃镜像上做可写挂载，被拒且理由在 D23 那张单子上的，交 `judge_writable_mount_refusal` 判，不交模型（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2136`）。我在同一批态上照第二截的传法调它（盘上变没变照实交；只读读出交「在允许集合里」——这两行在 E171 里 O1-崩溃判官 0 态红），原样输出（`out/refusal-judge-summary.out`，s4 树 core，被拒的行全贴、尾巴的 `unchanged_after_mount1=true` 每行都有，省了）：

```
core=s4 label=base1 mount1=refused:SomeSystemConfigurationSlotUnreadOrUnverified second_cut_verdict=legal:NewerPublishWitnessedBySystemConfiguration count=20
core=s4 label=base2 mount1=refused:SomeSystemConfigurationSlotUnreadOrUnverified second_cut_verdict=legal:NewerPublishWitnessedBySystemConfiguration count=115
core=s4 label=r242fix mount1=refused:AgainstTheRecordAtTheWitnessedCounter second_cut_verdict=legal:NewerPublishWitnessedBySystemConfiguration count=3
core=s4 label=r242fix mount1=refused:AgainstTheSelectedVersionsLastRecord second_cut_verdict=legal:NewerPublishWitnessedBySystemConfiguration count=3
core=s4 label=r242new mount1=refused:AgainstTheRecordAtTheWitnessedCounter second_cut_verdict=legal:NewerPublishWitnessedBySystemConfiguration count=3
core=s4 label=r269fix mount1=refused:AgainstTheRecordAtTheWitnessedCounter second_cut_verdict=legal:NewerPublishWitnessedBySystemConfiguration count=3
core=s4 label=r269fix mount1=refused:AgainstTheSelectedVersionsLastRecord second_cut_verdict=legal:NewerPublishWitnessedBySystemConfiguration count=40
core=s4 label=r269fix mount1=refused:Undecidable second_cut_verdict=legal:NewerPublishWitnessedBySystemConfiguration count=3
core=s4 label=r269new mount1=refused:AgainstTheRecordAtTheWitnessedCounter second_cut_verdict=legal:NewerPublishWitnessedBySystemConfiguration count=3
core=s4 label=r269new mount1=refused:Undecidable second_cut_verdict=legal:NewerPublishWitnessedBySystemConfiguration count=3
```

（上面每行前面是 `name=refusal_judge `，后面的 `mount2=` 与第一次相同；撕裂系统配置那一种拒在第 242、269、270 行上的 245、135、250 态同样全判 legal，见同一文件。今天的树的 core 在第 242、269 行上逐行同数。）

- 所以第 242、269 行不是「取样点（历史）不敏感」：带这段历史的判官今天就有，在这些态上判合法。缺的是判据：第二截「拒得合法」那四条里没有一条问「这个崩溃态没注入读故障，判据凭什么为真」。
- 第 23 行在 s4 树上重做（r1 用的是今天的 core）：基线建 233 个 inode 0 条红，打了第 23 行变异的 core 建 233 个 inode 红 I-1.1、I-9.4、I-9.12，与 r1 逐字同（`out/hist-s4.tsv` 第 3、6 行）。这一行的 R2 站住。

### 3. R4：「三条按设计都不判 M4 等号」写宽了一半

- 判决把第 27、112、277 行归「三条按设计都不判实现恢复的 M4 等号」（`research/prompts/e171-conclusion-r1-main-verification.md:37`）。O1-崩溃判官的 ③ 判「不许恢复到比它旧的根」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-harness/src/model.rs:2643`），这正是 M4 的一半：实现恢复落到已持久最新根以下就红。第 281 行就是这样被抓的（「比盘上已持久的最新根槽旧」（`.claude/kb/experiments/171-三oracle合谋测试.md:188`））。
- 三条都不判的是另一半：j* 比已持久的最新根新、要靠记录重建，而实现停在两者之间。第 27、112、277 行都在这一半。改写成「三条不判 M4 里『靠记录重建的那一段』」才准。
- 未触达行里按名字是「恢复少施加」的：第 213 行「前一条不在盘上也断前缀」（产物第 3447 行），未触达（第 3448 行 `touched=false`），要「前一条记录不在盘上」的态才分得开，两条流里没有（推的，没跑）。

### 4. R3、R5 没攻

第 15、34 行的 D3 已定项 10 ③ 与里程碑验收句两条，这一条腿没找到反面的事实，没打。

## 三、H3：G5 的文字改法与 C407 两种改写（攻击面 ③）

### 1. C407 的两种改写各自判得出什么

第一轮留下的两种：「改写成什么（正推：钉四列矩阵；攻方：只钉「每行至少一条红」、表加 O3 独有的那一形、关一条降为测量）」那一句（`research/prompts/e171-conclusion-r1-main-verification.md:64`）。

- **钉四列矩阵**：对 O3 零判别力。E171 计入分母的 37 行上 O3 一列一格都没红：「E7RESULT name=q1_column_red_rows counted_rows=37 O1=12 O1-对拍=11 O1-崩溃判官=4 O2=15 O3甲=0 O3乙=0」（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out:4525`）。一个对每个态都答「在」的 O3，在钉住的行矩阵上一格都不翻，门禁照绿。它钉的是「这张表今天的读数不变」，不是「每条 oracle 各判得到别人判不到的东西」。另一面：第一轮改法表里每一条补 oracle 的改法（O3 加 W、O2 加头 ID 核对……）都是为了把某个全放行格翻红，一落地门禁就红在改好上，要重钉。
- **只钉「每行至少一条红」并补 O3 独有的一形**：照 D13 已定项 6「独立性要被验证」的字面更合——它要的是每一列各有一行只有它红（不被别的列包住），关掉哪一列，那一行就全放行。三处要先定：
  1. 12 行全放行行今天就违反「至少一条红」，做成门禁当天就红；要么逐行挂补 oracle 的欠账、在表里标成已知例外，要么不把它们放进表。
  2. O3 独有那一形要取「丢的那一版内容看不出」的那种（暖机空发布那一形）；带内容的那一形，列里只要有 RefFS 就不是 O3 独有（第一节第 1 段）。它是三处屏障 / FUA 一起拿掉的写流，`crates/mutations.tsv` 里没有单处变异能造出来（r1 推的），要另写一个坏实现。
  3. 其实 E171 Q2 里已经有 O3 独有的坏镜像：「o3_only_strict=12」（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out:4759`）、「o3_only_strict=17」（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out:4760`）。C407 原文就允许「一份已知坏镜像（或坏实现）」，O3 那一列可以先用这 29 张，不必等新坏实现。
- 代价（算术，没跑）：整表一行的墙钟约 47.6 秒（「rows=117 table_wall_ns=5569579167726」（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out:286`），两条线道），钉 37 行约 29 分钟；每列一行只靠它的行（例：第 32、233、8 行）加 O3 那一形，4 行约 3 分钟。坏镜像那几张不经逐行驱动，代价没量。
- 「关一条仍有别的腿红」降为测量：同意。它要的是冗余（两条都判得到同一个坏），Q3 的 23 / 25 只靠一条说的是三条的抓取几乎不相交，是分工，不是缺陷。

### 2. G5 四条文字改法

1. **关系**（「支撑『独立性要被验证』、不支撑也不推翻『三条互相独立』」）：没找到反面事实，不攻。
2. **依据**（引 Q1 存在性、Q3「23 / 25 只靠一条」、O3 在这张表上 0 红且表里没有它独有的那一类）：写窄了。漏了 Q2 里 O3 独有的 29 张坏镜像（上一段第 3 点）。只引「O3 0 红」读起来像 O3 没有独有的判别力，而 Q2 的读数说它有，只是不在行表那一类坏实现上。
3. **O3 行「判什么」窄化成「规约的恢复规则下、实现自己发布过的版的集合」**（`research/prompts/e171-conclusion-r1-main-verification.md:61`）：写错一半。
   - A(c) 是录制流里写过根槽、流序落在 [ack(c), begun(c)] 的各版，包含还没返回的那一次；「发布过」读起来是「已确认的」。
   - O3 自己从 kb 推出来的那一半恰是这个窗口（它是 O3 独有判别力的来处，第一节），新句子把它丢了。
   - 没写系统配置槽：O3 只拿它择槽与几何，见证值不判（第 242、269、270 行全放行的一半原因）。
   - 已确认按「落了的最晚一条写」定，与 RefFS 按「关段的那一步」定不同（第一节那 9 态）。
   - 我给的写法（零轮）：「按规约的恢复规则（择系统配置、择根、前缀五条）从镜像恢复出的那一版，要落在录制流里写过根槽、流序在 [已确认, 已开始] 之间的某一版上；每一版的内容取实现写的根，已确认按崩溃态里落了的最晚一条写定；系统配置槽除择槽与几何外不判」。
4. **「本工程功能正确性的全部故事就是这三个 oracle」**（`.claude/kb/decisions/13-验证路线.md:167`）：要改，但理由比第一轮给的更直接——同一张表的 O1 行已经把崩溃之后允许看到的集合交给 RefFS（`.claude/kb/decisions/13-验证路线.md:161`），RefFS 是已定项 3 的另一样东西，这一句与 D13 自己的表就不一致，不必等 E171。改写要把 RefFS（O1 的崩溃那一半）点名写进去，再写点名测试、E165 与 Verus。

### 3. C407 改写的判别力，换成 D13 字面的三条时会不会变

照 D13 的表，O1 一列应是「模型对拍 + RefFS」，不是「模型对拍 + 模型的崩溃判官」。E171 的四列换成 D13 字面的列之后，「O3 独有」只剩内容看不出那一类；C407 门禁的列要先定是哪一种，否则「补 O3 独有一形」的那一行在一种列法下只红 O3、在另一种下 O3 与 RefFS 都红。

## 四、H4：第一轮两腿都没碰过的（攻击面 ④）

### 1. 自崩 6 行：五行没有可判的前缀，第 125 行有 13 次调用的前缀被整条丢掉

装置的写法：「驱动怎么结束的：每一步都成功；某一步交回错误（这条流在那里截断，仍照常判）；某一步 panic（实现自崩，这条流不导出）。」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1116`）。报错截断照判，panic 整条不导出，两种结局待遇不同。逐行看导出目录：

| 行 | panic 在哪 | 之前有没有可判的前缀 |
|---|---|---|
| 85、235、272 | 第一次调用之前，core 自己的字段表断言：「assertion `left == right` failed: 系统运行配置写出的字节数与字段表不符」（`research/target/e171-dumps/rows-s4/row-85/device.err:3`），另两行同形（`research/target/e171-dumps/rows-s4-f4/row-235/device.err:3`、`research/target/e171-dumps/rows-s4-f4/row-272/device.err:3`）；三行两条流的 `drive_ns=0` | 没有。效果同编不过 |
| 6、7 | 第一次 publish，第一条写之前：「2	publish	22	22	panic」（`research/target/e171-dumps/rows-s4/row-6/dump/new_pool_file_creation/call-outcomes.tsv:3`，第 7 行同） | 只有取号、暖机，写流与基线同 |
| 125 | 流二第 14 次调用（卸载）：「13	publish	498	537	ok	agrees」（`research/target/e171-dumps/rows-s4-f4/row-125/dump/fixed_script_through_e_then_unmount/call-outcomes.tsv:14`），「14	unmount	537	537	panic	not_modelled」（同一文件第 15 行）；流一跑完、判过（产物第 2042 行 `row_o3 row=125 status=ran`） | 有：537 条 op、13 次调用 |

所以前五行不进分母是对的（第 85、235、272 行与编不过同形；按登记 5.2 编不过要补位，这三行没补，表因此少了三行——推的，补不补归主 agent）。第 125 行不一样，我在 s4 树上把它重驱了一遍。

先核驱动（`opus_r2_drive_probe.rs`，照装置流二的调用序列，内存稀疏盘、不建镜像文件）。原样输出（`out/compare-ops.out`）：

```
name=compare_ops label=s4base dump=baseline-run1/fixed_script_through_e_then_unmount probe_ops=595 dump_ops=595 compared=595 identical_prefix=595 differing_in_compared=0 first_difference=-
name=compare_ops label=s4m34 dump=rows-s4/row-34/dump/fixed_script_through_e_then_unmount probe_ops=595 dump_ops=595 compared=595 identical_prefix=595 differing_in_compared=0 first_difference=-
name=compare_ops label=s4m125 dump=baseline-run1/fixed_script_through_e_then_unmount probe_ops=537 dump_ops=595 compared=537 identical_prefix=108 differing_in_compared=117 first_difference=108:mine=0/write/824295424/16384:theirs=0/write/824295424/16384
name=compare_ops label=s4m125 dump=rows-s4/row-34/dump/fixed_script_through_e_then_unmount probe_ops=537 dump_ops=595 compared=537 identical_prefix=108 differing_in_compared=131 first_difference=108:mine=0/write/824295424/16384:theirs=0/write/824295424/16384
name=compare_ops label=base dump=baseline-run1/fixed_script_through_e_then_unmount probe_ops=599 dump_ops=595 compared=595 identical_prefix=40 differing_in_compared=405 first_difference=40:mine=0/write/823296000/16384:theirs=0/write/823296000/16384
```

- 前两行：驱动在 s4 树上逐字节复现 E171 的基线流二（595 条 op，盘、种类、偏移、长度、内容 sha256 全同），打第 34 行变异也逐字节复现第 34 行的导出——驱动可信。
- 第三行：第 125 行在卸载之前的 537 条 op 里，从第 108 条（第 4 次调用可写挂载里）起与基线不同，117 条不同；与第 34 行也不同（第四行），不是同一种坏法。

每次调用之后对「到这一刻的全持久态」跑 O2（`out/drive-s4-m125.tsv`，前 3 次调用 `o2_full_persisted=-`）：「s4m125	call	4	mount_writable	92	179	ok	o2_full_persisted=I-3.11」（`research/prompts/e171-conclusion-r2-opus-model/out/drive-s4-m125.tsv:5`）……「s4m125	call	13	publish	498	537	ok	o2_full_persisted=I-3.11」（`research/prompts/e171-conclusion-r2-opus-model/out/drive-s4-m125.tsv:14`），「s4m125	call	14	unmount	537	537	panic:attempt_to_subtract_with_overflow	o2_full_persisted=not_judged」（`research/prompts/e171-conclusion-r2-opus-model/out/drive-s4-m125.tsv:15`）。

- 判：照装置对报错的规矩（截断照判），第 125 行是触达的、计入分母、O2 在第 4–13 次调用之后的全持久态上都红 I-3.11 的一行。D 37 → 38；它进 Q3 哪一格要看 O1-崩溃判官与 O3 在取样态上红不红（O1-对拍 13 次调用里第 2–13 次都是 `agrees`，取样态我没造，推的：多半只靠 O2）。Q1 的名单不变。
- panic 本身不是三条的判据：O1-对拍不比 panic（卸载 `not_modelled`），E171 的矩阵里这一行四列全是 quiet（产物第 2043、2044 行）。release 编法开着溢出检查（`research/prompts/e171-conclusion-r1-opus-model/workspace-Cargo.toml` 第 20 行 `overflow-checks = true`），所以这一行自崩是编法的性质：关掉溢出检查的编法里它不 panic，坏的记账照样写下去（推的）。
- 改法（零轮）：装置对 panic 与报错同样截断照判，panic 那一次调用记成「没返回」；自崩只在「第一次调用之前」时才不进分母、照编不过补位。

### 2. 只靠一条的 23 行里有没有误红

- 有三行取决于第二节第 1 段那一判：照 R1 字面，第 86、102、265 行是误红；照 kb 的写序口径，不是。
- 别的 20 行：照实验页「那一条红在哪」与「另外两条为什么看不到」两栏逐行看红的对象与条款，没找到误红。第 249 行（一个数据单元的发布也切成两条记录，O1-对拍红 jsn 3 对 4）要看新池新建文件的字节表是不是钉死了一次发布一条记录；O3 的锚点断言 A3 照一条记录写，我没去字节表里核，交主 agent。

### 3. 第 155、161 行改触达定义算进分母之后，名单之外还变什么

重跑第一轮正推的 `research/prompts/e171-conclusion-r1-sonnet-model/recount.py`（在补位产物上），原样输出：

```
include_untouched_but_red=False D=37 quiet_rows_alpha=12 15,23,27,34,42,92,112,242,269,270,271,277
   only=O1:10[32,45,108,115,172,233,237,249,250,281] O2:13[8,65,86,87,90,99,102,135,234,238,245,258,265] O3:0[]
include_untouched_but_red=True D=39 quiet_rows_alpha=12 15,23,27,34,42,92,112,242,269,270,271,277
   only=O1:12[32,45,108,115,155,161,172,233,237,249,250,281] O2:13[8,65,86,87,90,99,102,135,234,238,245,258,265] O3:0[]
categories {'counted': 37, 'self_crash': 6, 'untouched': 238}
```

名单之外会跟着变的读数（算术，从产物现数）：被抓行 25 → 27；扩表那一张的分母 18 → 20（两行序号都 ≥ 118，`name=q1_denominator_by_table table=step_4_expansion` 那一行）；C407 原改法「关一条仍有别的腿红」成立的行仍是第 11、244 两行，比例 2 / 25 → 2 / 27；Q1 三种读法的名单、O3 一列都不变。连同上一段的第 125 行一起改，D = 40。

### 4. 今天的树不是 E171 跑的那棵树

- 同一份驱动在今天的树上跑，第 40 条 op 起与 E171 基线不同，整条流 599 条（基线 595），回退那一次多出 4 条 op（上面 `compare-ops.out` 第五行；逐次调用的起止在 `out/drive-base.tsv`）。今天的树新增了 `container_tree.rs`、`deleted_inodes.rs`、`unlink.rs` 等，`transaction.rs` 差 2000 行。
- 所以 E171 的 281 行、37 行、12 行全放行，都是 s4 树上的读数。正文写「方案按 `crates/` 今天的实现来谈」：落在今天的树上的变异点不一定同义，全放行名单不能直接当今天实现的名单。要把 E171 的结论写进 D13 当今天的依据，要么写明是 s4 树的读数，要么在今天的树上重跑整表（重型，归主 agent）。
- 第一轮攻方的实现一侧数（第 242 行 9 / 276、第 269 行 6 + 46）是用今天的 core 去挂 s4 写出的态；这一次用 s4 的 core 在全部批 1、批 2 态上重做，见证类的拒同样是 9 与 52 态（第二节第 1 段）；第 23 行同。第一轮那几个数不受这一条影响。

## 打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）

「臂」在这一轮指第一轮判决给的改判与改法（R1–R7、第三节改法表的各行）。

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足判据字面的哪一个分句 | 跑前给的改法在打中的格上还中不中 |
|---|---|---|---|---|
| R1：已定项 12 只管根 + journal（二·1） | 分辨：R1 与「写序口径」两种读法在第 86、102、265 行（Q3）与第 242、269 行（Q1）上给相反的判 | 看得到：系统配置槽就在同一张镜像里，W 只读单镜像加重放（O3 已有的代码） | 正文五「攻方打中某条改判或改法」；打中的是 R1 的依据句「已定项 12 是中间状态合法性的定案」 | R1 的去向「写序本身是 O3 读流时能核的静态性质」不中第 242、269 行的坏处（坏处在见证值，不在读流）；W 中（量过：61 / 61） |
| R2：第 242、269 行「补历史」不够（二·2） | 分辨：「取样点（历史）不敏感」与「判据缺一条」两种归类 | 看得到：判拒绝的那一道在崩溃镜像上复算了判据，它缺的是「无读故障」这一个输入，崩溃注入是知道自己注没注故障的 | 同上；打中的是改法表「O1 历史加崩溃态上可写挂载」那一行 | 那一行不中（第二截已有这段历史，61 个见证拒全判 legal，量过）；中的是给第二截加一条（推的）或 O3 加 W（量过） |
| R6 / G3：「已确认的丢了」不全是 O3 独有（一·1） | 分辨：带内容与不带内容两形 | 看得到：RefFS 的输入是调用结局与读出的内容，丢的那一版带内容就分得开 | 打中的是 R6 里「攻方造了一个只让 O3 红的坏实现」所推出的「表里没有 O3 独有的那一类」的射程 | C407 改写「表加 O3 独有的那一形」：带内容那一形在有 RefFS 的列法下不中（RefFS 也红），暖机那一形中 |
| R4：M4 那一半写宽（二·3） | 不分辨第 27、112、277 行的归类，只收窄措辞 | 看得到：崩溃判官 ③ 有盘上已持久的根 | 打中的是 R4 的结论句 | 不涉及改法 |
| H4：第 125 行被自崩丢掉（四·1） | 分辨：「自崩不进分母」与「自崩照报错截断照判」两种计法在这一行上给不同的 D 与 Q3 | 看得到：装置在 panic 之前已经录下 537 条 op，只是不导出 | 登记读法 5 / 装置的「这条流不导出」，不是三条 oracle 的判据 | 不涉及第一轮改法；Q1 结论字不变 |
| H4：今天的树不是 E171 的树（四·4） | 不分辨臂（所有改判一起受影响）——照规则另立一笔账，不拿它判臂 | — | 正文二「方案按 `crates/` 今天的实现来谈」与 E171 读数的出处 | 第一轮实现一侧的数在 s4 core 上重做逐态同，不受影响 |
| H3：钉矩阵对 O3 零判别力（三·1） | 分辨 C407 两种改写 | 看得到：O3 一列在产物里 37 行 0 红 | 正文 H3「各自的判别力与代价」 | 钉矩阵不中「O3 失灵」；「每行至少一条红 + O3 独有一形」中 |

## 改法表（全部零轮：只在我的模型上量过或只是推的）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| O3 加谓词 W：无读故障的崩溃镜像上，D23 已定项 14 可写挂载那条见证判据必须为假 | 第 242、269 行（系统配置见证值超前或取错） | 量过（我的模型）：基线 0 / 576 态；37 行里只第 242 行 15 态、第 269 行 63 态为真，第 270 行与别的 34 行 0；与实现两次挂载逐态对上（见证类拒 61 态全在 W 为真里，W 为假 0 个见证类拒） |
| 崩溃注入第二截 `judge_writable_mount_refusal` 加一条：崩溃镜像没注入读故障而见证判据为真 ⇒ 拒得不合法 | 同上，在已有的「崩溃之后可写挂载」历史里 | 推的（没实现）；撕裂系统配置那一支（判不出按真，C331）要先分开，否则基线 135 个拒一起红 |
| r1 的「见证值 ≥ 已确认那一版轮换的 tail」 | 第 270 行 | r1 量过，这一轮没重做 |
| E171 的 O1 崩溃一列照 D13 表接 RefFS（或与崩溃判官并列） | 「已确认的带内容发布丢了」那一类 | 量过两形：带内容那一形 RefFS 红 36 态（含 O3 红的 27 态），暖机那一形 0 态；37 行整表没跑 |
| 装置对 panic 与报错同样截断照判；只在第一次调用之前 panic 的才不进分母、照编不过补位 | 第 125 行（及第 85、235、272 行的补位） | 第 125 行全持久态上的 O2 量过；取样态与另两列推的 |
| C407 改写：门禁钉「每一列至少一行（或一张坏镜像）只有它红」，O3 那一列先用 Q2 的 29 张 O3 独有坏镜像，再补暖机那一形 | C407、D13 已定项 6「独立性要被验证」 | 推的 |
| D13 已定项 6 O3 行「判什么」照第三节第 2 段第 3 条改；「验不到什么」加「系统配置槽的见证值」 | G5 第 3 条 | 文字 |
| E171 的结论写进 D13 时写明是 s4 树的读数，或在今天的树上重跑整表 | 四·4 | 文字 / 重型，归主 agent |

几个改法各修哪一格（「量过」贴的是本报告各节原样输出，「推的」没实现没跑）：

| 格 \ 改法 | O3 加 W | 第二截加一条 | r1 的 tail 一句 | O1 崩溃列接 RefFS | 自崩照报错判 |
|---|---|---|---|---|---|
| 第 242 行 | 中（量过） | 中（推的） | 不中（推的） | 不中（推的：这一行读出的内容与基线同） | — |
| 第 269 行 | 中（量过） | 中（推的） | 不中（推的） | 不中（推的） | — |
| 第 270 行 | 不中（量过：0 态） | 不中（推的） | 中（r1 量过） | 不中（推的） | — |
| 带内容的「已确认丢了」 | 不中（推的） | 不中（推的） | 不中（推的） | 中（量过） | — |
| 第 125 行 | — | — | — | — | 中（O2 全持久态量过） |

## 没打中的形状

- R3（第 15、34 行按 D3 已定项 10 ③）、R5（里程碑验收句不算条款）：没找到反面的事实，没打。
- G5 第 1 条（关系一句）：没找到反面的事实。
- 第 270 行在谓词 W 上：0 态（第 270 行坏在「返回之后见证值落后」，单镜像看不出，只有拿已确认那一版去比才看得出）。W 不替 r1 的那一句。
- 只靠一条的另 20 行：没找到误红（第 249 行没核字节表，交主 agent）。
- 自崩的第 6、7、85、235、272 行：没有可判的前缀，不进分母这一判没打穿。
- 未触达行里按名字是「恢复少施加」的第 213 行：没造「前一条记录不在盘上」的态，没跑。
- 用户动作放开扫：第 242、269 行崩溃之后的用户动作只有「可写挂载」一步会碰见证判据，拒在任何写之前、盘上不变，再挂一次（mount2）结局相同，所以这里的「分辨」不是装置写死后缀造出来的；第 23 行「建 233 个 inode」的个数是固定的（r1 同），没扫更少的个数（少于一片叶装得下的个数不分裂，推的）。

取样范围：E171 导出目录里基线与计入分母的 37 行，两条流的批 1、批 2（W 一共 19 361 态）；第 242、269、270 行与基线的批 1、批 2 各挂两次（s4 core 2 398 态、今天的 core 第 242、269 行 1 306 态）；两种写流各 288 态（末段全枚举）；流二驱动四遍（无崩溃态，只判每次调用之后的全持久态）；第 23 行一段短历史。

## 这条腿自己的限度

1. 全部数是副本上的：两棵树的拷贝在草稿目录里编；探针是自写的 harness 档测试目标，不是入库装置。照规则，要引进 kb 先在入库装置上重做。
2. 崩溃态的取法：E171 导出的态用的是 E171 自己的批 1、批 2（不是层 0 全量：两条流的全量是 16 777 260 与 1 662 648 564 态，各超过一段历史约 10⁶ 的上限，按规则不跑）；两种写流我自己造，末段 7 次写的整写子集全枚举、撕裂另加，一个不落，前面各段与基线同、没重判；第 125 行只判了每次调用之后的全持久态（15 个），它流二的层 0 全量与基线同一量级（约 1.66 × 10⁹，超上限），没跑。这一条腿全部原型判过的态合起来约 3 万（W 19 361 + 逐态 W 2 398 + 挂载两次 3 704、另有不带第二截判定的那一版 2 398 + 两形的判官 1 440 + O3 576），远低于 10⁷。
3. RefFS 是直接调 crate 跑的，操作序列照翻译层第 13 行的对应手写，崩溃落在最后一个操作里；没经 `reference_filesystem_translation` 整层跑（它要一段 `HistoryRun`），hi 与翻译层的归属规则一致是推的。
4. O1-崩溃判官调的是真函数，但「模型提交过的版本」是我照两条流的调用手写的（暖机两版、新池新建文件一版），不是理想模型推出来的；「盘上已持久的最新根」取 O3 版本表里落了的根槽写。
5. 第二截那一道拒绝判定，只读读出那一格我交的是「在允许集合里」（这两行 E171 里崩溃判官 0 态红），没让模型或 RefFS 现判。
6. 谓词 W 是照 D23 已定项 14 那一段文字写的，读盘与重放借冻结 O3 的代码；它与实现挂载对上（61 / 61）是两条不共用代码的路子，但两条都照同一段条款写，条款若写错两条一起错。
7. 一次抽样：「没打中」的格按规则不支撑任何结论。
8. 一次误操作：第一组判官跑在前台超过 240 秒，被转到后台；我按 pid 自底向上停了它（先停循环外壳防它再起下一件，再停正在跑的探针），停完列过一个不剩，那一组的输出全部丢弃、之后分片重跑。
9. 机器负载：编译与探针都 `nice -n 19` 跑，开跑时负载 27 上下、别的会话有 `cargo test` 在跑；耗时数不当性能数用。

## 没做什么

- 没跑任何 checker 档测试、没起 E171 装置、没在今天的树上重跑整表；没改仓里任何文件（写的只有本报告与模型目录）。
- 没在 37 行上跑 RefFS；没在第 125 行的取样态上跑 O1-崩溃判官与 O3；没实现改法表里任何一条（W 只是只读的数法）。
- 没核第 249 行的字节表；没造第 213 行要的态。
- 没判辩方那几格，没替主 agent 采纳。

## cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/e171-conclusion-r2-opus-output.md` 的末行：

```
  ✓ 核了 30 处引文，对上 30 处，没判 0 处（逐处列在上面）
```
