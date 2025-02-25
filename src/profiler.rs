//! A high-performance profiler for tracing RPC server execution with multi-threaded support.
//! 
//! # Overview
//! 
//! The profiler module provides a comprehensive solution for tracing and profiling RPC server execution,
//! with support for multi-threaded environments, detailed event tracking, and flexible output formats.
//! 
//! # Features
//! 
//! - Task timing with precise start/end markers
//! - Thread-safe profiling in concurrent environments
//! - Rich event annotation system
//! - Multiple output formats (JSON, ZIP)
//! - Special handling for transaction and commit events
//! - Global singleton instance with thread-safe access
//! 
//! # Examples
//! 
//! ```rust
//! use altius_benchtools::profiler;
//! 
//! // Start timing a task
//! profiler::start("my_task");
//! 
//! // Add notes to the current task
//! profiler::note_str("my_task", "operation", "database_query");
//! profiler::note_str("my_task", "query_type", "SELECT");
//! 
//! // Record timing points
//! profiler::note_time("my_task", "query_start");
//! 
//! // End timing the task
//! profiler::end("my_task");
//! 
//! // Export results
//! profiler::dump_json("profile_output.json");
//! ```
//! 
//! # Multi-threaded Usage
//! 
//! The profiler is thread-safe and can be used across multiple threads:
//! 
//! ```rust
//! use std::thread;
//! use altius_benchtools::profiler;
//! 
//! let handle = thread::spawn(|| {
//!     profiler::start("worker_task");
//!     profiler::note_str("worker_task", "thread_info", "worker_1");
//!     // ... perform work ...
//!     profiler::end("worker_task");
//! });
//! ```
//! 
//! # Output Format
//! 
//! The profiler generates a structured JSON output containing:
//! 
//! - Task timing information (start time, duration)
//! - Thread identification
//! - Custom annotations and notes
//! - Special event types (transactions, commits)
//! 
//! Example output structure:
//! ```json
//! {
//!   "title": {},
//!   "details": [
//!     [
//!       {
//!         "type": "transaction",
//!         "tx": "task_name",
//!         "runtime": 1234567,
//!         "start": 1000000,
//!         "end": 2234567,
//!         "status": "success",
//!         "detail": {
//!           "operation": "database_query",
//!           "query_type": "SELECT"
//!         }
//!       }
//!     ]
//!   ]
//! }
//! ```
//! 
//! # Note on Thread Safety
//! 
//! The profiler uses a global singleton instance protected by a mutex to ensure thread safety.
//! All operations are atomic and can be safely performed from any thread.

use once_cell::sync::Lazy;
use serde_json::{json, Map, Value};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    sync::Mutex,
    thread::current,
    time::Instant,
};
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

/// Global profiler instance initialized lazily
static PROFILER: Lazy<Mutex<Profiler>> = Lazy::new(|| {
    Mutex::new(Profiler {
        genesis: Instant::now(),
        thread_tasks: HashMap::new(),
        global_tasks: HashMap::new(),
    })
});

#[derive(Debug)]
struct Profiler {
    genesis: Instant,
    thread_tasks: HashMap<
        String, // thread id
        HashMap<
            String, // task name
            Vec<(
                u128,               // start time
                Option<u128>,       // end time (optional)
                Map<String, Value>, // other description
            )>,
        >,
    >,
    global_tasks: HashMap<
        String,       // task name
        (u128, bool), // occurrence count & is ended
    >,
}

