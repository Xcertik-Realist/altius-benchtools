use altius_benchtools::profiler;

#[test]
fn test_profiler() {
    for _ in 0..1000 {
        profiler::start("basic-time");
        profiler::end("basic-time");
    }
    for _ in 0..1000 {
        profiler::start_multi("multi");
        profiler::end_multi("multi");
    }
    for i in 0..1000 {
        profiler::start("note-unchecked");
        profiler::note_str_unchecked("note-unchecked", format!("key-{}", i).as_str(), format!("value-{}", i).as_str());
        profiler::end("note-unchecked");
    }

    profiler::dump_json("./tests/output.json");
}
