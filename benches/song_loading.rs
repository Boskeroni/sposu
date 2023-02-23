use std::{fs::{self, File}, io::BufReader};

use criterion::{black_box, Criterion, criterion_main, criterion_group};
use libosu::prelude::Beatmap;

pub fn load_songs(song_path: &str) {
    let song_folder = fs::read_dir(song_path).unwrap();
    let mut used_songs = Vec::new();

    // for every folder in the directory
    for folder in song_folder {
        let folder = folder.unwrap();
        if !folder.metadata().unwrap().is_dir() {
            continue;
        }
        let valid_folder = fs::read_dir(folder.path().as_path()).unwrap();
        let mp3_folder = fs::read_dir(folder.path().as_path()).unwrap();

        let mut mp3_ratio = 0;
        for file in mp3_folder {
            if let Some(e) = file.unwrap().path().extension() {
                if e == "mp3" {
                    mp3_ratio += 1;
                }
            }
        }
        
        for file in valid_folder {
            let file = file.unwrap();
            if let Some(e) = file.path().extension() {
                if e.to_string_lossy() != "osu" {
                    continue;
                }
            } else {
                continue;
            };

            let osu_file_reader = BufReader::new(File::open(&file.path()).unwrap());    
            if let Ok(i) = Beatmap::parse(osu_file_reader) {
                let name_artist = format!("{}-{}", i.title, i.artist);
                if used_songs.iter().any(|e| e == &name_artist) {
                    continue;
                }
                used_songs.push(name_artist);
                let _path = folder.path().to_string_lossy().replace(&song_path, "");
                mp3_ratio -= 1;
                if mp3_ratio == 0 {
                    break;
                }
            }
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("osu_song_loader");
    group.sample_size(10);
    group.bench_function("song_loading", |b| b.iter(|| load_songs(black_box("D:/Apps/osu!/Songs/"))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);