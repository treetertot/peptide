use std::ops::Range;

use crate::drawing::{ChildPositions, Propagation, RealPosition};
use crate::slot::{accum_par, accum_seq, Slot};

#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub slots: Vec<Slot>,
    pub offset: [f32; 2],
}
impl Squishy for Grid {
    fn slotify(&self) -> Slot {
        let x = accum_par(self.slots.chunks(self.width).map(|chunk| {
            accum_seq(
                chunk
                    .iter()
                    .map(|slot| slide_range(slot.x.clone(), self.offset[0] * 2.0)),
            )
        }));
        let y = accum_seq(self.slots.chunks(self.width).map(|chunk| {
            accum_par(
                chunk
                    .iter()
                    .map(|slot| slide_range(slot.y.clone(), self.offset[1] * 2.0)),
            )
        }));
        Slot { x, y }
    }
    fn split(
        &self,
        buffer: &mut Vec<Range<f32>>,
        target: &RealPosition,
        children: &mut ChildPositions,
    ) -> Propagation {
        // here be dragons
        if children.0.len() < self.slots.len() {
            let diff = self.slots.len() - children.0.len();
            children.0.extend((0..diff).map(|_| RealPosition {
                layer: target.layer + 1,
                rect: [0.; 4],
            }));
        }
        buffer.clear();
        let num_rows = self.slots.len() / self.width;
        let width = target.rect[2] - target.rect[0];
        let column_widths = (0..self.width)
            .map(|n| {
                self.slots[n..]
                    .iter()
                    .step_by(self.width)
                    .map(|s| s.x.clone())
            })
            .map(accum_par);
        buffer.extend(column_widths);
        let total_width = accum_seq(buffer.iter().map(Clone::clone));
        let scale_factor = reverse_lerp(width, total_width);
        let output_columns = (0..self.width).map(|n| {
            (0..num_rows)
                .map(move |m| m * self.width + n)
        });
        let writing = buffer
            .iter()
            .map(|r| {
                let inv_scale = 1. - scale_factor;
                inv_scale * r.start + scale_factor * r.end
            })
            .zip(output_columns);

        let mut start_x = -self.offset[0];
        for (width, out_column) in writing {
            start_x += self.offset[0] * 2.0;
            let end_x = start_x + width;
            for idx in out_column {
                let rect = &mut children.0[idx].rect;
                rect[0] = start_x;
                rect[2] = end_x;
            }
            start_x = end_x;
        }

        buffer.clear();
        let height = target.rect[3] - target.rect[1];
        let row_heights = self.slots.chunks(self.width)
            .map(IntoIterator::into_iter)
            .map(|a|
                a.map(|s| s.y.clone())
            )
            .map(accum_par);
        buffer.extend(row_heights);
        let total_width = accum_seq(buffer.iter().map(Clone::clone));
        let scale_factor = reverse_lerp(height, total_width);
        let output_rows = children.0.chunks_mut(self.width);
        // heights are being calculated slightly wrongs
        let writing = buffer
            .iter()
            .map(|r| {
                let inv_scale = 1. - scale_factor;
                inv_scale * r.start + scale_factor * r.end
            })
            .zip(output_rows);
        start_x = -self.offset[1];
        for (height, out_row) in writing {
            start_x += self.offset[1];
            let end_x = start_x + height;
            for row in out_row {
                let rect = &mut row.rect;
                rect[1] = start_x;
                rect[3] = end_x;
            }
            start_x = end_x;
        }

        Propagation::Continue
    }
}

fn slide_range(range: Range<f32>, slide: f32) -> Range<f32> {
    range.start + slide..range.end + slide
}

fn reverse_lerp(x: f32, range: Range<f32>) -> f32 {
    let diff = f32::EPSILON.max(range.end - range.start);
    (x - range.start) / diff
}

#[test]
fn grid_slotting() {
    use smallvec::SmallVec;
    let grid = Grid {
        width: 2,
        slots: vec![
            Slot {
                x: 2.0..4.0,
                y: 4.0..5.0,
            },
            Slot {
                x: 2.5..5.0,
                y: 5.0..6.0,
            },
            Slot {
                x: 2.0..4.0,
                y: 4.0..5.0,
            },
            Slot {
                x: 2.5..5.0,
                y: 5.0..6.0,
            },
        ],
        offset: [0.0; 2],
    };
    assert_eq!(
        grid.slotify(),
        Slot {
            x: 4.5..9.0,
            y: 10.0..10.0
        }
    );
    let mut buffer = Vec::new();
    let position = RealPosition {
        layer: -1,
        rect: [0.0, 0.0, 9.0, 10.0]
    };
    let mut children = ChildPositions(SmallVec::new());
    grid.split(&mut buffer, &position, &mut children);
    assert_eq!(children.0[0].rect, [0.0, 0.0, 4.0, 5.0]);
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
