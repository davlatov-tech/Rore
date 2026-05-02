use crate::reactive::signals::{create_effect, Signal};
use std::marker::PhantomData;

/// Memo - bu qimmatli hisob-kitoblarni keshlovchi Reaktiv Quti.
#[derive(Debug)]
pub struct Memo<T> {
    pub signal: Signal<T>, // MUAMMO HAL QILINDI: Public va Option'siz!
    _marker: PhantomData<T>,
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal,
            _marker: PhantomData,
        }
    }
}
impl<T> Copy for Memo<T> {}

impl<T: Clone + PartialEq + 'static> Memo<T> {
    pub fn new<F: FnMut() -> T + 'static>(mut f: F) -> Self {
        // Dastlabki qiymatni darhol hisoblab, sof signal yaratamiz
        let initial_val = f();
        let sig = Signal::new(initial_val);

        // Qolgan o'zgarishlarni Effect kuzatadi
        create_effect(move || {
            let new_val = f();
            if sig.get_untracked() != new_val {
                sig.set(new_val);
            }
        });

        Self {
            signal: sig,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> T {
        self.signal.get()
    }
}

pub fn create_memo<T, F>(f: F) -> Memo<T>
where
    T: Clone + PartialEq + 'static,
    F: FnMut() -> T + 'static,
{
    Memo::new(f)
}
