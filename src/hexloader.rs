/// Intel HEX file format loader
/// 
/// Reference: Intel HEX format specification
/// 
/// Intel HEX format consists of ASCII text lines with the format:
/// :LLAAAATT[DD...]CC
/// 
/// LL = byte count (number of data bytes)
/// AAAA = address (16-bit)
/// TT = record type (00=data, 01=EOF, 04=extended address, etc.)
/// DD = data bytes
/// CC = checksum

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Record types in Intel HEX format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordType {
    Data = 0x00,
    EndOfFile = 0x01,
    ExtendedSegmentAddress = 0x02,
    StartSegmentAddress = 0x03,
    ExtendedLinearAddress = 0x04,
    StartLinearAddress = 0x05,
}

impl RecordType {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(RecordType::Data),
            0x01 => Some(RecordType::EndOfFile),
            0x02 => Some(RecordType::ExtendedSegmentAddress),
            0x03 => Some(RecordType::StartSegmentAddress),
            0x04 => Some(RecordType::ExtendedLinearAddress),
            0x05 => Some(RecordType::StartLinearAddress),
            _ => None,
        }
    }
}

/// A single record from an Intel HEX file
#[derive(Debug, Clone)]
pub struct HexRecord {
    pub byte_count: u8,
    pub address: u16,
    pub record_type: RecordType,
    pub data: Vec<u8>,
    pub checksum: u8,
}

impl HexRecord {
    /// Parse a single line of HEX format
    pub fn parse(line: &str) -> Result<Self, String> {
        let line = line.trim();
        
        // Must start with ':'
        if !line.starts_with(':') {
            return Err("HEX line must start with ':'".to_string());
        }
        
        // Remove the ':' prefix
        let line = &line[1..];
        
        // Must have even number of hex digits
        if line.len() % 2 != 0 {
            return Err("HEX line must have even number of characters".to_string());
        }
        
        // Parse bytes
        let mut bytes = Vec::new();
        for i in (0..line.len()).step_by(2) {
            let byte_str = &line[i..i+2];
            let byte = u8::from_str_radix(byte_str, 16)
                .map_err(|_| format!("Invalid hex byte: {}", byte_str))?;
            bytes.push(byte);
        }
        
        // Must have at least 5 bytes (count, addr_hi, addr_lo, type, checksum)
        if bytes.len() < 5 {
            return Err("HEX line too short".to_string());
        }
        
        let byte_count = bytes[0];
        let address = ((bytes[1] as u16) << 8) | (bytes[2] as u16);
        let record_type = RecordType::from_u8(bytes[3])
            .ok_or_else(|| format!("Invalid record type: 0x{:02X}", bytes[3]))?;
        
        // Data bytes
        let data_end = 4 + byte_count as usize;
        if bytes.len() != data_end + 1 {
            return Err(format!("Byte count mismatch: expected {}, got {}", 
                byte_count, bytes.len() - 5));
        }
        
        let data = bytes[4..data_end].to_vec();
        let checksum = bytes[data_end];
        
        // Verify checksum
        let calculated_checksum = Self::calculate_checksum(&bytes[0..data_end]);
        if calculated_checksum != checksum {
            return Err(format!("Checksum mismatch: expected 0x{:02X}, got 0x{:02X}",
                calculated_checksum, checksum));
        }
        
        Ok(HexRecord {
            byte_count,
            address,
            record_type,
            data,
            checksum,
        })
    }
    
    /// Calculate checksum for a sequence of bytes
    fn calculate_checksum(bytes: &[u8]) -> u8 {
        let sum = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        0u8.wrapping_sub(sum)
    }
}

/// Loaded program data
#[derive(Debug, Clone)]
pub struct HexProgram {
    /// Program memory (14-bit instructions)
    pub program: Vec<u16>,
    
    /// EEPROM data (if present)
    pub eeprom: Vec<u8>,
    
    /// Configuration word (if present)
    pub config: Option<u16>,
    
    /// Start address
    pub start_address: u16,
}

/// HEX file loader
pub struct HexLoader;

impl HexLoader {
    /// Load a HEX file from a path
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<HexProgram, String> {
        let file = File::open(path.as_ref())
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            lines.push(line);
        }
        