impl Profiler {
    /// Returns a reference to the global profiler instance
    fn global() -> &'static Mutex<Profiler> {
        &PROFILER
    }

    /// Gets the current thread's name as a string
    fn get_current_thread_name() -> String {
        let thread_id = current().id();
        format!("{:?}", thread_id)
    }

    /// Inserts a new thread into the profiler if it doesn't exist
    /// Returns true if the thread was newly inserted, false if it already existed
    fn insert_thread(&mut self, thread: &str) -> bool {
        let non_exist = !self.thread_tasks.contains_key(thread);
        if non_exist {
            self.thread_tasks.insert(thread.to_string(), HashMap::new());
        }
        non_exist
    }

    /// Inserts a new task for a specific thread if it doesn't exist
    /// Returns true if the task was newly inserted, false if it already existed
    fn insert_thread_task(&mut self, task: &str, thread: &str) -> bool {
        self.insert_thread(thread);
        let thread_tasks = self.thread_tasks.get_mut(thread).unwrap();
        let non_exist = !thread_tasks.contains_key(task);
        if non_exist {
            thread_tasks.insert(task.to_string(), vec![]);
        }
        non_exist
    }

    /// Inserts a new task for the current thread if it doesn't exist
    /// Returns true if the task was newly inserted, false if it already existed
    fn insert_current_thread_task(&mut self, task: &str) -> bool {
        self.insert_thread_task(task, &Profiler::get_current_thread_name())
    }

    /// Gets a reference to the events vector for a specific task and thread
    /// Panics if either the thread or task don't exist
    fn must_get(&self, task: &str, thread: &str) -> &Vec<(u128, Option<u128>, Map<String, Value>)> {
        self.thread_tasks.get(thread).unwrap().get(task).unwrap()
    }

    /// Gets a reference to the events vector for a specific task in the current thread
    /// Panics if either the thread or task don't exist
    fn must_get_current(&self, task: &str) -> &Vec<(u128, Option<u128>, Map<String, Value>)> {
        self.thread_tasks
            .get(&Profiler::get_current_thread_name())
            .unwrap()
            .get(task)
            .unwrap()
    }

    /// Gets a mutable reference to the events vector for a specific task and thread
    /// Panics if either the thread or task don't exist
    fn must_get_mut(
        &mut self,
        task: &str,
        thread: &str,
    ) -> &mut Vec<(u128, Option<u128>, Map<String, Value>)> {
        self.thread_tasks
            .get_mut(thread)
            .unwrap()
            .get_mut(task)
            .unwrap()
    }

    /// Gets a mutable reference to the events vector for a specific task in the current thread
    /// Panics if either the thread or task don't exist
    fn must_get_mut_current(
        &mut self,
        task: &str,
    ) -> &mut Vec<(u128, Option<u128>, Map<String, Value>)> {
        self.thread_tasks
            .get_mut(&Profiler::get_current_thread_name())
            .unwrap()
            .get_mut(task)
            .unwrap()
    }

    /// Clears all profiling data from the profiler
    fn clear(&mut self) {
        self.thread_tasks.clear();
    }
}

/// Returns the genesis time when the profiler was initialized.
/// 
/// This timestamp serves as the reference point for all timing measurements
/// in the profiler. All durations are calculated relative to this time.
/// 
/// # Returns
/// 
/// * `Instant` - The initialization timestamp of the profiler
pub fn get_genesis() -> Instant {
    let profiler = Profiler::global().lock().unwrap();
    profiler.genesis
}

/// Starts timing a new task in the current thread.
/// 
/// This function begins tracking a new task's execution time. Each task must be ended
/// with a corresponding call to [`end()`]. Multiple tasks can be tracked simultaneously,
/// but nested tasks of the same name are not supported.
/// 
/// # Arguments
/// 
/// * `task` - A string identifier for the task to be timed
/// 
/// # Panics
/// 
/// * Panics if the last event for this task name is not ended (i.e., if you try to start
///   a task that was already started but not ended)
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// profiler::start("database_query");
/// // ... perform database operation ...
/// profiler::end("database_query");
/// ```
pub fn start(task: &str) {
    let mut profiler = Profiler::global().lock().unwrap();
    let genesis = profiler.genesis;
    match profiler.insert_current_thread_task(task) {
        false => assert!(
            profiler.must_get_current(task).last().unwrap().1.is_some(),
            "the last event must be end"
        ),
        true => (),
    };
    profiler.must_get_mut_current(task).push((
        Instant::now().duration_since(genesis).as_nanos(),
        None,
        Map::new(),
    ));
}

