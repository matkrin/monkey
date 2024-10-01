use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::Result;
use line_editor::parse_key_event;
use line_editor::KeyCode;
use line_editor::KeyModifiers;
use monkey::Lexer;
use monkey::Node;
use monkey::Parser;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console::clear;
use xterm_js_rs::addons::fit::FitAddon;
use xterm_js_rs::BellStyle;
use xterm_js_rs::{Terminal, TerminalOptions, Theme};

mod line_editor;
use crate::line_editor::LineEditor;
use monkey::Environment;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn test() -> String {
    "hello from test".to_string()
}

fn test2() -> String {
    "hello from test2".to_string()
}

const PROMPT: &str = "monkey❯ ";

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let mut commands: HashMap<String, fn() -> String> = HashMap::new();
    commands.insert("test".to_string(), test);
    commands.insert("test2".to_string(), test2);

    let terminal: Terminal = Terminal::new(
        TerminalOptions::new()
            .with_cursor_blink(false)
            .with_cursor_width(10)
            .with_font_size(16)
            .with_draw_bold_text_in_bright_colors(true)
            .with_right_click_selects_word(true)
            .with_bell_style(BellStyle::Both)
            .with_theme(
                Theme::new()
                    .with_foreground("#98FB98")
                    .with_background("#000000"),
            ),
    );

    let terminal_element = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("terminal")
        .unwrap();

    terminal.open(terminal_element.dyn_into()?);

    let term: Terminal = terminal.clone().dyn_into()?;
    let mut line_editor = LineEditor::new(term, PROMPT);
    line_editor.prompt();
    let environment = Rc::new(RefCell::new(Environment::new()));


    let callback_ondata = Closure::wrap(Box::new(move |e: String| {
        let input_bytes = e.as_bytes();
        let key = parse_key_event(input_bytes).unwrap();
        log!("{}", e);
        match key.modifiers {
            KeyModifiers::None => match key.code {
                KeyCode::Char(c) => {
                    line_editor.insert_char(c);
                }
                KeyCode::Enter => {
                    let lexer = Lexer::new(line_editor.buffer());
                    let mut parser = Parser::new(lexer);
                    let (program, errors) = parser.parse_program();

                    for error in errors {
                        line_editor.write_line(&format!("{}", error));
                    }

                    match monkey::eval(Node::Program(program), &environment) {
                        Ok(evaluated) => line_editor.enter(&format!("{}", evaluated)),
                        Err(e) => line_editor.enter(&format!("{}", e)),
                    };
                }
                KeyCode::Backspace => {
                    line_editor.delete_left();
                }
                KeyCode::Delete => {
                    line_editor.delete_right();
                }
                KeyCode::Left => {
                    line_editor.move_left(1);
                }
                KeyCode::Right => {
                    line_editor.move_right(1);
                }
                KeyCode::Home => {
                    line_editor.move_start();
                }
                KeyCode::End => {
                    line_editor.move_end();
                }
                _ => {}
            },
            KeyModifiers::Control => match key.code {
                KeyCode::Char(c) => match c {
                    'l' => line_editor.clear_screen(),
                    'a' => line_editor.move_start(),
                    'e' => line_editor.move_end(),
                    'b' => line_editor.move_left(1),
                    'f' => line_editor.move_right(1),
                    'd' => line_editor.delete_right(),
                    'h' => line_editor.delete_left(),
                    'u' => line_editor.delete_line(),
                    'k' => line_editor.delete_from_cursor(),
                    // 'c' => line_buffer.term.write(&format!("\x1b[{}D", 3)),
                    _ => {}
                },
                _ => {}
            },
            KeyModifiers::Alt => match key.code {
                KeyCode::Char(c) => match c {
                    'b' => line_editor.word_left(),
                    'f' => line_editor.word_right(),
                    _ => {}
                },
                KeyCode::Left => {
                    line_editor.word_left();
                }
                KeyCode::Right => {
                    line_editor.word_right();
                }
                _ => {}
            },
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);

    terminal.on_data(callback_ondata.as_ref().unchecked_ref());

    callback_ondata.forget();

    let addon = FitAddon::new();
    terminal.load_addon(addon.clone().dyn_into::<FitAddon>()?.into());
    addon.fit();
    terminal.focus();

    Ok(())
}
