# m3-retire-r1 攻方腿第二次抽样（Opus，s2）报告（2026-10-09）

攻击面：正文 `research/prompts/_m3-retire-r1-body.md` 第四节分工表「云端攻方」那一行，K2、K3、K4、K5。只读冻结树 `~/.local/share/singlefs/work/m3-retire-r1/tree/`（拷进草稿目录改）与 kb；照正文独立做，没读这一轮另两条腿的产出（禁读清单）。代码引文写冻结树里的全路径:行号；kb 引文写 kb 文件自己的行号或 `文件@小节`。
不重复上游三轮攻过的角度（第一轮：E177 51 段与合并 64 段；第二轮：回退之后谱系链被环再分配、Z 三形、区间墓碑、N1 分隔 key 还原；第三轮：克隆头共享叶、副本坏一份、清扫删行、删 I-9.4 第三句）。

状态：完成（分段落盘）。

## 复跑与文件

模型目录 `research/prompts/m3-retire-r1-opus-s2-model/`：`src/` 是纯模型（Rust，从 `research/prompts/m3-leaf-death-r3-opus-s2-model/src/model.rs` 拷起改，写路径换成 E176 的 M3，回退目标放开到被前一次回退抛弃的那一段，加六种退役读法、九条 K5 候选判据与 K4 计数）；`copy/opus_s2_retire_history.rs` 是放进冻结树副本 `crates/singlefs-harness/tests/` 的三条用例（内存稀疏盘 `SparseBlockDevice`）；`tools/summarize.py` 汇总；`expected/` 是第一次复跑 `rerun1/` 的 `summary/`。复跑（仓根；每条经内存包装，编译 6G、其余 4G，16 线程；同一时刻只起一条 cargo）：

```
S=scripts; M=research/prompts/m3-retire-r1-opus-s2-model; O=<空目录>
export S2_MODEL_DIR=$PWD/$M SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree S2_THREADS=16
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 16 bash $M/rerun.sh $O build
for st in sweep demos determinism; do bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 16 bash $M/rerun.sh $O $st; done
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 16 bash $M/rerun.sh $O copybuild
for st in copyrun summarize; do bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 16 bash $M/rerun.sh $O $st; done
bash $M/compare.sh $O
```

交回前在空目录 `/home/fy5090/.local/share/singlefs/work/m3-retire-r1-opus-s2/rerun2/` 里按上面从头跑通（`rerun1/` 与它各自独立）：sweep 304 段 `histories=304 elapsed_seconds=3.1`，副本用例「test result: ok. 3 passed; 0 failed」，`compare.sh` 原样输出：

```
same cascade.txt
same copy.txt
same determinism.txt
same double.txt
same sweep-summary.txt
same trace.txt
```

