/// Interactive command-line interface for the simulator

use std::io::{self, Write};
use crate::{Simulator, Debugger};

pub struct Cli {
    simulator: Simulator,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            simulator: Simulator::new(),
        }
    }
    
    /// Main REPL loop
    pub fn run(&mut self) {
        println!("PIC12F629/675 Interactive Simulator");
        println!("Type 'help' for available commands\n");
        
        self.simulator.reset();
        
        loop {
            print!("pic> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }
            
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            
            if input == "quit" || input == "exit" {
                break;
            }
            
            self.handle_command(input);
        }
        
        println!("Goodbye!");
    }
    
    fn handle_command(&mut self, input: &str) {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        
        match parts[0] {
            "help" | "h" => self.cmd_help(),
            "reset" | "r" => self.cmd_reset(),
            "step" | "s" => self.cmd_step(parts.get(1)),
            "run" => self.cmd_run(),
            "continue" | "c" => self.cmd_continue(),
            "break" | "b" => self.cmd_break(parts.get(1)),
            "delete" | "d" => self.cmd_delete(parts.get(1)),
            "info" | "i" => self.cmd_info(parts.get(1)),
            "disasm" => self.cmd_disasm(parts.get(1), parts.get(2)),
            "dump" => self.cmd_dump(parts.get(1), parts.get(2)),
            "load" => self.cmd_load(&parts[1..]),
            "reg" => self.cmd_registers(),
            "pc" => self.cmd_pc(parts.get(1)),
            "gpio" => self.cmd_gpio(parts.get(1), parts.get(2)),
            "setpin" => self.cmd_setpin(parts.get(1), parts.get(2)),
            "interrupt" => self.cmd_interrupt(),
            _ => println!("Unknown command: {}", parts[0]),
        }
    }
    
    fn cmd_help(&self) {
        println!("Available commands:");
        println!("  help, h              - Show this help");
        println!("  reset, r             - Reset the simulator");
        println!("  step [n], s [n]      - Execute n instructions (default: 1)");
        println!("  run                  - Run until breakpoint or error");
        println!("  continue, c          - Continue execution");
        println!("  break <addr>, b      - Set breakpoint at address");
        println!("  delete <addr>, d     - Delete breakpoint");
        println!("  info <what>, i       - Show info (breakpoints, stack, etc.)");
        println!("  disasm [addr] [n]    - Disassemble n instructions from addr");
        println!("  dump [addr] [n]      - Dump n bytes of memory from addr");
        println!("  load <hex> <hex>...  - Load program (hex words)");
        println!("  reg                  - Show registers");
        println!("  pc [addr]            - Show/set program counter");
        println!("  quit, exit           - Exit simulator");
        println!("  gpio [show]          - Show GPIO state");
        println!("  setpin <pin> <0|1>   - Set external pin state");
        println!("  int, interrupt       - Show interrupt status");
    }
    
    fn cmd_reset(&mut self) {
        self.simulator.reset();
        println!("Simulator reset");
    }
    
    fn cmd_step(&mut self, count_str: Option<&&str>) {
        let count: u64 = count_str
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        
        for _ in 0..count {
            let pc = self.simulator.cpu().get_pc();
            let word = self.simulator.cpu().memory().read_program(pc);
            
            match self.simulator.step() {
                Ok(cycles) => {
                    let asm = Debugger::disassemble(word);
                    println!("0x{:04X}: {} ({} cycles)", pc, asm, cycles);
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }
        
        println!("\nPC = 0x{:04X}, W = 0x{:02X}, Cycles = {}",
            self.simulator.cpu().get_pc(),
            self.simulator.cpu().read_w(),
            self.simulator.stats().cycles_elapsed
        );
    }
    
    fn cmd_run(&mut self) {
        println!("Running...");
        match self.simulator.run() {
            Ok(_) => println!("Stopped at breakpoint or completion"),
            Err(e) => println!("Error: {}", e),
        }
        
        println!("PC = 0x{:04X}, Cycles = {}",
            self.simulator.cpu().get_pc(),
            self.simulator.stats().cycles_elapsed
        );
    }
    
    fn cmd_continue(&mut self) {
        self.cmd_run();
    }
    
    fn cmd_break(&mut self, addr_str: Option<&&str>) {
        if let Some(addr_str) = addr_str {
            if let Ok(addr) = parse_hex(addr_str) {
                self.simulator.add_breakpoint(addr as u16);
                println!("Breakpoint set at 0x{:04X}", addr);
            } else {
                println!("Invalid address: {}", addr_str);
            }
        } else {
            println!("Usage: break <address>");
        }
    }
    
    fn cmd_delete(&mut self, addr_str: Option<&&str>) {
        if let Some(addr_str) = addr_str {
            if let Ok(addr) = parse_hex(addr_str) {
                self.simulator.remove_breakpoint(addr as u16);
                println!("Breakpoint deleted at 0x{:04X}", addr);
            } else {
                println!("Invalid address: {}", addr_str);
            }
        } else {
            println!("Usage: delete <address>");
        }
    }
    
    fn cmd_info(&self, what: Option<&&str>) {
        match what {
            Some(&"breakpoints") | Some(&"b") => {
                let bps = self.simulator.breakpoints();
                if bps.is_empty() {
                    println!("No breakpoints set");
                } else {
                    println!("Breakpoints:");
                    for bp in bps {
                        println!("  0x{:04X}", bp);
                    }
                }
            }
            Some(&"stack") | Some(&"s") => {
                Debugger::display_stack(self.simulator.cpu());
            }
            Some(&"stats") => {
                let stats = self.simulator.stats();
                println!("Instructions: {}", stats.instructions_executed);
                println!("Cycles:       {}", stats.cycles_elapsed);
            }
            _ => {
                println!("Usage: info <what>");
                println!("  breakpoints, b - Show breakpoints");
                println!("  stack, s       - Show stack");
                println!("  stats          - Show statistics");
            }
        }
    }
    
    fn cmd_disasm(&self, addr_str: Option<&&str>, count_str: Option<&&str>) {
        let addr = addr_str
            .and_then(|s| parse_hex(s).ok())
            .unwrap_or(self.simulator.cpu().get_pc() as u32) as u16;
        
        let count = count_str
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        
        Debugger::disassemble_range(self.simulator.cpu(), addr, count);
    }
    
    fn cmd_dump(&self, addr_str: Option<&&str>, count_str: Option<&&str>) {
        let addr = addr_str
            .and_then(|s| parse_hex(s).ok())
            .unwrap_or(0) as u8;
        
        let count = count_str
            .and_then(|s| s.parse().ok())
            .unwrap_or(64);
        
        Debugger::dump_memory(self.simulator.cpu(), addr, count);
    }
    
    fn cmd_load(&mut self, words: &[&str]) {
        let mut program = Vec::new();
        
        for word_str in words {
            if let Ok(word) = parse_hex(word_str) {
                program.push(word as u16);
            } else {
                println!("Invalid hex value: {}", word_str);
                return;
            }
        }
        
        self.simulator.load_program(&program);
        println!("Loaded {} instructions", program.len());
    }
    
    fn cmd_registers(&self) {
        Debugger::display_registers(self.simulator.cpu());
    }
    
    fn cmd_pc(&mut self, addr_str: Option<&&str>) {
        if let Some(addr_str) = addr_str {
            if let Ok(addr) = parse_hex(addr_str) {
                self.simulator.cpu_mut().set_pc(addr as u16);
                println!("PC set to 0x{:04X}", addr);
            } else {
                println!("Invalid address: {}", addr_str);
            }
        } else {
            println!("PC = 0x{:04X}", self.simulator.cpu().get_pc());
        }
    }

    fn cmd_gpio(&self, subcmd: Option<&&str>, _arg: Option<&&str>) {
        match subcmd {
            None | Some(&"show") => {
                Debugger::display_gpio(self.simulator.cpu());
            }
            _ => println!("Usage: gpio [show]"),
        }
    }

    fn cmd_setpin(&mut self, pin_str: Option<&&str>, value_str: Option<&&str>) {
        if let (Some(pin_str), Some(value_str)) = (pin_str, value_str) {
            if let Ok(pin) = pin_str.parse::<u8>() {
                if pin < 6 {
                    let value = *value_str == "1" || value_str.to_lowercase() == "high";
                    self.simulator.cpu_mut().gpio_mut().set_external_pin(pin, value);
                    println!("Set external pin GP{} to {}", pin, if value { "HIGH" } else { "LOW" });
                } else {
                    println!("Invalid pin number (must be 0-5)");
                }
            } else {
                println!("Invalid pin number");
            }
        } else {
            println!("Usage: setpin <pin> <0|1>");
        }
    }

    fn cmd_interrupt(&self) {
        Debugger::display_interrupts(self.simulator.cpu());
    }    
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse hex string (with or without 0x prefix)
fn parse_hex(s: &str) -> Result<u32, std::num::ParseIntError> {
    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
    u32::from_str_radix(s, 16)
}