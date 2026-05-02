use crate::reactive::signals::{create_effect, Signal, RUNTIME};
use rore_types::Color;
use std::any::TypeId;
use std::cell::RefCell;

// =========================================================================
// 1. GLOBAL CONTEXT API (Prop-Drilling Qotili)
// =========================================================================
pub fn provide_context<T: 'static + Clone + Send>(value: T) {
    RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        rt_mut
            .global_contexts
            .insert(TypeId::of::<T>(), Box::new(value));
    });
}

pub fn use_context<T: 'static + Clone>() -> Option<T> {
    RUNTIME.with(|rt| {
        let rt_ref = rt.borrow();
        rt_ref
            .global_contexts
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref::<T>().cloned())
    })
}

// =========================================================================
// 2. REAKTIV ANIMATSIYALAR (Tween Dvigateli)
// =========================================================================

/// Animatsiya uslublari (Matematik Easing)
#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Linear,
    EaseOut,
    EaseInOut,
    Bounce,
}

/// Oradagi piksellar yoki ranglarni hisoblovchi Trait
pub trait Tweenable {
    fn interpolate(start: &Self, end: &Self, t: f32) -> Self;
}

// Sonlar uchun (Kenglik, balandlik, radius)
impl Tweenable for f32 {
    fn interpolate(start: &Self, end: &Self, t: f32) -> Self {
        start + (end - start) * t
    }
}

// Ranglar uchun (Silliq o'tuvchi animatsiya)
// Ranglar uchun (Silliq o'tuvchi animatsiya)
impl Tweenable for Color {
    fn interpolate(start: &Self, end: &Self, t: f32) -> Self {
        // Ranglarni kasr ko'rinishida hisoblab, keyin u8 ga o'tkazamiz
        let r = (start.r as f32 + (end.r as f32 - start.r as f32) * t).round() as u8;
        let g = (start.g as f32 + (end.g as f32 - start.g as f32) * t).round() as u8;
        let b = (start.b as f32 + (end.b as f32 - start.b as f32) * t).round() as u8;
        let a = start.a + (end.a - start.a) * t;

        Color::rgba(r, g, b, a)
    }
}

trait TweenTask {
    fn tick(&mut self, dt: f32) -> bool; // true qaytarsa, animatsiya tugagan bo'ladi
}

struct TweenInstance<T: Tweenable + Clone + 'static> {
    output: Signal<T>,
    start: T,
    end: T,
    duration: f32,
    elapsed: f32,
    easing: Easing,
}

impl<T: Tweenable + Clone + 'static> TweenTask for TweenInstance<T> {
    fn tick(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        let mut progress = self.elapsed / self.duration;
        if progress >= 1.0 {
            progress = 1.0;
        }

        // Matematik Easing Formulalari
        let t = match self.easing {
            Easing::Linear => progress,
            Easing::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
            Easing::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
                }
            }
            Easing::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                let mut p = progress;
                if p < 1.0 / d1 {
                    n1 * p * p
                } else if p < 2.0 / d1 {
                    p -= 1.5 / d1;
                    n1 * p * p + 0.75
                } else if p < 2.5 / d1 {
                    p -= 2.25 / d1;
                    n1 * p * p + 0.9375
                } else {
                    p -= 2.625 / d1;
                    n1 * p * p + 0.984375
                }
            }
        };

        // Yangi oraliq qiymatni hisoblab, UI ga yuboramiz
        let current_val = T::interpolate(&self.start, &self.end, t);
        self.output.set(current_val);

        progress >= 1.0
    }
}

// Global Animatsiyalar Navbati
thread_local! {
    pub static ACTIVE_TWEENS: RefCell<Vec<Box<dyn TweenTask>>> = RefCell::new(Vec::new());
}

/// Dasturchi uchun interfeys: Asosiy signalni silliq ishlovchi signalga aylantiradi
pub fn create_tween<T: Tweenable + Clone + PartialEq + 'static>(
    target: Signal<T>,
    duration: f32,
    easing: Easing,
) -> Signal<T> {
    let tweened = Signal::new(target.get_untracked());

    create_effect(move || {
        let new_target = target.get();
        let current = tweened.get_untracked();

        if current != new_target {
            ACTIVE_TWEENS.with(|tweens| {
                tweens.borrow_mut().push(Box::new(TweenInstance {
                    output: tweened,
                    start: current,
                    end: new_target,
                    duration,
                    elapsed: 0.0,
                    easing,
                }));
            });
        }
    });

    tweened
}

pub fn tick_tweens(dt: f32) -> bool {
    ACTIVE_TWEENS.with(|tweens| {
        let mut list = tweens.borrow_mut();
        list.retain_mut(|tween| !tween.tick(dt));
        !list.is_empty() // TRUE: Animatsiya hali davom etyapti!
    })
}
