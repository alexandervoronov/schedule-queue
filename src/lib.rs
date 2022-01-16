use parking_lot::Mutex;
use priority_queue::PriorityQueue;
use std::{
    cmp::Reverse,
    future,
    hash::Hash,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::Notify;

pub struct ScheduleQueue<T: Hash + Eq> {
    queue: Arc<Mutex<PriorityQueue<T, Reverse<SystemTime>>>>,
    notify: Arc<Notify>,
}

impl<T: Hash + Eq> Default for ScheduleQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash + Eq> ScheduleQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(PriorityQueue::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Schedule event at the given time.
    /// If an equivalent event already existed, its previous scheduled time is returned.
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

    /// Remove an event from the schedule. Returns the event and its scheduled time.
    pub fn deschedule(&mut self, item: &T) -> Option<(T, SystemTime)> {
        let mut queue = self.queue.lock();
        queue
            .remove(item)
            .map(|(value, Reverse(old_time))| (value, old_time))
    }

    /// Returns immediately if the queue has an event at or past SystemTime::now(),
    /// otherwise blocks until the time of the earliest scheduled event.
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
