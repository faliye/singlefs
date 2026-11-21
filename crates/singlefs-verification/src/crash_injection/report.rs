//! 崩溃注入的报告：一批历史判完收成的计数、发现与留给别的机器的给定状态表。

use super::*;
use std::fmt::Write as _;

/// 报告里注明「有给定状态表交的是占位结论」那一行的开头（[`CrashInjectionReport::render`]）。
pub const GIVEN_STATES_TABLES_LEFT_TO_OTHER_HOSTS_NOTE: &str = "两台分片留给别的机器";

/// 一批种子跑下来的崩溃注入报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashInjectionReport {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub draw: CrashPointDraw,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub tally: CrashInjectionTally,
    pub known_red_hits: Vec<KnownRedAtACrashPoint>,
    /// 按种子从小到大排；同一截同一个签名只留第一个种子。
    pub new_findings: Vec<CrashPointFinding>,
    /// 这一批判红的崩溃镜像写在哪（这一次调用自己新建的那一份目录，跑完删掉的那一档也照写）；一个都没落盘时是 None。
    pub image_directory: Option<PathBuf>,
    pub image_retention: FailureImageRetention,
    /// 这一批照哪一组断言判（交「重放这一整批」那一行写出来；跑的时候不读它）。
    pub assertions: CrashInjectionAssertions,
    /// 给定状态表的判器叫停了（[`GivenStatesJudge::stop_is_requested`]），没领的那几片里有几段历史：0 是整批都跑了；
    /// 不为 0 时计数与新发现只是领了的那几片的，调用方照「本次未跑」交回、不照它判。
    pub histories_left_by_the_stop_request: u64,
}

impl CrashInjectionReport {
    /// 第一行就是这次用的种子基与规模：种子基是这个测试周期写死的那一个（[`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`]），
    /// 报告与失败信息里不带它就重放不了，下一个周期重抽之后这一批数也就换了。
    /// 判红的崩溃镜像落在哪写路径（给人看）；比对两份报告用 [`CrashInjectionReport::render_the_conclusions`]。
    #[must_use]
    pub fn render(&self) -> String {
        self.render_given(ImagePathsInTheText::Shown)
    }

    /// 同 [`CrashInjectionReport::render`]，判红的崩溃镜像落在哪不写路径、只写落没落盘与留没留：镜像目录每次调用各不相同
    /// （[`create_the_image_directory_of_a_campaign_call`]），同一批种子跑两遍（线程数不同、判器不同、两台分片与整份）比的是这一份。
    #[must_use]
    pub fn render_the_conclusions(&self) -> String {
        self.render_given(ImagePathsInTheText::LeftOut)
    }

    /// 两份报告除了判红的崩溃镜像落在哪的路径之外逐样相同：这一批的镜像目录与每个新发现的镜像都只比落没落盘、不比路径
    /// （镜像目录每次调用各不相同，[`create_the_image_directory_of_a_campaign_call`]），别的字段与 `==` 一样逐个比（已知红比全部，不只比文字里那前几个）。
    /// 同一批种子跑两遍、各自落了镜像（GPU 判器那一趟与 CPU 判器那一趟）比的是这一份；结构体的 `==` 连路径一起比，两趟各落了镜像、路径不同就判不等。
    #[must_use]
    pub fn has_the_same_conclusions_as(&self, other: &Self) -> bool {
        let Self {
            first_seed,
            seed_count,
            operations_per_history,
            draw,
            weights,
            execution,
            tally,
            known_red_hits,
            new_findings,
            image_directory,
            image_retention,
            assertions,
            histories_left_by_the_stop_request,
        } = self;
        *first_seed == other.first_seed
            && *seed_count == other.seed_count
            && *operations_per_history == other.operations_per_history
            && *draw == other.draw
            && *weights == other.weights
            && *execution == other.execution
            && *tally == other.tally
            && *known_red_hits == other.known_red_hits
            && new_findings.len() == other.new_findings.len()
            && new_findings
                .iter()
                .zip(&other.new_findings)
                .all(|(finding, other_finding)| finding.has_the_same_conclusions_as(other_finding))
            && image_directory.is_some() == other.image_directory.is_some()
            && *image_retention == other.image_retention
            && *assertions == other.assertions
            && *histories_left_by_the_stop_request == other.histories_left_by_the_stop_request
    }

