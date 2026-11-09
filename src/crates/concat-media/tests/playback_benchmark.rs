// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Benchmark for real 120 FPS video playback on a 60 FPS timeline with NVDEC and prefetch.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use concat_core::time::FrameRate;
use concat_media::prefetch::{Cursor, Direction, Moment, Prefetcher};
use concat_media::{
    FrameRequest, ReaderPool, convert_count, download_count, reset_perf_counters,
    set_hardware_decode, set_playback_active,
};

#[test]
#[ignore = "requires real local test video /home/paolo/Scrivania/Spettacolo Hellaverse Lucca C&G 2026.MOV"]
fn benchmark_real_playback_120fps_to_60fps() {
    let video_path = Path::new("/home/paolo/Scrivania/Spettacolo Hellaverse Lucca C&G 2026.MOV");
    if !video_path.exists() {
        eprintln!("Test video not found: {}", video_path.display());
        return;
    }

    // Enable hardware acceleration (NVDEC) and playback active mode
    set_hardware_decode(true);
    set_playback_active(true);
    reset_perf_counters();

    // 1 GB cache budget, 4 readers
    let pool = Arc::new(ReaderPool::new(1024 * 1024 * 1024, 4));
    let prefetcher = Prefetcher::new(Arc::clone(&pool), 2);

    // Timeline at 59.940 fps: 60000 / 1001
    let timeline_fps = FrameRate::NTSC_60;
    let frame_seconds = timeline_fps.frame_duration().as_f64();
    let total_frames = 180; // 3 seconds of playback at 59.94 fps

    println!("\n========================================================");
    println!("STARTING PLAYBACK BENCHMARK (180 frames @ 59.94 fps)");
    println!("Video: {}", video_path.display());
    println!("Preview size: 540x960 (1/4 scale)");
    println!("========================================================");

    let mut waits = Vec::with_capacity(total_frames);
    let mut dropped_frames = 0;
    let mut worst_wait = Duration::ZERO;

    // Primping: In the real app, frame 0 is already displayed on the monitor before Play is pressed.
    let req0 = FrameRequest::new(video_path, timeline_fps.time_of_frame(0), 540, 960);
    let _ = pool.frame(&req0);

    // Initial prime: submit the first 16 moments to prefetcher
    let initial_ahead: Vec<Moment> = (1..=16)
        .map(|ahead| {
            let time_rat = timeline_fps.time_of_frame(ahead);
            let t = time_rat.as_f64();
            Moment {
                time: t,
                frames: vec![FrameRequest::new(video_path, time_rat, 540, 960)],
            }
        })
        .collect();
    prefetcher.advance(
        Cursor {
            time: 0.0,
            direction: Direction::Forward,
            rate: 1.0,
        },
        frame_seconds,
        initial_ahead,
    );

    // Give prefetcher workers time to decode the initial ahead buffer before playback starts
    std::thread::sleep(Duration::from_millis(300));

    // Reset counters before measuring the playback loop
    reset_perf_counters();

    let benchmark_start = Instant::now();

    for i in 0..total_frames {
        let time_rat = timeline_fps.time_of_frame(i as i64);
        let t = time_rat.as_f64();

        // Queue prefetch ahead (up to 16 frames ahead)
        let ahead_moments: Vec<Moment> = (1..=16)
            .map(|ahead| {
                let ahead_rat = timeline_fps.time_of_frame((i + ahead) as i64);
                let ahead_t = ahead_rat.as_f64();
                Moment {
                    time: ahead_t,
                    frames: vec![FrameRequest::new(video_path, ahead_rat, 540, 960)],
                }
            })
            .collect();
        prefetcher.advance(
            Cursor {
                time: t,
                direction: Direction::Forward,
                rate: 1.0,
            },
            frame_seconds,
            ahead_moments,
        );

        // Measure time taken to pull the frame
        let t_frame_start = Instant::now();
        let req = FrameRequest::new(video_path, time_rat, 540, 960);
        let frame_res = pool.frame(&req);
        let wait = t_frame_start.elapsed();

        waits.push(wait);
        if wait > worst_wait {
            worst_wait = wait;
        }

        match frame_res {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Frame {} at {:.3}s dropped: {:?}", i, t, e);
                dropped_frames += 1;
            }
        }

        // Pacing: simulate audio clock / Slint 59.94 fps cadence
        let target_frame_end = benchmark_start + Duration::from_secs_f64((i + 1) as f64 * frame_seconds);
        let now = Instant::now();
        if now < target_frame_end {
            std::thread::sleep(target_frame_end - now);
        }
    }

    set_playback_active(false);
    prefetcher.drain();

    let downloads = download_count();
    let converts = convert_count();
    let total_wait: Duration = waits.iter().sum();
    let avg_wait = total_wait / total_frames as u32;

    println!("\n========================================================");
    println!("BENCHMARK RESULTS (180 frames @ 59.94 fps):");
    println!("  Total frames requested: {}", total_frames);
    println!("  Dropped/skipped frames: {}", dropped_frames);
    println!("  Average wait per frame: {:.3} ms", avg_wait.as_secs_f64() * 1000.0);
    println!("  Worst wait:             {:.3} ms", worst_wait.as_secs_f64() * 1000.0);
    println!("  hardware::download:     {}", downloads);
    println!("  convert (swscale):      {}", converts);
    println!("========================================================\n");

    assert_eq!(dropped_frames, 0, "Expected 0 dropped/skipped frames!");
    assert!(
        avg_wait.as_millis() < 16,
        "Average wait ({:?}) should be well below 16.6 ms budget",
        avg_wait
    );
    assert!(
        downloads <= 200,
        "Expected ~180 downloads, got {}",
        downloads
    );
    assert!(
        converts <= 200,
        "Expected ~180 converts, got {}",
        converts
    );
}
