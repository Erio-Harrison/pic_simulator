/// PIC12F629/675 GPIO Port Implementation
/// 
/// Reference: Section 3.0 - I/O Ports (Page 15-18)
/// 
/// The PIC12F629/675 has a 6-bit bidirectional port (GPIO):
/// - GP5, GP4, GP3, GP2, GP1, GP0
/// - GP3 is input only (no TRIS control)
/// - Each pin can be configured as input or output via TRISIO
/// - Weak pull-ups available on GPIO<0:5> when enabled

/// GPIO pin state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinState {
    Low,
    High,
    HighZ, // High impedance (input mode)
}

/// GPIO port controller
#[derive(Debug, Clone)]
pub struct Gpio {
    /// Current port value (what's written to GPIO register)
    port_value: u8,
    
    /// Port direction (TRISIO) - 1 = input, 0 = output
    tris: u8,
    
    /// Weak pull-up enable (WPU) - 1 = enabled, 0 = disabled
    weak_pullup: u8,
    
    /// External pin states (simulates external world)
    external_pins: u8,
    
    /// Pin output enable (from peripherals like comparator)
    peripheral_output_enable: u8,
    
    /// Pin output value (from peripherals)
    peripheral_output_value: u8,
}

impl Gpio {
    /// Create new GPIO controller
    pub fn new() -> Self {
        Self {
            port_value: 0x00,
            tris: 0x3F,        // All inputs by default
            weak_pullup: 0x00,  // Pull-ups disabled
            external_pins: 0x3F, // All high by default
            peripheral_output_enable: 0x00,
            peripheral_output_value: 0x00,
        }
    }
    
    /// Reset to power-on state
    pub fn reset(&mut self) {
        self.port_value = 0x00;
        self.tris = 0x3F;       // All inputs
        self.weak_pullup = 0x00;
        self.external_pins = 0x3F;
        self.peripheral_output_enable = 0x00;
        self.peripheral_output_value = 0x00;
    }
    
    /// Write to GPIO register
    pub fn write_gpio(&mut self, value: u8) {
        self.port_value = value & 0x3F;
    }
    
    /// Read from GPIO register
    /// Returns the actual pin states considering direction, pull-ups, and external inputs
    pub fn read_gpio(&self) -> u8 {
        let mut result = 0u8;
        
        for bit in 0..6 {
            let mask = 1 << bit;
            
            // Check if pin is controlled by peripheral
            if self.peripheral_output_enable & mask != 0 {
                // Peripheral controls this pin
                if self.peripheral_output_value & mask != 0 {
                    result |= mask;
                }
            } else if self.tris & mask != 0 {
                // Input mode - read from external pins
                // Apply weak pull-up if enabled
                if self.weak_pullup & mask != 0 {
                    // If external pin is high-Z or high, read as high
                    if self.external_pins & mask != 0 {
                        result |= mask;
                    }
                } else {
                    // No pull-up, read external state directly
                    if self.external_pins & mask != 0 {
                        result |= mask;
                    }
                }
            } else {
                // Output mode - read from port latch
                if self.port_value & mask != 0 {
                    result |= mask;
                }
            }
        }
        
        result
    }
    
    /// Write to TRISIO register (direction control)
    pub fn write_tris(&mut self, value: u8) {
        // GP3 is always input
        self.tris = (value & 0x3F) | 0x08;
    }
    
    /// Read TRISIO register
    pub fn read_tris(&self) -> u8 {
        self.tris
    }
    
    /// Write to WPU (Weak Pull-Up) register
    pub fn write_wpu(&mut self, value: u8) {
        self.weak_pullup = value & 0x37; // GP3 and GP5 don't have pull-ups
    }
    
    /// Read WPU register
    pub fn read_wpu(&self) -> u8 {
        self.weak_pullup
    }
    
    /// Set external pin state (for simulation)
    pub fn set_external_pin(&mut self, pin: u8, state: bool) {
        if pin < 6 {
            if state {
                self.external_pins |= 1 << pin;
            } else {
                self.external_pins &= !(1 << pin);
            }
        }
    }
    
    /// Get external pin state
    pub fn get_external_pin(&self, pin: u8) -> bool {
        if pin < 6 {
            (self.external_pins & (1 << pin)) != 0
        } else {
            false
        }
    }
    
