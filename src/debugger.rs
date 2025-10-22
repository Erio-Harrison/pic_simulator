/// PIC12F629/675 Debugger
/// 
/// Provides debugging utilities: disassembler, memory dump, register display

use crate::{Cpu, InstructionDecoder, Instruction};
use crate::cpu::{registers, status_bits};

pub struct Debugger;

impl Debugger {
    /// Disassemble an instruction word
    pub fn disassemble(word: u16) -> String {
        match InstructionDecoder::decode(word) {
            Ok(inst) => Self::format_instruction(&inst),
            Err(_) => format!("??? 0x{:04X}", word),
        }
    }
    
    /// Format an instruction as assembly-like string
    pub fn format_instruction(inst: &Instruction) -> String {
        match inst {
            Instruction::ADDWF { f, d } => format!("ADDWF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::ANDWF { f, d } => format!("ANDWF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::CLRF { f } => format!("CLRF 0x{:02X}", f),
            Instruction::CLRW => "CLRW".to_string(),
            Instruction::COMF { f, d } => format!("COMF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::DECF { f, d } => format!("DECF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::DECFSZ { f, d } => format!("DECFSZ 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::INCF { f, d } => format!("INCF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::INCFSZ { f, d } => format!("INCFSZ 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::IORWF { f, d } => format!("IORWF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::MOVF { f, d } => format!("MOVF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::MOVWF { f } => format!("MOVWF 0x{:02X}", f),
            Instruction::NOP => "NOP".to_string(),
            Instruction::RLF { f, d } => format!("RLF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::RRF { f, d } => format!("RRF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::SUBWF { f, d } => format!("SUBWF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::SWAPF { f, d } => format!("SWAPF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            Instruction::XORWF { f, d } => format!("XORWF 0x{:02X}, {}", f, if *d == 0 { "W" } else { "F" }),
            
            Instruction::BCF { f, b } => format!("BCF 0x{:02X}, {}", f, b),
            Instruction::BSF { f, b } => format!("BSF 0x{:02X}, {}", f, b),
            Instruction::BTFSC { f, b } => format!("BTFSC 0x{:02X}, {}", f, b),
            Instruction::BTFSS { f, b } => format!("BTFSS 0x{:02X}, {}", f, b),
            
            Instruction::ADDLW { k } => format!("ADDLW 0x{:02X}", k),
            Instruction::ANDLW { k } => format!("ANDLW 0x{:02X}", k),
            Instruction::CALL { k } => format!("CALL 0x{:03X}", k),
            Instruction::CLRWDT => "CLRWDT".to_string(),
            Instruction::GOTO { k } => format!("GOTO 0x{:03X}", k),
            Instruction::IORLW { k } => format!("IORLW 0x{:02X}", k),
            Instruction::MOVLW { k } => format!("MOVLW 0x{:02X}", k),
            Instruction::RETFIE => "RETFIE".to_string(),
            Instruction::RETLW { k } => format!("RETLW 0x{:02X}", k),
            Instruction::RETURN => "RETURN".to_string(),
            Instruction::SLEEP => "SLEEP".to_string(),
            Instruction::SUBLW { k } => format!("SUBLW 0x{:02X}", k),
            Instruction::XORLW { k } => format!("XORLW 0x{:02X}", k),
        }
    }
    
    /// Disassemble a range of program memory
    pub fn disassemble_range(cpu: &Cpu, start: u16, count: u16) {
        println!("\nDisassembly:");
        println!("Addr   Hex    Assembly");
        println!("------ ------ ----------------");
        
        for i in 0..count {
            let addr = start + i;
            let word = cpu.memory().read_program(addr);
            let asm = Self::disassemble(word);
            
            let marker = if addr == cpu.get_pc() { ">" } else { " " };
            println!("{} 0x{:04X} 0x{:04X} {}", marker, addr, word, asm);
        }
    }
    
    /// Display CPU registers
    pub fn display_registers(cpu: &Cpu) {
        println!("\nRegisters:");
        println!("  W      = 0x{:02X} ({})", cpu.read_w(), cpu.read_w());
        println!("  PC     = 0x{:04X}", cpu.get_pc());
        
        let status = cpu.read_register(registers::STATUS);
        println!("  STATUS = 0x{:02X} [C={} DC={} Z={} PD={} TO={}]",
            status,
            if status & (1 << status_bits::C) != 0 { "1" } else { "0" },
            if status & (1 << status_bits::DC) != 0 { "1" } else { "0" },
            if status & (1 << status_bits::Z) != 0 { "1" } else { "0" },
            if status & (1 << status_bits::PD) != 0 { "1" } else { "0" },
            if status & (1 << status_bits::TO) != 0 { "1" } else { "0" },
        );
        
        let fsr = cpu.read_register(registers::FSR);
        println!("  FSR    = 0x{:02X}", fsr);
        
        let pclath = cpu.read_register(registers::PCLATH);
        println!("  PCLATH = 0x{:02X}", pclath);
        
        let intcon = cpu.read_register(registers::INTCON);
        println!("  INTCON = 0x{:02X}", intcon);
        
        let gpio = cpu.read_register(registers::GPIO);
        println!("  GPIO   = 0x{:02X} = 0b{:06b}", gpio, gpio & 0x3F);
    }
    
    /// Display special function registers
    pub fn display_sfr(cpu: &Cpu) {
        println!("\nSpecial Function Registers:");
        println!("  Address  Name       Value");
        println!("  -------  ---------  -----");
        
        let sfr_list = [
            (0x00, "INDF"),
            (0x01, "TMR0"),
            (0x02, "PCL"),
            (0x03, "STATUS"),
            (0x04, "FSR"),
            (0x05, "GPIO"),
            (0x0A, "PCLATH"),
            (0x0B, "INTCON"),
            (0x0C, "PIR1"),
            (0x0E, "TMR1L"),
            (0x0F, "TMR1H"),
            (0x10, "T1CON"),
        ];
        
        for (addr, name) in sfr_list.iter() {
            let val = cpu.read_register(*addr);
            println!("  0x{:02X}     {:9}  0x{:02X}", addr, name, val);
        }
    }
    
    /// Dump memory region
    pub fn dump_memory(cpu: &Cpu, start: u8, count: u8) {
        println!("\nMemory Dump:");
        println!("Addr  +0 +1 +2 +3 +4 +5 +6 +7  +8 +9 +A +B +C +D +E +F  ASCII");
        println!("----  -----------------------------------------------  ----------------");
        
        let mut addr = start;
        while addr < start.saturating_add(count) {
            print!("0x{:02X}  ", addr);
            
            // Print hex values
            let mut ascii = String::new();
            for i in 0..16 {
                if addr.saturating_add(i) >= start.saturating_add(count) {
                    print!("   ");
                    ascii.push(' ');
                } else {
                    let val = cpu.read_register(addr + i);
                    print!("{:02X} ", val);
                    
                    // ASCII representation
                    if val >= 0x20 && val <= 0x7E {
                        ascii.push(val as char);
                    } else {
                        ascii.push('.');
                    }
                }
                
                if i == 7 {
                    print!(" ");
                }
            }
            
            println!(" {}", ascii);
            addr = addr.saturating_add(16);
        }
    }
    
    /// Display stack contents
    pub fn display_stack(cpu: &Cpu) {
        println!("\nStack:");
        let stack = cpu.memory().get_stack();
        let depth = cpu.memory().stack_depth();
        
        if depth == 0 {
            println!("  (empty)");
        } else {
            println!("  Level  Address");
            println!("  -----  -------");
            for i in 0..depth {
                println!("  {}      0x{:04X}", i, stack[i]);
            }
        }
    }
    
    /// Full state dump
    pub fn dump_state(cpu: &Cpu) {
        Self::display_registers(cpu);
        Self::display_stack(cpu);
        println!("\nCycles: {}", cpu.get_cycles());
    }

    /// Display GPIO pin states with visual representation
    pub fn display_gpio(cpu: &Cpu) {
        println!("\nGPIO Port State:");
        println!("┌────┬────┬────┬────┬────┬────┐");
        println!("│GP5 │GP4 │GP3 │GP2 │GP1 │GP0 │");
        println!("├────┼────┼────┼────┼────┼────┤");
        
        // Show pin states
        print!("│");
        for pin in (0..6).rev() {
            let state = cpu.gpio().get_pin_state(pin);
            match state {
                crate::gpio::PinState::High => print!(" ●  │"),
                crate::gpio::PinState::Low => print!(" ○  │"),
                crate::gpio::PinState::HighZ => print!(" -  │"),
            }
        }
        println!();
        
        // Show directions
        print!("│");
        for pin in (0..6).rev() {
            if cpu.gpio().is_input(pin) {
                print!(" IN │");
            } else {
                print!("OUT │");
            }
        }
        println!();
        
        println!("└────┴────┴────┴────┴────┴────┘");
        
        // Show register values
        let gpio_val = cpu.read_register(crate::cpu::registers::GPIO);
        let tris_val = cpu.gpio().read_tris();
        let wpu_val = cpu.gpio().read_wpu();
        
        println!("\nGPIO   = 0x{:02X} = 0b{:06b}", gpio_val, gpio_val);
        println!("TRISIO = 0x{:02X} = 0b{:06b}", tris_val, tris_val);
        println!("WPU    = 0x{:02X} = 0b{:06b}", wpu_val, wpu_val);
    }

    /// Display interrupt status
    pub fn display_interrupts(cpu: &Cpu) {
        println!("\nInterrupt Status:");
        
        let intcon = cpu.read_register(crate::cpu::registers::INTCON);
        let pie1 = cpu.read_register(crate::cpu::registers::PIE1);
        let pir1 = cpu.read_register(crate::cpu::registers::PIR1);
        
        println!("  INTCON = 0x{:02X}", intcon);
        println!("    GIE   = {} (Global Interrupt Enable)", if intcon & 0x80 != 0 { "1" } else { "0" });
        println!("    PEIE  = {} (Peripheral Interrupt Enable)", if intcon & 0x40 != 0 { "1" } else { "0" });
        println!("    T0IE  = {} (TMR0 Overflow Interrupt Enable)", if intcon & 0x20 != 0 { "1" } else { "0" });
        println!("    INTE  = {} (External Interrupt Enable)", if intcon & 0x10 != 0 { "1" } else { "0" });
        println!("    GPIE  = {} (GPIO Change Interrupt Enable)", if intcon & 0x08 != 0 { "1" } else { "0" });
        println!("    T0IF  = {} (TMR0 Overflow Flag)", if intcon & 0x04 != 0 { "1" } else { "0" });
        println!("    INTF  = {} (External Interrupt Flag)", if intcon & 0x02 != 0 { "1" } else { "0" });
        println!("    GPIF  = {} (GPIO Change Flag)", if intcon & 0x01 != 0 { "1" } else { "0" });
        
        println!("\n  PIE1 = 0x{:02X}", pie1);
        println!("    EEIE   = {}", if pie1 & 0x80 != 0 { "1" } else { "0" });
        println!("    ADIE   = {}", if pie1 & 0x40 != 0 { "1" } else { "0" });
        println!("    CMIE   = {}", if pie1 & 0x08 != 0 { "1" } else { "0" });
        println!("    TMR1IE = {}", if pie1 & 0x01 != 0 { "1" } else { "0" });
        
        println!("\n  PIR1 = 0x{:02X}", pir1);
        println!("    EEIF   = {}", if pir1 & 0x80 != 0 { "1" } else { "0" });
        println!("    ADIF   = {}", if pir1 & 0x40 != 0 { "1" } else { "0" });
        println!("    CMIF   = {}", if pir1 & 0x08 != 0 { "1" } else { "0" });
        println!("    TMR1IF = {}", if pir1 & 0x01 != 0 { "1" } else { "0" });
        
        println!("\n  In ISR: {}", if cpu.interrupts().in_isr() { "Yes" } else { "No" });
    }    
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_disassemble() {
        assert_eq!(Debugger::disassemble(0x3055), "MOVLW 0x55");
        assert_eq!(Debugger::disassemble(0x00A0), "MOVWF 0x20");
        assert_eq!(Debugger::disassemble(0x2900), "GOTO 0x100");
    }
}