use std::{sync::mpsc, thread};

#[test]
fn thread_mpsc() {
    let v1 = vec![10, 20, 30, 40, 50, 60, 70];
    let v2 = vec![1, 2, 3, 4, 5, 6, 7];
    let (tx, rx) = mpsc::channel::<String>();

    let x = 10;

    assert_eq!(v1.len(), v2.len());

    for i in 0..v1.len() {
        let v1 = v1.clone();
        let v2 = v2.clone();
        let tx = tx.clone();

        let handle = thread::spawn(move || {
            println!("x: {}", &x);
            let s = format!("Thread {}, {}+{}={}", i, v1[i], v2[i], v1[i] + v2[i]);
            tx.send(s).unwrap();
        });
        let _ = handle.join();
    }
    drop(tx);

    for result in rx {
        println!("{}", result);
    }
}