量：纯模型不调 crates 的 `enumerate_layer0`；崩溃点在模型里对在飞发布的单元写逐子集枚举，没有一处因写数超上限跳过（计数行里没有 `crash_points_skipped_more_writes_than_k_max`，`crash_k_peak=3`），族 r「crash_points=8783 crash_states_enumerated=19912」、族 up「crash_points=414 crash_states_enumerated=924」。副本用例不跑崩溃枚举，层 0 状态数 0。先跑 4 段量挂钟（0.2 秒）再定全量 304 段，没缩历史。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`）：

```
f4859473c75bf71953eea4198a0939f51b0d6923de3a018fb2b0f7aeb80d107b  ./Cargo.lock
bf6a8592a7cc4a811ada84da7a172c96e163ec8748b68cfd836a03e94353b32e  ./Cargo.toml
b6b0ed2ae8335a70408ba60219923189de10853851f4190411daca7207786211  ./compare.sh
5cc5060c084d1ac4f1e2d4af715bb8c22c026f5a3806874bdbdc9269f653724b  ./copy/opus_s2_retire_history.rs
fe5c64e85adf6a66da239a2a96f3a3dadb1b5357c2670e42627c2d86f9ee3f02  ./expected/cascade.txt
3ad6c91c3cf4d61811351d51e1ff686380fd5b73704f5a41019caaba75c3c504  ./expected/copy.txt
82887bc2e8f61ea0d9fb0326ff70e06514deba1f75c6f5f5607573f0bb23e627  ./expected/determinism.txt
358f4f1a89b3716605e950edc86446a538ba8a32bca69ffa80ed9f9241b6c768  ./expected/double.txt
98a1b03d3d0d9c7b226d638c5944cc2209187d4b6de80412fdf38ec3345ebfe5  ./expected/sorted-output-sha256.txt
d50148e1dc615370c5aa0a9a48e4957ccfec433f4166bf9d37f3ca1368b1b114  ./expected/sweep-summary.txt
1a7e4549681bb85f4fb7c3af576182ac22cae2ac4e82cea7e4c95f3e147ef363  ./expected/trace.txt
8376ff171e846bc50715950b0a599126bb77fa6f0e5c26e9c464bdf7beff01c9  ./rerun.sh
2f459e7fe3a615f711ad8e2dcf32c14d0f2ad1cb96c4e4c22fdc4169af09c601  ./src/demos.rs
963446f3b1ff2fec2988a0cbc6d35bcd8720fdd9279c0e25c043c4d4a4cf4ca6  ./src/main.rs
27e26f31e00a68de3590b9a9194b0f7086a22ba5f88cf57ccb31b4c252ee2621  ./src/model.rs
9ad6d7e1bdd7a17d5efe7ef90a2efbefdf1d6ab3be4983430fe973109679552c  ./tools/summarize.py
```

（`expected/sorted-output-sha256.txt` 里那一行 `sweep-sorted.txt` 的整份排序输出 1.1 MB，没放进 `expected/`；它在 `rerun1/summary/` 与 `rerun2/summary/` 里，`compare.sh` 只比 `expected/` 里有的。）

## 各格判定一览

「量过」= 在我的模型或草稿目录的冻结树副本上跑出来的（不是入库装置上的数，主 agent 要在入库装置上重做才能引）；「推的」= 按条款或代码推、没跑。数都引自 `expected/` 里的原样行。没打中的都是一次抽样里的。读法与判据的名字见 K2 节开头的定义表。

| 格 | 被攻的候选形态 | 判定 | 节 |
|---|---|---|---|
| K2 同一身份退役两次 | 写者「已有就不追加」（读法 first） | **打中复活**（量过）：只要一次管理员回退。C 在 txg 10 摘掉 → 回退到 txg 9（C 活着、它最后一条记录在删除队列）→ 续做在 txg 14 再摘 C；树里只留 (C, 10)，10 不在谱系上，C 在链上那一版被收、已删的记录 9 复活。随机族 r 288 / 288 段都中；crates 副本坐实两步都可达 | K2.1 |
| K2 同一身份退役两次 | 写者「按身份覆盖」（读法 over） | **打中复活**（量过）：要两次回退，第二次回退到第一次摘掉那一版（被第一次回退抛弃的那一段里；今天的回退候选集收它，crates 副本坐实）；随机族 r 263 / 288 段 | K2.1 |
| K2 同一身份退役两次 | 追加、同身份两条都留，按死亡 txg 在不在谱系上滤（读法 both） | 没打中（量过）：族 r 510 262 版、族 up 25 228 版、手造单次回退 14 846 条与两次回退 261 218 条，复活与误判死亡全 0 | K2.1 |
| K2 死亡 txg 取哪次 | 取被退役那片最后一版的诞生代号（读法 lastw） | **打中误判死亡**（量过）：回退到 C 最后一版写出之后、摘掉之前的那一版，C 活着而它的死亡 txg 落在链上；族 r 288 / 288 段，m 峰值 29 | K2.2 |
| K2 死亡 txg 取哪次 | 取摘掉或吸收那次的 checkpoint_txg（正文候选） | 没打中（与 both 同一组数） | K2.2 |
| K2「头 ID」段 | 退役树全池一份、记录不带头 | 推的打中误判死亡：跨头首次 COW 给 origin 身份写的退役记录被 origin 头的重建读到；退役树每头一棵（头 ID 取 inode 树的树 ID，同墓碑树）就不用这一段。没量（crates 没有克隆头） | K2.3 |
| K2 同一次发布分裂→合并→再分裂 | — | 推的：今天的 crates 不可达（一次发布只做一件事） | K2.3 |
| K3 回退、崩溃、丢根 | 退役树随现行那一版、容器只按 P2 择版本、记录按死亡 txg 滤（X1 甲）+ 追加 | 没打中（量过）：回退目标放开（5 103 次落在不在现行谱系上的根）、崩溃 8 783 点 / 19 912 状态、丢根 3 837 次，both 全 0 | K3 |
| K4 每次发布几条 | 「删空摘掉、合并吸收各一条」 | 从头照 M3 写：每次发布至多 1 条（量过）；**上界式打中**（量过）：M1 写出来的树换 M3 之后一次最后一批写 6 条、2 个退役容器（手造），随机族 up 峰值 3 条 / 2 个容器 | K4.1 |
| K4 预留形态 | 谱系那一项照搬：容器 2 + 根 1 + 内部层数（按要接的那一版现读） | **这一项在根满长高那次不是上界**（crates 副本量，谱系树）：写 4 块、算 3 块；整个 ckpt_cost 540 对这次写 12 块，总量没超卖 | K4.2 |
| K4 不许因写退役被拒 | 退役写进删除类发布、空间罩在环窗口 W | **打中，不分辨臂**（crates 副本量）：今天删空一片那次发布每盘新占 45 / 85 槽，W = 36（100 MiB 盘、20 / 40 片叶）；退役记录要写进的正是这一次 | K4.3 |
| K5 判据 | 被退役身份不在同一版现行叶里（不按谱系滤） | **打中合法镜像误报**（量过）：族 r 213 373 版；按谱系滤的那一写法 0 | K5 |
| K5 判据 | 同一身份在退役树里至多一条（全部记录） | **打中合法镜像误报**（量过）：族 r 473 608 版；只判谱系上的 0 | K5 |
| K5 判据 | 死亡 txg ≤ 本版 txg；≥ 出生代（含严格 >）；非最右容器装满；容器号 1..n | 没打中（量过，全 0） | K5 |
| K5 GPU | — | 没做 | K5 |

## 读法与判据的定义（模型里写死的，`src/model.rs` 的 `check` 与 `k5`）

扫盘重建（照 `.claude/kb/decisions/08-核心索引结构.md:203`「退役树里有退役记录指认的身份不收，退役记录带死亡 txg、按谱系滤」）：叶容器过已发布谓词 P2（单元写序实例 + 实例表行）且诞生代号在最新那一版的谱系父链上，同身份取 (诞生代号, 实例, 槽) 最大；退役容器只按 P2 择版本（随现行的树，用户定的 X1 甲）；退役记录按下表生效，生效的身份整组不收；同号几条取供出它的叶版本最新的。与真值逐条比 (号, 出生代, nlink)：多出或不同记复活 r，缺的记误判死亡 m。

| 名 | 退役记录怎么生效 | 对应的写法 |
|---|---|---|
| both | 每条死亡 txg 在谱系父链上的都生效 | 追加；同一身份两条都留（正文候选 + 不去重） |
| first | 同一身份只认最先追加的那条，它在链上才生效 | 写者「树里已有这个身份就不再追加」 |
| over | 同一身份只认最后追加的那条 | 写者「按身份覆盖」（退役树按身份做 key 时的 upsert） |
| lastw | 死亡 txg 换成被退役那片最后一版的诞生代号 | 死亡 txg 取「容器最后活着的那一版」 |
| nofilter | 全部生效，不看谱系（对照：第二轮已知误判死亡） | — |
| none | 不读退役（对照：已知复活） | — |

K5 九条候选判据逐版判，每一版都是模型里合法写路径写出来的（合法镜像），红一次记一次误报：k5a 被退役身份不在同一版现行叶里（全部记录）、k5b 同上只判死亡 txg 在谱系上的、k5c 死亡 txg ≤ 这一版的 txg、k5d 死亡 txg ≥ 被退役身份的出生代、k5d_strict 死亡 txg > 出生代、k5e 同一身份在退役树里至多一条（全部记录）、k5f 同上只判谱系上的、k5g 非最右退役容器装满、k5h 退役容器号沿叶序是 1..n。

## K2　退役记录的字段表

### K2.1　同一个身份会被退役两次；第二条只能追加

正文 `research/prompts/_m3-retire-r1-body.md:37`「同一个身份会不会被退役两次（身份里带出生代）；会的话第二条怎么办。」

**会，只要一次管理员回退。** 最短的一段（叶容量 4、退役容器 3 条，`expected/trace.txt` 第 1–17 行原样，第 9–15 行是 C 在盘上更早的 7 版、略）：

```
S2 name=trace reader=first allow_second_rollback=false rb1_index=9 ops=[] rb2_index=None
  rollback to txg 9
  publish txg 13 inst 1 seq 13 admin rollback leaves=[(1, 1, 4), (5, 2, 4), (9, 2, 1)] ret_records=1
  publish txg 14 inst 1 seq 14 unlink last batch leaves=[(1, 1, 4), (5, 2, 4)] ret_records=2
  current txg=14 lineage=[(13, 9)] chain=[1, 2, 3, 4, 5, 6, 7, 8, 9, 13, 14]
  live leaves now: [(1, 1, [1, 2, 3, 4]), (5, 2, [5, 6, 7, 8])]
  retirement record who=9/2 death=10 on_chain=false last_write=9 last_write_on_chain=true
  retirement record who=9/2 death=14 on_chain=true last_write=9 last_write_on_chain=true
  leaf C version slot=10 birth=9 inst=1 published=true on_chain=true records=[(9, 0)]
   reader=first first_hit=seq=14 txg=14 inst=1 r=1 m=0 r_peak=1 m_peak=0
