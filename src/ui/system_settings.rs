//! The system accessibility settings: Reduce Motion, Increase Contrast and
//! Reduce Transparency (ADR 0101 Decision 11, musicindex ADR 0068 R-A11Y-1).
//!
//! This file has no `HitIt` type, so that it can move to musicindex-lib when
//! Studio needs it. The trait [`AccessibilitySource`] gives the three
//! settings. The portal source reads them from the xdg-desktop-portal Settings
//! interface, namespace `org.freedesktop.appearance`. The fake source of the
//! tests needs no system.

#![warn(clippy::pedantic)]

use std::future::Future;

use anyhow::Result;

/// The three system settings. `false` means no preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AccessibilitySettings {
    pub reduce_motion: bool,
    pub increase_contrast: bool,
    pub reduce_transparency: bool,
}

impl gpui::Global for AccessibilitySettings {}

impl AccessibilitySettings {
    /// Returns the settings installed on `cx`, or no preference when the
    /// source has not given any value yet.
    #[must_use]
    pub fn current(cx: &gpui::App) -> Self {
        cx.try_global::<Self>().copied().unwrap_or_default()
    }
}

/// A source of the three settings, with a signal for a change.
pub trait AccessibilitySource {
    /// Reads the three settings.
    ///
    /// # Errors
    ///
    /// Returns an error when the system does not answer.
    fn read(&self) -> impl Future<Output = Result<AccessibilitySettings>>;

    /// Waits until one setting can have changed.
    ///
    /// # Errors
    ///
    /// Returns an error when the source cannot report changes. The caller
    /// then stops the wait.
    fn changed(&mut self) -> impl Future<Output = Result<()>>;
}

