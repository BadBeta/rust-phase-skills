// Image Processing with Rust WASM
// Demonstrates performance-critical pixel manipulation

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

// ========================================
// 1. Basic Image Buffer
// ========================================

#[wasm_bindgen]
pub struct ImageBuffer {
    width: u32,
    height: u32,
    data: Vec<u8>,  // RGBA format
}

#[wasm_bindgen]
impl ImageBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            data: vec![0u8; size],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get pointer for zero-copy access from JS
    pub fn data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    /// Load image data from JS ImageData
    pub fn load_from_data(&mut self, data: Clamped<Vec<u8>>) {
        self.data = data.0;
    }

    /// Get data for JS ImageData
    pub fn get_data(&self) -> Clamped<Vec<u8>> {
        Clamped(self.data.clone())
    }
}

// ========================================
// 2. Basic Filters
// ========================================

#[wasm_bindgen]
impl ImageBuffer {
    /// Invert colors
    pub fn invert(&mut self) {
        for i in (0..self.data.len()).step_by(4) {
            self.data[i] = 255 - self.data[i];         // R
            self.data[i + 1] = 255 - self.data[i + 1]; // G
            self.data[i + 2] = 255 - self.data[i + 2]; // B
            // Alpha unchanged
        }
    }

    /// Grayscale conversion
    pub fn grayscale(&mut self) {
        for i in (0..self.data.len()).step_by(4) {
            let r = self.data[i] as f32;
            let g = self.data[i + 1] as f32;
            let b = self.data[i + 2] as f32;

            // Luminosity method
            let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;

            self.data[i] = gray;
            self.data[i + 1] = gray;
            self.data[i + 2] = gray;
        }
    }

    /// Adjust brightness
    pub fn brightness(&mut self, amount: i32) {
        for i in (0..self.data.len()).step_by(4) {
            self.data[i] = clamp_u8(self.data[i] as i32 + amount);
            self.data[i + 1] = clamp_u8(self.data[i + 1] as i32 + amount);
            self.data[i + 2] = clamp_u8(self.data[i + 2] as i32 + amount);
        }
    }

    /// Adjust contrast
    pub fn contrast(&mut self, factor: f32) {
        let factor = factor.max(0.0);
        for i in (0..self.data.len()).step_by(4) {
            self.data[i] = clamp_u8(((self.data[i] as f32 - 128.0) * factor + 128.0) as i32);
            self.data[i + 1] = clamp_u8(((self.data[i + 1] as f32 - 128.0) * factor + 128.0) as i32);
            self.data[i + 2] = clamp_u8(((self.data[i + 2] as f32 - 128.0) * factor + 128.0) as i32);
        }
    }

    /// Sepia tone
    pub fn sepia(&mut self) {
        for i in (0..self.data.len()).step_by(4) {
            let r = self.data[i] as f32;
            let g = self.data[i + 1] as f32;
            let b = self.data[i + 2] as f32;

            self.data[i] = clamp_u8((0.393 * r + 0.769 * g + 0.189 * b) as i32);
            self.data[i + 1] = clamp_u8((0.349 * r + 0.686 * g + 0.168 * b) as i32);
            self.data[i + 2] = clamp_u8((0.272 * r + 0.534 * g + 0.131 * b) as i32);
        }
    }
}

#[inline]
fn clamp_u8(value: i32) -> u8 {
    value.max(0).min(255) as u8
}

// ========================================
// 3. Convolution Filters (Blur, Sharpen)
// ========================================

