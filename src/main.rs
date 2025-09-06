use image;

// 画像を読み込む
fn main() {
    // 画像を読み込む
    let img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        image::open("input/north.png").unwrap().to_rgb8();

    let mut pixels: Vec<bool> = vec![];

    for pixel in img.pixels() {
        pixels.push((pixel[0] as f64 + pixel[1] as f64 + pixel[2] as f64) / 3.0 < 230.0);
    }

    let mut output_image =
        image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::new(img.width(), img.height());

    for (index, pixel) in output_image.pixels_mut().enumerate() {
        let color = if pixels[index] { 0 } else { 255 };
        pixel[0] = color;
        pixel[1] = color;
        pixel[2] = color;
    }

    // 画像を保存
    output_image
        .save_with_format("output/north.png", image::ImageFormat::Png)
        .unwrap();

    println!("処理が完了しました。");
}
