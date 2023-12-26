use crate::*;

#[derive(Debug)]
pub struct OverlapChecker
{
    entries: Vec<OverlapCheckerEntry>,
}

#[derive(Debug)]
struct OverlapCheckerEntry
{
    pub position: usize,
    pub size: usize,
    pub span: diagn::Span,
}

impl OverlapChecker
{
    pub fn new() -> OverlapChecker
    {
        OverlapChecker {
            entries: Vec::new(),
        }
    }

    pub fn check_and_insert(
        &mut self,
        report: &mut diagn::Report,
        span: diagn::Span,
        position: usize,
        size: usize,
    ) -> Result<(), ()>
    {
        let (index, maybe_overlapping_entry) = self.check_overlap(position, size);

        if let Some(overlapping_entry) = maybe_overlapping_entry
        {
            report.push_parent("output overlap", span);

            report.note_span("overlaps with:", overlapping_entry.span);

            report.pop_parent();

            return Err(());
        }

        let new_entry = OverlapCheckerEntry {
            position,
            size,
            span,
        };

        self.entries.insert(index, new_entry);

        Ok(())
    }

    fn check_overlap(&self, position: usize, size: usize) -> (usize, Option<&OverlapCheckerEntry>)
    {
        let index = self.entries.binary_search_by(|e| e.position.cmp(&position));

        match index
        {
            Ok(i) =>
            {
                if self.entries[i].size > 0 && size > 0
                {
                    return (i + 1, Some(&self.entries[i]));
                }

                (i + 1, None)
            }

            Err(i) =>
            {
                if i < self.entries.len()
                {
                    let next = &self.entries[i];
                    if position + size > next.position
                    {
                        return (i, Some(next));
                    }
                }

                if i > 0 && i - 1 < self.entries.len()
                {
                    let prev = &self.entries[i - 1];
                    if prev.position + prev.size > position
                    {
                        return (i - 1, Some(prev));
                    }
                }

                (i, None)
            }
        }
    }
}
