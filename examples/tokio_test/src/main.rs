use log::{debug, info};
use env_logger::Env;
use std::default;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, Duration};

use rs_observable::ChObservable;

#[derive(Debug)]
struct ObserverObj {
    pub v: Arc<Mutex<Option<String>>>,
    pub id: Option<u32>,
    h: Option<JoinHandle<()>>,
}

impl ObserverObj {
    pub fn new() -> Self {
        let o = ObserverObj {
            v: Arc::new(Mutex::new(None)),
            id: None,
            h: None,
        };
        o
    }

    pub async fn register(&mut self, cho: &mut ChObservable<String>) {
        let (id, mut rx) = cho.register().await;
        self.id = Some(id);
        let value = self.v.clone();
        let h = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(s) => {
                        debug!("[id={}]received value, request lock ...", id);
                        let mut g = value.lock().unwrap();
                        debug!("[id={}]received value, got lock.", id);
                        let v: &mut Option<String> = &mut g;
                        *v = Some(s);
                    }
                    None => debug!("[id={}]received NONE value.", id),
                };
            };
        });

        self.h = Some(h);
    }
}

#[tokio::main]

async fn main() {
    fn check_val(id: u32, ov: &Arc<Mutex<Option<String>>>, expected: &Option<String>) {
        let g = ov.lock().unwrap();
        let v: &Option<String> = &g;
        println!("Observer [id={}], content: {:?}", id, v);
        assert_eq!(v, expected);
    }
    async fn check_val2(id: u32, ov: &Arc<Mutex<Option<String>>>, expected: &Option<String>) {
        let mut sleep_time = 100u64;
        let empty_str = "".to_string();
        for i in 0..10 {
            println!("try to get lock [id={}] ...", id);
            let g = ov.lock().unwrap();
            println!("got lock [id={}]", id);
            let v: &Option<String> = &g;
            println!("Observer [id={}], content: {:?}", id, v);
            if expected.is_none() {
                if v.is_none() {
                    println!("Observer [id={}], content: {:?}, loop: {}", id, v, i);
                    return;
                }
            } else {
                if let Some(c) = v {
                    if *c == *v.as_ref().unwrap_or_else(|| &empty_str) {
                        println!("Observer [id={}], content: {:?}, loop: {}", id, v, i);
                        return;
                    }
                }
            }
            let d = Duration::from_millis(sleep_time);
            sleep_time = sleep_time * 2;
            sleep(d).await;
        }
        let g = ov.lock().unwrap();
        let v: &Option<String> = &g;
        println!("Observer [id={}], content: {:?}, -", id, v);
        assert_eq!(v, expected);
    }

    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    info!("'rs_observable tokio_test started");

    let mut cho: ChObservable<String> = ChObservable::new();
    let mut o1: ObserverObj = ObserverObj::new();
    o1.register(&mut cho).await;
    let mut o2: ObserverObj = ObserverObj::new();
    o2.register(&mut cho).await;
    let mut o3: ObserverObj = ObserverObj::new();
    o3.register(&mut cho).await;
    let expected_none = None;
    check_val(o1.id.unwrap(), &o1.v, &expected_none);
    check_val(o2.id.unwrap(), &o2.v, &expected_none);
    check_val(o3.id.unwrap(), &o3.v, &expected_none);
    let t1 = "test-99".to_string();
    match cho.notify(&t1).await {
        Ok(()) => (),
        Err(_) => assert!(false, "receive error while notify"),
    };
    // let d = Duration::from_millis(500);
    // sleep(d).await;

    let expected_1 = Some(t1);
    // since notify is async we have to way until the value have changed
    check_val2(o1.id.unwrap(), &o1.v, &expected_1).await;
    check_val2(o2.id.unwrap(), &o2.v, &expected_1).await;
    check_val2(o3.id.unwrap(), &o3.v, &expected_1).await;

    let mut o4: ObserverObj = ObserverObj::new();
    o4.register(&mut cho).await;
    check_val2(o1.id.unwrap(), &o1.v, &expected_1).await;
    check_val2(o2.id.unwrap(), &o2.v, &expected_1).await;
    check_val2(o3.id.unwrap(), &o3.v, &expected_1).await;
    check_val2(o4.id.unwrap(), &o4.v, &expected_none).await;

    let t2 = "test-999".to_string();
    match cho.notify(&t2).await {
        Ok(()) => (),
        Err(_) => assert!(false, "receive error while notify"),
    };
    let expected_2 = Some(t2);
    check_val2(o1.id.unwrap(), &o1.v, &expected_2).await;
    check_val2(o2.id.unwrap(), &o2.v, &expected_2).await;
    check_val2(o3.id.unwrap(), &o3.v, &expected_2).await;
    check_val2(o4.id.unwrap(), &o4.v, &expected_2).await;
    
}
