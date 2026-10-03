"""在仓副本的 crash_amplification.rs 里，把四行变异（1587、1731、1732、1733）各做成一个环境变量开关：
开关设了时与那一行的替换文同义，没设时与冻结副本逐字同义。只给原型用，一次编译跑几种。
用法：python3 patch_mutations.py <仓副本根>（在 r2 的 patch_tree.py 之后）"""
import sys, os
path = f"{sys.argv[1]}/crates/singlefs-checker-tier/src/crash_amplification.rs"
s = open(path, encoding="utf-8").read()
def once(old, new):
    global s
    assert s.count(old) == 1, old
    s = s.replace(old, new, 1)
on = lambda name: f'std::env::var_os("{name}").is_some()'
# 1587 崩溃放量 复用键不罩写表
once('''            write_table_with_barriers: step_write_table_with_barriers(
                flow.writes,
                &owned_segments,
                first_write..last_write,
            ),''', f'''            write_table_with_barriers: if {on("R3_MUT_1587")} {{
                let _ = (&owned_segments, last_write);
                [0; DIGEST_BYTES]
            }} else {{
                step_write_table_with_barriers(
                    flow.writes,
                    &owned_segments,
                    first_write..last_write,
                )
            }},''')
# 1732 崩溃放量 块键只接点名（M3 读回）
once('''        VerdictBlockKey {
            input_fingerprint: InputFingerprint(*self.reuse_key.as_bytes()),
            stream_or_node_name: StreamOrNodeName::new(&self.identity)
                .expect("节点名不超过 u16::MAX 字节"),
            enumeration_plan_hash: EnumerationPlanHash(self.plan_hash),
            block_start: BlockStartStateIndex(local.start),
        }''', f'''        if {on("R3_MUT_1732")} {{
            return VerdictBlockKey {{
                input_fingerprint: InputFingerprint([0; DIGEST_BYTES]),
                stream_or_node_name: StreamOrNodeName::new(&self.name)
                    .expect("节点名不超过 u16::MAX 字节"),
                enumeration_plan_hash: EnumerationPlanHash([0; DIGEST_BYTES]),
                block_start: BlockStartStateIndex(local.start),
            }};
        }}
        VerdictBlockKey {{
            input_fingerprint: InputFingerprint(*self.reuse_key.as_bytes()),
            stream_or_node_name: StreamOrNodeName::new(&self.identity)
                .expect("节点名不超过 u16::MAX 字节"),
            enumeration_plan_hash: EnumerationPlanHash(self.plan_hash),
            block_start: BlockStartStateIndex(local.start),
        }}''')
# 1731 崩溃放量 第一个点之前没有根槽写时版本表整张进判器输入（J1）
once('''                    .chain(self.newest_root_in_the_base_image)''',
     f'''                    .chain(self.newest_root_in_the_base_image.filter(|_| !{on("R3_MUT_1731")}))''')
# 1733 崩溃放量 根槽写 txg 相等也当严格递增
once('''            .all(|pair| pair[0].1 .0 < pair[1].1 .0);''',
     f'''            .all(|pair| if {on("R3_MUT_1733")} {{ pair[0].1 .0 <= pair[1].1 .0 }} else {{ pair[0].1 .0 < pair[1].1 .0 }});''')
open(path + ".tmp", "w", encoding="utf-8").write(s); os.replace(path + ".tmp", path)
print("patched mutations behind R3_MUT_1587/1731/1732/1733", path)
