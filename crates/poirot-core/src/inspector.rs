///TODO: See what needs to be put here
pub struct Trace {}

pub trait Inspector {
    fn inspect(&self, trace: &mut Trace);
}