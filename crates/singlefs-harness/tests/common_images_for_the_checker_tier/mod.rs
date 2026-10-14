//! harness 档集成测试真跑实现搭出来的镜像，池级 checker 那一遍交给 checker 档两段判（用户 2026-10-14 弹窗定「搬进 checker 档两段」：
//! harness 用例只留实现的行为与理想模型、RefFS 的比对；`.claude/rules/verification.md`「定义与名字」：harness 档的依赖闭包里没有 checker 档）。
//!
//! 做法（样板是 `instance_table_page_full.rs`）：
//! - 一条 harness 用例里搭场景、核实现行为的那一截写成 `common_images_for_the_checker_tier/<测试文件名>.rs` 里与用例同名的公开函数
//!   （场景函数），收一个 [`PoolCheckerImages`]；原先就地跑池级 checker 的那一处换成 [`PoolCheckerImages::hand_out`]：
//!   交出那一刻的镜像（交一个造镜像的闭包，harness 档不造）连同池级 checker 该判成什么（[`PoolCheckerExpectation`]，原先那几条断言逐条写进去）。
//! - harness 档的测试文件 `mod` 这一份、经 `#[path]` 引那一份场景文件，用例拿 [`PoolCheckerImages::left_to_the_checker_tier`] 调场景函数：
//!   镜像不造、池级 checker 不跑，只核实现的行为。
//! - checker 档的同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）经 `#[path]`
//!   引同一份场景文件与它用到的 `tests/common*/mod.rs`，拿 [`PoolCheckerImages::collected_for_the_checker_tier`] 调同一个场景函数，
//!   重建同一批镜像；每张镜像建成一张给定状态表（基线是这张镜像、写表为空、一个空的持久集合）交崩溃点重放的给定状态引擎：
//!   先写进判定库、判段从库里读回来判，配了 GPU 只由 GPU 判器判，再逐张核 [`PoolCheckerExpectation`]。
//!
//! 这一份只依赖 `singlefs_harness` 的内存池，不引池级 checker 与 checker 档：两档各 `mod` 一次。
#![allow(
    dead_code,
    reason = "harness 档的用例只用交镜像那一半，checker 档的用例只用收镜像那一半"
)]

use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};

/// 交给 checker 档的一张镜像上池级 checker 该判成什么：原先就地跑池级 checker 之后的那几条断言逐条写成下面几样，checker 档逐样核。
/// 不变量用 `singlefs_checker::image::IMPLEMENTED_INVARIANTS` 里的编号（如 `"I-3.1"`）。
///
/// 判到位与违例细节只在这一趟现判时有：checker 档那一侧每条用例起跑先清空自己那一份判定库，结论都是这一趟现判的；
/// 违例细节只有 CPU 判器给（GPU 判器只交位），[`PoolCheckerExpectation::violation_detail_contains`] 那几样在 GPU 判的结论上只核违例位。
#[derive(Clone, Debug)]
pub struct PoolCheckerExpectation {
    /// `Some`：判红的恰好是这几条（与次序无关；空就是一条都不红）。`None`：判红的有哪几条不限，只核下面几样。
    pub exactly_violated: Option<Vec<&'static str>>,
    /// 这几条判红（别的不管）。
    pub violated: Vec<&'static str>,
    /// 这几条不红（别的不管）。
    pub not_violated: Vec<&'static str>,
    /// 这几条判成立：判到了且不红。
    pub holds: Vec<&'static str>,
    /// 这几条没判到（不适用或部分没判）：判到位不置、也不红。
    pub not_judged: Vec<&'static str>,
    /// 这几条判红、违例细节里有这一段字（CPU 判器给细节；GPU 判的结论上只核它红）。
    pub violation_detail_contains: Vec<(&'static str, String)>,
    /// 这几条红了的话，违例细节要满足这一判（说明它判的是什么）；不红不管（CPU 判器给细节；GPU 判的结论上不核）。
    pub violation_detail_if_violated_satisfies: Vec<ViolationDetailCondition>,
    /// 同上，判法带着场景这一刻算出的一个数（例如记账比遍历多出的字节数要恰好是它）。
    pub violation_detail_if_violated_satisfies_given_a_value:
        Vec<ViolationDetailConditionGivenAValue>,
    /// 与同一个场景里先交的别的镜像比（「改坏之后别的判决逐条不变」这一类）；判到位要现判，与 `holds` 一样。
    pub compared_with_an_earlier_image: Vec<ComparedWithAnEarlierImage>,
}

/// 这一张与同一个场景里先交的、叫 `label` 的那一张比。
#[derive(Clone, Debug)]
pub struct ComparedWithAnEarlierImage {
    pub label: String,
    pub comparison: ComparisonWithAnEarlierImage,
}

/// 与先交的那一张怎么比。
#[derive(Clone, Debug)]
pub enum ComparisonWithAnEarlierImage {
    /// 除了这几条，每一条的判到与违例都与那一张逐条相同。
    JudgedAlikeExcept(Vec<&'static str>),
    /// 那一张上判到的不变量里，这一张上没判到的至多这么多条；这一张上至少判到一条（不许整片没判）。
    AtMostThisManyJudgedThereAreNotJudgedHere(usize),
    /// 判红的不变量与那一张逐条相同，两张都有违例细节（CPU 判器给）时细节也逐字相同。
    ViolatedAlike,
}

/// 一条不变量红了时它的违例细节要满足的一判。
#[derive(Clone, Debug)]
pub struct ViolationDetailCondition {
    pub invariant: &'static str,
    /// 这一判说的是什么（对不上时打出来）。
    pub described_as: &'static str,
    pub holds_for: fn(&str) -> bool,
}

/// 一条不变量红了时它的违例细节要满足的一判，判法带着场景这一刻算出的一个数。
#[derive(Clone, Debug)]
pub struct ViolationDetailConditionGivenAValue {
    pub invariant: &'static str,
    /// 这一判说的是什么（对不上时连同那个数打出来）。
    pub described_as: &'static str,
    pub value: u64,
    pub holds_for: fn(&str, u64) -> bool,
}

impl PoolCheckerExpectation {
    /// 池级 checker 一条都不红。
    #[must_use]
    pub fn no_invariant_violated() -> Self {
        Self::exactly_these_violated(&[])
    }

