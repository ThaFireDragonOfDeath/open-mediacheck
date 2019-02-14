extern crate image;

//mod common;

use common::OutputMode;
use self::image::{Pixel,imageops,FilterType};

pub struct TargetPicture {
    pic_file_path: String,
}

impl TargetPicture {
    pub fn new(new_pic_file_path: &String) -> TargetPicture {
        TargetPicture {
            pic_file_path: new_pic_file_path.to_string(),
        }
    }
    
    pub fn calc_ahash(&self, hash_bitsize: u16, output_mode: OutputMode) -> String {
        let color_groups: u8 = 16;
        let bits_per_field: u8 = 4;
        
        let fields_count = (hash_bitsize / bits_per_field as u16) as f32;
        let horpixfields = fields_count.sqrt() as u32;
        let vertpixfields = horpixfields;

        let img = image::open(&self.pic_file_path).unwrap();
        let resized_img = imageops::resize(&img, horpixfields, vertpixfields, FilterType::Nearest);
        
        let mut final_hash: String = String::from("");
        
        for current_pixel in resized_img.pixels() {
            let current_pix_cat: u8 = TargetPicture::calc_cat(&current_pixel.to_rgb(), color_groups);
            let current_pix_cat_bin: String = format!("{:04b}", current_pix_cat);
            final_hash.push_str(&current_pix_cat_bin);
        }
        
        final_hash
    }
    
    fn calc_cat(&pix: &image::Rgb<u8>, color_groups: u8) -> u8 {
        let pixel_rgb = pix.to_rgb();
        let avg_pixel_rgb: u16 = (pixel_rgb[0] as u16 + pixel_rgb[1] as u16 + pixel_rgb[2] as u16) as u16 / 3 as u16;
        let color_groups_base: u8 = (256 as u16 / color_groups as u16) as u8;
        let current_color_group: f32 = avg_pixel_rgb as f32 / color_groups_base as f32;
        let final_color_group: u8;
        
        if current_color_group < 0.0 {
            final_color_group = 0;
        }
        else {
            final_color_group = current_color_group.trunc() as u8;
        }
        
        final_color_group
    }
    
    pub fn set_filepath(&mut self, new_pic_file_path: &String) {
        self.pic_file_path = new_pic_file_path.to_string();
    }
}
