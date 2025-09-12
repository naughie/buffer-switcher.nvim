use crate::pattern::Target;

use nvim_router::nvim_rs::Value;

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct BufferId(Value);

impl From<BufferId> for Value {
    fn from(value: BufferId) -> Self {
        value.0
    }
}

impl Eq for BufferId {}

impl PartialOrd for BufferId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for BufferId {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(lhs) = self.0.as_i64()
            && let Some(rhs) = other.0.as_i64()
        {
            lhs.cmp(&rhs)
        } else if let Some(lhs) = self.0.as_f64()
            && let Some(rhs) = other.0.as_f64()
            && let Some(ord) = lhs.partial_cmp(&rhs)
        {
            ord
        } else if let Some(lhs) = self.0.as_str()
            && let Some(rhs) = other.0.as_str()
        {
            lhs.cmp(rhs)
        } else {
            Ordering::Less
        }
    }
}

impl BufferId {
    pub(super) fn from_id(v: &Value) -> Self {
        Self(v.clone())
    }
}

#[derive(Debug)]
pub(super) struct Buffer {
    pub(super) id: BufferId,
    pub(super) file: Target,
    pub(super) metadata: Value,
}

#[derive(Debug, Default)]
pub(super) struct BufferList(Vec<Buffer>);

impl<'a> IntoIterator for &'a BufferList {
    type Item = &'a Buffer;
    type IntoIter = std::slice::Iter<'a, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<Buffer> for BufferList {
    fn from_iter<T: IntoIterator<Item = Buffer>>(iter: T) -> Self {
        Self(<_ as FromIterator<_>>::from_iter(iter))
    }
}
