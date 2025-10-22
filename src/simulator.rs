/// PIC12F629/675 Simulator - Main execution loop and control
/// 
/// This module provides the main simulator interface that ties together
/// the CPU, memory, instruction decoder, and executor.

use crate::{Cpu, InstructionDecoder, Executor};
use std::path::Path;
use crate::hexloader::{HexLoader, HexProgram};

/// Simulator state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulatorState {
    Running,
    Paused,
    Halted,
    Error,
}

/// Simulator statistics
#[derive(Debug, Clone)]
pub struct SimulatorStats {
    pub instructions_executed: u64,
    pub cycles_elapsed: u64,
}

/// Main simulator
pub struct Simulator {
    cpu: Cpu,
    state: SimulatorState,
    stats: SimulatorStats,
    breakpoints: Vec<u16>,
}

impl Simulator {
    /// Create a new simulator
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            state: SimulatorState::Paused,
            stats: SimulatorStats {
                instructions_executed: 0,
                cycles_elapsed: 0,
            },
            breakpoints: Vec::new(),
        }
    }
    
    /// Reset the simulator
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.state = SimulatorState::Paused;
        self.stats = SimulatorStats {
            instructions_executed: 0,
            cycles_elapsed: 0,
        };
    }
    
    /// Load a program into memory
    pub fn load_program(&mut self, program: &[u16]) {
        self.cpu.memory_mut().load_program(program);
    }
    
    /// Execute a single instruction (step)
    pub fn step(&mut self) -> Result<u8, String> {
        if self.state == SimulatorState::Halted {
            return Err("Simulator is halted".to_string());
        }
        
        // Check if CPU is sleeping
        if self.cpu.is_sleeping() {
            // In sleep mode, only tick WDT and check for wake-up conditions
            let wdt_timeout = self.cpu.wdt_mut().tick();
            
            if wdt_timeout {
                // WDT timeout - wake up from sleep
                self.cpu.wake_up(false);
                return Ok(1);
            }
            
            // Check for interrupts to wake up
            let intcon = self.cpu.read_register(crate::cpu::registers::INTCON);
            let pie1 = self.cpu.read_register(crate::cpu::registers::PIE1);
            let pir1 = self.cpu.read_register(crate::cpu::registers::PIR1);
            
            let (should_interrupt, _) = self.cpu.interrupts().check_interrupts(intcon, pie1, pir1);
            
            if should_interrupt {
                // Wake up by interrupt
                self.cpu.wake_up(true);
                // Continue to normal execution
            } else {
                // Still sleeping, just consume 1 cycle
                self.cpu.add_cycles(1);
                return Ok(1);
            }
        }
        
        // Normal execution (not sleeping or just woke up)
        
        // Check for interrupts BEFORE fetching next instruction
        let interrupted = self.cpu.check_and_handle_interrupts();
        
        // Fetch instruction
        let pc = self.cpu.get_pc();
        let instruction_word = self.cpu.fetch_instruction();
        
        // Decode instruction
        let instruction = InstructionDecoder::decode(instruction_word)
            .map_err(|e| format!("Decode error at PC=0x{:04X}: {}", pc, e))?;
        
        // Increment PC before execution
        self.cpu.increment_pc();
        
        // Execute instruction
        let cycles = Executor::execute(&mut self.cpu, instruction);
        
        // Tick timers and WDT for each cycle consumed
        for _ in 0..cycles {
            let (tmr0_overflow, tmr1_overflow) = self.cpu.timers_mut().tick();
            
            // Tick WDT
            let wdt_timeout = self.cpu.wdt_mut().tick();
            
            if wdt_timeout && !self.cpu.is_sleeping() {
                // WDT timeout during normal operation causes reset
                println!("⚠ WDT timeout - resetting CPU");
                self.cpu.reset();
                return Ok(cycles);
            }
            
            // Handle timer overflows
            if tmr0_overflow {
                let intcon = self.cpu.read_register(crate::cpu::registers::INTCON);
                self.cpu.write_register(crate::cpu::registers::INTCON, intcon | 0x04);
            }
            
            if tmr1_overflow {
                let pir1 = self.cpu.read_register(crate::cpu::registers::PIR1);
                self.cpu.write_register(crate::cpu::registers::PIR1, pir1 | 0x01);
            }
        }
        
        // Add extra cycles if interrupt was serviced
        let total_cycles = if interrupted {
            cycles + 2
        } else {
            cycles
        };
        
        // Update statistics
        self.stats.instructions_executed += 1;
        self.stats.cycles_elapsed += total_cycles as u64;
        self.cpu.add_cycles(total_cycles as u64);
        
        Ok(total_cycles)
    }
    
    /// Run until breakpoint or error
    pub fn run(&mut self) -> Result<(), String> {
        self.state = SimulatorState::Running;
        
        while self.state == SimulatorState::Running {
            let pc = self.cpu.get_pc();
            
            // Check for breakpoint
            if self.breakpoints.contains(&pc) {
                self.state = SimulatorState::Paused;
                return Ok(());
            }
            
            // Execute one instruction
            if let Err(e) = self.step() {
                self.state = SimulatorState::Error;
                return Err(e);
            }
        }
        
        Ok(())
    }
    
    /// Run for a specific number of instructions
    pub fn run_n_instructions(&mut self, n: u64) -> Result<(), String> {
        for _ in 0..n {
            self.step()?;
        }
        Ok(())
    }
    
    /// Run for a specific number of cycles
    pub fn run_n_cycles(&mut self, n: u64) -> Result<(), String> {
        let target_cycles = self.stats.cycles_elapsed + n;
        
        while self.stats.cycles_elapsed < target_cycles {
            self.step()?;
        }
        
        Ok(())
    }
    
    /// Pause execution
    pub fn pause(&mut self) {
        if self.state == SimulatorState::Running {
            self.state = SimulatorState::Paused;
        }
    }
    
    /// Halt execution (cannot be resumed without reset)
    pub fn halt(&mut self) {
        self.state = SimulatorState::Halted;
    }
    
    /// Get current state
    pub fn state(&self) -> SimulatorState {
        self.state
    }
    
    /// Get reference to CPU
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }
    
    /// Get mutable reference to CPU
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }
    
    /// Get statistics
    pub fn stats(&self) -> &SimulatorStats {
        &self.stats
    }
    
    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, address: u16) {
        if !self.breakpoints.contains(&address) {
            self.breakpoints.push(address);
        }
    }
    
    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, address: u16) {
        self.breakpoints.retain(|&bp| bp != address);
    }
    
    /// Clear all breakpoints
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }
    
    /// Get all breakpoints
    pub fn breakpoints(&self) -> &[u16] {
        &self.breakpoints
    }
    
    /// Print CPU state (for debugging)
    pub fn print_state(&self) {
        println!("PC:     0x{:04X}", self.cpu.get_pc());
        println!("W:      0x{:02X}", self.cpu.read_w());
        
        let status = self.cpu.read_register(0x03);
        println!("STATUS: 0x{:02X} [C={} DC={} Z={}]",
            status,
            if status & 0x01 != 0 { "1" } else { "0" },
            if status & 0x02 != 0 { "1" } else { "0" },
            if status & 0x04 != 0 { "1" } else { "0" },
        );
        
        println!("Cycles: {}", self.stats.cycles_elapsed);
        println!("Instructions: {}", self.stats.instructions_executed);
    }

    /// Load a HEX file
    pub fn load_hex_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let hex_program = HexLoader::load_file(path)?;
        self.load_hex_program(hex_program);
        Ok(())
    }
    
    /// Load a HEX program from string
    pub fn load_hex_string(&mut self, content: &str) -> Result<(), String> {
        let hex_program = HexLoader::load_from_string(content)?;
        self.load_hex_program(hex_program);
        Ok(())
    }
    
    /// Load a parsed HEX program
    fn load_hex_program(&mut self, hex_program: HexProgram) {
        // Load program memory
        self.cpu.memory_mut().load_program(&hex_program.program);
        
        // Load EEPROM if present
        if !hex_program.eeprom.is_empty() {
            for (i, &byte) in hex_program.eeprom.iter().enumerate() {
                if i < 128 {
                    self.cpu.memory_mut().write_eeprom(i as u8, byte);
                }
            }
        }
        
        // Set PC to start address
        self.cpu.set_pc(hex_program.start_address);
    }
    
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simulator_creation() {
        let sim = Simulator::new();
        assert_eq!(sim.state(), SimulatorState::Paused);
    }
    
    #[test]
    fn test_load_and_run_simple_program() {
        let mut sim = Simulator::new();
        sim.reset();
        
        // Program: MOVLW 0x55, MOVWF 0x20, NOP
        let program = vec![
            0x3055,  // MOVLW 0x55
            0x00A0,  // MOVWF 0x20
            0x0000,  // NOP
        ];
        
        sim.load_program(&program);
        
        // Run 3 instructions
        sim.run_n_instructions(3).unwrap();
        
        assert_eq!(sim.cpu().read_w(), 0x55);
        assert_eq!(sim.cpu().read_register(0x20), 0x55);
        assert_eq!(sim.stats().instructions_executed, 3);
    }
    
    #[test]
    fn test_breakpoint() {
        let mut sim = Simulator::new();
        sim.reset();
        
        // Program with loop
        let program = vec![
            0x3055,  // 0x000: MOVLW 0x55
            0x00A0,  // 0x001: MOVWF 0x20
            0x2800,  // 0x002: GOTO 0x000 (infinite loop)
        ];
        
        sim.load_program(&program);
        sim.add_breakpoint(0x002);
        
        // Run until breakpoint
        sim.run().unwrap();
        
        assert_eq!(sim.cpu().get_pc(), 0x002);
        assert_eq!(sim.state(), SimulatorState::Paused);
    }
    
    #[test]
    fn test_step() {
        let mut sim = Simulator::new();
        sim.reset();
        
        // MOVLW 0x42
        sim.load_program(&[0x3042]);
        
        let cycles = sim.step().unwrap();
        assert_eq!(cycles, 1);
        assert_eq!(sim.cpu().read_w(), 0x42);
        assert_eq!(sim.stats().instructions_executed, 1);
    }
}