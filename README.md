# Schedule Queue

Rust async library for scheduling events in SystemTime.

Provides interface to schedule an event at given system time and
to wait till the occurrence of the first scheduled event.

## Example: Sleep Sort

Here is a simple example how the Schedule Queue can enable implemenation of
[sleep sort algorithm](https://stackoverflow.com/questions/6474318/what-is-the-time-complexity-of-the-sleep-sort).

This code reads space-separated integers from command-line args and prints each number to `stderr`
with delay in seconds equal to the number.

Full implementation can be found in
[examples/sleep_sort.rs](https://github.com/alexandervoronov/schedule-queue/blob/main/examples/sleep_sort.rs).

```rust
    let mut queue = ScheduleQueue::new();

    let numbers = env::args()
        .skip(1)
        .filter_map(|arg| arg.parse::<i32>().ok())
        .collect::<Vec<_>>();
    let arg_count = numbers.len();

    let now = SystemTime::now();
    numbers.into_iter().for_each(|value| {
        queue.schedule(value, add_seconds(now, value));
    });

    let start_time = Instant::now();
    for index in 0..arg_count {
        if index > 0 {
            eprint!(" ");
        }
        eprint!("{}", queue.next().await.0);
    }
    eprintln!(
        "\nPrinted {arg_count} numbers in {}ms",
        start_time.elapsed().as_millis()
    );
```

The example input and output for this sleep sort implementation:

```
% cargo run --example sleep_sort -- 1 5 -3 2 19 -7 -2 12 4
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/examples/sleep_sort 1 5 -3 2 19 -7 -2 12 4`
-7 -3 -2 1 2 4 5 12 19
Printed 9 numbers in 19004ms
```