    /// Set all external pins at once
    pub fn set_external_pins(&mut self, value: u8) {
        self.external_pins = value & 0x3F;
    }
    
    /// Get current output values (what would be driven if pins are outputs)
    pub fn get_output_values(&self) -> u8 {
        self.port_value
    }
    
    /// Get pin direction (true = input, false = output)
    pub fn is_input(&self, pin: u8) -> bool {
        if pin < 6 {
            (self.tris & (1 << pin)) != 0
        } else {
            false
        }
    }
    
    /// Get pin state considering all factors
    pub fn get_pin_state(&self, pin: u8) -> PinState {
        if pin >= 6 {
            return PinState::HighZ;
        }
        
        let mask = 1 << pin;
        
        // Check peripheral control first
        if self.peripheral_output_enable & mask != 0 {
            if self.peripheral_output_value & mask != 0 {
                return PinState::High;
            } else {
                return PinState::Low;
            }
        }
        
        // Check if input or output
        if self.tris & mask != 0 {
            // Input mode
            PinState::HighZ
        } else {
            // Output mode
            if self.port_value & mask != 0 {
                PinState::High
            } else {
                PinState::Low
            }
        }
    }
    
    /// Enable peripheral control of a pin (e.g., for comparator output)
    pub fn set_peripheral_control(&mut self, pin: u8, enable: bool, value: bool) {
        if pin < 6 {
            let mask = 1 << pin;
            
            if enable {
                self.peripheral_output_enable |= mask;
                if value {
                    self.peripheral_output_value |= mask;
                } else {
                    self.peripheral_output_value &= !mask;
                }
            } else {
                self.peripheral_output_enable &= !mask;
            }
        }
    }
    
    /// Get a visual representation of the port
    pub fn get_visual_state(&self) -> String {
        let mut result = String::new();
        
        for pin in (0..6).rev() {
            let state = self.get_pin_state(pin);
            let is_input = self.is_input(pin);
            let has_pullup = (self.weak_pullup & (1 << pin)) != 0;
            
            result.push_str(&format!("GP{}: ", pin));
            
            match state {
                PinState::High => result.push_str("HIGH"),
                PinState::Low => result.push_str("LOW "),
                PinState::HighZ => result.push_str("IN  "),
            }
            
            if is_input {
                result.push_str(" [IN");
                if has_pullup {
                    result.push_str(" ↑");
                }
                result.push_str("]");
            } else {
                result.push_str(" [OUT]");
            }
            
            if pin > 0 {
                result.push_str(", ");
            }
        }
        
        result
    }
}

impl Default for Gpio {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpio_creation() {
        let gpio = Gpio::new();
        assert_eq!(gpio.read_tris(), 0x3F); // All inputs
    }
    
    #[test]
    fn test_output_mode() {
        let mut gpio = Gpio::new();
        
        // Set GP0 as output
        gpio.write_tris(0x3E); // GP0 = output, others = input
        
        // Write 1 to GP0
        gpio.write_gpio(0x01);
        
        // Should read back as 1
        assert_eq!(gpio.read_gpio() & 0x01, 0x01);
    }
    
    #[test]
    fn test_input_mode() {
        let mut gpio = Gpio::new();
        
        // Set GP0 as input (default)
        gpio.write_tris(0x3F);
        
        // Set external pin high
        gpio.set_external_pin(0, true);
        
        // Should read high
        assert_eq!(gpio.read_gpio() & 0x01, 0x01);
        
        // Set external pin low
        gpio.set_external_pin(0, false);
        
        // Should read low
        assert_eq!(gpio.read_gpio() & 0x01, 0x00);
    }
    
    #[test]
    fn test_weak_pullup() {
        let mut gpio = Gpio::new();
        
        // Set GP0 as input
        gpio.write_tris(0x3F);
        
        // Enable weak pull-up on GP0
        gpio.write_wpu(0x01);
        
        // External pin floating (high-Z) should read as high
        gpio.set_external_pin(0, true);
        assert_eq!(gpio.read_gpio() & 0x01, 0x01);
    }
    
    #[test]
    fn test_gp3_always_input() {
        let mut gpio = Gpio::new();
        
        // Try to set GP3 as output
        gpio.write_tris(0x00);
        
        // GP3 should still be input
        assert!(gpio.is_input(3));
    }
}