use schedule_queue::ScheduleQueue;
use std::{
    env,
    time::{Duration, Instant, SystemTime},
};

fn add_seconds(timepoint: SystemTime, offset_seconds: i32) -> SystemTime {
    if offset_seconds >= 0 {
        timepoint + Duration::from_secs(offset_seconds as u64)
    } else {
        timepoint - Duration::from_secs((-offset_seconds) as u64)
    }
}

/// Sort numbers fed as command-line args using sleep sort algorithm
/// https://stackoverflow.com/questions/6474318/what-is-the-time-complexity-of-the-sleep-sort
#[tokio::main]
async fn main() {
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
}
