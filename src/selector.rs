use crate::dir::{Dir, Weights};

pub struct DirSelector;

impl DirSelector {
    /// Matching dirs, best frecency first.
    pub fn find_best<'a>(dirs: &'a [Dir], query: &[String], now: u64, weights: &Weights) -> Vec<&'a Dir> {
        let words: Vec<String> = query.iter().map(|w| w.to_lowercase()).collect();
        let mut found: Vec<&Dir> = dirs
            .iter()
            .filter(|d| matches(&d.path.to_string_lossy().to_lowercase(), &words))
            .collect();
        found.sort_by(|a, b| b.frecency(now, weights).total_cmp(&a.frecency(now, weights)));
        found
    }
}

/// Every word appears in order; the last one inside the last path segment.
fn matches(path: &str, words: &[String]) -> bool {
    let Some((last, others)) = words.split_last() else {
        return true;
    };
    let mut pos = 0;
    for word in others {
        match path[pos..].find(word.as_str()) {
            Some(i) => pos += i + word.len(),
            None => return false,
        }
    }
    let last_segment = path.rfind('/').map_or(0, |i| i + 1);
    path[pos.max(last_segment)..].contains(last.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(path: &str, count: f64) -> Dir {
        Dir { path: path.into(), count, last_access: 0 }
    }

    fn query(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    fn paths(found: Vec<&Dir>) -> Vec<String> {
        found.iter().map(|d| d.path.display().to_string()).collect()
    }

    #[test]
    fn matches_substring_case_insensitively() {
        let dirs = vec![dir("/home/hane/Projets", 1.0), dir("/tmp", 1.0)];
        assert_eq!(paths(DirSelector::find_best(&dirs, &query(&["proj"]), 0, &Weights::default())), ["/home/hane/Projets"]);
    }

    #[test]
    fn sorts_by_frecency() {
        let now = 1_000_000;
        let dirs = vec![
            Dir { path: "/a/proj1".into(), count: 10.0, last_access: 0 },
            Dir { path: "/a/proj2".into(), count: 3.0, last_access: now },
            Dir { path: "/a/proj3".into(), count: 1.0, last_access: 0 },
        ];
        assert_eq!(
            paths(DirSelector::find_best(&dirs, &query(&["proj"]), now, &Weights::default())),
            ["/a/proj2", "/a/proj1", "/a/proj3"]
        );
    }

    #[test]
    fn words_must_match_in_order() {
        let dirs = vec![dir("/work/api/src", 1.0), dir("/src/api/work", 1.0)];
        assert_eq!(paths(DirSelector::find_best(&dirs, &query(&["work", "src"]), 0, &Weights::default())), ["/work/api/src"]);
    }

    #[test]
    fn last_word_must_match_last_segment() {
        let dirs = vec![dir("/home/proj/docs", 5.0), dir("/home/proj", 1.0)];
        assert_eq!(paths(DirSelector::find_best(&dirs, &query(&["proj"]), 0, &Weights::default())), ["/home/proj"]);
    }

    #[test]
    fn empty_query_matches_everything() {
        let dirs = vec![dir("/a", 1.0), dir("/b", 2.0)];
        assert_eq!(paths(DirSelector::find_best(&dirs, &[], 0, &Weights::default())), ["/b", "/a"]);
    }
}
