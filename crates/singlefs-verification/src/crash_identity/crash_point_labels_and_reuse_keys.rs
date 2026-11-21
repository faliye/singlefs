//! 崩溃点的标签、复用键、一步的两样属性与覆盖报告：身份只作索引，复用键只由属性算出。

use super::*;

// ============================== 标签与复用键 ==============================

/// 崩溃点标签：设这个点的文件的仓内路径（登记时冻结）、崩溃点名称。只能交给 [`CoverageReport`]，不能格式化、不能比较。
pub struct NodeLabel {
    code_file_in_repository: String,
    crash_point_name: String,
}

/// 标签不合规矩：仓内路径是绝对路径、带 `.` 或 `..` 段、为空；崩溃点名称为空。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeLabelRejected {
    pub reason: String,
}

impl NodeLabel {
    /// # Errors
    /// 仓内路径是绝对路径、带 `..` 段或为空；崩溃点名称为空。身份要能跨机器，路径只收仓内相对路径。
    pub fn new(
        code_file_in_repository: &str,
        crash_point_name: &str,
    ) -> Result<Self, NodeLabelRejected> {
        let rejected = |reason: &str| {
            Err(NodeLabelRejected {
                reason: reason.to_string(),
            })
        };
        if code_file_in_repository.is_empty() || code_file_in_repository.starts_with('/') {
            return rejected("代码文件名要写仓内相对路径");
        }
        if code_file_in_repository
            .split('/')
            .any(|part| part == ".." || part == ".")
        {
            return rejected("仓内路径里不许有 . 或 .. 段");
        }
        if crash_point_name.is_empty() {
            return rejected("崩溃点名称不许为空");
        }
        Ok(Self {
            code_file_in_repository: code_file_in_repository.to_string(),
            crash_point_name: crash_point_name.to_string(),
        })
    }

    /// 身份串：标签两段加复用键的十六进制。只在本 crate 里拼（写进判定存储的节点名、覆盖报告），标签本身不外露文字。
    pub(crate) fn identity_text(&self, reuse_key: &ReuseKey) -> String {
        format!(
            "{}::{}::{}",
            self.code_file_in_repository,
            self.crash_point_name,
            hexadecimal_text(&reuse_key.0)
        )
    }
}

/// 复用键：只能从属性算出（[`StepAttributes::reuse_key`]）。这一步的输入没变就是同一个键；判法变不变另按判法单元指纹在判定行上分。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReuseKey([u8; DIGEST_BYTES]);

impl ReuseKey {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.0
    }
}

/// 一步的两样属性（各是 SHA-256）：这一步写下的字节（带屏障的写表）与它起步的镜像。只有流程进复用键：
/// 判法是判定行的属性（`verdict_store::JudgingUnitsFingerprint`），不在这里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepAttributes {
    pub write_table_with_barriers: [u8; DIGEST_BYTES],
    pub parent_output: [u8; DIGEST_BYTES],
}

impl StepAttributes {
    /// 两样按固定次序、各带名字拼起来取 SHA-256。
    #[must_use]
    pub fn reuse_key(&self) -> ReuseKey {
        let mut message = Vec::new();
        for (name, digest) in [
            ("write_table_with_barriers", &self.write_table_with_barriers),
            ("parent_output", &self.parent_output),
        ] {
            message.extend_from_slice(name.as_bytes());
            message.push(b'=');
            message.extend_from_slice(digest);
            message.push(b'\n');
        }
        ReuseKey(sha256_digest(&message))
    }
}

/// 一个崩溃点在这一趟里的去向：核了、复用了库里的判定、按测试代码里的 ignore 跳过了（分叉控制「预设的 ignore 就是剪枝」）、
/// 它的段按展开上限一个都没展开（没有状态可核，也是一种剪枝）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverageState {
    Checked,
    Reused,
    Ignored,
    NotExpanded,
}

impl CoverageState {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Checked => "checked",
            Self::Reused => "reused",
            Self::Ignored => "ignored",
            Self::NotExpanded => "not_expanded",
        }
    }
}

/// 覆盖报告：只写。收下路径、标签与它这一趟是复用还是重跑，交回逐行 `<路径> <身份> <去向>`（给人看、给门禁数）。
#[derive(Default)]
pub struct CoverageReport {
    lines: Vec<String>,
}

impl CoverageReport {
    pub fn record(
        &mut self,
        path: &str,
        label: NodeLabel,
        reuse_key: &ReuseKey,
        state: CoverageState,
    ) {
        self.lines.push(format!(
            "{path} {} {}",
            label.identity_text(reuse_key),
            state.name()
        ));
    }

    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_label_takes_only_relative_paths_inside_the_repository() {
        assert!(
            NodeLabel::new("/home/x/crates/a.rs", "t").is_err(),
            "绝对路径不收：身份要能跨机器"
        );
        assert!(NodeLabel::new("crates/../a.rs", "t").is_err());
        let label = NodeLabel::new("crates/singlefs-verification/tests/a.rs", "m::t")
            .expect("仓内相对路径");
        let key = StepAttributes {
            write_table_with_barriers: [1; 32],
            parent_output: [2; 32],
        }
        .reuse_key();
        let text = label.identity_text(&key);
        assert!(
            text.starts_with("crates/singlefs-verification/tests/a.rs::m::t::"),
            "身份三段、不带步号：{text}"
        );
        assert_eq!(
            text.len(),
            "crates/singlefs-verification/tests/a.rs::m::t::".len() + 64
        );
    }

    #[test]
    fn each_attribute_changes_the_reuse_key() {
        let base = StepAttributes {
            write_table_with_barriers: [1; 32],
            parent_output: [2; 32],
        };
        let key = base.reuse_key();
        assert_ne!(
            StepAttributes {
                write_table_with_barriers: [9; 32],
                parent_output: base.parent_output,
            }
            .reuse_key(),
            key
        );
        assert_ne!(
            StepAttributes {
                write_table_with_barriers: base.write_table_with_barriers,
                parent_output: [9; 32],
            }
            .reuse_key(),
            key
        );
        assert_eq!(base.reuse_key(), key, "同样的属性同一个键");
    }
}
