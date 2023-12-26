use crate::*;

pub type SpanIndex = u32;

#[derive(Copy, Clone, Hash, Eq)]
pub struct Span
{
    pub file_handle: util::FileServerHandle,
    location: (SpanIndex, SpanIndex),
}

impl Span
{
    pub fn new(file_handle: util::FileServerHandle, start: SpanIndex, end: SpanIndex) -> Span
    {
        Span {
            file_handle,
            location: (start, end),
        }
    }

    pub fn new_dummy() -> Span
    {
        Span {
            file_handle: 0,
            location: (SpanIndex::MAX, SpanIndex::MAX),
        }
    }

    pub fn location(&self) -> Option<(SpanIndex, SpanIndex)>
    {
        if self.location.0 == SpanIndex::MAX
        {
            return None;
        }

        Some(self.location)
    }

    pub fn before(&self) -> Span
    {
        if self.location.0 == SpanIndex::MAX
        {
            *self
        }
        else
        {
            let start = self.location.0;

            Span {
                file_handle: self.file_handle,
                location: (start, start),
            }
        }
    }

    pub fn after(&self) -> Span
    {
        if self.location.0 == SpanIndex::MAX
        {
            *self
        }
        else
        {
            let end = self.location.1;

            Span {
                file_handle: self.file_handle,
                location: (end, end),
            }
        }
    }

    pub fn join(&self, other: Span) -> Span
    {
        match (self.location, other.location)
        {
            (_, (SpanIndex::MAX, _)) => *self,
            ((SpanIndex::MAX, _), _) => other,
            (self_loc, other_loc) =>
            {
                assert!(
                    self.file_handle == other.file_handle,
                    "joining spans from different files"
                );

                let start = std::cmp::min(self_loc.0, other_loc.0);

                let end = std::cmp::max(self_loc.1, other_loc.1);

                Span {
                    file_handle: self.file_handle,
                    location: (start, end),
                }
            }
        }
    }
}

impl PartialEq for Span
{
    fn eq(&self, other: &diagn::Span) -> bool
    {
        self.file_handle == other.file_handle && self.location == other.location
    }
}

impl std::fmt::Debug for Span
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("Span(")?;
        write!(f, "file#{:?}", &self.file_handle)?;

        if self.location.0 != SpanIndex::MAX
        {
            f.write_str("[")?;
            <SpanIndex as std::fmt::Debug>::fmt(&self.location.0, f)?;
            f.write_str("..")?;
            <SpanIndex as std::fmt::Debug>::fmt(&self.location.1, f)?;
            f.write_str("]")?;
        }

        f.write_str(")")?;
        Ok(())
    }
}
