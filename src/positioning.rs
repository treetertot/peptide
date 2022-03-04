use std::ops::Range;
use smallvec::SmallVec;

#[derive(Debug, Clone, PartialEq)]
pub struct Slot {
    pub x: Range<f32>,
    pub y: Range<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RealPosition {
    pub layer: i16,
    /// [start x. start y. end x, end y]
    pub rect: [f32; 4],
}

#[derive(Debug, Clone)]
/// A structure for passing positions to multiple children
pub struct ChildPositions(pub SmallVec<[RealPosition; 2]>);

pub enum Propagation {
    Continue,
    Stop,
}

pub trait Squishy {
    fn slotify(&self) -> Slot;
    fn split(
        &self,
        _buffer: &mut Vec<Range<f32>>,
        _target: &RealPosition,
        _children: &mut ChildPositions,
    ) -> Propagation {
        Propagation::Stop
    }
}

pub fn accum_seq<I: Iterator<Item = Range<f32>>>(iter: I) -> Range<f32> {
    iter.fold(0.0..0.0, |old, new| {
        (old.start + new.start)..(old.end + new.end)
    })
}

pub fn accum_par<I: Iterator<Item = Range<f32>>>(mut iter: I) -> Range<f32> {
    let start = match iter.next() {
        Some(r) => r,
        None => return 0.0..0.0,
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
