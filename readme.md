



Scrollbar<'_>

A widget to display a scrollbar

The following components of the scrollbar are customizable in symbol and style. Note the scrollbar is represented horizontally but it can also be set vertically (which is actually the default).

<--▮------->
^  ^   ^   ^
│  │   │   └ end
│  │   └──── track
│  └──────── thumb
└─────────── begin
Important
You must specify the [ScrollbarState::content_length] before rendering the Scrollbar, or else the Scrollbar will render blank.

Examples
use ratatui::{prelude::*, widgets::*};

let vertical_scroll = 0; // from app state

let items = vec![
    Line::from("Item 1"),
    Line::from("Item 2"),
    Line::from("Item 3"),
];
let paragraph = Paragraph::new(items.clone())
    .scroll((vertical_scroll as u16, 0))
    .block(Block::new().borders(Borders::RIGHT)); // to show a background for the scrollbar

let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
    .begin_symbol(Some("↑"))
    .end_symbol(Some("↓"));

let mut scrollbar_state = ScrollbarState::new(items.len()).position(vertical_scroll);

let area = frame.size();
// Note we render the paragraph
frame.render_widget(paragraph, area);
// and the scrollbar, those are separate widgets
frame.render_stateful_widget(
    scrollbar,
    area.inner(&Margin {
        // using an inner vertical margin of 1 unit makes the scrollbar inside the block
        vertical: 1,
        horizontal: 0,
    }),
    &mut scrollbar_state,
);