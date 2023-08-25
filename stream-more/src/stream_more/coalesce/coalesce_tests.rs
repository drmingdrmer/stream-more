use std::task::Context;
use std::task::Poll;

use futures::executor::block_on;
use futures::stream;
use futures::stream::iter;
use futures::StreamExt;

use crate::StreamMore;

#[test]
fn test_coalesce_empty() -> anyhow::Result<()> {
    let data = iter(vec![]);
    let got = data.coalesce(|x: u64, y| Ok(x + y)).collect::<Vec<_>>();

    let got = block_on(got);
    assert_eq!(0, got.len());
    Ok(())
}

#[test]
fn test_coalesce_one_item() -> anyhow::Result<()> {
    let data = iter(vec![1u64]);
    let got = data.coalesce(|x: u64, y| Ok(x + y)).collect::<Vec<_>>();

    let got = block_on(got);
    assert_eq!(vec![1u64], got);
    Ok(())
}

#[test]
fn test_coalesce_basic() -> anyhow::Result<()> {
    let data = iter(vec![-1., -2., -3., 3., 1., 0., -1.]);
    let got = data
        .coalesce(|x, y| {
            //
            if (x >= 0.) == (y >= 0.) {
                Ok(x + y)
            } else {
                Err((x, y))
            }
        })
        .collect::<Vec<_>>();

    let got = block_on(got);
    assert_eq!(vec![-6., 4., -1.], got);
    Ok(())
}

#[test]
fn test_coalesce_pending() -> anyhow::Result<()> {
    use Poll::Pending;
    use Poll::Ready;

    let mut poll_results = vec![Pending, Ready(Some(8)), Pending, Ready(Some(4)), Pending, Ready(None)];
    let x = stream::poll_fn(move |ctx: &mut Context<'_>| {
        // Wake up the runtime after a Pending
        ctx.waker().wake_by_ref();
        poll_results.remove(0)
    });

    let z = x.coalesce(|x: u64, y| Ok(x + y));
    let got = block_on(z.collect::<Vec<u64>>());

    assert_eq!(vec![12], got);

    Ok(())
}

#[test]
fn test_coalesce_no_more_polling_after_none() -> anyhow::Result<()> {
    let mut poll_results = vec![Poll::Ready(Some(4)), Poll::Ready(None), Poll::Ready(Some(1))];

    let x = stream::poll_fn(move |_ctx: &mut Context<'_>| poll_results.remove(0));

    let z = x.coalesce(|x, y| Ok(x + y));
    let got = block_on(z.collect::<Vec<u64>>());
    assert_eq!(vec![4], got);

    Ok(())
}
