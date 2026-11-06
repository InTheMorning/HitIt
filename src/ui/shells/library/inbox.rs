//! The Updates and Setup sections of the Inbox (ADR 0089 Decision 3).
//!
//! The Tracks section is the Library track list, which `content_list.rs`
//! renders. This shell renders the other two sections. Each item shows the
//! sentence and the button of its view model. Each button runs the action
//! that already exists: the feed update, the tag update popup, or the
//! capability repair of Settings (ADR 0066). The shell decides no
//! availability.

#![warn(clippy::pedantic)]

use gpui::{
    div, prelude::*, AnyElement, ClickEvent, Context, FontWeight, IntoElement, SharedString,
};

use crate::library::{LibraryApp, LibraryAppEvent};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::{Button as UiButton, Label, SectionHeader};
use crate::ui::tokens::{FontSize, SemanticColor, Size, Spacing};
use crate::view_models::inbox::{
    InboxPageVm, InboxSection, InboxSetupItem, InboxUpdateItem, InboxUpdateKind,
};
use crate::view_models::startup::StartupAvailability;

/// Renders the Updates and Setup sections of `page`, or `None` when neither
/// holds an item. The Tracks section is not part of this element.
pub(crate) fn render_inbox_sections(
    page: &InboxPageVm,
    cx: &mut Context<LibraryApp>,
) -> Option<AnyElement> {
    let mut blocks: Vec<AnyElement> = Vec::new();
    for section in page.sections() {
        match section.section {
            InboxSection::Tracks => {}
            InboxSection::Updates => {
                blocks.push(render_updates_section(section.title, page.updates(), cx));
            }
            InboxSection::Setup => {
                blocks.push(render_setup_section(section.title, page.setup(), cx));
            }
        }
    }
    if blocks.is_empty() {
        return None;
    }
    Some(
        div()
            .id("inbox-sections")
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w_full()
            .min_w_0()
            .max_h(Size::ColumnRegular.scaled(cx))
            .overflow_y_scroll()
            .px(Spacing::MD.scaled(cx))
            .pb(Spacing::SM.scaled(cx))
            .gap(Spacing::SM.scaled(cx))
            .children(blocks)
            .into_any_element(),
    )
}

fn render_updates_section(
    title: &'static str,
    items: &[InboxUpdateItem],
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(Spacing::XS.scaled(cx))
        .child(SectionHeader::new(title))
        .children(items.iter().map(|item| {
            let kind = item.kind;
            let button_id = match kind {
                InboxUpdateKind::FeedUpdates => "inbox-apply-feed-updates",
                InboxUpdateKind::TagUpdates => "inbox-tag-update",
            };
            div()
                .flex()
                .flex_col()
                .gap(Spacing::XXS.scaled(cx))
                .child(
                    Label::new(item.text.clone())
                        .size(FontSize::Caption)
                        .weight(FontWeight::MEDIUM),
                )
                .child(
                    UiButton::styled(button_id, ControlStyle::Secondary)
                        .label(item.button_label.clone())
                        .a11y_label(item.a11y_label.clone())
                        .disabled(item.disabled)
                        .on_click(cx.listener(
                            move |this, _: &ClickEvent, window, cx| match kind {
                                InboxUpdateKind::FeedUpdates => this.apply_all_feed_updates(cx),
                                InboxUpdateKind::TagUpdates => {
                                    this.open_tag_update_popup(window, cx);
                                }
                            },
                        )),
                )
                .into_any_element()
        }))
        .into_any_element()
}

fn render_setup_section(
    title: &'static str,
    items: &[InboxSetupItem],
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(Spacing::XS.scaled(cx))
        .child(SectionHeader::new(title))
        .children(items.iter().enumerate().map(|(index, item)| {
            let action = item.action.clone();
            let disabled = action.availability != StartupAvailability::Available;
            let button_id = SharedString::from(format!("inbox-setup-{index}"));
            div()
                .flex()
                .flex_col()
                .gap(Spacing::XXS.scaled(cx))
                .child(
                    Label::new(item.text.clone())
                        .size(FontSize::Caption)
                        .color(SemanticColor::SecondaryLabel),
                )
                .child(
                    UiButton::styled(button_id, ControlStyle::Secondary)
                        .label(action.label)
                        .a11y_label(action.a11y_label.clone())
                        .disabled(disabled)
                        .on_click(cx.listener(move |_, _: &ClickEvent, _window, cx| {
                            cx.emit(LibraryAppEvent::InboxSetupAction(action.action));
                        })),
                )
                .into_any_element()
        }))
        .into_any_element()
}
