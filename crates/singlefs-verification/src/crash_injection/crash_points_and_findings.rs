//! 崩溃点与发现：一个崩溃点（哪一段、持久了哪几个写）、它判到哪一截、红在哪一截时记下的发现与已知的红。

use super::*;
use std::fmt::Write as _;

/// 一段历史上摆几个崩溃状态、怎么挑。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashPointDraw {
    /// 按种子抽这么多个（每个是「一段 + 那一段的一个真子集」，同一段同一个子集只摆一次）。
    Sampled { crash_points_per_history: usize },
    /// 写数不超过 `segment_writes` 的段整段枚举（全部真子集），更长的段按种子各抽 `sampled_in_longer_segments` 个：
    /// 写死的那几段历史靠它把小段摆全，判别力不靠运气。
    EveryProperSubsetOfShortSegments {
        segment_writes: usize,
        sampled_in_longer_segments: usize,
    },
    /// 就摆这一个崩溃状态（靶向用例用，代码三方 m2-supp3-item3-code-第 1 题选「靶向用例 + 变异行」）：
    /// 从第 `segment_index` 段的第一个写起数 `writes_from_the_segment_on` 个写，其中下标（从这一段的第一个写数起）在
    /// `withheld_among_them` 里的扣下、别的持久；更早的段整段持久，这几个写之后的一个都没持久。
    ///
    /// 这几个写可以越过这一段的末尾，越过的那几道屏障当设备没守：记录核对器两条判据防的正是屏障没守住才有的状态
    /// （记录与根之间那道没守：根在而记录不在；单元与根之间的没守：恢复自称的那一版之前的发布缺单元），
    /// 今天的持久顺序下它们在段内真子集里一个都摆不出来，判据接没接上只有这样摆得出来看（K3-④⑨）。
    ///
    /// 只给写死的一段历史经 [`inject_crashes_into_history`] 用，不进战役：写死的段号只对写死的那一段历史成立，重放行也写不出它
    /// （[`run_crash_injection_campaign`] 拿到它就 panic）。段号不是候选段、这几个写越过最后一个跑完的操作（或写表末尾）、
    /// `writes_from_the_segment_on` 是 0、扣下的下标越界或一个都没有时 panic——写死的输入写错了，不悄悄摆别的状态。
    ExactlyOneCrashState {
        segment_index: usize,
        writes_from_the_segment_on: usize,
        withheld_among_them: &'static [usize],
    },
}

/// 靶向用例那一支（[`CrashPointDraw::ExactlyOneCrashState`]）拿去跑战役或写重放行时的 panic 消息。
pub(super) const TARGETED_CRASH_STATE_IS_NOT_A_CAMPAIGN_DRAW: &str =
    "靶向用例那一个崩溃状态不进战役：写死的段号只对写死的那一段历史成立，重放行写不出它（只经 inject_crashes_into_history 摆）";

/// 失败时那份崩溃镜像留不留。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureImageRetention {
    /// 留着，路径打进报告（设了变量跑的那一批：现场要能重看）。
    KeepTheImageFiles,
    /// 整批跑完就把这一批的镜像目录删掉（门禁档不在临时目录里留垃圾）。
    DeleteTheImageFilesWhenTheRunFinishes,
}

/// 一个崩溃状态：截在录制流的第几段、那一段里哪几个写持久了。
/// 更早的段整段持久、更晚的段一个都没持久；这一段里持久的是一个真子集（全持久等于「没崩在这一段」，那个状态由下一段的空子集摆）。
/// 靶向用例（[`CrashPointDraw::ExactlyOneCrashState`]）摆的那一个例外：持久集合从这一段的第一个写起，可以越过这一段的末尾。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrashPoint {
    pub segment_index: usize,
    /// 这一段里第 k 个写持久了没有（长度就是这一段的写数；靶向用例摆的那一个，长度是它从这一段的第一个写起数的写数）。
    pub persisted_within_the_segment: Vec<bool>,
    /// 这一段里第一个没持久的写落在历史的哪一步里。
    pub step: StepPosition,
    /// 那一步是哪一类操作；落在起点里的是 None。
    pub operation_kind: Option<HistoryOperationKind>,
}