```

每一步许可它的那一句：

1. 前缀：建 12 个、unlink 12、11、10、9，第四次的最后一批删空叶 C = (9, 2)、摘掉，txg 10 写 (C, 10)——`.claude/kb/decisions/08-核心索引结构.md:171`「删空摘掉的容器同一次发布给它的身份写一条退役记录」。
2. txg 13 管理员回退到 txg 9（unlink 9 意图那一版：C 活着、9 在删除队列里）——候选集只要「按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效」（`.claude/kb/decisions/16-发布语义.md:39`「| 回退候选集 | 按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效」）；退役树照抄 cur（正文 `research/prompts/_m3-retire-r1-body.md:41`「管理员回退：退役树照抄 cur」，与 `.claude/kb/decisions/23-journal的角色与格式.md:378`「墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)」同向）。
3. txg 14 续做删除队列，最后一批再删空 C、再写一条 (C, 14)——`/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree/crates/singlefs-core/src/transaction.rs:6079`「会话里每一次用户改动先走空、unlink 一次调用走完全部批，」。
4. 扫盘重建：链是 14 → 13 → 9 → 8 …，(C, 10) 不在链上、(C, 14) 在。first 只认 (C, 10) ⇒ C 不算退役 ⇒ C 在链上最新那一版（slot 10，诞生代号 9）被收，已经删掉的记录 9 回到重建视图：复活 r = 1。both 认 (C, 14) ⇒ 0。

over（按身份覆盖）要第二次回退：同一段前缀，txg 13 回退到 9、txg 14 再摘 C，txg 15 回退到 txg 10（第一次摘掉那一版，被第一次回退抛弃的那一段里），链 16 → 15 → 10 → 9 …，(C, 10) 在链上、(C, 14) 不在；over 只剩 (C, 14) ⇒ C 那片诞生代号 9 的版本被收，复活 1（`expected/trace.txt` 第 72–91 行；第 91 行「reader=over first_hit=seq=15 txg=15 inst=1 r=1 m=0 r_peak=1 m_peak=0」）。

**今天的 crates 接受这两步**（副本用例 `the_same_leaf_identity_is_dropped_twice_and_a_second_rollback_lands_in_the_segment_the_first_rollback_abandoned`，两块 4 GiB 内存稀疏盘，`expected/copy.txt` 第 11–17 行原样）：

```
S2C step=unlink_234_first_drop txg=10 leaves=["(65537,2,1,3)#233"] deleted_inodes=[] checker_violations=[]
S2C name=first_drop identity=(65537,2,234,4) intent_txg=9 drop_publish_txg=10
S2C step=rollback_1_to_the_intent_of_234 txg=11 leaves=["(65537,2,1,3)#233", "(65537,2,234,4)#1"] deleted_inodes=[234] checker_violations=[]
S2C step=queue_continued_second_drop txg=12 leaves=["(65537,2,1,3)#233"] deleted_inodes=[] checker_violations=[]
S2C name=second_drop identity=(65537,2,234,4) drop_publish_txg=12
S2C step=rollback_2_to_the_first_drop_root txg=13 leaves=["(65537,2,1,3)#233"] deleted_inodes=[] checker_violations=[]
S2C name=c_version_on_the_chain slot=50324 bytes=32768 unchanged_on_disk=true lineage_tree_records=["LineageRecord { rollback_txg: CheckpointTxg(11), rollback_target_txg: CheckpointTxg(9) }", "LineageRecord { rollback_txg: CheckpointTxg(13), rollback_target_txg: CheckpointTxg(10) }"]
```

同一个身份四段 (65537, 2, 234, 4) 在 txg 10、12 各被摘一次；第二次回退落在 txg 10（第一次回退抛弃的那一段）被接受——`roll_back_by_a_forward_publish` 的候选判定只按实例表排除被崩溃恢复抛弃的根（`/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree/crates/singlefs-core/src/mount.rs:7586`「if abandoned_by_table(&target_root, current_table) {」），不排除被回退抛弃的；C 在链上那一版（意图那次写的，slot 50324）回退之后逐字节还在原槽；每一版池级 checker 一条不红。退役记录今天没有写者，副本只打印每次摘掉的身份与 txg，「写者会在 txg 10、12 各写一条」是按 D8 那一句推的。

放开扫（用户动作不写死）：前缀固定，第一次回退取前缀的 11 个候选各一次，之后用户动作逐个放开（unlink 任一活号、建 1 个、空发布，至多 3 步），每个后缀末尾既判「不再回退」，也逐个判第二次回退到那时的每个候选再空发布一次（`expected/double.txt` 第 33–35、48–51、64 行原样，环形几何 ring_lowest；宽几何 wide 的数逐格相同——这组历史至多二十几次发布，环还没转一圈、没有槽被复用，复用那一维只在随机族里有）：

```
S2 name=single_prefix geometry=ring_lowest retired_identity=9/2 death_txg=10 prefix_txg=12 rb1_candidates=11 suffix_depth=3 runs=14846 runs_with_c_retired_twice=4458
S2 name=single geometry=ring_lowest reader=both tree=inode runs=14846 r_runs=0 m_runs=0 first=none
S2 name=single geometry=ring_lowest reader=first tree=inode runs=14846 r_runs=2138 m_runs=0 first=rb1->txg3_ops=[Unlink(11),Unlink(10),Unlink(9)]_rb2=none@seq=20 txg=20 inst=1 r=1 m=0
S2 name=single geometry=ring_lowest reader=over tree=inode runs=14846 r_runs=0 m_runs=0 first=none
S2 name=double_prefix geometry=ring_lowest retired_identity=9/2 death_txg=10 prefix_txg=12 rb1_candidates=11 suffix_depth=3 runs=261218 runs_with_c_retired_twice=90953
S2 name=double geometry=ring_lowest reader=both tree=inode runs=261218 r_runs=0 m_runs=0 first=none
S2 name=double geometry=ring_lowest reader=first tree=inode runs=261218 r_runs=51541 m_runs=0 first=rb1->txg1_ops=[]_rb2->txg9@seq=15 txg=15 inst=1 r=1 m=0
S2 name=double geometry=ring_lowest reader=over tree=inode runs=261218 r_runs=13374 m_runs=0 first=rb1->txg3_ops=[Unlink(11),Unlink(10),Unlink(9)]_rb2->txg10@seq=21 txg=21 inst=1 r=1 m=0
```

随机族 r（288 段，M3 写路径，崩溃与丢根注入开着，`expected/sweep-summary.txt` 原样）：

```
S2SUM family=r reader=both histories=288 versions_judged=510262 r_versions=0 m_versions=0 r_peak=0 m_peak=0 histories_hit=0
S2SUM family=r reader=first histories=288 versions_judged=510262 r_versions=241066 m_versions=0 r_peak=89 m_peak=0 histories_hit=288
S2SUM family=r reader=over histories=288 versions_judged=510262 r_versions=107111 m_versions=0 r_peak=24 m_peak=0 histories_hit=263
```

四句：① 分辨臂：分辨，同一批历史上 both 全 0、first / over 中；② 判别子：重建那一刻读得到每条记录的死亡 txg 与谱系树；first / over 的写者在写的那一刻把另一条丢了，盘上就没有分辨它的东西——这是写法丢信息，不是判别子观测不到，both 保住了；③ 满足的是正文第五节 `research/prompts/_m3-retire-r1-body.md:67`「一个形态被攻方坐实一形误判死亡、复活、准入超卖或合法镜像误报（主 agent 现查），出局。」里的「复活」；④ 正文没给跑前改法；追加不去重（both）在打中的每一格上 0（量过）。

由此推的连带（没量）：退役树不能按身份做唯一 key（按身份建的索引只能是「身份 + 死亡 txg」或纯追加）；同一身份的不重复判据只能判谱系上的（K5）；写者不许在写之前查「是否已退役」去重。

### K2.2　死亡 txg：只能取摘掉或吸收那次

正文 `research/prompts/_m3-retire-r1-body.md:36`「死亡 txg 取哪次发布的 checkpoint_txg：摘掉或吸收那次。」——正文候选没被打中（both 那一组数）。对照写法「死亡 txg = 被退役那片最后一版的诞生代号」（lastw，看着也「带 txg」）**打中误判死亡**：同一段前缀，txg 13 回退到 txg 9，C 的最后一版正是 txg 9 写的（意图那次把 9 改成 nlink = 0），记录 (C, last_write 9) 的 9 在链上，C 被整组不收，正在删除中的记录 9 从重建视图里没了（`expected/trace.txt` 第 19 行起、第 35 行原样「   reader=lastw first_hit=seq=13 txg=13 inst=1 r=0 m=1 r_peak=0 m_peak=1」）。随机族 r（`expected/sweep-summary.txt` 第 12 行原样）：

```
S2SUM family=r reader=lastw histories=288 versions_judged=510262 r_versions=0 m_versions=105933 r_peak=0 m_peak=29 histories_hit=288
```

四句：分辨（both 0）；判别子看得到（回退目标那一版的 txg 夹在最后一版与摘掉那次之间，正是 lastw 丢掉的那一段）；满足第五节「误判死亡」；改法取摘掉那次（量过 0）。只要被回退到的根落在「最后一次写」与「摘掉」之间就中，所以死亡 txg 不能取任何早于摘掉那次发布的值。

### K2.3　「头 ID」段、同一次发布里分裂 → 合并 → 再分裂（推的，没量）

**头 ID**（正文 `research/prompts/_m3-retire-r1-body.md:34`「执行这次退役的头的树 ID（「头 ID」」）。跨头首次 COW 给旧身份写退役记录：`.claude/kb/decisions/08-核心索引结构.md:171`「新身份 = (本头树 ID, 类型 2, 原容器号, 本次发布代号)——保留容器号、只换出生树与出生代，同时给旧身份写退役记录」。旧身份的出生树是 origin，而 origin 头自己还在用这个身份。
- 退役树全池一份（树表头 ID 写 0，像谱系树：`.claude/kb/decisions/08-核心索引结构.md:262`「`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537），谱系树写 0（全池一份）」）、记录不带头：origin 头的重建读到克隆头写的那条，把自己的活叶整组不收 ⇒ 误判死亡（推的）。这一形下「头 ID」段防的就是它。
- 退役树每头一棵（头 ID 取 inode 树的树 ID，同墓碑树）：每条记录归属由所在的树给出，「头 ID」段多余。但建克隆时新头的退役树必须从 origin 那一刻的退役树共享起（不是零根），否则 origin 在分叉之前摘掉的身份在克隆头的重建里没有记录 ⇒ 复活（推的）。
- 今天这两形都判不了：`.claude/kb/decisions/08-核心索引结构.md:207`「有克隆头时共享叶头里的出生树是 origin」那一段写明多头镜像的重建没有输入，crates 也没有克隆头。上游第三轮攻过克隆头（候选一没打中），这里只补「全池一份 + 不带头」这一形的推论。

