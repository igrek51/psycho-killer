use std::borrow::BorrowMut;
use std::sync::{Mutex, MutexGuard, Once};
use std::time::{SystemTime, UNIX_EPOCH};

static mut LOGS_MESSAGES: Option<Mutex<Vec<String>>> = None;
static INIT_LOGS: Once = Once::new();

fn global_logs_list<'a>() -> &'a Mutex<Vec<String>> {
    INIT_LOGS.call_once(|| unsafe {
        *LOGS_MESSAGES.borrow_mut() = Some(Mutex::new(vec![]));
    });
    unsafe { LOGS_MESSAGES.as_ref().unwrap() }
}

pub fn log(msg: &str) {
    let mut guard: MutexGuard<'_, Vec<String>> = global_logs_list().lock().unwrap();
    let vector: &mut Vec<String> = &mut *guard;
    let time_str = current_time_str();
    vector.push(format!("[{}] {}", time_str, msg));
}

pub fn print_logs() {
    let guard: MutexGuard<'_, Vec<String>> = global_logs_list().lock().unwrap();
    let vector: Vec<String> = guard.clone();
    for log in vector {
        eprintln!("{}", log);
    }
}

pub fn current_time_str() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let seconds_since_epoch = now.as_secs();
    let hours = (seconds_since_epoch / 3600) % 24;
    let minutes = (seconds_since_epoch / 60) % 60;
    let seconds = seconds_since_epoch % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
