/// 提交步骤的封闭枚举（样本：两个成员，覆盖表的列）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    UnitWrite,
    Barrier,
}

impl StepKind {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            StepKind::UnitWrite => "unit_write",
            StepKind::Barrier => "barrier",
        }
    }
}
