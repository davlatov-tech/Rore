use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};

pub static ACTIVE_TICKERS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub u64);

pub struct ReactiveRuntime {
    next_id: u64,
    signals: HashMap<SignalId, Box<dyn Any>>,
    effects: HashMap<EffectId, (Option<ScopeId>, Box<dyn FnMut()>)>,
    signal_subscribers: HashMap<SignalId, HashSet<EffectId>>,
    active_effect: Option<EffectId>,
    pub pending_effects: HashSet<EffectId>,

    // =========================================================
    // TICKER (Animatsiya / Fizika) xotirasi
    // =========================================================
    tickers: HashMap<u64, (Option<ScopeId>, Box<dyn FnMut(f32)>)>,

    scope_signals: HashMap<ScopeId, HashSet<SignalId>>,
    scope_effects: HashMap<ScopeId, HashSet<EffectId>>,
    scope_tickers: HashMap<ScopeId, HashSet<u64>>, // Scope ga ulangan tickerlar

    pub cleanups: HashMap<ScopeId, Vec<Box<dyn FnOnce()>>>,
    pub error_handlers: HashMap<ScopeId, Vec<Box<dyn FnMut(&(dyn Any + Send + 'static))>>>,

    pub active_scope: Option<ScopeId>,
    pub batch_depth: usize,
    pub global_contexts: HashMap<TypeId, Box<dyn Any>>,
}

impl ReactiveRuntime {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            signals: HashMap::new(),
            effects: HashMap::new(),
            signal_subscribers: HashMap::new(),
            active_effect: None,
            pending_effects: HashSet::new(),
            tickers: HashMap::new(),
            scope_signals: HashMap::new(),
            scope_effects: HashMap::new(),
            scope_tickers: HashMap::new(),
            cleanups: HashMap::new(),
            error_handlers: HashMap::new(),
            active_scope: None,
            batch_depth: 0,
            global_contexts: HashMap::new(),
        }
    }

    pub fn generate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

thread_local! {
    pub static RUNTIME: RefCell<ReactiveRuntime> = RefCell::new(ReactiveRuntime::new());
}

// =========================================================================
// CONTEXT API
// =========================================================================

pub fn provide_context<T: 'static + Clone + Send + Sync>(value: T) {
    RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        let type_id = TypeId::of::<T>();
        rt_mut.global_contexts.insert(type_id, Box::new(value));
    });
}

pub fn use_context<T: 'static + Clone>() -> Option<T> {
    RUNTIME.with(|rt| {
        let rt_ref = rt.borrow();
        let type_id = TypeId::of::<T>();
        rt_ref
            .global_contexts
            .get(&type_id)
            .and_then(|any_val| any_val.downcast_ref::<T>().cloned())
    })
}

// =========================================================================
// BATCH VA UNTRACK
// =========================================================================

pub fn batch<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    RUNTIME.with(|rt| {
        rt.borrow_mut().batch_depth += 1;
    });

    let result = f();

    let should_process = RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        rt_mut.batch_depth -= 1;
        rt_mut.batch_depth == 0
    });

    if should_process {
        process_pending_effects();
    }

    result
}

pub fn untrack<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let prev_effect = RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        let prev = rt_mut.active_effect;
        rt_mut.active_effect = None;
        prev
    });

    let result = f();

    RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        rt_mut.active_effect = prev_effect;
    });

    result
}

// =========================================================================
// SCOPE, XOTIRA VA XAVFSIZLIK (Garbage Collection)
// =========================================================================

pub fn get_active_scope() -> Option<ScopeId> {
    RUNTIME.with(|rt| rt.borrow().active_scope)
}

pub fn create_scope<F, R>(f: F) -> (ScopeId, R)
where
    F: FnOnce() -> R,
{
    let scope_id = RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        ScopeId(rt_mut.generate_id())
    });

    let prev_scope = RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        let prev = rt_mut.active_scope;
        rt_mut.active_scope = Some(scope_id);
        prev
    });

    let result = f();

    RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        rt_mut.active_scope = prev_scope;
    });

    (scope_id, result)
}

pub fn on_cleanup<F: FnOnce() + 'static>(f: F) {
    RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        if let Some(scope_id) = rt_mut.active_scope {
            rt_mut
                .cleanups
                .entry(scope_id)
                .or_default()
                .push(Box::new(f));
        }
    });
}

pub fn catch_error<F: FnMut(&(dyn Any + Send + 'static)) + 'static>(f: F) {
    RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        if let Some(scope_id) = rt_mut.active_scope {
            rt_mut
                .error_handlers
                .entry(scope_id)
                .or_default()
                .push(Box::new(f));
        }
    });
}

