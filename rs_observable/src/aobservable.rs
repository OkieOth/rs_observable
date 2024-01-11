/// Simple thread safe observer pattern implementation

use std::sync::{Arc, Mutex};

use crate::observable::Observer;

struct StoredObserver<T: Clone> {
    pub id: u32,
    pub observer: Arc<Mutex<Box<dyn Observer<T> + Send + Sync>>>,
}

impl<T: Clone> StoredObserver<T> {
    pub fn new(id: u32, observer: Arc<Mutex<Box<dyn Observer<T> + Send + Sync>>>) -> Self {
        StoredObserver{
            id,
            observer,
        }
    }
}

pub struct AObservable<T: Clone> {
    observers: Mutex<Vec<StoredObserver<T>>>,
    next_id: u32,
}

impl<T: Clone> AObservable<T> {
    pub fn new() -> Self {
        AObservable {
            observers: Mutex::new(Vec::new()),
            next_id: 1,
        }
    }

    pub fn register(&mut self, observer: Arc<Mutex<Box<dyn Observer<T> + Send + Sync>>>) -> u32 {
        let mut l = self.observers.lock().unwrap();
        let v: &mut Vec<StoredObserver<T>> = &mut l;
        let id = self.next_id;
        self.next_id += 1;
        v.push(StoredObserver::new(id, observer));
        id
    }

    pub fn unregister(&mut self, observer_id: u32) {
        let mut l = self.observers.lock().unwrap();
        let v: &mut Vec<StoredObserver<T>> = &mut l;
        let mut found: Option<usize> = None;
        for (i, e) in v.iter().enumerate() {
            if e.id == observer_id {
                found = Some(i);
                break;
            }
        }
        if let Some(index_to_remove) = found {
            v.remove(index_to_remove);
        }
    }

    pub fn notify_observers(&self, data: T) {
        let mut l = self.observers.lock().unwrap();
        let v: &mut Vec<StoredObserver<T>> = &mut l;

        for o in v {
            let mut g = o.observer.lock().unwrap();
            let obs: &mut Box<dyn Observer<T> + Send + Sync> = &mut g;
            obs.notify(data.clone());
        }
    }

    pub fn notify_observers_borrowed(&self, data: &T) {
        let mut l = self.observers.lock().unwrap();
        let v: &mut Vec<StoredObserver<T>> = &mut l;

        for o in v {
            let mut g = o.observer.lock().unwrap();
            let obs: &mut Box<dyn Observer<T> + Send + Sync> = &mut g;
            obs.notify(data.clone());
        }
    }

}

mod tests {
    #![allow(dead_code)]
    use std::ops::Deref;
    use std::sync::{Arc, Mutex};
    use crate::observable::Observer;

    #[derive(Debug)]

    struct ObserverString {
        pub value: String,
    }

    impl ObserverString {
        pub fn new(v: &str) -> Self {
            ObserverString {
                value: v.to_string(),
            }
        }
    }

    impl Observer<String> for ObserverString {
        fn notify(&mut self, data: String) {
            println!("notify was called: {}", data);
            self.value = data;
        }
    }

    // fn check_value(s: &Arc<Mutex<Box<dyn Observer<MyString> + Send + Sync>>>, value_to_compare: &str) {
    //     let mut g = s.lock().unwrap();
    //     let sv: &mut Box<dyn Observer<MyString> + Sync + Send> = &mut g;
    //     assert(&sv.)
    // }

    #[test]
    fn int_test() {
        use std::sync::{Arc, Mutex};
        use crate::aobservable::AObservable;


        let mut o = AObservable::<String>::new();
        let s1 = Arc::new(Mutex::new(Box::new(ObserverString::new("test1"))));
        let s1_id = o.register(s1 as Arc<Mutex<Box<dyn Observer<String> + Send + Sync>>>);
        let s2 = Arc::new(Mutex::new(Box::new(ObserverString::new("test2")) as Box<dyn Observer<String> + Send + Sync>));
        o.register(s2.clone());
        let s3 = Arc::new(Mutex::new(Box::new(ObserverString::new("test3")) as Box<dyn Observer<String> + Send + Sync>)); 
        o.register(s3.clone());

        assert_eq!(s1.borrow().value, MyString::new("test1"));
        assert_eq!(s2.borrow().value, MyString::new("test2"));
        assert_eq!(s3.borrow().value, MyString::new("test3"));

        o.notify_observers(MyString::new("test4"));

        assert_eq!(s1.borrow().value, MyString::new("test4"));
        assert_eq!(s2.borrow().value, MyString::new("test4"));
        assert_eq!(s3.borrow().value, MyString::new("test4"));

        o.unregister(s1_id);

        o.notify_observers(MyString::new("test5"));

        assert_eq!(s1.borrow().value, MyString::new("test4"));
        assert_eq!(s2.borrow().value, MyString::new("test5"));
        assert_eq!(s3.borrow().value, MyString::new("test5"));

        let s4 = Arc::new(Mutex::new(Box::new(ObserverString::new("test20")) as Box<dyn Observer<MyString> + Send + Sync>));
        o.register(s4.clone());

        assert_eq!(s1.borrow().value, MyString::new("test4"));
        assert_eq!(s2.borrow().value, MyString::new("test5"));
        assert_eq!(s3.borrow().value, MyString::new("test5"));
        assert_eq!(s4.borrow().value, MyString::new("test20"));

        o.notify_observers(MyString::new("test21"));

        assert_eq!(s1.borrow().value, MyString::new("test4"));
        assert_eq!(s2.borrow().value, MyString::new("test21"));
        assert_eq!(s3.borrow().value, MyString::new("test21"));
        assert_eq!(s4.borrow().value, MyString::new("test21"));
    }
}