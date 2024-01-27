use image::DynamicImage;
struct SigmoidParams {
    pub gain: f32,
    pub mid: f32,
}
impl SigmoidParams {
    pub fn new(gain: f32, mid: f32) -> Self {
        Self { gain, mid }
    }
}
pub struct CorrectionParams {
    gamma: Option<f32>,
    sigmoid: Option<SigmoidParams>,
}
impl From<&crate::Args> for CorrectionParams {
    fn from(args: &crate::Args) -> Self {
        let gamma = if args.gamma_correction {
            Some(args.gamma)
        } else {
            None
        };
        let sigmoid = if args.sigmoid_correction {
            Some(SigmoidParams::new(args.gain, args.inflection))
        } else {
            None
        };
        Self { gamma, sigmoid }
    }
}
fn sigmoid(x: f32, gain: f32, mid: f32) -> f32 {
    1.0 / (1.0 + ((mid - x) * gain).exp())
}
fn scaled_sigmoid(x: f32, gain: f32, mid: f32) -> f32 {
    let min = sigmoid(0.0, gain, mid);
    let max = sigmoid(1.0, gain, mid);
    let s = sigmoid(x, gain, mid);
    (s - min) / (max - min)
}
fn sigmoid_lut(params: SigmoidParams) -> [u8; 256] {
    let mut lut = [0u8; 256];
    let gain = params.gain;
    let mid = params.mid;
    lut.iter_mut()
        .enumerate()
        .for_each(|(i, v)| *v = (255f32 * scaled_sigmoid(i as f32 / 255f32, gain, mid)) as u8);
    lut
}
fn gamma_lut(gamma: f32) -> [u8; 256] {
    let mut lut = [0u8; 256];
    lut.iter_mut()
        .enumerate()
        .for_each(|(i, v)| *v = (255f32 * (i as f32 / 255f32).powf(1f32 / gamma)) as u8);
    lut
}
fn gamma_with_sig_lut(params: CorrectionParams) -> [u8; 256] {
    let mut lut = [0u8; 256];
    let gamma = params.gamma.unwrap();
    let (gain, mid) = {
        let sigmoid_params = params.sigmoid.unwrap();
        (sigmoid_params.gain, sigmoid_params.mid)
    };
    lut.iter_mut().enumerate().for_each(|(i, v)| {
        *v = (255f32 * scaled_sigmoid((i as f32 / 255f32).powf(1f32 / gamma), gain, mid)) as u8
    });
    lut
}
pub fn correction(image: DynamicImage, params: CorrectionParams) -> DynamicImage {
    // imageはrgb8
    let condition = ((params.sigmoid.is_some() as u8) << 1) | (params.gamma.is_some() as u8);
    match condition {
        // 補正無し
        0 => image,
        // 補正あり
        1..=3 => {
            let lut = match condition {
                1 => gamma_lut(params.gamma.unwrap()),
                2 => sigmoid_lut(params.sigmoid.unwrap()),
                3 => gamma_with_sig_lut(params),
                _ => {
                    unreachable!("This Code is unreachable.")
                }
            };
            DynamicImage::from(
                image::RgbImage::from_raw(
                    image.width(),
                    image.height(),
                    image
                        .into_bytes()
                        .into_iter()
                        .map(|v| lut[v as usize])
                        .collect::<Vec<u8>>(),
                )
                .unwrap(),
            )
        }

        _ => {
            unreachable!("This Code is unreachable.")
        }
    }
}