        Self::load_from_lines(&lines)
    }
    
    /// Load a HEX file from a string
    pub fn load_from_string(content: &str) -> Result<HexProgram, String> {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self::load_from_lines(&lines)
    }
    
    /// Load from a collection of lines
    fn load_from_lines(lines: &[String]) -> Result<HexProgram, String> {
        let mut program_bytes: Vec<u8> = Vec::new();
        let mut max_address = 0u32;
        let mut extended_address = 0u32;
        let mut eeprom_data = Vec::new();
        let mut config_word = None;
        
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            
            // Parse the record
            let record = HexRecord::parse(line)
                .map_err(|e| format!("Line {}: {}", line_num + 1, e))?;
            
            match record.record_type {
                RecordType::Data => {
                    // Calculate absolute address
                    let abs_address = extended_address + record.address as u32;
                    
                    // Determine if this is program memory, EEPROM, or config
                    if abs_address >= 0x2100 && abs_address < 0x2180 {
                        // EEPROM data (0x2100-0x217F)
                        let eeprom_addr = (abs_address - 0x2100) as usize;
                        
                        // Expand EEPROM buffer if needed
                        if eeprom_addr + record.data.len() > eeprom_data.len() {
                            eeprom_data.resize(eeprom_addr + record.data.len(), 0xFF);
                        }
                        
                        // Copy data
                        for (i, &byte) in record.data.iter().enumerate() {
                            eeprom_data[eeprom_addr + i] = byte;
                        }
                    } else if abs_address == 0x2007 {
                        // Configuration word
                        if record.data.len() >= 2 {
                            config_word = Some(
                                (record.data[0] as u16) | ((record.data[1] as u16) << 8)
                            );
                        }
                    } else {
                        // Program memory
                        let prog_addr = abs_address as usize;
                        
                        // Expand program buffer if needed
                        if prog_addr + record.data.len() > program_bytes.len() {
                            program_bytes.resize(prog_addr + record.data.len(), 0xFF);
                        }
                        
                        // Copy data
                        for (i, &byte) in record.data.iter().enumerate() {
                            program_bytes[prog_addr + i] = byte;
                        }
                        
                        max_address = max_address.max(abs_address + record.data.len() as u32);
                    }
                }
                
                RecordType::EndOfFile => {
                    // End of file - we're done
                    break;
                }
                
                RecordType::ExtendedLinearAddress => {
                    // Extended linear address (upper 16 bits)
                    if record.data.len() >= 2 {
                        extended_address =
                            (((record.data[0] as u32) << 8) | (record.data[1] as u32)) << 16;
                    }
                }

                RecordType::ExtendedSegmentAddress => {
                    // Extended segment address (upper 16 bits shifted by 4)
                    if record.data.len() >= 2 {
                        extended_address =
                            (((record.data[0] as u32) << 8) | (record.data[1] as u32)) << 4;
                    }
                }

                _ => {
                    // Ignore other record types
                }
            }
        }
        
        // Convert bytes to 14-bit words for program memory
        // PIC uses little-endian: low byte first, then high byte
        let mut program = Vec::new();
        for i in (0..program_bytes.len()).step_by(2) {
            if i + 1 < program_bytes.len() {
                let low = program_bytes[i] as u16;
                let high = program_bytes[i + 1] as u16;
                let word = low | (high << 8);
                program.push(word & 0x3FFF); // Mask to 14 bits
            } else {
                // Odd number of bytes - pad with 0xFF
                program.push(program_bytes[i] as u16);
            }
        }
        
        Ok(HexProgram {
            program,
            eeprom: eeprom_data,
            config: config_word,
            start_address: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_data_record() {
        let line = ":10000000FF3E00008F0E0000000000000000000016";
        let record = HexRecord::parse(line).unwrap();
        
        assert_eq!(record.byte_count, 0x10);
        assert_eq!(record.address, 0x0000);
        assert_eq!(record.record_type, RecordType::Data);
        assert_eq!(record.data.len(), 16);
        assert_eq!(record.data[0], 0xFF);
        assert_eq!(record.data[1], 0x3E);
    }
    
    #[test]
    fn test_parse_eof_record() {
        let line = ":00000001FF";
        let record = HexRecord::parse(line).unwrap();
        
        assert_eq!(record.byte_count, 0);
        assert_eq!(record.record_type, RecordType::EndOfFile);
    }
    
    #[test]
    fn test_checksum_verification() {
        // Valid checksum
        let line = ":10000000FF3E00008F0E0000000000000000000016";
        assert!(HexRecord::parse(line).is_ok());
        
        // Invalid checksum
        let line = ":10000000FF3E00008F0E00000000000000000000E3";
        assert!(HexRecord::parse(line).is_err());
    }
    
    #[test]
    fn test_load_simple_program() {
        let hex = r#"
:020000040000FA
:02000000553079
:020002002000DC
:00000001FF
"#;
        
        let program = HexLoader::load_from_string(hex).unwrap();
        assert_eq!(program.program.len(), 2);
        assert_eq!(program.program[0], 0x3055); // MOVLW 0x55
        assert_eq!(program.program[1], 0x0020); // MOVWF 0x20
    }
}