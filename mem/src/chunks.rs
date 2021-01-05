use core::iter::FromIterator;
use core::ops::Range;
use core::{
    cmp::{max, min},
    panic,
};

use multiboot2::MemoryArea;

/// Used to represent regions in memory that may, or may not, be contiguous.
#[derive(Debug, Clone)]
pub enum MemoryChunks<const N: usize> {
    /// Used when chunks are together in memory.
    ///
    /// ```text
    /// ___XXXXXXXXXXX___
    ///    ^ {start}  ^ {end}
    /// ```
    Contiguous { start: usize, end: usize },

    /// Used to group multiple chunks together when they may not always be together in memory.
    ///
    /// ```text
    /// XX__YYYYY___XXXXXXXXX___XXXXX
    /// ^ {body[0]} ^ {body[1]} ^{body[2]}
    /// ```
    Segregated {
        body: [(usize, usize); N],
        length: usize,
    },
}

impl<const N: usize> Default for MemoryChunks<N> {
    fn default() -> Self {
        Self::Contiguous { start: 0, end: 0 }
    }
}

impl<I, const N: usize> From<Range<I>> for MemoryChunks<N>
where
    I: Into<usize>,
{
    fn from(range: Range<I>) -> Self {
        let start = range.start.into();
        let end = range.end.into();
        Self::Contiguous { start, end }
    }
}

impl<T, U, const N: usize> From<(T, U)> for MemoryChunks<N>
where
    T: Into<usize>,
    U: Into<usize>,
{
    fn from((start, end): (T, U)) -> Self {
        let start = start.into();
        let end = end.into();

        Self::Contiguous { start, end }
    }
}

impl<T, U, const N: usize> From<[(T, U); N]> for MemoryChunks<N>
where
    T: Into<usize> + Copy,
    U: Into<usize> + Copy,
{
    fn from(arr: [(T, U); N]) -> Self {
        let mut body = [(0usize, 0usize); N];
        let length = N;

        let mut it = arr
            .iter()
            .cloned()
            .map(|(t, u)| (t.into(), u.into()))
            .enumerate();

        for (idx, (t, u)) in it {
            body[idx] = (t, u);
        }

        Self::Segregated { body, length }
    }
}

impl<const N: usize> FromIterator<(usize, usize)> for MemoryChunks<N> {
    fn from_iter<T: IntoIterator<Item = (usize, usize)>>(iter: T) -> Self {
        let mut body = [(0usize, 0usize); N];
        let mut length = 0;

        for item in iter.into_iter().take(N) {
            body[length] = item;
            length += 1;
        }

        Self::Segregated { body, length }
    }
}

impl<'a, const N: usize> FromIterator<&'a MemoryArea> for MemoryChunks<N> {
    fn from_iter<T: IntoIterator<Item = &'a MemoryArea>>(iter: T) -> Self {
        let mut body = [(0usize, 0usize); N];
        let mut length = 0;

        let mut it = iter.into_iter().take(N);

        while let Some(item) = it.next() {
            body[length] = (item.start_address() as usize, item.end_address() as usize);
            length += 1;
        }

        Self::Segregated { body, length }
    }
}

