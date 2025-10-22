/// PIC12F629/675 Instruction Executor
/// 
/// Reference: Section 10.2 - Instruction Descriptions (Page 73-80)

use crate::cpu::{Cpu, registers, status_bits};
use crate::instruction::Instruction;

pub struct Executor;

impl Executor {
    /// Execute a single instruction and return cycles consumed
    pub fn execute(cpu: &mut Cpu, instruction: Instruction) -> u8 {
        match instruction {
            // ==================== Byte-Oriented Operations ====================
            
            Instruction::ADDWF { f, d } => Self::addwf(cpu, f, d),
            Instruction::ANDWF { f, d } => Self::andwf(cpu, f, d),
            Instruction::CLRF { f } => Self::clrf(cpu, f),
            Instruction::CLRW => Self::clrw(cpu),
            Instruction::COMF { f, d } => Self::comf(cpu, f, d),
            Instruction::DECF { f, d } => Self::decf(cpu, f, d),
            Instruction::DECFSZ { f, d } => Self::decfsz(cpu, f, d),
            Instruction::INCF { f, d } => Self::incf(cpu, f, d),
            Instruction::INCFSZ { f, d } => Self::incfsz(cpu, f, d),
            Instruction::IORWF { f, d } => Self::iorwf(cpu, f, d),
            Instruction::MOVF { f, d } => Self::movf(cpu, f, d),
            Instruction::MOVWF { f } => Self::movwf(cpu, f),
            Instruction::NOP => 1,
            Instruction::RLF { f, d } => Self::rlf(cpu, f, d),
            Instruction::RRF { f, d } => Self::rrf(cpu, f, d),
            Instruction::SUBWF { f, d } => Self::subwf(cpu, f, d),
            Instruction::SWAPF { f, d } => Self::swapf(cpu, f, d),
            Instruction::XORWF { f, d } => Self::xorwf(cpu, f, d),
            
            // ==================== Bit-Oriented Operations ====================
            
            Instruction::BCF { f, b } => Self::bcf(cpu, f, b),
            Instruction::BSF { f, b } => Self::bsf(cpu, f, b),
            Instruction::BTFSC { f, b } => Self::btfsc(cpu, f, b),
            Instruction::BTFSS { f, b } => Self::btfss(cpu, f, b),
            
            // ==================== Literal & Control Operations ====================
            
            Instruction::ADDLW { k } => Self::addlw(cpu, k),
            Instruction::ANDLW { k } => Self::andlw(cpu, k),
            Instruction::CALL { k } => Self::call(cpu, k),
            Instruction::CLRWDT => Self::clrwdt(cpu),
            Instruction::GOTO { k } => Self::goto(cpu, k),
            Instruction::IORLW { k } => Self::iorlw(cpu, k),
            Instruction::MOVLW { k } => Self::movlw(cpu, k),
            Instruction::RETFIE => Self::retfie(cpu),
            Instruction::RETLW { k } => Self::retlw(cpu, k),
            Instruction::RETURN => Self::ret(cpu),
            Instruction::SLEEP => Self::sleep(cpu),
            Instruction::SUBLW { k } => Self::sublw(cpu, k),
            Instruction::XORLW { k } => Self::xorlw(cpu, k),
        }
    }
    
    // ==================== Byte-Oriented Implementations ====================
    
    /// ADDWF: Add W and f
    fn addwf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let w = cpu.read_w();
        let val = cpu.read_register(f);
        let result = w.wrapping_add(val);
        
        // Carry flag
        let carry = (w as u16 + val as u16) > 0xFF;
        cpu.update_carry_flag(carry);
        
        // Digit carry (bit 4)
        let dc = ((w & 0x0F) + (val & 0x0F)) > 0x0F;
        cpu.update_digit_carry_flag(dc);
        
        // Zero flag
        cpu.update_zero_flag(result);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    /// ANDWF: AND W with f
    fn andwf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let w = cpu.read_w();
        let val = cpu.read_register(f);
        let result = w & val;
        cpu.update_zero_flag(result);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    /// CLRF: Clear f
    fn clrf(cpu: &mut Cpu, f: u8) -> u8 {
        cpu.write_register(f, 0);
        cpu.set_status_bit(status_bits::Z);
        1
    }
    
    /// CLRW: Clear W
    fn clrw(cpu: &mut Cpu) -> u8 {
        cpu.write_w(0);
        cpu.set_status_bit(status_bits::Z);
        1
    }
    
