#[derive(Debug, Clone)]
pub struct RealPosition {
    pub layer: i16,
    /// [start x. start y. end x, end y]
    pub rect: [f32; 4]
}

pub struct ChildPositions (pub Vec<RealPosition>);

/// A signal to emit to reconfigure a branch
pub struct Reconfigure;

pub trait Parent {
    fn split(&self, target: &RealPosition, children: &mut ChildPositions) -> Propagation;
}

pub enum Propagation {
    Continue,
    Stop
}
