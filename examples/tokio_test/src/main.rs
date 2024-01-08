use std::default;
use std::{sync::Arc};
use env_logger::Env;
use log::info;
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;

use rs_observable::ObservedValue;


#[derive(Default)]
struct TypeToObserve {
    i: i32,
    oi: Option<i32>,
    s: String,
    os: Option<String>,
}

impl Clone for TypeToObserve {
    fn clone(&self) -> Self {
        TypeToObserve {
            i: self.i,
            oi: self.oi.clone(),
            s: self.s.clone(),
            os: self.os.clone(),
        }
    }
}

unsafe impl Send for TypeToObserve {}



#[tokio::main]

async fn main() {
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    info!("'simple tokio example started");

    let o = Arc::new(Mutex::new(ObservedValue::<TypeToObserve>::new()));
    tokio::spawn(async move {
        let sleep_time = Duration::from_secs(1);
        info!("sleep ...");
        sleep( sleep_time ).await;
        {
            let v1 = TypeToObserve::default();
            let mut guard = o.lock().await;
            let ov = &mut guard;
            ov.set_value(&v1);
        }
    });
}
