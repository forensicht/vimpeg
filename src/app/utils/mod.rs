use relm4::gtk::{self, glib, prelude::EditableExt};
use std::time::Duration;

pub fn on_format_timestamp(entry: &gtk::Entry) {
    let text = entry.text().to_string();
    if text.is_empty() {
        return;
    }
    let formatted_text = format_timestamp(&text);

    if text != formatted_text {
        let entry = entry.clone();
        glib::timeout_add_local_once(Duration::from_millis(100), move || {
            entry.set_text(&formatted_text);
            entry.set_position(text.len() as i32 + 1);
        });
    }
}

fn format_timestamp(input: &str) -> String {
    let mut cleaned = input.chars().filter(|c| c.is_numeric()).collect::<String>();
    if cleaned.len() > 8 {
        cleaned.truncate(8);
    }

    if cleaned.len() == 6 {
        format!(
            "{:02}:{:02}:{:02}",
            cleaned.get(0..2).unwrap_or("00"),
            cleaned.get(2..4).unwrap_or("00"),
            cleaned.get(4..6).unwrap_or("00"),
        )
    } else if cleaned.len() == 5 {
        format!(
            "{:02}:{:02}:{:01}",
            cleaned.get(0..2).unwrap_or("00"),
            cleaned.get(2..4).unwrap_or("00"),
            cleaned.get(4..5).unwrap_or("0"),
        )
    } else if cleaned.len() == 4 {
        format!(
            "{:02}:{:02}",
            cleaned.get(0..2).unwrap_or("00"),
            cleaned.get(2..4).unwrap_or("00"),
        )
    } else if cleaned.len() == 3 {
        format!(
            "{:02}:{:01}",
            cleaned.get(0..2).unwrap_or("00"),
            cleaned.get(2..3).unwrap_or("0")
        )
    } else if cleaned.len() == 2 {
        format!("{:02}", cleaned.get(0..2).unwrap_or("00"))
    } else {
        cleaned
    }
}
