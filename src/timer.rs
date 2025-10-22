/// PIC12F629/675 Timer Implementation
/// 
/// Reference: 
/// - Section 4.0 - Timer0 Module (Page 19-22)
/// - Section 5.0 - Timer1 Module (Page 23-28)
/// 
/// Timer0: 8-bit timer/counter with prescaler
/// Timer1: 16-bit timer/counter

/// Timer0 configuration and state
#[derive(Debug, Clone)]
pub struct Timer0 {
    /// Timer0 counter value (8-bit)
    counter: u8,
    
    /// Prescaler value (shared with WDT)
    prescaler: u16,
    
    /// Prescaler assignment (false = Timer0, true = WDT)
    prescaler_assigned_to_wdt: bool,
    
    /// Prescaler rate (1:2, 1:4, 1:8, ..., 1:256)
    prescaler_rate: u16,
    
    /// Clock source (false = internal, true = external T0CKI pin)
    clock_source_external: bool,
    
    /// Edge select for external clock (false = increment on low-to-high, true = high-to-low)
    edge_select: bool,
}

impl Timer0 {
    pub fn new() -> Self {
        Self {
            counter: 0,
            prescaler: 0,
            prescaler_assigned_to_wdt: false,
            prescaler_rate: 2,
            clock_source_external: false,
            edge_select: false,
        }
    }
    
    pub fn reset(&mut self) {
        self.counter = 0;
        self.prescaler = 0;
        self.prescaler_assigned_to_wdt = false;
        self.prescaler_rate = 2;
        self.clock_source_external = false;
        self.edge_select = false;
    }
    
    /// Read TMR0 register
    pub fn read_counter(&self) -> u8 {
        self.counter
    }
    
    /// Write to TMR0 register (also clears prescaler)
    pub fn write_counter(&mut self, value: u8) {
        self.counter = value;
        self.prescaler = 0; // Writing to TMR0 clears prescaler
    }
    
    /// Configure from OPTION_REG
    /// Reference: Section 2.3 - OPTION_REG Register
    pub fn configure_from_option(&mut self, option_reg: u8) {
        // Bit 5: T0CS - Timer0 Clock Source Select
        self.clock_source_external = (option_reg & 0x20) != 0;
        
        // Bit 4: T0SE - Timer0 Source Edge Select
        self.edge_select = (option_reg & 0x10) != 0;
        
        // Bit 3: PSA - Prescaler Assignment
        self.prescaler_assigned_to_wdt = (option_reg & 0x08) != 0;
        
        // Bits 2-0: PS<2:0> - Prescaler Rate Select
        let ps_bits = option_reg & 0x07;
        self.prescaler_rate = match ps_bits {
            0 => 2,
            1 => 4,
            2 => 8,
            3 => 16,
            4 => 32,
            5 => 64,
            6 => 128,
            7 => 256,
            _ => 2,
        };
        
        // If prescaler assignment changes, clear prescaler
        self.prescaler = 0;
    }
    
    /// Increment timer on each instruction cycle (if internal clock)
    /// Returns true if overflow occurred (TMR0 wrapped from 0xFF to 0x00)
    pub fn tick(&mut self) -> bool {
        if self.clock_source_external {
            // External clock source - not implemented in this tick
            return false;
        }
        
        if self.prescaler_assigned_to_wdt {
            // No prescaler for Timer0, increment directly
            let (new_val, overflow) = self.counter.overflowing_add(1);
            self.counter = new_val;
            overflow
        } else {
            // Use prescaler
            self.prescaler += 1;
            if self.prescaler >= self.prescaler_rate {
                self.prescaler = 0;
                let (new_val, overflow) = self.counter.overflowing_add(1);
                self.counter = new_val;
                return overflow;
            }
            false
        }
    }
    
    /// Get current prescaler value (for debugging)
    pub fn get_prescaler(&self) -> u16 {
        self.prescaler
    }
}

impl Default for Timer0 {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer1 configuration and state
#[derive(Debug, Clone)]
pub struct Timer1 {
    /// Timer1 counter value (16-bit)
    counter: u16,
    
    /// Timer1 enabled
    enabled: bool,
    
    /// Clock source (false = internal Fosc/4, true = external)
    clock_source_external: bool,
    
