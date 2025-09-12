use crate::buffer_list::{Buffer, BufferId, BufferList};
use crate::pattern::{Pattern, Target};

use nvim_router::nvim_rs::Value;

use std::cmp::Ordering;
use std::iter::Rev;
use std::ops::ControlFlow;
use std::ops::Range;

type VecIntoIter<T> = <Vec<T> as IntoIterator>::IntoIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Score(u16);

impl Score {
    fn with_penalty(penalty: usize) -> Self {
        let penalty = penalty.try_into().unwrap_or(u16::MAX);
        Score(u16::MAX - penalty)
    }
}

#[derive(Debug)]
pub(super) struct Item<'a> {
    pub(super) buf_id: BufferId,
    pub(super) content: &'a Target,
    score: Score,
    pub(super) metadata: Value,
    pub(super) matched: Match,
}

impl PartialEq for Item<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.buf_id == other.buf_id
    }
}
impl Eq for Item<'_> {}

impl PartialOrd for Item<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then_with(|| other.content.len().cmp(&self.content.len()))
            .then_with(|| other.content.cmp(self.content))
            .then_with(|| self.buf_id.cmp(&other.buf_id))
    }
}

impl<'a> Item<'a> {
    fn from(buf: &'a Buffer, score: Score, matched: Match) -> Self {
        Self {
            buf_id: buf.id.clone(),
            content: &buf.file,
            score,
            metadata: buf.metadata.clone(),
            matched,
        }
    }
}

#[derive(Debug)]
pub(super) enum Match {
    Sub(Range<usize>),
    Fuzzy(Vec<Range<usize>>),
    None,
}

impl IntoIterator for Match {
    type Item = Range<usize>;
    type IntoIter = MatchIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        use std::iter::{empty, once};
        match self {
            Self::Sub(v) => MatchIntoIter::Sub(once(v)),
            Self::Fuzzy(v) => MatchIntoIter::Fuzzy(v.into_iter()),
            Self::None => MatchIntoIter::None(empty()),
        }
    }
}

pub(super) enum MatchIntoIter {
    Sub(std::iter::Once<Range<usize>>),
    Fuzzy(VecIntoIter<Range<usize>>),
    None(std::iter::Empty<Range<usize>>),
}

impl Iterator for MatchIntoIter {
    type Item = Range<usize>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self {
            Self::Sub(it) => it.len(),
            Self::Fuzzy(it) => it.len(),
            Self::None(it) => it.len(),
        };
        (len, Some(len))
    }

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Sub(it) => it.next(),
            Self::Fuzzy(it) => it.next(),
            Self::None(it) => it.next(),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct RankedItems<'a> {
    end_with: Vec<Item<'a>>,
    substring: Vec<Item<'a>>,
    fuzzy: Vec<Item<'a>>,
    nonmatch: Vec<Item<'a>>,
}

impl RankedItems<'_> {
    fn sort(&mut self) {
        self.end_with.sort_unstable();
        self.substring.sort_unstable();
        self.fuzzy.sort_unstable();
    }
}

impl<'a> IntoIterator for RankedItems<'a> {
    type Item = Item<'a>;
    type IntoIter = RankingIntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RankingIntoIter {
            end_with: self.end_with.into_iter().rev(),
            substring: self.substring.into_iter().rev(),
            fuzzy: self.fuzzy.into_iter().rev(),
            nonmatch: self.nonmatch.into_iter(),
        }
    }
}

pub(super) struct RankingIntoIter<'a> {
    end_with: Rev<VecIntoIter<Item<'a>>>,
    substring: Rev<VecIntoIter<Item<'a>>>,
    fuzzy: Rev<VecIntoIter<Item<'a>>>,
    nonmatch: VecIntoIter<Item<'a>>,
}

impl<'a> Iterator for RankingIntoIter<'a> {
    type Item = Item<'a>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len =
            self.end_with.len() + self.substring.len() + self.fuzzy.len() + self.nonmatch.len();
        (len, Some(len))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.end_with.next() {
            return Some(item);
        }
        if let Some(item) = self.substring.next() {
            return Some(item);
        }
        if let Some(item) = self.fuzzy.next() {
            return Some(item);
        }
        self.nonmatch.next()
    }
}

pub(super) fn rank<'a>(buffers: &'a BufferList, input: Pattern) -> RankedItems<'a> {
    if input.is_empty() {
        let mut ranking = RankedItems {
            nonmatch: buffers
                .into_iter()
                .map(|target| Item::from(target, Score(0), Match::None))
                .collect(),
            ..Default::default()
        };

        ranking.sort();
        return ranking;
    }

    let mut ranking = RankedItems::default();

    't: for target in buffers {
        let mut tester = input.test(&target.file);

        match tester.next() {
            Some(ControlFlow::Break(item)) => {
                if item.roffset == 0 {
                    ranking
                        .end_with
                        .push(Item::from(target, Score(0), Match::Sub(item.range)));
                } else {
                    ranking.substring.push(Item::from(
                        target,
                        Score::with_penalty(item.roffset),
                        Match::Sub(item.range),
                    ));
                }
            }
            Some(ControlFlow::Continue(item)) => {
                let mut matched = vec![item.range];

                for item in tester {
                    match item {
                        ControlFlow::Break(item) => {
                            let penalty = item.roffset + (item.range.end - item.range.start);
                            let score = Score::with_penalty(penalty);

                            matched.push(item.range);

                            ranking
                                .fuzzy
                                .push(Item::from(target, score, Match::Fuzzy(matched)));
                            continue 't;
                        }
                        ControlFlow::Continue(item) => {
                            matched.push(item.range);
                        }
                    }
                }
                ranking
                    .nonmatch
                    .push(Item::from(target, Score(0), Match::None));
            }
            None => {
                ranking
                    .nonmatch
                    .push(Item::from(target, Score(0), Match::None));
            }
        }
    }

    ranking.sort();

    ranking
}
