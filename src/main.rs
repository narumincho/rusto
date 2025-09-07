use image;

fn main() {
    let input_dir = std::fs::read_dir("./input").unwrap();
    for entry in input_dir {
        let entry = entry.expect("ファイルの取得に失敗しました");
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "png" {
                    if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                        input_output(file_name.to_string());
                    }
                }
            }
        }
    }
    println!("全画像の処理が完了しました。");
}

fn input_output(file_name: String) {
    // 入力画像パスを動的に生成
    let input_path = format!("input/{}.png", file_name);
    let img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::open(&input_path)
        .unwrap_or_else(|_| panic!("{} の読み込みに失敗", input_path))
        .to_rgb8();

    let mut pixels: Vec<bool> = vec![];

    for pixel in img.pixels() {
        pixels.push((pixel[0] as f64 + pixel[1] as f64 + pixel[2] as f64) / 3.0 < 230.0);
    }

    // 画像を保存（必要ならコメント解除）
    // let output_img_path = format!("output/{}.png", file_name);
    // create_output_image(pixels, img.width(), img.height())
    //     .save_with_format(&output_img_path, image::ImageFormat::Png)
    //     .unwrap();

    // コマンドを保存
    let output_cmd_path = format!("output/wall/data/wall/function/{}.mcfunction", file_name);
    std::fs::write(
        &output_cmd_path,
        create_commands(pixels, img.width(), img.height()),
    )
    .unwrap();
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
                    "smooth_stone"
                } else {
                    "deepslate_tiles"
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
