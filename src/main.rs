use std::ops::Add;

use image::imageops;
use image_sender_test::data::image::{Image, Tile};
use rand::RngExt;

fn main() {
    let source_image = Image::load("input4k.png");

    let width = source_image.width();
    let height = source_image.height();

    let tile_size = 128;
    let tiles_per_packet = 10;
    let jpeg_quality = 75;
    let base_scale = 0.5;
    let base_quality = 15;

    let affine_a = 13;
    let affine_b = 5;

    let filter = image::imageops::FilterType::Triangle;

    let packet_loss_chance = 0.2;

    let base_width = (width as f32 * base_scale) as u32;
    let base_height = (height as f32 * base_scale) as u32;

    let source_base = source_image.clone()
        .resize(base_width, base_height, filter)
        .into_jpeg_bytes(base_quality);

    let packets: Vec<_> = source_image
        .into_tiles(tile_size, tile_size)
        .into_iter()
        .map(|t| Tile::<Vec<u8>> { 
            coords: t.coords, 
            image: t.image.into_jpeg_bytes(jpeg_quality) })
        .collect::<Vec<_>>()
        .chunks(tiles_per_packet)
        .map(|c| c.to_vec())
        .enumerate()
        .collect();

    let base_size = source_base.len();
    let total_size = packets.iter().fold(0, |sum, (_, v)| 
        sum + 2 + v.iter().fold(0, |sum2, t| 
            sum2 + t.image.len()
        )
    )
    .add(base_size);

    println!("Base size: {base_size} bytes");
    println!("Total size: {total_size} bytes");

    let original_packets_num = packets.len();

    let packets = lose_packets(packets, packet_loss_chance);

    let remaining_packets_num = packets.len();
    let lost_packets_num = original_packets_num - remaining_packets_num;

    println!("Original packets: {original_packets_num}");
    println!("Remaining packets: {remaining_packets_num}");
    println!("Lost packets: {lost_packets_num}");

    let result_tiles: Vec<_> = packets
        .into_iter()
        .map(|(_, t)| t)
        .flatten()
        .map(|t| Tile { coords: t.coords, image: Image::from_jpeg_bytes(t.image) })
        .collect();

    let mut result_image = Image::from_jpeg_bytes(source_base).resize(width, height, filter);
    let result_top = Image::from_tiles(width, height, tile_size, tile_size, result_tiles);

    imageops::overlay(result_image.inner_mut(), result_top.inner(), 0, 0);

    result_image.save("output4k.png");
}

fn lose_packets<T>(input: Vec<T>, chance_dropped: f64) -> Vec<T> {    
    let mut rng = rand::rng();
    input
        .into_iter()
        .filter(|_| {
            let roll = rng.random_range(0.0..1.0);
            roll >= chance_dropped
        })
        .collect()
}

fn permute(x: u32, n: u32, a: u32, b: u32) -> u32 {
    let m = n + 1;
    (a * x + b) % m
}