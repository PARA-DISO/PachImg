use image::{DynamicImage, Rgb, RgbImage};
use palette::{convert::IntoColor, Lab, Xyz};
use std::collections::VecDeque;
pub fn cvt_image_with_lut(
    img: &DynamicImage,
    lut: &[[[u8; 3]; 4]],
    force_tiling: bool,
) -> DynamicImage {
    let mut px_ref = VecDeque::<usize>::from([0, 1, 2, 3]);
    // 色の置換処理
    let lambda = |px: &Rgb<u8>| -> ([u8; 6], [u8; 6]) {
        let n = ((px[0] as usize) << 8) | ((px[1] as usize) << 4) | px[2] as usize;
        let [w, x, y, z] = if force_tiling {
            let tmp = [px_ref[0], px_ref[1], px_ref[2], px_ref[3]];
            px_ref[0] = tmp[2];
            px_ref[1] = tmp[3];
            px_ref[2] = tmp[0];
            px_ref[3] = tmp[1];
            tmp
        } else {
            [0, 1, 2, 3]
        };
        let px = {
            let [pixel_a, pixel_b, pixel_c, pixel_d] = [lut[n][0], lut[n][1], lut[n][2], lut[n][3]];
            if force_tiling
                && ((pixel_a[0] + pixel_a[1] * 2 + pixel_a[2] * 4
                    == pixel_b[0] + pixel_b[1] * 2 + pixel_b[2] * 4)
                    || (pixel_c[0] + pixel_c[1] * 2 + pixel_c[2] * 4
                        == pixel_d[0] + pixel_d[1] * 2 + pixel_d[2] * 4))
            {
                [pixel_c, pixel_b, pixel_a, pixel_d]
            } else {
                [pixel_a, pixel_b, pixel_c, pixel_d]
            }
        };
        (
            [px[w][0], px[w][1], px[w][2], px[x][0], px[x][1], px[x][2]],
            [px[y][0], px[y][1], px[y][2], px[z][0], px[z][1], px[z][2]],
        )
    };

    RgbImage::from_vec(img.width() * 2, img.height() * 4, {
        let (odd, even): (Vec<[u8; 6]>, Vec<[u8; 6]>) = if let Some(rgb_image) = img.as_rgb8() {
            rgb_image.pixels().map(lambda).unzip()
        } else {
            img.to_rgb8().pixels().map(lambda).unzip()
        };
        odd.chunks(img.width() as usize)
            .zip(even.chunks(img.width() as usize))
            .flat_map(|(row0, row1)| row0.iter().chain(row0).chain(row1).chain(row1).flatten())
            .map(|x| x * 255)
            .collect::<Vec<u8>>()
    })
    .unwrap()
    .into()
}
pub const fn make_lut() -> [[[u8; 3]; 4]; 4096] {
    let mut i = 0;
    let mut array = [[[0u8; 3]; 4]; 4096];
    while i < 4096 {
        array[i][0][0] = (i & 0b000_000_000_001) as u8;
        array[i][0][1] = ((i & 0b000_000_000_010) >> 1) as u8;
        array[i][0][2] = ((i & 0b000_000_000_100) >> 2) as u8;

        array[i][1][0] = ((i & 0b000_000_001_000) >> 3) as u8;
        array[i][1][1] = ((i & 0b000_000_010_000) >> 4) as u8;
        array[i][1][2] = ((i & 0b000_000_100_000) >> 5) as u8;

        array[i][2][0] = ((i & 0b000_001_000_000) >> 6) as u8;
        array[i][2][1] = ((i & 0b000_010_000_000) >> 7) as u8;
        array[i][2][2] = ((i & 0b000_100_000_000) >> 8) as u8;

        array[i][3][0] = ((i & 0b001_000_000_000) >> 9) as u8;
        array[i][3][1] = ((i & 0b010_000_000_000) >> 10) as u8;
        array[i][3][2] = ((i & 0b100_000_000_000) >> 11) as u8;
        i += 1;
    }
    array
}
pub fn cvt_image_from_full_color(img: &DynamicImage) -> DynamicImage {
    const LUT: [[[u8; 3]; 4]; 4096] = make_lut();
    let mut cache = std::collections::HashMap::<u32, ([u8; 6], [u8; 6])>::new();
    let mut hit = 0usize;
    let mut access = 0usize;
    let lambda = |px: &Rgb<u8>| {
        access += 1;
        let n = ((px[0] as u32) << 16) | ((px[1] as u32) << 8) | px[2] as u32;
        if let Some(data) = cache.get(&n) {
            hit += 1;
            *data
        } else {
            let item = LUT
                .iter()
                .min_by_key(|data| (evaluation(data, &[px[0], px[1], px[2]]) * 65535f32) as usize)
                .unwrap();
            let item = (
                [
                    item[0][0], item[0][1], item[0][2], item[1][0], item[1][1], item[1][2],
                ],
                [
                    item[2][0], item[2][1], item[2][2], item[3][0], item[3][1], item[3][2],
                ],
            );
            cache.insert(n, item);
            item
        }
    };
    let img = RgbImage::from_vec(img.width() * 2, img.height() * 4, {
        let (odd, even): (Vec<[u8; 6]>, Vec<[u8; 6]>) = if let Some(rgb_image) = img.as_rgb8() {
            rgb_image.pixels().map(lambda).unzip()
        } else {
            img.to_rgb8().pixels().map(lambda).unzip()
        };
        odd.chunks(img.width() as usize)
            .zip(even.chunks(img.width() as usize))
            .flat_map(|(row0, row1)| row0.iter().chain(row0).chain(row1).chain(row1).flatten())
            .map(|x| x * 255)
            .collect::<Vec<u8>>()
    })
    .unwrap()
    .into();
    println!(
        "cache access: {access}, hit: {hit}, miss ratio: {}, cache size: {}",
        (access - hit) as f64 / access as f64,
        cache.len()
    );
    img
}
/**
 * 24bitRGB Color Image to 12bitRGB Color Image
 */