    /// COMF: Complement f
    fn comf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let val = cpu.read_register(f);
        let result = !val;
        cpu.update_zero_flag(result);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    /// DECF: Decrement f
    fn decf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let val = cpu.read_register(f);
        let result = val.wrapping_sub(1);
        cpu.update_zero_flag(result);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    /// DECFSZ: Decrement f, Skip if 0
    fn decfsz(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let val = cpu.read_register(f);
        let result = val.wrapping_sub(1);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        
        if result == 0 {
            cpu.increment_pc(); // Skip next
            2
        } else {
            1
        }
    }
    
    /// INCF: Increment f
    fn incf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let val = cpu.read_register(f);
        let result = val.wrapping_add(1);
        cpu.update_zero_flag(result);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    /// INCFSZ: Increment f, Skip if 0
    fn incfsz(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let val = cpu.read_register(f);
        let result = val.wrapping_add(1);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        
        if result == 0 {
            cpu.increment_pc();
            2
        } else {
            1
        }
    }
    
    /// IORWF: Inclusive OR W with f
    fn iorwf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let w = cpu.read_w();
        let val = cpu.read_register(f);
        let result = w | val;
        cpu.update_zero_flag(result);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    /// MOVF: Move f
    fn movf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let val = cpu.read_register(f);
        cpu.update_zero_flag(val);
        
