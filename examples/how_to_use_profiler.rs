use std::{thread, fs};
use altius_benchtools::profiler;

fn expensive_calculation(seed: u128, repeat: u128) -> u128 {
    let modulus: u128 = 18446744073709551617;
    let mut base: u128 = 12142100291992418551 + seed;
    for _ in 0..repeat {
        base = (base * base) % modulus;
    }
    base
}


fn different_task_in_different_threads(output_path: &str) {
    // For different tasks in different threads, use `profiler::start`, `profiler::end` and
    // `profiler::note` to record the time spent on each task.
    // Note that you can't start another task with the same name and the same thread id before
    // the previous task is ended.
    profiler::clear();
    let thread_num = 3;
    let sub_task_num = 5;
    let mut handles = vec![];

    for thread_id in 0..thread_num {
        let handle = thread::spawn(move || {
            for task_id in 0..sub_task_num {
                let seed = thread_id * sub_task_num + task_id;
                let task_name = format!("task-{}-{}", thread_id, task_id);
                profiler::start(task_name.as_str());
                profiler::note_str(
                    task_name.as_str(), 
                    "extra-descreption", 
                    format!("You can add any extra description here for {}", task_name).as_str()
                );
                expensive_calculation(seed as u128, 10000);
                profiler::end(task_name.as_str());
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    profiler::dump_json(output_path);
}


fn global_task_ignore_thread(output_path: &str) {
    // For global tasks which not care about the thread id, use `profiler::start_multi` and
    // `profiler::end_multi` to record the time spent on each task.
    
    // Note that you can't start another task with the same name before the previous task is ended.
    // Each task with the same name will have an auto-incremented ID in the output json file.
    profiler::clear();
    let thread_num = 3;
    let sub_task_num = 5;
    let mut handles = vec![];

    for thread_id in 0..thread_num {
        let handle = thread::spawn(move || {
            for task_id in 0..sub_task_num {
                let seed = thread_id * sub_task_num + task_id;
                let task_name = format!("task-in-thread-{}", thread_id);
                profiler::start_multi(task_name.as_str());
                profiler::note_str_multi(
                    task_name.as_str(), 
                    "seed", 
                    format!("The seed for this task is {}", seed).as_str()
                );
                expensive_calculation(seed as u128, 10000);
                profiler::end_multi(task_name.as_str());
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    profiler::dump_json(output_path);
}


fn global_record_ignore_start_and_end(output_path: &str) {
    // When you want to record some parameters without start and end, use `profiler::note_str_unchecked`
    profiler::clear();
    let thread_num = 3;
    let sub_task_num = 5;
    let mut handles = vec![];

    for thread_id in 0..thread_num {
        let handle = thread::spawn(move || {
            for task_id in 0..sub_task_num {
                let seed = thread_id * sub_task_num + task_id;
                let result = expensive_calculation(seed as u128, 10000);
                profiler::note_str_unchecked(
                    "result-for-tasks", 
                    format!("seed-{}", seed).as_str(), 
                    result.to_string().as_str()
                );
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    profiler::dump_json(output_path);
}


fn main() {
    fs::create_dir_all("outputs").unwrap();
    different_task_in_different_threads("outputs/profiler-output-0.json");
    global_task_ignore_thread("outputs/profiler-output-1.json");
    global_record_ignore_start_and_end("outputs/profiler-output-2.json");
}
