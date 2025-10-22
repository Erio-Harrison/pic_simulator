/// PIC12F629/675 CPU Core
/// 
/// Reference: Data Sheet Section 1.0 - Device Overview
/// 
/// CPU Features:
/// - 8-bit RISC architecture
/// - 35 instructions (all single cycle except branches)
/// - 8-level hardware stack
/// - Direct, indirect, and relative addressing modes

use crate::{gpio::Gpio, memory::Memory, timer::TimerController, interrupt::InterruptController, wdt::Wdt};

/// Special Function Register addresses
/// Reference: Section 2.2 - Register File Map (Table 2-1)
pub mod registers {
    // Common registers (accessible in both banks)
    pub const INDF: u8 = 0x00;      // Indirect addressing (not a physical register)
    pub const TMR0: u8 = 0x01;      // Timer0 register
    pub const PCL: u8 = 0x02;       // Program Counter Low byte
    pub const STATUS: u8 = 0x03;    // Status register
    pub const FSR: u8 = 0x04;       // File Select Register (for indirect addressing)
    pub const GPIO: u8 = 0x05;      // General Purpose I/O
    pub const PCLATH: u8 = 0x0A;    // Program Counter Latch High
    pub const INTCON: u8 = 0x0B;    // Interrupt Control register
    
    // Bank 0 specific registers
    pub const PIR1: u8 = 0x0C;      // Peripheral Interrupt Request register 1
    pub const TMR1L: u8 = 0x0E;     // Timer1 Low byte
    pub const TMR1H: u8 = 0x0F;     // Timer1 High byte
    pub const T1CON: u8 = 0x10;     // Timer1 Control register
    pub const CMCON: u8 = 0x19;     // Comparator Control register
    pub const ADRESH: u8 = 0x1E;    // ADC Result High byte (12F675 only)
    pub const ADCON0: u8 = 0x1F;    // ADC Control register 0 (12F675 only)
    
    // Bank 1 specific registers (accessed when RP0=1 in STATUS)
    pub const OPTION_REG: u8 = 0x81;  // Option register
    pub const TRISIO: u8 = 0x85;      // GPIO Tri-state register
    pub const PIE1: u8 = 0x8C;        // Peripheral Interrupt Enable register 1
    pub const PCON: u8 = 0x8E;        // Power Control register
    pub const OSCCAL: u8 = 0x90;      // Oscillator Calibration register
    pub const WPU: u8 = 0x95;         // Weak Pull-Up register (IOC in some docs)
    pub const IOC: u8 = 0x96;         // Interrupt-On-Change register
    pub const ANSEL: u8 = 0x9F;       // Analog Select register (12F675 only)
}

/// STATUS register bit definitions
/// Reference: Section 2.3 - STATUS Register (Page 14)
pub mod status_bits {
    pub const IRP: u8 = 7;   // Register Bank Select bit (not used in 12F629/675)
    pub const RP1: u8 = 6;   // Register Bank Select bit (not used in 12F629/675)
    pub const RP0: u8 = 5;   // Register Bank Select bit (0=Bank0, 1=Bank1)
    pub const TO: u8 = 4;    // Time-out bit
    pub const PD: u8 = 3;    // Power-down bit
    pub const Z: u8 = 2;     // Zero flag
    pub const DC: u8 = 1;    // Digit Carry/Borrow flag
    pub const C: u8 = 0;     // Carry/Borrow flag
}

/// PIC12F629/675 CPU
pub struct Cpu {
    /// Memory system
    memory: Memory,
    
    /// Working register (W) - accumulator
    /// Reference: Section 2.0 - All operations use W register
    w: u8,
    
    /// Program Counter (13-bit, 0x0000 - 0x1FFF, but only 0x000-0x3FF used for 1K)
    /// Reference: Section 2.6 - Program Counter
    pc: u16,
    
    /// Cycle counter
    cycles: u64,

    /// GPIO controller
    gpio: Gpio,

    /// Time controller
    timers: TimerController, 

    /// Interrupts controller
    interrupts: InterruptController, 

    /// Watchdog Timer
    wdt: Wdt,

    /// Is sleeping or not
    sleeping: bool, 
}

