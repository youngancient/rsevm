// src/bin/debug_tui.rs

use alloy_primitives::{Address, U256};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use evm::{evm::EVM, helpers::get_supported_opcode_name};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span},
    widgets::{Block, Borders,Clear, List, ListItem, Paragraph, ListState},
};
use std::{io, time::Duration};


// Users can edit this
fn user_input() -> (Vec<u8>, u64) {
    let program = vec![
        0x60, 0x69,
        0x60, 0x01, 
        0x55,
        ];
    let desired_gas = 1000;
    (program, desired_gas)
}

fn main() -> Result<(), anyhow::Error> {
    // PUSH1 10, PUSH1 20, ADD
    let (program, desired_gas) = user_input();

    // 1. Setup EVM passing in the program and desired_gas
    let mut evm = EVM::new(Address::ZERO, program, desired_gas, U256::ZERO, vec![]);

    // 2. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Run the UI Loop
    let res = run_app(&mut terminal, &mut evm, desired_gas);

    // 4. Cleanup Terminal (Crucial! Otherwise your terminal stays messed up)
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}


fn run_app<B: Backend>(terminal: &mut Terminal<B>, evm: &mut EVM, initial_gas: u64) -> io::Result<()> {
    // Track if an error happened
    let mut last_error: Option<String> = None;
    
    // Track the Scroll Position of the Bytecode
    let mut code_state = ListState::default();

    // Track last executed command
    let mut last_action_text: String = "Ready".to_string();
    loop {
        // AUTO-SCROLL LOGIC: Always select the current PC
        // This forces the UI to scroll down as the PC increases
        code_state.select(Some(evm.pc));

        terminal.draw(|f| {
            // ... [Keep your existing Layout definitions: main_chunks, top_row, bottom_row] ...
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.area());

            let top_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(33), Constraint::Percentage(34), Constraint::Percentage(33)].as_ref())
                .split(main_chunks[0]);

            let bottom_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(main_chunks[1]);

            // --- WIDGET 1: BYTECODE (Stateful!) ---
            let code_items: Vec<ListItem> = evm.program.iter().enumerate().map(|(i, &byte)| {
                // HEX Index + HEX Value + Opcode Name
                // Example: 0000: 60 [PUSH1]
                let op_name = get_supported_opcode_name(byte);
                let content = format!("{:04x}: {:02x}  [{}]", i, byte, op_name);

                let style = if i == evm.pc { 
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray).add_modifier(Modifier::BOLD) 
                } else { 
                    Style::default().fg(Color::Gray) 
                };
                ListItem::new(Span::styled(content, style))
            }).collect();
            
            let code_list = List::new(code_items)
                .block(Block::default().borders(Borders::ALL).title(" Bytecode "))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));
            
            // KEY CHANGE: render_stateful_widget instead of render_widget
            f.render_stateful_widget(code_list, top_row[0], &mut code_state);

            // --- WIDGET 2: MEMORY (Refined Width) ---
            // Changed from 32 to 16 bytes per row to fit the screen better
            let memory_items: Vec<ListItem> = evm.memory.memory
                .chunks(16) 
                .enumerate()
                .map(|(i, chunk)| {
                    // Create the hex part: "01 02 A3 ..."
                    let hex_val = chunk.iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<String>>()
                        .join(" ");
                    
                    // Optional: Add ASCII representation on the right?
                    // For now, let's just keep it clean with Offset + Hex
                    ListItem::new(format!("{:04x}: {}", i * 16, hex_val))
                })
                .collect();
            f.render_widget(List::new(memory_items).block(Block::default().borders(Borders::ALL).title(" Memory (16b) ")), top_row[1]);

             let storage_items: Vec<ListItem> = evm.storage.storage.iter().map(|(key, value)| {
                ListItem::new(format!("S[{:#x}]:\n{:#x}", key, value))
            }).collect();
            f.render_widget(List::new(storage_items).block(Block::default().borders(Borders::ALL).title(" Storage ")), top_row[2]);

            let stack_items: Vec<ListItem> = evm.stack.items.iter().enumerate().rev().map(|(i, val)| ListItem::new(format!("[{}] {:#x}", i, val))).collect();
            f.render_widget(List::new(stack_items).block(Block::default().borders(Borders::ALL).title(" Stack ")), bottom_row[0]);

            let status_block = Block::default().borders(Borders::ALL).title(" Status ");
            
            if let Some(err_msg) = &last_error {
                let error_text = format!("CRITICAL ERROR:\n{}\n\nPress 'q' to Quit", err_msg);
                let paragraph = Paragraph::new(error_text)
                    .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)) 
                    .block(status_block.border_style(Style::default().fg(Color::Red))); 
                f.render_widget(paragraph, bottom_row[1]);
            } else {
                // Used saturating_sub to prevent crash on gas underflow
                let gas_used = initial_gas.saturating_sub(evm.gas);
                // the PC has 2 reps : Hex and Decimal
                // Hex is the best and most natural especially cos of JUMP ops
                // decimal for those not so familiar with Hex
                let status_text = format!(
                    "PC: {:#04x} ({})\nGas Used: {}\nGas Left: {}\n\nLast Op: {}\n\n[ Controls ]\n'n' : Next Step\n'q' : Quit", 
                    evm.pc, evm.pc, gas_used, evm.gas, last_action_text
                );
                f.render_widget(Paragraph::new(status_text).block(status_block), bottom_row[1]);
            }
            
            // Popup logic
            if let Some(err_msg) = &last_error {
                let area = centered_rect(60, 30, f.area());
                let popup = Paragraph::new(format!("Execution Halted!\n\nReason: {}", err_msg))
                    .style(Style::default().bg(Color::Red).fg(Color::White))
                    .block(Block::default().borders(Borders::ALL).title(" ERROR "));
                f.render_widget(Clear, area);
                f.render_widget(popup, area);
            }
        })?;

        // ... [Keep Input Handling] ...
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('n') => {
                        if last_error.is_none() {
                            // PEEK
                            if evm.pc < evm.program.len() {
                                let op = evm.program[evm.pc];
                                let op_name = get_supported_opcode_name(op);
                                last_action_text = format!("Executed {} ({:#02x})", op_name, op);
                            } else {
                                last_action_text = "End of Code".to_string();
                            }

                             match evm.step() {
                                Ok(cont) => {
                                    if !cont { last_error = Some("Program Finished (STOP)".to_string()); }
                                }
                                Err(e) => { last_error = Some(format!("{:?}", e)); }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
// Helper to center the popup (Standard Ratatui boilerplate)
fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}