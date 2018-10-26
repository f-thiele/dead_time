extern crate rand;
extern crate threadpool;
extern crate num_cpus;

use std::env;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
use std::collections::HashMap;
use rand::Rng;
use std::io;
use std::io::Write;
use std::time::Instant;

struct Buffer {
    queue: u64, // events waiting to be read out
    limit: u64, // maximum amount of events we can store at the same time
    wait_time: u64, // countdown until we have succesfully read out an event
}

impl Buffer {
    pub fn add(&mut self) {
        // add an event, we check in the main loop whether it is free to accept events

        self.queue += 1;
        // println!("Added to queue. nEvents = {:}", self.queue);
    }

    pub fn read(&mut self) {
        // if the buffer contains events and is not busy reading we start a new read countdown
        if self.is_filled() && self.wait_time == 0 {
            self.wait_time = 424;
            // println!("Start reading new event. nEvents = {:}", self.queue);
        }
    }

    pub fn is_filled(&self) -> bool {
        // does the buffer contain events?
        self.queue > 0
    }

    pub fn free(&self) -> bool {
        // does the buffer have capacity to accept new events?
        self.queue < self.limit
    }

    pub fn step(&mut self) {
        // perform a buffer step, if it is filled and we are in readout mode we continue reading
        // the buffer out, if it was the last readout step we remove the event from the buffer

        if self.is_filled() && self.wait_time > 0 {
            // println!("Step. nEvents = {:}, wait_time = {:}", self.queue, self.wait_time);
            self.wait_time -= 1;
            if self.wait_time == 0 {
                self.queue -= 1;
            }
        }
    }
}

fn calc_dead_time(buffer_limit : u64, max_event: u64) -> (f64, f64) {
    let mut rng = rand::thread_rng();
    let l1a_prob : f64 = 75./40000.; // probability for a L1A to occur: 75 kHz over 40 MHz
    let mut block : u64 = 0; // counts remaining events that the L1A is blocked, 0 is free
    let mut n_l1a : u64 = 0; // counts of occured L1As
    let mut not_recorded : u64 = 0; // counts of events where L1A trigger but haven't been recorded
    let mut events : u64 = 0; // events simulated

    // initialize an empty buffer with a fixed limit and currently no readout
    let mut b = Buffer{queue : 0, limit : buffer_limit, wait_time : 0};

    loop { // do until we generated enough events

        // perform a buffer step, if it is filled and we are in readout mode we continue reading
        // the buffer out, if it was the last readout step we remove the event from the buffer and have
        // space to record a new one below
        b.step();

        if block > 0 {
            // the block variable has two uses: counting the L1A block and also counting the arrival time
            // of a triggered event until the buffer
            //
            // every event we see whether we have the L1A block existing, if yes we reduce it by one
            block -= 1;
            if block == 0 {
                // if reducing the L1A block by one leads to 0, this is the time the event arrives at the
                // buffer
                if b.free() {
                    // if the buffer is free to handle it, we add it
                    b.add();
                } else {
                    // if the buffer is not free, we discard the event and count it as dead time
                    not_recorded += 1;
                }
            }
        }

        b.read(); // check if we have events and start reading them

        let p : f64 = rng.gen_range(0.0, 1.0);
        events += 1;

        if p < l1a_prob {
            // L1A would be sent
            n_l1a += 1;

            if block == 0 {
                // we are currently not blocked
                // set now block timer to 5
                block = 5;
            } else {
                // we are blocked, do not record event, count as dead time
                not_recorded += 1;
            }
        }

        if events == max_event {
            // stop running after max_event have been generated
            let ratio : f64 = not_recorded as f64 / n_l1a as f64;
            return (ratio * 100., (ratio*(1.-ratio)/n_l1a as f64).sqrt()*100.)
        }
    }
}

fn main() {
    // use the following two values to loop over different buffer sizes
    let mut max_limit : u64 = 15; // default max buffer size
    let mut min_limit : u64 = 0;  // default min buffer size

    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        // first argument is maximum buffer size
        // optional but mandatory if minimum buffer size is requested
        max_limit = args[1].parse::<u64>().unwrap();
    }
    if args.len() >= 3 {
        // second argument is minimum buffer size, optional
        min_limit = args[2].parse::<u64>().unwrap();
    }

    let max_event : u64 = 1_000_000_000; // can be configurable but for now that's fine

    let pool = ThreadPool::new(num_cpus::get()); // start as many threads as we have CPUs
    let (tx, rx) = channel(); // define sending and receiving interface of channel

    let now = Instant::now();

    for l in min_limit..(max_limit+1) {
        let tx = tx.clone();
        pool.execute(move || {
                     let time_perc : (f64, f64) = calc_dead_time(l, max_event);
                     tx.send((l, time_perc)).expect("Could not send data!");});
    }

    let mut dead_times = HashMap::new();

    for finished in min_limit..(max_limit+1) {
        let (l, time_perc) = rx.recv().unwrap();
        print!("Finished {:.2}%.\r", (finished-min_limit) as f64 / max_limit as f64 * 100.);
        io::stdout().flush().unwrap();
        dead_times.insert(l, time_perc);
    }

    println!("Finished all computations. Will now output results below:");
    for l in min_limit..(max_limit+1) {
        let res : (f64, f64) = dead_times[&l];
        println!("Buffer size {:} =>  {:.3} +- {:.3}%", l, res.0, res.1);
    }
    let new_now = Instant::now();
    println!("Time for calculations: {:?}", new_now.duration_since(now));

}
