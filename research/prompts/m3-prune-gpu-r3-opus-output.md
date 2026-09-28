# m3-prune-gpu-r3 云端攻方（Opus）报告（2026-09-28）

攻击面：第一节 A1–A9（最承重的 A1 身份标记、A9 节点代码摘要）。数全部出自草稿里的仓副本与我自己的模型，**不算入库装置上的数**。每个打中都有一行 `must_be_nonzero=`。自己提的改法全部只在我的模型上量过或只是推的，**被攻过零轮**。

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-prune-gpu-r3-opus-model/`。

```
bash research/prompts/m3-prune-gpu-r3-opus-model/rerun.sh <仓根> <一个不存在的草稿目录>
```

它把仓拷两份：一份放进第一、二轮攻方原型（`attack.rs`、`r2.rs`，与第二轮模型目录里的逐字节相同）与这一轮的 `r3.rs`，打三份补丁（E161 挂接、crates 探针、`r3-hook.patch`），开 `verdict-store` 编 E161 装置，跑五个 r3 世界；另一份交给 `variants.sh`，逐个换一处真代码、增量重编、跑点名的世界，换回原样（换回时不带旧的修改时刻，cargo 才会重编）；再跑一条 harness 档测试两次（`comment_flip.sh`）与不编译的几件，最后逐个与 `outputs/` 比，只剥掉 `seconds=` 与 `build_seconds=`，整行不扔。编译与跑都经 `run-with-memory-cap.sh 8G` 与 `capped.sh 4`。

`SHA256SUMS`（66 个文件）：

```
b568fd39d3948b513a1c9be1b8159b5cd1edafaa9963b491fc80c5dd880ed522  ./a1_rename_history.sh
0f426dea942d1b47d5c2054766cd3290b1d45d83d269cf54a225dd944fbe86da  ./a1_scan.py
53009b3750ffc85fcabb7ce24f856e35e03152cf33b441a2d760d355c7b6c1f5  ./a7/a7_label_guard.sh
a5eb9f1a1e0f9f4bfa2e9546488f6f823ea2a427189b6bfb5055411b67b435a5  ./a7/case_forge_the_key_from_a_label_strong.rs
c2f6c2dc46bd1adf50bf4f8caaa6d712a08b00d9f286a246e23b8448f360a28a  ./a7/case_forge_the_key_from_a_label_weak.rs
9e01ea224c1fee96436bb68134d99fb256ea05f7ebf1e76aa4e2fc689a9b17a5  ./a7/case_label_as_store_key_strong.rs
aeaa09d61fd7db43fcd3a38bcdf7e8bcaff6d0a611ac7d3dfcd1fdc9f17b8fc0  ./a7/case_label_as_store_key_weak.rs
9e746b0b454953c9f194df8ffe488804f49ca8a2c4bf38e825e1528143af71be  ./a7/case_label_in_a_hash_map_strong.rs
392ea66ca48955bc6a29fcefba203e3b553d054088b7a19b18fe050cf5bea867  ./a7/case_label_in_a_hash_map_weak.rs
fcb023f45a7e1ce27924afdfb8fe5d7aeb39816b2bb8c0d91d9e474a21fe6c1e  ./a7/case_label_text_as_an_in_memory_key_strong.rs
f095f2107b3788fdee9e289d1cc27353f9696b95164f83354df92147aaa0f877  ./a7/case_label_text_as_an_in_memory_key_weak.rs
fded895fca591e451aa86a5c486030333d23ea628eb799251215acc25009f123  ./a7/guard_strong.rs
ed3f17b7567439276e2fdef92a90db1b236eaeee7845080981d32745cb898811  ./a7/guard_weak.rs
10a821834a212240de5bcb4cc8b022e09eaf3340381b010760a05b2b03f40de3  ./a8_fingerprint.sh
52d5f3ba4eff4ce48601f75c17c7f03895bfefbbaa469cb9f1ee9497ea05bbea  ./a9_static_scan.py
3f70b0696f1effc4263b2fdc806ec588258d3ec0e85d9faf5c2d42ead6e75125  ./apply_mutation_row.py
cee7b16e0da35e4c7d9370eee0677b7d54fc5863e8b2432947780108fdca2f35  ./attack.rs
af884bb93e159b1294ff322afefc12d90e6ba743fab92234964eb62919b72570  ./comment_flip.sh
ea5194ccec6ccdfac811888f4456fd4904f9091bfcc0b77561a554aa81065a73  ./compare_dumps.py
a681cc9482760bbedb9c997b5ee06d5f83c3226eadd66b9bc131dfcfaa01abe7  ./crates-probes.patch
43c308a1db987d535eb09611d796db6e5f666486b90f2e28a2cddb3ef5f1ec19  ./e161-hook.patch
e37a61f5bc02534a3811b83a88137e40a919a4e33f8e9255b7225a89b857150d  ./f1-harness.patch
3aa87cedde3ac1a358c46302dde473eec0f73c508377f143176b21c244f415f4  ./line-shift.patch
394365e4b845e8235bb2d9d16700033c809556cf69458f7980c2c32b1a9d0f18  ./mkfs-watermark.patch
fdbd284f070b283471db7909a158c71706c5c4757c507f82d5a0f45844579f24  ./model-comment.patch
b921321e853e31bc654e6c20985b569a77f9d398e2daf2067df8c9ab4ee95045  ./outputs/base--r2-f1-shifted.out
69d53f29e7e9c38d607a68ff216650b6c8f3ecd6726dc6ed8a020d2ac4aa4339  ./outputs/base--r3-dump.out
85f8b3e2c67ff13e767f278dadb25df3694d90e907a6387e3e493e1953a463d1  ./outputs/base--r3-panic-location.out
97ea0f50ea452f6c8cbdb4bc814ef0520ef65e8bbc220285fd9413f73bf3477b  ./outputs/base--tree-compare.out
a28ab04258ba61a96c64a7661a3bb3c1a278f28eaf9b49c3b4a07fdf989f7450  ./outputs/f1-candidates-in-the-harness--r2-f1-shifted.out
69d53f29e7e9c38d607a68ff216650b6c8f3ecd6726dc6ed8a020d2ac4aa4339  ./outputs/f1-candidates-in-the-harness--r3-dump.out
bd04caa59b1eeb030f103761a0938c973b11ea5bcf5822ce44dd04d5b7baf338  ./outputs/f1-candidates-in-the-harness--tree-compare.out
ef9a452612539c2032c947b1347725d8e9877c49c8ab0f91a257c1195032e580  ./outputs/line-shift-in-the-allocator--r3-panic-location.out
4695ce7e5c1f69637a95ce6d542ad98d447cf9c15a095b17fc7a9668397c611b  ./outputs/line-shift-in-the-allocator--tree-compare.out
8a812de733afc7c5e9ec4404b2769e41070decdee6c0235e0d3b3941a179c9ca  ./outputs/m23-recovery-chains-other-instances--r3-dump.out
0a92de2771b1996f13361bc589a9cf1b3d34b94b88166f2671fa058d6dde81a3  ./outputs/m23-recovery-chains-other-instances--tree-compare.out
ea2cd61d76cc26e22a439afd450838731dc48cda7d52cabbb96afc2a26581315  ./outputs/m457-checker-payload-checksum-offset--r3-dump.out
b27917ab387c5b2e4e1658902ee3254562e682af97bb330dd35e75fde223f533  ./outputs/m457-checker-payload-checksum-offset--tree-compare.out
5ea844b68f4a97eeb51a6e1822dfd7a263ccee3635874a06d162cd52a9949043  ./outputs/m57-no-barrier-between-records-and-root--r3-dump.out
7e4c941cfa65230b0cc72bde6c8dc4845058e34c6630cc5412cdcd16846db626  ./outputs/m57-no-barrier-between-records-and-root--tree-compare.out
11b9d6cde2722a98e73c095c7a0b1461ff8dca7001884de09eb9ef5ccf8387ac  ./outputs/mkfs-watermark--r3-dump.out
a2ce43a5f71a98665b706d1845a8e2b38bb653b2e8570d98409a9475b46d7d95  ./outputs/mkfs-watermark--tree-compare.out
6ec4c38c7340f18044028bc435342915d22a1ad214850ba7c19ffe5414e2f7af  ./outputs/r3-a1-rename-history.out
eebc14a8abe39c2236a2fbf195e84a2aa13512ed4deb60de6a074538ecfca4d1  ./outputs/r3-a1-scan.out
b1bb073a77a0d7defa3440e40e3cb901d3453d6322963864c4e09d9fe662739a  ./outputs/r3-a7-label-guard.out
f7dd32f11fc88181945a976c2bf29db0493e17c09e239b11e2bdc8b57043a966  ./outputs/r3-a8-fingerprint.out
ced7ca320b32b670ca64bca5da0dd78721b8f9a23bb49ce83b7e7136d379d9a2  ./outputs/r3-a9-comment-flip.out
cb1b7f001214b34160dab3320c7b145cc611dbd2912ce115ef04030c82b40db8  ./outputs/r3-a9-static-scan.out
e7f2c756e60d033575a4448a1070b88601a25de43fc90891b852fa4edffb245b  ./outputs/r3-attribute-reuse.out
69d53f29e7e9c38d607a68ff216650b6c8f3ecd6726dc6ed8a020d2ac4aa4339  ./outputs/r3-dump.out
d598e0011aea6836e2167bdd49a1ccc75dc6d77315ac132b201bdde75a31ea8d  ./outputs/r3-f1-torn.out
78ce0c1f4fa42a2dac8baebf5c10250464a672555b76df1a8040360a2118af66  ./outputs/r3-messages-by-batch.out
11e478bf2e966665201e512c3f49703b914f4e9f6e497e6ea0bc25d961b22eb9  ./outputs/r3-multi-record.out
85f8b3e2c67ff13e767f278dadb25df3694d90e907a6387e3e493e1953a463d1  ./outputs/r3-panic-location.out
4ee08512e76fafdc1884d16c68fa46b29f15ea740c2bd554fb58f5f2f5e9852f  ./outputs/r3-pipeline.out
c57e5f6f5c4abb6b39c07ef5a75232ea8dbdf0539ac24c4356ae7a1779fd2fe6  ./outputs/r3-rust-tokens-selftest.out
bdd943cb4b58780328c726b5a54b1a20529b81d32b867f139037382ba88aca37  ./outputs/root-before-records--r3-dump.out
7e4c941cfa65230b0cc72bde6c8dc4845058e34c6630cc5412cdcd16846db626  ./outputs/root-before-records--tree-compare.out
93d5cf91f24d633aeca384a3e4e3d2d0f3d957366cc2ba8c2b6b8285ce4246cd  ./pipeline_model.py
9b465ee94325e4e775499d78d7414accf04d4f0d2e1684b6b67dc797d54f511e  ./r2.rs
1ec5d38f2daa3f23b3465691abf08820e5777ea0279d41dcfb14d96a166c5109  ./r3-hook.patch
4a3ce9f0de68c4fa5d53f610db73e308eb015cbf24d70153b6987760702b8f7f  ./r3.rs
188f6032ccc2d57b1389bfafe15ce1ee4d57d22432f12d7461f663b8c1ac7533  ./rerun.sh
373e71d4fb7b578d886cebca6f8e87ac42dd3ac71bd0b946a836adf9f9e85737  ./rust_tokens.py
c412c7c9b7968c20185504f848821d92a9d35b057f0892324d1204afeb6d8e76  ./transaction-root-before-records.diff
afb7137d137bb46b109fa63c603c122b63c5d02b091fcb98e6a3d080436c40f8  ./variants.sh
```

取样（定义 3b 与正文第五节）：历史全在内存稀疏盘（`SparseBlockDevice`）上录；崩溃状态一律经 crates 的 `enumerate_layer0_selecting_versions_observing_each_state` 取（段内整写子集连第三态逐个展开），每个世界跑前现算 `closed_form_state_count` 与带第三态的实数，断言不超过 10⁶（`r2_state_budget` 行）。这一轮全部原型跑的状态数合计：r3-dump 57 × 7 份编译 + m57 那份 293、r3-f1-torn 47 + 47、r3-multi-record 4096 + 4096、r3-messages-by-batch 96，另加第二轮十六个世界在新副本上复跑一次（第二轮报告的 `r2_state_budget` 行合计 15941 + 90），总计约 2.5 万个状态，远低于 10⁷。**缩法照第二轮**：带 24 写以上单元段的历史只展开段长 < 16 的段（正文第五节「只跑小域」），其余段只以整段持久进入后面的状态；多记录发布的单元段截成录制流前缀的前 12 次写（缩历史长度），这 12 次写的 4095 个状态一个不落。这与定义 3b「留下的每段历史，它的崩溃状态一个不落」冲突，照派发正文办，写在「这条腿自己的限度」。估时：先跑一小段（r3-dump 10.3 秒、r3-multi-record 168 秒），全量挂钟主要是编译（全量一次约 8 分钟、增量每个变体 35–50 秒），合计约 30 分钟，没超过 40 分钟线，没再缩。

## 各格判定一览

| 格 | 被攻的那一版 | 判定 | 世界（输出行） | must_be_nonzero |
|---|---|---|---|---|
| A1① 哈希串 | 身份 + 属性；「写出的内容哈希」三种读法（扇区终值 / 按次序的写列表 / 连屏障的写表） | **打中，不分辨①的三个候选**：去掉记录与根之间那道屏障（`crates/mutations.tsv` 第 57 行登记的真变异），写出的字节与次序全同，前两种读法下四步全复用，284 个状态 KV 里没有或不同、其中 36 个红全漏；带屏障的写表 0 | `outputs/r3-attribute-reuse.out:1` | 284（漏红 36） |
| A1 属性（扇区终值那一读） | 同上 | **打中**：先根后记录的真实现，扇区终值相同，38 个状态复用错、2 个判定不同；按次序的写列表 0 | `outputs/r3-attribute-reuse.out:3` | 38 |
| A1「只改 core 只重录内容哈希变了的」 | 同上 | **打中（过程一格，判定一格没中）**：恢复跨实例接记录（第 23 行真变异），三种内容哈希都不变，3 个状态录下的恢复报告不同、按项红绿 0 个不同 | `outputs/r3-attribute-reuse.out:2` 的 `stale_full_digest=3` | 3（过程）/ 0（判定） |
| A1 / A2 属性罩不罩 harness | 属性 = 内容哈希 + core 摘要 + 判法摘要（checker）+ 版本表 | **打中**：只改 `singlefs-harness` 的候选槽（F1 改法），core 与 checker 一字不动，2 个状态 I-7.8 从红变绿 | `outputs/base--r2-f1-shifted.out:4` 对 `outputs/f1-candidates-in-the-harness--r2-f1-shifted.out:4` | 2 |
| A1② 改名 | 跟着现名走 / 登记时冻结 | **打中「跟着现名走」的代价**：crates 的 14 天历史里 83 份测试文件在同一天改名（今天 114 份），测试函数名消失 27、新出 47；冻结那一臂在这份历史上没撞（旧名被占回 0） | `outputs/r3-a1-rename-history.out:1` | 83 |
| A1 唯一性 | 文件名（不带目录）+ 测试函数名 | 没打中：1153 个 `#[test]`、0 处撞；但 `common/mod.rs` 被 79 个测试目标带进去 | `outputs/r3-a1-scan.out:1` | 0 |
| A9 行号进结果 | 词法单元摘要（去注释、不看位置） | **打中**：只在 `allocator.rs` 加一行注释，摘要不变，接 panic 的入口记下的位置 371 → 372 | `outputs/r3-panic-location.out:1`、`outputs/line-shift-in-the-allocator--r3-panic-location.out:1`、`outputs/line-shift-in-the-allocator--tree-compare.out:1` | 1 |
| A9 备注进结果 | 同上 | **打中**：只改一行注释，词法摘要相同，harness 档一条测试 0 → 101 | `outputs/r3-a9-comment-flip.out:1` | 1 |
| A9 节点代码按函数取 | 走到的函数的词法摘要 | **打中**：checker 一个常量（第 457 行真变异），全部 fn 项的摘要不变，15 个状态判定不同 | `outputs/m457-checker-payload-checksum-offset--tree-compare.out:1`、`outputs/r3-attribute-reuse.out:5` | 15 |
| A9 节点代码按走到的文件取 | 覆盖率插桩的文件集 | **打中（静态）**：`singlefs-format/src/lib.rs` 按 D13 已定项 5 不放函数，0 个非测试 fn，走到的文件集里永远没有它；fn 项之外的 const / static 741 个、派生 Ord 的 enum 37 个 | `outputs/r3-a9-static-scan.out:1` | 781 |
| A9 输入取父身份 | 输入 = 父节点身份 + 这一步定义 | **打中**：只改 mkfs 写进第 0 代根的水位，后面三步的节点代码（`transaction.rs`）一字不动、父身份不变，三步的写表全变 | `outputs/r3-attribute-reuse.out:6` | 3（步） |
| A2 正文从哪现算 | 违例正文不进向量，门禁读违例时从类的代表状态现算 | **打中**：63 个状态 6 个 P7 类，16 个状态拿到别人的正文；按项红绿 0 | `outputs/r3-messages-by-batch.out:2` | 16 |
| A3 P7 / F1 换流 | P7 键；F1 候选；收严的「覆盖字节在盘上」候选 | 没打中：多记录发布单元段前 12 次写 4095 个状态 P7 1 类、0 同键异判；撕裂覆盖那一形造不出（单元写不撕） | `outputs/r3-multi-record.out:5`、`outputs/r3-f1-torn.out:2` | 0 |
| A4 符号键在记录段 | 第二轮符号键 | **打中「省不下」**：12 写记录段 4095 个状态记录核对器判定 1 种、符号键 4095 类；收严版 2 类、0 不一致 | `outputs/r3-multi-record.out:3` | 4094 |
| A5 流水线 | 录入 / 核对经 KV 交接，任意处被杀 | **打中五种弱形**（记录先标后写、已核对先标后写结果、追加计数、布尔已核对、批大于积压上限）；原子批 + 带判法摘要的标记 0 | `outputs/r3-pipeline.out:1`–`:6` | 14 / 2 / 2 / 2 / 1 |
| A6 领块 | 领块状态进 KV、不带租期，一张卡掉线 | **打中**（响的，不是读错）：4 个终局有块永远没人核；领块只在内存 0；单机与两台按批挑代表时正文 0 个不同 | `outputs/r3-pipeline.out:7`、`:8`；`outputs/r3-messages-by-batch.out:2` | 4 |
| A7 标签不当键 | 类型上：键只能从写表算，标签不实现 Hash / Eq | **打中弱形**：标签有 `Display` 就能 `to_string()` 当内存里的键，编得过；收严形（不给任何格式化）四种写法全部编不过 | `outputs/r3-a7-label-guard.out:4`、`:9` | 1 |
| A8 指纹跨机器 | `admission.py crash-case-manifest --judging-digest --toolchain --build-environment` | **打中**：换路径、换编译目录与线程数不变；个人 `CARGO_HOME` 配置、上层目录的 `.cargo/config.toml`、仓里 `.cargo/config.toml` 的 gcc 路径三样不碍判定却各改指纹 | `outputs/r3-a8-fingerprint.out:1`–`:8` | 3 |

