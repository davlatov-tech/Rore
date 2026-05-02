use crate::reactive::resource::{Resource, ResourceState};
use crate::widgets::base::Widget;

pub struct Suspense<T> {
    _resource: Resource<T>,
}

impl<T: Clone + Send + Sync + 'static> Suspense<T> {
    pub fn new(resource: Resource<T>) -> SuspenseBuilder<T, fn() -> Box<dyn Widget>> {
        SuspenseBuilder {
            resource,
            fallback: None,
        }
    }
}

pub struct SuspenseBuilder<T, F1> {
    resource: Resource<T>,
    fallback: Option<F1>,
}

impl<T: Clone + Send + Sync + 'static, F1: Fn() -> Box<dyn Widget> + Send + Sync + 'static>
    SuspenseBuilder<T, F1>
{
    pub fn fallback<F: Fn() -> Box<dyn Widget> + Send + Sync + 'static>(
        self,
        fallback: F,
    ) -> SuspenseBuilder<T, F> {
        SuspenseBuilder {
            resource: self.resource,
            fallback: Some(fallback),
        }
    }

    pub fn child<F2: Fn(T) -> Box<dyn crate::widgets::base::Widget> + Send + 'static>(
        self,
        child: F2,
    ) -> crate::widgets::show::Show {
        let res = self.resource;
        let fb_arc = std::sync::Arc::new(
            self.fallback
                .expect("Suspense: Fallback kiritilishi shart!"),
        );

        let fb_clone1 = fb_arc.clone();
        let fb_clone2 = fb_arc.clone();

        crate::widgets::show::Show::new(
            move || !res.loading(), // SHART: Yuklanish tugadimi?
            move || {
                if let ResourceState::Resolved(val) = res.read() {
                    child(val)
                } else {
                    fb_clone1() // Xavfsizlik uchun
                }
            },
            move || fb_clone2(),
        )
    }
}
