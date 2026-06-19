use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, List, ListDirection, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, Mode};
use crate::theme;

pub fn render(f: &mut Frame, app: &App) {
    let th = &theme::THEMES[app.theme_index];
    let area = f.area();

    let main_layout = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(3),
    ]);
    let [main_area, footer_area] = main_layout.areas(area);

    let content_layout = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ]);
    let [list_area, details_area] = content_layout.areas(main_area);

    render_host_list(f, app, th, list_area);
    render_details(f, app, th, details_area);
    render_footer(f, th, footer_area);

    match &app.mode {
        Mode::Adding => render_form(f, app, th, area, "Add Host"),
        Mode::Editing => render_form(f, app, th, area, "Edit Host"),
        Mode::Deleting(idx) => render_delete_confirm(f, app, th, area, *idx),
        Mode::ThemeSelect => render_theme_selector(f, app, th, area),
        Mode::Message(msg) => render_message(f, th, area, msg),
        _ => {}
    }
}

fn render_host_list(f: &mut Frame, app: &App, th: &theme::Theme, area: Rect) {
    let sort_label = match app.sort_field {
        crate::app::SortField::Name => if app.sort_reverse { "name\u{2193}" } else { "name\u{2191}" },
        crate::app::SortField::LastSeen => if app.sort_reverse { "seen\u{2193}" } else { "seen\u{2191}" },
    };
    let subtitle = if !app.search.is_empty() {
        format!(" filter: {} | {} ", app.search, sort_label)
    } else {
        format!(" {} hosts | {} ", app.filtered.len(), sort_label)
    };

    let block = Block::default()
        .title(" sshmenu ")
        .title_bottom(subtitle)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(th.border));

    if app.filtered.is_empty() {
        let msg = if app.search.is_empty() {
            vec![
                Line::from(""),
                Line::from("  No hosts configured").centered(),
                Line::from("  Press 'a' to add one").centered(),
                Line::from(""),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from("  No hosts match your search").centered(),
                Line::from(""),
            ]
        };
        let p = Paragraph::new(Text::from(msg)).block(block);
        f.render_widget(p, area);
        return;
    }

    let sep = Style::new().fg(th.border);

    let header = Row::new(vec![
        Cell::from(Span::styled(" Name", Style::new().fg(th.header_fg).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled(" \u{2502} SSH", Style::new().fg(th.header_fg).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled(" \u{2502} Tags", Style::new().fg(th.header_fg).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled(" \u{2502} Last", Style::new().fg(th.header_fg).add_modifier(Modifier::BOLD))),
    ])
    .style(Style::new().bg(th.header_bg))
    .height(1);

    let mut rows: Vec<Row> = Vec::with_capacity(app.filtered.len());
    for (list_i, &host_idx) in app.filtered.iter().enumerate() {
        let host = &app.hosts[host_idx];
        let is_selected = list_i == app.selected;

        let pin = if host.pinned { "\u{1f4cc}" } else { "" };

        let tags = if host.tags.is_empty() {
            String::new()
        } else if host.tags.len() > 2 {
            format!("[{}, {} +{}]", host.tags[0], host.tags[1], host.tags.len() - 2)
        } else {
            format!("[{}]", host.tags.join(", "))
        };

        let last = if let Some(ts) = host.last_seen {
            let elapsed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts as u64))
                .unwrap_or_default();
            let mins = elapsed.as_secs() / 60;
            if mins < 1 { "now".to_string() }
            else if mins < 60 { format!("{}m", mins) }
            else if mins < 1440 { format!("{}h", mins / 60) }
            else { format!("{}d", mins / 1440) }
        } else {
            String::new()
        };

        let ssh = format!("{}@{}:{}", host.user, host.host, host.port);

        let row_style = if is_selected {
            Style::new().bg(th.selected_bg).fg(th.selected_fg)
        } else if list_i % 2 == 0 {
            Style::new().bg(th.row_even)
        } else {
            Style::new().bg(th.row_odd_bg).fg(th.row_odd_fg)
        };

        rows.push(Row::new(vec![
            Cell::from(Line::from(vec![
                Span::raw(" "),
                Span::styled(pin, Style::new().fg(th.tag_fg)),
                Span::styled(
                    format!(" {}", host.name),
                    Style::new().fg(th.name_fg).add_modifier(Modifier::BOLD),
                ),
            ])),
            Cell::from(Line::from(vec![
                Span::styled(" \u{2502} ", sep),
                Span::styled(ssh, Style::new().fg(th.ssh_fg)),
            ])),
            Cell::from(Line::from(vec![
                Span::styled(" \u{2502} ", sep),
                Span::styled(tags, Style::new().fg(th.tag_fg)),
            ])),
            Cell::from(Line::from(vec![
                Span::styled(" \u{2502} ", sep),
                Span::styled(format!("{:>8}", last), Style::new().fg(th.footer_fg)),
            ])),
        ])
        .style(row_style)
        .height(1));
    }

    let widths = [
        Constraint::Length(22),
        Constraint::Length(26),
        Constraint::Min(8),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::new().bg(th.selected_bg));

    f.render_widget(table, area);
}

