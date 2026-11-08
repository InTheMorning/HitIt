//! The accent glow at the top of the sidebar (musicindex ADR 0056 R-SIDE-1).
//!
//! The layer is absolute, full width, and `SidebarGlow::LENGTH_PX` high. It has
//! no child and no handler, so the content of the sidebar draws over it and
//! clicks pass to that content. The caller places it first in a fixed
//! (`relative`) sidebar container, outside the scroll area, so it stays at
//! the top of the panel when the content scrolls.

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, App, Div};

use crate::ui::style::color;
use crate::ui::tokens::SidebarGlow;

/// Builds the glow layer for the active theme profile and UI scale.
#[must_use]
pub fn sidebar_glow_layer(cx: &App) -> Div {
    div()
        .absolute()
        .top_0()
        .left_0()
        .w_full()
        .h(SidebarGlow::length(cx))
        .bg(SidebarGlow::background(color::sidebar_glow_top()))
}
