pub mod frame;
pub mod stack;
pub struct CallStack {
    pub frames: Vec<frame::Frame>,
    pub limit: usize,
}
