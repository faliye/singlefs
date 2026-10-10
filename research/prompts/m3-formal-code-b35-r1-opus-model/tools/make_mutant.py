# 攻方原型：从冻结副本的 walk.rs 拷出三份 CPU 变异（每份恰好一处替换，锚点恰好命中一次）。
import sys
path, which = sys.argv[1], sys.argv[2]
s = open(path).read()
pairs = {
    # mutations.tsv「第35批 35-1 FL：由记录施加出来的那一版照旧取先到的那条记录」同一改法
    "mutr": ("                ) => carries_the_last_record_flag,", "                ) => false,"),
    # mutations.tsv「第35批 35-1 FL：下一次挂载先施加的那一版照旧取先到的那条记录」同一改法
    "mutn": ("            record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0\n                && record.commit_marker == CommitMarker::Present\n                && record.instance == newest_root.instance",
             "            record.commit_marker == CommitMarker::Present\n                && record.instance == newest_root.instance"),
    # GPU 走读内核「Q3 甲：树表指针全零照旧当没有树表」那一行在 CPU 的对应：三类版本都不跟过去
    "mutz": ("""        if tree_table_pointer_view.all_zero {
            self.follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core(
                &tree_table_pointer_view,
                version_name,
                tree_table_pointer_holder,
            );
            return;
        }""", """        if tree_table_pointer_view.all_zero {
            let _ = tree_table_pointer_holder;
            return;
        }"""),
}
old, new = pairs[which]
assert s.count(old) == 1, (which, s.count(old))
open(path, "w").write(s.replace(old, new))