表里与下文写成 `outputs/…` 的，都在 `research/prompts/m3-prune-gpu-r3-opus-model/outputs/` 下。

## 被攻的最强版本

| 件 | 我攻的那一版 | 从哪来 |
|---|---|---|
| 身份 | `<代码文件名>::<验证目标名>::<流程步号>::<哈希串>`；流程步号 = 路径号 . 步号 . 步内第几段 . 段内各次写的落法（0 没落、1 撕裂、2 落了），落法串本身当子集序号，不取枚举次序（换展开方式、开关 ignore 不改它，修第二轮 r2-b3 那一格）；全部持久那一个记 `end` | 用户设想；子集序号取落法串是我加的 |
| 属性 | 写出的内容哈希（三种读法都算）、录入时的代码摘要（core）、判法摘要（checker）、版本表 | 正文 A1 行 |
| 重录重核 | 一步连同它前面每一步（与 mkfs 那份基线）的内容哈希都没变，就用 KV 里的旧结果；只改 checker 只重核 | 正文 A1 行「只改 core 只重录内容哈希变了的那一步与它下面的分支」 |
| 节点代码 | 词法摘要：去注释（文档注释两种读法都算）、不看位置；「走到的代码」按函数取 / 按走到的文件取 / 整个文件取三种都量 | 用户定的判法；取法三种是我列的 |
| 流水线 | 录入写块与「已录入」标记，核对按批取「已录入且未核」写回结果与「已核对」标记；最强形是一次原子批、标记记判法摘要 | 第二轮改法 9；最强形是我加的 |

