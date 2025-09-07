use image;

fn main() {
    // 画像を読み込む
    let img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        image::open("input/north.png").unwrap().to_rgb8();

    let mut pixels: Vec<bool> = vec![];

    for pixel in img.pixels() {
        pixels.push((pixel[0] as f64 + pixel[1] as f64 + pixel[2] as f64) / 3.0 < 230.0);
    }

    // 画像を保存
    // create_output_image(pixels, img.width(), img.height())
    //     .save_with_format("output/north.png", image::ImageFormat::Png)
    //     .unwrap();

    // コマンドを保存

    std::fs::write(
        "output/wall/data/wall/function/north.mcfunction",
        create_commands(pixels, img.width(), img.height()),
    )
    .unwrap();

    println!("処理が完了しました。");
}

fn create_commands(pixels: Vec<bool>, width: u32, height: u32) -> String {
    let mut commands = String::new();
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) as usize;
            commands.push_str(&format!(
                "setblock ~{x} ~{} ~0 minecraft:{}\n",
                height - y,
                if pixels[index] {
                    "deepslate_tiles"
                } else {
                    "smooth_stone"
                }
            ));
        }
    }
    commands
}

/// 正常に読み込めているかの画像を生成する
fn create_output_image(
    pixels: Vec<bool>,
    width: u32,
    height: u32,
) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let mut output_image = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::new(width, height);

    for (index, pixel) in output_image.pixels_mut().enumerate() {
        let color = if pixels[index] { 0 } else { 255 };
        pixel[0] = color;
        pixel[1] = color;
        pixel[2] = color;
    }

    output_image
}
