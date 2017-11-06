extern crate termion;
extern crate rusting;
extern crate async_command;

use rusting::{Rust, e};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor, terminal_size, async_stdin, style};

use std::io::Write;
use std::ascii::AsciiExt;

macro_rules! out {
	($fmt:expr) => ({
		let mut stdout = ::std::io::stdout().into_raw_mode().rust(e);
		write!(stdout, $fmt).rust();
		stdout.flush().rust(e);
	});
	($fmt:expr, $($arg:tt)*) => ({
		let mut stdout = ::std::io::stdout().into_raw_mode().rust(e);
		write!(stdout, $fmt, $($arg)*).rust(e);
		stdout.flush().rust(e);
	});
}

fn main() {
	let mut history = Vec::new();
	let (mut buffer, mut temp_buffer, mut sic_buffer) = (String::new(), String::new(), String::new());
	let (mut cursor_pos, mut history_pos) = (0, 0);
	let mut sic = async_command::AsyncCommand::new(&mut ::std::process::Command::new("sic").arg("-n").arg("magnusi"));

	sic.run();

	out!("{}{}{}", clear::All, cursor::Goto(1, 1), cursor::Hide);
	loop {
		out!("{}{}", clear::All, cursor::Goto(1,1));
		let (col, line) = terminal_size().rust(e);
		
		if let Some(c) = async_stdin().keys().next() {
			match c.rust(e) {
				Key::Ctrl('q') => break,
				Key::Char('\n')=> {
					history.push(buffer.clone());
					history_pos = history.len() - 1;
					sic.push(buffer.as_bytes());
					sic.push(b"\n");
					buffer.clear();
					cursor_pos = 0;
				},
				Key::Char(c) if c.is_ascii() => {
					buffer.insert(cursor_pos, c);
					cursor_pos +=1;
				},
				Key::Backspace => {
					let len = buffer.len();
					if cursor_pos == len && len != 0 {
						buffer.truncate(len-1);
						cursor_pos -= 1;
					} else if len != 0 && cursor_pos != 0 {
						buffer.remove(cursor_pos);
						cursor_pos -= 1;
					}
				},
				Key::Left  => if cursor_pos != 0 { cursor_pos -= 1; },
				Key::Right => if cursor_pos != buffer.len() { cursor_pos += 1; },
				Key::Up => if history_pos != 0 && history_pos != history.len() - 1 {
						buffer = history[history_pos].clone();
						history_pos -= 1;
						cursor_pos = buffer.len();
					} else if history_pos == history.len() -1 && history.len() != 1 {
						temp_buffer = buffer.clone();
						buffer = (&history[history_pos]).to_string();
						history_pos -= 1;
						cursor_pos = buffer.len();
					},
				Key::Down => {
					if history_pos == history.len() -1 { buffer = temp_buffer.clone(); }
					else { history_pos += 1; buffer = history[history_pos].clone(); }
					cursor_pos = buffer.len();
				},
				_  => (),
			}
		}

		for line in sic.packets() {
			sic_buffer.push_str(&(line + "\n\r"));
		}

		out!("{}{}\n\n", cursor::Goto(1, 1), &sic_buffer);
		out!("{}{}{}{}rusic 0.1{}\n\r", cursor::Goto(1, line-2), style::Bold, style::Invert, termion::color::Fg(termion::color::LightYellow), style::Reset);
		out!("{}{}", cursor::Goto(1, line-1), (0..col).fold::<String, _>(String::new(), |mut a, _| { a.push('_'); a}));
		out!("{}{}", cursor::Goto(1, line), &buffer);
		out!("{}{}", cursor::Show, cursor::Goto((cursor_pos as u16 + 1) % col, line));

		::std::thread::sleep(::std::time::Duration::from_millis(33));
	}
	out!("{}", cursor::Show);
}
