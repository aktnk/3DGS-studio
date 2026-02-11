use anyhow::{Context, Result};
use image::{imageops::FilterType, DynamicImage};
use ndarray::{Array, Array4};
use ort::{
    inputs,
    session::{builder::GraphOptimizationLevel, Session},
    value::{DynValue, Tensor},
};
use std::path::Path;

/// エンコーダーが出力する特徴量データを保持する構造体
pub struct EncodedImage {
    pub image_embed: DynValue,
    pub high_res_feats: Vec<DynValue>,
    pub original_size: (u32, u32),
}

/// SAM2 モデルを管理するエンジンの構造体
pub struct Sam2Engine {
    pub encoder: Session,
    pub decoder: Session,
}

impl Sam2Engine {
    pub fn new<P: AsRef<Path>>(model_dir: P) -> Result<Self> {
        let model_dir = model_dir.as_ref();
        let encoder_path = model_dir.join("sam2_hiera_tiny_encoder.onnx");
        let decoder_path = model_dir.join("sam2_hiera_tiny_decoder.onnx");

        let encoder = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(&encoder_path)
            .context("Failed to load encoder")?;

        let decoder = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(&decoder_path)
            .context("Failed to load decoder")?;

        Ok(Self { encoder, decoder })
    }

    pub fn preprocess(&self, img: &DynamicImage) -> Result<Array4<f32>> {
        let resized = img.resize_exact(1024, 1024, FilterType::Triangle);
        let rgb_img = resized.to_rgb8();

        let mean = [123.675f32, 116.28f32, 103.53f32];
        let std = [58.395f32, 57.12f32, 57.375f32];

        let mut tensor = Array4::zeros((1, 3, 1024, 1024));
        for y in 0..1024 {
            for x in 0..1024 {
                let pixel = rgb_img.get_pixel(x, y);
                for c in 0..3 {
                    let val = (pixel[c] as f32 - mean[c]) / std[c];
                    tensor[[0, c, y as usize, x as usize]] = val;
                }
            }
        }
        Ok(tensor)
    }

    pub fn encode_image(&mut self, tensor: Array4<f32>, original_size: (u32, u32)) -> Result<EncodedImage> {
        let shape = vec![1, 3, 1024, 1024];
        let data = tensor.into_raw_vec();
        let input_tensor = Tensor::from_array((shape, data))?;

        let mut outputs = self.encoder.run(inputs![
            "image" => input_tensor
        ])?;

        let image_embed = outputs
            .remove("image_embed")
            .context("Failed to get image_embed")?;

        let feat0 = outputs
            .remove("high_res_feats_0")
            .context("Failed to get high_res_feats_0")?;

        let feat1 = outputs
            .remove("high_res_feats_1")
            .context("Failed to get high_res_feats_1")?;

        Ok(EncodedImage {
            image_embed,
            high_res_feats: vec![feat0, feat1],
            original_size,
        })
    }

    pub fn predict(
        &mut self,
        encoded: &EncodedImage,
        points: &[(f32, f32)],
        labels: &[f32],
    ) -> Result<Vec<Array4<f32>>> {
        
        let num_points = points.len();

        let mut flat_coords = Vec::with_capacity(num_points * 2);
        for (x, y) in points {
            flat_coords.push(*x);
            flat_coords.push(*y);
        }
        let point_coords = Tensor::from_array(([1, num_points, 2], flat_coords))?;
        let point_labels = Tensor::from_array(([1, num_points], labels.to_vec()))?;

        let mask_input_data = vec![0.0f32; 1 * 1 * 256 * 256];
        let mask_input = Tensor::from_array(([1, 1, 256, 256], mask_input_data))?;
        let has_mask_input = Tensor::from_array(([1], vec![0.0f32]))?;

        // 削除: orig_im_size の作成と入力を削除しました

        let outputs = self.decoder.run(inputs![
            "image_embed" => encoded.image_embed.view(),
            "high_res_feats_0" => encoded.high_res_feats[0].view(),
            "high_res_feats_1" => encoded.high_res_feats[1].view(),
            "point_coords" => point_coords,
            "point_labels" => point_labels,
            "mask_input" => mask_input,
            "has_mask_input" => has_mask_input
            // 削除: "orig_im_size" => orig_im_size
        ])?;

        let masks_dyn = outputs.get("masks").context("Failed to get masks")?;
        
        let (shape, data) = masks_dyn.try_extract_tensor::<f32>()?;
        
        let shape_dim = (
            shape[0] as usize,
            shape[1] as usize,
            shape[2] as usize,
            shape[3] as usize
        );
        
        let masks_array = Array::from_shape_vec(shape_dim, data.to_vec())
            .context("Failed to convert tensor to ndarray")?;
        
        Ok(vec![masks_array])
    }
}

// ==================================================================================
// テストコード
// ==================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use image::{DynamicImage, RgbImage, Rgb};

    fn get_model_path() -> PathBuf {
        PathBuf::from(r"C:\Users\ak2tn\projects\3dgs-studio\assets\models\sam2")
    }

    #[test]
    fn test_pipeline_encode_and_predict() {
        let model_dir = get_model_path();
        if !model_dir.exists() {
            println!("SKIPPING TEST: Model directory not found at {:?}", model_dir);
            return;
        }

        let mut engine = Sam2Engine::new(&model_dir)
            .expect("Failed to load engine");

        let width = 100;
        let height = 100;
        let mut img_buffer = RgbImage::new(width, height);
        for pixel in img_buffer.pixels_mut() {
            *pixel = Rgb([255, 0, 0]);
        }
        let original_img = DynamicImage::ImageRgb8(img_buffer);

        println!("Running preprocess...");
        let input_tensor = engine.preprocess(&original_img)
            .expect("Preprocess failed");

        println!("Running encoder...");
        let encoded_result = engine.encode_image(input_tensor, (height, width))
            .expect("Encode failed");
        
        println!("Encoder output received. Embed shape: {:?}", encoded_result.image_embed.shape());

        println!("Running decoder (predict)...");
        let points = vec![(50.0, 50.0)]; 
        let labels = vec![1.0]; 

        let masks = engine.predict(&encoded_result, &points, &labels)
            .expect("Prediction failed");

        let shape = masks[0].shape();
        println!("Generated masks shape: {:?}", shape);

        assert!(shape.len() == 4, "Mask should be 4D tensor");
        assert!(shape[1] >= 1, "Should have at least 1 mask");

        println!("SUCCESS: Full pipeline executed successfully!");
    }
}