    /// Prescaler value (1:1, 1:2, 1:4, 1:8)
    prescaler_rate: u16,
    
    /// Current prescaler counter
    prescaler: u16,
    
    /// Oscillator enable
    oscillator_enabled: bool,
    
    /// External clock synchronization
    sync_external_clock: bool,
}

impl Timer1 {
    pub fn new() -> Self {
        Self {
            counter: 0,
            enabled: false,
            clock_source_external: false,
            prescaler_rate: 1,
            prescaler: 0,
            oscillator_enabled: false,
            sync_external_clock: true,
        }
    }
    
    pub fn reset(&mut self) {
        self.counter = 0;
        self.enabled = false;
        self.clock_source_external = false;
        self.prescaler_rate = 1;
        self.prescaler = 0;
        self.oscillator_enabled = false;
        self.sync_external_clock = true;
    }
    
    /// Read low byte of Timer1
    pub fn read_low(&self) -> u8 {
        (self.counter & 0xFF) as u8
    }
    
    /// Read high byte of Timer1
    pub fn read_high(&self) -> u8 {
        ((self.counter >> 8) & 0xFF) as u8
    }
    
    /// Write low byte of Timer1
    pub fn write_low(&mut self, value: u8) {
        self.counter = (self.counter & 0xFF00) | (value as u16);
    }
    
    /// Write high byte of Timer1
    pub fn write_high(&mut self, value: u8) {
        self.counter = (self.counter & 0x00FF) | ((value as u16) << 8);
    }
    
    /// Configure from T1CON register
    /// Reference: Section 5.1 - T1CON Register
    pub fn configure_from_t1con(&mut self, t1con: u8) {
        // Bit 7: Unimplemented
        
        // Bit 6: TMR1GE - Timer1 Gate Enable (not implemented in basic version)
        
        // Bits 5-4: T1CKPS<1:0> - Timer1 Input Clock Prescale Select
        let prescaler_bits = (t1con >> 4) & 0x03;
        self.prescaler_rate = match prescaler_bits {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            _ => 1,
        };
        
        // Bit 3: T1OSCEN - Timer1 Oscillator Enable
        self.oscillator_enabled = (t1con & 0x08) != 0;
        
        // Bit 2: T1SYNC - Timer1 External Clock Input Synchronization
        self.sync_external_clock = (t1con & 0x04) == 0; // Note: 0 = sync, 1 = no sync
        
        // Bit 1: TMR1CS - Timer1 Clock Source Select
        self.clock_source_external = (t1con & 0x02) != 0;
        
        // Bit 0: TMR1ON - Timer1 On
        self.enabled = (t1con & 0x01) != 0;
    }
    
    /// Increment timer on each instruction cycle (if enabled and using internal clock)
    /// Returns true if overflow occurred (wrapped from 0xFFFF to 0x0000)
    pub fn tick(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        
        if self.clock_source_external {
            // External clock source - not implemented in this basic tick
            return false;
        }
        
        // Internal clock (Fosc/4)
        self.prescaler += 1;
        if self.prescaler >= self.prescaler_rate {
            self.prescaler = 0;
            let (new_val, overflow) = self.counter.overflowing_add(1);
            self.counter = new_val;
            return overflow;
        }
        
        false
    }
    
    /// Get current counter value (for debugging)
    pub fn get_counter(&self) -> u16 {
        self.counter
    }
    
    /// Check if timer is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for Timer1 {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer controller managing both Timer0 and Timer1
#[derive(Debug, Clone)]
pub struct TimerController {
    pub timer0: Timer0,
    pub timer1: Timer1,
}

impl TimerController {
    pub fn new() -> Self {
        Self {
            timer0: Timer0::new(),
            timer1: Timer1::new(),
        }
    }
    
    pub fn reset(&mut self) {
        self.timer0.reset();
        self.timer1.reset();
    }
    
    /// Tick both timers (called once per instruction cycle)
    /// Returns (tmr0_overflow, tmr1_overflow)
    pub fn tick(&mut self) -> (bool, bool) {
        let tmr0_overflow = self.timer0.tick();
        let tmr1_overflow = self.timer1.tick();
        (tmr0_overflow, tmr1_overflow)
    }
}

impl Default for TimerController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timer0_basic() {
        let mut tmr0 = Timer0::new();
        
