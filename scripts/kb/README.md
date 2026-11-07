# scripts/kb

kb 与记录的定点改写、三方材料的抄录、引文与外部文献、回扫与变更史整理。

| 文件 | 做什么 |
|---|---|
| `replace-once.py`、`replace-batch.py`、`insert-row.py`、`lib_atomic_replace.py`、`lib_notation_check.py` | 共写文件时的定点替换、批量替换、插一行；`replace-once.py` 与 `insert-row.py` 写仓里的 `.md` 之前用 `lib_notation_check.py` 判新串里的钟点、时区词、时间戳（判据是 `.claude/gate.d/lib-forbidden-notations.py` 的钟点形态，与写入闸、门禁 doc-text 同一份），带的拒绝写入（退出码 7 与 6）；`lib_atomic_replace.py` 分开报改名之前的失败（`ReplaceRefused`，原文不动）与改名之后的打断（`ReplacedThenInterrupted` / `ReplacedThenFailed`，新内容已换上），调用方照它分开退出码，后一种不当没写重跑 |
| `ledger-append-section.py` | 往拆分提案现行账本（登记表 `.claude/gate.d/split-proposal-ledgers.tsv` 里 current 那一份）追加一节：现取最大节号加一，一次写入索引表末行与末尾新节；从读账本、取号到改名换上罩在账本锁里（账本那个 inode 上的 flock，拿到锁再核路径还指着它），两个会话同时用它追加时后到的等前一个换上之后取下一个号，等锁超时退 10；不拿锁的写法（Edit、`replace-once.py` 这类）读到写之间改过账本的，写前复核 sha256 拒（退出码 4，重跑）；新文本带钟点、时区词、时间戳或会话号同样拒（6、7）；正文不收一、二级标题（去掉 0–3 个前导空格认，围栏按 `lib_markdown.py`；8）；`write-guard.sh` 与 `bash-command-detector.sh` 拦直接往现行账本追加「## <节号>、」标题，出路指到它 |
| `append-record.py` | 往 `records/` 下一份已有的 `.md` 末尾追加一段文本：写之前照写闸、门禁 doc-text 同一份判据判钟点、时区词、时间戳（退出码 6）与会话号、批次代号（7），命中就拒、点名追加之后的第几行（共用 `lib_notation_check.py`）；不收一、二级标题（去掉 0–3 个前导空格认，围栏按 `lib_markdown.py`；8；新开一节用 Edit，现行账本加节经 `ledger-append-section.py`）；读时记 sha256、换上之前复核、同目录临时文件加改名换上，撞了拒（4）；`bash-command-detector.sh` 的 ⑲ 拦 Bash 里往 `records/` 下 `.md` 的追加（`>>`、`&>>`、`tee -a`、python 的 `open(…, 'a')`），出路指到它 |
| `quote-kb.py`、`kb-sections.py`、`quote-rust-items.py` | 整段抄 kb 条款（回读之外另用 `kb-sections.py` 的 `heading_bounds()` 另算 `@标题` 取法的起止行，对不上退 7；「不抄」理由点名的节出口里没有退 8；`--cited` 认路径引的 kb 文件；正向核对按清单点名的那份文件整节核，别的文件里的同名标题、只抄了标题行都算没抄，退 4）、生成小节清单（`heading_bounds()` 供 `quote-kb.py` import）、按项名抄 Rust 代码（连同项上方的文档注释与属性一起抄，属性认不准退 6，不交半截） |
| `kb-spec-check.py`、`relabel-item.py`、`sweep-term.py`、`stale-candidates.py`、`batch-target.py`、`stale-candidates-benchmark-facts.tsv`、`stale-candidates-record-carriers.tsv`、`lib_basis_paths.py` | 写回规格检查（可一次给几份同批规格做跨规格状态词比对；译文编号照术语表写半角译名；「依据」里的路径不许是已跟踪文件里没进 `.claude/batch-scope` 的未提交改动，未跟踪的新文件放行并另列名单）、分项改标签、术语改名、阶段同步的候选表与它的盲写基准、records/ 下哪几份记录算现状载体的登记表（`--check-facts` 同样核「出处」里的路径；`--changes` 不给 `--target` 而工作区或暂存区有相对 HEAD 的改动时拒跑，出路是 `batch-target.py`，有改动的路径全登记在 `.claude/batch-scope` 里的才照跑，没有单次放行的参数）、把这一批的改动（点名整份取的路径、按块挑的块——块的切法同 `scripts/gate/stage-mine.py`，HEAD 里没有的文件以真索引或搬迁前旧路径那一版当底——与已打在工作区上的补丁目录 `--patch`，倒掉补丁再三方合成）单独做成一个不进任何分支的提交对象当 `--target`（临时索引，真索引与 ref 不动，标准输出只打哈希）、两者共用的依据路径核法 |
| `history-consolidate.py`、`doc-consolidate.py`、`decision-slim-check.py`、`rules-sweep-audit.py` | 变更史与文档整理、瘦身后的丢失核对（`history-consolidate.py loss-check` 比对产物行时，反引号里的 `<路径>` 与 `git show <提交>:<同一路径>` 算同一处，路径、产物名改了别的字照旧判丢） |
| `fetch-refs.sh`、`pdf-text.py`、`verify-citations.sh` | 取回并固定 `docs/` 下七份 PDF（两棵源码树与 `docs/rfc8439.txt` 不归它，各自的取得方式与校验边界写在 `verify-citations.sh` 的树登记表 `TREE_LIST`）、抽 PDF 文字、核承重引文 |
| `claim-experiment.sh`、`lib_experiment_number_directories.py`、`archive-past-rounds.py` | 取实验号（查号照目录登记 `.claude/gate.d/experiment-number-directories.tsv`，读法在 `lib_experiment_number_directories.py`，门禁 doc-experiments 的 experiment-orphans 格读同一份）；早于本轮的实验记录归档（删完把引用改成 `git show <删之前的提交>:<原路径>`；可重跑，磁盘上已删而索引里还跟踪着的只报数） |
| `milestone-close-owed-check.py` | 里程碑收口前核欠账表开着的行里还有没有点名它的账：只认「去向：」之后开头的落点（里程碑文件、「里程碑<号>」、「未排」，「、」连写的几个都算），落点之后的说明不算，门禁 doc-process-records 的 milestone-close-owed 格调 `--closed`；`--destination-form` 判开着的每一行去向段开头是不是这三种落点之一、落点的里程碑文件在不在，同一道门禁的 owed-destination-form 格调它 |
| `path-successor.py` | 此刻不在的仓内路径找关联路径并写出依据（搬迁登记、git 改名链、删它的提交与前一版取法 `<提交>^:<路径>`、那次提交里内容重合不低于三成的同扩展名文件、同名唯一、从没进过 git 时按后缀唯一）；门禁 doc-registries 的 governance-refs 格 import 它，红行带它的结论句 |
| `lib_markdown.py` | 文档整理工具与两个追加工具（`append-record.py`、`ledger-append-section.py`）共用的 Markdown 围栏状态，区分标记字符、长度与缩进，保护代码中的标题和原文 |
| `tests/test_review_regressions.py` | 隔离临时目录与临时 Git 仓中的审核回归；运行 `python3 -m unittest discover -s scripts/kb/tests -v`，下载测试使用本地替身 |

