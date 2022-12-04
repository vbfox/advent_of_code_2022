pub struct CharSliceIterator<'a> {
    s: &'a str,
    index: usize,
}

impl<'a> CharSliceIterator<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, index: 0 }
    }
}

impl<'a> Iterator for CharSliceIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.s.len() {
            let slice = &self.s[self.index..=self.index];
            self.index += 1;
            Some(slice)
        } else {
            None
        }
    }
}

pub trait CharSliceExt {
    fn char_slices(&self) -> CharSliceIterator;
}

impl<T: AsRef<str>> CharSliceExt for T {
    fn char_slices(&self) -> CharSliceIterator {
        CharSliceIterator::new(self.as_ref())
    }
}

// --------------------------------------------------------------------------

fn single<T>(mut iterator: impl Iterator<Item = T>) -> Option<T> {
    match (iterator.next(), iterator.next()) {
        (Some(item), None) => Some(item),
        _ => None,
    }
}

pub trait SingleExt<T> {
    fn single(self) -> Option<T>;
}

impl<Item, T: Iterator<Item = Item>> SingleExt<Item> for T {
    fn single(self) -> Option<Item> {
        single(self)
    }
}

#[cfg(test)]
mod single_tests {
    use super::*;

    #[test]
    fn single_test() {
        assert_eq!(single("a".chars()), Some('a'));
        assert_eq!(single("ab".chars()), None);
        assert_eq!(single("".chars()), None);
    }
}

// --------------------------------------------------------------------------

pub fn find_common_items<'a, T: Eq>(items: &'a Vec<Vec<T>>) -> Vec<&'a T> {
    let mut common_items: Vec<&'a T> = Vec::new();

    for item in &items[0] {
        if common_items.contains(&item) {
            continue;
        }

        if items[1..].iter().all(|i| i.contains(item)) {
            common_items.push(item);
        }
    }

    common_items
}

#[cfg(test)]
mod find_common_items_tests {
    use super::*;

    #[test]
    fn all() {
        let items = vec![
            vec![1, 2, 3, 4, 5],
            vec![1, 2, 3, 4, 5],
            vec![1, 2, 3, 4, 5],
        ];

        let common_items = find_common_items(&items);

        assert_eq!(common_items, vec![&1, &2, &3, &4, &5]);
    }

    #[test]
    fn some() {
        let items = vec![vec![2, 3, 4, 5], vec![1, 2, 3, 4, 5], vec![1, 2, 4, 5]];

        let common_items = find_common_items(&items);

        assert_eq!(common_items, vec![&2, &4, &5]);
    }

    #[test]
    fn none() {
        let items = vec![vec![2, 4, 5], vec![1, 3], vec![5]];

        let common_items = find_common_items(&items);

        assert_eq!(common_items, Vec::<&i32>::default());
    }
}
