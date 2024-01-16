/// Implementation of async, tokio based observers. The approach
/// uses async channels instead of trait callbacks

use log::debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};
use std::fmt::{self, Debug, Formatter};

#[derive(Debug)]
struct StoredObserver<T> {
    tx: Sender<T>,
    id: u32,
}

impl<T> StoredObserver<T> {
    pub fn new(id: u32, tx: Sender<T>) -> Self {
        StoredObserver { tx, id }
    }
}

/// Async, multithreading-ready Observale that use channels instead of callbacks
pub struct ChObservable<T: Clone> {
    /// Registered bservers
    observers: Arc<Mutex<Vec<StoredObserver<T>>>>,
    /// Next available observerId for registrations
    next_id: u32,
}

impl<T: Clone + Debug> Debug for ChObservable<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChObservable")
            .field("observers", &self.observers)
            .field("next_id", &self.next_id)
            .finish()
    }
}

impl<T: Clone> ChObservable<T> {
    pub fn new() -> Self {
        /// creates a new object
        ChObservable {
            observers: Arc::new(Mutex::new(Vec::new())),
            next_id: 1,
        }
    }

    /// This function registers a new observer. It returns the ID of the registered
    /// observer and a channel receiver to get the new values
    ///
    pub async fn register(&mut self) -> (u32, Receiver<T>) {
        let mut g = self.observers.lock().await;
        let observers: &mut Vec<StoredObserver<T>> = &mut g;
        let id = self.next_id;
        self.next_id += 1;
        let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel(10);
        observers.push(StoredObserver::new(id, tx));
        debug!("register observer: id={}", id);
        (id, rx)
    }

    /// This function unregisters an observer.
    ///
    /// ## Arguments
    /// * `observer_id` - ID returned after the registration of an observer
    ///
    pub async fn unregister(&mut self, observer_id: u32) {
        let mut g = self.observers.lock().await;
        let observers: &mut Vec<StoredObserver<T>> = &mut g;
        let mut found: Option<usize> = None;
        debug!("receive unregister observer request: id={}", observer_id);
        for (i, e) in observers.iter().enumerate() {
            if e.id == observer_id {
                found = Some(i);
                break;
            }
        }
        if let Some(index_to_remove) = found {
            debug!("unregister observer request: id={}", observer_id);
            observers.remove(index_to_remove);
        }
    }

    /// Triggers the notification of the restistered observers.
    ///
    /// ## Arguments
    /// * `data` - data that should be passed to the observers
    pub async fn notify(&self, data: &T) -> Result<(), SendError<T>> {
        debug!("received notify request");
        let mut g = self.observers.lock().await;
        let observers: &mut Vec<StoredObserver<T>> = &mut g;
        debug!("start to notify ...");
        for o in observers {
            o.tx.send(data.clone()).await?;
        }
        debug!("notified.");
        Ok(())
    }
}

/// Observable wrapper around a specific value
pub struct ChObservedValue<T: Clone> {
    /// Value to be wrapped
    value: Arc<Mutex<Option<T>>>,
    /// Observable implementation
    observable: Arc<Mutex<ChObservable<Option<T>>>>,
}

impl<T: Clone> ChObservedValue<T> {
    /// Creates an new object
    pub fn new() -> Self {
        ChObservedValue {
            observable: Arc::new(Mutex::new(ChObservable::<Option<T>>::new())),
            value: Arc::new(Mutex::new(None)),
        }
    }


    async fn set_value_impl(&mut self, v: Option<T>) {
        let mut g = self.value.lock().await;
        let o: &mut Option<T> = &mut g;
        *o = v;
    }

    async fn notify_impl(&mut self, v: &Option<T>) {
        let mut g = self.observable.lock().await;
        let o: &mut ChObservable<Option<T>> = &mut g;
        let _ = o.notify(v).await;
    }

    /// Set a new value to the object. All registered observers are
    /// called to get notified.
    ///
    /// ## Arguments
    /// * `v` - value to set
    ///
    pub async fn set_value(&mut self, v: &T) {
        let new_v = Some(v.clone());
        self.set_value_impl(new_v.clone()).await;
        self.notify_impl(&new_v).await;
    }

    /// Reset the value of the object. All registered observers are
    /// called to get notified.
    ///
    pub async fn reset_value(&mut self) {
        let new_v = None;
        self.set_value_impl(None).await;
        self.notify_impl(&new_v).await;
    }

    /// This function registers a new observer. It returns the ID of the registered
    /// observer and a channel receiver to get the new values
    ///
    pub async fn register(&mut self) -> (u32, Receiver<Option<T>>) {
        let mut g = self.observable.lock().await;
        let o: &mut ChObservable<Option<T>> = &mut g;
        o.register().await
    }

    /// This function unregisters an observer.
    ///
    /// ## Arguments
    /// * `observer_id` - ID returned after the registration of an observer
    ///
    pub async fn unregister(&mut self, observer_id: u32) {
        let mut g = self.observable.lock().await;
        let o: &mut ChObservable<Option<T>> = &mut g;
        o.unregister(observer_id).await;
    }