        if d == 0 {
            cpu.write_w(val);
        } else {
            cpu.write_register(f, val);
        }
        1
    }
    
    /// MOVWF: Move W to f
    fn movwf(cpu: &mut Cpu, f: u8) -> u8 {
        let w = cpu.read_w();
        cpu.write_register(f, w);
        1
    }
    
    /// RLF: Rotate Left f through Carry
    fn rlf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let val = cpu.read_register(f);
        let old_carry = if cpu.test_status_bit(status_bits::C) { 1 } else { 0 };
        let result = (val << 1) | old_carry;
        let new_carry = (val & 0x80) != 0;
        
        cpu.update_carry_flag(new_carry);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    /// RRF: Rotate Right f through Carry
    fn rrf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let val = cpu.read_register(f);
        let old_carry = if cpu.test_status_bit(status_bits::C) { 0x80 } else { 0 };
        let result = (val >> 1) | old_carry;
        let new_carry = (val & 0x01) != 0;
        
        cpu.update_carry_flag(new_carry);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    /// SUBWF: Subtract W from f
    fn subwf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let w = cpu.read_w();
        let val = cpu.read_register(f);
        let result = val.wrapping_sub(w);
        
        // Carry = 1 if NO borrow (val >= w)
        cpu.update_carry_flag(val >= w);
        
        // Digit carry
        cpu.update_digit_carry_flag((val & 0x0F) >= (w & 0x0F));
        
        cpu.update_zero_flag(result);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    /// SWAPF: Swap nibbles in f
    fn swapf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let val = cpu.read_register(f);
        let result = (val << 4) | (val >> 4);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    /// XORWF: Exclusive OR W with f
    fn xorwf(cpu: &mut Cpu, f: u8, d: u8) -> u8 {
        let w = cpu.read_w();
        let val = cpu.read_register(f);
        let result = w ^ val;
        cpu.update_zero_flag(result);
        
        if d == 0 {
            cpu.write_w(result);
        } else {
            cpu.write_register(f, result);
        }
        1
    }
    
    // ==================== Bit-Oriented Implementations ====================
    
    /// BCF: Bit Clear f
    fn bcf(cpu: &mut Cpu, f: u8, b: u8) -> u8 {
        let val = cpu.read_register(f);
        cpu.write_register(f, val & !(1 << b));
        1
    }
    
    /// BSF: Bit Set f
    fn bsf(cpu: &mut Cpu, f: u8, b: u8) -> u8 {
        let val = cpu.read_register(f);
        cpu.write_register(f, val | (1 << b));
        1
    }
    
    /// BTFSC: Bit Test f, Skip if Clear
    fn btfsc(cpu: &mut Cpu, f: u8, b: u8) -> u8 {
        let val = cpu.read_register(f);
        if (val & (1 << b)) == 0 {
            cpu.increment_pc();
            2
        } else {
            1
        }
    }
    
    /// BTFSS: Bit Test f, Skip if Set
    fn btfss(cpu: &mut Cpu, f: u8, b: u8) -> u8 {
        let val = cpu.read_register(f);
        if (val & (1 << b)) != 0 {
            cpu.increment_pc();
            2
        } else {
            1
        }
    }
    
    // ==================== Literal & Control Implementations ====================
    
    /// ADDLW: Add Literal and W
    fn addlw(cpu: &mut Cpu, k: u8) -> u8 {
        let w = cpu.read_w();
        let result = w.wrapping_add(k);
        
        cpu.update_carry_flag((w as u16 + k as u16) > 0xFF);
        cpu.update_digit_carry_flag(((w & 0x0F) + (k & 0x0F)) > 0x0F);
        cpu.update_zero_flag(result);
        
        cpu.write_w(result);
        1
    }
    
    /// ANDLW: AND Literal with W
    fn andlw(cpu: &mut Cpu, k: u8) -> u8 {
        let result = cpu.read_w() & k;
        cpu.update_zero_flag(result);
        cpu.write_w(result);
        1
    }
    
    /// CALL: Call Subroutine
    fn call(cpu: &mut Cpu, k: u16) -> u8 {
        cpu.push_pc();
        let pclath = cpu.read_register(registers::PCLATH);
        let addr = ((pclath as u16 & 0x18) << 8) | k;
        cpu.set_pc(addr);
        2
    }
    
    /// CLRWDT: Clear Watchdog Timer
    fn clrwdt(cpu: &mut Cpu) -> u8 {
        cpu.wdt_mut().clear();
        
        // Set TO and PD bits in STATUS
        let status = cpu.read_register(registers::STATUS);
        cpu.write_register(registers::STATUS, status | 0x18); // Set TO and PD
        
        1 // 1 cycle
    }
    
    /// GOTO: Go to Address
    fn goto(cpu: &mut Cpu, k: u16) -> u8 {
        let pclath = cpu.read_register(registers::PCLATH);
        let addr = ((pclath as u16 & 0x18) << 8) | k;
        cpu.set_pc(addr);
        2
    }
    
    /// IORLW: Inclusive OR Literal with W
    fn iorlw(cpu: &mut Cpu, k: u8) -> u8 {
        let result = cpu.read_w() | k;
        cpu.update_zero_flag(result);
        cpu.write_w(result);
        1
    }
    
    /// MOVLW: Move Literal to W
    fn movlw(cpu: &mut Cpu, k: u8) -> u8 {
        cpu.write_w(k);
        1
    }
    
    /// RETFIE: Return from Interrupt
    fn retfie(cpu: &mut Cpu) -> u8 {
        let addr = cpu.pop_pc();
        cpu.set_pc(addr);
        
        // Set GIE bit (re-enable interrupts)
        let intcon = cpu.read_register(registers::INTCON);
        cpu.write_register(registers::INTCON, intcon | 0x80);
        
        // Exit ISR state
        cpu.interrupts_mut().exit_isr();
        
        2
    }
    
    /// RETLW: Return with Literal in W
    fn retlw(cpu: &mut Cpu, k: u8) -> u8 {
        cpu.write_w(k);
        let addr = cpu.pop_pc();
        cpu.set_pc(addr);
        2
    }
    
    /// RETURN: Return from Subroutine
    fn ret(cpu: &mut Cpu) -> u8 {
        let addr = cpu.pop_pc();
        cpu.set_pc(addr);
        2
    }
    
    /// SLEEP: Enter sleep mode
    fn sleep(cpu: &mut Cpu) -> u8 {
        cpu.enter_sleep();
        1 // 1 cycle
    }
    
    /// SUBLW: Subtract W from Literal
    fn sublw(cpu: &mut Cpu, k: u8) -> u8 {
        let w = cpu.read_w();
        let result = k.wrapping_sub(w);
        
        cpu.update_carry_flag(k >= w);
        cpu.update_digit_carry_flag((k & 0x0F) >= (w & 0x0F));
        cpu.update_zero_flag(result);
        
        cpu.write_w(result);
        1
    }
    
    /// XORLW: Exclusive OR Literal with W
    fn xorlw(cpu: &mut Cpu, k: u8) -> u8 {
        let result = cpu.read_w() ^ k;
        cpu.update_zero_flag(result);
        cpu.write_w(result);
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_movlw() {
        let mut cpu = Cpu::new();
        cpu.reset();
        Executor::execute(&mut cpu, Instruction::MOVLW { k: 0x42 });
        assert_eq!(cpu.read_w(), 0x42);
    }
    
    #[test]
    fn test_addwf() {
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.write_w(0x10);
        cpu.write_register(0x20, 0x25);
        Executor::execute(&mut cpu, Instruction::ADDWF { f: 0x20, d: 0 });
        assert_eq!(cpu.read_w(), 0x35);
    }
}