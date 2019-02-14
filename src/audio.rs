extern crate hound;
extern crate rustfft;

use common::OutputMode;
use self::hound::{WavReader,WavSamples,Sample};
use std::sync::Arc;
use self::rustfft::FFTplanner;
use self::rustfft::num_complex::Complex;
use self::rustfft::num_traits::Zero;

use std::fs;
use std::io;

pub struct TargetAudio {
    audio_file_path: String,
}

impl TargetAudio {
    pub fn new(new_audio_file_path: &String) -> TargetAudio {
        TargetAudio {
            audio_file_path: new_audio_file_path.to_string(),
        }
    }

    pub fn calc_gen_ahash(&self, hash_bitsize: u16, output_mode: OutputMode) {
        // Define constants
        let hash_area_offset_in_seconds: f32 = 4.0; // Crop x Seconds from left and write befor hashing (remove silince on beginning and end)
        let hash_area_size_in_seconds: f32 = 0.5; // Calc frequency spectrum from the area of x seconds (lower is faster (but with fewer precision))
        let hash_area_distance_in_seconds: f32 = 4.0; // Distance in seconds between the audio areas
        let audio_groups: u8 = 9; // Audio groups (have to be: (4 or 8 or 16 or 32) + 1)
        let audio_group_bitcount: u8 = audio_groups - 1; // One audio category is encoded x bits
        let frequency_sidecrop_left: u32 = 150; // Beginn sectrum analysis at x Hz
        let frequency_sidecrop_right: u32 = 8000; // End spectrum analysis at x Hz
        
        // Open WAV Audio and read informations
        // WAV Audio hast to be encoded as pcm_f32le
        let mut reader = hound::WavReader::open(&self.audio_file_path).unwrap();
        let samples_count: u32 = reader.duration();
        let samples_per_second: u32 = reader.spec().sample_rate;
        let audio_duration_in_seconds: u32 = samples_count / samples_per_second;
        
        let duration_time_in_samples: u32 = (hash_area_size_in_seconds as f32 * samples_per_second as f32) as u32;
        
        let mut remaining_audio: f32 = audio_duration_in_seconds as f32 - hash_area_offset_in_seconds as f32;
        let mut audio_hashes: Vec<String> = Vec::new();
        
        // For every hash area
        while ((hash_area_offset_in_seconds as f32 + hash_area_size_in_seconds as f32 + hash_area_distance_in_seconds as f32) < (remaining_audio as f32)) {
            //Calc begin time
            let begin_time: u32 = ((audio_duration_in_seconds as f32 - remaining_audio as f32) * samples_per_second as f32) as u32;
            
            // Read target samples
            let mut current_complex_samples: Vec<Complex<f32>> = TargetAudio::get_samples(&mut reader, begin_time, duration_time_in_samples);
            let mut fft_samples: Vec<Complex<f32>> = vec![Zero::zero(); duration_time_in_samples as usize];
            
            // Do fft calculations (to get the frequency spectrum of the read audio data)
            let mut planner = FFTplanner::new(false);
            let fft = planner.plan_fft(duration_time_in_samples as usize);
            fft.process(&mut current_complex_samples, &mut fft_samples);
            
            // Subtract the read audio from the remaining audio for the next loop
            remaining_audio -= (hash_area_size_in_seconds as f32 + hash_area_distance_in_seconds as f32);
            
            // Normalize the fft data
            let mut fft_samples_abs = TargetAudio::normalize_fftsamples(&mut fft_samples, hash_bitsize, samples_per_second, frequency_sidecrop_left, frequency_sidecrop_right, audio_group_bitcount, hash_area_size_in_seconds);
            let mut current_hash: String = String::from("");
            
            // For every fft value in the vector
            for fft in &fft_samples_abs {
                // Debug
                //print!{"{} ", *fft}
                
                let current_audio_cat: u8 = TargetAudio::calc_cat(*fft, audio_groups, hash_bitsize); // Calc the audio category
                
                // Debug
                print!{"{} ", current_audio_cat}
                
                // Encode the category to binary
                let current_audio_cat_bin = TargetAudio::cat_to_bin(current_audio_cat, audio_group_bitcount);
                current_hash.push_str(&current_audio_cat_bin); // Append binary string to the result string of this area
                
            }
            
            // Debug
            println!("");
            println!("Hash: {}", current_hash);
        }
    }
    
    fn calc_cat(fft_val: u8, audio_groups: u8, hash_bitsize: u16) -> u8 {
        let audio_groups_base: u8 = (hash_bitsize as u16 / audio_groups as u16) as u8;
        let current_audio_group: f32 = fft_val as f32 / audio_groups_base as f32;
        let final_audio_group: u8;
        
        if current_audio_group < 0.0 {
            final_audio_group = 0;
        }
        else if current_audio_group > (audio_groups - 1) as f32 {
            final_audio_group = audio_groups - 1;
        }
        else {
            final_audio_group = current_audio_group.trunc() as u8;
        }
        
        final_audio_group
    }
    
