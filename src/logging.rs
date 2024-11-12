//! This code is from a pull request to zbus that was never merged. https://github.com/REALERvolker1/zbus/blob/26a7cd8109fb0329240478296fce3ef955aaa821/zbus/src/abstractions/logging.rs#L2
#[allow(unused_macros)]
#[cfg(feature = "tracing")]
mod log {
    /// A macro for [`tracing::debug`], meant to make it optional.
    macro_rules! debug {
        ($first:literal $(, $arg:expr)+$(,)?) => {
            ::tracing::debug!($first $(, $arg)+)
        };
        ($only:literal) => {
            ::tracing::debug!($only)
        };
    }
    /// A macro for [`tracing::trace`], meant to make it optional.
    macro_rules! trace {
        ($first:literal $(, $arg:expr)+$(,)?) => {
            ::tracing::trace!($first $(, $arg)+)
        };
        ($only:literal) => {
            ::tracing::trace!($only)
        };
    }
    /// A macro for [`tracing::warn`], meant to make it optional.
    ///
    /// This is named warning because I was getting internal attribute collisions
    macro_rules! warning {
        ($first:literal $(, $arg:expr)+$(,)?) => {
            ::tracing::warn!($first $(, $arg)+)
        };
        ($only:literal) => {
            ::tracing::warn!($only)
        };
    }
    /// A macro for [`tracing::info`], meant to make it optional.
    macro_rules! info {
        ($first:literal $(, $arg:expr)+$(,)?) => {
            ::tracing::info!($first $(, $arg)+)
        };
        ($only:literal) => {
            ::tracing::info!($only)
        };
    }

    pub(crate) use debug;
    pub(crate) use info;
    pub(crate) use trace;
    pub(crate) use warning;
}

#[cfg(not(feature = "tracing"))]
mod log {
    /// A macro for `tracing::debug` that does nothing, because the tracing feature is disabled.
    macro_rules! debug {
        ($first:literal $(, $arg:expr)*$(,)?) => {
            {
                #[allow(unused_variables)]
                let _ = (&$first $(, &$arg)*);
            }
        };
    }
    /// A macro for `tracing::trace` that does nothing, because the tracing feature is disabled.
    macro_rules! trace {
        ($first:literal $(, $arg:expr)*$(,)?) => {
            {
                #[allow(unused_variables)]
                let _ = (&$first $(, &$arg)*);
            }
        };
    }

    /// A macro for `tracing::warn` that does nothing, because the tracing feature is disabled.
    macro_rules! warning {
        ($first:literal $(, $arg:expr)*$(,)?) => {
            {
                #[allow(unused_variables)]
                let _ = (&$first $(, &$arg)*);
            }
        };
    }
    /// A macro for `tracing::info` that does nothing, because the tracing feature is disabled.
    macro_rules! info {
        ($first:literal $(, $arg:expr)*$(,)?) => {
            {
                #[allow(unused_variables)]
                let _ = (&$first $(, &$arg)*);
            }
        };
    }

    pub(crate) use debug;
    pub(crate) use info;
    pub(crate) use trace;
    pub(crate) use warning;
}

pub(crate) use log::*;
