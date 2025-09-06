use image::{DynamicImage, GenericImageView, ImageFormat, io::Reader as ImageReader};
use std::path::Path;

// 画像を読み込む
fn main() {
    // 画像を読み込む
    let img: DynamicImage = image::open("input/north.png").unwrap();

    // グレイスケールに変換
    let gray_img: DynamicImage = img.grayscale();

    // RGBA画像に変換し、透明度を1.0（不透明）に設定
    let mut rgba_img = gray_img.to_rgba8();
    for pixel in rgba_img.pixels_mut() {
        pixel[3] = 255; // アルファチャンネルを255（不透明）に設定
    }

    // 画像を保存
    rgba_img
        .save_with_format("output/north.png", ImageFormat::Png)
        .unwrap();

    println!("処理が完了しました。");
}
