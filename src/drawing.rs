use smallvec::SmallVec;

#[derive(Debug, Clone, PartialEq)]
pub struct RealPosition {
    pub layer: i16,
    /// [start x. start y. end x, end y]
    pub rect: [f32; 4],
}

#[derive(Debug, Clone)]
/// A structure for passing positions to multiple children
pub struct ChildPositions(pub SmallVec<[RealPosition; 2]>);

/// A signal to emit to reconfigure a branch
pub struct Reconfigure;

pub trait Parent {
    fn split(&self, target: &RealPosition, children: &mut ChildPositions) -> Propagation;
}

pub enum Propagation {
    Continue,
    Stop,
}