**分裂 → 合并 → 再分裂在同一次发布**（`research/prompts/_m3-retire-r1-body.md:36` 那一行后半）：今天的 crates 一次发布只做一件事——建 inode 只往最右的叶追加，unlink 最后一批只删一条（`/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree/crates/singlefs-core/src/inode_tree.rs:16`「//! **删除只有一种落法**」那一段；意图之前队列必须走空，`/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree/crates/singlefs-core/src/transaction.rs:5975`「"发意图之前队列已走空：任何一版的 deleted_inodes 至多一条 key（D8 已定项 16 第 6 条）"」），这一形不可达。哪天一次发布合并多个操作时，被吸收的右半若是这次发布刚分出来、从没发布过的身份，给它写退役记录对重建无害（那个身份在盘上没有已发布的版本），但 K5 的「死亡 txg > 出生代」（严格）会在这种镜像上红，「≥」不红（推的；`.claude/kb/decisions/08-核心索引结构.md:207`「同一次发布内「分裂 → 合并 → 再分裂」也造不出同身份两版」保证不撞身份）。

## K3　退役树在发布、回退、崩溃恢复里随哪一版

被攻的形态：退役树照抄 cur（正文 `research/prompts/_m3-retire-r1-body.md:41`「管理员回退：退役树照抄 cur」），退役容器只按 P2 择版本，记录按死亡 txg 在不在谱系父链上滤，追加不去重（both）。这一轮新加的角度是把回退目标放开到「被前一次回退抛弃的那一段」里的根（上游第二轮的回退只回到现行谱系上的祖先，它的模型回退时把版本表截在目标那里）。

