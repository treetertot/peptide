use std::ops::Range;

use crate::positioning::{
    accum_par, accum_seq, ChildPositions, Propagation, RealPosition, Slot, Squishy, reverse_lerp
};

#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
}
impl Squishy for Grid {
    fn slotify(&self, slots: &[Slot], offset: [f32; 2]) -> Slot {
        let x = accum_par(slots.chunks(self.width).map(|chunk| {
            accum_seq(
                chunk
                    .iter()
                    .map(|slot| slide_range(slot.x.clone(), offset[0] * 2.0)),
            )
        }));
        let y = accum_seq(slots.chunks(self.width).map(|chunk| {
            accum_par(
                chunk
                    .iter()
                    .map(|slot| slide_range(slot.y.clone(), offset[1] * 2.0)),
            )
        }));
        Slot { x, y }
    }
    fn split(
        &self,
        slots: &[Slot],
        offset: [f32; 2],
        target: &RealPosition,
        children: &mut ChildPositions,
    ) -> Propagation {
        // here be dragons

        // run this expression outside of the function now that the components are separated
        if children.0.len() < slots.len() {
            let diff = slots.len() - children.0.len();
            children.0.extend((0..diff).map(|_| RealPosition {
                layer: target.layer + 1,
                rect: [0.; 4],
            }));
        }

        let num_rows = slots.len() / self.width;
        let width = target.rect[2] - target.rect[0];
        // There may be a way to rework the iterators to avoid the buffer
        // In that case buffer should be removed from the trait
        let column_widths = (0..self.width)
            .map(|n| slots[n..].iter().step_by(self.width).map(|s| s.x.clone()))
            .map(accum_par);
        let total_width = accum_seq(column_widths.clone());
        let scale_factor = reverse_lerp(width, total_width);
        let output_columns =
            (0..self.width).map(|n| (0..num_rows).map(move |m| m * self.width + n));
        let writing = column_widths
            .map(|r| {
                let inv_scale = 1. - scale_factor;
                inv_scale * r.start + scale_factor * r.end
            })
            .zip(output_columns);

        let mut start_x = -offset[0];
        for (width, out_column) in writing {
            start_x += offset[0] * 2.0;
            let end_x = start_x + width;
            for idx in out_column {
                let rect = &mut children.0[idx].rect;
                rect[0] = start_x;
                rect[2] = end_x;
            }
            start_x = end_x;
        }

        let height = target.rect[3] - target.rect[1];
        let row_heights = slots
            .chunks(self.width)
            .map(IntoIterator::into_iter)
            .map(|a| a.map(|s| s.y.clone()))
            .map(accum_par);
        let total_width = accum_seq(row_heights.clone());
        let scale_factor = reverse_lerp(height, total_width);
        let output_rows = children.0.chunks_mut(self.width);
        // heights are being calculated slightly wrongs
        let writing = row_heights
            .map(|r| {
                let inv_scale = 1. - scale_factor;
                inv_scale * r.start + scale_factor * r.end
            })
            .zip(output_rows);
        start_x = -offset[1];
        for (height, out_row) in writing {
            start_x += offset[1];
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

#[test]
fn grid_slotting() {
    use smallvec::SmallVec;
    let grid = Grid { width: 2 };
    let slots = vec![
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
    ];
    let offset = [0.0; 2];
    assert_eq!(
        grid.slotify(&slots, offset),
        Slot {
            x: 4.5..9.0,
            y: 10.0..10.0
        }
    );
    let position = RealPosition {
        layer: -1,
        rect: [0.0, 0.0, 9.0, 10.0],
    };
    let mut children = ChildPositions(SmallVec::new());
    grid.split(&slots, offset, &position, &mut children);
    assert_eq!(children.0[0].rect, [0.0, 0.0, 4.0, 5.0]);
}