pub fn dispose_scope(scope_id: ScopeId) {
    let cleanups = RUNTIME.with(|rt| rt.borrow_mut().cleanups.remove(&scope_id));
    if let Some(cleanups) = cleanups {
        for cleanup in cleanups {
            cleanup();
        }
    }

    RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();

        rt_mut.error_handlers.remove(&scope_id);

        if let Some(effects) = rt_mut.scope_effects.remove(&scope_id) {
            for effect_id in effects {
                rt_mut.effects.remove(&effect_id);
                rt_mut.pending_effects.remove(&effect_id);
                for (_, subs) in rt_mut.signal_subscribers.iter_mut() {
                    subs.remove(&effect_id);
                }
            }
        }

        if let Some(signals) = rt_mut.scope_signals.remove(&scope_id) {
            for signal_id in signals {
                rt_mut.signals.remove(&signal_id);
                rt_mut.signal_subscribers.remove(&signal_id);
            }
        }

        // YANGI: Tickerlarni yo'q qilish va Uyqu Qulfini (Wake Lock) yechish
        if let Some(tickers) = rt_mut.scope_tickers.remove(&scope_id) {
            for t_id in tickers {
                rt_mut.tickers.remove(&t_id);
                ACTIVE_TICKERS.fetch_sub(1, Ordering::SeqCst);
            }
        }
    });
}

// =========================================================================
// SIGNALS (Holat qutilari)
// =========================================================================

#[derive(Debug)]
pub struct Signal<T> {
    pub id: SignalId,
    _marker: PhantomData<T>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}
impl<T> Copy for Signal<T> {}

impl<T: 'static + Clone> Signal<T> {
    pub fn new(value: T) -> Self {
        RUNTIME.with(|rt| {
            let mut rt_mut = rt.borrow_mut();
            let id = SignalId(rt_mut.generate_id());
            rt_mut.signals.insert(id, Box::new(value));

            if let Some(scope_id) = rt_mut.active_scope {
                rt_mut.scope_signals.entry(scope_id).or_default().insert(id);
            }

            Self {
                id,
                _marker: PhantomData,
            }
        })
    }

    pub fn get(&self) -> T {
        RUNTIME.with(|rt| {
            let mut rt_mut = rt.borrow_mut();
            if let Some(effect_id) = rt_mut.active_effect {
                rt_mut
                    .signal_subscribers
                    .entry(self.id)
                    .or_default()
                    .insert(effect_id);
            }
            let any_val = rt_mut
                .signals
                .get(&self.id)
                .expect("Signal topilmadi yoki o'chirilgan!");
            any_val
                .downcast_ref::<T>()
                .expect("Signal tipi xato!")
                .clone()
        })
    }

    pub fn get_untracked(&self) -> T {
        RUNTIME.with(|rt| {
            let rt_ref = rt.borrow();
            let any_val = rt_ref.signals.get(&self.id).expect("Signal topilmadi!");
            any_val
                .downcast_ref::<T>()
                .expect("Signal tipi xato!")
                .clone()
        })
    }

    pub fn set(&self, value: T) {
        let should_process = RUNTIME.with(|rt| {
            let mut rt_mut = rt.borrow_mut();
            rt_mut.signals.insert(self.id, Box::new(value));

            if let Some(subs) = rt_mut.signal_subscribers.get(&self.id).cloned() {
                for effect_id in subs {
                    rt_mut.pending_effects.insert(effect_id);
                }
            }
            rt_mut.batch_depth == 0
        });

        if should_process {
            process_pending_effects();
        }
    }

    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        let mut current = self.get_untracked();
        f(&mut current);
        self.set(current);
    }
}

pub fn create_signal_untracked<T: 'static>(value: T) -> Signal<T> {
    RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        let id = SignalId(rt_mut.generate_id());
        rt_mut.signals.insert(id, Box::new(value));

        if let Some(scope_id) = rt_mut.active_scope {
            rt_mut.scope_signals.entry(scope_id).or_default().insert(id);
        }

        Signal {
            id,
            _marker: PhantomData,
        }
    })
}

pub fn set_signal_untyped<T: 'static>(id: SignalId, value: T) {
    let should_process = RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        rt_mut.signals.insert(id, Box::new(value));
        if let Some(subs) = rt_mut.signal_subscribers.get(&id).cloned() {
            for effect_id in subs {
                rt_mut.pending_effects.insert(effect_id);
            }
        }
        rt_mut.batch_depth == 0
    });

    if should_process {
        process_pending_effects();
    }
}

pub fn get_signal_untyped<T: 'static + Clone>(id: SignalId) -> Option<T> {
    RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        if let Some(effect_id) = rt_mut.active_effect {
            rt_mut
                .signal_subscribers
                .entry(id)
                .or_default()
                .insert(effect_id);
        }
        rt_mut
            .signals
            .get(&id)
            .and_then(|any| any.downcast_ref::<T>().cloned())
    })
}

