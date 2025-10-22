use eframe::egui;

use crate::{Simulator, Debugger};
use crate::cpu::registers;

/// GUI simulator state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuiSimulatorState {
    Idle,      // No program loaded
    Running,   // Continuously executing
    Paused,    // Execution paused
}

/// Main GUI application structure
pub struct SimulatorApp {
    // Core simulator instance
    simulator: Simulator,
    
    // GUI state
    gui_state: GuiSimulatorState,
    
    // Execution control
    target_frequency: u32,  // Target execution frequency in Hz
    
    // Disassembly cache: (address, instruction_word, assembly_string)
    disassembly_cache: Vec<(u16, u16, String)>,
    
    // Performance tracking
    last_update_time: std::time::Instant,
    actual_frequency: f32,  // Actual execution frequency
    
    // UI panel visibility
    show_memory_viewer: bool,
    memory_view_address: u8,
    show_timer_panel: bool,
    show_interrupt_panel: bool,
    
    // Statistics
    instructions_this_second: u64,

    // Debug tracking
    last_gpio: u8, 
}

impl SimulatorApp {
    /// Create a new simulator app
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut sim = Simulator::new();
        sim.reset();
        
        // Initialize disassembly cache to prevent index out of bounds
        let mut cache = Vec::new();
        for addr in 0..1024u16 {
            let word = sim.cpu().memory().read_program(addr);
            let asm = Debugger::disassemble(word);
            cache.push((addr, word, asm));
        }
        