    fn cat_to_bin(cat: u8, bitcount: u8) -> String {
        let mut bin_str: String = String::from("");
        
        for x in 0..bitcount - cat {
            bin_str.push('0');
        }
        
        for x in 0..cat {
            bin_str.push('1');
        }
        
        bin_str
    }
    
    fn normalize_fftsamples(fft_samples: &mut Vec<Complex<f32>>, hash_bitsize: u16, samples_per_second: u32, frequency_sidecrop_left: u32, frequency_sidecrop_right: u32, audio_group_bitcount: u8, hash_area_size_in_seconds: f32) -> Vec<u8> {
        let mut fft_samples_abs: Vec<f32> = Vec::new();
        
        // Read the half of the values and make them absolute
        let mut read_fft: u32 = 0;
        for fft in fft_samples {
            if (read_fft < samples_per_second / 2) {
                fft_samples_abs.push(fft.norm());
            }
            
            read_fft += 1;
        }
        
        // Crop frequencies (set by frequency_sidecrop_left/right)
        let mut fft_samples_abs_crop: Vec<f32> = Vec::new();
        read_fft = 0;
        
        for fft_abs in fft_samples_abs {
            if (read_fft >= (frequency_sidecrop_left as f32 * hash_area_size_in_seconds) as u32 && read_fft <= (frequency_sidecrop_right as f32 * hash_area_size_in_seconds) as u32) {
                fft_samples_abs_crop.push(fft_abs);
                
                // Debug
                //println!("Push: {}", fft_abs);
            }
            
            read_fft += 1;
        }
        
        // Downsampling the fft values
        let target_fft_count: u8 = (hash_bitsize as u32 / audio_group_bitcount as u32) as u8 + 1;
        let hash_width_factor: u32 = fft_samples_abs_crop.len() as u32 / target_fft_count as u32;
        let mut fft_sampled_width_avg: Vec<f32> = Vec::new();
        
        //println!("Width Factor: {}", hash_width_factor);
        //println!("Begin Factor: {}", hash_begin_factor);
        
        //println!("Current len: {}", target_fft_count);
        //println!("Current target_fft_count: {}", target_fft_count);
        //println!("Current hash_width_factor: {}", hash_width_factor);
        
        for x in 0..target_fft_count {
            //println!("Current aufter crop: {}", fft_abs);
            //println!("Current x: {}", x);
            
            let mut fft_current_sum: f64 = 0.0;
            fft_current_sum += fft_samples_abs_crop[(hash_width_factor * x as u32) as usize] as f64;
            fft_current_sum += fft_samples_abs_crop[(hash_width_factor * x as u32 + 1) as usize] as f64;
            fft_current_sum += fft_samples_abs_crop[(hash_width_factor * x as u32 + 2) as usize] as f64;
            fft_current_sum += fft_samples_abs_crop[(hash_width_factor * x as u32 + 3) as usize] as f64;
            
            
            let fft_current_w_avg: f32 = (fft_current_sum / 4.0 as f64) as f32;
            fft_sampled_width_avg.push(fft_current_w_avg);
            
            if (x == 0) {
                //println!("FFT RAW: {}", fft_current_w_avg);
            }
        }
        
        // Get maximum of all remaining fft values
        let mut fft_max: f32 = 0.0;
        
        for fft_abs in &fft_sampled_width_avg {
            //println!("Abs val: {}", fft_abs);
            
            if(*fft_abs > fft_max) {
                fft_max = *fft_abs;
            }
        }
        
        let mut fft_samples_norm: Vec<u8> = Vec::new();
        let mut fft_current_iter: u32 = 0;
        
        for fft_abs in &fft_sampled_width_avg {
            let fft_norm: u8 = (fft_abs / fft_max * 255.0) as u8; // Normalise aplitude (to detect the same audio at different volume)
            
            if(fft_current_iter != 0) {
                fft_samples_norm.push(fft_norm);
            }
            
            fft_current_iter += 1;
        }
        
        fft_samples_norm
    }

    fn get_samples(reader: &mut WavReader<io::BufReader<fs::File>>, begin_time: u32, duration_time_in_samples: u32) -> Vec<Complex<f32>> {
        let mut duration_samples = duration_time_in_samples;
        let mut wav_samples: Vec<Complex<f32>> = Vec::new();
        
        // Set read position to the begin time position
        reader.seek(begin_time).unwrap();
        
        // For every read sample
        for current_sample in reader.samples::<f32>() {
            // If the end of the target duration is reached
            if (duration_samples == 0) {
                break;
            }
            
            // Convert the read value in a complex number and append it to the resault array
            let current_complex_sample: Complex<f32> = Complex::new(current_sample.unwrap() as f32, 0.0);
            wav_samples.push(current_complex_sample);
            
            duration_samples -= 1;
        }
        
        wav_samples
    }
}
