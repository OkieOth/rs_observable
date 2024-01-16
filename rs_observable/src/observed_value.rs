/// A single threaded observable wrapper, put around a monitored varlue

use crate::observable::{Observable, Observer};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::cell::RefCell;

/// Object that holds the monitored value and its observers
pub struct ObservedValue<T: Clone> {
    observable: Observable<Option<T>>,
    value: Option<T>,
}

impl<T: Clone> ObservedValue<T> {
    /// Create a new instance
    pub fn new() -> Self {
        ObservedValue {
            observable: Observable::<Option<T>>::new(),
            value: None,
        }
    }

    /// Set a new value to the object. All registered observers are
    /// called to get notified.
    /// 
    /// ## Arguments
    /// * `v` - value to set
    /// 
    pub fn set_value(&mut self, v: &T) {
        self.value = Some(v.clone());
        self.observable.notify_observers(Some(v.clone()));
    }

    /// Reset the value of the object. All registered observers are
    /// called to get notified.
    ///
    pub fn reset_value(&mut self) {
        self.value = None;
        self.observable.notify_observers(None);
    }

    /// This function registers a new observer. It returns the ID of the registered
    /// observer.
    /// 
    /// ## Arguments
    /// * `observer` - implementation of the Observer trait that should be registered
    /// 
    pub fn register(&mut self, observer: Rc<RefCell<dyn Observer<Option<T>> + Send + Sync>>) -> u32 {
        self.observable.register(observer)
    }

    /// This function unregisters an observer.
    /// 
    /// ## Arguments
    /// * `observer_id` - ID returned after the registration of an observer
    /// 
    pub fn unregister(&mut self, observer_id: u32) {
        self.observable.unregister(observer_id);
    }

}

impl<T: Clone> Deref for ObservedValue<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Clone> DerefMut for ObservedValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

mod tests {
    //#![allow(dead_code)]
    use crate::observed_value::ObservedValue;
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

    struct ObserverString {
        pub value: Option<MyString>,
    }

    impl ObserverString {
        pub fn new() -> Self {
            ObserverString {
                value: None,
            }
        }
    }

    impl Observer<Option<MyString>> for ObserverString {
        fn notify(&mut self, data: Option<MyString>) {
            println!("notify was called: {:?}", data);
            self.value = data;
        }
    }


    #[test]
    fn test_01() {
        use std::rc::Rc;
        use std::cell::RefCell;
        use crate::observable::Observable;

        let mut o = ObservedValue::<MyString>::new();


        let s1 = Rc::new(RefCell::new(ObserverString::new()));
        let s1_id = o.register(s1.clone());
        let s2 = Rc::new(RefCell::new(ObserverString::new()));
        o.register(s2.clone());
        let s3 = Rc::new(RefCell::new(ObserverString::new())); 
        o.register(s3.clone());


        assert!(s1.borrow().value.is_none());
        assert!(s2.borrow().value.is_none());
        assert!(s3.borrow().value.is_none());

        let v = MyString::new("test_01");
        o.set_value(&v);

        assert_eq!(*s1.borrow().value.as_ref().unwrap(), v);
        assert_eq!(*s2.borrow().value.as_ref().unwrap(), v);
        assert_eq!(*s3.borrow().value.as_ref().unwrap(), v);

        let s4 = Rc::new(RefCell::new(ObserverString::new()));
        o.register(s4.clone());

        assert_eq!(*s1.borrow().value.as_ref().unwrap(), v);
        assert_eq!(*s2.borrow().value.as_ref().unwrap(), v);
        assert_eq!(*s3.borrow().value.as_ref().unwrap(), v);
        assert!(s4.borrow().value.is_none());

        let v2 = MyString::new("test_02");
        o.set_value(&v2);

        assert_eq!(*s1.borrow().value.as_ref().unwrap(), v2);
        assert_eq!(*s2.borrow().value.as_ref().unwrap(), v2);
        assert_eq!(*s3.borrow().value.as_ref().unwrap(), v2);
        assert_eq!(*s4.borrow().value.as_ref().unwrap(), v2);

        o.unregister(s1_id);

        o.reset_value();

        assert_eq!(*s1.borrow().value.as_ref().unwrap(), v2);
        assert!(s2.borrow().value.is_none());
        assert!(s3.borrow().value.is_none());
        assert!(s4.borrow().value.is_none());
    }
}