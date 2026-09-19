# 转述核对表：c381-r1-local-attack（2026-09-19）

逐句核对 `research/prompts/c381-r1-local-attack.md` 里每一句转述与中文原文（行号现查 kb 与 crates 源文件，不是背景材料 `_c381-r1-background.md` 的行号；仅第 14、15 两行按定义引用背景材料自己的行号，因为那两条是本轮自己的问题框架，不是 kb 条款）。格式：英文项 / 原文文件:行 / 首稿缺的 / 定稿。

| 英文项（提示文件里的句子，节选） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Fact 1: "persistent write order... fsync... does not return until the root slot write is persisted... every unit that record names must have its checksum individually verified" | `.claude/kb/decisions/16-发布语义.md:287-288`（「一次发布的持久顺序恒为...fsync 等根槽持久之后才返回。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。」） | 无遗漏，逐句核对完整对应 | 按原文直译，未增删 |
| Fact 2: "recovery falls back to the previous instance's root, and replay, following the strict prefix rule, stops at the instance boundary" | `.claude/kb/decisions/16-发布语义.md:293`（「恢复退到上一个实例的根，重放按严格前缀停在实例边界（D23 已定项14 的注1）」） | 首稿写成 "because the prefix rule never crosses an instance boundary"，把括注（D23 已定项14 的注1）的内容当成因果解释塞了进来，原句本身并没有这句因果说明 | 改回贴近字面的 "following the strict prefix rule, stops at the instance boundary"，去掉自造的因果分句 |
| Fact 5: "...a resend would repeatedly rewrite the exact same slot, collapsing R independent failure domains into one" | `.claude/kb/decisions/23-journal的角色与格式.md:691`（「不推进就是反复重写同一个槽，把 R 个失败域用成一个」） | 首稿把「槽」译成 "ring region"，与同句前半「根环区域 = txg mod R」里的「区域」混为一谈，两个中文名词（槽/区域）在原文里不是同一个词 | 改回字面对应词 "slot"，"region" 只保留给「区域」那半句 |
| Fact 6 part 5: "...only applied up to the transaction number W recorded in that rollback row; without this fifth part, an ordinary recovery that landed on the rolled back root would replay the entire abandoned timeline back in, silently undoing the administrator's rollback" | `.claude/kb/decisions/23-journal的角色与格式.md:1225`（「所选根的实例在实例表里有回退行时，该实例的记录只施加到回退行的W为止（C113 定案P2，2026-09-05：否则一次落在R_old上的普通恢复会把被回退抛弃的那段时间线整段重放回来，管理员的回退被静默撤销）」） | 首稿只抄了规则本身，删掉了括注里「否则……回退被静默撤销」这半句理由 | 补回理由半句，译成 "without this fifth part... silently undoing the administrator's rollback" |
| Fact 9: 不含 "攻方历史里 C 与 E 都是 txg 5" 这个具体例子 | `_c381-r1-background.md:34`（「攻方历史里 C 与 E 都是 txg 5」） | 有意不译：攻方 Z2-c 那段具体历史属于 K4（攻方原历史挂不挂得上），K4 明确不归本地攻方腿（分工表：本地攻方分到 K1、K3，不许碰 K2、K4、K6），译进去会把 K4 的材料带进只判 K1/K3 的这条腿 | 保留省略，不补 |
| S1 stage: "before any copy of the journal record has been sent to any device" | `_c381-r1-background.md:42`（「S1 单元写到一半（记录一份都没发）」） | 首稿加了 "to any device" 这个限定，原文只说「一份都没发」没点名设备 | 保留这处添加：与 fact 11「两份都」呼应，注明是本轮自己的问题框架（正文问一），不是 kb 条款，添加是为了让「份数」这个量词落到「设备」这个单位上，避免模型把「一份」理解成别的粒度 |
| S2 stage: "zero, one, or two copies of the journal record may actually have persisted to the two devices" | `_c381-r1-background.md:42`（「记录可能落了一份、两份或零份」）；两块盘的事实见 `.claude/kb/decisions/16-发布语义.md:542`（「两次正好覆盖两块盘」）等多处 | 首稿加了「to the two devices」，原句本身没有点名设备数 | 保留这处添加：本工程第一版固定两块盘是仓里反复出现的既定事实，不是这一条本地攻方腿自己发明的数，添加只是把「份数」与「盘数」对齐，减少模型把「两份」误解成别的东西的空间 |
| Arm A stage S4: "this is today's running code, per the comment in fact 13" | `_c381-r1-background.md:52`（「取号自己的写报错时回卷超级块槽（今天的代码，mount.rs 第849行注释）」）；注释原文见 `crates/singlefs-core/src/mount.rs:849`（「回卷只管取号自己那几次写报错，管不到取号之后的发布失败。」） | 无内容遗漏；把原来的文件名+行号引用（mount.rs 第849行）换成了「fact 13」这种指路方式 | 按本轮规则（提示里禁止出现代码行号与文件行号，答复要按事实编号或函数名指），citation 形式改写，内容未删减 |

## 其余核对：英文比原文多出来的限定词（未改主体，逐条记为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| Fact 3 reason 1: "still alive as of that earlier root" | `.claude/kb/decisions/16-发布语义.md:91`（「而生于 C 之前、死于 C 的块在那时仍然活着」） | 原句「那时」没有点名是哪一个根；英文补成 "as of that earlier root" | 「那时」在上一分句里指的就是「崩溃后回到的 C-1 的根」那一刻，补出来是把代词的先行词钉死，不加新事实 |
| Fact 4: "the guarantee of keeping at least four rollback candidate states" | `.claude/kb/decisions/16-发布语义.md:399`（「最少保留 4 个状态」） | 「状态」被具体化成 "rollback candidate states" | 已定项1整条讲的都是回退候选集，同一决策里「4个状态」在别处（16-发布语义.md:615「最少保留4个」）明确指「可退到的不同状态」，补出「rollback candidate」是把术语落回它在同一文件里已有的所指，不是引入新概念 |
| Fact 8: "resulting version object" | `crates/singlefs-core/src/transaction.rs`（返回类型 `TransactionOutput`，背景材料未直接译出这个类型名） | 把代码类型名 TransactionOutput 意译成 "resulting version object" | 提示要求答复不写代码标识符级别的实现细节、只按功能描述；意译不改变「调用方拿不到这次发布的产出」这个事实 |
| Fact 11: "chains correctly to the previous record" | `.claude/kb/decisions/23-journal的角色与格式.md:1206` 一带（「链接得上」，未点名链到谁） | 补出「to the previous record」 | 「链接得上」在 journal 记录格式里向来指反向链（previous_hash）指向前一条记录，补出对象不引入新事实，只是把「链接」这个动词的宾语显式化 |

## 历史版本

（暂无历史）