/// Starts timing a new task that may be called multiple times with the same name.
/// 
/// This function is specifically designed for tasks that need to be executed multiple times
/// with the same base name. It automatically appends an index to the task name to differentiate
/// between multiple instances.
/// 
/// # Arguments
/// 
/// * `base_task` - The base name for the task. The actual task name will be `{base_task}-[{index}]`
/// 
/// # Panics
/// 
/// * Panics if the last event for this task name is not ended
/// 
/// # Thread Safety
/// 
/// * This function only works in the main thread
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// // First instance
/// profiler::start_multi("batch_process");
/// // ... process batch 1 ...
/// profiler::end_multi("batch_process");
/// 
/// // Second instance
/// profiler::start_multi("batch_process");
/// // ... process batch 2 ...
/// profiler::end_multi("batch_process");
/// ```
pub fn start_multi(base_task: &str) {
    let mut profiler = Profiler::global().lock().unwrap();
    let genesis = profiler.genesis;
    let count = match profiler.global_tasks.get_mut(base_task) {
        None => {
            profiler
                .global_tasks
                .insert(base_task.to_string(), (1, false));
            0
        }
        Some((count, is_ended)) => {
            assert!(*is_ended, "the last event must be end");
            *count += 1;
            *is_ended = false;
            *count - 1
        }
    };
    let task = &format!("{}-[{}]", base_task, count);
    match profiler.insert_thread_task(task, "main") {
        false => assert!(
            profiler.must_get(task, "main").last().unwrap().1.is_some(),
            "the last event must be end"
        ),
        true => (),
    };
    profiler.must_get_mut(task, "main").push((
        Instant::now().duration_since(genesis).as_nanos(),
        None,
        Map::new(),
    ));
}

/// Ends timing for a task in the current thread.
/// 
/// This function stops tracking a task's execution time and records its duration.
/// It must be called after a corresponding [`start()`] call for the same task.
/// 
/// # Arguments
/// 
/// * `task` - The string identifier of the task to end
/// 
/// # Panics
/// 
/// * Panics if the last event for this task was not started (i.e., if you try to end
///   a task that wasn't started)
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// profiler::start("api_request");
/// // ... perform API request ...
/// profiler::end("api_request");
/// ```
pub fn end(task: &str) {
    let mut profiler = Profiler::global().lock().unwrap();
    assert!(
        profiler.must_get_current(task).last().unwrap().1.is_none(),
        "the last event must be start"
    );
    profiler.must_get_mut_current(task).last_mut().unwrap().1 =
        Some(Instant::now().duration_since(profiler.genesis).as_nanos());
}

/// Ends timing for a task that was called multiple times.
/// 
/// This function ends timing for a task started with [`start_multi()`]. It must be called
/// after a corresponding start_multi call with the same base task name.
/// 
/// # Arguments
/// 
/// * `base_task` - The base name of the task to end (same as used in start_multi)
/// 
/// # Panics
/// 
/// * Panics if the last event for this task was not started
/// * Panics if called from a non-main thread
/// 
/// # Thread Safety
/// 
/// * This function only works in the main thread
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// // First batch
/// profiler::start_multi("batch_process");
/// // ... process batch 1 ...
/// profiler::end_multi("batch_process"); // Ends "batch_process-[0]"
/// 
/// // Second batch
/// profiler::start_multi("batch_process");
/// // ... process batch 2 ...
/// profiler::end_multi("batch_process"); // Ends "batch_process-[1]"
/// ```
pub fn end_multi(base_task: &str) {
    let mut profiler = Profiler::global().lock().unwrap();
    let (count, is_ended) = profiler.global_tasks.get_mut(base_task).unwrap();
    assert!(!*is_ended, "the last event must not be end");
    *is_ended = true;
    let task = &format!("{}-[{}]", base_task, *count - 1);
    assert!(
        profiler.must_get(task, "main").last().unwrap().1.is_none(),
        "the last event must be start"
    );
    profiler.must_get_mut(task, "main").last_mut().unwrap().1 =
        Some(Instant::now().duration_since(profiler.genesis).as_nanos());
}

