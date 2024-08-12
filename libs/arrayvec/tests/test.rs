use arrayvec::*;

#[test]
fn test_unsize_deref() {
    let mut sized = ArrayVecSized::<u32, 4>::new();
    sized.push(42);
    let un_sized: &ArrayVec<u32> = &sized;
    let slice: &[u32] = un_sized;
    assert_eq!(sized.len(), 1);
    assert_eq!(un_sized.len(), 1);
    assert_eq!(slice, &[42]);
}

#[test]
fn test_try_insert() {
    let mut arr = ArrayVecSized::<i32, 8>::new();
    for i in 100..110 {
        let _ = arr.try_insert(0, i);
    }

    assert_eq!(arr.as_slice(), &[107, 106, 105, 104, 103, 102, 101, 100]);
    assert_eq!(arr.pop(), Some(100));

    arr.try_insert(3, 42).unwrap();

    assert_eq!(arr.as_slice(), &[107, 106, 105, 42, 104, 103, 102, 101]);
}

#[test]
fn test_append() {
    let mut arr1 = ArrayVecSized::<i32, 8>::new();
    for i in 100..104{
        arr1.push(i);
    }

    let mut arr2 = ArrayVecSized::<i32, 9>::new();
    arr2.push(222);
    arr2.append_range(&mut arr1, 1..);

    assert_eq!(arr1.as_slice(), &[100]);
    assert_eq!(arr2.as_slice(), &[222, 101, 102, 103]);
}

#[test]
fn test_drop() {
    struct AssignOnDrop<'a>(&'a mut bool);
    impl<'a> core::ops::Drop for AssignOnDrop<'a> {
        fn drop(&mut self) {
            *self.0 = true;
        }
    }

    let mut boolarr = [false; 6];
    let mut arr = ArrayVecSized::<AssignOnDrop<'_>, 8>::new();

    let [a, b, c, d, ..] = &mut boolarr;
    arr.push(AssignOnDrop(a));
    arr.push(AssignOnDrop(b));
    arr.push(AssignOnDrop(c));

    core::mem::forget(arr.remove(1));
    arr.try_insert(1, AssignOnDrop(d)).unwrap();

    drop(arr);
    assert_eq!(boolarr, [true, false, true, true, false, false]);
}
