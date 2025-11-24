#[cfg(target_arch = "wasm32")]
mod time_inner {
    use std::time::Duration;

    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = Date, js_name = now)]
        fn get_timestamp() -> f64;
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant(u64);

    impl Instant {
        #[inline]
        #[must_use]
        pub fn now() -> Self {
            Self(get_timestamp() as u64)
        }

        #[inline]
        #[must_use]
        pub fn elapsed(&self) -> Duration {
            Duration::from_millis(Self::now().0 - self.0)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod time_inner {
    use std::ops::Deref;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant(std::time::Instant);

    impl Deref for Instant {
        type Target = std::time::Instant;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Instant {
        #[inline]
        #[must_use]
        pub fn now() -> Self {
            Self(std::time::Instant::now())
        }
    }
}

pub use time_inner::Instant;
