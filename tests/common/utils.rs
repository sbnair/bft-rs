use rand::distributions::{Distribution, Normal, Uniform};
use std::fs::{self, read_dir};
use std::time::Duration;
use std::u64::MAX as MAX_U64;

use super::config::*;
use digest_hash::EndianInput;
use digest_hash::{BigEndian, Hash};
use log::info;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use sha2::{Digest, Sha256};

pub fn generate_block(byzantine: bool) -> Vec<u8> {
    let random_size = get_random_integer(BLOCK_SIZE) as usize;
    let size = if random_size < MAX_BLOCK_SIZE {
        if random_size < MIN_BLOCK_SIZE {
            MIN_BLOCK_SIZE
        } else {
            random_size
        }
    } else {
        MAX_BLOCK_SIZE
    };
    let mut vec = Vec::with_capacity(size);
    unsafe {
        vec.set_len(size);
    }
    let mark = if byzantine { 1u8 } else { 0u8 };
    let rand_num = get_random_integer(RANDOM_U8) as u8;
    vec.insert(0, mark);
    vec.insert(1, rand_num);
    vec
}

pub fn check_block_result(block: &[u8]) -> bool {
    !block.is_empty() && block[0] == 0u8
}

pub fn check_txs_result() -> bool {
    get_dice_result(CHECK_TXS_FAILED_RATE)
}

pub fn check_txs_delay() -> Duration {
    let rand_num = get_random_integer(CHECK_TXS_DELAY);
    let delay = if rand_num < MIN_DELAY {
        MIN_DELAY
    } else {
        rand_num
    };
    Duration::from_millis(delay)
}

pub fn sync_delay(height_diff: u64) -> Duration {
    if height_diff < 2 {
        return Duration::from_millis(0u64);
    }
    let rand_num = get_random_integer(SYNC_DELAY);
    let delay = if rand_num < MIN_DELAY {
        MIN_DELAY
    } else {
        rand_num
    };
    Duration::from_millis(delay * height_diff)
}

pub fn commit_delay() -> Duration {
    let rand_num = get_random_integer(COMMIT_DELAY);
    let delay = if rand_num < MIN_DELAY {
        MIN_DELAY
    } else {
        rand_num
    };
    Duration::from_millis(delay)
}

pub fn is_message_lost() -> bool {
    get_dice_result(MESSAGE_LOST_RATE)
}

pub fn message_delay() -> Duration {
    let rand_num = get_random_integer(MESSAGE_DELAY);
    let cost_time = if rand_num < MAX_DELAY {
        if rand_num < MIN_DELAY {
            MIN_DELAY
        } else {
            rand_num
        }
    } else {
        MAX_DELAY
    };
    Duration::from_millis(cost_time)
}

pub fn generate_address() -> Vec<u8> {
    let mut vec = Vec::with_capacity(ADDRESS_SIZE);
    for _i in 0..ADDRESS_SIZE {
        vec.push(get_random_integer(RANDOM_U8) as u8);
    }
    vec
}

pub fn hash(msg: &[u8]) -> Vec<u8> {
    let mut hasher = BigEndian::<Sha256>::new();
    hash_slice(msg, &mut hasher);
    let output = hasher.result().as_ref().to_vec();
    output
}

// simplified for test
pub fn sign(_hash_value: &[u8], address: &[u8]) -> Vec<u8> {
    address.to_vec()
}

pub fn clean_wal() {
    let mut i = 0;
    let mut dir = format!("{}{}", WAL_ROOT, i);
    while read_dir(&dir).is_ok() {
        fs::remove_dir_all(&dir).unwrap();
        i += 1;
        dir = format!("{}{}", WAL_ROOT, i);
    }
    info!("Successfully clean wal logs!");
}

pub fn clean_log_file(path: &str) {
    if fs::read(path).is_ok() {
        fs::remove_file(path).unwrap();
    }
}

pub fn set_log_file(path: &str, level: LevelFilter) {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} - {m}\n")))
        .build(path)
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(level))
        .unwrap();
    log4rs::init_config(config).unwrap();
}

fn get_dice_result(likelihood: f64) -> bool {
    let rand_num = get_random_integer(RANDOM_U64) as f64;
    let rate = rand_num / ((MAX_U64 - 1) as f64);
    rate > likelihood
}

pub enum RandomMode {
    Normal(f64, f64),
    Uniform(u64, u64),
}

fn get_random_integer(mode: RandomMode) -> u64 {
    let v;
    match mode {
        RandomMode::Normal(_, _) => {
            v = get_random_float(mode) as u64;
        }
        RandomMode::Uniform(lower_bound, upper_bound) => {
            let between = Uniform::from(lower_bound..upper_bound);
            v = between.sample(&mut rand::thread_rng());
        }
    }
    v
}

fn get_random_float(mode: RandomMode) -> f64 {
    let v;
    match mode {
        RandomMode::Normal(mean, standard_deviation) => {
            let normal = Normal::new(mean, standard_deviation);
            v = normal.sample(&mut rand::thread_rng());
        }
        RandomMode::Uniform(_, _) => {
            v = get_random_integer(mode) as f64;
        }
    }
    v
}

fn hash_slice<T, H>(slice: &[T], digest: &mut H)
where
    T: Hash,
    H: EndianInput,
{
    for elem in slice {
        elem.hash(digest);
    }
}