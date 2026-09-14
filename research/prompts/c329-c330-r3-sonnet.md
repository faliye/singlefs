你是 C329 与 C330 三方论证第三轮的**辩方腿**。立场：复核第二轮判决（`research/prompts/c329-c330-r2-main-verification.md` 第四节）——找得出读错、推错、漏看的地方就报，找不出就写「辩不动」；之后正推 X4（⑤ 的副作用：行删不掉时实例表长到多少、切换预留受不受影响）与 X6（代价与条款）。

## 先读

1. 背景材料：`research/prompts/_c329-c330-r3-background.md`。
2. 第二轮：`research/prompts/c329-c330-r2-main-verification.md`、`c329-c330-r2-opus-output.md`、`c329-c330-r2-sonnet-output.md`。
3. 自己去查（引用时写 kb 文件自己的行号）：`.claude/kb/decisions/18-块里携带什么信息.md` 第 879 行、`.claude/kb/decisions/23-journal的角色与格式.md` 第 691、1235、1236 行、`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 3、`.claude/kb/invariants.md` I-1.2 与 I-3.8 两行。

## 要做的

- X1：① Q330 的出局——它的反例（新实例的记录从前缀末 + 1 接着写，覆盖读不出的那条根唯一的记录见证）能不能不改 D23 已定项 14 注 3 就救回来；② 写序主人那一收严——有没有一次切换照旧了两个实例的事务、主人不止一个的情形；③ ⑤ 之后行会不会永远删不掉。
- X4、X6：数出来，第一个事务的字节变不变，逐字抄出冲突的两句（若有）。

## 交付

- **分段写**：报告写进 `research/prompts/c329-c330-r3-sonnet-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），后面用 `>>` 追加。
- 引条款整行抄，行号写 kb 文件的；不许用「本条」「本节」「上文」这类指代。不许改仓里任何已有文件、不许 git 的任何写操作。请在大约 30 分钟内交出。最后回复只写一句指向报告。
