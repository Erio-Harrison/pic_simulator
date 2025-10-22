/// PIC12F629/675 Instruction Set
/// 
/// Reference: Section 10.0 - Instruction Set Summary (Page 71-80)
/// 
/// The PIC12F629/675 has 35 instructions divided into three categories:
/// 1. Byte-oriented operations (14 instructions)
/// 2. Bit-oriented operations (4 instructions)
/// 3. Literal and control operations (17 instructions)
/// 
/// Instruction Format: 14-bit word
/// - Byte-oriented: [6-bit opcode][1-bit d][7-bit f]
/// - Bit-oriented: [4-bit opcode][3-bit b][7-bit f]
/// - Literal/Control: [6-bit opcode][8-bit k] or [3-bit opcode][11-bit k]

/// Instruction enumeration representing all 35 PIC instructions
/// Reference: Table 10-2 - PIC12F629/675 Instruction Set (Page 72)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    // ==================== Byte-Oriented File Register Operations ====================
    
    /// Add W and f
    /// Operation: (W) + (f) -> dest
    /// Flags affected: C, DC, Z
    ADDWF { f: u8, d: u8 },
    
    /// AND W with f
    /// Operation: (W) AND (f) -> dest
    /// Flags affected: Z
    ANDWF { f: u8, d: u8 },
    
    /// Clear f
    /// Operation: 0 -> f
    /// Flags affected: Z
    CLRF { f: u8 },
    
    /// Clear W
    /// Operation: 0 -> W
    /// Flags affected: Z
    CLRW,
    
    /// Complement f
    /// Operation: NOT (f) -> dest
    /// Flags affected: Z
    COMF { f: u8, d: u8 },
    
    /// Decrement f
    /// Operation: (f) - 1 -> dest
    /// Flags affected: Z
    DECF { f: u8, d: u8 },
    
    /// Decrement f, Skip if 0
    /// Operation: (f) - 1 -> dest, skip if result = 0
    /// Flags affected: None
    DECFSZ { f: u8, d: u8 },
    
    /// Increment f
    /// Operation: (f) + 1 -> dest
    /// Flags affected: Z
    INCF { f: u8, d: u8 },
    
    /// Increment f, Skip if 0
    /// Operation: (f) + 1 -> dest, skip if result = 0
    /// Flags affected: None
    INCFSZ { f: u8, d: u8 },
    
    /// Inclusive OR W with f
    /// Operation: (W) OR (f) -> dest
    /// Flags affected: Z
    IORWF { f: u8, d: u8 },
    
    /// Move f
    /// Operation: (f) -> dest
    /// Flags affected: Z
    MOVF { f: u8, d: u8 },
    
    /// Move W to f
    /// Operation: (W) -> f
    /// Flags affected: None
    MOVWF { f: u8 },
    
    /// No Operation
    /// Operation: None
    /// Flags affected: None
    NOP,
    
    /// Rotate Left f through Carry
    /// Operation: (f) << 1 with Carry -> dest
    /// Flags affected: C
    RLF { f: u8, d: u8 },
    
    /// Rotate Right f through Carry
    /// Operation: (f) >> 1 with Carry -> dest
    /// Flags affected: C
    RRF { f: u8, d: u8 },
    
    /// Subtract W from f
    /// Operation: (f) - (W) -> dest
    /// Flags affected: C, DC, Z
    SUBWF { f: u8, d: u8 },
    
    /// Swap nibbles in f
    /// Operation: (f<3:0>) <-> (f<7:4>) -> dest
    /// Flags affected: None
    SWAPF { f: u8, d: u8 },
    
    /// Exclusive OR W with f
    /// Operation: (W) XOR (f) -> dest
    /// Flags affected: Z
    XORWF { f: u8, d: u8 },
    
    // ==================== Bit-Oriented File Register Operations ====================
    
    /// Bit Clear f
    /// Operation: 0 -> f<b>
    /// Flags affected: None
    BCF { f: u8, b: u8 },
    
    /// Bit Set f
    /// Operation: 1 -> f<b>
    /// Flags affected: None
    BSF { f: u8, b: u8 },
    
    /// Bit Test f, Skip if Clear
    /// Operation: Skip if f<b> = 0
    /// Flags affected: None
    BTFSC { f: u8, b: u8 },
    
    /// Bit Test f, Skip if Set
    /// Operation: Skip if f<b> = 1
    /// Flags affected: None
    BTFSS { f: u8, b: u8 },
    
    // ==================== Literal and Control Operations ====================
    
    /// Add Literal and W
    /// Operation: (W) + k -> W
    /// Flags affected: C, DC, Z
    ADDLW { k: u8 },
    
    /// AND Literal with W
    /// Operation: (W) AND k -> W
    /// Flags affected: Z
    ANDLW { k: u8 },
    
    /// Call Subroutine
    /// Operation: (PC+1) -> Stack, k -> PC<10:0>
    /// Flags affected: None
    CALL { k: u16 },
    
    /// Clear Watchdog Timer
    /// Operation: 0 -> WDT, 1 -> TO, 1 -> PD
    /// Flags affected: TO, PD
    CLRWDT,
    
    /// Go to Address
    /// Operation: k -> PC<10:0>
    /// Flags affected: None
    GOTO { k: u16 },
    
    /// Inclusive OR Literal with W
    /// Operation: (W) OR k -> W
    /// Flags affected: Z
    IORLW { k: u8 },
    
    /// Move Literal to W
    /// Operation: k -> W
    /// Flags affected: None
    MOVLW { k: u8 },
    
    /// Return from Interrupt
    /// Operation: Stack -> PC, 1 -> GIE
    /// Flags affected: None
    RETFIE,
    
    /// Return with Literal in W
    /// Operation: k -> W, Stack -> PC
    /// Flags affected: None
    RETLW { k: u8 },
    
    /// Return from Subroutine
    /// Operation: Stack -> PC
    /// Flags affected: None
    RETURN,
    
    /// Go into Standby mode
    /// Operation: 0 -> WDT, 1 -> TO, 0 -> PD
    /// Flags affected: TO, PD
    SLEEP,
    
    /// Subtract W from Literal
    /// Operation: k - (W) -> W
    /// Flags affected: C, DC, Z
    SUBLW { k: u8 },
    
    /// Exclusive OR Literal with W
    /// Operation: (W) XOR k -> W
    /// Flags affected: Z
    XORLW { k: u8 },
}

