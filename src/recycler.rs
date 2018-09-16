/// An object to minimize memory allocations.
pub struct Recycler<T> {
    landfill: Vec<T>,
}

/// A function that leaves the given type in the same state as Default,
/// but starts with an existing type instead of allocating a new one.
pub trait Reset {
    fn reset(&mut self);
}

impl<T> Default for Recycler<T> {
    fn default() -> Self {
        Recycler { landfill: vec![] }
    }
}

impl<T: Default + Reset> Recycler<T> {
    pub fn allocate(&mut self) -> T {
        match self.landfill.pop() {
            Some(mut x) => {
                x.reset();
                x
            }
            None => Default::default(),
        }
    }

    pub fn recycle(&mut self, x: T) {
        self.landfill.push(x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Foo {
        x: u8,
    }

    impl Reset for Foo {
        fn reset(&mut self) {
            self.x = 0;
        }
    }

    #[test]
    fn test_allocate() {
        let mut recycler: Recycler<Foo> = Recycler::default();
        assert_eq!(recycler.allocate().x, 0);
    }

    #[test]
    fn test_recycle() {
        let mut recycler: Recycler<Foo> = Recycler::default();

        {
            let mut foo = recycler.allocate();
            foo.x = 1;
            recycler.recycle(foo);
        }
        assert_eq!(recycler.landfill.len(), 1);

        assert_eq!(recycler.allocate().x, 0);
        assert_eq!(recycler.landfill.len(), 0);
    }
}
