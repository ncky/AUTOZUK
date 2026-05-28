use serde::Serialize;

#[derive(Clone, Copy)]
pub(crate) struct Mulberry32 {
    pub(crate) s: u32,
}

impl Mulberry32 {
    pub(crate) fn new(seed: u32) -> Self {
        Self { s: seed }
    }

    pub(crate) fn next_f64(&mut self) -> f64 {
        self.s = self.s.wrapping_add(0x6D2B79F5);
        let mut t = self.s;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        ((t ^ (t >> 14)) as f64) / 4_294_967_296.0
    }
}

#[derive(Serialize)]
pub(crate) struct TileOut {
    pub(crate) x: i32,
    pub(crate) y: i32,
}
