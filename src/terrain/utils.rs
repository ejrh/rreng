use std::ops::Range;

#[derive(Clone, Debug, Default)]
pub struct Range2(pub Range<usize>, pub Range<usize>);

impl Range2 {
    pub(crate) fn overlaps(&self, other: &Range2) -> bool {
        self.0.start < other.0.end && self.0.end > other.0.start
            && self.1.start < other.1.end && self.1.end > other.1.start
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty() || self.1.is_empty()
    }

    pub(crate) fn expand_to(&mut self, row: usize, col: usize) {
        if self.is_empty() {
            self.0 = row..row + 1;
            self.1 = col..col + 1;
        } else {
            self.0.start = self.0.start.min(row);
            self.0.end = self.0.end.max(row + 1);
            self.1.start = self.1.start.min(col);
            self.1.end = self.1.end.max(col + 1);
        }
    }
}

pub fn restrict_ranges(from_r: &mut Range<isize>, to_r: &mut Range<isize>, limit: isize) {
    if to_r.start < 0 {
        let excess = -to_r.start;
        from_r.start += excess;
        to_r.start += excess;
    }
    if to_r.end > limit {
        let excess = to_r.end - limit;
        from_r.end -= excess;
        to_r.end -= excess;
    }
    #[allow(unstable_name_collisions)]
    if from_r.is_empty() || to_r.is_empty() {
        *from_r = 0..0;
        *to_r = 0..0;
    }
}

pub fn get_copyable_range(src_width: usize, offset: isize, dest_width: usize) -> (Range<usize>, Range<usize>) {
    let mut from_r = 0..src_width as isize;
    let mut to_r = offset..offset+src_width as isize;
    restrict_ranges(&mut from_r, &mut to_r, dest_width as isize);
    let from_r = from_r.start as usize..from_r.end as usize;
    let to_r = to_r.start as usize..to_r.end as usize;
    (from_r, to_r)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_copyable_range() {
        let (mut from_r, mut to_r) = (0..10, 0..10);
        restrict_ranges(&mut from_r, &mut to_r, 10);
        assert_eq!((0..10, 0..10), (from_r, to_r));

        let (mut from_r, mut to_r) = (0..10, -5..5);
        restrict_ranges(&mut from_r, &mut to_r, 10);
        assert_eq!((5..10, 0..5), (from_r, to_r));

        let (mut from_r, mut to_r) = (0..10, 5..15);
        restrict_ranges(&mut from_r, &mut to_r, 10);
        assert_eq!((0..5, 5..10), (from_r, to_r));
    }
}
