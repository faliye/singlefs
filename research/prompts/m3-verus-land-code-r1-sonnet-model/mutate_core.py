#!/usr/bin/env python3
"""对核（singlefs-replay-prefix/src/lib.rs）的可执行代码做单点改动，用差分装置（改前 replay_journal 为准绳）量每个改动红不红。
用法：mutate_core.py <草稿根> [名字…]；草稿根里要有 tree_m/（冻结副本的拷贝）与 pristine_prefix/（原样备份）、run_m.sh。
每条：(名字, 原文, 替换文, Verus 预测, 理由)。Verus 预测是读 spec 与不变式推的，没跑 Verus；主 agent 要核就把原文/替换文按 verus-mutations.tsv 的格式搬过去。
"""
import subprocess, sys, shutil, pathlib, re

ROOT = pathlib.Path(sys.argv[1])
LIB = ROOT / "tree_m/crates/singlefs-replay-prefix/src/lib.rs"
PRISTINE = ROOT / "pristine_prefix/src/lib.rs"

M = [
 ("A 去掉在飞记录数上限那一断",
  "        if taken >= in_flight_limit {\n            break ;\n        }",
  "        if false && taken >= in_flight_limit {\n            break ;\n        }",
  "绿", "spec 与不变式都不提在飞上限；invariant_except_break 里 taken<=index 只在不 break 的路径上要"),
 ("B 去掉反向链回调那一断",
  "        if back_chain_broken(index) {\n            break ;\n        }",
  "        if false && back_chain_broken(index) {\n            break ;\n        }",
  "绿", "ensures 不提反向链（已定项 14 五条口径第 1、已定项 7）"),
 ("C 去掉『上一条没提交标记而换了事务号』那一断",
  "            if !previous_record_of_the_open_publish.is_commit && record.transaction\n                != previous_record_of_the_open_publish.transaction {\n                break ;\n            }",
  "            if false && !previous_record_of_the_open_publish.is_commit && record.transaction\n                != previous_record_of_the_open_publish.transaction {\n                break ;\n            }",
  "绿", "spec 没有提交标记（五条口径第 3 条）"),
 ("D 去掉『末条不带提交标记』那一断",
  "        if !record.is_commit && record_ends_its_publish(record) {\n            break ;\n        }",
  "        if false && !record.is_commit && record_ends_its_publish(record) {\n            break ;\n        }",
  "绿", "同上"),
 ("E 去掉『末条之后同发布还有读得出的记录』那一断",
  "        if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(\n            records,\n            index,\n        ) {\n            break ;\n        }",
  "        if false && record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(\n            records,\n            index,\n        ) {\n            break ;\n        }",
  "绿", "spec 没有这一格（已定项 4 读者规则）"),
 ("F 点名单元校验恒过",
  "        let all_verified = !verify_named_units || named_units_verified(index);",
  "        let all_verified = true;",
  "绿", "ensures 不提点名校验（五条口径第 4 条）；verification_failed==0 的不变式反而更好证"),
 ("G 去掉 jsn 严格连续那一断（已知：核没证它）",
  "                if record.instance != expected_key.0 || record.counter != expected_key.1 {\n                    break ;\n                }",
  "                if false && (record.instance != expected_key.0 || record.counter != expected_key.1) {\n                    break ;\n                }",
  "绿", "主 agent 2026-10-01 已现跑坐实（第二轮判决第二节第 2 行）"),
 ("H 无锚点时不看 txg = 根 txg + 1，只看序号",
  "                if record.checkpoint_txg != chain_start_txg_without_anchor\n                    || record.ordinal_within_publish != 1 {\n                    break ;\n                }",
  "                if record.ordinal_within_publish != 1 {\n                    break ;\n                }",
  "绿", "spec 没有链首 txg（已定项 14 注 1）；不变式只留了 chain_start 的等式"),
 ("I 无锚点那一支不看序号（tsv-448 原来那条；核里另有第 456 行那一判兜着）",
  "                if record.checkpoint_txg != chain_start_txg_without_anchor\n                    || record.ordinal_within_publish != 1 {\n                    break ;\n                }",
  "                if record.checkpoint_txg != chain_start_txg_without_anchor {\n                    break ;\n                }",
  "绿", "不变式 expected_next is None ==> open 为空 + 第 456 行的判，返回值同；读盘次序不同（反向链回调先于序号那一判）"),
 ("J maximum_applied_transaction 不更新",
  "        report.maximum_applied_transaction = maximum_transaction_of(\n            records,\n            open.as_slice(),\n            report.maximum_applied_transaction,\n        );",
  "        report.maximum_applied_transaction = report.maximum_applied_transaction;",
  "绿", "ensures 只管 prefix_applied 与 rebuilt，不管这个字段；maximum_transaction_of 没有 ensures"),
 ("K 一条都不施加（取到第一条就收工）",
  "        // [原文 3109] take(in_flight_limit)\n        if taken >= in_flight_limit {",
  "        if true {\n            break ;\n        }\n        if taken >= in_flight_limit {",
  "绿", "空前缀满足 m2_holds：空拼接为真、prefix_applied==0、rebuilt==root_segment"),
 ("L 控制：prefix_applied 少加",
  "        report.prefix_applied += open.len();",
  "        report.prefix_applied += open.len() - 1;",
  "红", "不变式 report.prefix_applied == applied.len() 与 ensures 都管它"),
 ("M 控制：去掉『一次发布第一条序号是 1』那一断",
  "        if open.is_empty() && record.ordinal_within_publish != 1 {\n            break ;\n        }",
  "        if false && open.is_empty() && record.ordinal_within_publish != 1 {\n            break ;\n        }",
  "红", "is_open_publish 要序号 == 位置 + 1，锚点读得出那一支只靠这一判"),
 ("N 控制：去掉『txg 与攒着的上一条相同』那一断",
  "            if record.checkpoint_txg != previous_record_of_the_open_publish.checkpoint_txg {\n                break ;\n            }",
  "            if false && record.checkpoint_txg != previous_record_of_the_open_publish.checkpoint_txg {\n                break ;\n            }",
  "红", "is_open_publish 要同 txg"),
]

def run(name_filter):
    base = PRISTINE.read_text()
    for name, old, new, prediction, why in M:
        if name_filter and name.split()[0] not in name_filter:
            continue
        hits = base.count(old)
        assert hits == 1, f"{name}: 原文命中 {hits} 次"
        LIB.write_text(base.replace(old, new))
        log = f"mut_{name.split()[0]}.log"
        subprocess.run(["bash", str(ROOT / "run_m.sh"), log, "--", "zz_diff::random_family", "zz_diff::exhaustive_structural_quick", "--nocapture", "--test-threads=1"], env={**__import__('os').environ, "DIFF_N": "500000"})
        text = (ROOT / log).read_text()
        results = re.findall(r"RESULT\[[^\]]*\] Tally \{ total: (\d+), diverged: (\d+)", text)
        labels = re.findall(r"RESULT\[([^\]]*)\]", text)
        compiled = "error[" not in text and "could not compile" not in text
        print(f"MUT {name} | Verus 预测={prediction} | 编译={'过' if compiled else '没过'} | " + "; ".join(f"{l}: {d}/{t}" for l, (t, d) in zip(labels, results)), flush=True)
    LIB.write_text(base)

if __name__ == "__main__":
    run(set(sys.argv[2:]))