    /// Returns a reference to the contained value
    pub fn value_ref(&self) -> &Arc<Mutex<Option<T>>> {
        &self.value
    }

    /// Returns a mutable reference to the contained value
    pub fn value_mutref(&mut self) -> &mut Arc<Mutex<Option<T>>> {
        &mut self.value
    }

}

#[cfg(test)]
mod tests {
    use log::debug;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::task::JoinHandle;
    use tokio::sync::mpsc::Receiver;

    use crate::chobservable::{ChObservable, ChObservedValue};

    #[derive(Debug)]
    struct ObserverObj {
        pub v: Arc<Mutex<Option<String>>>,
        observable: Arc<Mutex<ChObservable<String>>>,
        pub id: Option<u32>,
        h: Option<JoinHandle<()>>,
    }


    impl ObserverObj {
        pub fn new() -> Self {
            let o = ObserverObj {
                v: Arc::new(Mutex::new(None)),
                observable: Arc::new(Mutex::new(ChObservable::new())),
                id: None,
                h: None,
            };
            o
        }

        pub async fn observe(&mut self)-> (u32, Receiver<String>) {
            let mut g = self.observable.lock().await;
            let o: &mut ChObservable<String> = &mut g;
            o.register().await
        }

        pub async fn register(&mut self, cho: &mut ChObservable<String>) {
            let (id, mut rx) = cho.register().await;
            self.id = Some(id);
            let value = self.v.clone();
            let o = self.observable.clone();
            let h = tokio::spawn(async move {
                loop {
                    match rx.recv().await {
                        Some(s) => {
                            {
                                debug!("[id={}]received value, request lock ...", id);
                                let mut g = value.lock().await;
                                debug!("[id={}]received value, got lock.", id);
                                let v: &mut Option<String> = &mut g;
                                *v = Some(s.clone());
                            }
                            {
                                let x: &mut ChObservable<String>;
                                debug!("[id={}]request lock, to inform about values ...", id);
                                let mut og = o.lock().await;
                                debug!("[id={}]got lock, to inform about values", id);
                                x = &mut og;
                                let _ = x.notify(&s).await;
                            };
                        },
                        None => debug!("[id={}]received NONE value.", id),
                    };
                };
            });
            self.h = Some(h);
        }
    }

