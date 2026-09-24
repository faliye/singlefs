# m2-checker-supp-code-r1 云端攻方腿（K1 / K2）的探针

两份都是 Rust 集成测试，只在仓副本上跑过，**不入库、不进主工作区**。副本做法：
`rsync -a --exclude target --exclude .git <仓> /tmp/claude-1000/m2-checker-supp-r1-opus/repo/`，
基准是 HEAD `11a551b` 加上这一批未提交的改动（开工快照 `research/prompts/m2-checker-supp-code-r1-start-snapshot.sha256` 九个文件全部 OK）。

## 文件

| 文件 | 放到副本的哪里 | 跑法 |
|---|---|---|
| `opus_probe_txn_chain.rs` | `crates/singlefs-harness/tests/opus_probe_txn_chain.rs` | `nice -n 19 cargo test -p singlefs-harness --test opus_probe_txn_chain` |
| `step_five_write_order_scan.rs.append` | 追加到 `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` 末尾 | `nice -n 19 cargo test -p singlefs-harness --test second_transaction_step_five_reuse opus_probe -- --nocapture` |

## 判别力自证（写序扫描那一份）

把 `crates/singlefs-core/src/transaction.rs` 第 1247 行改回旧写法
（`transaction: previous.record.transaction + 1,`），同一份扫描必须判红：
盘 0 上八个码 1 单元的写序去重之后只剩七个，重的那一对是 `(3, 1)`。
改回来之后再跑必须转绿（八个写序互不相同）。

## 这两份探针不证明什么

- 不判 K3（checker 判据本身）与 K4（早退条件），那两面归别的腿。
- 副本上量出来的数不算入库装置上的数：要引用先在入库装置上重做一次。