**没打中**（量过）。随机族 r 288 段（三种叶容量 4 / 6 / 12、三种几何、两种挑回退目标的法，每段 800–1 200 次用户动作，崩溃与丢根注入开着），计数行原样（`expected/sweep-summary.txt` 第 31 行）：

```
S2SUM family=r histories=288 absorbs=27231 borrows=18097 crash_points=8783 crash_states_enumerated=19912 disk_slots=179685 drops=1493 final_txg=456633 instances=12908 lost_root_one=1284 lost_root_one_readable_but_unusable=1313 lost_root_three=1240 publishes=510262 publishes_with_1_retirement_records=28232 recoveries=32532 retirement_records=28724 retirements_of_an_identity_already_in_the_carried_tree=7113 rollbacks=15971 rollbacks_off_chain_taken=5103 rollbacks_to_a_root_off_the_current_chain=2921 crash_k_peak=3 retirement_containers_written_in_one_publish_peak=1 retirement_records_in_one_publish_peak=1
```

在这些历史上 both 的复活与误判死亡全 0（K2.1 那一行）；装置看得见错：不读退役的 none 复活、不按谱系滤的 nofilter 误判死亡（第 13、14 行原样）：

```
S2SUM family=r reader=nofilter histories=288 versions_judged=510262 r_versions=0 m_versions=213373 r_peak=0 m_peak=42 histories_hit=288
S2SUM family=r reader=none histories=288 versions_judged=510262 r_versions=406682 m_versions=0 r_peak=255 m_peak=0 histories_hit=288
```

逐问：
- 「R_old 之后才死、cur 里已退役的容器，回退之后在 R_old 那一版 inode 树里又是活叶：扫描重建按谱系滤能不能把它判成活的」——能：手造单次回退那一组 14 846 条 both 0，而 nofilter 12 984 条误判死亡（`expected/double.txt` 第 46 行原样「S2 name=single geometry=ring_lowest reader=nofilter tree=inode runs=14846 r_runs=0 m_runs=12984 first=rb1->txg2_ops=[]_rb2=none@seq=13 txg=13 inst=1 r=0 m=4」）。条件是 K2.1 那一条：记录只追加。
- 崩溃恢复：退役树随所选根，被抛弃那一段写的记录在没过 P2 的单元里，读侧不收；8 783 个崩溃点、19 912 个崩溃状态、3 837 次丢根（含读得出而被抛弃的 1 313 次）上 both 0。没找到「被抛弃段里死掉、所选根里还活着的容器被误判死亡」的一形。
- 「照搬墓碑树」那几条：容器号从 1 起、下一个 = 最大号 + 1，有新记录才重写，满了才关，第一版不回收——K5 的 k5g（非最右容器装满）、k5h（容器号 1..n）在 51 万版上一次不红（K5 节）；回退照抄 cur 之后容器号接着 cur 发，崩溃恢复之后接着所选根发，两种都没撞号。根满长高这一条模型没建内部节点，没攻（它在 crates 里与墓碑树、谱系树共用 `container_tree.rs`，K4.2 量了它的代价项）。
- 常规挂载路径会不会读退役树：不判死活，推的——死活只由扫盘重建与 checker 用；常规路径读它只为把它引用的单元算进账（与墓碑树同）。没量。

这一格两次抽样里的第一次（同一轮的另一条攻方腿）我没读，不知道它攻了什么；这一格我这次「没打中」，一次抽样。

## K4　准入与承诺量

### K4.1　一次发布写几条：「各一条」只在从头照 M3 写的树上成立

正文 `research/prompts/_m3-retire-r1-body.md:45`「一次发布至多写几条退役记录：删空摘掉、合并吸收各一条；」。

推的（再量）：今天一次发布只删一条记录（K2.3 引的两处）。照 E176 的 M3（`scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs:211`「/// 从容器号为 `start_container_number` 的那片起，按 `order` 查它与左右邻两对，和 ≤ 233 就并；并完从幸存那片接着查，直到没有一对装得下。」）每次发布之后「相邻两片都装不进一片」对整棵树成立（删一条只动它所在那片的两对；并完的幸存那片比被并的两片都大；借不改借方与出借方之和），由此删一条至多一次「摘或并」：删空的那片原来只剩 1 条，它两边的邻居都满，摘掉之后新相邻那一对装不下；没删空时至多并一次。量到（族 r，从头照 M3 写）：每次有退役的发布都是 1 条，`retirement_records_in_one_publish_peak=1`（K3 那一行计数原样）。比「各一条」还紧。