#[wasm_bindgen]
impl ImageBuffer {
    /// Box blur
    pub fn blur(&mut self, radius: u32) {
        if radius == 0 {
            return;
        }

        let kernel_size = (radius * 2 + 1) as usize;
        let kernel_area = (kernel_size * kernel_size) as f32;

        let src = self.data.clone();
        let width = self.width as usize;
        let height = self.height as usize;
        let radius = radius as i32;

        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0.0f32;
                let mut g_sum = 0.0f32;
                let mut b_sum = 0.0f32;

                for ky in -radius..=radius {
                    for kx in -radius..=radius {
                        let px = (x as i32 + kx).max(0).min(width as i32 - 1) as usize;
                        let py = (y as i32 + ky).max(0).min(height as i32 - 1) as usize;
                        let idx = (py * width + px) * 4;

                        r_sum += src[idx] as f32;
                        g_sum += src[idx + 1] as f32;
                        b_sum += src[idx + 2] as f32;
                    }
                }

                let idx = (y * width + x) * 4;
                self.data[idx] = (r_sum / kernel_area) as u8;
                self.data[idx + 1] = (g_sum / kernel_area) as u8;
                self.data[idx + 2] = (b_sum / kernel_area) as u8;
            }
        }
    }

    /// Sharpen filter
    pub fn sharpen(&mut self) {
        let kernel: [[f32; 3]; 3] = [
            [0.0, -1.0, 0.0],
            [-1.0, 5.0, -1.0],
            [0.0, -1.0, 0.0],
        ];

        self.apply_kernel(&kernel);
    }

    /// Edge detection
    pub fn edge_detect(&mut self) {
        let kernel: [[f32; 3]; 3] = [
            [-1.0, -1.0, -1.0],
            [-1.0, 8.0, -1.0],
            [-1.0, -1.0, -1.0],
        ];

        self.apply_kernel(&kernel);
    }

    fn apply_kernel(&mut self, kernel: &[[f32; 3]; 3]) {
        let src = self.data.clone();
        let width = self.width as usize;
        let height = self.height as usize;

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let mut r = 0.0f32;
                let mut g = 0.0f32;
                let mut b = 0.0f32;

                for ky in 0..3 {
                    for kx in 0..3 {
                        let px = x + kx - 1;
                        let py = y + ky - 1;
                        let idx = (py * width + px) * 4;
                        let k = kernel[ky][kx];

                        r += src[idx] as f32 * k;
                        g += src[idx + 1] as f32 * k;
                        b += src[idx + 2] as f32 * k;
                    }
                }

                let idx = (y * width + x) * 4;
                self.data[idx] = clamp_u8(r as i32);
                self.data[idx + 1] = clamp_u8(g as i32);
                self.data[idx + 2] = clamp_u8(b as i32);
            }
        }
    }
}

// ========================================
// 4. SIMD-Optimized Processing
// ========================================

#[cfg(target_feature = "simd128")]
mod simd {
    use super::*;
    use std::arch::wasm32::*;

    pub fn invert_simd(data: &mut [u8]) {
        let all_255 = u8x16_splat(255);

        for chunk in data.chunks_exact_mut(16) {
            let v = v128_load(chunk.as_ptr() as *const v128);
            let inverted = u8x16_sub(all_255, v);
            v128_store(chunk.as_mut_ptr() as *mut v128, inverted);
        }
    }
}

// ========================================
// 5. Zero-Copy Access from JS
// ========================================

#[wasm_bindgen]
pub fn get_wasm_memory() -> JsValue {
    wasm_bindgen::memory()
}

// ========================================
// Usage from JavaScript:
// ========================================
//
// import init, { ImageBuffer, get_wasm_memory } from './pkg/app.js';
//
// await init();
//
// // Get image from canvas
// const ctx = canvas.getContext('2d');
// const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
//
// // Create WASM buffer
// const buffer = new ImageBuffer(canvas.width, canvas.height);
//
// // Zero-copy access to WASM memory
// const memory = get_wasm_memory();
// const wasmData = new Uint8ClampedArray(
//     memory.buffer,
//     buffer.data_ptr(),
//     buffer.data_len()
// );
//
// // Copy image data to WASM
// wasmData.set(imageData.data);
//
// // Apply filter in WASM
// buffer.grayscale();
//
// // Copy back to canvas
// imageData.data.set(wasmData);
// ctx.putImageData(imageData, 0, 0);
//
// // Clean up
// buffer.free();
