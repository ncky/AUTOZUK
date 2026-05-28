use crate::engine::{MobProjectile, PlayerProjectile};

#[derive(Clone, Copy)]
pub(crate) struct DamageResult {
    pub(crate) damage: i32,
    pub(crate) died: bool,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PendingMobRemoval {
    pub(crate) tick: i32,
    pub(crate) mob_id: usize,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PendingPlayerHit {
    pub(crate) tick: i32,
    pub(crate) mob_id: usize,
    pub(crate) damage: i32,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PendingRecoil {
    pub(crate) tick: i32,
    pub(crate) mob_id: usize,
    pub(crate) damage: i32,
}

#[derive(Clone)]
pub(crate) struct SmallQueue<T: Copy + Default, const N: usize> {
    pub(crate) inline: [T; N],
    pub(crate) inline_len: usize,
    pub(crate) overflow: Vec<T>,
}

impl<T: Copy + Default, const N: usize> SmallQueue<T, N> {
    pub(crate) fn new() -> Self {
        Self {
            inline: [T::default(); N],
            inline_len: 0,
            overflow: Vec::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.inline_len == 0 && self.overflow.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.inline_len + self.overflow.len()
    }

    pub(crate) fn push(&mut self, item: T) {
        if self.inline_len < N {
            self.inline[self.inline_len] = item;
            self.inline_len += 1;
        } else {
            self.overflow.push(item);
        }
    }

    pub(crate) fn get(&self, idx: usize) -> T {
        if idx < self.inline_len {
            self.inline[idx]
        } else {
            self.overflow[idx - self.inline_len]
        }
    }

    pub(crate) fn set(&mut self, idx: usize, item: T) {
        if idx < self.inline_len {
            self.inline[idx] = item;
        } else {
            self.overflow[idx - self.inline_len] = item;
        }
    }

    pub(crate) fn swap_remove(&mut self, idx: usize) -> T {
        if idx < self.inline_len {
            let item = self.inline[idx];
            self.inline_len -= 1;
            if idx < self.inline_len {
                self.inline[idx] = self.inline[self.inline_len];
            }
            item
        } else {
            self.overflow.swap_remove(idx - self.inline_len)
        }
    }

    pub(crate) fn retain(&mut self, mut keep: impl FnMut(T) -> bool) {
        let mut write = 0usize;
        for read in 0..self.inline_len {
            let item = self.inline[read];
            if keep(item) {
                self.inline[write] = item;
                write += 1;
            }
        }
        self.inline_len = write;
        self.overflow.retain(|item| keep(*item));
    }

    pub(crate) fn first_mut(&mut self, mut pred: impl FnMut(T) -> bool) -> Option<&mut T> {
        for idx in 0..self.inline_len {
            if pred(self.inline[idx]) {
                return Some(&mut self.inline[idx]);
            }
        }
        self.overflow.iter_mut().find(|item| pred(**item))
    }

    pub(crate) fn for_each(&self, mut f: impl FnMut(T)) {
        for idx in 0..self.inline_len {
            f(self.inline[idx]);
        }
        for item in &self.overflow {
            f(*item);
        }
    }
}

pub(crate) type PendingMobRemovalQueue = SmallQueue<PendingMobRemoval, 16>;
pub(crate) type PendingPlayerHitQueue = SmallQueue<PendingPlayerHit, 32>;
pub(crate) type PendingRecoilQueue = SmallQueue<PendingRecoil, 16>;
pub(crate) type PlayerProjectileQueue = SmallQueue<PlayerProjectile, 8>;
pub(crate) type MobProjectileQueue = SmallQueue<MobProjectile, 32>;
