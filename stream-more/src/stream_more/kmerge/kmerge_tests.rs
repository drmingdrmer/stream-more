use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures::executor::block_on;
use futures::stream;
use futures::stream::iter;
use futures::stream::BoxStream;
use futures::Stream;
use futures::StreamExt;

use crate::stream_more::comparators::Descending;
use crate::stream_more::kmerge::KMerge;
use crate::stream_more::StreamMore;

#[test]
fn test_ref_item() -> anyhow::Result<()> {
    struct Foo {
        i: u64,
    }

    fn build_it(foo: &Foo) -> BoxStream<&u64> {
        iter(vec![&foo.i]).boxed()
    }

    let f1 = Foo { i: 3 };
    let f2 = Foo { i: 4 };

    let x = build_it(&f1);
    let y = build_it(&f2);

    let z = x.kmerge_max().merge(y);
    let got = block_on(z.collect::<Vec<&u64>>());
    assert_eq!(vec![&4, &3], got);

    Ok(())
}

#[test]
fn test_kmerge_max() -> anyhow::Result<()> {
    let x = iter([2, 4, 5]);
    let y = iter([1, 3, 6]);

    let z = x.kmerge_max().merge(y);
    let got = block_on(z.collect::<Vec<u64>>());
    assert_eq!(vec![2, 4, 5, 1, 3, 6], got);

    //

    let x = iter([5, 4, 2]);
    let y = iter([6, 3, 1]);

    let z = x.kmerge_max().merge(y);
    let got = block_on(z.collect::<Vec<u64>>());
    assert_eq!(vec![6, 5, 4, 3, 2, 1], got);

    Ok(())
}

#[test]
fn test_kmerge_min() -> anyhow::Result<()> {
    let x = iter([5, 4, 2]);
    let y = iter([6, 3, 1]);

    let z = x.kmerge_min().merge(y);
    let got = block_on(z.collect::<Vec<u64>>());

    assert_eq!(vec![5, 4, 2, 6, 3, 1], got);

    //

    let x = iter([2, 4, 5]);
    let y = iter([1, 3, 6]);

    let z = x.kmerge_min().merge(y);
    let got = block_on(z.collect::<Vec<u64>>());

    assert_eq!(vec![1, 2, 3, 4, 5, 6], got);
    Ok(())
}

#[test]
fn test_kmerge_by() -> anyhow::Result<()> {
    let x = iter([4, 5, 2]);
    let y = iter([6, 3, 1]);

    let z = x.kmerge_by(|a, b| a % 3 < b % 3).merge(y);
    let got = block_on(z.collect::<Vec<u64>>());

    assert_eq!(
        vec![
            //
            6, // 0
            3, // 0
            4, // 1
            1, // 1
            5, // 2
            2, // 2
        ],
        got
    );
    Ok(())
}

#[test]
fn test_kmerge_struct_by() -> anyhow::Result<()> {
    let x = iter([4, 5, 2]);
    let y = iter([6, 3, 1]);

    let z = KMerge::by(|a, b| a < b).merge(x).merge(y);
    let got = block_on(z.collect::<Vec<u64>>());

    assert_eq!(vec![4, 5, 2, 6, 3, 1], got);

    Ok(())
}

#[test]
fn test_kmerge_struct_by_cmp() -> anyhow::Result<()> {
    let x = iter([4, 5, 2]);
    let y = iter([6, 3, 1]);

    let z = KMerge::by_cmp(Descending).merge(x).merge(y);
    let got = block_on(z.collect::<Vec<u64>>());

    assert_eq!(vec![6, 4, 5, 3, 2, 1], got);

    Ok(())
}

#[test]
fn test_3_way_kmerge() -> anyhow::Result<()> {
    let x = iter([2, 5, 7]);
    let y = iter([1, 3, 6]);
    let w = iter([2, 4, 8]);

    let z = x.kmerge_min().merge(y).merge(w);

    let got = block_on(z.collect::<Vec<u64>>());

    assert_eq!(vec![1, 2, 2, 3, 4, 5, 6, 7, 8], got);
    Ok(())
}

#[test]
fn test_pending() -> anyhow::Result<()> {
    use Poll::Pending;
    use Poll::Ready;

    let mut poll_results = vec![Pending, Ready(Some(8)), Pending, Ready(Some(4)), Pending, Ready(None)];
    let x = stream::poll_fn(move |ctx: &mut Context<'_>| {
        // Wake up the runtime after a Pending
        ctx.waker().wake_by_ref();
        poll_results.remove(0)
    });

    let y = iter([6, 3, 1]);

    let z = x.kmerge_max().merge(y);
    let got = block_on(z.collect::<Vec<u64>>());

    assert_eq!(vec![8, 6, 4, 3, 1], got);

    Ok(())
}

#[test]
fn test_no_more_polling_after_none() -> anyhow::Result<()> {
    let mut poll_results = vec![Poll::Ready(Some(4)), Poll::Ready(None), Poll::Ready(Some(1))];

    let x = stream::poll_fn(move |_ctx: &mut Context<'_>| poll_results.remove(0));

    let y = iter([3, 1]);

    let z = x.kmerge_max().merge(y);
    let got = block_on(z.collect::<Vec<u64>>());
    assert_eq!(vec![4, 3, 1], got);

    Ok(())
}

#[test]
fn test_continue_append_more_streams_after_polling() -> anyhow::Result<()> {
    let mut z = iter([1, 3]).kmerge_min().merge(iter([2, 4]));

    let cx = &mut Context::from_waker(futures::task::noop_waker_ref());

    assert_eq!(Poll::Ready(Some(1)), Pin::new(&mut z).poll_next(cx));
    assert_eq!(Poll::Ready(Some(2)), Pin::new(&mut z).poll_next(cx));

    let z = z.merge(iter([1, 5]));

    let got = block_on(z.collect::<Vec<_>>());
    assert_eq!(vec![1, 3, 4, 5], got);
    Ok(())
}
