mod audio;
mod common;
mod picture;

use self::audio::{TargetAudio};
use self::common::OutputMode;
use self::picture::{TargetPicture};

fn main() {
    println!("Beginn");
    
    let target_file_path: String = String::from("/home/voldracarno/Bilder/test.png");
    let target_file_path2: String = String::from("/home/voldracarno/Bilder/test.jpg");
    let target_file_path3: String = String::from("/home/voldracarno/Bilder/test2.jpg");
    let target_file_path4: String = String::from("/home/voldracarno/Bilder/test3.jpg");
    
    let mut target_picture: TargetPicture = TargetPicture::new(&target_file_path);
    
    let img_hash: String = target_picture.calc_ahash(64, OutputMode::Bin);
    
    target_picture.set_filepath(&target_file_path2);
    let img_hash2: String = target_picture.calc_ahash(64, OutputMode::Bin);
    
    target_picture.set_filepath(&target_file_path3);
    let img_hash3: String = target_picture.calc_ahash(64, OutputMode::Bin);
    
    target_picture.set_filepath(&target_file_path4);
    let img_hash4: String = target_picture.calc_ahash(64, OutputMode::Bin);
    
    println!("Result: {}", img_hash);
    println!("Result: {}", img_hash2);
    println!("Result: {}", img_hash3);
    println!("Result: {}", img_hash4);
    println!("Bitcount: {}", img_hash.len());
    
    
    let audio_test_file: String = String::from("/home/voldracarno/Musik/test5-1.wav");
    let audio_test_file2: String = String::from("/home/voldracarno/Musik/test5-3.wav");
    let audio_test_file3: String = String::from("/home/voldracarno/Musik/test6-1.wav");
    
    let mut target_audio: TargetAudio = TargetAudio::new(&audio_test_file);
    let mut target_audio2: TargetAudio = TargetAudio::new(&audio_test_file2);
    let mut target_audio3: TargetAudio = TargetAudio::new(&audio_test_file3);
    
    target_audio.calc_gen_ahash(256, OutputMode::Bin);
    println!("----------------");
    target_audio2.calc_gen_ahash(256, OutputMode::Bin);
    println!("----------------");
    target_audio3.calc_gen_ahash(256, OutputMode::Bin);
}
