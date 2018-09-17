/// A function that leaves the given type in the same state as Default,
/// but starts with an existing type instead of allocating a new one.
pub trait Reset {
    fn reset(&mut self);
}

pub struct Recyclable<'a, T: 'a + Default + Reset> {
    val: Option<T>,
    landfill: &'a mut Vec<T>,
}

impl<'a, T: Default + Reset> Drop for Recyclable<'a, T> {
    fn drop(&mut self) {
        if let Some(val) = self.val.take() {
            self.landfill.push(val);
        }
    }
}

/// An object to minimize memory allocations.
pub struct Recycler<T: Default + Reset> {
    landfill: Vec<T>,
}

impl<T: Default + Reset> Default for Recycler<T> {
    fn default() -> Self {
        Recycler { landfill: vec![] }
    }
}

impl<T: Default + Reset> Recycler<T> {
    pub fn allocate(&mut self) -> Recyclable<T> {
        let val = Some(
            self.landfill
                .pop()
                .map(|mut val| {
                    val.reset();
                    val
                }).unwrap_or_default(),
        );
        Recyclable {
            val,
            landfill: &mut self.landfill,
        }
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
        assert_eq!(recycler.allocate().val.as_ref().unwrap().x, 0);
    }

    #[test]
    fn test_recycle() {
        let mut recycler: Recycler<Foo> = Recycler::default();

        {
            let mut foo = recycler.allocate();
            foo.val.as_mut().unwrap().x = 1;
        }
        assert_eq!(recycler.landfill.len(), 1);

        {
            let foo = recycler.allocate();
            assert_eq!(foo.val.as_ref().unwrap().x, 0);
        }
        assert_eq!(recycler.landfill.len(), 1);
    }
}
