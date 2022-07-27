#[repr(C, align(4))]
// This gets written to DMAC_DESC_ADDR_REGN in a funky way
pub struct Descriptor {
    configuration: u32,
    source_address: u32,
    destination_address: u32,
    byte_counter: u32,
    parameter: u32,
    link: u32,
}

impl Descriptor {
    pub fn set_source(&mut self, source: u64) {
        assert!(source < (1 << 34));
        self.source_address = source as u32;
        //                  332222222222 11 11 11111100 00000000
        //                  109876543210 98 76 54321098 76543210
        self.parameter &= 0b111111111111_11_00_11111111_11111111;
        self.parameter |= (((source >> 32) & 0b11) << 16) as u32;
    }

    pub fn set_dest(&mut self, dest: u64) {
        assert!(dest < (1 << 34));
        self.destination_address = dest as u32;
        //                  332222222222 11 11 11111100 00000000
        //                  109876543210 98 76 54321098 76543210
        self.parameter &= 0b111111111111_00_11_11111111_11111111;
        self.parameter |= (((dest >> 32) & 0b11) << 18) as u32;
    }

    pub fn end_link(&mut self) {
        self.link = 0xFFFF_F800;
    }
}

pub struct DescriptorConfig {
    pub source: *const (),
    pub destination: *mut (),

    // NOTE: Max is < 2^25, or < 32MiB
    pub byte_counter: usize,
    pub link: Option<*const ()>,
    pub wait_clock_cycles: u8,

    pub bmode: BModeSel,

    pub dest_width: DataWidth,
    pub dest_addr_mode: AddressMode,
    pub dest_block_size: BlockSize,
    pub dest_drq_type: DestDrqType,

    pub src_data_width: DataWidth,
    pub src_addr_mode: AddressMode,
    pub src_block_size: BlockSize,
    pub src_drq_type: SrcDrqType,
}

#[repr(u8)]
pub enum SrcDrqType {
    Sram = 0,
    Dram = 1,
    OwaRx = 2,
    I2sPcm0Rx = 3,
    I2sPcm1Rx = 4,
    I2sPcm2Rx = 5,
    AudioCodec = 7,
    Dmic = 8,
    GpADC = 12,
    TpADC = 13,
    Uart0Rx = 14,
    Uart1Rx = 15,
    Uart2Rx = 16,
    Uart3Rx = 17,
    Uart4Rx = 18,
    Uart5Rx = 19,
    Spi0Rx = 22,
    Spi1Rx = 23,
    Usb0Ep1 = 30,
    Usb0Ep2 = 31,
    Usb0Ep3 = 32,
    Usb0Ep4 = 33,
    Usb0Ep5 = 34,
    Twi0 = 43,
    Twi1 = 44,
    Twi2 = 45,
    Twi3 = 46,
}

#[repr(u8)]
pub enum DestDrqType {
    Sram = 0,
    Dram = 1,
    OwaTx = 2,
    I2sPcm0Tx = 3,
    I2sPcm1Tx = 4,
    I2sPcm2Tx = 5,
    AudioCodec = 7,
    IrTx = 13,
    Uart0Tx = 14,
    Uart1Tx = 15,
    Uart2Tx = 16,
    Uart3Tx = 17,
    Uart4Tx = 18,
    Uart5Tx = 19,
    Spi0Tx = 22,
    Spi1Tx = 23,
    Usb0Ep1 = 30,
    Usb0Ep2 = 31,
    Usb0Ep3 = 32,
    Usb0Ep4 = 33,
    Usb0Ep5 = 34,
    Ledc = 42,
    Twi0 = 43,
    Twi1 = 44,
    Twi2 = 45,
    Twi3 = 46,
}

// TODO: Verify bits or bytes?
pub enum BlockSize {
    Byte1,
    Byte4,
    Byte8,
    Byte16
}

pub enum AddressMode {
    LinearMode,
    IoMode,
}

pub enum DataWidth {
    Bit8,
    Bit16,
    Bit32,
    Bit64,
}

pub enum BModeSel {
    Normal,
    BMode,
}

// descriptor.set_source(chunk.as_ptr() as usize as u64);
// descriptor.set_dest(thr_addr);
// descriptor.byte_counter = chunk.len() as u32;

// // I think? DMAC_CFG_REGN
// descriptor.configuration = 0;
// descriptor.configuration |= 0b0 << 30;  // BMODE_SEL: Normal
// descriptor.configuration |= 0b00 << 25; // DEST_WIDTH: 8-bit
// descriptor.configuration |= 0b1 << 24;  // DMA_ADDR_MODE: Dest IO Mode
// descriptor.configuration |= 0b00 << 22; // Dest block size: 1
// descriptor.configuration |= 0b001110 << 16; // !!! Dest DRQ Type - UART0
// descriptor.configuration |= 0b00 << 9; // Source width 8 bit
// descriptor.configuration |= 0b0 << 8; // Source Linear Mode
// descriptor.configuration |= 0b00 << 6; // Source block size 1
// descriptor.configuration |= 0b000001 << 0; // Source DRQ type - DRAM

// descriptor.end_link();
