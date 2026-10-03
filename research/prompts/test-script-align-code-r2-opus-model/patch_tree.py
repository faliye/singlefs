"""在仓副本的 crash_amplification.rs 里加一个只给原型看的口子：每个点的判器输入原文（digest_for_a_point 取摘要之前的那段文字）。
不改判法、不改摘要：digest_for_a_point 照旧对同一段文字取 SHA-256。用法：python3 patch_tree.py <仓副本根>"""
import sys, os
path = f"{sys.argv[1]}/crates/singlefs-checker-tier/src/crash_amplification.rs"
s = open(path, encoding="utf-8").read()
def once(old, new):
    global s
    assert s.count(old) == 1, old
    s = s.replace(old, new, 1)
once('''    fn digest_for_a_point(
        &self,
        flow: &CrashFlow<'_>,
        first_write: usize,
        last_write: usize,
    ) -> [u8; DIGEST_BYTES] {
        let mut text = self.base_devices_text.clone();''', '''    fn digest_for_a_point(
        &self,
        flow: &CrashFlow<'_>,
        first_write: usize,
        last_write: usize,
    ) -> [u8; DIGEST_BYTES] {
        sha256_digest(self.text_for_a_point(flow, first_write, last_write).as_bytes())
    }

    fn text_for_a_point(
        &self,
        flow: &CrashFlow<'_>,
        first_write: usize,
        last_write: usize,
    ) -> String {
        let mut text = self.base_devices_text.clone();''')
once('''            WritesAfterThePoint::MayAllBeRead { suffix_digests } => text.push_str(&format!(
                "writes_after_the_point={}\\n",
                hexadecimal_text(&suffix_digests[last_write])
            )),
        }
        sha256_digest(text.as_bytes())
    }
}''', '''            WritesAfterThePoint::MayAllBeRead { suffix_digests } => text.push_str(&format!(
                "writes_after_the_point={}\\n",
                hexadecimal_text(&suffix_digests[last_write])
            )),
        }
        text
    }
}

/// 原型口子（攻方腿加的，不在冻结副本里）：每个点的判器输入原文与它管的写区间，次序同 `plan_crash_points`。
#[must_use]
pub fn judge_input_texts_of_each_point(flow: &CrashFlow<'_>) -> Vec<(usize, usize, String)> {
    let owner_of_segment = owner_of_each_segment(flow);
    let inputs = JudgeInputsOfTheFlow::of(flow);
    flow.crash_points
        .iter()
        .enumerate()
        .map(|(point_index, point)| {
            let owned_writes: BTreeSet<usize> = (0..flow.segments.len())
                .filter(|segment| owner_of_segment[*segment] == point_index)
                .flat_map(|segment| flow.segments[segment].iter().copied())
                .collect();
            let first_write = owned_writes.first().copied().unwrap_or(point.writes.start);
            let last_write = owned_writes.last().map_or(first_write, |last| last + 1);
            (first_write, last_write, inputs.text_for_a_point(flow, first_write, last_write))
        })
        .collect()
}''')
open(path + ".tmp", "w", encoding="utf-8").write(s); os.replace(path + ".tmp", path)
print("patched", path)