/// Adds a key-value note to the last event of a task.
/// 
/// This function allows you to annotate a task with additional metadata in the form
/// of key-value pairs. The value can be any valid JSON value.
/// 
/// # Arguments
/// 
/// * `task` - The string identifier of the task to annotate
/// * `key` - The key for the metadata entry
/// * `value` - The value to associate with the key (any valid JSON value)
/// 
/// # Panics
/// 
/// * Panics if the last event was not started (i.e., if you try to add a note to
///   a task that wasn't started or was already ended)
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// use serde_json::json;
/// 
/// profiler::start("http_request");
/// profiler::note("http_request", "method", json!("POST"));
/// profiler::note("http_request", "headers", json!({
///     "Content-Type": "application/json",
///     "Authorization": "Bearer token"
/// }));
/// // ... perform request ...
/// profiler::end("http_request");
/// ```
pub fn note(task: &str, key: &str, value: Value) {
    let mut profiler = Profiler::global().lock().unwrap();
    assert!(
        profiler.must_get_current(task).last().unwrap().1.is_none(),
        "the last event must be start"
    );
    profiler
        .must_get_mut_current(task)
        .last_mut()
        .unwrap()
        .2
        .insert(key.to_string(), value);
}

/// Adds a string key-value note to the last event of a task.
/// 
/// This is a convenience wrapper around [`note()`] that automatically converts
/// the string value to a JSON string value.
/// 
/// # Arguments
/// 
/// * `task` - The string identifier of the task to annotate
/// * `key` - The key for the metadata entry
/// * `value` - The string value to associate with the key
/// 
/// # Panics
/// 
/// * Panics if the last event was not started
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// profiler::start("request");
/// profiler::note_str("request", "endpoint", "/api/v1/users");
/// profiler::note_str("request", "method", "GET");
/// // ... perform request ...
/// profiler::end("request");
/// ```
pub fn note_str(task: &str, key: &str, value: &str) {
    note(task, key, Value::String(value.to_string()));
}

/// Adds multiple key-value notes to the last event of a task.
/// 
/// This function allows you to add multiple annotations at once by providing
/// a map of key-value pairs.
/// 
/// # Arguments
/// 
/// * `task` - The string identifier of the task to annotate
/// * `description` - A mutable map containing the key-value pairs to add
/// 
/// # Panics
/// 
/// * Panics if the last event was not started
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// use serde_json::{Map, Value, json};
/// 
/// profiler::start("query");
/// let mut desc = Map::new();
/// desc.insert("table".to_string(), json!("users"));
/// desc.insert("type".to_string(), json!("SELECT"));
/// desc.insert("filter".to_string(), json!({"age": ">= 18"}));
/// profiler::notes("query", &mut desc);
/// // ... perform query ...
/// profiler::end("query");
/// ```
pub fn notes(task: &str, description: &mut Map<String, Value>) {
    let mut profiler = Profiler::global().lock().unwrap();
    profiler
        .must_get_mut_current(task)
        .last_mut()
        .unwrap()
        .2
        .append(description);
}

/// Adds the current time as a value for a key in the last event of a task.
/// 
/// This function records the current timestamp relative to the profiler's genesis time
/// and adds it as a note to the task.
/// 
/// # Arguments
/// 
/// * `task` - The string identifier of the task to annotate
/// * `key` - The key under which to store the timestamp
/// 
/// # Panics
/// 
/// * Panics if the last event was not started
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// profiler::start("long_operation");
/// // ... initial setup ...
/// profiler::note_time("long_operation", "setup_complete");
/// // ... main work ...
/// profiler::note_time("long_operation", "work_complete");
/// // ... cleanup ...
/// profiler::end("long_operation");
/// ```
pub fn note_time(task: &str, key: &str) {
    let mut profiler = Profiler::global().lock().unwrap();
    let genesis = profiler.genesis;
    assert!(
        profiler.must_get_current(task).last().unwrap().1.is_none(),
        "the last event must be start"
    );
    profiler
        .must_get_mut_current(task)
        .last_mut()
        .unwrap()
        .2
        .insert(
            key.to_string(),
            (Instant::now().duration_since(genesis).as_nanos() as u64).into(),
        );
}