pub fn cvt_4bit_color(img: &DynamicImage) -> DynamicImage {
    RgbImage::from_vec(
        img.width(),
        img.height(),
        img.to_rgb8()
            .pixels()
            .flat_map(|px| {
                [
                    ((px[0] as u32 * 0xF + 0x87) >> 8) as u8,
                    ((px[1] as u32 * 0xF + 0x87) >> 8) as u8,
                    ((px[2] as u32 * 0xF + 0x87) >> 8) as u8,
                ]
            })
            .collect::<Vec<u8>>(),
    )
    .unwrap()
    .into()
}

fn evaluation(input: &[[u8; 3]; 4], trg: &[u8; 3]) -> f32 {
    const COEFF: f32 = 255f32;
    // convert color space to l*a*b* from RGB
    let lab = cvt_color_space(&[
        (trg[0] as f32) / COEFF,
        (trg[1] as f32) / COEFF,
        (trg[2] as f32) / COEFF,
    ]);
    // calc tile color
    let mixed = [
        (input[0][0] + input[1][0] + input[2][0] + input[3][0]) as f32 / 4f32,
        (input[0][1] + input[1][1] + input[2][1] + input[3][1]) as f32 / 4f32,
        (input[0][2] + input[1][2] + input[2][2] + input[3][2]) as f32 / 4f32,
    ];

    cvt_color_space(&mixed)
        .iter()
        .zip(lab)
        .fold(0f32, |acc, (s, t)| acc + (s - t).abs())
}
/**
 * conver RGB Color Space to L*a*b* COlor Space
 */
fn cvt_color_space(rgb: &[f32; 3]) -> [f32; 3] {
    let [r, g, b] = [rgb[0], rgb[1], rgb[2]];
    let x = r * 0.4124564 + g * 0.3575761 + b * 0.1804375;
    let y = r * 0.2126729 + g * 0.7151522 + b * 0.0721750;
    let z = r * 0.0193339 + g * 0.1191920 + b * 0.9503041;
    let lab_data: Lab = Xyz::new(x, y, z).into_color();
    let (l, a, b) = lab_data.into_components();
    [l, a, b]
}
