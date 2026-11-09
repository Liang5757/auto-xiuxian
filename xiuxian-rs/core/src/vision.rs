use image::GenericImageView;
use image::{GrayImage, RgbImage, imageops};
use std::collections::HashMap;
pub struct Template {
    w: usize,
    h: usize,
    centered: Vec<f32>,
    norm: f32,
}
impl Template {
    pub fn new(im: GrayImage) -> Self {
        let (w, h) = im.dimensions();
        let raw = im.as_raw();
        let mean = raw.iter().map(|&v| v as f32).sum::<f32>() / raw.len() as f32;
        let centered: Vec<_> = raw.iter().map(|&v| v as f32 - mean).collect();
        let norm = centered.iter().map(|v| v * v).sum::<f32>().sqrt();
        Self {
            w: w as _,
            h: h as _,
            centered,
            norm,
        }
    }
    pub fn matches(&self, im: &GrayImage, x: i32, y: i32, r: i32) -> bool {
        let stride = im.width() as usize;
        let pixels = im.as_raw();
        for yy in (y - r).max(0)..=(y + r).min(im.height() as i32 - self.h as i32) {
            for xx in (x - r).max(0)..=(x + r).min(im.width() as i32 - self.w as i32) {
                let (mut dot, mut sum, mut sq) = (0f64, 0u64, 0u64);
                for row in 0..self.h {
                    let start = (yy as usize + row) * stride + xx as usize;
                    for (&v, &t) in pixels[start..start + self.w]
                        .iter()
                        .zip(&self.centered[row * self.w..(row + 1) * self.w])
                    {
                        sum += v as u64;
                        sq += v as u64 * v as u64;
                        dot += v as f64 * t as f64
                    }
                }
                let variance = sq as f64 - (sum as f64).powi(2) / (self.w * self.h) as f64;
                if variance > 0.0 && dot > 0.78 * self.norm as f64 * variance.sqrt() {
                    return true;
                }
            }
        }
        false
    }
}
pub struct Frame {
    rgb: RgbImage,
    pub gray: GrayImage,
}
impl Frame {
    pub fn new(rgb: RgbImage) -> Self {
        let gray = imageops::grayscale(&rgb);
        Self { rgb, gray }
    }
    pub fn crop(&self, x: u32, y: u32, w: u32, h: u32) -> Template {
        Template::new(imageops::crop_imm(&self.gray, x, y, w, h).to_image())
    }
    pub fn slots(&self) -> Vec<(usize, Template)> {
        let mut slots = vec![];
        for i in 0..10 {
            let y = 201 + i * 63;
            let area = imageops::crop_imm(&self.rgb, 1286, y, 44, 45).to_image();
            if area.as_raw().iter().map(|&v| v as u32).sum::<u32>() < 223 * (44 * 45 * 3) {
                slots.push((i as usize, self.crop(1286, y, 44, 45)))
            }
        }
        slots
    }
}
pub struct Refs(HashMap<&'static str, Template>);
impl Refs {
    pub fn load() -> Self {
        let mut map = HashMap::new();
        macro_rules! add {
            ($($name:literal),*) => {
                $(
                    let bytes = include_bytes!(concat!("refs/", $name, ".png"));
                    let gray = image::load_from_memory(bytes).unwrap().to_luma8();
                    map.insert($name, Template::new(gray));
                )*
            };
        }
        add!(
            "choose",
            "collect",
            "current",
            "empty",
            "expedition",
            "notice",
            "origin",
            "petpanel",
            "safe",
            "travel",
            "world",
            "ripe_notice",
            "garden",
            "cave",
            "plot_empty",
            "quick",
            "growing"
        );
        Self(map)
    }
    pub fn has(&self, f: &Frame, key: &str, x: i32, y: i32) -> bool {
        self.0[key].matches(&f.gray, x, y, 6)
    }
    pub fn returned(&self, f: &Frame) -> bool {
        for offset in (0..150).step_by(30) {
            let count = imageops::crop_imm(&f.rgb, 1045, 209 + offset, 212, 30)
                .pixels()
                .filter(|(_, _, p)| {
                    p[2] as i32 > p[0] as i32 + 35 && p[2] as i32 > p[1] as i32 + 25
                })
                .take(30)
                .count();
            if count == 30
                && (1080..1142)
                    .step_by(10)
                    .any(|x| self.has(f, "notice", x, 215 + offset as i32))
            {
                return true;
            }
        }
        false
    }
}
#[cfg(test)]
pub fn from_file(path: &str) -> Result<Frame, image::ImageError> {
    let im: image::DynamicImage = image::open(path)?;
    Ok(Frame::new(
        im.resize_exact(1612, 894, imageops::FilterType::Triangle)
            .to_rgb8(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blank_frame_has_no_return_notice() {
        assert!(!Refs::load().returned(&Frame::new(RgbImage::new(1612, 894))));
    }
    #[test]
    #[ignore = "requires a captured battle screen with returned teams"]
    fn benchmark_notice() {
        let f = from_file(&std::env::var("XIUXIAN_SAMPLE").unwrap()).unwrap();
        let refs = Refs::load();
        let start = std::time::Instant::now();
        for _ in 0..50 {
            assert!(refs.returned(std::hint::black_box(&f)));
        }
        println!(
            "notice benchmark: {:.3} ms/frame",
            start.elapsed().as_secs_f64() * 20.
        );
    }
}
