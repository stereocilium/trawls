use std::{thread, time};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

mod image;
pub mod status;

#[derive(Copy, Clone)]
pub enum CometMode {
    Falling,
    Raising,
    Normal,
}

pub fn run_merge(lightframe_files: Vec<PathBuf>, mode: CometMode, state: status::Status) {
    let num_threads = num_cpus::get();
    println!("System has {} cores and {} threads. Using {} worker threads.", num_cpus::get_physical(), num_threads, num_threads);


    let result = Arc::new(Mutex::new(vec![]));
    let mut thread_handles = vec![];

    for _ in 0..num_threads {
        let q = Arc::clone(&result);
        let s = state.clone();
        thread_handles.push(thread::spawn(move || {
            queue_worker(q, s);
        }));
    }

    lightframe_files.par_iter()
        .zip(0..lightframe_files.len())
        .for_each(|(e, i)| process_image(e, Arc::clone(&result), i, lightframe_files.len(), mode, state.clone()));

    for t in thread_handles {
        t.join().unwrap_or(());
    }

    let mut data = result.lock().unwrap();
    let _raw_image = data.pop().unwrap();
    println!("Processing done");
}


fn queue_worker(queue: Arc<Mutex<Vec<image::Image>>>, state: status::Status) {
    loop {
        let mut q = queue.lock().unwrap();
        if q.len() <= 1 {
            if state.loading_done() { return; } else {
                // Queue is empty but work is not done yet => Wait.
                drop(q);
                thread::sleep(time::Duration::from_millis(20));
                continue;
            }
        }
        state.start_merging();

        let v1 = q.pop().unwrap();
        let v2 = q.pop().unwrap();
        drop(q);

        let res = v1.merge(v2);
        queue.lock().unwrap().push(res);
        state.finish_merging();
    }
}


fn process_image(entry: &PathBuf, queue: Arc<Mutex<Vec<image::Image>>>, index: usize, num_images: usize, mode: CometMode, state: status::Status) {
    state.start_loading();

    let intensity = match mode {
        CometMode::Falling => 1.0 - index as f32 / num_images as f32,
        CometMode::Raising => index as f32 / num_images as f32,
        CometMode::Normal => 1.0,
    };

    let img = image::Image::load_from_raw(entry.as_path(), intensity).unwrap();
    queue.lock().unwrap().push(img);

    state.finish_loading();
}