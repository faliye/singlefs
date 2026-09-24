# 本地攻方运行记录（m2-placement-falsepositive-r1）

提示文件：research/prompts/m2-placement-falsepositive-r1-local-attack.md
核对表：research/prompts/m2-placement-falsepositive-r1-local-attack-translation-audit.md

## 一、逐次调用

前台跑 `bash research/scripts/ask-local.sh research/prompts/m2-placement-falsepositive-r1-local-attack.md`，未用 setsid / & / disown。共 4 次调用，2 次判红作废、2 次判绿（占用 s1、s2 两个编号）。

| 次序 | 落盘文件 | 退出码 | 字词损坏闸判定 | 词数（wc -w） | oov-check.py 生词清单 | 分类 |
|---|---|---|---|---|---|---|
| 1 | m2-placement-falsepositive-r1-local-attack-output-s1.md | 0 | corruption-check.py 绿；oov-check.py 绿（生词=17，拼接=0） | 361 | falsified、criterion's（其余为 falsified 一词的重复出现，dict.fromkeys 去重后只剩这两个不同词干） | 干净 |
| 2（第一次判红，落 void1） | m2-placement-falsepositive-r1-local-attack-output-void1.md | 5 | oov-check.py 判红：拼接 meaningfully(=meaning+fully) | 未计（判红即作废，不计入干净样本） | meaningfully | 作废（判红即作废，不当成三方不一致） |
| 2（第二次判红，落 void2） | m2-placement-falsepositive-r1-local-attack-output-void2.md | 5 | corruption-check.py 判红：实词自复读 disagreements（原文「placement disagreements disagreements completes」，同一实词连续出现两次，真实的模型吐重，不是检测器误报） | 未计 | 不适用（corruption-check.py 判红，未跑到 oov-check.py 那一步的独立清单；见下方复核） | 作废 |
| 2（第三次，落 s2） | m2-placement-falsepositive-r1-local-attack-output-s2.md | 0 | corruption-check.py 绿；oov-check.py 绿（生词=17，拼接=0） | 515 | falsified、Reintroduce（其余为 falsified 一词的重复出现） | 干净 |

## 二、判红两次的复核记录

第一次判红（void1）：oov-check.py 把 "meaningfully" 切成 "meaning"+"fully" 判成粘连词（规则 A：两截各自都在词表里、长度都 ≥5）。人工通读 void1 全文，这是一个标准英语副词、拼写完整，不是缺头粘连词（`achievesceives` 那一类），像是检测器规则 A 对「实词+ly 派生副词」缺一条豁免。按规则「闸判红之后，那一轮作废重跑，不许记成三方不一致」，不因为疑似检测器误报而采用，仍旧作废重跑；未设 `ASK_LOCAL_ALLOW_CORRUPT=1`。

第二次判红（void2）：`disagreements disagreements` 原样连续出现两次、中间只隔一个空格，人工核对上下文（Z2.1 那一行）确认是真实的模型重吐，不是检测器误报，作废重跑正确。

两次判红都不计入「干净样本」，也都不算一次独立观测；这一轮的样本数以 s1、s2 两份干净样本为准。

## 三、F4（提示里）检索记录

检索目标：仓内是否有测试在释放（release）时机对单元校验和（unit_checksum）做断言。

命令：`grep -rn "unit_checksum" crates/singlefs-core/src/transaction.rs`（release 判定所在文件），命中 0 处。

命令：`grep -rln "unit_checksum" crates/singlefs-harness/tests/*.rs crates/singlefs-core/src/*.rs`，命中：
first_transaction_step_five_publish.rs、first_transaction_step_one_mkfs.rs、first_transaction_step_two_data_unit.rs、second_transaction_supplement_three_bad_disk_input.rs、second_transaction_step_zero_layer0.rs（均为写入/读取/恢复时机的断言，逐一读过，无一处是在 `placements_to_release_via_mapping` 这条释放路径上断言校验和）；以及三个非测试源文件 pointer.rs、records.rs、recovery.rs、make_filesystem.rs、mounted_read.rs（字段定义与读路径校验，非测试）。

## 四、闲置计数

至此已用 4 次调用，未达到「连续五次调用（判红作废的也算）拿不到两份干净样本才停」的上限；两份干净样本（s1、s2）已到手，按规则停止抽样。

## 五、没做什么

未运行 cargo test / cargo build（本轮定义未要求，agent-common「不做」一节禁止未经明写的编译）。未读取云端攻方（Opus）、云端正推（Sonnet）两条腿的输出文件，避免样本间互相沾染。未对样本内容做解读、总结或采纳，也未判两份样本方向是否一致——那是主 agent 的事。
