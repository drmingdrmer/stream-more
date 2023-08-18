use std::cmp::Ordering;

use compare::Compare;

use crate::stream_more::kmerge::heap_entry::HeapEntryCmp;

#[test]
fn test_peeked_cmp() -> anyhow::Result<()> {
    use Ordering::Equal;
    use Ordering::Greater;
    use Ordering::Less;

    use crate::stream_more::comparators::Ascending;
    use crate::stream_more::comparators::Descending;
    use crate::stream_more::peeked::Peeked::No;
    use crate::stream_more::peeked::Peeked::Yes;

    // no vs yes
    {
        assert_eq!(HeapEntryCmp::new(Descending).compare(&No, &Yes(())), Greater);
        assert_eq!(HeapEntryCmp::new(Ascending).compare(&No, &Yes(())), Greater);
    }

    // no vs no
    {
        assert_eq!(HeapEntryCmp::<(), _>::new(Descending).compare(&No, &No), Equal);
        assert_eq!(HeapEntryCmp::<(), _>::new(Ascending).compare(&No, &No), Equal);
    }

    // yes vs no
    {
        assert_eq!(HeapEntryCmp::new(Descending).compare(&Yes(()), &No), Less);
        assert_eq!(HeapEntryCmp::new(Ascending).compare(&Yes(()), &No), Less);
    }

    // yes vs yes
    {
        assert_eq!(HeapEntryCmp::new(Descending).compare(&Yes(()), &Yes(())), Equal);
        assert_eq!(HeapEntryCmp::new(Descending).compare(&Yes(1), &Yes(1)), Equal);
        assert_eq!(HeapEntryCmp::new(Descending).compare(&Yes(2), &Yes(1)), Greater);
        assert_eq!(HeapEntryCmp::new(Descending).compare(&Yes(2), &Yes(3)), Less);

        assert_eq!(HeapEntryCmp::new(Ascending).compare(&Yes(()), &Yes(())), Equal);
        assert_eq!(HeapEntryCmp::new(Ascending).compare(&Yes(1), &Yes(1)), Equal);
        assert_eq!(HeapEntryCmp::new(Ascending).compare(&Yes(2), &Yes(1)), Less);
        assert_eq!(HeapEntryCmp::new(Ascending).compare(&Yes(2), &Yes(3)), Greater);
    }
    Ok(())
}