impl<const N: usize> MemoryChunks<N> {
    /// Get the const capacity of the chunks.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<(usize, usize)> {
        match self {
            Self::Contiguous { start, end } => {
                if idx == 0 {
                    Some((*start, *end))
                } else {
                    None
                }
            },

            Self::Segregated { body, length } => {
                body[..*length].get(idx).cloned()
            }
        }
    }

    /// Swap the contents of two, equally sized, chunks.
    #[inline]
    pub fn swap(&mut self, other: &mut Self) {
        let tmp = self.clone();
        *self = other.clone();
        *other = tmp;
    }

    /// Returns `true` if any of the `range` intersects with the chunks.
    #[inline]
    pub fn contains<I>(&self, range: &Range<I>) -> bool
    where
        I: Into<usize> + Copy,
    {
        let range_start = range.start.into();
        let range_stop = range.end.into();

        match self {
            Self::Contiguous { start, end } => max(range_start, *start) <= min(range_stop, *end),
            Self::Segregated { body, length } => body[..*length]
                .iter()
                .cloned()
                .any(|(start, stop)| max(range_start, start) <= min(range_stop, stop)),
        }
    }

    /// Poke a hole into a chunk, splitting it in two.
    ///
    /// For example, with this layout:
    ///
    /// ```text
    /// | 0x00      | 0x1000     | 0x2000
    /// XX__YYYYY___XXXXXXXXXXXXXX
    /// ^ {body[0]} ^ {body[1]}
    /// ```
    ///
    /// And poking a hole at (`0x1800..0x1900`)
    ///
    /// ```text
    /// | 0x00              | 1800h | 2000h
    /// XX__YYYYY___XXXXXXXXX___XXXXX
    /// ^ {body[0]} ^ {body[1]} ^{body[2]} (1900h)
    /// ```
    pub fn poke<T, U>(&mut self, (t, u): (T, U)) -> Option<MemoryChunks<{ 1 }>>
    where
        T: Into<usize>,
        U: Into<usize>,
    {
        let (hole_start, hole_end) = (t.into(), u.into());

        match self {
            Self::Contiguous { start, end } => {
                // The hole is entirely not witin the range.
                //
                // Its either to the left:
                //
                //  {hole_start}
                //  | | {hole_end}
                // _OOO_XXXXXXXXXXX__
                //      ^ {start} ^ {end}
                //
                // Or its to the right:
                //
                //                  {hole_start}
                //                  | | {hole_end}
                // __XXXXXXXXXXX_OOO
                //   ^ {start} ^ {end}
                //
                // But its not intersecting.
                if (*start > hole_start) && (*start > hole_end)
                    || (*end < hole_start) && (*end < hole_end)
                {
                    None
                }
                // The range is entirely within the hole!
                //
                //   {hole_start}
                //   |    {start}
                //   |    | |{end}
                // __OOOOOXXXOOO__
                //             ^ {hole_end}
                //
                else if *start >= hole_start && *end < hole_end {
                    let mut body = [(0usize, 0usize); N];

                    *self = MemoryChunks::<{ N }>::Segregated { body, length: 0 };

                    Some(MemoryChunks::<{ 1 }>::Contiguous {
                        start: hole_start,
                        end: hole_end,
                    })
                }
                // The hole is entirely within the range.
                //
                //       {hole_start}
                //       | | {hole_end}
                // __XXXXOOOXXX__
                //   ^ {start} ^ {end}
                //
                else if hole_start > *start && hole_end < *end {
                    let mut body = [(0usize, 0usize); N];
                    body[0] = (*start, hole_start);
                    body[1] = (*end, hole_end);

                    *self = MemoryChunks::<{ N }>::Segregated { body, length: 2 };

                    Some(MemoryChunks::<{ 1 }>::Contiguous {
                        start: hole_start,
                        end: hole_end,
                    })
                }
                // The hole is clipping the LEFT side.
                //
                //   {hole_start}
                //   | | {hole_end}
                // _OOOOXXXXXXXX__
                //   ^ {start} ^ {end}
                //
                else if hole_start <= *start && hole_end > *start {
                    let mut body = [(0usize, 0usize); N];
                    body[0] = (hole_end, *end);

                    *self = MemoryChunks::<{ N }>::Segregated { body, length: 1 };

                    Some(MemoryChunks::<{ 1 }>::Contiguous {
                        start: hole_start,
                        end: hole_end,
                    })
                }
                // The hole is clipping the RIGHT side.
                //
                //         {hole_start}
                //         |    | {hole_end}
                // __XXXXXXOOOOOO__
                //   ^ {start} ^ {end}
                //
                else if hole_start > *start && hole_end >= *end {
                    let mut body = [(0usize, 0usize); N];
                    body[0] = (*start, hole_start);

                    *self = MemoryChunks::<{ N }>::Segregated { body, length: 1 };

                    Some(MemoryChunks::<{ 1 }>::Contiguous {
                        start: hole_start,
                        end: hole_end,
                    })
                }
                // unreachable case...
                else {
                    unreachable!()
                }
            }

            Self::Segregated {
                body: areas,
                length,
            } => {
                assert!(
                    N > *length && (N - *length) >= 2,
                    "Not enough remaining space to split! (length is {:?}, N is {:?})",
                    length,
                    N
                );

                let self_length = length;

                for (idx, (start, end)) in areas.iter().cloned().enumerate() {
                    let mut cursor = MemoryChunks::<{ 2 }>::Contiguous { start, end };

                    match cursor.poke((hole_start, hole_end)) {
                        Some(hole) => {
                            let (left, right, length) = match cursor {
                                MemoryChunks::Contiguous { .. } => unreachable!(),
                                MemoryChunks::Segregated { body, length } => {
                                    (body[0], body[1], length)
                                }
                            };

                            if length == 1 {
                                // The chunk was truncated.
                                areas[idx] = left;
                            } else if length == 0 {
                                // The chunk was destroyed.
                                areas[idx] = (0, 0);
                            }else {
                                // The chunk was split.
                                areas[idx] = left;
                                areas[idx + 1] = right;
                                *self_length += 1;
                            }

                            return Some(hole);
                        }

                        None => continue,
                    }
                }

                None
            }
        }
    }
}