fn render_details(f: &mut Frame, app: &App, th: &theme::Theme, area: Rect) {
    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::new().fg(th.border));

    let host = app.selected_host();

    let lines = if let Some(host) = host {
        let pin = if host.pinned { "pinned \u{1f4cc}" } else { "no" };

        let last_seen = if let Some(ts) = host.last_seen {
            let elapsed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts as u64))
                .unwrap_or_default();
            let secs = elapsed.as_secs();
            if secs < 60 { "just now".to_string() }
            else if secs < 3600 { format!("{}m ago", secs / 60) }
            else if secs < 86400 { format!("{}h ago", secs / 3600) }
            else { format!("{}d ago", secs / 86400) }
        } else {
            "never".to_string()
        };

        let tags_str = if host.tags.is_empty() {
            "-".to_string()
        } else {
            host.tags.join(", ")
        };

        let ssh_cmd = format!("ssh -p {} {}@{}", host.port, host.user, host.host);

        vec![
            Line::from(vec![
                Span::styled(&host.name, Style::new().fg(th.name_fg).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Basic", Style::new().fg(th.header_fg).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(format!("  Host:     {}", host.host)),
            Line::from(format!("  User:     {}", host.user)),
            Line::from(format!("  Port:     {}", host.port)),
            Line::from(format!("  Tags:     {}", tags_str)),
            Line::from(format!("  Pinned:   {}", pin)),
            Line::from(format!("  Last SSH: {}", last_seen)),
            Line::from(format!("  Connects: {}", host.ssh_count)),
            Line::from(""),
            Line::from(vec![
                Span::styled("Command", Style::new().fg(th.header_fg).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("  {}", ssh_cmd), Style::new().fg(th.ssh_fg)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Keys", Style::new().fg(th.header_fg).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  [a] add     [e] edit    [d] del"),
            Line::from("  [g] ping    [p] pin     [c] copy"),
            Line::from("  [s] sort    [/] search  [t] theme"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Enter", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
                Span::raw(" to connect  "),
                Span::styled("[q]", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
                Span::raw(" to quit"),
            ]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from("  No host selected").centered(),
            Line::from(""),
            Line::from("  Press 'a' to add one").centered(),
        ]
    };

    let p = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn render_footer(f: &mut Frame, th: &theme::Theme, area: Rect) {
    let text = vec![
        Line::from(vec![
            Span::styled(" [a] ", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
            Span::raw("add "),
            Span::styled("[e] ", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
            Span::raw("edit "),
            Span::styled("[d] ", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
            Span::raw("del "),
            Span::styled("[g] ", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
            Span::raw("ping "),
            Span::styled("[p] ", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
            Span::raw("pin "),
            Span::styled("[c] ", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
            Span::raw("copy "),
            Span::styled("[s] ", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
            Span::raw("sort "),
            Span::styled("[/]", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
            Span::raw(" search "),
            Span::styled("[t]", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
            Span::raw(" theme "),
            Span::styled("[q]", Style::new().fg(th.footer_key).add_modifier(Modifier::BOLD)),
            Span::raw(" quit"),
        ]),
    ];

    let p = Paragraph::new(Text::from(text))
        .block(Block::default().borders(Borders::TOP))
        .style(Style::new().fg(th.footer_fg));
    f.render_widget(p, area);
}

fn render_theme_selector(f: &mut Frame, app: &App, th: &theme::Theme, area: Rect) {
    let popup = centered_rect(30, 40, area);

    let mut items: Vec<ListItem> = theme::THEMES.iter().enumerate().map(|(i, t)| {
        let preview = &theme::THEMES[i];
        let style = if i == app.theme_index {
            Style::new().bg(preview.selected_bg).fg(preview.selected_fg)
        } else {
            Style::new().fg(th.label_fg)
        };
        let indicator = if i == app.theme_index { " \u{2713} " } else { "   " };
        ListItem::new(Line::from(vec![
            Span::styled(indicator, Style::new().fg(preview.border)),
            Span::styled(t.name, Style::new().fg(preview.border)),
        ])).style(style)
    }).collect();

    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(vec![
        Span::styled(" [Enter]", Style::new().fg(th.footer_key)),
        Span::raw(" select  "),
        Span::styled("[Esc]", Style::new().fg(th.footer_key)),
        Span::raw(" cancel"),
    ])).style(Style::new().fg(th.footer_fg)));

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Theme ")
                .borders(Borders::ALL)
                .border_style(Style::new().fg(th.border)),
        )
        .direction(ListDirection::TopToBottom);

    f.render_widget(Clear, popup);
    f.render_widget(list, popup);
}

fn render_form(f: &mut Frame, app: &App, th: &theme::Theme, area: Rect, title: &str) {
    let popup = centered_rect(50, 40, area);

    let form_labels = ["Name", "Host", "Port", "User", "Tags (comma sep)"];
    let form_values = [
        app.form.name.as_str(),
        app.form.host_val.as_str(),
        app.form.port.as_str(),
        app.form.user.as_str(),
        app.form.tags.as_str(),
    ];

    let mut lines = Vec::new();
    for (i, label) in form_labels.iter().enumerate() {
        let is_focused = i == app.form.focus;
        let prefix = if is_focused { " > " } else { "   " };
        let label_style = if is_focused {
            Style::new().fg(th.focused_label_fg).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(th.label_fg)
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{}{}: ", prefix, label), label_style),
            Span::styled(form_values[i].to_string(), Style::new().fg(th.input_fg)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" [Tab]", Style::new().fg(th.footer_key)),
        Span::raw(" next  "),
        Span::styled("[Shift+Tab]", Style::new().fg(th.footer_key)),
        Span::raw(" prev  "),
        Span::styled("[Enter]", Style::new().fg(th.footer_key)),
        Span::raw(" save  "),
        Span::styled("[Esc]", Style::new().fg(th.footer_key)),
        Span::raw(" cancel"),
    ]));

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .style(Style::new().fg(th.border));

    let p = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(Clear, popup);
    f.render_widget(p, popup);
}

fn render_delete_confirm(f: &mut Frame, app: &App, th: &theme::Theme, area: Rect, idx: usize) {
    let host = &app.hosts[idx];
    let popup = centered_rect(40, 20, area);

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Delete "),
            Span::styled(host.name.clone(), Style::new().fg(th.delete_fg).add_modifier(Modifier::BOLD)),
            Span::raw("?"),
        ]).centered(),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [y] ", Style::new().fg(th.delete_fg)),
            Span::raw("yes  "),
            Span::styled("[n]", Style::new().fg(th.delete_fg)),
            Span::raw(" no"),
        ]).centered(),
    ];

    let block = Block::default()
        .title(" Delete host ")
        .borders(Borders::ALL)
        .style(Style::new().fg(th.delete_fg));

    let p = Paragraph::new(Text::from(text))
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(Clear, popup);
    f.render_widget(p, popup);
}

fn render_message(f: &mut Frame, th: &theme::Theme, area: Rect, msg: &str) {
    let popup = centered_rect(40, 20, area);

    let text = vec![
        Line::from(""),
        Line::from(msg).centered(),
        Line::from(""),
        Line::from("Press any key to continue").centered(),
    ];

    let block = Block::default()
        .title(" Message ")
        .borders(Borders::ALL)
        .style(Style::new().fg(th.border));

    let p = Paragraph::new(Text::from(text))
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(Clear, popup);
    f.render_widget(p, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