/// Instruction decoder
/// Reference: Section 10.0 - Instruction formats and opcodes
pub struct InstructionDecoder;

impl InstructionDecoder {
    /// Decode a 14-bit instruction word into an Instruction enum
    /// Reference: Table 10-2 - Instruction opcode mapping
    pub fn decode(word: u16) -> Result<Instruction, String> {
        // Special control instructions are checked first 
        // (Before checking byte operations)
        // CLRWDT, RETFIE, RETURN, SLEEP
        match word {
            0x0064 => return Ok(Instruction::CLRWDT),
            0x0009 => return Ok(Instruction::RETFIE),
            0x0008 => return Ok(Instruction::RETURN),
            0x0063 => return Ok(Instruction::SLEEP),
            _ => {}
        }
        
        // CALL and GOTO have special format with 11-bit address
        // CALL: 100k kkkk kkkk (top 3 bits = 100)
        // GOTO: 101k kkkk kkkk (top 3 bits = 101)
        let top3 = (word >> 11) & 0x07;
        if top3 == 0b100 {
            let k = word & 0x7FF;
            return Ok(Instruction::CALL { k });
        }
        if top3 == 0b101 {
            let k = word & 0x7FF;
            return Ok(Instruction::GOTO { k });
        }
        
        let opcode = (word >> 8) & 0x3F; // Top 6 bits
        
        // Byte-oriented file register operations (14 instructions)
        // Format: 00xxxx dfff ffff
        if opcode & 0x30 == 0x00 {
            let d = ((word >> 7) & 0x01) as u8;
            let f = (word & 0x7F) as u8;
            
            return match (word >> 8) & 0x3F {
                0x07 => Ok(Instruction::ADDWF { f, d }),
                0x05 => Ok(Instruction::ANDWF { f, d }),
                0x01 if d == 1 => Ok(Instruction::CLRF { f }),
                0x01 if d == 0 && f == 0 => Ok(Instruction::CLRW),
                0x09 => Ok(Instruction::COMF { f, d }),
                0x03 => Ok(Instruction::DECF { f, d }),
                0x0B => Ok(Instruction::DECFSZ { f, d }),
                0x0A => Ok(Instruction::INCF { f, d }),
                0x0F => Ok(Instruction::INCFSZ { f, d }),
                0x04 => Ok(Instruction::IORWF { f, d }),
                0x08 => Ok(Instruction::MOVF { f, d }),
                0x00 if d == 1 => Ok(Instruction::MOVWF { f }),
                0x00 if d == 0 && f == 0 => Ok(Instruction::NOP),
                0x0D => Ok(Instruction::RLF { f, d }),
                0x0C => Ok(Instruction::RRF { f, d }),
                0x02 => Ok(Instruction::SUBWF { f, d }),
                0x0E => Ok(Instruction::SWAPF { f, d }),
                0x06 => Ok(Instruction::XORWF { f, d }),
                _ => Err(format!("Unknown byte-oriented instruction: 0x{:04X}", word)),
            };
        }
        
        // Bit-oriented file register operations (4 instructions)
        // Format: 01bb bfff ffff
        if opcode & 0x30 == 0x10 {
            let b = ((word >> 7) & 0x07) as u8;
            let f = (word & 0x7F) as u8;
            
            return match (word >> 10) & 0x03 {
                0x00 => Ok(Instruction::BCF { f, b }),
                0x01 => Ok(Instruction::BSF { f, b }),
                0x02 => Ok(Instruction::BTFSC { f, b }),
                0x03 => Ok(Instruction::BTFSS { f, b }),
                _ => Err(format!("Unknown bit-oriented instruction: 0x{:04X}", word)),
            };
        }
        
        // 8-bit literal operations (top 6 bits)
        let k = (word & 0xFF) as u8;
        
        match opcode {
            0x3E => Ok(Instruction::ADDLW { k }),
            0x39 => Ok(Instruction::ANDLW { k }),
            0x38 => Ok(Instruction::IORLW { k }),
            0x30..=0x33 => Ok(Instruction::MOVLW { k }),
            0x3C => Ok(Instruction::SUBLW { k }),
            0x3A => Ok(Instruction::XORLW { k }),
            0x34..=0x37 => Ok(Instruction::RETLW { k }),
            
            _ => Err(format!("Unknown instruction: 0x{:04X}", word)),
        }
    }
    