批量替换将同一文件的路径别名合并成顺序计划；空锚点和非正整数 `count` 拒绝，写前复核原字节。逐处写盘的旧行为开关 `REPLACE_BATCH_WRITE_EACH` 与 `REPLACE_BATCH_BREAK` 过同一道入口守卫，只在自证里认，平常跑设了退 2；`--dry-run` 不论设没设它都只核不写。实验占号依赖 `flock`，同仓的工作树共用 Git 公共目录中的锁，并统一去掉编号的前导零。工作树清单按 NUL 分隔读取；根路径含换行时，现有逐行查号接口无法表示，明确拒绝占号和查询下一个编号。

归档保留带 `assume-unchanged` 或 `skip-worktree` 索引标记的文件；依据检查对未显式放行的这类路径报错，因为 Git 可能跳过其工作区差异。归档在差异检查前记录文件状态，删除前再复核；这不是跨进程事务，复核与删除之间仍不能与任意外部写入互斥。PDF 抽取遇到损坏或截断的 FlateDecode 流会报错，不以部分解压结果充当完整证据。

去重的边界：命令入口、退出码和各自的任务保留。批量替换的钟点与会话名检查共用新串遍历，并复用锚点核对时的原文快照；三个文档整理脚本与两个追加工具共用 `lib_markdown.py` 的顶层围栏状态。`quote-kb.py` 与 `kb-sections.py` 的节边界复核、迁移结果的独立内容核对仍保留各自算法，避免执行与核对共用同一份变换而同时漏错。普通文档整理、变更史整理和文件间迁移的语义不同，不合成一个大脚本。

`fetch-refs.sh` 下载时记录完整 SHA-256 到文献旁的 `.sha256` 文件和取得日志；`--check` 只读，并与记录比较。旧缓存缺少记录时会拒绝校验，需要重新下载比对；与旧副本相同才建立记录，不同则保留旧副本并拒绝覆盖。PDF 丢失但摘要仍在时，只恢复与原摘要一致的副本；摘要损坏或下载内容不同均拒绝，不重置原记录。哈希不符时先保留旧副本、查明原因。这里校验的是下载后内容是否变化，不代表出版方签名验证。