**打中上界式**（量过）：这条整树不变量只在从头照 M3 写的树上有。今天的 crates 是 M1（只删空即摘，`/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree/crates/singlefs-core/src/inode_tree.rs:16`「//! **删除只有一种落法**」那一段），它写出来的树里相邻的小叶随处可见；换成 M3 之后第一次最后一批，E176 的「并到没有一对装得下」会连着吸收。`expected/cascade.txt` 原样（叶容量 6、退役容器 3 条）：

```
S2 name=cascade leaf_cap=6 retirement_container_cap=3 keep=1 unlinked=7 sizes_before=[1,1,1,1,1,1,1,1,1,1] retirement_records_before=0 sizes_after=[6,1,1,1] retirement_records_in_the_last_batch_publish=6 retirement_containers_written_in_that_publish=2 absorbs=5 drops=1
S2 name=cascade leaf_cap=6 retirement_container_cap=3 keep=2 unlinked=2 sizes_before=[2,2,2,2,2,2,2,2,2,2] retirement_records_before=0 sizes_after=[5,2,2,2,2,2,2,2] retirement_records_in_the_last_batch_publish=2 retirement_containers_written_in_that_publish=1 absorbs=2 drops=0
```

随机族 up（前 400 次动作照 M1、之后照 M3，16 段，`expected/sweep-summary.txt` 第 32 行节选两段原样）：「publishes_with_2_retirement_records=4 publishes_with_3_retirement_records=2」「retirement_containers_written_in_one_publish_peak=2 retirement_records_in_one_publish_peak=3」。产品容量推的上界：一次最后一批 = 1 次摘 + 至多（一个根装得下的叶数 135 − 1）次吸收，写满一个退役容器（959 条）之前至多跨 2 个容器。

再加跨头首次 COW（推的，crates 没有克隆头）：一次发布里本头第一次改写了几片与 origin 共享的叶，就写几条，与删除无关。

由此我给的式子（只在我的模型上量过、被攻过零轮）：一次发布的退役条数 R ≤ D + A + B，D = 这次删掉的记录数（今天 ≤ 1），A = 这次吸收的片数（从头照 M3 写的树上 A + D ≤ 1；否则 A ≤ 叶数 − 1），B = 本头这次第一次 COW 的共享叶数；写的退役容器数 ≤ 1 + ⌈(R − 1) ÷ 959⌉。

四句：不分候选形态（它判的是上界式，不是 K2 的哪一种记录）；系统看得到（写路径知道这次摘了几片）；满足的不是第五节字面的「准入超卖」，是「上界式不成立」——要折成超卖得看预留按「每次发布一个退役容器」算：那时 2 个容器对 1 个，少 2 块乘副本数（推的）；改法：要么 M3 上线时先把 M1 写出来的树整理成不变量成立的样子、或第一次只并一对（E176 的写法是写路径策略，D8 已定项 6 说它不进格式），要么上界按 R 现算。用户若认「M1 写出来的池不进 M3」（格式没冻结），这一格作废，「各一条」退成「至多 1 条」。

### K4.2　照搬谱系那一项：根满长高那次少算 1 块（crates 副本量，谱系树）

正文 `research/prompts/_m3-retire-r1-body.md:46`「ckpt_cost（D28 已定项 4）要不要像谱系那样单列一项（容器 2 + 根 1 + 内部层数）」。谱系那一项按「要接在后面的那一版」现读内部层数（`/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree/crates/singlefs-core/src/admission.rs:967`「let lineage_tree_internal_levels = file_version.lineage_tree_internal_levels();」），而长高那次新开的右半与新根都写在这一次（`/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree/crates/singlefs-core/src/container_tree.rs:6`「//! 码 2 节点装满时末尾分裂——左半保留全部原有条目、一个字节不动，右半从触发分裂的那条新条目起；根装满时长一层，新根 1 个码 2、」）。副本用例 `the_lineage_term_read_on_the_version_to_build_on_against_the_blocks_the_growth_publish_writes`（旋钮压到一片 1 条、一个节点 2 条，回退四次，`expected/copy.txt` 第 3–6 行原样）：

```
S2C name=lineage_term rollback=1 txg=4 internal_levels_before=0 lineage_term_before=3 lineage_blocks_written=3 lineage_roles=["tl@0", "tl"] ckpt_cost_before=540 reserve_pool_role_blocks_written=11 all_role_blocks_written=11 checker_violations=[]
S2C name=lineage_term rollback=2 txg=5 internal_levels_before=0 lineage_term_before=3 lineage_blocks_written=3 lineage_roles=["tl@1", "tl"] ckpt_cost_before=540 reserve_pool_role_blocks_written=11 all_role_blocks_written=11 checker_violations=[]
S2C name=lineage_term rollback=3 txg=6 internal_levels_before=0 lineage_term_before=3 lineage_blocks_written=4 lineage_roles=["tl@2", "tl^1.1", "tl"] ckpt_cost_before=540 reserve_pool_role_blocks_written=12 all_role_blocks_written=12 checker_violations=[]
S2C name=lineage_term rollback=4 txg=7 internal_levels_before=1 lineage_term_before=4 lineage_blocks_written=4 lineage_roles=["tl@3", "tl^1.1", "tl"] ckpt_cost_before=541 reserve_pool_role_blocks_written=12 all_role_blocks_written=12 checker_violations=[]
```

第三次回退长高：这一项算 3、写 4。整个 ckpt_cost 540 对这次全部写 12 块，4 GiB 盘上总量没超卖（分配记录树那一项按整棵树算，余量大）；窄盘上总量会不会超没量。`.claude/kb/decisions/28-挂载期承诺量.md:116`「零根与只有根的谱系树 3 块、长高一层 4 块」写的是长高之后那一版的数，长高那一次按上一版现读拿不到 4。退役树照搬这一项，同一形在第 136 片退役容器（135 × 959 = 129 465 条之后）那次发布出现。

