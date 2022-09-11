pub struct W3Crc {
    table: [u32; 256],
}
impl W3Crc {
    pub fn make_crc_table() -> Self {
        let mut table = W3Crc { table: [0; 256] };
        let mut c: u32;

        for n in 0..256 {
            c = n;
            for _ in 0..8 {
                if c % 2 == 1 {
                    c = 0xedb88320 ^ (c >> 1);
                } else {
                    c = c >> 1;
                }
            }
            table.table[n as usize] = c;
        }
        table
    }
    pub fn update_crc(&self, crc: u32, buf: &[u8]) -> u32 {
        let mut c = crc;
        for n in 0..buf.len() {
            c = self.table[((c ^ buf[n as usize] as u32) & 0xff) as usize] ^ (c >> 8);
        }
        c
    }
    pub fn crc(&self, buf: &[u8]) -> u32 {
        self.update_crc(0xffffffff, buf) ^ 0xffffffff
    }
}
