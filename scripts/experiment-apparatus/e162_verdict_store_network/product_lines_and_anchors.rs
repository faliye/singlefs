//! 产物行的缓冲与拼装、网络锚点行与锚点跑法。

use super::*;

/// 产物行：每行 `E7RESULT ` 起头，收尾 `name=done emitted=N`（N 连 done 行自己都算）。
pub(super) struct ProductLines {
    emitter: Emitter,
}

impl ProductLines {
    pub(super) fn new() -> Self {
        Self { emitter: Emitter::new() }
    }

    pub(super) fn line(&mut self, body: String) {
        let text = self.emitter.emit_raw(&body);
        let mut standard_output = std::io::stdout().lock();
        writeln!(standard_output, "{text}").expect("写得进标准输出");
        standard_output.flush().expect("刷得出标准输出");
    }

    pub(super) fn finish(&mut self) {
        let text = self.emitter.finish();
        println!("{text}");
    }
}

// ============================== 锚点 ==============================

/// 送块那个 bin 的锚点（A1–A6、A11 的帧长、B10、B10b），加上只在这个 bin 里有的 A11 后半（F 的块文件长）与 A10。
fn network_anchor_lines() -> Vec<(String, String, String, bool)> {
    let mut lines: Vec<(String, String, String, bool)> = anchor_checks()
        .into_iter()
        .map(|check| {
            let matches = check.matches();
            (check.identifier, check.computed, check.registered.to_string(), matches)
        })
        .collect();
    let file_length = (FILE_HEADER_BYTES + 65_536).to_string();
    let file_matches = file_length == "65584";
    lines.push(("A11_file".to_string(), file_length, "65584".to_string(), file_matches));
    let path_blocks = 2 * PATH_BLOCK_COUNT;
    let value_bytes = format!("{}/{}", path_blocks * PRIMARY_STATES_PER_BLOCK, path_blocks * SMALL_STATES_PER_BLOCK);
    let value_matches = value_bytes == "6553600000/102400000";
    lines.push(("A10".to_string(), value_bytes, "6553600000/102400000".to_string(), value_matches));
    lines
}

pub(super) fn run_anchors(output: &mut ProductLines) -> i32 {
    let lines = network_anchor_lines();
    let mismatches = lines.iter().filter(|(_, _, _, matches)| !matches).count();
    for (identifier, computed, _registered, matches) in &lines {
        output.line(format!("name=anchor id={identifier} value={computed} matches_registration={matches}"));
    }
    output.line(format!("name=verdict part=anchors anchors_checked={} anchor_mismatches={mismatches}", lines.len()));
    i32::from(mismatches > 0)
}

#[cfg(test)]
#[path = "product_lines_and_anchors_tests.rs"]
mod tests;
