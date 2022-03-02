use crate::slot::{Slot, Squishy, accum_par, accum_seq, correct_range, max_default_b};

#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub slots: Vec<Slot>
}
impl Grid {

}
impl Squishy for Grid {
    fn slotify(&self) -> Slot {
        let x = accum_par(
            self.slots.chunks(self.width)
                .map(|chunk| {
                    accum_seq(
                        chunk.iter().map(|slot| slot.x.clone() )
                    )
                })
        );
        let y = accum_seq(
            self.slots.chunks(self.width)
                .map(|chunk| {
                    accum_par(
                        chunk.iter().map(|slot| slot.y.clone() )
                    )
                })
        );
        Slot {
            x,
            y
        }
    }
}

#[test]
fn grid_slotting() {
    let grid = Grid {
        width: 2,
        slots: vec![
            Slot {
                x: 2.0..4.0,
                y: 4.0..5.0
            },
            Slot {
                x: 2.5..5.0,
                y: 5.0..6.0
            },
            Slot {
                x: 2.0..4.0,
                y: 4.0..5.0
            },
            Slot {
                x: 2.5..5.0,
                y: 5.0..6.0
            }
        ]
    };
    assert_eq!(grid.slotify(), Slot { x: 4.5..9.0, y: 10.0..10.0 })
}

#[derive(Debug, Clone)]
pub enum Alignment {
    Left,
    Center,
    Right
}

#[derive(Debug, Clone)]
pub struct Floating {
    pub h_alignment: Alignment,
    pub v_alignment: Alignment,
    pub w_max: f32,
    pub h_max: f32,
    pub inner: Slot
}
impl Squishy for Floating {
    fn slotify(&self) -> Slot {
        let x = correct_range(self.inner.x.start..max_default_b(self.inner.x.start, self.w_max));
        let y = correct_range(self.inner.y.start..max_default_b(self.inner.y.start, self.h_max));
        Slot {
            x,
            y
        }
    }
}
