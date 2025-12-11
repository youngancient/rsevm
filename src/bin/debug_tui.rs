use alloy_primitives::{Address, U256, hex};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use evm::{evm::EVM, helpers::get_supported_opcode_name};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout,Position},
    style::{Color, Modifier, Style},
    text::{Span},
    widgets::{Block, Borders,Clear, List, ListItem, Paragraph, ListState},
};
use std::{io, time::Duration};

fn main() -> Result<(), anyhow::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // RUN SETUP WIZARD
    let setup_res = run_setup_wizard(&mut terminal);

    // Run the UI Loop
    if let Ok(Some((program, desired_gas))) = setup_res {
        let mut evm = EVM::new(Address::ZERO, program, desired_gas, U256::ZERO, vec![]);
        
        // Run the Main Debugger Loop
        let res = run_app(&mut terminal, &mut evm, desired_gas);
        if let Err(err) = res {
            println!("Runtime Error: {:?}", err);
        }
    }
    // Cleanup Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}


// --- SETUP WIZARD LOGIC ---

enum SetupStage {
    ModeSelect,
    SampleSelect,
    ManualInput,
    GasInput,
}

struct SampleProgram {
    name: &'static str,
    code: Vec<u8>,
}

fn run_setup_wizard<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<Option<(Vec<u8>, u64)>> {
    let mut stage = SetupStage::ModeSelect;
    
    // UI States
    let mut mode_index = 0; // 0: Samples, 1: Manual
    let mut sample_state = ListState::default();
    sample_state.select(Some(0));
    
    // Input Buffers
    let mut input_buffer = String::new();
    let mut selected_program: Vec<u8> = Vec::new();

    // Data: Samples
    let samples = vec![
        SampleProgram { 
            name: "Grand Tour (Math, Mem, Storage)", 
            code: vec![
                0x60, 0x0A, 0x60, 0x14, 0x01, 0x60, 0x00, 0x52, 0x60, 0x1E, 
                0x60, 0x01, 0x55, 0x60, 0x00, 0x51, 0x60, 0x01, 0x01, 0x60, 
                0x20, 0x52, 0x60, 0x01, 0x54, 0x60, 0x02, 0x02, 0x60, 0x02, 0x55, 0x00
            ] 
        },
        SampleProgram { 
            name: "Countdown Loop (Jumps)", 
            code: vec![
                0x60, 0x05, 0x5B, 0x80, 0x15, 0x60, 0x0F, 0x57, 0x60, 0x01, 
                0x90, 0x03, 0x60, 0x02, 0x56, 0x5B, 0x00 
            ] 
        },
        SampleProgram {
            name: "Simple Memory Interaction",
            code: vec![0x60, 0xFF, 0x60, 0x00, 0x52, 0x60, 0x00, 0x51]
        }
    ];

    loop {
        terminal.draw(|f| {
            // Background
            let size = f.area();
            let block = Block::default().title(" rsevm Setup ").borders(Borders::ALL);
            f.render_widget(block, size);

            let area = centered_rect(60, 40, size);

            match stage {
                SetupStage::ModeSelect => {
                    let title = Paragraph::new("Welcome to rsevm!\n\nSelect Input Mode:")
                        .style(Style::default().add_modifier(Modifier::BOLD))
                        .alignment(ratatui::layout::Alignment::Center);
                    
                    // Simple manual list rendering
                    let modes = vec![" Use Sample Program ", " Enter Bytecode Manually "];
                    let items: Vec<ListItem> = modes.iter().enumerate().map(|(i, m)| {
                        let style = if i == mode_index { 
                            Style::default().fg(Color::Black).bg(Color::Cyan) 
                        } else { 
                            Style::default() 
                        };
                        ListItem::new(Span::styled(*m, style))
                    }).collect();
                    
                    let list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title(" Mode Selection "));
                    
                    let chunks = Layout::default()
                        .constraints([Constraint::Length(4), Constraint::Min(0)].as_ref())
                        .split(area);
                    
                    f.render_widget(title, chunks[0]);
                    f.render_widget(list, chunks[1]);
                },

                SetupStage::SampleSelect => {
                    let items: Vec<ListItem> = samples.iter().map(|s| {
                        ListItem::new(s.name)
                    }).collect();
                    
                    let list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title(" Select Sample "))
                        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan));
                    
                    f.render_stateful_widget(list, area, &mut sample_state);
                },

                SetupStage::ManualInput => {
                    let instructions = Paragraph::new("Enter Bytecode (Hex string, e.g., '60ff01'):\n(Press Enter to Confirm)")
                        .alignment(ratatui::layout::Alignment::Center);
                    
                    let input = Paragraph::new(input_buffer.as_str())
                        .style(Style::default().fg(Color::Yellow))
                        .block(Block::default().borders(Borders::ALL).title(" Input "));

                    let chunks = Layout::default()
                        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                        .split(area);
                    
                    f.render_widget(instructions, chunks[0]);
                    f.render_widget(input, chunks[1]);

                    // Make cursor blink at end of input
                    f.set_cursor_position(
                        Position::new(chunks[1].x + input_buffer.len() as u16 + 1,
                        chunks[1].y + 1)
                    )
                },

                SetupStage::GasInput => {
                    let instructions = Paragraph::new("Enter Gas Limit (Default: 25000):")
                        .alignment(ratatui::layout::Alignment::Center);
                    
                    let input = Paragraph::new(input_buffer.as_str())
                        .style(Style::default().fg(Color::Green))
                        .block(Block::default().borders(Borders::ALL).title(" Gas "));

                    let chunks = Layout::default()
                        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                        .split(area);
                    
                    f.render_widget(instructions, chunks[0]);
                    f.render_widget(input, chunks[1]);
                     
                    f.set_cursor_position(
                        Position::new(chunks[1].x + input_buffer.len() as u16 + 1,
                        chunks[1].y + 1)
                    )
                }
            }
        })?;

        // Input Handling
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Esc {
                return Ok(None); // Quit
            }

            match stage {
                SetupStage::ModeSelect => {
                    match key.code {
                        KeyCode::Up => if mode_index > 0 { mode_index -= 1 },
                        KeyCode::Down => if mode_index < 1 { mode_index += 1 },
                        KeyCode::Enter => {
                            if mode_index == 0 { stage = SetupStage::SampleSelect; }
                            else { 
                                stage = SetupStage::ManualInput; 
                                input_buffer.clear(); 
                            }
                        }
                        _ => {}
                    }
                },
                SetupStage::SampleSelect => {
                    match key.code {
                        KeyCode::Up => {
                            let i = sample_state.selected().unwrap_or(0);
                            if i > 0 { sample_state.select(Some(i - 1)); }
                        },
                        KeyCode::Down => {
                            let i = sample_state.selected().unwrap_or(0);
                            if i < samples.len() - 1 { sample_state.select(Some(i + 1)); }
                        },
                        KeyCode::Enter => {
                            let i = sample_state.selected().unwrap_or(0);
                            selected_program = samples[i].code.clone();
                            stage = SetupStage::GasInput;
                            input_buffer = "25000".to_string(); // Default gas
                        },
                        _ => {}
                    }
                },
                SetupStage::ManualInput => {
                    match key.code {
                        KeyCode::Char(c) => {
                            if c.is_digit(16) || c == 'x' || c == 'X' { input_buffer.push(c); } // Only allow hex
                        },
                        KeyCode::Backspace => { input_buffer.pop(); },
                        KeyCode::Enter => {
                            if !input_buffer.is_empty() {
                                // remove unnecessary user parsed elements
                                let clean_input = input_buffer
                                .trim()
                                .replace(" ", "")
                                .replace("0x", "")
                                .replace("0X", "");
                                // Parse Hex
                                if let Ok(bytes) = hex::decode(&clean_input) {
                                    selected_program = bytes;
                                    stage = SetupStage::GasInput;
                                    input_buffer = "25000".to_string(); // Reset buffer for gas
                                } else {
                                    // Should ideally show error, but simple retry for now
                                    input_buffer.clear(); 
                                }
                            }
                        },
                        _ => {}
                    }
                },
                SetupStage::GasInput => {
                    match key.code {
                        KeyCode::Char(c) => {
                            if c.is_digit(10) { input_buffer.push(c); } // Only allow numbers
                        },
                        KeyCode::Backspace => { input_buffer.pop(); },
                        KeyCode::Enter => {
                            let gas: u64 = input_buffer.parse().unwrap_or(25000);
                            return Ok(Some((selected_program, gas)));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}


// MAIN APPLICATION LOGIC -> EVM SCREEN
fn run_app<B: Backend>(terminal: &mut Terminal<B>, evm: &mut EVM, initial_gas: u64) -> io::Result<()> {
    // Track if an error happened
    let mut stop_reason: Option<String> = None;
    
    // Track the Scroll Position of the Bytecode
    let mut code_state = ListState::default();

    // Track last executed command
    let mut last_action_text: String = "Ready".to_string();
    loop {
        // AUTO-SCROLL LOGIC: Always select the current PC
        // This forces the UI to scroll down as the PC increases
        code_state.select(Some(evm.pc));

        terminal.draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.area());

            let top_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(29), Constraint::Percentage(42), Constraint::Percentage(29)].as_ref())
                .split(main_chunks[0]);

            let bottom_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(main_chunks[1]);

            // --- WIDGET 1: BYTECODE (Stateful!) ---
            let code_items: Vec<ListItem> = evm.program.iter().enumerate().map(|(i, &byte)| {
                // HEX Index + HEX Value 
                // Example: 0000: 0x60
                let content = format!("{:04x}: {:#04x}", i, byte);

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
                    
                    ListItem::new(format!("{:04x}: {}", i * 16, hex_val))
                })
                .collect();
            f.render_widget(List::new(memory_items).block(Block::default().borders(Borders::ALL).title(" Memory (16b) ")), top_row[1]);

             let storage_items: Vec<ListItem> = evm.storage.storage.iter().map(|(key, value)| {
                ListItem::new(format!("S[{:#x}]: {:#x}", key, value))
            }).collect();
            f.render_widget(List::new(storage_items).block(Block::default().borders(Borders::ALL).title(" Storage ")), top_row[2]);

            let stack_items: Vec<ListItem> = evm.stack.items.iter().enumerate().rev().map(|(i, val)| ListItem::new(format!("[{}] {:#x}", i, val))).collect();
            f.render_widget(List::new(stack_items).block(Block::default().borders(Borders::ALL).title(" Stack ")), bottom_row[0]);

            let status_block = Block::default().borders(Borders::ALL).title(" Status ");
            
           if let Some(msg) = &stop_reason {
                let is_success = msg.contains("Program Finished");
                
                // Choose Color: Green for Success, Red for Error
                let color = if is_success { Color::Green } else { Color::Red };
                let title = if is_success { " COMPLETE " } else { " CRITICAL ERROR " };

                let status_text = format!("{}:\n{}\n\nPress 'q' to Quit", title, msg);
                
                let paragraph = Paragraph::new(status_text)
                    .style(Style::default().fg(color).add_modifier(Modifier::BOLD)) 
                    .block(status_block.border_style(Style::default().fg(color))); 
                
                f.render_widget(paragraph, bottom_row[1]);

                // Render Popup
                let area = centered_rect(60, 30, f.area());
                let popup = Paragraph::new(format!("\nReason: {}", msg))
                    .style(Style::default().bg(color).fg(Color::Black)) // Black text on Color background
                    .block(Block::default().borders(Borders::ALL).title(title));
                f.render_widget(Clear, area);
                f.render_widget(popup, area);

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
        })?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('n') => {
                        if stop_reason.is_none() {
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
                                    if !cont { stop_reason = Some("Program Finished (STOP)".to_string()); }
                                }
                                Err(e) => { stop_reason = Some(format!("{:?}", e)); }
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