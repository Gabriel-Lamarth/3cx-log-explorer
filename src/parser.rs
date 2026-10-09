use crate::model::{LogEvent, LogLevel};
use std::io::{self, BufRead};

/// Parses a complete string, useful for small fixtures and unit tests.
pub fn parse_text(text: &str) -> Vec<LogEvent> {
    let mut events = Vec::new();
    // A string-backed reader cannot fail during reading.
    parse_reader(io::Cursor::new(text.as_bytes()), |event| events.push(event))
        .expect("reading from an in-memory string cannot fail");
    events
}

/// Stream logical events from a reader. A new timestamped record closes the
/// preceding event; non-record lines are attached to the current event as
/// continuations (e.g. stack traces or multi-line HTTP response bodies).
pub fn parse_reader<R, F>(reader: R, mut on_event: F) -> io::Result<()>
where
    R: BufRead,
    F: FnMut(LogEvent),
{
    let mut pending: Option<LogEvent> = None;
    let mut buffer = String::new();
    let mut line_number = 0usize;
    let mut reader = reader;

    loop {
        buffer.clear();
        let bytes = reader.read_line(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        line_number += 1;
        while buffer.ends_with('\n') || buffer.ends_with('\r') {
            buffer.pop();
        }

        if let Some(event) = parse_record(&buffer, line_number) {
            if let Some(previous) = pending.replace(event) {
                on_event(previous);
            }
        } else if let Some(current) = pending.as_mut() {
            current.raw.push('\n');
            current.raw.push_str(&buffer);
            current.message.push('\n');
            current.message.push_str(&buffer);
            current.end_line = line_number;
        }
    }

    if let Some(event) = pending {
        on_event(event);
    }
    Ok(())
}

fn parse_record(line: &str, line_number: usize) -> Option<LogEvent> {
    let mut fields = line.splitn(4, '|');
    let timestamp = fields.next()?.trim();
    let thread_id = fields.next()?.trim();
    let level_text = fields.next()?.trim();
    let rest = fields.next()?;

    if !looks_like_timestamp(timestamp) || thread_id.is_empty() || level_text.is_empty() {
        return None;
    }

    let rest = rest.trim_start();
    let (source, message) = if rest.starts_with('[') {
        if let Some(close) = rest.find(']') {
            let source = rest[1..close].trim().to_string();
            let message = rest[close + 1..].trim_start().to_string();
            (Some(source), message)
        } else {
            (None, rest.to_string())
        }
    } else {
        (None, rest.to_string())
    };

    Some(LogEvent {
        timestamp: timestamp.to_string(),
        thread_id: thread_id.to_string(),
        level: LogLevel::parse(level_text),
        source,
        flow_id: extract_flow_id(&message),
        call_id: extract_call_id(&message),
        component: extract_component(&message),
        message,
        raw: line.to_string(),
        start_line: line_number,
        end_line: line_number,
    })
}

fn looks_like_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 23 {
        return false;
    }
    // Expected form: YYYY-MM-DD HH:MM:SS.mmm
    [4usize, 7].iter().all(|&i| bytes[i] == b'-')
        && bytes[10] == b' '
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'.'
        && bytes.iter().enumerate().all(|(i, byte)| {
            matches!(i, 4 | 7 | 10 | 13 | 16 | 19)
                || byte.is_ascii_digit()
        })
}

fn extract_call_id(message: &str) -> Option<String> {
    for marker in ["CallID ", "CallId ", "CallID=", "CallId="] {
        if let Some(index) = message.find(marker) {
            let value = message[index + marker.len()..]
                .trim_start()
                .split(|ch: char| ch.is_whitespace() || ch == ',' || ch == ';')
                .next()
                .unwrap_or_default();
            if !value.is_empty() {
                return Some(value.trim_matches(&['\'', '"'][..]).to_string());
            }
        }
    }
    None
}

fn extract_flow_id(message: &str) -> Option<String> {
    let marker = ".Callflow.";
    let marker_index = message.find(marker)?;
    let start = message[..marker_index]
        .rfind(|ch: char| ch.is_whitespace() || ch == '[' || ch == ']')
        .map(|index| index + 1)
        .unwrap_or(0);
    let after_marker = marker_index + marker.len();
    let suffix = &message[after_marker..];
    let flow_number_len = suffix.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if flow_number_len == 0 {
        return None;
    }
    Some(format!("{}.Callflow.{}", &message[start..marker_index], &suffix[..flow_number_len]))
}

fn extract_component(message: &str) -> Option<String> {
    // Most CFD messages identify the active component as [C:x.y]-name.
    if let Some(marker) = message.find("[C:") {
        if let Some(close) = message[marker..].find("]-") {
            let after = marker + close + 2;
            let tail = &message[after..];
            let name = tail.split(" - ").next().unwrap_or_default().trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    // Fallback for records where only the operation phrase is present.
    for marker in ["Start executing component '", "Start executing branch '", "Evaluating condition for branch '"] {
        if let Some(index) = message.find(marker) {
            let tail = &message[index + marker.len()..];
            if let Some(end) = tail.find('\'') {
                let name = &tail[..end];
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}