/// Adds a string key-value note to the last event of a task that was called multiple times.
/// 
/// This function adds a string note to a task created with [`start_multi()`]. It must be used
/// with the base task name (without the index suffix).
/// 
/// # Arguments
/// 
/// * `base_task` - The base name of the task (same as used in start_multi)
/// * `key` - The key for the metadata entry
/// * `value` - The string value to associate with the key
/// 
/// # Panics
/// 
/// * Panics if the last event was not started
/// * Panics if called from a non-main thread
/// 
/// # Thread Safety
/// 
/// * This function only works in the main thread
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// profiler::start_multi("batch_job");
/// profiler::note_str_multi("batch_job", "status", "processing");
/// // ... process batch ...
/// profiler::note_str_multi("batch_job", "status", "completed");
/// profiler::end_multi("batch_job");
/// ```
pub fn note_str_multi(base_task: &str, key: &str, value: &str) {
    let mut profiler = Profiler::global().lock().unwrap();
    let (count, is_ended) = profiler.global_tasks.get_mut(base_task).unwrap();
    assert!(!*is_ended, "the last event must not be end");
    let task = &format!("{}-[{}]", base_task, *count - 1);
    assert!(
        profiler.must_get(task, "main").last().unwrap().1.is_none(),
        "the last event must be start"
    );
    profiler
        .must_get_mut(task, "main")
        .last_mut()
        .unwrap()
        .2
        .insert(key.to_string(), Value::String(value.to_string()));
}

/// Adds a string key-value note to a task without the usual safety checks.
/// 
/// This is an unchecked version of [`note_str()`] that will create a new task entry
/// if one doesn't exist. Use with caution as it bypasses the normal start/end task flow.
/// 
/// # Arguments
/// 
/// * `task` - The string identifier of the task to annotate
/// * `key` - The key for the metadata entry
/// * `value` - The string value to associate with the key
/// 
/// # Safety
/// 
/// This function is marked as unchecked because it:
/// - Does not verify if the task exists
/// - Creates a new task entry if none exists
/// - Does not enforce the normal start/end task flow
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// // Note: This bypasses normal task flow - use with caution
/// profiler::note_str_unchecked("background_task", "status", "running");
/// ```
pub fn note_str_unchecked(task: &str, key: &str, value: &str) {
    let mut profiler = Profiler::global().lock().unwrap();
    let genesis = profiler.genesis;
    match profiler.insert_thread_task(task, "main") {
        true => profiler.must_get_mut(task, "main").push((
            Instant::now().duration_since(genesis).as_nanos(),
            None,
            Map::new(),
        )),
        false => (),
    };
    profiler
        .must_get_mut(task, "main")
        .last_mut()
        .unwrap()
        .2
        .insert(key.to_string(), Value::String(value.to_string()));
}

/// Clears all profiling data from the profiler.
/// 
/// This function removes all recorded tasks, events, and their associated metadata
/// from the profiler. The genesis time is preserved.
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// // After some profiling...
/// profiler::clear(); // Reset profiler state
/// ```
pub fn clear() {
    let mut profiler = Profiler::global().lock().unwrap();
    profiler.clear();
}