impl CrashPoint {
    /// 段内持久的那几个写不是这一段的前缀：有一个写被扣下了，它后面却有写持久了
    /// （「后发的写先持久」那一类状态，只截前缀摆不出来）。
    #[must_use]
    pub fn withholds_a_write_before_a_persisted_one(&self) -> bool {
        self.persisted_within_the_segment
            .iter()
            .position(|persisted| !persisted)
            .is_some_and(|withheld| {
                self.persisted_within_the_segment[withheld..]
                    .iter()
                    .any(|persisted| *persisted)
            })
    }

    /// 给人看：第几段、段内持久了几个写、扣下的写在段内的下标。
    #[must_use]
    pub fn render(&self) -> String {
        let withheld: Vec<usize> = self
            .persisted_within_the_segment
            .iter()
            .enumerate()
            .filter(|(_, persisted)| !**persisted)
            .map(|(index, _)| index)
            .collect();
        format!(
            "第 {} 段（{} 个写，持久 {} 个，扣下段内第 {:?} 个）",
            self.segment_index,
            self.persisted_within_the_segment.len(),
            self.persisted_within_the_segment
                .iter()
                .filter(|persisted| **persisted)
                .count(),
            withheld
        )
    }
}

/// 第一次崩溃镜像上挂载透明比的是哪个动作之后的池（`singlefs_verification::mount_transparency::MountTransparencyAction` 的前两种）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MountTransparencyComparison {
    /// 只挂载（取号、写行、暖机与挂载推的抬 F），不接发布：v2a。
    TheMountAlone,
    /// 挂载加一次发布，比发布之前那一版。
    TheMountAndOnePublish,
}

impl MountTransparencyComparison {
    pub const ALL: [MountTransparencyComparison; 2] = [
        MountTransparencyComparison::TheMountAlone,
        MountTransparencyComparison::TheMountAndOnePublish,
    ];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            MountTransparencyComparison::TheMountAlone => "the_mount_alone",
            MountTransparencyComparison::TheMountAndOnePublish => "the_mount_and_one_publish",
        }
    }
}

/// 一个崩溃状态上的判定落在哪一截（代码审阅第 2 条「崩溃注入层补可写挂载」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashStateStage {
    /// 崩溃镜像本身：只读恢复（看 journal）、问模型、池级 checker、记录核对器。
    CrashImage,
    /// 在那份崩溃后镜像上可写挂载（取号、写行、暖机）再发一次布之后的池：池级 checker、记录核对器（崩溃态镜像是那份崩溃镜像，
    /// 实现恢复后的镜像是这个池）；挂载或那次发布被拒（单元区墙之外）也记在这一截。
    AfterTheWritableMountAndOnePublish,
    /// 可写挂载途中再崩一次（二次崩溃），崩在挂载的这一段里：只读恢复、问模型、池级 checker、记录核对器（历史与挂载写出的发布都核）。
    SecondCrashInsideTheWritableMount(WritableMountPhase),
    /// 二次崩溃镜像上再起一次可写挂载之后的池：读出的内容要与没崩过的那一次挂载之后读出的相同（I-8.4 的 R-换）。
    RemountAfterASecondCrashInsideTheWritableMount(WritableMountPhase),
    /// 挂载透明：崩溃镜像上做了这个动作之后的池，只读恢复与崩溃镜像上的只读恢复逐字段比。二次崩溃镜像上的那一判记在
    /// [`CrashStateStage::SecondCrashInsideTheWritableMount`] 那一截的违例里。
    MountTransparencyAfterTheCrash(MountTransparencyComparison),
}

impl CrashStateStage {
    /// 报告与镜像目录里的名字。
    #[must_use]
    pub fn name(self) -> String {
        match self {
            CrashStateStage::CrashImage => "crash_image".to_string(),
            CrashStateStage::AfterTheWritableMountAndOnePublish => {
                "after_the_writable_mount_and_one_publish".to_string()
            }
            CrashStateStage::SecondCrashInsideTheWritableMount(phase) => {
                format!("second_crash_inside_{}", phase.name())
            }
            CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(phase) => {
                format!("remount_after_a_second_crash_inside_{}", phase.name())
            }
            CrashStateStage::MountTransparencyAfterTheCrash(comparison) => {
                format!("mount_transparency_of_{}", comparison.name())
            }
        }
    }
}

