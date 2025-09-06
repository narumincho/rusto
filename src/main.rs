use std::boxed::box_new;

use image::{self, Pixel};

// 画像を読み込む
fn main() {
    // 画像を読み込む
    let img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        image::open("input/north.png").unwrap().to_rgb8();

    let mut pixels: Vec<bool> = vec![];

    for pixel in img.pixels() {
        pixels
            .push(pixel[3] > 0 && (pixel[0] as u16 + pixel[1] as u16 + pixel[2] as u16) / 3 < 230);
    }

    let mut output_image = image::ImageBuffer::new(img.width(), img.height());

    for (index, pixel) in output_image.pixels_mut().enumerate() {
        let value = pixels[index];
        pixel[0] = if value { 0 } else { 255 };
        pixel[1] = if value { 0 } else { 255 };
        pixel[2] = if value { 0 } else { 255 };
        pixel[3] = 255;
    }

    // 画像を保存
    output_image
        .save_with_format("output/north.png", image::ImageFormat::Png)
        .unwrap();

    println!("処理が完了しました。");
}
