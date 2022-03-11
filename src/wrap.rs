use std::ops::Range;

use crate::positioning::{Slot, Squishy, Propagation, RealPosition, ChildPositions, reverse_lerp, lerp};

pub enum Direction {
    /// increasing x and y
    RightUp,
    /// increasing x decreasing y
    RightDown,
    /// decreasing x increasing y
    LeftUp,
    /// decreasing x and y
    LeftDown
}

pub struct Wrapped {
    pub direction: Direction,
    pub width: Range<f32>,
}
impl Squishy for Wrapped {
    fn slotify(&self, slots: &[Slot], offset: [f32; 2]) -> Slot {
        let max_rows = rows_under(slots, 1., offset, self.width.end);
        // can collapse into one fold
        let min_height: f32 = max_rows.clone().map(|row| row.height + (2.0 * offset[1])).sum();
        let max_width = max_rows.map(|r| r.width).fold(0.0, f32::max);

        let min_rows = rows_over(slots, 0., offset, self.width.start);
        // likewise
        let max_height: f32 = min_rows.clone().map(|row| row.height + (2.0 * offset[1])).sum();
        let min_width = min_rows.map(|r| r.width).fold(0.0, f32::max);

        Slot {
            x: min_width..max_width,
            y: min_height..max_height
        }
    }
    fn split(
        &self,
        slots: &[Slot],
        offset: [f32; 2],
        target: &RealPosition,
        children: &mut ChildPositions,
    ) -> Propagation {
        let slot = self.slotify(slots, offset);
        let width = target.size(0);
        let x_factor = reverse_lerp(width, slot.x);
        let y_factor = reverse_lerp(target.size(1), slot.y);
        
        // use rows_under with that x_factor
        let sign = match &self.direction {
            &Direction::RightUp => [1.0, 1.0],
            &Direction::RightDown => [1.0, -1.0],
            &Direction::LeftUp => [-1.0, 1.0],
            &Direction::LeftDown => [-1.0, -1.0]
        };
        let mut current_y = -sign[1] * offset[1] + map_lerp(sign[1], target.rect[1]..target.rect[3]);
        let rows = rows_under(slots, x_factor, offset, width);
        for row in rows {
            current_y += sign[1] * 2.0 * offset[1];
            let mut current_x = -sign[0] * offset[0];
            for (slot, out) in slots[row.slot_nums.clone()].iter().zip(&mut children.0[row.slot_nums]) {
                current_x += sign[0] * 2.0 * offset[0] + map_lerp(sign[0], target.rect[0]..target.rect[2]);
                let end_x = current_x + lerp(x_factor, slot.x.clone());
                let end_y = current_y + lerp(y_factor, slot.y.clone());
                let child_target = RealPosition {
                    layer: target.layer + 1,
                    rect: [current_x, current_y, end_x, end_y]
                };
                *out = child_target;
            }
        }
        Propagation::Stop
    }
}

fn map_lerp(x: f32, range: Range<f32>) -> f32 {
    lerp(x * 0.5 + 0.5, range)
}

//Height is max. Width is min
fn rows_over<'a>(slots: &'a [Slot], interpolation: f32, offset: [f32; 2], over: f32) -> impl 'a + Iterator<Item = RowInfo> + Clone {
    let mut row_width_total = 0.;
    let mut row_height_max = 0.;
    let mut start_idx = 0;
    slots.iter().enumerate().filter_map(move |(current, s)| {
        // lerp = s.x.start if x == 0
        row_width_total += lerp(interpolation, s.x.clone()) + (2.0 * offset[0]);
        row_height_max = s.y.end.max(row_height_max);
        match row_width_total > over {
            false => None,
            true => {
                let info = RowInfo {
                    width: row_width_total,
                    height: row_height_max,
                    slot_nums: start_idx..1 + current,
                };
                row_width_total = 0.;
                row_height_max = 0.;
                start_idx = current + 1;
                Some(info)
            }
        }
    })
}
// Height is the min. Width is max
fn rows_under<'a>(slots: &'a [Slot], interpolation: f32, offset: [f32; 2], under: f32) -> impl 'a + Iterator<Item = RowInfo> + Clone {
    let mut row_width_total = 0.;
    let mut row_height_max = 0.;
    let mut start_idx = 0;
    slots.iter().enumerate().filter_map(move |(current, s)| {
        let test_width = row_width_total + lerp(interpolation, s.x.clone()) + (2.0 * offset[0]);
        let test_height = s.y.start.max(row_height_max);
        match row_width_total < under {
            false => {
                row_width_total = test_width;
                row_height_max = test_height;
                None
            }
            true => {
                let info = RowInfo {
                    width: row_width_total,
                    height: row_height_max,
                    slot_nums: start_idx..current,
                };
                row_width_total = s.x.start;
                row_height_max = s.y.start;
                start_idx = current + 1;
                Some(info)
            }
        }
    })
}
struct RowInfo {
    slot_nums: Range<usize>,
    width: f32,
    height: f32,
}
