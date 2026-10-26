回复与报告用中文。

重型测试：不跑
批次：records/2026-10-18-里程碑三收尾与近期欠账.md#批 10
判决：research/prompts/m3-close-c693-r3-main-verification.md（用户定案在文末）、research/prompts/m3-close-c656-r3-main-verification.md（用户定案在文末）、research/prompts/m3-close-rootring-r3-main-verification.md；调度记录 records/2026-10-18-里程碑三收尾与近期欠账.md 第 316、382、431、441 行与文末批 10 那几条
草稿目录：/home/fy5090/.local/share/singlefs/work/m3-close-w10-kb-milestones-layout
认领：bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m3-close-w10-kb-milestones-layout --purpose 'kb 写回批：里程碑与布局表'
报告：/home/fy5090/.local/share/singlefs/work/m3-close-w10-kb-milestones-layout/report.md
规格检查已判：写回员自己起草并跑 kb-spec-check.py
快照冲突已判：m3-batch63-code-r1 是另一会话叫停、不写判决的轮（records/2026-10-13-复用键与全仓改名.md 第 46 行），这一件只改 .claude/kb/milestone/03-third-txn.md 的作业点与出口行，不碰它快照里的 crates 文件

## 写范围

只改这几份：
- `.claude/kb/milestone/03-third-txn.md`
- `.claude/kb/milestone/08-eighth-txn.md`
- `.claude/kb/milestone/09-ninth-txn.md`
- `.claude/kb/milestone/14-fourteenth-txn.md`
- `.claude/kb/milestone/16-sixteenth-txn.md`
- `.claude/kb/layout/01-first-txn.md`

不碰的：
- `.claude/kb/milestone/04-fourth-txn.md`：里程碑四的会话在改，主 agent 自己写。
- `milestone/01-first-txn.md` 不改：里程碑一那条验收已经过了。
- 布局表 `:114`、`:162`、`:410` 三行与 C773 宽度连带那几行（`:60`、`:64`、`:67`、`:82`、`:103`）这一批不写，等实现批。

三语译文不写。

已知：`kb-spec-check.py` 会把已暂存的 `.en.md` 译文当登记位，报假红（`records/2026-09-16-subagent拆分提案.md` 第七十七条）。在只有中文 kb 的镜像根上跑，绿了就照规格写。报告抄两趟的末行。

## 关的是哪几条

条目清单 `/home/fy5090/.local/share/singlefs/work/m3-close-w10-writeback-inventory/inventory.md`（sha256 f0fb9d01…）里这几份文件的分组。动手前按整行重新定位。

1. **`出口5同宽`**：里程碑三出口表第 5 行改成与 D13 已定项 1 同宽（调度记录第 316 行）。
2. **`C693-墓碑挪八`、`C693-删头挪十四`**：
   - 里程碑三里点名 C693 的行把 C693 挪出；
   - 墓碑折叠（C344 并进 C383）进里程碑八一·1；
   - 删头台阶行进里程碑十四二·2，并补前提「按头的谱系记录形态」（用户「两件都挪」）。
3. **`C563-C584-销账`**（里程碑三两行）：
   - C584 销账，从作业点里拿掉；
   - C563 不销账，射程收窄到登记点名之外那四处调用方（`mount.rs` 两处、`recovery.rs` 一处、`mount.rs` 一处，调度记录第 382 行），作业点保留。
4. **新欠账落作业点**：
   - C863 钉槽的两类坏实现镜像上判不出：里程碑三二·4，与 C773 同点；
   - C866 F2 退回不分字节丢了与内容拒收：里程碑三二·4，与 C755 同批；
   - C867 另外六处读者只拒窄于字段表的条目宽：里程碑三二·5，与 C780 同点；
   - C864 别的自证结构的读错位没人查：里程碑十六一·1（全量 scrub），主 agent 定。
   作业点编号以文件现状为准；对不上的点名报告。
5. **C656 作业点**：里程碑三 C656 那个作业点写进用户定案，四样：
   - 乙两趟写序；
   - 对外入口从随机源生成 fsid、留内层入口，验收改「同参数同 fsid 两次逐字节相同」；
   - 新增一条从用过的盘起枚举 mkfs 自己的崩溃枚举用例；
   - C659 验收格改成「前 8 字节碰撞时 I-7.7 红、读者读对、红随取号自愈」，钉一张坏镜像。
6. **`段序列-取号两轮`**（`layout/01-first-txn.md:422`）：固定脚本流段序列开头的取号段照批 9 甲改：1678377204 → 1678377220，到 D 36 → 38 段。报告 `~/.local/share/singlefs/work/m3-close-w9-landing-pins/report.md`，数去补丁现查。

## 出口

- 六条照规格写完，镜像根上 `kb-spec-check.py` 绿。
- 报告列每处旧行、新行与依据。
- 判决没给、要判断的，停下点名，不自己补。