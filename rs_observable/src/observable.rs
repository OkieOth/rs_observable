/// Simple thread safe observer pattern implementation

use std::sync::{Arc, Mutex};
use std::cell::RefCell;

pub trait Observer<T: Clone> {
    fn notify(&mut self, data: T);
}

struct StoredObserver<T: Clone> {
    pub id: u32,
    pub observer: Arc<RefCell<dyn Observer<T> + Send + Sync>>,
}

impl<T: Clone> StoredObserver<T> {
    pub fn new(id: u32, observer: Arc<RefCell<dyn Observer<T> + Send + Sync>>) -> Self {
        StoredObserver{
            id,
            observer,
        }
    }
}

pub struct Observable<T: Clone> {
    observers: Mutex<Vec<StoredObserver<T>>>,
    next_id: u32,
}

impl<T: Clone> Observable<T> {
    pub fn new() -> Self {
        Observable {
            observers: Mutex::new(Vec::new()),
            next_id: 1,
        }
    }

    pub fn register(&mut self, observer: Arc<RefCell<dyn Observer<T> + Send + Sync>>) -> u32 {
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
            o.observer.borrow_mut().notify(data.clone());
        }
    }

    pub fn notify_observers_borrowed(&self, data: &T) {
        let mut l = self.observers.lock().unwrap();
        let v: &mut Vec<StoredObserver<T>> = &mut l;

        for o in v {
            o.observer.borrow_mut().notify(data.clone());
        }
    }

}

mod tests {
    #![allow(dead_code)]
    use std::ops::Deref;
    use crate::observable::Observer;

    #[derive(Debug)]
    struct MyString(String);

    impl MyString {
        pub fn new(v: &str) -> MyString {
            MyString(v.to_string())
        }
    }

    impl Clone for MyString {
        fn clone(&self) -> Self {
            MyString(self.0.clone())
        }
    }

    impl PartialEq for MyString {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Deref for MyString {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    struct ObserverString {
        pub value: MyString,
    }

    impl ObserverString {
        pub fn new(v: &str) -> Self {
            ObserverString {
                value: MyString(v.to_string()),
            }
        }
    }

    impl Observer<MyString> for ObserverString {
        fn notify(&mut self, data: MyString) {
            println!("notify was called: {}", data.0);
            self.value = data;
        }
    }

    #[test]
    fn int_test() {
        use std::sync::Arc;
        use std::cell::RefCell;
        use crate::observable::Observable;


        let mut o = Observable::<MyString>::new();
        let s1 = Arc::new(RefCell::new(ObserverString::new("test1")));
        let s1_id = o.register(s1.clone());
        let s2 = Arc::new(RefCell::new(ObserverString::new("test2")));
        o.register(s2.clone());
        let s3 = Arc::new(RefCell::new(ObserverString::new("test3"))); 
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

        let s4 = Arc::new(RefCell::new(ObserverString::new("test20")));
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