    /// 判红的恰好是这几条。
    #[must_use]
    pub fn exactly_these_violated(invariants: &[&'static str]) -> Self {
        Self {
            exactly_violated: Some(invariants.to_vec()),
            violated: Vec::new(),
            not_violated: Vec::new(),
            holds: Vec::new(),
            not_judged: Vec::new(),
            violation_detail_contains: Vec::new(),
            violation_detail_if_violated_satisfies: Vec::new(),
            violation_detail_if_violated_satisfies_given_a_value: Vec::new(),
            compared_with_an_earlier_image: Vec::new(),
        }
    }

    /// 判红的有哪几条不限：只核之后加上的那几样。
    #[must_use]
    pub fn only_the_named_invariants() -> Self {
        Self {
            exactly_violated: None,
            violated: Vec::new(),
            not_violated: Vec::new(),
            holds: Vec::new(),
            not_judged: Vec::new(),
            violation_detail_contains: Vec::new(),
            violation_detail_if_violated_satisfies: Vec::new(),
            violation_detail_if_violated_satisfies_given_a_value: Vec::new(),
            compared_with_an_earlier_image: Vec::new(),
        }
    }

    #[must_use]
    pub fn and_violated(mut self, invariant: &'static str) -> Self {
        self.violated.push(invariant);
        self
    }

    #[must_use]
    pub fn and_not_violated(mut self, invariant: &'static str) -> Self {
        self.not_violated.push(invariant);
        self
    }

    #[must_use]
    pub fn and_none_of_these_violated(mut self, invariants: &[&'static str]) -> Self {
        self.not_violated.extend_from_slice(invariants);
        self
    }

    #[must_use]
    pub fn and_holds(mut self, invariant: &'static str) -> Self {
        self.holds.push(invariant);
        self
    }

    #[must_use]
    pub fn and_not_judged(mut self, invariant: &'static str) -> Self {
        self.not_judged.push(invariant);
        self
    }

    #[must_use]
    pub fn and_if_violated_the_detail_satisfies(
        mut self,
        invariant: &'static str,
        described_as: &'static str,
        holds_for: fn(&str) -> bool,
    ) -> Self {
        self.violation_detail_if_violated_satisfies
            .push(ViolationDetailCondition {
                invariant,
                described_as,
                holds_for,
            });
        self
    }

    /// 同 [`PoolCheckerExpectation::and_if_violated_the_detail_satisfies`]，判法另带场景这一刻算出的一个数 `value`。
    #[must_use]
    pub fn and_if_violated_the_detail_satisfies_given_a_value(
        mut self,
        invariant: &'static str,
        described_as: &'static str,
        value: u64,
        holds_for: fn(&str, u64) -> bool,
    ) -> Self {
        self.violation_detail_if_violated_satisfies_given_a_value
            .push(ViolationDetailConditionGivenAValue {
                invariant,
                described_as,
                value,
                holds_for,
            });
        self
    }

    /// 除了 `except` 那几条，每一条的判到与违例都与同一个场景里先交的 `label` 那张镜像逐条相同。
    #[must_use]
    pub fn and_judged_like_the_image(mut self, label: &str, except: &[&'static str]) -> Self {
        self.compared_with_an_earlier_image
            .push(ComparedWithAnEarlierImage {
                label: label.to_string(),
                comparison: ComparisonWithAnEarlierImage::JudgedAlikeExcept(except.to_vec()),
            });
        self
    }

    /// 判红的不变量与同一个场景里先交的 `label` 那张镜像逐条相同（两张都有违例细节时细节也逐字相同）。
    #[must_use]
    pub fn and_violated_like_the_image(mut self, label: &str) -> Self {
        self.compared_with_an_earlier_image
            .push(ComparedWithAnEarlierImage {
                label: label.to_string(),
                comparison: ComparisonWithAnEarlierImage::ViolatedAlike,
            });
        self
    }

    /// 同一个场景里先交的 `label` 那张镜像上判到的不变量里，这一张上没判到的至多 `at_most` 条；这一张至少判到一条。
    #[must_use]
    pub fn and_at_most_this_many_judged_on_the_image_not_judged_here(
        mut self,
        label: &str,
        at_most: usize,
    ) -> Self {
        self.compared_with_an_earlier_image
            .push(ComparedWithAnEarlierImage {
                label: label.to_string(),
                comparison: ComparisonWithAnEarlierImage::AtMostThisManyJudgedThereAreNotJudgedHere(
                    at_most,
                ),
            });
        self
    }

    #[must_use]
    pub fn and_violated_with_a_detail_containing(
        mut self,
        invariant: &'static str,
        text: &str,
    ) -> Self {
        self.violation_detail_contains
            .push((invariant, text.to_string()));
        self
    }
}

/// 交出去的一张镜像，或同一个基线上的一批崩溃态（基线叠上写表里持久了的那几个写；每个状态各一张镜像）。
#[derive(Clone, Debug)]
pub struct PoolCheckerImage {
    /// 这张镜像（这批崩溃态）在这个场景里的名字（进给定状态表的表名；一个场景里各张不同）。
    pub label: String,
    /// 交的是一张镜像时就是它；交的是一批崩溃态时是它们的基线。
    pub image: MemoryPool,
    /// 基线之后的写表（一张镜像时为空）与它的切段。
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    /// 每个状态一个持久集合，与 `writes` 逐条对应（一张镜像时是一个空的）。
    pub persisted_sets: Vec<Vec<bool>>,
    /// 每个状态上池级 checker 都该判成这样。
    pub expected: PoolCheckerExpectation,
    /// GPU 判器许建不出这张镜像的判表（盘上没有一份自证过的系统配置这一类），写明为什么；`None` 就是 GPU 判不了照判红。
    pub the_gpu_may_refuse_its_tables_because: Option<&'static str>,
}

/// 场景函数交镜像的去处。
#[derive(Debug)]
pub enum PoolCheckerImages {
    /// harness 档：镜像不造、不收，池级 checker 不跑，只记交过的名字（一个场景里重名照样判红：checker 档那一侧收镜像时同一条断言，
    /// 在 harness 档随时跑到，不等提交时 checker 档那一趟才发现）。
    LeftToTheCheckerTier { labels_handed_out: Vec<String> },
    /// checker 档：每次交的都造出来收下，场景跑完一起建表交引擎判。
    CollectedForTheCheckerTier(Vec<PoolCheckerImage>),
}

impl PoolCheckerImages {
    #[must_use]
    pub fn left_to_the_checker_tier() -> Self {
        Self::LeftToTheCheckerTier {
            labels_handed_out: Vec::new(),
        }
    }

    #[must_use]
    pub fn collected_for_the_checker_tier() -> Self {
        Self::CollectedForTheCheckerTier(Vec::new())
    }

    /// harness 档的用例跑一个场景（镜像不造、不收），场景跑完至少交过一张镜像：与 checker 档那一侧「一张都没交就判红」同一条，
    /// harness 档随时跑到。只在某一条路上交镜像的场景不用它，直接拿 [`PoolCheckerImages::left_to_the_checker_tier`] 调。
    pub fn run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenario: impl FnOnce(&mut PoolCheckerImages),
    ) {
        let mut images = Self::left_to_the_checker_tier();
        scenario(&mut images);
        assert!(
            images.handed_out() > 0,
            "场景一张镜像都没交给池级 checker：checker 档那一侧同名用例会判红"
        );
    }

    /// 交一张镜像：`build_the_image` 只在 checker 档那一侧调（harness 档不为池级 checker 造镜像）。
    pub fn hand_out(
        &mut self,
        label: &str,
        build_the_image: impl FnOnce() -> MemoryPool,
        expected: PoolCheckerExpectation,
    ) {
        self.hand_out_allowing_the_gpu_to_refuse_its_tables(label, build_the_image, expected, None);
    }

    /// 同 [`PoolCheckerImages::hand_out`]，`because` 给了就是 GPU 判器许建不出这一张的判表（写明为什么）。
    pub fn hand_out_allowing_the_gpu_to_refuse_its_tables(
        &mut self,
        label: &str,
        build_the_image: impl FnOnce() -> MemoryPool,
        expected: PoolCheckerExpectation,
        because: Option<&'static str>,
    ) {
        match self {
            Self::LeftToTheCheckerTier { labels_handed_out } => {
                record_a_label_left_to_the_checker_tier(labels_handed_out, label);
            }
            Self::CollectedForTheCheckerTier(images) => {
                assert!(
                    images.iter().all(|image| image.label != label),
                    "一个场景里交出去的镜像各有各的名字：{label} 交了两次"
                );
                images.push(PoolCheckerImage {
                    label: label.to_string(),
                    image: build_the_image(),
                    writes: Vec::new(),
                    segments: Vec::new(),
                    persisted_sets: vec![Vec::new()],
                    expected,
                    the_gpu_may_refuse_its_tables_because: because,
                });
            }
        }
    }

    /// 交一批崩溃态：`build_the_crash_states` 交回基线、写表、切段与每个状态的持久集合（只在 checker 档那一侧调），每个状态上
    /// 池级 checker 都该判成 `expected`。checker 档把它们建成一张给定状态表。
    pub fn hand_out_crash_states(
        &mut self,
        label: &str,
        build_the_crash_states: impl FnOnce() -> (
            MemoryPool,
            Vec<RetainedWrite>,
            Vec<Vec<usize>>,
            Vec<Vec<bool>>,
        ),
        expected: PoolCheckerExpectation,
    ) {
        match self {
            Self::LeftToTheCheckerTier { labels_handed_out } => {
                record_a_label_left_to_the_checker_tier(labels_handed_out, label);
            }
            Self::CollectedForTheCheckerTier(images) => {
                assert!(
                    images.iter().all(|image| image.label != label),
                    "一个场景里交出去的镜像各有各的名字：{label} 交了两次"
                );
                let (base, writes, segments, persisted_sets) = build_the_crash_states();
                assert!(
                    persisted_sets
                        .iter()
                        .all(|persisted| persisted.len() == writes.len()),
                    "{label}：每个持久集合与写表逐条对应"
                );
                images.push(PoolCheckerImage {
                    label: label.to_string(),
                    image: base,
                    writes,
                    segments,
                    persisted_sets,
                    expected,
                    the_gpu_may_refuse_its_tables_because: None,
                });
            }
        }
    }

    /// 交了几张。
    #[must_use]
    pub fn handed_out(&self) -> usize {
        match self {
            Self::LeftToTheCheckerTier { labels_handed_out } => labels_handed_out.len(),
            Self::CollectedForTheCheckerTier(images) => images.len(),
        }
    }

    /// checker 档那一侧收下的镜像，按交的次序。
    ///
    /// # Panics
    /// 这是 harness 档那一种（不收镜像）。
    #[must_use]
    pub fn into_collected(self) -> Vec<PoolCheckerImage> {
        match self {
            Self::CollectedForTheCheckerTier(images) => images,
            Self::LeftToTheCheckerTier { .. } => {
                panic!("harness 档那一种不收镜像：checker 档拿 collected_for_the_checker_tier 调场景函数")
            }
        }
    }
}

/// harness 档那一种记下交过的一个名字：与 checker 档收镜像时一样，一个场景里重名判红。
fn record_a_label_left_to_the_checker_tier(labels_handed_out: &mut Vec<String>, label: &str) {
    assert!(
        labels_handed_out
            .iter()
            .all(|handed_out| handed_out != label),
        "一个场景里交出去的镜像各有各的名字：{label} 交了两次"
    );
    labels_handed_out.push(label.to_string());
}
