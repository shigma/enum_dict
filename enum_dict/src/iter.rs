use core::iter::FusedIterator;

pub struct Values<I>(pub I);

impl<I> From<I> for Values<I> {
    #[inline]
    fn from(iter: I) -> Self {
        Values(iter)
    }
}

impl<I, K, V> Iterator for Values<I>
where
    I: Iterator<Item = (K, V)>,
{
    type Item = V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, value)| value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }
}

impl<I, K, V> ExactSizeIterator for Values<I>
where
    I: ExactSizeIterator<Item = (K, V)>,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<I, K, V> DoubleEndedIterator for Values<I>
where
    I: DoubleEndedIterator<Item = (K, V)>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, value)| value)
    }
}

impl<I, K, V> FusedIterator for Values<I> where I: FusedIterator<Item = (K, V)> {}