pub fn create_selector<T, U, F>(signal: Signal<T>, mut getter: F) -> Signal<U>
where
    T: 'static + Clone,
    U: 'static + Clone + PartialEq,
    F: FnMut(&T) -> U + 'static,
{
    let initial = getter(&signal.get_untracked());
    let derived = Signal::new(initial);

    create_effect(move || {
        let new_val = getter(&signal.get());
        if new_val != derived.get_untracked() {
            derived.set(new_val);
        }
    });

    derived
}

pub fn create_effect<F: FnMut() + 'static>(mut f: F) -> EffectId {
    let (id, prev_effect, active_scope) = RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        let new_id = EffectId(rt_mut.generate_id());
        let prev = rt_mut.active_effect;
        let scope = rt_mut.active_scope;
        rt_mut.active_effect = Some(new_id);
        (new_id, prev, scope)
    });

    let _ = catch_unwind(AssertUnwindSafe(|| {
        f();
    }));

    RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        rt_mut.active_effect = prev_effect;
        rt_mut.effects.insert(id, (active_scope, Box::new(f)));

        if let Some(scope_id) = active_scope {
            rt_mut.scope_effects.entry(scope_id).or_default().insert(id);
        }
        id
    })
}

// =========================================================================
// YANGI: REAKTIV TICKERS (Animatsiya tsikli)
// =========================================================================

pub fn create_ticker<F: FnMut(f32) + 'static>(f: F) -> u64 {
    let id = RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        let id = rt_mut.generate_id();
        let scope = rt_mut.active_scope;

        rt_mut.tickers.insert(id, (scope, Box::new(f)));
        if let Some(scope_id) = scope {
            rt_mut.scope_tickers.entry(scope_id).or_default().insert(id);
        }
        id
    });
    // Uyqu Qulfini yopamiz (Dvigatelga uxlash taqiqlanadi!)
    ACTIVE_TICKERS.fetch_add(1, Ordering::SeqCst);
    id
}

pub fn tick_all(dt: f32) {
    RUNTIME.with(|rt| {
        rt.borrow_mut().batch_depth += 1;
    });

    RUNTIME.with(|rt| {
        // Tickerlarni ajratib olamiz (Borrow checker xato bermasligi uchun)
        let mut tickers = {
            let mut rt_mut = rt.borrow_mut();
            std::mem::take(&mut rt_mut.tickers)
        };

        for (_, (_, ref mut f)) in tickers.iter_mut() {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                f(dt);
            }));
        }

        let mut rt_mut = rt.borrow_mut();
        // Ish tugagach, ularni yana qaytarib joyiga qo'yamiz
        for (k, v) in tickers {
            rt_mut.tickers.insert(k, v);
        }
    });

    RUNTIME.with(|rt| {
        rt.borrow_mut().batch_depth -= 1;
    });
    process_pending_effects();
}

pub fn process_pending_effects() {
    RUNTIME.with(|rt| {
        rt.borrow_mut().batch_depth += 1;
    });

    RUNTIME.with(|rt| {
        let mut pending = {
            let mut rt_mut = rt.borrow_mut();
            std::mem::take(&mut rt_mut.pending_effects)
        };

        while !pending.is_empty() {
            for effect_id in pending.drain() {
                let effect_data = {
                    let mut rt_inner = rt.borrow_mut();
                    rt_inner.effects.remove(&effect_id)
                };

                if let Some((owner_scope, mut f)) = effect_data {
                    let prev = {
                        let mut rt_inner = rt.borrow_mut();
                        let p = rt_inner.active_effect;
                        rt_inner.active_effect = Some(effect_id);
                        p
                    };

                    let result = catch_unwind(AssertUnwindSafe(|| {
                        f();
                    }));

                    let mut rt_inner = rt.borrow_mut();
                    rt_inner.active_effect = prev;
                    rt_inner.effects.insert(effect_id, (owner_scope, f));

                    if let Err(payload) = result {
                        if let Some(scope_id) = owner_scope {
                            if let Some(mut handlers) = rt_inner.error_handlers.remove(&scope_id) {
                                for handler in &mut handlers {
                                    handler(&*payload);
                                }
                                rt_inner.error_handlers.insert(scope_id, handlers);
                            }
                        }
                    }
                }
            }

            pending = {
                let mut rt_inner = rt.borrow_mut();
                std::mem::take(&mut rt_inner.pending_effects)
            };
        }
    });

    RUNTIME.with(|rt| {
        rt.borrow_mut().batch_depth -= 1;
    });
}

pub fn set_signal_any(id: SignalId, value: Box<dyn Any + Send>) {
    let should_process = RUNTIME.with(|rt| {
        let mut rt_mut = rt.borrow_mut();
        rt_mut.signals.insert(id, value);
        if let Some(subs) = rt_mut.signal_subscribers.get(&id).cloned() {
            for effect_id in subs {
                rt_mut.pending_effects.insert(effect_id);
            }
        }
        rt_mut.batch_depth == 0
    });

    if should_process {
        process_pending_effects();
    }
}