        Self {
            simulator: sim,
            gui_state: GuiSimulatorState::Idle,
            target_frequency: 10,  // 1kHz - easier to observe LED blinking
            disassembly_cache: cache,
            last_update_time: std::time::Instant::now(),
            actual_frequency: 0.0,
            show_memory_viewer: true,
            memory_view_address: 0x20,
            show_timer_panel: true,
            show_interrupt_panel: true,
            instructions_this_second: 0,
            last_gpio: 0,
        }
    }
    
    /// Update disassembly cache after loading a program
    fn update_disassembly_cache(&mut self) {
        self.disassembly_cache.clear();
        
        for addr in 0..1024u16 {
            let word = self.simulator.cpu().memory().read_program(addr);
            let asm = Debugger::disassemble(word);
            self.disassembly_cache.push((addr, word, asm));
        }
    }
    
    /// Load a built-in test program (LED blink)
    pub fn load_test_program(&mut self) {
        let program = vec![
            // Initialize - set all pins as OUTPUT
            0x1683,  // 0x000: BSF STATUS, RP0 (switch to Bank 1)
            0x3000,  // 0x001: MOVLW 0x00
            0x0085,  // 0x002: MOVWF TRISIO (all outputs)
            0x1283,  // 0x003: BCF STATUS, RP0 (back to Bank 0)
            
            // Main loop: turn on GP0
            0x3001,  // 0x004: MOVLW 0x01
            0x0085,  // 0x005: MOVWF GPIO (GP0 = HIGH)
            0x200B,  // 0x006: CALL delay
            
            // Turn off GP0
            0x3000,  // 0x007: MOVLW 0x00
            0x0085,  // 0x008: MOVWF GPIO (GP0 = LOW)
            0x200B,  // 0x009: CALL delay
            0x2804,  // 0x00A: GOTO 0x004 (main loop)
            
            // Delay subroutine
            0x30FF,  // 0x00B: delay: MOVLW 0xFF
            0x00A0,  // 0x00C: MOVWF 0x20 
            0x0BA0,  // 0x00D: inner: DECFSZ 0x20, F
            0x280D,  // 0x00E: GOTO inner
            0x0064,  // 0x00F: CLRWDT
            0x0008,  // 0x010: RETURN
        ];
        
        self.simulator.load_program(&program);
        self.update_disassembly_cache();
        self.gui_state = GuiSimulatorState::Paused;
    }
    
    /// Load a HEX file using file dialog
    fn load_hex_file(&mut self) {
        // Open file dialog
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Intel HEX", &["hex"])
            .pick_file()
        {
            match self.simulator.load_hex_file(&path) {
                Ok(_) => {
                    self.update_disassembly_cache();
                    self.gui_state = GuiSimulatorState::Paused;
                    println!("✅ Loaded HEX file: {:?}", path);
                }
                Err(e) => {
                    eprintln!("❌ Failed to load HEX file: {}", e);
                }
            }
        }
    }
    
    /// Draw the code panel (disassembly view)
    fn draw_code_panel(&self, ui: &mut egui::Ui, current_pc: u16) {
        ui.heading("Disassembly");
        ui.add_space(5.0);
        
        // Safety check for empty cache
        if self.disassembly_cache.is_empty() {
            ui.label("No program loaded");
            return;
        }
        
        // Show only non-zero instructions or PC-nearby code
        let start = current_pc.saturating_sub(10);
        let end = (current_pc + 30).min(self.disassembly_cache.len() as u16);
        
        egui::ScrollArea::vertical()
            .max_height(f32::INFINITY)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(400.0);
                
                for addr in start..end {
                    if addr >= self.disassembly_cache.len() as u16 {
                        break;
                    }
                    
                    let (_, word, asm) = &self.disassembly_cache[addr as usize];
                    
                    // Skip empty instructions (unless it's the current PC)
                    if *word == 0 && addr != current_pc {
                        continue;
                    }
                    
                    let is_current = addr == current_pc;
                    let text = format!("0x{:04X}: {:04X}  {}", addr, word, asm);
                    
                    if is_current {
                        ui.colored_label(egui::Color32::RED, format!("▶ {}", text));
                    } else {
                        ui.label(text);
                    }
                }
            });
    }
    
    /// Draw a single GPIO pin
    fn draw_gpio_pin(&mut self, ui: &mut egui::Ui, pin: u8, gpio: u8, trisio: u8) {
        let is_input = (trisio & (1 << pin)) != 0;
        let is_high = (gpio & (1 << pin)) != 0;
        
        ui.vertical(|ui| {
            ui.label(format!("GP{}", pin));
            
            // Allocate space for the LED circle
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(50.0, 50.0),
                egui::Sense::click(),
            );
            
            // Input pins can be toggled by clicking
            if is_input && response.clicked() {
                let current = self.simulator.cpu().gpio().get_external_pin(pin);
                self.simulator.cpu_mut().gpio_mut().set_external_pin(pin, !current);
            }
            
            let painter = ui.painter();
            let center = rect.center();
            let radius = 20.0;
        
            // Add smooth brightness effect (fade between ON/OFF)          
            let brightness = if is_high { 1.0 } else { 0.15 };

            let color = if is_input {
                if is_high {
                    egui::Color32::from_rgba_unmultiplied(100, 200, 255, (255.0 * brightness) as u8)
                } else {
                    egui::Color32::from_rgba_unmultiplied(30, 60, 100, (255.0 * brightness) as u8)
                }
            } else {
                if is_high {
                    egui::Color32::from_rgba_unmultiplied(50, 255, 50, (255.0 * brightness) as u8)
                } else {
                    egui::Color32::from_rgba_unmultiplied(50, 255, 50, (255.0 * brightness) as u8)
                }
            };
            
            // Draw LED circle
            painter.circle_filled(center, radius, color);
            
            // Add glow effect when LED is ON
            if !is_input && is_high {
                painter.circle_filled(center, radius + 3.0, 
                    egui::Color32::from_rgba_premultiplied(50, 255, 50, 50));
            }
            
            painter.circle_stroke(center, radius, 
                egui::Stroke::new(2.0, egui::Color32::WHITE));
            
            // Direction label
            let dir_text = if is_input { "IN" } else { "OUT" };
            ui.label(egui::RichText::new(dir_text).small());
            
            // State label
            let state_text = if is_high { "HIGH" } else { "LOW" };
            ui.label(egui::RichText::new(state_text)
                .small()
                .color(if is_high { egui::Color32::GREEN } else { egui::Color32::GRAY }));
        });
    }
    
    /// Draw GPIO port panel
    fn draw_gpio_panel(&mut self, ui: &mut egui::Ui, gpio: u8, trisio: u8) {
        ui.heading("GPIO Port");
        ui.label(egui::RichText::new("Click input pins to toggle").small().italics());
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            for pin in 0..6 {
                self.draw_gpio_pin(ui, pin, gpio, trisio);
                if pin < 5 {
                    ui.add_space(10.0);
                }
            }
        });
        
        ui.add_space(10.0);
        
        // Show register values
        ui.horizontal(|ui| {
            ui.label(format!("GPIO:   0b{:06b} (0x{:02X})", gpio, gpio));
        });
        ui.horizontal(|ui| {
            ui.label(format!("TRISIO: 0b{:06b} (0x{:02X})", trisio, trisio));
        });
    }
    
    /// Draw control panel (Run, Pause, Step, Reset buttons)
    fn draw_control_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Control");
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            // Step button - execute one instruction
            if ui.button("⏭ Step").clicked() {
                let _ = self.simulator.step();
                self.gui_state = GuiSimulatorState::Paused;
            }
            
            // Run/Pause button
            let (run_text, run_color) = match self.gui_state {
                GuiSimulatorState::Running => ("⏸ Pause", egui::Color32::RED),
                _ => ("▶ Run", egui::Color32::GREEN),
            };
            
            if ui.button(egui::RichText::new(run_text).color(run_color)).clicked() {
                self.gui_state = match self.gui_state {
                    GuiSimulatorState::Running => GuiSimulatorState::Paused,
                    _ => GuiSimulatorState::Running,
                };
            }
            
            // Reset button
            if ui.button(egui::RichText::new("⏹ Reset").color(egui::Color32::RED)).clicked() {
                self.simulator.reset();
                self.gui_state = GuiSimulatorState::Paused;
            }
            
            // Step 100 button - execute 100 instructions quickly
            if ui.button("⏭ Step 100").clicked() {
                for _ in 0..100 {
                    let _ = self.simulator.step();
                }
                self.gui_state = GuiSimulatorState::Paused;
            }
        });
        
        ui.add_space(10.0);
        
        // Speed slider
        ui.horizontal(|ui| {
            ui.label("Speed:");
            ui.add(egui::Slider::new(&mut self.target_frequency, 1_000..=10_000_000)
                .logarithmic(true)
                .custom_formatter(|n, _| {
                    if n >= 1_000_000.0 {
                        format!("{:.1} MHz", n / 1_000_000.0)
                    } else if n >= 1_000.0 {
                        format!("{:.0} kHz", n / 1_000.0)
                    } else {
                        format!("{:.0} Hz", n)
                    }
                }));
        });
        
        // Statistics
        ui.add_space(5.0);
        ui.label(format!("Instructions: {}", self.simulator.stats().instructions_executed));
        ui.label(format!("Cycles: {}", self.simulator.stats().cycles_elapsed));
        
        if self.gui_state == GuiSimulatorState::Running {
            ui.label(format!("Actual: {:.0} Hz", self.actual_frequency));
        }
    }
    
    /// Draw memory viewer panel
    fn draw_memory_viewer(&mut self, ui: &mut egui::Ui) {
        if !self.show_memory_viewer {
            return;
        }
        
        ui.heading("Memory Viewer");
        ui.add_space(5.0);
        
        // Address input
        ui.horizontal(|ui| {
            ui.label("Start Address:");
            ui.add(egui::DragValue::new(&mut self.memory_view_address)
                .prefix("0x")
                .hexadecimal(2, false, true));
        });
        
        ui.add_space(5.0);
        
        // Display memory in hex dump format
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Addr");
                    ui.label("  +0  +1  +2  +3  +4  +5  +6  +7");
                });
                ui.separator();
                
                for row in 0..8 {
                    let addr = self.memory_view_address.saturating_add(row * 8);
                    ui.horizontal(|ui| {
                        ui.label(format!("0x{:02X}", addr));
                        for col in 0..8 {
                            let byte_addr = addr.saturating_add(col);
                            let value = self.simulator.cpu().read_register(byte_addr);
                            ui.label(format!(" {:02X}", value));
                        }
                    });
                }
            });
    }
    
    /// Draw timer panel (TMR0, TMR1)
    fn draw_timer_panel(&self, ui: &mut egui::Ui) {
        if !self.show_timer_panel {
            return;
        }
        
        ui.heading("Timers");
        ui.add_space(5.0);
        
        // Timer0
        let tmr0 = self.simulator.cpu().read_register(registers::TMR0);
        ui.label(format!("TMR0: 0x{:02X} ({})", tmr0, tmr0));
        
        // Timer1
        let tmr1l = self.simulator.cpu().read_register(registers::TMR1L);
        let tmr1h = self.simulator.cpu().read_register(registers::TMR1H);
        let tmr1 = ((tmr1h as u16) << 8) | (tmr1l as u16);
        ui.label(format!("TMR1: 0x{:04X} ({})", tmr1, tmr1));
        
        // T1CON register
        let t1con = self.simulator.cpu().read_register(registers::T1CON);
        ui.label(format!("T1CON: 0b{:08b}", t1con));
    }
}