`rust_tokens.py` 是这个摘要的一份实现（手写的 Rust 词法器：嵌套块注释、原始字符串里的 `//`、字符字面量与生命周期分得开），自证 10 格全过（`outputs/r3-rust-tokens-selftest.out:1`）。

## A1 身份标记

### ① 哈希串哈希什么：三个候选在同一格上一起中（世界 r3-dump × 变体 m57）

历史：第一条流 + txg 4 覆盖写，按操作录，步 0 取号、步 1 两次暖机、步 2 新池新建文件、步 3 覆盖写（`outputs/base--r3-dump.out` 的 `r3_dump_step` 四行）。变体只打 `crates/mutations.tsv` 第 57 行那条真变异：「步 3：记录与根之间少一道屏障」（`crates/mutations.tsv:57`）。两份编译上文件名、目标名、路径号与流程定义逐字相同，所以哈希串取「流程定义」时身份逐字相同；取「写出的内容」时要看内容哈希怎么算：

```
E7RESULT name=r3_attribute_reuse world=m57-no-barrier-between-records-and-root mkfs_base_same=true segment_lengths_same=false states_old=57 states_new=293 red_old=0 red_new=36 same_identity_verdict_differs=0 same_identity_full_digest_differs=0 identities_only_in_new=284 identities_only_in_old=48 final_sectors:steps_reused=0/1/2/3:stale=284:stale_full_digest=284:missed_red=36 write_list:steps_reused=0/1/2/3:stale=284:stale_full_digest=284:missed_red=36 write_table_with_barriers:steps_reused=0:stale=0:stale_full_digest=0:missed_red=0 steps_whose_write_table_changed=1/2/3 must_be_nonzero=284
```

- 写出的字节与次序一个不变，扇区终值与写列表两种内容哈希四步全同 ⇒ 四步都照 KV 复用；改坏的编译真跑出 293 个状态（屏障没了，记录与根并成一段），其中 284 个身份 KV 里没有，36 个红，一个都没被核到。
- 哈希串三个候选：「流程定义」不含写出的字节，身份不变；「写出的内容」「两样都放」里的内容若照 A1 行的字面「写出的内容」取（字节），同样不变。所以这一格**不分辨①的三个候选**（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」表第一行）：病根在共用的前提「写出的内容 = 字节」里。分辨它的是内容哈希罩不罩屏障：带段边界的写表那一读 0。
- 状态集合的定义就是按屏障切段：「录下来的写请求流**按设备**切成段」（`.claude/kb/decisions/13-验证路线.md:72`）。屏障不在字节里，却定了崩溃状态有哪些。

四句：①分辨臂——对①的三个候选不分辨；对「内容哈希怎么算」分辨（带屏障的写表不中）。②系统看不看得到——看得到，录制流里屏障是一条操作。③分句——正文 A1 行「稳不稳（插一步、删一步……改参数，已有身份会不会漂）」反过来的那一形：身份不漂、身份指的状态集合变了；与「属性判重录重核会不会漏」那一分句。④改法——内容哈希罩整张写表：每次写的盘、种类、FUA、偏移、字节，连同它落在这一步第几段（屏障切在哪）。量过：同一行 `write_table_with_barriers:…:stale=0`。这个改法同时让第三个候选「两样都放」与第一个在这一格上等价，①该在「流程定义」与「写表（带屏障）」之间选。

### 属性按「扇区终值」算：先根后记录的真实现（变体 root-before-records）

同一份第一轮 F2 调查员补丁（先根与系统配置、后记录）：

```
E7RESULT name=r3_attribute_reuse world=root-before-records mkfs_base_same=true segment_lengths_same=false states_old=57 states_new=57 red_old=0 red_new=0 same_identity_verdict_differs=2 same_identity_full_digest_differs=4 identities_only_in_new=36 identities_only_in_old=36 final_sectors:steps_reused=0/1/2/3:stale=38:stale_full_digest=40:missed_red=0 write_list:steps_reused=0:stale=0:stale_full_digest=0:missed_red=0 write_table_with_barriers:steps_reused=0:stale=0:stale_full_digest=0:missed_red=0 steps_whose_write_table_changed=1/2/3 must_be_nonzero=0
```

扇区终值那一读下 38 个状态复用错（其中同一身份 2 个判定不同）；写列表那一读按次序、就分开了（`must_be_nonzero=0` 那一字段按写列表那一读算，这一格的数取 `final_sectors:…:stale=38`）。所以「写出的内容哈希」若实现成「这一步写完的镜像差」（第一轮 T4 被打中的「父结束镜像」那一族），先根后记录这类只改次序的 core 改动全漏。这一小段的历史今天五样判定都不红（`red_new=0`），漏的是「该重跑而没重跑」，第二轮在同一补丁上量过的 27 个真洞（`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5-implementation-root-before-records.out:2` 的「root_without_own_record=27」）要靠按自己身份归属的判据才红，复用错了它们一样看不到。

### 「只改 core 只重录内容哈希变了的那一步」：恢复改了而写没变（变体 m23）

`crates/mutations.tsv` 第 23 行：「步 3：前缀跨实例边界（把别的实例的记录也接上）」（`crates/mutations.tsv:23`），只改 `recovery.rs`。三种内容哈希、基线全同（`outputs/r3-attribute-reuse.out:2` 的 `steps_whose_write_table_changed=none`），`stale_full_digest=3`：3 个状态的恢复报告不同，按项红绿 0 个不同。这一格在判定一层没打中（这一小段上这条变异不红）；在「KV 存（输入，节点代码，输出），输出里有录入的过程」这一层打中：属性若只有内容哈希，恢复代码的改动一个状态都不重录。改法：录入的代码摘要进重录条件（A1 行本来列了「录入时的代码摘要」这个属性，只是重录那句只提内容哈希）；推的，没另跑（它就是 A9 的节点代码那一格）。

### 属性罩不罩 harness（变体 f1-candidates-in-the-harness，同时是 A2 的「判法改了而名字没改」）

只改 `crates/singlefs-harness/src/memory_pool.rs` 的叠加候选（第二轮的 F1 改法写进真代码，`f1-harness.patch`），core、checker、checker-tier 一字不动（`outputs/f1-candidates-in-the-harness--tree-compare.out:1` 只有这一个文件不同）。第二轮自己的世界在两份编译上：

- 原样：「red_under_todays_overlay_candidates=2 red_names={"checker:I-7.8": 2}」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/base--r2-f1-shifted.out:4`）
- 只改 harness：「red_under_todays_overlay_candidates=0 red_names={}」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/f1-candidates-in-the-harness--r2-f1-shifted.out:4`）

