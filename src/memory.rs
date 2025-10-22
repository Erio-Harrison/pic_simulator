/// PIC12F629/675 Memory System
/// 
/// Reference: Data Sheet Section 2.0 - Memory Organization (Page 9-20)
/// 
/// Memory Architecture:
/// - Program Memory: 1024 x 14-bit words (Flash)
/// - Data Memory: 128 bytes (RAM) with Bank switching
/// - Stack: 8 levels x 13-bit (Hardware stack for PC)
/// - EEPROM: 128 bytes (Non-volatile data storage)

/// Program memory size: 1024 words of 14-bit instructions
pub const PROGRAM_MEMORY_SIZE: usize = 1024;

/// Data memory size: 256 bytes
pub const DATA_MEMORY_SIZE: usize = 256;

/// Stack depth: 8 levels
pub const STACK_DEPTH: usize = 8;

/// EEPROM size: 128 bytes
pub const EEPROM_SIZE: usize = 128;

/// Memory system for PIC12F629/675
pub struct Memory {
    /// Program memory (Flash): 1024 x 14-bit instructions
    /// Reference: Section 2.1 Program Memory Organization
    program_memory: [u16; PROGRAM_MEMORY_SIZE],
    
    /// Data memory (RAM): 128 bytes with bank switching
    /// Reference: Section 2.2 Data Memory Organization
    /// - Bank 0: 0x00-0x7F
    /// - Bank 1: 0x80-0xFF (mirrors with different registers)
    data_memory: [u8; DATA_MEMORY_SIZE],
    
    /// Hardware stack: 8 levels of 13-bit addresses
    /// Reference: Section 2.0 - 8-Level Deep Hardware Stack
    stack: [u16; STACK_DEPTH],
    
    /// Stack pointer (0-7)
    stack_pointer: usize,
    
    /// EEPROM data memory: 128 bytes
    /// Reference: Section 8.0 Data EEPROM Memory
    eeprom: [u8; EEPROM_SIZE],
}

impl Memory {
    /// Create a new memory system with all memory initialized to zero
    pub fn new() -> Self {
        Self {
            program_memory: [0; PROGRAM_MEMORY_SIZE],
            data_memory: [0; DATA_MEMORY_SIZE],
            stack: [0; STACK_DEPTH],
            stack_pointer: 0,
            eeprom: [0; EEPROM_SIZE],
        }
    }
    
    // ==================== Program Memory ====================
    
    /// Read a 14-bit instruction from program memory
    /// Address is masked to 13 bits (0x000 - 0x3FF for 1K words)
    pub fn read_program(&self, address: u16) -> u16 {
        let addr = (address as usize) & 0x3FF; // Mask to 10 bits for 1K
        self.program_memory[addr]
    }
    
    /// Write a 14-bit instruction to program memory
    /// Used for loading programs (not during normal execution)
    pub fn write_program(&mut self, address: u16, value: u16) {
        let addr = (address as usize) & 0x3FF;
        self.program_memory[addr] = value & 0x3FFF; // Mask to 14 bits
    }
    
    /// Load a program from a slice of 14-bit instructions
    pub fn load_program(&mut self, program: &[u16]) {
        let len = program.len().min(PROGRAM_MEMORY_SIZE);
        for i in 0..len {
            self.program_memory[i] = program[i] & 0x3FFF;
        }
    }
    
    // ==================== Data Memory ====================
    
    /// Read a byte from data memory
    /// Reference: Section 2.2 - Data Memory Organization
    /// 
    /// Special addresses:
    /// - 0x00: INDF (indirect addressing, not a physical register)
    /// - 0x02: PCL (Program Counter Low)
    /// - 0x03: STATUS
    /// - 0x04: FSR (File Select Register for indirect addressing)
    pub fn read_data(&self, address: u8) -> u8 {
        let addr = (address as usize) & 0x7F; // Mask to 7 bits (128 bytes)
        self.data_memory[addr]
    }
    
    /// Write a byte to data memory
    pub fn write_data(&mut self, address: u8, value: u8) {
        let addr = (address as usize) & 0x7F;
        self.data_memory[addr] = value;
    }
    
    /// Read from data memory with bank selection
    /// Reference: Section 2.2 - Bank switching via RP0 bit in STATUS register
    /// 
    /// Bank 0: RP0 = 0 (addresses 0x00-0x7F)
    /// Bank 1: RP0 = 1 (addresses 0x80-0xFF, but physically 0x00-0x7F with different mapping)
    pub fn read_data_banked(&self, address: u8, bank: u8) -> u8 {
        // For PIC12F629/675, banking affects addresses 0x0C and above
        // Addresses 0x00-0x0B are common across banks
        let addr = if address < 0x0C || bank == 0 {
            address as usize
        } else {
            // Bank 1: offset by 0x80 but wrap within our 128-byte array
            ((address as usize) | 0x80) & 0x7F
        };
        self.data_memory[addr]
    }
    
