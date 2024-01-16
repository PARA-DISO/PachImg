///
/// `pachimg` is a CLI Application to convert image to 3bit color.
/// 
mod cvt_image;
use clap::Parser;
use cvt_image::{cvt_4bit_color, cvt_image_from_full_color, cvt_image_with_lut};
use image::DynamicImage;
use std::fs::File;
use std::io::BufReader;
/// Image Conversion to 3bit color
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input image file Name
    #[arg(short, long)]
    input: String,
    /// Output image file name [default: output.png]
    #[arg(short, long, default_value_t = String::from("output.png"))]
    output: String,
    /// Keep original image size [default: false]
    #[arg(long, default_value_t = false)]
    original_size: bool,
    /// Operate so that it is tiled as much as possible [default: false]
    #[arg(long, default_value_t = false)]
    force_tiling: bool,
    /// Lookup table file name
    #[arg(short, long, default_value_t = String::from(""))]
    lookup: String,
}
fn main() {
    let args = Args::parse();
    // 画像の縮小(幅1/2, 高さ1/4)
    let image = {
        let img = image::open(args.input.as_str()).unwrap();
        if args.original_size {
            // 高さ 1/4 幅1/2に縮小
            DynamicImage::from(
                img.resize_exact(
                    img.width() / 2,
                    img.height() / 4,
                    image::imageops::FilterType::Lanczos3,
                )
                .to_rgb8(),
            )
        } else {
            // 画像サイズをPC88の画面サイズに合わせる
            // FIXME: w == h の画像でアスペクト比が崩れる
            DynamicImage::from(
                if img.width() <= img.height() {
                    let tmp = img.resize(640, 400, image::imageops::FilterType::Lanczos3);
                    tmp.resize_exact(tmp.width() / 2, 100, image::imageops::FilterType::Lanczos3)
                } else {
                    let tmp = img.resize(640, 400, image::imageops::FilterType::Lanczos3);
                    tmp.resize_exact(320, tmp.height() / 4, image::imageops::FilterType::Lanczos3)
                }
                .to_rgb8(),
            )
        }
    };

    if !args.lookup.is_empty() {
        // LUTを使う
        let lut: Vec<[[u8; 3]; 4]> = {
            // LUTのロード
            let file = File::open(args.lookup.as_str()).unwrap();
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        };
        // 減色処理
        let cvt_color_image = cvt_4bit_color(&image);
        // 画層変換 & 書き出し
        let _ = cvt_image_with_lut(&cvt_color_image, &lut, args.force_tiling)
            .save(args.output.as_str());
    } else {
        // 画層変換 & 書き出し
        let _ = cvt_image_from_full_color(&image).save(args.output.as_str());
    }
}