        // Write value
        tmr0.write_counter(0x10);
        assert_eq!(tmr0.read_counter(), 0x10);
    }
    
    #[test]
    fn test_timer0_prescaler_1_2() {
        let mut tmr0 = Timer0::new();
        
        // Configure prescaler to 1:2
        let option_reg = 0x00; // PS=000 (1:2), PSA=0 (assigned to TMR0)
        tmr0.configure_from_option(option_reg);
        
        tmr0.write_counter(0xFE);
        
        // First tick: prescaler 0->1, counter stays at 0xFE
        assert!(!tmr0.tick());
        assert_eq!(tmr0.read_counter(), 0xFE);
        
        // Second tick: prescaler 1->0, counter 0xFE->0xFF
        assert!(!tmr0.tick());
        assert_eq!(tmr0.read_counter(), 0xFF);
        
        // Third tick: prescaler 0->1, counter stays at 0xFF
        assert!(!tmr0.tick());
        assert_eq!(tmr0.read_counter(), 0xFF);
        
        // Fourth tick: prescaler 1->0, counter 0xFF->0x00 (overflow!)
        assert!(tmr0.tick());
        assert_eq!(tmr0.read_counter(), 0x00);
    }
    
    #[test]
    fn test_timer0_no_prescaler() {
        let mut tmr0 = Timer0::new();
        
        // Assign prescaler to WDT (no prescaler for Timer0)
        let option_reg = 0x08; // PSA=1
        tmr0.configure_from_option(option_reg);
        
        tmr0.write_counter(0xFF);
        
        // Each tick increments counter directly
        assert!(tmr0.tick());
        assert_eq!(tmr0.read_counter(), 0x00);
    }
    
    #[test]
    fn test_timer1_basic() {
        let mut tmr1 = Timer1::new();
        
        // Write low and high bytes
        tmr1.write_low(0x34);
        tmr1.write_high(0x12);
        
        assert_eq!(tmr1.get_counter(), 0x1234);
        assert_eq!(tmr1.read_low(), 0x34);
        assert_eq!(tmr1.read_high(), 0x12);
    }
    
    #[test]
    fn test_timer1_counting() {
        let mut tmr1 = Timer1::new();
        
        // Enable Timer1 with 1:1 prescaler
        let t1con = 0x01; // TMR1ON=1, others default
        tmr1.configure_from_t1con(t1con);
        
        tmr1.write_low(0xFE);
        tmr1.write_high(0xFF);
        
        // Should be at 0xFFFE
        assert_eq!(tmr1.get_counter(), 0xFFFE);
        
        // Tick once: 0xFFFE -> 0xFFFF
        assert!(!tmr1.tick());
        assert_eq!(tmr1.get_counter(), 0xFFFF);
        
        // Tick again: 0xFFFF -> 0x0000 (overflow)
        assert!(tmr1.tick());
        assert_eq!(tmr1.get_counter(), 0x0000);
    }
    
    #[test]
    fn test_timer1_prescaler() {
        let mut tmr1 = Timer1::new();
        
        // Enable Timer1 with 1:4 prescaler
        let t1con = 0x21; // TMR1ON=1, T1CKPS=10 (1:4)
        tmr1.configure_from_t1con(t1con);
        
        tmr1.write_low(0x00);
        tmr1.write_high(0x00);
        
        // Need 4 ticks to increment counter once
        assert!(!tmr1.tick());
        assert_eq!(tmr1.get_counter(), 0x0000);
        
        assert!(!tmr1.tick());
        assert_eq!(tmr1.get_counter(), 0x0000);
        
        assert!(!tmr1.tick());
        assert_eq!(tmr1.get_counter(), 0x0000);
        
        assert!(!tmr1.tick());
        assert_eq!(tmr1.get_counter(), 0x0001);
    }
    
    #[test]
    fn test_timer1_disabled() {
        let mut tmr1 = Timer1::new();
        
        // Timer1 disabled (TMR1ON=0)
        let t1con = 0x00;
        tmr1.configure_from_t1con(t1con);
        
        tmr1.write_low(0x00);
        
        // Tick should not increment
        assert!(!tmr1.tick());
        assert_eq!(tmr1.get_counter(), 0x0000);
    }
}