四句：分辨形态（「按上一版现读」与「根所在的最右路径满时加 1」两种写法在这一次差 1）；系统看得到（最右路径满不满在形状里）；满足的是「这一项不是上界」，按第五节字面要「准入超卖」，4 GiB 上没坐实，不算出局，交主 agent 定；改法（只在副本上推的、被攻过零轮）：这一项取「容器 2 + 根 1 + 内部层数 + （最右路径从叶到根全满时 1）」。这一形也落在今天的谱系那一项上，不只是退役树。

### K4.3　退役记录要写进的那次发布，今天已经超出删除类发布的 W（crates 副本量，不分辨臂）

删除类发布整次不判式子，空间罩在环窗口里（`/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree/crates/singlefs-core/src/admission.rs:1239`「// （D28（挂载期承诺量） 已定项 1：它们的空间罩在环窗口与墓碑预付里，`transaction::prepare_the_version_publish`）。」）；W 是一次删除类发布每盘新占槽数的上界（`/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree/crates/singlefs-core/src/admission.rs:304`「/// W：一次删除类发布每盘新占槽数的上界（D28（挂载期承诺量） 已定项 1），W = 24 + Σ_ℓ min(N_ℓ, 2·(K + 26))。」，24 里普通分配算 16）。退役记录写在删空摘掉那一次，而今天删空一片时它右边每一片都重写（`/home/fy5090/.local/share/singlefs/work/m3-retire-r1/tree/crates/singlefs-core/src/inode_tree.rs:278`「///   它右边的每一片在树里的位置左移一格。位置是发布路径里的角色序号，挪了位置的那几片这次全部重写：内容与身份都不变，」）。副本用例 `slots_a_drop_publish_newly_takes_against_the_ring_window_bound_on_a_narrow_pool`（两块 100 MiB 内存盘、16 MiB journal 环；建 20 / 40 片满叶，把第二片逐个 unlink 到删空，`expected/copy.txt` 第 1–2 行原样）：

```
S2C name=ring_window_bound device_mib=100 leaves_before=20 W=36 window_slots=900 drop_publish_txg=470 drop_publish_new_slots_per_device=45 of_which_ordinary_allocation=37 roles=27 other_deletion_publishes_max_new_slots=15 max_ordinary=7 drop_exceeds_W=true checker_violations=[]
S2C name=ring_window_bound device_mib=100 leaves_before=40 W=36 window_slots=900 drop_publish_txg=470 drop_publish_new_slots_per_device=85 of_which_ordinary_allocation=77 roles=47 other_deletion_publishes_max_new_slots=15 max_ordinary=7 drop_exceeds_W=true checker_violations=[]
```

删空那次每盘新占 45 / 85 槽，W = 36；别的删除类发布至多 15。退役记录再加容器 2 + 根 1 + 内部层数（长高那次再加 1），只会更多。推的后果（没量）：池子停在窗口边上时，环转一圈之内连着几次删空就能把 (3S + 1) × W 用完，最后一批落不下——释放空间的操作被拒，撞 `.claude/kb/decisions/03-空间分配.md:316`「这个操作本身不需要申请空间。btrfs 没在设计层保证这点」（D3 已定项 17 定案句）。

四句：**不分辨臂**——不论退役预留取哪种形态，删空那次今天就超 W，病根是按位置重写右边每一片，属另一笔账（M3 实现若只重写内容变了的片，删空那次是 15 槽量级加退役那几块，推的）；系统看得到；满足的是「W 不是上界」，第五节「准入超卖」在副本上没坐实到拒绝那一步；改法：退役那几块不论放 ckpt_cost 还是放 W，都要先把删空那次的重写数收进 W 的式子（或改成内容记法），否则对这一格不起作用。4 GiB 盘没量（W = 368，余量更大）。

## K5　checker 判什么

正文 `research/prompts/_m3-retire-r1-body.md:51`「退役记录本身的判据候选：死亡 txg ≤ 这一版的 checkpoint_txg、≥ 被退役身份的容器出生代；被退役的身份不出现在同一版 inode 树的现行叶里；记录宽与登记相符。」我把它拆成 k5a–k5d，再加两条写者可能连带立的（k5e「身份唯一」，照墓碑那一条的样子）与两条「照搬墓碑树」的（k5g、k5h），逐版判族 r 的 510 262 版（每一版都是合法写路径写出来的），`expected/sweep-summary.txt` 第 3–11 行原样：

```
S2SUM family=r reader=k5a_retired_identity_not_a_live_leaf_all_records histories=288 versions_judged=510262 r_versions=213373 m_versions=0 r_peak=1 m_peak=0 histories_hit=288
S2SUM family=r reader=k5b_retired_identity_not_a_live_leaf_on_chain histories=288 versions_judged=510262 r_versions=0 m_versions=0 r_peak=0 m_peak=0 histories_hit=0
S2SUM family=r reader=k5c_death_txg_at_most_this_version histories=288 versions_judged=510262 r_versions=0 m_versions=0 r_peak=0 m_peak=0 histories_hit=0
S2SUM family=r reader=k5d_death_txg_at_least_container_birth histories=288 versions_judged=510262 r_versions=0 m_versions=0 r_peak=0 m_peak=0 histories_hit=0
S2SUM family=r reader=k5d_strict_death_txg_after_container_birth histories=288 versions_judged=510262 r_versions=0 m_versions=0 r_peak=0 m_peak=0 histories_hit=0
S2SUM family=r reader=k5e_identity_unique_all_records histories=288 versions_judged=510262 r_versions=473608 m_versions=0 r_peak=1 m_peak=0 histories_hit=288
S2SUM family=r reader=k5f_identity_unique_on_chain histories=288 versions_judged=510262 r_versions=0 m_versions=0 r_peak=0 m_peak=0 histories_hit=0
S2SUM family=r reader=k5g_non_rightmost_container_full histories=288 versions_judged=510262 r_versions=0 m_versions=0 r_peak=0 m_peak=0 histories_hit=0
S2SUM family=r reader=k5h_container_numbers_one_to_n histories=288 versions_judged=510262 r_versions=0 m_versions=0 r_peak=0 m_peak=0 histories_hit=0
```