    /// Write to data memory with bank selection
    pub fn write_data_banked(&mut self, address: u8, value: u8, bank: u8) {
        let addr = if address < 0x0C || bank == 0 {
            address as usize
        } else {
            ((address as usize) | 0x80) & 0x7F
        };
        self.data_memory[addr] = value;
    }
    
    // ==================== Hardware Stack ====================
    
    /// Push a 13-bit address onto the hardware stack
    /// Reference: Section 2.0 - 8-Level Deep Hardware Stack
    /// 
    /// Note: If stack overflows (>8 levels), oldest value is lost
    pub fn push_stack(&mut self, address: u16) {
        let addr = address & 0x1FFF; // Mask to 13 bits
        
        if self.stack_pointer >= STACK_DEPTH {
            // Stack overflow: wrap around (oldest value is lost)
            // Shift all values down
            for i in 0..STACK_DEPTH-1 {
                self.stack[i] = self.stack[i+1];
            }
            self.stack[STACK_DEPTH-1] = addr;
        } else {
            self.stack[self.stack_pointer] = addr;
            self.stack_pointer += 1;
        }
    }
    
    /// Pop a 13-bit address from the hardware stack
    /// Returns 0 if stack is empty
    pub fn pop_stack(&mut self) -> u16 {
        if self.stack_pointer == 0 {
            // Stack underflow: return 0
            0
        } else {
            self.stack_pointer -= 1;
            self.stack[self.stack_pointer]
        }
    }
    
    /// Check if stack is empty
    pub fn is_stack_empty(&self) -> bool {
        self.stack_pointer == 0
    }
    
    /// Check if stack is full
    pub fn is_stack_full(&self) -> bool {
        self.stack_pointer >= STACK_DEPTH
    }
    
    /// Get current stack depth
    pub fn stack_depth(&self) -> usize {
        self.stack_pointer
    }
    
    /// Reset stack pointer
    pub fn reset_stack(&mut self) {
        self.stack_pointer = 0;
    }
    
    // ==================== EEPROM ====================
    
    /// Read a byte from EEPROM
    /// Reference: Section 8.0 - Data EEPROM Memory
    pub fn read_eeprom(&self, address: u8) -> u8 {
        let addr = (address as usize) & 0x7F; // 128 bytes
        self.eeprom[addr]
    }
    
    /// Write a byte to EEPROM
    pub fn write_eeprom(&mut self, address: u8, value: u8) {
        let addr = (address as usize) & 0x7F;
        self.eeprom[addr] = value;
    }
    
    // ==================== Utility Functions ====================
    
    /// Reset all memory to initial state
    pub fn reset(&mut self) {
        self.data_memory = [0; DATA_MEMORY_SIZE];
        self.stack_pointer = 0;
        // Note: Program memory and EEPROM are not cleared on reset
    }
    
    /// Get a view of the entire data memory (for debugging)
    pub fn get_data_memory(&self) -> &[u8; DATA_MEMORY_SIZE] {
        &self.data_memory
    }
    
    /// Get a view of the stack (for debugging)
    pub fn get_stack(&self) -> &[u16; STACK_DEPTH] {
        &self.stack
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_program_memory() {
        let mut mem = Memory::new();
        
        // Write and read program memory
        mem.write_program(0x100, 0x3FFF);
        assert_eq!(mem.read_program(0x100), 0x3FFF);
        
        // Test masking to 14 bits
        mem.write_program(0x200, 0xFFFF);
        assert_eq!(mem.read_program(0x200), 0x3FFF);
    }
    
    #[test]
    fn test_data_memory() {
        let mut mem = Memory::new();
        
        // Write and read data memory
        mem.write_data(0x20, 0xAB);
        assert_eq!(mem.read_data(0x20), 0xAB);
    }
    
    #[test]
    fn test_stack_operations() {
        let mut mem = Memory::new();
        
        // Test push and pop
        mem.push_stack(0x100);
        mem.push_stack(0x200);
        mem.push_stack(0x300);
        
        assert_eq!(mem.stack_depth(), 3);
        assert_eq!(mem.pop_stack(), 0x300);
        assert_eq!(mem.pop_stack(), 0x200);
        assert_eq!(mem.pop_stack(), 0x100);
        assert_eq!(mem.stack_depth(), 0);
        assert!(mem.is_stack_empty());
    }
    
    #[test]
    fn test_stack_overflow() {
        let mut mem = Memory::new();
        
        // Fill the stack
        for i in 0..10 {
            mem.push_stack(i * 0x100);
        }
        
        // Stack should be full
        assert!(mem.is_stack_full());
        assert_eq!(mem.stack_depth(), 8);
    }
    
    #[test]
    fn test_stack_underflow() {
        let mut mem = Memory::new();
        
        // Pop from empty stack should return 0
        assert_eq!(mem.pop_stack(), 0);
    }
    
    #[test]
    fn test_eeprom() {
        let mut mem = Memory::new();
        
        mem.write_eeprom(0x10, 0x55);
        assert_eq!(mem.read_eeprom(0x10), 0x55);
    }
}