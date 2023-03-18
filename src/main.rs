use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

// Define a task to be executed by a worker
type Task = fn() -> i32;

// The shared state between threads
struct SharedState {
    tasks: Vec<Task>,
    results: Vec<i32>,
}

impl SharedState {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            results: Vec::new(),
        }
    }

    fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    fn get_task(&mut self) -> Option<Task> {
        self.tasks.pop()
    }

    fn add_result(&mut self, result: i32) {
        self.results.push(result);
    }

    fn get_results(&mut self) -> Vec<i32> {
        std::mem::take(&mut self.results)
    }
}

fn main() {
    // Create a shared state between threads
    let shared_state_mutex = Arc::new(Mutex::new(SharedState::new()));

    // Create a condition variable for notifying workers of new tasks
    let condvar = Arc::new(Condvar::new());

    // Create a central coordinator node
    let cc_node = {
        let worker_state = shared_state_mutex.clone();
        let worker_condvar = condvar.clone();

        thread::spawn(move || {
            let tasks: &[Task] = &[
                || {
                    thread::sleep(Duration::from_secs(1));
                    42
                },
                || {
                    thread::sleep(Duration::from_secs(2));
                    12
                },
                || {
                    thread::sleep(Duration::from_secs(3));
                    99
                },
                || {
                    thread::sleep(Duration::from_secs(4));
                    37
                },
                || {
                    thread::sleep(Duration::from_secs(5));
                    87
                },
                || {
                    thread::sleep(Duration::from_secs(6));
                    55
                },
                || {
                    thread::sleep(Duration::from_secs(7));
                    22
                },
                || {
                    thread::sleep(Duration::from_secs(8));
                    7
                },
                || {
                    thread::sleep(Duration::from_secs(9));
                    31
                },
                || {
                    thread::sleep(Duration::from_secs(10));
                    62
                },
            ];

            let mut shared_state = worker_state.lock().unwrap();
            for task in tasks {
                shared_state.add_task(*task);
            }

            // Notify workers of new tasks
            worker_condvar.notify_all();
        })
    };

    // Spawn 10 worker threads
    let mut threads = Vec::<thread::JoinHandle<()>>::new();
    for i in 0..10 {
        let worker_state = shared_state_mutex.clone();
        let worker_condvar = condvar.clone();
        let thread_id = format!("Worker {}", i);

        let thread = thread::spawn(move || {
            loop {
                // Wait for a task to be assigned
                let mut shared_state = worker_state.lock().unwrap();
                let task = match shared_state.get_task() {
                    Some(t) => t,
                    None => {
                        // No task available, wait for notification
                        shared_state = worker_condvar.wait(shared_state).unwrap();
                        continue;
                    }
                };

                // Execute the task and add the result to the shared state
                let result = task();
                shared_state.add_result(result);

                // Check if all tasks have been completed
                if shared_state.results.len() == 10 {
                    break;
                }

                // Notify the coordinator node that a task has been completed
                worker_condvar.notify_one();

                // Wait for a new task to be assigned
                shared_state = worker_condvar.wait(shared_state).unwrap();

                // Check if all tasks have been completed
                if shared_state.results.len() == 10 {
                    break;
                }
            }
        });
    }

    // Wait for the coordinator node to finish
    cc_node.join().unwrap();
    // print the results
    let mut shared_state = shared_state_mutex.lock().unwrap();
    println!("{:?}", shared_state.get_results());
}
