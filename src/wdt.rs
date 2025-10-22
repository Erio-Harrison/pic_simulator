/// PIC12F629/675 Watchdog Timer (WDT)
/// 
/// Reference: Section 9.8 - Watchdog Timer (WDT) (Page 49-50)
/// 
/// The WDT is a free-running on-chip RC oscillator which does not require
/// any external components. During normal operation, a WDT timeout generates
/// a device RESET. If the device is in SLEEP mode, a WDT timeout causes the
/// device to wake-up and continue with normal operation.

/// Watchdog Timer controller
#[derive(Debug, Clone)]
pub struct Wdt {
    /// WDT counter (18-bit)
    counter: u32,
    
    /// WDT enabled
    enabled: bool,
    
    /// Prescaler value (shared with Timer0)
    prescaler: u16,
    
    /// Prescaler rate (1:1, 1:2, 1:4, ..., 1:128)
    prescaler_rate: u16,
    
    /// Prescaler assigned to WDT (not Timer0)
    prescaler_assigned: bool,
    
    /// WDT timeout period (nominal 18ms without prescaler)
    /// With maximum prescaler (1:128), timeout is ~2.3 seconds
    timeout_period: u32,
}

impl Wdt {
    /// Nominal WDT period without prescaler (in instruction cycles)
    /// Assuming 4MHz Fosc: 18ms / (1us/cycle) = 18000 cycles
    const NOMINAL_PERIOD: u32 = 18000;
    
    pub fn new() -> Self {
        Self {
            counter: 0,
            enabled: true, // WDT is enabled by default
            prescaler: 0,
            prescaler_rate: 1,
            prescaler_assigned: false,
            timeout_period: Self::NOMINAL_PERIOD,
        }
    }
    
    pub fn reset(&mut self) {
        self.counter = 0;
        self.enabled = true;
        self.prescaler = 0;
        self.prescaler_rate = 1;
        self.prescaler_assigned = false;
        self.timeout_period = Self::NOMINAL_PERIOD;
    }
    
    /// Clear WDT counter (CLRWDT instruction)
    pub fn clear(&mut self) {
        self.counter = 0;
        self.prescaler = 0;
    }
    
    /// Configure prescaler from OPTION_REG
    pub fn configure_prescaler(&mut self, option_reg: u8) {
        // Bit 3: PSA - Prescaler Assignment
        // 0 = Prescaler assigned to Timer0
        // 1 = Prescaler assigned to WDT
        self.prescaler_assigned = (option_reg & 0x08) != 0;
        
        if self.prescaler_assigned {
            // Bits 2-0: PS<2:0> - Prescaler Rate Select for WDT
            let ps_bits = option_reg & 0x07;
            self.prescaler_rate = match ps_bits {
                0 => 1,
                1 => 2,
                2 => 4,
                3 => 8,
                4 => 16,
                5 => 32,
                6 => 64,
                7 => 128,
                _ => 1,
            };
            
            self.timeout_period = Self::NOMINAL_PERIOD * (self.prescaler_rate as u32);
        } else {
            self.prescaler_rate = 1;
            self.timeout_period = Self::NOMINAL_PERIOD;
        }
        
        // Clear prescaler when assignment changes
        self.prescaler = 0;
    }
    
    /// Enable/disable WDT
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if WDT is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Tick WDT (called once per instruction cycle)
    /// Returns true if WDT timeout occurred
    pub fn tick(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        
        if self.prescaler_assigned && self.prescaler_rate > 1 {
            // Use prescaler
            self.prescaler += 1;
            if self.prescaler >= self.prescaler_rate {
                self.prescaler = 0;
                self.counter += 1;
            }
        } else {
            // No prescaler
            self.counter += 1;
        }
        
        // Check for timeout
        if self.counter >= self.timeout_period {
            self.counter = 0;
            self.prescaler = 0;
            return true; // WDT timeout - should cause reset or wake-up
        }
        
        false
    }
    
    /// Get current counter value (for debugging)
    pub fn get_counter(&self) -> u32 {
        self.counter
    }
    
    /// Get timeout period
    pub fn get_timeout_period(&self) -> u32 {
        self.timeout_period
    }
}

impl Default for Wdt {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wdt_creation() {
        let wdt = Wdt::new();
        assert!(wdt.is_enabled());
        assert_eq!(wdt.get_counter(), 0);
    }
    
    #[test]
    fn test_wdt_clear() {
        let mut wdt = Wdt::new();
        
        // Increment counter
        for _ in 0..100 {
            wdt.tick();
        }
        
        assert!(wdt.get_counter() > 0);
        
        // Clear WDT
        wdt.clear();
        assert_eq!(wdt.get_counter(), 0);
    }
    
    #[test]
    fn test_wdt_timeout() {
        let mut wdt = Wdt::new();
        
        // Run until just before timeout
        for _ in 0..(Wdt::NOMINAL_PERIOD - 1) {
            assert!(!wdt.tick());
        }
        
        // Next tick should cause timeout
        assert!(wdt.tick());
        
        // Counter should reset
        assert_eq!(wdt.get_counter(), 0);
    }
    
    #[test]
    fn test_wdt_prescaler() {
        let mut wdt = Wdt::new();
        
        // Set prescaler to 1:4
        let option_reg = 0x0A; // PSA=1, PS=010 (1:4)
        wdt.configure_prescaler(option_reg);
        
        assert_eq!(wdt.get_timeout_period(), Wdt::NOMINAL_PERIOD * 4);
    }
}