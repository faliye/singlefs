//! 一份镜像上池级 checker 的判定，只摘这一轮关心的六条（I-9.5、I-3.4、I-4.7、I-4.6、I-4.4、I-9.6）与全部违例。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::memory_pool::MemoryPool;

pub const WATCHED: [&str; 6] = ["I-9.5", "I-3.4", "I-4.7", "I-4.6", "I-4.4", "I-9.6"];

#[derive(Clone, Debug, Default)]
pub struct Judged {
    /// 六条各自的判定种类：H 成立、N 不适用、P 部分没判、V 违例。
    pub watched: Vec<(&'static str, char, String)>,
    /// 全部违例的不变量与第一处说明。
    pub violated: Vec<(&'static str, String)>,
}

pub fn kind(verdict: &InvariantVerdict) -> char {
    match verdict {
        InvariantVerdict::Holds => 'H',
        InvariantVerdict::NotApplicable(_) => 'N',
        InvariantVerdict::PartiallyJudged(_) => 'P',
        InvariantVerdict::Violated(_) => 'V',
    }
}

pub fn detail(verdict: &InvariantVerdict) -> String {
    match verdict {
        InvariantVerdict::Holds => String::new(),
        InvariantVerdict::NotApplicable(reason) => reason.to_string(),
        InvariantVerdict::PartiallyJudged(unjudged) => unjudged.clone(),
        InvariantVerdict::Violated(detail) => detail.clone(),
    }
}

pub fn judge(image: &MemoryPool) -> Judged {
    let report = check_pool_image(image);
    let mut judged = Judged::default();
    for (invariant, verdict) in &report {
        if WATCHED.contains(invariant) {
            judged.watched.push((invariant, kind(verdict), detail(verdict)));
        }
        if let InvariantVerdict::Violated(text) = verdict {
            judged.violated.push((invariant, text.clone()));
        }
    }
    judged
}

impl Judged {
    pub fn watched_string(&self) -> String {
        self.watched
            .iter()
            .map(|(invariant, kind, _)| format!("{invariant}={kind}"))
            .collect::<Vec<_>>()
            .join(",")
    }
    pub fn violated_names(&self) -> Vec<&'static str> {
        self.violated.iter().map(|(invariant, _)| *invariant).collect()
    }
    pub fn verdict_of(&self, invariant: &str) -> char {
        self.watched.iter().find(|(name, _, _)| *name == invariant).map_or('?', |(_, kind, _)| *kind)
    }
    pub fn detail_of(&self, invariant: &str) -> String {
        self.watched.iter().find(|(name, _, _)| *name == invariant).map_or(String::new(), |(_, _, text)| text.clone())
    }
}
