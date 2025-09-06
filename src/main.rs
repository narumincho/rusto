
use std::path::Path;
use image::{DynamicImage, GenericImageView, ImageFormat, io::Reader as ImageReader};

fn main() {
    // 入力画像のパス
    let input_path = Path::new("input/north-input.png");
    // 出力画像のパス
    let output_path = Path::new("output/north-input-gray.png");

    // 画像を読み込む
    let img = ImageReader::open(input_path)
        .expect("画像ファイルが開けません")
        .decode()
        .expect("画像のデコードに失敗しました");

    // グレイスケール化
    let gray_img = img.grayscale();

    // 出力フォルダがなければ作成
    std::fs::create_dir_all("output").expect("outputフォルダの作成に失敗しました");

    // グレイスケール画像を保存
    gray_img.save_with_format(output_path, ImageFormat::Png)
        .expect("画像の保存に失敗しました");

    println!("グレイスケール画像をoutputフォルダに保存しました。");
}