impl Cpu {
    /// Create a new CPU instance
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            w: 0,
            pc: 0,
            cycles: 0,
            gpio: Gpio::new(),
            timers: TimerController::new(),
            interrupts: InterruptController::new(),
            wdt: Wdt::new(), 
            sleeping: false,
        }
    }
    
    /// Reset the CPU to initial state
    /// Reference: Section 9.3 - Reset
    pub fn reset(&mut self) {
        self.pc = 0;
        self.w = 0;
        self.cycles = 0;
        self.memory.reset();
        self.gpio.reset();
        self.timers.reset();
        self.interrupts.reset();
        self.wdt.reset();
        self.sleeping = false;
        
        // Initialize STATUS register
        // Reference: Table 9-7 - Power-on Reset values
        self.write_register(registers::STATUS, 0x18); // TO=1, PD=1
        
        // Initialize other registers to their reset values
        self.write_register(registers::PCLATH, 0x00);
        self.write_register(registers::INTCON, 0x00);
        self.write_register(registers::TRISIO, 0x3F); 
        self.write_register(registers::PIE1, 0x00);
        self.write_register(registers::PIR1, 0x00);
    }
    
    /// Get GPIO Reference
    pub fn gpio(&self) -> &Gpio {
        &self.gpio
    }
    
    /// Get a mutable reference to the GPIO
    pub fn gpio_mut(&mut self) -> &mut Gpio {
        &mut self.gpio
    }


    // Get a reference to the timer controller
    pub fn timers(&self) -> &TimerController {
        &self.timers
    }
    
    // Get a mutable reference to the timer controller
    pub fn timers_mut(&mut self) -> &mut TimerController {
        &mut self.timers
    }

    // Get a reference to the interrupt controller
    pub fn interrupts(&self) -> &InterruptController {
        &self.interrupts
    }
    
    // Get a mutable reference to the interrupt controller
    pub fn interrupts_mut(&mut self) -> &mut InterruptController {
        &mut self.interrupts
    }

    /// Check for pending interrupts and handle them
    /// Returns true if an interrupt was serviced
    pub fn check_and_handle_interrupts(&mut self) -> bool {
        let intcon = self.read_register(registers::INTCON);
        let pie1 = self.read_register(registers::PIE1);
        let pir1 = self.read_register(registers::PIR1);
        
        let (should_interrupt, vector) = self.interrupts.check_interrupts(intcon, pie1, pir1);
        
        if should_interrupt && !self.interrupts.in_isr() {
            // Save return address on stack
            self.push_pc();
            
            // Clear GIE (Global Interrupt Enable)
            let intcon = self.read_register(registers::INTCON);
            self.write_register(registers::INTCON, intcon & !0x80);
            
            // Jump to interrupt vector
            self.set_pc(vector);
            
            // Mark as in ISR
            self.interrupts.enter_isr();
            
            return true;
        }
        
        false
    }    

    // Get WDT reference
    pub fn wdt(&self) -> &Wdt {
        &self.wdt
    }
    
    // Get a mutable WDT reference
    pub fn wdt_mut(&mut self) -> &mut Wdt {
        &mut self.wdt
    }
    
    // Enter sleep mode
    pub fn enter_sleep(&mut self) {
        self.sleeping = true;
        
        // Clear TO and PD bits in STATUS
        // TO=0, PD=0 indicates SLEEP
        let status = self.read_register(registers::STATUS);
        self.write_register(registers::STATUS, status & !0x18); // Clear TO and PD
    }
    
    // Wake up from sleep mode
    pub fn wake_up(&mut self, by_interrupt: bool) {
        self.sleeping = false;
        
        // Set TO bit (timeout occurred)
        let status = self.read_register(registers::STATUS);
        
        if by_interrupt {
            // Wake by interrupt: TO=1, PD=0
            self.write_register(registers::STATUS, (status | 0x10) & !0x08);
        } else {
            // Wake by WDT: TO=0, PD=1
            self.write_register(registers::STATUS, (status | 0x08) & !0x10);
        }
    }
    
    pub fn is_sleeping(&self) -> bool {
        self.sleeping
    }


    // ==================== Register Access ====================
    
    /// Read from a register with banking support
    /// Reference: Section 2.2 - Data Memory Organization
    pub fn read_register(&self, address: u8) -> u8 {
        // Handle special registers
        match address {
            registers::INDF => {
                // Indirect addressing: use FSR as address
                let fsr = self.memory.read_data(registers::FSR);
                self.memory.read_data(fsr)
            },
            registers::PCL => {
                // Return low byte of PC
                (self.pc & 0xFF) as u8
            },
            registers::GPIO => {
                // Read actual GPIO pin states
                self.gpio.read_gpio()
            },
            registers::TRISIO => {
                // Read TRIS register (Bank 1)
                self.gpio.read_tris()
            },
            registers::WPU => {
                // Read Weak Pull-Up register (Bank 1)
                self.gpio.read_wpu()
            },

            registers::TMR1L => {
                // Read Timer1 low byte
                self.timers.timer1.read_low()
            },
            registers::TMR1H => {
                // Read Timer1 high byte
                self.timers.timer1.read_high()
            },
            _ => {
                // Use banking for other registers
                let bank = self.get_bank();
                self.memory.read_data_banked(address, bank)
            }
        }
    }
    
    /// Write to a register with banking support
    pub fn write_register(&mut self, address: u8, value: u8) {
        let bank = self.get_bank();
        
        match address {
            registers::INDF => {
                let fsr = self.memory.read_data(registers::FSR);
                self.memory.write_data(fsr, value);
            },
            registers::PCL => {
                let pclath = self.memory.read_data(registers::PCLATH);
                self.pc = ((pclath as u16) << 8) | (value as u16);
            },
            registers::TMR0 => {
                self.memory.write_data_banked(address, value, bank);
            },
            registers::GPIO => {
                if bank == 0 {
                    // Bank 0: GPIO
                    self.gpio.write_gpio(value);
                    self.memory.write_data(address, value);
                } else {
                    // Bank 1: TRISIO (same address 0x05)
                    self.gpio.write_tris(value);
                    self.memory.write_data_banked(address, value, bank);
                }
            },
            registers::WPU => {
                self.gpio.write_wpu(value);
                self.memory.write_data_banked(address, value, bank);
            },
            registers::TMR1L => {
                self.timers.timer1.write_low(value);
            },
            registers::TMR1H => {
                self.timers.timer1.write_high(value);
            },
            registers::T1CON => {
                self.timers.timer1.configure_from_t1con(value);
                self.memory.write_data(address, value);
            },
            registers::OPTION_REG => {
                self.timers.timer0.configure_from_option(value);
                self.memory.write_data_banked(address, value, bank);
            },
            _ => {
                self.memory.write_data_banked(address, value, bank);
            }
        }
    }
    
    /// Get current bank selection from STATUS register
    /// Reference: Section 2.3 - STATUS Register, RP0 bit
    fn get_bank(&self) -> u8 {
        let status = self.memory.read_data(registers::STATUS);
        if status & (1 << status_bits::RP0) != 0 {
            1 // Bank 1
        } else {
            0 // Bank 0
        }
    }
    
    // ==================== Status Flag Operations ====================
    
    /// Set a bit in the STATUS register
    pub fn set_status_bit(&mut self, bit: u8) {
        let status = self.read_register(registers::STATUS);
        self.write_register(registers::STATUS, status | (1 << bit));
    }
    
    /// Clear a bit in the STATUS register
    pub fn clear_status_bit(&mut self, bit: u8) {
        let status = self.read_register(registers::STATUS);
        self.write_register(registers::STATUS, status & !(1 << bit));
    }
    
    /// Test a bit in the STATUS register
    pub fn test_status_bit(&self, bit: u8) -> bool {
        let status = self.memory.read_data(registers::STATUS);
        (status & (1 << bit)) != 0
    }
    
    /// Update Zero flag based on result
    pub fn update_zero_flag(&mut self, result: u8) {
        if result == 0 {
            self.set_status_bit(status_bits::Z);
        } else {
            self.clear_status_bit(status_bits::Z);
        }
    }
    
    /// Update Carry flag
    pub fn update_carry_flag(&mut self, carry: bool) {
        if carry {
            self.set_status_bit(status_bits::C);
        } else {
            self.clear_status_bit(status_bits::C);
        }
    }
    
    /// Update Digit Carry flag (for BCD operations)
    pub fn update_digit_carry_flag(&mut self, dc: bool) {
        if dc {
            self.set_status_bit(status_bits::DC);
        } else {
            self.clear_status_bit(status_bits::DC);
        }
    }
    
    /// Update all arithmetic flags (Z, C, DC)
    /// Reference: Section 10.0 - Instruction Set Summary
    pub fn update_arithmetic_flags(&mut self, result: u8, carry: bool, digit_carry: bool) {
        self.update_zero_flag(result);
        self.update_carry_flag(carry);
        self.update_digit_carry_flag(digit_carry);
    }
    
    // ==================== W Register Access ====================
    
    /// Read W register
    pub fn read_w(&self) -> u8 {
        self.w
    }
    
    /// Write W register
    pub fn write_w(&mut self, value: u8) {
        self.w = value;
    }
    
    // ==================== Program Counter Operations ====================
    
    /// Get current PC value
    pub fn get_pc(&self) -> u16 {
        self.pc
    }
    
    /// Set PC value (masked to 13 bits)
    pub fn set_pc(&mut self, address: u16) {
        self.pc = address & 0x1FFF;
    }
    
    /// Increment PC (for normal instruction flow)
    pub fn increment_pc(&mut self) {
        self.pc = (self.pc + 1) & 0x1FFF;
    }
    
    // ==================== Stack Operations ====================
    
    /// Push PC onto stack (for CALL instruction)
    pub fn push_pc(&mut self) {
        self.memory.push_stack(self.pc);
    }
    
    /// Pop PC from stack (for RETURN instruction)
    pub fn pop_pc(&mut self) -> u16 {
        self.memory.pop_stack()
    }
    
    // ==================== Memory Access ====================
    
    /// Get reference to memory system
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
    
    /// Get mutable reference to memory system
    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }
    
    // ==================== Execution Control ====================
    
    /// Get cycle count
    pub fn get_cycles(&self) -> u64 {
        self.cycles
    }
    
    /// Add cycles
    pub fn add_cycles(&mut self, cycles: u64) {
        self.cycles += cycles;
    }
    
    /// Fetch next instruction from program memory
    pub fn fetch_instruction(&self) -> u16 {
        self.memory.read_program(self.pc)
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_creation() {
        let cpu = Cpu::new();
        assert_eq!(cpu.get_pc(), 0);
        assert_eq!(cpu.read_w(), 0);
    }
    
    #[test]
    fn test_register_access() {
        let mut cpu = Cpu::new();
        
        cpu.write_register(registers::GPIO, 0x3F);
        // GPIO read might differ based on pin states
        let gpio_val = cpu.read_register(registers::GPIO);
        assert!(gpio_val <= 0x3F); // Should be valid 6-bit value
    }
    
    #[test]
    fn test_w_register() {
        let mut cpu = Cpu::new();
        
        cpu.write_w(0xAB);
        assert_eq!(cpu.read_w(), 0xAB);
    }
    
    #[test]
    fn test_status_flags() {
        let mut cpu = Cpu::new();
        
        // Test Zero flag
        cpu.update_zero_flag(0);
        assert!(cpu.test_status_bit(status_bits::Z));
        
        cpu.update_zero_flag(1);
        assert!(!cpu.test_status_bit(status_bits::Z));
        
        // Test Carry flag
        cpu.update_carry_flag(true);
        assert!(cpu.test_status_bit(status_bits::C));
    }
    
    #[test]
    fn test_pc_operations() {
        let mut cpu = Cpu::new();
        
        cpu.set_pc(0x100);
        assert_eq!(cpu.get_pc(), 0x100);
        
        cpu.increment_pc();
        assert_eq!(cpu.get_pc(), 0x101);
    }
    
    #[test]
    fn test_stack_operations() {
        let mut cpu = Cpu::new();
        
        cpu.push_pc();
        cpu.set_pc(0x200);
        cpu.push_pc();
        
        assert_eq!(cpu.pop_pc(), 0x200);
        assert_eq!(cpu.pop_pc(), 0x000);
    }
    
    #[test]
    fn test_banking() {
        let mut cpu = Cpu::new();
        
        // Default is Bank 0
        assert_eq!(cpu.get_bank(), 0);
        
        // Set RP0 bit to switch to Bank 1
        cpu.set_status_bit(status_bits::RP0);
        assert_eq!(cpu.get_bank(), 1);
        
        // Clear RP0 to return to Bank 0
        cpu.clear_status_bit(status_bits::RP0);
        assert_eq!(cpu.get_bank(), 0);
    }
        
    #[test]
    fn test_gpio_integration() {
        let mut cpu = Cpu::new();
        cpu.reset();
        
        // Test setting TRISIO (configure GP0 as output, others as input)
        cpu.write_register(registers::TRISIO, 0x3E); // Try to set GP0 = output
        
        // GP3 should be forced to input (bit 3 = 1)
        // But we wrote 0x3E which already has bit 3 = 1
        // So we should read back 0x3E
        assert_eq!(cpu.gpio().read_tris(), 0x3E); // 改这里！
        
        // Verify GP0 is actually configured as output
        assert!(!cpu.gpio().is_input(0));
        
        // Verify GP3 is forced as input
        assert!(cpu.gpio().is_input(3));
        
        // Test writing to GPIO
        cpu.write_register(registers::GPIO, 0x01);
        
        // Test reading GPIO
        let gpio_val = cpu.read_register(registers::GPIO);
        assert_eq!(gpio_val & 0x01, 0x01); // GP0 should be high
    }
    
    #[test]
    fn test_gpio_weak_pullup() {
        let mut cpu = Cpu::new();
        cpu.reset();
        
        // Enable weak pull-up on GP0
        cpu.write_register(registers::WPU, 0x01);
        assert_eq!(cpu.gpio().read_wpu(), 0x01);
    }
}