**k5a「被退役的身份不出现在同一版现行叶里」照字面（不按谱系滤）在合法镜像上误报**（量过）：K2.1 那一段 txg 13 那一版就是——回退到 txg 9，inode 树里 C 是活叶，照抄 cur 的退役树里有 (C, 10)。今天的 crates 写得出这张镜像（K2.1 副本输出 txg 11 那一行：C (65537, 2, 234, 4) 是活叶；它被摘掉那次是 txg 10，一个写者会在那时给它写一条）。只判死亡 txg 在这一版谱系父链上的记录（k5b）在同一批 51 万版上 0。这与墓碑那一条同形：`.claude/kb/invariants.md:308`「按 (号, 对象出生代) 不重复，这一句只判在谱系上的」。

**k5e「同一身份在退役树里至多一条」照全部记录判在合法镜像上误报**（量过）：K2.1 那一段 txg 14 之后的每一版都有 (C, 10)、(C, 14) 两条。k5f（只判谱系上的）0。k5e 若立，写者为了不红只能去重，正好落进 K2.1 的 first 或 over，那两种都复活——这一条判据与 K2.1 是同一个坑的两面。

k5c、k5d（含严格 >）、k5g、k5h：0，没打中。k5d 严格那一形只在同一次发布里生出又摘掉的身份上红，今天不可达（K2.3）。「记录宽与登记相符」没建（模型不落字节）。GPU 判器同判与否没做。

四句（k5a、k5e）：分辨（k5b、k5f 在同一批镜像上 0）；checker 判一张镜像时看得到这一版的谱系树，k5b 用的父链就是 I-9.18 那条；满足第五节「合法镜像误报」；改法：按谱系滤（量过 0）。

## 没打中的形状

- K2：追加、同身份两条都留、按死亡 txg 在谱系上滤（both）——随机族 r 288 段 / 510 262 版、族 up 16 段 / 25 228 版、手造单次回退 14 846 条、两次回退 261 218 条，复活与误判死亡全 0。取样：叶容量 4、6、12，退役容器 3、5 条，三种几何（不复用、最低复用、随机复用，根环 24），回退目标均匀挑或千分之 700 只挑不在现行谱系上的根，后缀 unlink 任一活号 / 建 1 个 / 空发布至多 3 步。
- K3：回退目标放开、崩溃逐子集、丢根三种，both 全 0（K3 节）。
- K4：从头照 M3 写的树上每次发布至多 1 条（族 r 峰值 1）。
- K5：k5b、k5c、k5d、k5d 严格、k5f、k5g、k5h 在 51 万版上 0。
- 试过而没写成判定的：叶容量 233 的产品几何没跑（每段要几千次动作才摘得到叶，按比例换成小容量）；数据单元与墓碑没进这次的模型（上游第二、三轮量过 Z 与 TSZ，这一轮的格不碰它们）。

## 这条腿自己的限度

- 模型只建 inode 叶与退役容器两类单元，没有内部节点、数据单元、墓碑；退役树的根满长高只在 crates 副本的谱系树上量过（K4.2），没在模型里建。
- 退役记录今天没有写者：crates 副本只坐实「同一身份被摘两次」「回退到被前一次回退抛弃的那一段」两步与 C 那一版仍在盘上，退役记录本身与扫盘重建的读法只在模型里；从副本到模型那一步（写者会在两次摘掉各写一条）是按 D8 那一句推的。
- 头 ID、跨头重生、多头重建都是推的；M3 只照 E176 的写路径建了一种（先查右邻那一对、均分借），左先查那一变体没跑。
- K4.2 与 K4.3 只量了一种几何各一组（4 GiB 上 ckpt_cost、100 MiB 上 W）；「准入超卖」到拒绝那一步没在副本上造出来。
- 一次抽样：没打中的各格都只这一次。

## 什么现象会推翻这份报告的结论

- K2.1：若管理员回退被限定为只能回到「现行那一版谱系父链上」的根、且回退之后不许续做删除队列，单次回退那一形与两次回退那一形都不可达——今天 `mount.rs:7586` 那一判与 D16 回退候选集那一行都不这样限定，副本里两步都做成了。
- K2.2：若死亡 txg 取最后一版的诞生代号时，最后一版之后、摘掉之前的根都进不了回退候选集。
- K4.1：若用户定「M1 写出来的池不跟 M3」，或 M3 实现第一次只并一对，「各一条」那条上界式回来（至多 1 条）。
- K4.2：若长高那次的那一块另有一项罩着（我没在 ckpt_cost 的别的项里找到专给它的），这一项少算 1 块就不算错。
- K4.3：若删空那次的按位置重写在 M3 实现里换成只重写内容变了的片，100 MiB 上的超出会落回 W 之内（推的）。
- K5：若 checker 判的不是单张镜像而是「同一条时间线上」的版本集合，k5a 字面那一写法的语义要重判。

## 没做什么

- 没读禁读清单里同一轮另两条腿的产出；没判正推、辩方那几格；不替主 agent 采纳。
- 副本上量的数（`copy/` 三条用例）不是入库装置上的数，主 agent 要在入库装置上重做才能引；模型的数同样。
- 没跑任何重型测试，没跑 checker 档；副本只跑 harness 档的一个测试目标 `opus_s2_retire_history`（名字不带 layer0），经内存包装。
- 没调 `enumerate_layer0`：副本用例不枚举崩溃状态；崩溃只在纯模型里逐子集枚举。
- K5 的 GPU 判器、「记录宽与登记相符」、K1 与 K6 那几格（不归这条腿）没做。
- 中途没收到主 agent 的消息。

草稿：交回前删了自己建的仓副本与编译目录（`/home/fy5090/.local/share/singlefs/work/m3-retire-r1-opus-s2/` 下的 `repo/`、`repo-target/`、`dev/target/`，与 `rerun1/`、`rerun2/` 各自的 `repo/`、`repo-target/`、`target/`）；留下的只有文本输出：`rerun1/`、`rerun2/` 的 `*.out`、`*.log`、`summary/`（`rerun1/summary/` 拷成了 `expected/`），`dev/` 下开发时的几份输出。复核照「复跑与文件」一节在空目录里重建。
