"""改法甲（攻方腿提的，只在攻方腿的模型上量过、被攻过零轮）：第一个文件一个数据单元都没有时上段叶一律写标签 0，size 0 也写。

用法：python3 apply_fix_candidate_a.py <副本里的 crates/singlefs-core/src/extent_tree.rs>；旧串不是恰好一处就退 3，一个字节不写。
"""
import pathlib
import sys

OLD = """    pub const fn of_a_file_without_data_units(size_in_bytes: u64) -> Option<Self> {
        if size_in_bytes > 0 {
            Some(ExtentUpperLeafTarget::NoDataUnit)
        } else {
            None
        }
    }"""
NEW = """    pub const fn of_a_file_without_data_units(size_in_bytes: u64) -> Option<Self> {
        let _ = size_in_bytes;
        Some(ExtentUpperLeafTarget::NoDataUnit)
    }"""

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
if text.count(OLD) != 1:
    print(f"旧串在 {path} 里命中 {text.count(OLD)} 次，要恰好 1 次", file=sys.stderr)
    sys.exit(3)
path.write_text(text.replace(OLD, NEW), encoding="utf-8")
print(f"name=fix_candidate_a applied={path}")