同一个不变量名 I-7.8，2 个状态从红变绿；A1 的属性里内容哈希、core 摘要、判法摘要（checker）、版本表一样都没变 ⇒ 照 A1 的规矩不重录不重核，旧的 2 个红留着（反过来改坏 harness 就是 2 个假绿留着）。候选槽是 checker 的输入（`ImageReader::candidate_unit_slots`），住在 harness：「slots.insert(write.offset.0 / SLOT_BYTES);」（`crates/singlefs-harness/src/memory_pool.rs:747`）。同类的还有切段（`writes_and_segments`，harness）与枚举器、撕裂约定、记录核对器、oracle（都在 checker-tier 的 `crash.rs`），它们既不是 core 也不是 checker crate。

四句：①分辨臂——分：判法摘要罩「checker、harness 与 checker-tier 里判定走到的代码」时不中（整份词法摘要在这个文件上不同，同一行 `tokens_doc_as_comment_differs`）。②看得到——看得到。③分句——A2 行「攻『不变量判法改了而名字没改』：是不是一定落在 A1 的判法摘要里」：不一定。④改法——判法摘要按 A9 的节点代码算，范围是这一步实际走到的全部 crate（含 harness、checker-tier），不按「core / checker」两个名字分；推的，没另跑。

### ② 代码文件改名、挪目录（`a1_rename_history.sh`，只读 git）

```
E7RESULT name=r3_a1_rename_history crates_history=2026-09-14..2026-09-27 renamed_files=103 of_which_test_files=83 current_test_files=114 rename_days=2026-09-27, rename_commits=973e1f58,bc57af7a, old_names_now_taken_again=0 test_function_names_gone=27 test_function_names_new=47 must_be_nonzero=83
```

- 正文写「推的、没量过这个仓一年改几次名」。量出来：crates 的 git 历史只有 14 天，改名全落在同一天的两次提交（「验证代码分两档三个包」「用例粒度与按内容起名」），103 份文件改名，其中测试文件 83 份（今天共 114 份）；同一段时间 `#[test]` 函数名没了 27 个、新出 47 个。
- 「跟着现名走」：那一天之后这 83 份文件下的每个身份都换了，按属性判的复用一个都用不上，等于全部重录重核。验证目标名（测试函数名）也在同一次提交里改，所以「冻结文件名」救不了目标名那一段——冻结得连目标名一起冻，身份里就没有一段是现名了。
- 「登记时冻结」：这份历史上旧名被新文件占回 0 次，撞名没发生；它的风险是登记表与现住址对不上时没人发现，推的，没造。
- 四句：①分辨臂——分：两臂在这一格上代价不同（跟着现名走全换，冻结不换）。②看得到——看得到（git 的改名检测）。③分句——A1 行「改名……已有身份会不会漂」。④改法——身份不放文件名与目标名：身份 = 登记表里只追加的编号 + 流程步号，文件名与目标名作「现住址」属性、覆盖报告里点名用（同 B1 只当标签）；推的，被攻过零轮。

### 唯一性（`a1_scan.py`，没打中）

