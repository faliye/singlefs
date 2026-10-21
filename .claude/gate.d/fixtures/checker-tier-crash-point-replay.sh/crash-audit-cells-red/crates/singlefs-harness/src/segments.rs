/// 提交步骤的封闭枚举（样本：比 crash-audit-cells-green 多一个成员 TrimHint，没有一条流罩住它）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    UnitWrite,
    Barrier,
    TrimHint,
}

impl StepKind {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            StepKind::UnitWrite => "unit_write",
            StepKind::Barrier => "barrier",
            StepKind::TrimHint => "trim_hint",
        }
    }
}