    /// Get the number of cycles an instruction takes
    /// Reference: Table 10-2 - Most instructions are 1 cycle, except branches
    pub fn get_cycles(instruction: &Instruction) -> u8 {
        match instruction {
            // 2-cycle instructions (branches and calls)
            Instruction::CALL { .. }
            | Instruction::GOTO { .. }
            | Instruction::RETFIE
            | Instruction::RETLW { .. }
            | Instruction::RETURN => 2,
            
            // Skip instructions are 1 cycle normally, 2 if skip occurs
            // (will be handled during execution)
            Instruction::BTFSC { .. }
            | Instruction::BTFSS { .. }
            | Instruction::DECFSZ { .. }
            | Instruction::INCFSZ { .. } => 1,
            
            // All other instructions are 1 cycle
            _ => 1,
        }
    }
    
    /// Check if instruction is a skip instruction
    pub fn is_skip_instruction(instruction: &Instruction) -> bool {
        matches!(
            instruction,
            Instruction::BTFSC { .. }
                | Instruction::BTFSS { .. }
                | Instruction::DECFSZ { .. }
                | Instruction::INCFSZ { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decode_movlw() {
        // MOVLW 0x55 = 0x3055
        let inst = InstructionDecoder::decode(0x3055).unwrap();
        assert_eq!(inst, Instruction::MOVLW { k: 0x55 });
    }
    
    #[test]
    fn test_decode_movwf() {
        // MOVWF 0x20 = 0x0020 | 0x0080 = 0x00A0
        let inst = InstructionDecoder::decode(0x00A0).unwrap();
        assert_eq!(inst, Instruction::MOVWF { f: 0x20 });
    }
    
    #[test]
    fn test_decode_addwf() {
        // ADDWF 0x20, W (d=0) = 0x0720
        let inst = InstructionDecoder::decode(0x0720).unwrap();
        assert_eq!(inst, Instruction::ADDWF { f: 0x20, d: 0 });
        
        // ADDWF 0x20, F (d=1) = 0x07A0
        let inst = InstructionDecoder::decode(0x07A0).unwrap();
        assert_eq!(inst, Instruction::ADDWF { f: 0x20, d: 1 });
    }
    
    #[test]
    fn test_decode_bcf() {
        // BCF 0x05, 0 = 0x1005
        let inst = InstructionDecoder::decode(0x1005).unwrap();
        assert_eq!(inst, Instruction::BCF { f: 0x05, b: 0 });
        
        // BCF 0x05, 7 = 0x1385
        let inst = InstructionDecoder::decode(0x1385).unwrap();
        assert_eq!(inst, Instruction::BCF { f: 0x05, b: 7 });
    }
    
    #[test]
    fn test_decode_goto() {
        // GOTO 0x100 = 0x2900
        let inst = InstructionDecoder::decode(0x2900).unwrap();
        assert_eq!(inst, Instruction::GOTO { k: 0x100 });
    }
    
    #[test]
    fn test_decode_call() {
        // CALL 0x100 = 0x2100
        let inst = InstructionDecoder::decode(0x2100).unwrap();
        assert_eq!(inst, Instruction::CALL { k: 0x100 });
    }
    
    #[test]
    fn test_decode_nop() {
        // NOP = 0x0000
        let inst = InstructionDecoder::decode(0x0000).unwrap();
        assert_eq!(inst, Instruction::NOP);
    }
    
    #[test]
    fn test_decode_clrw() {
        // CLRW = 0x0100
        let inst = InstructionDecoder::decode(0x0100).unwrap();
        assert_eq!(inst, Instruction::CLRW);
    }
    
    #[test]
    fn test_decode_return() {
        // RETURN = 0x0008
        let inst = InstructionDecoder::decode(0x0008).unwrap();
        assert_eq!(inst, Instruction::RETURN);
    }
    
    #[test]
    fn test_get_cycles() {
        assert_eq!(InstructionDecoder::get_cycles(&Instruction::NOP), 1);
        assert_eq!(InstructionDecoder::get_cycles(&Instruction::GOTO { k: 0 }), 2);
        assert_eq!(InstructionDecoder::get_cycles(&Instruction::CALL { k: 0 }), 2);
        assert_eq!(InstructionDecoder::get_cycles(&Instruction::RETURN), 2);
    }
}