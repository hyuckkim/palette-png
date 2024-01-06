extern crate console_error_panic_hook;
use std::mem::MaybeUninit;

use imagequant::{Attributes, RGBA};
use js_sys::Uint8ClampedArray;
use rgb::*;
use wasm_bindgen::prelude::*;

mod chunks;
mod w3crc;

#[wasm_bindgen]
extern "C" {}
#[wasm_bindgen]
pub fn ten() -> i32 {
    10
}
#[wasm_bindgen]
pub fn quantize(
    rawimage: Uint8ClampedArray,
    image_width: usize,
    image_height: usize,
    num_color: u32,
    dithering: f32,
    gamma: f64,
) -> Uint8ClampedArray {
    console_error_panic_hook::set_once();

    let imgvec = rawimage.to_vec();
    let image_buffer: &[RGBA] = imgvec.as_rgba();
    let size = image_width * image_height;

    let attr = set_attritubes(num_color);

    let mut img = attr
        .new_image(image_buffer, image_width, image_height, gamma)
        .unwrap();
    let mut res = attr.quantize(&mut img).unwrap();
    res.set_dithering_level(dithering).unwrap();

    let mut image8bit: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); size];
    res.remap_into(&mut img, &mut image8bit).unwrap();
    let pal = res.palette();

    let png = build_png(image_width as u32, image_height as u32, pal, image8bit);
    packaging(png)
}
fn set_attritubes(num_color: u32) -> Attributes {
    let mut attr = Attributes::new();
    attr.set_max_colors(num_color).unwrap();
    attr.set_quality(0, 80).unwrap();
    attr
}
fn build_png(width: u32, height: u32, pal: &[RGBA], bits: Vec<MaybeUninit<u8>>) -> Vec<u8> {
    let mut result: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
    let bit_depth = get_bit_depth_by(pal.len());

    let crc = w3crc::W3Crc::make_crc_table();
    result.append(&mut chunks::make_ihdr(width, height, bit_depth).data(&crc));
    result.append(&mut chunks::make_plte(pal).data(&crc));
    result.append(&mut chunks::make_idat(width, bits, bit_depth).data(&crc));

    let iend = chunks::Chunk::new(*b"IEND");
    result.append(&mut iend.data(&crc));

    result
}

fn packaging(vec: Vec<u8>) -> Uint8ClampedArray {
    let array = Uint8ClampedArray::new_with_length(vec.len() as u32);
    for i in 0..vec.len() {
        array.set_index(i as u32, vec[i]);
    }
    array
}
fn get_bit_depth_by(palettes: usize) -> u8 {
    match palettes {
        0 => panic!("palette's size cannot zero"),
        1..=2 => 1,
        3..=4 => 2,
        5..=16 => 4,
        _ => 8,
    }
}
#[wasm_bindgen]
pub fn read_palette(data: Uint8ClampedArray) -> Uint8ClampedArray {
    let datavec = data.to_vec();

    let mut i: usize = 8;
    if datavec[0..8] != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        Uint8ClampedArray::new_with_length(0)
    } else {
        loop {
            let length = merge_to_u32(&datavec[i..i + 4]).unwrap();
            i += 4;

            if &datavec[i..i + 4] == b"IDAT" {
                break Uint8ClampedArray::new_with_length(0);
            }
            if &datavec[i..i + 4] == b"PLTE" {
                i += 4;
                let colors = Uint8ClampedArray::new_with_length(length);
                for j in 0..length {
                    colors.set_index(j, datavec[i + j as usize]);
                }
                break colors;
            }
            i += 4 + length as usize + 4;
        }
    }
}
#[wasm_bindgen]
pub fn change_palette(
    data: Uint8ClampedArray,
    index: u8,
    r: u8,
    g: u8,
    b: u8,
) -> Uint8ClampedArray {
    let datavec = data.to_vec();

    let mut i: usize = 8;
    if datavec[0..8] != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        Uint8ClampedArray::new_with_length(0)
    } else {
        loop {
            let length = merge_to_u32(&datavec[i..i + 4]).unwrap();
            i += 4;

            if &datavec[i..i + 4] == b"IDAT" {
                break Uint8ClampedArray::new_with_length(0);
            }
            if &datavec[i..i + 4] == b"PLTE" {
                let mut newvec = datavec[i..i + length as usize + 4].to_vec();
                let j = 4 + (index * 3) as usize;
                newvec[j + 0] = r;
                newvec[j + 1] = g;
                newvec[j + 2] = b;
                let result = Uint8ClampedArray::new_with_length(datavec.len() as u32);
                for ii in 0..i {
                    result.set_index(ii as u32, datavec[ii]);
                }
                for ii in 0..newvec.len() {
                    result.set_index((i + ii) as u32, newvec[ii]);
                }
                let crc = w3crc::W3Crc::make_crc_table();
                let crc = crc.crc(&newvec).to_be_bytes();
                for ii in 0..4 {
                    result.set_index((i + newvec.len() + ii) as u32, crc[ii]);
                }
                for ii in (i + newvec.len() + 4)..datavec.len() {
                    result.set_index(ii as u32, datavec[ii]);
                }
                break result;
            }
            i += 4 + length as usize + 4;
        }
    }
}
fn merge_to_u32(data: &[u8]) -> Option<u32> {
    if data.len() > 4 {
        None
    } else {
        let mut result: u32 = 0;

        for i in data {
            result = result * 256 + *i as u32;
        }
        Some(result)
    }
}
