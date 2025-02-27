use altius_benchtools::profiler;
use std::{fs, thread};

/// Performs an expensive calculation for benchmarking purposes.
///
/// This function simulates a CPU-intensive task by performing repeated
/// modular exponentiation operations with a large prime modulus.
///
/// # Arguments
///
/// * `seed` - A seed value that affects the initial state of the calculation
/// * `repeat` - The number of iterations to perform
///
/// # Returns
///
/// The final result of the calculation after repeated squaring operations
fn expensive_calculation(seed: u128, repeat: u128) -> u128 {
    let modulus: u128 = 18446744073709551617;
    let mut base: u128 = 12142100291992418551 + seed;
    for _ in 0..repeat {
        base = (base * base) % modulus;
    }
    base
}

/// Demonstrates profiling different tasks across multiple threads.
///
/// This example shows how to use thread-specific profiling functions to track
/// the execution of distinct tasks running in separate threads. Each task is
/// uniquely identified by its name and thread ID.
///
/// The pattern used here is appropriate when:
/// - You need to track multiple distinct tasks across different threads
/// - Each task needs its own timing and metadata
/// - Tasks are uniquely identified by both name and thread ID
///
/// # Arguments
///
/// * `output_path` - Path where the JSON profiling data will be saved
///
/// # Notes
///
/// * Each task must have a unique name within its thread
/// * A task must be ended in the same thread where it was started
/// * Tasks cannot be nested with the same name in the same thread
///
/// # Example Output
///
/// * See `outputs/profiler-output-0.json` after running the example
fn different_task_in_different_threads(output_path: &str) {
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
                    format!("You can add any extra description here for {}", task_name).as_str(),
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

/// Demonstrates profiling tasks across multiple threads with global task tracking.
///
/// This example shows how to use thread-agnostic profiling functions to track
/// tasks that may be executed across different threads. Each task is identified
/// by its base name, and an auto-incremented index is appended to distinguish
/// multiple instances of the same task.
///
/// The pattern used here is appropriate when:
/// - You need to track tasks that may span multiple threads
/// - You want to reuse the same task name multiple times
/// - Task identity is more important than which thread executed it
///
/// # Arguments
///
/// * `output_path` - Path where the JSON profiling data will be saved
///
/// # Notes
///
/// * Tasks with the same name are automatically given unique identifiers
/// * A task must be ended before starting another with the same name
/// * Each task instance is tracked with an auto-incremented ID in the output
/// * All entries are recorded in the "main" thread regardless of actual thread
fn global_task_ignore_thread(output_path: &str) {
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
                    format!("The seed for this task is {}", seed).as_str(),
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

/// Demonstrates recording profiling data without explicit start/end markers.
///
/// This example shows how to use the unchecked profiling functions to record
/// data without following the normal start/end task flow. This is useful for
/// recording arbitrary data points or results without timing specific tasks.
///
/// The pattern used here is appropriate when:
/// - You need to record data points without timing concerns
/// - You want to collect results or metrics from various operations
/// - You don't need the strict start/end task structure
///
/// # Arguments
///
/// * `output_path` - Path where the JSON profiling data will be saved
///
/// # Notes
///
/// * This approach bypasses the normal task flow and safety checks
/// * Useful for recording results or metrics without timing concerns
/// * Creates task entries automatically if they don't exist
/// * All entries are recorded in the "main" thread regardless of actual thread
///
/// # Safety
///
/// This function uses `note_str_unchecked` which:
/// - Does not verify if the task exists
/// - Creates a new task entry if none exists
/// - Does not enforce the normal start/end task flow
fn global_record_ignore_start_and_end(output_path: &str) {
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
                    result.to_string().as_str(),
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

/// Runs all profiler examples and saves the output to JSON files.
///
/// This function demonstrates the three main profiling patterns:
/// 1. Thread-specific task profiling - Records distinct tasks in different threads
/// 2. Global task profiling across threads - Records tasks with auto-incremented IDs
/// 3. Unchecked data recording without start/end markers - Records data points without timing
///
/// Each example generates a separate JSON output file in the "outputs" directory.
fn main() {
    fs::create_dir_all("outputs").unwrap();
    different_task_in_different_threads("outputs/profiler-output-0.json");
    global_task_ignore_thread("outputs/profiler-output-1.json");
    global_record_ignore_start_and_end("outputs/profiler-output-2.json");
}