/// 一段历史上一类崩溃状态失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointFinding {
    pub signature: FailureSignature,
    pub seed: HistorySeed,
    pub crash_point: CrashPoint,
    /// 判红的是哪一截（崩溃镜像本身、之后的可写挂载与一次发布、挂载途中的二次崩溃）。
    pub stage: CrashStateStage,
    pub observation: FailureObservation,
    /// 恢复读回了什么（给人看）。
    pub read_back: String,
    /// 这份崩溃镜像落成文件之后放在哪；没落盘（不留现场，或写文件没成）时是 None。
    pub image_files: Option<PathBuf>,
}

/// 报告与新发现写成文字时，崩溃镜像落在哪的路径写不写进去。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ImagePathsInTheText {
    /// 写路径（给人看、回头找现场）。
    Shown,
    /// 不写路径：镜像目录每次调用各不相同（[`create_the_image_directory_of_a_campaign_call`]），两份报告比对时路径不进比对。
    LeftOut,
}

impl CrashPointFinding {
    #[must_use]
    pub fn render(&self) -> String {
        self.render_given(ImagePathsInTheText::Shown)
    }

    /// 同 [`CrashPointFinding::render`]，崩溃镜像落了盘只写一句、不写路径（两份报告比对时用这一份）。
    #[must_use]
    pub fn render_the_conclusions(&self) -> String {
        self.render_given(ImagePathsInTheText::LeftOut)
    }

    /// 两个新发现除了崩溃镜像落在哪的路径之外逐样相同：镜像只比落没落盘（[`CrashInjectionReport::has_the_same_conclusions_as`] 逐个用它）。
    #[must_use]
    pub fn has_the_same_conclusions_as(&self, other: &Self) -> bool {
        let Self {
            signature,
            seed,
            crash_point,
            stage,
            observation,
            read_back,
            image_files,
        } = self;
        *signature == other.signature
            && *seed == other.seed
            && *crash_point == other.crash_point
            && *stage == other.stage
            && *observation == other.observation
            && *read_back == other.read_back
            && image_files.is_some() == other.image_files.is_some()
    }

    pub(super) fn render_given(&self, image_paths: ImagePathsInTheText) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "崩溃状态上的新发现 {:?}（{}）：种子 {}，{}，落在 {:?}／{:?}",
            self.signature,
            self.stage.name(),
            self.seed.0,
            self.crash_point.render(),
            self.crash_point.step,
            self.crash_point.operation_kind
        );
        let _ = writeln!(text, "  恢复读回：{}", self.read_back);
        for (invariant, detail) in &self.observation.violations {
            let _ = writeln!(text, "  {invariant}：{detail}");
        }
        for aspect in record_check_aspects(&self.observation.record_check) {
            let _ = writeln!(text, "  记录核对器判红：{aspect}");
        }
        if let Some(panic) = &self.observation.panic {
            let _ = writeln!(text, "  panic 在 {}：{}", panic.location, panic.message);
        }
        if let Some(disagreement) = &self.observation.model_disagreement {
            let _ = writeln!(
                text,
                "  模型对不上（{}）：模型答 {}；实现 {}",
                disagreement.aspect.name(),
                disagreement.model_answer,
                disagreement.implementation_answer
            );
        }
        match (&self.image_files, image_paths) {
            (Some(directory), ImagePathsInTheText::Shown) => {
                let _ = writeln!(text, "  崩溃镜像留在：{}", directory.display());
            }
            (Some(_), ImagePathsInTheText::LeftOut) => {
                let _ = writeln!(text, "  崩溃镜像落了盘（路径见 render）");
            }
            (None, ImagePathsInTheText::Shown | ImagePathsInTheText::LeftOut) => {}
        }
        text
    }
}

/// RefFS 判的两截（第一截与二次崩溃那一截；挂载加一次发布之后那一截不判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StageTheReferenceFileSystemJudges {
    CrashImage,
    SecondCrashInsideTheWritableMount,
}

/// 崩溃状态上以「已知红」收尾的一次：种子、清单第几条、哪个崩溃状态、哪一截。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownRedAtACrashPoint {
    pub seed: HistorySeed,
    pub form: usize,
    pub crash_point: CrashPoint,
    pub stage: CrashStateStage,
}
