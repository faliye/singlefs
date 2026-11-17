#!/usr/bin/env python3
"""把攻方模型插进冻结副本的拷贝（只改拷贝）：替身卡加一个字段，让它的量代价照产品那一份走账；两处 `mod tests` 末尾插进模型用例。
用法：apply_model.py <拷贝的仓根> <模型目录>
每一处替换的原文在文件里要恰好命中一次，不然退 3、一个字都不写。"""
import pathlib
import sys


def replaced_once(text, old, new, where):
    count = text.count(old)
    if count != 1:
        print(f"  ✗ {where}：原文命中 {count} 次（要 1 次）", file=sys.stderr)
        sys.exit(3)
    return text.replace(old, new)


def inserted_before_the_last_brace(text, insertion, where):
    if not text.endswith("}\n"):
        print(f"  ✗ {where}：文件末尾不是 mod tests 的收尾大括号", file=sys.stderr)
        sys.exit(3)
    return text[: -len("}\n")] + insertion + "}\n"


def main():
    if len(sys.argv) != 3:
        print("用法：apply_model.py <拷贝的仓根> <模型目录>", file=sys.stderr)
        sys.exit(2)
    tree = pathlib.Path(sys.argv[1])
    model = pathlib.Path(sys.argv[2])
    background_gpu = tree / "crates/singlefs-background-gpu/src/background_gpu.rs"
    live_slot = tree / "crates/singlefs-background-gpu/src/live_slot_kernels.rs"
    text = background_gpu.read_text(encoding="utf-8")
    text = replaced_once(
        text,
        "        cost_measurement_failing_with: Option<GpuFailureCause>,\n        drops_counted: Arc<std::sync::atomic::AtomicU32>,\n    }\n",
        "        cost_measurement_failing_with: Option<GpuFailureCause>,\n        drops_counted: Arc<std::sync::atomic::AtomicU32>,\n        cost_measured_through_the_checksum_chunk: bool,\n    }\n",
        "替身卡的字段",
    )
    text = replaced_once(
        text,
        "                cost_measurement_failing_with: None,\n                drops_counted: Arc::new(std::sync::atomic::AtomicU32::new(0)),\n",
        "                cost_measurement_failing_with: None,\n                drops_counted: Arc::new(std::sync::atomic::AtomicU32::new(0)),\n                cost_measured_through_the_checksum_chunk: false,\n",
        "替身卡的构造",
    )
    text = replaced_once(
        text,
        "            self.cost_measurements += 1;\n",
        "            self.cost_measurements += 1;\n"
        "            if self.cost_measured_through_the_checksum_chunk {\n"
        "                self.chunk_submitted_on_the_cpu(COST_PROBE_CHUNK.bytes_held(), || ())?;\n"
        "            }\n",
        "替身卡的量代价",
    )
    text = inserted_before_the_last_brace(
        text,
        (model / "patch/background_gpu_tests.rs.txt").read_text(encoding="utf-8"),
        "background_gpu.rs",
    )
    live_slot_text = inserted_before_the_last_brace(
        live_slot.read_text(encoding="utf-8"),
        (model / "patch/live_slot_tests.rs.txt").read_text(encoding="utf-8"),
        "live_slot_kernels.rs",
    )
    background_gpu.write_text(text, encoding="utf-8")
    live_slot.write_text(live_slot_text, encoding="utf-8")
    print("name=apply_model replaced=3 inserted=2")


main()
