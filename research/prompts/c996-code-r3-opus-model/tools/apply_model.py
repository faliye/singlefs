#!/usr/bin/env python3
"""把攻方腿模型（c996-code-r3）插进冻结副本拷贝里三份文件的测试模块末尾（只改拷贝）。
用法：apply_model.py <拷贝的仓根> <模型目录>
每份目标文件要以测试模块收尾的「}\\n」结尾、里面还没有这一份模型的标记名，不然退 3、一个字都不写。"""
import pathlib
import sys

INSERTIONS = (
    (
        "crates/singlefs-background-gpu/src/background_gpu.rs",
        "src/attack_r3_background_gpu_tests.rs",
        "fn attack_r3_switch_back_while_an_opening_is_held(",
    ),
    (
        "crates/singlefs-background-gpu/src/live_slot_kernels.rs",
        "src/attack_r3_live_slot_tests.rs",
        "fn attack_model_a_repeated_segment_meets_the_full_list_assertion_first(",
    ),
    (
        "crates/singlefs-core/src/background_compute.rs",
        "src/attack_r3_core_tests.rs",
        "fn attack_r3_wake_with_a_failing_batch(",
    ),
)


def main():
    if len(sys.argv) != 3:
        print("用法：apply_model.py <拷贝的仓根> <模型目录>", file=sys.stderr)
        sys.exit(2)
    tree = pathlib.Path(sys.argv[1])
    model = pathlib.Path(sys.argv[2])
    new_texts = []
    for target, insertion, marker in INSERTIONS:
        text = (tree / target).read_text(encoding="utf-8")
        if marker in text:
            print(f"  ✗ {target}：已经插过（含 {marker}）", file=sys.stderr)
            sys.exit(3)
        if not text.endswith("\n}\n"):
            print(f"  ✗ {target}：末尾不是测试模块收尾的大括号", file=sys.stderr)
            sys.exit(3)
        added = (model / insertion).read_text(encoding="utf-8")
        new_texts.append((tree / target, text[: -len("}\n")] + added + "}\n"))
    for path, text in new_texts:
        path.write_text(text, encoding="utf-8")
    print(f"name=apply_model inserted={len(new_texts)}")


main()
