/// PIC12F629/675 Interrupt System
/// 
/// Reference: Section 8.0 - Interrupts (Page 31-34)
/// 
/// Interrupt sources:
/// - TMR0 overflow
/// - GPIO pin change
/// - Comparator output change
/// - A/D converter (12F675 only)
/// - EEPROM write complete
/// - Timer1 overflow

/// Interrupt controller
#[derive(Debug, Clone)]
pub struct InterruptController {
    /// Global interrupt enable (saved during interrupt)
    gie_saved: bool,
    
    /// Interrupt triggered flag (for debugging)
    interrupt_triggered: bool,
    
    /// Interrupt vector (always 0x0004 for PIC12F)
    interrupt_vector: u16,
}

impl InterruptController {
    pub fn new() -> Self {
        Self {
            gie_saved: false,
            interrupt_triggered: false,
            interrupt_vector: 0x0004,
        }
    }
    
    pub fn reset(&mut self) {
        self.gie_saved = false;
        self.interrupt_triggered = false;
    }
    
    /// Check if any interrupt should trigger
    /// Returns (should_interrupt, interrupt_vector)
    pub fn check_interrupts(&self, intcon: u8, pie1: u8, pir1: u8) -> (bool, u16) {
        // Check GIE (Global Interrupt Enable) - bit 7 of INTCON
        let gie = (intcon & 0x80) != 0;
        if !gie {
            return (false, 0);
        }
        
        // Check each interrupt source
        // Format: (interrupt_enable_bit, interrupt_flag_bit)
        
        // TMR0 Overflow Interrupt
        // INTCON: T0IE (bit 5), T0IF (bit 2)
        let t0ie = (intcon & 0x20) != 0;
        let t0if = (intcon & 0x04) != 0;
        if t0ie && t0if {
            return (true, self.interrupt_vector);
        }
        
        // INT External Interrupt (GP2/INT pin)
        // INTCON: INTE (bit 4), INTF (bit 1)
        let inte = (intcon & 0x10) != 0;
        let intf = (intcon & 0x02) != 0;
        if inte && intf {
            return (true, self.interrupt_vector);
        }
        
        // GPIO Port Change Interrupt
        // INTCON: GPIE (bit 3), GPIF (bit 0)
        let gpie = (intcon & 0x08) != 0;
        let gpif = (intcon & 0x01) != 0;
        if gpie && gpif {
            return (true, self.interrupt_vector);
        }
        
        // Peripheral Interrupts (enabled by PEIE in INTCON bit 6)
        let peie = (intcon & 0x40) != 0;
        if peie {
            // Timer1 Overflow Interrupt
            // PIE1: TMR1IE (bit 0), PIR1: TMR1IF (bit 0)
            let tmr1ie = (pie1 & 0x01) != 0;
            let tmr1if = (pir1 & 0x01) != 0;
            if tmr1ie && tmr1if {
                return (true, self.interrupt_vector);
            }
            
            // Comparator Interrupt
            // PIE1: CMIE (bit 3), PIR1: CMIF (bit 3)
            let cmie = (pie1 & 0x08) != 0;
            let cmif = (pir1 & 0x08) != 0;
            if cmie && cmif {
                return (true, self.interrupt_vector);
            }
            
            // A/D Converter Interrupt (12F675 only)
            // PIE1: ADIE (bit 6), PIR1: ADIF (bit 6)
            let adie = (pie1 & 0x40) != 0;
            let adif = (pir1 & 0x40) != 0;
            if adie && adif {
                return (true, self.interrupt_vector);
            }
            
            // EEPROM Write Complete Interrupt
            // PIE1: EEIE (bit 7), PIR1: EEIF (bit 7)
            let eeie = (pie1 & 0x80) != 0;
            let eeif = (pir1 & 0x80) != 0;
            if eeie && eeif {
                return (true, self.interrupt_vector);
            }
        }
        
        (false, 0)
    }
    