「test_functions=1153 files_with_tests=160 same_file_same_target_name=0」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/r3-a1-scan.out:1`）：同一文件里同名、不同目录下同名文件里同名，今天都是 0。同一行还有：`crates/singlefs-harness/tests/common/mod.rs` 被 79 个测试目标带进去、`crates/singlefs-checker-tier/tests/common/mod.rs` 被 20 个——流程（`build_pool`、`prepare`）写在共用文件里，「代码文件名」取写流程的文件时一份文件下挂几十个目标，取测试文件时同一条流程（第一条流的前缀 T1–T4）在几个文件下各有一个身份、共享不了。前一半不碍唯一，后一半是代价，不是撞。参数化那一形（一个测试函数里跑七条树分裂流：「for stream in TreeSplitStream::ALL {」，`crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs:416`）靠路径号分开，身份唯一的前提是路径号在一个函数里按流分号；推的，没造。

## A9 节点代码摘要与（输入，节点代码，输出）

### 行号进结果（世界 r3-panic-location × 变体 line-shift-in-the-allocator）

接 panic 的入口（坏盘输入与崩溃注入用的那一个）记的是文件加行号：「|location| format!("{}:{}", location.file(), location.line()),」（`crates/singlefs-harness/src/history.rs:1437`），崩溃注入把它写进报告正文：「let _ = writeln!(text, "  panic 在 {}：{}", panic.location, panic.message);」（`crates/singlefs-checker-tier/src/crash_injection.rs:342`）。变体只在 `allocator.rs` 的 `impl DeviceFreeMap {` 下面加一行注释：

- 原样：「location=crates/singlefs-core/src/allocator.rs:371」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/r3-panic-location.out:1`）
- 加一行注释：「location=crates/singlefs-core/src/allocator.rs:372」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/line-shift-in-the-allocator--r3-panic-location.out:1`）
- 词法摘要：「tokens_doc_as_comment_differs=[] tokens_doc_as_attribute_differs=[] fn_items_differs=[]」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/line-shift-in-the-allocator--tree-compare.out:1`）

输入不变、节点代码摘要不变、输出不同 ⇒ 反向检查（重跑一遍输出不同就判红）在这里假红；换机器不碍，改名碍（`location.file()` 是仓内路径，A1② 那 83 份一改名就全变）。仓里已经知道行号会漂：坏盘输入按「文件 + 消息片段、不按行号」认已知 panic（`crates/singlefs-checker-tier/src/bad_disk_input.rs:826`）。四句：①分辨臂——分：输出里去掉 `:行号` 再比就不中。②看得到——看得到。③分句——A9 行「行号会不会进结果……只挪了行而输出变了，反向检查就假红」。④改法——录进 KV 的输出与反向检查比的都是「去掉位置」的输出（panic 只记文件与消息，文件按登记编号换算）；推的，没另跑。

### 备注进结果（`comment_flip.sh`，harness 档一条测试）

`model_comparison.rs` 的一条测试读 `model.rs` 的原文逐行判：「let source = include_str!("model.rs");」（`crates/singlefs-harness/src/model_comparison.rs:1279`），只滤掉以 `//` 开头的整行：「.filter(|line| !line.starts_with("//"))」（`crates/singlefs-harness/src/model_comparison.rs:1283`）。变体只在 `use std::collections::{BTreeMap, BTreeSet};` 那一行尾加一句提到 `singlefs_core` 的注释：

```
E7RESULT name=r3_a9_comment_flip test=model_comparison::tests::the_model_module_uses_only_the_standard_library_and_the_format_constants exit_pristine=0 exit_comment_only=101 token_digest_same=true must_be_nonzero=1
```

用户定的「除了行数 备注之外。其他的改了就是改了」在这条目标上不成立：备注改了结局就翻。它不是崩溃状态节点，但 A1 的身份罩的是「验证目标」、门禁每一格都要能单独跑，这条就在门禁里。四句：①分辨臂——分：摘要按原文取（带注释）时这一改动会变，不中；`rust_tokens.py` 的原文那一列就是这一读法，推的，这个世界里没把它单独打出来。②看得到——看得到。③分句——A9 行「文档注释……算不算『备注』」的扩大：普通注释在读自己原文的目标上也不是备注。④改法——读自己源码的目标（`include_str!` 读 `.rs`、`file!`、`line!`、panic 位置）登记成「原文节点」，节点代码按原文取；静态扫描今天数出两处（`outputs/r3-a9-static-scan.out:1` 的 `reads_its_own_source_or_line_numbers`）；推的，没另跑。

### 节点代码按函数取：改一个常量（变体 m457）

`crates/mutations.tsv` 第 457 行：「记录头 311：池级 checker 的载荷校验和偏移没跟着后挪 4 字节」（`crates/mutations.tsv:457`），只改 fn 项外面的一行：「const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;」（`crates/singlefs-checker/src/lib.rs:671`）。

- 摘要：「fn_items_differs=[] outside_fn_items_differs=['crates/singlefs-checker/src/lib.rs']」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/m457-checker-payload-checksum-offset--tree-compare.out:1`）
- 判定：`outputs/r3-attribute-reuse.out:5` 的 `same_identity_verdict_differs=15`（内容哈希三种读法都不变，属于「只改 checker 只重核」那一路；节点代码按函数取时判法摘要也不变，15 个状态照旧复用）。

全部 fn 项（不管哪几个被走到，这是上界）的摘要不变，所以按函数取的任何一种节点代码（覆盖率按函数、调用图）都漏。整份文件的词法摘要变了（同一行 `tokens_doc_as_comment_differs`）。四句：①分辨臂——分：按文件取不中。②看得到——看得到。③分句——A9 行「有没有一处代码改了而节点代码摘要没变、输出却变了的世界（改常量……）」。④改法——按文件取；但按文件取的「走到的文件」照样漏，见下一格。

### 节点代码按「走到的文件」取：格式常量那一份文件永远不在里面（`a9_static_scan.py`）

「files_without_a_nontest_fn=3 detail=['crates/singlefs-checker-tier/src/lib.rs', 'crates/singlefs-core/src/lib.rs', 'crates/singlefs-format/src/lib.rs'] const_or_static_items_outside_fn_items=741 macro_rules_definitions=0 enums_deriving_ord=37」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/r3-a9-static-scan.out:1`）

- `singlefs-format/src/lib.rs` 按规矩不放函数：「这里只放标量：`pub const` 的值写成整数字面量，不写算式，也不放函数」（`crates/singlefs-format/src/lib.rs:9`）。覆盖率插桩只给有代码的区间记计数，一个 fn 都没有的文件在「走到的文件集」里永远不出现；而 core 写盘与 checker 读盘都按它的常量走，它是全仓牵连最广的一份。
- 另两份没有非测试 fn 的是 `lib.rs`（只有 `mod` 与 `pub use`），改它们的可见性或重导出同样在走到的文件集外。
- fn 项之外的 const / static 741 个、派生 `Ord` 的 enum 37 个（改成员次序就改比较与排序，fn 体一个字不变），按函数取的节点代码全罩不到；m457 那一格是它们的一个实例（量过）。
- 四句：①分辨臂——分：节点代码按「这一步链进来的每个 crate 的全部源文件」取（crate 粒度）不中。②看得到——看得到（`Cargo.toml` 的依赖就是 crate 闭包）。③分句——A9 行「覆盖率插桩、按调用图、按模块登记各有什么漏法」。④改法——节点代码 = 这一步走到的 crate 的闭包（全部源文件的词法摘要）+ `Cargo.lock` 里这些 crate 的依赖行 + 工具链版本；它比「走到的那部分代码」粗，崩溃节点几乎都链着 core、checker、harness、format 四个 crate，所以对这四个 crate 等于「整棵 crates/ 里这四个」；推的，没另跑。按 crate 取之后，用户要的「只重跑走到改动的节点」在这四个 crate 上省不下，要省只能在 crate 内按文件取、同时把没有 fn 的文件与 fn 外的项另算进每个节点（推的）。

### 输入取「父节点的身份」：只改 mkfs（变体 mkfs-watermark）

A9 行把输入写成「父节点的输出或身份 + 这一步自己的定义」。变体只改 mkfs 写进第 0 代根的水位（`mkfs-watermark.patch`，一行，在 `make_filesystem.rs` 里）：

「world=mkfs-watermark mkfs_base_same=false segment_lengths_same=true」…「steps_whose_write_table_changed=1/2/3」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/r3-attribute-reuse.out:6`）

基线（第 0 步的输出）变了，后面三步（暖机、新池新建文件、覆盖写）的写表跟着全变；这三步的节点代码在 `transaction.rs`，一字未动（`outputs/mkfs-watermark--tree-compare.out:1` 只有 `make_filesystem.rs` 不同），父节点身份（流程定义）也没变。输入取身份时这三步照 KV 复用旧的写表与结果；取父节点的输出（写表连基线的哈希）时全部重跑。这一小段按项红绿 0 个不同（`same_identity_verdict_differs=0`），漏的是录下的过程。四句：①分辨臂——分（取输出不中）。②看得到——看得到。③分句——A9 行「输入的定义罩不罩得住全部（基线、父节点的输出……）」。④改法——输入只许取父节点的输出哈希，不许取身份（第二轮 T4 改法「整条路径写表哈希」本来就是这一读，A9 行的「或身份」要删）；量过：`mkfs_base_same=false` 那一栏就是这个哈希变了。

### 反向检查的合法不确定：今天判定一侧没有，报告一侧有

- 判定一侧：core、checker 与 `memory_pool.rs` 里没有 `HashMap` / `HashSet` 与线程，`crash.rs` 只有并片那一处 `thread::scope`，注释写着线程数只影响快慢：「切法与线程数只影响跑得多快，不影响计数与」（`crates/singlefs-checker-tier/src/crash.rs:1332`）。第二轮十六个世界在我的新副本上复跑逐行相同（草稿 `r2-rerun.log` 十六行「与存档逐行相同」），也是这一侧确定的旁证。
- 报告一侧：层 0 的汇总行按片打本机核数：「per_shard_field("available_parallelism", by_shard, |threads| {」（`crates/singlefs-checker-tier/src/crash.rs:3069`）；第二轮量过计划哈希随线程数、分片、观察者变。KV 的「输出」若收了汇总行或计划哈希，两台同一身份的反向检查就假红。改法：输出只收逐状态的判定与录下的过程，不收汇总与计划；推的。
- 文档注释：`rust_tokens.py` 两种读法都算了；这一轮的七个变体里没有一处只改文档注释的，两种读法在七个变体上判得一样（`outputs/*--tree-compare.out` 的两列）。仓里没有 proc 宏或 `include_str!` 读文档注释的 crash 路径（静态扫描的最后一栏只有那两处），所以把 `///` 当备注今天不漏；推的，没造世界。

## A2 逐项原始判定能不能被骗

- **判法改了、名字没改**：上面 harness 那一格（I-7.8 同名、2 个状态翻）与 m457（checker 常量，15 个状态的项翻、名字全不变）两处，前者不落在「checker 的判法摘要」里，后者落在按文件取的摘要里、不落在按函数取的里。
- **违例正文从哪现算**（世界 r3-messages-by-batch，第二轮 r2-b4-messages 那段历史）：「states=63 p7_classes=6 distinct_texts=3 wrong_text_single_machine_one_batch=16」…「wrong_status_single=0 wrong_status_two_machines=0 must_be_nonzero=16」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/r3-messages-by-batch.out:2`）。按项红绿从类的代表取 0 个错；正文若从代表状态现算，16 个状态拿到别的状态的正文（I-7.7 点名的槽不同）。门禁「一次读出违例」要给每个违例状态自己的正文，就得按那个状态自己的身份从根重放再跑一遍 checker（C5 从根重放那一形），不能按类现算；改法推的，重放的代价没量。四句：①分辨臂——分（逐状态现算不中）。②看得到——看得到。③分句——A2 行「违例正文不进向量之后，门禁一次读违例时还能不能给出每个违例状态的正文（从哪现算）」。④改法——同上。
- **新加、删除、一拆二**：没造世界。按第二轮改法 7「读的一方现算」推：旧块里没有新名字，读的一方若把「没有」读成绿就是假绿；要求读的一方把缺项读成「本次未跑」，并且判法摘要变了整块重核；推的。

## A3 P7 与 F1 改法换一类流（没打中）

- **多记录发布的单元段**（世界 r3-multi-record：第一条流 + 顺序写 6 个数据单元，这一步的段是「36xU, 12xJ, 1xR, 2xS」，`outputs/r3-multi-record.out:1`）：单元段截成前 12 次写，4095 个状态：「read_set_key_classes=4095 read_set_key_inconsistent_full=0 read_set_key_inconsistent_status=0 p7_key_classes=1 p7_key_inconsistent_full=0 p7_key_inconsistent_status=0」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/r3-multi-record.out:5`），F1 候选与收严候选下 I-7.8 都 0 红。这一段上判定只有 1 种，所以它只说明「不假红、省得下」，说不了「同键异判」——同键异判要判定有两种以上的段才量得出。
- **F1 改法碰上撕裂的覆盖写**：我想造「孤儿节点在槽 60001，32K 数据单元从槽 60000 盖过来、撕裂那一态后半留着孤儿」让 F1 改法漏判。造不出：枚举器不撕单元写，「let is_copy_on_write = match write.kind {」（`crates/singlefs-checker-tier/src/crash.rs:1572`）下 `UnitWrite => true` 就不取第三态，这一段只出 3 个状态（`outputs/r3-f1-torn.out:2` 的 `rows`），漏判 0。单元写不撕是 D13 已定项 4 射程里写死的模型（`.claude/kb/decisions/13-验证路线.md:74`）；真盘上 32K 写会不会半截落归 C6，不在层 0。收严候选「覆盖那次写在这一槽上的字节确实在盘上才拿掉这一槽」在第二轮 F1 那一形上同样 0 假红（`outputs/r3-f1-torn.out:4`：今天的叠加 1 红、F1 候选 0、收严候选 0）。
- 树分裂、位置寻址两类流：没造。树分裂流靠测试里的 `common_tree_split` 按小节点容量搭池，那份搭建在测试目标里，原型里要自己照写一份；时间不够，没做，记在「没做什么」。

## A4 记录核对器符号键在大段上（世界 r3-multi-record，打中「省不下」）

顺序写 6 个数据单元的记录段 12 次写（6 条记录、两盘各一份），段内 4095 个状态全展开：

```
E7RESULT name=r3_a4_record_segment units=6 record_writes=12 states=4095 distinct_record_verdicts=1 distinct_status_vectors=2 symbolic_key_classes=4095 symbolic_key_inconsistent=0 tightened_key_classes=2 tightened_key_inconsistent=0 tightened_key_inconsistent_on_the_whole_status_vector=3366 seconds=33.5 must_be_nonzero=4094
```

- 第二轮的符号键把「每条根槽写与记录写在不在盘上」全放进键，记录段上类数 = 状态数 = 4095，记录核对器判定只有 1 种。位置寻址第三条流的记录段 290 写（第一轮事实表戊 1.2 表 P5 那一行），同一机理下是 2²⁹⁰ − 1 类（推的，按同一机理外推）。第二轮在合法复用段上量的「1 类」碰的是单元写，没碰记录写。
- 收严（我加的，零轮）：记录写的在不在盘上只在「按记录自己的（实例，txg）认出的那条根在盘上」时进键。同一行：2 类、记录核对器判定 0 不一致。它只是记录核对器的键：整份判定向量（含两遍 oracle）在这 2 类里 3366 个不一致（`distinct_status_vectors=2`，看 journal 那一遍恢复在根没落时按记录重建，读到哪几条记录会变），oracle 那一截仍要 P7 的恢复读集键。
- 与 F2 改法（记录按自己的（实例，txg）归发布）一起用：收严版就是按这一归属取的；一次发布 6 条记录同一个 txg，归属不碍。
- 四句：①分辨臂——分（收严版不中）。②看得到——看得到。③分句——A4 行「它的类数、精确性」里的类数；第一轮岔路单 R 行「键数接近状态数（复用无收益）」。④改法——收严版，量过（上一行），零轮；136 写单元段那一形没造（记录跨条流要建 N 个 inode，没做）。

## A5 流水线不重核不漏核（`pipeline_model.py`，打中五种弱形）

模型：2 块，录入进程按块号写（块数据 + 「已录入」），核对（一张或两张卡）按批取「已录入且未核」算完写回（结果 + 「已核对」）；两边在任意两次 KV 写之间各能被杀一次、从头重起只看 KV；一次判法摘要换新（D1 → D2）。穷举全部交错与被杀点，数终局里的错：

| 变体 | 终局错 | 出处 |
|---|---|---|
| 最强：录入与核对各一次原子批，「已核对」记判法摘要，结果按块键覆盖写，两张卡，批 1、积压上限 1、录入等着时照放批 | 0（167 个可达状态） | `outputs/r3-pipeline.out:1` |
| 「已录入」先标、数据后写 | 拿半块或没有的数据算：14 | `outputs/r3-pipeline.out:2` |
| 「已核对」先标、结果后写 | 标了已核对、结果没有：2 | `outputs/r3-pipeline.out:3` |
| 结果先写、标记后写，结果按追加计数 | 同一块计两次：2 | `outputs/r3-pipeline.out:4` |
| 「已核对」是布尔 | 判法换了、旧结果照当已核对：2 | `outputs/r3-pipeline.out:5` |
| 批大于积压上限、录入等着时不放批 | 两边死等：1 | `outputs/r3-pipeline.out:6` |

- 「已核对」标记先于结果落盘那一问：落在第三行，门禁把「标了、没有结果」当没有违例，是哑的。第二轮改法 9 只写「每块带『已录入 / 已核对』」，没写两样与结果同批、没写标记带判法摘要；最后一行的死等来自「按批取」加「积压上限」而批放不出来。
- 没有 GPU 时录完再由 CPU 核对：模型里核对是同一个纯函数，与后端无关，逐块相同是这个模型的前提、不是量出来的；GPU 与 CPU 两份实现算得一不一样，这个模型答不了（不碰 GPU）。
- 四句：①分辨臂——分（最强形 0）。②看得到——看得到（KV 就在）。③分句——A5 行「有没有一块被核两次或漏核、有没有『已核对』标记先于结果落盘」。④改法——最强形那一行的四条；只在这个模型上量过、零轮；RocksDB 真库在杀进程下的原子批（WAL 同步写）没量，归 E162。

## A6 按显存领块在双机下与单机逐块相同

- **领块状态怎么存**（同一个模型）：领块写进 KV、不带租期，一张卡领了之后掉线——「stuck=4」（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/r3-pipeline.out:7`）：4 个终局里有块永远没人核。这一形是响的（门禁报「本次未跑」），不是读错；领块只在进程内存里、卡掉线就放回，0（`outputs/r3-pipeline.out:8`）。改法：领块不落盘，或落盘带租期（到期别的卡可领）；前者量过、后者推的。
- **批的组成改不改结果**（世界 r3-messages-by-batch）：同一段 63 个状态，单机一批、单机每批 32、两台按块（16 个状态一块）交错分且各自按批挑代表，三种挑法下按项红绿都 0 个错，两台与单机给同一状态的正文 0 个不同（`outputs/r3-messages-by-batch.out:2` 的 `text_differs_single_versus_two_machines=0`）。没打中：这一段里每个 P7 类的最前那个状态恰好都在第一块，换块长或换段可能不同（推的，没扫）。正文按代表现算本身就错 16 个，见 A2。
- **块键不带分片号与线程数**：第二轮 r2-d1 量过计划哈希随线程数、分片、观察者变（「distinct_plan_hashes=6」，`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-d1.out:6`）；这一轮在新副本上复跑逐行相同。「一批装多少按显存」若让块长跟着批长变，就回到第二轮「换块长续跑 8 个状态重复罩住」那一格（`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-d1.out:5`）；块长固定、批 = 若干整块时不回到那一格（推的）。
- 卡的快慢与显存不同、中途掉线：模型里两张卡的交错是穷举的（快慢只是交错的一种），结论同上两条；显存大小只改批长，不在模型里。

## A7「B1 只当标签」的会红检查（`a7/a7_label_guard.sh`）

两种类型写法各编四种「把标签当复用键」的代码（rustc 只出元数据）：

| 写法 | 标签直接当库的键 | 标签进 HashMap | 从外面造一个键 | 标签的文字当内存里的键 |
|---|---|---|---|---|
| 弱：键只能 `PathWriteTableHash::of_write_table` 算出来，标签不实现 Hash / Eq，有 `Display` | 编不过 E0308 | 编不过 E0277 | 编不过 E0603 | **编得过** |
| 收严：同上，标签不给任何格式化，只能交给只写的覆盖报告 | E0308 | E0277 | E0603 | 编不过 E0599 |

（`research/prompts/m3-prune-gpu-r3-opus-model/outputs/r3-a7-label-guard.out` 第 1–8 行；第 9 行「label_as_key_compiling_under_the_weak_guard=1」。）弱写法被 `label.to_string()` 绕过；收严写法下四种都红。收严写法的代价：覆盖报告与 ignore 行要点名节点，只能经只写的报告类型出去，报告读回来的文字照样能当键（那一步出了类型检查的射程，要一道门禁：KV 与复用模块的源码里不许出现覆盖报告的读法），推的。四句：①分辨臂——分。②看得到——编译器看得到。③分句——A7 行「给一个会红的检查（类型上让标签当不了键，或门禁格）」。④改法——收严写法（量过：四种都编不过），零轮；门禁那一半推的。

## A8 指纹跨机器、跨人（`a8_fingerprint.sh`，打中）

同一份入库内容，崩溃枚举用例 `crash-case:layer0-first-stream` 的输入指纹（`admission.py crash-case-manifest … --judging-digest --toolchain --build-environment`，门禁 54 号同一算法）：

| 情形 | 与主仓相同 | 出处 |
|---|---|---|
| 仓拷到另一层路径 | 是 | `outputs/r3-a8-fingerprint.out:2` |
| 换 `CARGO_TARGET_DIR`、线程数、`CARGO_BUILD_JOBS` | 是 | `:3` |
| 这个人自己的 `CARGO_HOME/config.toml`（只设终端颜色） | **否** | `:4` |
| 仓外上层目录有一份 `.cargo/config.toml`（只设 offline） | **否** | `:5` |
| 另一台机器 gcc 是 14，改仓里 `.cargo/config.toml` 的头文件路径 | **否** | `:6` |
| `RUSTFLAGS="-C target-cpu=native"` | 否（这一样会改编出的代码，进指纹对） | `:7` |

- 读个人配置的那一句：「cargo_home = environment.get("CARGO_HOME") or os.path.join(environment.get("HOME") or os.path.expanduser("~"), ".cargo")」（`research/scripts/admission.py:549`）；仓里那份配置写的是本机头文件路径：「BINDGEN_EXTRA_CLANG_ARGS = "-I/usr/lib/gcc/x86_64-linux-gnu/13/include"」（`.cargo/config.toml:5`），它只在开 `verdict-store` 编 RocksDB 时用，不碍任何判定。
- 第二轮改法 2（路径写表哈希 ‖ 判法摘要）本身只由写表与源码算，换路径、换编译目录不变（推的，`rust_tokens.py` 的摘要只看仓内相对路径与内容）；上面三样是今天阶段指纹里「本机的东西」。工具链版本：`cargo -V`、`rustc -V` 的原文两台相同就相同，今天就是这么进的（`manifest` 那一行「<工具链：cargo 1.98.0 …>」），不带安装路径，跨机器可用。
- 四句：①分辨臂——分：只收会改编出代码的配置键（rustflags、target、runner、wrapper、profile）而不收整份配置文件时，第 4–6 行不中（推的）。②看得到——看得到。③分句——A8 行「本机才有的东西不许进（……本机环境变量与本机配置……）」。④改法——同①；推的，没另跑。同一个判法在两台上判得不同（浮点、并发次序、内存上限）怎么发现：指纹相同时只有重叠域的逐状态对拍（C4 那一格的固定小域）发现得了，复用本身发现不了；推的。

## 前两轮的反例世界在这一版下（身份 + 属性 + 节点代码）

第二轮十六个世界在这一轮的新副本上照第二轮 `rerun.sh` 复跑，十六个都「与存档逐行相同（只剥掉 seconds= 与 build_seconds= 字段）」（草稿 `/tmp/claude-1000/m3-prune-gpu-r3-opus/r2-rerun.log`），数没变。它们在这一轮被攻的版本下判什么：

| 世界 | 前一轮打中的 | 这一版下 | 依据 |
|---|---|---|---|
| r2-b1 并错（载荷改坏的 Z） | 种类串类身份把 Z 与 X 并成一个 | 身份相同（同一流程）、内容哈希不同 ⇒ 重录，不中 | 按定义推的；三种内容哈希都含载荷字节 |
| r2-b1 拆错（写入时间 +1 秒） | 路径写表哈希把判定相同的 12 个状态拆开 | 写入时间是流程参数时身份不同、省不下，照中（代价）；不是参数时身份相同、内容哈希不同照样重录，也省不下 | 推的 |
| r2-b2-versions | 版本表不在指纹里 | 版本表是 A1 的属性，变了就重核，不中 | 按定义推的 |
| r2-b3 单点 / ignore 开关 | 序号从物化父镜像现算、全流序号随 ignore 全变 | 子集序号取落法串：开关 ignore 不改别的段（r3-dump 的身份就是这一形）；单点必须按整条路径定每次写几态，照第二轮改法 3 | `outputs/base--r3-dump.out` 的身份列；单点那一半推的 |
| r2-b4 / r2-b4-messages | P7 键下正文不同 | 按项红绿 0 不中；正文从代表现算 16 个错（A2 那一格，新打中） | `outputs/r3-messages-by-batch.out:2` |
| r2-f1-shifted | 叠加候选 2 个假红 | 改法写进 harness 后 0；但 harness 不在属性里，改了不重核（新打中） | `outputs/f1-candidates-in-the-harness--r2-f1-shifted.out:4` |
| r2-b5 / r2-b5-implementation | 先根后记录今天不红 | 按自己身份归属 27 个真洞全红照旧；属性按扇区终值取时这一类实现改动全复用（新打中，38） | `outputs/r3-attribute-reuse.out:3` |
| r2-b5-legal-reuse | 符号键 1 类 | 单元段上照旧 1 类；记录段上 4095 类（新打中） | `outputs/r3-multi-record.out:3` |
| r2-b6 / r2-kinds / r2-b2-downstream | 没打中 | 不涉及身份写法，照旧 | 复跑逐行相同 |
| r2-b7 / r2-b8 | 计数格、编号宽、定义版本 | 逐项原始判定 + 读的一方现算照旧 0；新加一条不变量时缺项怎么读没定（A2 最后一条） | 推的 |
| r2-d1 | 按块原样导入 79、换块长 8、计划哈希六种、片跨节点 2 | 按内容换编号导入照旧 0；换块长那一格在「批长按显存」下会回来，除非块长固定 | 见 A6 |
| 第一轮 t4、torn、interior、order、reuse、p5、p6、collide、small | — | 第二轮已逐个判过；这一轮只有 torn 与本轮有关：撕裂约定在 `crash.rs`（checker-tier），不在 core 也不在 checker crate，按「core / checker」两个名字分的属性罩不到（同 harness 那一格，推的） | 第二轮报告「第一轮十个世界与 F1、F2 在最强版本下」一节 |

## 几个改法各修哪一格

全部只在我的模型上量过或只是推的，**被攻过零轮**。「量过」贴副本上的原样输出。

| 改法 | 修的格 | 量过 / 推的 |
|---|---|---|
| 写出的内容哈希 = 整张写表（盘、种类、FUA、偏移、字节）+ 每次写落在这一步第几段 | A1① m57（284 / 36）、扇区终值那一读（38） | 量过：`outputs/r3-attribute-reuse.out:1` 与 `:3` 的「write_table_with_barriers:steps_reused=0:stale=0」 |
| ①在「流程定义」与「带屏障的写表」之间选；「写出的内容 = 字节」那一读不许用 | 同上 | 推的（上一行的数给的是写表那一读 0） |
| 重录条件加录入的代码摘要（按 A9 的节点代码） | m23（3 个状态的过程） | 推的 |
| 判法摘要罩这一步走到的全部 crate（含 harness、checker-tier），不按 core / checker 两个名字分 | harness F1（2）、torn 约定 | 推的；`outputs/f1-candidates-in-the-harness--tree-compare.out:1` 只说明摘要按文件取会变 |
| 身份不放文件名与目标名，改用登记表只追加的编号；文件名、目标名当现住址属性 | A1② 改名（83 份） | 推的 |
| 节点代码按 crate 闭包取（全部源文件的词法摘要 + `Cargo.lock` 相关行 + 工具链版本） | m457（15）、格式常量文件、fn 外 741 项、派生 Ord 37 个 | m457 那一格量过（整份文件的词法摘要变了：`outputs/m457-checker-payload-checksum-offset--tree-compare.out:1`）；其余推的 |
| KV 的输出去掉位置（panic 只记文件登记编号与消息），不收汇总行与计划哈希 | 行号假红、本机核数进报告 | 推的 |
| 读自己原文的目标登记成原文节点 | 备注翻结局（1） | 推的；`outputs/r3-a9-comment-flip.out:1` 只说明词法摘要不够 |
| 输入只许取父节点的输出哈希 | mkfs 那一格（3 步） | 量过：`outputs/r3-attribute-reuse.out:6` 的「mkfs_base_same=false」 |
| 正文按违例状态自己的身份从根重放现算 | A2 正文（16） | 推的 |
| 记录核对器键：记录写的持久位只在它自己的根在盘上时进键 | A4（4095 → 2） | 量过：`outputs/r3-multi-record.out:3` 的「tightened_key_classes=2 tightened_key_inconsistent=0」；只罩记录核对器 |
| 流水线最强形（原子批、标记带判法摘要、结果按块键覆盖、批 ≤ 积压上限或等着就放批） | A5 五种 | 量过：`outputs/r3-pipeline.out:1` 的全 0 |
| 领块不落盘，或落盘带租期 | A6 掉线（4） | 不落盘量过（`outputs/r3-pipeline.out:8`）；租期推的 |
| 标签不给任何格式化、只交只写的报告 | A7（1） | 量过：`outputs/r3-a7-label-guard.out:8` |
| 指纹只收会改编出代码的配置键 | A8（3） | 推的 |

**改法能不能收严、收严后在打中的格上还中不中**：内容哈希那一条收成带屏障的写表后，m57 与先根后记录两格都 0（量过）；节点代码从按函数收成按 crate 闭包后 m457 不中（量过），代价是用户要的「只重跑走到改动的节点」在 core、checker、harness、format 四个 crate 上省不下（推的）；输入从「输出或身份」收成只取输出后 mkfs 那一格不中（量过）；标签从弱写法收成不给格式化后四种写法都编不过（量过）；其余推的。

### 给判决之后的小范围功能测试挑：每个改法对应一个现成的「必须报非 0」世界

每一行的「红的世界」是改法没做时报非 0 的那一个；做了改法之后同一个世界该报 0（「做了之后」一列里标量过的，我在模型里已经看到 0）。都在 harness 档或不编译的脚本里跑得动，不要 checker 档、不要 GPU。

| 改法 | 被测的功能 | 红的世界（今天非 0） | 做了之后 | 装置大小 |
|---|---|---|---|---|
| 内容哈希罩带屏障的写表 | 复用判定：core 只动屏障或次序时要重录 | `r3-dump` × 真变异第 57 行（284 / 漏红 36）；× 先根后记录（扇区终值那一读 38） | 0（量过） | 一次增量编译 + 57 / 293 个状态，秒级 |
| 重录条件含录入代码摘要 | 复用判定：只改恢复时要重录过程 | `r3-dump` × 真变异第 23 行（过程 3） | 推的 | 同上 |
| 判法摘要罩 harness 与 checker-tier | 复用判定：只改 harness 候选槽时要重核 | `r2-f1-shifted` × `f1-harness.patch`（I-7.8 2 → 0，属性全不变） | 推的 | 同上，5 个状态 |
| 节点代码按 crate 闭包取 | 节点代码摘要：改 fn 外常量要变 | `rust_tokens.py tree-compare` × 真变异第 457 行（`fn_items_differs=[]`，判定 15 个不同） | 整份文件摘要会变（量过） | 纯脚本，秒级 |
| 输出去掉位置 | 反向检查：只挪行不许假红 | `r3-panic-location` × `line-shift.patch`（371 → 372，摘要相同） | 推的 | 秒级 |
| 读自己原文的目标按原文取 | 节点代码摘要：只改注释也要重跑那一条 | `comment_flip.sh`（0 → 101，词法摘要相同） | 推的 | 一次 harness 档 `--lib` 编译 |
| 输入只取父节点输出 | 复用判定：只改 mkfs 时后面几步要重录 | `r3-dump` × `mkfs-watermark.patch`（3 步写表变、代码不变） | 0（量过：基线哈希变） | 同第一行 |
| 正文按状态自己现算 | 门禁读违例：每个违例状态给自己的正文 | `r3-messages-by-batch`（16） | 推的 | 63 个状态，秒级 |
| 记录核对器键只在根落了时收记录位 | 记录核对器剪枝：记录段上省得下 | `r3-multi-record` 的记录段（符号键 4095 类） | 2 类、0 不一致（量过） | 4096 个状态，约 35 秒 |
| 流水线最强形 | 录入 / 核对被杀、重跑、换判法 | `pipeline_model.py` 第 2–6 行 | 第 1 行全 0（量过） | 纯脚本，毫秒 |
| 领块不落盘或带租期 | 卡掉线不留死块 | `pipeline_model.py` 第 7 行（4） | 第 8 行 0（不落盘量过） | 同上 |
| 标签不给格式化 | B1 只当标签的会红检查 | `a7_label_guard.sh` 第 4 行编得过 | 第 8 行编不过（量过） | rustc 只出元数据，秒级 |
| 指纹只收会改编出代码的配置键 | 指纹跨机器、跨人 | `a8_fingerprint.sh` 第 4–6 行 | 推的 | 两份仓副本，约 1 分钟 |

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| A1 身份撞：同一文件同名目标、不同目录同名文件里同名目标 | 今天全仓 1153 个 `#[test]`、160 份文件 | 0（`outputs/r3-a1-scan.out:1`） |
| A1②「冻结文件名」撞：旧名被新文件占回 | crates 的全部 git 历史（14 天、103 次改名） | 0（`outputs/r3-a1-rename-history.out:1` 的 `old_names_now_taken_again=0`） |
| A3 P7 同键异判、F1 与收严候选假红 | 多记录发布单元段前 12 次写 4095 个状态 | 0 / 0 / 0；这一段判定只有 1 种，分辨力有限 |
| A3 F1 改法被撕裂的覆盖写骗过 | 孤儿节点 + 32K 覆盖写，3 个状态 | 造不出：单元写不撕 |
| A6 两台按块交错、各自按批挑代表，与单机给的正文不同 | r2-b4-messages 那段 63 个状态，块 16、批 32 | 0 |
| m23 在判定一层 | 第一条流 + 覆盖写小段 57 个状态 | 按项红绿 0 个不同（过程 3 个不同） |
| 反向检查被判定一侧的不确定骗（哈希表次序、线程） | core、checker、`memory_pool.rs`、`crash.rs` 源码 | 判定一侧没有 `HashMap` / `HashSet` 与自己起的线程（`crash.rs` 并片那一处除外）；第二轮十六个世界复跑逐行相同 |
| 文档注释当不当备注分得出结果 | 这一轮七个变体 | 两种读法判得一样；没造只改文档注释的变体 |

没造的形状（都推的）：树分裂与位置寻址两类流上的 P7 与 F1；136 写单元段（记录跨条）上的符号键；新加 / 删除 / 一拆二一条不变量时旧块怎么读；同一判法在两台上因浮点或内存上限判得不同；RocksDB 真库在杀进程下的原子批；GPU 与 CPU 两份核对实现逐块比。

## 这条腿自己的限度

- 副本上的数，**不算入库装置上的数**。真代码改动只有七处（三行 `crates/mutations.tsv` 的真变异、第一轮 F2 调查员的先根后记录、harness 的 F1 候选、挪一行、mkfs 的水位），都在草稿里的仓副本上改；世界里的历史有录真实现的（第一条流、覆盖写、顺序写），也有手摆的（F1 那两形、r2-b4-messages 的两个高实例节点）。
- 取样与定义 3b 冲突，照正文第五节「只跑小域」办：带 24 写以上单元段的历史那一段只以整段持久进入后面（r3-dump 的第一条流 + 覆盖写：只展开段长 < 16 的段，57 个状态；m57 变体下屏障没了、段变长，293 个），多记录发布的单元段截成前 12 次写。打中的格里 A1①（m57）、A9 m457、A2 正文、A4 都落在展开了的段上；m57 的 36 个漏红是在小域上数的，全量上更多（推的）。
- 「身份 + 属性」下的复用规则是我按 A1 行的字面写成代码的（`compare_dumps.py`：一步连同它前面每一步的内容哈希都没变就复用），换一种合法读法（比如一步的内容哈希把它前面的都串进去）结论不变，因为几格打中都是「内容哈希全不变」。
- 节点代码的三种取法里，「按走到的文件取」没有真的插桩（本机没有 `llvm-profdata`），用的是静态上界：没有非测试 fn 的文件在任何插桩文件集里都不出现。
- `a1_scan.py`、`a1_rename_history.sh`、`a9_static_scan.py`、`a8_fingerprint.sh` 读的是主仓的现状，别的会话改了仓，复跑的数会变（`rerun.sh` 比对时这四份可能「与存档不同」，那是仓变了，不是模型变了）。
- 第一次跑变体时，换回原样用的是带旧修改时刻的拷贝，cargo 按时刻判新旧，把先根后记录的 core 留在了后面两个变体（harness、m457）的编译里；那一趟的六份结果作废（草稿 `out-variants-first-contaminated/`，不引），改成换回时不带旧时刻、换新编译目录全量重跑一遍，报告里的数全出自第二趟。
- 与定义不一致的一处：写范围一节说仓副本用 `rsync -a --exclude target --exclude .git`，A8 要在副本里跑 `git ls-files`，`a8_fingerprint.sh` 拷的副本带了 `.git`（只读、没做 git 写操作）。别的照定义：第一次编译就经 `run-with-memory-cap.sh 8G`；没调任何 `prepare_*`；A8 那两份带 `.git` 的仓副本（886M、891M）是 `a8_fingerprint.sh` 建在草稿里的，跑完删了。
- 挑功能测试时最该先验的（我这边证据最薄的）：①A4 的收严键只在 6 条记录、判定 1 种的段上量过，判定有两种以上的记录段（先根后记录那种改坏的实现 + 多条记录）上没量，同键异判是不是 0 没数；②A3 的 P7「没打中」出自判定只有 1 种的段，分辨力弱；③A1②「冻结文件名」没撞是 14 天历史上的观测，窗口很短；④m57 的漏红 36 是小域上的数，全量上的数没量；⑤节点代码「按 crate 闭包取」省不下多少只是推的，没数。
- 这一轮辩方与本地腿不跑（主 agent 收尾时转来的），这份报告是第三轮唯一一条腿：上面每一格打中都只有这一条腿、这一次观测。

## 没做什么

- 没跑任何 checker 档测试、名字带 layer0 的目标、门禁 54、55、57、59 号、E162、E163，没碰 GPU。跑的测试只有 harness 档一条（`cargo test -p singlefs-harness --lib -- --exact model_comparison::tests::the_model_module_uses_only_the_standard_library_and_the_format_constants`，在草稿副本上，经内存包装）。
- 没判辩方、本地攻方那几格（前两轮判决的复核、代价算术）；没替主 agent 采纳；主仓 `crates/`、kb 一个字没改，只写了这份报告与模型目录。
- 没在入库装置上重做任何一个数；自己提的改法全部零轮，推的那几条没实现。
- 树分裂、位置寻址、记录跨条三类流没造（见「没打中的形状」末段）。

## 复跑核对与草稿

- 交回前照模型目录里的 `rerun.sh` 在一份新副本上从头复跑了一遍（两份仓副本、全量编译一次、七个变体增量编译、一条 harness 档测试、不编译的几件）：33 份输出都「与存档逐行相同（只剥掉 seconds= 与 build_seconds= 字段）」，0 份不同；日志 `/tmp/claude-1000/m3-prune-gpu-r3-opus/rerun-final.log`。第二轮模型在新副本上复跑的十六行在 `/tmp/claude-1000/m3-prune-gpu-r3-opus/r2-rerun.log`。
- 删了（都是这一条腿自己建的）：第二轮复跑的 `r2-rerun/target-repo`（5.3G）、`r2-rerun/target-repo-root-before-records`（5.3G）、`r2-rerun/repo`（369M）、`r2-rerun/repo-root-before-records`（369M）；`work/repo`（371M）、`work/variant`（371M）、`work/target`（5.3G）、`work/target-variant`（5.3G，第一趟那份另在重跑前删过一次，5G 上下）、`work/target-debug`（503M）；A8 的两份带 `.git` 的副本 `a8`（886M）、`a8b`（891M）；终验的 `rerun-check/repo`（375M）、`rerun-check/variant`（375M）、`rerun-check/target`（5.3G）、`rerun-check/target-variant`（5.3G）、`rerun-check/target-debug`（503M）、`rerun-check/a8`（897M）；两份变体用的源码快照 `pristine`（各 9.1M）。
- 草稿目录里留着的（共约 3M，没有编译目录与仓副本）：各次日志、`work/out-base/`、`out-variants/`（第二趟，报告的数出自这里）、`out-variants-first-contaminated/`（第一趟作废的输出，不引）、`model-draft/`（模型目录的草稿）、`patchgen/`、`progress.md`。
