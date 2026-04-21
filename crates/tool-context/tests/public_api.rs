use addzero_context::*;
use std::thread;

#[test]
fn set_get_and_remove_work_for_single_type() {
    ThreadLocalUtil::clear();
    ThreadLocalUtil::set::<String>("addzero".to_owned());

    assert_eq!(ThreadLocalUtil::get::<String>(), Some("addzero".to_owned()));
    assert!(ThreadLocalUtil::contains::<String>());
    assert_eq!(
        ThreadLocalUtil::remove::<String>(),
        Some("addzero".to_owned())
    );
    assert_eq!(ThreadLocalUtil::get::<String>(), None);
}

#[test]
fn values_are_separated_by_type() {
    ThreadLocalUtil::clear();
    ThreadLocalUtil::set::<String>("hello".to_owned());
    ThreadLocalUtil::set::<i32>(42);

    assert_eq!(ThreadLocalUtil::get::<String>(), Some("hello".to_owned()));
    assert_eq!(ThreadLocalUtil::get::<i32>(), Some(42));
}

#[test]
fn with_borrows_without_clone_requirement() {
    ThreadLocalUtil::clear();
    ThreadLocalUtil::set::<Vec<i32>>(vec![1, 2, 3]);

    let sum = ThreadLocalUtil::with::<Vec<i32>, _>(|value| {
        value.map(|numbers| numbers.iter().sum::<i32>())
    });

    assert_eq!(sum, Some(6));
}

#[test]
fn thread_local_values_are_isolated_per_thread() {
    ThreadLocalUtil::clear();
    ThreadLocalUtil::set::<String>("main".to_owned());

    let worker = thread::spawn(|| {
        assert_eq!(ThreadLocalUtil::get::<String>(), None);
        ThreadLocalUtil::set::<String>("worker".to_owned());
        ThreadLocalUtil::get::<String>()
    });

    assert_eq!(
        worker.join().expect("thread should join"),
        Some("worker".to_owned())
    );
    assert_eq!(ThreadLocalUtil::get::<String>(), Some("main".to_owned()));
}