    fn render_given(&self, image_paths: ImagePathsInTheText) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "种子基 {}（这个测试周期写死的；同一个构建加同一个种子基重放得出来），种子 [{}, {})，每段 {} 步，崩溃状态怎么摆 {:?}，比重：{}；{}",
            self.first_seed,
            self.first_seed,
            self.first_seed + self.seed_count,
            self.operations_per_history,
            self.draw,
            self.weights.name,
            self.execution.name()
        );
        match (&self.image_directory, self.image_retention, image_paths) {
            (
                Some(directory),
                FailureImageRetention::KeepTheImageFiles,
                ImagePathsInTheText::Shown,
            ) => {
                let _ = writeln!(text, "判红的崩溃镜像留在 {}", directory.display());
            }
            (
                Some(directory),
                FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
                ImagePathsInTheText::Shown,
            ) => {
                let _ = writeln!(
                    text,
                    "判红的崩溃镜像曾写在 {}，跑完已删",
                    directory.display()
                );
            }
            (Some(_), FailureImageRetention::KeepTheImageFiles, ImagePathsInTheText::LeftOut) => {
                let _ = writeln!(
                    text,
                    "判红的崩溃镜像留在这一次调用自己的镜像目录里（路径见 render）"
                );
            }
            (
                Some(_),
                FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
                ImagePathsInTheText::LeftOut,
            ) => {
                let _ = writeln!(
                    text,
                    "判红的崩溃镜像曾写在这一次调用自己的镜像目录里，跑完已删"
                );
            }
            (
                None,
                FailureImageRetention::KeepTheImageFiles
                | FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
                ImagePathsInTheText::Shown | ImagePathsInTheText::LeftOut,
            ) => {}
        }
        if self.histories_left_by_the_stop_request > 0 {
            let _ = writeln!(
                text,
                "叫停了：{} 段历史没领（下面的计数与新发现只是领了的那几片的，这一批不作数）",
                self.histories_left_by_the_stop_request
            );
        }
        if self.tally.given_states_tables_recorded_only > 0 {
            let _ = writeln!(
                text,
                "只跑了录入段：{} 张给定状态表写进库、没判，交的是占位结论（下面的计数与新发现不作数，判段那一趟从库里读回判）",
                self.tally.given_states_tables_recorded_only
            );
        }
        if self.tally.given_states_tables_left_to_other_hosts > 0 {
            let _ = writeln!(
                text,
                "{GIVEN_STATES_TABLES_LEFT_TO_OTHER_HOSTS_NOTE}：{} 张给定状态表归别的机器判、交的是占位结论（下面的计数偏少、新发现可能漏，这一批不作数；那一台判完、写进服务之后再跑一趟才作数）",
                self.tally.given_states_tables_left_to_other_hosts
            );
        }
        text.push_str(&self.tally.render());
        for (form_index, form) in KNOWN_RED_FORMS.iter().enumerate() {
            let hits: Vec<(u64, usize, String)> = self
                .known_red_hits
                .iter()
                .filter(|hit| hit.form == form_index)
                .map(|hit| (hit.seed.0, hit.crash_point.segment_index, hit.stage.name()))
                .collect();
            let _ = writeln!(
                text,
                "崩溃状态上的已知红第 {form_index} 条（{}）：{} 个崩溃状态；前几个 (种子, 段号, 哪一截) {:?}",
                form.closeout_table_row,
                hits.len(),
                hits.iter().take(8).collect::<Vec<_>>()
            );
        }
        for finding in &self.new_findings {
            text.push_str(&finding.render_given(image_paths));
            let _ = writeln!(
                text,
                "  复现这一段：{}",
                crash_injection_replay_line(
                    finding.seed.0,
                    1,
                    &self.history_sampling(),
                    self.draw,
                    CrashInjectionAssertions::NewFindingsOnly
                )
            );
        }
        let _ = writeln!(text, "重放这一整批：{}", self.replay_line());
        text
    }

    /// 这一批的取样参数（每段步数、比重、跑法）。
    #[must_use]
    pub fn history_sampling(&self) -> HistorySampling {
        HistorySampling {
            operations_per_history: self.operations_per_history,
            weights: self.weights,
            execution: self.execution,
        }
    }

    /// 重放这一整批的那一行命令（[`crash_injection_replay_line`]）：带着这一批照哪一组断言判，
    /// 照抄跑崩溃注入那条入口，红在结构或覆盖断言上的也红在同一条上。
    #[must_use]
    pub fn replay_line(&self) -> String {
        crash_injection_replay_line(
            self.first_seed,
            self.seed_count,
            &self.history_sampling(),
            self.draw,
            self.assertions,
        )
    }
}
