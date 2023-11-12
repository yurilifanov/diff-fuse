use crate::error::MergeError;
use crate::hunk::Hunk;
use crate::macros::merge_err;

pub enum FlaggedHunk<'a> {
    Left(&'a Hunk),
    Right(&'a Hunk),
}

impl FlaggedHunk<'_> {
    pub fn hunk(&self) -> &Hunk {
        match self {
            Self::Left(hunk) => hunk,
            Self::Right(hunk) => hunk,
        }
    }
}

pub trait HunkAdapter<T> {
    fn merge(&self, other: &T) -> Result<Hunk, MergeError>;
    fn overlaps(&self, other: &T) -> bool;
}

impl<'a> HunkAdapter<FlaggedHunk<'a>> for FlaggedHunk<'a> {
    fn merge(&self, other: &FlaggedHunk<'a>) -> Result<Hunk, MergeError> {
        // merging with the same flag shouldn't happen, because they
        // shouldn't overlap (as checked during parsing)
        match [self, other] {
            [Self::Left(lh), Self::Right(rh)] => lh.merge(rh),
            [Self::Right(rh), Self::Left(lh)] => lh.merge(rh),
            _ => Err(merge_err!("Tried merging hunks with the same flag")),
        }
    }

    fn overlaps(&self, other: &FlaggedHunk<'a>) -> bool {
        self.hunk().overlaps(other.hunk())
    }
}

impl HunkAdapter<Hunk> for FlaggedHunk<'_> {
    fn merge(&self, other: &Hunk) -> Result<Hunk, MergeError> {
        match self {
            Self::Left(hunk) => hunk.merge(other),
            Self::Right(hunk) => other.merge(hunk),
        }
    }

    fn overlaps(&self, other: &Hunk) -> bool {
        self.hunk().overlaps(other)
    }
}

impl std::fmt::Display for FlaggedHunk<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left(hunk) => write!(f, "{}", hunk),
            Self::Right(hunk) => write!(f, "{}", hunk),
        }
    }
}
