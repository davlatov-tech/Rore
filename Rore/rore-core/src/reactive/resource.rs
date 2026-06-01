use crate::reactive::command::{CommandQueue, UICommand};
use crate::reactive::signals::Signal;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

// GPU uxlab qolmasligi uchun aktiv ishlarni sanab turuvchi ko'rsatkich
pub static ACTIVE_RESOURCES: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub enum ResourceState<T> {
    Pending,
    Resolved(T),
}

#[derive(Debug)]
pub struct Resource<T> {
    pub signal: Signal<ResourceState<T>>,
}

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal,
        }
    }
}
impl<T> Copy for Resource<T> {}

impl<T: Clone + Send + Sync + 'static> Resource<T> {
    pub fn read(&self) -> ResourceState<T> {
        self.signal.get()
    }
    pub fn loading(&self) -> bool {
        matches!(self.signal.get(), ResourceState::Pending)
    }
}

pub fn create_resource<T, F>(f: F) -> Resource<T>
where
    T: Clone + Send + Sync + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let sig = Signal::new(ResourceState::Pending);
    let sig_id = sig.id.0;

    // Ish boshlandi, Dvigatelga "uxlama" deymiz
    ACTIVE_RESOURCES.fetch_add(1, Ordering::SeqCst);

    thread::spawn(move || {
        let result = f();

        let resolved_state = ResourceState::Resolved(result);

        // MUAMMO HAL QILINDI: Qutiga aniq tip berib yuboramiz
        CommandQueue::send(UICommand::UpdateResource(
            sig_id,
            Box::new(resolved_state) as Box<dyn std::any::Any + Send>,
        ));

        // Ish tugadi, endi uxlash mumkin
        ACTIVE_RESOURCES.fetch_sub(1, Ordering::SeqCst);
    });

    Resource { signal: sig }
}