/// Reads the settings. A failed read gives `false` for each setting and one
/// log line.
pub async fn read_or_default<S: AccessibilitySource>(source: &S) -> AccessibilitySettings {
    match source.read().await {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!(
                "The system accessibility settings are not available. HitIt uses no preference. {error:#}"
            );
            AccessibilitySettings::default()
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub use portal::PortalSource;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod portal {
    use std::future::poll_fn;
    use std::pin::Pin;
    use std::task::Poll;

    use anyhow::{Context as _, Result};
    use ashpd::desktop::settings::{
        Contrast, ReducedMotion, Settings, APPEARANCE_NAMESPACE, CONTRAST_KEY, REDUCED_MOTION_KEY,
    };
    use ashpd::zbus::export::futures_core::Stream;

    use super::{AccessibilitySettings, AccessibilitySource};

    /// A change signal of one key. The value is not read from the signal.
    type Changes = Pin<Box<dyn Stream<Item = ()>>>;

    /// The Settings portal of xdg-desktop-portal.
    ///
    /// The keys are `contrast` and `reduced-motion` of the namespace
    /// `org.freedesktop.appearance`. The portal has no key for Reduce
    /// Transparency, so that setting is always `false`.
    pub struct PortalSource {
        settings: Settings,
        contrast_changes: Changes,
        motion_changes: Changes,
    }

    /// Turns the signal of one key into a stream of unit values.
    async fn key_changes<T>(settings: &Settings, key: &str) -> Result<Changes>
    where
        T: TryFrom<ashpd::zvariant::OwnedValue> + 'static,
        ashpd::Error: From<<T as TryFrom<ashpd::zvariant::OwnedValue>>::Error>,
    {
        let stream = settings
            .receive_setting_changed_with_args::<T>(APPEARANCE_NAMESPACE, key)
            .await
            .with_context(|| format!("the Settings portal gives no change signal for {key}"))?;
        Ok(Box::pin(MapUnit(Box::pin(stream))))
    }

    /// A stream that drops the item of the inner stream.
    struct MapUnit<S>(Pin<Box<S>>);

    impl<S: Stream> Stream for MapUnit<S> {
        type Item = ();

        fn poll_next(
            mut self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> Poll<Option<()>> {
            self.0.as_mut().poll_next(cx).map(|item| item.map(|_| ()))
        }
    }

    impl PortalSource {
        /// Connects to the portal and subscribes to the change signals.
        ///
        /// # Errors
        ///
        /// Returns an error when the session bus or the portal is not
        /// available.
        pub async fn connect() -> Result<Self> {
            let settings = Settings::new()
                .await
                .context("the Settings portal is not available")?;
            let contrast_changes = key_changes::<Contrast>(&settings, CONTRAST_KEY).await?;
            let motion_changes =
                key_changes::<ReducedMotion>(&settings, REDUCED_MOTION_KEY).await?;
            Ok(Self {
                settings,
                contrast_changes,
                motion_changes,
            })
        }
    }

    /// Joins the two key reads. A portal can have one key and not the other,
    /// so a missing key gives no preference for that setting only. The read
    /// fails only when the portal gives neither key.
    fn combine(
        contrast: ashpd::Result<Contrast>,
        motion: ashpd::Result<ReducedMotion>,
    ) -> Result<AccessibilitySettings> {
        if let (Err(contrast_error), Err(_)) = (&contrast, &motion) {
            return Err(anyhow::anyhow!("{contrast_error}"))
                .context("the portal gives no contrast key and no reduced-motion key");
        }
        Ok(AccessibilitySettings {
            reduce_motion: motion.is_ok_and(|motion| motion == ReducedMotion::ReducedMotion),
            increase_contrast: contrast.is_ok_and(|contrast| contrast == Contrast::High),
            reduce_transparency: false,
        })
    }

    impl AccessibilitySource for PortalSource {
        async fn read(&self) -> Result<AccessibilitySettings> {
            combine(
                self.settings.contrast().await,
                self.settings.reduced_motion().await,
            )
        }

        async fn changed(&mut self) -> Result<()> {
            let contrast = &mut self.contrast_changes;
            let motion = &mut self.motion_changes;
            let ended = poll_fn(|cx| {
                let mut ended = true;
                for stream in [&mut *contrast, &mut *motion] {
                    match stream.as_mut().poll_next(cx) {
                        Poll::Ready(Some(())) => return Poll::Ready(false),
                        Poll::Ready(None) => {}
                        Poll::Pending => ended = false,
                    }
                }
                if ended {
                    Poll::Ready(true)
                } else {
                    Poll::Pending
                }
            })
            .await;
            if ended {
                anyhow::bail!("the portal change signals ended");
            }
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn missing() -> ashpd::Error {
            ashpd::Error::Portal(ashpd::PortalError::NotFound(
                "Requested setting not found".to_owned(),
            ))
        }

        /// A defect of 2026-11-19: a portal with no reduced-motion key also
        /// lost the contrast setting.
        #[test]
        fn a_missing_key_keeps_the_other_setting() {
            let settings = combine(Ok(Contrast::High), Err(missing())).unwrap();
            assert!(settings.increase_contrast);
            assert!(!settings.reduce_motion);
            let settings = combine(Err(missing()), Ok(ReducedMotion::ReducedMotion)).unwrap();
            assert!(settings.reduce_motion);
            assert!(!settings.increase_contrast);
        }

        #[test]
        fn the_read_fails_when_both_keys_are_missing() {
            assert!(combine(Err(missing()), Err(missing())).is_err());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::task::{Context, Poll, Waker};

    use super::*;

    /// A source that answers at once. `None` is a failed read.
    struct FakeSource {
        settings: Option<AccessibilitySettings>,
    }

    impl AccessibilitySource for FakeSource {
        async fn read(&self) -> Result<AccessibilitySettings> {
            self.settings
                .ok_or_else(|| anyhow::anyhow!("the fake source fails"))
        }

        async fn changed(&mut self) -> Result<()> {
            Ok(())
        }
    }

    /// Runs a future that has no real wait.
    fn ready<T>(future: impl Future<Output = T>) -> T {
        let mut future = std::pin::pin!(future);
        let mut cx = Context::from_waker(Waker::noop());
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("the fake source must not wait"),
        }
    }

    #[test]
    fn m1_each_setting_reaches_the_environment() {
        let source = FakeSource {
            settings: Some(AccessibilitySettings {
                reduce_motion: true,
                increase_contrast: true,
                reduce_transparency: true,
            }),
        };
        let settings = ready(read_or_default(&source));
        assert!(settings.reduce_motion);
        assert!(settings.increase_contrast);
        assert!(settings.reduce_transparency);
    }

    #[test]
    fn m6_a_failed_read_gives_false_for_each_setting() {
        let source = FakeSource { settings: None };
        assert_eq!(
            ready(read_or_default(&source)),
            AccessibilitySettings::default()
        );
    }
}
