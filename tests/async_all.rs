use tokio::{runtime::Builder, task::JoinSet};

#[test]
fn run_all_parallel() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("tokio-thread")
        .thread_stack_size(4 * 1024 * 1024)
        .enable_time()
        .build()
        .expect("Runtime can not created.");

    runtime.block_on(async move {
        let mut join_set = JoinSet::new();
        for i in 0..10 {
            join_set.spawn(start(format!("AsyncFn {i}")));
        }

        let _result = join_set.join_all().await;

        unsafe_test().await;
    });
}

async fn start(name: String) -> anyhow::Result<u32> {
    let mut rand_no = rand_range(10, 20);

    for _ in 0..10 {
        tokio::time::sleep(tokio::time::Duration::from_millis(rand_range(22, 99) as u64)).await;
        rand_no = rand_range(11, 99);
        println!("Name: {name} , rand_no: {rand_no}");
    }

    Ok(rand_no)
}

fn rand_range(start: u32, end: u32) -> u32 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    let (min, max) = if start < end {
        (start, end)
    } else {
        (end, start)
    };
    let diff = max.abs_diff(min);

    min + (nanos % diff)
}

async fn unsafe_test() {
    let num: u32 = 42;
    let p: *const u32 = &num;

    unsafe {
        assert_eq!(*p, num);
    }
}
