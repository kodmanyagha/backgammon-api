use std::{sync::Arc, thread, time::Duration};

use parking_lot::Mutex;

#[derive(Clone)]
struct User {
    id: u64,
    email: String,
}

unsafe impl Send for User {}
unsafe impl Sync for User {}

#[test]
fn send_sync_showcase_1() {
    let user = User {
        id: 10,
        email: "user1@test.com".to_string(),
    };

    let user_clone = user.clone();
    thread::spawn(move || run_me("th1".to_string(), user));
    thread::spawn(move || run_me("th2".to_string(), user_clone));

    thread::sleep(Duration::from_secs(2));
}

fn run_me(thread_name: String, user: User) {
    for i in 0..=10 {
        println!(
            "{} user id: {} , email: {} , idx: {}",
            thread_name, user.id, user.email, i
        );

        thread::sleep(Duration::from_millis(100));
    }
}

#[derive(Default, Clone, Debug)]
struct ExampleStruct {
    pub int_val: i32,
    pub arc_val: Arc<Mutex<i32>>,
}

#[test]
fn test_clone_struct() {
    let mut foo1 = ExampleStruct::default();
    let mut foo2 = foo1.clone();
    let mut foo3 = foo1.clone();

    foo1.int_val = 10;
    foo2.int_val = 20;
    foo3.int_val = 30;

    *foo1.arc_val.lock() += 10;
    *foo2.arc_val.lock() += 10;

    println!("foo1: {:?}", foo1);
    println!("foo2: {:?}", foo2);
    println!("foo3: {:?}", foo3);

    assert_eq!(10, foo1.int_val);
    assert_eq!(20, foo2.int_val);
    assert_eq!(30, foo3.int_val);

    assert_eq!(20, *foo1.arc_val.lock());
    assert_eq!(20, *foo2.arc_val.lock());
    assert_eq!(20, *foo3.arc_val.lock());
}
