use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct Slot {
    pub x: Range<f32>,
    pub y: Range<f32>
}

pub fn accum_par<I: Iterator<Item = Range<f32>>>(mut iter: I) -> Range<f32> {
    let start = match iter.next() {
        Some(r) => r,
        None => return 0.0..0.0
    };
    let r = iter.fold(start, |old, new| {
        let min = old.start.max(new.start);
        let max = old.end.min(new.end);
        min..max
    });
    correct_range(r)
}

#[inline(always)]
pub fn correct_range(range: Range<f32>) -> Range<f32> {
    if range.start > range.end {
        return range.end..range.start;
    }
    range
}

pub fn max_default_b(a: f32, b: f32) -> f32 {
    match a > b {
        true => a,
        false => b
    }
}

pub fn accum_seq<I: Iterator<Item = Range<f32>>>(iter: I) -> Range<f32> {
    iter.fold(0.0..0.0, |old, new| {
        (old.start + new.start)..(old.end + new.end)
    })
}

pub trait Squishy {
    fn slotify(&self) -> Slot;
}