/// Dumps the profiler data as a JSON string.
/// 
/// This function exports all profiling data in a structured JSON format. The output
/// includes timing information, thread identification, custom annotations, and special
/// event types (transactions, commits).
/// 
/// # Returns
/// 
/// * `String` - A pretty-printed JSON string containing all profiling data
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// // ... perform profiling ...
/// 
/// let json_data = profiler::dump();
/// println!("Profile data: {}", json_data);
/// ```
/// 
/// # Output Format
/// 
/// The output JSON has the following structure:
/// ```json
/// {
///   "title": {},
///   "details": [
///     [
///       {
///         "type": "transaction",
///         "tx": "task_name",
///         "runtime": 1234567,
///         "start": 1000000,
///         "end": 2234567,
///         "status": "success",
///         "detail": { ... }
///       }
///     ]
///   ]
/// }
/// ```
pub fn dump() -> String {
    let profiler = Profiler::global().lock().unwrap();
    let now = Instant::now().duration_since(profiler.genesis).as_nanos();

    let mut output_frontend = Value::Array(vec![]);

    for (_thread_name, thread_events) in &profiler.thread_tasks {
        let mut detail = vec![];
        for (name, thread_tasks) in thread_events {
            for event in thread_tasks {
                let (start, end_opt, description) = event;
                let duration = end_opt.unwrap_or(now) - start;

                match description.get("type") {
                    Some(Value::String(type_str)) => match type_str.as_str() {
                        "transaction" => detail.push(json!({
                            "type": "transaction",
                            "tx": name,
                            "runtime": duration,
                            "start": start,
                            "end": end_opt,
                            // "status": description.get("status").unwrap().as_str().unwrap_or("unknown"),
                            "status": match description.get("status") {
                                Some(value) => value.as_str().unwrap_or("unknown"),
                                None => "unknown",
                            },
                            "detail": description,
                        })),
                        "commit" => detail.push(json!({
                            "type": "commit",
                            "tx": match description.get("tx") {
                                Some(value) => value.as_str().unwrap_or("unknown"),
                                None => "unknown",
                            },
                            "runtime": duration,
                            "start": start,
                            "end": end_opt,
                            "detail": description,
                        })),
                        other_type => detail.push(json!({
                            "type": other_type,
                            "name": name,
                            "runtime": duration,
                            "start": start,
                            "end": end_opt,
                            "detail": description,
                        })),
                    },
                    _ => detail.push(json!({
                        "type": "other",
                        "name": name,
                        "runtime": duration,
                        "start": start,
                        "end": end_opt,
                        "detail": description,
                    })),
                }
            }
        }
        output_frontend
            .as_array_mut()
            .unwrap()
            .push(Value::Array(detail));
    }

    serde_json::to_string_pretty(&output_frontend).unwrap()
}

/// Dumps the profiler data to a JSON file at the specified path.
/// 
/// This function writes all profiling data to a file in a pretty-printed JSON format.
/// It's a convenience wrapper around [`dump()`] that handles file I/O.
/// 
/// # Arguments
/// 
/// * `output_path` - The path where the JSON file should be written
/// 
/// # Panics
/// 
/// * Panics if the file cannot be created or written to
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// // After some profiling...
/// profiler::dump_json("profile_results.json");
/// ```
pub fn dump_json(output_path: &str) {
    let result_json = dump();
    let mut file = File::create(output_path).unwrap();
    file.write_all(result_json.as_bytes()).unwrap();
}

/// Dumps the profiler data to a ZIP file containing a JSON file.
/// 
/// This function exports all profiling data to a compressed ZIP file containing
/// a JSON file. The ZIP file will contain a single JSON file with the same base name.
/// This is useful for storing large profiling datasets efficiently.
/// 
/// # Arguments
/// 
/// * `output_name` - The base name for the output files (without extension)
/// 
/// # Panics
/// 
/// * Panics if the ZIP file cannot be created or written to
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// // After some profiling...
/// profiler::dump_zip("profile_results");
/// // Creates profile_results.zip containing profile_results.json
/// ```
pub fn dump_zip(output_name: &str) {
    let result_json = dump();
    let file = File::create(output_name.to_string() + ".zip").unwrap();
    let mut zip = ZipWriter::new(BufWriter::new(file));
    let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);
    zip.start_file(output_name.to_string() + ".json", options)
        .unwrap();
    zip.write_all(result_json.as_bytes()).unwrap();
    zip.finish().unwrap();
}

/// Prints the current state of the profiler for debugging purposes.
/// 
/// This function prints a debug representation of the entire profiler state
/// to stdout. This is primarily intended for development and debugging use.
/// 
/// # Examples
/// 
/// ```rust
/// use altius_benchtools::profiler;
/// 
/// // After some profiling...
/// profiler::debug_print(); // Prints internal profiler state
/// ```
pub fn debug_print() {
    println!("Profiler: {:?}", PROFILER);
}
