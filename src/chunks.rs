
use std::mem::MaybeUninit;
use crate::w3crc::W3Crc;
use bit_vec::BitVec;
use rgb::RGBA;

pub fn make_ihdr(width: u32, height:u32, bit_depth: u8) -> Chunk {
    let mut chunk = Chunk::new(*b"IHDR");
    let (colour_type, compression_method, filter_method, interlace_method)
    = (3, 0, 0, 0);

    chunk.insert_u32(width);
    chunk.insert_u32(height);
    chunk.insert_bytes(&[
        bit_depth, 
        colour_type, 
        compression_method, 
        filter_method, 
        interlace_method]);

    chunk
}
pub fn make_plte(palette: &[RGBA<u8>]) -> Chunk {
    let mut chunk = Chunk::new(*b"PLTE");
    for color in palette {
        chunk.insert_bytes(&[color.r, color.g, color.b]);
    }
    chunk
}
pub fn make_idat(scanline: u32, bits: Vec<MaybeUninit<u8>>, bit_depth: u8, crc: &W3Crc) -> Chunk {
    let mut chunk = Chunk::new(*b"IDAT");

    let mut count = 0;
    let mut vec= BitVec::new();
    for bit in bits {
        if count == 0 {
            chunk.insert_u8(0);
            vec = BitVec::new();
        }
        push_bits(&mut vec, bit, bit_depth);
        count += 1;
        if count >= scanline {
            count = 0;
            chunk.insert_bytes(&vec.to_bytes());
        }
    }
    chunk.deflate_encode(crc);
    
    chunk
}
fn push_bits(vec: &mut BitVec, value: MaybeUninit<u8>, bit_depth: u8) {
    unsafe {
        let current= value.assume_init();
        for i in (0..bit_depth).rev() {
            vec.push((current >> i) % 2 == 1);
        }
    }
}
pub struct Chunk {
    name: [u8; 4],
    bit: Vec<u8>,
}

impl Chunk {
    pub fn new(name: [u8; 4]) -> Self {
        Chunk {
            name,
            bit: Vec::new(),
        }
    }
    pub fn data(&self, crc: &W3Crc) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        result.append(&mut (self.bit.len() as u32).to_be_bytes().to_vec());
        result.append(&mut self.name.to_vec());
        result.append(&mut self.bit.to_owned());
        let merge: Vec<u8> = 
            self.name
                .to_vec().into_iter().chain(
            self.bit
                .to_owned().into_iter())
            .collect();
        result.append(&mut 
            crc.crc(&merge)
            .to_be_bytes().to_vec());

        result
    }
    pub fn insert_u8(&mut self, data: u8) {
        self.bit.push(data);
    }
    pub fn insert_u32(&mut self, data: u32) {
        self.bit.append(&mut data.to_be_bytes().to_vec());
    }
    pub fn insert_bytes(&mut self, data: &[u8]) {
        for d in data {
            self.insert_u8(*d);
        }
    }
    pub fn deflate_encode(&mut self, crc: &W3Crc) {
        let (cmf, flg)
            = (0x78, 0xDA);
        let mut compressed = deflate::deflate_bytes(&self.bit);
        let hash = crc.crc(&self.bit);
        self.bit = Vec::new();
        self.insert_bytes(&[cmf, flg]);
        self.bit.append(&mut compressed);
        self.insert_u32(hash);
    }
}