impl eframe::App for SimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Execute simulator when running
        if self.gui_state == GuiSimulatorState::Running {
                let gpio = self.simulator.cpu().gpio().read_gpio();
                if gpio != self.last_gpio {
                    println!("GPIO changed: 0b{:06b}", gpio);
                    self.last_gpio = gpio;
                }
            let fps = 60.0;
            let cycles_per_frame = (self.target_frequency as f32 / fps).max(1.0) as u32;
            
            for _ in 0..cycles_per_frame {
                if let Err(e) = self.simulator.step() {
                    eprintln!("Error: {}", e);
                    self.gui_state = GuiSimulatorState::Paused;
                    break;
                }
            }
            
            self.instructions_this_second += cycles_per_frame as u64;
            
            // Update actual frequency measurement
            let elapsed = self.last_update_time.elapsed().as_secs_f32();
            if elapsed > 0.0 {
                self.actual_frequency = cycles_per_frame as f32 / elapsed;
                self.last_update_time = std::time::Instant::now();
            }
            
            // Request continuous repaint
            ctx.request_repaint();
        }
        
        // Get current simulator state
        let pc = self.simulator.cpu().get_pc();
        let w = self.simulator.cpu().read_w();
        let status = self.simulator.cpu().read_register(registers::STATUS);
        let gpio = self.simulator.cpu().gpio().read_gpio();
        let trisio = self.simulator.cpu().gpio().read_tris();
        let cycles = self.simulator.stats().cycles_elapsed;
        
        // ==================== Draw UI ====================
        
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📂 Load HEX File...").clicked() {
                        self.load_hex_file();
                        ui.close_menu();
                    }
                    if ui.button("🧪 Load Test Program").clicked() {
                        self.load_test_program();
                        ui.close_menu();
                    }
                    if ui.button("🔄 Reset").clicked() {
                        self.simulator.reset();
                        self.gui_state = GuiSimulatorState::Paused;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("❌ Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_memory_viewer, "Memory Viewer");
                    ui.checkbox(&mut self.show_timer_panel, "Timer Panel");
                    ui.checkbox(&mut self.show_interrupt_panel, "Interrupt Panel");
                });
            });
        });
        
        // Top status bar
        egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("PC: 0x{:04X}", pc)).strong());
                ui.separator();
                ui.label(format!("W: 0x{:02X}", w));
                ui.separator();
                ui.label(format!("STATUS: 0b{:08b}", status));
                ui.separator();
                ui.label(format!("Cycles: {}", cycles));
                ui.separator();
                
                // Running state indicator
                let (state_text, state_color) = match self.gui_state {
                    GuiSimulatorState::Running => ("🟢 RUNNING", egui::Color32::GREEN),
                    GuiSimulatorState::Paused => ("🟡 PAUSED", egui::Color32::RED),
                    GuiSimulatorState::Idle => ("⚪ IDLE", egui::Color32::GRAY),
                };
                ui.label(egui::RichText::new(state_text).color(state_color));
            });
        });
        
        // Left panel: Code disassembly
        egui::SidePanel::left("code_panel")
            .default_width(450.0)
            .show(ctx, |ui| {
                self.draw_code_panel(ui, pc);
            });
        
        // Right panel: Memory & Timer info
        egui::SidePanel::right("info_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_memory_viewer(ui);
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    self.draw_timer_panel(ui);
                });
            });
        
        // Center panel: GPIO & Control
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.draw_gpio_panel(ui, gpio, trisio);
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(20.0);
                self.draw_control_panel(ui);
            });
        });
    }
}