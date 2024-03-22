// CPU 6802 Flags

#[derive(Debug, PartialEq)]
pub enum Flag {
    Carry = 0b0000_0001,
    Zero = 0b0000_0010,
    Interrupt = 0b0000_0100,
    Decimal = 0b0000_1000,
    Break = 0b0001_0000,
    Unused = 0b0010_0000,
    Overflow = 0b0100_0000,
    Negative = 0b1000_0000,
}

enum AddressingMode {
    // todo !!
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8,
    pub program_counter: u16,
    pub memory: [u8; 65536],
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 65536],
        }
    }

    fn read_memory(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    fn load_program_into_memory(&mut self, program: Vec<u8>) {
        for (i, &byte) in program.iter().enumerate() {
            self.write_memory(i as u16, byte);
        }
    }

    fn update_zero_and_negative_flag(&mut self) {
        // set Zero flag to register A
        // if register A is = 0b0000_0000
        if self.register_a == 0 {
            // Status set as Zero flag (0b0000_0010)
            self.status |= Flag::Zero as u8;
        } else {
            // if register A != 0
            // clear zero flag by using bitwise AND
            self.status &= !(Flag::Zero as u8);
        }

        // if register A is = 0b11100000
        // compare register A with 0b1000_0000
        // 0b1110_0000 & 0b1000_0000 = 0b1000_0000 (128) Negative flag
        // Check MSB (Most Significant Bit) register A set to 1
        // Register A AND Negative Flag != 0 or result equal to 128 (0b1000_0000)
        if self.register_a & Flag::Negative as u8 != 0 {
            // Status set as Negative flag
            // Register Status now is for example in zero flag
            // Status = 0b000_0010
            // set negative flag
            // now status is 0b1000_0010
            self.status |= Flag::Negative as u8;
        } else {
            // if current accumulator is not negative
            // clear negative flag by using bitwise AND
            // and flip the negative flag to 0b0111_1111
            // Status = 0b000_0010 (zero flag)
            // 0b0000_0010 & 0b0111_1111 = 0b0000_0010
            self.status &= !(Flag::Negative as u8);
        }
    }

    fn lda(&mut self) {
        let operand = self.fetch_byte();
        self.register_a = operand;
        self.update_zero_and_negative_flag();
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.read_memory(self.program_counter);
        self.program_counter += 1;
        byte
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.load_program_into_memory(program);
        self.program_counter = 0;
        loop {
            let opscode = self.memory[self.program_counter as usize];
            // get opscode and go to next instruction
            self.program_counter += 1;

            match opscode {
                // -----------------------------
                // LOAD / STORE Operations
                // LDA, LDX, LDY , STA, STX, STY
                // -----------------------------

                // LDA (Load Accumulator)

                // Addresing mode

                // Immediate (0xA9)
                0xA9 => {
                    self.lda();
                }

                // Zero Page (0xA5)
                0xA5 => {
                    self.lda();
                }
                // END OF LOAD / STORE Operations
                // 0xAA => {
                //     self.register_x = self.register_a;
                //     if self.register_x == 0 {
                //         self.status = self.status | 0b0000_0010;
                //     } else {
                //         self.status = self.status & 0b1111_1101;
                //     }

                //     if self.register_x & 0b1000_0000 != 0 {
                //         self.status = self.status | 0b1000_0000;
                //     } else {
                //         self.status = self.status & 0b0111_1111;
                //     }
                // }
                0x00 => {
                    println!("BRK");
                    break;
                }
                _ => {
                    println!("Unrecognized opscode: {:x}", opscode);
                    break;
                }
            }
        }
    }
}

// -----------------------------
// TEST Section
// -----------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 5); // load 5 to register 5
                                       // The Zero flag should be clear
        assert_eq!(cpu.status & 0b0000_0010, 0);
        assert_eq!(cpu.status & 0b1000_0000, 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.status, 2); // 2 is zero flag 0b10
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xE0, 0x00]);

        assert_eq!(cpu.register_a, 224);
        assert_eq!(cpu.status, 128) //
    }
}
