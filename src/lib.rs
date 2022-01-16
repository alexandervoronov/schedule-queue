use futures_util::future;
use parking_lot::Mutex;
use priority_queue::PriorityQueue;
use std::{
    cmp::Reverse,
    hash::Hash,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::Notify;

pub struct ScheduleQueue<T: Hash + Eq> {
    queue: Arc<Mutex<PriorityQueue<T, Reverse<SystemTime>>>>,
    notify: Arc<Notify>,
}

impl<T: Hash + Eq> ScheduleQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(PriorityQueue::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn schedule(&mut self, item: T, time: SystemTime) -> Option<SystemTime> {
        let (old_time, should_notify) = {
            let mut queue = self.queue.lock();
            let should_notify = queue
                .peek()
                .map(|(_, Reverse(time))| time)
                .map_or(true, |closest_time| time < *closest_time);

            let old_time = queue
                .push(item, Reverse(time))
                .map(|Reverse(old_time)| old_time);

            (old_time, should_notify)
        };

        if should_notify {
            self.notify.notify_one();
        }
        old_time
    }

    pub fn deschedule(&mut self, item: &T) -> Option<(T, SystemTime)> {
        let mut queue = self.queue.lock();
        queue
            .remove(item)
            .map(|(value, Reverse(old_time))| (value, old_time))
    }

    pub async fn next(&mut self) -> (T, SystemTime) {
        loop {
            let duration_till_next = {
                let mut queue = self.queue.lock();
                if let Some((_, &Reverse(closest_time))) = queue.peek() {
                    let system_now = SystemTime::now();
                    if closest_time <= system_now {
                        return queue
                            .pop()
                            .map(|(item, Reverse(time))| (item, time))
                            .unwrap(); // peek guarantees safety of `.unwrap()`
                    }
                    Some(closest_time.duration_since(system_now).unwrap_or_default())
                } else {
                    None
                }
            };
            tokio::select! {
                _ = Self::sleep_for_duration(duration_till_next) => {},
                _ = self.notify.notified() => {}
            };
        }
    }

    async fn sleep_for_duration(duration: Option<Duration>) {
        match duration {
            Some(duration) => tokio::time::sleep(duration).await,
            None => future::pending().await,
        }
    }
}