    async fn check_val(id: u32, ov: &Arc<Mutex<Option<String>>>, expected: &Option<String>) {
        let g = ov.lock().await;
        let v: &Option<String> = &g;
        println!("Observer [id={}], content: {:?}", id, v);
        assert_eq!(v, expected);
    }
    async fn check_val2(id: u32, rx: &mut Receiver<String>, expected: &String) {
        debug!("[id2={}]i am waiting to get informed ...", id);
        match rx.recv().await {
            Some(v) => {
                debug!("[id2={}]i was informed", id);
                assert_eq!(v, *expected);
            },
            None => {
                debug!("[id2={}]i was informed 2", id);
                assert!(false);
            },
        };
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_chobservable_single() {

        let mut cho: ChObservable<String> = ChObservable::new();
        let mut o1: ObserverObj = ObserverObj::new();
        o1.register(&mut cho).await;
        let (_, mut o1_rx) = o1.observe().await;
        let mut o2: ObserverObj = ObserverObj::new();
        o2.register(&mut cho).await;
        let (_, mut o2_rx) = o2.observe().await;
        let mut o3: ObserverObj = ObserverObj::new();
        o3.register(&mut cho).await;
        let (_, mut o3_rx) = o3.observe().await;
        let expected_none = None;
        check_val(o1.id.unwrap(), &o1.v, &expected_none).await;
        check_val(o2.id.unwrap(), &o2.v, &expected_none).await;
        check_val(o3.id.unwrap(), &o3.v, &expected_none).await;
        let t1 = "test-99".to_string();
        match cho.notify(&t1).await {
            Ok(()) => (),
            Err(_) => assert!(false, "receive error while notify"),
        };
    
        let expected_1 = Some(t1.clone());
        // since notify is async we have to way until the value have changed
        check_val2(o1.id.unwrap(), &mut o1_rx, &t1).await;
        check_val2(o2.id.unwrap(), &mut o2_rx, &t1).await;
        check_val2(o3.id.unwrap(), &mut o3_rx, &t1).await;
    
        let mut o4: ObserverObj = ObserverObj::new();
        o4.register(&mut cho).await;
        let (_, mut o4_rx) = o4.observe().await;
        check_val(o1.id.unwrap(), &o1.v, &expected_1).await;
        check_val(o2.id.unwrap(), &o2.v, &expected_1).await;
        check_val(o3.id.unwrap(), &o3.v, &expected_1).await;
        check_val(o4.id.unwrap(), &o4.v, &expected_none).await;
    
        let t2 = "test-999".to_string();
        match cho.notify(&t2).await {
            Ok(()) => (),
            Err(_) => assert!(false, "receive error while notify"),
        };
        check_val2(o1.id.unwrap(), &mut o1_rx, &t2).await;
        check_val2(o2.id.unwrap(), &mut o2_rx, &t2).await;
        check_val2(o3.id.unwrap(), &mut o3_rx, &t2).await;
        check_val2(o4.id.unwrap(), &mut o4_rx, &t2).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_chobservable() {
        let mut cho: ChObservable<String> = ChObservable::new();
        let mut o1: ObserverObj = ObserverObj::new();
        o1.register(&mut cho).await;
        let (_, mut o1_rx) = o1.observe().await;
        let mut o2: ObserverObj = ObserverObj::new();
        o2.register(&mut cho).await;
        let (_, mut o2_rx) = o2.observe().await;
        let mut o3: ObserverObj = ObserverObj::new();
        o3.register(&mut cho).await;
        let (_, mut o3_rx) = o3.observe().await;
        let expected_none = None;
        check_val(o1.id.unwrap(), &o1.v, &expected_none).await;
        check_val(o2.id.unwrap(), &o2.v, &expected_none).await;
        check_val(o3.id.unwrap(), &o3.v, &expected_none).await;
        let t1 = "test-99".to_string();
        match cho.notify(&t1).await {
            Ok(()) => (),
            Err(_) => assert!(false, "receive error while notify"),
        };
    
        let expected_1 = Some(t1.clone());
        // since notify is async we have to way until the value have changed
        check_val2(o1.id.unwrap(), &mut o1_rx, &t1).await;
        check_val2(o2.id.unwrap(), &mut o2_rx, &t1).await;
        check_val2(o3.id.unwrap(), &mut o3_rx, &t1).await;
    
        let mut o4: ObserverObj = ObserverObj::new();
        o4.register(&mut cho).await;
        let (_, mut o4_rx) = o4.observe().await;
        check_val(o1.id.unwrap(), &o1.v, &expected_1).await;
        check_val(o2.id.unwrap(), &o2.v, &expected_1).await;
        check_val(o3.id.unwrap(), &o3.v, &expected_1).await;
        check_val(o4.id.unwrap(), &o4.v, &expected_none).await;
    
        let t2 = "test-999".to_string();
        match cho.notify(&t2).await {
            Ok(()) => (),
            Err(_) => assert!(false, "receive error while notify"),
        };
        check_val2(o1.id.unwrap(), &mut o1_rx, &t2).await;
        check_val2(o2.id.unwrap(), &mut o2_rx, &t2).await;
        check_val2(o3.id.unwrap(), &mut o3_rx, &t2).await;
        check_val2(o4.id.unwrap(), &mut o4_rx, &t2).await;
    }

    async fn check_val3(id: u32, rx: &mut Receiver<Option<String>>, expected: &String) {
        debug!("[id2={}]i am waiting to get informed ...", id);
        match rx.recv().await {
            Some(v) => {
                debug!("[id2={}]i was informed", id);
                assert_eq!(v.unwrap(), *expected);
            },
            None => {
                debug!("[id2={}]i was informed 2", id);
                assert!(false);
            },
        };
    }

    async fn check_val5(id: u32, rx: &mut Receiver<Option<String>>) {
        debug!("[id2={}]i am waiting to get informed ...", id);
        match rx.recv().await {
            Some(o) => {
                debug!("[id2={}]i was informed", id);
                assert_eq!(o, Option::None);
            },
            None => {
                debug!("[id2={}]i was informed 2", id);
                assert!(false);
            },
        };
    }

    async fn check_val4(cho: &ChObservedValue<String>, expected: &Option<String>) {
        let r = cho.value_ref();
        let g = r.lock().await;
        let os: &Option<String> = &g;
        assert_eq!(*os, *expected);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_chobservedvalue() {
        let mut cho: ChObservedValue<String> = ChObservedValue::new();
        let (id1,mut rx1) = cho.register().await;
        let (id2,mut rx2) = cho.register().await;
        let (id3,mut rx3) = cho.register().await;

        check_val4(&cho, &Option::None).await;

        let t1 = "test-99".to_string();
        cho.set_value(&t1).await;

        let expected_1 = Some(t1.clone());
        // since notify is async we have to way until the value have changed
        check_val3(id1, &mut rx1, &t1).await;
        check_val3(id2, &mut rx2, &t1).await;
        check_val3(id3, &mut rx3, &t1).await;

        let (id4,mut rx4) = cho.register().await;

        check_val4(&cho, &expected_1).await;

        let t2 = "test-999".to_string();
        cho.set_value(&t2).await;

        check_val3(id1, &mut rx1, &t2).await;
        check_val3(id2, &mut rx2, &t2).await;
        check_val3(id3, &mut rx3, &t2).await;
        check_val3(id4, &mut rx4, &t2).await;

        let expected_2 = Some(t2);
        check_val4(&cho, &expected_2).await;

        cho.reset_value().await;

        check_val5(id1, &mut rx1).await;
        check_val5(id2, &mut rx2).await;
        check_val5(id3, &mut rx3).await;
        check_val5(id4, &mut rx4).await;
    }

}