    /// Enter interrupt service routine
    /// Saves GIE and clears it
    pub fn enter_isr(&mut self) {
        self.gie_saved = true;
        self.interrupt_triggered = true;
    }
    
    /// Exit interrupt service routine
    /// Called when RETFIE is executed
    pub fn exit_isr(&mut self) {
        self.interrupt_triggered = false;
        // GIE will be restored by RETFIE instruction
    }
    
    /// Check if currently in ISR
    pub fn in_isr(&self) -> bool {
        self.interrupt_triggered
    }
    
    /// Get interrupt vector address
    pub fn get_vector(&self) -> u16 {
        self.interrupt_vector
    }
}

impl Default for InterruptController {
    fn default() -> Self {
        Self::new()
    }
}

/// Interrupt source enumeration (for debugging/logging)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptSource {
    Timer0Overflow,
    ExternalInt,
    GpioChange,
    Timer1Overflow,
    Comparator,
    AdConverter,
    EepromWrite,
}

impl InterruptSource {
    /// Get human-readable name
    pub fn name(&self) -> &str {
        match self {
            InterruptSource::Timer0Overflow => "TMR0 Overflow",
            InterruptSource::ExternalInt => "External INT",
            InterruptSource::GpioChange => "GPIO Change",
            InterruptSource::Timer1Overflow => "Timer1 Overflow",
            InterruptSource::Comparator => "Comparator",
            InterruptSource::AdConverter => "A/D Converter",
            InterruptSource::EepromWrite => "EEPROM Write",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interrupt_controller_creation() {
        let ic = InterruptController::new();
        assert!(!ic.in_isr());
        assert_eq!(ic.get_vector(), 0x0004);
    }
    
    #[test]
    fn test_no_interrupt_when_gie_disabled() {
        let ic = InterruptController::new();
        
        // GIE=0, T0IE=1, T0IF=1
        let intcon = 0x24; // GIE=0, T0IE=1, T0IF=1
        let pie1 = 0x00;
        let pir1 = 0x00;
        
        let (should_int, _) = ic.check_interrupts(intcon, pie1, pir1);
        assert!(!should_int);
    }
    
    #[test]
    fn test_tmr0_interrupt() {
        let ic = InterruptController::new();
        
        // GIE=1, T0IE=1, T0IF=1
        let intcon = 0xA4; // GIE=1, T0IE=1, T0IF=1
        let pie1 = 0x00;
        let pir1 = 0x00;
        
        let (should_int, vec) = ic.check_interrupts(intcon, pie1, pir1);
        assert!(should_int);
        assert_eq!(vec, 0x0004);
    }
    
    #[test]
    fn test_tmr1_interrupt() {
        let ic = InterruptController::new();
        
        // GIE=1, PEIE=1, TMR1IE=1, TMR1IF=1
        let intcon = 0xC0; // GIE=1, PEIE=1
        let pie1 = 0x01;   // TMR1IE=1
        let pir1 = 0x01;   // TMR1IF=1
        
        let (should_int, vec) = ic.check_interrupts(intcon, pie1, pir1);
        assert!(should_int);
        assert_eq!(vec, 0x0004);
    }
    
    #[test]
    fn test_interrupt_disabled_when_ie_cleared() {
        let ic = InterruptController::new();
        
        // GIE=1, T0IE=0, T0IF=1 (flag set but interrupt disabled)
        let intcon = 0x84; // GIE=1, T0IE=0, T0IF=1
        let pie1 = 0x00;
        let pir1 = 0x00;
        
        let (should_int, _) = ic.check_interrupts(intcon, pie1, pir1);
        assert!(!should_int);
    }
    
    #[test]
    fn test_isr_state() {
        let mut ic = InterruptController::new();
        
        assert!(!ic.in_isr());
        
        ic.enter_isr();
        assert!(ic.in_isr());
        
        ic.exit_isr();
        assert!(!ic.in_isr());
    }
}